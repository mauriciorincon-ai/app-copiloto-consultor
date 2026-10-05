//! EL ACENTO DEL MODELO — «Enriquecer el banco» (ADR 019 §3), **apagado de fábrica**.
//!
//! Con el interruptor de IA encendido, y una sola vez por ensayo, el modelo propone **hasta cinco
//! preguntas más** a partir de los títulos y la primera frase de las secciones de tu propuesta y de la
//! ficha de tu cliente. Lo determinista alrededor:
//!
//! - **Qué ve el modelo:** títulos y primeras frases, con un id (`S1`… de la propuesta, `C1`… de la
//!   ficha), y el idioma del ensayo ([`Peticion::texto`]). Jamás audio, jamás tu respuesta, jamás nada de
//!   una reunión. El nombre del cliente lo tapa la bóveda del proveedor externo antes de salir.
//! - **Qué puede devolver:** `{ "preguntas": [{ "texto": "…", "seccion": "S2" }] }`, nada más.
//! - **La regla dura, como en la síntesis:** una pregunta del modelo solo existe si [`fundar`] la
//!   acepta: nombra una sección que se le dio (por su id o su título exacto), comparte con ella al menos
//!   dos términos, termina en «?» y cabe en 25 palabras. Lo demás se tira **y se cuenta**.
//! - **El fallback:** el banco por reglas, que ya estaba armado y no espera a nadie. Si esto falla, tarda
//!   más del techo o no funda nada, el ensayo empieza igual y la pantalla dice por qué en una línea.
//!
//! Va por el mismo adaptador que la sugerencia (ADR 010/011): `mock` → el API si lo encendiste → el
//! modelo del sistema. Sin proveedor nuevo.

use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;

use super::banco::{self, De, Idioma, Pregunta, Regla};
use crate::corpus::seccion::Seccion;
use crate::propuestas::plegar;
use crate::sintesis::{Proveedor, Respuesta};

/// Cuántas preguntas suma el modelo, como mucho.
pub const MAXIMO: usize = 5;
/// Cuántas palabras puede tener una pregunta del modelo.
pub const PALABRAS_MAXIMAS: usize = 25;
/// Cuántos términos tiene que compartir con su sección para fundarse.
pub const TERMINOS_MINIMOS: usize = 2;
/// Cuántas palabras de la primera frase de cada sección ve el modelo.
const PRIMERA_LINEA: usize = 20;

/// Una sección tal como la ve el modelo.
#[derive(Clone, Debug)]
struct Vista {
    id: String,
    titulo: String,
    linea: String,
}

/// **Lo que se le pide al modelo.** Solo existe con al menos una sección con título.
#[derive(Clone, Debug)]
pub struct Peticion {
    secciones: Vec<Vista>,
    idioma: Idioma,
}

fn primera_frase(texto: &str, tope: usize) -> String {
    let frase = texto.split(['.', '?', '!', '\n']).map(str::trim).find(|f| !f.is_empty()).unwrap_or("");
    frase.split_whitespace().take(tope).collect::<Vec<_>>().join(" ")
}

impl Peticion {
    pub fn nueva(propuesta: &[Seccion], ficha: &[Seccion], idioma: Idioma) -> Option<Self> {
        let vista = |prefijo: &str, ss: &[Seccion]| -> Vec<Vista> {
            ss.iter()
                .filter_map(|s| s.titulo.as_ref().map(|t| (t, s)))
                .enumerate()
                .map(|(i, (t, s))| Vista { id: format!("{prefijo}{}", i + 1), titulo: t.clone(), linea: primera_frase(&s.texto, PRIMERA_LINEA) })
                .collect()
        };
        let mut secciones = vista("S", propuesta);
        secciones.extend(vista("C", ficha));
        (!secciones.is_empty()).then_some(Self { secciones, idioma })
    }

    /// El trabajo, el formato cerrado y la prohibición de inventar. En inglés porque es lo que mejor
    /// siguen los modelos; las preguntas, en el idioma de la primera línea del texto.
    pub fn instrucciones() -> &'static str {
        "You help a consultant rehearse before a client meeting. You get the titles and first lines of \
         the sections of the consultant's own proposal (S1, S2…) and of their notes about the client \
         (C1, C2…). Propose up to 5 questions this client is likely to ask, each about ONE of the given \
         sections and using its words; do not add facts, names or numbers that are not there. Write the \
         questions in the language named on the first line (es = Spanish, en = English). Reply with a \
         single JSON object and nothing else: \
         {\"preguntas\": [{\"texto\": \"<the question, up to 25 words, ending with ?>\", \
         \"seccion\": \"<the id of the section it is about: S1, C2…>\"}]}"
    }

    /// El idioma y las secciones, con sus ids. **Es todo lo que el modelo ve.**
    pub fn texto(&self) -> String {
        let mut t = format!("Language: {}\n", if self.idioma == Idioma::En { "en" } else { "es" });
        for s in &self.secciones {
            t.push_str(&format!("{} · {} — {}\n", s.id, s.titulo, s.linea));
        }
        t
    }

    fn seccion(&self, nombrada: &str) -> Option<&Vista> {
        let n = nombrada.trim();
        self.secciones
            .iter()
            .find(|s| s.id.eq_ignore_ascii_case(n))
            .or_else(|| self.secciones.iter().find(|s| plegar(&s.titulo) == plegar(n)))
    }
}

#[derive(Deserialize)]
struct Crudo {
    preguntas: Vec<Cruda>,
}

#[derive(Deserialize)]
struct Cruda {
    texto: String,
    seccion: String,
}

/// Lo que sobrevivió, y cuántas se tiraron.
#[derive(Debug, Clone, PartialEq)]
pub struct Enriquecido {
    pub preguntas: Vec<Pregunta>,
    pub descartadas: usize,
}

/// Por qué el banco no se enriqueció. Cerrado: la pantalla lo dice en una línea, en los dos idiomas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PorQueNo {
    /// El interruptor de IA está apagado (el de fábrica).
    Apagado,
    /// Nadie puede redactar ahora: sin API encendido y sin modelo del sistema.
    SinProveedor,
    /// No era el JSON del esquema.
    FueraDelEsquema,
    /// Contestó, y ninguna pregunta se pudo fundar en tus secciones.
    NadaFundado,
    /// Tu propuesta y la ficha no tienen secciones con título: no hay en qué fundar nada, y no se pregunta
    /// (auditoría del S4, B22; antes se decía «nada fundado» sin haber preguntado).
    SinSecciones,
    /// Las que propuso ya estaban en el banco.
    Repetidas,
    /// Pasó el techo, o llegó con el ensayo ya terminado.
    Tarde,
    /// El proveedor no contestó.
    Fallo,
}

/// Una pregunta del modelo, **si se funda**: sección dada, dos términos en común, «?» y 25 palabras.
fn fundar_una(c: &Cruda, peticion: &Peticion) -> Option<Pregunta> {
    let texto = c.texto.trim();
    let palabras = texto.split_whitespace().count();
    if !texto.ends_with('?') || palabras == 0 || palabras > PALABRAS_MAXIMAS {
        return None;
    }
    let s = peticion.seccion(&c.seccion)?;
    let comunes = banco::terminos(texto).intersection(&banco::terminos(&format!("{} {}", s.titulo, s.linea))).count();
    (comunes >= TERMINOS_MINIMOS).then(|| Pregunta {
        texto: texto.to_string(),
        regla: Regla::Modelo,
        de: De::Modelo,
        seccion: Some(s.titulo.clone()),
        clave: s.id.clone(),
        fuente: None,
    })
}

/// **El único camino de una pregunta del modelo al banco.** Valida el esquema y funda cada pregunta;
/// se queda con [`MAXIMO`] y cuenta las que tira (repetidas incluidas).
pub fn fundar(json: &str, peticion: &Peticion) -> Result<Enriquecido, PorQueNo> {
    let crudo: Crudo = crate::sintesis::objeto(json)
        .and_then(|o| serde_json::from_str(o).ok())
        .ok_or(PorQueNo::FueraDelEsquema)?;
    let mut preguntas: Vec<Pregunta> = Vec::new();
    let mut descartadas = 0;
    for c in &crudo.preguntas {
        match fundar_una(c, peticion) {
            Some(p) if preguntas.len() < MAXIMO && !preguntas.iter().any(|q| plegar(&q.texto) == plegar(&p.texto)) => preguntas.push(p),
            _ => descartadas += 1,
        }
    }
    if preguntas.is_empty() {
        return Err(PorQueNo::NadaFundado);
    }
    Ok(Enriquecido { preguntas, descartadas })
}

/// Lo que devolvió la llamada, con lo que costó.
pub struct Resultado {
    pub enriquecido: Result<Enriquecido, PorQueNo>,
    pub respuesta: Respuesta,
    pub ms: u64,
    /// El detalle de un fallo del proveedor, para el log (nunca a la pantalla).
    pub fallo: Option<String>,
    /// Pasado el techo, la respuesta puede llegar y **se cobra igual**: quien llama la espera aparte para
    /// sumarla al tope del mes, como en la sugerencia (auditoría del S2, M10).
    pub tarde: Option<std::sync::mpsc::Receiver<Result<Respuesta, String>>>,
}

/// **UNA LLAMADA, CON SU TECHO.** El proveedor corre en un hilo aparte; pasado `techo`, el banco se
/// queda como estaba.
pub fn enriquecer(proveedor: Arc<dyn Proveedor>, peticion: &Peticion, techo: Duration) -> Resultado {
    let reloj = Instant::now();
    let (tx, rx) = std::sync::mpsc::channel();
    let (instrucciones, texto) = (Peticion::instrucciones(), peticion.texto());
    let p = proveedor.clone();
    std::thread::spawn(move || {
        let _ = tx.send(p.redactar(instrucciones, &texto));
    });
    let llegada = rx.recv_timeout(techo);
    let ms = reloj.elapsed().as_millis() as u64;
    match llegada {
        Err(_) => Resultado { enriquecido: Err(PorQueNo::Tarde), respuesta: Respuesta::default(), ms, fallo: None, tarde: Some(rx) },
        Ok(Err(e)) => Resultado { enriquecido: Err(PorQueNo::Fallo), respuesta: Respuesta::default(), ms, fallo: Some(e), tarde: None },
        Ok(Ok(respuesta)) => Resultado { enriquecido: fundar(&respuesta.json, peticion), respuesta, ms, fallo: None, tarde: None },
    }
}

#[cfg(test)]
mod pruebas {
    use super::super::banco::pruebas::{ficha, propuesta};
    use super::*;
    use crate::sintesis::{mock::Mock, PorQueNoRedacta, Quien};

    fn peticion() -> Peticion {
        Peticion::nueva(&propuesta(), &ficha(), Idioma::Es).unwrap()
    }

    #[test]
    fn el_modelo_ve_titulos_y_primeras_frases_y_nada_mas() {
        let t = peticion().texto();
        assert!(t.starts_with("Language: es\n"));
        assert!(t.contains("S2 · Alcance — Cubre perfilado y limpieza de tres fuentes: el ERP, el POS de tiendas y el Excel de canal directo"));
        assert!(t.contains("C2 · Quién decide — La gerente general firma"));
        // La segunda frase de una sección no sale: «Una cuarta fuente se cotiza aparte».
        assert!(!t.contains("cuarta"), "{t}");
        assert_eq!(t.lines().count(), 1 + propuesta().len() + ficha().len());
    }

    /// **El grounding**: sección dada (por id o título), dos términos en común, «?» y 25 palabras. Lo que
    /// no, se tira y se cuenta.
    #[test]
    fn solo_se_funda_lo_que_nombra_una_seccion_dada_y_habla_de_ella() {
        let json = r#"{"preguntas":[
            {"texto":"¿La limpieza de las tres fuentes incluye el Excel de canal?","seccion":"S2"},
            {"texto":"¿Quién firma por parte de la gerente general?","seccion":"Quién decide"},
            {"texto":"¿Tienen garantía de resultados?","seccion":"S99"},
            {"texto":"¿Cuánto cuesta el tablero?","seccion":"S2"},
            {"texto":"La limpieza de las tres fuentes incluye el Excel","seccion":"S2"},
            {"texto":"¿La limpieza de las tres fuentes incluye el Excel de canal?","seccion":"S2"}
        ]}"#;
        let e = fundar(json, &peticion()).unwrap();
        assert_eq!(e.preguntas.len(), 2, "{e:#?}");
        assert_eq!(e.descartadas, 4, "sección que no existe · sin términos en común · sin «?» · repetida");
        assert_eq!(e.preguntas[1].seccion.as_deref(), Some("Quién decide"));
        assert!(e.preguntas.iter().all(|p| p.regla == Regla::Modelo && p.de == De::Modelo));
    }

    #[test]
    fn mas_de_cinco_se_cuentan_como_descartadas_y_fuera_del_esquema_no_hay_nada() {
        let una = r#"{"texto":"¿La limpieza de tres fuentes cubre el ERP NUM?","seccion":"S2"}"#;
        let json = format!(r#"{{"preguntas":[{}]}}"#, (1..=7).map(|n| una.replace("NUM", &n.to_string())).collect::<Vec<_>>().join(","));
        let e = fundar(&json, &peticion()).unwrap();
        assert_eq!((e.preguntas.len(), e.descartadas), (MAXIMO, 2));
        assert_eq!(fundar("no es json", &peticion()), Err(PorQueNo::FueraDelEsquema));
        assert_eq!(fundar(r#"{"preguntas":[]}"#, &peticion()), Err(PorQueNo::NadaFundado));
    }

    /// El `mock` recorre el camino entero: propone dos preguntas fundadas y una que no se funda.
    #[test]
    fn el_mock_recorre_el_contrato_entero() {
        let r = enriquecer(Arc::new(Mock), &peticion(), crate::sintesis::TECHO);
        let e = r.enriquecido.expect("el mock no propuso nada fundado");
        assert_eq!((e.preguntas.len(), e.descartadas), (2, 1));
        assert_eq!(r.respuesta.bytes_fuera, 0, "el mock no sale del Mac");
        assert!(e.preguntas.iter().all(|p| p.texto.starts_with('¿')), "en el idioma del ensayo");
    }

    struct Lento(Duration);
    impl Proveedor for Lento {
        fn quien(&self) -> Quien {
            Quien::Mock
        }
        fn nombre(&self) -> String {
            "lento".into()
        }
        fn disponible(&self) -> Result<(), PorQueNoRedacta> {
            Ok(())
        }
        fn redactar(&self, _i: &str, _t: &str) -> Result<Respuesta, String> {
            std::thread::sleep(self.0);
            Ok(Respuesta::default())
        }
    }

    /// **El techo**: el banco no espera más que eso, y lo que llegue tarde se puede cobrar aparte.
    #[test]
    fn pasado_el_techo_el_banco_se_queda_como_estaba() {
        let r = enriquecer(Arc::new(Lento(Duration::from_millis(400))), &peticion(), Duration::from_millis(100));
        assert_eq!(r.enriquecido, Err(PorQueNo::Tarde));
        assert!(r.ms < 300, "esperó {} ms con un techo de 100", r.ms);
        assert!(r.tarde.is_some());
    }

    #[test]
    fn sin_secciones_con_titulo_no_hay_peticion() {
        assert!(Peticion::nueva(&[Seccion { titulo: None, texto: "algo".into() }], &[], Idioma::Es).is_none());
    }
}
