//! LAS PREFERENCIAS — lo que eliges una vez y la app recuerda (ADR 002, enmienda 2).
//!
//! Hasta el sprint 002 solo persistía la cifra del gasto: el idioma de cada pista, los interruptores
//! de IA y la lectura automática volvían a su valor de fábrica al cerrar la app, y el manual lo
//! declaraba como limitación (deuda del S2 con pago en el sprint 003). Ahora viven en
//! `preferencias.json`, en la carpeta de configuración de la app, en 600 (`almacen`).
//!
//! **Qué entra:** lo que el usuario elige y no dice nada de ninguna reunión — idiomas por pista,
//! «redactar sugerencias», el proveedor externo y si está encendido, la lectura automática. Las
//! fases siguientes del sprint 003 suman la retención de las notas, «conservar lo que dije», la
//! ventana de la bandeja y la respuesta de la NDA de cada cliente.
//!
//! **Qué NO entra, a propósito:** la puerta local (nace cerrada en cada arranque, ADR 018: una
//! puerta que se recuerda abierta se abre sola), el modo solo audio (lo enciende una tecla, en la
//! reunión, sabiendo por dónde suena el Mac) y nada de lo que pasó en una reunión.
//!
//! **Un archivo torcido no detiene la app:** si no se puede leer, o es de otra versión, se usan las
//! de fábrica y el log lo dice — igual que el diccionario.

use serde::{Deserialize, Serialize};

use crate::sintesis::api::Externo;

/// Cómo se llama el archivo, al lado del diccionario y del gasto del mes.
pub const ARCHIVO: &str = "preferencias.json";

/// La versión del formato. Un archivo de otra versión no se interpreta a ciegas: se usan las de
/// fábrica y se dice.
pub const VERSION: u32 = 1;

/// Qué idioma escucha cada pista. Nacen las dos en español: el corpus y los clientes del usuario lo
/// son (auditoría del S2, A4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdiomasDePista {
    pub consultor: String,
    pub cliente: String,
}

impl Default for IdiomasDePista {
    fn default() -> Self {
        Self { consultor: "es-ES".into(), cliente: "es-ES".into() }
    }
}

/// Cuánto viven tus notas guardadas (ADR 015 §6). Una elección para todas las reuniones; cada
/// archivo se estampa con la suya al guardarse. «Nunca» no está: eso es «Cerrar sin guardar».
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Retencion {
    #[serde(rename = "7d")]
    Dias7,
    #[serde(rename = "30d")]
    Dias30,
    /// La de fábrica, como la maqueta («se borra solo en 90 días»). Mirada 19.
    #[default]
    #[serde(rename = "90d")]
    Dias90,
    #[serde(rename = "1a")]
    Anio,
    #[serde(rename = "siempre")]
    Siempre,
}

impl Retencion {
    pub const TODAS: [Retencion; 5] =
        [Retencion::Dias7, Retencion::Dias30, Retencion::Dias90, Retencion::Anio, Retencion::Siempre];

    /// Cuánto vive, en segundos. `None` es «siempre».
    pub fn segundos(self) -> Option<i64> {
        const DIA: i64 = 86_400;
        match self {
            Retencion::Dias7 => Some(7 * DIA),
            Retencion::Dias30 => Some(30 * DIA),
            Retencion::Dias90 => Some(90 * DIA),
            Retencion::Anio => Some(365 * DIA),
            Retencion::Siempre => None,
        }
    }

    /// El vencimiento de algo guardado `ahora` (segundos Unix). 0 es «siempre», como en la cabecera
    /// del archivo (`notas::cifrado`).
    pub fn vence(self, ahora: i64) -> i64 {
        self.segundos().map_or(0, |s| ahora + s)
    }
}

/// Lo que se recuerda. Los valores de fábrica son los de siempre: nada cambia para quien no toca
/// nada.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferencias {
    pub idiomas: IdiomasDePista,
    /// «Redactar sugerencias» (ADR 010). De fábrica, apagado.
    pub redactar: bool,
    /// El proveedor externo (ADR 011). Encendido solo si el usuario lo encendió **y** su clave
    /// sigue en el Llavero al arrancar — si la borró, no se enciende solo.
    pub api_encendida: bool,
    pub externo: Externo,
    /// La lectura automática de la pantalla (C8). De fábrica, encendida.
    pub lectura_automatica: bool,
    /// Cuánto viven tus notas guardadas (ADR 015 §6).
    pub retencion: Retencion,
    /// «Conservar mis turnos»: tus turnos, en texto, entran al archivo de la reunión. De fábrica,
    /// apagado (regla dura 1).
    pub conservar_mis_turnos: bool,
    /// Cuánto esperan en la bandeja las propuestas que no decidiste (ADR 016 §4). De fábrica, 3 h.
    pub ventana_de_la_bandeja: crate::bandeja::Ventana,
    /// Lo que respondiste de la NDA de cada cliente (ADR 017 §4), por su nombre. «Sin revisar» es no
    /// estar aquí. La NDA no cambia de una reunión a otra, por eso se guarda; con quién te reúnes hoy,
    /// no.
    pub ndas: std::collections::BTreeMap<String, crate::jurisdiccion::Nda>,
    /// La carpeta de tu corpus: la ruta, jamás su contenido. Al arrancar se vuelve a leer, en segundo
    /// plano (auditoría del S3, B29; decisión del usuario, 2026-09-28: «que la recuerde»).
    pub carpeta_del_corpus: Option<String>,
}

impl Default for Preferencias {
    fn default() -> Self {
        Self {
            idiomas: IdiomasDePista::default(),
            redactar: false,
            api_encendida: false,
            externo: Externo::Claude,
            lectura_automatica: true,
            retencion: Retencion::default(),
            conservar_mis_turnos: false,
            ventana_de_la_bandeja: crate::bandeja::Ventana::default(),
            ndas: std::collections::BTreeMap::new(),
            carpeta_del_corpus: None,
        }
    }
}

/// El archivo en disco: la versión y las preferencias al mismo nivel.
#[derive(Serialize, Deserialize)]
struct Archivo {
    version: u32,
    #[serde(flatten)]
    preferencias: Preferencias,
}

/// Un código de idioma como los que la app usa: `es-ES`, `en-US`. Lo que el usuario escriba a mano
/// en el archivo no llega al motor de voz sin pasar por aquí.
pub fn es_un_idioma(codigo: &str) -> bool {
    let b = codigo.as_bytes();
    b.len() == 5
        && b[0].is_ascii_lowercase()
        && b[1].is_ascii_lowercase()
        && b[2] == b'-'
        && b[3].is_ascii_uppercase()
        && b[4].is_ascii_uppercase()
}

/// Las preferencias de un texto, o las de fábrica con el motivo.
pub fn de_texto(texto: &str) -> Result<Preferencias, String> {
    let a: Archivo = serde_json::from_str(texto).map_err(|e| format!("no se entiende: {e}"))?;
    if a.version != VERSION {
        return Err(format!("es de la versión {}, y esta app lee la {VERSION}", a.version));
    }
    let mut p = a.preferencias;
    let fabrica = IdiomasDePista::default();
    if !es_un_idioma(&p.idiomas.consultor) {
        p.idiomas.consultor = fabrica.consultor.clone();
    }
    if !es_un_idioma(&p.idiomas.cliente) {
        p.idiomas.cliente = fabrica.cliente;
    }
    Ok(p)
}

pub fn a_texto(p: &Preferencias) -> String {
    serde_json::to_string_pretty(&Archivo { version: VERSION, preferencias: p.clone() }).unwrap_or_default()
}

/// Las del disco; si no hay archivo, las de fábrica en silencio (es el primer arranque); si lo hay
/// y no se puede leer, las de fábrica **y el motivo al log**.
pub fn leer(ruta: &std::path::Path) -> Preferencias {
    match std::fs::read_to_string(ruta) {
        Err(_) => Preferencias::default(),
        Ok(t) => de_texto(&t).unwrap_or_else(|e| {
            println!("[prefs] el archivo de preferencias {e}: se usan las de fábrica");
            Preferencias::default()
        }),
    }
}

pub fn guardar(ruta: &std::path::Path, p: &Preferencias) -> Result<(), String> {
    crate::almacen::escribir(ruta, a_texto(p).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn carpeta(nombre: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ag-prefs-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// **El reinicio:** lo que se guardó es lo que se lee en el arranque siguiente. ¿Puede fallar?
    /// Sí: sin `#[serde(default)]` un archivo viejo sin un campo nuevo deja de leerse, y sin la
    /// escritura el arranque vuelve a fábrica (bitácora del sprint 003).
    #[test]
    fn lo_que_se_elige_sobrevive_al_reinicio() {
        let d = carpeta("reinicio");
        let ruta = d.join(ARCHIVO);
        let elegidas = Preferencias {
            idiomas: IdiomasDePista { consultor: "es-ES".into(), cliente: "en-US".into() },
            redactar: true,
            api_encendida: true,
            externo: Externo::Groq,
            lectura_automatica: false,
            retencion: Retencion::Anio,
            conservar_mis_turnos: true,
            ventana_de_la_bandeja: crate::bandeja::Ventana::FinDelDia,
            ndas: [("Páramo Azul".to_string(), crate::jurisdiccion::Nda::LoProhibe)].into(),
            // La carpeta del corpus se recuerda (auditoría del S3, B29; decisión del usuario): sin
            // ella, tras reiniciar no había «Este cliente», ni NDA, ni corpus para la puerta.
            carpeta_del_corpus: Some("/Users/consultor/Corpus".into()),
        };
        guardar(&ruta, &elegidas).unwrap();
        assert_eq!(leer(&ruta), elegidas);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn sin_archivo_son_las_de_fabrica() {
        let p = leer(&carpeta("ausente").join(ARCHIVO));
        assert_eq!(p, Preferencias::default());
        assert_eq!((p.idiomas.cliente.as_str(), p.redactar, p.lectura_automatica), ("es-ES", false, true));
    }

    /// La retención de fábrica es la de la maqueta (90 días) y cada una vence lo que dice. «Siempre»
    /// es el 0 de la cabecera del archivo.
    #[test]
    fn la_retencion_vence_lo_que_dice() {
        assert_eq!(Preferencias::default().retencion, Retencion::Dias90);
        assert!(!Preferencias::default().conservar_mis_turnos, "tus turnos nacen apagados");
        assert_eq!(Preferencias::default().ventana_de_la_bandeja, crate::bandeja::Ventana::TresHoras, "la bandeja espera 3 h de fábrica");
        let ahora = 1_800_000_000;
        assert_eq!(Retencion::Dias7.vence(ahora), ahora + 7 * 86_400);
        assert_eq!(Retencion::Anio.vence(ahora), ahora + 365 * 86_400);
        assert_eq!(Retencion::Siempre.vence(ahora), 0);
        let texto = a_texto(&Preferencias { retencion: Retencion::Dias30, ..Preferencias::default() });
        assert!(texto.contains(r#""retencion": "30d""#), "{texto}");
    }

    /// Un archivo de un sprint anterior al que le falta un campo se sigue leyendo: lo que falta
    /// toma su valor de fábrica.
    #[test]
    fn un_campo_que_falta_toma_su_valor_de_fabrica() {
        let p = de_texto(r#"{"version":1,"redactar":true}"#).unwrap();
        assert!(p.redactar);
        assert_eq!(p.idiomas, IdiomasDePista::default());
    }

    /// Y al revés: un campo que ya no existe no rompe la lectura. `carpetaDeNotas` vivió en la fase 1
    /// del sprint 003, hasta que las notas pasaron a la carpeta de la app (decisión A): un archivo de
    /// entonces se sigue leyendo entero, y ese campo se ignora.
    #[test]
    fn un_campo_que_ya_no_existe_se_ignora() {
        let p = de_texto(r#"{"version":1,"redactar":true,"carpetaDeNotas":"/Users/quien/Notas"}"#).unwrap();
        assert!(p.redactar);
    }

    /// **Lo escrito a mano no llega al motor de voz sin validar.** Un código que no es de idioma
    /// vuelve al de fábrica; los demás campos se conservan.
    #[test]
    fn un_idioma_torcido_vuelve_al_de_fabrica() {
        let p = de_texto(r#"{"version":1,"idiomas":{"consultor":"es-ES","cliente":"../../x"},"redactar":true}"#).unwrap();
        assert_eq!(p.idiomas.cliente, "es-ES");
        assert!(p.redactar);
    }

    #[test]
    fn otra_version_o_un_archivo_torcido_caen_a_fabrica_con_motivo() {
        assert!(de_texto(r#"{"version":7}"#).unwrap_err().contains("versión 7"));
        assert!(de_texto("no es json").is_err());
    }

    #[test]
    fn nace_en_600() {
        use std::os::unix::fs::PermissionsExt;
        let d = carpeta("permisos");
        let ruta = d.join(ARCHIVO);
        guardar(&ruta, &Preferencias::default()).unwrap();
        assert_eq!(std::fs::metadata(&ruta).unwrap().permissions().mode() & 0o777, 0o600);
        let _ = std::fs::remove_dir_all(&d);
    }
}
