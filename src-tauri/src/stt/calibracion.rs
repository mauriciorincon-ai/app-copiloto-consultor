//! **La calibración de la transcripción** (sprint 006, ADR 022 §5): diez segundos sintéticos del kit para medir cuánto
//! tarda el motor en ESTE equipo. Sigue siendo `stt/`: el recorte entra al compilar, sin disco ni red.
//!
//! En el Mac la latencia del motor de Apple se midió una vez (ADR 006: 84 ms por seis segundos) y vale para todos los
//! Mac con macOS 26. En Windows no: el mismo modelo tarda lo que dé el procesador de cada PC, y por eso se mide en el
//! PC del usuario, con un audio que no es de nadie.

/// El recorte: los primeros 10 s de `docs/kit-de-prueba/audio/sala-es.wav` (`data/calibracion/LEEME.md`). **Solo entra
/// en el binario de Windows** (y en el de las pruebas): son 320 KB que el Mac no usa, porque su motor se midió una vez
/// (sprint 008, B9 del S6).
#[cfg(any(windows, test))]
const RECORTE: &[u8] = include_bytes!("../../../data/calibracion/sala-es-10s.wav");

/// **La pista del cliente** (sprint 010): la pregunta del kit, `docs/kit-de-prueba/audio/pregunta-es.wav` (unos 6 s, voz
/// sintética, cero datos reales). Con ella la calibración mide también lo que tarda el turno del cliente, la otra pista
/// de una videollamada, y Honestidad enseña las dos. Como el recorte, solo entra en el binario de Windows.
#[cfg(any(windows, test))]
const CLIENTE: &[u8] = include_bytes!("../../../docs/kit-de-prueba/audio/pregunta-es.wav");

/// El idioma del recorte (y de la pista del cliente).
pub const IDIOMA: &str = "es-ES";

/// **El techo**: del fin de turno a la ficha en ≤ 4 s (estándar de performance de escritorio). Si la transcripción sola
/// ya se lo come, el modelo es demasiado grande para este equipo.
pub const TECHO_MS: u32 = 4_000;

/// ¿Esta latencia se pasa del techo?
pub fn pasa_del_techo(ms: u32) -> bool {
    ms > TECHO_MS
}

/// Las muestras del recorte en flotante, mono a 16 kHz. Vacío solo si el recorte dejara de ser un WAV PCM de 16
/// bits a 16 kHz, y eso lo caza la prueba de abajo antes de que llegue a un PC.
#[cfg(any(windows, test))]
pub fn muestras() -> Vec<f32> {
    leer_wav(RECORTE).unwrap_or_default()
}

/// Las muestras de la pista del cliente, con el mismo formato y la misma regla que [`muestras`].
#[cfg(any(windows, test))]
pub fn muestras_del_cliente() -> Vec<f32> {
    leer_wav(CLIENTE).unwrap_or_default()
}

/// Un WAV PCM mono de 16 bits a 16 kHz, en flotantes entre -1 y 1. Cualquier otra cosa, `None`: la calibración no
/// adivina formatos.
#[cfg(any(windows, test))]
fn leer_wav(bytes: &[u8]) -> Option<Vec<f32>> {
    if bytes.get(0..4)? != b"RIFF" || bytes.get(8..12)? != b"WAVE" {
        return None;
    }
    let mut i = 12;
    let mut formato_valido = false;
    while i + 8 <= bytes.len() {
        let id = &bytes[i..i + 4];
        let largo = u32::from_le_bytes(bytes[i + 4..i + 8].try_into().ok()?) as usize;
        let cuerpo = bytes.get(i + 8..i + 8 + largo)?;
        match id {
            b"fmt " => {
                let u16_en = |k: usize| cuerpo.get(k..k + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
                let hz = cuerpo.get(4..8).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
                formato_valido = u16_en(0) == Some(1)
                    && u16_en(2) == Some(1)
                    && hz == Some(crate::capture::anillo::HZ)
                    && u16_en(14) == Some(16);
            }
            b"data" if formato_valido => {
                return Some(cuerpo.as_chunks::<2>().0.iter().map(|b| f32::from(i16::from_le_bytes(*b)) / 32_768.0).collect());
            }
            _ => {}
        }
        i += 8 + largo + (largo & 1);
    }
    None
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **El recorte no entra en el binario del Mac** (sprint 008, B9 del S6): `RECORTE` y quien lo lee llevan
    /// `cfg(any(windows, test))`, así que fuera de Windows solo existe en las pruebas.
    #[test]
    fn el_recorte_no_entra_en_el_binario_del_mac() {
        let fuente = include_str!("calibracion.rs");
        for pieza in ["const RECORTE", "const CLIENTE", "pub fn muestras()", "pub fn muestras_del_cliente", "fn leer_wav"] {
            let i = fuente.find(pieza).unwrap_or_else(|| panic!("falta {pieza}"));
            let antes = &fuente[..i];
            let atributo = antes.lines().rev().find(|l| l.trim_start().starts_with("#[")).unwrap_or("");
            assert_eq!(atributo.trim(), "#[cfg(any(windows, test))]", "{pieza} entraría en el binario de macOS");
        }
    }

    /// El recorte son diez segundos exactos a 16 kHz, y suenan (no es silencio).
    #[test]
    fn el_recorte_son_diez_segundos_que_suenan() {
        let m = muestras();
        assert_eq!(m.len(), 160_000, "el recorte no mide 10 s a 16 kHz");
        let pico = m.iter().fold(0.0f32, |a, x| a.max(x.abs()));
        assert!(pico > 0.1, "el recorte es casi silencio (pico {pico})");
    }

    /// La pista del cliente se lee (la cabecera del kit trae un bloque de relleno antes de los datos) y son unos 6 s
    /// que suenan: Honestidad dice «6 s» y `windows-honestidad-y-manual` ata esa cifra a este archivo.
    #[test]
    fn la_pista_del_cliente_son_unos_seis_segundos_que_suenan() {
        let m = muestras_del_cliente();
        let segundos = m.len() as f32 / crate::capture::anillo::HZ as f32;
        assert!((5.5..6.5).contains(&segundos), "la pista del cliente mide {segundos:.2} s, no unos 6");
        let pico = m.iter().fold(0.0f32, |a, x| a.max(x.abs()));
        assert!(pico > 0.1, "la pista del cliente es casi silencio (pico {pico})");
    }

    /// **El recorte es el principio del kit**, muestra a muestra: nada de audio que no venga del kit sintético.
    #[test]
    fn el_recorte_es_el_principio_del_kit() {
        let kit = leer_wav(include_bytes!("../../../docs/kit-de-prueba/audio/sala-es.wav")).expect("el kit es un WAV");
        assert_eq!(muestras()[..], kit[..160_000]);
    }

    /// Lo que no es un WAV PCM mono de 16 bits a 16 kHz no se lee.
    #[test]
    fn lo_que_no_es_el_formato_no_se_lee() {
        assert_eq!(leer_wav(b"no es un wav"), None);
        let mut otro = RECORTE.to_vec();
        otro[22] = 2; // dos canales
        assert_eq!(leer_wav(&otro), None);
    }

    /// El techo es el del estándar: 4 s.
    #[test]
    fn el_techo_es_de_cuatro_segundos() {
        assert!(!pasa_del_techo(4_000));
        assert!(pasa_del_techo(4_001));
    }
}
