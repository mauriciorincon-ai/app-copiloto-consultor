//! **La reunión en Windows** (sprint 007, ADR 022): qué videollamada corre en este PC. Sigue siendo `sesion/`:
//! módulo protegido. Los títulos de las ventanas son información del cliente y solo viven en memoria.
//!
//! Dos lecturas del propio PC, ninguna con permiso que pedir (por eso `titulos_legibles` es siempre cierto aquí y
//! «no se puede saber» no aplica en Windows):
//! - los procesos (`CreateToolhelp32Snapshot`): el nombre del ejecutable de cada uno, contra el catálogo;
//! - los títulos de las ventanas visibles **solo de los navegadores del catálogo** (`EnumWindows`), con la misma
//!   restricción que en macOS: [`super::a_quien_preguntar_titulos`]. El resto de ventanas del PC no se lee.
//!
//! Regla dura 9: todo es del propio equipo; no se toca ninguna otra máquina.

use super::{a_quien_preguntar_titulos, del_catalogo, Candidata, Programa, Vista};
use windows::core::BOOL;
use windows::Win32::Foundation::{CloseHandle, HWND, LPARAM, RECT};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
};

/// Lo que el sistema deja ver ahora mismo: los programas del catálogo que corren y, de los navegadores, los títulos
/// de sus ventanas.
pub fn mirar() -> Vista {
    let apps: Vec<(i32, String)> = procesos()
        .into_iter()
        .filter(|(_, exe)| del_catalogo(exe).is_some())
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

/// **La ventana que lee la pantalla** (sprint 007): la de la videollamada que eligió `objetivo_de`, con el mismo
/// criterio que la captura del Mac (`la_de_la_reunion`). Solo se leen los títulos de las ventanas de ese programa.
pub fn ventana_del_objetivo(ejecutable: &str, senales: &[String]) -> Option<HWND> {
    let pids: Vec<u32> =
        procesos().into_iter().filter(|(_, e)| e.eq_ignore_ascii_case(ejecutable)).map(|(p, _)| p).collect();
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

/// Los procesos del PC: identificador y nombre del ejecutable.
fn procesos() -> Vec<(u32, String)> {
    // SAFETY: la foto de procesos se recorre con una estructura del tamaño que Windows exige y se cierra al acabar.
    unsafe {
        let Ok(foto) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else { return Vec::new() };
        let mut e = PROCESSENTRY32W { dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32, ..Default::default() };
        let mut lista = Vec::new();
        let mut hay = Process32FirstW(foto, &mut e).is_ok();
        while hay {
            let fin = e.szExeFile.iter().position(|&c| c == 0).unwrap_or(e.szExeFile.len());
            lista.push((e.th32ProcessID, String::from_utf16_lossy(&e.szExeFile[..fin])));
            hay = Process32NextW(foto, &mut e).is_ok();
        }
        let _ = CloseHandle(foto);
        lista
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
