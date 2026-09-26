//! Captura de audio y pantalla — **MÓDULO PROTEGIDO** (regla del efímero verificable).
//!
//! Todo lo que vive bajo `capture/` es efímero por definición: ring buffers en RAM que mueren
//! al cerrar la sesión. Aquí **no se abre disco ni red**, y no es una costumbre: `pnpm
//! verify:ephemeral` barre este directorio buscando API de archivo y de socket, y la CI se pone
//! roja si aparece una. Si algún día un caso legítimo necesita escribir, exige un ADR citado en
//! la misma línea (`verify-ephemeral:allow`), no un atajo.
//!
//! Se llena en la fase 3 del sprint 001 (dos pistas: micrófono y audio del sistema).

pub mod anillo;
#[cfg(target_os = "macos")]
pub mod nativo;
pub mod remuestreo;

/// Fuera de macOS no hay grifos que abrir, y el resto del crate no tiene por qué enterarse.
///
/// El mismo patrón que `acople`: la app es de macOS, pero el crate compila en cualquier sitio para
/// que `cargo check` sirva de algo en una máquina prestada. Lo que no hace es fingir: cada
/// apertura devuelve el motivo escrito.
#[cfg(not(target_os = "macos"))]
pub mod nativo {
    use super::anillo::Anillo;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
    #[serde(rename_all = "kebab-case", tag = "salida")]
    pub enum Salida {
        Altavoces,
        Auriculares,
        Otra { nombre: String },
        NoSeSabe { motivo: super::PorQueNoSeSabe, nombre: Option<String> },
    }

    impl Salida {
        pub fn puede_haber_eco(&self) -> Option<bool> {
            None
        }
    }

    pub fn salida_de_audio() -> Salida {
        Salida::NoSeSabe { motivo: super::PorQueNoSeSabe::SinSalida, nombre: None }
    }

    pub struct Grifo {
        pub hz_del_dispositivo: u32,
    }

    impl Grifo {
        pub fn muestras_recibidas(&self) -> u64 {
            0
        }
        pub fn del_microfono(_anillo: Arc<Mutex<Anillo>>) -> Result<Self, super::NoAbrio> {
            Err(super::NoAbrio::por(super::PorQueNoAbrio::NoDejo, "la captura de audio solo existe en macOS"))
        }
        pub fn del_sistema(_anillo: Arc<Mutex<Anillo>>) -> Result<Self, super::NoAbrio> {
            Err(super::NoAbrio::por(super::PorQueNoAbrio::NoDejo, "el audio del sistema solo se captura en macOS"))
        }
    }
}

pub use anillo::Anillo;
pub use remuestreo::Remuestreador;

/// **Por qué una pista no abrió**, en un conjunto CERRADO.
///
/// Hasta el sprint 002 el motivo cruzaba a la pantalla como una frase libre en español —«no se pudo
/// crear el tap del audio del sistema (estado 560947818 «!hog»)»—, que no se traduce y enseña un
/// código de macOS a quien no puede hacer nada con él. La mirada 17-quater lo cerró en cinco
/// porqués, cada uno con su frase en los dos idiomas y su salida (`kit.html` §8-ter). El detalle
/// técnico no se pierde: viaja en [`NoAbrio::detalle`] y va al log.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PorQueNoAbrio {
    /// macOS no ha concedido el micrófono.
    SinPermisoDelMicrofono,
    /// macOS no ha concedido el audio del sistema (`kTCCServiceAudioCapture`).
    SinPermisoDelAudio,
    /// macOS contestó `!hog`: el dispositivo lo tiene cogido otra app.
    DispositivoOcupado,
    /// El dispositivo entrega un formato que el grifo no sabe leer.
    FormatoIlegible,
    /// Cualquier otro «no» de macOS. La salida es la misma: volver a intentarlo.
    NoDejo,
}

/// Una pista que no abrió: **el porqué que se enseña, y el detalle que se registra.**
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NoAbrio {
    pub porque: PorQueNoAbrio,
    /// Para el log, nunca para la pantalla: puede llevar códigos de macOS.
    pub detalle: String,
}

impl NoAbrio {
    pub fn por(porque: PorQueNoAbrio, detalle: impl Into<String>) -> Self {
        Self { porque, detalle: detalle.into() }
    }
}

impl std::fmt::Display for NoAbrio {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {}", self.porque, self.detalle)
    }
}

/// **Por qué no se sabe por dónde sale el sonido.** Cerrado por lo mismo que [`PorQueNoAbrio`]: el
/// nombre del dispositivo viaja aparte, porque la frase lo cita y el nombre no se traduce.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PorQueNoSeSabe {
    /// Este Mac no declara una salida de audio por defecto.
    SinSalida,
    /// El dispositivo no dice cómo está conectado.
    SinConexion,
    /// Conectado por dentro, pero no dice por dónde suena.
    SinFuente,
}

/// Marca de quién habló. **Por PISTA, jamás por biometría** (regla de cero huellas de voz):
/// el micrófono es el consultor y el audio del sistema es la contraparte, y eso se sabe por
/// el origen de la muestra, no por analizar la voz.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Pista {
    /// Micrófono: el consultor.
    Microfono,
    /// Audio del sistema: la contraparte.
    Sistema,
}

impl Pista {
    /// Etiqueta estable para telemetría y transcript. Metadata, nunca contenido.
    pub fn etiqueta(self) -> &'static str {
        match self {
            Pista::Microfono => "microfono",
            Pista::Sistema => "sistema",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_pista_identifica_al_hablante_sin_mirar_la_voz() {
        // El test que fija la regla: la atribución sale del ORIGEN de la muestra.
        assert_eq!(Pista::Microfono.etiqueta(), "microfono");
        assert_eq!(Pista::Sistema.etiqueta(), "sistema");
        assert_ne!(Pista::Microfono, Pista::Sistema);
    }
}
