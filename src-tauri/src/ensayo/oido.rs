//! EL OÍDO DEL ENSAYO — **solo el micrófono** (ADR 019 §6.1). **MÓDULO PROTEGIDO.**
//!
//! La reunión abre dos pistas (la escucha de `lib.rs::empezar`); el ensayo abre una, la tuya, con
//! `Grifo::del_microfono`, y nada más: ni el audio del sistema, ni la pantalla, ni el radar ámbar. Un
//! test sobre el código de este módulo lo vigila ([`pruebas::el_oido_solo_abre_el_microfono`]).
//!
//! Reutiliza las piezas de la escucha sin tocarlas: el anillo de 30 s, el detector por energía y el fin
//! de turno de 320 ms (`voz::Turnos`). Cada turno se transcribe **en cuanto se cierra**, en su propio
//! hilo, así que el anillo nunca es el techo de una respuesta: una de dos minutos son diez turnos.
//!
//! **Sordo** (`oye == false`, la sesión lo decide: mientras la voz lee, su cola, después de Enter):
//! el turno que estuviera a medias se cierra con el audio que ya tenía, y lo que entra no se le da al
//! detector — ni se transcribe ni le enseña al suelo de ruido una sala en silencio artificial. El reloj
//! de los turnos y el anillo se mueven juntos (`origen`), como en la escucha (hallazgo A5 del S1).
//!
//! Como la escucha: el audio vive en el anillo y en una sola copia por turno, que se pisa en cuanto el
//! motor devuelve el texto. Al log, el hecho y los milisegundos; jamás lo que dijiste.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use crate::capture::anillo::{Anillo, HZ};
use crate::diccionario::Diccionario;
use crate::stt::Motor;
use crate::voz::{Suceso, Turnos, MARCO};

/// Un turno tuyo cerrado: su audio (la única copia fuera del anillo), cuánto duró y hace cuánto acabó,
/// contado desde la muestra más nueva del anillo.
pub struct Cerrado {
    pub muestras: Option<Vec<f32>>,
    pub duracion_ms: u64,
    pub hace_ms: u64,
}

/// **La oreja**: del anillo a turnos cerrados. Sin hilos ni micrófono, para poder probarla con audio
/// inventado escrito en el anillo.
pub struct Oreja {
    anillo: Arc<Mutex<Anillo>>,
    turnos: Turnos,
    /// Índice global de la siguiente muestra que esta oreja no ha mirado.
    cursor: u64,
    /// Dónde cae el cero del reloj de los turnos, en muestras del anillo. Lo mueven la sordera y el
    /// reenganche: siempre `cursor == origen + reloj_ms·16 + sobrante`.
    origen: u64,
    sobrante: Vec<f32>,
}

fn indice(origen: u64, ms: usize) -> u64 {
    origen + (ms as u64 * HZ as u64) / 1000
}

impl Oreja {
    pub fn nueva(anillo: Arc<Mutex<Anillo>>) -> Self {
        let ahora = anillo.lock().map(|a| a.totales()).unwrap_or(0);
        Self { anillo, turnos: Turnos::default(), cursor: ahora, origen: ahora, sobrante: Vec::with_capacity(MARCO * 4) }
    }

    /// ¿Hay un turno tuyo en curso?
    pub fn hablando(&self) -> bool {
        self.turnos.hablando()
    }

    fn recortar(&self, desde_ms: usize, hasta_ms: usize, mas_nueva: u64) -> Cerrado {
        let (d, h) = (indice(self.origen, desde_ms), indice(self.origen, hasta_ms));
        Cerrado {
            muestras: self.anillo.lock().ok().and_then(|a| a.rango(d, h)),
            duracion_ms: (hasta_ms - desde_ms) as u64,
            hace_ms: mas_nueva.saturating_sub(h) * 1000 / HZ as u64,
        }
    }

    /// Mira lo que entró desde la última vez. Ver el encabezado del módulo para `oye`.
    pub fn mirar(&mut self, oye: bool) -> Vec<Cerrado> {
        let (nuevas, totales) = {
            let Ok(a) = self.anillo.lock() else { return Vec::new() };
            let t = a.totales();
            (a.rango(self.cursor, t), t)
        };
        let Some(nuevas) = nuevas else {
            // El anillo dio la vuelta entre dos latidos: ese audio ya no existe. Se dice y se vuelve
            // al presente, con el reloj y el origen juntos.
            println!("[ensayo] el oído se quedó atrás: ese audio ya se pisó y no se va a transcribir");
            self.cursor = totales;
            self.origen = totales;
            self.sobrante.clear();
            self.turnos.reiniciar();
            return Vec::new();
        };
        let mut salida = Vec::new();
        if !oye {
            if let Some(Suceso::Termina { desde_ms, hasta_ms }) = self.turnos.cerrar() {
                salida.push(self.recortar(desde_ms, hasta_ms, totales));
            }
            self.cursor += nuevas.len() as u64;
            self.origen += (nuevas.len() + self.sobrante.len()) as u64;
            self.sobrante.clear();
            return salida;
        }
        self.cursor += nuevas.len() as u64;
        self.sobrante.extend_from_slice(&nuevas);
        let mut usados = 0;
        while self.sobrante.len() - usados >= MARCO {
            let suceso = self.turnos.marco(&self.sobrante[usados..usados + MARCO]);
            usados += MARCO;
            if let Some(Suceso::Termina { desde_ms, hasta_ms }) = suceso {
                salida.push(self.recortar(desde_ms, hasta_ms, totales));
            }
        }
        self.sobrante.drain(..usados);
        salida
    }

    /// El corte: el detector olvida la sala y lo que quedaba a medias se pisa.
    pub fn olvidar(&mut self) {
        self.sobrante.fill(0.0);
        self.sobrante.clear();
        self.turnos.reiniciar();
        self.cursor = 0;
        self.origen = 0;
    }
}

/// Un turno tuyo ya transcrito, con su sitio en el reloj del ensayo. `texto: None` si se oyó y el motor
/// no lo pudo transcribir.
pub struct Transcrito {
    pub ronda: u64,
    pub desde_ms: u64,
    pub hasta_ms: u64,
    pub texto: Option<String>,
}

struct Encargo {
    ronda: u64,
    desde_ms: u64,
    hasta_ms: u64,
    muestras: Option<Vec<f32>>,
}

fn pisar_audio(m: &mut Option<Vec<f32>>) {
    if let Some(v) = m {
        v.fill(0.0);
        v.clear();
    }
}

/// **EL OÍDO**: el grifo del micrófono, la oreja y el hilo que transcribe.
pub struct Oido {
    anillo: Arc<Mutex<Anillo>>,
    /// Se conserva para que el grifo siga abierto: soltarlo lo cierra. `None` en las pruebas.
    grifo: Option<crate::capture::nativo::Grifo>,
    oreja: Oreja,
    manda: Option<Sender<Encargo>>,
    llegan: Receiver<Transcrito>,
    pendientes: Arc<AtomicUsize>,
    vivo: Arc<AtomicBool>,
}

impl Oido {
    /// **Abre el micrófono, y nada más.** Es la única entrada de audio del ensayo.
    pub fn del_microfono(idioma: &str, motor: Box<dyn Motor>, diccionario: Arc<Diccionario>) -> Result<Self, crate::capture::NoAbrio> {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let grifo = crate::capture::nativo::Grifo::del_microfono(anillo.clone())?;
        println!("[ensayo] micrófono abierto · {idioma}");
        Ok(Self::armar(anillo, Some(grifo), idioma, motor, diccionario))
    }

    /// El mismo oído sobre un anillo que llena otro: el de las pruebas, sin micrófono.
    pub fn con_anillo(anillo: Arc<Mutex<Anillo>>, idioma: &str, motor: Box<dyn Motor>, diccionario: Arc<Diccionario>) -> Self {
        Self::armar(anillo, None, idioma, motor, diccionario)
    }

    fn armar(
        anillo: Arc<Mutex<Anillo>>,
        grifo: Option<crate::capture::nativo::Grifo>,
        idioma: &str,
        motor: Box<dyn Motor>,
        diccionario: Arc<Diccionario>,
    ) -> Self {
        let (manda, recibe) = std::sync::mpsc::channel::<Encargo>();
        let (devuelve, llegan) = std::sync::mpsc::channel::<Transcrito>();
        let pendientes = Arc::new(AtomicUsize::new(0));
        let vivo = Arc::new(AtomicBool::new(true));
        {
            let (pendientes, vivo, idioma) = (pendientes.clone(), vivo.clone(), idioma.to_string());
            std::thread::spawn(move || {
                while let Ok(mut e) = recibe.recv() {
                    let mut texto = if vivo.load(Ordering::Relaxed) {
                        e.muestras
                            .as_ref()
                            .and_then(|m| motor.transcribir(&idioma, m, HZ).ok())
                            .filter(|t| !t.trim().is_empty())
                            .map(|t| diccionario.corregir(&t))
                    } else {
                        None
                    };
                    pisar_audio(&mut e.muestras);
                    // Metadata, jamás contenido: cuánto duró y cuántas letras salieron.
                    let letras = texto.as_ref().map(|t| t.chars().count());
                    let ms = e.hasta_ms.saturating_sub(e.desde_ms);
                    match letras {
                        Some(n) => println!("[ensayo] turno tuyo · {ms} ms · {n} letras"),
                        None => println!("[ensayo] turno tuyo · {ms} ms · sin texto"),
                    }
                    if vivo.load(Ordering::Relaxed) {
                        let _ = devuelve.send(Transcrito { ronda: e.ronda, desde_ms: e.desde_ms, hasta_ms: e.hasta_ms, texto: texto.take() });
                    } else if let Some(t) = &mut texto {
                        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
                        unsafe { t.as_mut_vec() }.fill(0);
                    }
                    pendientes.fetch_sub(1, Ordering::SeqCst);
                }
            });
        }
        let oreja = Oreja::nueva(anillo.clone());
        Self { anillo, grifo, oreja, manda: Some(manda), llegan, pendientes, vivo }
    }

    pub fn hablando(&self) -> bool {
        self.oreja.hablando()
    }

    pub fn pendientes(&self) -> usize {
        self.pendientes.load(Ordering::SeqCst)
    }

    /// **Un latido**: lo que entró, a la oreja; cada turno cerrado, a transcribir con la ronda en que se
    /// oyó y su sitio en el reloj del ensayo.
    pub fn latir(&mut self, oye: bool, ronda: u64, ahora_ms: u64) {
        if !self.vivo.load(Ordering::Relaxed) {
            return;
        }
        for c in self.oreja.mirar(oye) {
            let hasta_ms = ahora_ms.saturating_sub(c.hace_ms);
            let desde_ms = hasta_ms.saturating_sub(c.duracion_ms);
            if let Some(manda) = &self.manda {
                self.pendientes.fetch_add(1, Ordering::SeqCst);
                if manda.send(Encargo { ronda, desde_ms, hasta_ms, muestras: c.muestras }).is_err() {
                    self.pendientes.fetch_sub(1, Ordering::SeqCst);
                }
            }
        }
    }

    /// Lo que ya volvió del motor.
    pub fn recibidos(&self) -> Vec<Transcrito> {
        self.llegan.try_iter().collect()
    }

    /// **El corte**: primero se cierra el grifo, después se vacía el vaso (`corte::TODAS`).
    pub fn cortar(&mut self) {
        self.vivo.store(false, Ordering::Relaxed);
        self.grifo = None;
        if let Ok(mut a) = self.anillo.lock() {
            a.vaciar();
        }
        self.oreja.olvidar();
        // Soltar el canal acaba el hilo en cuanto termine el encargo que tenga en las manos; lo que
        // vuelva de él se pisa sin enviarse.
        self.manda = None;
        for mut t in self.llegan.try_iter() {
            if let Some(x) = &mut t.texto {
                // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
                unsafe { x.as_mut_vec() }.fill(0);
            }
        }
    }

    /// ¿Hay un micrófono abierto? (Para Honestidad y el corte.)
    pub fn abierto(&self) -> bool {
        self.grifo.is_some()
    }
}

impl Drop for Oido {
    fn drop(&mut self) {
        self.cortar();
    }
}

#[cfg(test)]
pub(crate) mod pruebas {
    use super::*;
    use crate::stt::{Disponibilidad, Fallo};

    /// Voz inventada: una senoide audible, para que el detector por energía la oiga.
    pub fn voz(ms: usize) -> Vec<f32> {
        let n = ms * HZ as usize / 1000;
        (0..n).map(|i| 0.3 * (i as f32 * 0.07).sin()).collect()
    }

    pub fn silencio(ms: usize) -> Vec<f32> {
        vec![0.0005; ms * HZ as usize / 1000]
    }

    /// Un motor que «transcribe» diciendo cuántas muestras le llegaron: así se ve qué audio se mandó.
    pub struct Contador;
    impl Motor for Contador {
        fn nombre(&self) -> &'static str {
            "contador"
        }
        fn disponibilidad(&self, _: &str) -> Disponibilidad {
            Disponibilidad::Listo
        }
        fn instalar(&self, _: &str) -> Disponibilidad {
            Disponibilidad::Listo
        }
        fn transcribir(&self, _: &str, m: &[f32], _: u32) -> Result<String, Fallo> {
            Ok(format!("{} muestras", m.len()))
        }
        fn techo_de_idiomas(&self) -> u32 {
            1
        }
        fn idiomas(&self) -> Vec<String> {
            vec!["es-ES".into()]
        }
    }

    fn escribir(anillo: &Arc<Mutex<Anillo>>, m: &[f32]) {
        anillo.lock().unwrap().escribir(m);
    }

    /// Un turno de voz entre silencios sale cerrado, con su audio entero y hace cuánto acabó.
    #[test]
    fn un_turno_tuyo_sale_con_su_audio_y_su_sitio() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let mut o = Oreja::nueva(anillo.clone());
        escribir(&anillo, &silencio(600)); // el detector aprende la sala
        assert!(o.mirar(true).is_empty());
        escribir(&anillo, &voz(1_000));
        assert!(o.mirar(true).is_empty());
        assert!(o.hablando());
        escribir(&anillo, &silencio(500));
        let c = o.mirar(true);
        assert_eq!(c.len(), 1);
        let t = &c[0];
        assert!((900..=1_040).contains(&t.duracion_ms), "duró {}", t.duracion_ms);
        assert_eq!(t.muestras.as_ref().unwrap().len() as u64, t.duracion_ms * 16);
        // Acabó hace el medio segundo de silencio, más o menos un marco.
        assert!((460..=520).contains(&t.hace_ms), "hace {}", t.hace_ms);
    }

    /// **Sordo, no oye**: la voz de la app que entra mientras lee no se convierte en turno, y el turno
    /// que estaba a medias se cierra con lo que tenía. Después, el reloj y el anillo siguen juntos: el
    /// turno siguiente se recorta con SU audio.
    #[test]
    fn sordo_no_oye_y_despues_el_reloj_sigue_cuadrado() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let mut o = Oreja::nueva(anillo.clone());
        escribir(&anillo, &silencio(600));
        o.mirar(true);
        escribir(&anillo, &voz(700));
        assert!(o.mirar(true).is_empty());
        // Se queda sordo a media frase: el turno se cierra con lo que ya tenía.
        escribir(&anillo, &voz(2_000));
        let c = o.mirar(false);
        assert_eq!(c.len(), 1, "el turno a medias se cierra al quedarse sordo");
        assert!((600..=760).contains(&c[0].duracion_ms), "duró {}", c[0].duracion_ms);
        // Más voz mientras está sordo: nada.
        escribir(&anillo, &voz(1_500));
        assert!(o.mirar(false).is_empty());
        assert!(!o.hablando());
        // Vuelve a oír: un silencio, un turno de 800 ms y su cierre. Si el origen no se hubiera movido
        // con la sordera, el recorte pediría el audio de la voz de la app.
        escribir(&anillo, &silencio(400));
        o.mirar(true);
        let mut marcado = voz(800);
        marcado[0] = 0.299; // una firma en la primera muestra del turno
        escribir(&anillo, &marcado);
        o.mirar(true);
        escribir(&anillo, &silencio(500));
        let c = o.mirar(true);
        assert_eq!(c.len(), 1);
        let m = c[0].muestras.as_ref().unwrap();
        assert!((m[0] - 0.299).abs() < 1e-6 || m.iter().take(400).any(|x| (*x - 0.299).abs() < 1e-6), "el turno no empieza en su audio");
        assert!((700..=840).contains(&c[0].duracion_ms), "duró {}", c[0].duracion_ms);
    }

    /// El camino entero con hilo: el turno va a transcribirse y vuelve con su ronda y su sitio; el corte
    /// cierra, vacía el anillo y lo que vuelva tarde no se entrega.
    #[test]
    fn el_oido_transcribe_cada_turno_y_el_corte_lo_vacia() {
        let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
        let mut o = Oido::con_anillo(anillo.clone(), "es-ES", Box::new(Contador), Arc::new(Diccionario::default()));
        escribir(&anillo, &silencio(600));
        o.latir(true, 7, 600);
        escribir(&anillo, &voz(1_000));
        o.latir(true, 7, 1_600);
        escribir(&anillo, &silencio(500));
        o.latir(true, 7, 2_100);
        let mut llegados = Vec::new();
        // Hasta 2 s de espera al hilo que transcribe: en el runner de la CI todo va más lento.
        for _ in 0..400 {
            llegados.extend(o.recibidos());
            if !llegados.is_empty() && o.pendientes() == 0 {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        assert_eq!(llegados.len(), 1);
        let t = &llegados[0];
        assert_eq!(t.ronda, 7);
        assert!(t.texto.as_deref().unwrap().ends_with("muestras"));
        assert!((1_560..=1_660).contains(&t.hasta_ms), "acabó en {}", t.hasta_ms);
        assert!(t.desde_ms < t.hasta_ms);
        o.cortar();
        assert_eq!(anillo.lock().unwrap().totales(), 0, "el anillo se vació");
        assert!(!o.abierto());
    }

    /// **Solo el micrófono** (ADR 019 §6.1): el oído del ensayo no nombra el audio del sistema, la
    /// pantalla ni el radar. Las agujas se arman con `concat!` para no contarse a sí mismas.
    #[test]
    fn el_oido_solo_abre_el_microfono() {
        for (nombre, fuente) in [
            ("oido.rs", include_str!("oido.rs")),
            ("sesion.rs", include_str!("sesion.rs")),
            ("evaluacion.rs", include_str!("evaluacion.rs")),
            ("mod.rs", include_str!("mod.rs")),
        ] {
            for prohibida in [concat!("del_", "sistema("), concat!("pantalla", "::"), concat!("radar", "::"), concat!("Escucha::", "arrancar")] {
                assert!(!fuente.contains(prohibida), "ensayo/{nombre} nombra «{prohibida}»: el ensayo solo abre el micrófono");
            }
        }
        assert_eq!(include_str!("oido.rs").matches(concat!("Grifo::", "del_microfono(")).count(), 1, "una sola entrada de audio");
    }
}
