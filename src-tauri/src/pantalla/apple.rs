//! El lado de Rust del puente hacia ScreenCaptureKit y Vision. **Aquí vive todo el `unsafe` de
//! `pantalla/`**, igual que `stt/apple.rs` concentra el de la transcripción y `habla/apple.rs` el de
//! la voz: una sola puerta, vigilada.
//!
//! Las dos funciones que cruzan están declaradas abajo tal y como las expone
//! `nativo/Pantalla.swift`, y **las dos escriben en memoria que pone Rust**: el cuadro va a parar al
//! búfer de [`Cuadro`], y el texto leído a un búfer que se pisa con ceros en cuanto se ha partido en
//! líneas. Ninguna deja un puntero vivo ni una copia del lado de Swift.
//!
//! Sin puente de Swift (`swiftc` ausente), [`ojos`] devuelve [`super::Ciegos`] y la lectura de pantalla no
//! existe — la app funciona y Sesión lo dice.

use super::{Lector, LineaLeida, Ojo};

#[cfg(all(target_os = "macos", puente_de_swift))]
mod puente {
    use std::os::raw::{c_char, c_int};

    #[link(name = "agstt", kind = "static")]
    extern "C" {
        pub fn ag_pantalla_mirar(
            bundle: *const c_char,
            senales: *const c_char,
            lado_maximo: c_int,
            salida: *mut u8,
            capacidad: c_int,
            ancho: *mut c_int,
            alto: *mut c_int,
        ) -> c_int;
        pub fn ag_pantalla_leer(
            gris: *const u8,
            ancho: c_int,
            alto: c_int,
            salida: *mut c_char,
            capacidad: c_int,
        ) -> c_int;
    }
}

/// Lo que devuelve el puente cuando no pudo. Espejo de `CodigoDePantalla` en Swift.
#[cfg(all(target_os = "macos", puente_de_swift))]
const SIN_PERMISO: i32 = -1;
#[cfg(all(target_os = "macos", puente_de_swift))]
const SIN_VENTANA: i32 = -2;
#[cfg(all(target_os = "macos", puente_de_swift))]
const CABE_MAL: i32 = -4;
#[cfg(all(target_os = "macos", puente_de_swift))]
const SIN_SOPORTE: i32 = -5;

/// Sitio para el texto de una pantalla. Una diapositiva densa son unos pocos KB; una ventana de
/// Meet con el chat abierto, algo más. Si no cabe, el puente lo dice en vez de cortar a medias.
#[cfg(all(target_os = "macos", puente_de_swift))]
const TEXTO_MAXIMO: usize = 256 * 1024;

/// ScreenCaptureKit + Vision, cuando el puente está.
#[cfg(all(target_os = "macos", puente_de_swift))]
pub struct DelSistema;

#[cfg(all(target_os = "macos", puente_de_swift))]
impl Ojo for DelSistema {
    // (los tipos van con su ruta: fuera de macOS este bloque no existe y el `use` quedaría huérfano)
    fn nombre(&self) -> &'static str {
        "apple-screencapturekit"
    }

    fn mirar(
        &self,
        objetivo: &super::Objetivo,
        cuadro: &mut super::Cuadro,
    ) -> Result<(), super::NoSeVe> {
        use super::{NoSeVe, ANCHO_MAXIMO};
        let bundle = std::ffi::CString::new(objetivo.bundle.as_str())
            .map_err(|_| NoSeVe::Fallo("el identificador de la app lleva un cero dentro".into()))?;
        let senales = std::ffi::CString::new(objetivo.senales.join(","))
            .map_err(|_| NoSeVe::Fallo("las señales del título llevan un cero dentro".into()))?;
        if cuadro.gris.len() < ANCHO_MAXIMO * ANCHO_MAXIMO {
            cuadro.gris.resize(ANCHO_MAXIMO * ANCHO_MAXIMO, 0);
        }
        let (mut ancho, mut alto) = (0, 0);
        let codigo = unsafe {
            puente::ag_pantalla_mirar(
                bundle.as_ptr(),
                senales.as_ptr(),
                ANCHO_MAXIMO as i32,
                cuadro.gris.as_mut_ptr(),
                cuadro.gris.len() as i32,
                &mut ancho,
                &mut alto,
            )
        };
        match codigo {
            1 => {
                cuadro.ajustar(ancho.max(0) as usize, alto.max(0) as usize);
                Ok(())
            }
            SIN_PERMISO => Err(NoSeVe::SinPermiso),
            SIN_VENTANA => Err(NoSeVe::SinVentana),
            SIN_SOPORTE => Err(NoSeVe::Fallo(
                "este macOS no trae ScreenCaptureKit con capturas sueltas (14+)".into(),
            )),
            otro => Err(NoSeVe::Fallo(format!(
                "ScreenCaptureKit no entregó el cuadro (código {otro})"
            ))),
        }
    }
}

#[cfg(all(target_os = "macos", puente_de_swift))]
impl Lector for DelSistema {
    fn leer(&self, cuadro: &super::Cuadro) -> Result<Vec<LineaLeida>, String> {
        if cuadro.ancho == 0 || cuadro.alto == 0 || cuadro.gris.len() < cuadro.ancho * cuadro.alto {
            return Err("no hay cuadro que leer".into());
        }
        let mut texto = vec![0u8; TEXTO_MAXIMO];
        let n = unsafe {
            puente::ag_pantalla_leer(
                cuadro.gris.as_ptr(),
                cuadro.ancho as i32,
                cuadro.alto as i32,
                texto.as_mut_ptr() as *mut std::os::raw::c_char,
                texto.len() as i32,
            )
        };
        let salida = if n >= 0 {
            let fin = texto.iter().position(|&b| b == 0).unwrap_or(texto.len());
            Ok(partir(&String::from_utf8_lossy(&texto[..fin])))
        } else if n == CABE_MAL {
            Err("la pantalla traía más texto del que cabe; no se leyó a medias".into())
        } else {
            Err(format!("Vision no pudo leer el cuadro (código {n})"))
        };
        // Lo leído es texto de un tercero: el búfer se pisa antes de soltarse.
        texto.fill(0);
        salida
    }
}

/// El formato del puente: una línea por renglón leído, `confianza \t alto \t texto`. Se parte
/// aquí, en Rust seguro, y es lo único de este archivo que se prueba sin el Mac.
pub fn partir(crudo: &str) -> Vec<LineaLeida> {
    crudo
        .lines()
        .filter_map(|l| {
            let mut partes = l.splitn(3, '\t');
            let confianza = partes.next()?.parse().ok()?;
            let alto = partes.next()?.parse().ok()?;
            let texto = partes.next()?.to_string();
            Some(LineaLeida {
                texto,
                confianza,
                alto,
            })
        })
        .collect()
}

/// Los ojos y el lector de la casa: los del sistema si el puente está, y los ciegos si no.
#[cfg(all(target_os = "macos", puente_de_swift))]
pub fn ojos() -> (Box<dyn Ojo>, Box<dyn Lector>) {
    (Box::new(DelSistema), Box::new(DelSistema))
}

#[cfg(not(all(target_os = "macos", puente_de_swift)))]
pub fn ojos() -> (Box<dyn Ojo>, Box<dyn Lector>) {
    (Box::new(super::Ciegos), Box::new(super::Ciegos))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_formato_del_puente_se_parte_bien() {
        let l = partir(
            "0.98\t0.061\tRentabilidad por canal\n0.5\t0.02\tMargen: 23 %\tcon tab\nbasura\n",
        );
        assert_eq!(l.len(), 2, "la línea sin campos se descarta, no revienta");
        assert_eq!(l[0].texto, "Rentabilidad por canal");
        assert_eq!(
            l[1].texto, "Margen: 23 %\tcon tab",
            "el texto puede traer tabuladores"
        );
        assert!((l[0].alto - 0.061).abs() < 1e-6);
    }
}
