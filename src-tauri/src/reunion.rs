//! LA REUNIÓN ABIERTA — el cuaderno de la reunión y su archivo, cableados con la app (C9, ADR 015).
//!
//! Lo que es tuyo lo decide `notas::Cuaderno` (protegido y puro); lo que va a disco pasa por
//! `carpeta.rs`. Aquí se decide **cuándo**: cuándo empieza una reunión y cuándo se cierra, cuándo se
//! protege el cuaderno, qué hace ⌃⌥P, qué pasa al salir de la app y cada cuánto se barre lo vencido.
//!
//! **Una reunión está abierta** desde que empieza una sesión hasta que la guardas o la descartas.
//! Parar de escuchar no la cierra —queda «al cerrar», con tus notas a la vista— y ⌥⎋ tampoco: tus
//! notas sobreviven al corte, y el cuaderno sigue protegido (ADR 015 §10).

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Instant;

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::bandeja::{self, Bandeja, Ventana};
use crate::carpeta::{self, Carpeta, DelLlavero, Guardada, Reunion};
use crate::desbloqueo::{self, Desbloqueo};
use crate::notas::{self, Contenido, Cuaderno, Encabezado, EnEspera, FichaFijada, Resumen};
use crate::propuestas::Propuesta;
use crate::{prefs, vencimiento, ventana};

/// El evento que avisa a la ventana principal de que el cuaderno cambió desde fuera (⌃⌥P, el
/// principio o el fin de una reunión, el corte). Es una señal: la pantalla vuelve a preguntar.
pub const EVENTO: &str = "cuaderno";

/// El evento que lleva a la banda la última propuesta sin decidir —o `null` si no queda ninguna—:
/// la línea pasiva de la mirada 20 (ADR 016 §3).
pub const EVENTO_PROPUESTA: &str = "propuesta";

/// El evento que le dice a la banda que ⌃⌥P fijó la ficha que enseña: la señal «fijada».
pub const EVENTO_FIJADA: &str = "fijada";

/// Mientras la app corre, se barre lo vencido **al llegar el próximo vencimiento**, y como mucho
/// cada hora. Con la app cerrada lo cumple launchd (ADR 016 §5).
pub const BARRIDO_CADA: std::time::Duration = std::time::Duration::from_secs(3_600);

/// La bandeja que la pantalla puede leer sin pedir nada: la que se acaba de escribir al cerrar, o la
/// que se abrió con Touch ID. Vive en memoria mientras la app corre (ADR 016 §4).
struct BandejaAbierta {
    archivo: String,
    contenido: bandeja::Contenido,
    vence: i64,
    /// Las que guardaste desde aquí mientras la app sigue abierta: la pantalla las enseña «guardada».
    guardadas: Vec<Propuesta>,
}

impl Drop for BandejaAbierta {
    fn drop(&mut self) {
        self.guardadas.iter_mut().for_each(notas::pisar_propuesta);
    }
}

/// Lo que pasó al cerrar: lo que se guardó y cuántas propuestas fueron a la bandeja o murieron.
#[derive(Debug, Default)]
struct Cierre {
    guardada: Option<(Guardada, Resumen)>,
    a_la_bandeja: usize,
    murieron: usize,
}

#[derive(Default)]
pub struct ElCuaderno {
    cuaderno: Mutex<Cuaderno>,
    abierta: AtomicBool,
    /// Cuándo empezó la reunión: la fecha del reloj (para el nombre y el encabezado) y el instante
    /// (para los minutos).
    empezo: Mutex<Option<(notas::Fecha, Instant)>>,
    /// Cuándo dejaste de escuchar. Los minutos cuentan la sesión, no lo que tardes en guardar.
    termino: Mutex<Option<Instant>>,
    /// **Lo que muere, contado** («Muere al cerrar»): cuántos turnos dijo el cliente y cuántas veces
    /// se leyó la pantalla en esta reunión. Solo el número: el contenido nunca pasa por aquí.
    turnos_del_cliente: AtomicU32,
    lecturas: AtomicU32,
    pub desbloqueo: Desbloqueo,
    bandeja: Mutex<Option<BandejaAbierta>>,
    /// Al arrancar se midió que la tarea de borrado no corrió con la app cerrada (ADR 016 §5).
    no_corrio: AtomicBool,
    /// macOS no dejó proteger el cuaderno al empezar (auditoría del S3, B4).
    sin_proteger: AtomicBool,
    /// **«Este cliente»** (ADR 017 §3): el que elegiste en Sesión. Nombra el archivo de la reunión y
    /// elige la bandera y la NDA. Vive en memoria, mientras la app esté abierta: no se guarda.
    cliente: Mutex<Option<String>>,
    /// La reunión va en **modo solo notas** (ADR 017 §5): sin captura. La banda y Sesión lo dicen.
    solo_notas: AtomicBool,
}

/// Lo que la pantalla de Notas enseña del cuaderno de ahora.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VistaDelCuaderno {
    pub nota: String,
    pub acuerdos: Vec<String>,
    pub fijadas: Vec<FichaFijada>,
    pub resumen: Resumen,
    pub conservar_mis_turnos: bool,
    /// Hay una reunión abierta (sesión en marcha, o parada y sin guardar ni descartar).
    pub abierta: bool,
    /// La sesión sigue escuchando: Notas enseña «durante»; parada, «al cerrar».
    pub escuchando: bool,
    /// Lo que se escribiría al guardar ahora: «2026-09-27 · 47 min → reunion-2026-09-27-1402.ghost».
    pub previsto: Option<Previsto>,
    /// «Muere al cerrar»: cuántos turnos dijo el cliente y cuántas lecturas de pantalla hubo.
    pub turnos_del_cliente: u32,
    pub lecturas: u32,
    pub retencion: prefs::Retencion,
    /// Las propuestas que esperan tu decisión, en el orden en que llegaron (ADR 016 §3).
    pub propuestas: Vec<EnEspera>,
    /// Se llegó al tope de propuestas y alguna no entró.
    pub lleno: bool,
    /// Cuánto esperarán en la bandeja las que no decidas.
    pub ventana: Ventana,
    /// macOS no dejó proteger el cuaderno al empezar: tu nota se vería al compartir la pantalla entera
    /// (auditoría del S3, B4). Notas lo dice en una franja.
    pub sin_proteger: bool,
}

/// La bandeja que Notas enseña: la que vence antes (ADR 016 §4).
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VistaDeLaBandeja {
    pub archivo: String,
    /// Segundos Unix.
    pub vence: i64,
    pub bytes: u64,
    /// `None`: cerrada con llave. Se escribió en otra sesión de la app, y abrirla pide Touch ID como
    /// abrir una reunión.
    pub propuestas: Option<Vec<Propuesta>>,
    /// Las que guardaste desde aquí mientras la app sigue abierta.
    pub guardadas: Vec<Propuesta>,
    /// Cuántas bandejas más esperan, además de esta.
    pub mas: usize,
    pub ventana: Ventana,
}

/// Lo que Honestidad dice de la bandeja, sin abrirla: cuándo vence la próxima y si la tarea de
/// borrado corrió (ADR 016 §6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDeLaBandeja {
    /// El vencimiento más próximo, en segundos Unix; `None` si no hay bandeja.
    pub vence: Option<i64>,
    /// Al arrancar había algo vencido hace más de 2 minutos con la tarea instalada: no corrió.
    pub no_corrio: bool,
}

/// La línea de «al cerrar» que dice qué archivo va a nacer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Previsto {
    /// «2026-09-27».
    pub fecha: String,
    pub minutos: u32,
    pub cliente: Option<String>,
    pub archivo: String,
}

/// Las reuniones guardadas. Dónde viven no viaja: siempre en la carpeta de la app (decisión A), que la
/// interfaz nombra en su idioma y enseña con «Mostrar en Finder».
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListaDeReuniones {
    pub reuniones: Vec<Reunion>,
}

impl ElCuaderno {
    fn con<T>(&self, f: impl FnOnce(&mut Cuaderno) -> T) -> Option<T> {
        self.cuaderno.lock().ok().map(|mut c| f(&mut c))
    }

    pub fn abierta(&self) -> bool {
        self.abierta.load(Ordering::Relaxed)
    }

    fn vacio(&self) -> bool {
        self.con(|c| c.vacio()).unwrap_or(true)
    }

    // ---- las reglas, sin la app: se prueban con una carpeta temporal y una llave en memoria ----

    /// Abre una reunión. Devuelve si la anterior seguía abierta **con algo tuyo o propuestas sin
    /// decidir** —entonces hay que guardarla antes, y lo hace quien llama—.
    fn hay_que_guardar_la_anterior(&self) -> bool {
        self.abierta() && self.hay_que_decidir()
    }

    fn hay_que_decidir(&self) -> bool {
        self.con(|c| c.hay_que_decidir()).unwrap_or(false)
    }

    fn abrir(&self, conservar_mis_turnos: bool, fecha: notas::Fecha) {
        self.con(|c| c.conservar_mis_turnos(conservar_mis_turnos));
        self.turnos_del_cliente.store(0, Ordering::Relaxed);
        self.lecturas.store(0, Ordering::Relaxed);
        if let Ok(mut e) = self.empezo.lock() {
            *e = Some((fecha, Instant::now()));
        }
        if let Ok(mut t) = self.termino.lock() {
            *t = None;
        }
        self.abierta.store(true, Ordering::Relaxed);
    }

    /// Paraste de escuchar. Devuelve si la reunión se cerró: no había nada tuyo **ni propuestas
    /// esperando tu decisión** (ADR 016 §4, que enmienda el ADR 015 §7).
    fn terminar(&self) -> bool {
        if let Ok(mut t) = self.termino.lock() {
            t.get_or_insert_with(Instant::now);
        }
        if !self.con(|c| c.hay_que_decidir()).unwrap_or(false) {
            return self.cerrar();
        }
        false
    }

    /// Olvida el cuaderno y cierra la reunión. Devuelve si estaba abierta (hay que soltar la
    /// protección del cuaderno).
    fn cerrar(&self) -> bool {
        self.con(Cuaderno::olvidar);
        if let Ok(mut e) = self.empezo.lock() {
            *e = None;
        }
        if let Ok(mut t) = self.termino.lock() {
            *t = None;
        }
        self.abierta.swap(false, Ordering::Relaxed)
    }

    fn cliente(&self) -> Option<String> {
        self.cliente.lock().ok().and_then(|c| c.clone())
    }

    fn escuchando(&self) -> bool {
        self.abierta()
            && self.empezo.lock().map(|e| e.is_some()).unwrap_or(false)
            && self.termino.lock().map(|t| t.is_none()).unwrap_or(false)
    }

    /// La fecha y los minutos de la reunión: los de la sesión, o los de ahora si no hubo sesión.
    fn cuando(&self) -> (notas::Fecha, u32) {
        let empezo = self.empezo.lock().ok().and_then(|e| *e);
        let termino = self.termino.lock().ok().and_then(|t| *t).unwrap_or_else(Instant::now);
        match empezo {
            Some((f, desde)) => (f, (termino.saturating_duration_since(desde).as_secs() / 60) as u32),
            None => (fecha_de_ahora(), 0),
        }
    }

    /// Lo que se escribiría al guardar ahora. `None` sin nada tuyo: no habría archivo.
    fn previsto(&self, carpeta: &Carpeta, bandeja: &Bandeja) -> Option<Previsto> {
        if self.vacio() {
            return None;
        }
        let (fecha, minutos) = self.cuando();
        let cliente = self.cliente();
        let base = notas::nombre_del_archivo(cliente.as_deref(), &fecha);
        Some(Previsto {
            fecha: format!("{}-{:02}-{:02}", fecha.anio, fecha.mes, fecha.dia),
            minutos,
            archivo: carpeta::nombre_libre(&base, &[carpeta.raiz(), bandeja.raiz()])
                .unwrap_or_else(|| format!("{base}.{}", carpeta::EXTENSION)),
            cliente,
        })
    }

    /// Guarda en `carpeta` lo tuyo y deja en `bandeja` lo que no decidiste, hasta `vence_bandeja`
    /// (`None`: «al cerrar», no hay bandeja y mueren aquí). Sin nada tuyo ni propuestas, cierra y ya.
    ///
    /// **La bandeja se escribe PRIMERO.** Si falla, no se escribió nada y la reunión sigue abierta
    /// entera; si después fallan las notas, la bandeja recién escrita se deshace. Nada tuyo se pierde
    /// por un error de disco o de Llavero, y nada queda a medias.
    fn guardar_en(
        &self,
        carpeta: &Carpeta,
        bandeja: &Bandeja,
        llaves: &dyn carpeta::Llaves,
        retencion: prefs::Retencion,
        vence_bandeja: Option<i64>,
        ahora: i64,
    ) -> Result<Cierre, String> {
        if !self.hay_que_decidir() {
            self.cerrar();
            return Ok(Cierre::default());
        }
        let (fecha, minutos) = self.cuando();
        // «Este cliente» (ADR 017 §3): el que elegiste; sin elegir, ninguno. La app no lo adivina.
        let cliente = self.cliente();
        let encabezado = Encabezado { empezo: fecha.como_texto(), minutos, cliente: cliente.clone() };
        let (contenido, resumen, vacio, pendientes): (Contenido, Resumen, bool, Vec<Propuesta>) = self
            .con(|c| {
                let pendientes = c.en_espera().iter().map(|e| e.propuesta.clone()).collect();
                (c.contenido(encabezado.clone()), c.resumen(), c.vacio(), pendientes)
            })
            .ok_or("el cuaderno quedó en mal estado")?;
        let base = notas::nombre_del_archivo(cliente.as_deref(), &fecha);
        // Libre en las notas **y** en la bandeja (A1): la bandeja de otra reunión de hoy no se pisa.
        let archivo = carpeta::nombre_libre(&base, &[carpeta.raiz(), bandeja.raiz()])
            .ok_or("no queda un nombre libre para esta reunión")?;
        let vence_de_la_reunion = retencion.vence(ahora);

        let sin_decidir = pendientes.len();
        let a_la_bandeja = if pendientes.is_empty() {
            None
        } else {
            let b = bandeja::Contenido {
                version: bandeja::VERSION,
                reunion: archivo.clone(),
                encabezado,
                vence_de_la_reunion,
                cerro: ahora,
                propuestas: pendientes,
            };
            bandeja.dejar(llaves, &b, vence_bandeja)?.then_some(b)
        };
        let guardada = if vacio {
            None
        } else {
            match carpeta.guardar_como(llaves, &contenido, &archivo, vence_de_la_reunion) {
                Ok(g) => Some((g, resumen)),
                Err(e) => {
                    if a_la_bandeja.is_some() {
                        let _ = bandeja.borrar(&archivo);
                    }
                    return Err(e);
                }
            }
        };
        self.cerrar();
        let a = a_la_bandeja.as_ref().map_or(0, |b| b.propuestas.len());
        if let (Some(contenido), Some(vence)) = (a_la_bandeja, vence_bandeja) {
            if let Ok(mut m) = self.bandeja.lock() {
                *m = Some(BandejaAbierta { archivo, contenido, vence, guardadas: Vec::new() });
            }
        }
        Ok(Cierre { guardada, a_la_bandeja: a, murieron: sin_decidir - a })
    }

    /// Suelta la bandeja abierta en memoria si su archivo ya no está entre las `vivas` (auditoría del
    /// S3, B16): vencida y borrada, lo del cliente no puede seguir en RAM porque haya otra bandeja viva.
    fn soltar_si_no_vive(&self, vivas: &[Reunion]) {
        if let Ok(mut m) = self.bandeja.lock() {
            if m.as_ref().is_some_and(|a| !vivas.iter().any(|v| v.archivo == a.archivo)) {
                *m = None;
            }
        }
    }

    /// La bandeja abierta en memoria, si es `archivo` y sigue viva.
    fn con_la_abierta<T>(&self, archivo: &str, f: impl FnOnce(&mut BandejaAbierta) -> T) -> Option<T> {
        let mut m = self.bandeja.lock().ok()?;
        m.as_mut().filter(|a| a.archivo == archivo).map(f)
    }
}

fn preferencias<R: Runtime>(app: &AppHandle<R>) -> prefs::Preferencias {
    app.try_state::<crate::LasPreferencias>()
        .and_then(|lp| lp.actuales.lock().ok().map(|p| p.clone()))
        .unwrap_or_default()
}

/// La carpeta de tus notas: `~/Library/Application Support/<app>/notas/`, junto a la bandeja.
///
/// **No en Documentos** (ADR 016, decisión A del usuario, 2026-09-27): desde launchd, el `sh` que
/// borra lo vencido con la app cerrada no puede entrar en Documentos —el permiso es de la app, no
/// suyo—, así que ahí la retención solo se cumplía al abrir la app. Aquí se cumple aunque no la abras,
/// macOS no pide el permiso de Documentos y ninguna copia viaja a iCloud.
pub fn carpeta<R: Runtime>(app: &AppHandle<R>) -> Carpeta {
    Carpeta::en(carpeta_de_la_app(app).join(carpeta::CARPETA))
}

/// La bandeja: `~/Library/Application Support/<app>/bandeja/`, **no en Documentos** (ADR 016 §4).
pub fn la_bandeja_de<R: Runtime>(app: &AppHandle<R>) -> Bandeja {
    Bandeja::en(carpeta_de_la_app(app).join(bandeja::CARPETA))
}

/// `~/Library/Application Support/<app>/`, en 700: la de las notas, la bandeja y la puerta local.
pub fn carpeta_de_la_app<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    app.path().app_data_dir().unwrap_or_else(|_| std::env::temp_dir().join(vencimiento::APP))
}

fn avisar<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.emit_to(ventana::PRINCIPAL, EVENTO, ());
}

/// La banda enseña la última propuesta sin decidir, o nada (ADR 016 §3).
fn a_la_banda<R: Runtime>(app: &AppHandle<R>, el: &ElCuaderno) {
    let ultima: Option<Propuesta> = el.con(|c| c.en_espera().last().map(|e| e.propuesta.clone())).flatten();
    let _ = app.emit_to(ventana::BANDA, EVENTO_PROPUESTA, ultima);
}

pub fn vista<R: Runtime>(app: &AppHandle<R>) -> Option<VistaDelCuaderno> {
    let el = app.try_state::<ElCuaderno>()?;
    let retencion = preferencias(app).retencion;
    let (abierta, escuchando) = (el.abierta(), el.escuchando());
    let previsto = el.previsto(&carpeta(app), &la_bandeja_de(app));
    let (turnos_del_cliente, lecturas) =
        (el.turnos_del_cliente.load(Ordering::Relaxed), el.lecturas.load(Ordering::Relaxed));
    let ventana = preferencias(app).ventana_de_la_bandeja;
    let sin_proteger = el.sin_proteger.load(Ordering::Relaxed);
    el.con(|c| VistaDelCuaderno {
        nota: c.nota().to_string(),
        acuerdos: c.acuerdos().to_vec(),
        fijadas: c.fijadas().to_vec(),
        resumen: c.resumen(),
        conservar_mis_turnos: c.conserva_mis_turnos(),
        abierta,
        escuchando,
        previsto,
        turnos_del_cliente,
        lecturas,
        retencion,
        propuestas: c.en_espera().to_vec(),
        lleno: c.lleno(),
        ventana,
        sin_proteger,
    })
}

/// Una lectura de la pantalla en esta reunión: se cuenta para «Muere al cerrar».
pub fn contar_una_lectura<R: Runtime>(app: &AppHandle<R>) {
    if let Some(el) = app.try_state::<ElCuaderno>() {
        el.lecturas.fetch_add(1, Ordering::Relaxed);
    }
}

/// Empieza una sesión. Si la reunión anterior seguía abierta con algo tuyo dentro, **se guarda
/// primero**: nada tuyo se pierde por empezar otra.
pub fn al_empezar<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    if el.hay_que_guardar_la_anterior() {
        match guardar(app) {
            Ok(_) => println!("[notas] la reunión anterior seguía abierta: guardada antes de empezar otra"),
            Err(e) => println!("[notas] la reunión anterior seguía abierta y no se pudo guardar: {}", sin_ruta(&e)),
        }
    }
    el.abrir(preferencias(app).conservar_mis_turnos, fecha_de_ahora());
    let fallo = ventana::proteger_el_cuaderno(app, true).is_err();
    el.sin_proteger.store(fallo, Ordering::Relaxed);
    avisar(app);
}

/// Dejaste de escuchar. Con algo tuyo dentro, la reunión sigue abierta —y el cuaderno protegido—
/// hasta que la guardes o la descartes. **Sin nada tuyo, se cierra aquí**: no hay nada que guardar ni
/// que proteger, y dejar el cuaderno negro al compartir hasta la próxima sesión no protegería nada.
pub fn al_terminar<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    el.solo_notas.store(false, Ordering::Relaxed);
    if el.terminar() {
        let _ = ventana::proteger_el_cuaderno(app, false);
        el.sin_proteger.store(false, Ordering::Relaxed);
    }
    avisar(app);
}

/// **«Este cliente»** (ADR 017 §3): el que elegiste, o ninguno.
pub fn elegir_cliente<R: Runtime>(app: &AppHandle<R>, nombre: Option<String>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    if let Ok(mut c) = el.cliente.lock() {
        *c = nombre;
    }
    // Al log, el hecho: el nombre de tu cliente no va a un archivo que no pediste.
    println!("[cliente] {}", if el.cliente().is_some() { "elegido" } else { "sin elegir" });
    avisar(app);
}

pub fn cliente<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    app.try_state::<ElCuaderno>().and_then(|el| el.cliente())
}

/// La reunión que empieza va, o no, en modo solo notas. Lo apaga `al_terminar`.
pub fn marcar_solo_notas<R: Runtime>(app: &AppHandle<R>, si: bool) {
    if let Some(el) = app.try_state::<ElCuaderno>() {
        el.solo_notas.store(si, Ordering::Relaxed);
    }
}

pub fn solo_notas<R: Runtime>(app: &AppHandle<R>) -> bool {
    app.try_state::<ElCuaderno>().is_some_and(|el| el.abierta() && el.solo_notas.load(Ordering::Relaxed))
}

/// Con qué busca `⌃⌥A` en solo notas: **la última línea de tu nota** (ADR 017 §5). `None` sin nota.
pub fn ultima_linea_de_la_nota<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    let el = app.try_state::<ElCuaderno>()?;
    el.con(|c| notas::ultima_linea(c.nota()).map(str::to_string)).flatten()
}

/// ⌥⎋, pieza `TusTurnos`: tus turnos y la ficha vigente mueren; tus notas, tus acuerdos y tus
/// fijadas se quedan.
pub fn cortar<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    el.con(Cuaderno::cortar);
    al_terminar(app);
}

/// ⌥⎋, pieza `Propuestas`: las que esperaban tu decisión mueren, y la línea de la banda con ellas.
/// Las guardadas se quedan; las bandejas de reuniones ya cerradas siguen su ventana (ADR 016 §4).
pub fn cortar_las_propuestas<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    el.con(Cuaderno::cortar_las_propuestas);
    a_la_banda(app, &el);
    al_terminar(app);
}

fn cerrar<R: Runtime>(app: &AppHandle<R>, el: &ElCuaderno) {
    if el.cerrar() {
        let _ = ventana::proteger_el_cuaderno(app, false);
        el.sin_proteger.store(false, Ordering::Relaxed);
    }
    avisar(app);
}

/// «Guardar cifrado y cerrar». `Ok(None)` si no había nada tuyo: no hay archivo, y la reunión se cierra
/// igual. Las propuestas sin decidir van a la bandeja con tu ventana, o mueren si es «al cerrar». Si
/// falla, **la nota se queda** en memoria y la reunión, abierta: se dice por qué.
pub fn guardar<R: Runtime>(app: &AppHandle<R>) -> Result<Option<Guardada>, String> {
    let el = app.try_state::<ElCuaderno>().ok_or("el cuaderno no está listo")?;
    let estaba = el.abierta();
    let p = preferencias(app);
    let ahora = carpeta::ahora();
    let vence_bandeja = ventana_de_prueba(p.ventana_de_la_bandeja.vence(ahora, vencimiento::fin_del_dia(ahora)), ahora);
    let hecho = el.guardar_en(&carpeta(app), &la_bandeja_de(app), &DelLlavero, p.retencion, vence_bandeja, ahora)?;
    if estaba && !el.abierta() {
        let _ = ventana::proteger_el_cuaderno(app, false);
        el.sin_proteger.store(false, Ordering::Relaxed);
    }
    if hecho.a_la_bandeja > 0 || hecho.murieron > 0 {
        println!("[propuestas] al cerrar: {} a la bandeja · {} murieron", hecho.a_la_bandeja, hecho.murieron);
    }
    // Tus notas vencen con su retención y la bandeja con su ventana: las dos entran en la lista.
    // Si nada cambió, launchd no se toca.
    poner_al_dia_el_vencimiento(app);
    a_la_banda(app, &el);
    avisar(app);
    Ok(hecho.guardada.map(|(guardada, resumen)| {
        println!("{}", carpeta::linea_de_log(&resumen, &guardada));
        guardada
    }))
}

/// **Solo en la compilación de desarrollo:** con `AG_BANDEJA_DE_PRUEBA` en el entorno, toda ventana
/// que no sea «al cerrar» dura 2 minutos. Es como la parada del gate corto ve a launchd borrar la bandeja
/// con la app cerrada sin esperar 3 h (ADR 016 §5). En la de distribución no existe.
fn ventana_de_prueba(vence: Option<i64>, ahora: i64) -> Option<i64> {
    #[cfg(debug_assertions)]
    if std::env::var_os("AG_BANDEJA_DE_PRUEBA").is_some() {
        return vence.map(|_| ahora + 120);
    }
    let _ = ahora;
    vence
}

/// «Cerrar sin guardar»: lo tuyo de esta reunión se tira, con las letras pisadas.
pub fn descartar<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    let resumen = el.con(|c| c.resumen()).unwrap_or_default();
    cerrar(app, &el);
    a_la_banda(app, &el);
    println!(
        "[notas] cerrada sin guardar · {} acuerdos · {} fijadas · {} turnos tuyos · {} propuestas, tirados",
        resumen.acuerdos, resumen.fijadas, resumen.turnos, resumen.propuestas + resumen.sin_decidir
    );
}

/// Al salir de la app: si queda algo tuyo sin guardar, se guarda (ADR 015 §7).
pub fn al_salir<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    if !el.hay_que_decidir() {
        return;
    }
    match guardar(app) {
        Ok(_) => println!("[notas] al salir quedaban notas sin guardar: guardadas"),
        Err(e) => println!("[notas] al salir quedaban notas sin guardar y NO se pudieron guardar: {}", sin_ruta(&e)),
    }
}

/// La banda enseña una ficha: el cuaderno la recuerda por si pulsas ⌃⌥P.
pub fn ver<R: Runtime>(app: &AppHandle<R>, aparicion: &crate::ficha::Aparicion) {
    let crate::ficha::Respuesta::Ficha(f) = &aparicion.respuesta else { return };
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    el.con(|c| {
        c.ver(FichaFijada {
            titular: f.titular.clone(),
            documento: f.fuente.documento.clone(),
            seccion: f.fuente.seccion.clone(),
            unidad: f.fuente.unidad,
            linea: f.linea.clone(),
        })
    });
}

/// Un turno recién transcrito: el cuaderno decide si es tuyo (`notas::Cuaderno::oir`). Los del
/// cliente solo se CUENTAN, para «Muere al cerrar».
pub fn oir<R: Runtime>(app: &AppHandle<R>, turno: &crate::stt::Turno) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    if turno.pista == crate::capture::Pista::Sistema && !turno.texto.trim().is_empty() {
        el.turnos_del_cliente.fetch_add(1, Ordering::Relaxed);
    }
    el.con(|c| c.oir(turno));
}

/// Las propuestas de un turno (ADR 016): las reglas deciden, y el cuaderno las guarda esperando tu
/// sí. Sin reunión abierta no hay dónde proponer. Al log, solo cuántas.
pub fn proponer<R: Runtime>(app: &AppHandle<R>, turno: &crate::stt::Turno, conoce: &dyn Fn(&str) -> bool) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    if !el.abierta() {
        return;
    }
    let fijadas = el.con(|c| c.para_las_reglas()).unwrap_or_default();
    let nuevas = crate::propuestas::proponer(turno, &crate::propuestas::Contexto { fijadas: &fijadas, conoce });
    if nuevas.is_empty() {
        return;
    }
    let entraron = el.con(|c| c.proponer(nuevas)).unwrap_or(0);
    if entraron > 0 {
        println!("[propuestas] {entraron} nueva(s)");
        a_la_banda(app, &el);
        avisar(app);
    }
}

/// «Guardar» en Notas: la propuesta pasa a tu archivo.
pub fn guardar_propuesta<R: Runtime>(app: &AppHandle<R>, id: u32) -> bool {
    decidir_una(app, |c| c.guardar_propuesta(id))
}

/// «No» en Notas: muere en ese momento.
pub fn descartar_propuesta<R: Runtime>(app: &AppHandle<R>, id: u32) -> bool {
    decidir_una(app, |c| c.descartar_propuesta(id))
}

/// ⌃⌥↵: guarda la última propuesta, la que la banda enseña.
pub fn guardar_la_ultima<R: Runtime>(app: &AppHandle<R>) -> bool {
    let hecha = decidir_una(app, |c| c.guardar_la_ultima().is_some());
    println!("[propuestas] ⌃⌥↵ · {}", if hecha { "guardada" } else { "no había ninguna" });
    hecha
}

fn decidir_una<R: Runtime>(app: &AppHandle<R>, f: impl FnOnce(&mut Cuaderno) -> bool) -> bool {
    let Some(el) = app.try_state::<ElCuaderno>() else { return false };
    let hecha = el.con(f).unwrap_or(false);
    if hecha {
        a_la_banda(app, &el);
        avisar(app);
    }
    hecha
}

/// Cuánto esperarán en la bandeja las que no decidas. Se recuerda para las reuniones siguientes.
pub fn fijar_ventana<R: Runtime>(app: &AppHandle<R>, ventana: Ventana) {
    crate::recordar(app, |p| p.ventana_de_la_bandeja = ventana);
    avisar(app);
}

// ---- la bandeja (ADR 016 §4) --------------------------------------------------------------------

/// La bandeja que vence antes, para Notas. Si la sesión de la app ya se desbloqueó, se abre sola;
/// si no, llega cerrada con llave y abrirla pide Touch ID.
pub fn la_bandeja<R: Runtime>(app: &AppHandle<R>) -> Option<VistaDeLaBandeja> {
    let el = app.try_state::<ElCuaderno>()?;
    let b = la_bandeja_de(app);
    let ahora = carpeta::ahora();
    let vivas = vivas_en(&b, ahora);
    el.soltar_si_no_vive(&vivas);
    let primera = vivas.first().cloned()?;
    if el.con_la_abierta(&primera.archivo, |_| ()).is_none() && el.desbloqueo.desbloqueado() {
        abrir_del_disco(&el, &b, &primera.archivo);
    }
    let (propuestas, guardadas) = el
        .con_la_abierta(&primera.archivo, |a| (Some(a.contenido.propuestas.clone()), a.guardadas.clone()))
        .unwrap_or((None, Vec::new()));
    Some(VistaDeLaBandeja {
        archivo: primera.archivo,
        vence: primera.vence,
        bytes: primera.bytes,
        propuestas,
        guardadas,
        mas: vivas.len() - 1,
        ventana: preferencias(app).ventana_de_la_bandeja,
    })
}

/// El error, **sin rutas**, para el log (auditoría del S3, B30; ADR 015 §9): el nombre de una reunión
/// lleva el del cliente, y los errores del disco traen la ruta entera. Cada ruta absoluta, hasta el
/// «: » que la separa de su motivo, se cambia por `[ruta]`; el motivo del sistema se queda.
fn sin_ruta(e: &str) -> String {
    let mut fuera = String::with_capacity(e.len());
    let mut resto = e;
    while let Some(i) = resto.char_indices().find(|&(i, c)| c == '/' && (i == 0 || resto[..i].ends_with(' '))).map(|(i, _)| i) {
        fuera.push_str(&resto[..i]);
        fuera.push_str("[ruta]");
        resto = resto[i..].find(": ").map_or("", |j| &resto[i + j..]);
    }
    fuera.push_str(resto);
    fuera
}

/// Las bandejas que no han vencido a `ahora`, la que vence antes primero.
fn vivas_en(b: &Bandeja, ahora: i64) -> Vec<Reunion> {
    let mut vivas: Vec<Reunion> = b.lista().into_iter().filter(|r| r.vence > ahora).collect();
    vivas.sort_by_key(|r| r.vence);
    vivas
}

fn abrir_del_disco(el: &ElCuaderno, b: &Bandeja, archivo: &str) -> bool {
    match b.abrir(&DelLlavero, archivo) {
        Ok((contenido, vence)) => {
            if let Ok(mut m) = el.bandeja.lock() {
                *m = Some(BandejaAbierta { archivo: archivo.to_string(), contenido, vence, guardadas: Vec::new() });
            }
            true
        }
        Err(e) => {
            println!("[bandeja] no se pudo abrir: {}", sin_ruta(&e));
            false
        }
    }
}

/// «Abrir la bandeja»: pide el desbloqueo una vez por sesión de la app, como exportar.
pub fn abrir_la_bandeja<R: Runtime>(app: &AppHandle<R>, archivo: &str, idioma: &str) -> Result<(), String> {
    let el = app.try_state::<ElCuaderno>().ok_or("el cuaderno no está listo")?;
    el.desbloqueo.asegurar(|| desbloqueo::pedir(desbloqueo::razon(idioma)))?;
    if !abrir_del_disco(&el, &la_bandeja_de(app), archivo) {
        return Err("la bandeja no se pudo abrir".into());
    }
    avisar(app);
    Ok(())
}

/// «Guardar» o «No» sobre la propuesta `indice` de la bandeja abierta. Guardar la lleva a su
/// reunión con el vencimiento de la reunión; «No» la mata. La última se lleva el archivo.
pub fn decidir_en_la_bandeja<R: Runtime>(app: &AppHandle<R>, archivo: &str, indice: usize, guardar: bool) -> Result<(), String> {
    let el = app.try_state::<ElCuaderno>().ok_or("el cuaderno no está listo")?;
    decidir_sin_avisar(app, &el, archivo, indice, guardar)?;
    println!("[bandeja] una propuesta {}", if guardar { "guardada en su reunión" } else { "descartada" });
    poner_al_dia_el_vencimiento(app);
    avisar(app);
    Ok(())
}

/// «Guardar todas» o «Descartar todas».
pub fn decidir_toda_la_bandeja<R: Runtime>(app: &AppHandle<R>, archivo: &str, guardar: bool) -> Result<(), String> {
    let el = app.try_state::<ElCuaderno>().ok_or("el cuaderno no está listo")?;
    let mut cuantas = 0;
    let r = loop {
        match el.con_la_abierta(archivo, |a| a.contenido.propuestas.len()) {
            Some(0) => break Ok(()),
            Some(_) => {}
            None => break Err("abre la bandeja primero".to_string()),
        }
        if let Err(e) = decidir_sin_avisar(app, &el, archivo, 0, guardar) {
            break Err(e);
        }
        cuantas += 1;
    };
    println!("[bandeja] {cuantas} propuesta(s) {}", if guardar { "guardadas en su reunión" } else { "descartadas" });
    poner_al_dia_el_vencimiento(app);
    avisar(app);
    r
}

fn decidir_sin_avisar<R: Runtime>(app: &AppHandle<R>, el: &ElCuaderno, archivo: &str, indice: usize, guardar: bool) -> Result<(), String> {
    let b = la_bandeja_de(app);
    let notas = carpeta(app);
    el.con_la_abierta(archivo, |a| -> Result<(), String> {
        if indice >= a.contenido.propuestas.len() {
            return Err("esa propuesta ya no está en la bandeja".into());
        }
        if guardar {
            b.guardar(&DelLlavero, &notas, archivo, indice)?;
        } else {
            b.quitar(&DelLlavero, archivo, indice)?;
        }
        let mut p = a.contenido.propuestas.remove(indice);
        if guardar {
            a.guardadas.push(p);
        } else {
            notas::pisar_propuesta(&mut p);
        }
        Ok(())
    })
    .unwrap_or_else(|| Err("abre la bandeja primero".into()))
}

/// Cambiar la ventana desde la bandeja: la vuelve a sellar con el vencimiento nuevo, contado desde
/// el cierre y con su techo. «Al cerrar» se la lleva ahora. Se recuerda para las siguientes.
pub fn cambiar_la_ventana<R: Runtime>(app: &AppHandle<R>, archivo: &str, ventana: Ventana) -> Result<(), String> {
    let el = app.try_state::<ElCuaderno>().ok_or("el cuaderno no está listo")?;
    el.con_la_abierta(archivo, |_| ()).ok_or("abre la bandeja primero")?;
    let mut nuevo = None;
    la_bandeja_de(app).revencer(&DelLlavero, archivo, |cerro| {
        nuevo = ventana_de_prueba(ventana.vence(cerro, vencimiento::fin_del_dia(cerro)), carpeta::ahora());
        nuevo
    })?;
    match nuevo {
        Some(vence) => {
            el.con_la_abierta(archivo, |a| a.vence = vence);
        }
        None => {
            if let Ok(mut m) = el.bandeja.lock() {
                *m = None;
            }
        }
    }
    crate::recordar(app, |p| p.ventana_de_la_bandeja = ventana);
    println!("[bandeja] ventana cambiada desde la bandeja");
    poner_al_dia_el_vencimiento(app);
    avisar(app);
    Ok(())
}

/// Lo que Honestidad enseña de la bandeja, sin abrirla.
pub fn estado_de_la_bandeja<R: Runtime>(app: &AppHandle<R>) -> EstadoDeLaBandeja {
    let ahora = carpeta::ahora();
    EstadoDeLaBandeja {
        vence: la_bandeja_de(app).lista().into_iter().map(|r| r.vence).filter(|v| *v > ahora).min(),
        no_corrio: app.try_state::<ElCuaderno>().is_some_and(|el| el.no_corrio.load(Ordering::Relaxed)),
    }
}

// ---- el vencimiento (ADR 016 §5) ---------------------------------------------------------------

fn la_tarea<R: Runtime>(app: &AppHandle<R>) -> Option<vencimiento::Tarea> {
    let casa = app.path().home_dir().ok()?;
    Some(vencimiento::Tarea::de_la_app(&casa, &carpeta_de_la_app(app)))
}

/// Lo que launchd tiene que borrar con la app cerrada: **tus notas y la bandeja**. Las notas entraron
/// con la decisión A (ADR 016): desde que viven en la carpeta de la app, el `sh` de launchd puede
/// borrarlas. Las que guardaste con «siempre» no vencen y no entran.
fn lo_que_vence<R: Runtime>(app: &AppHandle<R>) -> Vec<vencimiento::Pendiente> {
    lo_que_vence_en(&carpeta(app), &la_bandeja_de(app))
}

pub fn lo_que_vence_en(notas: &Carpeta, bandeja: &Bandeja) -> Vec<vencimiento::Pendiente> {
    notas.pendientes().into_iter().chain(bandeja.pendientes()).collect()
}

/// Pone la tarea de launchd al día con lo que vence. Solo toca launchd si la lista cambió.
pub fn poner_al_dia_el_vencimiento<R: Runtime>(app: &AppHandle<R>) {
    let Some(tarea) = la_tarea(app) else { return };
    match vencimiento::al_dia(&tarea, &lo_que_vence(app), &vencimiento::hora_del_mac) {
        Ok(vencimiento::Hecho::SinCambios) => {}
        Ok(vencimiento::Hecho::Registrada) => println!("[vencimiento] la tarea de borrado, al día"),
        Ok(vencimiento::Hecho::Quitada) => println!("[vencimiento] nada que vencer: la tarea de borrado, quitada"),
        Err(e) => {
            println!("[vencimiento] {}", sin_ruta(&e));
            // Sin la tarea registrada, lo que vence no se borra con la app cerrada: Honestidad lo dice
            // en rojo, como cuando la tarea no corrió (auditoría del S3, B27).
            if let Some(el) = app.try_state::<ElCuaderno>() {
                el.no_corrio.store(true, Ordering::Relaxed);
                avisar(app);
            }
        }
    }
}

/// ⌃⌥P: fija la ficha que la banda enseña.
pub fn fijar<R: Runtime>(app: &AppHandle<R>) -> Option<FichaFijada> {
    let el = app.try_state::<ElCuaderno>()?;
    let fijada = el.con(|c| c.fijar_la_vigente().cloned()).flatten();
    println!("[notas] ⌃⌥P · {}", if fijada.is_some() { "ficha fijada" } else { "no había ficha que fijar" });
    // La señal «fijada» de la banda (mirada 20, fila 9): símbolo, texto y color, regla 8.
    let _ = app.emit_to(ventana::BANDA, EVENTO_FIJADA, fijada.is_some());
    avisar(app);
    fijada
}

pub fn escribir<R: Runtime>(app: &AppHandle<R>, texto: &str) {
    if let Some(el) = app.try_state::<ElCuaderno>() {
        el.con(|c| c.escribir(texto));
    }
}

pub fn acordar<R: Runtime>(app: &AppHandle<R>, texto: &str) -> bool {
    app.try_state::<ElCuaderno>().and_then(|el| el.con(|c| c.acordar(texto))).unwrap_or(false)
}

pub fn quitar_acuerdo<R: Runtime>(app: &AppHandle<R>, indice: usize) -> bool {
    app.try_state::<ElCuaderno>().and_then(|el| el.con(|c| c.quitar_acuerdo(indice))).unwrap_or(false)
}

pub fn soltar_fijada<R: Runtime>(app: &AppHandle<R>, indice: usize) -> bool {
    app.try_state::<ElCuaderno>().and_then(|el| el.con(|c| c.soltar_fijada(indice))).unwrap_or(false)
}

pub fn conservar_mis_turnos<R: Runtime>(app: &AppHandle<R>, si: bool) {
    if let Some(el) = app.try_state::<ElCuaderno>() {
        el.con(|c| c.conservar_mis_turnos(si));
    }
    crate::recordar(app, |p| p.conservar_mis_turnos = si);
}

pub fn lista<R: Runtime>(app: &AppHandle<R>) -> ListaDeReuniones {
    ListaDeReuniones { reuniones: carpeta(app).lista() }
}

/// ⌃⌥N y «Anotar para después»: el cuaderno al frente, en Notas, con el cursor al final de tu nota.
pub fn ir_a_notas<R: Runtime>(app: &AppHandle<R>) {
    let Some(v) = app.get_webview_window(ventana::PRINCIPAL) else { return };
    let navego = v.url().map_err(|e| e.to_string()).and_then(|mut url| {
        url.set_query(Some("pantalla=notas&foco=nota"));
        v.navigate(url).map_err(|e| e.to_string())
    });
    let _ = v.unminimize();
    let _ = v.show();
    let _ = v.set_focus();
    match navego {
        Ok(()) => println!("[notas] ⌃⌥N · el cuaderno al frente, en tu nota"),
        Err(e) => println!("[notas] ⌃⌥N · no se pudo abrir Notas en el cuaderno: {e}"),
    }
}

/// Abre una reunión guardada con **el desbloqueo que le toca a quien la pide**: hoy solo la puerta
/// local, que lleva el suyo y lo pide una vez por apertura (auditoría del S3, M4). En la pantalla no hay
/// «abrir»: se exporta.
pub fn abrir_con<R: Runtime>(
    app: &AppHandle<R>,
    desbloqueo: &desbloqueo::Desbloqueo,
    archivo: &str,
    idioma: &str,
) -> Result<Contenido, String> {
    desbloqueo.asegurar(|| desbloqueo::pedir(desbloqueo::razon(idioma)))?;
    let t = Instant::now();
    let c = carpeta(app).abrir(&DelLlavero, archivo)?;
    println!("[notas] reunión abierta en {} ms", t.elapsed().as_millis());
    Ok(c)
}

/// Exporta a texto donde elijas. Pide el desbloqueo de la sesión de la app. `Ok(false)` si cancelaste el
/// diálogo.
pub fn exportar<R: Runtime>(app: &AppHandle<R>, archivo: &str, idioma: &str) -> Result<bool, String> {
    use tauri_plugin_dialog::DialogExt;
    let el = app.try_state::<ElCuaderno>().ok_or("el cuaderno no está listo")?;
    el.desbloqueo.asegurar(|| desbloqueo::pedir(desbloqueo::razon(idioma)))?;
    let sugerido = archivo.trim_end_matches(&format!(".{}", carpeta::EXTENSION)).to_string() + ".md";
    let Some(destino) = app.dialog().file().set_file_name(&sugerido).blocking_save_file() else {
        return Ok(false);
    };
    let destino = destino.into_path().map_err(|e| format!("ese destino no es una ruta: {e}"))?;
    carpeta(app).exportar(&DelLlavero, archivo, &destino, idioma)?;
    println!("[notas] exportada a texto, sin cifrado");
    Ok(true)
}

pub fn borrar<R: Runtime>(app: &AppHandle<R>, archivo: &str) -> Result<(), String> {
    carpeta(app).borrar(archivo)?;
    println!("[notas] una reunión guardada, borrada a mano");
    poner_al_dia_el_vencimiento(app);
    Ok(())
}

pub fn fijar_retencion<R: Runtime>(app: &AppHandle<R>, retencion: prefs::Retencion) {
    crate::recordar(app, |p| p.retencion = retencion);
    avisar(app);
}

/// «Mostrar en Finder»: la carpeta de tus notas, o una reunión dentro de ella, seleccionada en Finder.
/// Lo hace macOS (`NSWorkspace`), sin lanzar ningún programa. Si aún no guardaste nada, la carpeta
/// nace aquí, 700, para que haya qué enseñar.
pub fn mostrar_en_finder<R: Runtime>(app: &AppHandle<R>, archivo: Option<&str>) -> Result<(), String> {
    let notas = carpeta(app);
    let ruta = match archivo {
        Some(a) => notas.ruta_de(a)?,
        None => {
            crate::almacen::carpeta_privada(notas.raiz())?;
            notas.raiz().to_path_buf()
        }
    };
    tauri_plugin_opener::reveal_item_in_dir(&ruta).map_err(|e| format!("Finder no pudo enseñarla: {e}"))
}

/// Barre lo vencido ahora y luego **al llegar el próximo vencimiento** (como mucho, cada hora),
/// mientras la app corre, y deja la tarea de launchd al día.
///
/// Antes de barrer, **mide** si la tarea corrió con la app cerrada (ADR 016 §5): con la tarea
/// instalada y algo vencido hace más de 2 minutos, no corrió —desactivada en Ítems de inicio, o
/// launchd no la cargó— y Honestidad lo dice. Se borra igual aquí.
pub fn arrancar_el_barrido<R: Runtime>(app: &AppHandle<R>) {
    if let Some(el) = app.try_state::<ElCuaderno>() {
        let instalada = la_tarea(app).is_some_and(|t| t.plist.exists());
        let no = vencimiento::no_corrio(&lo_que_vence(app), carpeta::ahora(), instalada);
        el.no_corrio.store(no, Ordering::Relaxed);
        if no {
            println!("[vencimiento] la tarea de borrado no corrió con la app cerrada: ¿desactivada en Ítems de inicio?");
        }
    }
    let mango = app.clone();
    std::thread::spawn(move || loop {
        let ahora = carpeta::ahora();
        let notas = carpeta(&mango).barrer(ahora);
        if notas > 0 {
            println!("[notas] {notas} reunión(es) vencida(s), borrada(s)");
        }
        let b = la_bandeja_de(&mango);
        let bandejas = b.barrer(ahora);
        if bandejas > 0 {
            println!("[bandeja] {bandejas} bandeja(s) vencida(s), borrada(s)");
            if let Some(el) = mango.try_state::<ElCuaderno>() {
                el.soltar_si_no_vive(&vivas_en(&b, ahora));
            }
            avisar(&mango);
        }
        poner_al_dia_el_vencimiento(&mango);
        std::thread::sleep(hasta_el_proximo(&mango, ahora));
    });
}

/// Cuánto dormir: hasta el próximo vencimiento de notas o bandeja, como mucho [`BARRIDO_CADA`].
fn hasta_el_proximo<R: Runtime>(app: &AppHandle<R>, ahora: i64) -> std::time::Duration {
    let proximo = lo_que_vence(app)
        .into_iter()
        .map(|p| p.vence)
        .filter(|v| *v > ahora)
        .min();
    match proximo {
        Some(v) => std::time::Duration::from_secs((v - ahora + 1) as u64).min(BARRIDO_CADA),
        None => BARRIDO_CADA,
    }
}

/// La fecha y la hora del reloj del Mac, como `mes_de_hoy` en `lib.rs`: sin una biblioteca de fechas.
fn fecha_de_ahora() -> notas::Fecha {
    // SEGURIDAD: `time` y `localtime_r` escriben en estructuras que viven en esta función.
    unsafe {
        let ahora = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&ahora, &mut tm);
        notas::Fecha {
            anio: tm.tm_year + 1900,
            mes: (tm.tm_mon + 1) as u32,
            dia: tm.tm_mday as u32,
            hora: tm.tm_hour as u32,
            minuto: tm.tm_min as u32,
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::carpeta::doble::EnMemoria;

    fn carpeta(nombre: &str) -> Carpeta {
        let d = std::env::temp_dir().join(format!("ag-reunion-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        Carpeta::en(d)
    }

    const HOY: notas::Fecha = notas::Fecha { anio: 2026, mes: 9, dia: 27, hora: 14, minuto: 2 };
    const CIERRE: i64 = 1_790_517_600;

    fn bandeja(nombre: &str) -> Bandeja {
        let d = std::env::temp_dir().join(format!("ag-reunion-bandeja-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        Bandeja::en(d)
    }

    fn propuesta(texto: &str) -> Propuesta {
        Propuesta {
            regla: crate::propuestas::Regla::Cifra,
            de: crate::propuestas::De::Cliente,
            texto: texto.into(),
            ficha: None,
            seccion: None,
            hora: "14:16".into(),
        }
    }

    #[test]
    fn si_guardar_falla_la_nota_se_queda_y_la_reunion_sigue_abierta() {
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| c.escribir("Piden la cuarta fuente."));
        let sin_llavero = EnMemoria { no_contesta: true, ..EnMemoria::default() };
        assert!(el.guardar_en(&carpeta("falla"), &bandeja("falla"), &sin_llavero, prefs::Retencion::Dias90, None, 0).is_err());
        assert_eq!(el.con(|c| c.nota().to_string()).unwrap(), "Piden la cuarta fuente.", "la nota se perdió al fallar");
        assert!(el.abierta(), "la reunión se cerró aunque no se guardó");
    }

    #[test]
    fn guardar_cierra_la_reunion_y_deja_el_archivo_con_su_nombre() {
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| c.acordar("Cuarta fuente: cotización aparte"));
        let c = carpeta("guarda");
        let (g, r) = el.guardar_en(&c, &bandeja("guarda"), &EnMemoria::default(), prefs::Retencion::Dias7, None, 1_000).unwrap().guardada.unwrap();
        assert_eq!(g.archivo, "reunion-2026-09-27-1402.ghost");
        assert_eq!(g.vence, 1_000 + 7 * 86_400);
        assert_eq!(r.acuerdos, 1);
        assert!(!el.abierta() && el.vacio(), "guardada, la reunión tiene que cerrarse y el cuaderno vaciarse");
        let _ = std::fs::remove_dir_all(c.raiz());
    }

    #[test]
    fn sin_nada_tuyo_parar_cierra_la_reunion_y_no_hay_archivo() {
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        assert!(el.terminar(), "una reunión vacía se queda abierta —y el cuaderno protegido— sin motivo");
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| c.escribir("algo"));
        assert!(!el.terminar(), "con algo tuyo, parar no cierra: queda «al cerrar»");
        assert!(el.abierta());
        // y con propuestas esperando, tampoco (ADR 016 §4): queda la decisión de qué hacer con ellas
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| c.proponer(vec![propuesta("12 semanas")]));
        assert!(!el.terminar(), "con propuestas sin decidir, parar cerró la reunión y se las llevó");
        let c = carpeta("vacia");
        let vacio = ElCuaderno::default();
        let b = bandeja("vacia");
        let cierre = vacio.guardar_en(&c, &b, &EnMemoria::default(), prefs::Retencion::Dias90, Some(CIERRE + 3_600), 0).unwrap();
        assert!(cierre.guardada.is_none() && cierre.a_la_bandeja == 0);
        assert!(!c.raiz().exists() && !b.raiz().exists(), "sin nada tuyo se creó la carpeta o un archivo");
    }

    /// **Al cerrar, lo que no decidiste va a la bandeja y lo tuyo a tu archivo**, con el mismo nombre
    /// en las dos carpetas para que «Guardar» desde la bandeja encuentre su reunión (ADR 016 §4).
    /// Demostrado en rojo: sin `bandeja.dejar` en `guardar_en`, las dos propuestas se pierden al
    /// cerrar y la bandeja no existe.
    #[test]
    fn al_cerrar_lo_que_no_decidiste_va_a_la_bandeja_y_lo_tuyo_a_tu_archivo() {
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| {
            c.escribir("Piden la cuarta fuente.");
            c.proponer(vec![propuesta("12 semanas"), propuesta("cuatro fuentes"), propuesta("el viernes")]);
            let id = c.en_espera()[2].id;
            c.guardar_propuesta(id);
        });
        let (c, b, llaves) = (carpeta("a-la-bandeja"), bandeja("a-la-bandeja"), EnMemoria::default());
        let cierre = el.guardar_en(&c, &b, &llaves, prefs::Retencion::Dias90, Some(CIERRE + 3 * 3_600), CIERRE).unwrap();
        assert_eq!((cierre.a_la_bandeja, cierre.murieron), (2, 0));
        let g = cierre.guardada.unwrap().0;
        let en_la_bandeja = b.lista();
        assert_eq!(en_la_bandeja.len(), 1, "las que no decidiste no fueron a la bandeja");
        assert_eq!(en_la_bandeja[0].archivo, g.archivo, "la bandeja no apunta a su reunión");
        assert_eq!(en_la_bandeja[0].vence, CIERRE + 3 * 3_600);
        let (dentro, _) = b.abrir(&llaves, &g.archivo).unwrap();
        assert_eq!(dentro.propuestas.iter().map(|p| p.texto.as_str()).collect::<Vec<_>>(), ["12 semanas", "cuatro fuentes"]);
        assert_eq!(dentro.vence_de_la_reunion, CIERRE + 90 * 86_400);
        let reunion = c.abrir(&llaves, &g.archivo).unwrap();
        assert_eq!(reunion.propuestas.len(), 1, "a tu archivo solo va la que guardaste");
        assert!(!el.abierta() && !el.hay_que_decidir());
        assert!(el.con_la_abierta(&g.archivo, |a| a.contenido.propuestas.len()) == Some(2), "la bandeja recién escrita se lee sin pedir nada");
        let _ = std::fs::remove_dir_all(c.raiz());
        let _ = std::fs::remove_dir_all(b.raiz());
    }

    /// **Decisión A (ADR 016):** lo que launchd borra con la app cerrada trae tus notas **y** la
    /// bandeja; una reunión guardada para «siempre» no entra. ¿Puede fallar? Sí, y se vio en rojo: con
    /// la lista de antes —solo la bandeja— tus notas vencidas esperaban a que abrieras la app.
    #[test]
    fn launchd_se_lleva_tus_notas_y_la_bandeja() {
        let (c, b, llaves) = (carpeta("launchd"), bandeja("launchd"), EnMemoria::default());
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|cu| {
            cu.escribir("Piden la cuarta fuente.");
            cu.proponer(vec![propuesta("12 semanas")]);
        });
        let g = el.guardar_en(&c, &b, &llaves, prefs::Retencion::Dias90, Some(CIERRE + 3 * 3_600), CIERRE).unwrap().guardada.unwrap().0;
        let para_siempre = ElCuaderno::default();
        para_siempre.abrir(false, notas::Fecha { minuto: 30, ..HOY });
        para_siempre.con(|cu| cu.escribir("Esta la guardo para siempre."));
        assert!(para_siempre.guardar_en(&c, &b, &llaves, prefs::Retencion::Siempre, None, CIERRE).unwrap().guardada.is_some());
        let lista: Vec<(PathBuf, i64)> = lo_que_vence_en(&c, &b).into_iter().map(|p| (p.ruta, p.vence)).collect();
        assert_eq!(lista.len(), 2, "la reunión de «siempre» entró, o faltó algo: {lista:?}");
        assert!(lista.contains(&(c.raiz().join(&g.archivo), CIERRE + 90 * 86_400)), "tus notas no están en la lista de launchd: {lista:?}");
        assert!(lista.contains(&(b.raiz().join(&g.archivo), CIERRE + 3 * 3_600)), "la bandeja no está en la lista de launchd: {lista:?}");
        let _ = std::fs::remove_dir_all(c.raiz());
        let _ = std::fs::remove_dir_all(b.raiz());
    }

    /// **«Este cliente» nombra el archivo** (ADR 017 §3, que el ADR 015 §2 esperaba) y entra en su
    /// encabezado. ¿Puede fallar? Sí: con el cliente de antes —ninguno—, el archivo se llama
    /// `reunion-2026-09-27-1402.ghost` y es rojo (bitácora).
    #[test]
    fn el_cliente_elegido_nombra_el_archivo() {
        let (c, b, llaves) = (carpeta("cliente"), bandeja("cliente"), EnMemoria::default());
        let el = ElCuaderno::default();
        *el.cliente.lock().unwrap() = Some("Páramo Azul".into());
        el.abrir(false, HOY);
        el.con(|cu| cu.escribir("Piden la cuarta fuente."));
        assert_eq!(el.previsto(&c, &b).unwrap().archivo, "paramo-azul-2026-09-27.ghost");
        let g = el.guardar_en(&c, &b, &llaves, prefs::Retencion::Dias90, None, CIERRE).unwrap().guardada.unwrap().0;
        assert_eq!(g.archivo, "paramo-azul-2026-09-27.ghost");
        assert_eq!(c.abrir(&llaves, &g.archivo).unwrap().encabezado.cliente.as_deref(), Some("Páramo Azul"));
        let _ = std::fs::remove_dir_all(c.raiz());
    }

    /// **Dos reuniones del mismo cliente el mismo día no se pisan** (auditoría del S3, A1). El nombre
    /// tiene que estar libre en las notas **y** en la bandeja: una llamada sin nota deja su bandeja
    /// sin archivo de notas, y la siguiente, mirando solo `notas/`, tomaba su nombre, pisaba su bandeja
    /// (de 3 propuestas quedaba 1) y recibía lo que se guardara desde ella. Demostrado en rojo con el
    /// código de antes: una sola bandeja y 1 propuesta viva.
    #[test]
    fn dos_reuniones_del_mismo_cliente_el_mismo_dia_no_se_pisan() {
        let (c, b, llaves) = (carpeta("mismo-dia"), bandeja("mismo-dia"), EnMemoria::default());
        let primera = ElCuaderno::default();
        *primera.cliente.lock().unwrap() = Some("Páramo Azul".into());
        primera.abrir(false, HOY);
        primera.con(|cu| cu.proponer(vec![propuesta("12 semanas"), propuesta("cuatro fuentes")]));
        let r1 = primera.guardar_en(&c, &b, &llaves, prefs::Retencion::Dias90, Some(CIERRE + 3_600), CIERRE).unwrap();
        assert!(r1.guardada.is_none() && r1.a_la_bandeja == 2);

        let segunda = ElCuaderno::default();
        *segunda.cliente.lock().unwrap() = Some("Páramo Azul".into());
        segunda.abrir(false, notas::Fecha { hora: 16, ..HOY });
        segunda.con(|cu| {
            cu.escribir("Piden la cuarta fuente.");
            cu.proponer(vec![propuesta("el viernes")]);
        });
        assert_ne!(segunda.previsto(&c, &b).unwrap().archivo, "paramo-azul-2026-09-27.ghost", "lo previsto pisa la bandeja de la primera");
        let r2 = segunda.guardar_en(&c, &b, &llaves, prefs::Retencion::Dias90, Some(CIERRE + 3_600), CIERRE).unwrap();
        let de_la_segunda = r2.guardada.unwrap().0.archivo;

        let bandejas = b.lista();
        assert_eq!(bandejas.len(), 2, "la segunda pisó la bandeja de la primera: {bandejas:?}");
        let vivas: usize = bandejas.iter().map(|x| b.abrir(&llaves, &x.archivo).unwrap().0.propuestas.len()).sum();
        assert_eq!(vivas, 3, "se dejaron 3 propuestas en la bandeja y quedan {vivas}");
        let de_la_primera = bandejas.iter().map(|x| b.abrir(&llaves, &x.archivo).unwrap().0).find(|x| x.propuestas.len() == 2).unwrap();
        assert_ne!(de_la_primera.reunion, de_la_segunda, "la bandeja de la primera apunta a las notas de la segunda");

        b.guardar(&llaves, &c, &de_la_primera.reunion, 0).unwrap();
        assert_eq!(c.lista().len(), 2, "«Guardar» desde la bandeja de la primera no creó su propio archivo");
        assert!(c.abrir(&llaves, &de_la_segunda).unwrap().propuestas.is_empty(), "la propuesta de la primera acabó en la segunda");
        let _ = std::fs::remove_dir_all(c.raiz());
        let _ = std::fs::remove_dir_all(b.raiz());
    }

    /// **La bandeja abierta muere con su archivo aunque haya otra viva** (auditoría del S3, B16). Antes
    /// solo se soltaba cuando no quedaba ninguna: con otra bandeja viva, lo del cliente seguía en RAM
    /// pasada su hora. Demostrado en rojo con `soltar_si_no_vive` vacía.
    #[test]
    fn la_bandeja_abierta_muere_con_su_archivo_aunque_haya_otra() {
        let (c, b, llaves) = (carpeta("suelta"), bandeja("suelta"), EnMemoria::default());
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|cu| cu.proponer(vec![propuesta("12 semanas")]));
        el.guardar_en(&c, &b, &llaves, prefs::Retencion::Dias90, Some(CIERRE + 3_600), CIERRE).unwrap();
        let abierta = b.lista()[0].archivo.clone();
        assert!(el.con_la_abierta(&abierta, |_| ()).is_some());
        let otra = ElCuaderno::default();
        otra.abrir(false, notas::Fecha { hora: 16, ..HOY });
        otra.con(|cu| cu.proponer(vec![propuesta("el viernes")]));
        otra.guardar_en(&c, &b, &llaves, prefs::Retencion::Dias90, Some(CIERRE + 7_200), CIERRE).unwrap();
        std::fs::remove_file(b.raiz().join(&abierta)).unwrap();
        let vivas = vivas_en(&b, CIERRE);
        assert_eq!(vivas.len(), 1, "la otra bandeja tenía que seguir viva");
        el.soltar_si_no_vive(&vivas);
        assert!(el.con_la_abierta(&abierta, |_| ()).is_none(), "la bandeja vencida sigue en memoria");
        let _ = std::fs::remove_dir_all(c.raiz());
        let _ = std::fs::remove_dir_all(b.raiz());
    }

    /// **Los logs de error no llevan el nombre del cliente** (auditoría del S3, B30; ADR 015 §9). El
    /// nombre del archivo lleva el del cliente, y los errores del disco traen la ruta entera. Demostrado
    /// en rojo con el código de antes: «no se pudo leer paramo-azul-2026-09-27.ghost…» y la ruta del
    /// temporal en el error de escribir.
    #[test]
    fn los_errores_que_se_loguean_no_nombran_al_cliente() {
        let c = carpeta("sin-nombre");
        let e = c.abrir_en_claro(&EnMemoria::default(), "paramo-azul-2026-09-27.ghost").unwrap_err();
        assert!(!e.contains("paramo"), "el error de abrir nombra al cliente: {e}");
        let tapon = std::env::temp_dir().join(format!("ag-reunion-tapon-log-{}", std::process::id()));
        std::fs::write(&tapon, b"").unwrap();
        let e = crate::almacen::escribir(&tapon.join("paramo-azul-2026-09-27.ghost"), b"x").unwrap_err();
        let log = sin_ruta(&e);
        assert!(!log.contains("paramo") && !log.contains('/'), "el log lleva la ruta: {log}");
        assert!(log.starts_with("no se pudo crear"), "se perdió el motivo: {log}");
        assert_eq!(sin_ruta("sin ruta: nada que cortar"), "sin ruta: nada que cortar");
        let _ = std::fs::remove_file(&tapon);
    }

    /// Con la ventana «al cerrar», las que no decidiste mueren y **no se escribe nada** en la bandeja.
    #[test]
    fn con_la_ventana_en_cero_mueren_al_cerrar() {
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| c.proponer(vec![propuesta("12 semanas"), propuesta("cuatro fuentes")]));
        let (c, b) = (carpeta("cero"), bandeja("cero"));
        let cierre = el.guardar_en(&c, &b, &EnMemoria::default(), prefs::Retencion::Dias90, None, CIERRE).unwrap();
        assert_eq!((cierre.a_la_bandeja, cierre.murieron), (0, 2));
        assert!(cierre.guardada.is_none());
        assert!(!b.raiz().exists() && !c.raiz().exists(), "con «al cerrar» se escribió algo");
        assert!(!el.abierta() && !el.hay_que_decidir());
    }

    /// **Si tus notas no se pueden guardar, la bandeja recién escrita se deshace** y la reunión sigue
    /// abierta entera, propuestas incluidas: nada queda a medias. Demostrado en rojo: sin el
    /// `bandeja.borrar` del error, quedaba una bandeja apuntando a una reunión que no existe.
    #[test]
    fn si_las_notas_fallan_la_bandeja_se_deshace() {
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| {
            c.escribir("Piden la cuarta fuente.");
            c.proponer(vec![propuesta("12 semanas")]);
        });
        let b = bandeja("deshace");
        // Una carpeta de notas que no se puede crear: cuelga de un archivo.
        let tapon = std::env::temp_dir().join(format!("ag-reunion-tapon-{}", std::process::id()));
        std::fs::write(&tapon, b"").unwrap();
        let notas = Carpeta::en(tapon.join("Angel Ghost"));
        assert!(el.guardar_en(&notas, &b, &EnMemoria::default(), prefs::Retencion::Dias90, Some(CIERRE + 3_600), CIERRE).is_err());
        assert!(b.lista().is_empty(), "quedó una bandeja apuntando a una reunión que no se guardó");
        assert!(el.abierta(), "la reunión se cerró aunque no se guardó");
        assert_eq!(el.con(|c| c.en_espera().len()).unwrap(), 1, "las propuestas se perdieron al fallar");
        let _ = std::fs::remove_file(&tapon);
        let _ = std::fs::remove_dir_all(b.raiz());
    }

    #[test]
    fn empezar_otra_con_la_anterior_abierta_pide_guardarla() {
        let el = ElCuaderno::default();
        assert!(!el.hay_que_guardar_la_anterior());
        el.abrir(false, HOY);
        assert!(!el.hay_que_guardar_la_anterior(), "vacía no hay nada que guardar");
        el.con(|c| c.escribir("de la reunión anterior"));
        assert!(el.hay_que_guardar_la_anterior(), "se perdería lo de la reunión anterior");
        // Solo con propuestas sin decidir, también: irían a la bandeja (ADR 016 §4). Demostrado en
        // rojo con la condición de antes (`!self.vacio()`), que las tiraba al empezar otra.
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| c.proponer(vec![propuesta("12 semanas")]));
        assert!(el.hay_que_guardar_la_anterior(), "empezar otra reunión tiraría las propuestas sin decidir");
    }
}
