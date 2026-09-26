//! **(a) El modelo del sistema** — Apple Foundation Models, dentro del Mac, por el puente de Swift
//! (`nativo/Sintesis.swift`). Cero bytes fuera, cero costo. Necesita Apple Intelligence activado en
//! Ajustes; si no está, lo dice con su motivo y la app sigue con la ficha sola.

use super::{PorQueNoRedacta, Proveedor, Quien, Respuesta};

#[cfg(all(target_os = "macos", puente_de_swift))]
mod puente {
    use std::os::raw::{c_char, c_double, c_int};

    #[link(name = "agstt", kind = "static")]
    extern "C" {
        pub fn ag_sintesis_disponible() -> c_int;
        pub fn ag_sintesis_redactar(
            instrucciones: *const c_char,
            texto: *const c_char,
            segundos: c_double,
            salida: *mut c_char,
            capacidad: c_int,
        ) -> c_int;
    }
}

pub struct DelSistema;

/// El código del puente, en el conjunto cerrado de la app.
fn motivo(codigo: i32) -> PorQueNoRedacta {
    match codigo {
        -1 => PorQueNoRedacta::AppleIntelligenceApagado,
        -2 => PorQueNoRedacta::MacNoCompatible,
        -3 => PorQueNoRedacta::ModeloDescargandose,
        _ => PorQueNoRedacta::MacNoCompatible,
    }
}

impl Proveedor for DelSistema {
    fn quien(&self) -> Quien {
        Quien::Sistema
    }

    fn nombre(&self) -> String {
        "Modelo del sistema".into()
    }

    #[cfg(all(target_os = "macos", puente_de_swift))]
    fn disponible(&self) -> Result<(), PorQueNoRedacta> {
        // SEGURIDAD: una llamada sin argumentos que solo pregunta a macOS.
        match unsafe { puente::ag_sintesis_disponible() } {
            0 => Ok(()),
            c => Err(motivo(c)),
        }
    }

    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    fn disponible(&self) -> Result<(), PorQueNoRedacta> {
        let _ = motivo;
        Err(PorQueNoRedacta::SinPuente)
    }

    #[cfg(all(target_os = "macos", puente_de_swift))]
    fn redactar(&self, instrucciones: &str, texto: &str) -> Result<Respuesta, String> {
        use std::ffi::CString;
        let i = CString::new(instrucciones).map_err(|_| "instrucciones con un cero dentro")?;
        let mut t = CString::new(texto).map_err(|_| "texto con un cero dentro")?.into_bytes_with_nul();
        let mut salida = vec![0u8; 8 * 1024];
        // SEGURIDAD: los dos textos terminan en cero y viven hasta que la llamada vuelve; la salida
        // tiene exactamente `capacidad` bytes y el puente escribe menos, con su cero final.
        let n = unsafe {
            puente::ag_sintesis_redactar(
                i.as_ptr(),
                t.as_ptr() as *const std::os::raw::c_char,
                // Un poco más que el techo de Rust: quien corta es `sugerir`, con su reloj.
                (super::TECHO.as_secs_f64() + 1.0) as std::os::raw::c_double,
                salida.as_mut_ptr() as *mut std::os::raw::c_char,
                salida.len() as std::os::raw::c_int,
            )
        };
        // El turno del cliente iba ahí dentro: se pisa.
        t.fill(0);
        if n < 0 {
            return Err(format!("el modelo del sistema no redactó (código {n})"));
        }
        let json = String::from_utf8_lossy(&salida[..n as usize]).into_owned();
        salida.fill(0);
        Ok(Respuesta { json, ..Default::default() })
    }

    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    fn redactar(&self, _i: &str, _t: &str) -> Result<Respuesta, String> {
        Err("esta compilación no trae el puente del modelo del sistema".into())
    }
}
