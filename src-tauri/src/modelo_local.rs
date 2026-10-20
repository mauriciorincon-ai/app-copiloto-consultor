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
    // Si se baja otro modelo por la misma puerta, se dice sin marcar «la descarga falló» de este (auditoría del S14, B-3).
    if modelo_whisper::bajando() {
        return Err("ya se está bajando un modelo".into());
    }
    if BAJANDO.swap(true, Ordering::SeqCst) {
        return Err("ya se está bajando el modelo local".into());
    }
    al_empezar();
    let r = modelo_whisper::bajar(&QWEN);
    FALLO.store(r.is_err(), Ordering::SeqCst);
    BAJANDO.store(false, Ordering::SeqCst);
    r
}

/// **Quién guarda el modelo y cuándo se va** (auditoría del S14, M-1). Antes `soltar()` esperaba el cerrojo mientras otro
/// hilo cargaba 1,1 GB: el corte, que suelta el modelo en su segunda pieza, hacía esperar a todo lo demás lo que durase la
/// carga; y un hilo de sugerencia ya lanzado podía recargar el modelo DESPUÉS del corte y dejarlo residente. Ahora un
/// `soltar()` que llega con una carga en curso **no espera**: avisa, y la carga entrega el modelo a quien lo pidió y no lo
/// guarda.
#[cfg_attr(not(windows), allow(dead_code))]
pub struct Guarda<T> {
    dentro: std::sync::Mutex<Option<std::sync::Arc<T>>>,
    /// Hay un `obtener` en curso (cargando o a punto).
    ocupada: AtomicBool,
    /// Un `soltar` llegó mientras estaba ocupada: al terminar, no se queda.
    soltar_al_terminar: AtomicBool,
}

#[cfg_attr(not(windows), allow(dead_code))]
impl<T> Guarda<T> {
    pub const fn nueva() -> Self {
        Self { dentro: std::sync::Mutex::new(None), ocupada: AtomicBool::new(false), soltar_al_terminar: AtomicBool::new(false) }
    }

    /// Lo que hay, o lo carga con `cargar`. Si un `soltar()` llegó mientras tanto, quien lo pidió lo usa y no queda guardado.
    pub fn obtener<E>(&self, cargar: impl FnOnce() -> Result<T, E>) -> Result<std::sync::Arc<T>, E> {
        self.soltar_al_terminar.store(false, Ordering::SeqCst);
        self.ocupada.store(true, Ordering::SeqCst);
        let mut g = self.dentro.lock().unwrap_or_else(|e| e.into_inner());
        let (m, recien_cargado) = match g.as_ref() {
            Some(m) => (m.clone(), false),
            None => match cargar() {
                Ok(v) => (std::sync::Arc::new(v), true),
                Err(e) => {
                    self.ocupada.store(false, Ordering::SeqCst);
                    return Err(e);
                }
            },
        };
        if recien_cargado {
            *g = Some(m.clone());
        }
        self.ocupada.store(false, Ordering::SeqCst);
        if self.soltar_al_terminar.swap(false, Ordering::SeqCst) {
            g.take();
        }
        Ok(m)
    }

    /// ¿Había algo guardado? Con un `obtener` en curso no espera: deja el aviso y devuelve `false`.
    pub fn soltar(&self) -> bool {
        if self.ocupada.load(Ordering::SeqCst) {
            self.soltar_al_terminar.store(true, Ordering::SeqCst);
            if self.ocupada.load(Ordering::SeqCst) {
                return false;
            }
            // Terminó justo ahora sin ver el aviso: se retira y se suelta aquí.
            if !self.soltar_al_terminar.swap(false, Ordering::SeqCst) {
                return false;
            }
        }
        self.dentro.lock().unwrap_or_else(|e| e.into_inner()).take().is_some()
    }
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
    use std::sync::{Arc, OnceLock};

    /// El modelo en memoria, si se cargó en esta sesión.
    static CARGADO: Guarda<LlamaModel> = Guarda::nueva();

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
        CARGADO.obtener(|| {
            let ruta = ruta().ok_or(PorQueNoRedacta::SinModeloLocal)?;
            crate::hardware::vigilar("el modelo local");
            let reloj = std::time::Instant::now();
            let m = LlamaModel::load_from_file(backend(), &ruta, &LlamaModelParams::default()).map_err(|e| {
                println!("[modelo-local] no cargó: {e}");
                PorQueNoRedacta::NoDisponible
            })?;
            println!("[modelo-local] cargado en {} ms", reloj.elapsed().as_millis());
            Ok(m)
        })
    }

    /// **Lo suelta de la memoria** (el corte, el final de la sesión, «Redactar» apagado). Una sugerencia en camino tiene
    /// su propia copia del `Arc` y lo suelta al terminar; lo que traiga se tira por la época. Devuelve si había algo.
    pub fn soltar() -> bool {
        let suelto = CARGADO.soltar();
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
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// **Un corte durante la carga no espera a la carga y el modelo no queda residente** (auditoría del S14, M-1).
    #[test]
    fn un_soltar_durante_la_carga_no_espera_y_el_modelo_no_queda() {
        let g = Arc::new(Guarda::<u32>::nueva());
        let g2 = g.clone();
        let carga = std::thread::spawn(move || {
            g2.obtener::<()>(|| {
                std::thread::sleep(Duration::from_millis(500));
                Ok(7)
            })
        });
        std::thread::sleep(Duration::from_millis(120)); // la carga ya empezó
        let t = Instant::now();
        let solto = g.soltar();
        assert!(t.elapsed() < Duration::from_millis(120), "soltar esperó a la carga: {:?}", t.elapsed());
        assert!(!solto, "no había nada guardado todavía");
        assert_eq!(*carga.join().unwrap().unwrap(), 7, "quien lo pidió lo usa");
        assert!(!g.soltar(), "el modelo quedó residente después del corte");
    }

    /// Sin corte, el modelo se guarda y se reutiliza; y soltar lo suelta.
    #[test]
    fn sin_corte_el_modelo_se_guarda_se_reutiliza_y_se_suelta() {
        let g = Guarda::<u32>::nueva();
        assert_eq!(*g.obtener::<()>(|| Ok(1)).unwrap(), 1);
        assert_eq!(*g.obtener::<()>(|| Ok(2)).unwrap(), 1, "no se vuelve a cargar lo que ya está");
        assert!(g.soltar());
        assert!(!g.soltar());
        assert_eq!(*g.obtener::<()>(|| Ok(3)).unwrap(), 3, "tras soltarlo, se carga otra vez");
        assert!(g.obtener::<&str>(|| Err("no cargó")).is_ok(), "con el modelo ya guardado no vuelve a cargar");
        assert!(g.soltar());
        assert_eq!(g.obtener::<&str>(|| Err("no cargó")).unwrap_err(), "no cargó");
        assert!(!g.soltar(), "una carga fallida no deja nada");
    }

    /// El aviso de un corte no se queda pegado a la carga siguiente.
    #[test]
    fn el_aviso_de_un_corte_no_afecta_a_la_carga_siguiente() {
        let g = Guarda::<u32>::nueva();
        g.soltar_al_terminar.store(true, Ordering::SeqCst); // un aviso viejo
        assert_eq!(*g.obtener::<()>(|| Ok(5)).unwrap(), 5);
        assert!(g.soltar(), "el aviso viejo hizo que no se guardara");
    }

    /// IA no dice «la descarga falló» de este modelo porque se esté bajando otro por la misma puerta (auditoría, B-3): se
    /// consulta ANTES de marcar nada. (La bandera de la puerta es de `modelo_whisper`: se comprueba en la fuente.)
    #[test]
    fn bajar_mira_si_la_puerta_esta_ocupada_antes_de_marcar_nada() {
        let fuente = include_str!("modelo_local.rs");
        let bajar = &fuente[fuente.find("pub fn bajar(").expect("bajar")..];
        let (mira, marca) = (bajar.find("modelo_whisper::bajando()"), bajar.find("BAJANDO.swap(true"));
        assert!(mira.is_some() && mira < marca, "bajar marca antes de mirar si la puerta está ocupada");
    }

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
