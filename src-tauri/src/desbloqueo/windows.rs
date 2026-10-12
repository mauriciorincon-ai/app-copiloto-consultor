//! **El desbloqueo en Windows: Windows Hello** (sprint 008, ADR 022). El mismo papel que Touch ID en el Mac: abrir tus
//! notas guardadas, tus ensayos o la bandeja de otra sesión pide tu PIN, tu huella o tu cara **una vez por sesión de
//! la app**, y la puerta local lo pide una vez por apertura.
//!
//! Dos pasos de `Windows.Security.Credentials.UI.UserConsentVerifier`:
//! 1. `CheckAvailabilityAsync`: si este PC no tiene Windows Hello configurado, no hay diálogo y se dice por qué;
//! 2. `RequestVerificationForWindowAsync` (`IUserConsentVerifierInterop`), con la ventana principal de la app como
//!    dueña: una app de escritorio no tiene la ventana de WinRT que pide `RequestVerificationAsync`, y sin dueño el
//!    diálogo puede quedar detrás.
//!
//! Se pregunta en un hilo propio que vive lo que la app, con COM multihilo iniciado una vez: quien pide puede ser el
//! hilo de la ventana, que no debe quedarse esperando a que contestes, y un hilo de vida corta que abre y cierra COM
//! tumbó el proceso de pruebas en el S7 (auditoría, A3).

use super::{de_windows_hello, Respuesta};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;
use windows::core::HSTRING;
use windows::Security::Credentials::UI::{
    UserConsentVerificationResult, UserConsentVerifier, UserConsentVerifierAvailability,
};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::WinRT::IUserConsentVerifierInterop;

/// La ventana principal de la app, como número: la deja aquí el arranque (`recordar_la_ventana`).
static VENTANA: AtomicUsize = AtomicUsize::new(0);

/// Lo que tarda como mucho una respuesta: lo que dura pensarlo, con holgura. Pasado eso, cuenta como «no».
const ESPERA: Duration = Duration::from_secs(180);

/// El arranque deja aquí la ventana principal, para que el diálogo de Windows Hello salga delante de ella.
pub fn recordar_la_ventana(hwnd: usize) {
    VENTANA.store(hwnd, Ordering::Relaxed);
}

struct Peticion {
    razon: String,
    responde: mpsc::Sender<Respuesta>,
}

fn hilo() -> &'static Mutex<mpsc::Sender<Peticion>> {
    static UNO: OnceLock<Mutex<mpsc::Sender<Peticion>>> = OnceLock::new();
    UNO.get_or_init(|| {
        let (manda, recibe) = mpsc::channel::<Peticion>();
        let _ = std::thread::Builder::new().name("ag-hello".into()).spawn(move || {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
            // SAFETY: COM se inicia una vez en este hilo, que vive lo que la app.
            let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
            for Peticion { razon, responde } in recibe {
                let _ = responde.send(preguntar(&razon));
            }
        });
        Mutex::new(manda)
    })
}

/// Pide Windows Hello y espera a que contestes.
pub fn pedir(razon: &str) -> Respuesta {
    let (responde, respuesta) = mpsc::channel();
    let enviado = hilo().lock().ok().and_then(|h| h.send(Peticion { razon: razon.to_string(), responde }).ok());
    if enviado.is_none() {
        return Respuesta::NoSePuede;
    }
    respuesta.recv_timeout(ESPERA).unwrap_or(Respuesta::NoQuiso)
}

fn preguntar(razon: &str) -> Respuesta {
    let hay = UserConsentVerifier::CheckAvailabilityAsync().and_then(|a| a.join());
    let Ok(hay) = hay else { return Respuesta::NoSePuede };
    if hay != UserConsentVerifierAvailability::Available {
        return Respuesta::NoSePuede;
    }
    let ventana = HWND(VENTANA.load(Ordering::Relaxed) as *mut core::ffi::c_void);
    let resultado = (|| -> windows::core::Result<UserConsentVerificationResult> {
        let interop = windows::core::factory::<UserConsentVerifier, IUserConsentVerifierInterop>()?;
        // SAFETY: la ventana es la principal de la app (o nula, si el arranque aún no la dejó: Windows la toma por
        // «sin dueño»), y el texto vive hasta que la llamada vuelve.
        let pregunta: windows_future::IAsyncOperation<UserConsentVerificationResult> =
            unsafe { interop.RequestVerificationForWindowAsync(ventana, &HSTRING::from(razon)) }?;
        pregunta.join()
    })();
    resultado.map(|r| de_windows_hello(r.0)).unwrap_or(Respuesta::NoSePuede)
}
