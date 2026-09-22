//! Bajar el audio a 16 kHz **sin inventarse voz por el camino**.
//!
//! macOS entrega lo que le da la gana: el tap del sistema mide 48 kHz, el micrófono lo que diga
//! el dispositivo (48 kHz casi siempre, 44,1 kHz con algunos interfaces). Dentro de la app todo
//! vive a 16 kHz —lo que quieren el detector de voz y el transcriptor—, así que hay una conversión
//! obligatoria en la puerta de entrada.
//!
//! **Y esa conversión tiene un fallo que no avisa.** Quedarse con una muestra de cada tres es
//! lo evidente, cuesta tres líneas y funciona… hasta que entra un sonido agudo. Todo lo que pase
//! de 8 kHz (la mitad de 16) no desaparece al decimar: **se dobla hacia abajo y reaparece como un
//! tono grave que nunca existió**. Un siseo de 18 kHz —una silla que chirría, una consonante
//! fricativa, el ventilador de un portátil— vuelve convertido en 2 kHz a todo volumen, justo en
//! mitad de la banda de la voz humana. El detector de voz lo oiría como alguien hablando, el fin
//! de turno se dispararía sobre silencio y la ficha saldría sin que nadie hubiera preguntado
//! nada.
//!
//! Por eso el remuestreador filtra antes de decimar. El test `un_agudo_no_se_convierte_en_voz`
//! es el que lo cobra: quitar el filtro lo deja en rojo y deja verdes a todos los demás.

/// Cuántos coeficientes tiene el filtro. Impar a propósito: así hay un centro exacto y el retardo
/// que introduce es un número entero de muestras, no medio.
const TOMAS: usize = 63;

/// Dónde corta el filtro, en fracción de la frecuencia de salida. 0,45 deja la voz entera (la
/// inteligibilidad vive por debajo de 4 kHz) y se apoya en el margen que queda hasta 0,5 para
/// caer de verdad antes de Nyquist.
const CORTE: f32 = 0.45;

/// Convierte un flujo continuo de audio a otra frecuencia. **Tiene estado**: se llama una vez por
/// bloque que entrega macOS y recuerda lo justo para que dos bloques seguidos suenen como uno
/// solo. Un remuestreador sin memoria deja un chasquido en cada costura, y a 100 bloques por
/// segundo eso es un zumbido.
pub struct Remuestreador {
    hz_entrada: u32,
    hz_salida: u32,
    coeficientes: Vec<f32>,
    /// Las muestras de entrada que el filtro todavía necesita ver para calcular las siguientes.
    cola: Vec<f32>,
    /// Dónde está el lector dentro del flujo filtrado, en muestras y con decimales.
    posicion: f64,
}

impl Remuestreador {
    pub fn nuevo(hz_entrada: u32, hz_salida: u32) -> Self {
        Self {
            hz_entrada,
            hz_salida,
            coeficientes: filtro(hz_entrada, hz_salida),
            cola: Vec::new(),
            posicion: 0.0,
        }
    }

    /// ¿Hay algo que convertir? Si las dos frecuencias coinciden, el remuestreador se aparta.
    pub fn transparente(&self) -> bool {
        self.hz_entrada == self.hz_salida
    }

    pub fn convertir(&mut self, entrada: &[f32]) -> Vec<f32> {
        if self.transparente() {
            return entrada.to_vec();
        }
        let mut completo = std::mem::take(&mut self.cola);
        completo.extend_from_slice(entrada);
        if completo.len() < TOMAS {
            self.cola = completo;
            return Vec::new();
        }

        // Filtrado a la frecuencia de ENTRADA. `filtrado[i]` es la muestra `i` del flujo original
        // ya sin agudos; de ahí se leerá con paso fraccionario.
        let utiles = completo.len() - TOMAS + 1;
        let mut filtrado = Vec::with_capacity(utiles);
        for i in 0..utiles {
            let mut suma = 0.0;
            for (k, c) in self.coeficientes.iter().enumerate() {
                suma += c * completo[i + k];
            }
            filtrado.push(suma);
        }

        let paso = self.hz_entrada as f64 / self.hz_salida as f64;
        let mut salida = Vec::new();
        while self.posicion + 1.0 < filtrado.len() as f64 {
            let entero = self.posicion.floor() as usize;
            let resto = (self.posicion - entero as f64) as f32;
            salida.push(filtrado[entero] * (1.0 - resto) + filtrado[entero + 1] * resto);
            self.posicion += paso;
        }

        // Lo consumido se tira; lo que el filtro aún necesita ver se guarda para el bloque
        // siguiente. Sin esto, cada bloque empezaría con el filtro a cero y la costura sonaría.
        let consumidas = self.posicion.floor() as usize;
        self.posicion -= consumidas as f64;
        self.cola = completo.split_off(consumidas.min(completo.len()));
        salida
    }
}

/// Un paso bajo de seno cardinal con ventana de Hamming. Es el filtro de libro: el que se escribe
/// cuando no hay una razón para escribir otro.
fn filtro(hz_entrada: u32, hz_salida: u32) -> Vec<f32> {
    // La frecuencia de corte se expresa en ciclos por muestra DE ENTRADA, que es donde se aplica.
    let fc = (CORTE * hz_salida.min(hz_entrada) as f32) / hz_entrada as f32;
    let centro = (TOMAS - 1) as f32 / 2.0;
    let mut h: Vec<f32> = (0..TOMAS)
        .map(|n| {
            let x = n as f32 - centro;
            let sinc = if x.abs() < 1e-6 {
                2.0 * fc
            } else {
                (2.0 * std::f32::consts::PI * fc * x).sin() / (std::f32::consts::PI * x)
            };
            let ventana = 0.54 - 0.46 * (2.0 * std::f32::consts::PI * n as f32 / (TOMAS - 1) as f32).cos();
            sinc * ventana
        })
        .collect();
    // Normalizar a ganancia 1 en continua: sin esto el volumen cambiaría al remuestrear, y el
    // detector de voz mide energía.
    let suma: f32 = h.iter().sum();
    if suma.abs() > 1e-9 {
        for c in &mut h {
            *c /= suma;
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tono(hz: f32, muestreo: u32, muestras: usize) -> Vec<f32> {
        (0..muestras)
            .map(|n| (2.0 * std::f32::consts::PI * hz * n as f32 / muestreo as f32).sin())
            .collect()
    }

    fn rms(x: &[f32]) -> f32 {
        if x.is_empty() {
            return 0.0;
        }
        (x.iter().map(|v| v * v).sum::<f32>() / x.len() as f32).sqrt()
    }

    #[test]
    fn la_voz_pasa_entera() {
        let mut r = Remuestreador::nuevo(48_000, 16_000);
        let dentro = tono(1_000.0, 48_000, 48_000);
        let fuera = r.convertir(&dentro);
        assert!(
            (fuera.len() as i64 - 16_000).abs() < 100,
            "salieron {} muestras, se esperaban ~16000",
            fuera.len()
        );
        // Un seno tiene RMS 1/√2 ≈ 0,707. Se admite un 5 % de pérdida por el filtro.
        assert!(rms(&fuera) > 0.67, "el tono de 1 kHz salió a {:.3} de RMS", rms(&fuera));
    }

    /// **El gate de este módulo.** Un agudo por encima de Nyquist tiene que MORIR, no bajar de
    /// octava. Se ve en rojo sustituyendo `convertir` por una decimación de una de cada tres:
    /// el tono de 18 kHz reaparece a 2 kHz con RMS ~0,7 y este test cae solo.
    #[test]
    fn un_agudo_no_se_convierte_en_voz() {
        let mut r = Remuestreador::nuevo(48_000, 16_000);
        let dentro = tono(18_000.0, 48_000, 48_000);
        let fuera = r.convertir(&dentro);
        assert!(
            rms(&fuera) < 0.05,
            "un siseo de 18 kHz salió a {:.3} de RMS: se dobló dentro de la banda de la voz y el \
             detector lo va a oír como alguien hablando",
            rms(&fuera)
        );
    }

    /// Dos bloques seguidos tienen que sonar igual que uno entero. Si el remuestreador olvidara
    /// su estado entre llamadas, la costura metería un chasquido cada 10 ms.
    #[test]
    fn dos_bloques_seguidos_suenan_como_uno_solo() {
        let dentro = tono(1_000.0, 48_000, 24_000);
        let mut entero = Remuestreador::nuevo(48_000, 16_000);
        let de_una = entero.convertir(&dentro);

        let mut troceado = Remuestreador::nuevo(48_000, 16_000);
        let mut de_a_trozos = Vec::new();
        for bloque in dentro.chunks(512) {
            de_a_trozos.extend(troceado.convertir(bloque));
        }

        assert_eq!(de_una.len(), de_a_trozos.len(), "el troceado cambió cuántas muestras salen");
        let peor = de_una
            .iter()
            .zip(&de_a_trozos)
            .map(|(a, b)| (a - b).abs())
            .fold(0.0f32, f32::max);
        assert!(peor < 1e-4, "la costura entre bloques se oye: diferencia máxima {peor}");
    }

    #[test]
    fn si_no_hay_nada_que_convertir_no_se_toca_el_audio() {
        let mut r = Remuestreador::nuevo(16_000, 16_000);
        assert!(r.transparente());
        let dentro = tono(1_000.0, 16_000, 1_000);
        assert_eq!(r.convertir(&dentro), dentro);
    }

    #[test]
    fn tambien_convierte_desde_44100() {
        let mut r = Remuestreador::nuevo(44_100, 16_000);
        let fuera = r.convertir(&tono(1_000.0, 44_100, 44_100));
        assert!((fuera.len() as i64 - 16_000).abs() < 100, "salieron {} muestras", fuera.len());
        assert!(rms(&fuera) > 0.67);
    }

    #[test]
    fn un_bloque_demasiado_corto_se_guarda_para_el_siguiente() {
        let mut r = Remuestreador::nuevo(48_000, 16_000);
        assert!(r.convertir(&[0.5; 10]).is_empty(), "con 10 muestras no hay filtro que aplicar");
        assert!(!r.convertir(&[0.5; 4_000]).is_empty());
    }
}
