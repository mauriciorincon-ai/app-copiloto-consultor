//! La cara nativa del acople: **Accessibility** para medir y encoger ventanas ajenas, y
//! **NSWorkspace** para saber quién está al frente y comprobar que un PID sigue siendo quien era.
//!
//! Todo lo `unsafe` de este módulo vive aquí, y es poco a propósito: seis funciones de
//! `ApplicationServices` y dos clases de AppKit. Hacia arriba solo salen [`Marco`], `String` y
//! `bool` — la lógica que decide qué hacer con esos valores está en el módulo padre, en Rust
//! seguro y con tests.
//!
//! **Lo que NO se pide, pudiendo:** títulos de ventana, contenido, jerarquía de elementos, el
//! árbol de ninguna aplicación. La Accessibility API es una llave maestra; aquí se usan tres
//! atributos (`AXWindows`, `AXPosition`, `AXSize`) y se escribe uno.

use super::Marco;
use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use core_foundation_sys::array::{CFArrayGetCount, CFArrayGetValueAtIndex, CFArrayRef};
use core_foundation_sys::base::{CFRelease, CFTypeRef};
use core_foundation_sys::dictionary::CFDictionaryRef;
use core_foundation_sys::string::CFStringRef;
use std::ffi::c_void;

type AXUIElementRef = CFTypeRef;
type AXValueRef = CFTypeRef;
type AXError = i32;

const EXITO: AXError = 0;
const TIPO_CGPOINT: u32 = 1;
const TIPO_CGSIZE: u32 = 2;

/// La clave que hace que macOS **pregunte** al usuario en vez de responder «no» en silencio.
/// Es el literal que declara `ApplicationServices` (`kAXTrustedCheckOptionPrompt`); se escribe
/// aquí en vez de enlazar el símbolo global para no arrastrar una extern static por una cadena.
const CLAVE_PREGUNTAR: &str = "AXTrustedCheckOptionPrompt";

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CGPoint {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct CGSize {
    ancho: f64,
    alto: f64,
}

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrustedWithOptions(opciones: CFDictionaryRef) -> u8;
    fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
    fn AXUIElementCopyAttributeValue(
        elemento: AXUIElementRef,
        atributo: CFStringRef,
        salida: *mut CFTypeRef,
    ) -> AXError;
    fn AXUIElementSetAttributeValue(
        elemento: AXUIElementRef,
        atributo: CFStringRef,
        valor: CFTypeRef,
    ) -> AXError;
    fn AXValueCreate(tipo: u32, puntero: *const c_void) -> AXValueRef;
    fn AXValueGetValue(valor: AXValueRef, tipo: u32, puntero: *mut c_void) -> u8;
}

/// ¿Nos concedió el usuario el permiso de Accesibilidad? **No pregunta**: solo mira.
pub fn permiso_concedido() -> bool {
    let vacio: CFDictionary<CFString, CFBoolean> = CFDictionary::from_CFType_pairs(&[]);
    unsafe { AXIsProcessTrustedWithOptions(vacio.as_concrete_TypeRef()) != 0 }
}

/// Pide el permiso: macOS abre su propio diálogo y lleva a Ajustes del Sistema. Devuelve si ya
/// estaba concedido — **nunca** `true` por haber preguntado: el usuario tiene que ir a Ajustes y
/// volver, y hasta entonces la respuesta honesta es `false` y la banda flota.
pub fn pedir_permiso() -> bool {
    let opciones = CFDictionary::from_CFType_pairs(&[(
        CFString::new(CLAVE_PREGUNTAR).as_CFType(),
        CFBoolean::true_value().as_CFType(),
    )]);
    unsafe { AXIsProcessTrustedWithOptions(opciones.as_concrete_TypeRef()) != 0 }
}

/// Copia un atributo. El valor vuelve con **regla de creación**: quien lo recibe lo libera.
unsafe fn atributo(elemento: AXUIElementRef, nombre: &str) -> Option<CFTypeRef> {
    let clave = CFString::new(nombre);
    let mut salida: CFTypeRef = std::ptr::null();
    let error = AXUIElementCopyAttributeValue(elemento, clave.as_concrete_TypeRef(), &mut salida);
    (error == EXITO && !salida.is_null()).then_some(salida)
}

unsafe fn leer_axvalue<T: Default>(valor: AXValueRef, tipo: u32) -> Option<T> {
    let mut destino = T::default();
    let ok = AXValueGetValue(valor, tipo, &mut destino as *mut T as *mut c_void) != 0;
    ok.then_some(destino)
}

/// Posición y tamaño de una ventana, en puntos y con origen arriba-izquierda.
unsafe fn marco_de(ventana: AXUIElementRef) -> Option<Marco> {
    let p = atributo(ventana, "AXPosition")?;
    let punto: Option<CGPoint> = leer_axvalue(p, TIPO_CGPOINT);
    CFRelease(p);
    let punto = punto?;

    let s = atributo(ventana, "AXSize")?;
    let tamano: Option<CGSize> = leer_axvalue(s, TIPO_CGSIZE);
    CFRelease(s);
    let tamano = tamano?;

    Some(Marco::nuevo(punto.x, punto.y, tamano.ancho, tamano.alto))
}

/// Escribe el alto de una ventana y **devuelve lo que el sistema dejó**, releyéndolo. Muchas
/// aplicaciones acotan lo que se les pide; guardar lo pedido en vez de lo conseguido dejaría la
/// devolución sin poder reconocer su propia huella.
unsafe fn poner_alto(ventana: AXUIElementRef, alto: f64) -> Option<Marco> {
    let actual = marco_de(ventana)?;
    let pedido = CGSize {
        ancho: actual.ancho,
        alto,
    };
    let valor = AXValueCreate(TIPO_CGSIZE, &pedido as *const CGSize as *const c_void);
    if valor.is_null() {
        return None;
    }
    let clave = CFString::new("AXSize");
    let error = AXUIElementSetAttributeValue(ventana, clave.as_concrete_TypeRef(), valor);
    CFRelease(valor);
    (error == EXITO).then(|| marco_de(ventana)).flatten()
}

/// Un elemento de aplicación y sus ventanas, con la liberación resuelta por el tipo — el `Drop`
/// existe porque este módulo mezcla dos reglas de memoria (creación y obtención) y confiarlas a
/// la disciplina de quien lo lea es cómo se pierde un objeto de Core Foundation.
struct Aplicacion {
    elemento: AXUIElementRef,
}

impl Aplicacion {
    fn de(pid: i32) -> Option<Self> {
        let elemento = unsafe { AXUIElementCreateApplication(pid) };
        (!elemento.is_null()).then_some(Self { elemento })
    }

    /// Las ventanas de la aplicación, ya medidas. Cada `(indice, marco)` basta para operar: el
    /// índice solo vale dentro de esta llamada, así que nada se guarda de una a otra.
    fn ventanas(&self) -> Vec<(usize, Marco)> {
        unsafe {
            let Some(lista) = atributo(self.elemento, "AXWindows") else {
                return Vec::new();
            };
            let arreglo = lista as CFArrayRef;
            let n = CFArrayGetCount(arreglo);
            let mut salida = Vec::new();
            for i in 0..n {
                let v = CFArrayGetValueAtIndex(arreglo, i) as AXUIElementRef;
                if v.is_null() {
                    continue;
                }
                if let Some(m) = marco_de(v) {
                    salida.push((i as usize, m));
                }
            }
            CFRelease(lista);
            salida
        }
    }

    /// Aplica `accion` a la ventana `indice` volviendo a pedir la lista — las referencias de la
    /// lista anterior murieron con su `CFRelease`.
    fn con_ventana<T>(&self, indice: usize, accion: impl FnOnce(AXUIElementRef) -> T) -> Option<T> {
        unsafe {
            let lista = atributo(self.elemento, "AXWindows")?;
            let arreglo = lista as CFArrayRef;
            let resultado = (indice < CFArrayGetCount(arreglo) as usize)
                .then(|| CFArrayGetValueAtIndex(arreglo, indice as isize) as AXUIElementRef)
                .filter(|v| !v.is_null())
                .map(accion);
            CFRelease(lista);
            resultado
        }
    }
}

impl Drop for Aplicacion {
    fn drop(&mut self) {
        unsafe { CFRelease(self.elemento) }
    }
}

/// Las ventanas de un proceso, medidas. Vacío si no hay permiso o el proceso no las expone.
pub fn ventanas_de(pid: i32) -> Vec<(usize, Marco)> {
    Aplicacion::de(pid).map(|a| a.ventanas()).unwrap_or_default()
}

/// Pone el alto de una ventana concreta y devuelve el marco resultante **leído del sistema**.
pub fn encoger(pid: i32, indice: usize, alto: f64) -> Option<Marco> {
    let app = Aplicacion::de(pid)?;
    app.con_ventana(indice, |v| unsafe { poner_alto(v, alto) })
        .flatten()
}

// ---------------------------------------------------------------------------------------------
// NSWorkspace — quién está al frente, y el fondo de escritorio
// ---------------------------------------------------------------------------------------------

/// PID y nombre de la aplicación que está al frente, **saltándonos a nosotros mismos**.
/// `None` si la de delante somos nosotros o si el sistema no contesta.
pub fn aplicacion_al_frente() -> Option<(i32, String)> {
    use objc2_app_kit::NSWorkspace;
    let nuestro = std::process::id() as i32;
    let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let pid = app.processIdentifier();
    if pid == nuestro {
        return None;
    }
    let nombre = app
        .localizedName()
        .map(|n| n.to_string())
        .unwrap_or_else(|| format!("pid {pid}"));
    Some((pid, nombre))
}

/// ¿El PID de la huella sigue siendo la misma aplicación? El cerrojo contra el reciclado de PID:
/// tras una caída, ese número puede pertenecer ya a otro programa, y devolverle un tamaño a la
/// ventana de otro sería el peor fallo que este módulo puede cometer.
pub fn sigue_siendo(pid: i32, nombre: &str) -> bool {
    use objc2_app_kit::NSRunningApplication;
    NSRunningApplication::runningApplicationWithProcessIdentifier(pid)
        .and_then(|a| a.localizedName())
        .is_some_and(|n| n.to_string() == nombre)
}

/// La ruta del fondo de escritorio de la pantalla principal, si el sistema la da.
pub fn fondo_de_escritorio() -> Option<std::path::PathBuf> {
    use objc2_app_kit::{NSScreen, NSWorkspace};
    use objc2_foundation::MainThreadMarker;
    let hilo = MainThreadMarker::new()?;
    let pantalla = NSScreen::mainScreen(hilo)?;
    let url = NSWorkspace::sharedWorkspace().desktopImageURLForScreen(&pantalla)?;
    Some(std::path::PathBuf::from(url.path()?.to_string()))
}
