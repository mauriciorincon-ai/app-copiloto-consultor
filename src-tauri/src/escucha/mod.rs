//! **La escucha** — lo que junta las piezas de la fase 3 y las pone a funcionar a la vez.
//!
//! `capture` abre los grifos, `voz` corta los turnos, `stt` los convierte en texto. Ninguno de los
//! tres sabe de los otros, y esa separación es la que ha permitido probarlos por su cuenta con
//! audio inventado. Aquí se reúnen, y aquí aparecen los problemas que solo existen cuando algo
//! corre de verdad: dos pistas a la vez, un transcriptor que tarda un cuarto de segundo, y un
//! kill-switch que puede caer en cualquier instante de todo lo anterior.
//!
//! **MÓDULO PROTEGIDO.** Tiene en las manos la voz del cliente entera; no la guarda en ninguna
//! parte, y `pnpm verify:ephemeral` lo comprueba — **desde la fase 2 de la auditoría del sprint
//! 001, no antes**: esta línea decía lo mismo durante dos fases y el módulo no estaba en la lista
//! del script (hallazgo A4). Hoy además hay un gate sobre el gate: quien lleva esta marca está en
//! esa lista o el script falla.
//!
//! ### Las tres decisiones de forma que tiene este módulo
//!
//! **Una el audio no se copia dos veces.** La tentación es ir acumulando el turno en curso en un
//! `Vec` aparte mientras la máquina de turnos dice «sigue hablando». Es más fácil de escribir y
//! deja la voz del cliente viviendo en dos sitios: el anillo y ese `Vec`. Dos sitios son dos
//! vaciados, y el segundo es el que alguien olvidará el día que añada una pieza. Así que el audio
//! vive **solo** en el anillo, y el turno se pide por su sitio en el tiempo cuando ya terminó.
//!
//! **Dos: transcribir no puede parar la escucha.** El motor tarda unos 250 ms en un turno de seis
//! segundos. Hacerlo en el mismo hilo que mira los marcos dejaría a la otra pista ciega durante
//! ese cuarto de segundo — justo cuando el otro interlocutor empieza a responder, que es el
//! momento más probable. Hay dos hilos: uno mira, otro transcribe, y entre ellos un canal.
//!
//! **Tres: lo que no se pudo hacer se cuenta.** Si un turno se pierde porque su audio ya se pisó,
//! o si el motor no está, la novedad que sale de aquí lo dice con nombre. La banda no puede
//! quedarse en blanco sin que nadie sepa por qué.

use crate::capture::anillo::{Anillo, HZ};
use crate::capture::Pista;
use crate::disparo::{Contexto, Disparador};
use crate::ficha::{Aparicion, Respuesta};
use crate::stt::{Disponibilidad, Fallo, Motor, Turno, Ventana};
use crate::voz::{Suceso, Turnos, MARCO};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Cada cuánto mira el hilo de escucha si hay marcos nuevos. Cuarenta milisegundos son dos marcos:
/// lo bastante a menudo para que el fin de turno no se retrase de forma apreciable frente a los
/// 320 ms que ya cuesta decidirlo, y lo bastante poco para no despertar la CPU cien veces por
/// segundo durante una reunión de una hora.
const LATIDO_MS: u64 = 40;

/// Lo que pasa mientras se escucha. Sale de aquí hacia quien quiera enterarse — en la app, hacia
/// la banda y la pantalla de Honestidad.
#[derive(Clone, Debug, serde::Serialize)]
// `rename_all` solo toca los NOMBRES de las variantes; los campos de dentro seguían en
// snake_case mientras el resto del contrato es camelCase. Nadie los leía todavía, y así es
// como una inconsistencia espera a que alguien la encuentre en producción.
#[serde(rename_all = "kebab-case", rename_all_fields = "camelCase", tag = "que")]
pub enum Novedad {
    /// Alguien empezó a hablar en esta pista.
    Empieza { pista: Pista },
    /// Un turno terminó y ya tiene texto.
    Turno(Turno),
    /// Un turno terminó y **no** se pudo transcribir. Con su motivo.
    SinTexto { pista: Pista, desde_ms: usize, hasta_ms: usize, motivo: String },
    /// El detector oyó algo demasiado corto para ser un turno.
    Ruido { pista: Pista, duracion_ms: usize },
    /// El disparador decidió que había que buscar, y esto es lo que salió: una ficha del corpus
    /// del usuario, o la declaración de que no hay nada con su maniobra.
    Aparece(Box<Aparicion>),
}

/// Lo que la pantalla de Honestidad enseña de una pista.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDePista {
    /// ¿Se pudo abrir el grifo? **Esto, y no el número de muestras, es lo que distingue una avería
    /// de un silencio.** Cuando nadie habla, macOS no entrega ni una muestra: cero no es un fallo.
    pub abierta: bool,
    /// Si no se pudo abrir, por qué. En español, para enseñarlo tal cual.
    pub motivo: Option<String>,
    pub bytes: usize,
    /// Los mismos bytes, escritos como los escribe la maqueta («1,8 MB»). Se formatean **aquí y no
    /// en la interfaz** porque ya hay un formateador probado en `red`, y dos formateadores acaban
    /// discrepando el día que uno redondee distinto.
    pub legible: String,
    pub segundos: f32,
    pub muestras_recibidas: u64,
    pub hablando: bool,
}

/// Todo lo que vive en memoria ahora mismo por culpa de la escucha.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDeEscucha {
    pub escuchando: bool,
    pub microfono: EstadoDePista,
    pub sistema: EstadoDePista,
    pub turnos_en_memoria: usize,
    pub bytes_del_transcript: usize,
    /// Lo que suman los tres búferes, para el «RAM · …» de la cabecera. Sumado, no estimado.
    pub ram_legible: String,
    /// Qué motor transcribe, y si puede. Lo enseña la pantalla de Idioma.
    pub motor: &'static str,
}

struct PistaViva {
    cual: Pista,
    anillo: Arc<Mutex<Anillo>>,
    /// Se conserva para que el grifo siga abierto: soltarlo lo cierra.
    _grifo: Option<crate::capture::nativo::Grifo>,
    motivo: Option<String>,
    turnos: Turnos,
    /// Índice global de la primera muestra que esta pista vio.
    origen: u64,
    /// Cuántas muestras se han metido ya en la máquina de turnos.
    procesadas: u64,
    /// Muestras que llegaron pero no llenaron un marco completo.
    sobrante: Vec<f32>,
    idioma: String,
    /// **Dónde cae el cero de ESTA pista en el reloj de la escucha.**
    ///
    /// Cada pista cuenta su tiempo con sus propias muestras, y ese reloj vuelve a cero cada vez que
    /// su anillo da la vuelta y hay que reengancharse al presente. Mientras las dos pistas
    /// arrancaran juntas y ninguna se reenganchara, sus relojes coincidían por casualidad — y en
    /// esa casualidad se apoyaban dos cosas: el detector de eco, que compara un turno del micrófono
    /// con los del sistema, y el disparador, que mide la espera entre fichas. Tras un reenganche de
    /// una sola pista los dos comparaban tiempos de relojes distintos. Hallazgo A5 de la auditoría.
    ///
    /// Con esto, lo que sale de la pista hacia el resto de la app va **en el reloj de la escucha**,
    /// que es uno solo y no vuelve atrás nunca.
    desfase_ms: usize,
}

impl PistaViva {
    fn abrir(cual: Pista, idioma: &str) -> Self {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let abierto = match cual {
            Pista::Microfono => crate::capture::nativo::Grifo::del_microfono(anillo.clone()),
            Pista::Sistema => crate::capture::nativo::Grifo::del_sistema(anillo.clone()),
        };
        let (grifo, motivo) = match abierto {
            Ok(g) => (Some(g), None),
            Err(e) => (None, Some(e)),
        };
        let origen = anillo.lock().map(|a| a.totales()).unwrap_or(0);
        Self {
            cual,
            anillo,
            _grifo: grifo,
            motivo,
            turnos: Turnos::default(),
            origen,
            procesadas: 0,
            sobrante: Vec::with_capacity(MARCO * 4),
            idioma: idioma.to_string(),
            // Las dos pistas se abren en el mismo instante que nace la escucha, así que su cero es
            // el cero del reloj común. Lo que lo mueve es el reenganche, y lo mueve `mirar`.
            desfase_ms: 0,
        }
    }

    /// Un instante del reloj de esta pista, traído al reloj de la escucha.
    fn en_el_reloj_comun(&self, ms: usize) -> usize {
        ms + self.desfase_ms
    }

    /// El índice global de la muestra que corresponde a un instante del reloj de esta pista.
    fn indice(&self, ms: usize) -> u64 {
        self.origen + (ms as u64 * HZ as u64) / 1000
    }

    fn estado(&self) -> EstadoDePista {
        let (bytes, segundos) = self
            .anillo
            .lock()
            .map(|a| (a.bytes(), a.segundos()))
            .unwrap_or((0, 0.0));
        EstadoDePista {
            abierta: self._grifo.is_some(),
            motivo: self.motivo.clone(),
            bytes,
            legible: crate::red::formatear(bytes as u64),
            segundos,
            muestras_recibidas: self._grifo.as_ref().map(|g| g.muestras_recibidas()).unwrap_or(0),
            hablando: self.turnos.hablando(),
        }
    }
}

/// LO QUE LA ESCUCHA NECESITA DEL CORPUS, y nada más.
///
/// La misma frontera que `stt::Motor`: la escucha no conoce a tantivy, conoce a quien sabe
/// buscar. Así se puede probar el camino entero —turno, disparo, ficha— sin índice, y así el
/// corpus se puede cambiar sin tocar una línea de aquí.
pub trait Buscador: Send + Sync {
    fn buscar(&self, texto: &str, cuantos: usize) -> Vec<crate::corpus::Hallazgo>;
    /// Las palabras distintivas del corpus, para el motivo «término tuyo» del disparador.
    fn vocabulario(&self) -> Vec<String>;
}

/// El buscador de primera clase para cuando no hay corpus: no encuentra nada y lo dice sin
/// romperse. Con él, la app funciona desde el primer arranque —sin carpeta señalada— y lo que
/// enseña es «no tengo nada» con su maniobra, que es la verdad.
pub struct SinCorpus;

impl Buscador for SinCorpus {
    fn buscar(&self, _texto: &str, _cuantos: usize) -> Vec<crate::corpus::Hallazgo> {
        Vec::new()
    }
    fn vocabulario(&self) -> Vec<String> {
        Vec::new()
    }
}

/// Lo que el hilo de escucha manda al hilo que transcribe.
struct Encargo {
    pista: Pista,
    idioma: String,
    desde_ms: usize,
    hasta_ms: usize,
    /// El audio del turno, ya recortado. Es la única copia que se hace, y vive lo que tarda el
    /// motor: se suelta en cuanto vuelve el texto.
    muestras: Option<Vec<f32>>,
    /// Cuándo cerró el turno. El presupuesto del sprint —ficha en ≤4 s— se cuenta desde aquí,
    /// que es el instante que el usuario percibe: el cliente terminó de hablar.
    cerro: std::time::Instant,
}

/// La escucha en marcha.
pub struct Escucha {
    /// El disparador vive aquí y no dentro del hilo porque **el kill-switch tiene que poder
    /// alcanzarlo**: guarda la última pregunta del cliente, y eso es contenido de terceros.
    disparador: Arc<Mutex<Disparador>>,
    pistas: Arc<Mutex<Vec<PistaViva>>>,
    ventana: Arc<Mutex<Ventana>>,
    viva: Arc<AtomicBool>,
    motor: &'static str,
}

impl Escucha {
    /// Abre las dos pistas y arranca los dos hilos.
    ///
    /// `avisar` recibe cada novedad. No se le pasa nada de la reunión que no sea lo que va a
    /// enseñarse: la firma es la frontera.
    pub fn arrancar(
        idioma_del_consultor: &str,
        idioma_del_cliente: &str,
        motor: Box<dyn Motor>,
        buscador: Arc<dyn Buscador>,
        avisar: impl Fn(Novedad) + Send + Sync + 'static,
    ) -> Self {
        // **El reloj de la escucha**: uno solo para las dos pistas, monótono, que no vuelve atrás
        // ni cuando un anillo da la vuelta. Los relojes de cada pista se cuentan en muestras y
        // valen para pedirle su trozo al anillo; para todo lo demás —el eco, el disparador, la
        // banda— manda este.
        let nacio = std::time::Instant::now();
        let pistas = Arc::new(Mutex::new(vec![
            PistaViva::abrir(Pista::Microfono, idioma_del_consultor),
            PistaViva::abrir(Pista::Sistema, idioma_del_cliente),
        ]));
        let ventana = Arc::new(Mutex::new(Ventana::nueva()));
        let viva = Arc::new(AtomicBool::new(true));
        let nombre_del_motor = motor.nombre();

        for p in pistas.lock().unwrap().iter() {
            match &p.motivo {
                Some(m) => println!("[escucha] pista «{}» NO abierta: {m}", p.cual.etiqueta()),
                None => println!("[escucha] pista «{}» abierta", p.cual.etiqueta()),
            }
        }

        let (manda, recibe): (Sender<Encargo>, Receiver<Encargo>) = std::sync::mpsc::channel();
        let avisar = Arc::new(avisar);
        let disparador = Arc::new(Mutex::new(Disparador::nuevo()));

        // Hilo 1 — mira los marcos y corta los turnos.
        {
            let pistas = pistas.clone();
            let viva = viva.clone();
            let avisar = avisar.clone();
            std::thread::spawn(move || {
                while viva.load(Ordering::Relaxed) {
                    if let Ok(mut lista) = pistas.lock() {
                        for p in lista.iter_mut() {
                            mirar(p, &manda, avisar.as_ref(), nacio);
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(LATIDO_MS));
                }
            });
        }

        // Hilo 2 — transcribe lo que el primero le pasa. Lo que hace con cada encargo vive en
        // `atender`, fuera del hilo, para poder probar el caso que este sprint no probaba: que el
        // corte llegue justo en medio.
        {
            let ventana = ventana.clone();
            let avisar = avisar.clone();
            let disparador = disparador.clone();
            let viva_aqui = viva.clone();
            std::thread::spawn(move || {
                for mut encargo in recibe {
                    let Some((novedad, aparicion)) = atender(
                        &mut encargo,
                        &*motor,
                        &*buscador,
                        &ventana,
                        &disparador,
                        &viva_aqui,
                    ) else {
                        continue;
                    };
                    // El turno va PRIMERO y la ficha después: el transcript de la banda se pinta
                    // en cuanto hay texto, sin esperar a una búsqueda que puede no llegar nunca.
                    avisar(novedad);
                    if let Some(a) = aparicion {
                        avisar(Novedad::Aparece(Box::new(a)));
                    }
                }
            });
        }

        Self { disparador, pistas, ventana, viva, motor: nombre_del_motor }
    }

    pub fn estado(&self) -> EstadoDeEscucha {
        let lista = self.pistas.lock().unwrap();
        let de = |cual: Pista| {
            lista
                .iter()
                .find(|p| p.cual == cual)
                .map(|p| p.estado())
                .unwrap_or(EstadoDePista {
                    abierta: false,
                    motivo: Some("esa pista no se abrió nunca".into()),
                    bytes: 0,
                    legible: crate::red::formatear(0),
                    segundos: 0.0,
                    muestras_recibidas: 0,
                    hablando: false,
                })
        };
        let (turnos, bytes) = self
            .ventana
            .lock()
            .map(|v| (v.cuantos(), v.bytes()))
            .unwrap_or((0, 0));
        let (microfono, sistema) = (de(Pista::Microfono), de(Pista::Sistema));
        let ram = (microfono.bytes + sistema.bytes + bytes) as u64;
        EstadoDeEscucha {
            escuchando: self.viva.load(Ordering::Relaxed),
            microfono,
            sistema,
            turnos_en_memoria: turnos,
            bytes_del_transcript: bytes,
            ram_legible: crate::red::formatear(ram),
            motor: self.motor,
        }
    }

    /// Los últimos turnos, para que la banda los pinte.
    pub fn ultimos_turnos(&self, cuantos: usize) -> Vec<Turno> {
        self.ventana
            .lock()
            .map(|v| v.ultimos(cuantos).into_iter().cloned().collect())
            .unwrap_or_default()
    }

    /// **El kill-switch.** Cierra los grifos, pisa los anillos y olvida el transcript.
    ///
    /// El orden importa y es el mismo que declara `corte::TODAS`: primero se cierra el grifo, y
    /// solo después se vacía el anillo. Al revés, el anillo volvería a tener muestras un
    /// instante más tarde — vaciar un vaso que todavía recibe agua.
    pub fn cortar(&self) {
        self.viva.store(false, Ordering::Relaxed);
        if let Ok(mut lista) = self.pistas.lock() {
            for p in lista.iter_mut() {
                p._grifo = None; // soltarlo cierra el grifo
            }
            for p in lista.iter_mut() {
                if let Ok(mut a) = p.anillo.lock() {
                    a.vaciar();
                }
                p.turnos.reiniciar();
                p.sobrante.clear();
                p.procesadas = 0;
                p.origen = 0;
            }
        }
        if let Ok(mut v) = self.ventana.lock() {
            v.vaciar();
        }
        // La última pregunta del cliente también es transcript, aunque viva en otro sitio.
        if let Ok(mut d) = self.disparador.lock() {
            d.reiniciar();
        }
    }
}

impl Drop for Escucha {
    fn drop(&mut self) {
        self.viva.store(false, Ordering::Relaxed);
    }
}

/// Saca del anillo lo que haya llegado y se lo da a la máquina de turnos, marco a marco.
fn mirar(
    p: &mut PistaViva,
    manda: &Sender<Encargo>,
    avisar: &dyn Fn(Novedad),
    nacio: std::time::Instant,
) {
    // `rango` distingue dos vacíos que no significan lo mismo, y de ahí sale toda la lógica de
    // abajo: `Some(vacío)` es «no ha entrado nada desde la última vez», que es lo normal cuando
    // nadie habla; `None` es «ese audio ya se pisó», que solo pasa si el anillo dio la vuelta
    // entera —treinta segundos— entre dos latidos de cuarenta milisegundos.
    let (nuevas, totales) = {
        let Ok(a) = p.anillo.lock() else { return };
        let totales = a.totales();
        (a.rango(p.origen + p.procesadas, totales), totales)
    };
    let Some(nuevas) = nuevas else {
        // Volver a engancharse al presente, **diciéndolo**. Que la app se salte medio minuto de
        // reunión y nadie se entere es exactamente el silencio que esta casa no se permite; al
        // log va el hecho y los segundos, nunca lo que se dijo.
        let perdidos = (totales.saturating_sub(p.origen + p.procesadas)) as f32 / HZ as f32;
        println!(
            "[escucha] la pista «{}» se quedó atrás {perdidos:.1}s: ese audio ya se pisó y no se \
             va a transcribir",
            p.cual.etiqueta()
        );
        p.origen = totales;
        p.procesadas = 0;
        p.sobrante.clear();
        // **Y el reloj de los turnos se reinicia CON el origen.** `indice()` traduce un instante
        // del reloj de la pista a una muestra del anillo sumando los dos, así que moverle uno solo
        // dejaría cada turno pidiendo el trozo equivocado: se transcribiría un momento de la
        // reunión creyendo que es otro. Eso es peor que perder el audio — es inventarlo.
        p.turnos.reiniciar();
        // **Y el desfase con el reloj común se recalcula aquí.** El cero de esta pista pasa a ser
        // este instante; sin esto, sus turnos volverían a contar desde cero y el resto de la app
        // —el eco, que compara las dos pistas, y el disparador, que mide la espera entre fichas—
        // compararía tiempos de dos relojes distintos.
        p.desfase_ms = nacio.elapsed().as_millis() as usize;
        return;
    };
    if nuevas.is_empty() && p.sobrante.len() < MARCO {
        return;
    }
    p.procesadas += nuevas.len() as u64;
    p.sobrante.extend_from_slice(&nuevas);

    let mut usados = 0;
    while p.sobrante.len() - usados >= MARCO {
        let marco: Vec<f32> = p.sobrante[usados..usados + MARCO].to_vec();
        usados += MARCO;
        let Some(suceso) = p.turnos.marco(&marco) else { continue };
        match suceso {
            Suceso::Empieza { .. } => avisar(Novedad::Empieza { pista: p.cual }),
            Suceso::DemasiadoCorto { duracion_ms } => {
                avisar(Novedad::Ruido { pista: p.cual, duracion_ms })
            }
            Suceso::Termina { desde_ms, hasta_ms } => {
                let muestras = p
                    .anillo
                    .lock()
                    .ok()
                    .and_then(|a| a.rango(p.indice(desde_ms), p.indice(hasta_ms)));
                let _ = manda.send(Encargo {
                    pista: p.cual,
                    idioma: p.idioma.clone(),
                    // El audio se pidió con el reloj de la pista, que es el que sabe qué muestra
                    // es cuál; lo que sale de aquí va en el reloj de la escucha, que es el único
                    // que las dos pistas comparten.
                    desde_ms: p.en_el_reloj_comun(desde_ms),
                    hasta_ms: p.en_el_reloj_comun(hasta_ms),
                    muestras,
                    cerro: std::time::Instant::now(),
                });
            }
        }
    }
    p.sobrante.drain(..usados);
}

/// ¿Es este turno del micrófono un reflejo de lo que el cliente acaba de decir por los altavoces?
///
/// Mira solo los turnos del sistema que siguen en la ventana; más atrás no hace falta, porque un
/// eco por definición ocurre a la vez que su origen.
fn huele_a_eco(turno: &Turno, ventana: &Ventana) -> bool {
    let del_sistema: Vec<crate::voz::Tramo> = ventana
        .ultimos(crate::stt::ventana::TURNOS)
        .into_iter()
        .filter(|t| t.pista == Pista::Sistema)
        .map(|t| crate::voz::Tramo {
            desde_ms: t.desde_ms,
            hasta_ms: t.hasta_ms,
            texto: &t.texto,
        })
        .collect();
    crate::voz::eco::es_eco(
        &crate::voz::Tramo {
            desde_ms: turno.desde_ms,
            hasta_ms: turno.hasta_ms,
            texto: &turno.texto,
        },
        &del_sistema,
    )
}

/// UN ENCARGO, ATENDIDO: de audio a novedad, y a ficha si toca. `None` cuando el corte ya llegó y
/// no hay nada que anunciar.
///
/// **Vive fuera del hilo para poder probar el corte a media transcripción.** El kill-switch no
/// alcanzaba a este camino: el audio que ya iba en vuelo se transcribía DESPUÉS del corte,
/// repoblaba la ventana recién vaciada y volvía a guardar en el disparador la pregunta que
/// `reiniciar()` había pisado. La banda acababa enseñando una ficha de una reunión que el usuario
/// había cortado. Lo encontró la auditoría del sprint (hallazgo A2).
///
/// Se comprueba **dos veces** porque son dos ventanas distintas: antes de empezar (el encargo
/// estaba en la cola) y al volver del motor (que tarda un cuarto de segundo, y la tecla se pulsa
/// cuando el usuario quiere). En los dos casos el audio del encargo se pisa: es la única copia que
/// existe fuera del anillo, así que el `vaciar()` del corte no lo alcanza.
fn atender(
    encargo: &mut Encargo,
    motor: &dyn Motor,
    buscador: &dyn Buscador,
    ventana: &Mutex<Ventana>,
    disparador: &Mutex<Disparador>,
    viva: &AtomicBool,
) -> Option<(Novedad, Option<Aparicion>)> {
    if !viva.load(Ordering::Relaxed) {
        olvidar(encargo);
        return None;
    }
    let mut novedad = transcribir(motor, encargo);
    if !viva.load(Ordering::Relaxed) {
        olvidar(encargo);
        if let Novedad::Turno(t) = &mut novedad {
            // Las letras se pisan antes de soltarlas, igual que en la ventana de turnos y en el
            // disparador: es texto del cliente.
            // SEGURIDAD: se escriben ceros sobre bytes que ya eran UTF-8 válido, y el cero también
            // lo es, así que la cadena sigue siendo válida en todo momento.
            unsafe { t.texto.as_mut_vec() }.fill(0);
        }
        return None;
    }

    let mut aparicion = None;
    if let Novedad::Turno(t) = &mut novedad {
        // El eco se decide **con la ventana delante**: hace falta saber qué dijo el cliente para
        // saber si el micrófono lo está repitiendo. Por eso vive aquí y no en `transcribir`, que no
        // conoce a nadie más.
        if t.pista == Pista::Microfono {
            if let Ok(v) = ventana.lock() {
                t.eco = huele_a_eco(t, &v);
            }
        }
        if let Ok(mut v) = ventana.lock() {
            v.empujar(t.clone());
        }
        if let Ok(mut d) = disparador.lock() {
            aparicion = buscar_si_toca(&mut d, buscador, t, encargo);
        }
    }
    Some((novedad, aparicion))
}

/// Del turno del cliente a la ficha — o a la declaración de que no hay nada.
///
/// **La latencia se mide de verdad**, desde que el turno cerró hasta que la ficha está armada:
/// es el trayecto que el usuario percibe y el que el presupuesto del sprint acota en 4 s. No se
/// estima ni se promedia: cada aparición trae la suya.
fn buscar_si_toca(
    disparador: &mut Disparador,
    buscador: &dyn Buscador,
    turno: &Turno,
    encargo: &Encargo,
) -> Option<Aparicion> {
    let vocabulario = buscador.vocabulario();
    let ctx = Contexto { ahora_ms: turno.hasta_ms, vocabulario: &vocabulario };
    let motivo = disparador.mirar(turno, &ctx)?;

    let hallazgos = buscador.buscar(&turno.texto, crate::ficha::TOP);
    let respuesta = crate::ficha::armar(&turno.texto, &hallazgos);
    let ms = encargo.cerro.elapsed().as_millis() as u64;

    // Metadata, jamás contenido: ni la pregunta ni la ficha pasan por el log.
    let que = match &respuesta {
        Respuesta::Ficha(_) => "ficha",
        Respuesta::SinResultado { .. } => "sin resultado",
    };
    println!("[ficha] {que} por «{}» en {ms} ms · {} candidatas", motivo.etiqueta(), hallazgos.len());

    Some(Aparicion { respuesta, motivo, ms, hora: turno.hora.clone() })
}

/// Pisa el audio de un encargo que no se va a transcribir.
///
/// `muestras` es **la única copia del audio de ese turno fuera del anillo**, y la hizo el hilo que
/// mira los marcos. Soltar el `Vec` dejaría sus bytes en el montón hasta que otra cosa los pisara;
/// aquí se pisan a mano, que es lo mismo que hace el anillo al vaciarse.
fn olvidar(encargo: &mut Encargo) {
    if let Some(m) = &mut encargo.muestras {
        m.fill(0.0);
        m.clear();
    }
}

fn transcribir(motor: &dyn Motor, encargo: &Encargo) -> Novedad {
    let Encargo { pista, idioma, desde_ms, hasta_ms, muestras, cerro: _ } = encargo;
    let (pista, desde_ms, hasta_ms) = (*pista, *desde_ms, *hasta_ms);
    let sin_texto = |motivo: String| Novedad::SinTexto { pista, desde_ms, hasta_ms, motivo };

    let Some(muestras) = muestras else {
        return sin_texto(
            "el audio de ese turno ya se había pisado: la reunión iba más rápido que la \
             transcripción"
                .into(),
        );
    };
    match motor.transcribir(idioma, muestras, HZ) {
        Ok(texto) if texto.trim().is_empty() => {
            sin_texto("el motor no reconoció palabras en ese turno".into())
        }
        Ok(texto) => {
            Novedad::Turno(Turno { pista, desde_ms, hasta_ms, texto, hora: la_hora(), eco: false })
        }
        Err(Fallo::NoDisponible(d)) => sin_texto(explicar(&d, idioma)),
        Err(Fallo::Motor(c)) => sin_texto(format!("el motor de transcripción falló ({c})")),
    }
}

/// La hora del reloj como «14:02», sin dependencias y sin fecha.
///
/// Sin fecha a propósito: la banda enseña la hora de un turno que dura lo que dura la reunión, y
/// un día concreto no le añade nada a nadie salvo precisión sobre cuándo ocurrió una conversación.
fn la_hora() -> String {
    let segundos_desde_epoca = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    // La zona horaria del sistema, en segundos, sacada de la diferencia entre el reloj local y UTC
    // tal y como la cuenta `libc`. Traer una biblioteca de fechas entera para escribir «14:02»
    // sería desproporcionado.
    let desplazamiento = desplazamiento_local();
    let del_dia = (segundos_desde_epoca as i64 + desplazamiento).rem_euclid(86_400);
    format!("{:02}:{:02}", del_dia / 3600, (del_dia % 3600) / 60)
}

#[cfg(unix)]
fn desplazamiento_local() -> i64 {
    // `localtime_r` rellena `tm_gmtoff` con los segundos que la zona local lleva sobre UTC.
    unsafe {
        let ahora: libc::time_t = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        if libc::localtime_r(&ahora, &mut tm).is_null() {
            return 0;
        }
        tm.tm_gmtoff as i64
    }
}

#[cfg(not(unix))]
fn desplazamiento_local() -> i64 {
    0
}

/// Traduce el estado del motor a algo que se pueda enseñar. Es la última frontera antes de la
/// pantalla, y por eso aquí se escribe en español llano y no en jerga.
pub fn explicar(d: &Disponibilidad, idioma: &str) -> String {
    match d {
        Disponibilidad::Listo => "listo".into(),
        Disponibilidad::SinModelo => {
            format!("el modelo de {idioma} no está instalado en este Mac")
        }
        Disponibilidad::IdiomaDesconocido => {
            format!("este Mac no sabe transcribir {idioma}")
        }
        Disponibilidad::SinMotor { motivo } => motivo.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// El invariante que hace que un turno se transcriba con SU audio y no con el de otro
    /// momento: `indice()` suma el origen de la pista y el reloj de los turnos, así que los dos
    /// tienen que moverse a la vez.
    ///
    /// **Y este test tuvo que escribirse dos veces.** La primera versión movía el origen y
    /// reiniciaba el reloj *a mano* y luego comprobaba que cuadraban — comprobaba su propia
    /// aritmética, no el código, y por eso pasó en verde con el defecto puesto. La buena hace lo
    /// único que sirve: **llama a `mirar()`** con el anillo ya dado la vuelta, y deja que sea el
    /// código el que decida.
    ///
    /// Se ve en rojo quitando el `p.turnos.reiniciar()` de la rama del reenganche.
    #[test]
    fn al_reengancharse_el_reloj_y_el_origen_se_mueven_juntos() {
        let anillo = Arc::new(Mutex::new(crate::capture::Anillo::de_la_app()));
        let mut p = PistaViva {
            cual: Pista::Sistema,
            anillo: anillo.clone(),
            _grifo: None,
            motivo: None,
            turnos: Turnos::default(),
            origen: 0,
            procesadas: 0,
            sobrante: Vec::new(),
            idioma: "es-ES".into(),
            desfase_ms: 0,
        };
        // La pista lleva un rato vista: el reloj de los turnos ha avanzado.
        for _ in 0..100 {
            p.turnos.marco(&[0.0; MARCO]);
        }
        p.procesadas = 100 * MARCO as u64;
        assert_eq!(p.indice(p.turnos.reloj_ms()), donde_va_el_reloj(&p));

        // Y ahora el anillo da la vuelta ENTERA sin que nadie mire: tanto que lo más viejo que le
        // queda dentro es más nuevo que lo que la pista tenía pendiente. Ese audio ya no existe.
        {
            let mut a = anillo.lock().unwrap();
            let capacidad = a.capacidad();
            a.escribir(&vec![0.0; capacidad + 40_000]);
        }

        let (manda, _recibe) = std::sync::mpsc::channel();
        mirar(&mut p, &manda, &|_| {}, std::time::Instant::now());

        assert_eq!(p.procesadas, 0, "el reenganche no llegó a ocurrir: el test no prueba nada");
        assert_eq!(
            p.indice(p.turnos.reloj_ms()),
            donde_va_el_reloj(&p),
            "tras reengancharse, el reloj de los turnos y el origen del audio apuntan a sitios \
             distintos: cada turno siguiente pediría el trozo equivocado y se transcribiría un \
             momento de la reunión creyendo que es otro"
        );
    }

    /// **Y tras el reenganche, lo que sale de la pista sigue en el reloj de la escucha.**
    ///
    /// El reloj de cada pista se cuenta en muestras y vuelve a cero al reengancharse; el de la
    /// escucha no. De que los dos no se confundan dependen dos cosas que fallaban en silencio: el
    /// detector de eco —que compara un turno del micrófono contra los del sistema, cada uno con su
    /// reloj— y la espera entre fichas del disparador. Hallazgo A5.
    ///
    /// Se ve en rojo devolviendo `desde_ms`/`hasta_ms` sin pasar por `en_el_reloj_comun`.
    #[test]
    fn tras_el_reenganche_los_turnos_siguen_en_el_reloj_de_la_escucha() {
        // La escucha lleva cinco segundos en marcha. Se finge hacia atrás para que el test no tenga
        // que esperarlos: lo que importa es que el desfase se lea del reloj común, no del de aquí.
        let nacio = std::time::Instant::now() - std::time::Duration::from_millis(5_000);
        let anillo = Arc::new(Mutex::new(crate::capture::Anillo::de_la_app()));
        let mut p = PistaViva {
            cual: Pista::Sistema,
            anillo: anillo.clone(),
            _grifo: None,
            motivo: None,
            turnos: Turnos::default(),
            origen: 0,
            procesadas: 0,
            sobrante: Vec::new(),
            idioma: "es-ES".into(),
            desfase_ms: 0,
        };
        let (manda, recibe) = std::sync::mpsc::channel();

        // El anillo da la vuelta entera sin que nadie mire: la pista se queda atrás y se reengancha.
        {
            let mut a = anillo.lock().unwrap();
            let capacidad = a.capacidad();
            a.escribir(&vec![0.0; capacidad + 40_000]);
        }
        mirar(&mut p, &manda, &|_| {}, nacio);
        assert_eq!(p.procesadas, 0, "el reenganche no llegó a ocurrir: el test no prueba nada");
        assert!(
            (4_950..=5_200).contains(&p.desfase_ms),
            "el cero de la pista no se colocó donde va el reloj de la escucha: {} ms",
            p.desfase_ms
        );

        // Y ahora alguien habla. El orden importa y lo enseñó este test en rojo: **medio segundo
        // de sala callada primero**. El detector por energía dedica sus primeros 25 marcos a medir
        // el silencio de esta sala, y si lo primero que oye es la frase, aprende que la frase es el
        // silencio y no vuelve a oír nada. Es el fallo que `vad.rs` declara —si la app arranca con
        // alguien ya hablando, pierde ese turno— visto desde el arnés.
        {
            let mut a = anillo.lock().unwrap();
            let hz = crate::capture::anillo::HZ as usize;
            a.escribir(&vec![0.0; hz * 6 / 10]);
            a.escribir(&voz(hz));
            a.escribir(&vec![0.0; hz / 2]);
        }
        mirar(&mut p, &manda, &|_| {}, nacio);

        let encargo = recibe.try_recv().expect("el turno no se cerró: el test no prueba nada");
        assert!(
            encargo.desde_ms >= 4_950,
            "el turno salió con el reloj de la pista ({} ms) y no con el de la escucha: el eco y \
             la espera entre fichas compararían relojes distintos",
            encargo.desde_ms
        );
    }

    /// «Voz» para el detector por energía: una onda bien por encima del suelo de ruido. No pretende
    /// parecerse a una voz — el detector mira energía, no timbre.
    fn voz(muestras: usize) -> Vec<f32> {
        (0..muestras)
            .map(|i| {
                let t = i as f32 / crate::capture::anillo::HZ as f32;
                0.5 * (2.0 * std::f32::consts::PI * 180.0 * t).sin()
            })
            .collect()
    }

    /// Qué muestra del anillo corresponde al instante en que va el reloj de los turnos.
    ///
    /// No es `origen + procesadas` a secas, y el matiz importa: `procesadas` cuenta lo que se sacó
    /// del anillo, y el reloj solo avanza con **marcos completos**. Lo que sobra esperando a
    /// completar el marco siguiente vive en `sobrante` y todavía no ha llegado al reloj.
    fn donde_va_el_reloj(p: &PistaViva) -> u64 {
        p.origen + p.procesadas - p.sobrante.len() as u64
    }

    #[test]
    fn cada_motivo_se_explica_en_castellano_y_sin_codigos() {
        let casos = [
            Disponibilidad::SinModelo,
            Disponibilidad::IdiomaDesconocido,
            Disponibilidad::SinMotor { motivo: "este Mac no trae el transcriptor".into() },
        ];
        for d in casos {
            let texto = explicar(&d, "es-ES");
            assert!(texto.len() > 15, "«{texto}» no explica nada");
            assert!(
                !texto.contains("SinModelo") && !texto.contains('{'),
                "«{texto}» está enseñando el nombre del tipo"
            );
        }
    }

    /// EL CORTE A MEDIA TRANSCRIPCIÓN. `⌥⎋` no alcanzaba a este camino: el turno que ya estaba en
    /// la cola —o dentro del motor— se transcribía después, repoblaba la ventana recién vaciada y
    /// volvía a dejar la pregunta del cliente en el disparador. El usuario cortaba y la banda
    /// seguía enseñando su reunión.
    ///
    /// Se ve en rojo quitando cualquiera de las dos comprobaciones de `viva` en `atender`.
    #[test]
    fn el_corte_alcanza_al_turno_que_ya_estaba_en_vuelo() {
        /// Un motor que devuelve texto siempre — y que, si se le da un interruptor, **lo apaga
        /// mientras transcribe**. Eso es el caso real: el motor tarda un cuarto de segundo y el
        /// usuario pulsa `⌥⎋` cuando le da la gana, no entre dos turnos.
        struct Loro(Option<Arc<AtomicBool>>, Arc<AtomicBool>);
        impl Motor for Loro {
            fn nombre(&self) -> &'static str {
                "loro"
            }
            fn disponibilidad(&self, _idioma: &str) -> Disponibilidad {
                Disponibilidad::Listo
            }
            fn instalar(&self, _idioma: &str) -> Disponibilidad {
                Disponibilidad::Listo
            }
            fn transcribir(&self, _idioma: &str, _muestras: &[f32], _hz: u32) -> Result<String, Fallo> {
                // Que este motor haya trabajado se nota: tras el corte, el audio del cliente no
                // tiene que llegar ni al transcriptor.
                self.1.store(true, Ordering::Relaxed);
                if let Some(interruptor) = &self.0 {
                    interruptor.store(false, Ordering::Relaxed);
                }
                Ok("¿Ustedes tienen certificación ISO 27001?".into())
            }
            fn techo_de_idiomas(&self) -> u32 {
                1
            }
            fn idiomas(&self) -> Vec<String> {
                vec!["es-ES".into()]
            }
        }

        let ventana = Mutex::new(Ventana::nueva());
        let disparador = Mutex::new(Disparador::nuevo());
        let viva = Arc::new(AtomicBool::new(true));
        let mut encargo = Encargo {
            pista: Pista::Sistema,
            idioma: "es-ES".into(),
            desde_ms: 0,
            hasta_ms: 2_000,
            // El audio del turno, que es la única copia fuera del anillo.
            muestras: Some(vec![0.7; 32_000]),
            cerro: std::time::Instant::now(),
        };

        // Primero, con la escucha viva: el turno llega a la ventana. Sin esto el test pasaría
        // también con un `atender` que no hiciera nada nunca.
        let trabajo = Arc::new(AtomicBool::new(false));
        let (novedad, _) = atender(
            &mut encargo,
            &Loro(None, trabajo.clone()),
            &SinCorpus,
            &ventana,
            &disparador,
            &viva,
        )
        .expect("con la escucha viva el turno tiene que llegar");
        assert!(matches!(novedad, Novedad::Turno(_)));
        assert_eq!(ventana.lock().unwrap().cuantos(), 1);

        /// Lo que tiene que quedar tras un corte, mire uno donde mire.
        fn nada_sobrevivio(
            encargo: &Encargo,
            ventana: &Mutex<Ventana>,
            devuelto: &Option<(Novedad, Option<Aparicion>)>,
            cuando: &str,
        ) {
            assert!(devuelto.is_none(), "{cuando}: se anunció una novedad de la reunión cortada");
            assert_eq!(
                ventana.lock().unwrap().cuantos(),
                0,
                "{cuando}: el turno repobló la ventana que el corte acababa de vaciar"
            );
            assert!(
                encargo.muestras.as_ref().is_none_or(|m| m.is_empty()),
                "{cuando}: el audio sobrevivió al corte, y es la única copia fuera del anillo"
            );
        }

        let mut en_vuelo = |desde_ms: usize| Encargo {
            pista: Pista::Sistema,
            idioma: "es-ES".into(),
            desde_ms,
            hasta_ms: desde_ms + 2_000,
            // El audio del turno, que es la única copia fuera del anillo.
            muestras: Some(vec![0.7; 32_000]),
            cerro: std::time::Instant::now(),
        };

        // **Caso uno: el corte llegó mientras el encargo esperaba su turno en la cola.**
        viva.store(false, Ordering::Relaxed);
        ventana.lock().unwrap().vaciar();
        disparador.lock().unwrap().reiniciar();
        let mut encolado = en_vuelo(2_400);
        trabajo.store(false, Ordering::Relaxed);
        let devuelto = atender(
            &mut encolado,
            &Loro(None, trabajo.clone()),
            &SinCorpus,
            &ventana,
            &disparador,
            &viva,
        );
        nada_sobrevivio(&encolado, &ventana, &devuelto, "esperando en la cola");
        assert!(
            !trabajo.load(Ordering::Relaxed),
            "el audio del cliente llegó al transcriptor DESPUÉS del corte: no es solo trabajo \
             gastado, es lo que la app promete no hacer"
        );

        // **Caso dos: el corte llega DENTRO del motor**, que es donde de verdad cae — el motor
        // tarda y la tecla se pulsa a mitad. Este caso es el que el primer arreglo no cubría: con
        // solo la comprobación de la entrada, el turno se anunciaba igual.
        viva.store(true, Ordering::Relaxed);
        let mut mientras_transcribia = en_vuelo(5_400);
        let devuelto = atender(
            &mut mientras_transcribia,
            &Loro(Some(viva.clone()), trabajo.clone()),
            &SinCorpus,
            &ventana,
            &disparador,
            &viva,
        );
        nada_sobrevivio(&mientras_transcribia, &ventana, &devuelto, "dentro del motor");
    }

    /// El turno perdido tiene su propia novedad, distinta del turno callado. Si los dos acabaran en
    /// el mismo sitio, la banda enseñaría un hueco y nadie sabría si el cliente calló o si la app
    /// se quedó atrás.
    #[test]
    fn un_turno_cuyo_audio_se_piso_no_se_confunde_con_uno_callado() {
        let motor = crate::stt::Mudo::por("sin motor de prueba");
        let perdido = transcribir(
            &motor,
            &Encargo {
                pista: Pista::Sistema,
                idioma: "es-ES".into(),
                desde_ms: 0,
                hasta_ms: 900,
                muestras: None,
                cerro: std::time::Instant::now(),
            },
        );
        match perdido {
            Novedad::SinTexto { motivo, .. } => assert!(motivo.contains("pisado")),
            otro => panic!("{otro:?}"),
        }
    }

    /// El eco se decide mirando la ventana, así que se prueba con la ventana montada: el mismo
    /// caso real que dio origen al módulo, ahora atravesando la escucha.
    #[test]
    fn el_micro_que_repite_al_cliente_sale_marcado_como_eco() {
        let mut v = Ventana::nueva();
        v.empujar(Turno {
            pista: Pista::Sistema,
            desde_ms: 740,
            hasta_ms: 6_100,
            texto: "¿Tienen certificación ISO27.001 y la limpieza de datos eso está dentro del alcance.".into(),
            hora: "14:02".into(),
            eco: false,
        });
        let del_micro = Turno {
            pista: Pista::Microfono,
            desde_ms: 3_420,
            hasta_ms: 6_240,
            texto: "Y la limpieza de datos, eso está dentro del alcance?".into(),
            hora: "14:02".into(),
            eco: false,
        };
        assert!(huele_a_eco(&del_micro, &v));

        let de_verdad = Turno {
            pista: Pista::Microfono,
            desde_ms: 6_500,
            hasta_ms: 8_000,
            texto: "Sí, las tres fuentes que acordamos están incluidas".into(),
            hora: "14:03".into(),
            eco: false,
        };
        assert!(!huele_a_eco(&de_verdad, &v), "se marcó como eco una respuesta real del consultor");
    }

    /// EL CAMINO ENTERO, sin audio y sin disco: corpus indexado → turno del cliente → disparo →
    /// ficha. Es el trayecto que la app promete y el único test que lo recorre de una pieza.
    struct CorpusDePrueba(crate::corpus::Corpus);

    impl Buscador for CorpusDePrueba {
        fn buscar(&self, texto: &str, cuantos: usize) -> Vec<crate::corpus::Hallazgo> {
            self.0.buscar(texto, cuantos).unwrap_or_default()
        }
        fn vocabulario(&self) -> Vec<String> {
            self.0.vocabulario().to_vec()
        }
    }

    fn corpus_de_prueba() -> CorpusDePrueba {
        let c = crate::corpus::Corpus::en_memoria().unwrap();
        // Se mete por el índice directamente: lo que se prueba aquí es el camino de la escucha,
        // no los lectores de archivo, que ya tienen los suyos.
        CorpusDePrueba(c)
    }

    fn turno_del_cliente(texto: &str, hasta_ms: usize) -> Turno {
        Turno {
            pista: Pista::Sistema,
            desde_ms: hasta_ms.saturating_sub(2_000),
            hasta_ms,
            texto: texto.into(),
            hora: "14:02".into(),
            eco: false,
        }
    }

    fn encargo_cerrado_ahora() -> Encargo {
        Encargo {
            pista: Pista::Sistema,
            idioma: "es-ES".into(),
            desde_ms: 0,
            hasta_ms: 2_000,
            muestras: None,
            cerro: std::time::Instant::now(),
        }
    }

    #[test]
    fn del_turno_del_cliente_a_la_ficha_de_su_corpus() {
        use crate::corpus::seccion::Seccion;
        let corpus = corpus_de_prueba();
        corpus
            .0
            .indice_para_pruebas()
            .meter("/c/propuesta.md", "Páramo Azul · Propuesta", Some(crate::corpus::Unidad::Propuesta), false, &[
                Seccion {
                    titulo: Some("Plazo de entrega".into()),
                    texto: "La entrega completa toma cuatro semanas desde la firma del contrato."
                        .into(),
                },
            ])
            .unwrap();

        let mut d = Disparador::nuevo();
        let turno = turno_del_cliente("¿En cuántas semanas hacen la entrega completa?", 2_000);
        let a = buscar_si_toca(&mut d, &corpus, &turno, &encargo_cerrado_ahora())
            .expect("la pregunta del cliente no disparó");

        assert_eq!(a.motivo, crate::disparo::Motivo::Pregunta);
        let Respuesta::Ficha(f) = a.respuesta else { panic!("no encontró la sección que responde") };
        assert_eq!(f.fuente.seccion.as_deref(), Some("Plazo de entrega"));
        assert!(f.linea.contains("cuatro semanas"));
        // El presupuesto del sprint. Sin audio ni modelo esto son microsegundos; lo que este
        // número vigila es que no se cuele una espera en el camino.
        assert!(a.ms < 4_000, "la ficha tardó {} ms", a.ms);
    }

    /// Y el otro lado, que es el que más se va a ver: el corpus no tiene nada. La app dice qué
    /// buscó y ofrece una maniobra, en vez de enseñar la sección «menos mala».
    #[test]
    fn sin_nada_en_el_corpus_la_app_dice_que_busco_y_sugiere_como_conducirse() {
        let corpus = corpus_de_prueba();
        let mut d = Disparador::nuevo();
        let turno = turno_del_cliente("¿Ustedes tienen certificación ISO 27001?", 2_000);
        let a = buscar_si_toca(&mut d, &corpus, &turno, &encargo_cerrado_ahora()).expect("no disparó");

        let Respuesta::SinResultado { buscado, maniobra, .. } = a.respuesta else {
            panic!("armó una ficha sobre un corpus vacío")
        };
        assert!(buscado.contains("27001"), "no dice qué buscó: «{buscado}»");
        assert_eq!(maniobra, "credencial");
    }

    /// El primer arranque de la app: sin carpeta señalada no hay corpus, y la app **tiene que
    /// funcionar igual**. Lo que enseña entonces es «no tengo nada» con su maniobra, que es la
    /// verdad, y no una pantalla de error.
    #[test]
    fn sin_corpus_señalado_la_app_responde_igual_en_vez_de_romperse() {
        let mut d = Disparador::nuevo();
        let turno = turno_del_cliente("¿Cuánto costaría el proyecto completo?", 2_000);
        let a = buscar_si_toca(&mut d, &SinCorpus, &turno, &encargo_cerrado_ahora())
            .expect("sin corpus dejó de disparar");
        let Respuesta::SinResultado { maniobra, cercanas, .. } = a.respuesta else {
            panic!("armó una ficha sin corpus")
        };
        assert!(cercanas.is_empty());
        assert_eq!(maniobra, "cifra");
    }

    /// El eco con altavoces internos: el mismo turno llega por las dos pistas. Si el de micrófono
    /// disparara, la app buscaría y enseñaría la ficha dos veces por la misma pregunta.
    #[test]
    fn el_eco_del_propio_altavoz_no_dispara_una_segunda_ficha() {
        let corpus = corpus_de_prueba();
        let mut d = Disparador::nuevo();
        let del_cliente = turno_del_cliente("¿En cuántas semanas hacen la entrega?", 2_000);
        assert!(buscar_si_toca(&mut d, &corpus, &del_cliente, &encargo_cerrado_ahora()).is_some());

        let por_el_microfono = Turno { pista: Pista::Microfono, eco: true, ..del_cliente };
        assert!(
            buscar_si_toca(&mut d, &corpus, &por_el_microfono, &encargo_cerrado_ahora()).is_none(),
            "el eco disparó una segunda ficha por la misma pregunta"
        );
    }

    #[test]
    fn sin_motor_el_turno_sale_con_el_motivo_del_motor() {
        let motor = crate::stt::Mudo::por("este Mac no trae el transcriptor de macOS 26");
        let novedad = transcribir(
            &motor,
            &Encargo {
                pista: Pista::Sistema,
                idioma: "es-ES".into(),
                desde_ms: 0,
                hasta_ms: 900,
                muestras: Some(vec![0.1; 14_400]),
                cerro: std::time::Instant::now(),
            },
        );
        match novedad {
            Novedad::SinTexto { motivo, .. } => assert!(motivo.contains("macOS 26"), "«{motivo}»"),
            otro => panic!("{otro:?}"),
        }
    }
}
