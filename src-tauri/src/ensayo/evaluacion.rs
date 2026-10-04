//! LA EVALUACIÓN DE UNA RESPUESTA — cuatro cifras medidas, sin nota ni puntaje (ADR 019 §6.6).
//!
//! **Pura**: entran la pregunta, los turnos de tu respuesta ya transcritos (con su sitio en el reloj del
//! ensayo), las fichas que el disparo habría enseñado para esa pregunta y el idioma; sale la
//! [`Evaluacion`]. Nada de aquí busca en el corpus ni toca el micrófono: lo que necesita del mundo se lo
//! dan hecho. Tarda microsegundos; el presupuesto de 500 ms del plan es para el camino entero (buscar +
//! armar + evaluar) y lo mide el kit.
//!
//! - **Evidencia**: una ficha cuenta como **citada** si tu respuesta comparte con ella al menos
//!   [`TERMINOS_PARA_CITAR`] términos **que no estaban ya en la pregunta** (prefijo de cinco letras, sin
//!   tildes ni vacías: [`banco::terminos`]). La condición de la pregunta no estaba en el ADR y la
//!   impuso el kit: las fichas salen de buscar las palabras de la pregunta, así que repetir la pregunta
//!   en la respuesta «citaba» las tres siempre. Las demás son «tenías y no usaste», nunca «te faltó»;
//!   y si lo dijiste con otras palabras, «Sí lo dije» manda. La jerga técnica ya viene corregida: la
//!   transcripción pasa por tu diccionario (B3) antes de llegar aquí.
//! - **Tiempo**: desde que terminó la pregunta hasta que cerraste la respuesta.
//! - **Ritmo**: palabras por minuto entre tu primera palabra y el fin de tu último turno.
//! - **Muletillas**: las de `data/ensayo/muletillas.json`, por frases enteras, sobre lo que la
//!   transcripción conserva.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::banco::{self, Idioma};
use crate::ficha::{Fuente, Respaldo};
use crate::propuestas::plegar;

const MULETILLAS: &str = include_str!("../../../data/ensayo/muletillas.json");

/// Cuántos términos tiene que compartir tu respuesta con una ficha para contar como citada.
pub const TERMINOS_PARA_CITAR: usize = 2;
/// Con menos palabras que esto no hay ritmo que medir: «sí, claro» no tiene velocidad.
pub const PALABRAS_PARA_EL_RITMO: usize = 5;
/// Ni con menos tiempo que esto: tres palabras en medio segundo darían 360 ppm.
pub const MS_PARA_EL_RITMO: u64 = 2_000;

/// Un turno de tu respuesta, en el reloj del ensayo. `texto: None` si se oyó y no se pudo transcribir.
#[derive(Clone, Debug, PartialEq)]
pub struct Tramo {
    pub desde_ms: u64,
    pub hasta_ms: u64,
    pub texto: Option<String>,
}

/// Una ficha que tenías para esa pregunta, y si la usaste.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evidencia {
    pub titular: String,
    pub fuente: Fuente,
    pub citada: bool,
    /// La marcaste tú con «Sí lo dije»: cuenta como citada y la pantalla no lo esconde.
    pub dicha_por_ti: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Muletilla {
    /// Como la escribe el catálogo: «o sea», «I mean».
    pub frase: String,
    pub veces: u32,
}

/// Las cuatro cifras de una respuesta.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Evaluacion {
    pub evidencia: Vec<Evidencia>,
    pub tiempo_ms: u64,
    /// `None` con muy poco que medir ([`PALABRAS_PARA_EL_RITMO`], [`MS_PARA_EL_RITMO`]).
    pub ppm: Option<u32>,
    /// De la más repetida a la menos; solo las que aparecieron.
    pub muletillas: Vec<Muletilla>,
    /// Para el ritmo y el kit; la pantalla no la enseña.
    #[serde(skip)]
    pub palabras: u32,
}

impl Evaluacion {
    pub fn citadas(&self) -> usize {
        self.evidencia.iter().filter(|e| e.citada || e.dicha_por_ti).count()
    }

    pub fn total_de_muletillas(&self) -> u32 {
        self.muletillas.iter().map(|m| m.veces).sum()
    }

    /// «Sí lo dije»: la ficha `i` cuenta como citada. Otra pulsación lo deshace. `false` si no existe.
    pub fn si_lo_dije(&mut self, i: usize) -> bool {
        match self.evidencia.get_mut(i) {
            Some(e) if !e.citada => {
                e.dicha_por_ti = !e.dicha_por_ti;
                true
            }
            _ => false,
        }
    }
}

/// Lo que hace falta para evaluar una respuesta.
pub struct Entrada<'a> {
    pub pregunta: &'a str,
    pub tramos: &'a [Tramo],
    /// Las fichas que el disparo habría enseñado (`ficha::armar`, su respaldo): hasta tres.
    pub respaldo: &'a [Respaldo],
    pub idioma: Idioma,
    /// Cuándo terminó la pregunta (la voz calló, o apareció sin voz).
    pub empezo_ms: u64,
    /// Cuándo cerraste: Enter, o el fin de tu voz si cerró el silencio.
    pub cerro_ms: u64,
}

// ─── el catálogo de muletillas ───────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Fila {
    frase: String,
    #[serde(default)]
    no_tras: Vec<String>,
}

#[derive(Deserialize)]
struct Archivo {
    version: u32,
    fecha: String,
    es: Vec<Fila>,
    en: Vec<Fila>,
}

/// Una muletilla del catálogo, ya en palabras plegadas.
struct Plegada {
    frase: String,
    palabras: Vec<String>,
    no_tras: Vec<String>,
}

pub struct Catalogo {
    pub version: u32,
    pub fecha: String,
    es: Vec<Plegada>,
    en: Vec<Plegada>,
}

impl Catalogo {
    fn de_texto(t: &str) -> Result<Self, String> {
        let a: Archivo = serde_json::from_str(t).map_err(|e| format!("muletillas.json: {e}"))?;
        let plegar_filas = |v: Vec<Fila>| -> Vec<Plegada> {
            v.into_iter()
                .map(|f| Plegada {
                    palabras: banco::palabras(&f.frase),
                    no_tras: f.no_tras.iter().map(|p| plegar(p)).collect(),
                    frase: f.frase,
                })
                .collect()
        };
        Ok(Self { version: a.version, fecha: a.fecha, es: plegar_filas(a.es), en: plegar_filas(a.en) })
    }

    fn del(&self, idioma: Idioma) -> &[Plegada] {
        match idioma {
            Idioma::Es => &self.es,
            Idioma::En => &self.en,
        }
    }

    /// Cuántas frases del catálogo hay en cada idioma (para el kit y la bitácora).
    pub fn cuantas(&self, idioma: Idioma) -> usize {
        self.del(idioma).len()
    }
}

pub fn catalogo() -> &'static Catalogo {
    static UNO: OnceLock<Catalogo> = OnceLock::new();
    UNO.get_or_init(|| Catalogo::de_texto(MULETILLAS).expect("data/ensayo/muletillas.json"))
}

/// Las muletillas de un texto, de la más repetida a la menos. Por frases enteras, sin solaparse, y sin
/// contar la que va detrás de una palabra de su «no_tras».
pub fn muletillas(texto: &str, idioma: Idioma) -> Vec<Muletilla> {
    let palabras = banco::palabras(texto);
    let mut salida: Vec<Muletilla> = Vec::new();
    for m in catalogo().del(idioma) {
        let n = m.palabras.len();
        if n == 0 {
            continue;
        }
        let (mut i, mut veces) = (0, 0u32);
        while i + n <= palabras.len() {
            let antes = i.checked_sub(1).map(|j| palabras[j].as_str());
            if palabras[i..i + n] == m.palabras[..] && !antes.is_some_and(|a| m.no_tras.iter().any(|x| x == a)) {
                veces += 1;
                i += n;
            } else {
                i += 1;
            }
        }
        if veces > 0 {
            salida.push(Muletilla { frase: m.frase.clone(), veces });
        }
    }
    salida.sort_by_key(|m| std::cmp::Reverse(m.veces));
    salida
}

/// Palabras por minuto entre la primera palabra y el fin del último turno con texto.
pub fn ppm(tramos: &[Tramo]) -> (u32, Option<u32>) {
    let con_texto: Vec<&Tramo> = tramos.iter().filter(|t| t.texto.as_deref().is_some_and(|x| !x.trim().is_empty())).collect();
    let palabras: usize = con_texto.iter().map(|t| t.texto.as_deref().unwrap_or("").split_whitespace().count()).sum();
    let (Some(primero), Some(ultimo)) = (con_texto.first(), con_texto.last()) else { return (0, None) };
    let ms = ultimo.hasta_ms.saturating_sub(primero.desde_ms);
    if palabras < PALABRAS_PARA_EL_RITMO || ms < MS_PARA_EL_RITMO {
        return (palabras as u32, None);
    }
    (palabras as u32, Some(((palabras as u64 * 60_000) / ms) as u32))
}

/// **LA EVALUACIÓN.** Ver el encabezado del módulo.
pub fn evaluar(e: &Entrada) -> Evaluacion {
    let respuesta: String = e
        .tramos
        .iter()
        .filter_map(|t| t.texto.as_deref())
        .collect::<Vec<_>>()
        .join(" ");
    let de_la_pregunta = banco::terminos(e.pregunta);
    let tuyos: std::collections::HashSet<String> =
        banco::terminos(&respuesta).difference(&de_la_pregunta).cloned().collect();
    let evidencia = e
        .respaldo
        .iter()
        .map(|r| {
            let de_la_ficha = banco::terminos(&format!("{} {}", r.titular, r.linea));
            Evidencia {
                titular: r.titular.clone(),
                fuente: r.fuente.clone(),
                citada: tuyos.intersection(&de_la_ficha).count() >= TERMINOS_PARA_CITAR,
                dicha_por_ti: false,
            }
        })
        .collect();
    let (palabras, ppm) = ppm(e.tramos);
    Evaluacion {
        evidencia,
        tiempo_ms: e.cerro_ms.saturating_sub(e.empezo_ms),
        ppm,
        muletillas: muletillas(&respuesta, e.idioma),
        palabras,
    }
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;
    use crate::corpus::Unidad;

    pub fn respaldo(titular: &str, linea: &str, unidad: Unidad) -> Respaldo {
        Respaldo {
            titular: titular.into(),
            linea: linea.into(),
            fuente: Fuente { documento: "Propuesta Páramo Azul".into(), seccion: Some(titular.into()), unidad: Some(unidad), conjeturada: false },
        }
    }

    pub fn tramo(desde_ms: u64, hasta_ms: u64, texto: &str) -> Tramo {
        Tramo { desde_ms, hasta_ms, texto: Some(texto.into()) }
    }

    fn las_tres() -> Vec<Respaldo> {
        vec![
            respaldo("Supuestos", "Si los extractos del ERP llegan tarde, el cronograma se corre día por día.", Unidad::Propuesta),
            respaldo("Cierre en Sur del Valle", "La limpieza de datos tomó tres semanas porque el inventario venía duplicado.", Unidad::Caso),
            respaldo("Etapa 2 · limpiar antes del tablero", "Ningún tablero se publica antes de reconciliar las fuentes con contabilidad.", Unidad::Marco),
        ]
    }

    const PREGUNTA: &str = "¿Qué pasa con el plazo si el ERP no entrega los datos limpios a tiempo?";

    #[test]
    fn el_catalogo_de_muletillas_se_lee_en_los_dos_idiomas() {
        let c = catalogo();
        assert_eq!((c.version, c.fecha.as_str()), (1, "2026-10-04"));
        assert!(c.cuantas(Idioma::Es) >= 5 && c.cuantas(Idioma::En) >= 5);
        // Lo que la transcripción de Apple quita no se promete contar (ADR 019 §6.6).
        let todas: Vec<&str> = c.es.iter().chain(&c.en).map(|m| m.frase.as_str()).collect();
        for quitada in ["eh", "um", "uh", "em"] {
            assert!(!todas.contains(&quitada), "«{quitada}» está en el catálogo y la transcripción no la conserva");
        }
    }

    /// **Citar es traer lo de la ficha, no repetir la pregunta.** La respuesta que solo devuelve las
    /// palabras de la pregunta no cita nada; la que trae el «día por día» y las «tres semanas», sí.
    #[test]
    fn repetir_la_pregunta_no_cuenta_como_citar() {
        let eco = [tramo(0, 4_000, "Si el ERP no entrega los datos limpios a tiempo, el plazo se mueve.")];
        let e = evaluar(&Entrada { pregunta: PREGUNTA, tramos: &eco, respaldo: &las_tres(), idioma: Idioma::Es, empezo_ms: 0, cerro_ms: 4_000 });
        assert_eq!(e.citadas(), 0, "{:?}", e.evidencia);

        let buena = [tramo(
            0,
            9_000,
            "El supuesto dos lo cubre: si los extractos llegan tarde el cronograma se corre día por día. En Sur del Valle la limpieza tomó tres semanas por el inventario duplicado.",
        )];
        let e = evaluar(&Entrada { pregunta: PREGUNTA, tramos: &buena, respaldo: &las_tres(), idioma: Idioma::Es, empezo_ms: 0, cerro_ms: 9_000 });
        let citadas: Vec<bool> = e.evidencia.iter().map(|x| x.citada).collect();
        assert_eq!(citadas, vec![true, true, false], "{:?}", e.evidencia);
        assert_eq!(e.citadas(), 2);
    }

    /// «Sí lo dije» manda, se deshace con otra pulsación, y no toca una que ya estaba citada.
    #[test]
    fn si_lo_dije_manda_y_se_deshace() {
        let t = [tramo(0, 3_000, "Lo vemos con el equipo.")];
        let mut e = evaluar(&Entrada { pregunta: PREGUNTA, tramos: &t, respaldo: &las_tres(), idioma: Idioma::Es, empezo_ms: 0, cerro_ms: 3_000 });
        assert_eq!(e.citadas(), 0);
        assert!(e.si_lo_dije(2));
        assert_eq!(e.citadas(), 1);
        assert!(e.evidencia[2].dicha_por_ti);
        assert!(e.si_lo_dije(2));
        assert_eq!(e.citadas(), 0);
        assert!(!e.si_lo_dije(9), "una ficha que no existe");
    }

    #[test]
    fn el_ritmo_va_de_la_primera_palabra_al_fin_del_ultimo_turno() {
        // 24 palabras en 12 s (de 2 s a 14 s, con una pausa en medio que cuenta): 120 ppm.
        let t = [
            tramo(2_000, 7_000, "uno dos tres cuatro cinco seis siete ocho nueve diez once doce"),
            Tramo { desde_ms: 7_500, hasta_ms: 8_000, texto: None },
            tramo(9_000, 14_000, "uno dos tres cuatro cinco seis siete ocho nueve diez once doce"),
        ];
        assert_eq!(ppm(&t), (24, Some(120)));
        // Muy poco que medir: ni ritmo ni división por cero.
        assert_eq!(ppm(&[tramo(0, 900, "sí, claro")]), (2, None));
        assert_eq!(ppm(&[]), (0, None));
        assert_eq!(ppm(&[Tramo { desde_ms: 0, hasta_ms: 5_000, texto: None }]), (0, None));
    }

    #[test]
    fn el_tiempo_va_de_la_pregunta_al_cierre() {
        let t = [tramo(1_200, 8_000, "algo")];
        let e = evaluar(&Entrada { pregunta: PREGUNTA, tramos: &t, respaldo: &[], idioma: Idioma::Es, empezo_ms: 1_000, cerro_ms: 8_000 });
        assert_eq!(e.tiempo_ms, 7_000);
        assert!(e.evidencia.is_empty(), "sin fichas para esa pregunta, no hay nada que acusar");
    }

    /// Por frases enteras, en el idioma del ensayo, sin contar las que no lo son.
    #[test]
    fn las_muletillas_se_cuentan_por_frases_enteras_y_sin_las_que_no_lo_son() {
        let es = "O sea, básicamente el plazo corre desde la entrega. O sea, si llegan tarde... vamos a ver el cronograma; a ver, o sea, se corre.";
        let m = muletillas(es, Idioma::Es);
        assert_eq!(
            m,
            vec![
                Muletilla { frase: "o sea".into(), veces: 3 },
                Muletilla { frase: "básicamente".into(), veces: 1 },
                Muletilla { frase: "a ver".into(), veces: 1 },
            ]
        );
        // «sea» suelto no es «o sea»; «osea» tampoco es una frase del catálogo.
        assert!(muletillas("Sea cual sea el plazo", Idioma::Es).is_empty());

        let en = "I mean, basically, it's like the timeline moves. I'd like to show you what I mean, you know, what kind of data, kind of messy.";
        let m = muletillas(en, Idioma::En);
        let veces = |f: &str| m.iter().find(|x| x.frase == f).map(|x| x.veces).unwrap_or(0);
        assert_eq!(veces("I mean"), 1, "«what I mean» no cuenta: {m:?}");
        assert_eq!(veces("basically"), 1);
        assert_eq!(veces("like"), 1, "«I'd like» no cuenta: {m:?}");
        assert_eq!(veces("you know"), 1);
        assert_eq!(veces("kind of"), 1, "«what kind of» no cuenta: {m:?}");
        // El catálogo de un idioma no cuenta en el otro.
        assert!(muletillas(es, Idioma::En).is_empty());
    }

    #[test]
    fn una_respuesta_vacia_no_rompe_nada() {
        let e = evaluar(&Entrada { pregunta: PREGUNTA, tramos: &[], respaldo: &las_tres(), idioma: Idioma::Es, empezo_ms: 500, cerro_ms: 200 });
        assert_eq!((e.citadas(), e.ppm, e.palabras, e.tiempo_ms, e.total_de_muletillas()), (0, None, 0, 0, 0));
        assert_eq!(e.evidencia.len(), 3);
    }

    /// Respondiste en el otro idioma: la evidencia se mide igual (los términos no tienen idioma) y las
    /// muletillas, en el del ensayo — la pantalla no inventa unas que no busca.
    #[test]
    fn una_respuesta_en_el_otro_idioma_se_mide_sin_inventar() {
        let t = [tramo(0, 6_000, "I mean, the schedule moves day by day if the extracts arrive late, basically.")];
        let e = evaluar(&Entrada { pregunta: PREGUNTA, tramos: &t, respaldo: &las_tres(), idioma: Idioma::Es, empezo_ms: 0, cerro_ms: 6_000 });
        assert_eq!(e.total_de_muletillas(), 0);
        assert_eq!(e.ppm, Some(140));
        // «extracts» y «extractos» comparten prefijo; el resto no: una sola, no cita.
        assert_eq!(e.citadas(), 0);
    }
}
