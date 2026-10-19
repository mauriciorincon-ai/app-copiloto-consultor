//! **El modelo local de la sugerencia en Windows: dónde vive, cómo se baja, cuándo se carga y cuándo se suelta**
//! (sprint 014, ADR 023 enmienda 2).
//!
//! No es un módulo protegido, y a propósito, como `modelo_whisper.rs`: aquí está todo lo que toca el disco por el modelo
//! y no se ve un solo turno. La generación —donde sí entra el turno— vive en `sintesis/local.rs`, que es protegido.
//!
//! **Sin una puerta a la red nueva.** El GGUF baja por `modelo_whisper::bajar`, la única línea de red escrita en Rust
//! (`contador-de-red` sigue contando una): origen fijo con la revisión del repositorio, tamaño exacto, huella SHA-256,
//! espacio en disco, retomar tras un corte, y la guarda que la corta si empieza una reunión. Se baja solo cuando lo
//! pides en IA («Install the model»), jamás con una reunión abierta.
//!
//! **En memoria, lo que dura una sesión.** Se carga la primera vez que hace falta (o al empezar la sesión, si «Redactar
//! sugerencias» está encendido) y se suelta al terminar la sesión, con el corte y al apagar «Redactar». Ocupa ≈ 1,1 GB.

use crate::modelo_whisper::{self, Modelo};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// **El modelo** (leído el 2026-10-19 de la API de Hugging Face, en su revisión fija): Qwen2.5-1.5B-Instruct en GGUF,
/// cuantizado Q4_K_M, licencia apache-2.0. Medido en la Surface: mediana 2,1 s con 5 hilos, 12 de 17 sugerencias del
/// kit pasan `fundar`, cero conexiones del proceso (anexo del ADR 023).
pub const QWEN: Modelo = Modelo {
    nombre: "Qwen2.5-1.5B-Instruct Q4_K_M",
    archivo: "qwen2.5-1.5b-instruct-q4_k_m.gguf",
    bytes: 1_117_320_736,
    sha256: "6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e",
    origen: "https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/resolve/91cad51170dc346986eccefdc2dd33a9da36ead9/qwen2.5-1.5b-instruct-q4_k_m.gguf",
};

/// La revisión del repositorio del modelo: la URL dice siempre lo mismo.
pub const REVISION: &str = "91cad51170dc346986eccefdc2dd33a9da36ead9";

/// ¿Se está bajando ahora? Lo enseña IA; la puerta tiene su propio «una descarga a la vez».
static BAJANDO: AtomicBool = AtomicBool::new(false);

/// ¿Falló la última vez que se pidió? IA lo dice y deja volver a pedirlo; el motivo, al log.
static FALLO: AtomicBool = AtomicBool::new(false);

/// Lo que IA enseña del modelo local.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDelModeloLocal {
    /// ¿Existe en esta plataforma? Solo en Windows: en el Mac redacta el modelo del sistema. **No cruza**: la pantalla ya
    /// sabe en qué sistema está (`usePlataforma`), y un campo sin lector es un huérfano (`contrato-con-lectores`).
    #[serde(skip)]
    pub existe: bool,
    /// ¿Está entero en este equipo?
    pub instalado: bool,
    pub bajando: bool,
    /// ¿Falló la última descarga?
    pub fallo: bool,
    /// Lo que mide, en bytes: lo que dice «Install the model».
    pub bytes: u64,
}

/// El estado de ahora. Pura, salvo mirar el tamaño del archivo.
pub fn estado() -> EstadoDelModeloLocal {
    EstadoDelModeloLocal {
        existe: cfg!(windows),
        instalado: instalado(),
        bajando: BAJANDO.load(Ordering::Relaxed),
        fallo: FALLO.load(Ordering::Relaxed),
        bytes: QWEN.bytes,
    }
}

/// ¿Está entero en este equipo? Se mira el tamaño, como con whisper: la huella se comprobó al bajarlo. Fuera de Windows,
/// nunca.
pub fn instalado() -> bool {
    cfg!(windows) && ruta().is_some()
}

/// Dónde está, si está entero.
pub fn ruta() -> Option<PathBuf> {
    modelo_whisper::listo(&QWEN)
}

/// **Lo baja**, lo comprueba y lo deja listo, por la puerta de whisper. Bloquea: se llama en otro hilo. `al_empezar` se
/// llama ya marcado «bajando», para que IA lo diga.
pub fn bajar(al_empezar: impl FnOnce()) -> Result<PathBuf, String> {
    if BAJANDO.swap(true, Ordering::SeqCst) {
        return Err("ya se está bajando el modelo local".into());
    }
    al_empezar();
    let r = modelo_whisper::bajar(&QWEN);
    FALLO.store(r.is_err(), Ordering::SeqCst);
    BAJANDO.store(false, Ordering::SeqCst);
    r
}

#[cfg(windows)]
pub use cargado::{backend, cargado, soltar};

#[cfg(windows)]
mod cargado {
    use super::*;
    use crate::sintesis::PorQueNoRedacta;
    use llama_cpp_2::llama_backend::LlamaBackend;
    use llama_cpp_2::model::params::LlamaModelParams;
    use llama_cpp_2::model::LlamaModel;
    use std::sync::{Arc, Mutex, OnceLock};

    /// El modelo en memoria, si se cargó en esta sesión.
    static CARGADO: Mutex<Option<Arc<LlamaModel>>> = Mutex::new(None);

    /// El backend de llama.cpp: una vez por proceso, **con sus registros apagados** (lo que imprime puede llevar trozos
    /// del prompt, y el prompt lleva el turno del cliente; ADR 003). Solo pregunta por la CPU: no abre ningún dispositivo,
    /// y por eso no lleva la guarda de hardware (la lleva la carga del modelo).
    pub fn backend() -> &'static LlamaBackend {
        static B: OnceLock<LlamaBackend> = OnceLock::new();
        B.get_or_init(|| {
            let mut b = LlamaBackend::init().expect("el backend de llama.cpp ya estaba iniciado fuera de la app");
            b.void_logs();
            b
        })
    }

    /// **El modelo, cargado**: el de esta sesión, o se carga ahora (≈ 1 s en la Surface). Sin el archivo entero, el motivo.
    pub fn cargado() -> Result<Arc<LlamaModel>, PorQueNoRedacta> {
        let mut g = CARGADO.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(m) = g.as_ref() {
            return Ok(m.clone());
        }
        let ruta = ruta().ok_or(PorQueNoRedacta::SinModeloLocal)?;
        crate::hardware::vigilar("el modelo local");
        let reloj = std::time::Instant::now();
        let m = LlamaModel::load_from_file(backend(), &ruta, &LlamaModelParams::default()).map_err(|e| {
            println!("[modelo-local] no cargó: {e}");
            PorQueNoRedacta::NoDisponible
        })?;
        println!("[modelo-local] cargado en {} ms", reloj.elapsed().as_millis());
        let m = Arc::new(m);
        *g = Some(m.clone());
        Ok(m)
    }

    /// **Lo suelta de la memoria** (el corte, el final de la sesión, «Redactar» apagado). Una sugerencia en camino tiene
    /// su propia copia del `Arc` y lo suelta al terminar; lo que traiga se tira por la época. Devuelve si había algo.
    pub fn soltar() -> bool {
        let suelto = CARGADO.lock().unwrap_or_else(|e| e.into_inner()).take().is_some();
        if suelto {
            println!("[modelo-local] soltado de la memoria");
        }
        suelto
    }
}

/// Fuera de Windows no hay modelo local que soltar.
#[cfg(not(windows))]
pub fn soltar() -> bool {
    false
}

/// **El generador de esta sesión** para `sintesis::local::Local`: el modelo cargado (o el motivo), con la mitad de los
/// núcleos.
pub fn generador() -> Result<std::sync::Arc<dyn crate::sintesis::local::Generador>, crate::sintesis::PorQueNoRedacta> {
    #[cfg(windows)]
    {
        let nucleos = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
        let m = cargado()?;
        Ok(std::sync::Arc::new(crate::sintesis::local::Llama::nuevo(m, crate::sintesis::local::hilos_para(nucleos))))
    }
    #[cfg(not(windows))]
    {
        Err(crate::sintesis::PorQueNoRedacta::SinModeloLocal)
    }
}

/// **Lo carga en otro hilo** para que la primera sugerencia de la sesión no espere (al empezar la sesión con «Redactar»
/// encendido). Sin el modelo instalado, no hace nada.
pub fn precargar() {
    if !instalado() {
        return;
    }
    std::thread::spawn(|| {
        let _ = generador();
    });
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// La huella, el tamaño y el origen son los publicados, con la revisión fija y no `main`.
    #[test]
    fn la_huella_el_tamano_y_el_origen_son_los_publicados() {
        assert_eq!(QWEN.sha256.len(), 64);
        assert!(QWEN.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        assert_eq!(QWEN.bytes, 1_117_320_736);
        assert!(QWEN.origen.starts_with("https://huggingface.co/Qwen/Qwen2.5-1.5B-Instruct-GGUF/"));
        assert!(QWEN.origen.contains(&format!("/resolve/{REVISION}/")) && !QWEN.origen.contains("/resolve/main/"));
        assert!(QWEN.origen.ends_with(QWEN.archivo));
    }

    /// Vive en la misma carpeta que los de whisper, la de la app, y fuera de Windows no existe.
    #[test]
    fn vive_con_los_de_whisper_y_solo_en_windows() {
        let base = std::env::temp_dir().join("ag-local");
        assert_eq!(modelo_whisper::ruta_en(base.clone(), &QWEN), base.join("modelos").join(QWEN.archivo));
        let e = estado();
        assert_eq!(e.existe, cfg!(windows));
        assert_eq!(e.bytes, QWEN.bytes);
        if !cfg!(windows) {
            assert!(!e.instalado && !instalado() && generador().is_err());
        }
    }

    /// **Nunca con una reunión**: la descarga va por la puerta de whisper, que lleva la guarda; y el arranque borra los
    /// `.parcial` de su carpeta, también los de este modelo.
    #[test]
    fn baja_por_la_puerta_de_whisper_con_su_guarda() {
        let fuente = include_str!("modelo_local.rs");
        let cuerpo = fuente.split("pub fn bajar(").nth(1).unwrap().split("\n}\n").next().unwrap();
        assert!(cuerpo.contains("modelo_whisper::bajar(&QWEN)"), "el modelo local no baja por la puerta de whisper");
        assert!(!fuente.contains(concat!("ureq", "::")), "una puerta a la red nueva");
        assert_eq!(
            modelo_whisper::ruta_en(PathBuf::from("x"), &QWEN).parent(),
            modelo_whisper::ruta_en(PathBuf::from("x"), &modelo_whisper::PEQUENO).parent(),
            "los .parcial de este modelo los limpia el arranque solo si viven en la misma carpeta"
        );
    }
}
