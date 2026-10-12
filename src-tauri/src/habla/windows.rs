//! **La voz de Windows** (sprint 007, ADR 022): la ficha al oído y las preguntas del ensayo, con el sintetizador del
//! sistema (`Windows.Media.SpeechSynthesis`). Sigue siendo `habla/`: módulo protegido, ni disco ni red.
//!
//! Tres piezas, en un hilo propio (`ag-voz`) que es el único que toca WinRT y el sonido:
//! - **la voz** de cada idioma: la primera del sistema con ese idioma exacto, y si no, con el mismo idioma base;
//! - **la frase**: `SynthesizeTextToStreamAsync` la entrega como un WAV **en memoria**, que `pcm.rs` lee. Nada se
//!   escribe en disco: un WAV con la ficha leída sería una grabación con otro nombre;
//! - **el sonido**: un flujo de salida de `cpal` sobre el dispositivo por defecto. Si suena por altavoces o por
//!   auriculares lo decide antes `cabe_decirla`, con la salida real (`capture/windows.rs`).
//!
//! Callar es cambiar de generación: lo que suena se corta y lo que esperaba en la cola se tira. Y cada vez que
//! suena, lo apunta en `super::empieza_a_sonar`: el loopback de Windows también oye la voz de la app, y la escucha
//! la marca como eco en vez de tomarla por el cliente.

use super::{pcm, Voz};
use crate::capture::remuestreo::Remuestreador;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use windows::core::HSTRING;
use windows::Media::SpeechSynthesis::{SpeechSynthesizer, VoiceInformation};
use windows::Storage::Streams::{Buffer, DataReader, InputStreamOptions};

/// La voz del sistema, compartida: hay un solo hilo que habla, igual que en el Mac hay un solo sintetizador.
pub struct DelSistema;

/// Lo que se le pide al hilo de la voz: decir una frase. **«¿Hay voz para este idioma?» no pasa por aquí** (auditoría
/// del S7, A3): ese hilo está ocupado mientras suena la frase, y quien preguntaba —el hilo que transcribe, o la
/// ventana— se quedaba esperando detrás de ella hasta 3 s y recibía un «no» falso.
struct Orden {
    idioma: String,
    texto: String,
    generacion: u64,
}

/// Lo que se le pregunta al hilo de las preguntas: si hay voz para un idioma.
struct Pregunta {
    idioma: String,
    responde: mpsc::Sender<bool>,
}

/// El hilo de las preguntas (`ag-voz-pregunta`): otro hilo, con su propio COM multihilo, que vive lo que la app como el de
/// la voz y solo contesta «¿hay voz para este idioma?». Un hilo corto que iniciara y cerrara COM en cada pregunta tumbó
/// el proceso de pruebas de Windows (acceso a memoria inválido al desmontar COM bajo WinRT): el ciclo de vida de COM es
/// el mismo que el del hilo de la voz, que ya funcionaba.
fn preguntador() -> &'static Mutex<mpsc::Sender<Pregunta>> {
    static UNO: OnceLock<Mutex<mpsc::Sender<Pregunta>>> = OnceLock::new();
    UNO.get_or_init(|| {
        let (manda, recibe) = mpsc::channel::<Pregunta>();
        let _ = std::thread::Builder::new().name("ag-voz-pregunta".into()).spawn(move || contestar(recibe));
        Mutex::new(manda)
    })
}

fn contestar(recibe: mpsc::Receiver<Pregunta>) {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
    // SAFETY: COM se inicia una vez en este hilo, que vive lo que la app.
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    for Pregunta { idioma, responde } in recibe {
        let _ = responde.send(voz_para(&idioma).is_some());
    }
}

struct Trabajador {
    orden: Mutex<mpsc::Sender<Orden>>,
    estado: Arc<Estado>,
}

#[derive(Default)]
struct Estado {
    /// Frases pedidas que todavía no acabaron (sonando o en la cola).
    pendientes: AtomicUsize,
    /// Callar sube la generación: lo de una generación vieja no suena.
    generacion: AtomicU64,
}

fn trabajador() -> &'static Trabajador {
    static UNO: OnceLock<Trabajador> = OnceLock::new();
    UNO.get_or_init(|| {
        let (manda, recibe) = mpsc::channel();
        let estado = Arc::new(Estado::default());
        let suyo = estado.clone();
        let _ = std::thread::Builder::new().name("ag-voz".into()).spawn(move || trabajar(recibe, suyo));
        Trabajador { orden: Mutex::new(manda), estado }
    })
}

/// La voz de este PC.
pub fn voz() -> Box<dyn Voz> {
    Box::new(DelSistema)
}

impl Voz for DelSistema {
    fn nombre(&self) -> &'static str {
        "windows-speechsynthesizer"
    }

    /// Se pregunta al **hilo de las preguntas**, no al que habla: ese está ocupado mientras suena la frase.
    fn hay_para(&self, idioma: &str) -> bool {
        let (responde, respuesta) = mpsc::channel();
        let pedido = preguntador()
            .lock()
            .is_ok_and(|p| p.send(Pregunta { idioma: idioma.into(), responde }).is_ok());
        pedido && respuesta.recv_timeout(Duration::from_secs(3)).unwrap_or(false)
    }

    fn decir(&self, idioma: &str, texto: &str) -> Result<(), String> {
        if texto.trim().is_empty() {
            return Err("no hay nada que decir".into());
        }
        if !self.hay_para(idioma) {
            return Err(format!("este PC no tiene voz para «{idioma}»"));
        }
        crate::hardware::vigilar("los altavoces (la voz de la app)");
        let t = trabajador();
        let generacion = t.estado.generacion.load(Ordering::SeqCst);
        t.estado.pendientes.fetch_add(1, Ordering::SeqCst);
        let mandada = t
            .orden
            .lock()
            .is_ok_and(|o| o.send(Orden { idioma: idioma.into(), texto: texto.into(), generacion }).is_ok());
        if !mandada {
            t.estado.pendientes.fetch_sub(1, Ordering::SeqCst);
            return Err("el hilo de la voz no está".into());
        }
        Ok(())
    }

    fn callar(&self) {
        trabajador().estado.generacion.fetch_add(1, Ordering::SeqCst);
    }

    fn hablando(&self) -> bool {
        trabajador().estado.pendientes.load(Ordering::SeqCst) > 0
    }
}

/// El hilo de la voz: COM multihilo, un sintetizador, y la cola.
fn trabajar(recibe: mpsc::Receiver<Orden>, estado: Arc<Estado>) {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
    // SAFETY: COM se inicia una vez en este hilo, que vive lo que la app.
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    let sintetizador = SpeechSynthesizer::new().ok();
    for Orden { idioma, mut texto, generacion } in recibe {
        if estado.generacion.load(Ordering::SeqCst) == generacion {
            if let Err(e) = decir_ahora(sintetizador.as_ref(), &idioma, &texto, generacion, &estado) {
                // El motivo es nuestro o de Windows, jamás la frase (ADR 003).
                println!("[habla] la voz de Windows no sonó: {e}");
            }
        }
        // La frase es evidencia del consultor: se pisa antes de soltarse.
        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
        unsafe { texto.as_mut_vec() }.fill(0);
        estado.pendientes.fetch_sub(1, Ordering::SeqCst);
    }
}

/// La voz del sistema para un idioma: el idioma exacto («es-ES») y, si no hay, el mismo idioma base («es»).
fn voz_para(idioma: &str) -> Option<VoiceInformation> {
    let todas = SpeechSynthesizer::AllVoices().ok()?;
    let n = todas.Size().ok()?;
    let voces: Vec<(VoiceInformation, String)> = (0..n)
        .filter_map(|i| todas.GetAt(i).ok())
        .filter_map(|v| v.Language().ok().map(|l| (v, l.to_string())))
        .collect();
    let base = idioma.split('-').next().unwrap_or(idioma);
    voces
        .iter()
        .find(|(_, l)| l.eq_ignore_ascii_case(idioma))
        .or_else(|| voces.iter().find(|(_, l)| l.split('-').next().is_some_and(|b| b.eq_ignore_ascii_case(base))))
        .map(|(v, _)| v.clone())
}

/// Sintetiza la frase en memoria y la hace sonar. Vuelve al acabar o al callarla.
fn decir_ahora(
    sintetizador: Option<&SpeechSynthesizer>,
    idioma: &str,
    texto: &str,
    generacion: u64,
    estado: &Estado,
) -> Result<(), String> {
    let s = sintetizador.ok_or("Windows no dio un sintetizador de voz")?;
    let voz = voz_para(idioma).ok_or_else(|| format!("este PC no tiene voz para «{idioma}»"))?;
    let e = |e: windows::core::Error| format!("{e}");
    s.SetVoice(&voz).map_err(e)?;
    let flujo = s.SynthesizeTextToStreamAsync(&HSTRING::from(texto)).map_err(e)?.join().map_err(e)?;
    let largo = u32::try_from(flujo.Size().map_err(e)?).map_err(|_| "la frase sintetizada no cabe".to_string())?;
    let buffer = Buffer::Create(largo).map_err(e)?;
    let leido = flujo.ReadAsync(&buffer, largo, InputStreamOptions::None).map_err(e)?.join().map_err(e)?;
    let mut bytes = vec![0u8; leido.Length().map_err(e)? as usize];
    DataReader::FromBuffer(&leido).map_err(e)?.ReadBytes(&mut bytes).map_err(e)?;
    let frase = pcm::leer(&bytes);
    bytes.fill(0);
    let mut frase = frase?;
    super::empieza_a_sonar(Instant::now());
    let sonado = sonar(&frase, generacion, estado);
    super::deja_de_sonar(Instant::now());
    frase.muestras.fill(0.0);
    sonado
}

/// Las muestras que suenan, dentro del flujo. Se pisan al soltarse.
struct Sonando {
    muestras: Vec<f32>,
    posicion: Arc<AtomicUsize>,
    canales: usize,
}

impl Sonando {
    fn llenar<T: cpal::SizedSample + FromSample<f32>>(&mut self, salida: &mut [T]) {
        let mut p = self.posicion.load(Ordering::Relaxed);
        for marco in salida.chunks_mut(self.canales.max(1)) {
            let m = self.muestras.get(p).copied().unwrap_or(0.0);
            for s in marco.iter_mut() {
                *s = T::from_sample_(m);
            }
            p += 1;
        }
        self.posicion.store(p, Ordering::Relaxed);
    }
}

impl Drop for Sonando {
    fn drop(&mut self) {
        self.muestras.fill(0.0);
    }
}

/// Hace sonar la frase en el dispositivo de salida por defecto, a su frecuencia y en todos sus canales.
fn sonar(frase: &pcm::Frase, generacion: u64, estado: &Estado) -> Result<(), String> {
    let host = cpal::default_host();
    let dispositivo = host.default_output_device().ok_or("este PC no tiene un dispositivo de salida")?;
    let formato = dispositivo.default_output_config().map_err(|e| e.to_string())?;
    let hz = formato.sample_rate();
    let muestras = Remuestreador::nuevo(frase.hz, hz).convertir(&frase.muestras);
    let total = muestras.len();
    let posicion = Arc::new(AtomicUsize::new(0));
    let sonando = Sonando { muestras, posicion: posicion.clone(), canales: usize::from(formato.channels()) };
    let config = formato.config();
    let flujo = match formato.sample_format() {
        SampleFormat::F32 => construir::<f32>(&dispositivo, config, sonando),
        SampleFormat::I16 => construir::<i16>(&dispositivo, config, sonando),
        SampleFormat::U16 => construir::<u16>(&dispositivo, config, sonando),
        SampleFormat::I32 => construir::<i32>(&dispositivo, config, sonando),
        otro => return Err(format!("la salida pide muestras {otro}, que la voz no sabe dar")),
    }
    .map_err(|e| e.to_string())?;
    flujo.play().map_err(|e| e.to_string())?;
    // Espera a que suene entera —más lo que tarda el último bloque en salir— o a que la callen.
    let techo = Instant::now() + Duration::from_secs_f64(total as f64 / f64::from(hz.max(1))) + Duration::from_secs(2);
    while posicion.load(Ordering::Relaxed) < total
        && estado.generacion.load(Ordering::SeqCst) == generacion
        && Instant::now() < techo
    {
        std::thread::sleep(Duration::from_millis(20));
    }
    if estado.generacion.load(Ordering::SeqCst) == generacion {
        std::thread::sleep(Duration::from_millis(120));
    }
    drop(flujo);
    Ok(())
}

fn construir<T>(dispositivo: &cpal::Device, config: cpal::StreamConfig, mut sonando: Sonando) -> Result<cpal::Stream, cpal::Error>
where
    T: cpal::SizedSample + FromSample<f32>,
{
    dispositivo.build_output_stream::<T, _, _>(
        config,
        move |salida: &mut [T], _: &cpal::OutputCallbackInfo| sonando.llenar(salida),
        |e: cpal::Error| println!("[habla] la salida avisó: {}", e.kind()),
        None,
    )
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **Preguntar si hay voz no espera a que acabe de hablar** (auditoría del S7, A3). Con una frase larga sonando,
    /// `hay_para` contesta al momento y con la verdad. Solo con alguien delante del PC: hace sonar la voz.
    #[test]
    #[ignore = "en vivo: hace sonar la voz del PC por su salida, con alguien delante"]
    fn en_vivo_hay_para_no_espera_a_que_acabe_de_hablar() {
        let voz = DelSistema;
        voz.decir(
            "es-ES",
            "Esta es una frase larga para que la voz tarde varios segundos en decirla, y la pregunta de si hay voz \
             no tenga que esperar detrás de ella.",
        )
        .expect("este PC no tiene voz para es-ES");
        std::thread::sleep(Duration::from_millis(500));
        let antes = Instant::now();
        let hay = voz.hay_para("es-ES");
        let tarda = antes.elapsed();
        voz.callar();
        assert!(hay, "con la voz hablando, hay_para dijo que no hay voz");
        assert!(tarda < Duration::from_millis(500), "hay_para esperó {tarda:?} detrás de la frase");
    }
}
