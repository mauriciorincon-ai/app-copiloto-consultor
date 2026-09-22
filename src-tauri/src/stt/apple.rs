//! El lado de Rust del puente hacia `SpeechAnalyzer`. **Aquí vive todo el `unsafe` de `stt/`**,
//! igual que `acople/ax.rs` concentra el del acople: una sola puerta, vigilada, y el resto del
//! módulo en Rust seguro.
//!
//! Las cuatro funciones que cruzan la frontera están declaradas abajo tal y como las expone
//! `nativo/Transcriptor.swift`. Ninguna recibe ni devuelve punteros que sobrevivan a la llamada:
//! el audio entra prestado desde el anillo, el texto sale copiado a un buffer nuestro. Si la
//! librería de Swift no se compiló (`swiftc` ausente), este archivo no declara nada y
//! [`motor`] devuelve el mudo con su motivo.

use super::{Disponibilidad, Fallo, Motor, Mudo};

/// Cuánto texto cabe en una transcripción de un turno. Treinta segundos de habla rápida rondan las
/// 900 letras; ocho kilobytes son holgura de sobra sin ser una reserva absurda por turno.
const CABIDA: usize = 8 * 1024;

#[cfg(all(target_os = "macos", puente_de_swift))]
mod puente {
    use std::os::raw::{c_char, c_int};

    #[link(name = "agstt", kind = "static")]
    extern "C" {
        pub fn ag_stt_disponible() -> c_int;
        pub fn ag_stt_estado(idioma: *const c_char) -> c_int;
        pub fn ag_stt_techo_de_idiomas() -> c_int;
        pub fn ag_stt_idiomas(salida: *mut c_char, capacidad: c_int) -> c_int;
        pub fn ag_stt_instalar(idioma: *const c_char) -> c_int;
        pub fn ag_stt_transcribir(
            idioma: *const c_char,
            muestras: *const f32,
            n: c_int,
            hz: f64,
            salida: *mut c_char,
            capacidad: c_int,
        ) -> c_int;
    }
}

/// Los códigos que devuelve el puente. Se traducen aquí para que el resto de la app no vea
/// números: un `-3` no se puede enseñar a nadie, «el modelo de español no está instalado» sí.
#[cfg(all(target_os = "macos", puente_de_swift))]
fn traducir(codigo: i32) -> Disponibilidad {
    match codigo {
        2 => Disponibilidad::Listo,
        1 => Disponibilidad::SinModelo,
        0 => Disponibilidad::IdiomaDesconocido,
        -1 => Disponibilidad::SinMotor {
            motivo: "este Mac no trae el transcriptor de macOS 26".into(),
        },
        -2 => Disponibilidad::IdiomaDesconocido,
        -3 => Disponibilidad::SinModelo,
        otro => Disponibilidad::SinMotor { motivo: format!("el motor devolvió {otro}") },
    }
}

/// El motor de Apple, cuando existe.
#[cfg(all(target_os = "macos", puente_de_swift))]
pub struct Apple;

#[cfg(all(target_os = "macos", puente_de_swift))]
impl Motor for Apple {
    fn nombre(&self) -> &'static str {
        "apple-speechanalyzer"
    }

    fn disponibilidad(&self, idioma: &str) -> Disponibilidad {
        let Ok(c) = std::ffi::CString::new(idioma) else {
            return Disponibilidad::IdiomaDesconocido;
        };
        if unsafe { puente::ag_stt_disponible() } != 1 {
            return Disponibilidad::SinMotor {
                motivo: "este Mac no trae el transcriptor de macOS 26".into(),
            };
        }
        traducir(unsafe { puente::ag_stt_estado(c.as_ptr()) })
    }

    fn instalar(&self, idioma: &str) -> Disponibilidad {
        let Ok(c) = std::ffi::CString::new(idioma) else {
            return Disponibilidad::IdiomaDesconocido;
        };
        traducir(unsafe { puente::ag_stt_instalar(c.as_ptr()) })
    }

    fn transcribir(&self, idioma: &str, muestras: &[f32], hz: u32) -> Result<String, Fallo> {
        match self.disponibilidad(idioma) {
            Disponibilidad::Listo => {}
            otra => return Err(Fallo::NoDisponible(otra)),
        }
        let c = std::ffi::CString::new(idioma).map_err(|_| {
            Fallo::NoDisponible(Disponibilidad::IdiomaDesconocido)
        })?;
        let mut salida = vec![0i8; CABIDA];
        let escritos = unsafe {
            puente::ag_stt_transcribir(
                c.as_ptr(),
                muestras.as_ptr(),
                muestras.len() as i32,
                hz as f64,
                salida.as_mut_ptr(),
                CABIDA as i32,
            )
        };
        if escritos < 0 {
            return Err(match traducir(escritos) {
                Disponibilidad::SinMotor { .. } => Fallo::Motor(escritos),
                otra => Fallo::NoDisponible(otra),
            });
        }
        let bytes: Vec<u8> = salida[..escritos as usize].iter().map(|b| *b as u8).collect();
        String::from_utf8(bytes).map_err(|_| Fallo::Motor(escritos))
    }

    fn techo_de_idiomas(&self) -> u32 {
        unsafe { puente::ag_stt_techo_de_idiomas() }.max(0) as u32
    }

    fn idiomas(&self) -> Vec<String> {
        let mut salida = vec![0i8; CABIDA];
        let escritos = unsafe { puente::ag_stt_idiomas(salida.as_mut_ptr(), CABIDA as i32) };
        if escritos <= 0 {
            return Vec::new();
        }
        let bytes: Vec<u8> = salida[..escritos as usize].iter().map(|b| *b as u8).collect();
        String::from_utf8(bytes)
            .unwrap_or_default()
            .split(',')
            .filter(|s| !s.is_empty())
            .map(|s| s.replace('_', "-"))
            .collect()
    }
}

/// El motor de este Mac, o el mudo con su razón.
pub fn motor() -> Box<dyn Motor> {
    #[cfg(all(target_os = "macos", puente_de_swift))]
    {
        let a = Apple;
        // Se pregunta una vez al arrancar: si el sistema no trae el motor, se devuelve el mudo con
        // ese motivo en vez de un Apple que va a fallar en cada turno con el mismo mensaje.
        if unsafe { puente::ag_stt_disponible() } == 1 {
            return Box::new(a);
        }
        Box::new(Mudo::por("este Mac no trae el transcriptor de macOS 26"))
    }
    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    {
        Box::new(Mudo::por(
            "la app se compiló sin el puente de transcripción (falta swiftc o no es macOS)",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El motor de la casa **siempre existe**, tenga o no transcriptor debajo. Lo que cambia es lo
    /// que contesta, y en el peor caso contesta un motivo legible — nunca un `panic` ni un
    /// `Option` que alguien vaya a desenvolver a ciegas en mitad de una reunión.
    #[test]
    fn siempre_hay_un_motor_y_siempre_tiene_nombre() {
        let m = motor();
        assert!(!m.nombre().is_empty());
        if let Disponibilidad::SinMotor { motivo } = m.disponibilidad("es-ES") {
            assert!(!motivo.is_empty(), "un motivo vacío no explica nada");
        }
    }

    /// Un idioma que no existe no puede acabar en un `panic` ni en una transcripción inventada.
    #[test]
    fn un_idioma_inventado_se_contesta_sin_romperse() {
        let m = motor();
        let d = m.disponibilidad("xx-ZZ");
        assert!(
            matches!(d, Disponibilidad::IdiomaDesconocido | Disponibilidad::SinMotor { .. }),
            "«xx-ZZ» devolvió {d:?}"
        );
    }

    /// Una cadena con un cero interior no puede cruzar a C. Se rechaza aquí, con nombre, en vez de
    /// dejar que `CString::new` reviente en la llamada.
    #[test]
    fn un_idioma_con_un_cero_dentro_no_cruza_la_frontera() {
        let m = motor();
        assert!(matches!(
            m.disponibilidad("es\0ES"),
            Disponibilidad::IdiomaDesconocido | Disponibilidad::SinMotor { .. }
        ));
    }
}
