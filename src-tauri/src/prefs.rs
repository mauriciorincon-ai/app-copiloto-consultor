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
    /// **El idioma de la sala, en presencial** (ADR 020 §6; ADR 002, enmienda 9). `None` es «el del
    /// cliente», que es el de fábrica: un archivo de antes del sprint 005, sin este campo, se lee igual.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sala: Option<String>,
}

impl IdiomasDePista {
    /// En qué idioma escucha la sala: el elegido, o el del cliente.
    pub fn de_la_sala(&self) -> &str {
        self.sala.as_deref().unwrap_or(&self.cliente)
    }
}

impl Default for IdiomasDePista {
    fn default() -> Self {
        Self { consultor: "es-ES".into(), cliente: "es-ES".into(), sala: None }
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
    /// Dónde vive la banda (ADR 004 enmienda 1, ADR 002 enmienda 8). De fábrica, **arriba**, bajo la
    /// barra de menús y junto a la cámara (decisión del usuario, 2026-09-27); `⌃⌥B` la alterna.
    pub posicion_de_la_banda: crate::ventana::Borde,
    /// Que ya viste, en Sesión, el aviso de la primera vez con la banda arriba. «Entendido» lo apaga
    /// para siempre; un archivo del H1, que no lo tiene, lo enseña una vez.
    pub aviso_de_arriba_visto: bool,
    /// «Enriquecer el banco» (ADR 019 §3): un modelo propone hasta cinco preguntas de más en el
    /// ensayo. De fábrica, apagado: el ensayo es entero sin él.
    pub enriquecer_el_banco: bool,
    /// Lo que whisper tardó en este PC con cada modelo (sprint 008, M3 del S6). Metadatos de costo, en claro 600 con
    /// el resto: ni texto ni audio. Sin esto, al reabrir la app Honestidad decía «sin medir» y el escalón al modelo
    /// pequeño se olvidaba hasta la siguiente calibración.
    pub latencias_de_whisper: LatenciasDeWhisper,
    /// **A qué altura vive la banda arriba** (sprint 011, ADR 004 enmienda 3, ADR 002 enmienda 10): cuántos puntos la
    /// bajaste con el asa de posición desde el borde superior del área útil. De fábrica 0, pegada arriba como siempre; abajo
    /// no se mira. **Con 0 no se escribe**: el archivo de quien nunca la baja —el del Mac, entero— no cambia ni un byte.
    #[serde(skip_serializing_if = "es_cero")]
    pub desplazamiento_de_la_banda: u32,
    /// **Los clientes con el modo transparente encendido** (sprint 012, ADR 024), por su nombre. Como la NDA: es del
    /// cliente y no cambia de una reunión a otra. **Vacío no se escribe**: el archivo de quien no lo usa —el del Mac,
    /// entero— no cambia ni un byte.
    #[serde(skip_serializing_if = "std::collections::BTreeSet::is_empty")]
    pub transparentes: std::collections::BTreeSet<String>,
}

/// Lo que no se escribe en el archivo si es el valor de fábrica (el desplazamiento de la banda, sprint 011).
fn es_cero(n: &u32) -> bool {
    *n == 0
}

/// Cuánto tardó cada modelo de whisper en transcribir el recorte de calibración, en milisegundos (`None`: sin medir).
/// Desde el sprint 010, pista por pista: tu turno de 10 s (el que decide el modelo) y el del cliente; un archivo de
/// antes no trae las del cliente y quedan sin medir.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LatenciasDeWhisper {
    pub grande_ms: Option<u32>,
    pub pequeno_ms: Option<u32>,
    pub grande_cliente_ms: Option<u32>,
    pub pequeno_cliente_ms: Option<u32>,
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
            posicion_de_la_banda: crate::ventana::Borde::Arriba,
            aviso_de_arriba_visto: false,
            enriquecer_el_banco: false,
            latencias_de_whisper: LatenciasDeWhisper::default(),
            desplazamiento_de_la_banda: 0,
            transparentes: std::collections::BTreeSet::new(),
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

/// Un proveedor que salió de la app (Gemini, decisión del usuario 2026-09-29, ADR 011) no tumba el
/// archivo entero: vuelve el de fábrica **y el API queda apagado** — jamás se enciende solo con otro.
/// Devuelve el nombre del que salió, para el log.
fn sin_proveedor_que_salio(v: &mut serde_json::Value) -> Option<String> {
    let o = v.as_object_mut()?;
    let cual = o.get("externo")?.as_str()?.to_string();
    if serde_json::from_value::<Externo>(serde_json::Value::String(cual.clone())).is_ok() {
        return None;
    }
    o.insert("externo".into(), serde_json::to_value(Preferencias::default().externo).ok()?);
    o.insert("apiEncendida".into(), serde_json::Value::Bool(false));
    Some(cual)
}

/// Las preferencias de un texto, o las de fábrica con el motivo.
pub fn de_texto(texto: &str) -> Result<Preferencias, String> {
    let mut v: serde_json::Value = serde_json::from_str(texto).map_err(|e| format!("no se entiende: {e}"))?;
    if let Some(cual) = sin_proveedor_que_salio(&mut v) {
        println!("[prefs] el proveedor «{cual}» ya no está en la app: el API queda apagado");
    }
    let a: Archivo = serde_json::from_value(v).map_err(|e| format!("no se entiende: {e}"))?;
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
    // La sala torcida vuelve a «la del cliente», que es su valor de fábrica (ADR 002, enmienda 9).
    if p.idiomas.sala.as_deref().is_some_and(|s| !es_un_idioma(s)) {
        p.idiomas.sala = None;
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

/// **Apaga lo que esta plataforma ya no tiene** (sprint 013, auditoría B-4): el archivo de quien encendió el API o la
/// sugerencia redactada en Windows con el sprint 012 sigue diciéndolo, y lo que se recuerda no puede prometer lo que la
/// app ya no hace. En Windows el API pasa a apagado siempre; «redactar», solo si el modelo local no está instalado en
/// este PC (sprint 014); «enriquecer el banco», siempre, porque no se midió con el modelo local. `mock` (la CI y el kit)
/// no se toca. Devuelve si cambió algo, para guardar el archivo una vez. Un derivado jamás nace menos privado que su
/// fuente: apagar un envío a un tercero es más privado, no menos. El Mac vuelve sin cambiar un byte.
pub fn para_esta_plataforma(
    p: &mut Preferencias,
    plataforma: crate::sintesis::Plataforma,
    mock: bool,
    local_instalado: bool,
) -> bool {
    let antes = (p.api_encendida, p.redactar, p.enriquecer_el_banco);
    if !plataforma.admite_api() {
        p.api_encendida = false;
    }
    if !plataforma.promete_redactar(mock, local_instalado) {
        p.redactar = false;
    }
    if !plataforma.promete_enriquecer(mock) {
        p.enriquecer_el_banco = false;
    }
    antes != (p.api_encendida, p.redactar, p.enriquecer_el_banco)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn carpeta(nombre: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ag-prefs-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    /// **Lo que Windows ya no tiene se apaga y se recuerda apagado**; el Mac no cambia ni un byte, y el `mock` sigue. Con
    /// el modelo local instalado (sprint 014), «redactar» se queda; el API y el banco enriquecido, no.
    #[test]
    fn windows_apaga_lo_que_ya_no_tiene_y_el_mac_no_toca_nada() {
        use crate::sintesis::Plataforma::{Mac, Windows};
        let viejo = Preferencias { api_encendida: true, redactar: true, enriquecer_el_banco: true, ..Preferencias::default() };
        let mut w = viejo.clone();
        assert!(para_esta_plataforma(&mut w, Windows, false, false), "debía decir que cambió");
        assert!(!w.api_encendida && !w.redactar && !w.enriquecer_el_banco, "{w:?}");
        assert_eq!(Preferencias { api_encendida: false, redactar: false, enriquecer_el_banco: false, ..viejo.clone() }, w);
        assert!(!para_esta_plataforma(&mut w, Windows, false, false), "la segunda vez ya no hay nada que apagar");
        let mut con_local = viejo.clone();
        assert!(para_esta_plataforma(&mut con_local, Windows, false, true));
        assert!(con_local.redactar && !con_local.api_encendida && !con_local.enriquecer_el_banco, "{con_local:?}");
        for local in [false, true] {
            let mut m = viejo.clone();
            assert!(!para_esta_plataforma(&mut m, Mac, false, local));
            assert_eq!(m, viejo, "el Mac no cambia");
        }
        let mut con_mock = viejo.clone();
        assert!(para_esta_plataforma(&mut con_mock, Windows, true, false), "el API sí se apaga aunque haya mock");
        assert!(con_mock.redactar && con_mock.enriquecer_el_banco && !con_mock.api_encendida);
    }

    /// **El reinicio:** lo que se guardó es lo que se lee en el arranque siguiente. ¿Puede fallar?
    /// Sí: sin `#[serde(default)]` un archivo viejo sin un campo nuevo deja de leerse, y sin la
    /// escritura el arranque vuelve a fábrica (bitácora del sprint 003).
    #[test]
    fn lo_que_se_elige_sobrevive_al_reinicio() {
        let d = carpeta("reinicio");
        let ruta = d.join(ARCHIVO);
        let elegidas = Preferencias {
            idiomas: IdiomasDePista { consultor: "es-ES".into(), cliente: "en-US".into(), sala: Some("es-ES".into()) },
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
            // La banda abajo y el aviso ya visto (sprint 004): lo contrario de fábrica, para que un
            // `default` que se coma el campo caiga aquí.
            posicion_de_la_banda: crate::ventana::Borde::Abajo,
            aviso_de_arriba_visto: true,
            enriquecer_el_banco: true,
            // Lo que whisper tardó en este PC se recuerda (M3 del S6): lo contrario de «sin medir».
            latencias_de_whisper: LatenciasDeWhisper { grande_ms: Some(5_200), pequeno_ms: Some(1_900), ..Default::default() },
            // La altura de la banda se recuerda (sprint 011): lo contrario de fábrica.
            desplazamiento_de_la_banda: 132,
            // El modo transparente se recuerda por cliente (sprint 012).
            transparentes: ["Páramo Azul".to_string()].into(),
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

    /// Gemini salió de la app (decisión del usuario, 2026-09-29; ADR 011). Unas preferencias que lo
    /// tenían elegido no se pierden enteras —la NDA de cada cliente, la retención…— y el API queda
    /// apagado: la app jamás lo enciende sola con un proveedor que no elegiste.
    #[test]
    fn un_proveedor_que_salio_no_borra_las_demas_ni_enciende_otro() {
        let texto = r#"{"version":1,"redactar":true,"apiEncendida":true,"externo":"gemini","retencion":"30d"}"#;
        let p = de_texto(texto).expect("el archivo entero se perdía por un proveedor que ya no está");
        assert_eq!(p.externo, Externo::Claude);
        assert!(!p.api_encendida, "se encendió solo con otro proveedor");
        assert!(p.redactar);
        assert_eq!(p.retencion, Retencion::Dias30);
        // Uno que sigue en la app se lee tal cual.
        let groq = de_texto(r#"{"version":1,"apiEncendida":true,"externo":"groq"}"#).unwrap();
        assert_eq!((groq.externo, groq.api_encendida), (Externo::Groq, true));
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

    /// **Un archivo sin las medidas de whisper las deja sin medir** (M3 del S6), y las medidas se escriben y se leen.
    #[test]
    fn las_medidas_de_whisper_son_opcionales_y_viajan() {
        let p = de_texto(r#"{"version":1,"redactar":true}"#).unwrap();
        assert_eq!(p.latencias_de_whisper, LatenciasDeWhisper::default());
        let con = Preferencias {
            latencias_de_whisper: LatenciasDeWhisper { grande_ms: Some(5_200), grande_cliente_ms: Some(7_400), ..Default::default() },
            ..p
        };
        let texto = a_texto(&con);
        assert!(texto.contains(r#""grandeMs": 5200"#), "{texto}");
        assert!(texto.contains(r#""grandeClienteMs": 7400"#), "{texto}");
        let leidas = de_texto(&texto).unwrap().latencias_de_whisper;
        assert_eq!((leidas.grande_ms, leidas.grande_cliente_ms, leidas.pequeno_cliente_ms), (Some(5_200), Some(7_400), None));
    }

    /// **Unas preferencias del sprint 009 se siguen leyendo** (sprint 010): traen tu turno y no la pista del cliente;
    /// lo que traen vale y lo que no, queda sin medir.
    #[test]
    fn las_medidas_de_antes_de_la_pista_del_cliente_se_leen() {
        let p = de_texto(r#"{"version":1,"latenciasDeWhisper":{"grandeMs":9144,"pequenoMs":1632}}"#).unwrap();
        assert_eq!(
            p.latencias_de_whisper,
            LatenciasDeWhisper { grande_ms: Some(9_144), pequeno_ms: Some(1_632), grande_cliente_ms: None, pequeno_cliente_ms: None }
        );
    }

    /// **Un archivo del H1 abre la banda arriba y enseña el aviso una vez** (sprint 004): no tiene ni
    /// la posición ni el aviso, y los dos toman su valor de fábrica. Así se escriben en el archivo.
    #[test]
    fn un_archivo_del_h1_abre_la_banda_arriba_y_enseña_el_aviso() {
        let p = de_texto(r#"{"version":1,"redactar":true,"retencion":"30d"}"#).unwrap();
        assert_eq!(p.posicion_de_la_banda, crate::ventana::Borde::Arriba);
        assert!(!p.aviso_de_arriba_visto);
        assert!(!p.enriquecer_el_banco, "el acento del ensayo nace apagado");
        let texto = a_texto(&Preferencias { posicion_de_la_banda: crate::ventana::Borde::Abajo, ..p });
        assert!(texto.contains(r#""posicionDeLaBanda": "abajo""#), "{texto}");
        assert!(texto.contains(r#""avisoDeArribaVisto": false"#), "{texto}");
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

    /// **La sala** (sprint 005, ADR 020 §6): un archivo de antes, sin el campo, escucha la sala en el
    /// idioma del cliente; una sala torcida vuelve a eso mismo; y una elegida se respeta.
    #[test]
    fn la_sala_vale_la_del_cliente_si_no_se_eligio() {
        let antes = de_texto(r#"{"version":1,"idiomas":{"consultor":"es-ES","cliente":"en-US"}}"#).unwrap();
        assert_eq!(antes.idiomas.sala, None);
        assert_eq!(antes.idiomas.de_la_sala(), "en-US");
        let torcida = de_texto(r#"{"version":1,"idiomas":{"consultor":"es-ES","cliente":"en-US","sala":"../x"}}"#).unwrap();
        assert_eq!(torcida.idiomas.de_la_sala(), "en-US");
        let elegida = de_texto(r#"{"version":1,"idiomas":{"consultor":"es-ES","cliente":"en-US","sala":"es-ES"}}"#).unwrap();
        assert_eq!(elegida.idiomas.de_la_sala(), "es-ES");
    }

    #[test]
    fn otra_version_o_un_archivo_torcido_caen_a_fabrica_con_motivo() {
        assert!(de_texto(r#"{"version":7}"#).unwrap_err().contains("versión 7"));
        assert!(de_texto("no es json").is_err());
    }

    // Comprueba permisos de Unix (600/700): en Windows la carpeta hereda la ACL del perfil (ADR 022 §2).
    #[cfg(unix)]
    #[test]
    fn nace_en_600() {
        use std::os::unix::fs::PermissionsExt;
        let d = carpeta("permisos");
        let ruta = d.join(ARCHIVO);
        guardar(&ruta, &Preferencias::default()).unwrap();
        assert_eq!(std::fs::metadata(&ruta).unwrap().permissions().mode() & 0o777, 0o600);
        let _ = std::fs::remove_dir_all(&d);
    }

    /// **La altura de la banda (sprint 011): un archivo de antes la lee en 0, y el 0 no se escribe.** Así el archivo
    /// de quien nunca baja la banda —en el Mac, todos— sale idéntico al de antes del sprint. ¿Puede fallar? Sí: sin el
    /// `skip_serializing_if`, el siguiente guardado añadiría `"desplazamientoDeLaBanda": 0` a todos los archivos.
    #[test]
    fn la_altura_de_la_banda_nace_en_0_y_el_0_no_se_escribe() {
        let p = de_texto(r#"{"version":1,"posicionDeLaBanda":"arriba"}"#).unwrap();
        assert_eq!(p.desplazamiento_de_la_banda, 0);
        let texto = a_texto(&p);
        assert!(!texto.contains("desplazamientoDeLaBanda"), "el 0 se escribió: {texto}");
        let bajada = a_texto(&Preferencias { desplazamiento_de_la_banda: 132, ..p });
        assert!(bajada.contains(r#""desplazamientoDeLaBanda": 132"#), "{bajada}");
        assert_eq!(de_texto(&bajada).unwrap().desplazamiento_de_la_banda, 132);
    }

    /// **El modo transparente se recuerda por cliente, y vacío no se escribe** (sprint 012, ADR 024): el archivo de quien no
    /// lo usa —el del Mac, entero— no cambia ni un byte, y uno de antes se lee con la lista vacía. ¿Puede fallar? Sí: sin el
    /// `skip_serializing_if`, el siguiente guardado añadiría `"transparentes": []` a todos los archivos.
    #[test]
    fn los_clientes_transparentes_se_recuerdan_y_vacio_no_se_escribe() {
        let p = de_texto(r#"{"version":1}"#).unwrap();
        assert!(p.transparentes.is_empty());
        let texto = a_texto(&p);
        assert!(!texto.contains("transparentes"), "la lista vacía se escribió: {texto}");
        let con = a_texto(&Preferencias { transparentes: ["Páramo Azul".to_string()].into(), ..p });
        assert!(con.contains("transparentes"), "{con}");
        assert!(de_texto(&con).unwrap().transparentes.contains("Páramo Azul"));
    }
}
