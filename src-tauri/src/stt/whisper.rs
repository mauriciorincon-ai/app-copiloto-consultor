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

/// Vuelve a poner las medidas que la app guardó la vez anterior (sprint 008, M3 del S6). Lo que ya se midió en esta
/// sesión manda sobre lo guardado. Este módulo no escribe nada: quien guarda es `prefs`, que recibe solo dos números.
pub fn restaurar_latencias(grande_ms: Option<u32>, pequeno_ms: Option<u32>) {
    restaurar_en((&LATENCIA_GRANDE, grande_ms), (&LATENCIA_PEQUENO, pequeno_ms));
}

/// La parte pura: cada medida guardada entra solo si ese modelo aún no se midió en esta sesión.
fn restaurar_en(grande: (&AtomicU32, Option<u32>), pequeno: (&AtomicU32, Option<u32>)) {
    for (donde, ms) in [grande, pequeno] {
        if let Some(ms) = ms.filter(|&ms| ms > 0) {
            let _ = donde.compare_exchange(0, ms, Ordering::Relaxed, Ordering::Relaxed);
        }
    }
}

/// **Con qué modelo se transcribe** (ADR 022 §5): el grande; el pequeño solo si el grande es el único que falta, o si
/// en este PC el grande se pasó del techo de 4 s y el pequeño ya está bajado.
pub fn elegir(grande_listo: bool, pequeno_listo: bool, latencia_grande: Option<u32>) -> Option<Modelo> {
    let grande_lento = latencia_grande.is_some_and(super::calibracion::pasa_del_techo);
    match (grande_listo, pequeno_listo) {
        (true, true) if grande_lento => Some(PEQUENO),
        (true, _) => Some(GRANDE),
        (false, true) => Some(PEQUENO),
        (false, false) => None,
    }
}

/// El modelo con el que se transcribiría ahora mismo, y su ruta. `None`: no hay ninguno entero.
pub fn el_modelo() -> Option<(Modelo, PathBuf)> {
    let grande = modelo_whisper::listo(&GRANDE);
    let pequeno = modelo_whisper::listo(&PEQUENO);
    match elegir(grande.is_some(), pequeno.is_some(), latencia_ms(&GRANDE))? {
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
    /// **Baja el modelo grande** (1,6 GB, con su huella) y mide lo que tarda en este PC. Solo desde «Instalar» en
    /// Idioma, y jamás en reunión (lo impide `lib.rs::instalar_idioma`).
    fn instalar(&self, idioma: &str) -> Disponibilidad {
        if idioma_de_whisper(idioma).is_none() {
            return Disponibilidad::IdiomaDesconocido;
        }
        match modelo_whisper::bajar(&GRANDE) {
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

    pub fn calibrar() -> Result<u32, Fallo> {
        let audio = crate::stt::calibracion::muestras();
        let codigo_whisper = idioma_de_whisper(crate::stt::calibracion::IDIOMA).unwrap_or("es");
        let (modelo, ms) = con_el_modelo(|modelo, estado| {
            let inicio = std::time::Instant::now();
            transcribir_en(estado, codigo_whisper, &audio)?;
            Ok((modelo, inicio.elapsed().as_millis().min(u128::from(u32::MAX)) as u32))
        })?;
        anotar_latencia(&modelo, ms);
        println!("[stt] whisper: {} tardó {ms} ms en 10 s de calibración", modelo.nombre);
        Ok(ms)
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
        assert_eq!(elegir(true, true, None), Some(GRANDE), "sin medir, el grande");
        assert_eq!(elegir(true, true, Some(3_000)), Some(GRANDE));
        assert_eq!(elegir(true, true, Some(6_000)), Some(PEQUENO));
        assert_eq!(elegir(true, false, Some(6_000)), Some(GRANDE), "lento, pero el pequeño no está: el grande");
        assert_eq!(elegir(false, true, None), Some(PEQUENO));
        assert_eq!(elegir(false, false, None), None);
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
        assert_eq!(elegir(true, true, Some(grande.load(Ordering::Relaxed))), Some(PEQUENO));
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
