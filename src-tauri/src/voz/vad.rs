//! ¿Hay voz en estos 20 ms?
//!
//! **Código primero, en su forma literal.** La orden nombra a Silero —una red neuronal de 2 MB
//! que se descarga aparte— y la tentación es empezar por ahí. Pero la regla de esta casa dice que
//! la funcionalidad interna se resuelve primero con programación, y que el modelo tiene que
//! *ganarse* el puesto con una medición. Así que el puesto lo ocupa esto: un detector por energía
//! con suelo de ruido adaptativo, que cabe en una pantalla, no descarga nada, corre en
//! microsegundos y se puede leer entero. Si el kit de la fase 5 demuestra que no alcanza, el ADR
//! del STT y del VAD cambia el motor **con el número delante**; hasta entonces, cambiar de motor
//! sería preferencia, no ingeniería.
//!
//! **El problema real que resuelve el suelo adaptativo.** Un umbral fijo funciona en el escritorio
//! del que lo programó y en ningún otro sitio: en una cafetería todo es voz, en un estudio
//! insonorizado nada lo es. Y el ruido de una videollamada no es constante —el aire acondicionado
//! arranca, el portátil sube el ventilador, alguien mueve una silla—. Por eso el detector no
//! compara contra una constante sino contra **su propia estimación del silencio de esta sala**.
//!
//! **Y el detalle que costó un test en rojo.** La primera versión actualizaba el suelo en TODOS
//! los marcos, subiendo despacio (una parte entre quinientas). Parecía prudente. El test de la
//! frase larga lo tumbó en el segundo 3,7: subir despacio sigue siendo subir, y quince segundos
//! seguidos de voz arrastran el suelo hasta que el hablante queda por debajo de su propio umbral
//! — la app se habría quedado muda justo con el cliente que más habla. La regla correcta es otra:
//! **el suelo solo se mueve cuando el detector NO está oyendo voz.** Mientras alguien habla, la
//! medida del silencio de la sala se congela, porque no hay silencio que medir.
//!
//! Eso abre la puerta a quedarse atascado: si un ruido nuevo y constante arranca mientras alguien
//! habla, el detector lo tomará por voz y, como cree oír voz, nunca volverá a medir. Contra eso
//! está [`PACIENCIA_MS`]: nadie habla treinta segundos sin una sola pausa de 20 ms. Si el detector
//! lo cree, es que se equivoca, y lo honesto es volver a medir la sala desde cero.
//!
//! **Lo que se descartó y por qué.** La tasa de cruces por cero es la otra mitad del VAD clásico,
//! y se probó a usarla para rechazar siseos. Corta las fricativas: la `s` del plural español y la
//! `f` inglesa son ruido de banda ancha con energía baja, exactamente lo que el filtro tiraría. Un
//! fin de turno que se come la última sílaba es peor que uno que aguanta 20 ms de más.

/// Lo que cualquier detector de voz tiene que saber hacer. El `trait` existe para que el motor sea
/// **sustituible con una medición delante** y no por gusto: cuando el kit diga si Silero aporta,
/// entra por aquí sin tocar el fin de turno ni la captura.
pub trait Detector {
    /// ¿Hay voz en este marco? El marco viene a [`crate::voz::MARCO`] muestras.
    fn hay_voz(&mut self, marco: &[f32]) -> bool;

    /// Vuelve al estado de recién nacido. El kill-switch lo llama: lo aprendido sobre el ruido de
    /// la sala también es información de la reunión.
    fn olvidar(&mut self);
}

/// Por debajo de esto no hay voz aunque el suelo esté aún más abajo. Es el que salva a un
/// micrófono en una habitación insonorizada, donde el suelo de ruido tiende a cero y cualquier
/// cosa lo multiplica por diez.
const PISO_ABSOLUTO: f32 = 0.004;

/// Cuántas veces el suelo tiene que superar la energía para que cuente como voz.
const SOBRE_EL_SUELO: f32 = 3.0;

/// Con qué rapidez el suelo BAJA cuando la sala se calla. Alto = se adapta enseguida, que es lo
/// que hace falta para no perderse el arranque del turno siguiente.
const BAJA: f32 = 0.20;

/// Con qué rapidez el suelo SUBE cuando el ruido de fondo crece. Despacio: un suelo que sube de
/// golpe ensordece a la app ante cualquier portazo.
const SUBE: f32 = 0.02;

/// Con qué rapidez se aprende la sala en los primeros milisegundos. Aquí sí conviene converger
/// rápido en las dos direcciones: es una medición, no un seguimiento.
const APRENDE: f32 = 0.25;

/// Cuánto aguanta el detector afirmando «voz» sin una sola pausa antes de sospechar de sí mismo.
/// Nadie habla medio minuto sin un hueco de 20 ms; si el detector lo cree, lo que oye es un ruido
/// que empezó mientras alguien hablaba y que congeló la medida del silencio. Vuelve a medir.
pub const PACIENCIA_MS: usize = 30_000;

/// El detector de esta app.
pub struct PorEnergia {
    suelo: f32,
    /// Cuántos marcos ha visto. Los primeros se usan para conocer la sala y se dan por silencio:
    /// arrancar declarando voz por no tener referencia es el fallo más fácil de este diseño.
    vistos: usize,
    /// Cuántos marcos seguidos lleva diciendo «voz». Alimenta a [`PACIENCIA_MS`].
    seguidas: usize,
}

/// Cuántos marcos dura el aprendizaje inicial: medio segundo de escucha antes de afirmar nada.
///
/// **Lo que este medio segundo cuesta, dicho aquí y probado abajo:** si la app arranca con alguien
/// ya hablando, aprende que esa voz es el silencio de la sala y se pierde ese turno. Se recupera
/// sola en la primera pausa (el suelo baja deprisa), y se prefiere ese fallo al contrario —una
/// app que no oye un turno molesta menos que una que dispara fichas sobre el ruido de una
/// cafetería.
const APRENDIZAJE: usize = 25;

impl Default for PorEnergia {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl PorEnergia {
    pub fn nuevo() -> Self {
        Self { suelo: PISO_ABSOLUTO, vistos: 0, seguidas: 0 }
    }

    /// La energía de un marco, como valor eficaz. Es la medida que corresponde al volumen que
    /// percibe una persona, y la que no se descoloca por una muestra suelta muy alta.
    pub fn energia(marco: &[f32]) -> f32 {
        if marco.is_empty() {
            return 0.0;
        }
        (marco.iter().map(|m| m * m).sum::<f32>() / marco.len() as f32).sqrt()
    }

    /// El suelo de ruido que el detector cree que tiene esta sala. Se expone para el log y para
    /// los tests; no lo usa nadie más.
    pub fn suelo(&self) -> f32 {
        self.suelo
    }
}

impl Detector for PorEnergia {
    fn hay_voz(&mut self, marco: &[f32]) -> bool {
        let e = Self::energia(marco);
        self.vistos += 1;

        // Medio segundo midiendo la sala, sin afirmar nada.
        if self.vistos <= APRENDIZAJE {
            self.suelo = self.suelo * (1.0 - APRENDE) + e * APRENDE;
            return false;
        }

        let voz = e > (self.suelo * SOBRE_EL_SUELO).max(PISO_ABSOLUTO);
        if !voz {
            // Solo aquí se mueve el suelo: mientras hay voz no hay silencio que medir.
            self.seguidas = 0;
            let inercia = if e < self.suelo { BAJA } else { SUBE };
            self.suelo = self.suelo * (1.0 - inercia) + e * inercia;
            return false;
        }

        self.seguidas += 1;
        if self.seguidas * super::MARCO_MS >= PACIENCIA_MS {
            // Se atascó. Volver a medir es la única salida honesta.
            self.olvidar();
            return false;
        }
        true
    }

    fn olvidar(&mut self) {
        self.suelo = PISO_ABSOLUTO;
        self.vistos = 0;
        self.seguidas = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voz::MARCO;

    fn silencio() -> Vec<f32> {
        vec![0.0; MARCO]
    }

    /// Ruido de sala: aleatorio pero REPRODUCIBLE. Un test de audio con `rand` de verdad es un
    /// test que un día falla en la CI y nadie sabe por qué.
    fn ruido(nivel: f32, semilla: u32) -> Vec<f32> {
        let mut x = semilla.wrapping_mul(2_654_435_761).max(1);
        (0..MARCO)
            .map(|_| {
                x ^= x << 13;
                x ^= x >> 17;
                x ^= x << 5;
                ((x as f32 / u32::MAX as f32) * 2.0 - 1.0) * nivel
            })
            .collect()
    }

    fn voz(nivel: f32, marco_n: usize) -> Vec<f32> {
        (0..MARCO)
            .map(|n| {
                let t = (marco_n * MARCO + n) as f32 / 16_000.0;
                (2.0 * std::f32::consts::PI * 180.0 * t).sin() * nivel
            })
            .collect()
    }

    #[test]
    fn el_silencio_no_es_voz() {
        let mut d = PorEnergia::nuevo();
        for _ in 0..100 {
            assert!(!d.hay_voz(&silencio()));
        }
    }

    #[test]
    fn no_afirma_nada_mientras_conoce_la_sala() {
        let mut d = PorEnergia::nuevo();
        // Aunque llegue voz clarísima desde el primer marco, el medio segundo inicial es de
        // escucha: el detector no afirma sobre una sala que todavía no ha medido.
        for i in 0..APRENDIZAJE {
            assert!(!d.hay_voz(&voz(0.3, i)), "afirmó voz en el marco {i}, todavía aprendiendo");
        }
    }

    #[test]
    fn tras_medir_una_sala_callada_la_voz_se_oye_enseguida() {
        let mut d = PorEnergia::nuevo();
        for _ in 0..APRENDIZAJE {
            d.hay_voz(&silencio());
        }
        assert!(d.hay_voz(&voz(0.3, APRENDIZAJE)));
    }

    /// **La limitación, escrita como test y no como comentario.** Arrancar la app con alguien ya
    /// hablando le enseña al detector que esa voz es el silencio de la sala: ese turno se pierde.
    /// Lo que no se puede perder es el siguiente — y no se pierde, porque el suelo baja deprisa.
    #[test]
    fn si_arranca_a_media_frase_pierde_ese_turno_y_recupera_el_oido_en_la_pausa() {
        let mut d = PorEnergia::nuevo();
        for i in 0..APRENDIZAJE {
            d.hay_voz(&voz(0.3, i));
        }
        assert!(
            !d.hay_voz(&voz(0.3, APRENDIZAJE)),
            "detectó la voz sobre la que aprendió: sería suerte, no diseño"
        );
        for _ in 0..50 {
            d.hay_voz(&silencio()); // un segundo de pausa
        }
        assert!(d.hay_voz(&voz(0.3, 200)), "no recuperó el oído tras la pausa: el turno siguiente también se pierde");
    }

    /// El otro lado de congelar el suelo mientras hay voz: si un ruido constante arranca a mitad
    /// de una frase, el detector puede quedarse creyendo que oye voz para siempre. A los treinta
    /// segundos desconfía de sí mismo y vuelve a medir.
    #[test]
    fn un_ruido_que_no_cesa_obliga_a_volver_a_medir_la_sala() {
        let mut d = PorEnergia::nuevo();
        for _ in 0..APRENDIZAJE {
            d.hay_voz(&silencio());
        }
        let marcos_de_paciencia = PACIENCIA_MS / crate::voz::MARCO_MS;
        for i in 0..marcos_de_paciencia - 1 {
            assert!(d.hay_voz(&ruido(0.15, i as u32)), "dejó de oír el ruido en el marco {i}");
        }
        assert!(!d.hay_voz(&ruido(0.15, 9_999)), "a los 30 s seguidos debería desconfiar y volver a medir");
        // Y tras volver a medir, ese mismo ruido ya es el suelo de la sala: deja de ser voz.
        for i in 0..APRENDIZAJE {
            d.hay_voz(&ruido(0.15, i as u32));
        }
        assert!(!d.hay_voz(&ruido(0.15, 7_777)), "volvió a medir y siguió llamando voz al ruido");
    }

    #[test]
    fn una_voz_normal_sobre_ruido_de_sala_se_oye() {
        let mut d = PorEnergia::nuevo();
        for i in 0..50 {
            d.hay_voz(&ruido(0.01, i));
        }
        assert!(d.suelo() < 0.02, "el suelo se fue a {:.4} con ruido de 0,01", d.suelo());
        assert!(d.hay_voz(&voz(0.2, 51)), "una voz a 0,2 sobre ruido de 0,01 no se detectó");
    }

    /// **El gate del detector.** Una frase larga no puede acabar sonando a silencio. Si el suelo
    /// subiera deprisa (`SUBE` alto), el detector se «acostumbraría» a la voz y el turno se
    /// cerraría en mitad de la frase, con la ficha saliendo mientras el cliente sigue hablando.
    ///
    /// Se ve en rojo devolviendo la actualización del suelo a todos los marcos (como estaba en la
    /// primera versión): la voz deja de detectarse en el segundo 3,7.
    #[test]
    fn una_frase_larga_no_se_convierte_en_silencio() {
        let mut d = PorEnergia::nuevo();
        for i in 0..50 {
            d.hay_voz(&ruido(0.01, i));
        }
        // 15 segundos seguidos hablando: 750 marcos de 20 ms.
        for i in 0..750 {
            assert!(
                d.hay_voz(&voz(0.2, 100 + i)),
                "dejó de oír la voz en el segundo {:.1}: el suelo se comió al hablante",
                i as f32 * 0.02
            );
        }
    }

    /// El suelo tiene que BAJAR deprisa: si una sala ruidosa se calla, el detector debe recuperar
    /// la sensibilidad en menos de un segundo o se perderá el arranque del turno siguiente.
    #[test]
    fn cuando_la_sala_se_calla_el_detector_recupera_el_oido() {
        let mut d = PorEnergia::nuevo();
        for i in 0..200 {
            d.hay_voz(&ruido(0.15, i)); // sala ruidosa desde el primer marco: se aprende así
        }
        let ruidoso = d.suelo();
        assert!(ruidoso > 0.05, "la sala ruidosa no llegó a medirse: suelo {ruidoso:.4}");
        for _ in 0..50 {
            d.hay_voz(&silencio()); // un segundo de calma
        }
        assert!(
            d.suelo() < ruidoso / 10.0,
            "tras un segundo de calma el suelo sigue en {:.4} (venía de {:.4})",
            d.suelo(),
            ruidoso
        );
    }

    #[test]
    fn olvidar_deja_al_detector_como_recien_nacido() {
        let mut d = PorEnergia::nuevo();
        for i in 0..200 {
            d.hay_voz(&ruido(0.15, i));
        }
        assert!(d.suelo() > PISO_ABSOLUTO);
        assert_ne!(d.vistos, 0);
        d.olvidar();
        assert_eq!(d.suelo(), PISO_ABSOLUTO);
        assert!(!d.hay_voz(&voz(0.3, 0)), "tras olvidar debería volver a aprender la sala");
    }
}
