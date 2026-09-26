//! LOS PERMISOS DE macOS — leerlos, no pedirlos.
//!
//! La maqueta lo decidió y este módulo lo obedece: *«Tú los concedes en el sistema, no aquí. La
//! app funciona sin ellos: solo hace menos.»* De ahí salen las dos reglas de aquí dentro:
//!
//! 1. **Se leen, no se piden.** Ningún diálogo aparece por sorpresa. El botón de la pantalla abre
//!    el panel de Ajustes del Sistema que corresponde, y el usuario decide allí. Un permiso
//!    pedido en el primer arranque, antes de que la app haya demostrado nada, se deniega — y un
//!    «no» de macOS es muchísimo más caro de deshacer que un «todavía no».
//! 2. **«No lo sé» es un estado**, y distinto de «no concedido». macOS no siempre distingue entre
//!    *nunca se preguntó* y *se denegó*; decir «denegado» cuando lo que hay es silencio manda al
//!    usuario a buscar un permiso que nunca rechazó.
//!
//! Los cuatro permisos y de dónde sale cada respuesta:
//!
//! | Permiso | API | Qué distingue |
//! |---|---|---|
//! | Micrófono | `AVCaptureDevice.authorizationStatusForMediaType:` | los cuatro estados |
//! | Audio del sistema | `TCCAccessPreflight("kTCCServiceAudioCapture")` — **privada**, ver [`nativo::audio`] | sí / no |
//! | Pantalla | `CGPreflightScreenCaptureAccess()` | sí / no — **no distingue** denegado de sin preguntar |
//! | Accesibilidad (el acople) | `AXIsProcessTrusted` | sí / no |
//!
//! **Audio del sistema y pantalla son dos permisos**, aunque Ajustes los enseñe en el mismo panel
//! («Grabación de pantalla y audio del sistema»). Hasta el sprint 002 la app leía el de pantalla y lo
//! usaba para las dos filas; la mirada 17-quater lo encontró falso, comprobado en `tccd` de este Mac.

use serde::Serialize;

/// El estado de UN permiso.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Estado {
    /// Nunca se preguntó, o el sistema no distingue y no hay concesión.
    SinConceder,
    Concedido,
    /// El usuario dijo que no, o una política del equipo lo impide.
    Denegado,
    /// No se pudo averiguar. **No es «no»**: es que no lo sabemos.
    NoSeSabe,
}

impl Estado {
    /// De los cuatro valores de `AVAuthorizationStatus`. Cualquier otro número es de una versión
    /// de macOS que no conocemos, y eso es «no lo sé», no «no».
    pub fn de_avfoundation(codigo: isize) -> Self {
        match codigo {
            0 => Estado::SinConceder, // notDetermined
            1 => Estado::Denegado,    // restricted (política del equipo)
            2 => Estado::Denegado,    // denied
            3 => Estado::Concedido,   // authorized
            _ => Estado::NoSeSabe,
        }
    }

    /// De lo que contesta `TCCAccessPreflight`. **Solo el `0` es «concedido»**: los demás números
    /// no están documentados, y tomarlos por «denegado» mandaría al usuario a revocar algo que quizá
    /// nunca rechazó.
    pub fn de_tcc(codigo: i32) -> Self {
        if codigo == 0 {
            Estado::Concedido
        } else {
            Estado::SinConceder
        }
    }
}

/// Los cuatro permisos que esta app llega a necesitar. Ni uno más.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Permisos {
    pub microfono: Estado,
    /// El audio del sistema: la pista del cliente.
    pub audio: Estado,
    pub pantalla: Estado,
    pub accesibilidad: Estado,
}

/// Cuál de los estados que la maqueta dibuja le toca a la pantalla.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cara {
    SinConceder,
    Concedido,
    /// Alguno se cayó. La maqueta lo llama «revocado a mitad» y es el caso que de verdad duele:
    /// pasa **durante** una reunión, porque desde Sequoia macOS revalida el permiso de pantalla
    /// cada cierto tiempo.
    Revocado,
}

/// Los que la pantalla de permisos exige para escuchar. La accesibilidad no entra: es del acople,
/// vive en su propia tarjeta y la app hace su trabajo entero sin ella.
fn los_de_escuchar(p: &Permisos) -> [Estado; 2] {
    [p.microfono, p.audio]
}

/// Qué cara pone la pantalla de permisos.
///
/// El orden de las preguntas es el del daño: **primero si algo se cayó**. Un permiso denegado en
/// mitad de una reunión es lo único de esta pantalla que interrumpe al usuario, y preguntarlo
/// después de «¿están todos?» lo escondería detrás del caso feliz.
pub fn cara(p: &Permisos) -> Cara {
    if los_de_escuchar(p).contains(&Estado::Denegado) {
        return Cara::Revocado;
    }
    if los_de_escuchar(p).iter().all(|e| *e == Estado::Concedido) {
        return Cara::Concedido;
    }
    Cara::SinConceder
}

/// ¿Puede la app escuchar la reunión? Las dos pistas necesitan sus dos permisos.
pub fn puede_escuchar(p: &Permisos) -> bool {
    p.microfono == Estado::Concedido && p.audio == Estado::Concedido
}

/// ¿Puede acoplar la ventana de la reunión?
pub fn puede_acoplar(p: &Permisos) -> bool {
    p.accesibilidad == Estado::Concedido
}

/// El panel de Ajustes del Sistema donde se concede cada permiso.
///
/// Se abre el panel, **no se pide el permiso**: es la decisión de la maqueta y además es lo único
/// que funciona para la Accesibilidad, que no tiene diálogo propio que se pueda invocar dos veces.
pub fn ajustes_de(cual: &str) -> Option<&'static str> {
    match cual {
        "microfono" => Some("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"),
        "pantalla" => Some("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"),
        // El mismo panel que la pantalla: Ajustes los enseña juntos aunque sean dos permisos.
        "audio" => Some("x-apple.systempreferences:com.apple.preference.security?Privacy_ScreenCapture"),
        "accesibilidad" => Some("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------------
// La lectura nativa
// ---------------------------------------------------------------------------------------------

#[cfg(target_os = "macos")]
mod nativo {
    use super::*;

    // Enlazar el framework hace que su clase exista en tiempo de ejecución. No se usa ninguna
    // función suya directamente: la llamada va por mensaje de Objective-C, que es lo que evita
    // arrastrar las vinculaciones enteras de AVFoundation —enormes— por una sola pregunta.
    #[link(name = "AVFoundation", kind = "framework")]
    extern "C" {}

    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        /// Pregunta **sin** provocar el diálogo. La variante `CGRequestScreenCaptureAccess` sí lo
        /// provoca, y por eso no se usa: aquí se lee, no se pide.
        fn CGPreflightScreenCaptureAccess() -> bool;
    }

    /// `AVMediaTypeAudio`, que es literalmente el código de cuatro letras `soun`.
    const MEDIO_AUDIO: &str = "soun";

    pub fn microfono() -> Estado {
        use objc2::runtime::AnyClass;
        use objc2_foundation::NSString;

        let Some(clase) = AnyClass::get(c"AVCaptureDevice") else {
            return Estado::NoSeSabe;
        };
        let medio = NSString::from_str(MEDIO_AUDIO);
        let codigo: isize =
            unsafe { objc2::msg_send![clase, authorizationStatusForMediaType: &*medio] };
        Estado::de_avfoundation(codigo)
    }

    /// `CGPreflightScreenCaptureAccess` solo dice sí o no. **No distingue** «denegado» de «nunca
    /// se preguntó», así que un `false` se reporta como [`Estado::SinConceder`] y no como
    /// denegado: mandar al usuario a revocar algo que nunca concedió es peor que quedarse corto.
    pub fn pantalla() -> Estado {
        if unsafe { CGPreflightScreenCaptureAccess() } {
            Estado::Concedido
        } else {
            Estado::SinConceder
        }
    }

    /// **El permiso del audio del sistema, con una API privada — y por qué.**
    ///
    /// `kTCCServiceAudioCapture` no tiene API pública para LEERLO sin provocar el diálogo. La
    /// privada `TCCAccessPreflight` sí, y es la que usa el ejemplo de Apple de *process taps* que
    /// circula (AudioCap). Probada en este Mac (bitácora, mirada 17-quater): contesta `0` para lo
    /// concedido.
    ///
    /// Se busca con `dlsym` y no se enlaza: si una versión futura de macOS la quita, la app arranca
    /// igual y la fila dice **«no se sabe»**. Y solo el `0` es «concedido»; cualquier otro número es
    /// «sin conceder», nunca «denegado», por la regla 2 de este módulo. Es una API privada en una
    /// app que se firma y se notariza fuera de la App Store: queda declarada aquí y en la bitácora.
    pub fn audio() -> Estado {
        use std::ffi::{c_int, c_void};
        type Preflight = unsafe extern "C" fn(*const c_void, *const c_void) -> c_int;

        let h = unsafe {
            libc::dlopen(
                c"/System/Library/PrivateFrameworks/TCC.framework/Versions/A/TCC".as_ptr(),
                libc::RTLD_LAZY,
            )
        };
        if h.is_null() {
            return Estado::NoSeSabe;
        }
        let f = unsafe { libc::dlsym(h, c"TCCAccessPreflight".as_ptr()) };
        if f.is_null() {
            return Estado::NoSeSabe;
        }
        let preflight: Preflight = unsafe { std::mem::transmute::<*mut c_void, Preflight>(f) };
        let servicio = objc2_foundation::NSString::from_str("kTCCServiceAudioCapture");
        let puntero = &*servicio as *const objc2_foundation::NSString as *const c_void;
        Estado::de_tcc(unsafe { preflight(puntero, std::ptr::null()) })
    }

    pub fn accesibilidad() -> Estado {
        if crate::acople::hay_permiso() {
            Estado::Concedido
        } else {
            Estado::SinConceder
        }
    }

    pub fn leer() -> Permisos {
        Permisos {
            microfono: microfono(),
            audio: audio(),
            pantalla: pantalla(),
            accesibilidad: accesibilidad(),
        }
    }
}

#[cfg(not(target_os = "macos"))]
mod nativo {
    use super::*;
    pub fn leer() -> Permisos {
        Permisos {
            microfono: Estado::NoSeSabe,
            audio: Estado::NoSeSabe,
            pantalla: Estado::NoSeSabe,
            accesibilidad: Estado::NoSeSabe,
        }
    }
}

pub use nativo::leer;

#[cfg(test)]
mod tests {
    use super::*;

    /// El audio del sistema va con la pantalla en los casos de antes: eran el mismo estado hasta el
    /// sprint 002, y así los tests de siempre siguen diciendo lo que decían.
    fn p(m: Estado, pa: Estado, a: Estado) -> Permisos {
        Permisos { microfono: m, audio: pa, pantalla: pa, accesibilidad: a }
    }

    /// **Audio y pantalla son dos permisos, y escuchar depende del de audio.** Con la pantalla
    /// negada la app escucha igual: lo que pierde es leerla.
    #[test]
    fn escuchar_depende_del_audio_y_no_de_la_pantalla() {
        let sin_pantalla = Permisos {
            microfono: Estado::Concedido,
            audio: Estado::Concedido,
            pantalla: Estado::SinConceder,
            accesibilidad: Estado::Concedido,
        };
        assert!(puede_escuchar(&sin_pantalla));
        let sin_audio = Permisos { audio: Estado::SinConceder, pantalla: Estado::Concedido, ..sin_pantalla };
        assert!(!puede_escuchar(&sin_audio));
    }

    #[test]
    fn de_tcc_solo_el_cero_es_concedido() {
        assert_eq!(Estado::de_tcc(0), Estado::Concedido);
        for otro in [1, 2, -1, 99] {
            assert_eq!(Estado::de_tcc(otro), Estado::SinConceder, "{otro}");
        }
    }

    #[test]
    fn los_codigos_de_avfoundation_se_traducen_uno_a_uno() {
        assert_eq!(Estado::de_avfoundation(0), Estado::SinConceder);
        assert_eq!(Estado::de_avfoundation(1), Estado::Denegado);
        assert_eq!(Estado::de_avfoundation(2), Estado::Denegado);
        assert_eq!(Estado::de_avfoundation(3), Estado::Concedido);
    }

    /// Un código que no conocemos es de una versión de macOS futura. «No lo sé» nunca puede
    /// degradarse a «concedido»: sería afirmar un permiso que no tenemos.
    #[test]
    fn un_codigo_desconocido_es_no_lo_se_y_jamas_concedido() {
        for codigo in [-1, 4, 99] {
            assert_eq!(Estado::de_avfoundation(codigo), Estado::NoSeSabe);
        }
    }

    #[test]
    fn con_los_dos_concedidos_la_pantalla_esta_en_verde() {
        assert_eq!(cara(&p(Estado::Concedido, Estado::Concedido, Estado::SinConceder)), Cara::Concedido);
    }

    /// El caso que de verdad duele, y por eso se pregunta ANTES que el feliz: macOS revalida el
    /// permiso de pantalla cada cierto tiempo y se cae **en mitad de una reunión**.
    #[test]
    fn uno_denegado_gana_a_todo_lo_demas() {
        assert_eq!(cara(&p(Estado::Concedido, Estado::Denegado, Estado::Concedido)), Cara::Revocado);
        assert_eq!(cara(&p(Estado::Denegado, Estado::Concedido, Estado::Concedido)), Cara::Revocado);
    }

    #[test]
    fn sin_preguntar_nada_la_pantalla_invita_en_vez_de_alarmar() {
        assert_eq!(cara(&p(Estado::SinConceder, Estado::SinConceder, Estado::SinConceder)), Cara::SinConceder);
        assert_eq!(cara(&p(Estado::Concedido, Estado::SinConceder, Estado::SinConceder)), Cara::SinConceder);
    }

    /// La accesibilidad es del acople y la app hace su trabajo entero sin ella: no puede poner la
    /// pantalla de permisos en rojo.
    #[test]
    fn la_accesibilidad_no_pinta_la_pantalla_de_permisos() {
        let sin = p(Estado::Concedido, Estado::Concedido, Estado::Denegado);
        assert_eq!(cara(&sin), Cara::Concedido);
        assert!(puede_escuchar(&sin));
        assert!(!puede_acoplar(&sin));
    }

    #[test]
    fn no_se_puede_escuchar_con_lo_que_no_se_sabe() {
        assert!(!puede_escuchar(&p(Estado::NoSeSabe, Estado::Concedido, Estado::Concedido)));
        assert!(!puede_escuchar(&p(Estado::Concedido, Estado::SinConceder, Estado::Concedido)));
    }

    #[test]
    fn cada_permiso_sabe_a_que_panel_de_ajustes_lleva() {
        for cual in ["microfono", "audio", "pantalla", "accesibilidad"] {
            let url = ajustes_de(cual).unwrap_or_else(|| panic!("«{cual}» no sabe adónde lleva"));
            assert!(url.starts_with("x-apple.systempreferences:"), "{url}");
        }
        assert_eq!(ajustes_de("camara"), None, "esta app no pide la cámara");
    }
}
