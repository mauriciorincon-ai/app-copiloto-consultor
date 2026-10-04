//! **El centinela de la regla 25** (kit v1.36.0, «el comando de pruebas por defecto no toca
//! hardware»; en esta app vive dentro de la regla 22, «las protecciones del Mac se enseñan ANTES de
//! tocarlas»).
//!
//! Cada entrada nativa que abre algo que macOS protege o que el usuario nota —el micrófono, el
//! audio del sistema, los altavoces, el reconocimiento de voz, la captura de pantalla, escribir en
//! una ventana ajena, el aviso de Accessibility, el Llavero, el desbloqueo, launchd— llama a
//! [`vigilar`] ANTES de cruzar a C. Con [`VARIABLE`] puesta, la llamada **aborta el proceso** con
//! el nombre de lo que se iba a abrir.
//!
//! Por qué abortar y no `panic!`: la mitad de esas entradas corre en un hilo de fondo (la escucha
//! abre sus grifos fuera del hilo del test), y un pánico en un hilo ajeno deja el test en verde.
//! Un aborto tumba el binario de pruebas entero, y `cargo test` sale en rojo con el nombre delante.
//!
//! La CI corre `cargo test` a secas con la variable puesta: si un test que abre el micrófono pierde
//! su `#[ignore = "hardware: …"]`, el paso cae en el acto, antes de tocar nada. En la app de
//! verdad la variable no existe y esto es una lectura de entorno por cada apertura.

/// El nombre de la variable que arma el centinela.
pub const VARIABLE: &str = "AG_SIN_HARDWARE";

/// ¿Está armado? Vacía o «0» cuentan como desarmado, para que nadie lo arme sin querer con un
/// `AG_SIN_HARDWARE=` de más en un script.
pub fn armado(valor: Option<&std::ffi::OsStr>) -> bool {
    valor.is_some_and(|v| !v.is_empty() && v != "0")
}

/// Se llama en la primera línea de cada entrada nativa, antes de cruzar a C.
pub fn vigilar(que: &'static str) {
    if armado(std::env::var_os(VARIABLE).as_deref()) {
        eprintln!(
            "[hardware] {que}: se iba a abrir con {VARIABLE} puesta. Un `cargo test` a secas no \
             toca el Mac (regla 25): ese test lleva #[ignore = \"hardware: …\"] y lo corre la CI \
             con --include-ignored."
        );
        std::process::abort();
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn sin_la_variable_el_centinela_no_existe() {
        assert!(!armado(None));
    }

    #[test]
    fn vacia_o_cero_no_lo_arman() {
        assert!(!armado(Some(OsStr::new(""))));
        assert!(!armado(Some(OsStr::new("0"))));
    }

    #[test]
    fn cualquier_otro_valor_lo_arma() {
        assert!(armado(Some(OsStr::new("1"))));
        assert!(armado(Some(OsStr::new("si"))));
    }
}
