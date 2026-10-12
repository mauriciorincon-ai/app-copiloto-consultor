//! **La tarea de vencimiento en el Programador de tareas de Windows** (sprint 008, ADR 016 enmienda 4). La app no
//! lanza ningún programa para esto: habla con el Programador por COM (`ITaskService`), registra el XML que generó
//! `super::xml_de_windows` en la carpeta raíz con el nombre de la tarea y, cuando ya no vence nada, la borra. Lo que
//! corre después, al minuto de cada vencimiento, es `powershell.exe` lanzado por Windows, no por la app.
//!
//! Se habla con el Programador desde un hilo propio que vive lo que la app, con COM multihilo iniciado una vez: quien
//! pide puede ser el hilo de la ventana, con COM en otro modo, y un hilo de vida corta que abre y cierra COM tumbó el
//! proceso de pruebas en el S7 (auditoría, A3).

use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;
use windows::core::BSTR;
use windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED};
use windows::Win32::System::TaskScheduler::{
    ITaskFolder, ITaskService, TaskScheduler, TASK_CREATE_OR_UPDATE, TASK_LOGON_INTERACTIVE_TOKEN,
};
use windows::Win32::System::Variant::VARIANT;

/// Lo que se le pide al hilo: registrar una tarea con su XML, o quitarla.
enum Orden {
    Registrar { nombre: String, xml: String, responde: mpsc::Sender<Result<(), String>> },
    Quitar { nombre: String, responde: mpsc::Sender<Result<(), String>> },
}

fn hilo() -> &'static Mutex<mpsc::Sender<Orden>> {
    static UNO: OnceLock<Mutex<mpsc::Sender<Orden>>> = OnceLock::new();
    UNO.get_or_init(|| {
        let (manda, recibe) = mpsc::channel::<Orden>();
        let _ = std::thread::Builder::new().name("ag-tareas".into()).spawn(move || {
            // SAFETY: COM se inicia una vez en este hilo, que vive lo que la app.
            let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
            for orden in recibe {
                match orden {
                    Orden::Registrar { nombre, xml, responde } => {
                        let _ = responde.send(raiz().and_then(|r| registrar_en(&r, &nombre, &xml)));
                    }
                    Orden::Quitar { nombre, responde } => {
                        let _ = responde.send(raiz().and_then(|r| quitar_de(&r, &nombre)));
                    }
                }
            }
        });
        Mutex::new(manda)
    })
}

/// Le pasa una orden al hilo del Programador y espera su respuesta. Es **la única entrada** de este archivo al
/// Programador de tareas, y por eso lleva el centinela de la regla 25.
fn pedir(orden: impl FnOnce(mpsc::Sender<Result<(), String>>) -> Orden) -> Result<(), String> {
    crate::hardware::vigilar("el Programador de tareas de Windows");
    let (responde, respuesta) = mpsc::channel();
    hilo()
        .lock()
        .map_err(|_| "el hilo del Programador de tareas no contesta".to_string())?
        .send(orden(responde))
        .map_err(|_| "el hilo del Programador de tareas no contesta".to_string())?;
    respuesta
        .recv_timeout(Duration::from_secs(20))
        .map_err(|_| "el Programador de tareas no contestó a tiempo".to_string())?
}

/// Registra (o sustituye) la tarea con este nombre y este XML.
pub fn registrar(nombre: &str, xml: &str) -> Result<(), String> {
    pedir(|responde| Orden::Registrar { nombre: nombre.into(), xml: xml.into(), responde })
}

/// Quita la tarea con este nombre, si existe.
pub fn quitar(nombre: &str) {
    if let Err(e) = pedir(|responde| Orden::Quitar { nombre: nombre.into(), responde }) {
        println!("[vencimiento] {e}");
    }
}

/// La carpeta raíz del Programador de tareas del usuario.
fn raiz() -> Result<ITaskFolder, String> {
    let fallo = |e: windows::core::Error| format!("el Programador de tareas no se abrió ({:#x})", e.code().0);
    // SAFETY: interfaces COM de este hilo, que ya inició COM; las cadenas y las variantes viven durante la llamada.
    unsafe {
        let servicio: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER).map_err(fallo)?;
        let nada = VARIANT::default();
        servicio.Connect(&nada, &nada, &nada, &nada).map_err(fallo)?;
        servicio.GetFolder(&BSTR::from("\\")).map_err(fallo)
    }
}

fn registrar_en(raiz: &ITaskFolder, nombre: &str, xml: &str) -> Result<(), String> {
    let nada = VARIANT::default();
    // SAFETY: ver `raiz`. La tarea corre como el usuario que la registra y solo con su sesión abierta.
    unsafe {
        raiz.RegisterTask(
            &BSTR::from(nombre),
            &BSTR::from(xml),
            TASK_CREATE_OR_UPDATE.0,
            &nada,
            &nada,
            TASK_LOGON_INTERACTIVE_TOKEN,
            &nada,
        )
    }
    .map(|_| ())
    .map_err(|e| format!("el Programador de tareas no aceptó la tarea de vencimiento ({:#x})", e.code().0))
}

fn quitar_de(raiz: &ITaskFolder, nombre: &str) -> Result<(), String> {
    // SAFETY: ver `raiz`. Que la tarea no exista no es un fallo: ya no está.
    match unsafe { raiz.DeleteTask(&BSTR::from(nombre), 0) } {
        Ok(()) => Ok(()),
        Err(e) if e.code() == windows::core::HRESULT::from_win32(2) => Ok(()),
        Err(e) => Err(format!("el Programador de tareas no quitó la tarea de vencimiento ({:#x})", e.code().0)),
    }
}
