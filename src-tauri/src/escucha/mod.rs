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
use crate::diccionario::Diccionario;
use crate::disparo::{Contexto, Disparador, Motivo};
use crate::ficha::{Aparicion, Respuesta};
use crate::stt::{Disponibilidad, Fallo, Motor, Turno, Ventana};
use crate::voz::{Suceso, Turnos, MARCO};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex};

/// Cada cuánto mira el hilo de escucha si hay marcos nuevos. Cuarenta milisegundos son dos marcos:
/// lo bastante a menudo para que el fin de turno no se retrase de forma apreciable frente a los
/// 320 ms que ya cuesta decidirlo, y lo bastante poco para no despertar la CPU cien veces por
/// segundo durante una reunión de una hora.
const LATIDO_MS: u64 = 40;

/// Cada cuánto se pregunta el hilo que transcribe si el cliente lleva demasiado rato callado.
///
/// El umbral del disparo por silencio son cuatro segundos, así que cuatrocientos milisegundos son
/// un diez por ciento de retraso en el peor caso — imperceptible al lado de los 320 ms que ya
/// cuesta decidir un fin de turno. Y el hilo sigue pasando el resto del tiempo **dormido en el
/// canal**, igual que antes: esto no es un bucle nuevo, es el mismo con un despertador.
const LATIDO_DEL_SILENCIO_MS: u64 = 400;

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
    /// **El usuario pidió leer la pantalla (`⌃⌥L`) y no había texto** —una cámara, un vídeo, una
    /// pantalla en negro—. Se contesta igual, porque alguien preguntó: sin esto la tecla parecería
    /// rota (mirada 17-quater, «pantalla · nada que leer»).
    NadaEnPantalla { hora: String },
}

/// Lo que la pantalla de Honestidad enseña de una pista.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDePista {
    /// ¿Se pudo abrir el grifo? **Esto, y no el número de muestras, es lo que distingue una avería
    /// de un silencio.** Cuando nadie habla, macOS no entrega ni una muestra: cero no es un fallo.
    pub abierta: bool,
    /// Si no se pudo abrir, **por qué, en un conjunto cerrado** (mirada 17-quater). Sesión lo pinta
    /// con su frase y su salida; el detalle técnico se queda en el log.
    pub motivo: Option<crate::capture::PorQueNoAbrio>,
    pub bytes: usize,
    // **`legible` salió del contrato en el sprint 002.** Mandaba los mismos bytes ya escritos
    // («1,8 MB») con la razón de no tener dos formateadores; la fase 5 del sprint 001 descubrió que
    // hacían falta dos, porque el separador decimal es interfaz y este venía siempre con coma
    // —«1,8 MB» dentro de «What lives in memory now»—. Desde entonces la pantalla los formatea con
    // el idioma puesto y **este campo cruzaba la costura sin que nadie lo leyera**: uno de los
    // diecisiete huérfanos que el gate del contrato no puede ver, porque compara la forma y no si
    // alguien mira.
    //
    // **Los tres de abajo se quedan en Rust y dejan de cruzar** (decisión del usuario en la mirada
    // 17-quater: «4 fuera, 3 se ven»). `hablando` lo usa el modo solo audio para no hablar encima
    // de nadie; los otros dos son para el log. Ninguno tenía sitio en una pantalla.
    #[serde(skip)]
    pub segundos: f32,
    #[serde(skip)]
    pub muestras_recibidas: u64,
    #[serde(skip)]
    pub hablando: bool,
}

/// Todo lo que vive en memoria ahora mismo por culpa de la escucha.
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDeEscucha {
    pub escuchando: bool,
    pub microfono: EstadoDePista,
    pub sistema: EstadoDePista,
    /// Fuera del contrato por la misma decisión: Honestidad cuenta el transcript por sus bytes.
    #[serde(skip)]
    pub turnos_en_memoria: usize,
    pub bytes_del_transcript: usize,
    // `ram_legible` salió del contrato en el sprint 002, por lo mismo que `legible`: la cabecera
    // «RAM · …» suma los tres búferes y los escribe **en la pantalla**, con el idioma puesto.
    /// Qué motor transcribe. **No cruza**: Idioma lo lee de `que_sabe_transcribir`, que además dice
    /// cuántos idiomas admite y por qué falta. Cruzaba dos veces el mismo dato y el gate de lectores,
    /// que compara por nombre, daba por leída esta copia porque la otra sí lo estaba.
    #[serde(skip)]
    pub motor: &'static str,
}

struct PistaViva {
    cual: Pista,
    anillo: Arc<Mutex<Anillo>>,
    /// Se conserva para que el grifo siga abierto: soltarlo lo cierra.
    _grifo: Option<crate::capture::nativo::Grifo>,
    no_abrio: Option<crate::capture::NoAbrio>,
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
        let (grifo, no_abrio) = match abierto {
            Ok(g) => (Some(g), None),
            Err(e) => (None, Some(con_su_permiso(cual, e, &crate::permisos::leer()))),
        };
        let origen = anillo.lock().map(|a| a.totales()).unwrap_or(0);
        Self {
            cual,
            anillo,
            _grifo: grifo,
            no_abrio,
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
            motivo: self.no_abrio.as_ref().map(|n| n.porque),
            bytes,
            segundos,
            muestras_recibidas: self._grifo.as_ref().map(|g| g.muestras_recibidas()).unwrap_or(0),
            hablando: self.turnos.hablando(),
        }
    }
}

/// **Si una pista no abrió y su permiso no está concedido, el porqué es el permiso.**
///
/// El grifo solo ve el error de Core Audio, que sin permiso puede ser cualquier cosa —`!hog`
/// incluido—; quien sabe del permiso es `permisos`. El permiso manda porque su salida es la que
/// arregla las demás: con él negado, cerrar otra app no serviría de nada. «No se sabe» no cuenta
/// como negado: si la pregunta a macOS no contesta, se queda el porqué del grifo.
fn con_su_permiso(
    cual: Pista,
    mut e: crate::capture::NoAbrio,
    permisos: &crate::permisos::Permisos,
) -> crate::capture::NoAbrio {
    use crate::capture::PorQueNoAbrio::{SinPermisoDelAudio, SinPermisoDelMicrofono};
    use crate::permisos::Estado;
    let (estado, sin) = match cual {
        Pista::Microfono => (permisos.microfono, SinPermisoDelMicrofono),
        Pista::Sistema => (permisos.audio, SinPermisoDelAudio),
    };
    if matches!(estado, Estado::SinConceder | Estado::Denegado) {
        e.porque = sin;
    }
    e
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
    /// El reloj de la escucha y el buscador, para la ficha que pide la PANTALLA (C8): no nace de un
    /// turno, pero tiene que pasar por el mismo disparador —la misma espera entre fichas— y medirse
    /// con el mismo reloj que los turnos.
    nacio: std::time::Instant,
    buscador: Arc<dyn Buscador>,
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
        diccionario: Arc<Diccionario>,
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
            match &p.no_abrio {
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
        //
        // **Y desde el sprint 002 no espera el encargo siguiente para siempre.** El quinto motivo
        // del disparador —el silencio— es el único que no nace de un turno, así que necesitaba un
        // bucle que tictaquee cuando nadie habla. Es este, con `recv_timeout`: el hilo que ya
        // conoce al buscador y a la ventana, en vez del latido de los marcos, que no conoce a
        // ninguno de los dos.
        {
            let avisar = avisar.clone();
            let suyo = ElQueTranscribe {
                motor,
                buscador: buscador.clone(),
                diccionario,
                ventana: ventana.clone(),
                disparador: disparador.clone(),
                pistas: pistas.clone(),
                viva: viva.clone(),
                nacio,
                espera: std::time::Duration::from_millis(LATIDO_DEL_SILENCIO_MS),
            };
            std::thread::spawn(move || while suyo.latir(&recibe, avisar.as_ref()) {});
        }

        Self { disparador, pistas, ventana, viva, motor: nombre_del_motor, nacio, buscador }
    }

    /// **LA PANTALLA PIDE FICHA** (C8). La pantalla que comparte el cliente cambió y trae una cifra o
    /// uno de tus términos; `consulta` es lo que aporta.
    ///
    /// Pasa por el MISMO disparador que los turnos: si hace menos de seis segundos que salió una
    /// ficha, o si esta pantalla ya trajo la suya, no hay otra. Y **solo devuelve fichas**: una
    /// pantalla que no encuentra nada en tu corpus no interrumpe a nadie para decir «no tengo nada».
    /// Ese aviso es la respuesta a una pregunta, y aquí nadie preguntó.
    pub fn por_pantalla(&self, consulta: &str) -> Option<Aparicion> {
        if !self.viva.load(Ordering::Relaxed) {
            return None;
        }
        let empezo = std::time::Instant::now();
        let ahora_ms = self.nacio.elapsed().as_millis() as usize;
        let motivo = self.disparador.lock().ok()?.por_pantalla(consulta, ahora_ms)?;
        let a = armar_y_anunciar(self.buscador.as_ref(), consulta, motivo, &la_hora(), empezo);
        matches!(a.respuesta, Respuesta::Ficha(_)).then_some(a)
    }

    /// **La lectura que PIDIÓ el usuario** con su atajo. Como `⌃⌥A`: se salta la espera y responde
    /// siempre, también con «no tengo nada», porque alguien preguntó.
    pub fn pedida_por_pantalla(&self, consulta: &str) -> Option<Aparicion> {
        let empezo = std::time::Instant::now();
        let ahora_ms = self.nacio.elapsed().as_millis() as usize;
        let motivo = self.disparador.lock().ok()?.a_mano(ahora_ms);
        Some(armar_y_anunciar(self.buscador.as_ref(), consulta, motivo, &la_hora(), empezo))
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
                    motivo: Some(crate::capture::PorQueNoAbrio::NoDejo),
                    bytes: 0,
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
        EstadoDeEscucha {
            escuchando: self.viva.load(Ordering::Relaxed),
            microfono,
            sistema,
            turnos_en_memoria: turnos,
            bytes_del_transcript: bytes,
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

/// **LO QUE EL HILO QUE TRANSCRIBE TIENE EN LAS MANOS.**
///
/// Existe para que su latido viva **fuera** del hilo, que es la misma razón por la que `atender`
/// vive fuera: lo que no se puede llamar desde un test no se puede probar. Y aquí eso no es una
/// preferencia de estilo — es la deuda que este sprint paga: `Disparador::por_silencio` estuvo un
/// sprint entero escrito y **probado** sin un solo llamador, así que probar solo al ayudante y
/// dejar el cable a oscuras habría sido volver a dejar la misma deuda, esta vez con un test verde
/// encima.
struct ElQueTranscribe {
    motor: Box<dyn Motor>,
    buscador: Arc<dyn Buscador>,
    /// La jerga del consultor, para corregir el turno **antes** de que nadie lo mire.
    diccionario: Arc<Diccionario>,
    ventana: Arc<Mutex<Ventana>>,
    disparador: Arc<Mutex<Disparador>>,
    /// Solo para una pregunta: ¿está el cliente hablando ahora mismo?
    pistas: Arc<Mutex<Vec<PistaViva>>>,
    viva: Arc<AtomicBool>,
    /// El reloj de la escucha, el único que las dos pistas comparten.
    nacio: std::time::Instant,
    espera: std::time::Duration,
}

impl ElQueTranscribe {
    /// **Un latido**: atiende el encargo que haya llegado y, si no llegó ninguno, mira el silencio.
    /// Devuelve `false` cuando el canal se cerró y el hilo debe acabarse.
    fn latir(&self, recibe: &Receiver<Encargo>, avisar: &dyn Fn(Novedad)) -> bool {
        match recibe.recv_timeout(self.espera) {
            Ok(mut encargo) => {
                if let Some((novedad, aparicion)) = atender(
                    &mut encargo,
                    &*self.motor,
                    &*self.buscador,
                    &self.diccionario,
                    &self.ventana,
                    &self.disparador,
                    &self.viva,
                ) {
                    // El turno va PRIMERO y la ficha después: el transcript de la banda se pinta en
                    // cuanto hay texto, sin esperar a una búsqueda que puede no llegar nunca.
                    avisar(novedad);
                    if let Some(a) = aparicion {
                        avisar(Novedad::Aparece(Box::new(a)));
                    }
                }
                true
            }
            // Nadie habló en lo que dura la espera: el hueco donde vive el disparo por silencio.
            Err(RecvTimeoutError::Timeout) => {
                // **El corte llega también aquí.** Es la misma ventana del hallazgo A2: si el
                // usuario pulsó la tecla, lo último que puede pasar es que la banda estrene una
                // ficha de la reunión que acaba de cortar. El hilo se acaba un latido después, en
                // cuanto el primero suelte el canal.
                if self.viva.load(Ordering::Relaxed) {
                    if let Some(a) = el_silencio_pide_ficha(
                        &self.ventana,
                        &self.disparador,
                        &*self.buscador,
                        self.nacio.elapsed().as_millis() as usize,
                        self.cliente_hablando(),
                    ) {
                        avisar(Novedad::Aparece(Box::new(a)));
                    }
                }
                true
            }
            Err(RecvTimeoutError::Disconnected) => false,
        }
    }

    fn cliente_hablando(&self) -> bool {
        self.pistas
            .lock()
            .is_ok_and(|l| l.iter().any(|p| p.cual == Pista::Sistema && p.turnos.hablando()))
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
    diccionario: &Diccionario,
    ventana: &Mutex<Ventana>,
    disparador: &Mutex<Disparador>,
    viva: &AtomicBool,
) -> Option<(Novedad, Option<Aparicion>)> {
    if !viva.load(Ordering::Relaxed) {
        olvidar(encargo);
        return None;
    }
    let mut novedad = transcribir(motor, diccionario, encargo);
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
    Some(armar_y_anunciar(buscador, &turno.texto, motivo, &turno.hora, encargo.cerro))
}

/// **EL SILENCIO COMO DISPARO** — el quinto motivo, que existía sin cablear desde el sprint 001.
///
/// La VISION nombra cinco —«una pregunta, un término tuyo, una cifra o un silencio disparan la
/// ficha; y un atajo global»— y `Disparador::por_silencio` estaba escrito y probado desde la fase 4
/// **sin un solo llamador fuera de sus tests**. El manual lo declaraba como limitación en vez de
/// prometerlo, que fue lo honesto; esto es el pago.
///
/// **Qué ficha trae.** La de lo último que dijo el cliente, que es justo lo que no disparó solo: una
/// frase sin pregunta, sin cifra y sin término del corpus. Si esa frase ya trajo su ficha, el
/// silencio no trae una segunda igual — por eso `por_silencio` recibe el texto y no una cadena
/// vacía.
///
/// **Por qué la voz del cliente no se copia.** Lo último que dijo ya vive en la ventana del
/// transcript, que es el sitio que el kill-switch alcanza; la búsqueda se hace sobre una
/// **referencia**, con el candado de la ventana puesto, para no dejar una segunda copia en un hilo
/// al que `cortar()` no llega. El precio es que quien consulte el estado en ese instante espera lo
/// que dure la búsqueda, y en esta app ese es el lado correcto del trato.
fn el_silencio_pide_ficha(
    ventana: &Mutex<Ventana>,
    disparador: &Mutex<Disparador>,
    buscador: &dyn Buscador,
    ahora_ms: usize,
    cliente_hablando: bool,
) -> Option<Aparicion> {
    // Mientras el cliente habla no hay silencio que valga, aunque su turno anterior cerrara hace
    // rato: el que está en curso todavía no tiene fin, así que el reloj diría que lleva callado
    // justo cuando no lo está.
    if cliente_hablando {
        return None;
    }
    let empezo = std::time::Instant::now();
    // Los dos candados en este orden y en ningún otro: es el único sitio de la app donde se anidan.
    let v = ventana.lock().ok()?;
    let ultimo = v.ultimo_de(Pista::Sistema)?;
    let vocabulario = buscador.vocabulario();
    let ctx = Contexto { ahora_ms, vocabulario: &vocabulario };
    let motivo =
        disparador.lock().ok()?.por_silencio(&ultimo.texto, ultimo.hasta_ms, &ctx)?;
    Some(armar_y_anunciar(buscador, &ultimo.texto, motivo, &ultimo.hora, empezo))
}

/// De un texto del cliente a la aparición, con su medida y su línea de log.
///
/// Lo comparten los dos caminos que disparan —el turno que acaba de cerrarse y el silencio que vino
/// después— y por eso existe: dos copias de esto acabarían midiendo o anunciando distinto, y la
/// medida es la que el presupuesto de 4 s del sprint acota.
///
/// `desde` es el instante desde el que se mide, y no es el mismo en los dos casos: para un turno es
/// **cuándo cerró**, que es lo que el usuario percibe; para el silencio es cuándo se decidió
/// buscar, porque el fin de turno de ese texto quedó cuatro segundos atrás y contarlos sería
/// cargarle a la búsqueda una espera que es del diseño.
fn armar_y_anunciar(
    buscador: &dyn Buscador,
    texto: &str,
    motivo: Motivo,
    hora: &str,
    desde: std::time::Instant,
) -> Aparicion {
    let hallazgos = buscador.buscar(texto, crate::ficha::TOP);
    let respuesta = crate::ficha::armar(texto, &hallazgos);
    let ms = desde.elapsed().as_millis() as u64;

    // Metadata, jamás contenido: ni la pregunta ni la ficha pasan por el log.
    let que = match &respuesta {
        Respuesta::Ficha(_) => "ficha",
        Respuesta::SinResultado { .. } => "sin resultado",
    };
    println!("[ficha] {que} por «{}» en {ms} ms · {} candidatas", motivo.etiqueta(), hallazgos.len());

    Aparicion { respuesta, motivo, ms, hora: hora.to_string() }
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

fn transcribir(motor: &dyn Motor, diccionario: &Diccionario, encargo: &Encargo) -> Novedad {
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
            // **La corrección va AQUÍ y en ningún otro sitio**, que es donde el texto nace. Si
            // viviera más adelante —en la banda, o justo antes de buscar— el disparador vería
            // «power by» y no reconocería el término, el índice buscaría la errata, y la ventana del
            // transcript guardaría una cosa mientras la ficha se armó con otra. Un turno tiene una
            // sola forma, y esta es.
            let texto = diccionario.corregir(&texto);
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
pub(crate) fn la_hora() -> String {
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
        Disponibilidad::SinMotor { motivo } => motivo.en_el_log().into(),
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
            no_abrio: None,
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
            no_abrio: None,
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

    /// **El permiso manda sobre el error del grifo** — y «no se sabe» no manda. Sin permiso, Core
    /// Audio puede contestar cualquier cosa, `!hog` incluido; la salida útil es la del permiso.
    #[test]
    fn si_falta_el_permiso_el_porque_es_el_permiso() {
        use crate::capture::{NoAbrio, PorQueNoAbrio::*};
        use crate::permisos::{Estado, Permisos};
        let con = |microfono, audio| Permisos {
            microfono,
            audio,
            pantalla: Estado::Concedido,
            accesibilidad: Estado::Concedido,
        };
        let ocupado = || NoAbrio::por(DispositivoOcupado, "estado «!hog»");
        let sin_audio = con(Estado::Concedido, Estado::SinConceder);
        assert_eq!(con_su_permiso(Pista::Sistema, ocupado(), &sin_audio).porque, SinPermisoDelAudio);
        // El permiso de OTRA pista no cambia nada.
        assert_eq!(con_su_permiso(Pista::Microfono, ocupado(), &sin_audio).porque, DispositivoOcupado);
        let sin_micro = con(Estado::Denegado, Estado::Concedido);
        assert_eq!(con_su_permiso(Pista::Microfono, ocupado(), &sin_micro).porque, SinPermisoDelMicrofono);
        // «No se sabe» no es «no»: se queda el porqué del grifo.
        let no_se_sabe = con(Estado::Concedido, Estado::NoSeSabe);
        assert_eq!(con_su_permiso(Pista::Sistema, ocupado(), &no_se_sabe).porque, DispositivoOcupado);
        // Y el detalle viaja intacto al log.
        assert_eq!(con_su_permiso(Pista::Sistema, ocupado(), &sin_audio).detalle, "estado «!hog»");
    }

    #[test]
    fn cada_motivo_se_explica_en_castellano_y_sin_codigos() {
        let casos = [
            Disponibilidad::SinModelo,
            Disponibilidad::IdiomaDesconocido,
            Disponibilidad::SinMotor { motivo: crate::stt::PorQueNoHayMotor::SinTranscriptor },
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
            &Diccionario::default(),
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

        let en_vuelo = |desde_ms: usize| Encargo {
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
            &Diccionario::default(),
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
            &Diccionario::default(),
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
        let motor = crate::stt::Mudo::por(crate::stt::PorQueNoHayMotor::SinPuente);
        let perdido = transcribir(
            &motor,
            &Diccionario::default(),
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

    /// **El diccionario que no corrige nada.** Los tests de este módulo miden el camino del turno,
    /// no la jerga; con la semilla puesta, cualquiera que metiera «power by» en un texto de prueba
    /// vería cambiar el resultado por un motivo que no es el que está probando. El diccionario tiene
    /// sus propios tests, que sí miden lo suyo.
    fn sin_diccionario() -> Arc<Diccionario> {
        Arc::new(Diccionario::default())
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
        let motor = crate::stt::Mudo::por(crate::stt::PorQueNoHayMotor::SinTranscriptor);
        let novedad = transcribir(
            &motor,
            &Diccionario::default(),
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

    /// **EL QUINTO MOTIVO, CABLEADO — el camino entero del silencio.** El cliente dice algo que por
    /// sí solo no dispara, se calla, y la ficha llega sin que nadie más hable.
    ///
    /// Es la deuda del sprint 001: `por_silencio` estaba escrito y probado y **no tenía un solo
    /// llamador fuera de sus tests**, así que la app no disparó por silencio en toda la vida del
    /// sprint y el manual lo declaró como limitación. Se ve en rojo borrando la llamada a
    /// `el_silencio_pide_ficha` del brazo del `Timeout` — que es exactamente el estado en que el
    /// sprint 001 lo dejó.
    #[test]
    fn el_cliente_se_queda_callado_y_la_ficha_llega_sin_que_nadie_hable() {
        use crate::corpus::seccion::Seccion;
        let corpus = corpus_de_prueba();
        corpus
            .0
            .indice_para_pruebas()
            .meter("/c/marco.md", "Gobierno de datos · Marco", Some(crate::corpus::Unidad::Marco), false, &[
                Seccion {
                    titulo: Some("Traspaso en dos oleadas".into()),
                    texto: "El traspaso desde un proveedor anterior se hace en dos oleadas, sin \
                            cortar el servicio."
                        .into(),
                },
            ])
            .unwrap();

        let ventana = Mutex::new(Ventana::nueva());
        let disparador = Mutex::new(Disparador::nuevo());

        // Lo que el cliente dijo, y que por sí solo NO dispara: no es pregunta, no trae cifra y no
        // nombra nada del corpus. Si esto disparara, el test estaría probando otra cosa — así que
        // se comprueba, no se supone.
        let dicho = "Nosotros veníamos trabajando con el proveedor anterior";
        let turno = turno_del_cliente(dicho, 3_000);
        {
            let mut d = disparador.lock().unwrap();
            assert!(
                buscar_si_toca(&mut d, &corpus, &turno, &encargo_cerrado_ahora()).is_none(),
                "esa frase disparó sola: este test ya no prueba el silencio"
            );
        }
        ventana.lock().unwrap().empujar(turno);

        // Tres segundos callado todavía no son los cuatro que el disparador pide.
        let antes = el_silencio_pide_ficha(&ventana, &disparador, &corpus, 3_000 + 3_000, false);
        assert!(antes.is_none(), "disparó antes de que el silencio fuera largo");

        // Y con el cliente hablando no hay silencio, aunque el reloj diga que su último turno cerró
        // hace rato: el que está en curso no tiene fin todavía.
        assert!(
            el_silencio_pide_ficha(&ventana, &disparador, &corpus, 3_000 + 9_000, true).is_none(),
            "disparó con el cliente hablando encima"
        );

        let a = el_silencio_pide_ficha(&ventana, &disparador, &corpus, 3_000 + 9_000, false)
            .expect("el silencio no disparó: `por_silencio` se quedó otra vez sin llamador");
        assert_eq!(a.motivo, Motivo::SilencioLargo);
        let Respuesta::Ficha(f) = a.respuesta else {
            panic!("el silencio disparó pero no trajo la sección que responde")
        };
        assert_eq!(f.fuente.seccion.as_deref(), Some("Traspaso en dos oleadas"));
        assert!(a.ms < 4_000, "la ficha del silencio tardó {} ms", a.ms);

        // Y no insiste: el latido siguiente mira el mismo turno y se calla. Sin pasarle el texto a
        // `por_silencio` esto era una segunda ficha idéntica cada seis segundos.
        assert!(
            el_silencio_pide_ficha(&ventana, &disparador, &corpus, 3_000 + 30_000, false).is_none(),
            "el silencio repitió la ficha que acababa de enseñar"
        );
    }

    /// **EL CABLE, no el ayudante.** El latido de verdad: un canal real, un `recv_timeout` real que
    /// vence sin que llegue ningún encargo, y la aparición saliendo por `avisar` — que es el camino
    /// por el que la banda se enteraría.
    ///
    /// Este test existe porque el anterior no bastaba. `el_silencio_pide_ficha` probado y sin
    /// llamador sería **la misma deuda del sprint 001 otra vez**, esta vez con un test verde encima
    /// que la taparía. Se ve en rojo borrando la llamada a `el_silencio_pide_ficha` del brazo del
    /// `Timeout`: el ayudante sigue probado, y este test se cae.
    #[test]
    fn el_latido_sin_encargos_saca_la_ficha_del_silencio_por_donde_la_banda_la_oye() {
        use crate::corpus::seccion::Seccion;
        let corpus = Arc::new(corpus_de_prueba());
        corpus
            .0
            .indice_para_pruebas()
            .meter("/c/marco.md", "Gobierno de datos · Marco", Some(crate::corpus::Unidad::Marco), false, &[
                Seccion {
                    titulo: Some("Traspaso en dos oleadas".into()),
                    texto: "El traspaso desde un proveedor anterior se hace en dos oleadas.".into(),
                },
            ])
            .unwrap();

        let ventana = Arc::new(Mutex::new(Ventana::nueva()));
        ventana
            .lock()
            .unwrap()
            .empujar(turno_del_cliente("Nosotros veníamos trabajando con el proveedor anterior", 3_000));

        let suyo = ElQueTranscribe {
            motor: Box::new(crate::stt::Mudo::por(crate::stt::PorQueNoHayMotor::SinPuente)),
            buscador: corpus,
            diccionario: sin_diccionario(),
            ventana,
            disparador: Arc::new(Mutex::new(Disparador::nuevo())),
            // Sin pistas abiertas nadie está hablando, que es lo que este test necesita.
            pistas: Arc::new(Mutex::new(Vec::new())),
            viva: Arc::new(AtomicBool::new(true)),
            // La escucha lleva doce segundos en marcha: el turno cerró en el 3 000 y lleva nueve
            // callado. Se finge hacia atrás para que el test no tenga que esperarlos.
            nacio: std::time::Instant::now() - std::time::Duration::from_millis(12_000),
            espera: std::time::Duration::from_millis(20),
        };

        let (manda, recibe) = std::sync::mpsc::channel::<Encargo>();
        let dichas: Arc<Mutex<Vec<Novedad>>> = Arc::new(Mutex::new(Vec::new()));
        let apuntar = {
            let dichas = dichas.clone();
            move |n: Novedad| dichas.lock().unwrap().push(n)
        };

        // El canal sigue abierto y vacío: exactamente la reunión en la que nadie habla.
        assert!(suyo.latir(&recibe, &apuntar), "el latido se dio por acabado con el canal abierto");

        let salio = dichas.lock().unwrap();
        let [Novedad::Aparece(a)] = &salio[..] else {
            panic!("el latido no sacó la ficha del silencio: {:?}", salio)
        };
        assert_eq!(a.motivo, Motivo::SilencioLargo);

        // Y cuando el hilo que mira los marcos suelta el canal, este se da por acabado.
        drop(manda);
        assert!(!suyo.latir(&recibe, &apuntar), "el hilo se quedaría girando con el canal cerrado");
    }

    /// **Y el corte alcanza al latido.** Tras el kill-switch la banda no estrena una ficha de la
    /// reunión que el usuario acaba de cortar: es la ventana del hallazgo A2, en el camino nuevo.
    #[test]
    fn tras_el_corte_el_silencio_ya_no_saca_ninguna_ficha() {
        let ventana = Arc::new(Mutex::new(Ventana::nueva()));
        ventana
            .lock()
            .unwrap()
            .empujar(turno_del_cliente("Nosotros veníamos trabajando con el proveedor anterior", 3_000));

        let suyo = ElQueTranscribe {
            motor: Box::new(crate::stt::Mudo::por(crate::stt::PorQueNoHayMotor::SinPuente)),
            buscador: Arc::new(corpus_de_prueba()),
            diccionario: sin_diccionario(),
            ventana,
            disparador: Arc::new(Mutex::new(Disparador::nuevo())),
            pistas: Arc::new(Mutex::new(Vec::new())),
            viva: Arc::new(AtomicBool::new(false)),
            nacio: std::time::Instant::now() - std::time::Duration::from_millis(12_000),
            espera: std::time::Duration::from_millis(20),
        };

        let (_manda, recibe) = std::sync::mpsc::channel::<Encargo>();
        let dichas: Arc<Mutex<Vec<Novedad>>> = Arc::new(Mutex::new(Vec::new()));
        let apuntar = {
            let dichas = dichas.clone();
            move |n: Novedad| dichas.lock().unwrap().push(n)
        };
        suyo.latir(&recibe, &apuntar);
        assert!(
            dichas.lock().unwrap().is_empty(),
            "la banda estrenó una ficha después del kill-switch"
        );
    }

    /// Una reunión que arranca callada no estrena la banda con una ficha de la nada: sin nada dicho
    /// no hay silencio que interpretar.
    #[test]
    fn una_reunion_que_arranca_callada_no_dispara_nada() {
        let corpus = corpus_de_prueba();
        let ventana = Mutex::new(Ventana::nueva());
        let disparador = Mutex::new(Disparador::nuevo());
        assert!(el_silencio_pide_ficha(&ventana, &disparador, &corpus, 600_000, false).is_none());
    }

    /// Y el silencio del **consultor** no dispara nada: la ficha existe para responder a lo que
    /// preguntan del otro lado, y si la propia voz del usuario contara, la app le contestaría a él
    /// cada vez que se queda pensando. El último turno de la ventana es del micrófono; el disparo
    /// mira el del sistema, y no hay ninguno.
    #[test]
    fn el_silencio_del_consultor_no_dispara() {
        let corpus = corpus_de_prueba();
        let ventana = Mutex::new(Ventana::nueva());
        let disparador = Mutex::new(Disparador::nuevo());
        ventana.lock().unwrap().empujar(Turno {
            pista: Pista::Microfono,
            ..turno_del_cliente("Déjame mirar la propuesta que les mandamos", 3_000)
        });
        assert!(el_silencio_pide_ficha(&ventana, &disparador, &corpus, 3_000 + 9_000, false).is_none());
    }
}
