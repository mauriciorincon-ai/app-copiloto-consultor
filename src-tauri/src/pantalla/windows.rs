//! **La pantalla en Windows** (sprint 007, ADR 022): la ventana de la reunión y su texto. Sigue siendo `pantalla/`:
//! módulo protegido, ni disco ni red. Lo que se ve es del cliente y solo vive en memoria.
//!
//! Las mismas dos piezas que en el Mac, detrás de los mismos rasgos:
//! - **los ojos** ([`Ojo`]): la ventana de la videollamada que eligió Sesión (`sesion::windows::ventana_del_objetivo`,
//!   el mismo criterio que `Pantalla.swift`), capturada con `PrintWindow` y `PW_RENDERFULLCONTENT`, que pide a
//!   Windows el contenido compuesto aunque la ventana esté tapada. La banda de la app no sale: es otra ventana;
//! - **el lector** ([`Lector`]): `Windows.Media.Ocr`, con un motor por idioma (el del perfil del usuario y, si no,
//!   español o inglés). Si Windows no tiene el reconocimiento de texto de ningún idioma, lo dice.
//!
//! **Cuándo se lee no se decide aquí.** Lo decide el `Vigia` con la huella por zonas, en `mod.rs`, igual que en el
//! Mac: estos ojos capturan y este lector lee cuando el `Vigia` dice «leer». Su test lo comprueba leyendo este
//! archivo: el reconocimiento solo vive dentro del lector.
//!
//! Dos diferencias con Vision, dichas: Windows no da una confianza por línea (se toma 1,0, así que el filtro de
//! confianza de `refuerzo.rs` no actúa aquí), y el alto de cada línea sale del rectángulo de sus palabras.

use super::{Cuadro, Lector, LineaLeida, NoSeVe, Objetivo, Ojo};
use std::sync::Mutex;
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{
    CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SelectObject,
    BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
};
use windows::Win32::Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS};
use windows::Win32::UI::WindowsAndMessaging::{GetWindowRect, PW_RENDERFULLCONTENT};

/// Los ojos y el lector de Windows. Los píxeles de la captura viven en un búfer que se reutiliza y se pisa.
pub struct DelSistema {
    bgra: Mutex<Vec<u8>>,
}

pub fn ojos() -> (Box<dyn Ojo>, Box<dyn Lector>) {
    (Box::new(DelSistema { bgra: Mutex::new(Vec::new()) }), Box::new(DelSistema { bgra: Mutex::new(Vec::new()) }))
}

impl Ojo for DelSistema {
    fn nombre(&self) -> &'static str {
        "windows-printwindow"
    }

    fn mirar(&self, objetivo: &Objetivo, cuadro: &mut Cuadro) -> Result<(), NoSeVe> {
        crate::hardware::vigilar("la captura de pantalla");
        let hwnd = crate::sesion::windows::ventana_del_objetivo(&objetivo.bundle, &objetivo.senales)
            .ok_or(NoSeVe::SinVentana)?;
        let mut bgra = self.bgra.lock().map_err(|_| NoSeVe::Fallo("el búfer de la captura quedó envenenado".into()))?;
        let (ancho, alto) = capturar(hwnd, &mut bgra).map_err(NoSeVe::Fallo)?;
        let hecho = super::a_gris(&bgra, ancho, alto, cuadro).map_err(NoSeVe::Fallo);
        // Lo capturado es la pantalla del cliente: se pisa en cuanto pasa a grises.
        bgra.fill(0);
        hecho
    }
}

impl Lector for DelSistema {
    fn leer(&self, cuadro: &Cuadro) -> Result<Vec<LineaLeida>, String> {
        if cuadro.ancho == 0 || cuadro.alto == 0 || cuadro.gris.len() < cuadro.ancho * cuadro.alto {
            return Err("no hay cuadro que leer".into());
        }
        con_com();
        let motor = motor()?;
        let mut bgra = super::gris_a_bgra(cuadro);
        let leido = reconocer(&motor, &bgra, cuadro.ancho, cuadro.alto);
        // Lo que se le pasó al lector es la pantalla del cliente: se pisa antes de soltarse.
        bgra.fill(0);
        leido
    }
}

/// El hilo que lee la pantalla es de la app y no trae COM: se inicia en multihilo, y si ya estaba, no pasa nada.
fn con_com() {
    use windows::Win32::System::Com::{CoInitializeEx, COINIT_MULTITHREADED};
    // SAFETY: iniciar COM en el hilo actual; un segundo inicio en el mismo modo solo devuelve «ya estaba».
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
}

/// El motor de lectura: el de los idiomas del perfil del usuario, y si Windows no tiene ninguno, español o inglés.
fn motor() -> Result<OcrEngine, String> {
    use windows::Globalization::Language;
    if let Ok(m) = OcrEngine::TryCreateFromUserProfileLanguages() {
        return Ok(m);
    }
    for idioma in ["es-ES", "en-US"] {
        if let Ok(l) = Language::CreateLanguage(&windows::core::HSTRING::from(idioma)) {
            if let Ok(m) = OcrEngine::TryCreateFromLanguage(&l) {
                return Ok(m);
            }
        }
    }
    Err("Windows no tiene el reconocimiento de texto de ningún idioma instalado (Ajustes → Hora e idioma)".into())
}

/// El reconocimiento, **el único sitio de este archivo que lee texto**. Solo lo llama el lector.
fn reconocer(motor: &OcrEngine, bgra: &[u8], ancho: usize, alto: usize) -> Result<Vec<LineaLeida>, String> {
    let e = |e: windows::core::Error| format!("el lector de Windows no pudo: {e}");
    let escritor = DataWriter::new().map_err(e)?;
    escritor.WriteBytes(bgra).map_err(e)?;
    let buffer = escritor.DetachBuffer().map_err(e)?;
    let mapa = SoftwareBitmap::CreateCopyFromBuffer(&buffer, BitmapPixelFormat::Bgra8, ancho as i32, alto as i32)
        .map_err(e)?;
    let resultado = motor.RecognizeAsync(&mapa).map_err(e)?.join().map_err(e)?;
    let lineas = resultado.Lines().map_err(e)?;
    let mut leidas = Vec::new();
    for i in 0..lineas.Size().map_err(e)? {
        let linea = lineas.GetAt(i).map_err(e)?;
        let texto = linea.Text().map_err(e)?.to_string();
        let palabras = linea.Words().map_err(e)?;
        let mut alto_px = 0.0f32;
        for j in 0..palabras.Size().map_err(e)? {
            if let Ok(r) = palabras.GetAt(j).and_then(|p| p.BoundingRect()) {
                alto_px = alto_px.max(r.Height);
            }
        }
        leidas.push(LineaLeida { texto, confianza: 1.0, alto: alto_px / alto as f32 });
    }
    Ok(leidas)
}

/// La ventana entera, en BGRA de arriba abajo, en `bgra`. Devuelve su tamaño.
fn capturar(hwnd: HWND, bgra: &mut Vec<u8>) -> Result<(usize, usize), String> {
    // SAFETY: cada objeto GDI que se crea aquí se suelta antes de volver, pase lo que pase con la captura; `GetDIBits`
    // escribe como mucho `ancho × alto × 4` bytes en un búfer de ese tamaño.
    unsafe {
        let mut r = RECT::default();
        GetWindowRect(hwnd, &mut r).map_err(|e| format!("no se pudo medir la ventana: {e}"))?;
        let (ancho, alto) = (r.right - r.left, r.bottom - r.top);
        if ancho <= 0 || alto <= 0 {
            return Err("la ventana de la reunión no tiene tamaño".into());
        }
        let pantalla = GetDC(None);
        let memoria = CreateCompatibleDC(Some(pantalla));
        let mapa = CreateCompatibleBitmap(pantalla, ancho, alto);
        let antes = SelectObject(memoria, HGDIOBJ::from(mapa));
        let pintada = PrintWindow(hwnd, memoria, PRINT_WINDOW_FLAGS(PW_RENDERFULLCONTENT)).as_bool();
        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: ancho,
                // Negativo: las filas de arriba abajo, como las lee el cuadro.
                biHeight: -alto,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let (a, h) = (ancho as usize, alto as usize);
        bgra.clear();
        bgra.resize(a * h * 4, 0);
        SelectObject(memoria, antes);
        let filas = GetDIBits(
            memoria,
            mapa,
            0,
            alto as u32,
            Some(bgra.as_mut_ptr().cast()),
            &mut info,
            DIB_RGB_COLORS,
        );
        let _ = DeleteObject(HGDIOBJ::from(mapa));
        let _ = DeleteDC(memoria);
        ReleaseDC(None, pantalla);
        if !pintada || filas == 0 {
            bgra.fill(0);
            return Err("Windows no entregó la ventana de la reunión".into());
        }
        Ok((a, h))
    }
}
