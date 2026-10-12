//! El grifo de Windows (sprint 006, ADR 022): **el micrófono por WASAPI con `cpal`**, detrás de la misma
//! frontera que `nativo.rs` en macOS. Sigue siendo `capture/`: módulo protegido, ni disco ni red.
//!
//! Lo que hace hoy: abre el micrófono por defecto del PC, lo deja en mono, lo lleva a 16 kHz y lo escribe en el
//! anillo, igual que el grifo del Mac. Lo que todavía no, cada cosa con su motivo: el audio del sistema (el
//! *loopback* de WASAPI, que es la pista del cliente en una videollamada) llega con la videollamada en el sprint
//! 007; y saber si el sonido sale por auriculares o por altavoces, también.
//!
//! **El flujo vive en su propio hilo.** Ese hilo abre el dispositivo, arranca el flujo y espera la orden de parar;
//! el grifo solo guarda el canal para darla y el hilo para esperarlo. Así el grifo se mueve entre hilos sin pedirle
//! nada a `cpal`, y el flujo muere en el mismo hilo que lo creó.

use super::anillo::Anillo;
use super::remuestreo::Remuestreador;
use super::{NoAbrio, PorQueNoAbrio};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex};

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
    /// lo mismo que en macOS; lo que cambia es que en Windows, hasta el sprint 007, [`salida_de_audio`] contesta
    /// «no se sabe» en vez de suponer auriculares.
    pub fn puede_haber_eco(&self) -> Option<bool> {
        match self {
            Salida::Altavoces | Salida::AltavozExterno { .. } => Some(true),
            Salida::Auriculares => Some(false),
            Salida::Otra { .. } | Salida::NoSeSabe { .. } => None,
        }
    }
}

/// Por dónde sale el sonido en este PC: todavía no se pregunta (sprint 007, con la voz al oído).
pub fn salida_de_audio() -> Salida {
    Salida::NoSeSabe { motivo: super::PorQueNoSeSabe::SinSalida, nombre: None }
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
        let entradas = Arc::new(AtomicU64::new(0));
        let cuenta = entradas.clone();
        let (listo, abierto) = mpsc::channel::<Result<u32, NoAbrio>>();
        let (parar, orden) = mpsc::channel::<()>();
        let hilo = std::thread::Builder::new()
            .name("ag-microfono".into())
            .spawn(move || {
                let flujo = match abrir(anillo, cuenta) {
                    Ok((flujo, hz)) => {
                        let _ = listo.send(Ok(hz));
                        flujo
                    }
                    Err(no) => {
                        let _ = listo.send(Err(no));
                        return;
                    }
                };
                // Espera la orden de parar; si el grifo se suelta sin darla, el canal se cierra y también vuelve.
                let _ = orden.recv();
                drop(flujo);
            })
            .map_err(|e| NoAbrio::por(PorQueNoAbrio::NoDejo, format!("no se pudo crear el hilo del micrófono: {e}")))?;
        match abierto.recv() {
            Ok(Ok(hz)) => Ok(Self { parar: Some(parar), hilo: Some(hilo), entradas, hz_del_dispositivo: hz }),
            Ok(Err(no)) => {
                let _ = hilo.join();
                Err(no)
            }
            Err(_) => {
                let _ = hilo.join();
                Err(NoAbrio::por(PorQueNoAbrio::NoDejo, "el hilo del micrófono terminó sin contestar"))
            }
        }
    }

    /// El audio del sistema (la pista del cliente en una videollamada) **todavía no** se abre en Windows: llega
    /// con la videollamada en el sprint 007 (ADR 022). Se dice, no se finge.
    pub fn del_sistema(_anillo: Arc<Mutex<Anillo>>) -> Result<Self, NoAbrio> {
        Err(NoAbrio::por(
            PorQueNoAbrio::NoDejo,
            "el audio del sistema en Windows llega con la videollamada, en el sprint 007 (ADR 022)",
        ))
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

/// Lo que el callback necesita tener a mano. Vive dentro del cierre del flujo y muere con él.
struct Destino {
    anillo: Arc<Mutex<Anillo>>,
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
        self.entradas.fetch_add(a_dieciseis.len() as u64, Ordering::Relaxed);
        // El mismo candado corto que en macOS (`nativo.rs`): copiar unos cientos de flotantes.
        if let Ok(mut a) = self.anillo.lock() {
            a.escribir(&a_dieciseis);
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

/// Abre el micrófono por defecto y arranca su flujo. Devuelve el flujo y los hercios del dispositivo.
fn abrir(anillo: Arc<Mutex<Anillo>>, entradas: Arc<AtomicU64>) -> Result<(cpal::Stream, u32), NoAbrio> {
    let host = cpal::default_host();
    let dispositivo = host
        .default_input_device()
        .ok_or_else(|| NoAbrio::por(PorQueNoAbrio::NoDejo, "este PC no tiene un micrófono por defecto"))?;
    let formato = dispositivo.default_input_config().map_err(no_abrio)?;
    let hz = formato.sample_rate();
    let destino = Destino {
        anillo,
        remuestreador: Remuestreador::nuevo(hz, super::anillo::HZ),
        entradas,
        canales: formato.channels(),
    };
    let config = formato.config();
    let flujo = match formato.sample_format() {
        SampleFormat::F32 => construir::<f32>(&dispositivo, config, destino),
        SampleFormat::F64 => construir::<f64>(&dispositivo, config, destino),
        SampleFormat::I16 => construir::<i16>(&dispositivo, config, destino),
        SampleFormat::I32 => construir::<i32>(&dispositivo, config, destino),
        SampleFormat::I8 => construir::<i8>(&dispositivo, config, destino),
        SampleFormat::U16 => construir::<u16>(&dispositivo, config, destino),
        SampleFormat::U8 => construir::<u8>(&dispositivo, config, destino),
        otro => {
            return Err(NoAbrio::por(
                PorQueNoAbrio::FormatoIlegible,
                format!("el micrófono entrega muestras {otro}, que el grifo no sabe leer"),
            ))
        }
    }
    .map_err(no_abrio)?;
    flujo.play().map_err(no_abrio)?;
    Ok((flujo, hz))
}

fn construir<T>(dispositivo: &cpal::Device, config: cpal::StreamConfig, mut destino: Destino) -> Result<cpal::Stream, cpal::Error>
where
    T: cpal::SizedSample,
    f32: FromSample<T>,
{
    dispositivo.build_input_stream::<T, _, _>(
        config,
        move |datos: &[T], _: &cpal::InputCallbackInfo| destino.recibir(datos),
        // Solo el tipo del aviso: es un metadato, jamás audio (ADR 003).
        |e: cpal::Error| println!("[captura] el micrófono avisó: {}", e.kind()),
        None,
    )
}

/// El «no» de WASAPI, en el conjunto cerrado que enseña la pantalla. El detalle va al log.
fn no_abrio(e: cpal::Error) -> NoAbrio {
    let porque = match e.kind() {
        cpal::ErrorKind::PermissionDenied => PorQueNoAbrio::SinPermisoDelMicrofono,
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

    /// Lo que WASAPI dice que no se traduce al porqué que la pantalla sabe enseñar.
    #[test]
    fn el_no_de_wasapi_se_dice_con_su_porque() {
        let no = no_abrio(cpal::Error::new(cpal::ErrorKind::PermissionDenied));
        assert_eq!(no.porque, PorQueNoAbrio::SinPermisoDelMicrofono);
        let no = no_abrio(cpal::Error::new(cpal::ErrorKind::DeviceBusy));
        assert_eq!(no.porque, PorQueNoAbrio::DispositivoOcupado);
        let no = no_abrio(cpal::Error::new(cpal::ErrorKind::BackendError));
        assert_eq!(no.porque, PorQueNoAbrio::NoDejo);
    }

    /// El audio del sistema todavía no se abre en Windows, y lo dice sin tocar el hardware.
    #[test]
    fn el_audio_del_sistema_dice_que_llega_en_el_s7() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let Err(no) = Grifo::del_sistema(anillo) else { panic!("el audio del sistema abrió en Windows") };
        assert!(no.detalle.contains("sprint 007"), "{}", no.detalle);
        assert_eq!(salida_de_audio().puede_haber_eco(), None);
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
}
