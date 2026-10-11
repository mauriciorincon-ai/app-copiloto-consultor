//! LA TOLERANCIA A TU VOZ — la sala, en el modo presencial (ADR 020 §3).
//!
//! En la sala tu voz y la del cliente entran por el mismo micrófono, y la app no sabe de quién es cada frase:
//! la regla dura 4 prohíbe averiguarlo por la voz. Lo que queda son **reglas sobre el texto y el tiempo**,
//! delante del disparador de siempre y solo para los turnos sin dueño (`Quien::SinAtribuir`):
//!
//! - **C1 · espera tras una ficha.** Lo que sigue a una ficha suele ser tu respuesta: durante un rato, la sala no
//!   dispara.
//! - **C2 · eco de la ficha.** Un turno que repite las palabras de la ficha que acabas de ver suele ser tú
//!   leyéndola. Solo cuentan las fichas con resultado, y se olvidan con el corte.
//! - **C3 · un término tuyo no basta.** Tú nombras tu corpus todo el rato; en la sala hace falta una pregunta o una
//!   cifra.
//! - **El silencio**, que puede apagarse en la sala: lo último que se dijo suele ser tuyo.
//! - **El tope de turno**: dos voces con pausas cortas se juntan en un turno; el tope lo corta antes de que pase de
//!   los 30 s del anillo (lo aplica `voz::turno::Turnos::con_tope`).
//!
//! **Ningún valor está cableado aquí.** La línea base, [`Tolerancia::SIN_FILTRO`], es el disparador tal cual; la
//! regla que usa la app la fija `data/presencial/reglas.json`, dentro del binario, y sale de la medición del kit v4
//! (`docs/kit-de-prueba/presencial.json`). Todo es código: ni un modelo, ni un rasgo de la voz.

use super::{normalizar, Motivo};

const REGLAS: &str = include_str!("../../../data/presencial/reglas.json");

/// Las perillas de la tolerancia. Cada una, apagada, deja el disparador como está.
#[derive(Clone, Debug, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tolerancia {
    /// C1: tras una ficha con resultado, la sala no dispara durante estos milisegundos. `0`, apagada. Por
    /// debajo de la espera de siempre entre fichas (`ESPERA_MS`) no cambia nada.
    pub espera_tras_ficha_ms: usize,
    /// C2: un turno con al menos esta fracción de sus palabras con peso en la última ficha no dispara. `None`,
    /// apagada.
    pub eco_de_la_ficha: Option<f32>,
    /// C3: en la sala, nombrar un término de tu corpus no basta.
    pub solo_pregunta_o_cifra: bool,
    /// Si el silencio largo dispara en la sala.
    pub silencio: bool,
    /// Lo más que dura un turno de la sala, en milisegundos. `None`, sin tope.
    pub tope_de_turno_ms: Option<usize>,
}

/// El archivo de la regla: la tolerancia y lo que la acompaña para quien lo lea.
#[derive(serde::Deserialize)]
struct Archivo {
    version: u32,
    tolerancia: Tolerancia,
}

impl Tolerancia {
    /// **La línea base del kit**: el disparador tal cual, sin ninguna perilla movida.
    pub const SIN_FILTRO: Tolerancia = Tolerancia {
        espera_tras_ficha_ms: 0,
        eco_de_la_ficha: None,
        solo_pregunta_o_cifra: false,
        silencio: true,
        tope_de_turno_ms: None,
    };

    /// **La regla de la casa**: la de `data/presencial/reglas.json`. Un archivo que no se lee no llega a un
    /// binario: lo vigila `la_regla_de_la_casa_se_lee`.
    pub fn de_la_casa() -> Tolerancia {
        static CASA: std::sync::OnceLock<Tolerancia> = std::sync::OnceLock::new();
        CASA.get_or_init(|| Self::leer(REGLAS).unwrap_or(Self::SIN_FILTRO)).clone()
    }

    /// Lee un archivo de regla. Pública para el kit, que lee la de la casa y la compara con la línea base.
    pub fn leer(json: &str) -> Result<Tolerancia, String> {
        let archivo: Archivo = serde_json::from_str(json).map_err(|e| e.to_string())?;
        if archivo.version != 1 {
            return Err(format!("versión {} de la regla de la sala: esta app lee la 1", archivo.version));
        }
        if let Some(f) = archivo.tolerancia.eco_de_la_ficha {
            if !(0.0..=1.0).contains(&f) {
                return Err(format!("ecoDeLaFicha {f}: es una fracción, entre 0 y 1"));
            }
        }
        Ok(archivo.tolerancia)
    }

    /// Cuántas perillas mueve respecto a la línea base: el desempate del kit (a igual medida, la más simple).
    pub fn perillas(&self) -> usize {
        let b = &Self::SIN_FILTRO;
        [
            self.espera_tras_ficha_ms != b.espera_tras_ficha_ms,
            self.eco_de_la_ficha != b.eco_de_la_ficha,
            self.solo_pregunta_o_cifra != b.solo_pregunta_o_cifra,
            self.silencio != b.silencio,
            self.tope_de_turno_ms != b.tope_de_turno_ms,
        ]
        .iter()
        .filter(|movida| **movida)
        .count()
    }

    /// **¿Deja disparar este turno de la sala?** `motivo` es el que dieron las reglas de siempre; `ahora_ms`, el
    /// reloj de la escucha; `vista`, la última ficha con resultado, si la hubo.
    pub fn deja(&self, motivo: Motivo, texto: &str, ahora_ms: usize, vista: Option<&FichaVista>) -> bool {
        if self.solo_pregunta_o_cifra && motivo == Motivo::TerminoDelCorpus {
            return false;
        }
        if motivo == Motivo::SilencioLargo && !self.silencio {
            return false;
        }
        let Some(vista) = vista else { return true };
        // El reloj puede volver atrás (un anillo que dio la vuelta): una ficha «del futuro» no cuenta.
        if self.espera_tras_ficha_ms > 0 && ahora_ms >= vista.ms && ahora_ms - vista.ms < self.espera_tras_ficha_ms {
            return false;
        }
        if let Some(umbral) = self.eco_de_la_ficha {
            if vista.repite(texto) >= umbral {
                return false;
            }
        }
        true
    }
}

impl Default for Tolerancia {
    fn default() -> Self {
        Self::de_la_casa()
    }
}

/// **La última ficha con resultado**: cuándo salió y sus palabras con peso. Son de TU corpus —su titular y su
/// línea—, no del cliente; aun así se olvidan con el corte, con el resto del disparador.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FichaVista {
    pub ms: usize,
    palabras: Vec<String>,
}

impl FichaVista {
    pub fn nueva(ms: usize, titular: &str, linea: &str) -> Self {
        Self { ms, palabras: palabras_con_peso(&format!("{titular} {linea}")) }
    }

    /// Qué fracción de las palabras con peso del turno está en la ficha: 0, ninguna; 1, todas. Un turno sin
    /// palabras con peso no repite nada.
    pub fn repite(&self, texto: &str) -> f32 {
        let del_turno = palabras_con_peso(texto);
        if del_turno.is_empty() {
            return 0.0;
        }
        let comunes = del_turno.iter().filter(|p| self.palabras.contains(p)).count();
        comunes as f32 / del_turno.len() as f32
    }
}

/// Las que dice todo el mundo, en los dos idiomas: no distinguen una frase de otra.
const VACIAS: &[&str] = &[
    "para", "pero", "como", "esta", "este", "esto", "estos", "estas", "porque", "sobre", "entre", "desde", "hasta",
    "cuando", "donde", "tiene", "tienen", "tenemos", "hacer", "puede", "pueden", "todo", "todos", "bien", "nosotros",
    "ustedes", "ellos", "entonces", "tambien", "that", "this", "with", "from", "have", "what", "when",
    "where", "they", "there", "their", "would", "could", "should", "about", "which", "your", "then", "than", "will",
    "just", "into", "does",
];

/// Las palabras con peso de un texto, sin repetir: de cuatro letras o más, o con una cifra, y fuera las vacías.
fn palabras_con_peso(texto: &str) -> Vec<String> {
    let mut salida: Vec<String> = Vec::new();
    for p in normalizar(texto).split_whitespace() {
        let pesa = p.chars().count() >= 4 || p.chars().any(|c| c.is_ascii_digit());
        if pesa && !VACIAS.contains(&p) && !salida.iter().any(|s| s == p) {
            salida.push(p.to_string());
        }
    }
    salida
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **La regla de la casa se lee**, y un archivo torcido no pasa en silencio. ¿Puede fallar? Sí: con un campo
    /// mal escrito en `reglas.json`, `deny_unknown_fields` lo rechaza y este test es rojo (bitácora del sprint 005,
    /// fase 1).
    #[test]
    fn la_regla_de_la_casa_se_lee() {
        Tolerancia::leer(REGLAS).expect("data/presencial/reglas.json no se lee");
        assert!(Tolerancia::leer(r#"{"version":1,"tolerancia":{"esperaTrasFicha":0}}"#).is_err());
        assert!(Tolerancia::leer(r#"{"version":2,"tolerancia":{"esperaTrasFichaMs":0,"ecoDeLaFicha":null,"soloPreguntaOCifra":false,"silencio":true,"topeDeTurnoMs":null}}"#).is_err());
        assert!(Tolerancia::leer(r#"{"version":1,"tolerancia":{"esperaTrasFichaMs":0,"ecoDeLaFicha":1.5,"soloPreguntaOCifra":false,"silencio":true,"topeDeTurnoMs":null}}"#).is_err());
    }

    /// **La línea base no quita nada**: con `SIN_FILTRO`, todo lo que dieron las reglas de siempre pasa.
    #[test]
    fn sin_filtro_lo_deja_todo() {
        let vista = FichaVista::nueva(1_000, "Plazo de entrega: cuatro semanas", "La entrega completa toma cuatro semanas");
        for motivo in [Motivo::Pregunta, Motivo::Cifra, Motivo::TerminoDelCorpus, Motivo::SilencioLargo] {
            assert!(Tolerancia::SIN_FILTRO.deja(motivo, "la entrega completa toma cuatro semanas", 1_500, Some(&vista)));
        }
        assert_eq!(Tolerancia::SIN_FILTRO.perillas(), 0);
    }

    /// **Cada perilla, sola, hace lo suyo y nada más.** ¿Puede fallar? Sí: con la comparación de C1 al revés
    /// (`>=`), la espera tras la ficha callaría lo de después y dejaría lo de durante (bitácora del sprint 005,
    /// fase 1).
    #[test]
    fn cada_perilla_hace_lo_suyo() {
        let vista = FichaVista::nueva(10_000, "Plazo de entrega: cuatro semanas", "La entrega completa toma cuatro semanas desde la firma");

        let c1 = Tolerancia { espera_tras_ficha_ms: 12_000, ..Tolerancia::SIN_FILTRO };
        assert!(!c1.deja(Motivo::Pregunta, "¿y el precio?", 15_000, Some(&vista)), "C1 dejó disparar durante la espera");
        assert!(c1.deja(Motivo::Pregunta, "¿y el precio?", 22_000, Some(&vista)), "C1 calló después de la espera");
        assert!(c1.deja(Motivo::Pregunta, "¿y el precio?", 5_000, Some(&vista)), "C1 contó una ficha del futuro");
        assert!(c1.deja(Motivo::Pregunta, "¿y el precio?", 15_000, None), "C1 calló sin ficha vista");

        let c2 = Tolerancia { eco_de_la_ficha: Some(0.5), ..Tolerancia::SIN_FILTRO };
        assert!(!c2.deja(Motivo::Cifra, "Sí, la entrega completa son cuatro semanas desde la firma.", 30_000, Some(&vista)), "C2 dejó disparar tu lectura de la ficha");
        assert!(c2.deja(Motivo::Pregunta, "¿Y quién revisa el tablero de margen?", 30_000, Some(&vista)), "C2 calló una pregunta sobre otra cosa");

        let c3 = Tolerancia { solo_pregunta_o_cifra: true, ..Tolerancia::SIN_FILTRO };
        assert!(!c3.deja(Motivo::TerminoDelCorpus, "lo del Páramo Azul", 0, None), "C3 dejó disparar un término suelto");
        assert!(c3.deja(Motivo::Pregunta, "¿Páramo Azul?", 0, None) && c3.deja(Motivo::Cifra, "12 semanas", 0, None));

        let sin_silencio = Tolerancia { silencio: false, ..Tolerancia::SIN_FILTRO };
        assert!(!sin_silencio.deja(Motivo::SilencioLargo, "lo último", 0, None), "el silencio apagado disparó");
        assert!(sin_silencio.deja(Motivo::Pregunta, "¿y lo último?", 0, None));

        for (t, n) in [(c1, 1), (c2, 1), (c3, 1), (sin_silencio, 1)] {
            assert_eq!(t.perillas(), n);
        }
    }

    /// **Qué cuenta como repetir la ficha**: las palabras con peso, sin mayúsculas ni tildes, y las cifras.
    #[test]
    fn repetir_la_ficha_se_mide_en_palabras_con_peso() {
        let vista = FichaVista::nueva(0, "Certificaciones", "No hay certificación ISO vigente");
        assert!(vista.repite("No, certificación ISO vigente no hay.") > 0.6);
        assert_eq!(vista.repite("ajá, sí"), 0.0, "un turno sin palabras con peso no repite nada");
        assert!(vista.repite("¿Cuánto cuesta una fuente adicional?") < 0.2);
    }
}
