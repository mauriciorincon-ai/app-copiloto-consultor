//! EL LLAVERO — donde la app guarda sus secretos: en el Llavero de macOS del usuario, jamás en un
//! archivo.
//!
//! Tres servicios, uno por dueño, para que un secreto no se confunda con otro ni se borre con él:
//!
//! | Servicio | Qué guarda | Quién lo lee |
//! |---|---|---|
//! | «Angel Ghost · API» | la clave de cada proveedor externo (ADR 011) | `sintesis::api`, en el instante de enviar |
//! | «Angel Ghost · notas» | la llave de tus notas cifradas (ADR 015) | `notas`, al guardar y al abrir |
//! | «Angel Ghost · puerta» | el token de la puerta local (ADR 018) | la puerta y `ghost` |
//!
//! Hoy los tres van al **llavero de inicio de sesión** (el de archivo): se abre con tu sesión, no se
//! sincroniza con iCloud y viaja con tus copias de Time Machine y con el Asistente de migración,
//! protegido por tu contraseña. El `WhenUnlockedThisDeviceOnly` de `nativo/Llavero.swift` solo opera
//! en el llavero de protección de datos, que llega con la firma (ADR 015, enmienda 2; auditoría del
//! S3, A2). Hasta el sprint 003 esto vivía
//! dentro de `sintesis/api.rs` con un solo servicio; salió de ahí cuando lo necesitaron las notas y
//! la puerta, y el servicio de las claves del API no cambió de nombre, así que las claves que el
//! usuario ya guardó se siguen encontrando.

/// De quién es el secreto.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Servicio {
    Api,
    Notas,
    Puerta,
}

impl Servicio {
    /// El nombre con que aparece en «Acceso a Llaveros». **El de `Api` no se puede cambiar**: es el
    /// de las claves que el usuario guardó en el sprint 002.
    pub fn nombre(self) -> &'static str {
        match self {
            Servicio::Api => "Angel Ghost · API",
            Servicio::Notas => "Angel Ghost · notas",
            Servicio::Puerta => "Angel Ghost · puerta",
        }
    }
}

#[cfg(all(target_os = "macos", puente_de_swift))]
mod puente {
    use std::os::raw::{c_char, c_int};

    #[link(name = "agstt", kind = "static")]
    extern "C" {
        pub fn ag_llavero_guardar(servicio: *const c_char, cuenta: *const c_char, clave: *const c_char) -> c_int;
        pub fn ag_llavero_leer(servicio: *const c_char, cuenta: *const c_char, salida: *mut u8, capacidad: c_int) -> c_int;
        pub fn ag_llavero_borrar(servicio: *const c_char, cuenta: *const c_char) -> c_int;
        pub fn ag_llavero_hay(servicio: *const c_char, cuenta: *const c_char) -> c_int;
    }
}

/// Guarda (o reemplaza) un secreto. El texto que llega se pisa con ceros en cuanto el Llavero lo
/// tiene.
pub fn guardar(servicio: Servicio, cuenta: &str, secreto: &str) -> Result<(), String> {
    crate::hardware::vigilar("el Llavero");
    #[cfg(all(target_os = "macos", puente_de_swift))]
    {
        let s = std::ffi::CString::new(servicio.nombre()).map_err(|e| e.to_string())?;
        let c = std::ffi::CString::new(cuenta).map_err(|e| e.to_string())?;
        let mut k = std::ffi::CString::new(secreto)
            .map_err(|_| "el secreto lleva un cero dentro")?
            .into_bytes_with_nul();
        // SEGURIDAD: tres textos terminados en cero que viven hasta que la llamada vuelve.
        let r = unsafe {
            puente::ag_llavero_guardar(s.as_ptr(), c.as_ptr(), k.as_ptr() as *const std::os::raw::c_char)
        };
        k.fill(0);
        if r == 0 { Ok(()) } else { Err(format!("el Llavero no lo guardó ({r})")) }
    }
    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    {
        let _ = (servicio, cuenta, secreto);
        Err("el Llavero solo existe en macOS con el puente".into())
    }
}

/// Lee un secreto, si hay. El búfer intermedio se pisa con ceros; quien lo recibe responde del suyo.
pub fn leer(servicio: Servicio, cuenta: &str) -> Option<String> {
    crate::hardware::vigilar("el Llavero");
    #[cfg(all(target_os = "macos", puente_de_swift))]
    {
        let s = std::ffi::CString::new(servicio.nombre()).ok()?;
        let c = std::ffi::CString::new(cuenta).ok()?;
        let mut salida = vec![0u8; 1024];
        // SEGURIDAD: la salida mide `capacidad` y el puente escribe menos.
        let n = unsafe { puente::ag_llavero_leer(s.as_ptr(), c.as_ptr(), salida.as_mut_ptr(), salida.len() as i32) };
        let secreto = (n > 0).then(|| String::from_utf8_lossy(&salida[..n as usize]).into_owned());
        salida.fill(0);
        secreto
    }
    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    {
        let _ = (servicio, cuenta);
        None
    }
}

/// ¿Hay secreto? Se pregunta por los atributos, **sin leerlo**.
pub fn hay(servicio: Servicio, cuenta: &str) -> bool {
    crate::hardware::vigilar("el Llavero");
    #[cfg(all(target_os = "macos", puente_de_swift))]
    {
        let (Ok(s), Ok(c)) = (std::ffi::CString::new(servicio.nombre()), std::ffi::CString::new(cuenta)) else {
            return false;
        };
        // SEGURIDAD: dos textos terminados en cero que viven hasta que la llamada vuelve.
        unsafe { puente::ag_llavero_hay(s.as_ptr(), c.as_ptr()) == 1 }
    }
    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    {
        let _ = (servicio, cuenta);
        false
    }
}

/// ¿Hay secreto? Como [`hay`], pero **distingue «no hay» de «el Llavero no contestó»**. Quien vaya a
/// crear un secreto si falta tiene que usar esta: con [`hay`], un Llavero bloqueado se lee como «no
/// hay», y crear uno nuevo encima borraría el que había. Para la llave de las notas eso es perder
/// todas las reuniones guardadas (ADR 015 §4).
pub fn existe(servicio: Servicio, cuenta: &str) -> Result<bool, String> {
    crate::hardware::vigilar("el Llavero");
    #[cfg(all(target_os = "macos", puente_de_swift))]
    {
        let s = std::ffi::CString::new(servicio.nombre()).map_err(|e| e.to_string())?;
        let c = std::ffi::CString::new(cuenta).map_err(|e| e.to_string())?;
        // SEGURIDAD: dos textos terminados en cero que viven hasta que la llamada vuelve.
        match unsafe { puente::ag_llavero_hay(s.as_ptr(), c.as_ptr()) } {
            1 => Ok(true),
            0 => Ok(false),
            r => Err(format!("el Llavero no contestó ({r})")),
        }
    }
    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    {
        let _ = (servicio, cuenta);
        Err("el Llavero solo existe en macOS con el puente".into())
    }
}

/// Borra un secreto (bien también si no había).
pub fn borrar(servicio: Servicio, cuenta: &str) -> Result<(), String> {
    crate::hardware::vigilar("el Llavero");
    #[cfg(all(target_os = "macos", puente_de_swift))]
    {
        let s = std::ffi::CString::new(servicio.nombre()).map_err(|e| e.to_string())?;
        let c = std::ffi::CString::new(cuenta).map_err(|e| e.to_string())?;
        // SEGURIDAD: dos textos terminados en cero.
        match unsafe { puente::ag_llavero_borrar(s.as_ptr(), c.as_ptr()) } {
            0 => Ok(()),
            r => Err(format!("el Llavero no lo borró ({r})")),
        }
    }
    #[cfg(not(all(target_os = "macos", puente_de_swift)))]
    {
        let _ = (servicio, cuenta);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **El nombre del servicio del API no se toca**: las claves que el usuario guardó en el sprint
    /// 002 viven bajo él, y cambiarlo las dejaría huérfanas sin un solo error.
    #[test]
    fn el_servicio_del_api_conserva_su_nombre() {
        assert_eq!(Servicio::Api.nombre(), "Angel Ghost · API");
    }

    #[test]
    fn cada_dueno_tiene_su_servicio() {
        let nombres = [Servicio::Api, Servicio::Notas, Servicio::Puerta].map(Servicio::nombre);
        let mut unicos = nombres.to_vec();
        unicos.sort();
        unicos.dedup();
        assert_eq!(unicos.len(), 3, "dos dueños compartirían servicio: {nombres:?}");
    }
}
