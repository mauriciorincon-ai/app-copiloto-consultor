//! Transcripción local — **MÓDULO PROTEGIDO** (regla del efímero verificable).
//!
//! Mismas reglas que `capture/`: RAM y nada más. El audio no sale del equipo para volverse texto
//! (regla de nada crudo fuera), y el texto que produce vive en una ventana deslizante de turnos
//! que muere al cerrar la sesión.
//!
//! **Qué decidió el ADR del motor, y con qué números.** La orden dejaba la elección abierta —
//! Apple `SpeechAnalyzer` con respaldo `whisper-rs`, «el motor se decide por ADR con medición, no
//! por preferencia»—. La medición está en la bitácora (fase 3a) y en `decisions/006`: seis
//! segundos de audio transcritos en **84 ms**, setenta veces más rápido que el tiempo real, con el
//! modelo ya instalado y sin que el usuario descargue nada al repo ni a su carpeta. Whisper
//! large-v3-turbo habría costado 1,5 GB de descarga para ir más despacio. No hubo empate que
//! deshacer.
//!
//! **Y el `trait` no sobra por eso.** Existe porque el motor elegido **no está siempre**: pide
//! macOS 26, pide que el modelo del idioma esté instalado, y pide que la app se haya compilado con
//! el puente de Swift. Cada una de esas tres ausencias tiene que tener un nombre que la app pueda
//! enseñar, y no un fallo genérico. El proveedor [`Mudo`] es de primera clase por lo mismo: es lo
//! que corre en la CI, donde no hay ni micrófono ni modelo.

pub mod apple;
pub mod ventana;

pub use ventana::Ventana;

use crate::capture::Pista;

/// Un turno transcrito. Es lo que la banda enseña y lo que el disparador de la fase 4 leerá.
///
/// **No lleva marca de tiempo del reloj del sistema, solo del audio.** Un `SystemTime` sería una
/// fecha real de una reunión real, y esto es un objeto que existe para vivir en memoria y morir;
/// darle una fecha absoluta sería amueblarlo para persistir.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Turno {
    /// Quién habló, **por el origen de la muestra** (regla de cero huellas de voz).
    pub pista: Pista,
    pub desde_ms: usize,
    pub hasta_ms: usize,
    pub texto: String,
    /// La hora del reloj a la que empezó este turno, «14:02», como la escribe la maqueta.
    ///
    /// Vive en memoria con el turno y muere con él, igual que el texto. Se calcula **aquí y no en
    /// la interfaz** porque la interfaz no sabe cuándo empezó la sesión, y hacérselo saber
    /// obligaría a pasear por la app una marca de tiempo de una reunión real — que es justo la
    /// clase de dato que esta app procura no tener dando vueltas.
    #[serde(default)]
    pub hora: String,
    /// **Este turno es un reflejo, no una persona.** Se pone cuando el micrófono captó por los
    /// altavoces lo que el cliente estaba diciendo (ver `voz::eco`). No se borra el turno: se
    /// marca, porque un turno que desaparece sin decir por qué es justo el silencio que esta app
    /// no se permite — y porque la respuesta de verdad es «ponte los auriculares», que solo puede
    /// dar el usuario.
    #[serde(default)]
    pub eco: bool,
}

impl Turno {
    pub fn duracion_ms(&self) -> usize {
        self.hasta_ms.saturating_sub(self.desde_ms)
    }
}

/// En qué estado está el motor para un idioma dado. Los cuatro se enseñan tal cual en la pantalla
/// de Idioma: la app no colapsa «no puedo» en una sola palabra.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case", tag = "estado")]
pub enum Disponibilidad {
    /// Se puede transcribir ahora mismo.
    Listo,
    /// El motor conoce el idioma pero su modelo no está en este Mac. Lo instala macOS, a petición
    /// del usuario, desde la pantalla de Idioma.
    SinModelo,
    /// El motor no conoce ese idioma. No hay nada que instalar.
    IdiomaDesconocido,
    /// No hay motor en este Mac. El motivo es del sistema, no del idioma.
    SinMotor { motivo: String },
}

/// Por qué no se pudo transcribir un turno.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fallo {
    NoDisponible(Disponibilidad),
    /// El motor devolvió un error. Se guarda el código crudo: es lo único que se puede registrar
    /// sin escribir en el log nada de lo que se dijo.
    Motor(i32),
}

/// Lo que cualquier motor de transcripción tiene que saber hacer.
pub trait Motor: Send + Sync {
    /// Nombre corto para la pantalla y el log. Metadato, jamás contenido.
    fn nombre(&self) -> &'static str;
    fn disponibilidad(&self, idioma: &str) -> Disponibilidad;
    /// Instala el modelo del idioma. **Usa la red** y solo se llama si el usuario lo pide.
    fn instalar(&self, idioma: &str) -> Disponibilidad;
    /// Transcribe un turno entero. `muestras` es mono a `hz`.
    fn transcribir(&self, idioma: &str, muestras: &[f32], hz: u32) -> Result<String, Fallo>;
    /// Cuántos idiomas puede tener listos a la vez. macOS impone un techo (cinco) y la pantalla de
    /// Idioma lo enseña; un motor que no transcribe contesta cero, que es la verdad.
    fn techo_de_idiomas(&self) -> u32;
    /// Los idiomas que este Mac sabe transcribir. Se **pregunta al sistema**, no se lleva escrita:
    /// una lista nuestra desfasada ofrecería idiomas que no existen o escondería los que sí.
    fn idiomas(&self) -> Vec<String>;
}

/// El motor que no transcribe **y lo dice**.
///
/// No es un apaño para los tests: es el proveedor que corre en la integración continua y en
/// cualquier Mac sin macOS 26, y el patrón `mock como proveedor de primera clase` que esta casa
/// aplica desde hace varios ciclos. Lo importante es lo que NO hace: no devuelve texto inventado,
/// no devuelve cadena vacía como si hubiera oído silencio. Devuelve el motivo.
pub struct Mudo {
    motivo: String,
}

impl Mudo {
    pub fn por(motivo: impl Into<String>) -> Self {
        Self { motivo: motivo.into() }
    }
}

impl Motor for Mudo {
    fn nombre(&self) -> &'static str {
        "mudo"
    }
    fn disponibilidad(&self, _idioma: &str) -> Disponibilidad {
        Disponibilidad::SinMotor { motivo: self.motivo.clone() }
    }
    fn instalar(&self, idioma: &str) -> Disponibilidad {
        self.disponibilidad(idioma)
    }
    fn transcribir(&self, idioma: &str, _muestras: &[f32], _hz: u32) -> Result<String, Fallo> {
        Err(Fallo::NoDisponible(self.disponibilidad(idioma)))
    }
    fn techo_de_idiomas(&self) -> u32 {
        0
    }
    fn idiomas(&self) -> Vec<String> {
        Vec::new()
    }
}

/// El motor de este Mac. En macOS con el puente compilado, Apple; si no, el que dice por qué no.
pub fn motor_de_la_casa() -> Box<dyn Motor> {
    apple::motor()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_motor_mudo_no_inventa_silencio() {
        let m = Mudo::por("no hay macOS 26 en esta máquina");
        let fallo = m.transcribir("es-ES", &[0.1; 16_000], 16_000).unwrap_err();
        match fallo {
            Fallo::NoDisponible(Disponibilidad::SinMotor { motivo }) => {
                assert!(motivo.contains("macOS 26"));
            }
            otro => panic!("devolvió {otro:?} en vez del motivo"),
        }
    }

    /// El detalle que parece menor y no lo es: una cadena vacía es indistinguible de «el cliente
    /// no dijo nada», y la banda la pintaría como un turno en blanco. El motor ausente tiene que
    /// doler en el tipo, no en la vista.
    #[test]
    fn un_motor_ausente_no_puede_confundirse_con_un_turno_callado() {
        let m = Mudo::por("sin puente");
        assert!(m.transcribir("es-ES", &[0.0; 100], 16_000).is_err());
    }

    /// Un motor que no transcribe no puede ofrecer idiomas ni prometer cuántos caben. Cero y lista
    /// vacía son la verdad; cualquier otra cosa sería una pantalla de Idioma con botones muertos.
    #[test]
    fn el_motor_mudo_no_ofrece_idiomas_que_no_tiene() {
        let m = Mudo::por("sin puente");
        assert_eq!(m.techo_de_idiomas(), 0);
        assert!(m.idiomas().is_empty());
    }

    #[test]
    fn el_turno_sabe_cuanto_duro() {
        let t = Turno {
            pista: Pista::Sistema,
            desde_ms: 1_000,
            hasta_ms: 3_400,
            texto: "¿eso está dentro del alcance?".into(),
            hora: "14:02".into(),
            eco: false,
        };
        assert_eq!(t.duracion_ms(), 2_400);
    }
}
