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

/// La revisión del repositorio de modelos desde la que se baja (sprint 008, B12 del S7). Antes se bajaba de `main`, una
/// referencia que se mueve: la huella protegía la integridad, no la disponibilidad —si el repositorio cambiaba el
/// archivo, la descarga fallaba por una huella que ya no coincidía—. Con la revisión fija la URL dice siempre lo
/// mismo; verificada el 2026-10-12 contra el árbol de esa revisión: el tamaño y la huella de cada archivo son los de
/// abajo.
pub const REVISION: &str = "5359861c739e955e79d9a303bcbc70fb988958b1";

/// El modelo de la app en Windows x64 (ADR 006: large-v3-turbo). 1,6 GB. En un PC ARM64 «Instalar» baja el pequeño,
/// porque este no cabe en 4 s (ADR 022, enmienda 3; `stt::whisper::por_defecto`).
pub const GRANDE: Modelo = Modelo {
    nombre: "large-v3-turbo",
    archivo: "ggml-large-v3-turbo.bin",
    bytes: 1_624_555_275,
    sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
    origen: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-large-v3-turbo.bin",
};

/// El pequeño, 0,5 GB: el escalón para cuando la calibración pasa de 4 s en tu PC (ADR 022 §5) y, desde el sprint 009,
/// el que «Instalar» baja en un PC ARM64 de Windows (ADR 022, enmienda 3).
pub const PEQUENO: Modelo = Modelo {
    nombre: "small",
    archivo: "ggml-small.bin",
    bytes: 487_601_967,
    sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
    origen: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-small.bin",
};

/// La carpeta de los modelos, dentro de la carpeta privada de la app.
pub const CARPETA: &str = "modelos";

/// ¿Se está bajando ahora? Una descarga a la vez.
static BAJANDO: AtomicBool = AtomicBool::new(false);

/// **Cuánto puede esperar cada fase de la descarga** (sprint 008, M1 del S6). Sin esto una conexión colgada dejaba
/// `BAJANDO` activo hasta reiniciar la app: ni se podía reintentar ni se podía decir qué pasaba. Conectar y recibir la
/// primera respuesta son segundos; el cuerpo entero tiene un techo que deja bajar 1,6 GB a una línea mala (ver la
/// prueba) pero no para siempre.
pub const TECHO_DE_CONECTAR: std::time::Duration = std::time::Duration::from_secs(20);
pub const TECHO_DE_LA_RESPUESTA: std::time::Duration = std::time::Duration::from_secs(60);
pub const TECHO_DE_LA_DESCARGA: std::time::Duration = std::time::Duration::from_secs(3 * 3600);

/// **Quién decide que hay que parar una descarga en curso** (sprint 008, M2 del S6). La guarda «nunca en reunión» se
/// miraba solo al empezar: si la reunión empezaba a mitad de los 1,6 GB, la descarga seguía ocupando la red y el
/// procesador. `lib.rs` pone aquí, al arrancar, la pregunta «¿hay algo en marcha?»; cada trozo de la descarga la hace.
static GUARDA: std::sync::OnceLock<Box<dyn Fn() -> bool + Send + Sync>> = std::sync::OnceLock::new();

/// Pone la pregunta que corta una descarga. Una sola vez, al arrancar.
pub fn poner_la_guarda(hay_que_parar: impl Fn() -> bool + Send + Sync + 'static) {
    let _ = GUARDA.set(Box::new(hay_que_parar));
}

fn hay_que_parar() -> bool {
    GUARDA.get().is_some_and(|f| f())
}

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

/// **¿Cabe el modelo en el disco?** (sprint 008, B5 del S6): el tamaño del modelo más un 10 % de aire, que el sistema
/// necesita para no quedarse sin espacio a medias. Pura.
pub fn cabe(libre: u64, m: &Modelo) -> bool {
    libre >= m.bytes + m.bytes / 10
}

/// Cuánto espacio libre tiene el disco de `carpeta` para este usuario. `None` si Windows no contesta (entonces se baja
/// y, si falta, la escritura falla y el `.parcial` se borra).
#[cfg(windows)]
fn espacio_libre(carpeta: &Path) -> Option<u64> {
    use windows::core::HSTRING;
    use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
    let mut libre = 0u64;
    // SAFETY: el nombre es una cadena ancha que vive durante la llamada y `libre` es un entero propio.
    unsafe { GetDiskFreeSpaceExW(&HSTRING::from(carpeta.as_os_str()), Some(&mut libre), None, None) }.ok()?;
    Some(libre)
}

/// **Borra los `.parcial` que dejó una caída** (sprint 008, B5 del S6): un corte en plena descarga los borra solo si la
/// app llega a ver el error; si el proceso muere, queda un parcial de hasta 1,6 GB. Al arrancar se borran. Devuelve
/// cuántos.
pub fn limpiar_parciales(carpeta: &Path) -> usize {
    let Ok(entradas) = std::fs::read_dir(carpeta) else { return 0 };
    entradas
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "parcial"))
        .filter(|e| std::fs::remove_file(e.path()).is_ok())
        .count()
}

/// Lo que se hace al arrancar: limpiar los parciales de la carpeta de modelos de este usuario.
pub fn limpiar_lo_que_quedo() {
    let Some(ruta) = ruta(&PEQUENO) else { return };
    let Some(carpeta) = ruta.parent() else { return };
    let n = limpiar_parciales(carpeta);
    if n > 0 {
        println!("[modelo] {n} descarga(s) a medias de una caída anterior, borradas");
    }
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
        if let Some(libre) = destino.parent().and_then(|c| {
            std::fs::create_dir_all(c).ok()?;
            espacio_libre(c)
        }) {
            if !cabe(libre, m) {
                return Err(format!(
                    "no hay espacio en el disco: el modelo pide {} MB y quedan {} MB libres",
                    m.bytes / 1_000_000,
                    libre / 1_000_000
                ));
            }
        }
        println!("[modelo] bajando {} ({} bytes)", m.nombre, m.bytes);
        let agente: ureq::Agent = ureq::Agent::config_builder().timeout_connect(Some(TECHO_DE_CONECTAR)).timeout_recv_response(Some(TECHO_DE_LA_RESPUESTA)).timeout_global(Some(TECHO_DE_LA_DESCARGA)).build().into(); // verify-ephemeral:allow — ADR 022 §4: la única puerta Rust, el modelo de whisper, con su huella y sus techos de tiempo
        let respuesta = agente.get(m.origen).call().map_err(|e| format!("la descarga no empezó: {e}"))?;
        descargar_a(respuesta.into_body().into_reader(), m, &destino)
    })();
    BAJANDO.store(false, Ordering::SeqCst);
    hecho
}

/// **Escribe un modelo que llega de un lector y lo comprueba**: tamaño exacto y huella SHA-256 fija. Si algo no
/// coincide, o la lectura se corta, el `.parcial` se borra y no queda destino: un modelo a medias no se usa nunca
/// (ADR 022 §4). Vive fuera de Windows a propósito, para que la CI del Mac pruebe lo que escribe la descarga.
pub fn descargar_a(lector: impl std::io::Read, m: &Modelo, destino: &Path) -> Result<PathBuf, String> {
    descargar_con_guarda(lector, m, destino, &hay_que_parar)
}

/// Lo mismo, con la pregunta «¿hay que parar?» como parámetro: se hace antes de cada trozo, y si dice que sí la
/// descarga se corta, el `.parcial` se borra y el motivo lo dice (M2 del S6).
pub fn descargar_con_guarda(
    mut lector: impl std::io::Read,
    m: &Modelo,
    destino: &Path,
    parar: &dyn Fn() -> bool,
) -> Result<PathBuf, String> {
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
            if parar() {
                return Err("la descarga se paró porque empezó una reunión o un ensayo; vuelve a pedirla al terminar".into());
            }
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
            // B12 del S7: la URL lleva la revisión fija, no `main`, que se mueve.
            assert!(m.origen.contains(&format!("/resolve/{REVISION}/")), "{}: el origen no usa la revisión fija", m.nombre);
            assert!(!m.origen.contains("/resolve/main/"), "{}", m.nombre);
        }
        assert_eq!(GRANDE.bytes, 1_624_555_275);
        assert_eq!(PEQUENO.bytes, 487_601_967);
    }

    /// **El espacio en disco y los parciales de una caída** (sprint 008, B5 del S6).
    #[test]
    fn el_modelo_pide_espacio_y_los_parciales_de_una_caida_se_borran() {
        assert!(!cabe(GRANDE.bytes, &GRANDE), "justo el tamaño no alcanza: falta el aire");
        assert!(cabe(GRANDE.bytes + GRANDE.bytes / 10, &GRANDE));
        assert!(cabe(u64::MAX, &PEQUENO));
        let carpeta = std::env::temp_dir().join(format!("ag-parciales-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&carpeta);
        std::fs::create_dir_all(&carpeta).unwrap();
        std::fs::write(carpeta.join("ggml-small.parcial"), b"a medias").unwrap();
        std::fs::write(carpeta.join("ggml-large-v3-turbo.parcial"), b"a medias").unwrap();
        std::fs::write(carpeta.join("ggml-small.bin"), b"entero").unwrap();
        assert_eq!(limpiar_parciales(&carpeta), 2);
        assert!(carpeta.join("ggml-small.bin").exists(), "un modelo entero no se toca");
        assert!(!carpeta.join("ggml-small.parcial").exists());
        assert_eq!(limpiar_parciales(&carpeta.join("no-existe")), 0);
        let _ = std::fs::remove_dir_all(&carpeta);
    }

    /// **La descarga se corta si empieza una reunión** (sprint 008, M2 del S6): no queda destino ni parcial.
    #[test]
    fn una_descarga_se_para_si_empieza_una_reunion() {
        let carpeta = std::env::temp_dir().join(format!("ag-guarda-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&carpeta);
        let destino = carpeta.join("m.bin");
        let m = Modelo { nombre: "t", archivo: "m.bin", bytes: 4, sha256: "x", origen: "https://huggingface.co/ggerganov/whisper.cpp/resolve/x/m.bin" };
        let llamadas = std::cell::Cell::new(0);
        // La reunión empieza antes del segundo trozo.
        let error = descargar_con_guarda(&b"hola"[..], &m, &destino, &|| {
            llamadas.set(llamadas.get() + 1);
            llamadas.get() > 1
        });
        assert!(llamadas.get() >= 2, "la guarda se pregunta antes de cada trozo, no solo al empezar");
        assert!(error.is_err());
        assert!(error.unwrap_err().contains("empezó una reunión"));
        assert!(!destino.exists() && !destino.with_extension("parcial").exists(), "no queda nada a medias");
        let _ = std::fs::remove_dir_all(&carpeta);
    }

    /// **Los techos de tiempo** (M1 del S6): conectar y la primera respuesta son segundos, y el techo del cuerpo deja
    /// bajar el modelo grande a una línea de 1,5 Mbit/s sin que una conexión colgada ocupe `BAJANDO` para siempre.
    #[test]
    fn los_techos_de_la_descarga_dejan_bajar_a_una_linea_mala() {
        assert!(TECHO_DE_CONECTAR.as_secs() <= 30 && TECHO_DE_LA_RESPUESTA.as_secs() <= 90);
        let bits = GRANDE.bytes * 8;
        assert!(bits / TECHO_DE_LA_DESCARGA.as_secs() <= 1_500_000, "el techo no alcanza para una línea de 1,5 Mbit/s");
        assert!(TECHO_DE_LA_DESCARGA.as_secs() <= 4 * 3600, "un techo de más de cuatro horas es no tener techo");
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
