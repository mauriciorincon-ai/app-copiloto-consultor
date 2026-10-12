//! **El acople en Windows** (sprint 008, ADR 004 enmienda 2, ADR 022): las mismas lecturas y escrituras que `ax.rs`
//! en el Mac, con las mismas firmas, para que la maniobra entera (`nativo` en `mod.rs`: el turno, soltar primero,
//! leer lo que de verdad quedó, deshacer, la huella) sea **la misma** en los dos sistemas.
//!
//! Lo que cambia es la API y lo que no hace falta:
//! - **Sin permiso que pedir**: Windows deja mover y redimensionar las ventanas de otro proceso del mismo usuario con
//!   `SetWindowPos`. `permiso_concedido` es siempre cierto, y la app lo dice («no pide permiso en Windows»).
//! - **Lo que se ve, no el rectángulo**: se mide con `DWMWA_EXTENDED_FRAME_BOUNDS` (sin los bordes invisibles de
//!   redimensionar) y se escribe el rectángulo entero con esos bordes por fuera (`super::a_escribir`).
//! - **Puntos, como la franja**: los píxeles físicos se dividen por la escala del monitor de cada ventana
//!   (`GetDpiForMonitor`), que es como Tauri da la franja de la banda.
//! - **El índice de una ventana** es su posición en la lista de ventanas visibles de su proceso **ordenada por su
//!   identificador**, que no cambia mientras la ventana vive (el orden de delante a atrás cambia con cada clic).
//!
//! Como en el Mac, no se lee ningún título ni nada de dentro de las ventanas: solo su proceso y su geometría.

use super::{a_escribir, a_puntos, bordes, Destino, Marco, Rect};
use windows::core::BOOL;
use windows::Win32::Foundation::{HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetWindowRect, GetWindowThreadProcessId, IsIconic, IsWindowVisible,
    SetWindowPos, SWP_NOACTIVATE, SWP_NOOWNERZORDER, SWP_NOZORDER,
};

/// Windows no pide permiso para acoplar.
pub fn permiso_concedido() -> bool {
    true
}

/// No hay diálogo que abrir: ya está «concedido».
pub fn pedir_permiso() -> bool {
    true
}

/// El ejecutable de un proceso, de la foto de procesos (`sesion::windows::procesos`).
pub fn nombre_de(pid: i32) -> Option<String> {
    crate::sesion::windows::procesos().into_iter().find(|p| p.pid as i32 == pid).map(|p| p.exe)
}

/// ¿El proceso de la huella sigue siendo el mismo programa? El cerrojo contra el reciclado de números de proceso.
pub fn sigue_siendo(pid: i32, nombre: &str) -> bool {
    nombre_de(pid).is_some_and(|n| n.eq_ignore_ascii_case(nombre))
}

/// El proceso de la ventana que está al frente y su ejecutable, **saltándonos a nosotros mismos**.
pub fn aplicacion_al_frente() -> Option<(i32, String)> {
    // SAFETY: solo lee qué ventana está al frente y de qué proceso es.
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.is_invalid() {
        return None;
    }
    let pid = pid_de(hwnd);
    if pid == 0 || pid == std::process::id() {
        return None;
    }
    Some((pid as i32, nombre_de(pid as i32)?))
}

/// Las ventanas de un proceso que se ven (visibles, sin minimizar y sin ocultar por DWM), medidas en puntos, con su
/// índice estable. Leer las ventanas de otro programa lleva el centinela de la regla 25, como `Aplicacion::de` en el
/// Mac.
pub fn ventanas_de(pid: i32) -> Vec<(usize, Marco)> {
    crate::hardware::vigilar("las ventanas de otro programa en Windows");
    las_de(pid).into_iter().enumerate().filter_map(|(i, h)| Some((i, marco_de(h)?))).collect()
}

/// Lee una ventana concreta, sin escribir nada.
pub fn marco_de_la_ventana(pid: i32, indice: usize) -> Option<Marco> {
    marco_de(*las_de(pid).get(indice)?)
}

/// Pone el alto de lo que se ve de una ventana (el borde de arriba y el ancho no se mueven) y devuelve cómo quedó,
/// **leído del sistema**.
pub fn encoger(pid: i32, indice: usize, alto: f64) -> Option<Marco> {
    let h = *las_de(pid).get(indice)?;
    let m = marco_de(h)?;
    poner(h, Marco { alto, ..m })
}

/// Lleva la esquina de lo que se ve de una ventana a `(x, y)` sin cambiar su tamaño y devuelve cómo quedó.
pub fn mover(pid: i32, indice: usize, x: f64, y: f64) -> Option<Marco> {
    let h = *las_de(pid).get(indice)?;
    let m = marco_de(h)?;
    poner(h, Marco { x, y, ..m })
}

/// El fondo de escritorio para el relleno: en Windows no se lee (el relleno queda en negro, que no filtra nada).
pub fn fondo_de_escritorio() -> Option<std::path::PathBuf> {
    None
}

/// **La ventana de la reunión como destino del acople arriba**: su proceso, su ejecutable y su índice. La elige
/// `sesion::windows::ventana_del_objetivo`, que es donde se leen títulos; aquí solo se traduce a lo que la maniobra
/// entiende.
pub fn destino_de(hwnd: HWND) -> Option<Destino> {
    let pid = pid_de(hwnd) as i32;
    let indice = las_de(pid).iter().position(|h| *h == hwnd)?;
    Some(Destino { pid, app: nombre_de(pid)?, indice })
}

/// **La única escritura** sobre la ventana de otro programa: lo que se ve de ella, en `objetivo`. Sin cambiar su
/// orden ni activarla. Devuelve cómo quedó, leído después.
fn poner(h: HWND, objetivo: Marco) -> Option<Marco> {
    crate::hardware::vigilar("mover o encoger una ventana de otro programa en Windows");
    let (ventana, visible) = (rect_de(h)?, visible_de(h)?);
    let (x, y, ancho, alto) = a_escribir(objetivo, escala_de(h), bordes(ventana, visible));
    // SAFETY: una ventana que acaba de salir de `EnumWindows`; si ya no existe, Windows devuelve un error.
    unsafe { SetWindowPos(h, None, x, y, ancho, alto, SWP_NOZORDER | SWP_NOACTIVATE | SWP_NOOWNERZORDER) }.ok()?;
    marco_de(h)
}

fn pid_de(h: HWND) -> u32 {
    let mut pid = 0u32;
    // SAFETY: solo escribe el número de proceso en `pid`.
    unsafe { GetWindowThreadProcessId(h, Some(&mut pid)) };
    pid
}

/// Las ventanas de primer nivel de un proceso que se ven, ordenadas por su identificador.
fn las_de(pid: i32) -> Vec<HWND> {
    unsafe extern "system" fn una(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: `lparam` apunta a la lista que vive en `las_de` mientras dura `EnumWindows`, que es síncrona.
        unsafe {
            let lista = &mut *(lparam.0 as *mut Vec<HWND>);
            if IsWindowVisible(hwnd).as_bool() && !IsIconic(hwnd).as_bool() {
                lista.push(hwnd);
            }
        }
        BOOL(1)
    }
    let mut todas: Vec<HWND> = Vec::new();
    // SAFETY: ver la retrollamada.
    let _ = unsafe { EnumWindows(Some(una), LPARAM(&mut todas as *mut Vec<HWND> as isize)) };
    let mut suyas: Vec<HWND> =
        todas.into_iter().filter(|h| pid_de(*h) as i32 == pid && !oculta(*h) && marco_de(*h).is_some()).collect();
    suyas.sort_by_key(|h| h.0 as usize);
    suyas
}

/// Una ventana «visible» que DWM tiene escondida (las de las apps de la tienda en otro escritorio, por ejemplo).
fn oculta(h: HWND) -> bool {
    let mut oculta = 0u32;
    // SAFETY: DWM escribe un `u32` en `oculta`.
    unsafe {
        DwmGetWindowAttribute(h, DWMWA_CLOAKED, &mut oculta as *mut u32 as *mut _, std::mem::size_of::<u32>() as u32)
    }
    .is_ok()
        && oculta != 0
}

fn rect_de(h: HWND) -> Option<Rect> {
    let mut r = RECT::default();
    // SAFETY: Windows escribe el rectángulo de la ventana en `r`.
    unsafe { GetWindowRect(h, &mut r) }.ok()?;
    Some(Rect { izquierda: r.left, arriba: r.top, derecha: r.right, abajo: r.bottom })
}

/// Lo que se ve de una ventana, sin sus bordes invisibles. Si DWM no contesta, el rectángulo entero.
fn visible_de(h: HWND) -> Option<Rect> {
    let mut r = RECT::default();
    // SAFETY: DWM escribe un `RECT` en `r`.
    let dwm = unsafe {
        DwmGetWindowAttribute(h, DWMWA_EXTENDED_FRAME_BOUNDS, &mut r as *mut RECT as *mut _, std::mem::size_of::<RECT>() as u32)
    };
    match dwm {
        Ok(()) => Some(Rect { izquierda: r.left, arriba: r.top, derecha: r.right, abajo: r.bottom }),
        Err(_) => rect_de(h),
    }
}

/// La escala del monitor donde está la ventana: 1,0 al 100 %, 1,5 al 150 %.
fn escala_de(h: HWND) -> f64 {
    let (mut x, mut y) = (96u32, 96u32);
    // SAFETY: el monitor más cercano a la ventana existe siempre; Windows escribe sus puntos por pulgada.
    let ok = unsafe { GetDpiForMonitor(MonitorFromWindow(h, MONITOR_DEFAULTTONEAREST), MDT_EFFECTIVE_DPI, &mut x, &mut y) };
    if ok.is_ok() && x > 0 {
        f64::from(x) / 96.0
    } else {
        1.0
    }
}

/// Lo que se ve de una ventana, en puntos; `None` si no tiene tamaño.
fn marco_de(h: HWND) -> Option<Marco> {
    let v = visible_de(h)?;
    (v.derecha > v.izquierda && v.abajo > v.arriba).then(|| a_puntos(v, escala_de(h)))
}
