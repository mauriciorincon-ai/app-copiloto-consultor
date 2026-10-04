//! EL BANCO DE PREGUNTAS — por reglas publicadas, sin modelo (ADR 019 §2).
//!
//! **Puro**: entran las secciones de tu propuesta y de la ficha de tu cliente, ya troceadas, y el
//! idioma; salen preguntas, cada una con la regla que la produjo, de dónde sale y lo que tomó del
//! documento. Las reglas y sus plantillas viven en `data/ensayo/reglas.json` y las objeciones en
//! `data/ensayo/objeciones.json`, dentro del binario (como el catálogo de las propuestas).
//!
//! Las seis reglas, en el orden en que reparten:
//!
//! 1. **seccion** — una pregunta por cada sección de la propuesta que un cliente revisa (alcance,
//!    supuestos, entregables, precio, plazo…), reconocida por su título;
//! 2. **cifra** — «¿cómo llegaron a «cuatro semanas»?», por cada cifra que escribiste (menos las del
//!    contexto y los supuestos, que son del cliente);
//! 3. **compromiso** — «si no se cumple «…», ¿qué pasa?», por cada frase que promete algo;
//! 4. **riesgo** — «¿quién asume el riesgo de «…»?», por cada «si…» y cada supuesto;
//! 5. **cliente** — lo que la ficha de tu cliente dice que le importa (quién decide, acuerdos previos…);
//! 6. **objecion** — las del catálogo cuyas etiquetas tocan tu propuesta, con su fuente.
//!
//! Con un tope corto, el reparto va **por turnos** entre las reglas: un ensayo de cinco no se queda con
//! cinco preguntas de sección.

use std::collections::HashSet;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::corpus::seccion::Seccion;
use crate::propuestas::plegar;
use crate::radar::Bilingue;

const REGLAS: &str = include_str!("../../../data/ensayo/reglas.json");
const OBJECIONES: &str = include_str!("../../../data/ensayo/objeciones.json");

/// La regla que produjo una pregunta. `Modelo` es la del acento opt-in ([`super::enriquecer`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Regla {
    Seccion,
    Cifra,
    Compromiso,
    Riesgo,
    Cliente,
    Objecion,
    Modelo,
}

impl Regla {
    /// Las del banco, en el orden en que reparten.
    pub const DEL_BANCO: [Regla; 6] =
        [Regla::Seccion, Regla::Cifra, Regla::Compromiso, Regla::Riesgo, Regla::Cliente, Regla::Objecion];

    pub fn id(self) -> &'static str {
        match self {
            Regla::Seccion => "seccion",
            Regla::Cifra => "cifra",
            Regla::Compromiso => "compromiso",
            Regla::Riesgo => "riesgo",
            Regla::Cliente => "cliente",
            Regla::Objecion => "objecion",
            Regla::Modelo => "modelo",
        }
    }
}

/// De dónde sale una pregunta: lo que la pantalla agrupa en «De dónde salen las preguntas».
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum De {
    Propuesta,
    Ficha,
    Objeciones,
    /// Sugerida por el modelo: la pantalla la marca así, nunca mezclada sin marca (ADR 019 §3).
    Modelo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Idioma {
    Es,
    En,
}

impl Idioma {
    fn de(self, b: &Bilingue) -> String {
        match self {
            Idioma::Es => b.es.clone(),
            Idioma::En => b.en.clone(),
        }
    }
}

/// Una pregunta del banco. Se lee de vuelta de un ensayo guardado ([`super::guardado`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pregunta {
    pub texto: String,
    pub regla: Regla,
    pub de: De,
    /// El título de la sección de donde sale. `None` en las objeciones.
    pub seccion: Option<String>,
    /// Lo que la regla tomó del documento: la clase de la sección, la cifra, la frase o el id de la
    /// objeción. Es lo que el kit compara.
    pub clave: String,
    /// La fuente publicada de una objeción («criterio del builder» si no la hay).
    pub fuente: Option<String>,
}

// ─── el catálogo ─────────────────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FilaDeRegla {
    id: String,
    de: String,
    nombre: Bilingue,
}

#[derive(Deserialize)]
struct Clase {
    clase: String,
    titulos: Vec<String>,
    pregunta: Bilingue,
}

#[derive(Deserialize)]
struct ConPalabras {
    palabras: Vec<String>,
    pregunta: Bilingue,
}

#[derive(Deserialize)]
struct PorRegla {
    cifra: usize,
    compromiso: usize,
    riesgo: usize,
    objecion: usize,
}

#[derive(Deserialize)]
struct Vacias {
    es: Vec<String>,
    en: Vec<String>,
}

#[derive(Deserialize)]
struct ArchivoDeReglas {
    version: u32,
    fecha: String,
    topes: Vec<usize>,
    reglas: Vec<FilaDeRegla>,
    secciones: Vec<Clase>,
    sin_cifras: Vec<String>,
    sin_compromisos: Vec<String>,
    cifra: Bilingue,
    compromiso: ConPalabras,
    riesgo: ConPalabras,
    cliente: Vec<Clase>,
    por_regla: PorRegla,
    frase_maxima: usize,
    vacias: Vacias,
}

#[derive(Deserialize)]
struct Objecion {
    id: String,
    etiquetas: Vec<String>,
    pregunta: Bilingue,
    fuente: String,
}

#[derive(Deserialize)]
struct ArchivoDeObjeciones {
    version: u32,
    fecha: String,
    etiquetas: std::collections::BTreeMap<String, Vec<String>>,
    objeciones: Vec<Objecion>,
}

pub struct Catalogo {
    pub version: u32,
    pub fecha: String,
    /// Los topes que la pantalla deja elegir (5 · 8 · 12).
    pub topes: Vec<usize>,
    reglas: Vec<FilaDeRegla>,
    secciones: Vec<Clase>,
    sin_cifras: Vec<String>,
    sin_compromisos: Vec<String>,
    cifra: Bilingue,
    compromiso: ConPalabras,
    riesgo: ConPalabras,
    cliente: Vec<Clase>,
    por_regla: PorRegla,
    frase_maxima: usize,
    vacias_es: HashSet<String>,
    vacias_en: HashSet<String>,
    objeciones: ArchivoDeObjeciones,
}

impl Catalogo {
    fn de_textos(reglas: &str, objeciones: &str) -> Result<Catalogo, String> {
        let r: ArchivoDeReglas = serde_json::from_str(reglas).map_err(|e| format!("las reglas del ensayo no se leen: {e}"))?;
        let o: ArchivoDeObjeciones =
            serde_json::from_str(objeciones).map_err(|e| format!("las objeciones del ensayo no se leen: {e}"))?;
        Ok(Catalogo {
            version: r.version,
            fecha: r.fecha,
            topes: r.topes,
            reglas: r.reglas,
            secciones: r.secciones,
            sin_cifras: r.sin_cifras,
            sin_compromisos: r.sin_compromisos,
            cifra: r.cifra,
            compromiso: r.compromiso,
            riesgo: r.riesgo,
            cliente: r.cliente,
            por_regla: r.por_regla,
            frase_maxima: r.frase_maxima,
            vacias_es: r.vacias.es.iter().map(|v| plegar(v)).collect(),
            vacias_en: r.vacias.en.iter().map(|v| plegar(v)).collect(),
            objeciones: o,
        })
    }

    /// El nombre de una regla en los dos idiomas, para «De dónde salen las preguntas».
    pub fn nombre(&self, regla: Regla) -> Option<&Bilingue> {
        self.reglas.iter().find(|r| r.id == regla.id()).map(|r| &r.nombre)
    }

    /// De dónde dice el catálogo que sale cada regla («propuesta», «ficha», «objeciones»): lo que la
    /// pantalla agrupa en «De dónde salen las preguntas». Un test comprueba que coincide con el código.
    pub fn de(&self, regla: Regla) -> Option<&str> {
        self.reglas.iter().find(|r| r.id == regla.id()).map(|r| r.de.as_str())
    }

    /// La versión y la fecha de las objeciones, que la pantalla enseña junto a «catálogo publicado».
    pub fn objeciones_version(&self) -> (u32, &str) {
        (self.objeciones.version, &self.objeciones.fecha)
    }

    fn vacia(&self, plegada: &str) -> bool {
        self.vacias_es.contains(plegada) || self.vacias_en.contains(plegada)
    }
}

pub fn catalogo() -> &'static Catalogo {
    static UNO: OnceLock<Catalogo> = OnceLock::new();
    UNO.get_or_init(|| Catalogo::de_textos(REGLAS, OBJECIONES).expect("data/ensayo/"))
}

// ─── palabras ────────────────────────────────────────────────────────────────────────────────────

/// Las palabras de un texto, plegadas (sin mayúsculas ni tildes) y sin signos. La evaluación las usa
/// para contar muletillas y palabras por minuto.
pub(crate) fn palabras(texto: &str) -> Vec<String> {
    plegar(texto)
        .split(|ch: char| !(ch.is_alphanumeric() || ch == '%' || ch == '\''))
        .filter(|p| !p.is_empty())
        .map(str::to_string)
        .collect()
}

/// ¿Aparece esta palabra o frase del catálogo, por palabras enteras? Una palabra del catálogo de cinco
/// letras o más vale también como prefijo de la del texto: «presupuesto» casa con «presupuestos» y
/// «fuente» con «fuentes», pero «entregables» no casa con «entrega».
fn aparece(palabras_del_texto: &[String], del_catalogo: &str) -> bool {
    let buscada = palabras(del_catalogo);
    match buscada.as_slice() {
        [] => false,
        [una] => palabras_del_texto.iter().any(|p| p == una || (una.chars().count() >= 5 && p.starts_with(una.as_str()))),
        varias => palabras_del_texto.windows(varias.len()).any(|w| w == varias),
    }
}

/// **Los términos de un texto** para compararlo con otro: las palabras de cuatro letras o más que no
/// son vacías, recortadas a sus cinco primeras letras («fuentes» y «fuente» son la misma). Lo usan el
/// acento del modelo para fundar sus preguntas y, desde la fase 3, la evaluación de la respuesta.
pub fn terminos(texto: &str) -> HashSet<String> {
    let c = catalogo();
    palabras(texto)
        .into_iter()
        .filter(|p| p.chars().count() >= 4 && !c.vacia(p))
        .map(|p| p.chars().take(5).collect())
        .collect()
}

/// El idioma de unos documentos, por sus palabras vacías. `None` si empatan: quien llama usa el del
/// usuario (ADR 019 §6.7).
pub fn idioma_de(secciones: &[Seccion]) -> Option<Idioma> {
    let c = catalogo();
    let (mut es, mut en) = (0usize, 0usize);
    for s in secciones {
        for p in palabras(&s.texto) {
            // Las que están en las dos listas («a») no votan.
            match (c.vacias_es.contains(&p), c.vacias_en.contains(&p)) {
                (true, false) => es += 1,
                (false, true) => en += 1,
                _ => {}
            }
        }
    }
    match es.cmp(&en) {
        std::cmp::Ordering::Greater => Some(Idioma::Es),
        std::cmp::Ordering::Less => Some(Idioma::En),
        std::cmp::Ordering::Equal => None,
    }
}

/// Las frases de una sección, sin el punto final.
fn frases(texto: &str) -> Vec<String> {
    let mut salida = Vec::new();
    let mut actual = String::new();
    let mut previo = ' ';
    for ch in texto.chars() {
        let corta = matches!(ch, '.' | '?' | '!' | ';' | '\n') && !previo.is_ascii_digit();
        if corta {
            let f = actual.trim();
            if !f.is_empty() {
                salida.push(f.to_string());
            }
            actual.clear();
        } else {
            actual.push(ch);
        }
        previo = ch;
    }
    let f = actual.trim();
    if !f.is_empty() {
        salida.push(f.to_string());
    }
    salida
}

/// Una frase recortada a `tope` palabras, con «…» si se cortó. Si dentro del tope hay una coma o dos
/// puntos, corta ahí: «Cubre perfilado y limpieza de tres fuentes…» se entiende; «… el ERP, el…», no.
fn recortar(frase: &str, tope: usize) -> String {
    let ps: Vec<&str> = frase.split_whitespace().collect();
    if ps.len() <= tope {
        return ps.join(" ").trim_end_matches([',', ':']).to_string();
    }
    let hasta = (3..tope).find(|&i| ps[i - 1].ends_with([',', ':'])).unwrap_or(tope);
    format!("{}…", ps[..hasta].join(" ").trim_end_matches([',', ':']))
}

/// La clase de un título, si cae en alguna.
fn clase_de<'a>(titulo: &str, clases: &'a [Clase]) -> Option<&'a Clase> {
    let ps = palabras(titulo);
    clases.iter().find(|c| c.titulos.iter().any(|t| aparece(&ps, t)))
}

// ─── las reglas ──────────────────────────────────────────────────────────────────────────────────

/// **El banco ENTERO**, sin tope: todas las preguntas que dan las reglas, regla por regla y en el orden
/// de tus documentos. Es lo que mide el kit.
pub fn todas(propuesta: &[Seccion], ficha: &[Seccion], idioma: Idioma) -> Vec<Vec<Pregunta>> {
    let c = catalogo();
    let clase_de_seccion = |s: &Seccion| s.titulo.as_deref().and_then(|t| clase_de(t, &c.secciones));
    let fuera = |s: &Seccion, lista: &[String]| clase_de_seccion(s).is_some_and(|k| lista.contains(&k.clase));

    // 1 · por sección
    let mut por_seccion: Vec<Pregunta> = Vec::new();
    for s in propuesta {
        let (Some(titulo), Some(k)) = (s.titulo.as_deref(), clase_de_seccion(s)) else { continue };
        if por_seccion.iter().any(|p| p.clave == k.clase) {
            continue;
        }
        por_seccion.push(Pregunta {
            texto: idioma.de(&k.pregunta),
            regla: Regla::Seccion,
            de: De::Propuesta,
            seccion: Some(titulo.to_string()),
            clave: k.clase.clone(),
            fuente: None,
        });
    }

    // 2 · por cifra
    let mut por_cifra: Vec<Pregunta> = Vec::new();
    for s in propuesta.iter().filter(|s| !fuera(s, &c.sin_cifras)) {
        for cifra in crate::propuestas::cifras(&s.texto, idioma == Idioma::En) {
            if por_cifra.iter().any(|p| plegar(&p.clave) == plegar(&cifra)) {
                continue;
            }
            por_cifra.push(Pregunta {
                texto: idioma.de(&c.cifra).replace("{cifra}", &cifra),
                regla: Regla::Cifra,
                de: De::Propuesta,
                seccion: s.titulo.clone(),
                clave: cifra,
                fuente: None,
            });
        }
    }

    // 3 · por compromiso: la primera frase de cada sección que promete algo
    let mut por_compromiso: Vec<Pregunta> = Vec::new();
    for s in propuesta.iter().filter(|s| !fuera(s, &c.sin_compromisos)) {
        let promete = frases(&s.texto)
            .into_iter()
            .find(|f| c.compromiso.palabras.iter().any(|m| aparece(&palabras(f), m)));
        if let Some(f) = promete {
            let frase = recortar(&f, c.frase_maxima);
            por_compromiso.push(Pregunta {
                texto: idioma.de(&c.compromiso.pregunta).replace("{frase}", &frase),
                regla: Regla::Compromiso,
                de: De::Propuesta,
                seccion: s.titulo.clone(),
                clave: frase,
                fuente: None,
            });
        }
    }

    // 4 · por riesgo: primero los «si…», después el resto de los supuestos
    let mut por_riesgo: Vec<Pregunta> = Vec::new();
    let condicional = |f: &str| c.riesgo.palabras.iter().any(|m| aparece(&palabras(f), m));
    let es_supuesto = |s: &Seccion| clase_de_seccion(s).is_some_and(|k| k.clase == "supuestos");
    let mut candidatas: Vec<(&Seccion, String)> = Vec::new();
    for s in propuesta {
        candidatas.extend(frases(&s.texto).into_iter().filter(|f| condicional(f)).map(|f| (s, f)));
    }
    for s in propuesta.iter().filter(|s| es_supuesto(s)) {
        candidatas.extend(frases(&s.texto).into_iter().filter(|f| !condicional(f)).map(|f| (s, f)));
    }
    for (s, f) in candidatas {
        let frase = recortar(&f, c.frase_maxima);
        if por_riesgo.iter().any(|p| p.clave == frase) {
            continue;
        }
        por_riesgo.push(Pregunta {
            texto: idioma.de(&c.riesgo.pregunta).replace("{frase}", &frase),
            regla: Regla::Riesgo,
            de: De::Propuesta,
            seccion: s.titulo.clone(),
            clave: frase,
            fuente: None,
        });
    }

    // 5 · lo que su ficha dice que le importa
    let mut por_cliente: Vec<Pregunta> = Vec::new();
    for s in ficha {
        let (Some(titulo), Some(k)) = (s.titulo.as_deref(), s.titulo.as_deref().and_then(|t| clase_de(t, &c.cliente))) else {
            continue;
        };
        if por_cliente.iter().any(|p| p.clave == k.clase) {
            continue;
        }
        por_cliente.push(Pregunta {
            texto: idioma.de(&k.pregunta),
            regla: Regla::Cliente,
            de: De::Ficha,
            seccion: Some(titulo.to_string()),
            clave: k.clase.clone(),
            fuente: None,
        });
    }

    // 6 · las objeciones que tocan tu propuesta: las específicas primero, las generales después
    let todo: Vec<String> = propuesta.iter().flat_map(|s| palabras(&format!("{} {}", s.titulo.as_deref().unwrap_or(""), s.texto))).collect();
    let tocadas: HashSet<&str> = c
        .objeciones
        .etiquetas
        .iter()
        .filter(|(_, ps)| ps.iter().any(|p| aparece(&todo, p)))
        .map(|(e, _)| e.as_str())
        .collect();
    let especificas = c.objeciones.objeciones.iter().filter(|o| o.etiquetas.iter().any(|e| tocadas.contains(e.as_str())));
    let generales = c.objeciones.objeciones.iter().filter(|o| o.etiquetas.iter().all(|e| e == "general"));
    let por_objecion: Vec<Pregunta> = especificas
        .chain(generales)
        .map(|o| Pregunta {
            texto: idioma.de(&o.pregunta),
            regla: Regla::Objecion,
            de: De::Objeciones,
            seccion: None,
            clave: o.id.clone(),
            fuente: Some(o.fuente.clone()),
        })
        .collect();

    vec![
        por_seccion,
        recortadas(por_cifra, c.por_regla.cifra),
        recortadas(por_compromiso, c.por_regla.compromiso),
        recortadas(por_riesgo, c.por_regla.riesgo),
        por_cliente,
        recortadas(por_objecion, c.por_regla.objecion),
    ]
}

fn recortadas(mut v: Vec<Pregunta>, tope: usize) -> Vec<Pregunta> {
    v.truncate(tope);
    v
}

/// **El banco del ensayo**: `tope` preguntas, repartidas **por turnos** entre las reglas —una de cada
/// una, y otra vuelta— para que un tope corto no se quede con una sola. Sin repetir el mismo texto.
pub fn armar(propuesta: &[Seccion], ficha: &[Seccion], idioma: Idioma, tope: usize) -> Vec<Pregunta> {
    let mut colas: Vec<std::collections::VecDeque<Pregunta>> =
        todas(propuesta, ficha, idioma).into_iter().map(Into::into).collect();
    let mut banco: Vec<Pregunta> = Vec::new();
    while banco.len() < tope && colas.iter().any(|c| !c.is_empty()) {
        for cola in colas.iter_mut() {
            if banco.len() >= tope {
                break;
            }
            while let Some(p) = cola.pop_front() {
                if !banco.iter().any(|b| plegar(&b.texto) == plegar(&p.texto)) {
                    banco.push(p);
                    break;
                }
            }
        }
    }
    banco
}

/// Las preguntas del modelo, **detrás** de las del banco y sin repetir ninguna (ADR 019 §3: suma, no
/// sustituye).
pub fn con_las_del_modelo(mut banco: Vec<Pregunta>, del_modelo: Vec<Pregunta>) -> Vec<Pregunta> {
    for p in del_modelo {
        if !banco.iter().any(|b| plegar(&b.texto) == plegar(&p.texto)) {
            banco.push(p);
        }
    }
    banco
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;

    pub fn sec(titulo: &str, texto: &str) -> Seccion {
        Seccion { titulo: Some(titulo.into()), texto: texto.into() }
    }

    /// La propuesta del kit, tal cual (`docs/kit-de-prueba/corpus/`).
    pub fn propuesta() -> Vec<Seccion> {
        vec![
            sec("Contexto", "Páramo Azul distribuye alimento para ganado en tres canales: mayorista, tiendas de vereda y venta directa. No sabe cuál deja margen."),
            sec("Alcance", "Cubre perfilado y limpieza de tres fuentes: el ERP, el POS de tiendas y el Excel de canal directo. Una cuarta fuente se cotiza aparte."),
            sec("Supuestos", "El cliente entrega los extractos del ERP en la primera semana. Si llegan tarde, el cronograma se corre día por día."),
            sec("Entregables", "Un tablero de margen por canal, el cuaderno de limpieza reproducible y un taller de cierre de tres horas."),
            sec("Precio", "Tarifa cerrada de cuatro semanas. Incluye el taller de cierre y dos rondas de revisión. Una fuente adicional se cotiza aparte."),
            sec("Plazo de entrega", "La entrega completa toma cuatro semanas contadas desde la firma del contrato."),
        ]
    }

    pub fn ficha() -> Vec<Seccion> {
        vec![
            sec("Quiénes son", "Distribuidora familiar de alimento para ganado, ciento veinte empleados, tres departamentos."),
            sec("Quién decide", "La gerente general firma. El jefe de sistemas tiene veto técnico sobre los accesos."),
            sec("Acuerdos previos", "Se firmó acuerdo de confidencialidad en marzo. No hay penalidad por terminación anticipada."),
            sec("Historial", "Un diagnóstico corto el año pasado, sin continuidad por cambio de gerencia."),
        ]
    }

    #[test]
    fn el_catalogo_se_lee_y_nombra_cada_regla_en_los_dos_idiomas() {
        let c = catalogo();
        assert_eq!((c.version, c.topes.as_slice()), (1, &[5, 8, 12][..]));
        for r in Regla::DEL_BANCO {
            let n = c.nombre(r).unwrap_or_else(|| panic!("la regla {} no tiene fila en el catálogo", r.id()));
            assert!(!n.es.is_empty() && !n.en.is_empty());
        }
        // Lo que el catálogo publica como origen de cada regla es lo que el código hace.
        let t = todas(&propuesta(), &ficha(), Idioma::Es);
        for (i, r) in Regla::DEL_BANCO.iter().enumerate() {
            let publicado = c.de(*r).unwrap();
            for p in &t[i] {
                let hecho = match p.de {
                    De::Propuesta => "propuesta",
                    De::Ficha => "ficha",
                    De::Objeciones => "objeciones",
                    De::Modelo => "modelo",
                };
                assert_eq!(hecho, publicado, "la regla {} publica «{publicado}» y produce «{hecho}»", r.id());
            }
        }
        // Cada objeción con su fuente y en los dos idiomas: ninguna cifra inventada sin dueño.
        for o in &c.objeciones.objeciones {
            assert!(!o.fuente.trim().is_empty(), "la objeción {} no tiene fuente", o.id);
            assert!(o.pregunta.es.ends_with('?') && o.pregunta.en.ends_with('?'), "{} no es una pregunta", o.id);
            for e in &o.etiquetas {
                assert!(e == "general" || c.objeciones.etiquetas.contains_key(e), "{}: etiqueta «{e}» sin palabras", o.id);
            }
        }
    }

    #[test]
    fn las_seis_reglas_leen_lo_que_tu_escribiste() {
        let t = todas(&propuesta(), &ficha(), Idioma::Es);
        let claves = |i: usize| t[i].iter().map(|p| p.clave.clone()).collect::<Vec<_>>();
        assert_eq!(claves(0), ["contexto", "alcance", "supuestos", "entregables", "precio", "plazo"]);
        assert_eq!(claves(1), ["tres fuentes", "tres horas", "cuatro semanas", "dos rondas"], "las del contexto son del cliente");
        assert_eq!(
            t[2].iter().map(|p| p.seccion.as_deref().unwrap()).collect::<Vec<_>>(),
            ["Alcance", "Precio", "Plazo de entrega"],
            "lo que dicen los supuestos lo hace el cliente: no es tu promesa"
        );
        assert_eq!(t[3][0].clave, "Si llegan tarde, el cronograma se corre día por día", "el «si…» va primero");
        assert_eq!(claves(4), ["decide", "acuerdos", "historial"]);
        assert_eq!(claves(5), ["numeros-correctos", "no-cuadra-con-el-erp", "nadie-lo-usa"]);
        assert_eq!(t[1][2].texto, "¿Cómo llegaron a «cuatro semanas»?");
        assert!(t[5][0].fuente.as_deref().unwrap().contains("Kuznetsova"));
    }

    /// Un tope corto reparte por turnos: con cinco, cinco reglas distintas; con doce, las seis.
    #[test]
    fn un_tope_corto_reparte_entre_las_reglas() {
        let cinco = armar(&propuesta(), &ficha(), Idioma::Es, 5);
        let reglas: HashSet<Regla> = cinco.iter().map(|p| p.regla).collect();
        assert_eq!((cinco.len(), reglas.len()), (5, 5));
        let doce = armar(&propuesta(), &ficha(), Idioma::Es, 12);
        assert_eq!(doce.len(), 12);
        assert_eq!(doce.iter().map(|p| p.regla).collect::<HashSet<_>>().len(), 6);
        // Ninguna repetida.
        let textos: HashSet<String> = doce.iter().map(|p| plegar(&p.texto)).collect();
        assert_eq!(textos.len(), doce.len());
    }

    #[test]
    fn en_ingles_el_banco_es_ingles() {
        let p = vec![
            sec("Scope", "Covers cleansing of two sources: the ERP and the web store."),
            sec("Pricing", "Fixed fee for three weeks, including two review rounds."),
            sec("Assumptions", "If the extracts arrive late, the schedule slips day for day."),
        ];
        let f = vec![sec("Who decides", "The CFO signs.")];
        assert_eq!(idioma_de(&p), Some(Idioma::En));
        let b = armar(&p, &f, Idioma::En, 12);
        assert!(b.iter().any(|q| q.texto == "How did you arrive at “two review rounds”?"), "{b:#?}");
        assert!(b.iter().any(|q| q.texto == "What does the person who signs need to see to say yes?"));
        assert!(b.iter().all(|q| !q.texto.contains('¿')), "se coló una en español");
    }

    #[test]
    fn el_idioma_es_el_de_la_propuesta_y_si_empata_no_se_adivina() {
        assert_eq!(idioma_de(&propuesta()), Some(Idioma::Es));
        assert_eq!(idioma_de(&[sec("x", "ERP POS")]), None);
    }

    /// Sin nada que reconocer —una propuesta sin títulos ni cifras— el banco no se inventa preguntas de
    /// sección: quedan las objeciones generales.
    #[test]
    fn sin_titulos_ni_cifras_quedan_las_generales() {
        let p = vec![Seccion { titulo: None, texto: "Hablamos del proyecto.".into() }];
        let b = armar(&p, &[], Idioma::Es, 8);
        assert!(b.iter().all(|q| q.regla == Regla::Objecion), "{b:#?}");
        assert!(!b.is_empty());
    }

    #[test]
    fn las_del_modelo_van_detras_y_sin_repetir() {
        let banco = armar(&propuesta(), &ficha(), Idioma::Es, 5);
        let repetida = Pregunta { regla: Regla::Modelo, de: De::Modelo, ..banco[0].clone() };
        let nueva = Pregunta { texto: "¿Y el POS de tiendas cuándo llega?".into(), ..repetida.clone() };
        let junto = con_las_del_modelo(banco.clone(), vec![repetida, nueva.clone()]);
        assert_eq!(junto.len(), banco.len() + 1);
        assert_eq!(junto.last(), Some(&nueva));
    }

    #[test]
    fn una_frase_larga_se_corta_donde_se_entiende() {
        assert_eq!(
            recortar("Cubre perfilado y limpieza de tres fuentes: el ERP, el POS de tiendas y el Excel", 10),
            "Cubre perfilado y limpieza de tres fuentes…"
        );
        assert_eq!(
            recortar("La entrega completa toma cuatro semanas contadas desde la firma del contrato", 10),
            "La entrega completa toma cuatro semanas contadas desde la firma…"
        );
        assert_eq!(recortar("Incluye el taller de cierre y dos rondas de revisión", 10), "Incluye el taller de cierre y dos rondas de revisión");
    }

    #[test]
    fn los_terminos_comparan_sin_tildes_ni_plurales() {
        let t = terminos("Las fuentes del ERP y la limpieza");
        assert!(t.contains("fuent") && t.contains("limpi"));
        assert!(!t.contains("las") && !t.contains("del"));
        assert_eq!(terminos("Fuente"), terminos("fuentes"));
    }
}
