//! EL DESBLOQUEO DE TUS NOTAS GUARDADAS (ADR 015 §5).
//!
//! Exportar una reunión guardada, o leer una bandeja de otra sesión, pide Touch ID o la contraseña
//! del Mac **una vez por sesión de la app**; al salir se olvida. La puerta local lleva su propio
//! desbloqueo y lo pide **una vez por apertura** (ADR 018, enmienda 1). Guardar no lo pide: al
//! cerrar una reunión no hay que pararse.
//!
//! Qué protege: a alguien frente a tu Mac desbloqueado que abre Angel Ghost para leer tus reuniones.
//!
//! **En Windows (sprint 008, ADR 022): Windows Hello**, con tu PIN, tu huella o tu cara (`desbloqueo/windows.rs`).

use std::sync::atomic::{AtomicBool, Ordering};

/// En Windows, el desbloqueo es Windows Hello.
#[cfg(windows)]
#[path = "desbloqueo/windows.rs"]
pub mod windows;

/// Lo que contestó el sistema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Respuesta {
    Hecho,
    /// Cancelaste, o no se reconoció.
    NoQuiso,
    /// Este Mac no tiene con qué pedirlo (una cuenta sin contraseña).
    NoSePuede,
}

#[cfg(all(target_os = "macos", puente_de_swift))]
mod puente {
    use std::os::raw::{c_char, c_int};
    extern "C" {
        pub fn ag_desbloquear(razon: *const c_char) -> c_int;
    }
}

/// Pide el desbloqueo con el diálogo del sistema. Bloquea hasta que contestas.
pub fn pedir(razon: &str) -> Respuesta {
    crate::hardware::vigilar("el desbloqueo (Touch ID, contraseña o Windows Hello)");
    #[cfg(all(target_os = "macos", puente_de_swift))]
    {
        let Ok(r) = std::ffi::CString::new(razon) else { return Respuesta::NoSePuede };
        // SEGURIDAD: un texto terminado en cero que vive hasta que la llamada vuelve.
        match unsafe { puente::ag_desbloquear(r.as_ptr()) } {
            1 => Respuesta::Hecho,
            0 => Respuesta::NoQuiso,
            _ => Respuesta::NoSePuede,
        }
    }
    #[cfg(windows)]
    {
        windows::pedir(razon)
    }
    #[cfg(not(any(all(target_os = "macos", puente_de_swift), windows)))]
    {
        let _ = razon;
        Respuesta::NoSePuede
    }
}

/// **Lo que contestó Windows Hello, en la respuesta de la app** (`UserConsentVerificationResult`). Pura, para que su
/// prueba corra también en el Mac: 0 verificado · 6 cancelado · 5 demasiados intentos, que es un «no» tuyo y no un
/// fallo del PC · lo demás (sin dispositivo, sin configurar, desactivado por directiva, ocupado) es que este PC no
/// tiene con qué pedirlo.
#[cfg_attr(not(windows), allow(dead_code))]
pub(crate) fn de_windows_hello(resultado: i32) -> Respuesta {
    match resultado {
        0 => Respuesta::Hecho,
        5 | 6 => Respuesta::NoQuiso,
        _ => Respuesta::NoSePuede,
    }
}

/// Lo que dice el diálogo debajo del nombre de la app, en el idioma de la interfaz.
pub fn razon(idioma: &str) -> &'static str {
    if idioma.starts_with("en") { "open your saved notes" } else { "abrir tus notas guardadas" }
}

/// Para qué se pide el desbloqueo cuando lo que se abre son tus ensayos (ADR 015, enmienda 4): el mismo
/// desbloqueo que tus notas, y macOS dice para qué.
pub fn razon_de_los_ensayos(idioma: &str) -> &'static str {
    if idioma.starts_with("en") { "open your saved rehearsals" } else { "abrir tus ensayos guardados" }
}

/// El recuerdo del desbloqueo mientras la app está abierta.
#[derive(Default)]
pub struct Desbloqueo {
    hecho: AtomicBool,
}

impl Desbloqueo {
    /// Si ya se desbloqueó en esta sesión de la app, no vuelve a preguntar; si no, pregunta con `pedir`.
    pub fn asegurar(&self, pedir: impl FnOnce() -> Respuesta) -> Result<(), String> {
        if self.hecho.load(Ordering::Relaxed) {
            return Ok(());
        }
        match pedir() {
            Respuesta::Hecho => {
                self.hecho.store(true, Ordering::Relaxed);
                Ok(())
            }
            Respuesta::NoQuiso => Err("no se desbloqueó: tus notas siguen cerradas".into()),
            #[cfg(not(windows))]
            Respuesta::NoSePuede => Err(
                "este Mac no tiene con qué pedir el desbloqueo: tu cuenta necesita una contraseña para abrir tus notas".into(),
            ),
            // Sprint 008 (auditoría del S6, M9): sin Windows Hello configurado no hay con qué pedirlo, y se dice dónde
            // se configura. Sin fecha prometida: el desbloqueo ya está.
            #[cfg(windows)]
            Respuesta::NoSePuede => Err(
                "este PC no tiene Windows Hello configurado: añade un PIN, tu huella o tu cara en Configuración → Cuentas → Opciones de inicio de sesión para abrir tus notas".into(),
            ),
        }
    }

    pub fn desbloqueado(&self) -> bool {
        self.hecho.load(Ordering::Relaxed)
    }

    /// Olvida el desbloqueo: la próxima vez vuelve a preguntar. Lo usa la puerta al abrirse y al
    /// cerrarse, para pedirlo **una vez por apertura** (auditoría del S3, M4).
    pub fn olvidar(&self) {
        self.hecho.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::cell::Cell;

    #[test]
    fn se_pide_una_vez_por_sesion_de_la_app() {
        let d = Desbloqueo::default();
        let veces = Cell::new(0);
        let pedir = || {
            veces.set(veces.get() + 1);
            Respuesta::Hecho
        };
        d.asegurar(pedir).unwrap();
        d.asegurar(|| {
            veces.set(veces.get() + 1);
            Respuesta::Hecho
        })
        .unwrap();
        assert_eq!(veces.get(), 1, "se volvió a pedir Touch ID dentro de la misma sesión de la app");
    }

    #[test]
    fn cancelar_no_abre_y_la_siguiente_vuelve_a_preguntar() {
        let d = Desbloqueo::default();
        assert!(d.asegurar(|| Respuesta::NoQuiso).is_err());
        assert!(!d.desbloqueado());
        // El «no se puede» dice qué falta: la contraseña de la cuenta en macOS; Windows Hello configurado, en Windows. Y
        // no promete ningún sprint (auditoría del S6, M9).
        let no = d.asegurar(|| Respuesta::NoSePuede).unwrap_err();
        assert!(no.contains(if cfg!(windows) { "Windows Hello" } else { "contraseña" }), "{no}");
        assert!(!no.to_lowercase().contains("sprint"), "el usuario no sabe qué es un sprint: {no}");
        assert!(d.asegurar(|| Respuesta::Hecho).is_ok());
    }

    /// **La puerta pide el suyo, una vez por apertura** (auditoría del S3, M4). El desbloqueo de la
    /// pantalla —exportar esta mañana— no abre tus reuniones a tu agente esta tarde: la puerta lleva
    /// su propio `Desbloqueo`, y al abrirse lo olvida. Demostrado en rojo con `olvidar` vacía: tras
    /// olvidar, no volvía a preguntar.
    #[test]
    fn la_puerta_pide_el_suyo_y_al_olvidarlo_vuelve_a_pedir() {
        let (pantalla, puerta) = (Desbloqueo::default(), Desbloqueo::default());
        let veces = Cell::new(0);
        let pedir = || {
            veces.set(veces.get() + 1);
            Respuesta::Hecho
        };
        pantalla.asegurar(pedir).unwrap();
        puerta.asegurar(pedir).unwrap();
        assert_eq!(veces.get(), 2, "el desbloqueo de la pantalla abrió la puerta sin preguntar");
        puerta.asegurar(pedir).unwrap();
        assert_eq!(veces.get(), 2, "dentro de la misma apertura, se volvió a preguntar");
        puerta.olvidar();
        assert!(!puerta.desbloqueado());
        puerta.asegurar(pedir).unwrap();
        assert_eq!(veces.get(), 3, "en la apertura siguiente no se volvió a preguntar");
        assert!(pantalla.desbloqueado(), "olvidar el de la puerta tocó el de la pantalla");
    }

    /// Windows Hello: verificado abre; cancelar o agotar los intentos es tu «no»; sin dispositivo, sin configurar,
    /// por directiva u ocupado, este PC no tiene con qué pedirlo.
    #[test]
    fn lo_que_dice_windows_hello_es_una_de_las_tres_respuestas() {
        assert_eq!(de_windows_hello(0), Respuesta::Hecho);
        assert_eq!(de_windows_hello(6), Respuesta::NoQuiso);
        assert_eq!(de_windows_hello(5), Respuesta::NoQuiso);
        for r in [1, 2, 3, 4, 99] {
            assert_eq!(de_windows_hello(r), Respuesta::NoSePuede, "{r}");
        }
    }

    #[test]
    fn el_dialogo_habla_el_idioma_de_la_interfaz() {
        assert_eq!(razon("es"), "abrir tus notas guardadas");
        assert_eq!(razon("en"), "open your saved notes");
    }
}
