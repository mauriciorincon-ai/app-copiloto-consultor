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

/// Las claves que `ghost prefs cambiar` acepta.
fn delegables() -> String {
    Clave::TODAS.into_iter().filter(|c| c.delegable()).map(Clave::nombre).collect::<Vec<_>>().join(" · ")
}

pub fn ayuda(idioma: &str) -> String {
    let claves = delegables();
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
  ghost notes open <file>             asks for Touch ID on the Mac
  ghost --version · ghost --help

You open the door by hand, in Angel Ghost → AI. In a meeting it closes itself and denies everything.
The first time, macOS asks you whether ghost may use the key in your Keychain.
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
  ghost notas abrir <archivo>         pide Touch ID en el Mac
  ghost --version · ghost --help

La puerta se abre a mano, en Angel Ghost → IA. En reunión se cierra sola y lo deniega todo.
La primera vez, macOS te pregunta si ghost puede usar la llave de tu Llavero.
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
