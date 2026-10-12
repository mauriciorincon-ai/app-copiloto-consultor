//! **El WAV de la voz de Windows, leído en memoria** (sprint 007, ADR 022). Sigue siendo `habla/`: módulo protegido,
//! ni disco ni red.
//!
//! El sintetizador de Windows (`SpeechSynthesizer`) entrega la frase dicha como un flujo WAV **en memoria**. Aquí se
//! lee ese flujo —cabecera RIFF, trozo `fmt ` y trozo `data`— y sale en flotantes mono, listo para sonar. Nada se
//! escribe en ningún sitio: un WAV con la ficha leída en voz alta en el disco sería una grabación con otro nombre.
//! Pura, y por eso se prueba en el Mac.

/// El audio de una frase, en mono y en flotantes entre -1 y 1.
#[derive(Debug, PartialEq)]
pub struct Frase {
    pub hz: u32,
    pub muestras: Vec<f32>,
}

/// Lee un WAV PCM de 16 bits o flotante de 32 bits, de uno o más canales (se promedian a mono). Devuelve el motivo
/// si no lo entiende, sin entrar en pánico con un WAV raro.
pub fn leer(bytes: &[u8]) -> Result<Frase, String> {
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("no es un WAV".into());
    }
    let mut formato: Option<(u16, u16, u32, u16)> = None;
    let mut datos: Option<&[u8]> = None;
    let mut i = 12;
    while i + 8 <= bytes.len() {
        let id = &bytes[i..i + 4];
        let largo = u32::from_le_bytes([bytes[i + 4], bytes[i + 5], bytes[i + 6], bytes[i + 7]]) as usize;
        let cuerpo = i + 8;
        let fin = cuerpo.saturating_add(largo).min(bytes.len());
        match id {
            b"fmt " if fin - cuerpo >= 16 => {
                let b = &bytes[cuerpo..fin];
                let mut tipo = u16::from_le_bytes([b[0], b[1]]);
                let canales = u16::from_le_bytes([b[2], b[3]]);
                let hz = u32::from_le_bytes([b[4], b[5], b[6], b[7]]);
                let bits = u16::from_le_bytes([b[14], b[15]]);
                // WAVE_FORMAT_EXTENSIBLE: el tipo de verdad son los dos primeros bytes del subformato.
                if tipo == 0xFFFE && b.len() >= 26 {
                    tipo = u16::from_le_bytes([b[24], b[25]]);
                }
                formato = Some((tipo, canales, hz, bits));
            }
            b"data" => datos = Some(&bytes[cuerpo..fin]),
            _ => {}
        }
        // Los trozos van alineados a dos bytes.
        i = cuerpo.saturating_add(largo).saturating_add(largo % 2);
    }
    let (tipo, canales, hz, bits) = formato.ok_or("el WAV no trae su formato")?;
    let datos = datos.ok_or("el WAV no trae audio")?;
    if canales == 0 || hz == 0 {
        return Err("el WAV dice cero canales o cero hercios".into());
    }
    let sueltas: Vec<f32> = match (tipo, bits) {
        (1, 16) => datos.as_chunks::<2>().0.iter().map(|m| f32::from(i16::from_le_bytes(*m)) / 32_768.0).collect(),
        (3, 32) => datos.as_chunks::<4>().0.iter().map(|m| f32::from_le_bytes(*m)).collect(),
        _ => return Err(format!("formato WAV {tipo} de {bits} bits, que la voz no sabe leer")),
    };
    let c = usize::from(canales);
    let muestras = sueltas.chunks(c).map(|m| m.iter().sum::<f32>() / m.len() as f32).collect();
    Ok(Frase { hz, muestras })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn wav(tipo: u16, canales: u16, hz: u32, bits: u16, datos: &[u8], extra: &[u8]) -> Vec<u8> {
        let mut fmt = Vec::new();
        fmt.extend_from_slice(&tipo.to_le_bytes());
        fmt.extend_from_slice(&canales.to_le_bytes());
        fmt.extend_from_slice(&hz.to_le_bytes());
        fmt.extend_from_slice(&(hz * u32::from(canales) * u32::from(bits) / 8).to_le_bytes());
        fmt.extend_from_slice(&(canales * bits / 8).to_le_bytes());
        fmt.extend_from_slice(&bits.to_le_bytes());
        fmt.extend_from_slice(extra);
        let mut w = b"RIFF\0\0\0\0WAVE".to_vec();
        w.extend_from_slice(b"fmt ");
        w.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
        w.extend_from_slice(&fmt);
        // Un trozo que el lector tiene que saltarse, de largo impar (con su relleno).
        w.extend_from_slice(b"LIST\x03\0\0\0abc\0");
        w.extend_from_slice(b"data");
        w.extend_from_slice(&(datos.len() as u32).to_le_bytes());
        w.extend_from_slice(datos);
        w
    }

    /// El caso de la voz de Windows: mono, 16 bits.
    #[test]
    fn lee_el_pcm_de_16_bits_mono() {
        let datos: Vec<u8> = [16_384i16, -16_384, 0].iter().flat_map(|m| m.to_le_bytes()).collect();
        let f = leer(&wav(1, 1, 22_050, 16, &datos, &[])).unwrap();
        assert_eq!(f.hz, 22_050);
        assert_eq!(f.muestras, vec![0.5, -0.5, 0.0]);
    }

    /// Estéreo se promedia a mono, y el flotante de 32 bits también se lee, aunque venga como «extensible».
    #[test]
    fn promedia_los_canales_y_entiende_el_extensible() {
        let datos: Vec<u8> = [0.25f32, 0.75, -1.0, 1.0].iter().flat_map(|m| m.to_le_bytes()).collect();
        let mut extra = vec![22, 0, 32, 0, 3, 0, 0, 0];
        extra.extend_from_slice(&[3, 0]); // el subformato empieza por el tipo: 3 = flotante
        extra.extend_from_slice(&[0; 14]);
        let f = leer(&wav(0xFFFE, 2, 48_000, 32, &datos, &extra)).unwrap();
        assert_eq!(f.hz, 48_000);
        assert_eq!(f.muestras, vec![0.5, 0.0]);
    }

    /// Lo que no es un WAV, o es uno que no sabe leer, se dice; no revienta.
    #[test]
    fn un_wav_raro_se_dice_sin_reventar() {
        assert!(leer(b"no es nada").is_err());
        assert!(leer(&wav(1, 1, 16_000, 8, &[1, 2, 3], &[])).unwrap_err().contains("8 bits"));
        assert!(leer(&wav(1, 0, 16_000, 16, &[], &[])).is_err());
        let mut sin_datos = wav(1, 1, 16_000, 16, &[], &[]);
        sin_datos.truncate(sin_datos.len() - 8);
        assert!(leer(&sin_datos).is_err());
    }
}
