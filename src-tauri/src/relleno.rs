//! EL RELLENO — lo único que la captura encuentra donde está la banda.
//!
//! La banda lleva el flag que la borra de cualquier grabación, así que en una pantalla compartida
//! su franja mostraría **lo que haya detrás**. El relleno ocupa ese mismo rectángulo sin el flag.
//!
//! El diseño eligió **el fondo de escritorio del usuario**, con negro a una tecla. Este módulo
//! resuelve la primera mitad: lee la imagen del sistema y se la pasa al relleno. Y declara sus
//! límites en vez de fingir que no los tiene:
//!
//! - **Solo la pantalla principal.** `NSScreen::mainScreen` — un segundo monitor con otro fondo
//!   queda para cuando exista la banda por monitor.
//! - **Solo «Rellenar pantalla».** Es el modo por defecto de macOS y el que el relleno reproduce
//!   (`cover` + centrado). Con «Ajustar» o mosaico, el recorte no encajaría.
//! - **Entre [`SUELO`] y [`TECHO`].** Por arriba, porque un fondo dinámico de macOS es un HEIC de
//!   decenas de MB con todas las horas del día dentro y no se va a mover eso entero por el IPC
//!   para pintar 88 px. Por abajo, porque **macOS a veces no da el fondo y lo disimula**: cuando
//!   el usuario tiene un fondo dinámico o un salvapantallas aéreo, `desktopImageURL` devuelve
//!   `/System/Library/CoreServices/DefaultDesktop.heic`, que en este Mac son **54 bytes** — un
//!   marcador de posición, no una imagen. Sin suelo, esos 54 bytes viajan como `data:` válido,
//!   el webview no los sabe decodificar y la franja acaba negra igual… pero por un camino que
//!   nadie registró. El suelo convierte ese silencio en una línea de log.
//!
//! **En cualquier fallo, negro.** No es un modo degradado: es la otra opción que el diseño
//! aprobó, y la única que no filtra nada. Un relleno que no se puede pintar no puede dejar la
//! franja transparente.

use base64::Engine;
use std::path::Path;

/// Por encima de esto no se intenta: ver la nota del módulo.
pub const TECHO: u64 = 12 * 1024 * 1024;

/// Por debajo de esto no es una imagen, es un marcador de posición del sistema. Ver la nota del
/// módulo: el `DefaultDesktop.heic` de macOS pesa 54 bytes.
pub const SUELO: u64 = 4 * 1024;

/// El nombre del evento que lleva la imagen a la ventana del relleno.
pub const EVENTO: &str = "fondo-de-escritorio";

/// El tipo MIME por extensión. Deliberadamente corto: lo que macOS usa de fondo y el webview
/// sabe pintar. Una extensión desconocida devuelve `None` y la franja se queda negra — adivinar
/// el MIME de un archivo que no reconocemos es cómo se pinta un rectángulo roto.
pub fn mime(ruta: &Path) -> Option<&'static str> {
    let ext = ruta.extension()?.to_str()?.to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => Some("image/jpeg"),
        "png" => Some("image/png"),
        "heic" | "heif" => Some("image/heic"),
        "tif" | "tiff" => Some("image/tiff"),
        _ => None,
    }
}

/// El resultado de intentar leer el fondo. El motivo viaja porque «se quedó negro» sin motivo no
/// se puede depurar: el techo, la extensión y el permiso fallan igual de silenciosos.
pub enum Fondo {
    Imagen(String),
    Negro(String),
}

/// Lee el fondo de escritorio y lo devuelve como `data:` listo para CSS.
pub fn leer(ruta: &Path) -> Fondo {
    let Some(mime) = mime(ruta) else {
        return Fondo::Negro(format!(
            "el fondo «{}» no es un formato que sepamos pintar",
            ruta.display()
        ));
    };
    let peso = match std::fs::metadata(ruta) {
        Ok(m) => m.len(),
        Err(e) => return Fondo::Negro(format!("no se pudo leer el fondo: {e}")),
    };
    if peso > TECHO {
        return Fondo::Negro(format!(
            "el fondo pesa {} MB (techo {} MB): se queda negro",
            peso / 1024 / 1024,
            TECHO / 1024 / 1024
        ));
    }
    if peso < SUELO {
        return Fondo::Negro(format!(
            "«{}» pesa {peso} bytes: es el marcador de posición de macOS, no un fondo — \
             el sistema no expone la imagen real (fondo dinámico o aéreo). Se queda negro",
            ruta.display()
        ));
    }
    match std::fs::read(ruta) {
        Ok(bytes) => Fondo::Imagen(format!(
            "data:{mime};base64,{}",
            base64::engine::general_purpose::STANDARD.encode(&bytes)
        )),
        Err(e) => Fondo::Negro(format!("no se pudo leer el fondo: {e}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn el_mime_sale_de_la_extension_y_lo_desconocido_no_se_adivina() {
        assert_eq!(mime(&PathBuf::from("/a/b.HEIC")), Some("image/heic"));
        assert_eq!(mime(&PathBuf::from("/a/b.jpg")), Some("image/jpeg"));
        assert_eq!(mime(&PathBuf::from("/a/b.webp")), None);
        assert_eq!(mime(&PathBuf::from("/a/sin-extension")), None);
    }

    #[test]
    fn un_fondo_que_no_existe_deja_la_franja_negra_con_su_motivo() {
        let Fondo::Negro(motivo) = leer(&PathBuf::from("/no/existe.jpg")) else {
            panic!("un archivo ausente tenía que dar negro")
        };
        assert!(!motivo.is_empty(), "negro sin motivo no se puede depurar");
    }

    #[test]
    fn un_fondo_de_formato_desconocido_no_se_intenta_leer_siquiera() {
        assert!(matches!(
            leer(&PathBuf::from("/System/Library/CoreServices/x.mov")),
            Fondo::Negro(_)
        ));
    }

    /// El techo existe para que un fondo dinámico de 80 MB no cruce el IPC. Se comprueba con un
    /// archivo de verdad porque el cálculo de tamaño es justo donde se cuela un error de unidad.
    #[test]
    fn un_fondo_por_encima_del_techo_se_queda_negro() {
        let ruta = std::env::temp_dir().join(format!("ag-fondo-{}.png", std::process::id()));
        std::fs::write(&ruta, vec![0u8; (TECHO + 1) as usize]).unwrap();
        let Fondo::Negro(motivo) = leer(&ruta) else {
            panic!("por encima del techo tenía que dar negro")
        };
        assert!(motivo.contains("techo"), "motivo poco claro: {motivo}");
        let _ = std::fs::remove_file(&ruta);
    }

    /// El caso que se encontró EN VIVO: macOS devuelve `DefaultDesktop.heic` de 54 bytes cuando
    /// el usuario tiene un fondo dinámico. Sin suelo eso pasaba por imagen buena, salía como
    /// `data:` válido, el webview no lo decodificaba y la franja quedaba negra **sin que nada lo
    /// dijera**. Un fallo silencioso que se ve exactamente igual que un acierto.
    #[test]
    fn el_marcador_de_posicion_de_macos_no_pasa_por_fondo() {
        let ruta = std::env::temp_dir().join(format!("ag-fondo-ph-{}.heic", std::process::id()));
        std::fs::write(&ruta, vec![0u8; 54]).unwrap();
        let Fondo::Negro(motivo) = leer(&ruta) else {
            panic!("54 bytes no son un fondo de escritorio")
        };
        assert!(motivo.contains("54 bytes"), "motivo poco claro: {motivo}");
        let _ = std::fs::remove_file(&ruta);
    }

    #[test]
    fn un_fondo_normal_sale_como_data_url_con_su_mime() {
        let ruta = std::env::temp_dir().join(format!("ag-fondo-ok-{}.jpg", std::process::id()));
        std::fs::write(&ruta, [b"\xff\xd8\xff\xe0".as_slice(), &vec![7u8; 8192]].concat()).unwrap();
        let Fondo::Imagen(url) = leer(&ruta) else {
            panic!("tenía que salir imagen")
        };
        assert!(url.starts_with("data:image/jpeg;base64,"), "{}", &url[..40]);
        let _ = std::fs::remove_file(&ruta);
    }
}
