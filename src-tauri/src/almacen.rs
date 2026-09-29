//! LO QUE LA APP ESCRIBE — un solo escritor para todo lo que persiste.
//!
//! Hasta el sprint 002 cada archivo tenía su propio ayudante (el diccionario, el gasto del mes, la
//! huella del acople), privados y tipados a un solo uso. El sprint 003 añade preferencias, notas
//! cifradas, la bandeja y la lista de vencimientos: cinco copias de la misma regla habrían sido
//! cinco oportunidades de olvidarla en una. Aquí vive **una**:
//!
//! 1. **Un archivo nace cerrado (600), no se aprieta después** (regla 17-bis): `create_new` con su
//!    modo, sin la ventana en 644 que la primera versión del diccionario dejaba abierta.
//! 2. **Nunca desaparece a medias:** se escribe a un temporal, se fuerza a disco (`sync_all`) y se
//!    RENOMBRA encima, que es atómico (auditoría del S2, M10). Una caída deja el archivo viejo o el
//!    nuevo, jamás uno vacío.
//! 3. **Su carpeta es solo del dueño (700)**, y si ya existía floja se repara — solo la carpeta del
//!    archivo, nunca sus padres: `~/Library/Application Support` no es de esta app.
//!
//! Qué se escribe, dónde y por qué lo decide cada módulo con su ADR (002 y sus enmiendas); este
//! archivo solo decide **cómo**.

use std::path::Path;

/// Permisos de todo archivo que la app escribe: solo su dueño.
pub const ARCHIVO: u32 = 0o600;
/// Permisos de toda carpeta que la app crea para sus archivos.
pub const CARPETA: u32 = 0o700;

/// Escribe `bytes` en `ruta` de una vez: temporal que nace cerrado, a disco, y renombrado encima.
pub fn escribir(ruta: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(padre) = ruta.parent() {
        carpeta_privada(padre)?;
    }
    poner(ruta, bytes)
}

/// Lo mismo, en una carpeta **que no es de la app**: la crea si falta, pero **no toca sus permisos**.
/// Tiene dos usos: la de `~/Library/LaunchAgents` (ADR 016), donde vive el plist de la tarea de
/// vencimiento y que es del usuario y de otras apps; y **el destino que eliges al exportar** una
/// reunión (auditoría del S3, M5), que es tuyo. El archivo nace 600 igual.
pub fn escribir_en_carpeta_ajena(ruta: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).map_err(|e| format!("no se pudo crear {}: {e}", padre.display()))?;
    }
    poner(ruta, bytes)
}

fn poner(ruta: &Path, bytes: &[u8]) -> Result<(), String> {
    let temporal = temporal_de(ruta);
    // Un temporal que dejó una caída anterior no puede bloquear esta escritura.
    let _ = std::fs::remove_file(&temporal);
    if let Err(e) = nacer_cerrado(&temporal, bytes) {
        let _ = std::fs::remove_file(&temporal);
        return Err(e);
    }
    std::fs::rename(&temporal, ruta).map_err(|e| {
        let _ = std::fs::remove_file(&temporal);
        format!("no se pudo dejar {} en su sitio: {e}", ruta.display())
    })
}

/// El temporal de una escritura: el mismo nombre con `.tmp` detrás, en la misma carpeta (el
/// renombrado solo es atómico dentro del mismo volumen).
pub fn temporal_de(ruta: &Path) -> std::path::PathBuf {
    let mut nombre = ruta.file_name().map(|n| n.to_os_string()).unwrap_or_default();
    nombre.push(".tmp");
    ruta.with_file_name(nombre)
}

/// Crea el archivo **ya con sus permisos puestos** y falla si ya existía: entre un `exists()` y la
/// escritura nadie puede colarse con otro archivo.
#[cfg(unix)]
pub fn nacer_cerrado(ruta: &Path, bytes: &[u8]) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(ARCHIVO)
        .open(ruta)
        .map_err(|e| format!("no se pudo crear {}: {e}", ruta.display()))?;
    f.write_all(bytes).map_err(|e| format!("no se pudo escribir {}: {e}", ruta.display()))?;
    f.sync_all().map_err(|e| format!("no se pudo llevar {} al disco: {e}", ruta.display()))
}

#[cfg(not(unix))]
pub fn nacer_cerrado(ruta: &Path, bytes: &[u8]) -> Result<(), String> {
    std::fs::write(ruta, bytes).map_err(|e| format!("no se pudo escribir {}: {e}", ruta.display()))
}

/// Crea la carpeta si falta y la deja en 700. **Solo la última**: sus padres no son de esta app.
pub fn carpeta_privada(carpeta: &Path) -> Result<(), String> {
    std::fs::create_dir_all(carpeta).map_err(|e| format!("no se pudo crear {}: {e}", carpeta.display()))?;
    cerrar_permisos(carpeta, CARPETA).map(|_| ())
}

/// Aprieta los permisos si los encuentra flojos. **Devuelve si hubo que repararlos**: es lo que
/// permite probar que un archivo nació cerrado en vez de cerrarse un instante después.
#[cfg(unix)]
pub fn cerrar_permisos(ruta: &Path, modo: u32) -> Result<bool, String> {
    use std::os::unix::fs::PermissionsExt;
    let md = std::fs::metadata(ruta).map_err(|e| e.to_string())?;
    if md.permissions().mode() & 0o777 == modo {
        return Ok(false);
    }
    std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(modo))
        .map_err(|e| format!("no se pudieron cerrar los permisos de {}: {e}", ruta.display()))?;
    Ok(true)
}

#[cfg(not(unix))]
pub fn cerrar_permisos(_ruta: &Path, _modo: u32) -> Result<bool, String> {
    Ok(false)
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    /// Una carpeta propia por test (patrón de pruebas, regla 10 del kit): nadie pisa a nadie.
    fn carpeta(nombre: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("ag-almacen-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn modo(p: &Path) -> u32 {
        std::fs::metadata(p).unwrap().permissions().mode() & 0o777
    }

    /// **La carpeta ajena no se toca.** Demostrado en rojo: con `carpeta_privada(padre)` en
    /// `escribir_en_carpeta_ajena`, la carpeta pasa de 755 a 700.
    #[test]
    fn en_una_carpeta_ajena_el_archivo_nace_600_y_la_carpeta_queda_como_estaba() {
        let d = carpeta("ajena");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o755)).unwrap();
        let f = d.join("tarea.plist");
        escribir_en_carpeta_ajena(&f, b"<plist/>").unwrap();
        assert_eq!(modo(&f), ARCHIVO);
        assert_eq!(modo(&d), 0o755, "la carpeta del usuario cambió de permisos");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// **Nace en 600, sin ventana.** Si `cerrar_permisos` no tuvo nada que reparar, el archivo nunca
    /// existió abierto. ¿Puede fallar? Sí: con `std::fs::write` en `nacer_cerrado` nace en 644 y
    /// esto es rojo (bitácora del sprint 003).
    #[test]
    fn el_archivo_nace_cerrado_y_su_carpeta_tambien() {
        let d = carpeta("nace");
        let ruta = d.join("prefs.json");
        escribir(&ruta, b"{}").unwrap();
        assert_eq!(modo(&ruta), ARCHIVO);
        assert!(!cerrar_permisos(&ruta, ARCHIVO).unwrap(), "hubo que cerrarlo: nació abierto");
        assert_eq!(modo(&d), CARPETA, "la carpeta quedó legible por otras cuentas del Mac");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// **Nunca desaparece a medias** (M10 del S2): cada escritura deja lo último, cerrado y sin
    /// temporal al lado.
    #[test]
    fn se_reemplaza_sin_desaparecer_y_sin_dejar_el_temporal() {
        let d = carpeta("reemplaza");
        let ruta = d.join("costo-del-mes.json");
        for texto in ["uno", "dos"] {
            escribir(&ruta, texto.as_bytes()).unwrap();
            assert_eq!(std::fs::read_to_string(&ruta).unwrap(), texto);
            assert!(!temporal_de(&ruta).exists(), "quedó el temporal");
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    /// Un temporal que dejó una caída anterior no bloquea la escritura siguiente (el `create_new`
    /// fallaría sobre él).
    #[test]
    fn un_temporal_viejo_no_bloquea() {
        let d = carpeta("viejo");
        let ruta = d.join("bandeja.ghost");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::write(temporal_de(&ruta), b"resto de una caida").unwrap();
        escribir(&ruta, b"nuevo").unwrap();
        assert_eq!(std::fs::read(&ruta).unwrap(), b"nuevo");
        let _ = std::fs::remove_dir_all(&d);
    }

    /// **La carpeta floja se repara** (regla 17-bis): una versión anterior pudo dejarla en 755.
    #[test]
    fn la_carpeta_floja_se_repara() {
        let d = carpeta("floja");
        std::fs::create_dir_all(&d).unwrap();
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o755)).unwrap();
        escribir(&d.join("x"), b"x").unwrap();
        assert_eq!(modo(&d), CARPETA);
        let _ = std::fs::remove_dir_all(&d);
    }
}
