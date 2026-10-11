//! El almacén de secretos en Windows (sprint 006, ADR 022): el **Administrador de credenciales** de tu usuario
//! (Credential Manager), con los mismos tres servicios que el Llavero del Mac.
//!
//! Cada secreto es una credencial **genérica** que se llama «<servicio> / <cuenta>», y así se ve en Control Panel →
//! Credential Manager → Windows Credentials, que es también donde se borra a mano. Se guarda con persistencia
//! `LOCAL_MACHINE`: se queda en este PC para tu usuario, cifrada por Windows con tu sesión, y no viaja con un perfil
//! itinerante de empresa (eso sería `ENTERPRISE`).
//!
//! El centinela de la regla 25 lo ponen las funciones de `llavero.rs` que llaman aquí, antes de cruzar.

use super::Servicio;
use windows_sys::Win32::Foundation::{GetLastError, ERROR_NOT_FOUND};
use windows_sys::Win32::Security::Credentials::{
    CredDeleteW, CredFree, CredReadW, CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
};

/// El nombre de la credencial: el servicio y la cuenta, como se ven en el Administrador de credenciales.
pub fn nombre(servicio: Servicio, cuenta: &str) -> String {
    format!("{} / {}", servicio.nombre(), cuenta)
}

/// Un texto en UTF-16 terminado en cero, como lo pide Windows. Un cero dentro no cruza.
fn ancho(texto: &str) -> Result<Vec<u16>, String> {
    if texto.contains('\0') {
        return Err("el texto lleva un cero dentro".into());
    }
    Ok(texto.encode_utf16().chain(std::iter::once(0)).collect())
}

/// Guarda (o reemplaza) un secreto. La copia intermedia se pisa con ceros en cuanto Windows la tiene.
pub fn guardar(servicio: Servicio, cuenta: &str, secreto: &str) -> Result<(), String> {
    if secreto.contains('\0') {
        return Err("el secreto lleva un cero dentro".into());
    }
    let mut destino = ancho(&nombre(servicio, cuenta))?;
    let mut usuario = ancho(cuenta)?;
    let mut blob = secreto.as_bytes().to_vec();
    let credencial = CREDENTIALW {
        Type: CRED_TYPE_GENERIC,
        TargetName: destino.as_mut_ptr(),
        UserName: usuario.as_mut_ptr(),
        CredentialBlobSize: blob.len() as u32,
        CredentialBlob: blob.as_mut_ptr(),
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        ..Default::default()
    };
    // SEGURIDAD: los tres búferes viven hasta que la llamada vuelve, y Windows copia lo que guarda.
    let ok = unsafe { CredWriteW(&credencial, 0) } != 0;
    // SEGURIDAD: sin argumentos; lee el último error de este hilo.
    let error = if ok { 0 } else { unsafe { GetLastError() } };
    blob.fill(0);
    if ok {
        Ok(())
    } else {
        Err(format!("el Administrador de credenciales no lo guardó ({error})"))
    }
}

/// Lee un secreto: `Ok(None)` si no hay, `Err` si Windows no contestó. La copia de Windows se pisa con ceros antes
/// de soltarla.
pub fn leer(servicio: Servicio, cuenta: &str) -> Result<Option<String>, String> {
    let destino = ancho(&nombre(servicio, cuenta))?;
    let mut credencial: *mut CREDENTIALW = std::ptr::null_mut();
    // SEGURIDAD: un nombre terminado en cero; Windows deja en `credencial` un bloque suyo que se suelta con CredFree.
    if unsafe { CredReadW(destino.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credencial) } == 0 {
        // SEGURIDAD: sin argumentos; lee el último error de este hilo.
        let error = unsafe { GetLastError() };
        return if error == ERROR_NOT_FOUND {
            Ok(None)
        } else {
            Err(format!("el Administrador de credenciales no contestó ({error})"))
        };
    }
    // SEGURIDAD: `credencial` es el bloque válido que Windows acaba de entregar; el secreto mide lo que dice
    // `CredentialBlobSize`, se copia, se pisa con ceros y el bloque se suelta una sola vez.
    let secreto = unsafe {
        let c = &*credencial;
        let largo = c.CredentialBlobSize as usize;
        let texto = if c.CredentialBlob.is_null() || largo == 0 {
            String::new()
        } else {
            let texto = String::from_utf8_lossy(std::slice::from_raw_parts(c.CredentialBlob, largo)).into_owned();
            std::ptr::write_bytes(c.CredentialBlob, 0, largo);
            texto
        };
        CredFree(credencial as *const core::ffi::c_void);
        texto
    };
    Ok(Some(secreto).filter(|s| !s.is_empty()))
}

/// ¿Hay secreto? Windows no deja preguntar sin leerlo: se lee y la copia se pisa con ceros en el acto.
pub fn existe(servicio: Servicio, cuenta: &str) -> Result<bool, String> {
    Ok(leer(servicio, cuenta)?.map(|s| s.into_bytes().fill(0)).is_some())
}

/// Borra un secreto (bien también si no había).
pub fn borrar(servicio: Servicio, cuenta: &str) -> Result<(), String> {
    let destino = ancho(&nombre(servicio, cuenta))?;
    // SEGURIDAD: un nombre terminado en cero que vive hasta que la llamada vuelve.
    if unsafe { CredDeleteW(destino.as_ptr(), CRED_TYPE_GENERIC, 0) } != 0 {
        return Ok(());
    }
    // SEGURIDAD: sin argumentos; lee el último error de este hilo.
    match unsafe { GetLastError() } {
        ERROR_NOT_FOUND => Ok(()),
        error => Err(format!("el Administrador de credenciales no lo borró ({error})")),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// El nombre lleva el servicio y la cuenta: así se encuentra a mano en el Administrador de credenciales.
    #[test]
    fn el_nombre_lleva_servicio_y_cuenta() {
        assert_eq!(nombre(Servicio::Notas, "llave"), "Angel Ghost · notas / llave");
    }

    /// Un cero dentro no cruza a Windows, y todo lo que cruza termina en cero.
    #[test]
    fn un_cero_dentro_no_cruza() {
        assert!(ancho("a\0b").is_err());
        assert_eq!(ancho("ñ").unwrap(), vec![0xF1, 0]);
        assert!(guardar(Servicio::Api, "prueba", "con\0cero").is_err());
    }

    /// **El Administrador de credenciales de verdad**: guarda, lee, reemplaza y borra una credencial de prueba. Lo
    /// corre la CI de Windows en su runner efímero (`build-windows`), jamás un `cargo test` a secas en tu PC.
    #[test]
    #[ignore = "hardware: escribe y borra una credencial de prueba en el Administrador de credenciales"]
    fn guarda_lee_reemplaza_y_borra() {
        let cuenta = format!("prueba-ci-{}", std::process::id());
        borrar(Servicio::Puerta, &cuenta).unwrap();
        assert_eq!(leer(Servicio::Puerta, &cuenta).unwrap(), None, "una credencial que no existe se leyó");
        guardar(Servicio::Puerta, &cuenta, "uno").unwrap();
        assert_eq!(leer(Servicio::Puerta, &cuenta).unwrap().as_deref(), Some("uno"));
        guardar(Servicio::Puerta, &cuenta, "dos · ñ").unwrap();
        assert_eq!(leer(Servicio::Puerta, &cuenta).unwrap().as_deref(), Some("dos · ñ"));
        borrar(Servicio::Puerta, &cuenta).unwrap();
        assert_eq!(leer(Servicio::Puerta, &cuenta).unwrap(), None, "la credencial sobrevivió al borrado");
        borrar(Servicio::Puerta, &cuenta).unwrap();
    }
}
