//! EL MODO DE LA REUNIÓN — normal, o solo notas (C11, ADR 017 §5).
//!
//! Una sola pregunta, y pura para poder demostrarla en rojo: **¿este modo abre la captura?** La
//! captura es todo lo que oye o mira la reunión: micrófono, audio del sistema, transcripción, lectura
//! de pantalla (la automática y `⌃⌥L`) y el radar ámbar, que lee la ventana de la reunión. `empezar`
//! (en `lib.rs`) la obedece antes de arrancar nada, y un test de su fuente vigila que la obedezca.
//!
//! En **solo notas** no se abre nada de eso, ni una vez: la NDA de tu cliente puede prohibir
//! «grabar o transcribir por cualquier medio», o puedes decidirlo tú. Siguen tus notas, fijar, `⌃⌥A`
//! (que busca con la última línea de tu nota), el radar coral y `⌥⎋`.

/// No cruza a la pantalla: el evento `modo` es una señal sin dato (auditoría del S3, B14).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modo {
    Normal,
    SoloNotas,
}

/// ¿Abre este modo la captura?
pub fn abre_la_captura(modo: Modo) -> bool {
    match modo {
        Modo::Normal => true,
        Modo::SoloNotas => false,
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **Solo notas no abre la captura.** ¿Puede fallar? Sí: con `SoloNotas => true` es rojo
    /// (bitácora), y `lib.rs` arrancaría micrófono, sistema y pantalla en una reunión cuya NDA lo
    /// prohíbe.
    #[test]
    fn solo_notas_no_abre_la_captura() {
        assert!(!abre_la_captura(Modo::SoloNotas));
        assert!(abre_la_captura(Modo::Normal));
    }
}
