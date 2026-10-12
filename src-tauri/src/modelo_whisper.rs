//! **El modelo de whisper: dónde vive, cómo se baja y cómo se comprueba** (ADR 022 §4 y §5).
//!
//! No es un módulo protegido, y a propósito: el transcriptor (`stt/whisper.rs`) sí lo es —no abre archivos ni red—,
//! así que todo lo que toca el disco o la red por el modelo vive aquí, en la capa que no ve un solo turno. Lo que se
//! baja es el modelo, no nada tuyo: entra el modelo, no sale texto ni audio.
//!
//! **La única puerta a la red escrita en Rust** (`contador-de-red` la cuenta: exactamente una), y solo en Windows. Se
//! abre cuando pulsas «Instalar» en Idioma, jamás durante una reunión, y lo bajado se compara con la huella SHA-256
//! fija en este archivo: si no coincide, se borra y no se usa.

use std::path::{Path, PathBuf};
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
/// Lo que llega de la red lo escribe [`descargar_a`]; esta función solo pone la puerta. Ver ADR 022 §4.
#[cfg(windows)]
pub fn bajar(m: &Modelo) -> Result<PathBuf, String> {
    crate::hardware::vigilar("la descarga del modelo de whisper");
    if BAJANDO.swap(true, Ordering::SeqCst) {
        return Err("ya se está bajando el modelo".into());
    }
    let hecho = (|| -> Result<PathBuf, String> {
        let destino = ruta(m).ok_or("no se encontró la carpeta local de la app (LOCALAPPDATA)")?;
        if entero(&destino, m) {
            return Ok(destino);
        }
        println!("[modelo] bajando {} ({} bytes)", m.nombre, m.bytes);
        let respuesta = ureq::get(m.origen).call().map_err(|e| format!("la descarga no empezó: {e}"))?; // verify-ephemeral:allow — ADR 022 §4: la única puerta Rust, el modelo de whisper, con su huella
        descargar_a(respuesta.into_body().into_reader(), m, &destino)
    })();
    BAJANDO.store(false, Ordering::SeqCst);
    hecho
}

/// **Escribe un modelo que llega de un lector y lo comprueba**: tamaño exacto y huella SHA-256 fija. Si algo no
/// coincide, o la lectura se corta, el `.parcial` se borra y no queda destino: un modelo a medias no se usa nunca
/// (ADR 022 §4). Vive fuera de Windows a propósito, para que la CI del Mac pruebe lo que escribe la descarga.
pub fn descargar_a(mut lector: impl std::io::Read, m: &Modelo, destino: &Path) -> Result<PathBuf, String> {
    use sha2::{Digest, Sha256};
    use std::io::Write;

    let carpeta = destino.parent().ok_or("ruta del modelo sin carpeta")?;
    std::fs::create_dir_all(carpeta).map_err(|e| format!("no se pudo crear la carpeta del modelo: {e}"))?;
    let parcial = destino.with_extension("parcial");
    let escrito = (|| -> Result<(), String> {
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
            return Err(format!("el modelo bajado no es el esperado ({total} bytes, huella distinta)"));
        }
        Ok(())
    })();
    match escrito {
        Ok(()) => {
            std::fs::rename(&parcial, destino).map_err(|e| format!("no se pudo dejar el modelo en su sitio: {e}"))?;
            println!("[modelo] {} listo: tamaño y huella comprobados", m.nombre);
            Ok(destino.to_path_buf())
        }
        Err(e) => {
            // B5 de la auditoría: un corte a mitad no deja un parcial de 1,6 GB en disco.
            let _ = std::fs::remove_file(&parcial);
            Err(format!("{e}: borrado"))
        }
    }
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

    /// Una huella de prueba: la de `hola` (4 bytes) y el modelo falso que la lleva.
    fn modelo_de_prueba(bytes: u64, texto: &[u8]) -> Modelo {
        use sha2::{Digest, Sha256};
        let hex: String = Sha256::digest(texto).iter().map(|b| format!("{b:02x}")).collect();
        let sha = Box::leak(hex.into_boxed_str());
        Modelo { nombre: "prueba", archivo: "prueba.bin", bytes, sha256: sha, origen: "https://prueba.invalid/prueba.bin" }
    }

    fn carpeta_de_prueba(nombre: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!("ag-descarga-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        base
    }

    /// Un lector que da unos bytes y luego se corta, como una red que se cae a mitad.
    struct SeCorta(usize);
    impl std::io::Read for SeCorta {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            if self.0 == 0 {
                return Err(std::io::Error::other("cortado"));
            }
            let n = self.0.min(buf.len()).min(2);
            buf[..n].fill(b'h');
            self.0 -= n;
            Ok(n)
        }
    }

    /// **Lo bueno pasa**: el tamaño y la huella exactos dejan el modelo en su sitio, sin `.parcial`.
    #[test]
    fn una_descarga_exacta_queda_en_su_sitio() {
        let base = carpeta_de_prueba("exacta");
        let m = modelo_de_prueba(4, b"hola");
        let destino = base.join("modelos").join("prueba.bin");
        let r = descargar_a(&b"hola"[..], &m, &destino).expect("una descarga exacta debe pasar");
        assert_eq!(r, destino);
        assert_eq!(std::fs::read(&destino).unwrap(), b"hola");
        assert!(!destino.with_extension("parcial").exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    /// **Un byte de más aborta**: no queda ni el parcial ni el destino.
    #[test]
    fn un_byte_de_mas_aborta_y_no_deja_nada() {
        let base = carpeta_de_prueba("mas");
        let m = modelo_de_prueba(4, b"hola");
        let destino = base.join("modelos").join("prueba.bin");
        assert!(descargar_a(&b"holaX"[..], &m, &destino).is_err());
        assert!(!destino.exists(), "un modelo de más se quedó como bueno");
        assert!(!destino.with_extension("parcial").exists(), "el parcial se quedó en disco");
        let _ = std::fs::remove_dir_all(&base);
    }

    /// **Una huella distinta con el mismo tamaño se borra**: no es el modelo publicado.
    #[test]
    fn una_huella_distinta_se_borra() {
        let base = carpeta_de_prueba("huella");
        let m = modelo_de_prueba(4, b"hola");
        let destino = base.join("modelos").join("prueba.bin");
        assert!(descargar_a(&b"chao"[..], &m, &destino).is_err());
        assert!(!destino.exists());
        assert!(!destino.with_extension("parcial").exists());
        let _ = std::fs::remove_dir_all(&base);
    }

    /// **Una descarga que se corta a mitad no deja nada** (B5 de la auditoría del S6).
    #[test]
    fn una_descarga_cortada_no_deja_parcial() {
        let base = carpeta_de_prueba("cortada");
        let m = modelo_de_prueba(4, b"hola");
        let destino = base.join("modelos").join("prueba.bin");
        assert!(descargar_a(SeCorta(3), &m, &destino).is_err());
        assert!(!destino.exists());
        assert!(!destino.with_extension("parcial").exists(), "el parcial sobrevivió al corte");
        let _ = std::fs::remove_dir_all(&base);
    }
}
