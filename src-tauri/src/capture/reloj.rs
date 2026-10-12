//! **El reloj de una pista que puede callarse del todo** (sprint 007, ADR 022). Sigue siendo `capture/`: módulo
//! protegido, ni disco ni red.
//!
//! El *loopback* de WASAPI —el audio del sistema en Windows, la pista del cliente— **no entrega nada mientras nada
//! suena**. El tap de macOS entrega ceros. La diferencia importa porque la escucha fecha cada turno por la cuenta de
//! muestras de su anillo: una pista que no recibe nada durante diez minutos tiene un reloj diez minutos atrasado, y
//! el eco —que compara cuándo habló cada pista— compararía dos relojes distintos. Este reloj rellena con silencio
//! el hueco que mide el reloj de pared, y nunca se adelanta: deja una holgura para el audio que todavía viene en
//! camino.
//!
//! **Lo que se escribe son ceros**: el silencio no es audio de nadie. Y el anillo sigue sin crecer: tras dormir el PC
//! una hora, entran sus treinta segundos de ceros y el reloj cuenta la hora entera (`Anillo::silencio`).

use super::anillo::Anillo;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Lo que el reloj deja sin rellenar: el audio que WASAPI todavía no entregó pero ya sonó. Dos o tres periodos del
/// motor de audio, con margen; por debajo del fin de turno (320 ms) para que un silencio de verdad cierre turnos.
pub const HOLGURA: Duration = Duration::from_millis(200);

/// Cada cuánto se mira si falta silencio cuando no llega nada. La mitad de la holgura: el reloj no se atrasa más
/// de lo que la holgura ya admite.
pub const LATIDO: Duration = Duration::from_millis(100);

/// **Cuántas muestras de silencio faltan** para que una pista que ya tiene `escritas` y va a recibir `llegan` no se
/// atrase respecto al reloj de pared: lo debido a `transcurrido` menos la holgura, nunca negativo. Pura.
pub fn silencio_que_falta(escritas: u64, llegan: u64, transcurrido: Duration, hz: u32, holgura: Duration) -> u64 {
    let debidas = transcurrido.saturating_sub(holgura).as_micros() * u128::from(hz) / 1_000_000;
    u64::try_from(debidas).unwrap_or(u64::MAX).saturating_sub(escritas.saturating_add(llegan))
}

/// Un anillo con su reloj de pared. Quien recibe audio escribe por aquí, y un latido escribe silencio cuando no
/// llega nada.
pub struct Reloj {
    anillo: Arc<Mutex<Anillo>>,
    cuenta: Mutex<Cuenta>,
}

struct Cuenta {
    /// El instante en que el anillo tenía cero muestras.
    inicio: Instant,
    /// Cuántas muestras escribió este reloj desde `inicio`. Si el anillo tiene otras, alguien lo vació.
    escritas: u64,
}

impl Reloj {
    pub fn nuevo(anillo: Arc<Mutex<Anillo>>) -> Self {
        Self::desde(anillo, Instant::now())
    }

    fn desde(anillo: Arc<Mutex<Anillo>>, inicio: Instant) -> Self {
        Self { anillo, cuenta: Mutex::new(Cuenta { inicio, escritas: 0 }) }
    }

    /// Escribe `bloque` —lo que acaba de sonar— después del silencio que falte hasta `ahora`. Con un bloque vacío
    /// es el latido: solo silencio, si falta.
    ///
    /// **El corte vacía el anillo** (`Anillo::vaciar` pone su reloj a cero). Si el anillo no tiene lo que este reloj
    /// escribió, el reloj vuelve a empezar con él en vez de rellenar de golpe todo lo que pasó antes del corte.
    pub fn escribir(&self, bloque: &[f32], ahora: Instant) {
        let Ok(mut c) = self.cuenta.lock() else { return };
        let Ok(mut a) = self.anillo.lock() else { return };
        if a.totales() != c.escritas {
            c.inicio = ahora;
            c.escritas = a.totales();
        }
        let falta = silencio_que_falta(
            c.escritas,
            bloque.len() as u64,
            ahora.saturating_duration_since(c.inicio),
            a.hz(),
            HOLGURA,
        );
        if falta > 0 {
            a.silencio(falta);
        }
        a.escribir(bloque);
        c.escritas = a.totales();
    }

    /// El latido: el silencio que falte hasta `ahora`, si falta.
    pub fn al_dia(&self, ahora: Instant) {
        self.escribir(&[], ahora);
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    const HZ: u32 = 16_000;

    fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// **Nunca se adelanta**: dentro de la holgura no falta nada, y lo que ya llegó cuenta.
    #[test]
    fn dentro_de_la_holgura_no_falta_silencio() {
        assert_eq!(silencio_que_falta(0, 0, ms(150), HZ, HOLGURA), 0);
        assert_eq!(silencio_que_falta(0, 0, ms(200), HZ, HOLGURA), 0);
        assert_eq!(silencio_que_falta(0, 0, ms(1_200), HZ, HOLGURA), 16_000, "un segundo pasada la holgura");
        assert_eq!(silencio_que_falta(10_000, 6_000, ms(1_200), HZ, HOLGURA), 0, "lo que llega cuenta");
        assert_eq!(silencio_que_falta(20_000, 0, ms(1_200), HZ, HOLGURA), 0, "adelantado no resta");
    }

    /// **Una pista callada sigue la hora**: diez segundos sin audio son diez segundos de reloj (menos la holgura), y
    /// el audio que llega después cae en su sitio, no diez segundos antes.
    #[test]
    fn una_pista_callada_sigue_la_hora() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let t0 = Instant::now();
        let reloj = Reloj::desde(anillo.clone(), t0);
        reloj.escribir(&[0.5; 160], t0 + ms(10));
        assert_eq!(anillo.lock().unwrap().totales(), 160, "al principio no hay hueco");
        reloj.al_dia(t0 + ms(10_000));
        assert_eq!(anillo.lock().unwrap().totales(), 9_800 * 16, "diez segundos menos la holgura");
        reloj.escribir(&[0.5; 160], t0 + ms(10_010));
        let a = anillo.lock().unwrap();
        assert_eq!(a.totales(), 9_810 * 16);
        assert_eq!(a.ultimas(160), vec![0.5; 160], "el audio nuevo queda al final, no se pisa con ceros");
    }

    /// **Tras dormir el PC una hora**, el reloj cuenta la hora y el anillo no crece.
    #[test]
    fn una_hora_dormido_no_hace_crecer_el_anillo() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let t0 = Instant::now();
        let reloj = Reloj::desde(anillo.clone(), t0);
        reloj.al_dia(t0 + Duration::from_secs(3_600));
        let a = anillo.lock().unwrap();
        assert_eq!(a.totales(), (3_600_000 - 200) * 16);
        assert_eq!(a.ocupadas(), a.capacidad());
    }

    /// **El corte vacía el anillo y el reloj vuelve a empezar con él**, sin rellenar lo que pasó antes del corte.
    #[test]
    fn tras_el_corte_el_reloj_vuelve_a_empezar() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let t0 = Instant::now();
        let reloj = Reloj::desde(anillo.clone(), t0);
        reloj.al_dia(t0 + ms(5_000));
        anillo.lock().unwrap().vaciar();
        reloj.escribir(&[0.5; 160], t0 + ms(5_100));
        assert_eq!(anillo.lock().unwrap().totales(), 160, "nada del tiempo de antes del corte");
        reloj.al_dia(t0 + ms(6_100));
        assert_eq!(anillo.lock().unwrap().totales(), 800 * 16, "y desde ahí, la hora");
    }
}
