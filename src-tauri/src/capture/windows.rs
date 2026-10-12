//! El grifo de Windows (ADR 022): **el micrófono (sprint 006) y el audio del sistema (sprint 007) por WASAPI con
//! `cpal`**, detrás de la misma frontera que `nativo.rs` en macOS. Sigue siendo `capture/`: módulo protegido, ni
//! disco ni red.
//!
//! Los dos grifos dejan el audio en mono, lo llevan a 16 kHz y lo escriben en su anillo, igual que en el Mac:
//! - **el micrófono**: el dispositivo de entrada por defecto del PC (tu pista, o la sala);
//! - **el audio del sistema**: el *loopback* del dispositivo de salida por defecto, lo que el PC hace sonar, que en
//!   una videollamada es el cliente. `cpal` lo abre así cuando se le pide una entrada sobre un dispositivo de
//!   salida. Dos diferencias con el tap del Mac, cada una con su respuesta: el loopback **no entrega nada mientras
//!   nada suena** (el reloj de la pista lo rellena con silencio, `reloj.rs`), y **oye también la voz de la propia
//!   app** (el tap del Mac excluye su proceso; en Windows lo que la app dijo se marca como eco en la escucha).
//!
//! Y por dónde sale el sonido ([`salida_de_audio`]): la forma que el propio Windows declara del dispositivo de
//! salida por defecto —altavoces, auriculares, HDMI—. Es lo que impide que la ficha suene por los altavoces con el
//! cliente oyéndola.
//!
//! **Cada flujo vive en su propio hilo.** Ese hilo abre el dispositivo, arranca el flujo y espera la orden de
//! parar; el grifo solo guarda el canal para darla y el hilo para esperarlo. Así el grifo se mueve entre hilos sin
//! pedirle nada a `cpal`, y el flujo muere en el mismo hilo que lo creó. El hilo del sistema, mientras espera, es
//! también el latido del reloj **y quien vuelve a abrir el loopback si Windows cambia de salida** (auditoría del S7,
//! A4): el flujo nace atado al dispositivo que era el de por defecto; si el usuario pasa a unos auriculares, `cpal`
//! avisa con `StreamInvalidated` o `DeviceNotAvailable`, el flujo viejo ya no entrega nada y, sin reabrirlo, el reloj
//! lo rellenaría con silencio y la pista parecería sana mientras la app no oye al cliente.

use super::anillo::Anillo;
use super::reloj::{Reloj, LATIDO};
use super::remuestreo::Remuestreador;
use super::{NoAbrio, PorQueNoAbrio, PorQueNoSeSabe};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};
use std::time::{Duration, Instant};

/// Por dónde sale el sonido. La misma forma que en macOS: cruza al contrato tal cual.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case", tag = "salida")]
pub enum Salida {
    Altavoces,
    Auriculares,
    Otra { nombre: String },
    AltavozExterno { nombre: String },
    NoSeSabe { motivo: super::PorQueNoSeSabe, nombre: Option<String> },
}

impl Salida {
    /// ¿Hay que avisar de que las pistas pueden mezclarse? `None` cuando no se sabe. Lo que significa cada salida es
    /// lo mismo que en macOS.
    pub fn puede_haber_eco(&self) -> Option<bool> {
        match self {
            Salida::Altavoces | Salida::AltavozExterno { .. } => Some(true),
            Salida::Auriculares => Some(false),
            Salida::Otra { .. } | Salida::NoSeSabe { .. } => None,
        }
    }
}

/// **Por dónde sale el sonido en este PC** (sprint 007): la forma que Windows declara del dispositivo de salida por
/// defecto. Se pregunta en un hilo propio con COM multihilo, porque quien pregunta puede ser el hilo principal de
/// la ventana, que ya tiene COM en otro modo.
pub fn salida_de_audio() -> Salida {
    std::thread::Builder::new()
        .name("ag-salida".into())
        .spawn(preguntar_la_salida)
        .ok()
        .and_then(|h| h.join().ok())
        .unwrap_or(Salida::NoSeSabe { motivo: PorQueNoSeSabe::SinFuente, nombre: None })
}

fn preguntar_la_salida() -> Salida {
    use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
    use windows::Win32::Media::Audio::{
        eConsole, eRender, IMMDeviceEnumerator, MMDeviceEnumerator, PKEY_AudioEndpoint_FormFactor,
    };
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
    };
    // SAFETY: COM se inicia y se cierra en este mismo hilo, que nadie más usa; las interfaces se sueltan antes de
    // `CoUninitialize` porque viven dentro del cierre.
    unsafe {
        let iniciado = CoInitializeEx(None, COINIT_MULTITHREADED).is_ok();
        let leida = (|| -> windows::core::Result<Option<Salida>> {
            let enumerador: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;
            let Ok(dispositivo) = enumerador.GetDefaultAudioEndpoint(eRender, eConsole) else {
                return Ok(None);
            };
            let propiedades = dispositivo.OpenPropertyStore(STGM_READ)?;
            let forma = propiedades.GetValue(&PKEY_AudioEndpoint_FormFactor).ok().and_then(|v| u32::try_from(&v).ok());
            let nombre =
                propiedades.GetValue(&PKEY_Device_FriendlyName).ok().map(|v| v.to_string()).filter(|n| !n.is_empty());
            Ok(Some(clasificar_forma(forma, nombre)))
        })();
        if iniciado {
            CoUninitialize();
        }
        match leida {
            Ok(Some(salida)) => salida,
            // Sin dispositivo de salida por defecto: no hay por dónde sonar.
            Ok(None) => Salida::NoSeSabe { motivo: PorQueNoSeSabe::SinSalida, nombre: None },
            Err(_) => Salida::NoSeSabe { motivo: PorQueNoSeSabe::SinFuente, nombre: None },
        }
    }
}

/// **La forma del dispositivo, en la salida que la app sabe enseñar.** Los números son los de `EndpointFormFactor`
/// de Windows: 1 altavoces · 3 auriculares · 5 cascos con micrófono · 6 auricular de mano · 8 S/PDIF · 9 pantalla
/// por HDMI o DisplayPort. Lo demás (línea, red, desconocido) se nombra y no se supone. Pura.
pub fn clasificar_forma(forma: Option<u32>, nombre: Option<String>) -> Salida {
    match (forma, nombre) {
        (Some(1), _) => Salida::Altavoces,
        (Some(3 | 5 | 6), _) => Salida::Auriculares,
        (Some(8 | 9), Some(nombre)) => Salida::AltavozExterno { nombre },
        (Some(8 | 9), None) => Salida::AltavozExterno { nombre: "HDMI o S/PDIF".into() },
        (Some(_), Some(nombre)) => Salida::Otra { nombre },
        (_, nombre) => Salida::NoSeSabe { motivo: PorQueNoSeSabe::SinFuente, nombre },
    }
}

/// Un grifo de audio abierto. Se cierra solo al soltarlo.
pub struct Grifo {
    parar: Option<mpsc::Sender<()>>,
    hilo: Option<std::thread::JoinHandle<()>>,
    entradas: Arc<AtomicU64>,
    pub hz_del_dispositivo: u32,
}

impl Grifo {
    /// Cuántas muestras (ya a 16 kHz) han entrado por este grifo desde que se abrió. Cero no es avería: si el
    /// micrófono no entrega nada, no hay nada que contar.
    pub fn muestras_recibidas(&self) -> u64 {
        self.entradas.load(Ordering::Relaxed)
    }

    /// Abre el micrófono: el dispositivo de entrada por defecto del PC.
    pub fn del_microfono(anillo: Arc<Mutex<Anillo>>) -> Result<Self, NoAbrio> {
        crate::hardware::vigilar("el micrófono");
        Self::en_su_hilo(Cual::Microfono, anillo)
    }

    /// Abre el audio del sistema (sprint 007): el *loopback* del dispositivo de salida por defecto, la pista del
    /// cliente en una videollamada.
    pub fn del_sistema(anillo: Arc<Mutex<Anillo>>) -> Result<Self, NoAbrio> {
        crate::hardware::vigilar("el audio del sistema");
        Self::en_su_hilo(Cual::Sistema, anillo)
    }

    fn en_su_hilo(cual: Cual, anillo: Arc<Mutex<Anillo>>) -> Result<Self, NoAbrio> {
        let entradas = Arc::new(AtomicU64::new(0));
        let cuenta = entradas.clone();
        let (listo, abierto) = mpsc::channel::<Result<u32, NoAbrio>>();
        let (parar, orden) = mpsc::channel::<()>();
        let hilo = std::thread::Builder::new()
            .name(cual.hilo().into())
            .spawn(move || {
                // El reloj solo hace falta donde el dispositivo puede callarse del todo: el loopback.
                let reloj = match cual {
                    Cual::Sistema => Some(Arc::new(Reloj::nuevo(anillo.clone()))),
                    Cual::Microfono => None,
                };
                let escritor = match &reloj {
                    Some(r) => Escritor::ConReloj(r.clone()),
                    None => Escritor::Directo(anillo),
                };
                // El callback de error lo levanta cuando `cpal` dice que el flujo ya no sirve.
                let roto = Arc::new(AtomicBool::new(false));
                let flujo = match abrir(cual, escritor, cuenta.clone(), roto.clone()) {
                    Ok((flujo, hz)) => {
                        let _ = listo.send(Ok(hz));
                        flujo
                    }
                    Err(no) => {
                        let _ = listo.send(Err(no));
                        return;
                    }
                };
                let mut flujo = Some(flujo);
                // Espera la orden de parar; si el grifo se suelta sin darla, el canal se cierra y también vuelve.
                // Con reloj, la espera late: cada `LATIDO` sin orden se pone el reloj al día y, si el flujo se
                // rompió, se vuelve a abrir sobre la salida de ahora.
                match reloj {
                    None => {
                        let _ = orden.recv();
                    }
                    Some(r) => {
                        let mut ultimo_intento: Option<Instant> = None;
                        loop {
                            match orden.recv_timeout(LATIDO) {
                                Err(mpsc::RecvTimeoutError::Timeout) => {
                                    let ahora = Instant::now();
                                    r.al_dia(ahora);
                                    let desde = ultimo_intento.map(|u| ahora.duration_since(u));
                                    if toca_reabrir(roto.load(Ordering::Relaxed), desde) {
                                        ultimo_intento = Some(ahora);
                                        // El flujo roto se suelta antes de abrir el nuevo: dos sobre la misma salida no caben.
                                        drop(flujo.take());
                                        roto.store(false, Ordering::Relaxed);
                                        match abrir(cual, Escritor::ConReloj(r.clone()), cuenta.clone(), roto.clone()) {
                                            Ok((nuevo, _)) => {
                                                flujo = Some(nuevo);
                                                println!("[captura] {} volvió a abrirse: Windows cambió de salida", cual.que());
                                            }
                                            Err(no) => {
                                                roto.store(true, Ordering::Relaxed);
                                                // Solo el porqué: es un metadato, jamás audio (ADR 003).
                                                println!("[captura] {} no pudo volver a abrirse: {:?}", cual.que(), no.porque);
                                            }
                                        }
                                    }
                                }
                                _ => break,
                            }
                        }
                    }
                }
                drop(flujo);
            })
            .map_err(|e| NoAbrio::por(PorQueNoAbrio::NoDejo, format!("no se pudo crear el hilo de {}: {e}", cual.que())))?;
        match abierto.recv() {
            Ok(Ok(hz)) => Ok(Self { parar: Some(parar), hilo: Some(hilo), entradas, hz_del_dispositivo: hz }),
            Ok(Err(no)) => {
                let _ = hilo.join();
                Err(no)
            }
            Err(_) => {
                let _ = hilo.join();
                Err(NoAbrio::por(PorQueNoAbrio::NoDejo, format!("el hilo de {} terminó sin contestar", cual.que())))
            }
        }
    }
}

/// Qué abre un grifo. No es la pista (eso lo decide la escucha): es el dispositivo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cual {
    Microfono,
    Sistema,
}

impl Cual {
    fn hilo(self) -> &'static str {
        match self {
            Cual::Microfono => "ag-microfono",
            Cual::Sistema => "ag-sistema",
        }
    }

    fn que(self) -> &'static str {
        match self {
            Cual::Microfono => "el micrófono",
            Cual::Sistema => "el audio del sistema",
        }
    }
}

impl Drop for Grifo {
    fn drop(&mut self) {
        if let Some(p) = self.parar.take() {
            let _ = p.send(());
        }
        if let Some(h) = self.hilo.take() {
            let _ = h.join();
        }
    }
}

/// Por dónde llega el audio al anillo: directo (el micrófono, que no se calla nunca) o por el reloj (el loopback).
enum Escritor {
    Directo(Arc<Mutex<Anillo>>),
    ConReloj(Arc<Reloj>),
}

/// Lo que el callback necesita tener a mano. Vive dentro del cierre del flujo y muere con él.
struct Destino {
    escritor: Escritor,
    remuestreador: Remuestreador,
    entradas: Arc<AtomicU64>,
    canales: u16,
}

impl Destino {
    /// El callback del audio. Corre en el hilo de audio de WASAPI: sin imprimir y sin entrar en pánico; lo que
    /// no se puede hacer, se salta.
    fn recibir<T>(&mut self, datos: &[T])
    where
        T: Copy,
        f32: FromSample<T>,
    {
        let mono = mezclar(datos, self.canales);
        let a_dieciseis = self.remuestreador.convertir(&mono);
        if a_dieciseis.is_empty() {
            return;
        }
        // Solo lo que de verdad entró: el silencio del reloj no es audio recibido.
        self.entradas.fetch_add(a_dieciseis.len() as u64, Ordering::Relaxed);
        // El mismo candado corto que en macOS (`nativo.rs`): copiar unos cientos de flotantes.
        match &self.escritor {
            Escritor::Directo(anillo) => {
                if let Ok(mut a) = anillo.lock() {
                    a.escribir(&a_dieciseis);
                }
            }
            Escritor::ConReloj(reloj) => reloj.escribir(&a_dieciseis, Instant::now()),
        }
    }
}

/// Deja en mono lo que WASAPI entrega: muestras **entrelazadas** (izquierda, derecha, izquierda…). Se promedian
/// los canales, como en macOS: unos auriculares mal balanceados pueden llevar casi toda la voz en uno solo.
fn mezclar<T>(datos: &[T], canales: u16) -> Vec<f32>
where
    T: Copy,
    f32: FromSample<T>,
{
    let canales = usize::from(canales.max(1));
    datos
        .chunks(canales)
        .map(|m| m.iter().map(|&x| f32::from_sample_(x)).sum::<f32>() / m.len() as f32)
        .collect()
}

/// Abre el dispositivo por defecto de `cual` y arranca su flujo. Devuelve el flujo y los hercios del dispositivo.
///
/// El audio del sistema es un flujo **de entrada sobre el dispositivo de salida**: así lo pide `cpal` para el
/// *loopback* de WASAPI, con el formato del propio dispositivo de salida (el que mezcla Windows).
///
/// `roto` lo levanta el callback de error cuando `cpal` dice que el flujo ya no sirve (`pide_reabrir`).
fn abrir(
    cual: Cual,
    escritor: Escritor,
    entradas: Arc<AtomicU64>,
    roto: Arc<AtomicBool>,
) -> Result<(cpal::Stream, u32), NoAbrio> {
    let host = cpal::default_host();
    let no = |e: cpal::Error| no_abrio(e, cual);
    let (dispositivo, formato) = match cual {
        Cual::Microfono => {
            let d = host
                .default_input_device()
                .ok_or_else(|| NoAbrio::por(PorQueNoAbrio::NoDejo, "este PC no tiene un micrófono por defecto"))?;
            let f = d.default_input_config().map_err(no)?;
            (d, f)
        }
        Cual::Sistema => {
            let d = host.default_output_device().ok_or_else(|| {
                NoAbrio::por(PorQueNoAbrio::NoDejo, "este PC no tiene un dispositivo de salida por defecto")
            })?;
            let f = d.default_output_config().map_err(no)?;
            (d, f)
        }
    };
    let hz = formato.sample_rate();
    let destino = Destino {
        escritor,
        remuestreador: Remuestreador::nuevo(hz, super::anillo::HZ),
        entradas,
        canales: formato.channels(),
    };
    let config = formato.config();
    let flujo = match formato.sample_format() {
        SampleFormat::F32 => construir::<f32>(cual, &dispositivo, config, destino, roto),
        SampleFormat::F64 => construir::<f64>(cual, &dispositivo, config, destino, roto),
        SampleFormat::I16 => construir::<i16>(cual, &dispositivo, config, destino, roto),
        SampleFormat::I32 => construir::<i32>(cual, &dispositivo, config, destino, roto),
        SampleFormat::I8 => construir::<i8>(cual, &dispositivo, config, destino, roto),
        SampleFormat::U16 => construir::<u16>(cual, &dispositivo, config, destino, roto),
        SampleFormat::U8 => construir::<u8>(cual, &dispositivo, config, destino, roto),
        otro => {
            return Err(NoAbrio::por(
                PorQueNoAbrio::FormatoIlegible,
                format!("{} entrega muestras {otro}, que el grifo no sabe leer", cual.que()),
            ))
        }
    }
    .map_err(no)?;
    flujo.play().map_err(no)?;
    Ok((flujo, hz))
}

fn construir<T>(
    cual: Cual,
    dispositivo: &cpal::Device,
    config: cpal::StreamConfig,
    mut destino: Destino,
    roto: Arc<AtomicBool>,
) -> Result<cpal::Stream, cpal::Error>
where
    T: cpal::SizedSample,
    f32: FromSample<T>,
{
    dispositivo.build_input_stream::<T, _, _>(
        config,
        move |datos: &[T], _: &cpal::InputCallbackInfo| destino.recibir(datos),
        // Solo el tipo del aviso: es un metadato, jamás audio (ADR 003).
        move |e: cpal::Error| {
            println!("[captura] {} avisó: {}", cual.que(), e.kind());
            if pide_reabrir(e.kind()) {
                roto.store(true, Ordering::Relaxed);
            }
        },
        None,
    )
}

/// Cada cuánto se reintenta abrir el loopback si falló al volver a abrirse.
const REINTENTO: Duration = Duration::from_secs(1);

/// ¿Este aviso de `cpal` dice que el flujo ya no sirve y hay que abrirlo otra vez? Cuando el dispositivo por defecto
/// cambia, el backend de WASAPI no reata el flujo (lo dice su código) y avisa con `StreamInvalidated` si hay otra
/// salida por defecto, o con `DeviceNotAvailable` si la salida se fue. Lo demás —un `Xrun`, un dispositivo ocupado, un
/// `DeviceChanged`, que `cpal` documenta como «el flujo sigue activo»— no rompe el flujo y no se reabre.
fn pide_reabrir(tipo: cpal::ErrorKind) -> bool {
    matches!(tipo, cpal::ErrorKind::StreamInvalidated | cpal::ErrorKind::DeviceNotAvailable)
}

/// ¿Toca volver a abrir? Con el flujo roto, ya la primera vez; si ese intento falló, una vez por `REINTENTO`.
fn toca_reabrir(roto: bool, desde_el_ultimo_intento: Option<Duration>) -> bool {
    roto && desde_el_ultimo_intento.is_none_or(|d| d >= REINTENTO)
}

/// El «no» de WASAPI, en el conjunto cerrado que enseña la pantalla. El detalle va al log. El permiso que falta
/// depende de qué se abría.
fn no_abrio(e: cpal::Error, cual: Cual) -> NoAbrio {
    let porque = match e.kind() {
        cpal::ErrorKind::PermissionDenied => match cual {
            Cual::Microfono => PorQueNoAbrio::SinPermisoDelMicrofono,
            Cual::Sistema => PorQueNoAbrio::SinPermisoDelAudio,
        },
        cpal::ErrorKind::DeviceBusy => PorQueNoAbrio::DispositivoOcupado,
        cpal::ErrorKind::UnsupportedConfig => PorQueNoAbrio::FormatoIlegible,
        _ => PorQueNoAbrio::NoDejo,
    };
    NoAbrio::por(porque, e.to_string())
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **La mezcla promedia los canales**, no se queda con el primero: una voz que solo entra por la derecha
    /// sigue oyéndose.
    #[test]
    fn la_mezcla_promedia_los_canales() {
        let estereo = [0.0f32, 0.8, 0.0, 0.8, 0.0, 0.8];
        assert_eq!(mezclar(&estereo, 2), vec![0.4, 0.4, 0.4]);
        assert_eq!(mezclar(&[0.5f32, -0.5], 1), vec![0.5, -0.5]);
        assert_eq!(mezclar(&[0.5f32, -0.5], 0), vec![0.5, -0.5], "cero canales se lee como mono, sin dividir por cero");
    }

    /// Las muestras enteras se llevan a flotantes entre -1 y 1 antes de mezclar.
    #[test]
    fn las_muestras_enteras_se_llevan_a_flotante() {
        let m = mezclar(&[i16::MAX, i16::MAX], 2);
        assert!((m[0] - 1.0).abs() < 1e-3, "{m:?}");
        let m = mezclar(&[u16::MIN], 1);
        assert!((m[0] + 1.0).abs() < 1e-3, "{m:?}");
    }

    /// Lo que WASAPI dice que no se traduce al porqué que la pantalla sabe enseñar, y el permiso que falta es el de
    /// lo que se abría.
    #[test]
    fn el_no_de_wasapi_se_dice_con_su_porque() {
        let no = no_abrio(cpal::Error::new(cpal::ErrorKind::PermissionDenied), Cual::Microfono);
        assert_eq!(no.porque, PorQueNoAbrio::SinPermisoDelMicrofono);
        let no = no_abrio(cpal::Error::new(cpal::ErrorKind::PermissionDenied), Cual::Sistema);
        assert_eq!(no.porque, PorQueNoAbrio::SinPermisoDelAudio);
        let no = no_abrio(cpal::Error::new(cpal::ErrorKind::DeviceBusy), Cual::Sistema);
        assert_eq!(no.porque, PorQueNoAbrio::DispositivoOcupado);
        let no = no_abrio(cpal::Error::new(cpal::ErrorKind::BackendError), Cual::Microfono);
        assert_eq!(no.porque, PorQueNoAbrio::NoDejo);
    }

    /// **La forma del dispositivo decide si la voz puede sonar**: altavoces y HDMI la oiría el cliente; auriculares y
    /// cascos, no; lo que Windows no clasifica se nombra y no se supone.
    #[test]
    fn la_forma_del_dispositivo_dice_por_donde_sale() {
        assert_eq!(clasificar_forma(Some(1), Some("Realtek".into())), Salida::Altavoces);
        for cascos in [3, 5, 6] {
            assert_eq!(clasificar_forma(Some(cascos), None), Salida::Auriculares, "forma {cascos}");
        }
        assert_eq!(
            clasificar_forma(Some(9), Some("DELL U2723QE".into())),
            Salida::AltavozExterno { nombre: "DELL U2723QE".into() }
        );
        assert_eq!(clasificar_forma(Some(8), None).puede_haber_eco(), Some(true));
        assert_eq!(clasificar_forma(Some(2), Some("Line Out".into())), Salida::Otra { nombre: "Line Out".into() });
        assert_eq!(clasificar_forma(Some(10), None).puede_haber_eco(), None);
        assert_eq!(clasificar_forma(None, None).puede_haber_eco(), None);
        assert_eq!(clasificar_forma(Some(1), None).puede_haber_eco(), Some(true));
        assert_eq!(clasificar_forma(Some(3), None).puede_haber_eco(), Some(false));
    }

    /// **El loopback se vuelve a abrir cuando Windows cambia de salida** (auditoría del S7, A4): solo los avisos que
    /// invalidan el flujo lo piden, y un intento que falló no se repite en cada latido.
    #[test]
    fn el_flujo_roto_se_vuelve_a_abrir_sin_prisa_de_mas() {
        use cpal::ErrorKind::*;
        for k in [StreamInvalidated, DeviceNotAvailable] {
            assert!(pide_reabrir(k), "{k:?} invalida el flujo");
        }
        for k in [Xrun, DeviceBusy, DeviceChanged, PermissionDenied, BackendError] {
            assert!(!pide_reabrir(k), "{k:?} no rompe el flujo");
        }
        assert!(!toca_reabrir(false, None), "sin avisos no se toca nada");
        assert!(toca_reabrir(true, None), "roto y sin intento previo: ya");
        assert!(!toca_reabrir(true, Some(Duration::from_millis(400))), "no más de un intento por segundo");
        assert!(toca_reabrir(true, Some(REINTENTO)), "pasado el segundo, otra vez");
    }

    /// Los dos grifos tienen su hilo con nombre y su texto: lo que sale en el log dice qué se abría.
    #[test]
    fn cada_grifo_se_llama_por_lo_que_abre() {
        assert_eq!(Cual::Microfono.hilo(), "ag-microfono");
        assert_eq!(Cual::Sistema.hilo(), "ag-sistema");
        assert_eq!(Cual::Sistema.que(), "el audio del sistema");
    }

    /// **El micrófono de verdad** (regla 25): abre y cuenta muestras. Solo con alguien delante del PC.
    #[test]
    #[ignore = "en vivo: abre el micrófono del PC, con alguien delante"]
    fn en_vivo_el_microfono_abre_y_cuenta() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let grifo = Grifo::del_microfono(anillo).expect("el micrófono no abrió");
        std::thread::sleep(std::time::Duration::from_millis(500));
        assert!(grifo.hz_del_dispositivo > 0);
        println!("[captura] {} muestras en medio segundo", grifo.muestras_recibidas());
    }

    /// **El audio del sistema de verdad** (regla 25): el loopback abre y su reloj sigue la hora aunque el PC no haga
    /// sonar nada. Solo con alguien delante del PC.
    #[test]
    #[ignore = "en vivo: abre el audio del sistema del PC, con alguien delante"]
    fn en_vivo_el_audio_del_sistema_sigue_la_hora() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let grifo = Grifo::del_sistema(anillo.clone()).expect("el audio del sistema no abrió");
        std::thread::sleep(std::time::Duration::from_millis(1_000));
        let totales = anillo.lock().unwrap().totales();
        assert!(totales >= 12_000, "un segundo de pista, haya sonido o no: {totales}");
        println!("[captura] {} muestras recibidas, {totales} en el reloj", grifo.muestras_recibidas());
    }

    /// **El audio del sistema sigue a la salida** (auditoría del S7, A4): con la sesión abierta, el usuario pasa la
    /// salida de los altavoces a unos auriculares y el loopback vuelve a entregar audio real. Solo con alguien delante
    /// del PC.
    #[test]
    #[ignore = "en vivo: pide cambiar la salida de audio del PC y hacer sonar algo, con alguien delante"]
    fn en_vivo_el_audio_del_sistema_sigue_al_cambiar_de_salida() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let grifo = Grifo::del_sistema(anillo).expect("el audio del sistema no abrió");
        println!("[captura] AHORA: pasa la salida de audio del PC a otro dispositivo (unos auriculares) en 15 s");
        std::thread::sleep(Duration::from_secs(15));
        let tras_el_cambio = grifo.muestras_recibidas();
        println!("[captura] AHORA: haz sonar algo en la nueva salida durante 8 s");
        std::thread::sleep(Duration::from_secs(8));
        let despues = grifo.muestras_recibidas();
        assert!(despues > tras_el_cambio, "el audio del sistema calló tras el cambio de salida: {tras_el_cambio} → {despues}");
    }
}
