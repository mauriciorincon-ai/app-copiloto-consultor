//! LA PANTALLA — leer lo que el cliente comparte, **solo cuando cambia**.
//!
//! **MÓDULO PROTEGIDO.** Lo que pasa por aquí es la pantalla de una reunión: la diapositiva del
//! cliente, los nombres de los participantes, su chat. La regla 1 de la casa lo pone del lado que
//! muere SIEMPRE, sin conmutador que lo encienda: el cuadro vive en un búfer de memoria que se pisa
//! con ceros al cortar, el texto leído se pisa antes de soltarse, y nada de esto toca un disco, un
//! log ni la red. `pnpm verify:ephemeral` barre esta carpeta y la de `nativo/`.
//!
//! ## El camino, y por qué tiene tres puertas
//!
//! ```text
//!   cada 500 ms           ¿cambió?              ¿≤ 1 por segundo?        ¿trae tema?
//!   ScreenCaptureKit ──▶  huella (pHash) ──▶    Vision (OCR) ──▶         refuerzo ──▶ buscador
//!   (solo la ventana      2 ms, en Rust          cientos de ms           cifras, títulos,
//!    de la reunión)                                                       tus términos
//! ```
//!
//! 1. **La huella** filtra los cuadros que no traen nada nuevo: un cursor, el grano del vídeo, el
//!    reloj de la llamada. Es aritmética barata sobre un búfer que ya está en memoria.
//! 2. **El vigía** ([`Vigia`]) espera a que la pantalla **se quede quieta** antes de leer —leer una
//!    transición entre diapositivas es leer basura— y no deja pasar más de una lectura por segundo,
//!    que es el presupuesto de CPU de la orden.
//! 3. **El refuerzo** ([`refuerzo`]) se queda con lo que habla del tema y tira la interfaz de la
//!    videollamada.
//!
//! ## Qué hace la app con lo que lee
//!
//! Dos cosas, y ninguna es «guardarlo»:
//!
//! - **Refuerza la búsqueda.** Cuando el cliente pregunta, lo que hay en su pantalla entra en la
//!   MISMA consulta con menos peso que sus palabras (`corpus::Corpus::buscar_con_pantalla`). No es
//!   una segunda búsqueda: es contexto.
//! - **Pide ficha por sí sola** si la pantalla nueva trae una cifra o uno de tus términos —la
//!   diapositiva de rentabilidad del cliente trae tu propuesta de rentabilidad sin que nadie
//!   pregunte—. Pasa por el mismo disparador que un turno, con su espera entre fichas.
//!
//! **Solo la ventana de la reunión.** ScreenCaptureKit puede ver la pantalla entera; aquí se le
//! pide UNA ventana, la de la videollamada que Sesión detectó. El correo del consultor, sus otros
//! documentos y la propia banda —que además está protegida de la captura— quedan fuera por
//! construcción, no por filtro.

pub mod apple;
pub mod huella;
pub mod refuerzo;

pub use refuerzo::{LineaLeida, Refuerzo};

use huella::{Huella, ZONAS};

/// Cada cuánto se mira la ventana de la reunión: 2 cuadros por segundo, el techo de la orden.
pub const CADA_MS: u64 = 500;

/// Más celdas cambiadas que esto **en alguna zona** contra lo último leído ⇒ la pantalla cambió.
///
/// Medido (bitácora, fase 3): un cursor mueve 2–5 celdas de su zona, el grano del vídeo 0, y entre
/// dos diapositivas del kit —dentro de la MISMA ventana de Meet— la menor diferencia es de 144. El
/// umbral cabe holgado entre las dos cosas; un cursor que se movió de un sitio a otro son dos
/// cursores, y sigue por debajo.
pub const CAMBIO: u32 = 12;

/// Como mucho esto **en cada zona** entre dos cuadros seguidos ⇒ la pantalla está quieta. Tiene que
/// ser menor que [`CAMBIO`]: el ruido que no mueve la pantalla tampoco puede moverla dos veces.
pub const QUIETO: u32 = 6;

/// Una zona que lleva cambiando esto de cuadros seguidos **no es una diapositiva: es vídeo** —la
/// cara de un participante, una animación—, y el vigía deja de mirarla hasta que se quede quieta.
/// Cuatro cuadros son dos segundos. Sin esto, una videollamada con cámaras encendidas estaría
/// «cambiando» siempre y se leería una vez por segundo toda la reunión.
pub const VIVA: u8 = 4;

/// Una lectura por segundo, como mucho. Es el presupuesto de CPU que fija la orden del sprint.
pub const RITMO_MS: u64 = 1_000;

/// El lado máximo al que se captura, en ancho y en alto. Vision lee bien texto de diapositiva a
/// este tamaño, y el búfer en grises ocupa como mucho 1600 × 1600 = 2,5 MB, reservados una vez.
pub const ANCHO_MAXIMO: usize = 1_600;

/// Un cuadro de la ventana de la reunión, en grises. **Se pisa con ceros al soltarse.**
#[derive(Debug, Default)]
pub struct Cuadro {
    pub ancho: usize,
    pub alto: usize,
    pub gris: Vec<u8>,
}

impl Cuadro {
    /// Un cuadro vacío con sitio para una captura del tamaño máximo. El búfer se reserva UNA vez y
    /// se reutiliza: capturar dos veces por segundo no puede ser reservar dos veces por segundo.
    pub fn con_sitio() -> Self {
        Self {
            ancho: 0,
            alto: 0,
            gris: vec![0; ANCHO_MAXIMO * ANCHO_MAXIMO],
        }
    }

    /// Los bytes de memoria que ocupa, estén llenos o no. Es lo que Honestidad cuenta.
    pub fn bytes(&self) -> usize {
        self.gris.len()
    }

    /// Fija el tamaño de la captura que acaba de llegar y **pisa la cola** si es más pequeña que
    /// la anterior. Sin esto, un cuadro de 800 px sobre uno de 1600 dejaba tres cuartos del viejo
    /// vivos en el búfer, y la cuenta de Honestidad —que mira el cuadro en uso— diría menos de lo
    /// que de verdad hay en memoria.
    pub fn ajustar(&mut self, ancho: usize, alto: usize) {
        let (antes, ahora) = (self.ancho * self.alto, ancho * alto);
        let fin = antes.min(self.gris.len());
        if ahora < fin {
            self.gris[ahora..fin].fill(0);
        }
        self.ancho = ancho;
        self.alto = alto;
    }

    /// Pisa el cuadro. Es lo que hace el kill-switch con la pieza «último fotograma».
    pub fn pisar(&mut self) {
        self.gris.fill(0);
        self.ancho = 0;
        self.alto = 0;
    }
}

impl Drop for Cuadro {
    fn drop(&mut self) {
        self.pisar();
    }
}

/// A qué ventana se mira: la de la videollamada que Sesión detectó.
#[derive(Debug, Clone, PartialEq)]
pub struct Objetivo {
    /// El identificador de la aplicación («us.zoom.xos», «com.google.Chrome»).
    pub bundle: String,
    /// Si es un navegador, lo que tiene que decir el título de la ventana («google meet»). Vacío
    /// para las aplicaciones de videollamada, que no necesitan título.
    pub senales: Vec<String>,
}

/// Por qué no se pudo mirar. **Un conjunto cerrado**, a propósito: la pantalla lo dice en dos
/// idiomas y un texto libre en español no se puede traducir.
#[derive(Debug, Clone, PartialEq)]
pub enum NoSeVe {
    /// macOS no ha concedido la grabación de pantalla. Se pregunta sin provocar el diálogo.
    SinPermiso,
    /// La ventana de la reunión no está en pantalla (minimizada, en otro escritorio, cerrada).
    SinVentana,
    /// Otra cosa. El detalle va al log —es metadata, no contenido— y no a la pantalla.
    Fallo(String),
}

/// Los ojos: capturan un cuadro de una ventana.
pub trait Ojo: Send + Sync {
    fn nombre(&self) -> &'static str;
    fn mirar(&self, objetivo: &Objetivo, cuadro: &mut Cuadro) -> Result<(), NoSeVe>;
}

/// El lector: del cuadro a sus líneas de texto.
pub trait Lector: Send + Sync {
    fn leer(&self, cuadro: &Cuadro) -> Result<Vec<LineaLeida>, String>;
}

/// **Los ojos que no ven**, de primera clase: sin puente de Swift, o fuera de macOS. La lectura de
/// pantalla simplemente no existe y lo dice con un motivo — la app funciona igual sin ella.
pub struct Ciegos;

impl Ojo for Ciegos {
    fn nombre(&self) -> &'static str {
        "ninguno"
    }
    fn mirar(&self, _objetivo: &Objetivo, _cuadro: &mut Cuadro) -> Result<(), NoSeVe> {
        Err(NoSeVe::Fallo(
            "esta compilación no trae el puente de pantalla".into(),
        ))
    }
}

impl Lector for Ciegos {
    fn leer(&self, _cuadro: &Cuadro) -> Result<Vec<LineaLeida>, String> {
        Err("esta compilación no trae el lector de texto".into())
    }
}

/// Qué hacer con el cuadro que acaba de llegar.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paso {
    /// Es lo mismo que ya se leyó.
    Igual,
    /// Cambió, pero todavía se está moviendo o ya se leyó hace menos de un segundo.
    Esperar,
    /// Cambió y se quedó quieto: se lee.
    Leer,
}

/// EL VIGÍA — decide cuándo vale la pena leer.
///
/// Es una máquina de estados sin reloj propio (recibe la hora), así que se prueba entera sin hilos
/// ni esperas. Trabaja **por zonas** (ver [`Huella`]) y con una máscara de las que no paran quietas.
#[derive(Debug, Default)]
pub struct Vigia {
    /// La huella de lo último que se leyó.
    leida: Option<Huella>,
    /// La huella del cuadro anterior, para saber si la pantalla está quieta.
    previa: Option<Huella>,
    /// Cuántos cuadros seguidos lleva cambiando cada zona. A partir de [`VIVA`], es vídeo.
    racha: [u8; ZONAS * ZONAS],
    ultima_lectura_ms: Option<u64>,
}

impl Vigia {
    pub fn mirar(&mut self, huella: Huella, ahora_ms: u64) -> Paso {
        let habia_previa = self.previa.is_some();
        if let Some(p) = &self.previa {
            for (r, d) in self.racha.iter_mut().zip(huella.distancias(p)) {
                *r = if d > QUIETO { r.saturating_add(1) } else { 0 };
            }
        }
        if let Some(mut vieja) = self.previa.replace(huella.clone()) {
            vieja.pisar();
        }
        let quietas: Vec<usize> = (0..ZONAS * ZONAS)
            .filter(|&i| self.racha[i] < VIVA)
            .collect();

        if let Some(l) = &self.leida {
            let d = huella.distancias(l);
            if quietas.iter().all(|&i| d[i] <= CAMBIO) {
                return Paso::Igual;
            }
        }
        // Quieta = ninguna zona (de las que no son vídeo) se movió desde el cuadro anterior. Una
        // transición o un desplazamiento no lo están, y leerlos sería leer un cuadro que ya no
        // existe cuando Vision termine.
        if !habia_previa || !quietas.iter().all(|&i| self.racha[i] == 0) {
            return Paso::Esperar;
        }
        if let Some(u) = self.ultima_lectura_ms {
            // Un reloj que retrocede no es «hace un instante» (la misma lección que el disparador).
            if ahora_ms >= u && ahora_ms - u < RITMO_MS {
                return Paso::Esperar;
            }
        }
        if let Some(mut vieja) = self.leida.replace(huella) {
            vieja.pisar();
        }
        self.ultima_lectura_ms = Some(ahora_ms);
        Paso::Leer
    }

    /// Lo que se leyó ahora, a la fuerza (el atajo): la próxima comparación parte de aquí.
    fn leido_a_la_fuerza(&mut self, huella: Huella, ahora_ms: u64) {
        self.olvidar();
        self.previa = Some(huella.clone());
        self.leida = Some(huella);
        self.ultima_lectura_ms = Some(ahora_ms);
    }

    /// Olvida lo que vio, **pisando las miniaturas**: la próxima pantalla quieta se lee aunque fuera
    /// la misma.
    pub fn olvidar(&mut self) {
        for h in [&mut self.leida, &mut self.previa].into_iter().flatten() {
            h.pisar();
        }
        *self = Self::default();
    }
}

/// Lo que salió de mirar una vez.
#[derive(Debug)]
pub enum Resultado {
    /// No hay videollamada abierta que mirar.
    SinReunion,
    NoSeVe(NoSeVe),
    Igual,
    Esperar,
    /// Se leyó. `ms` es lo que tardó Vision; `lineas`, cuántas devolvió; `aviso`, lo que el radar
    /// ámbar vio en esas mismas líneas (C14) — el aviso de grabación y los bots de notas.
    Leido {
        refuerzo: Refuerzo,
        aviso: crate::radar::Aviso,
        ms: u64,
        lineas: usize,
    },
    /// Vision falló. El motivo va al log.
    NoSeLeyo(String),
}

/// **UNA vuelta del vigía**: capturar, comparar y —si toca— leer. Sin hilos y sin reloj propio,
/// para poder probarla entera con ojos y lectores de mentira.
///
/// `forzar` es la lectura **bajo demanda**: el usuario la pidió con su atajo, así que se lee lo que
/// haya aunque no haya cambiado y aunque la última lectura fuera hace medio segundo.
#[allow(clippy::too_many_arguments)]
pub fn una_vuelta(
    ojo: &dyn Ojo,
    lector: &dyn Lector,
    objetivo: Option<&Objetivo>,
    vocabulario: &[String],
    cuadro: &mut Cuadro,
    vigia: &mut Vigia,
    ahora_ms: u64,
    forzar: bool,
) -> Resultado {
    let Some(objetivo) = objetivo else {
        return Resultado::SinReunion;
    };
    if let Err(e) = ojo.mirar(objetivo, cuadro) {
        return Resultado::NoSeVe(e);
    }
    let h = Huella::de(cuadro);
    let paso = if forzar {
        vigia.leido_a_la_fuerza(h, ahora_ms);
        Paso::Leer
    } else {
        vigia.mirar(h, ahora_ms)
    };
    match paso {
        Paso::Igual => Resultado::Igual,
        Paso::Esperar => Resultado::Esperar,
        Paso::Leer => {
            let reloj = std::time::Instant::now();
            match lector.leer(cuadro) {
                Ok(lineas) => {
                    let refuerzo = refuerzo::extraer(&lineas, vocabulario);
                    // El radar ámbar mira las MISMAS líneas, antes de que se pisen: no captura
                    // nada por su cuenta. Lo que Vision adivina no cuenta, igual que en el refuerzo.
                    let aviso = crate::radar::avisos::en_el_texto(
                        lineas
                            .iter()
                            .filter(|l| l.confianza >= refuerzo::CONFIANZA_MINIMA)
                            .map(|l| l.texto.as_str()),
                    );
                    let n = lineas.len();
                    // Las líneas son texto de un tercero: se pisan antes de soltarse.
                    for mut l in lineas {
                        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
                        unsafe { l.texto.as_mut_vec() }.fill(0);
                    }
                    Resultado::Leido {
                        refuerzo,
                        aviso,
                        ms: reloj.elapsed().as_millis() as u64,
                        lineas: n,
                    }
                }
                Err(e) => Resultado::NoSeLeyo(e),
            }
        }
    }
}

/// Qué está haciendo la lectura de pantalla, tal y como lo enseña Sesión.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Vista {
    /// El usuario apagó la lectura automática. El atajo sigue leyendo cuando él lo pide.
    Apagada,
    /// macOS no ha concedido la grabación de pantalla: la lectura no existe, y se dice.
    SinPermiso,
    /// Encendida, pero no hay videollamada que mirar (o su ventana no está en pantalla).
    EsperandoLaReunion,
    /// Mirando la ventana de la reunión.
    Leyendo,
    /// Algo falló al capturar o al leer. El detalle está en el log.
    NoPudo,
}

impl Vista {
    /// De lo que salió de una vuelta a lo que la pantalla enseña. `None` = no cambia nada.
    pub fn de(r: &Resultado) -> Option<Vista> {
        match r {
            Resultado::SinReunion | Resultado::NoSeVe(NoSeVe::SinVentana) => {
                Some(Vista::EsperandoLaReunion)
            }
            Resultado::NoSeVe(NoSeVe::SinPermiso) => Some(Vista::SinPermiso),
            Resultado::NoSeVe(NoSeVe::Fallo(_)) | Resultado::NoSeLeyo(_) => Some(Vista::NoPudo),
            Resultado::Igual | Resultado::Esperar | Resultado::Leido { .. } => Some(Vista::Leyendo),
        }
    }
}

/// Lo que la pantalla de Sesión y la de Honestidad preguntan. **Dos campos, los dos con lector**:
/// la vista es la fila de Sesión y los bytes son la fila de Honestidad.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDeLaPantalla {
    pub vista: Vista,
    /// Memoria que ocupa la lectura ahora mismo: el cuadro más el texto leído.
    ///
    /// Se llama así y no `bytes` a secas por una razón que no es de estilo: el gate de campos sin
    /// lector (`tests/unit/contrato-con-lectores.test.ts`) busca por NOMBRE, y `.bytes` ya lo lee
    /// Honestidad en las pistas de audio. Con el nombre corto, este campo habría pasado por leído sin
    /// que nadie lo leyera — el gate compara nombres, no tipos, y eso queda dicho en la bitácora.
    pub bytes_en_memoria: usize,
}

/// De dónde salió una lectura: del vigía, o del atajo del usuario.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Origen {
    /// La pantalla cambió y el vigía la leyó solo.
    Sola,
    /// El usuario la pidió con su atajo.
    Pedida,
}

/// Lo que se hace con cada lectura nueva: el refuerzo y de dónde salió.
pub type AlLeer = Box<dyn Fn(&Refuerzo, Origen) + Send>;

/// Lo que la lectura necesita del resto de la app, y nada más. **La firma es la frontera**: la
/// lectura no conoce ni la escucha, ni el corpus, ni la ventana de Tauri — recibe funciones.
pub struct Entorno {
    pub ojo: Box<dyn Ojo>,
    pub lector: Box<dyn Lector>,
    /// ¿A qué ventana se mira? Se pregunta cada pocos segundos: la reunión puede abrirse o cerrarse
    /// en mitad de la sesión.
    pub objetivo: Box<dyn Fn() -> Option<Objetivo> + Send>,
    /// Las palabras distintivas del corpus, para reconocer «tus términos» en la pantalla.
    pub vocabulario: Box<dyn Fn() -> Vec<String> + Send>,
    /// Se llama con cada lectura NUEVA. Es quien decide si pide ficha.
    pub al_leer: AlLeer,
    /// Se llama cuando cambia lo que Sesión tiene que enseñar.
    pub al_cambiar: Box<dyn Fn(EstadoDeLaPantalla) + Send>,
    /// Se llama cuando el radar ámbar ve algo NUEVO en la reunión: un aviso de grabación que no
    /// estaba, un bot más. El mismo aviso en cada diapositiva no se repite.
    pub al_avisar: Box<dyn Fn(&crate::radar::Aviso) + Send>,
}

/// Cada cuánto se vuelve a preguntar a qué ventana se mira. Preguntarlo en cada cuadro sería
/// recorrer las aplicaciones abiertas dos veces por segundo para una respuesta que casi nunca cambia.
const OBJETIVO_CADA_MS: u64 = 5_000;

/// Con qué grano duerme el hilo entre cuadros: lo bastante fino para que el corte y el atajo no
/// esperen medio segundo.
const GRANO_MS: u64 = 50;

/// LA LECTURA EN MARCHA — el hilo del vigía y lo que el kill-switch tiene que alcanzar.
///
/// El cuadro y el texto leído viven en `Arc<Mutex<…>>` **fuera** del hilo por la misma razón que
/// el disparador vive fuera del hilo de la escucha: [`Lectura::cortar`] tiene que poder pisarlos
/// desde otro sitio, y sin esperar a que el hilo se entere.
pub struct Lectura {
    viva: Arc<AtomicBool>,
    encendida: Arc<AtomicBool>,
    pedida: Arc<AtomicBool>,
    cuadro: Arc<Mutex<Cuadro>>,
    refuerzo: Arc<Mutex<Refuerzo>>,
    vista: Arc<Mutex<Vista>>,
}

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

impl Lectura {
    /// Arranca el vigía. `encendida` es el interruptor de Sesión: apagado, el hilo **no captura
    /// nada** —ni un cuadro— y solo despierta si el usuario pide una lectura con su atajo.
    pub fn arrancar(entorno: Entorno, encendida: bool) -> Self {
        let yo = Self {
            viva: Arc::new(AtomicBool::new(true)),
            encendida: Arc::new(AtomicBool::new(encendida)),
            pedida: Arc::new(AtomicBool::new(false)),
            cuadro: Arc::new(Mutex::new(Cuadro::con_sitio())),
            refuerzo: Arc::new(Mutex::new(Refuerzo::default())),
            vista: Arc::new(Mutex::new(if encendida {
                Vista::EsperandoLaReunion
            } else {
                Vista::Apagada
            })),
        };
        let (viva, enc, pedida) = (yo.viva.clone(), yo.encendida.clone(), yo.pedida.clone());
        let (cuadro, refuerzo, vista) = (yo.cuadro.clone(), yo.refuerzo.clone(), yo.vista.clone());
        std::thread::spawn(move || {
            let nacio = std::time::Instant::now();
            let mut vigia = Vigia::default();
            let mut objetivo: Option<Objetivo> = None;
            let mut objetivo_ms: Option<u64> = None;
            let mut antes = *vista.lock().unwrap();
            // Lo último que el radar ámbar contó. Se olvida cuando la reunión deja de verse.
            let mut ultimo_aviso = crate::radar::Aviso::default();
            while viva.load(Ordering::Relaxed) {
                let ahora = nacio.elapsed().as_millis() as u64;
                let forzar = pedida.swap(false, Ordering::Relaxed);
                let automatica = enc.load(Ordering::Relaxed);
                if !automatica && !forzar {
                    if antes != Vista::Apagada {
                        antes = Vista::Apagada;
                        *vista.lock().unwrap() = antes;
                        // Apagar la lectura es también olvidar lo que se leyó: el refuerzo no
                        // puede seguir empujando la búsqueda con una pantalla que ya no se mira.
                        refuerzo.lock().unwrap().olvidar();
                        vigia.olvidar();
                        (entorno.al_cambiar)(estado_de(&cuadro, &refuerzo, antes));
                    }
                    dormir(&viva, &pedida, CADA_MS);
                    continue;
                }
                if forzar || objetivo_ms.is_none_or(|m| ahora.saturating_sub(m) >= OBJETIVO_CADA_MS)
                {
                    objetivo = (entorno.objetivo)();
                    objetivo_ms = Some(ahora);
                }
                let vocabulario = (entorno.vocabulario)();
                let resultado = {
                    let Ok(mut c) = cuadro.lock() else { break };
                    una_vuelta(
                        entorno.ojo.as_ref(),
                        entorno.lector.as_ref(),
                        objetivo.as_ref(),
                        &vocabulario,
                        &mut c,
                        &mut vigia,
                        ahora,
                        forzar,
                    )
                };
                if !viva.load(Ordering::Relaxed) {
                    break;
                }
                match &resultado {
                    Resultado::Leido {
                        refuerzo: nuevo,
                        aviso,
                        ms,
                        lineas,
                    } => {
                        // Se avisa de lo NUEVO: el aviso de grabación sigue en la esquina de Meet
                        // en cada diapositiva, y la banda no puede repetirlo en cada una. Una
                        // lectura que no lo ve —Vision también falla— no borra lo que ya se contó.
                        if !aviso.vacio() && *aviso != ultimo_aviso {
                            println!(
                                "[radar] ámbar: grabación {} · {} bots",
                                if aviso.grabando { "sí" } else { "no" },
                                aviso.bots.len()
                            );
                            (entorno.al_avisar)(aviso);
                            ultimo_aviso = aviso.clone();
                        }
                        // Metadata, jamás contenido: cuántas líneas y cuánto tardó.
                        println!(
                            "[pantalla] leída en {ms} ms · {lineas} líneas · {} pistas",
                            nuevo.titulos.len() + nuevo.cifras.len() + nuevo.terminos.len()
                        );
                        let distinto = {
                            let mut r = refuerzo.lock().unwrap();
                            let distinto = *r != *nuevo;
                            r.olvidar();
                            *r = nuevo.clone();
                            distinto
                        };
                        if distinto || forzar {
                            (entorno.al_leer)(
                                nuevo,
                                if forzar { Origen::Pedida } else { Origen::Sola },
                            );
                        }
                    }
                    Resultado::SinReunion | Resultado::NoSeVe(_) => {
                        // Sin ventana que mirar, lo que se leyó ya no está en pantalla.
                        refuerzo.lock().unwrap().olvidar();
                        vigia.olvidar();
                        ultimo_aviso = crate::radar::Aviso::default();
                        if let Resultado::NoSeVe(NoSeVe::Fallo(m)) = &resultado {
                            println!("[pantalla] no se pudo mirar: {m}");
                        }
                    }
                    Resultado::NoSeLeyo(m) => println!("[pantalla] no se pudo leer: {m}"),
                    Resultado::Igual | Resultado::Esperar => {}
                }
                if let Some(v) = Vista::de(&resultado) {
                    // El atajo con la automática apagada no enciende la fila de Sesión.
                    let v = if automatica { v } else { Vista::Apagada };
                    if v != antes {
                        antes = v;
                        *vista.lock().unwrap() = v;
                        (entorno.al_cambiar)(estado_de(&cuadro, &refuerzo, v));
                    }
                }
                dormir(&viva, &pedida, CADA_MS);
            }
            // Al salir, por la razón que sea, no queda nada: ni el cuadro ni lo leído.
            if let Ok(mut c) = cuadro.lock() {
                c.pisar();
            }
            if let Ok(mut r) = refuerzo.lock() {
                r.olvidar();
            }
        });
        yo
    }

    /// El interruptor de Sesión.
    pub fn encender(&self, si: bool) {
        self.encendida.store(si, Ordering::Relaxed);
    }

    pub fn encendida(&self) -> bool {
        self.encendida.load(Ordering::Relaxed)
    }

    /// El atajo: lee la ventana de la reunión UNA vez, ahora, aunque la automática esté apagada.
    pub fn leer_ahora(&self) {
        self.pedida.store(true, Ordering::Relaxed);
    }

    /// Lo que la pantalla aporta ahora mismo a la búsqueda. Lo lee el buscador.
    pub fn refuerzo(&self) -> Arc<Mutex<Refuerzo>> {
        self.refuerzo.clone()
    }

    pub fn estado(&self) -> EstadoDeLaPantalla {
        estado_de(&self.cuadro, &self.refuerzo, *self.vista.lock().unwrap())
    }

    /// **El kill-switch, pieza «último fotograma».** Para el hilo y pisa el cuadro y lo leído.
    ///
    /// **No espera al hilo**, y es a propósito: `⌥⎋` llega por el manejador de atajos, en el hilo
    /// principal, y si el vigía está en mitad de una captura el candado del cuadro puede tardar lo
    /// que tarde ScreenCaptureKit en contestar (hasta 3 s de techo). Congelar la app entera tres
    /// segundos justo cuando el usuario pulsa la tecla de emergencia sería el peor momento posible.
    /// Si el cuadro está libre, se pisa aquí; si está en uso, lo pisa el propio hilo al salir de esa
    /// vuelta, que es lo primero que hace al ver `viva = false`. El texto leído se pisa siempre aquí:
    /// su candado nunca se retiene más que una copia.
    pub fn cortar(&self) {
        self.viva.store(false, Ordering::Relaxed);
        match self.cuadro.try_lock() {
            Ok(mut c) => c.pisar(),
            Err(_) => println!("[pantalla] cuadro en uso: lo pisa el vigía al terminar la vuelta"),
        }
        if let Ok(mut r) = self.refuerzo.lock() {
            r.olvidar();
        }
    }
}

impl Drop for Lectura {
    fn drop(&mut self) {
        self.cortar();
    }
}

/// Lo que se enseña: la vista, y los bytes del cuadro **solo si hay uno**. Un búfer reservado y
/// vacío no guarda nada de nadie, y contarlo asustaría sin razón.
fn estado_de(
    cuadro: &Mutex<Cuadro>,
    refuerzo: &Mutex<Refuerzo>,
    vista: Vista,
) -> EstadoDeLaPantalla {
    let del_cuadro = cuadro
        .lock()
        .map(|c| if c.ancho > 0 { c.ancho * c.alto } else { 0 })
        .unwrap_or(0);
    let del_texto = refuerzo.lock().map(|r| r.bytes()).unwrap_or(0);
    EstadoDeLaPantalla {
        vista,
        bytes_en_memoria: del_cuadro + del_texto,
    }
}

/// Duerme `ms` en trozos pequeños, despertando antes si se corta o si el usuario pide una lectura.
fn dormir(viva: &AtomicBool, pedida: &AtomicBool, ms: u64) {
    let mut resto = ms;
    while resto > 0 && viva.load(Ordering::Relaxed) && !pedida.load(Ordering::Relaxed) {
        let paso = resto.min(GRANO_MS);
        std::thread::sleep(std::time::Duration::from_millis(paso));
        resto -= paso;
    }
}

#[cfg(test)]
mod pruebas {
    use super::huella::pruebas::{con_cursor, diapositiva};
    use super::*;

    /// Unos ojos que devuelven, en orden, los cuadros que se les dan.
    struct Guion(Mutex<Vec<Result<Cuadro, NoSeVe>>>);

    impl Ojo for Guion {
        fn nombre(&self) -> &'static str {
            "guion"
        }
        fn mirar(&self, _o: &Objetivo, cuadro: &mut Cuadro) -> Result<(), NoSeVe> {
            let siguiente = self.0.lock().unwrap().remove(0)?;
            cuadro.gris[..siguiente.gris.len()].copy_from_slice(&siguiente.gris);
            cuadro.ajustar(siguiente.ancho, siguiente.alto);
            Ok(())
        }
    }

    /// Un lector que cuenta cuántas veces se le pidió leer.
    struct Contador(Mutex<usize>);

    impl Lector for Contador {
        fn leer(&self, _c: &Cuadro) -> Result<Vec<LineaLeida>, String> {
            *self.0.lock().unwrap() += 1;
            Ok(vec![LineaLeida {
                texto: "Margen por canal: 23 %".into(),
                confianza: 0.9,
                alto: 0.05,
            }])
        }
    }

    fn meet() -> Objetivo {
        Objetivo {
            bundle: "com.google.Chrome".into(),
            senales: vec!["google meet".into()],
        }
    }

    #[test]
    fn el_vigia_espera_a_que_la_pantalla_se_quede_quieta() {
        let a = Huella::de(&diapositiva(&[40, 70, 55], 1280, 720));
        let mut v = Vigia::default();
        assert_eq!(
            v.mirar(a.clone(), 0),
            Paso::Esperar,
            "el primer cuadro no tiene con qué compararse"
        );
        assert_eq!(v.mirar(a.clone(), 500), Paso::Leer);
        assert_eq!(v.mirar(a, 1_000), Paso::Igual);
    }

    #[test]
    fn una_transicion_no_se_lee_hasta_que_termina() {
        let a = Huella::de(&diapositiva(&[40, 70, 55], 1280, 720));
        let b = Huella::de(&diapositiva(&[85, 20, 60, 35, 90], 1280, 720));
        let c = Huella::de(&diapositiva(&[10, 95, 30, 80], 1280, 720));
        let mut v = Vigia::default();
        v.mirar(a.clone(), 0);
        v.mirar(a, 500);
        assert_eq!(
            v.mirar(b, 1_500),
            Paso::Esperar,
            "cambió, pero todavía no se sabe si se queda"
        );
        assert_eq!(v.mirar(c.clone(), 2_000), Paso::Esperar, "sigue moviéndose");
        assert_eq!(v.mirar(c, 2_500), Paso::Leer);
    }

    #[test]
    fn no_se_lee_mas_de_una_vez_por_segundo() {
        let a = Huella::de(&diapositiva(&[40, 70, 55], 1280, 720));
        let b = Huella::de(&diapositiva(&[85, 20, 60, 35, 90], 1280, 720));
        let mut v = Vigia::default();
        v.mirar(a.clone(), 0);
        assert_eq!(v.mirar(a, 100), Paso::Leer);
        v.mirar(b.clone(), 200);
        assert_eq!(
            v.mirar(b.clone(), 300),
            Paso::Esperar,
            "quieta, pero leída hace 200 ms"
        );
        assert_eq!(v.mirar(b, 1_100), Paso::Leer);
    }

    #[test]
    fn un_cursor_que_pasa_no_provoca_otra_lectura() {
        let a = diapositiva(&[40, 70, 55], 1280, 720);
        let con = con_cursor(diapositiva(&[40, 70, 55], 1280, 720), 300, 200);
        let mut v = Vigia::default();
        v.mirar(Huella::de(&a), 0);
        v.mirar(Huella::de(&a), 500);
        assert_eq!(v.mirar(Huella::de(&con), 1_000), Paso::Igual);
    }

    /// Una diapositiva quieta con **un participante moviéndose** en su recuadro: la zona del vídeo no
    /// para, y aun así la diapositiva se lee UNA vez y no una cada segundo.
    #[test]
    fn el_video_de_un_participante_no_impide_leer_ni_obliga_a_releer() {
        let con_video = |t: usize| {
            let mut c = diapositiva(&[40, 70, 55], 1280, 720);
            // Un recuadro de 240 × 160 abajo a la derecha, con un «rostro» que se mueve.
            for y in 540..700 {
                for x in 1020..1260 {
                    c.gris[y * 1280 + x] = (((x + t * 37) ^ (y * 3 + t * 11)) % 200) as u8;
                }
            }
            Huella::de(&c)
        };
        let mut v = Vigia::default();
        let mut lecturas = 0;
        for t in 0..20 {
            if v.mirar(con_video(t), t as u64 * 500) == Paso::Leer {
                lecturas += 1;
            }
        }
        assert_eq!(
            lecturas, 1,
            "con vídeo en pantalla se leyó {lecturas} veces en diez segundos"
        );
    }

    #[test]
    fn una_vuelta_lee_solo_cuando_toca_y_bajo_demanda_siempre() {
        let cuadros = vec![
            Ok(diapositiva(&[40, 70, 55], 640, 360)),
            Ok(diapositiva(&[40, 70, 55], 640, 360)),
            Ok(diapositiva(&[40, 70, 55], 640, 360)),
            Ok(diapositiva(&[40, 70, 55], 640, 360)),
        ];
        let ojo = Guion(Mutex::new(cuadros));
        let lector = Contador(Mutex::new(0));
        let (mut c, mut v) = (Cuadro::con_sitio(), Vigia::default());
        let o = meet();
        assert!(matches!(
            una_vuelta(&ojo, &lector, Some(&o), &[], &mut c, &mut v, 0, false),
            Resultado::Esperar
        ));
        let r = una_vuelta(&ojo, &lector, Some(&o), &[], &mut c, &mut v, 500, false);
        let Resultado::Leido {
            refuerzo, lineas, ..
        } = r
        else {
            panic!("tenía que leer: {r:?}")
        };
        assert_eq!(lineas, 1);
        assert_eq!(refuerzo.cifras, vec!["Margen por canal: 23 %"]);
        assert!(matches!(
            una_vuelta(&ojo, &lector, Some(&o), &[], &mut c, &mut v, 1_000, false),
            Resultado::Igual
        ));
        // El atajo: lee aunque sea lo mismo y aunque haga menos de un segundo.
        assert!(matches!(
            una_vuelta(&ojo, &lector, Some(&o), &[], &mut c, &mut v, 1_200, true),
            Resultado::Leido { .. }
        ));
        assert_eq!(
            *lector.0.lock().unwrap(),
            2,
            "dos lecturas en cuatro cuadros, ni una más"
        );
    }

    #[test]
    fn sin_reunion_ni_se_captura() {
        let ojo = Guion(Mutex::new(vec![]));
        let lector = Contador(Mutex::new(0));
        let r = una_vuelta(
            &ojo,
            &lector,
            None,
            &[],
            &mut Cuadro::con_sitio(),
            &mut Vigia::default(),
            0,
            true,
        );
        assert!(matches!(r, Resultado::SinReunion));
        assert_eq!(Vista::de(&r), Some(Vista::EsperandoLaReunion));
    }

    #[test]
    fn sin_permiso_se_dice_sin_permiso() {
        let ojo = Guion(Mutex::new(vec![Err(NoSeVe::SinPermiso)]));
        let r = una_vuelta(
            &ojo,
            &Ciegos,
            Some(&meet()),
            &[],
            &mut Cuadro::con_sitio(),
            &mut Vigia::default(),
            0,
            false,
        );
        assert_eq!(Vista::de(&r), Some(Vista::SinPermiso));
    }

    #[test]
    fn el_cuadro_se_pisa_al_cortar() {
        let mut c = Cuadro::con_sitio();
        c.gris[..4].copy_from_slice(&[9, 9, 9, 9]);
        c.ancho = 2;
        c.alto = 2;
        c.pisar();
        assert!(c.gris.iter().all(|&b| b == 0));
        assert_eq!((c.ancho, c.alto), (0, 0));
        assert_eq!(
            c.bytes(),
            ANCHO_MAXIMO * ANCHO_MAXIMO,
            "el búfer se reutiliza, no se suelta"
        );
    }

    #[test]
    fn un_cuadro_mas_pequeno_no_deja_vivo_el_anterior() {
        let mut c = Cuadro::con_sitio();
        c.gris[..100].fill(7);
        c.ajustar(10, 10);
        c.gris[..16].fill(3);
        c.ajustar(4, 4);
        assert!(
            c.gris[16..100].iter().all(|&b| b == 0),
            "la cola del cuadro grande sigue en memoria"
        );
        assert!(c.gris[..16].iter().all(|&b| b == 3));
    }

    /// Unos ojos que devuelven siempre la misma diapositiva y cuentan cuántas veces miraron.
    struct Fijos(Arc<Mutex<usize>>);

    impl Ojo for Fijos {
        fn nombre(&self) -> &'static str {
            "fijos"
        }
        fn mirar(&self, _o: &Objetivo, cuadro: &mut Cuadro) -> Result<(), NoSeVe> {
            *self.0.lock().unwrap() += 1;
            let d = diapositiva(&[40, 70, 55], 320, 180);
            cuadro.gris[..d.gris.len()].copy_from_slice(&d.gris);
            cuadro.ajustar(d.ancho, d.alto);
            Ok(())
        }
    }

    fn entorno(miradas: Arc<Mutex<usize>>, leidas: Arc<Mutex<Vec<Origen>>>) -> Entorno {
        Entorno {
            ojo: Box::new(Fijos(miradas)),
            lector: Box::new(Contador(Mutex::new(0))),
            objetivo: Box::new(|| Some(meet())),
            vocabulario: Box::new(Vec::new),
            al_leer: Box::new(move |_r, origen| leidas.lock().unwrap().push(origen)),
            al_cambiar: Box::new(|_| {}),
            al_avisar: Box::new(|_| {}),
        }
    }

    /// Una reunión de Meet con su aviso de grabación y un bot de notas en la lista, escritos con
    /// las frases del catálogo del radar.
    struct Grabada;

    impl Lector for Grabada {
        fn leer(&self, _c: &Cuadro) -> Result<Vec<LineaLeida>, String> {
            let linea = |t: &str| LineaLeida { texto: t.into(), confianza: 0.9, alto: 0.03 };
            Ok(vec![
                linea(&crate::radar::catalogo::grabacion()[0].frases[0]),
                linea(&crate::radar::catalogo::bots()[0].patrones[0]),
                linea("Margen por canal: 23 %"),
            ])
        }
    }

    /// **El radar ámbar avisa UNA vez de lo mismo.** El aviso de grabación sigue en la esquina de
    /// Meet en cada diapositiva; si la banda lo repitiera en cada una, taparía las fichas toda la
    /// reunión. Dos lecturas más, pedidas con el atajo, no lo repiten.
    #[test]
    fn el_radar_ambar_avisa_una_vez_de_lo_mismo() {
        let avisos = Arc::new(Mutex::new(Vec::<crate::radar::Aviso>::new()));
        let (miradas, leidas) = (Arc::new(Mutex::new(0)), Arc::new(Mutex::new(Vec::new())));
        let mut e = entorno(miradas, leidas.clone());
        e.lector = Box::new(Grabada);
        let a = avisos.clone();
        e.al_avisar = Box::new(move |x| a.lock().unwrap().push(x.clone()));
        let l = Lectura::arrancar(e, true);
        assert!(esperar_a(|| !avisos.lock().unwrap().is_empty()), "no avisó de la grabación");
        for _ in 0..2 {
            let antes = leidas.lock().unwrap().len();
            l.leer_ahora();
            assert!(esperar_a(|| leidas.lock().unwrap().len() > antes));
        }
        let avisos = avisos.lock().unwrap();
        assert_eq!(avisos.len(), 1, "el mismo aviso se repitió: {avisos:?}");
        assert!(avisos[0].grabando);
        assert_eq!(avisos[0].bots, vec![crate::radar::catalogo::bots()[0].nombre.clone()]);
        l.cortar();
    }

    fn esperar_a(que: impl Fn() -> bool) -> bool {
        for _ in 0..80 {
            if que() {
                return true;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        false
    }

    /// El hilo entero, con ojos de mentira: lee UNA vez una pantalla quieta, y el corte deja el
    /// cuadro y lo leído en cero.
    #[test]
    fn la_lectura_en_marcha_lee_una_vez_y_el_corte_lo_pisa_todo() {
        let (miradas, leidas) = (Arc::new(Mutex::new(0)), Arc::new(Mutex::new(Vec::new())));
        let l = Lectura::arrancar(entorno(miradas.clone(), leidas.clone()), true);
        assert!(
            esperar_a(|| !leidas.lock().unwrap().is_empty()),
            "la pantalla quieta no se leyó nunca"
        );
        assert_eq!(l.estado().vista, Vista::Leyendo);
        assert!(
            l.estado().bytes_en_memoria > 0,
            "hay un cuadro y un texto en memoria, y Honestidad tiene que verlos"
        );
        assert!(!l.refuerzo().lock().unwrap().vacio());
        std::thread::sleep(std::time::Duration::from_millis(1_200));
        assert_eq!(
            *leidas.lock().unwrap(),
            vec![Origen::Sola],
            "la misma pantalla se leyó dos veces"
        );

        l.cortar();
        assert!(esperar_a(|| l
            .cuadro
            .lock()
            .map(|c| c.gris.iter().all(|&b| b == 0))
            .unwrap_or(false)));
        assert!(l.refuerzo().lock().unwrap().vacio());
        assert_eq!(l.estado().bytes_en_memoria, 0);
        let tras_el_corte = *miradas.lock().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(700));
        assert_eq!(
            *miradas.lock().unwrap(),
            tras_el_corte,
            "el vigía siguió mirando después del corte"
        );
    }

    /// Apagada, **no se captura ni un cuadro**. El atajo lee una vez y dice que fue pedida.
    #[test]
    fn apagada_no_mira_nada_hasta_que_se_lo_piden() {
        let (miradas, leidas) = (Arc::new(Mutex::new(0)), Arc::new(Mutex::new(Vec::new())));
        let l = Lectura::arrancar(entorno(miradas.clone(), leidas.clone()), false);
        std::thread::sleep(std::time::Duration::from_millis(800));
        assert_eq!(
            *miradas.lock().unwrap(),
            0,
            "con la lectura apagada se capturó la pantalla"
        );
        assert_eq!(l.estado().vista, Vista::Apagada);
        l.leer_ahora();
        assert!(esperar_a(|| !leidas.lock().unwrap().is_empty()));
        assert_eq!(*leidas.lock().unwrap(), vec![Origen::Pedida]);
        assert_eq!(
            *miradas.lock().unwrap(),
            1,
            "el atajo es UNA lectura, no un encendido"
        );
        assert_eq!(
            l.estado().vista,
            Vista::Apagada,
            "leer a petición no enciende la fila de Sesión"
        );
    }

    #[test]
    fn los_ciegos_no_ven_y_lo_dicen() {
        let r = una_vuelta(
            &Ciegos,
            &Ciegos,
            Some(&meet()),
            &[],
            &mut Cuadro::con_sitio(),
            &mut Vigia::default(),
            0,
            true,
        );
        assert_eq!(Vista::de(&r), Some(Vista::NoPudo));
    }
}
