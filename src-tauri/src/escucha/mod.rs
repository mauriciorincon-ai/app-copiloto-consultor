//! **La escucha** — lo que junta las piezas de la fase 3 y las pone a funcionar a la vez.
//!
//! `capture` abre los grifos, `voz` corta los turnos, `stt` los convierte en texto. Ninguno de los
//! tres sabe de los otros, y esa separación es la que ha permitido probarlos por su cuenta con
//! audio inventado. Aquí se reúnen, y aquí aparecen los problemas que solo existen cuando algo
//! corre de verdad: dos pistas a la vez, un transcriptor que tarda un cuarto de segundo, y un
//! kill-switch que puede caer en cualquier instante de todo lo anterior.
//!
//! **MÓDULO PROTEGIDO.** Tiene en las manos la voz del cliente entera; no la guarda en ninguna
//! parte, y `pnpm verify:ephemeral` lo comprueba.
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
#[serde(rename_all = "kebab-case", tag = "que")]
pub enum Novedad {
    /// Alguien empezó a hablar en esta pista.
    Empieza { pista: Pista },
    /// Un turno terminó y ya tiene texto.
    Turno(Turno),
    /// Un turno terminó y **no** se pudo transcribir. Con su motivo.
    SinTexto { pista: Pista, desde_ms: usize, hasta_ms: usize, motivo: String },
    /// El detector oyó algo demasiado corto para ser un turno.
    Ruido { pista: Pista, duracion_ms: usize },
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
        }
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

/// Lo que el hilo de escucha manda al hilo que transcribe.
struct Encargo {
    pista: Pista,
    idioma: String,
    desde_ms: usize,
    hasta_ms: usize,
    /// El audio del turno, ya recortado. Es la única copia que se hace, y vive lo que tarda el
    /// motor: se suelta en cuanto vuelve el texto.
    muestras: Option<Vec<f32>>,
}

/// La escucha en marcha.
pub struct Escucha {
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
        avisar: impl Fn(Novedad) + Send + Sync + 'static,
    ) -> Self {
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

        // Hilo 1 — mira los marcos y corta los turnos.
        {
            let pistas = pistas.clone();
            let viva = viva.clone();
            let avisar = avisar.clone();
            std::thread::spawn(move || {
                while viva.load(Ordering::Relaxed) {
                    if let Ok(mut lista) = pistas.lock() {
                        for p in lista.iter_mut() {
                            mirar(p, &manda, avisar.as_ref());
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(LATIDO_MS));
                }
            });
        }

        // Hilo 2 — transcribe lo que el primero le pasa.
        {
            let ventana = ventana.clone();
            let avisar = avisar.clone();
            std::thread::spawn(move || {
                for encargo in recibe {
                    let mut novedad = transcribir(&*motor, encargo);
                    if let Novedad::Turno(t) = &mut novedad {
                        // El eco se decide **con la ventana delante**: hace falta saber qué dijo
                        // el cliente para saber si el micrófono lo está repitiendo. Por eso vive
                        // aquí y no en `transcribir`, que no conoce a nadie más.
                        if t.pista == Pista::Microfono {
                            if let Ok(v) = ventana.lock() {
                                t.eco = huele_a_eco(t, &v);
                            }
                        }
                        if let Ok(mut v) = ventana.lock() {
                            v.empujar(t.clone());
                        }
                    }
                    avisar(novedad);
                }
            });
        }

        Self { pistas, ventana, viva, motor: nombre_del_motor }
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
    }
}

impl Drop for Escucha {
    fn drop(&mut self) {
        self.viva.store(false, Ordering::Relaxed);
    }
}

/// Saca del anillo lo que haya llegado y se lo da a la máquina de turnos, marco a marco.
fn mirar(p: &mut PistaViva, manda: &Sender<Encargo>, avisar: &dyn Fn(Novedad)) {
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
                    desde_ms,
                    hasta_ms,
                    muestras,
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

fn transcribir(motor: &dyn Motor, encargo: Encargo) -> Novedad {
    let Encargo { pista, idioma, desde_ms, hasta_ms, muestras } = encargo;
    let sin_texto = |motivo: String| Novedad::SinTexto { pista, desde_ms, hasta_ms, motivo };

    let Some(muestras) = muestras else {
        return sin_texto(
            "el audio de ese turno ya se había pisado: la reunión iba más rápido que la \
             transcripción"
                .into(),
        );
    };
    match motor.transcribir(&idioma, &muestras, HZ) {
        Ok(texto) if texto.trim().is_empty() => {
            sin_texto("el motor no reconoció palabras en ese turno".into())
        }
        Ok(texto) => {
            Novedad::Turno(Turno { pista, desde_ms, hasta_ms, texto, hora: la_hora(), eco: false })
        }
        Err(Fallo::NoDisponible(d)) => sin_texto(explicar(&d, &idioma)),
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

    /// El turno perdido tiene su propia novedad, distinta del turno callado. Si los dos acabaran en
    /// el mismo sitio, la banda enseñaría un hueco y nadie sabría si el cliente calló o si la app
    /// se quedó atrás.
    #[test]
    fn un_turno_cuyo_audio_se_piso_no_se_confunde_con_uno_callado() {
        let motor = crate::stt::Mudo::por("sin motor de prueba");
        let perdido = transcribir(
            &motor,
            Encargo { pista: Pista::Sistema, idioma: "es-ES".into(), desde_ms: 0, hasta_ms: 900, muestras: None },
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

    #[test]
    fn sin_motor_el_turno_sale_con_el_motivo_del_motor() {
        let motor = crate::stt::Mudo::por("este Mac no trae el transcriptor de macOS 26");
        let novedad = transcribir(
            &motor,
            Encargo {
                pista: Pista::Sistema,
                idioma: "es-ES".into(),
                desde_ms: 0,
                hasta_ms: 900,
                muestras: Some(vec![0.1; 14_400]),
            },
        );
        match novedad {
            Novedad::SinTexto { motivo, .. } => assert!(motivo.contains("macOS 26"), "«{motivo}»"),
            otro => panic!("{otro:?}"),
        }
    }
}
