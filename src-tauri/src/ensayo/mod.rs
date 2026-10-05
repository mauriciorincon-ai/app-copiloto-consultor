//! EL ENSAYO (C18, sprint 004, ADR 019) — practicar la reunión antes de tenerla.
//!
//! Eliges el cliente y tu propuesta; la app te hace las preguntas que ese cliente probablemente hará,
//! te escucha responder y te dice qué evidencia de tu corpus usaste. **Código primero**: el banco sale
//! de reglas publicadas (`data/ensayo/`) y el modelo solo suma, si lo enciendes, un puñado de preguntas
//! fundadas en tus secciones.
//!
//! **MÓDULO PROTEGIDO** (`verify:ephemeral`): aquí se arma lo que se pregunta y se oye tu respuesta. Ni
//! disco ni red: los documentos los lee `lib.rs` (con `corpus`) y se los pasa ya troceados; lo que se
//! guarda de un ensayo lo escribe otro módulo, con la llave de tus notas.
//!
//! Las piezas: [`banco`] (las preguntas), [`enriquecer`] (el acento opt-in), [`sesion`] (la máquina de
//! estados, pura), [`oido`] (solo el micrófono), [`evaluacion`] (las cuatro cifras, pura) y [`guardado`] (lo
//! que queda de un ensayo, y su progreso: puro; lo escribe `ensayos.rs`). [`Ensayo`] las
//! lleva: un latido de 40 ms que pasa lo que oyó el micrófono a la sesión y ejecuta lo que ella pide. Lo
//! que necesita de la app —la voz, el corpus, la pantalla— entra por [`Mundo`], y así se prueba entero
//! sin Mac.

pub mod banco;
pub mod enriquecer;
pub mod evaluacion;
pub mod guardado;
pub mod oido;
pub mod sesion;

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;

use banco::{De, Idioma, Pregunta};
use enriquecer::{Enriquecido, PorQueNo};
use evaluacion::Evaluacion;
use oido::Oido;
use sesion::{Accion, Fase, Informe, Oyendo, Sesion, Suerte};

/// Cada cuánto late el ensayo: los mismos 40 ms que la escucha de la reunión.
const LATIDO: Duration = Duration::from_millis(40);

/// Lo que el ensayo necesita de la app, y nada más.
pub trait Mundo: Send + Sync + 'static {
    /// Empieza a leer la pregunta en voz alta. `false`: no hay voz para ese idioma.
    fn decir(&self, idioma: Idioma, texto: &str) -> bool;
    fn callar(&self);
    fn diciendo(&self) -> bool;
    /// Las fichas que el disparo habría enseñado para esta pregunta: el respaldo de `ficha::armar`.
    fn evidencia(&self, pregunta: &str) -> Vec<crate::ficha::Respaldo>;
    /// Cambió algo que la pantalla enseña: que vuelva a preguntar.
    fn avisar(&self);
}

/// Qué pasó con «Enriquecer el banco» en este ensayo (ADR 019 §4: la pantalla lo dice en una línea).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", rename_all_fields = "camelCase", tag = "que")]
pub enum EstadoDelBanco {
    /// El interruptor está apagado (el de fábrica): no se dice nada.
    Apagado,
    /// El modelo está en ello; el ensayo ya empezó con el banco por reglas.
    EnCamino,
    Sumadas { cuantas: usize },
    NoSeEnriquecio { porque: PorQueNo },
}

/// La pregunta como la pinta la pantalla: el texto y de dónde sale.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreguntaEnPantalla {
    pub texto: String,
    pub de: De,
    /// El título de la sección; en una objeción, `None`.
    pub seccion: Option<String>,
    /// La fuente publicada de una objeción.
    pub fuente: Option<String>,
}

impl From<&Pregunta> for PreguntaEnPantalla {
    fn from(p: &Pregunta) -> Self {
        Self { texto: p.texto.clone(), de: p.de, seccion: p.seccion.clone(), fuente: p.fuente.clone() }
    }
}

/// **Lo que la pantalla Ensayo enseña.** Se pide por comando y solo desde la ventana principal: el
/// evento `ensayo` es una señal sin dato.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VistaDelEnsayo {
    pub fase: Fase,
    pub cliente: String,
    /// La pregunta en curso, desde 0.
    pub indice: usize,
    pub total: usize,
    pub pregunta: Option<PreguntaEnPantalla>,
    /// La voz te está leyendo la pregunta.
    pub leyendo: bool,
    /// Pulsaste Enter y se espera la transcripción de lo último que dijiste.
    pub cerrando: bool,
    /// Lo que llevas dicho, en texto; en la evaluada, tu respuesta entera.
    pub respuesta: String,
    pub transcurrido_ms: u64,
    pub evaluacion: Option<Evaluacion>,
    /// En la evaluada, cuántas fichas usaste (citadas o «Sí lo dije»): la cuenta es de Rust, no de la
    /// pantalla (auditoría del S4, M12).
    pub usadas: usize,
    /// En la evaluada, cuántas muletillas; `None` si no hubo palabras que contar —sin modelo de voz, o una
    /// respuesta que no se transcribió—: cero en nada no es una cifra (§9-undecies).
    pub muletillas: Option<u32>,
    pub banco: EstadoDelBanco,
    pub informe: Option<Informe>,
}

/// Un documento tuyo que se puede elegir como propuesta.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Elegible {
    pub ruta: String,
    pub nombre: String,
}

/// «De dónde salen las preguntas»: cuántas da cada fuente con el tope elegido.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cuentas {
    pub propuesta: usize,
    pub ficha: usize,
    pub objeciones: usize,
}

impl Cuentas {
    pub fn de(banco: &[Pregunta]) -> Self {
        let n = |de: De| banco.iter().filter(|p| p.de == de).count();
        Self { propuesta: n(De::Propuesta), ficha: n(De::Ficha), objeciones: n(De::Objeciones) }
    }
}

/// **Lo que la pantalla «preparar» enseña**: con quién ensayas, con qué propuesta, cuántas preguntas
/// y de dónde salen. Se arma sin abrir nada: leer tus documentos no toca el micrófono.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Preparacion {
    /// Los clientes con ficha en tu corpus.
    pub clientes: Vec<String>,
    pub cliente: Option<String>,
    /// Tus propuestas que nombran a ese cliente.
    pub propuestas: Vec<Elegible>,
    /// La ruta de la elegida.
    pub propuesta: Option<String>,
    pub topes: Vec<usize>,
    pub tope: usize,
    pub cuentas: Cuentas,
    /// El de la propuesta (ADR 019 §6.7).
    pub idioma: Idioma,
    /// «Enriquecer el banco» en IA.
    pub enriquecer: bool,
    /// Este Mac sabe transcribir el idioma del ensayo. Sin eso, la app mide tu tiempo pero no tus palabras,
    /// y la pantalla lo dice antes de empezar.
    pub transcribe: bool,
    /// Ni propuesta ni ficha de ese cliente: no hay de dónde sacar preguntas.
    pub sin_corpus: bool,
    /// Cuántos ensayos tienes guardados con ese cliente, **contados por el nombre del archivo**, sin abrir
    /// ninguno (ADR 015, enmienda 4). Con uno o más, «Tu progreso con este cliente».
    pub guardados: usize,
}

/// Por qué el ensayo no empezó. Cerrado: la pantalla lo dice con su frase, en los dos idiomas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", rename_all_fields = "camelCase", tag = "que")]
pub enum NoEmpezo {
    /// Hay una reunión abierta: el ensayo y la reunión no conviven (ADR 019 §6.5).
    EnReunion,
    /// Sin propuesta ni ficha de ese cliente no hay preguntas.
    SinCorpus,
    /// El micrófono no se abrió, con su porqué de siempre.
    Microfono { porque: crate::capture::PorQueNoAbrio },
    /// Hay una videollamada abierta y el sonido sale por altavoces: tu micrófono la oiría y la llamada oiría
    /// la voz del ensayo (auditoría del S4, A1).
    Videollamada,
    /// Sin el permiso de Accesibilidad no se sabe si tu navegador está en una llamada, y el sonido sale por
    /// altavoces: se trata como si la hubiera.
    NoSeSabeSiHayLlamada,
}

/// **Con una videollamada abierta, el ensayo no empieza por altavoces que la app reconoce** (auditoría del S4,
/// A1; decisión del usuario del 2026-10-04: «con auriculares, sí»). En el ensayo no hay pista del sistema contra la que marcar el eco: con los
/// altavoces, el micrófono oiría a la otra parte y lo guardaría como tu respuesta —lo del cliente nunca se
/// guarda, regla dura 1—, y la llamada oiría la voz que lee tu propuesta. `eco` es lo que dice la salida de
/// audio (`Salida::puede_haber_eco`).
///
/// **Se para solo cuando la app SABE que se oiría** (`Some(true)`: los altavoces del Mac, un monitor por HDMI
/// o DisplayPort, AirPlay), con la misma regla que la voz en reunión (`habla::cabe_decirla`, H1). Unos
/// auriculares Bluetooth o USB no se distinguen de un altavoz por su conexión (`None`): pasan, como en
/// reunión, y el manual dice que con un altavoz así no se ensaya con una llamada abierta. Pararlos dejaba
/// fuera los AirPods, que son justo los auriculares de quien ensaya.
pub fn llamada_sin_auriculares(reunion: &crate::sesion::Reunion, eco: Option<bool>) -> Option<NoEmpezo> {
    use crate::sesion::Reunion;
    if eco != Some(true) {
        return None;
    }
    match reunion {
        Reunion::Ninguna => None,
        Reunion::Detectada { .. } => Some(NoEmpezo::Videollamada),
        Reunion::NoSePuedeSaber { .. } => Some(NoEmpezo::NoSeSabeSiHayLlamada),
    }
}

/// El idioma del ensayo: el de la propuesta; si no lo dice, el de la ficha; si empatan, el tuyo.
pub fn idioma_del_ensayo(propuesta: &[crate::corpus::seccion::Seccion], ficha: &[crate::corpus::seccion::Seccion], el_tuyo: Idioma) -> Idioma {
    banco::idioma_de(propuesta).or_else(|| banco::idioma_de(ficha)).unwrap_or(el_tuyo)
}

/// Con quién y cuándo: lo que nombra un ensayo al guardarlo (ADR 015, enmienda 4).
#[derive(Clone, Debug, PartialEq)]
pub struct Rotulo {
    pub cliente: String,
    /// El nombre de tu propuesta; vacío si las preguntas salen solo de la ficha del cliente.
    pub propuesta: String,
    /// La hora del Mac al empezar.
    pub empezo: crate::notas::Fecha,
}

/// Lo que el ensayo tiene en memoria ahora mismo: Honestidad lo cuenta.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Memoria {
    /// El anillo del micrófono del ensayo (30 s); cero desde que terminó.
    pub microfono: usize,
    /// Tus respuestas en texto, hasta que guardes o cierres.
    pub respuestas: usize,
}

struct Vivo {
    sesion: Sesion,
    /// `None` desde que el ensayo se cerró: el micrófono ya no está abierto.
    oido: Option<Oido>,
    rotulo: Rotulo,
    banco: EstadoDelBanco,
}

/// **EL ENSAYO EN MARCHA.**
pub struct Ensayo {
    vivo: Arc<Mutex<Option<Vivo>>>,
    mundo: Arc<dyn Mundo>,
    nacio: Instant,
    viva: Arc<AtomicBool>,
}

/// Ejecuta lo que la sesión pidió, y lo que eso a su vez pida, hasta que no quede nada.
fn ejecutar(v: &mut Vivo, mundo: &dyn Mundo, acciones: Vec<Accion>, ahora: u64) {
    let mut cola: VecDeque<Accion> = acciones.into();
    let mut avisar = false;
    while let Some(a) = cola.pop_front() {
        match a {
            Accion::Decir(texto) => {
                if !mundo.decir(v.sesion.idioma(), &texto) {
                    println!("[ensayo] no hay voz para el idioma del ensayo: se sigue sin leer");
                    cola.extend(v.sesion.sin_voz(ahora));
                }
            }
            Accion::Callar => mundo.callar(),
            Accion::Evaluar(c) => {
                let reloj = Instant::now();
                let respaldo = mundo.evidencia(&c.pregunta);
                let e = evaluacion::evaluar(&evaluacion::Entrada {
                    pregunta: &c.pregunta,
                    tramos: &c.tramos,
                    respaldo: &respaldo,
                    idioma: v.sesion.idioma(),
                    empezo_ms: c.empezo_ms,
                    cerro_ms: c.cerro_ms,
                });
                // Metadata, jamás contenido: cuánto tardó la evaluación y las cifras.
                let (citadas, fichas, ms) = (e.citadas(), e.evidencia.len(), reloj.elapsed().as_millis());
                println!("[ensayo] evaluada en {ms} ms · evidencia {citadas} de {fichas}");
                cola.extend(v.sesion.evaluada(c.indice, e));
            }
            Accion::Avisar => avisar = true,
        }
    }
    if v.sesion.fase() == Fase::Cerrado && v.oido.is_some() {
        // El ensayo terminó: el micrófono se cierra en este instante, no al cerrar la pantalla.
        v.oido = None;
        let i = v.sesion.informe();
        println!("[ensayo] terminado: {} respondidas, {} saltadas · micrófono cerrado", i.respondidas, i.saltadas);
    }
    if avisar {
        mundo.avisar();
    }
}

impl Ensayo {
    /// Empieza por la primera pregunta. `None` sin preguntas.
    pub fn arrancar(
        preguntas: Vec<Pregunta>,
        idioma: Idioma,
        voz: bool,
        rotulo: Rotulo,
        banco: EstadoDelBanco,
        oido: Oido,
        mundo: Arc<dyn Mundo>,
    ) -> Option<Self> {
        let nacio = Instant::now();
        let (sesion, acciones) = Sesion::nueva(preguntas, idioma, voz, 0)?;
        let mut v = Vivo { sesion, oido: Some(oido), rotulo, banco };
        ejecutar(&mut v, mundo.as_ref(), acciones, 0);
        let vivo = Arc::new(Mutex::new(Some(v)));
        let viva = Arc::new(AtomicBool::new(true));
        {
            let (vivo, viva, mundo) = (vivo.clone(), viva.clone(), mundo.clone());
            std::thread::spawn(move || {
                while viva.load(Ordering::Relaxed) && latir(&vivo, mundo.as_ref(), nacio) {
                    std::thread::sleep(LATIDO);
                }
            });
        }
        Some(Self { vivo, mundo, nacio, viva })
    }

    fn ahora(&self) -> u64 {
        self.nacio.elapsed().as_millis() as u64
    }

    fn con(&self, f: impl FnOnce(&mut Sesion, u64) -> Vec<Accion>) {
        let ahora = self.ahora();
        if let Ok(mut g) = self.vivo.lock() {
            if let Some(v) = g.as_mut() {
                let acciones = f(&mut v.sesion, ahora);
                ejecutar(v, self.mundo.as_ref(), acciones, ahora);
            }
        }
    }

    /// Enter.
    pub fn listo(&self) {
        self.con(|s, ahora| s.listo(ahora));
    }
    /// R.
    pub fn repetir(&self) {
        self.con(|s, ahora| s.repetir(ahora));
    }
    /// S.
    pub fn saltar(&self) {
        self.con(|s, ahora| s.saltar(ahora));
    }
    /// Esc.
    pub fn terminar(&self) {
        self.con(|s, _| s.terminar());
    }
    pub fn si_lo_dije(&self, i: usize) {
        self.con(|s, _| s.si_lo_dije(i));
    }

    /// Lo que volvió del modelo (ADR 019 §3): las fundadas van al final, marcadas; si no, la razón.
    pub fn con_lo_del_modelo(&self, r: Result<Enriquecido, PorQueNo>) {
        let ahora = self.ahora();
        if let Ok(mut g) = self.vivo.lock() {
            if let Some(v) = g.as_mut() {
                let acciones = match r {
                    Ok(e) => {
                        let antes = v.sesion.preguntas().len();
                        let a = v.sesion.con_las_del_modelo(e.preguntas);
                        let cuantas = v.sesion.preguntas().len() - antes;
                        // «Sumó 0» no es lo que pasó (auditoría del S4, B22): o ya estaban en el banco, o el
                        // ensayo había terminado cuando llegaron.
                        v.banco = match cuantas {
                            0 if v.sesion.fase() == Fase::Cerrado => EstadoDelBanco::NoSeEnriquecio { porque: PorQueNo::Tarde },
                            0 => EstadoDelBanco::NoSeEnriquecio { porque: PorQueNo::Repetidas },
                            cuantas => EstadoDelBanco::Sumadas { cuantas },
                        };
                        a
                    }
                    Err(porque) => {
                        v.banco = EstadoDelBanco::NoSeEnriquecio { porque };
                        Vec::new()
                    }
                };
                ejecutar(v, self.mundo.as_ref(), acciones, ahora);
                self.mundo.avisar();
            }
        }
    }

    /// El micrófono del ensayo está abierto (no ha terminado).
    pub fn escuchando(&self) -> bool {
        self.vivo.lock().is_ok_and(|g| g.as_ref().is_some_and(|v| v.oido.is_some()))
    }

    /// Lo que vive en memoria por culpa de este ensayo.
    pub fn memoria(&self) -> Memoria {
        let Ok(g) = self.vivo.lock() else { return Memoria::default() };
        let Some(v) = g.as_ref() else { return Memoria::default() };
        Memoria { microfono: v.oido.as_ref().map_or(0, Oido::bytes), respuestas: v.sesion.bytes_de_tus_respuestas() }
    }

    /// **Lo que se guarda** (ADR 015, enmienda 4): solo con el ensayo terminado, y sin audio —el oído ya se
    /// cerró y la sesión solo tiene texto—. `None` si todavía no terminó.
    pub fn para_guardar(&self) -> Option<guardado::Guardado> {
        let g = self.vivo.lock().ok()?;
        let v = g.as_ref()?;
        if v.sesion.fase() != Fase::Cerrado || v.oido.is_some() {
            return None;
        }
        Some(guardado::Guardado::nuevo(
            &v.rotulo.cliente,
            &v.rotulo.propuesta,
            v.sesion.idioma(),
            &v.rotulo.empezo.como_texto(),
            v.sesion.llegadas(),
        ))
    }

    /// Con quién y cuándo.
    pub fn rotulo(&self) -> Option<Rotulo> {
        self.vivo.lock().ok()?.as_ref().map(|v| v.rotulo.clone())
    }

    pub fn vista(&self) -> Option<VistaDelEnsayo> {
        let ahora = self.ahora();
        let g = self.vivo.lock().ok()?;
        let v = g.as_ref()?;
        let s = &v.sesion;
        let fase = s.fase();
        let actual = s.actual();
        let evaluacion = match (fase, s.suertes().get(actual)) {
            (Fase::Evaluada, Some(Some(Suerte::Respondida { evaluacion, .. }))) => Some(evaluacion.clone()),
            _ => None,
        };
        let respuesta = match (fase, s.suertes().get(actual)) {
            (Fase::Evaluada, Some(Some(Suerte::Respondida { respuesta, .. }))) => respuesta.clone(),
            (Fase::Respondiendo, _) => s.respuesta_en_curso(),
            _ => String::new(),
        };
        let usadas = evaluacion.as_ref().map_or(0, Evaluacion::citadas);
        let muletillas = evaluacion.as_ref().filter(|_| !respuesta.trim().is_empty()).map(Evaluacion::total_de_muletillas);
        Some(VistaDelEnsayo {
            fase,
            cliente: v.rotulo.cliente.clone(),
            indice: actual,
            total: s.preguntas().len(),
            pregunta: (fase != Fase::Cerrado).then(|| s.preguntas().get(actual).map(PreguntaEnPantalla::from)).flatten(),
            leyendo: s.leyendo(),
            cerrando: s.cerrando(),
            respuesta,
            transcurrido_ms: s.transcurrido(ahora),
            evaluacion,
            usadas,
            muletillas,
            banco: v.banco,
            informe: (fase == Fase::Cerrado).then(|| s.informe()),
        })
    }

    /// **El corte**, y «Cerrar sin guardar»: el micrófono se cierra, la voz calla y todo lo que dijiste se
    /// pisa. No queda nada.
    pub fn cortar(&self) {
        self.viva.store(false, Ordering::Relaxed);
        self.mundo.callar();
        if let Ok(mut g) = self.vivo.lock() {
            if let Some(mut v) = g.take() {
                if let Some(o) = v.oido.as_mut() {
                    o.cortar();
                }
                v.oido = None;
                v.sesion.vaciar();
            }
        }
    }
}

impl Drop for Ensayo {
    fn drop(&mut self) {
        self.cortar();
    }
}

/// **Un latido**: lo que oyó el micrófono pasa a la sesión, y la sesión decide. `false` cuando ya no hay
/// nada que llevar (cortado o terminado).
fn latir(vivo: &Mutex<Option<Vivo>>, mundo: &dyn Mundo, nacio: Instant) -> bool {
    let ahora = nacio.elapsed().as_millis() as u64;
    let Ok(mut g) = vivo.lock() else { return false };
    let Some(v) = g.as_mut() else { return false };
    let Some(o) = v.oido.as_mut() else { return false };
    o.latir(v.sesion.oye(), v.sesion.ronda(), ahora);
    // **Lo pendiente se cuenta ANTES de recoger lo transcrito** (auditoría del S4, B15): al revés, un turno
    // que vuelve del motor entre las dos lecturas no está en ninguna —ni recogido ni pendiente— y Enter
    // cerraría la respuesta sin él. Así, en el peor caso, se espera un latido de más.
    let oyendo = Oyendo { hablando: o.hablando(), pendientes: o.pendientes() };
    let mut acciones = Vec::new();
    for t in o.recibidos() {
        acciones.extend(v.sesion.turno(t.ronda, t.desde_ms, t.hasta_ms, t.texto));
    }
    acciones.extend(v.sesion.tick(ahora, oyendo, mundo.diciendo()));
    ejecutar(v, mundo, acciones, ahora);
    v.oido.is_some()
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::capture::anillo::Anillo;
    use crate::diccionario::Diccionario;
    use crate::ensayo::evaluacion::pruebas::respaldo;
    use crate::ensayo::oido::pruebas::{silencio, voz, Contador};
    use std::sync::atomic::AtomicUsize;

    /// Un mundo de mentira: la voz «lee» 300 ms y el corpus tiene una ficha por pregunta.
    #[derive(Default)]
    struct DeMentira {
        dijo: Mutex<Vec<String>>,
        hasta: Mutex<Option<Instant>>,
        avisos: AtomicUsize,
        sin_voz: bool,
    }

    impl Mundo for DeMentira {
        fn decir(&self, _: Idioma, texto: &str) -> bool {
            if self.sin_voz {
                return false;
            }
            self.dijo.lock().unwrap().push(texto.into());
            *self.hasta.lock().unwrap() = Some(Instant::now() + Duration::from_millis(300));
            true
        }
        fn callar(&self) {
            *self.hasta.lock().unwrap() = None;
        }
        fn diciendo(&self) -> bool {
            self.hasta.lock().unwrap().is_some_and(|h| Instant::now() < h)
        }
        fn evidencia(&self, _: &str) -> Vec<crate::ficha::Respaldo> {
            vec![respaldo("Muestras", "Una ficha que habla de muestras contadas", crate::corpus::Unidad::Propuesta)]
        }
        fn avisar(&self) {
            self.avisos.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn rotulo(cliente: &str) -> Rotulo {
        Rotulo {
            cliente: cliente.into(),
            propuesta: "Rentabilidad por canal".into(),
            empezo: crate::notas::Fecha { anio: 2026, mes: 10, dia: 4, hora: 9, minuto: 12 },
        }
    }

    fn preguntas() -> Vec<Pregunta> {
        ["¿Uno?", "¿Dos?"]
            .iter()
            .map(|t| Pregunta { texto: (*t).into(), regla: banco::Regla::Seccion, de: De::Propuesta, seccion: Some("Alcance".into()), clave: "alcance".into(), fuente: None })
            .collect()
    }

    fn esperar(e: &Ensayo, hasta: impl Fn(&VistaDelEnsayo) -> bool) -> VistaDelEnsayo {
        for _ in 0..400 {
            if let Some(v) = e.vista() {
                if hasta(&v) {
                    return v;
                }
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("no llegó: {:?}", e.vista());
    }

    /// **De punta a punta, sin Mac**: la voz lee, el micrófono espera, tu voz entra al anillo, el turno se
    /// transcribe, 2,5 s de silencio cierran, la evaluación sale con la ficha citada; Enter, la segunda;
    /// Esc, el informe — y el micrófono se cierra al terminar.
    #[test]
    fn un_ensayo_entero_con_audio_inventado() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let oido = Oido::con_anillo(anillo.clone(), "es-ES", Box::new(Contador), Arc::new(Diccionario::default()));
        let mundo = Arc::new(DeMentira::default());
        let nacio = Instant::now();
        let e = Ensayo::arrancar(preguntas(), Idioma::Es, true, rotulo("Páramo Azul"), EstadoDelBanco::Apagado, oido, mundo.clone()).unwrap();
        let v = e.vista().unwrap();
        assert_eq!((v.fase, v.leyendo, v.total), (Fase::Preguntando, true, 2));
        assert_eq!(mundo.dijo.lock().unwrap().as_slice(), ["¿Uno?"]);
        // Mientras lee entra «voz» (la de la app por los altavoces): no debe convertirse en respuesta.
        anillo.lock().unwrap().escribir(&voz(250));
        let v = esperar(&e, |v| v.fase == Fase::Respondiendo);
        assert_eq!(v.respuesta, "");
        // Tu respuesta, a ritmo de micrófono (trozos de 100 ms): silencio para que la oreja aprenda la
        // sala, un segundo de voz y silencio.
        for trozo in [silencio(600), voz(1_000), silencio(600)] {
            for c in trozo.chunks(1_600) {
                anillo.lock().unwrap().escribir(c);
                std::thread::sleep(Duration::from_millis(100));
            }
        }
        let v = esperar(&e, |v| !v.respuesta.is_empty());
        assert!(v.respuesta.ends_with("muestras"), "{}", v.respuesta);
        // Honestidad cuenta lo que vive en memoria: el anillo del micrófono y tu respuesta en texto.
        let m = e.memoria();
        assert!(m.microfono > 0 && m.respuestas > 0, "la memoria del ensayo no se cuenta: {m:?}");
        // 2,5 s después del fin de tu voz, la evaluación.
        let v = esperar(&e, |v| v.fase == Fase::Evaluada);
        // Las cuentas las hace Rust (auditoría del S4, M12): hubo palabras, así que las muletillas son una
        // cifra —cero—; y ninguna ficha usada todavía.
        assert_eq!((v.usadas, v.muletillas), (0, Some(0)));
        let ev = v.evaluacion.expect("evaluada");
        assert_eq!(ev.citadas(), 0, "«muestras» es un término, y la ficha pide dos");
        // El tiempo sale del reloj de verdad, así que solo se afirma lo que no depende de la máquina: al
        // menos el segundo de voz, y nunca más de lo que de verdad pasó. (La primera versión acotaba arriba
        // en 2,5 s: en el runner de la CI, más lento, dio 3,5 s y cayó en rojo — bitácora, fase 3.)
        let pasado = nacio.elapsed().as_millis() as u64;
        assert!((1_000..=pasado).contains(&ev.tiempo_ms), "de la pregunta al fin de tu voz: {} ms (pasaron {pasado} ms)", ev.tiempo_ms);
        e.si_lo_dije(0);
        assert_eq!(e.vista().unwrap().evaluacion.unwrap().citadas(), 1);
        assert_eq!(e.vista().unwrap().usadas, 1, "«Sí lo dije» no se cuenta en la vista");
        // **La siguiente pregunta en menos de un segundo** (plan, fase 3): Enter la arma y empieza a leerla
        // en el mismo instante; la pantalla solo tiene que volver a preguntar.
        let reloj = Instant::now();
        e.listo();
        let v = e.vista().unwrap();
        assert!(reloj.elapsed() < Duration::from_secs(1), "la siguiente tardó {:?}", reloj.elapsed());
        assert_eq!((v.indice, v.fase), (1, Fase::Preguntando));
        assert_eq!(mundo.dijo.lock().unwrap().last().map(String::as_str), Some("¿Dos?"));
        assert!(e.para_guardar().is_none(), "se pudo guardar un ensayo sin terminar");
        e.terminar();
        let v = esperar(&e, |v| v.fase == Fase::Cerrado);
        let i = v.informe.expect("el informe");
        assert_eq!((i.respondidas, i.saltadas, i.citadas), (1, 0, 1));
        assert!(!e.escuchando(), "al terminar, el micrófono se cierra");
        assert!(mundo.avisos.load(Ordering::Relaxed) >= 4);
        // **Lo que se guarda** (ADR 015, enmienda 4): la pregunta respondida —no la que se estaba leyendo al
        // pulsar Esc—, con quién y cuándo, y el mismo informe que la pantalla. El micrófono ya no ocupa nada.
        let m = e.memoria();
        assert_eq!(m.microfono, 0, "terminado, el anillo del micrófono sigue contando");
        assert!(m.respuestas > 0);
        let g = e.para_guardar().expect("un ensayo terminado se puede guardar");
        assert_eq!((g.cliente.as_str(), g.empezo.as_str(), g.llegadas.len()), ("Páramo Azul", "2026-10-04 09:12", 1));
        assert_eq!(g.informe(), i, "lo guardado no dice lo mismo que la pantalla");
        e.cortar();
        assert!(e.vista().is_none(), "tras el corte no queda nada");
        assert!(e.para_guardar().is_none(), "tras el corte se pudo guardar");
    }

    /// Sin voz para el idioma, la pregunta aparece y se escucha en el acto.
    #[test]
    fn sin_voz_se_responde_sin_esperar() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let oido = Oido::con_anillo(anillo, "es-ES", Box::new(Contador), Arc::new(Diccionario::default()));
        let mundo = Arc::new(DeMentira { sin_voz: true, ..Default::default() });
        let e = Ensayo::arrancar(preguntas(), Idioma::Es, true, rotulo("X"), EstadoDelBanco::Apagado, oido, mundo).unwrap();
        assert_eq!(e.vista().unwrap().fase, Fase::Respondiendo);
    }

    /// Lo del modelo llega cuando llega: suma al final y la pantalla lo cuenta; o dice por qué no.
    #[test]
    fn lo_del_modelo_suma_o_dice_por_que_no() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let oido = Oido::con_anillo(anillo, "es-ES", Box::new(Contador), Arc::new(Diccionario::default()));
        let e = Ensayo::arrancar(preguntas(), Idioma::Es, false, rotulo("X"), EstadoDelBanco::EnCamino, oido, Arc::new(DeMentira::default())).unwrap();
        assert_eq!(e.vista().unwrap().banco, EstadoDelBanco::EnCamino);
        let mut m = preguntas().remove(0);
        m.texto = "¿Del modelo?".into();
        m.de = De::Modelo;
        e.con_lo_del_modelo(Ok(Enriquecido { preguntas: vec![m], descartadas: 1 }));
        let v = e.vista().unwrap();
        assert_eq!((v.total, v.banco), (3, EstadoDelBanco::Sumadas { cuantas: 1 }));
        e.con_lo_del_modelo(Err(PorQueNo::Tarde));
        assert_eq!(e.vista().unwrap().banco, EstadoDelBanco::NoSeEnriquecio { porque: PorQueNo::Tarde });
    }

    /// **«El modelo sumó 0» no es lo que pasó** (auditoría del S4, B22): si todas ya estaban en el banco, se
    /// dice que estaban; si llegaron con el ensayo terminado, que llegaron tarde.
    #[test]
    fn lo_del_modelo_repetido_no_dice_que_sumo() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let oido = Oido::con_anillo(anillo, "es-ES", Box::new(Contador), Arc::new(Diccionario::default()));
        let e = Ensayo::arrancar(preguntas(), Idioma::Es, false, rotulo("X"), EstadoDelBanco::EnCamino, oido, Arc::new(DeMentira::default())).unwrap();
        let repetida = preguntas().remove(0);
        e.con_lo_del_modelo(Ok(Enriquecido { preguntas: vec![repetida.clone()], descartadas: 0 }));
        let v = e.vista().unwrap();
        assert_eq!((v.total, v.banco), (2, EstadoDelBanco::NoSeEnriquecio { porque: PorQueNo::Repetidas }));
        e.terminar();
        let mut nueva = repetida;
        nueva.texto = "¿Del modelo?".into();
        e.con_lo_del_modelo(Ok(Enriquecido { preguntas: vec![nueva], descartadas: 0 }));
        assert_eq!(e.vista().unwrap().banco, EstadoDelBanco::NoSeEnriquecio { porque: PorQueNo::Tarde });
    }

    /// **Lo pendiente se cuenta antes de recoger lo transcrito** (auditoría del S4, B15): al revés, un turno que
    /// vuelve del motor entre las dos lecturas no estaba en ninguna, y Enter cerraba la respuesta sin él.
    #[test]
    fn el_latido_cuenta_lo_pendiente_antes_de_recoger() {
        let fuente = include_str!("mod.rs");
        let desde = fuente.find(concat!("fn latir(vivo: &Mutex<Option<", "Vivo>>")).expect("el latido");
        let cuerpo = &fuente[desde..];
        let cuerpo = &cuerpo[..cuerpo.find("\n}\n").expect("su cierre")];
        let pendientes = cuerpo.find(concat!("o.", "pendientes()")).expect("el latido no cuenta lo pendiente");
        let recibidos = cuerpo.find(concat!("o.", "recibidos()")).expect("el latido no recoge lo transcrito");
        assert!(pendientes < recibidos, "se recoge lo transcrito antes de contar lo pendiente: un turno se puede perder");
    }

    /// **Con una videollamada abierta, no por altavoces que la app reconoce** (auditoría del S4, A1): la matriz
    /// entera. Sin llamada se ensaya con lo que sea; con una —o sin poder saberlo—, no por los altavoces del Mac,
    /// HDMI, DisplayPort ni AirPlay; Bluetooth y USB pasan.
    #[test]
    fn con_una_videollamada_solo_se_ensaya_con_auriculares() {
        use crate::sesion::{PorQueNoSeVe, Proteccion, Reunion};
        let ninguna = Reunion::Ninguna;
        let zoom = Reunion::Detectada { cliente: "Zoom".into(), titulo: None, proteccion: Proteccion::SinVerificar };
        let ciega = Reunion::NoSePuedeSaber { motivo: PorQueNoSeVe::SinAccesibilidad };
        for eco in [Some(true), Some(false), None] {
            assert_eq!(llamada_sin_auriculares(&ninguna, eco), None, "sin llamada, con {eco:?}");
        }
        assert_eq!(llamada_sin_auriculares(&zoom, Some(true)), Some(NoEmpezo::Videollamada));
        // Bluetooth o USB (los AirPods): no se distinguen de un altavoz, y pasan como en reunión (H1).
        assert_eq!(llamada_sin_auriculares(&zoom, None), None, "unos AirPods no dejan ensayar con la llamada abierta");
        assert_eq!(llamada_sin_auriculares(&zoom, Some(false)), None);
        assert_eq!(llamada_sin_auriculares(&ciega, Some(true)), Some(NoEmpezo::NoSeSabeSiHayLlamada));
        assert_eq!(llamada_sin_auriculares(&ciega, Some(false)), None);
    }
}
