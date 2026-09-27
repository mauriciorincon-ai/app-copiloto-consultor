//! EL DESBLOQUEO DE TUS NOTAS GUARDADAS (ADR 015 §5).
//!
//! Abrir o exportar una reunión guardada pide Touch ID o la contraseña del Mac **una vez por sesión
//! de la app**; al salir se olvida. Guardar no lo pide: al cerrar una reunión no hay que pararse.
//!
//! Qué protege: a alguien frente a tu Mac desbloqueado que abre Angel Ghost para leer tus reuniones.

use std::sync::atomic::{AtomicBool, Ordering};

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
    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    {
        let _ = razon;
        Respuesta::NoSePuede
    }
}

/// Lo que dice el diálogo debajo del nombre de la app, en el idioma de la interfaz.
pub fn razon(idioma: &str) -> &'static str {
    if idioma.starts_with("en") { "open your saved notes" } else { "abrir tus notas guardadas" }
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
            Respuesta::NoSePuede => Err(
                "este Mac no tiene con qué pedir el desbloqueo: tu cuenta necesita una contraseña para abrir tus notas".into(),
            ),
        }
    }

    pub fn desbloqueado(&self) -> bool {
        self.hecho.load(Ordering::Relaxed)
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
        assert!(d.asegurar(|| Respuesta::NoSePuede).unwrap_err().contains("contraseña"));
        assert!(d.asegurar(|| Respuesta::Hecho).is_ok());
    }

    #[test]
    fn el_dialogo_habla_el_idioma_de_la_interfaz() {
        assert_eq!(razon("es"), "abrir tus notas guardadas");
        assert_eq!(razon("en"), "open your saved notes");
    }
}
