//! **Los atajos de teclado, por plataforma** (sprint 007, ADR 022 enmienda 1).
//!
//! Misma acción, otra combinación. En macOS son los de siempre: `⌥⎋` corta y `⌃⌥` + letra hace lo demás. En
//! Windows esa familia no sirve:
//! - **`Ctrl+Alt` es AltGr** en muchos teclados (español, alemán, francés…): `Ctrl+Alt+E` escribe «€», y un atajo
//!   global con `Ctrl+Alt` le robaría al usuario sus caracteres;
//! - **`Alt+Esc` es del sistema** (cambia de ventana).
//!
//! Se eligió **`Alt+Mayús` + letra** porque es la única familia de letras que no aparece en ninguna de las seis tablas
//! publicadas que se leyeron el 2026-10-11 —Windows 11, Microsoft Teams, Zoom Workplace, Chrome, Edge y Google
//! Meet— ni en lo que Windows reserva (la tecla Windows y F12). La tabla, con su fuente por fila, vive en el ADR 022.
//! Lo que choca fuera de esas seis tablas está dicho allí también (Word y Excel usan algunas de estas letras).
//!
//! Un atajo que Windows no deja registrar —porque otra app ya lo tiene— **se dice**: queda apuntado aquí y Honestidad
//! lo enseña. El usuario no puede pulsar una tecla creyendo que cortó y no haber cortado nada.

use serde::Serialize;
use std::sync::Mutex;
use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

/// Lo que hace cada atajo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Accion {
    Corte,
    Transcript,
    Ayuda,
    ModoDeVoz,
    LeerLaPantalla,
    Radar,
    Anotar,
    Fijar,
    GuardarLaPropuesta,
    Banda,
    /// `⎋` sola, **solo mientras el modo de voz está encendido**, en las dos plataformas.
    Callar,
}

/// Todas, en el orden en que se registran y se enseñan.
pub const TODAS: [Accion; 11] = [
    Accion::Corte,
    Accion::Transcript,
    Accion::Ayuda,
    Accion::ModoDeVoz,
    Accion::LeerLaPantalla,
    Accion::Radar,
    Accion::Anotar,
    Accion::Fijar,
    Accion::GuardarLaPropuesta,
    Accion::Banda,
    Accion::Callar,
];

/// Las teclas de una acción en una plataforma, y cómo se escriben para quien las lee.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Teclas {
    pub modificadores: Option<Modifiers>,
    pub tecla: Code,
    pub texto: &'static str,
}

const fn t(modificadores: Option<Modifiers>, tecla: Code, texto: &'static str) -> Teclas {
    Teclas { modificadores, tecla, texto }
}

const CTRL_OPCION: Option<Modifiers> = Some(Modifiers::CONTROL.union(Modifiers::ALT));
const ALT_MAYUS: Option<Modifiers> = Some(Modifiers::ALT.union(Modifiers::SHIFT));

/// **macOS**: los de siempre, sin cambiar uno.
pub fn en_macos(a: Accion) -> Teclas {
    match a {
        Accion::Corte => t(Some(Modifiers::ALT), Code::Escape, "⌥⎋"),
        Accion::Transcript => t(CTRL_OPCION, Code::KeyT, "⌃⌥T"),
        Accion::Ayuda => t(CTRL_OPCION, Code::KeyA, "⌃⌥A"),
        Accion::ModoDeVoz => t(CTRL_OPCION, Code::KeyV, "⌃⌥V"),
        Accion::LeerLaPantalla => t(CTRL_OPCION, Code::KeyL, "⌃⌥L"),
        Accion::Radar => t(CTRL_OPCION, Code::KeyR, "⌃⌥R"),
        Accion::Anotar => t(CTRL_OPCION, Code::KeyN, "⌃⌥N"),
        Accion::Fijar => t(CTRL_OPCION, Code::KeyP, "⌃⌥P"),
        Accion::GuardarLaPropuesta => t(CTRL_OPCION, Code::Enter, "⌃⌥↵"),
        Accion::Banda => t(CTRL_OPCION, Code::KeyB, "⌃⌥B"),
        Accion::Callar => t(None, Code::Escape, "⎋"),
    }
}

/// **Windows**: `Alt+Mayús` + una letra libre en las seis tablas publicadas (ADR 022, enmienda 1). El corte, con la
/// Q, que tampoco usan Word ni Excel. El radar llega a Windows en el S8: su tecla está apartada y no se registra.
pub fn en_windows(a: Accion) -> Teclas {
    match a {
        Accion::Corte => t(ALT_MAYUS, Code::KeyQ, "Alt+Shift+Q"),
        Accion::Transcript => t(ALT_MAYUS, Code::KeyC, "Alt+Shift+C"),
        Accion::Ayuda => t(ALT_MAYUS, Code::KeyH, "Alt+Shift+H"),
        Accion::ModoDeVoz => t(ALT_MAYUS, Code::KeyW, "Alt+Shift+W"),
        Accion::LeerLaPantalla => t(ALT_MAYUS, Code::KeyD, "Alt+Shift+D"),
        Accion::Radar => t(ALT_MAYUS, Code::KeyU, "Alt+Shift+U"),
        Accion::Anotar => t(ALT_MAYUS, Code::KeyK, "Alt+Shift+K"),
        Accion::Fijar => t(ALT_MAYUS, Code::KeyF, "Alt+Shift+F"),
        Accion::GuardarLaPropuesta => t(ALT_MAYUS, Code::KeyZ, "Alt+Shift+Z"),
        Accion::Banda => t(ALT_MAYUS, Code::KeyX, "Alt+Shift+X"),
        Accion::Callar => t(None, Code::Escape, "Esc"),
    }
}

/// Las de este equipo.
pub fn de_este_equipo(a: Accion) -> Teclas {
    if cfg!(windows) {
        en_windows(a)
    } else {
        en_macos(a)
    }
}

/// El atajo de este equipo, listo para registrar.
pub fn atajo(a: Accion) -> Shortcut {
    let t = de_este_equipo(a);
    Shortcut::new(t.modificadores, t.tecla)
}

/// ¿Se registra esta acción en este equipo? Todas, salvo el radar en Windows hasta el S8: registrar su tecla le
/// quitaría una combinación al usuario para abrir una pantalla que todavía dice «todavía no».
pub fn se_registra_aqui(a: Accion) -> bool {
    !(cfg!(windows) && a == Accion::Radar)
}

/// Las acciones que el sistema no dejó registrar en esta sesión. Solo acciones: ni teclas pulsadas ni nada más.
static FALLIDAS: Mutex<Vec<Accion>> = Mutex::new(Vec::new());

/// Apunta si una acción quedó registrada o no.
pub fn apuntar(a: Accion, registrada: bool) {
    if let Ok(mut f) = FALLIDAS.lock() {
        f.retain(|x| *x != a);
        if !registrada {
            f.push(a);
        }
    }
}

/// ¿Quedó registrada? (Lo que no se intentó aún cuenta como registrado: no hay nada que decir de ello.)
pub fn registrada(a: Accion) -> bool {
    FALLIDAS.lock().map_or(true, |f| !f.contains(&a))
}

/// Un atajo tal como lo enseña la webview.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Atajo {
    pub accion: Accion,
    /// Cómo se escribe: «⌃⌥T» en macOS, «Alt+Shift+C» en Windows (la webview dice «Mayús» en español).
    pub teclas: &'static str,
    /// El sistema lo dejó registrar. `false` es «otra app ya tiene esta combinación»: Honestidad lo dice.
    pub registrado: bool,
}

/// Los atajos de un sistema, para el contrato. En Windows no cruza el radar, que no se registra hasta el S8.
pub fn para_el_contrato(en_windows_: bool) -> Vec<Atajo> {
    TODAS
        .iter()
        .filter(|a| !(en_windows_ && **a == Accion::Radar))
        .map(|&a| Atajo {
            accion: a,
            teclas: if en_windows_ { en_windows(a).texto } else { en_macos(a).texto },
            registrado: registrada(a),
        })
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **En Windows ningún atajo usa `Ctrl+Alt`** (AltGr) **ni la tecla Windows**, y ninguno se repite.
    #[test]
    fn en_windows_ninguno_es_altgr_ni_se_repite() {
        let mut vistas = Vec::new();
        for a in TODAS {
            let w = en_windows(a);
            let m = w.modificadores.unwrap_or(Modifiers::empty());
            assert!(!(m.contains(Modifiers::CONTROL) && m.contains(Modifiers::ALT)), "{a:?} es AltGr en muchos teclados");
            assert!(!m.contains(Modifiers::SUPER), "{a:?} usa la tecla Windows, que es del sistema");
            assert!(!(m.contains(Modifiers::ALT) && w.tecla == Code::Escape), "{a:?} es Alt+Esc, que es del sistema");
            assert!(w.tecla != Code::F12, "{a:?}: F12 es del depurador");
            assert!(!vistas.contains(&(w.modificadores, w.tecla)), "{a:?} repite combinación");
            vistas.push((w.modificadores, w.tecla));
        }
    }

    /// **Ninguna combinación de Windows está en las tablas publicadas** que se leyeron (ADR 022, enmienda 1): las
    /// letras de `Alt+Mayús` que usan Windows 11, Teams, Zoom, Chrome, Edge y Meet, copiadas de sus páginas.
    #[test]
    fn en_windows_ninguna_choca_con_las_tablas_publicadas() {
        // Alt+Shift+letra en las seis tablas (2026-10-11): Windows 11 P · Teams L M R O E A S V J Y · Zoom S I R G E
        // T · Chrome I A T N · Edge B I T. Meet no usa Alt+Shift.
        let usadas = "PLMROEASVJYIGTNB";
        for a in TODAS.iter().filter(|a| **a != Accion::Callar) {
            let w = en_windows(*a);
            assert_eq!(w.modificadores, ALT_MAYUS, "{a:?} fuera de la familia");
            let letra = w.texto.chars().last().unwrap();
            assert!(!usadas.contains(letra), "{a:?}: Alt+Shift+{letra} está en una tabla publicada");
            assert_eq!(w.texto, format!("Alt+Shift+{letra}"), "el texto dice lo que se registra");
        }
    }

    /// **macOS no cambia**: el corte es `⌥⎋` y lo demás `⌃⌥`.
    #[test]
    fn en_macos_son_los_de_siempre() {
        assert_eq!(en_macos(Accion::Corte).modificadores, Some(Modifiers::ALT));
        assert_eq!(en_macos(Accion::Corte).tecla, Code::Escape);
        for a in TODAS.iter().filter(|a| !matches!(a, Accion::Corte | Accion::Callar)) {
            assert_eq!(en_macos(*a).modificadores, CTRL_OPCION, "{a:?}");
        }
        assert_eq!(en_macos(Accion::Callar), t(None, Code::Escape, "⎋"));
    }

    /// Lo que no se pudo registrar se dice, y deja de decirse al registrarse.
    #[test]
    fn un_atajo_que_no_se_registra_se_dice() {
        apuntar(Accion::Banda, false);
        assert!(!registrada(Accion::Banda));
        assert!(para_el_contrato(true).iter().any(|x| x.accion == Accion::Banda && !x.registrado));
        apuntar(Accion::Banda, true);
        assert!(registrada(Accion::Banda));
        assert!(!para_el_contrato(true).iter().any(|x| x.accion == Accion::Radar), "el radar no cruza en Windows");
        assert_eq!(para_el_contrato(false).len(), TODAS.len());
    }
}
