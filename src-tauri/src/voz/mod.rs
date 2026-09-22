//! Saber **cuándo alguien habla y cuándo ha terminado de hablar** — sin salir de la memoria y sin
//! mirar quién es.
//!
//! Este módulo es la mitad determinista del sprint que más se parece a magia y menos lo es. No
//! reconoce palabras (eso es `stt`), no sabe de quién es la voz (eso lo sabe `capture` por el
//! origen de la muestra, jamás por la señal) y no decide qué enseñar (eso será el disparador de
//! la fase 4). Hace una sola cosa: partir un flujo continuo de audio en **turnos**, y avisar en
//! cuanto uno se cierra.
//!
//! **Por qué es la primera pieza y no la última.** Todo el presupuesto del sprint —≤4 s desde que
//! el cliente deja de hablar hasta que el consultor ve su ficha— se mide desde un instante que
//! hay que *decidir*: el fin de turno. Si se decide tarde, los cuatro segundos empiezan tarde y
//! ya no hay nada que optimizar después. Si se decide pronto, la app interrumpe a la persona a
//! mitad de frase. El objetivo de la orden son **160–400 ms** de silencio antes de dar el turno
//! por cerrado, y ese número es de producto, no de implementación.
//!
//! Cero red, cero disco, cero modelo: aritmética sobre las muestras que ya están en el anillo.

pub mod eco;
pub mod turno;
pub mod vad;

pub use eco::Tramo;
pub use turno::{Suceso, Turnos};
pub use vad::{Detector, PorEnergia};

/// Cuánto audio mira el detector de una vez. 20 ms es el tamaño de libro para voz: lo bastante
/// corto para que el fin de turno tenga resolución fina, lo bastante largo para que la energía de
/// una vocal no se confunda con el ruido de fondo de un instante.
pub const MARCO_MS: usize = 20;

/// Cuántas muestras son esos 20 ms a la frecuencia de la app.
pub const MARCO: usize = (crate::capture::anillo::HZ as usize * MARCO_MS) / 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn el_marco_son_veinte_milisegundos_a_la_frecuencia_de_la_app() {
        assert_eq!(MARCO, 320);
    }
}
