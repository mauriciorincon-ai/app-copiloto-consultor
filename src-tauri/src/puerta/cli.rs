//! LA LÍNEA DE ÓRDENES DE `ghost` (ADR 018 §8): de los argumentos a una [`Orden`]. Pura: leer el
//! archivo del kit, mirar el socket, leer el Llavero e imprimir lo hace `src/bin/ghost.rs`.
//!
//! En español, como la app, y con los alias en inglés: quien la usa es un agente que lee la ayuda, y
//! la ayuda sale en el idioma del Mac.

use std::path::PathBuf;

use super::{Clave, Orden};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Los códigos de salida: lo que un agente mira antes de leer el JSON.
pub mod salida {
    pub const HECHO: i32 = 0;
    pub const FALLO: i32 = 1;
    pub const DENEGADO: i32 = 2;
    pub const CERRADA: i32 = 3;
    pub const USO: i32 = 64;
}

/// Lo que pidieron en la línea de órdenes.
#[derive(Debug, PartialEq)]
pub enum Pedido {
    Version,
    Ayuda,
    Orden(Orden),
    /// El kit: `ghost` lee el archivo y arma la orden con las preguntas.
    Kit(PathBuf),
}

/// De los argumentos (sin el nombre del programa) a lo que se pide.
pub fn interpretar(args: &[String]) -> Result<Pedido, String> {
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let pedido = match a.as_slice() {
        [] | ["--help" | "-h" | "ayuda" | "help"] => Pedido::Ayuda,
        ["--version" | "-V"] => Pedido::Version,
        ["corpus", "buscar" | "search", texto @ ..] if !texto.is_empty() => {
            Pedido::Orden(Orden::Buscar { texto: texto.join(" ") })
        }
        ["corpus", "reindexar" | "reindex"] => Pedido::Orden(Orden::Reindexar {}),
        ["kit", archivo] => Pedido::Kit(PathBuf::from(archivo)),
        ["prefs", "leer" | "get"] => Pedido::Orden(Orden::LeerPrefs {}),
        ["prefs", "cambiar" | "set", clave, valor] => {
            Pedido::Orden(Orden::CambiarPref { clave: clave.to_string(), valor: valor.to_string() })
        }
        ["notas" | "notes", "listar" | "list"] => Pedido::Orden(Orden::ListarNotas {}),
        ["notas" | "notes", "abrir" | "open", archivo] => Pedido::Orden(Orden::AbrirNota { archivo: archivo.to_string() }),
        ["ia" | "ai", "--encender-api" | "--enable-api"] => Pedido::Orden(Orden::EncenderApi {}),
        _ => return Err(format!("no conozco «ghost {}»: mira ghost --help", a.join(" "))),
    };
    Ok(pedido)
}

/// El idioma de la ayuda: el del Mac (`LANG`), inglés si empieza por `en`, español si no.
pub fn idioma(lang: Option<&str>) -> &'static str {
    if lang.is_some_and(|l| l.starts_with("en")) {
        "en"
    } else {
        "es"
    }
}

/// El idioma de la ayuda según el entorno: `LANG` (el del Mac) y, si no dice nada —en Windows casi nunca lo dice—, el
/// idioma del usuario del sistema (auditoría del S12, M5). En el Mac, `LANG` y nada más, como siempre.
pub fn idioma_del_entorno() -> &'static str {
    let lang = std::env::var("LANG").ok().filter(|v| !v.is_empty());
    idioma(lang.or_else(idioma_del_sistema).as_deref())
}

#[cfg(windows)]
fn idioma_del_sistema() -> Option<String> {
    use windows_sys::Win32::Globalization::GetUserDefaultLocaleName;
    let mut nombre = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH
    // SEGURIDAD: un búfer propio con su largo; Windows escribe el nombre terminado en cero y devuelve cuántos caracteres.
    let n = unsafe { GetUserDefaultLocaleName(nombre.as_mut_ptr(), nombre.len() as i32) };
    (n > 1).then(|| String::from_utf16_lossy(&nombre[..n as usize - 1]))
}

#[cfg(not(windows))]
fn idioma_del_sistema() -> Option<String> {
    None
}

/// Las dos frases de la ayuda que hablan del sistema: quién pide la identidad al abrir una nota y quién guarda la llave.
/// En el Mac, Touch ID y el Llavero; en Windows, Windows Hello y el Administrador de credenciales (ADR 022 enmienda 4).
fn del_sistema(en: bool) -> (&'static str, &'static str) {
    match (cfg!(windows), en) {
        (false, true) => (
            "asks for Touch ID on the Mac, once per door opening",
            "Each time you open the door, macOS asks you whether ghost may use the key in your Keychain.",
        ),
        (false, false) => (
            "pide Touch ID en el Mac, una vez por apertura",
            "Cada vez que abres la puerta, macOS te pregunta si ghost puede usar la llave de tu Llavero.",
        ),
        (true, true) => (
            "asks for Windows Hello, once per door opening",
            "Each time you open the door, the app leaves a new key in Credential Manager and ghost reads it from there.",
        ),
        (true, false) => (
            "pide Windows Hello, una vez por apertura",
            "Cada vez que abres la puerta, la app deja una llave nueva en el Administrador de credenciales y ghost la lee de ahí.",
        ),
    }
}

/// Las claves que `ghost prefs cambiar` acepta.
fn delegables() -> String {
    Clave::TODAS.into_iter().filter(|c| c.delegable()).map(Clave::nombre).collect::<Vec<_>>().join(" · ")
}

pub fn ayuda(idioma: &str) -> String {
    let claves = delegables();
    let (abre_nota, la_llave) = del_sistema(idioma == "en");
    if idioma == "en" {
        format!(
            "ghost {VERSION} — Angel Ghost's local door (ADR 018)

Usage:
  ghost corpus search <text>          cards from your corpus
  ghost corpus reindex                re-read your folder, retry unreadable documents
  ghost kit <questions.json>          nDCG@5 and what it cannot find, against your corpus
  ghost prefs get
  ghost prefs set <key> <value>       {claves}
  ghost notes list
  ghost notes open <file>             {abre_nota}
  ghost --version · ghost --help

You open the door by hand, in Angel Ghost → AI. In a meeting or during a rehearsal it closes itself and denies everything.
{la_llave}
Output: JSON. Exit code: 0 done · 2 denied · 3 door closed · 1 failed · 64 usage.
"
        )
    } else {
        format!(
            "ghost {VERSION} — la puerta local de Angel Ghost (ADR 018)

Uso:
  ghost corpus buscar <texto>         las fichas de tu corpus
  ghost corpus reindexar              vuelve a leer tu carpeta y reintenta los ilegibles
  ghost kit <preguntas.json>          nDCG@5 y lo que no encuentra, contra tu corpus
  ghost prefs leer
  ghost prefs cambiar <clave> <valor> {claves}
  ghost notas listar
  ghost notas abrir <archivo>         {abre_nota}
  ghost --version · ghost --help

La puerta se abre a mano, en Angel Ghost → IA. En reunión o durante un ensayo se cierra sola y lo deniega todo.
{la_llave}
Salida: JSON. Código: 0 hecho · 2 denegado · 3 puerta cerrada · 1 fallo · 64 uso.
"
        )
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn de(linea: &str) -> Result<Pedido, String> {
        interpretar(&linea.split_whitespace().map(String::from).collect::<Vec<_>>())
    }

    /// **La ayuda dice cuándo pregunta macOS: en cada apertura** (auditoría del S3, B12 y M4). Decía «la
    /// primera vez», y el diálogo de la lista de acceso sale cada vez que abres la puerta, porque el
    /// token es nuevo; y el Touch ID de las notas, una vez por apertura. Demostrado en rojo con la ayuda
    /// de antes.
    #[test]
    fn la_ayuda_dice_que_pregunta_en_cada_apertura() {
        for (idioma, cada, una_vez) in [("es", "Cada vez que abres la puerta", "una vez por apertura"), ("en", "Each time you open the door", "once per door opening")] {
            let a = ayuda(idioma);
            assert!(!a.contains("La primera vez") && !a.contains("The first time"), "{idioma}: la ayuda dice «la primera vez»");
            assert!(a.contains(cada) && a.contains(una_vez), "{idioma}: la ayuda no dice cuándo pregunta:\n{a}");
        }
    }

    /// **En Windows la ayuda no habla del Mac** (auditoría del S12, M5): Windows Hello y el Administrador de credenciales
    /// en lugar de Touch ID y el Llavero, y el idioma del usuario cuando `LANG` no dice nada. En el Mac, el texto de
    /// siempre. ¿Puede fallar? Sí: quitar el `cfg!(windows)` de `del_sistema` es rojo en un sistema o en el otro.
    #[test]
    fn la_ayuda_habla_del_sistema_en_el_que_corre() {
        for idioma in ["es", "en"] {
            let a = ayuda(idioma);
            let del_mac = ["Touch ID", "macOS", "Llavero", "Keychain"].iter().any(|m| a.contains(m));
            if cfg!(windows) {
                assert!(!del_mac, "{idioma}: la ayuda de Windows nombra el Mac:\n{a}");
                assert!(a.contains("Windows Hello") && (a.contains("Credential Manager") || a.contains("Administrador de credenciales")));
            } else {
                assert!(del_mac && !a.contains("Windows Hello"), "{idioma}: la ayuda del Mac cambió");
            }
        }
        #[cfg(windows)]
        assert!(idioma_del_sistema().is_some_and(|i| i.len() >= 2), "Windows no dijo el idioma del usuario");
        #[cfg(not(windows))]
        assert!(idioma_del_sistema().is_none(), "en el Mac el idioma sale solo de LANG");
    }

    #[test]
    fn cada_orden_en_los_dos_idiomas() {
        assert_eq!(de("corpus buscar limpieza de datos"), Ok(Pedido::Orden(Orden::Buscar { texto: "limpieza de datos".into() })));
        assert_eq!(de("corpus search scope"), Ok(Pedido::Orden(Orden::Buscar { texto: "scope".into() })));
        assert_eq!(de("corpus reindexar"), de("corpus reindex"));
        assert_eq!(de("kit p.json"), Ok(Pedido::Kit(PathBuf::from("p.json"))));
        assert_eq!(de("prefs leer"), de("prefs get"));
        assert_eq!(
            de("prefs cambiar retencion 30d"),
            Ok(Pedido::Orden(Orden::CambiarPref { clave: "retencion".into(), valor: "30d".into() }))
        );
        assert_eq!(de("notas listar"), de("notes list"));
        assert_eq!(de("notas abrir a.ghost"), Ok(Pedido::Orden(Orden::AbrirNota { archivo: "a.ghost".into() })));
        assert_eq!(de("ia --encender-api"), Ok(Pedido::Orden(Orden::EncenderApi {})));
        assert_eq!(de("--version"), Ok(Pedido::Version));
        assert_eq!(de(""), Ok(Pedido::Ayuda));
    }

    #[test]
    fn lo_que_no_esta_se_rechaza_con_la_ayuda_a_mano() {
        for linea in ["corpus buscar", "corpus reindexar --unidad caso", "borrar todo", "prefs cambiar retencion", "kit"] {
            let e = de(linea).unwrap_err();
            assert!(e.contains("ghost --help"), "{linea}: {e}");
        }
    }

    #[test]
    fn la_ayuda_sale_en_el_idioma_del_mac_y_nombra_solo_lo_delegable() {
        assert_eq!(idioma(Some("en_US.UTF-8")), "en");
        assert_eq!(idioma(Some("es_CO.UTF-8")), "es");
        assert_eq!(idioma(None), "es");
        let es = ayuda("es");
        assert!(es.contains("ghost corpus buscar") && es.contains("retencion"));
        assert!(ayuda("en").contains("ghost corpus search"));
        for no in ["redactar", "conservar-mis-turnos", "proveedor"] {
            assert!(!es.contains(no), "la ayuda ofrece «{no}», que no es delegable");
        }
    }
}
