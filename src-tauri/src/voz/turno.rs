//! El **fin de turno**: decidir que alguien ha terminado de hablar.
//!
//! Es la decisión más delicada del sprint, y lo es porque las dos maneras de equivocarse duelen en
//! sitios distintos. Cerrar pronto significa que la app le pone una ficha delante al consultor
//! mientras el cliente sigue hablando: ruido en el peor momento. Cerrar tarde significa que la
//! ficha llega cuando el consultor ya improvisó una respuesta: inútil, aunque sea correcta. El
//! presupuesto entero del sprint —≤4 s de fin de turno a ficha— se cuenta desde este instante, así
//! que todo lo que se tarde de más aquí se gasta antes de empezar.
//!
//! La orden fija el objetivo: **entre 160 y 400 ms de silencio** antes de dar el turno por
//! cerrado. Esta máquina espera [`FIN_MS`], que cae dentro, y el test que lo comprueba mide la
//! latencia contra **los números de la orden**, no contra la constante de aquí — si midiera contra
//! la propia constante, subirla a dos segundos dejaría el gate en verde y la app inservible. Es la
//! misma confusión que ya se cobró dos gates en este sprint (la huella a 600 y el recorte del
//! acople), y se evita igual: el test no puede leer lo que vigila.
//!
//! **Lo que esta máquina NO hace**: no sabe de quién es la voz (hay una por pista, y la pista ya
//! dice quién es), no transcribe, y no decide si el turno merece una ficha. Corta el flujo en
//! trozos con principio y final; lo demás es de otros.

use super::vad::{Detector, PorEnergia};
use super::MARCO_MS;

/// Cuánto silencio cierra un turno. Dentro del objetivo 160–400 ms de la orden, y hacia la mitad
/// alta a propósito: entre interrumpir a alguien que respira y esperar un pestañeo de más, la app
/// espera.
pub const FIN_MS: usize = 320;

/// Cuánta voz seguida hace falta para creer que un turno empezó. Cien milisegundos son más que
/// cualquier chasquido y menos que cualquier vocal: un «sí» entero dura el triple.
pub const ARRANQUE_MS: usize = 100;

/// Por debajo de esto no fue un turno: fue una tos, un «ajá» o una silla. No se descartan por
/// molestos, se descartan porque disparar la búsqueda del corpus con 200 ms de audio entrega una
/// ficha sacada de nada.
pub const MINIMO_MS: usize = 300;

/// Lo que la máquina cuenta cuando cuenta algo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Suceso {
    /// Alguien empezó a hablar. Sirve para que la banda pueda decir «escuchando» de verdad.
    Empieza { en_ms: usize },
    /// Alguien terminó. `desde_ms`/`hasta_ms` acotan **solo la voz**: el silencio que hizo falta
    /// para tomar la decisión queda fuera, porque mandarlo al transcriptor sería transcribir
    /// silencio y pagar su latencia.
    Termina { desde_ms: usize, hasta_ms: usize },
    /// Hubo sonido, pero demasiado corto para ser un turno. No se anuncia como turno y **tampoco
    /// se esconde**: quien lo reciba puede decidir, y el log deja constancia de que la app oyó
    /// algo y decidió no actuar.
    DemasiadoCorto { duracion_ms: usize },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Estado {
    /// Nadie habla.
    Callado,
    /// Podría estar empezando: se acumulan marcos con voz hasta [`ARRANQUE_MS`].
    Arrancando { desde_ms: usize, seguidos: usize },
    /// Turno en curso.
    Hablando { desde_ms: usize, ultima_voz_ms: usize, silencios: usize },
}

/// Parte el audio de **una** pista en turnos.
pub struct Turnos<D: Detector = PorEnergia> {
    detector: D,
    estado: Estado,
    /// Milisegundos de audio procesados. Es el reloj de la pista, no el del sistema: así los
    /// tests son deterministas y el mismo audio da siempre los mismos turnos.
    reloj_ms: usize,
}

impl Default for Turnos<PorEnergia> {
    fn default() -> Self {
        Self::nuevo(PorEnergia::nuevo())
    }
}

impl<D: Detector> Turnos<D> {
    pub fn nuevo(detector: D) -> Self {
        Self { detector, estado: Estado::Callado, reloj_ms: 0 }
    }

    /// Cuántos milisegundos de audio lleva vistos esta pista.
    pub fn reloj_ms(&self) -> usize {
        self.reloj_ms
    }

    /// ¿Hay alguien hablando ahora mismo en esta pista?
    pub fn hablando(&self) -> bool {
        matches!(self.estado, Estado::Hablando { .. })
    }

    /// Un marco de 20 ms. Devuelve lo que haya pasado al verlo.
    pub fn marco(&mut self, muestras: &[f32]) -> Option<Suceso> {
        let voz = self.detector.hay_voz(muestras);
        let inicio_ms = self.reloj_ms;
        self.reloj_ms += MARCO_MS;
        let fin_ms = self.reloj_ms;

        match (self.estado, voz) {
            (Estado::Callado, false) => None,
            (Estado::Callado, true) => {
                self.estado = Estado::Arrancando { desde_ms: inicio_ms, seguidos: 1 };
                self.confirmar_arranque()
            }
            // Un hueco durante el arranque lo cancela: era un chasquido, no una persona.
            (Estado::Arrancando { .. }, false) => {
                self.estado = Estado::Callado;
                None
            }
            (Estado::Arrancando { desde_ms, seguidos }, true) => {
                self.estado = Estado::Arrancando { desde_ms, seguidos: seguidos + 1 };
                self.confirmar_arranque()
            }
            (Estado::Hablando { desde_ms, .. }, true) => {
                self.estado = Estado::Hablando { desde_ms, ultima_voz_ms: fin_ms, silencios: 0 };
                None
            }
            (Estado::Hablando { desde_ms, ultima_voz_ms, silencios }, false) => {
                let silencios = silencios + 1;
                if silencios * MARCO_MS < FIN_MS {
                    self.estado = Estado::Hablando { desde_ms, ultima_voz_ms, silencios };
                    return None;
                }
                self.estado = Estado::Callado;
                let duracion_ms = ultima_voz_ms.saturating_sub(desde_ms);
                if duracion_ms < MINIMO_MS {
                    Some(Suceso::DemasiadoCorto { duracion_ms })
                } else {
                    Some(Suceso::Termina { desde_ms, hasta_ms: ultima_voz_ms })
                }
            }
        }
    }

    /// Cierra lo que haya abierto. La llama el fin de la sesión: si el cliente estaba a media
    /// frase cuando se cortó, ese turno existió y quien lo tenga a medias debe poder cerrarlo.
    pub fn cerrar(&mut self) -> Option<Suceso> {
        match self.estado {
            Estado::Hablando { desde_ms, ultima_voz_ms, .. } => {
                self.estado = Estado::Callado;
                let duracion_ms = ultima_voz_ms.saturating_sub(desde_ms);
                (duracion_ms >= MINIMO_MS).then_some(Suceso::Termina { desde_ms, hasta_ms: ultima_voz_ms })
            }
            _ => {
                self.estado = Estado::Callado;
                None
            }
        }
    }

    /// El kill-switch. Vuelve al principio y hace que el detector olvide la sala.
    pub fn reiniciar(&mut self) {
        self.estado = Estado::Callado;
        self.reloj_ms = 0;
        self.detector.olvidar();
    }

    fn confirmar_arranque(&mut self) -> Option<Suceso> {
        let Estado::Arrancando { desde_ms, seguidos } = self.estado else { return None };
        if seguidos * MARCO_MS < ARRANQUE_MS {
            return None;
        }
        self.estado = Estado::Hablando { desde_ms, ultima_voz_ms: self.reloj_ms, silencios: 0 };
        Some(Suceso::Empieza { en_ms: desde_ms })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Un detector de mentira gobernado por el test: así lo que se prueba es la máquina de
    /// turnos, no el detector de energía (que tiene sus propios tests). Mezclar los dos haría que
    /// un fallo en cualquiera de ellos apuntara al otro.
    struct Guion {
        marcos: Vec<bool>,
        i: usize,
    }

    impl Detector for Guion {
        fn hay_voz(&mut self, _marco: &[f32]) -> bool {
            let v = self.marcos.get(self.i).copied().unwrap_or(false);
            self.i += 1;
            v
        }
        fn olvidar(&mut self) {
            self.i = 0;
        }
    }

    /// `#` es voz, `.` es silencio. Cada carácter, 20 ms.
    fn correr(patron: &str) -> (Vec<(usize, Suceso)>, Turnos<Guion>) {
        let marcos: Vec<bool> = patron.chars().map(|c| c == '#').collect();
        let n = marcos.len();
        let mut t = Turnos::nuevo(Guion { marcos, i: 0 });
        let mut sucesos = Vec::new();
        for k in 0..n {
            if let Some(s) = t.marco(&[0.0; 320]) {
                sucesos.push((k * MARCO_MS, s));
            }
        }
        (sucesos, t)
    }

    fn v(n: usize) -> String {
        "#".repeat(n)
    }
    fn s(n: usize) -> String {
        ".".repeat(n)
    }

    #[test]
    fn el_silencio_no_produce_turnos() {
        let (sucesos, _) = correr(&s(200));
        assert!(sucesos.is_empty());
    }

    #[test]
    fn un_turno_normal_empieza_y_termina() {
        // 1 s de voz, 1 s de silencio.
        let (sucesos, _) = correr(&format!("{}{}", v(50), s(50)));
        assert_eq!(sucesos.len(), 2, "sucesos: {sucesos:?}");
        assert_eq!(sucesos[0].1, Suceso::Empieza { en_ms: 0 });
        assert_eq!(sucesos[1].1, Suceso::Termina { desde_ms: 0, hasta_ms: 1000 });
    }

    /// **El gate del fin de turno.** La latencia se mide contra los números que la ORDEN escribió
    /// (160–400 ms), jamás contra [`FIN_MS`]. Un test que se mida contra su propia constante pasa
    /// en verde con cualquier valor — que es exactamente cómo este sprint ya dejó pasar dos gates
    /// muertos.
    ///
    /// Se ve en rojo poniendo `FIN_MS = 800`.
    #[test]
    fn el_fin_de_turno_cae_dentro_del_presupuesto_de_la_orden() {
        let (sucesos, _) = correr(&format!("{}{}", v(50), s(50)));
        let (detectado_ms, suceso) = sucesos[1];
        let Suceso::Termina { hasta_ms, .. } = suceso else { panic!("no terminó: {suceso:?}") };
        // `detectado_ms` es el inicio del marco que cerró el turno; el silencio empezó en
        // `hasta_ms`. La latencia es lo que la app tardó en decidir desde la última voz.
        let latencia = (detectado_ms + MARCO_MS) - hasta_ms;
        assert!(
            (160..=400).contains(&latencia),
            "el fin de turno tardó {latencia} ms; la orden pide entre 160 y 400"
        );
    }

    #[test]
    fn un_silencio_corto_dentro_de_una_frase_no_la_parte() {
        // Respirar entre dos frases: 200 ms de silencio, por debajo de los 320 que cierran.
        let (sucesos, _) = correr(&format!("{}{}{}{}", v(30), s(10), v(30), s(40)));
        let terminas: Vec<_> = sucesos.iter().filter(|(_, s)| matches!(s, Suceso::Termina { .. })).collect();
        assert_eq!(terminas.len(), 1, "la frase se partió en dos: {sucesos:?}");
        assert_eq!(terminas[0].1, Suceso::Termina { desde_ms: 0, hasta_ms: 1400 });
    }

    #[test]
    fn el_silencio_final_no_viaja_al_transcriptor() {
        let (sucesos, _) = correr(&format!("{}{}", v(50), s(50)));
        let Suceso::Termina { hasta_ms, .. } = sucesos[1].1 else { panic!() };
        assert_eq!(hasta_ms, 1000, "el turno se lleva el silencio pegado: {:?}", sucesos[1]);
    }

    #[test]
    fn un_chasquido_no_arranca_un_turno() {
        // Dos marcos de sonido (40 ms) no llegan a los 100 ms de arranque.
        let (sucesos, _) = correr(&format!("{}{}{}{}", s(10), v(2), s(10), v(2)));
        assert!(sucesos.is_empty(), "un chasquido abrió un turno: {sucesos:?}");
    }

    #[test]
    fn una_tos_se_reconoce_y_se_descarta_como_turno() {
        // 200 ms de sonido: pasa el arranque (100 ms) pero no llega al mínimo de turno (300 ms).
        let (sucesos, _) = correr(&format!("{}{}", v(10), s(30)));
        assert_eq!(sucesos.len(), 2);
        assert_eq!(sucesos[0].1, Suceso::Empieza { en_ms: 0 });
        assert!(
            matches!(sucesos[1].1, Suceso::DemasiadoCorto { duracion_ms: 200 }),
            "se dio por turno algo de 200 ms: {:?}",
            sucesos[1].1
        );
    }

    #[test]
    fn cerrar_la_sesion_cierra_el_turno_a_medias() {
        let (_, mut t) = correr(&v(50));
        assert!(t.hablando());
        assert_eq!(t.cerrar(), Some(Suceso::Termina { desde_ms: 0, hasta_ms: 1000 }));
        assert!(!t.hablando());
        assert_eq!(t.cerrar(), None, "cerrar dos veces no puede inventar un turno");
    }

    #[test]
    fn reiniciar_borra_el_reloj_y_lo_aprendido() {
        let (_, mut t) = correr(&format!("{}{}", v(50), s(50)));
        assert_eq!(t.reloj_ms(), 2000);
        t.reiniciar();
        assert_eq!(t.reloj_ms(), 0);
        assert!(!t.hablando());
    }
}
