//! ¿Está este equipo inscrito en un MDM? Lo contesta el propio sistema: macOS con `profiles`, Windows con
//! `IsDeviceRegisteredWithManagement` (sprint 008).
//!
//! Un MDM no siempre deja un proceso a la vista —la inscripción de Apple vive en el sistema—, así
//! que se le pregunta a `/usr/bin/profiles`, la herramienta de Apple para eso. Es **la única
//! llamada del radar que lanza un programa**, y es local: `profiles status` lee la configuración
//! de este equipo y no habla con nadie. Tarda ~0,1 s, así que se pregunta una vez al arrancar el
//! radar y no en cada vuelta.

/// `Some(true)` si macOS dice que está inscrito, `None` si no se pudo preguntar.
#[cfg(target_os = "macos")]
pub fn inscrito() -> Option<bool> {
    let salida = std::process::Command::new("/usr/bin/profiles")
        .args(["status", "-type", "enrollment"])
        .output()
        .ok()?;
    if !salida.status.success() {
        return None;
    }
    Some(dice_inscrito(&String::from_utf8_lossy(&salida.stdout)))
}

/// **En Windows** (sprint 008): se le pregunta a `IsDeviceRegisteredWithManagement`, la función que Windows publica
/// para eso (`mdmregistration.h`). Es local, no lanza ningún programa y no lee la cuenta inscrita: el segundo
/// argumento, que la devolvería, va vacío. `None` si Windows no contestó.
#[cfg(windows)]
pub fn inscrito() -> Option<bool> {
    use windows::core::BOOL;
    use windows::Win32::Management::MobileDeviceManagementRegistration::IsDeviceRegisteredWithManagement;
    let mut si = BOOL(0);
    // SAFETY: `si` vive durante la llamada y sin búfer para la cuenta Windows solo escribe el booleano.
    unsafe { IsDeviceRegisteredWithManagement(&mut si, None) }.ok()?;
    Some(si.as_bool())
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn inscrito() -> Option<bool> {
    None
}

/// `MDM enrollment: Yes` o `Yes (User Approved)` ⇒ inscrito; `No` ⇒ no. Fuera de macOS solo lo usan sus pruebas.
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
fn dice_inscrito(texto: &str) -> bool {
    texto.lines().any(|l| {
        let l = l.trim();
        l.strip_prefix("MDM enrollment:").is_some_and(|v| v.trim().starts_with("Yes"))
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn se_lee_lo_que_escribe_profiles() {
        assert!(!dice_inscrito("Enrolled via DEP: No\nMDM enrollment: No\n"));
        assert!(dice_inscrito("Enrolled via DEP: Yes\nMDM enrollment: Yes (User Approved)\n"));
        assert!(dice_inscrito("MDM enrollment: Yes\n"));
        assert!(!dice_inscrito(""));
    }
}
