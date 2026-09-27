//! LA SÍNTESIS (C7) — una frase de sugerencia, redactada a partir de las fichas que la app YA
//! encontró. ADR 010 «síntesis, código primero» y ADR 011 «proveedores del modelo y minimización».
//!
//! **MÓDULO PROTEGIDO.** Lo que entra aquí es el último turno del CLIENTE —texto de un tercero— y
//! las fichas del corpus del consultor. Ni disco, ni log con contenido, ni red desde Rust: la única
//! salida es la del proveedor externo, que va por el puente de Swift, declarada línea a línea y
//! contada por `red::registrar_salida` (ADR 011).
//!
//! ## Lo que es determinista alrededor del modelo
//!
//! - **Qué ve el modelo:** el turno y las tres fichas del top, cada una con un id (`F1`, `F2`,
//!   `F3`). Nada más (ver [`Peticion::texto`]).
//! - **Qué puede devolver:** un esquema cerrado `{titular, linea, fuente, confianza}`.
//! - **La regla dura que no compila:** una [`Sugerencia`] solo se construye con [`fundar`], y
//!   `fundar` exige que `fuente` sea **exactamente** el id de una de las fichas que se le dieron. Los
//!   campos son privados: no hay otra manera de fabricar el tipo que viaja a la banda.
//! - **Y la línea dice lo que dice esa ficha:** [`fiel::dice_lo_que_la_ficha`]. Una cita válida
//!   con una afirmación que la ficha no hace se descarta igual.
//! - **El techo:** [`TECHO`]. Pasado, no hay sugerencia; la ficha ya estaba en pantalla.
//! - **El fallback:** ninguno que hacer. La ficha y la maniobra son el camino de siempre y no
//!   esperan a la síntesis: si esto falla, simplemente no aparece nada debajo.

pub mod anonimo;
pub mod api;
pub mod fiel;
pub mod mock;
pub mod sistema;

use crate::ficha::{Fuente, Respaldo};
use std::time::{Duration, Instant};

/// El techo de una sugerencia. La orden fija ≤4 s de mediana y ≤6 s de p95; lo que tarde más no se
/// enseña: llegaría cuando el cliente ya está en otra cosa.
pub const TECHO: Duration = Duration::from_secs(6);

/// Cuántas fichas ve el modelo: las del top que la banda ya enseña (la mejor y sus dos acumuladas).
pub const FICHAS: usize = 3;

/// Los topes del esquema. Una «línea» de mil caracteres no es una sugerencia: es el modelo
/// escribiendo de más, y la banda la cortaría igual.
const TITULAR_MAXIMO: usize = 90;
const LINEA_MAXIMA: usize = 260;

/// **Lo que se le pide al modelo.** Solo existe con al menos una ficha: sin evidencia no hay nada
/// que sintetizar, y pedirle al modelo que responda sin fichas es pedirle que invente.
#[derive(Debug, Clone)]
pub struct Peticion {
    turno: String,
    fichas: Vec<Respaldo>,
}

impl Peticion {
    pub fn nueva(turno: &str, respaldo: &[Respaldo]) -> Option<Self> {
        if respaldo.is_empty() || turno.trim().is_empty() {
            return None;
        }
        Some(Self {
            turno: turno.trim().to_string(),
            fichas: respaldo.iter().take(FICHAS).cloned().collect(),
        })
    }

    pub fn fichas(&self) -> &[Respaldo] {
        &self.fichas
    }

    /// Lo que va delante de la pregunta: el trabajo, el formato cerrado y la prohibición de
    /// inventar. En inglés porque es lo que mejor siguen los modelos; la respuesta va **en el idioma
    /// de la ficha citada**, con sus palabras —una línea traducida no se puede cotejar (`fiel.rs`)—.
    pub fn instrucciones() -> &'static str {
        "You help a consultant during a live meeting. You get the client's last question and up to \
         three evidence cards from the consultant's own documents, each with an id (F1, F2, F3). \
         Write ONE short suggestion of what the consultant could say next, using ONLY what ONE card \
         says, in that card's own words and language — never add facts, numbers, names or negations \
         that are not in it. Reply with a single JSON object and nothing else: \
         {\"titular\": \"<up to 8 words>\", \"linea\": \"<one sentence, up to 30 words>\", \
         \"fuente\": \"<the id of the card you used: F1, F2 or F3>\", \
         \"confianza\": \"<alta | media | baja>\"}. \
         Use \"baja\" when no card really answers the question."
    }

    /// La pregunta y las fichas, con sus ids. **Es todo lo que el modelo ve.**
    pub fn texto(&self) -> String {
        let mut t = format!("Client: {}\n", self.turno);
        for (i, f) in self.fichas.iter().enumerate() {
            t.push_str(&format!("F{} · {} — {}\n", i + 1, f.titular, f.linea));
        }
        t
    }

    /// Pisa el turno del cliente antes de soltarlo, como el resto de la casa.
    pub fn pisar(&mut self) {
        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
        unsafe { self.turno.as_mut_vec() }.fill(0);
    }
}

impl Drop for Peticion {
    fn drop(&mut self) {
        self.pisar();
    }
}

/// Lo que devuelve un proveedor, **sin validar**. Nunca llega a la banda así.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct Crudo {
    pub titular: String,
    pub linea: String,
    pub fuente: String,
    pub confianza: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Confianza {
    Alta,
    Media,
    Baja,
}

impl Confianza {
    fn de(texto: &str) -> Option<Self> {
        match texto.trim().to_lowercase().as_str() {
            "alta" | "high" => Some(Self::Alta),
            "media" | "medium" => Some(Self::Media),
            "baja" | "low" => Some(Self::Baja),
            _ => None,
        }
    }
}

/// Quién redactó. La banda lo dice («en tu Mac» o el nombre del proveedor externo).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Quien {
    Sistema,
    Api,
    Mock,
}

/// La ficha que respalda la sugerencia: su titular y su fuente, para que se vea a la vez.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FichaCitada {
    pub titular: String,
    pub fuente: Fuente,
}

/// **La sugerencia.** Sus campos son privados y su único constructor es [`fundar`]: una sugerencia
/// que no cita una de las fichas que se le dieron **no puede existir**.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sugerencia {
    titular: String,
    linea: String,
    confianza: Confianza,
    ficha: FichaCitada,
    quien: Quien,
    /// El nombre que se enseña: «Modelo del sistema», «Claude Haiku»…
    nombre: String,
    /// Se queda en Rust: la latencia la enseña IA como mediana de la reunión, no sugerencia a
    /// sugerencia, y un campo que cruza sin lector es un huérfano (casilla 5 de la auditoría).
    #[serde(skip)]
    ms: u64,
}

impl Sugerencia {
    pub fn quien(&self) -> Quien {
        self.quien
    }
    pub fn ms(&self) -> u64 {
        self.ms
    }
    pub fn confianza(&self) -> Confianza {
        self.confianza
    }
}

/// Por qué una respuesta del modelo no llegó a ser sugerencia. Va al log como metadata —el
/// motivo, nunca el texto— y a la pantalla IA.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Descarte {
    /// No era el JSON del esquema, o le faltaba un campo, o se pasaba de largo.
    FueraDelEsquema,
    /// Citaba una fuente que no se le dio. **El caso que la regla existe para impedir.**
    SinFuente,
    /// Citaba bien, pero la línea dice algo que esa ficha no dice ([`fiel`]).
    NoLoDiceLaFicha,
    /// Pasó el techo.
    Tarde,
    /// El proveedor no contestó. El detalle, para el log.
    Fallo(String),
}

/// **EL ÚNICO CONSTRUCTOR DE [`Sugerencia`].** Valida el esquema y el grounding.
pub fn fundar(
    crudo: &Crudo,
    peticion: &Peticion,
    quien: Quien,
    nombre: &str,
    ms: u64,
) -> Result<Sugerencia, Descarte> {
    let titular = crudo.titular.trim();
    let linea = crudo.linea.trim();
    if titular.is_empty()
        || linea.is_empty()
        || titular.chars().count() > TITULAR_MAXIMO
        || linea.chars().count() > LINEA_MAXIMA
    {
        return Err(Descarte::FueraDelEsquema);
    }
    let confianza = Confianza::de(&crudo.confianza).ok_or(Descarte::FueraDelEsquema)?;
    // La fuente, EXACTA: «F2» y nada más. Ni «la ficha 2», ni «F2, F3», ni un título que se parezca.
    let id = crudo.fuente.trim().to_uppercase();
    let citada = peticion
        .fichas
        .iter()
        .enumerate()
        .find(|(i, _)| id == format!("F{}", i + 1))
        .map(|(_, f)| f)
        .ok_or(Descarte::SinFuente)?;
    if !fiel::dice_lo_que_la_ficha(linea, citada) {
        return Err(Descarte::NoLoDiceLaFicha);
    }
    let titular = if fiel::titular_de_la_ficha_o_el_turno(titular, citada, &peticion.turno) {
        titular
    } else {
        citada.titular.as_str()
    };
    Ok(Sugerencia {
        titular: titular.to_string(),
        linea: linea.to_string(),
        confianza,
        ficha: FichaCitada { titular: citada.titular.clone(), fuente: citada.fuente.clone() },
        quien,
        nombre: nombre.to_string(),
        ms,
    })
}

/// Lo que un proveedor contesta: el JSON (sin validar) y lo que costó sacarlo.
#[derive(Debug, Clone, Default)]
pub struct Respuesta {
    pub json: String,
    /// Bytes que salieron del Mac para obtenerla. Cero en los proveedores locales.
    pub bytes_fuera: u64,
    pub tokens_entrada: u64,
    pub tokens_salida: u64,
    /// Cuál de las peticiones de la reunión fue, si salió al API (B37): para ponerle su costo al
    /// volver. `None` en los proveedores locales.
    pub salida: Option<u64>,
}

/// Por qué un proveedor no puede redactar ahora mismo. Cerrado: IA lo dice en dos idiomas.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PorQueNoRedacta {
    /// Apple Intelligence está apagado en Ajustes.
    AppleIntelligenceApagado,
    /// Este Mac no puede usar el modelo del sistema.
    MacNoCompatible,
    /// macOS no lo deja usar ahora por otra razón que no dice (auditoría del S2, B2): decir «este
    /// Mac no es compatible» ahí era falso.
    NoDisponible,
    /// macOS todavía está descargando el modelo.
    ModeloDescargandose,
    /// Esta copia de la app se construyó sin el puente del modelo.
    SinPuente,
    /// El API no tiene clave.
    SinClave,
    /// Se llegó al tope del mes.
    TopeDelMes,
}

/// **Un proveedor**, local o externo. El mismo contrato para todos (ADR 011): `mock` incluido.
pub trait Proveedor: Send + Sync {
    fn quien(&self) -> Quien;
    fn nombre(&self) -> String;
    fn disponible(&self) -> Result<(), PorQueNoRedacta>;
    /// Redacta. Bloquea: [`sugerir`] lo llama en un hilo aparte y no espera más de su techo.
    fn redactar(&self, instrucciones: &str, texto: &str) -> Result<Respuesta, String>;
}

/// El JSON del modelo, aunque venga envuelto en «```json … ```» o con una frase delante: se toma
/// el primer objeto `{…}` completo. Lo demás se tira.
fn objeto(texto: &str) -> Option<&str> {
    let ini = texto.find('{')?;
    let fin = texto.rfind('}')?;
    (fin > ini).then(|| &texto[ini..=fin])
}

/// Lo que devolvió una llamada, con su sugerencia o su motivo, y lo que costó.
pub struct Resultado {
    pub sugerencia: Result<Sugerencia, Descarte>,
    pub respuesta: Respuesta,
    pub ms: u64,
    /// Pasado el techo, la respuesta todavía puede llegar —la red espera un segundo más— y **el
    /// proveedor la cobra igual**. No se enseña, pero quien llama puede esperarla para sumar su
    /// costo al tope del mes (auditoría del S2, M10). `None` si no hubo techo.
    pub tarde: Option<std::sync::mpsc::Receiver<Result<Respuesta, String>>>,
}

/// **UNA SUGERENCIA, CON SU TECHO.** El proveedor corre en un hilo aparte; si no contesta dentro de
/// `techo`, el resultado es [`Descarte::Tarde`] y lo que llegue después se tira sin mirarlo.
pub fn sugerir(
    proveedor: std::sync::Arc<dyn Proveedor>,
    peticion: &Peticion,
    techo: Duration,
) -> Resultado {
    let reloj = Instant::now();
    let (tx, rx) = std::sync::mpsc::channel();
    let (instrucciones, mut texto) = (Peticion::instrucciones(), peticion.texto());
    let (p, t) = (proveedor.clone(), texto.clone());
    std::thread::spawn(move || {
        let mut t = t;
        let _ = tx.send(p.redactar(instrucciones, &t));
        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
        unsafe { t.as_mut_vec() }.fill(0);
    });
    // SEGURIDAD: ídem.
    unsafe { texto.as_mut_vec() }.fill(0);
    let llegada = rx.recv_timeout(techo);
    let ms = reloj.elapsed().as_millis() as u64;
    match llegada {
        Err(_) => Resultado { sugerencia: Err(Descarte::Tarde), respuesta: Respuesta::default(), ms, tarde: Some(rx) },
        Ok(Err(e)) => Resultado { sugerencia: Err(Descarte::Fallo(e)), respuesta: Respuesta::default(), ms, tarde: None },
        Ok(Ok(respuesta)) => {
            let sugerencia = objeto(&respuesta.json)
                .and_then(|o| serde_json::from_str::<Crudo>(o).ok())
                .ok_or(Descarte::FueraDelEsquema)
                .and_then(|c| fundar(&c, peticion, proveedor.quien(), &proveedor.nombre(), ms));
            Resultado { sugerencia, respuesta, ms, tarde: None }
        }
    }
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;
    use crate::corpus::Unidad;
    use std::sync::Arc;

    pub fn respaldo() -> Vec<Respaldo> {
        let f = |t: &str, l: &str, doc: &str, u| Respaldo {
            titular: t.into(),
            linea: l.into(),
            fuente: Fuente { documento: doc.into(), seccion: None, unidad: Some(u), conjeturada: false },
        };
        vec![
            f("Limpieza de datos: incluida, hasta tres fuentes", "Una cuarta fuente es adicional y se cotiza aparte.", "Páramo Azul · §3.2 Alcance", Unidad::Propuesta),
            f("Etapa 2 Preparación", "Perfilar antes de modelar.", "Marco · Etapa 2", Unidad::Marco),
            f("Sur del Valle", "Cuatro fuentes en 9 semanas.", "Sur del Valle · Resultados", Unidad::Caso),
        ]
    }

    pub fn peticion() -> Peticion {
        Peticion::nueva("¿Y si sumamos el Excel de la fuerza comercial?", &respaldo()).unwrap()
    }

    fn crudo(fuente: &str) -> Crudo {
        Crudo {
            titular: "Tres fuentes dentro, la cuarta aparte".into(),
            linea: "Confirma que incluye hasta tres fuentes; una cuarta es adicional y se cotiza aparte.".into(),
            fuente: fuente.into(),
            confianza: "media".into(),
        }
    }

    #[test]
    fn una_sugerencia_que_cita_una_ficha_dada_se_funda_con_esa_ficha() {
        let mut c = crudo(" f3 ");
        c.linea = "Sur del Valle: cuatro fuentes en 9 semanas.".into();
        let s = fundar(&c, &peticion(), Quien::Mock, "mock", 10).unwrap();
        assert_eq!(s.ficha.titular, "Sur del Valle");
        assert_eq!(s.ficha.fuente.documento, "Sur del Valle · Resultados");
    }

    /// **El grounding.** Una fuente que no se le dio —otra ficha, un documento que el modelo se
    /// imaginó, dos a la vez— y la sugerencia no existe.
    #[test]
    fn una_sugerencia_sin_fuente_dada_se_descarta() {
        for mala in ["F4", "F0", "", "Páramo Azul · §3.2 Alcance", "F1, F2", "la ficha 1"] {
            assert_eq!(
                fundar(&crudo(mala), &peticion(), Quien::Mock, "mock", 10),
                Err(Descarte::SinFuente),
                "«{mala}» contó como fuente"
            );
        }
    }

    /// **La segunda mitad del grounding**, cableada: la cita es válida y la línea no es de esa
    /// ficha (la de F1 dicha como si fuera de F2).
    #[test]
    fn una_linea_que_la_ficha_citada_no_dice_se_descarta() {
        assert!(fundar(&crudo("F1"), &peticion(), Quien::Mock, "mock", 10).is_ok());
        assert_eq!(fundar(&crudo("F2"), &peticion(), Quien::Mock, "mock", 10), Err(Descarte::NoLoDiceLaFicha));
    }

    /// Un titular que no sale ni de la ficha ni de la pregunta no tumba la sugerencia: se cambia por
    /// el de la ficha, que sí es del consultor.
    #[test]
    fn un_titular_inventado_se_cambia_por_el_de_la_ficha() {
        let mut c = crudo("F1");
        c.titular = "Fourth Source".into();
        let s = fundar(&c, &peticion(), Quien::Mock, "mock", 10).unwrap();
        assert_eq!(s.titular, "Limpieza de datos: incluida, hasta tres fuentes");
        let mut c = crudo("F1");
        c.titular = "Tres fuentes, la cuarta aparte".into();
        assert_eq!(fundar(&c, &peticion(), Quien::Mock, "mock", 10).unwrap().titular, "Tres fuentes, la cuarta aparte");
    }

    #[test]
    fn fuera_del_esquema_no_hay_sugerencia() {
        let mut c = crudo("F1");
        c.confianza = "altísima".into();
        assert_eq!(fundar(&c, &peticion(), Quien::Mock, "m", 1), Err(Descarte::FueraDelEsquema));
        let mut c = crudo("F1");
        c.linea = "x".repeat(LINEA_MAXIMA + 1);
        assert_eq!(fundar(&c, &peticion(), Quien::Mock, "m", 1), Err(Descarte::FueraDelEsquema));
        let mut c = crudo("F1");
        c.titular = "  ".into();
        assert_eq!(fundar(&c, &peticion(), Quien::Mock, "m", 1), Err(Descarte::FueraDelEsquema));
    }

    #[test]
    fn sin_fichas_o_sin_turno_no_hay_peticion() {
        assert!(Peticion::nueva("¿algo?", &[]).is_none());
        assert!(Peticion::nueva("  ", &respaldo()).is_none());
        assert_eq!(Peticion::nueva("¿algo?", &[respaldo(), respaldo()].concat()).unwrap().fichas().len(), FICHAS);
    }

    /// Un proveedor de mentira que tarda lo que se le diga y contesta lo que se le diga.
    struct Lento(Duration, String);
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
            Ok(Respuesta { json: self.1.clone(), ..Default::default() })
        }
    }

    /// **El techo.** Lo que llega tarde no se enseña, y quien espera no espera más del techo.
    #[test]
    fn pasado_el_techo_no_hay_sugerencia_y_no_se_espera_de_mas() {
        let json = r#"{"titular":"a","linea":"b","fuente":"F1","confianza":"alta"}"#.to_string();
        let r = sugerir(Arc::new(Lento(Duration::from_millis(400), json)), &peticion(), Duration::from_millis(100));
        assert_eq!(r.sugerencia, Err(Descarte::Tarde));
        assert!(r.ms < 300, "esperó {} ms con un techo de 100", r.ms);
        // Y lo que llega tarde se puede esperar aparte, para cobrarlo (M10).
        let tarde = r.tarde.expect("pasado el techo, la respuesta tardía se pierde y no se cobra");
        let llego = tarde.recv_timeout(Duration::from_secs(2)).expect("no llegó").expect("falló");
        assert!(llego.json.contains("\"fuente\":\"F1\""));
    }

    #[test]
    fn el_json_envuelto_en_prosa_se_lee() {
        let json = "Claro:\n```json\n{\"titular\":\"a\",\"linea\":\"b\",\"fuente\":\"F2\",\"confianza\":\"baja\"}\n```".to_string();
        let r = sugerir(Arc::new(Lento(Duration::ZERO, json)), &peticion(), Duration::from_secs(1));
        assert_eq!(r.sugerencia.unwrap().ficha.titular, "Etapa 2 Preparación");
    }

    /// El modelo solo ve el turno y las tres fichas, con sus ids.
    #[test]
    fn el_modelo_ve_el_turno_y_las_fichas_y_nada_mas() {
        let t = peticion().texto();
        assert_eq!(t.lines().count(), 1 + FICHAS);
        assert!(t.starts_with("Client: ¿Y si sumamos"));
        assert!(t.contains("F1 · Limpieza de datos") && t.contains("F3 · Sur del Valle"));
    }
}
