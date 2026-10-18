//! **Transcripción en Windows con whisper** (sprint 006; ADR 006 enmendado por el ADR 022) — sigue siendo `stt/`:
//! módulo protegido, ni disco ni red desde aquí.
//!
//! Hace en Windows lo que `apple.rs` hace en el Mac: el audio de un turno entra prestado, el texto sale, y nada se
//! guarda. El modelo lo baja y lo comprueba `modelo_whisper.rs`, que no ve un solo turno; este archivo solo le pide
//! la ruta del que ya está entero.
//!
//! Lo que se decide sin whisper delante —qué idiomas, qué modelo, cómo se completa un turno corto— vive arriba y se
//! prueba en cualquier equipo. Lo que necesita whisper vive en `motor`, solo en Windows.

use super::{Disponibilidad, Fallo, Motor};
use crate::modelo_whisper::{self, Modelo, GRANDE, PEQUENO};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

/// Los idiomas que la app transcribe con whisper: los dos de la app (regla dura 7). whisper sabe muchos más; la app
/// ofrece los que tiene redactados en sus dos idiomas.
pub const IDIOMAS: [&str; 2] = ["es-ES", "en-US"];

/// El código de whisper para un idioma de la app: «es-MX» → «es». Solo español e inglés; lo demás, `None`.
pub fn idioma_de_whisper(codigo: &str) -> Option<&'static str> {
    match codigo.split(['-', '_']).next()? {
        "es" => Some("es"),
        "en" => Some("en"),
        _ => None,
    }
}

/// En qué estado está whisper para un idioma, sabiendo si hay un modelo entero.
pub fn disponibilidad_con(idioma: &str, hay_modelo: bool) -> Disponibilidad {
    match (idioma_de_whisper(idioma), hay_modelo) {
        (None, _) => Disponibilidad::IdiomaDesconocido,
        (Some(_), true) => Disponibilidad::Listo,
        (Some(_), false) => Disponibilidad::SinModelo,
    }
}

/// Lo que tardó cada modelo en transcribir el recorte de calibración en este PC, en ms (0: sin medir). Vive en
/// memoria: al reabrir la app se vuelve a medir, y Honestidad dice «sin medir» mientras tanto.
static LATENCIA_GRANDE: AtomicU32 = AtomicU32::new(0);
static LATENCIA_PEQUENO: AtomicU32 = AtomicU32::new(0);

fn latencia_de(m: &Modelo) -> &'static AtomicU32 {
    if m.archivo == PEQUENO.archivo {
        &LATENCIA_PEQUENO
    } else {
        &LATENCIA_GRANDE
    }
}

/// Lo que tardó `m` en el recorte de calibración, si ya se midió.
pub fn latencia_ms(m: &Modelo) -> Option<u32> {
    Some(latencia_de(m).load(Ordering::Relaxed)).filter(|&ms| ms > 0)
}

/// Anota lo que tardó `m`.
pub fn anotar_latencia(m: &Modelo, ms: u32) {
    latencia_de(m).store(ms.max(1), Ordering::Relaxed);
}

/// **Lo que tardó cada modelo con la pista del cliente** (sprint 010), en ms (0: sin medir). Se mide DESPUÉS de tu
/// turno de 10 s, con el modelo ya caliente, como lo atiende el transcriptor en una reunión. Honestidad la enseña junto a
/// la tuya; la regla ([`elegir`]) no la usa: decide con tu turno, como desde el sprint 009 (ADR 022, enmienda 3).
static LATENCIA_CLIENTE_GRANDE: AtomicU32 = AtomicU32::new(0);
static LATENCIA_CLIENTE_PEQUENO: AtomicU32 = AtomicU32::new(0);

fn latencia_del_cliente_de(m: &Modelo) -> &'static AtomicU32 {
    if m.archivo == PEQUENO.archivo {
        &LATENCIA_CLIENTE_PEQUENO
    } else {
        &LATENCIA_CLIENTE_GRANDE
    }
}

/// Lo que tardó `m` con la pista del cliente, si ya se midió.
pub fn latencia_del_cliente_ms(m: &Modelo) -> Option<u32> {
    Some(latencia_del_cliente_de(m).load(Ordering::Relaxed)).filter(|&ms| ms > 0)
}

/// Anota lo que tardó `m` con la pista del cliente.
pub fn anotar_latencia_del_cliente(m: &Modelo, ms: u32) {
    latencia_del_cliente_de(m).store(ms.max(1), Ordering::Relaxed);
}

/// Vuelve a poner las medidas que la app guardó la vez anterior (sprint 008, M3 del S6). Lo que ya se midió en esta
/// sesión manda sobre lo guardado. Este módulo no escribe nada: quien guarda es `prefs`, que recibe solo números.
pub fn restaurar_latencias(grande_ms: Option<u32>, pequeno_ms: Option<u32>) {
    restaurar_en((&LATENCIA_GRANDE, grande_ms), (&LATENCIA_PEQUENO, pequeno_ms));
}

/// Lo mismo con la pista del cliente (sprint 010). Unas preferencias de antes del sprint no la traen: `None`, sin medir.
pub fn restaurar_latencias_del_cliente(grande_ms: Option<u32>, pequeno_ms: Option<u32>) {
    restaurar_en((&LATENCIA_CLIENTE_GRANDE, grande_ms), (&LATENCIA_CLIENTE_PEQUENO, pequeno_ms));
}

/// La parte pura: cada medida guardada entra solo si ese modelo aún no se midió en esta sesión.
fn restaurar_en(grande: (&AtomicU32, Option<u32>), pequeno: (&AtomicU32, Option<u32>)) {
    for (donde, ms) in [grande, pequeno] {
        if let Some(ms) = ms.filter(|&ms| ms > 0) {
            let _ = donde.compare_exchange(0, ms, Ordering::Relaxed, Ordering::Relaxed);
        }
    }
}

/// **La regla** (ADR 022 §5, enmienda 3): de los modelos que hay, el MAYOR cuya calibración cabe en el techo de 4 s.
/// Uno que aún no se midió cuenta como que cabe (se mide al instalarlo y entonces manda su cifra). Si ninguno cabe,
/// el más rápido: la app transcribe con lo que haya y Honestidad dice que el equipo es lento, no se queda muda.
/// `candidatos` va de mayor a menor. Pura.
pub fn el_mayor_que_cabe(candidatos: &[(Modelo, Option<u32>)]) -> Option<Modelo> {
    candidatos
        .iter()
        .find(|(_, ms)| ms.is_none_or(|ms| !super::calibracion::pasa_del_techo(ms)))
        .or_else(|| candidatos.iter().min_by_key(|(_, ms)| ms.unwrap_or(u32::MAX)))
        .map(|(m, _)| *m)
}

/// **Con qué modelo se transcribe** (ADR 022 §5): la regla de arriba sobre los modelos ENTEROS en este PC, con lo que
/// cada uno tardó en la calibración. Con solo uno bajado, ese; con el grande y el pequeño, el grande mientras quepa.
pub fn elegir(
    grande_listo: bool,
    pequeno_listo: bool,
    latencia_grande: Option<u32>,
    latencia_pequeno: Option<u32>,
) -> Option<Modelo> {
    let mut candidatos = Vec::with_capacity(2);
    if grande_listo {
        candidatos.push((GRANDE, latencia_grande));
    }
    if pequeno_listo {
        candidatos.push((PEQUENO, latencia_pequeno));
    }
    el_mayor_que_cabe(&candidatos)
}

/// **Lo que tardó cada modelo con el turno de 10 s de la calibración en el equipo ARM64 de referencia** (la Surface con
/// Snapdragon X Plus; ADR 022, enmienda 3), en ms; `None`: sin medir allí. De ahí sale [`por_defecto`] en un PC ARM64: no
/// se baja 1,6 GB de un modelo que ya se sabe que no cabe. La cifra de cada PC, medida al instalar, sigue mandando en
/// [`elegir`].
///
/// Medido el 2026-10-17 en la Surface del usuario (Snapdragon X Plus X1P64100, plan Balanced, con corriente) con
/// `medicion_whisper_con_dos_pistas`, mediana de tres corridas con whisper.cpp optimizado: es el turno de 10 s que va
/// DETRÁS del turno del cliente, con el modelo ya caliente. `large-v3-turbo` 9144 ms (de 8523 a 18428) y `small` 1632 ms
/// (de 1590 a 1641). La calibración de la propia app mide la primera inferencia tras cargar el modelo, sin calentar, y
/// da algo más (8334 y 1928 ms en una corrida real; ADR, límite f): la decisión no cambia. Las cifras por pista y el
/// porqué están en el ADR 022, enmienda 3; `el_adr_dice_lo_mismo_que_la_constante` las ata.
pub const MEDIDO_EN_ARM: [(&str, Option<u32>); 2] = [("ggml-large-v3-turbo.bin", Some(9_144)), ("ggml-small.bin", Some(1_632))];

/// El modelo que «Instalar» baja. En un PC ARM64 de Windows, el mayor que cupo en el equipo de referencia; en cualquier
/// otro, el grande (ADR 006), como hasta ahora. En macOS no se usa: el motor es el del sistema.
pub fn por_defecto() -> Modelo {
    por_defecto_con(cfg!(all(windows, target_arch = "aarch64")), &MEDIDO_EN_ARM)
}

/// La parte pura de [`por_defecto`]: sin ARM, el grande; con ARM, la regla sobre lo medido en la referencia.
pub fn por_defecto_con(es_arm64: bool, medido: &[(&str, Option<u32>)]) -> Modelo {
    if !es_arm64 {
        return GRANDE;
    }
    let latencia = |m: &Modelo| medido.iter().find(|(archivo, _)| *archivo == m.archivo).and_then(|(_, ms)| *ms);
    el_mayor_que_cabe(&[(GRANDE, latencia(&GRANDE)), (PEQUENO, latencia(&PEQUENO))]).unwrap_or(GRANDE)
}

/// El modelo con el que se transcribiría ahora mismo, y su ruta. `None`: no hay ninguno entero.
pub fn el_modelo() -> Option<(Modelo, PathBuf)> {
    let grande = modelo_whisper::listo(&GRANDE);
    let pequeno = modelo_whisper::listo(&PEQUENO);
    match elegir(grande.is_some(), pequeno.is_some(), latencia_ms(&GRANDE), latencia_ms(&PEQUENO))? {
        m if m == GRANDE => grande.map(|r| (m, r)),
        m => pequeno.map(|r| (m, r)),
    }
}

/// whisper no transcribe menos de un segundo de audio; un turno corto («sí») se completa con silencio hasta 1,1 s.
pub const MINIMO: usize = 17_600;

/// El turno, completado con silencio si es más corto que [`MINIMO`].
pub fn completar(muestras: &[f32]) -> std::borrow::Cow<'_, [f32]> {
    if muestras.len() >= MINIMO {
        return std::borrow::Cow::Borrowed(muestras);
    }
    let mut largo = muestras.to_vec();
    largo.resize(MINIMO, 0.0);
    std::borrow::Cow::Owned(largo)
}

/// Pone a cero unas muestras (las copias del turno) antes de soltarlas.
pub fn borrar(muestras: &mut [f32]) {
    muestras.fill(0.0);
}

/// **Suelta el modelo cargado y lo que su estado retiene** (sprint 006, ADR 022, hallazgo A2 de la auditoría). La
/// última transcripción vive dentro del estado de whisper hasta la siguiente. El corte y «Terminar» la sueltan: el
/// texto de la sala no sobrevive a la sesión. Fuera de Windows no hay nada que soltar.
pub fn soltar() {
    #[cfg(windows)]
    {
        *motor::CARGADO.lock().unwrap_or_else(|e| e.into_inner()) = None;
    }
}

/// El motor de la casa en Windows.
#[cfg(windows)]
pub fn motor() -> Box<dyn Motor> {
    Box::new(Whisper)
}

/// whisper, con el modelo que haya en este PC.
pub struct Whisper;

impl Motor for Whisper {
    fn nombre(&self) -> &'static str {
        "whisper"
    }
    fn disponibilidad(&self, idioma: &str) -> Disponibilidad {
        disponibilidad_con(idioma, el_modelo().is_some())
    }
    /// **Baja el modelo por defecto de este equipo** ([`por_defecto`]: el grande, 1,6 GB, salvo en un PC ARM64 donde ya
    /// se sabe que no cabe), con su huella, y mide lo que tarda en este PC. Solo desde «Instalar» en Idioma, y jamás en
    /// reunión (lo impide `lib.rs::instalar_idioma`).
    fn instalar(&self, idioma: &str) -> Disponibilidad {
        if idioma_de_whisper(idioma).is_none() {
            return Disponibilidad::IdiomaDesconocido;
        }
        match modelo_whisper::bajar(&por_defecto()) {
            Ok(_) => {
                if let Err(f) = calibrar() {
                    println!("[stt] whisper: la calibración no se pudo medir ({f:?})");
                }
            }
            Err(e) => println!("[stt] whisper: {e}"),
        }
        self.disponibilidad(idioma)
    }
    fn transcribir(&self, idioma: &str, muestras: &[f32], hz: u32) -> Result<String, Fallo> {
        let Some(codigo) = idioma_de_whisper(idioma) else {
            return Err(Fallo::NoDisponible(Disponibilidad::IdiomaDesconocido));
        };
        motor::transcribir(codigo, muestras, hz)
    }
    /// whisper tiene todos sus idiomas a la vez: el techo son los que la app ofrece.
    fn techo_de_idiomas(&self) -> u32 {
        IDIOMAS.len() as u32
    }
    fn idiomas(&self) -> Vec<String> {
        IDIOMAS.iter().map(|c| c.to_string()).collect()
    }
}

/// **Mide** lo que tarda el modelo de ahora en transcribir el recorte de calibración, y lo anota. Solo la
/// transcripción: la carga del modelo (una vez por sesión de la app) no cuenta.
pub fn calibrar() -> Result<u32, Fallo> {
    motor::calibrar()
}

#[cfg(windows)]
mod motor {
    use super::*;
    use std::sync::Mutex;
    use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperError, WhisperState};

    /// El modelo cargado: el estado de whisper guarda su contexto. Uno a la vez; se carga al primer turno.
    pub(super) struct Cargado {
        modelo: Modelo,
        estado: WhisperState,
    }

    pub(super) static CARGADO: Mutex<Option<Cargado>> = Mutex::new(None);

    fn codigo(e: &WhisperError) -> i32 {
        match e {
            WhisperError::GenericError(c) => *c,
            _ => -1,
        }
    }

    /// Corre `f` con el modelo elegido, cargándolo si hace falta (y soltando el anterior si era otro).
    fn con_el_modelo<T>(f: impl FnOnce(Modelo, &mut WhisperState) -> Result<T, Fallo>) -> Result<T, Fallo> {
        let (modelo, ruta) = el_modelo().ok_or(Fallo::NoDisponible(Disponibilidad::SinModelo))?;
        con_este_modelo(modelo, ruta, f)
    }

    /// Lo mismo con un modelo dado: la calibración mide cada uno que esté entero, no solo el que se usaría.
    fn con_este_modelo<T>(
        modelo: Modelo,
        ruta: PathBuf,
        f: impl FnOnce(Modelo, &mut WhisperState) -> Result<T, Fallo>,
    ) -> Result<T, Fallo> {
        let mut cargado = CARGADO.lock().unwrap_or_else(|e| e.into_inner());
        if cargado.as_ref().map(|c| c.modelo) != Some(modelo) {
            *cargado = None;
            crate::hardware::vigilar("el modelo de whisper");
            // Sin backend de logs, whisper.cpp calla: ni su ruido ni, sobre todo, nada de lo transcrito (ADR 003).
            whisper_rs::install_logging_hooks();
            let contexto = WhisperContext::new_with_params(&ruta, WhisperContextParameters::default()).map_err(|e| {
                println!("[stt] whisper no cargó {}: {e}", modelo.nombre);
                Fallo::Motor(codigo(&e))
            })?;
            let estado = contexto.create_state().map_err(|e| Fallo::Motor(codigo(&e)))?;
            println!("[stt] whisper: {} cargado", modelo.nombre);
            *cargado = Some(Cargado { modelo, estado });
        }
        let c = cargado.as_mut().ok_or(Fallo::Motor(-1))?;
        f(c.modelo, &mut c.estado)
    }

    fn hilos() -> i32 {
        std::thread::available_parallelism().map(|n| n.get().min(8) as i32).unwrap_or(4)
    }

    fn transcribir_en(estado: &mut WhisperState, codigo_whisper: &'static str, audio: &[f32]) -> Result<String, Fallo> {
        let mut p = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        p.set_language(Some(codigo_whisper));
        p.set_n_threads(hilos());
        // Cada turno por su cuenta: arrastrar el texto del turno anterior como contexto es arrastrar al cliente.
        p.set_no_context(true);
        p.set_suppress_blank(true);
        p.set_print_special(false);
        p.set_print_progress(false);
        p.set_print_realtime(false);
        p.set_print_timestamps(false);
        estado.full(p, audio).map_err(|e| Fallo::Motor(codigo(&e)))?;
        let mut texto = String::new();
        for i in 0..estado.full_n_segments() {
            if let Some(Ok(t)) = estado.get_segment(i).map(|s| s.to_str_lossy().map(|t| t.into_owned())) {
                texto.push_str(&t);
            }
        }
        Ok(texto.trim().to_string())
    }

    pub fn transcribir(codigo_whisper: &'static str, muestras: &[f32], hz: u32) -> Result<String, Fallo> {
        let hz_app = crate::capture::anillo::HZ;
        // Las copias del audio del turno se ponen a cero al salir (ADR 022, hallazgo A2 de la auditoría del S6): el
        // turno es de un tercero y no debe sobrevivir a su uso.
        let mut a_dieciseis: Vec<f32> = if hz == hz_app {
            muestras.to_vec()
        } else {
            crate::capture::Remuestreador::nuevo(hz, hz_app).convertir(muestras)
        };
        let mut audio = completar(&a_dieciseis).into_owned();
        let texto = con_el_modelo(|_, estado| transcribir_en(estado, codigo_whisper, &audio));
        super::borrar(&mut a_dieciseis);
        super::borrar(&mut audio);
        texto
    }

    /// **Mide CADA modelo que esté entero** (sprint 009), **pista por pista** (sprint 010): Honestidad enseña una línea
    /// por modelo con lo que tardó tu turno de 10 s y el del cliente, y la regla ([`elegir`]) compara lo que tardó tu
    /// turno en cada uno. Se mide uno tras otro —el modelo cargado se suelta antes de cargar el siguiente— y, al
    /// terminar, devuelve lo que tardó el que se usaría ahora.
    pub fn calibrar() -> Result<u32, Fallo> {
        let audio = crate::stt::calibracion::muestras();
        let cliente = crate::stt::calibracion::muestras_del_cliente();
        let codigo_whisper = idioma_de_whisper(crate::stt::calibracion::IDIOMA).unwrap_or("es");
        let mut midio = false;
        let mut ultimo_fallo = None;
        for m in [GRANDE, PEQUENO] {
            let Some(ruta) = modelo_whisper::listo(&m) else { continue };
            // Un modelo que no se deja medir (poca memoria, un archivo dañado) no impide medir el otro (auditoría B3).
            match con_este_modelo(m, ruta, |_, estado| {
                let mut medir = |audio: &[f32]| -> Result<u32, Fallo> {
                    let inicio = std::time::Instant::now();
                    transcribir_en(estado, codigo_whisper, audio)?;
                    Ok(inicio.elapsed().as_millis().min(u128::from(u32::MAX)) as u32)
                };
                // Tu turno primero, como hasta el sprint 009: es la cifra que decide el modelo. El del cliente, detrás,
                // con el modelo ya caliente; si ese falla, tu cifra vale igual y la del cliente queda sin medir.
                let tuyo = medir(&audio)?;
                Ok((tuyo, medir(&completar(&cliente))))
            }) {
                Ok((ms, del_cliente)) => {
                    anotar_latencia(&m, ms);
                    match del_cliente {
                        Ok(ms_cliente) => {
                            anotar_latencia_del_cliente(&m, ms_cliente);
                            println!(
                                "[stt] whisper: {} tardó {ms} ms en tu pista (10 s) y {ms_cliente} ms en la del cliente",
                                m.nombre
                            );
                        }
                        Err(f) => println!(
                            "[stt] whisper: {} tardó {ms} ms en tu pista (10 s); la del cliente no se pudo medir ({f:?})",
                            m.nombre
                        ),
                    }
                    midio = true;
                }
                Err(f) => {
                    println!("[stt] whisper: {} no se pudo medir ({f:?})", m.nombre);
                    ultimo_fallo = Some(f);
                }
            }
        }
        if !midio {
            return Err(ultimo_fallo.unwrap_or(Fallo::NoDisponible(Disponibilidad::SinModelo)));
        }
        el_modelo().and_then(|(m, _)| latencia_ms(&m)).ok_or(Fallo::Motor(-1))
    }

    /// **Whisper con dos pistas, medido en este equipo** (sprint 007, decisión 3 del plan). Con dos pistas el
    /// transcriptor atiende en serie, el cliente primero: lo que importa es cuánto tarda el turno del cliente y cuánto
    /// le añade un turno tuyo detrás. La CI lo corre en `build-windows` con el modelo pequeño; **la cifra es del runner
    /// y se declara `manual`: la del PC del usuario manda**, en el gate del hito. Imprime solo milisegundos, jamás lo
    /// transcrito (ADR 003).
    #[cfg(test)]
    mod medicion {
        use super::*;

        #[test]
        #[ignore = "medición: whisper con dos pistas en este equipo; necesita AG_MODELO_WHISPER con un modelo entero"]
        fn medicion_whisper_con_dos_pistas() {
            let ruta = std::env::var("AG_MODELO_WHISPER").expect("falta AG_MODELO_WHISPER: la medición no midió nada");
            whisper_rs::install_logging_hooks();
            let contexto =
                WhisperContext::new_with_params(&ruta, WhisperContextParameters::default()).expect("el modelo no cargó");
            let mut estado = contexto.create_state().expect("whisper no dio su estado");
            let cliente = crate::habla::pcm::leer(include_bytes!("../../../docs/kit-de-prueba/audio/pregunta-es.wav"))
                .expect("el WAV del kit se lee")
                .muestras;
            let tuyo = crate::stt::calibracion::muestras();
            let mut medir = |audio: &[f32]| {
                let inicio = std::time::Instant::now();
                transcribir_en(&mut estado, "es", &completar(audio)).expect("whisper no transcribió");
                inicio.elapsed().as_millis() as u32
            };
            // La primera inferencia calienta pesos y cachés; en una reunión el modelo ya está cargado y caliente.
            medir(&cliente);
            let ms_cliente = medir(&cliente);
            let ms_tuyo = medir(&tuyo);
            let modelo = std::path::Path::new(&ruta).file_name().map(|n| n.to_string_lossy().into_owned());
            // Los segundos de cada audio salen de sus muestras, no de una cifra escrita a mano (B11 del S7): si el kit
            // o la calibración cambian, el mensaje sigue diciendo la verdad.
            let segundos = |audio: &[f32]| audio.len() as f32 / crate::capture::anillo::HZ as f32;
            println!(
                "[medición] whisper {} con dos pistas en serie, el cliente primero: el cliente ({:.1} s) {ms_cliente} ms · \
                 un turno tuyo detrás ({:.1} s) {ms_tuyo} ms · techo del fin de turno a la ficha {} ms · cifra del runner, \
                 manual: la del PC manda",
                modelo.unwrap_or_default(),
                segundos(&cliente),
                segundos(&tuyo),
                crate::stt::calibracion::TECHO_MS
            );
        }

        /// **El camino real de «Medir otra vez»** (sprint 009): `calibrar()` sobre los modelos que haya ENTEROS en la
        /// carpeta de la app de este PC —los mismos que usa la app instalada— y lo que Honestidad recibiría después
        /// (`plataforma::de_este_equipo`). Comprueba que se midió CADA modelo bajado, no solo el elegido, y que la
        /// regla eligió con esas cifras. Imprime solo nombres y milisegundos (ADR 003). No abre hardware: solo CPU.
        #[test]
        #[ignore = "medición: carga whisper (nativo: se corre a mano, SIN AG_SIN_HARDWARE) y calibra los modelos enteros de la carpeta de la app de este PC; sin ellos no mide nada"]
        fn la_calibracion_mide_cada_modelo_entero_y_honestidad_lo_recibe() {
            let enteros: Vec<Modelo> = [GRANDE, PEQUENO].into_iter().filter(|m| modelo_whisper::listo(m).is_some()).collect();
            assert!(!enteros.is_empty(), "ningún modelo entero en la carpeta de la app: la medición no midió nada");
            calibrar().expect("la calibración no pudo medir");
            for m in &enteros {
                assert!(latencia_ms(m).is_some(), "{} está entero y no se midió", m.nombre);
                assert!(latencia_del_cliente_ms(m).is_some(), "{}: la pista del cliente no se midió", m.nombre);
            }
            let t = crate::plataforma::de_este_equipo().transcripcion.expect("en Windows Honestidad recibe la transcripción");
            assert_eq!(t.modelos.len(), 2, "una línea por modelo, estén bajados o no");
            for l in &t.modelos {
                assert_eq!(l.bajado, enteros.iter().any(|m| m.nombre == l.nombre), "{}", l.nombre);
                assert_eq!(l.latencia_ms.is_some(), l.bajado, "{}: la cifra va con el modelo bajado", l.nombre);
                assert_eq!(l.latencia_cliente_ms.is_some(), l.bajado, "{}: la del cliente, también", l.nombre);
            }
            assert_eq!(t.modelos.iter().filter(|l| l.en_uso).count(), 1, "exactamente uno transcribe");
            println!("[medición] Honestidad recibiría: {:?}", t.modelos);
        }
    }
}

/// Fuera de Windows no hay whisper: macOS transcribe con su propio motor (ADR 006).
#[cfg(not(windows))]
mod motor {
    use super::*;

    pub fn transcribir(_codigo_whisper: &'static str, _muestras: &[f32], _hz: u32) -> Result<String, Fallo> {
        Err(Fallo::NoDisponible(Disponibilidad::SinModelo))
    }

    pub fn calibrar() -> Result<u32, Fallo> {
        Err(Fallo::NoDisponible(Disponibilidad::SinModelo))
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Español e inglés, con cualquier región; lo demás, desconocido, también lo que no cruzaría a C.
    #[test]
    fn solo_espanol_e_ingles_con_cualquier_region() {
        assert_eq!(idioma_de_whisper("es-ES"), Some("es"));
        assert_eq!(idioma_de_whisper("es-MX"), Some("es"));
        assert_eq!(idioma_de_whisper("en_GB"), Some("en"));
        assert_eq!(idioma_de_whisper("fr-FR"), None);
        assert_eq!(idioma_de_whisper("es\0ES"), None);
        assert_eq!(idioma_de_whisper(""), None);
        for c in IDIOMAS {
            assert!(idioma_de_whisper(c).is_some(), "{c}");
        }
    }

    /// Los tres estados que Idioma sabe enseñar, sin un cuarto inventado.
    #[test]
    fn la_disponibilidad_dice_si_falta_el_modelo_o_el_idioma() {
        assert_eq!(disponibilidad_con("es-ES", true), Disponibilidad::Listo);
        assert_eq!(disponibilidad_con("es-ES", false), Disponibilidad::SinModelo);
        assert_eq!(disponibilidad_con("xx-ZZ", true), Disponibilidad::IdiomaDesconocido);
    }

    /// **El escalón del modelo** (ADR 022 §5): el grande por defecto; el pequeño cuando el grande se pasó del techo
    /// en este PC y el pequeño ya está, o cuando es el único que hay.
    #[test]
    fn el_pequeno_solo_si_el_grande_es_lento_o_no_esta() {
        assert_eq!(elegir(true, true, None, None), Some(GRANDE), "sin medir, el grande");
        assert_eq!(elegir(true, true, Some(3_000), None), Some(GRANDE));
        assert_eq!(elegir(true, true, Some(4_000), Some(900)), Some(GRANDE), "justo en el techo todavía cabe");
        assert_eq!(elegir(true, true, Some(6_000), None), Some(PEQUENO));
        assert_eq!(elegir(true, false, Some(6_000), None), Some(GRANDE), "lento, pero el pequeño no está: el grande");
        assert_eq!(elegir(false, true, None, None), Some(PEQUENO));
        assert_eq!(elegir(false, false, None, None), None);
    }

    /// **La regla de la enmienda 3**: el mayor que cabe en 4 s; sin medir cuenta como que cabe; si ninguno cabe, el más
    /// rápido —la app no se queda muda en un equipo lento—.
    #[test]
    fn la_regla_es_el_mayor_que_cabe_y_si_ninguno_cabe_el_mas_rapido() {
        let c = |g, p| [(GRANDE, g), (PEQUENO, p)];
        assert_eq!(el_mayor_que_cabe(&c(Some(3_900), Some(800))), Some(GRANDE));
        assert_eq!(el_mayor_que_cabe(&c(Some(4_001), Some(800))), Some(PEQUENO));
        assert_eq!(el_mayor_que_cabe(&c(Some(9_000), None)), Some(PEQUENO), "el pequeño sin medir cabe hasta que se mida");
        assert_eq!(el_mayor_que_cabe(&c(None, None)), Some(GRANDE), "nada medido: el mayor");
        assert_eq!(el_mayor_que_cabe(&c(Some(9_000), Some(5_000))), Some(PEQUENO), "ninguno cabe: el más rápido");
        assert_eq!(el_mayor_que_cabe(&c(Some(5_000), Some(9_000))), Some(GRANDE), "ninguno cabe: el más rápido, aunque sea el grande");
        assert_eq!(el_mayor_que_cabe(&[]), None);
        // Y `elegir` es esa misma regla sobre los modelos que están enteros.
        assert_eq!(elegir(true, true, Some(9_000), Some(5_000)), Some(PEQUENO));
        assert_eq!(elegir(true, false, Some(9_000), None), Some(GRANDE));
    }

    /// **El defecto de «Instalar»** (enmienda 3): fuera de un PC ARM64 de Windows, el grande, como siempre; en uno, el
    /// mayor que cupo en el equipo de referencia. Sin cifra de referencia no se inventa ninguna: el grande.
    #[test]
    fn el_defecto_solo_cambia_en_un_pc_arm64_y_solo_con_una_medida() {
        let rapido = [(GRANDE.archivo, Some(2_100)), (PEQUENO.archivo, Some(700))];
        let lento = [(GRANDE.archivo, Some(6_300)), (PEQUENO.archivo, Some(1_900))];
        let sin_medir = [(GRANDE.archivo, None), (PEQUENO.archivo, None)];
        assert_eq!(por_defecto_con(false, &lento), GRANDE, "x64 y macOS no cambian aunque haya cifras de ARM");
        assert_eq!(por_defecto_con(true, &rapido), GRANDE);
        assert_eq!(por_defecto_con(true, &lento), PEQUENO);
        assert_eq!(por_defecto_con(true, &sin_medir), GRANDE);
        assert_eq!(por_defecto_con(true, &[]), GRANDE);
        // Lo que se distribuye: las constantes que salen del ADR llevan los dos archivos, en el orden de la regla.
        assert_eq!(MEDIDO_EN_ARM.map(|(a, _)| a), [GRANDE.archivo, PEQUENO.archivo]);
        // Y lo que dicen: en la Surface el grande pasó del techo y el pequeño cupo, así que en ARM64 se baja el pequeño.
        assert_eq!(por_defecto_con(true, &MEDIDO_EN_ARM), PEQUENO, "la cifra medida en la Surface elige el pequeño");
        assert_eq!(por_defecto_con(false, &MEDIDO_EN_ARM), GRANDE, "fuera de ARM64 de Windows sigue el grande");
    }

    /// **La constante y el ADR dicen lo mismo** (como `plataforma::la_lista_es_la_del_adr`): la tabla de la enmienda 3 es
    /// la fuente que se lee, y `MEDIDO_EN_ARM` no puede cambiar sin que cambie ella, ni al revés.
    #[test]
    fn el_adr_dice_lo_mismo_que_la_constante() {
        let adr = include_str!("../../../decisions/022-windows-detras-de-la-misma-frontera.md");
        let tabla: Vec<(&str, u32)> = adr
            .lines()
            .skip_while(|l| !l.starts_with("## Enmienda 3"))
            .skip(1)
            .take_while(|l| !l.starts_with("## "))
            // La tabla va dentro de un punto de la lista numerada, así que sus filas llegan sangradas.
            .filter(|l| l.trim_start().starts_with("| `ggml-"))
            .map(|l| {
                let celdas: Vec<&str> = l.split('|').map(str::trim).collect();
                let archivo = celdas[1].trim_matches('`');
                let ms = celdas[2].parse::<u32>().unwrap_or_else(|_| panic!("el ADR no trae una cifra entera en «{l}»"));
                (archivo, ms)
            })
            .collect();
        let constante: Vec<(&str, u32)> = MEDIDO_EN_ARM.iter().filter_map(|(a, ms)| Some((*a, (*ms)?))).collect();
        assert!(!tabla.is_empty(), "la enmienda 3 del ADR 022 no trae la tabla de lo medido");
        assert_eq!(tabla, constante);
    }

    /// **Lo medido la vez anterior vuelve al abrir la app, y lo medido hoy manda** (sprint 008, M3 del S6).
    #[test]
    fn las_medidas_guardadas_vuelven_y_lo_de_hoy_manda() {
        let (grande, pequeno) = (AtomicU32::new(0), AtomicU32::new(4_200));
        restaurar_en((&grande, Some(5_200)), (&pequeno, Some(1_900)));
        assert_eq!(grande.load(Ordering::Relaxed), 5_200, "sin medir hoy: vuelve lo guardado");
        assert_eq!(pequeno.load(Ordering::Relaxed), 4_200, "ya medido hoy: lo guardado no lo pisa");
        let vacio = AtomicU32::new(0);
        restaurar_en((&vacio, None), (&AtomicU32::new(0), Some(0)));
        assert_eq!(vacio.load(Ordering::Relaxed), 0, "nada guardado: sigue sin medir, no se inventa un cero");
        // Y con lo que vuelve, el escalón se decide como antes de cerrar la app.
        assert_eq!(
            elegir(true, true, Some(grande.load(Ordering::Relaxed)), Some(pequeno.load(Ordering::Relaxed))),
            Some(PEQUENO)
        );
    }

    /// Un turno corto se completa con silencio hasta lo que whisper acepta; uno largo pasa tal cual.
    #[test]
    fn un_turno_corto_se_completa_con_silencio() {
        let corto = completar(&[0.5; 100]);
        assert_eq!(corto.len(), MINIMO);
        assert_eq!(corto[99], 0.5);
        assert_eq!(corto[100], 0.0);
        let largo = vec![0.1; MINIMO + 1];
        assert!(matches!(completar(&largo), std::borrow::Cow::Borrowed(_)));
    }

    /// La latencia de cada modelo se anota aparte; cero no es «instantáneo», es «sin medir».
    #[test]
    fn la_latencia_se_anota_por_modelo() {
        anotar_latencia(&PEQUENO, 0);
        assert_eq!(latencia_ms(&PEQUENO), Some(1), "una medida de 0 ms se guarda como 1: medida, no vacía");
        anotar_latencia(&PEQUENO, 2_500);
        assert_eq!(latencia_ms(&PEQUENO), Some(2_500));
    }

    /// **La pista del cliente se anota aparte** (sprint 010): su cifra no pisa la de tu turno, que es la que decide.
    #[test]
    fn la_pista_del_cliente_se_anota_aparte() {
        anotar_latencia_del_cliente(&GRANDE, 0);
        assert_eq!(latencia_del_cliente_ms(&GRANDE), Some(1), "medida, no vacía");
        anotar_latencia_del_cliente(&GRANDE, 3_100);
        assert_eq!(latencia_del_cliente_ms(&GRANDE), Some(3_100));
        assert!(!std::ptr::eq(latencia_del_cliente_de(&GRANDE), latencia_de(&GRANDE)), "dos cifras, dos sitios");
        assert!(!std::ptr::eq(latencia_del_cliente_de(&PEQUENO), latencia_de(&PEQUENO)));
        assert!(!std::ptr::eq(latencia_del_cliente_de(&PEQUENO), latencia_del_cliente_de(&GRANDE)));
    }

    /// Sin modelo bajado, whisper dice «falta el modelo» y no transcribe nada inventado.
    #[test]
    fn sin_modelo_no_transcribe() {
        if el_modelo().is_some() {
            return; // un PC con el modelo de verdad: esta prueba es para la CI y para el Mac
        }
        assert_eq!(Whisper.disponibilidad("es-ES"), Disponibilidad::SinModelo);
        assert_eq!(Whisper.transcribir("es-ES", &[0.0; 16_000], 16_000), Err(Fallo::NoDisponible(Disponibilidad::SinModelo)));
        assert_eq!(Whisper.transcribir("fr-FR", &[0.0; 16_000], 16_000), Err(Fallo::NoDisponible(Disponibilidad::IdiomaDesconocido)));
        assert_eq!(Whisper.techo_de_idiomas(), 2);
    }
}
