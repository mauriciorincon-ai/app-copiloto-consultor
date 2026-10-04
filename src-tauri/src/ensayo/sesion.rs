//! LA SESIÓN DE ENSAYO — una máquina de estados **pura**, con el reloj por parámetro (ADR 019 §6).
//!
//! `preguntando` (la voz lee) → `respondiendo` (el micrófono oye) → `evaluada` (cuatro cifras) → la
//! siguiente… → `cerrado` (el informe). Más repetir (R), saltar (S), terminar (Esc) y «Sí lo dije».
//!
//! Nada de aquí abre un micrófono, habla ni busca: cada método devuelve [`Accion`]es y quien la lleva
//! ([`super::Ensayo`]) las ejecuta. Así cada regla de tiempo se prueba con un número, no con un Mac:
//!
//! - **La voz y el micrófono no se oyen**: mientras la voz lee, [`Sesion::oye`] es `false`, y sigue
//!   así [`COLA_DE_LA_VOZ_MS`] después de que calle. Sin reunión la voz puede salir por los altavoces;
//!   así la app no se transcribe a sí misma. (En reunión, la voz nunca sale por los altavoces: ADR 014,
//!   `habla::cabe_decirla`, que el ensayo no toca.)
//! - **La respuesta** se cierra con [`SILENCIO_DE_RESPUESTA_MS`] desde el fin de tu última voz, o con
//!   Enter. El silencio solo cuenta después de tu primera palabra: pensar antes de empezar no cierra
//!   nada.
//! - **Un turno de otra ronda se tira**: si repites la pregunta mientras se transcribía tu respuesta
//!   anterior, ese texto llega tarde y no se mezcla con la nueva ([`Sesion::ronda`]).

use serde::Serialize;

use super::banco::{self, Idioma, Pregunta};
use super::evaluacion::{Evaluacion, Muletilla, Tramo};

/// Cuánto silencio, desde el fin de tu última voz, cierra la respuesta. De partida (ADR 019 §6.2); se
/// ajusta con el kit y con la prueba de tu voz.
pub const SILENCIO_DE_RESPUESTA_MS: u64 = 2_500;
/// Cuánto sigue sordo el micrófono después de que la voz calla: lo que tarda el eco de la sala.
pub const COLA_DE_LA_VOZ_MS: u64 = 300;
/// Si la voz no empezó a sonar en este tiempo, no va a sonar: se responde sin esperarla.
pub const ESPERA_A_LA_VOZ_MS: u64 = 2_000;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fase {
    Preguntando,
    Respondiendo,
    Evaluada,
    Cerrado,
}

/// Lo que la sesión le pide a quien la lleva.
#[derive(Clone, Debug, PartialEq)]
pub enum Accion {
    /// Lee esta pregunta en voz alta.
    Decir(String),
    /// Calla lo que esté diciendo.
    Callar,
    /// La respuesta se cerró: evalúala y devuélvela con [`Sesion::evaluada`].
    Evaluar(Cerrada),
    /// Cambió algo que la pantalla enseña.
    Avisar,
}

/// Una respuesta cerrada, lista para evaluar.
#[derive(Clone, Debug, PartialEq)]
pub struct Cerrada {
    pub indice: usize,
    pub pregunta: String,
    pub tramos: Vec<Tramo>,
    pub empezo_ms: u64,
    pub cerro_ms: u64,
}

/// Qué pasó con una pregunta.
#[derive(Clone, Debug, PartialEq)]
pub enum Suerte {
    /// Tu respuesta en texto (es tuya: la pista del micrófono) y sus cifras.
    Respondida { respuesta: String, evaluacion: Evaluacion },
    Saltada,
}

/// Lo que el micrófono está haciendo en este instante, según el oído.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Oyendo {
    /// Hay un turno tuyo en curso.
    pub hablando: bool,
    /// Turnos cerrados que se están transcribiendo.
    pub pendientes: usize,
}

#[derive(Clone, Copy, Debug)]
struct Lectura {
    pedida_ms: u64,
    sono: bool,
    callo_ms: Option<u64>,
}

pub struct Sesion {
    preguntas: Vec<Pregunta>,
    idioma: Idioma,
    voz: bool,
    actual: usize,
    fase: Fase,
    lectura: Option<Lectura>,
    ronda: u64,
    abrio_ms: u64,
    tramos: Vec<Tramo>,
    cerrar_en: Option<u64>,
    evaluando: bool,
    suertes: Vec<Option<Suerte>>,
}

/// Pisa un texto antes de soltarlo, como el resto de la casa.
fn pisar(s: &mut String) {
    // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
    unsafe { s.as_mut_vec() }.fill(0);
    s.clear();
}

fn pisar_tramos(tramos: &mut Vec<Tramo>) {
    for t in tramos.iter_mut() {
        if let Some(x) = &mut t.texto {
            pisar(x);
        }
    }
    tramos.clear();
}

impl Sesion {
    /// Empieza por la primera pregunta. `None` sin preguntas: sin banco no hay ensayo.
    pub fn nueva(preguntas: Vec<Pregunta>, idioma: Idioma, voz: bool, ahora: u64) -> Option<(Self, Vec<Accion>)> {
        if preguntas.is_empty() {
            return None;
        }
        let n = preguntas.len();
        let mut s = Self {
            preguntas,
            idioma,
            voz,
            actual: 0,
            fase: Fase::Preguntando,
            lectura: None,
            ronda: 0,
            abrio_ms: ahora,
            tramos: Vec::new(),
            cerrar_en: None,
            evaluando: false,
            suertes: vec![None; n],
        };
        let acciones = s.preguntar(ahora);
        Some((s, acciones))
    }

    fn preguntar(&mut self, ahora: u64) -> Vec<Accion> {
        self.fase = Fase::Preguntando;
        self.ronda += 1;
        pisar_tramos(&mut self.tramos);
        self.cerrar_en = None;
        self.evaluando = false;
        if self.voz {
            self.lectura = Some(Lectura { pedida_ms: ahora, sono: false, callo_ms: None });
            vec![Accion::Decir(self.preguntas[self.actual].texto.clone()), Accion::Avisar]
        } else {
            self.abrir(ahora);
            vec![Accion::Avisar]
        }
    }

    fn abrir(&mut self, desde_ms: u64) {
        self.fase = Fase::Respondiendo;
        self.lectura = None;
        self.abrio_ms = desde_ms;
    }

    fn siguiente(&mut self, ahora: u64) -> Vec<Accion> {
        if self.actual + 1 < self.preguntas.len() {
            self.actual += 1;
            self.preguntar(ahora)
        } else {
            self.cerrar_la_sesion()
        }
    }

    fn cerrar_la_sesion(&mut self) -> Vec<Accion> {
        self.fase = Fase::Cerrado;
        self.lectura = None;
        self.ronda += 1;
        pisar_tramos(&mut self.tramos);
        vec![Accion::Callar, Accion::Avisar]
    }

    fn cerrar(&mut self, cerro_ms: u64) -> Vec<Accion> {
        self.evaluando = true;
        vec![Accion::Evaluar(Cerrada {
            indice: self.actual,
            pregunta: self.preguntas[self.actual].texto.clone(),
            tramos: self.tramos.clone(),
            empezo_ms: self.abrio_ms,
            cerro_ms: cerro_ms.max(self.abrio_ms),
        })]
    }

    /// ¿Escucha el micrófono ahora? Solo mientras respondes y antes de cerrar.
    pub fn oye(&self) -> bool {
        self.fase == Fase::Respondiendo && self.cerrar_en.is_none() && !self.evaluando
    }

    /// Sube cada vez que se abre una respuesta nueva o se descarta la que había.
    pub fn ronda(&self) -> u64 {
        self.ronda
    }

    /// **El latido**: la voz que calla abre la respuesta; el silencio o Enter la cierran.
    pub fn tick(&mut self, ahora: u64, o: Oyendo, voz_diciendo: bool) -> Vec<Accion> {
        match self.fase {
            Fase::Preguntando => {
                let Some(l) = &mut self.lectura else { return Vec::new() };
                if voz_diciendo {
                    l.sono = true;
                    l.callo_ms = None;
                } else if l.sono {
                    let callo = *l.callo_ms.get_or_insert(ahora);
                    if ahora.saturating_sub(callo) >= COLA_DE_LA_VOZ_MS {
                        // El reloj de tu respuesta empieza cuando la pregunta terminó, no cuando el
                        // micrófono dejó de estar sordo: la cola es de la app, no tuya.
                        self.abrir(callo);
                        return vec![Accion::Avisar];
                    }
                } else if ahora.saturating_sub(l.pedida_ms) >= ESPERA_A_LA_VOZ_MS {
                    self.abrir(ahora);
                    return vec![Accion::Avisar];
                }
                Vec::new()
            }
            Fase::Respondiendo => {
                if self.evaluando || o.hablando || o.pendientes > 0 {
                    return Vec::new();
                }
                if let Some(en) = self.cerrar_en {
                    return self.cerrar(en);
                }
                match self.tramos.last() {
                    Some(t) if ahora.saturating_sub(t.hasta_ms) >= SILENCIO_DE_RESPUESTA_MS => {
                        // Cerró el silencio: tu respuesta terminó cuando terminó tu voz. Los 2,5 s
                        // son la espera de la app y no se te cargan.
                        let fin = t.hasta_ms;
                        self.cerrar(fin)
                    }
                    _ => Vec::new(),
                }
            }
            Fase::Evaluada | Fase::Cerrado => Vec::new(),
        }
    }

    /// Un turno tuyo, ya transcrito (o `None` si se oyó y no se pudo). Solo cuenta si es de esta ronda y
    /// estás respondiendo; si no, se pisa y se tira.
    pub fn turno(&mut self, ronda: u64, desde_ms: u64, hasta_ms: u64, mut texto: Option<String>) -> Vec<Accion> {
        if ronda != self.ronda || self.fase != Fase::Respondiendo || self.evaluando {
            if let Some(t) = &mut texto {
                pisar(t);
            }
            return Vec::new();
        }
        // Tu voz no empezó antes de la pregunta ni acaba antes de empezar: si el micrófono entrega a
        // golpes, el reloj se ajusta aquí y no en cada cifra.
        let desde_ms = desde_ms.max(self.abrio_ms);
        self.tramos.push(Tramo { desde_ms, hasta_ms: hasta_ms.max(desde_ms), texto });
        vec![Accion::Avisar]
    }

    /// La voz no pudo leer (no hay voz para ese idioma): se responde sin ella desde ya, y el resto del
    /// ensayo va sin voz.
    pub fn sin_voz(&mut self, ahora: u64) -> Vec<Accion> {
        self.voz = false;
        if self.fase == Fase::Preguntando {
            self.abrir(ahora);
        }
        vec![Accion::Avisar]
    }

    /// **Enter**: deja de leer y escucha ya · cierra la respuesta · pasa a la siguiente.
    pub fn listo(&mut self, ahora: u64) -> Vec<Accion> {
        match self.fase {
            Fase::Preguntando => {
                self.abrir(ahora);
                vec![Accion::Callar, Accion::Avisar]
            }
            Fase::Respondiendo if self.cerrar_en.is_none() && !self.evaluando => {
                self.cerrar_en = Some(ahora);
                vec![Accion::Avisar]
            }
            Fase::Evaluada => self.siguiente(ahora),
            _ => Vec::new(),
        }
    }

    /// **R**: vuelve a hacer la pregunta y descarta lo que llevabas.
    pub fn repetir(&mut self, ahora: u64) -> Vec<Accion> {
        match self.fase {
            Fase::Preguntando | Fase::Respondiendo => {
                let mut a = vec![Accion::Callar];
                a.extend(self.preguntar(ahora));
                a
            }
            _ => Vec::new(),
        }
    }

    /// **S**: la pregunta queda saltada y pasa la siguiente.
    pub fn saltar(&mut self, ahora: u64) -> Vec<Accion> {
        match self.fase {
            Fase::Preguntando | Fase::Respondiendo => {
                self.suertes[self.actual] = Some(Suerte::Saltada);
                let mut a = vec![Accion::Callar];
                a.extend(self.siguiente(ahora));
                a
            }
            _ => Vec::new(),
        }
    }

    /// **Esc**: termina el ensayo. La pregunta en curso, si no se evaluó, no cuenta ni como respondida
    /// ni como saltada: no llegaste a contestarla.
    pub fn terminar(&mut self) -> Vec<Accion> {
        if self.fase == Fase::Cerrado {
            return Vec::new();
        }
        self.cerrar_la_sesion()
    }

    /// La evaluación de la respuesta que cerró [`Accion::Evaluar`].
    pub fn evaluada(&mut self, indice: usize, evaluacion: Evaluacion) -> Vec<Accion> {
        if indice != self.actual || self.fase != Fase::Respondiendo || !self.evaluando {
            return Vec::new();
        }
        let respuesta = self.respuesta_en_curso();
        pisar_tramos(&mut self.tramos);
        self.suertes[self.actual] = Some(Suerte::Respondida { respuesta, evaluacion });
        self.evaluando = false;
        self.cerrar_en = None;
        self.fase = Fase::Evaluada;
        vec![Accion::Avisar]
    }

    /// «Sí lo dije» sobre la ficha `i` de la pregunta recién evaluada.
    pub fn si_lo_dije(&mut self, i: usize) -> Vec<Accion> {
        if self.fase != Fase::Evaluada {
            return Vec::new();
        }
        match &mut self.suertes[self.actual] {
            Some(Suerte::Respondida { evaluacion, .. }) => {
                if evaluacion.si_lo_dije(i) {
                    vec![Accion::Avisar]
                } else {
                    Vec::new()
                }
            }
            _ => Vec::new(),
        }
    }

    /// Las preguntas del modelo llegan cuando llegan: van al final, marcadas, sin repetir.
    pub fn con_las_del_modelo(&mut self, mas: Vec<Pregunta>) -> Vec<Accion> {
        if self.fase == Fase::Cerrado || mas.is_empty() {
            return Vec::new();
        }
        let antes = self.preguntas.len();
        self.preguntas = banco::con_las_del_modelo(std::mem::take(&mut self.preguntas), mas);
        self.suertes.resize(self.preguntas.len(), None);
        if self.preguntas.len() == antes {
            return Vec::new();
        }
        vec![Accion::Avisar]
    }

    // ─── lo que la pantalla lee ──────────────────────────────────────────────────────────────────

    pub fn fase(&self) -> Fase {
        self.fase
    }
    pub fn idioma(&self) -> Idioma {
        self.idioma
    }
    pub fn actual(&self) -> usize {
        self.actual
    }
    pub fn preguntas(&self) -> &[Pregunta] {
        &self.preguntas
    }
    pub fn suertes(&self) -> &[Option<Suerte>] {
        &self.suertes
    }
    /// La voz está leyendo la pregunta.
    pub fn leyendo(&self) -> bool {
        self.fase == Fase::Preguntando && self.lectura.is_some()
    }
    /// Cerraste con Enter y se espera a la transcripción del último turno.
    pub fn cerrando(&self) -> bool {
        self.fase == Fase::Respondiendo && (self.cerrar_en.is_some() || self.evaluando)
    }
    /// Lo que llevas dicho, en texto.
    pub fn respuesta_en_curso(&self) -> String {
        self.tramos.iter().filter_map(|t| t.texto.as_deref()).collect::<Vec<_>>().join(" ")
    }
    /// Cuánto llevas respondiendo; en una evaluada, lo que tardaste.
    pub fn transcurrido(&self, ahora: u64) -> u64 {
        match (self.fase, self.suertes.get(self.actual)) {
            (Fase::Respondiendo, _) => self.cerrar_en.unwrap_or(ahora).saturating_sub(self.abrio_ms),
            (Fase::Evaluada, Some(Some(Suerte::Respondida { evaluacion, .. }))) => evaluacion.tiempo_ms,
            _ => 0,
        }
    }

    /// **El informe**: una fila por pregunta a la que llegaste, y los totales.
    pub fn informe(&self) -> Informe {
        let mut filas = Vec::new();
        for (i, (p, s)) in self.preguntas.iter().zip(&self.suertes).enumerate() {
            let Some(s) = s else { continue };
            let (citadas, evidencia, tiempo_ms, ppm, saltada) = match s {
                Suerte::Respondida { evaluacion: e, .. } => (e.citadas(), e.evidencia.len(), Some(e.tiempo_ms), e.ppm, false),
                Suerte::Saltada => (0, 0, None, None, true),
            };
            filas.push(Fila { numero: i + 1, texto: p.texto.clone(), citadas, evidencia, tiempo_ms, ppm, saltada });
        }
        let respondidas: Vec<&Evaluacion> = self
            .suertes
            .iter()
            .filter_map(|s| match s {
                Some(Suerte::Respondida { evaluacion, .. }) => Some(evaluacion),
                _ => None,
            })
            .collect();
        let media = |v: Vec<u64>| (!v.is_empty()).then(|| v.iter().sum::<u64>() / v.len() as u64);
        let mut cuenta: Vec<Muletilla> = Vec::new();
        for m in respondidas.iter().flat_map(|e| &e.muletillas) {
            match cuenta.iter_mut().find(|c| c.frase == m.frase) {
                Some(c) => c.veces += m.veces,
                None => cuenta.push(m.clone()),
            }
        }
        cuenta.sort_by_key(|m| std::cmp::Reverse(m.veces));
        Informe {
            respondidas: respondidas.len(),
            saltadas: filas.iter().filter(|f| f.saltada).count(),
            citadas: respondidas.iter().map(|e| e.citadas()).sum(),
            evidencia: respondidas.iter().map(|e| e.evidencia.len()).sum(),
            ppm_medio: media(respondidas.iter().filter_map(|e| e.ppm.map(u64::from)).collect()).map(|x| x as u32),
            muletillas: cuenta.iter().map(|m| m.veces).sum(),
            la_que_mas: cuenta.into_iter().next(),
            tiempo_medio_ms: media(respondidas.iter().map(|e| e.tiempo_ms).collect()),
            filas,
        }
    }

    /// Pisa todo lo que dijiste y suelta las preguntas. Lo llaman el corte, «Cerrar sin guardar» y el
    /// final de la vida de la sesión.
    pub fn vaciar(&mut self) {
        pisar_tramos(&mut self.tramos);
        for s in self.suertes.iter_mut() {
            if let Some(Suerte::Respondida { respuesta, .. }) = s {
                pisar(respuesta);
            }
            *s = None;
        }
        self.suertes.clear();
        self.preguntas.clear();
        self.fase = Fase::Cerrado;
        self.ronda += 1;
    }
}

impl Drop for Sesion {
    fn drop(&mut self) {
        self.vaciar();
    }
}

/// Una fila del informe.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fila {
    pub numero: usize,
    pub texto: String,
    pub citadas: usize,
    pub evidencia: usize,
    pub tiempo_ms: Option<u64>,
    pub ppm: Option<u32>,
    pub saltada: bool,
}

/// El informe del ensayo: los totales y una fila por pregunta.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Informe {
    pub respondidas: usize,
    pub saltadas: usize,
    pub citadas: usize,
    pub evidencia: usize,
    pub ppm_medio: Option<u32>,
    pub muletillas: u32,
    pub la_que_mas: Option<Muletilla>,
    pub tiempo_medio_ms: Option<u64>,
    pub filas: Vec<Fila>,
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::ensayo::banco::{De, Regla};
    use crate::ensayo::evaluacion::Evidencia;

    fn p(texto: &str) -> Pregunta {
        Pregunta { texto: texto.into(), regla: Regla::Seccion, de: De::Propuesta, seccion: Some("Alcance".into()), clave: "alcance".into(), fuente: None }
    }

    fn tres() -> Vec<Pregunta> {
        vec![p("¿Uno?"), p("¿Dos?"), p("¿Tres?")]
    }

    fn eval(tiempo_ms: u64, ppm: Option<u32>, citadas: usize) -> Evaluacion {
        let fuente = crate::ficha::Fuente { documento: "d".into(), seccion: None, unidad: None, conjeturada: false };
        Evaluacion {
            evidencia: (0..3).map(|i| Evidencia { titular: format!("f{i}"), fuente: fuente.clone(), citada: i < citadas, dicha_por_ti: false }).collect(),
            tiempo_ms,
            ppm,
            muletillas: vec![Muletilla { frase: "o sea".into(), veces: 2 }],
            palabras: 10,
        }
    }

    const CALLADO: Oyendo = Oyendo { hablando: false, pendientes: 0 };

    /// La evaluación que haría quien lleva la sesión, para poder recorrerla entera aquí.
    fn evaluar_si_toca(s: &mut Sesion, acciones: Vec<Accion>) -> Option<Cerrada> {
        let c = acciones.into_iter().find_map(|a| match a {
            Accion::Evaluar(c) => Some(c),
            _ => None,
        })?;
        s.evaluada(c.indice, eval(c.cerro_ms - c.empezo_ms, Some(130), 1));
        Some(c)
    }

    /// **Sordo mientras la voz lee, y 300 ms después.** Es la regla que deja a la voz salir por los
    /// altavoces sin reunión: el micrófono no se oye a la app.
    #[test]
    fn el_microfono_esta_sordo_mientras_la_voz_lee_y_su_cola() {
        let (mut s, a) = Sesion::nueva(tres(), Idioma::Es, true, 0).unwrap();
        assert_eq!(a[0], Accion::Decir("¿Uno?".into()));
        assert!(!s.oye());
        // La voz tarda en sonar (B21) y luego lee un segundo y medio.
        assert!(s.tick(300, CALLADO, false).is_empty());
        assert!(s.tick(400, CALLADO, true).is_empty());
        assert!(s.tick(1_900, CALLADO, true).is_empty());
        assert!(!s.oye(), "sordo mientras habla");
        // Calla a los 1 950: la cola de 300 ms sigue sorda.
        assert!(s.tick(1_950, CALLADO, false).is_empty());
        assert!(s.tick(2_200, CALLADO, false).is_empty());
        assert!(!s.oye(), "sordo durante la cola");
        assert_eq!(s.tick(2_250, CALLADO, false), vec![Accion::Avisar]);
        assert!(s.oye());
        assert_eq!(s.fase(), Fase::Respondiendo);
        // Tu tiempo empieza cuando la pregunta terminó, no al final de la cola.
        assert_eq!(s.transcurrido(3_950), 2_000);
    }

    #[test]
    fn si_la_voz_no_suena_se_responde_sin_esperarla() {
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, true, 0).unwrap();
        assert!(s.tick(1_999, CALLADO, false).is_empty());
        assert_eq!(s.tick(2_000, CALLADO, false), vec![Accion::Avisar]);
        assert!(s.oye());
        // Sin voz para el idioma: se avisa y el resto va sin voz.
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, true, 0).unwrap();
        s.sin_voz(10);
        assert!(s.oye());
    }

    /// **Pensar antes de empezar no cierra nada**; 2,5 s de silencio después de tu voz, sí, y tu tiempo
    /// termina con tu voz.
    #[test]
    fn la_respuesta_se_cierra_con_dos_segundos_y_medio_de_silencio_tras_tu_voz() {
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, false, 0).unwrap();
        assert!(s.oye());
        assert!(s.tick(20_000, CALLADO, false).is_empty(), "sin haber hablado, el silencio no cierra");
        let r = s.ronda();
        s.turno(r, 21_000, 25_000, Some("El supuesto dos lo cubre.".into()));
        assert!(s.tick(27_000, Oyendo { hablando: true, pendientes: 0 }, false).is_empty(), "hablando no cierra");
        s.turno(r, 26_000, 28_000, Some("O sea, se corre.".into()));
        assert!(s.tick(30_499, CALLADO, false).is_empty());
        assert!(s.tick(30_600, Oyendo { hablando: false, pendientes: 1 }, false).is_empty(), "con un turno transcribiéndose, no cierra");
        let a = s.tick(30_600, CALLADO, false);
        let c = evaluar_si_toca(&mut s, a).expect("el silencio cerró la respuesta");
        assert_eq!((c.empezo_ms, c.cerro_ms), (0, 28_000));
        assert_eq!(c.tramos.len(), 2);
        assert_eq!(s.fase(), Fase::Evaluada);
        match &s.suertes()[0] {
            Some(Suerte::Respondida { respuesta, .. }) => assert_eq!(respuesta, "El supuesto dos lo cubre. O sea, se corre."),
            otra => panic!("{otra:?}"),
        }
    }

    /// Enter cierra, pero espera al turno que se está transcribiendo; y tu tiempo es el de Enter.
    #[test]
    fn enter_cierra_y_espera_a_lo_que_falta_por_transcribir() {
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, false, 0).unwrap();
        let r = s.ronda();
        s.turno(r, 500, 4_000, Some("Primera parte.".into()));
        assert_eq!(s.listo(4_100), vec![Accion::Avisar]);
        assert!(!s.oye(), "tras Enter, lo que digas ya no entra");
        assert!(s.cerrando());
        assert!(s.tick(4_200, Oyendo { hablando: false, pendientes: 1 }, false).is_empty());
        s.turno(r, 3_900, 4_100, Some("y fin.".into()));
        let a = s.tick(4_400, CALLADO, false);
        let c = evaluar_si_toca(&mut s, a).unwrap();
        assert_eq!(c.cerro_ms, 4_100);
        assert_eq!(c.tramos.len(), 2);
        // Enter en la evaluada pasa a la siguiente.
        s.listo(5_000);
        assert_eq!((s.actual(), s.fase()), (1, Fase::Respondiendo));
    }

    /// Repetir descarta lo que llevabas, y lo que llegue de antes se tira.
    #[test]
    fn repetir_descarta_y_lo_que_llega_tarde_de_otra_ronda_se_tira() {
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, true, 0).unwrap();
        s.listo(100); // salta la lectura
        let vieja = s.ronda();
        s.turno(vieja, 200, 1_000, Some("Me equivoqué.".into()));
        let a = s.repetir(1_200);
        assert_eq!(a[0], Accion::Callar);
        assert!(a.contains(&Accion::Decir("¿Uno?".into())));
        assert_eq!(s.respuesta_en_curso(), "");
        s.sin_voz(1_300);
        assert!(s.turno(vieja, 900, 1_100, Some("tarde".into())).is_empty(), "un turno de otra ronda");
        assert_eq!(s.respuesta_en_curso(), "");
    }

    #[test]
    fn saltar_y_terminar_cuentan_lo_que_toca() {
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, false, 0).unwrap();
        // 1: respondida
        let r = s.ronda();
        s.turno(r, 0, 3_000, Some("algo".into()));
        let a = s.tick(6_000, CALLADO, false);
        evaluar_si_toca(&mut s, a).unwrap();
        s.listo(6_100);
        // 2: saltada
        s.saltar(6_200);
        assert_eq!((s.actual(), s.fase()), (2, Fase::Respondiendo));
        // 3: Esc a media respuesta: no cuenta.
        let r = s.ronda();
        s.turno(r, 6_300, 7_000, Some("a medias".into()));
        assert_eq!(s.terminar(), vec![Accion::Callar, Accion::Avisar]);
        assert_eq!(s.fase(), Fase::Cerrado);
        let i = s.informe();
        assert_eq!((i.respondidas, i.saltadas, i.filas.len()), (1, 1, 2));
        assert!(i.filas[1].saltada);
        assert!(s.terminar().is_empty());
        // Saltar la última cierra el ensayo.
        let (mut s, _) = Sesion::nueva(vec![p("¿Única?")], Idioma::Es, false, 0).unwrap();
        s.saltar(10);
        assert_eq!(s.fase(), Fase::Cerrado);
    }

    #[test]
    fn el_informe_suma_y_promedia_sin_inventar() {
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, false, 0).unwrap();
        for i in 0..3u64 {
            let r = s.ronda();
            let base = i * 100_000;
            s.turno(r, base + 1_000, base + 31_000, Some("algo dicho".into()));
            let a = s.tick(base + 40_000, CALLADO, false);
            evaluar_si_toca(&mut s, a).unwrap();
            s.listo(base + 41_000);
        }
        assert_eq!(s.fase(), Fase::Cerrado);
        let i = s.informe();
        assert_eq!((i.respondidas, i.saltadas, i.citadas, i.evidencia), (3, 0, 3, 9));
        assert_eq!(i.ppm_medio, Some(130));
        assert_eq!(i.muletillas, 6);
        assert_eq!(i.la_que_mas, Some(Muletilla { frase: "o sea".into(), veces: 6 }));
        assert!(i.tiempo_medio_ms.is_some());
    }

    #[test]
    fn si_lo_dije_solo_en_la_evaluada() {
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, false, 0).unwrap();
        assert!(s.si_lo_dije(2).is_empty());
        let r = s.ronda();
        s.turno(r, 0, 3_000, Some("algo".into()));
        let a = s.tick(6_000, CALLADO, false);
        evaluar_si_toca(&mut s, a).unwrap();
        assert_eq!(s.si_lo_dije(2), vec![Accion::Avisar]);
        assert_eq!(s.informe().citadas, 2);
    }

    #[test]
    fn las_del_modelo_van_al_final_y_no_llegan_a_un_ensayo_cerrado() {
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, false, 0).unwrap();
        let mut m = p("¿Del modelo?");
        m.de = De::Modelo;
        m.regla = Regla::Modelo;
        assert_eq!(s.con_las_del_modelo(vec![m.clone()]), vec![Accion::Avisar]);
        assert_eq!(s.preguntas().len(), 4);
        assert_eq!(s.preguntas()[3].de, De::Modelo);
        assert!(s.con_las_del_modelo(vec![m.clone()]).is_empty(), "sin repetir");
        s.terminar();
        assert!(s.con_las_del_modelo(vec![p("¿Otra?")]).is_empty());
    }

    #[test]
    fn vaciar_pisa_tus_respuestas_y_cierra() {
        let (mut s, _) = Sesion::nueva(tres(), Idioma::Es, false, 0).unwrap();
        let r = s.ronda();
        s.turno(r, 0, 3_000, Some("lo mío".into()));
        s.vaciar();
        assert_eq!((s.fase(), s.preguntas().len(), s.respuesta_en_curso().as_str()), (Fase::Cerrado, 0, ""));
        assert!(Sesion::nueva(Vec::new(), Idioma::Es, false, 0).is_none(), "sin banco no hay ensayo");
    }
}
