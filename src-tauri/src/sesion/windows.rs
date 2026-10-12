//! **La reunión en Windows** (sprint 007, ADR 022): qué videollamada corre en este PC. Sigue siendo `sesion/`:
//! módulo protegido. Los títulos de las ventanas son información del cliente y solo viven en memoria.
//!
//! Tres lecturas del propio PC, ninguna con permiso que pedir (por eso `titulos_legibles` es siempre cierto aquí y
//! «no se puede saber» no aplica en Windows):
//! - los procesos (`CreateToolhelp32Snapshot`): el nombre del ejecutable de cada uno, contra el catálogo;
//! - **las sesiones de captura de audio** (sprint 008, `IAudioSessionManager2`): qué procesos tienen un micrófono
//!   abierto ahora. Teams y Zoom corren residentes; solo cuentan si están en una llamada ([`super::en_llamada`]).
//!   Se lee el número del proceso y su estado, nada del sonido;
//! - los títulos de las ventanas visibles **solo de los navegadores del catálogo** (`EnumWindows`), con la misma
//!   restricción que en macOS: [`super::a_quien_preguntar_titulos`]. El resto de ventanas del PC no se lee.
//!
//! Regla dura 9: todo es del propio equipo; no se toca ninguna otra máquina.

use super::{a_quien_preguntar_titulos, del_catalogo, solo_en_llamada, Candidata, Proceso, Programa, Vista};
use std::sync::{mpsc, Mutex, OnceLock};
use std::time::Duration;
use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
};

/// Lo que el sistema deja ver ahora mismo: los programas del catálogo que corren —de Teams y Zoom, solo si están en
/// una llamada— y, de los navegadores, los títulos de sus ventanas.
pub fn mirar() -> Vista {
    crate::hardware::vigilar("los procesos, las sesiones de captura y las ventanas de Windows");
    let foto = procesos();
    let apps: Vec<(u32, String)> =
        foto.iter().filter(|p| del_catalogo(&p.exe).is_some()).map(|p| (p.pid, p.exe.clone())).collect();
    let captura = if apps.is_empty() { Some(Vec::new()) } else { con_captura() };
    let apps: Vec<(i32, String)> = solo_en_llamada(apps, &foto, captura.as_deref(), super::catalogo())
        .into_iter()
        .map(|(pid, exe)| (pid as i32, exe))
        .collect();
    let preguntables = a_quien_preguntar_titulos(&apps, true);
    let ventanas = if preguntables.is_empty() { Vec::new() } else { ventanas() };
    let programas = apps
        .into_iter()
        .map(|(pid, bundle)| Programa {
            titulos: if preguntables.contains(&pid) {
                ventanas.iter().filter(|v| v.pid as i32 == pid).filter_map(|v| titulo(v.hwnd)).collect()
            } else {
                Vec::new()
            },
            bundle,
        })
        .collect();
    Vista { programas, titulos_legibles: true }
}

/// **La ventana que lee la pantalla** (sprint 007) y a la que se acopla la banda (sprint 008): la de la
/// videollamada que eligió `objetivo_de`, con el mismo criterio que la captura del Mac (`la_de_la_reunion`). Solo
/// se leen los títulos de las ventanas de ese programa.
pub fn ventana_del_objetivo(ejecutable: &str, senales: &[String]) -> Option<HWND> {
    crate::hardware::vigilar("las ventanas de la videollamada en Windows");
    let pids: Vec<u32> =
        procesos().into_iter().filter(|p| p.exe.eq_ignore_ascii_case(ejecutable)).map(|p| p.pid).collect();
    if pids.is_empty() {
        return None;
    }
    let candidatas: Vec<(HWND, Candidata)> = ventanas()
        .into_iter()
        .filter(|v| pids.contains(&v.pid))
        // SAFETY: `IsIconic` y `GetWindowRect` solo leen el estado de una ventana que `EnumWindows` acaba de dar.
        .filter(|v| !unsafe { IsIconic(v.hwnd) }.as_bool())
        .filter_map(|v| {
            let mut r = RECT::default();
            unsafe { GetWindowRect(v.hwnd, &mut r) }.ok()?;
            let titulo = titulo(v.hwnd).unwrap_or_default();
            Some((v.hwnd, Candidata { ancho: r.right - r.left, alto: r.bottom - r.top, titulo }))
        })
        .collect();
    let solo: Vec<Candidata> = candidatas.iter().map(|(_, c)| c.clone()).collect();
    let i = super::la_de_la_reunion(&solo, senales)?;
    Some(candidatas[i].0)
}

/// Una ventana visible y el proceso al que pertenece.
struct Ventana {
    hwnd: HWND,
    pid: u32,
}

/// **Los procesos del PC**: el suyo, el de su padre y el nombre del ejecutable. La comparten la reunión y el radar
/// (`radar/procesos.rs`), que cotejan cada uno contra su catálogo. No pide permiso ni lee nada de dentro de ellos.
pub(crate) fn procesos() -> Vec<Proceso> {
    // SAFETY: la foto de procesos se recorre con una estructura del tamaño que Windows exige y se cierra al acabar.
    unsafe {
        let Ok(foto) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return Vec::new() };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut lista = Vec::new();
        let mut hay = Process32FirstW(foto, &mut e).is_ok();
        while hay {
            let fin = e.szExeFile.iter().position(|&c| c == 0).unwrap_or(e.szExeFile.len());
            lista.push(Proceso {
                pid: e.th32ProcessID,
                padre: e.th32ParentProcessID,
                exe: String::from_utf16_lossy(&e.szExeFile[..fin]),
            });
            hay = Process32NextW(foto, &mut e).is_ok();
        }
        let _ = CloseHandle(foto);
        lista
    }
}

/// **Qué procesos tienen ahora una sesión de captura activa**, en todos los micrófonos del PC. `None` si Windows no
/// contestó: quien llama lo trata como «no se sabe» y cuenta la reunión.
///
/// Se pregunta en un hilo propio que vive lo que la app, con COM multihilo iniciado una vez: quien pregunta puede
/// ser el hilo de la ventana, con COM en otro modo, y un hilo de vida corta que abre y cierra COM tumbó el proceso
/// de pruebas en el S7 (auditoría, A3).
fn con_captura() -> Option<Vec<u32>> {
    static HILO: OnceLock<Mutex<mpsc::Sender<mpsc::Sender<Option<Vec<u32>>>>>> = OnceLock::new();
    let hilo = HILO.get_or_init(|| {
        let (manda, recibe) = mpsc::channel::<mpsc::Sender<Option<Vec<u32>>>>();
        let _ = std::thread::Builder::new().name("ag-sesiones".into()).spawn(move || {
            use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
            // SAFETY: COM se inicia una vez en este hilo, que vive lo que la app.
            let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
            for responde in recibe {
                let _ = responde.send(leer_sesiones());
            }
        });
        Mutex::new(manda)
    });
    let (responde, respuesta) = mpsc::channel();
    hilo.lock().ok()?.send(responde).ok()?;
    respuesta.recv_timeout(Duration::from_secs(2)).ok().flatten()
}

fn leer_sesiones() -> Option<Vec<u32>> {
    use windows::core::Interface;
    use windows::Win32::Media::Audio::{
        eCapture, AudioSessionStateActive, IAudioSessionControl2, IAudioSessionManager2, IMMDeviceEnumerator,
        MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
    };
    use windows::Win32::System::Com::{CoCreateInstance, CLSCTX_ALL};
    // SAFETY: interfaces COM de este hilo, que ya inició COM; cada una se suelta al salir de su ámbito.
    unsafe {
        let enumerador: IMMDeviceEnumerator = CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let microfonos = enumerador.EnumAudioEndpoints(eCapture, DEVICE_STATE_ACTIVE).ok()?;
        let mut pids = Vec::new();
        for i in 0..microfonos.GetCount().ok()? {
            let Ok(microfono) = microfonos.Item(i) else { continue };
            let Ok(gestor) = microfono.Activate::<IAudioSessionManager2>(CLSCTX_ALL, None) else { continue };
            let Ok(sesiones) = gestor.GetSessionEnumerator() else { continue };
            for j in 0..sesiones.GetCount().unwrap_or(0) {
                let Ok(sesion) = sesiones.GetSession(j) else { continue };
                if sesion.GetState().ok() != Some(AudioSessionStateActive) {
                    continue;
                }
                if let Ok(pid) = sesion.cast::<IAudioSessionControl2>().and_then(|s| s.GetProcessId()) {
                    pids.push(pid);
                }
            }
        }
        Some(pids)
    }
}

/// Las ventanas de primer nivel visibles, con su proceso. No lee ningún título: eso lo decide quien llama.
fn ventanas() -> Vec<Ventana> {
    unsafe extern "system" fn una(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: `lparam` es el puntero a la lista que vive en `ventanas` mientras dura `EnumWindows`, que es
        // síncrona; nadie más la toca.
        unsafe {
            let lista = &mut *(lparam.0 as *mut Vec<Ventana>);
            if IsWindowVisible(hwnd).as_bool() {
                let mut pid = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                lista.push(Ventana { hwnd, pid });
            }
        }
        BOOL(1)
    }
    let mut lista: Vec<Ventana> = Vec::new();
    // SAFETY: ver la retrollamada.
    let _ = unsafe { EnumWindows(Some(una), LPARAM(&mut lista as *mut Vec<Ventana> as isize)) };
    lista
}

/// El título de una ventana, si tiene. **Información del cliente: solo memoria.**
fn titulo(hwnd: HWND) -> Option<String> {
    let mut b = [0u16; 512];
    // SAFETY: Windows escribe como mucho `b.len()` caracteres en el búfer.
    let n = unsafe { GetWindowTextW(hwnd, &mut b) };
    (n > 0).then(|| String::from_utf16_lossy(&b[..n as usize]))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// La foto de procesos de este PC se lee y trae al propio test con su padre. No pide permisos: corre en la CI de
    /// Windows a secas, sin el centinela.
    #[test]
    fn la_foto_de_procesos_de_este_pc_se_lee() {
        let foto = procesos();
        assert!(foto.len() > 10, "solo {} procesos: la foto no se está leyendo", foto.len());
        let yo = std::process::id();
        let mio = foto.iter().find(|p| p.pid == yo).expect("el propio test no aparece en la foto");
        assert!(mio.exe.to_lowercase().ends_with(".exe"), "ejecutable raro: {}", mio.exe);
        assert!(foto.iter().any(|p| p.pid == mio.padre), "el padre del test no está en la foto");
    }
}
