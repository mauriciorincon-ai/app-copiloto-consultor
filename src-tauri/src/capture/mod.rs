//! Captura de audio y pantalla — **MÓDULO PROTEGIDO** (regla del efímero verificable).
//!
//! Todo lo que vive bajo `capture/` es efímero por definición: ring buffers en RAM que mueren
//! al cerrar la sesión. Aquí **no se abre disco ni red**, y no es una costumbre: `pnpm
//! verify:ephemeral` barre este directorio buscando API de archivo y de socket, y la CI se pone
//! roja si aparece una. Si algún día un caso legítimo necesita escribir, exige un ADR citado en
//! la misma línea (`verify-ephemeral:allow`), no un atajo.
//!
//! Se llena en la fase 3 del sprint 001 (dos pistas: micrófono y audio del sistema).

/// Marca de quién habló. **Por PISTA, jamás por biometría** (regla de cero huellas de voz):
/// el micrófono es el consultor y el audio del sistema es la contraparte, y eso se sabe por
/// el origen de la muestra, no por analizar la voz.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
