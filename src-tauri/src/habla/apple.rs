//! El lado de Rust del puente hacia `AVSpeechSynthesizer`. **Aquí vive todo el `unsafe` de
//! `habla/`**, igual que `stt/apple.rs` concentra el de la transcripción y `acople/ax.rs` el del
//! acople: una sola puerta, vigilada, y el resto del módulo en Rust seguro.
//!
//! Las cuatro funciones que cruzan la frontera están declaradas abajo tal y como las expone
//! `nativo/Habla.swift`. Ninguna deja punteros vivos tras la llamada: el texto entra copiado a un
//! `CString` y lo que vuelve es un entero.
//!
//! Si la librería de Swift no se compiló (`swiftc` ausente), este archivo no declara nada y
//! [`voz`] devuelve la muda con ese motivo. Es la misma asimetría de `build.rs`: sin herramientas
//! de desarrollo la app funciona y pierde el modo; con ellas y un error de Swift, no compila.

use super::Voz;

#[cfg(all(target_os = "macos", puente_de_swift))]
mod puente {
    use std::os::raw::{c_char, c_int};

    #[link(name = "agstt", kind = "static")]
    extern "C" {
        pub fn ag_habla_voz(idioma: *const c_char) -> c_int;
        pub fn ag_habla_decir(idioma: *const c_char, texto: *const c_char) -> c_int;
        pub fn ag_habla_callar() -> c_int;
        pub fn ag_habla_hablando() -> c_int;
    }
}

/// La voz del sistema, cuando hay alguna.
#[cfg(all(target_os = "macos", puente_de_swift))]
pub struct DelSistema;

#[cfg(all(target_os = "macos", puente_de_swift))]
impl Voz for DelSistema {
    fn nombre(&self) -> &'static str {
        "apple-avspeechsynthesizer"
    }

    fn hay_para(&self, idioma: &str) -> bool {
        let Ok(c) = std::ffi::CString::new(idioma) else { return false };
        // Los paréntesis no son de adorno: un bloque `unsafe` en posición de sentencia se
        // parsea como sentencia y el `==` de después queda huérfano.
        (unsafe { puente::ag_habla_voz(c.as_ptr()) }) == 1
    }

    /// **El texto se comprueba aquí, no allá.** Un `\0` dentro de la cadena no puede cruzar a C, y
    /// dejar que `CString::new` reviente en la llamada sería un pánico en mitad de una reunión por
    /// un carácter que nadie escribió a mano. La ficha sale del corpus del usuario: un PDF mal leído
    /// puede traer cualquier byte.
    fn decir(&self, idioma: &str, texto: &str) -> Result<(), String> {
        let ci = std::ffi::CString::new(idioma)
            .map_err(|_| format!("el idioma «{idioma}» lleva un cero dentro"))?;
        // Los ceros se quitan en vez de rechazar la ficha entera: el usuario pidió que se la
        // leyeran, y callarse por un byte raro en un PDF sería el silencio que esta casa no admite.
        let limpio: String = texto.chars().filter(|c| *c != '\0').collect();
        let ct = std::ffi::CString::new(limpio)
            .map_err(|_| "la ficha no se pudo preparar para decirla".to_string())?;
        match unsafe { puente::ag_habla_decir(ci.as_ptr(), ct.as_ptr()) } {
            n if n > 0 => Ok(()),
            -1 => Err(format!("este Mac no tiene voz para «{idioma}»")),
            -2 => Err("no hay nada que decir".into()),
            otro => Err(format!("el sintetizador devolvió {otro}")),
        }
    }

    fn callar(&self) {
        unsafe { puente::ag_habla_callar() };
    }

    fn hablando(&self) -> bool {
        (unsafe { puente::ag_habla_hablando() }) == 1
    }
}

/// La voz de este Mac, o la muda con su razón.
pub fn voz() -> Box<dyn Voz> {
    #[cfg(all(target_os = "macos", puente_de_swift))]
    {
        // **No se pregunta «hay voz» al arrancar, y eso es a propósito.** El idioma con el que se va
        // a leer lo elige el usuario en la pantalla de Idioma y puede cambiar sin reiniciar; una voz
        // que se declarara muda al arrancar porque en ese momento no había voz para el español se
        // quedaría muda para siempre. La pregunta se hace por idioma, cada vez, en `hay_para`.
        Box::new(DelSistema)
    }
    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    {
        Box::new(super::Muda::por(
            "la app se compiló sin el puente de voz (falta swiftc o no es macOS)",
        ))
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **Un idioma con un cero dentro no cruza la frontera.** Se rechaza aquí, con nombre, en vez
    /// de dejar que `CString::new` reviente dentro de la llamada. Hermano del test que `stt/apple.rs`
    /// ya tiene: la misma clase de defecto, la misma puerta.
    #[test]
    fn un_idioma_con_un_cero_dentro_no_cruza_la_frontera() {
        let v = voz();
        assert!(!v.hay_para("es\0ES"));
        assert!(v.decir("es\0ES", "algo").is_err());
    }

    /// **Y una ficha con un cero dentro SÍ se dice**, sin el cero. La asimetría con el idioma es
    /// deliberada: un idioma inválido es un error de la app, y una ficha con un byte raro es un PDF
    /// del usuario. Lo primero se denuncia; lo segundo se limpia y se lee.
    ///
    /// Aquí solo se comprueba que no se rompe: si este Mac no tiene voz, el error es «no tiene voz»,
    /// que es otra cosa que un pánico.
    #[test]
    fn una_ficha_con_un_cero_dentro_no_rompe_nada() {
        let v = voz();
        let r = v.decir("es-ES", "Alcance\0 incluido");
        if let Err(e) = &r {
            assert!(!e.contains("cero dentro"), "la ficha se rechazó por el cero: {e}");
        }
        v.callar();
    }

    /// Callar sin haber hablado no puede romperse: es lo que hace `⌥⎋` cuando el modo está apagado,
    /// y el kill-switch no puede tener un camino que falle.
    #[test]
    fn callar_sin_haber_hablado_no_hace_nada_malo() {
        let v = voz();
        v.callar();
        v.callar();
        assert!(!v.hablando());
    }
}
