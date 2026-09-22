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
        NoSeSabe { motivo: String },
    }

    impl Salida {
        pub fn puede_haber_eco(&self) -> Option<bool> {
            None
        }
    }

    pub fn salida_de_audio() -> Salida {
        Salida::NoSeSabe { motivo: "Angel Ghost solo sabe mirar la salida de audio en macOS".into() }
    }

    pub struct Grifo {
        pub hz_del_dispositivo: u32,
    }

    impl Grifo {
        pub fn muestras_recibidas(&self) -> u64 {
            0
        }
        pub fn del_microfono(_anillo: Arc<Mutex<Anillo>>) -> Result<Self, String> {
            Err("la captura de audio de Angel Ghost solo existe en macOS".into())
        }
        pub fn del_sistema(_anillo: Arc<Mutex<Anillo>>) -> Result<Self, String> {
            Err("el audio del sistema solo se puede capturar en macOS".into())
        }
    }
}

pub use anillo::Anillo;
pub use remuestreo::Remuestreador;

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
