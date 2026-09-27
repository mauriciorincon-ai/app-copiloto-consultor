//! ¿Está este Mac inscrito en un MDM? Lo contesta el propio macOS.
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

#[cfg(not(target_os = "macos"))]
pub fn inscrito() -> Option<bool> {
    None
}

/// `MDM enrollment: Yes` o `Yes (User Approved)` ⇒ inscrito; `No` ⇒ no.
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
