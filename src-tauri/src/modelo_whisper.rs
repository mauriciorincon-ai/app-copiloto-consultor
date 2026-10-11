//! **El modelo de whisper: dónde vive, cómo se baja y cómo se comprueba** (ADR 022 §4 y §5).
//!
//! No es un módulo protegido, y a propósito: el transcriptor (`stt/whisper.rs`) sí lo es —no abre archivos ni red—,
//! así que todo lo que toca el disco o la red por el modelo vive aquí, en la capa que no ve un solo turno. Lo que se
//! baja es el modelo, no nada tuyo: entra el modelo, no sale texto ni audio.
//!
//! **La única puerta a la red escrita en Rust** (`contador-de-red` la cuenta: exactamente una), y solo en Windows. Se
//! abre cuando pulsas «Instalar» en Idioma, jamás durante una reunión, y lo bajado se compara con la huella SHA-256
//! fija en este archivo: si no coincide, se borra y no se usa.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// Un modelo de whisper, con lo que hace falta para bajarlo y comprobarlo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Modelo {
    /// Cómo lo enseña la app.
    pub nombre: &'static str,
    pub archivo: &'static str,
    /// Lo que mide entero, en bytes.
    pub bytes: u64,
    /// Su huella, leída del puntero LFS público del modelo el 2026-10-11.
    pub sha256: &'static str,
    pub origen: &'static str,
}

/// El modelo de la app en Windows (ADR 006: large-v3-turbo). 1,6 GB.
pub const GRANDE: Modelo = Modelo {
    nombre: "large-v3-turbo",
    archivo: "ggml-large-v3-turbo.bin",
    bytes: 1_624_555_275,
    sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
    origen: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin",
};

/// El escalón para cuando la calibración pasa de 4 s en tu PC (ADR 022 §5). 0,5 GB.
pub const PEQUENO: Modelo = Modelo {
    nombre: "small",
    archivo: "ggml-small.bin",
    bytes: 487_601_967,
    sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
    origen: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin",
};

/// La carpeta de los modelos, dentro de la carpeta privada de la app.
pub const CARPETA: &str = "modelos";

/// ¿Se está bajando ahora? Una descarga a la vez.
static BAJANDO: AtomicBool = AtomicBool::new(false);

/// Dónde vive el modelo. En Windows, `%LOCALAPPDATA%\<id>\modelos\`: la misma carpeta privada que `carpeta_privada`
/// en `lib.rs` (ADR 022 §2), calculada sin Tauri porque el motor de transcripción no tiene una mano de la app.
pub fn ruta(m: &Modelo) -> Option<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA")?;
    Some(ruta_en(PathBuf::from(base).join(crate::puerta::IDENTIFICADOR), m))
}

/// La ruta del modelo dentro de una carpeta de la app dada.
pub fn ruta_en(carpeta_de_la_app: PathBuf, m: &Modelo) -> PathBuf {
    carpeta_de_la_app.join(CARPETA).join(m.archivo)
}

/// El modelo, si está **entero**. Se mira el tamaño: la huella se comprueba al bajarlo, y rehacerla en cada arranque
/// serían 1,6 GB de lectura.
pub fn listo(m: &Modelo) -> Option<PathBuf> {
    let r = ruta(m)?;
    entero(&r, m).then_some(r)
}

fn entero(ruta: &std::path::Path, m: &Modelo) -> bool {
    std::fs::metadata(ruta).is_ok_and(|d| d.is_file() && d.len() == m.bytes)
}

/// ¿Se está bajando?
pub fn bajando() -> bool {
    BAJANDO.load(Ordering::Relaxed)
}

/// La huella de unos bytes, en hexadecimal. La comparación es contra el texto fijo de arriba.
pub fn coincide(calculada: &[u8], m: &Modelo) -> bool {
    hex(calculada) == m.sha256
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// **Baja el modelo**, lo comprueba y lo deja listo. Solo en Windows y solo si lo pides (Idioma, «Instalar»).
///
/// Se baja a `<archivo>.parcial`; si el tamaño o la huella no coinciden, se borra y no se usa; si coinciden, se renombra.
#[cfg(windows)]
pub fn bajar(m: &Modelo) -> Result<PathBuf, String> {
    use sha2::{Digest, Sha256};
    use std::io::{Read, Write};

    crate::hardware::vigilar("la descarga del modelo de whisper");
    if BAJANDO.swap(true, Ordering::SeqCst) {
        return Err("ya se está bajando el modelo".into());
    }
    let hecho = (|| -> Result<PathBuf, String> {
        let destino = ruta(m).ok_or("no se encontró la carpeta local de la app (LOCALAPPDATA)")?;
        if entero(&destino, m) {
            return Ok(destino);
        }
        let carpeta = destino.parent().ok_or("ruta del modelo sin carpeta")?;
        std::fs::create_dir_all(carpeta).map_err(|e| format!("no se pudo crear la carpeta del modelo: {e}"))?;
        let parcial = destino.with_extension("parcial");
        println!("[modelo] bajando {} ({} bytes)", m.nombre, m.bytes);
        let respuesta = ureq::get(m.origen).call().map_err(|e| format!("la descarga no empezó: {e}"))?; // verify-ephemeral:allow — ADR 022 §4: la única puerta Rust, el modelo de whisper, con su huella
        let mut lector = respuesta.into_body().into_reader();
        let mut archivo = std::fs::File::create(&parcial).map_err(|e| format!("no se pudo crear el archivo: {e}"))?;
        let mut huella = Sha256::new();
        let mut total: u64 = 0;
        let mut trozo = vec![0u8; 1 << 20];
        loop {
            let n = lector.read(&mut trozo).map_err(|e| format!("la descarga se cortó: {e}"))?;
            if n == 0 {
                break;
            }
            total += n as u64;
            if total > m.bytes {
                break;
            }
            huella.update(&trozo[..n]);
            archivo.write_all(&trozo[..n]).map_err(|e| format!("no se pudo escribir el modelo: {e}"))?;
        }
        archivo.sync_all().map_err(|e| format!("no se pudo cerrar el modelo: {e}"))?;
        drop(archivo);
        if total != m.bytes || !coincide(&huella.finalize(), m) {
            let _ = std::fs::remove_file(&parcial);
            return Err(format!("el modelo bajado no es el esperado ({total} bytes, huella distinta): borrado"));
        }
        std::fs::rename(&parcial, &destino).map_err(|e| format!("no se pudo dejar el modelo en su sitio: {e}"))?;
        println!("[modelo] {} listo: tamaño y huella comprobados", m.nombre);
        Ok(destino)
    })();
    BAJANDO.store(false, Ordering::SeqCst);
    hecho
}

/// Fuera de Windows no se baja nada: macOS transcribe con su propio motor (ADR 006).
#[cfg(not(windows))]
pub fn bajar(_m: &Modelo) -> Result<PathBuf, String> {
    Err("el modelo de whisper solo se usa en Windows (ADR 022)".into())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Las huellas son las del puntero LFS público (64 caracteres hexadecimales) y los tamaños, los suyos.
    #[test]
    fn las_huellas_y_los_tamanos_son_los_publicados() {
        for m in [GRANDE, PEQUENO] {
            assert_eq!(m.sha256.len(), 64, "{}", m.nombre);
            assert!(m.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()), "{}", m.nombre);
            assert!(m.origen.ends_with(m.archivo), "{}: el origen no baja el archivo que dice", m.nombre);
            assert!(m.origen.starts_with("https://huggingface.co/ggerganov/whisper.cpp/"), "{}", m.nombre);
        }
        assert_eq!(GRANDE.bytes, 1_624_555_275);
        assert_eq!(PEQUENO.bytes, 487_601_967);
    }

    /// Una huella distinta no coincide: el archivo bajado se tira.
    #[test]
    fn solo_coincide_la_huella_fija() {
        let mut buena = [0u8; 32];
        for (i, par) in GRANDE.sha256.as_bytes().chunks(2).enumerate() {
            buena[i] = u8::from_str_radix(std::str::from_utf8(par).unwrap(), 16).unwrap();
        }
        assert!(coincide(&buena, &GRANDE));
        let mut mala = buena;
        mala[31] ^= 1;
        assert!(!coincide(&mala, &GRANDE));
        assert!(!coincide(&buena, &PEQUENO));
    }

    /// El modelo vive en la carpeta privada de la app, en `modelos/`, y solo cuenta si está entero.
    #[test]
    fn vive_en_la_carpeta_de_la_app_y_cuenta_solo_entero() {
        let base = std::env::temp_dir().join(format!("ag-modelo-{}", std::process::id()));
        let r = ruta_en(base.clone(), &PEQUENO);
        assert_eq!(r, base.join("modelos").join("ggml-small.bin"));
        std::fs::create_dir_all(r.parent().unwrap()).unwrap();
        std::fs::write(&r, b"a medias").unwrap();
        assert!(!entero(&r, &PEQUENO), "un modelo a medias contó como listo");
        let _ = std::fs::remove_dir_all(&base);
    }
}
