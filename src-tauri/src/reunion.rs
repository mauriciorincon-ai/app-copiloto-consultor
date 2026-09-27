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

use crate::carpeta::{self, Carpeta, DelLlavero, Guardada, Reunion};
use crate::desbloqueo::{self, Desbloqueo};
use crate::notas::{self, Contenido, Cuaderno, Encabezado, FichaFijada, Resumen};
use crate::{prefs, ventana};

/// El evento que avisa a la ventana principal de que el cuaderno cambió desde fuera (⌃⌥P, el
/// principio o el fin de una reunión, el corte). Es una señal: la pantalla vuelve a preguntar.
pub const EVENTO: &str = "cuaderno";

/// Cada cuánto se barre lo vencido mientras la app corre. Con la app cerrada lo cumple launchd
/// (ADR 016, fase 2).
pub const BARRIDO_CADA: std::time::Duration = std::time::Duration::from_secs(3_600);

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

/// Las reuniones guardadas y dónde viven.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListaDeReuniones {
    /// `None` es la de fábrica, `~/Documents/Angel Ghost/` —la interfaz la nombra en su idioma, como
    /// Finder—; `Some` es la que elegiste, con `~` en vez de tu carpeta de usuario.
    pub carpeta: Option<String>,
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

    /// Abre una reunión. Devuelve si la anterior seguía abierta **con algo tuyo** —entonces hay que
    /// guardarla antes, y lo hace quien llama—.
    fn hay_que_guardar_la_anterior(&self) -> bool {
        self.abierta() && !self.vacio()
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
    fn previsto(&self, carpeta: &Carpeta) -> Option<Previsto> {
        if self.vacio() {
            return None;
        }
        let (fecha, minutos) = self.cuando();
        let cliente: Option<String> = None;
        let base = notas::nombre_del_archivo(cliente.as_deref(), &fecha);
        Some(Previsto {
            fecha: format!("{}-{:02}-{:02}", fecha.anio, fecha.mes, fecha.dia),
            minutos,
            archivo: carpeta.nombre_para(&base).unwrap_or_else(|| format!("{base}.{}", carpeta::EXTENSION)),
            cliente,
        })
    }

    /// Guarda en `carpeta`. `Ok(None)` si no había nada tuyo. **Si falla, el cuaderno se queda
    /// entero y la reunión abierta**: la nota no se pierde por un error de disco o de Llavero.
    fn guardar_en(
        &self,
        carpeta: &Carpeta,
        llaves: &dyn carpeta::Llaves,
        retencion: prefs::Retencion,
        ahora: i64,
    ) -> Result<Option<(Guardada, Resumen)>, String> {
        if self.vacio() {
            self.cerrar();
            return Ok(None);
        }
        let (fecha, minutos) = self.cuando();
        // El cliente llega en la fase 3 («Este cliente»). Hasta entonces, ninguno: la app no lo adivina.
        let cliente: Option<String> = None;
        let encabezado = Encabezado { empezo: fecha.como_texto(), minutos, cliente: cliente.clone() };
        let (contenido, resumen): (Contenido, Resumen) =
            self.con(|c| (c.contenido(encabezado), c.resumen())).ok_or("el cuaderno quedó en mal estado")?;
        let base = notas::nombre_del_archivo(cliente.as_deref(), &fecha);
        let guardada = carpeta.guardar(llaves, &contenido, &base, retencion.vence(ahora))?;
        self.cerrar();
        Ok(Some((guardada, resumen)))
    }
}

fn preferencias<R: Runtime>(app: &AppHandle<R>) -> prefs::Preferencias {
    app.try_state::<crate::LasPreferencias>()
        .and_then(|lp| lp.actuales.lock().ok().map(|p| p.clone()))
        .unwrap_or_default()
}

/// La carpeta de tus notas: la que elegiste, o `~/Documents/Angel Ghost/`.
pub fn carpeta<R: Runtime>(app: &AppHandle<R>) -> Carpeta {
    let raiz = match preferencias(app).carpeta_de_notas {
        Some(elegida) => PathBuf::from(elegida),
        None => app
            .path()
            .document_dir()
            .unwrap_or_else(|_| std::env::var("HOME").map(PathBuf::from).unwrap_or_default().join("Documents"))
            .join(carpeta::CARPETA),
    };
    Carpeta::en(raiz)
}

fn avisar<R: Runtime>(app: &AppHandle<R>) {
    let _ = app.emit_to(ventana::PRINCIPAL, EVENTO, ());
}

pub fn vista<R: Runtime>(app: &AppHandle<R>) -> Option<VistaDelCuaderno> {
    let el = app.try_state::<ElCuaderno>()?;
    let retencion = preferencias(app).retencion;
    let (abierta, escuchando) = (el.abierta(), el.escuchando());
    let previsto = el.previsto(&carpeta(app));
    let (turnos_del_cliente, lecturas) =
        (el.turnos_del_cliente.load(Ordering::Relaxed), el.lecturas.load(Ordering::Relaxed));
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
            Err(e) => println!("[notas] la reunión anterior seguía abierta y no se pudo guardar: {e}"),
        }
    }
    el.abrir(preferencias(app).conservar_mis_turnos, fecha_de_ahora());
    ventana::proteger_el_cuaderno(app, true);
    avisar(app);
}

/// Dejaste de escuchar. Con algo tuyo dentro, la reunión sigue abierta —y el cuaderno protegido—
/// hasta que la guardes o la descartes. **Sin nada tuyo, se cierra aquí**: no hay nada que guardar ni
/// que proteger, y dejar el cuaderno negro al compartir hasta la próxima sesión no protegería nada.
pub fn al_terminar<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    if el.terminar() {
        ventana::proteger_el_cuaderno(app, false);
    }
    avisar(app);
}

/// ⌥⎋: tus turnos y la ficha vigente mueren; tus notas, tus acuerdos y tus fijadas se quedan.
pub fn cortar<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    el.con(Cuaderno::cortar);
    al_terminar(app);
}

fn cerrar<R: Runtime>(app: &AppHandle<R>, el: &ElCuaderno) {
    if el.cerrar() {
        ventana::proteger_el_cuaderno(app, false);
    }
    avisar(app);
}

/// «Guardar cifrado y cerrar». `Ok(None)` si no había nada tuyo: no hay archivo, y la reunión se cierra
/// igual. Si falla, **la nota se queda** en memoria y la reunión, abierta: se dice por qué.
pub fn guardar<R: Runtime>(app: &AppHandle<R>) -> Result<Option<Guardada>, String> {
    let el = app.try_state::<ElCuaderno>().ok_or("el cuaderno no está listo")?;
    let estaba = el.abierta();
    let hecho = el.guardar_en(&carpeta(app), &DelLlavero, preferencias(app).retencion, carpeta::ahora())?;
    if estaba && !el.abierta() {
        ventana::proteger_el_cuaderno(app, false);
    }
    avisar(app);
    Ok(hecho.map(|(guardada, resumen)| {
        println!("{}", carpeta::linea_de_log(&resumen, &guardada));
        guardada
    }))
}

/// «Cerrar sin guardar»: lo tuyo de esta reunión se tira, con las letras pisadas.
pub fn descartar<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    let resumen = el.con(|c| c.resumen()).unwrap_or_default();
    cerrar(app, &el);
    println!(
        "[notas] cerrada sin guardar · {} acuerdos · {} fijadas · {} turnos tuyos, tirados",
        resumen.acuerdos, resumen.fijadas, resumen.turnos
    );
}

/// Al salir de la app: si queda algo tuyo sin guardar, se guarda (ADR 015 §7).
pub fn al_salir<R: Runtime>(app: &AppHandle<R>) {
    let Some(el) = app.try_state::<ElCuaderno>() else { return };
    if el.vacio() {
        return;
    }
    match guardar(app) {
        Ok(_) => println!("[notas] al salir quedaban notas sin guardar: guardadas"),
        Err(e) => println!("[notas] al salir quedaban notas sin guardar y NO se pudieron guardar: {e}"),
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
        let _ = app.emit(EVENTO, ());
    }
}

/// ⌃⌥P: fija la ficha que la banda enseña.
pub fn fijar<R: Runtime>(app: &AppHandle<R>) -> Option<FichaFijada> {
    let el = app.try_state::<ElCuaderno>()?;
    let fijada = el.con(|c| c.fijar_la_vigente().cloned()).flatten();
    println!("[notas] ⌃⌥P · {}", if fijada.is_some() { "ficha fijada" } else { "no había ficha que fijar" });
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
    let carpeta_elegida = preferencias(app).carpeta_de_notas.map(|c| con_virgulilla(&c));
    ListaDeReuniones { carpeta: carpeta_elegida, reuniones: carpeta(app).lista() }
}

/// `/Users/quien/Notas` → `~/Notas`: tu carpeta de usuario no hace falta en pantalla.
fn con_virgulilla(ruta: &str) -> String {
    match std::env::var("HOME") {
        Ok(casa) if !casa.is_empty() && ruta.starts_with(&casa) => format!("~{}", &ruta[casa.len()..]),
        _ => ruta.to_string(),
    }
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

/// Abre una reunión guardada: pide el desbloqueo una vez por sesión de la app (ADR 015 §5).
pub fn abrir<R: Runtime>(app: &AppHandle<R>, archivo: &str, idioma: &str) -> Result<Contenido, String> {
    let el = app.try_state::<ElCuaderno>().ok_or("el cuaderno no está listo")?;
    el.desbloqueo.asegurar(|| desbloqueo::pedir(desbloqueo::razon(idioma)))?;
    let t = Instant::now();
    let c = carpeta(app).abrir(&DelLlavero, archivo)?;
    println!("[notas] reunión abierta en {} ms", t.elapsed().as_millis());
    Ok(c)
}

/// Exporta a texto donde elijas. Pide el desbloqueo, como abrir. `Ok(false)` si cancelaste el diálogo.
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
    Ok(())
}

pub fn fijar_retencion<R: Runtime>(app: &AppHandle<R>, retencion: prefs::Retencion) {
    crate::recordar(app, |p| p.retencion = retencion);
    avisar(app);
}

/// Si macOS negó Documentos: elegir otra carpeta. `None` si cancelaste.
pub fn elegir_otra_carpeta<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let elegida = app.dialog().file().blocking_pick_folder()?.into_path().ok()?;
    let texto = elegida.to_string_lossy().into_owned();
    crate::recordar(app, |p| p.carpeta_de_notas = Some(texto.clone()));
    println!("[notas] carpeta de notas cambiada por el usuario");
    Some(texto)
}

/// Barre lo vencido ahora y luego cada hora, mientras la app corre.
pub fn arrancar_el_barrido<R: Runtime>(app: &AppHandle<R>) {
    let mango = app.clone();
    std::thread::spawn(move || loop {
        let borradas = carpeta(&mango).barrer(carpeta::ahora());
        if borradas > 0 {
            println!("[notas] {borradas} reunión(es) vencida(s), borrada(s)");
        }
        std::thread::sleep(BARRIDO_CADA);
    });
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

    #[test]
    fn si_guardar_falla_la_nota_se_queda_y_la_reunion_sigue_abierta() {
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| c.escribir("Piden la cuarta fuente."));
        let sin_llavero = EnMemoria { no_contesta: true, ..EnMemoria::default() };
        assert!(el.guardar_en(&carpeta("falla"), &sin_llavero, prefs::Retencion::Dias90, 0).is_err());
        assert_eq!(el.con(|c| c.nota().to_string()).unwrap(), "Piden la cuarta fuente.", "la nota se perdió al fallar");
        assert!(el.abierta(), "la reunión se cerró aunque no se guardó");
    }

    #[test]
    fn guardar_cierra_la_reunion_y_deja_el_archivo_con_su_nombre() {
        let el = ElCuaderno::default();
        el.abrir(false, HOY);
        el.con(|c| c.acordar("Cuarta fuente: cotización aparte"));
        let c = carpeta("guarda");
        let (g, r) = el.guardar_en(&c, &EnMemoria::default(), prefs::Retencion::Dias7, 1_000).unwrap().unwrap();
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
        el.con(|c| {
            c.proponer(vec![crate::propuestas::Propuesta {
                regla: crate::propuestas::Regla::Cifra,
                de: crate::propuestas::De::Cliente,
                texto: "12 semanas".into(),
                ficha: None,
                seccion: None,
                hora: "14:16".into(),
            }])
        });
        assert!(!el.terminar(), "con propuestas sin decidir, parar cerró la reunión y se las llevó");
        let c = carpeta("vacia");
        let vacio = ElCuaderno::default();
        assert_eq!(vacio.guardar_en(&c, &EnMemoria::default(), prefs::Retencion::Dias90, 0).unwrap(), None);
        assert!(!c.raiz().exists(), "sin nada tuyo se creó la carpeta o un archivo");
    }

    #[test]
    fn empezar_otra_con_la_anterior_abierta_pide_guardarla() {
        let el = ElCuaderno::default();
        assert!(!el.hay_que_guardar_la_anterior());
        el.abrir(false, HOY);
        assert!(!el.hay_que_guardar_la_anterior(), "vacía no hay nada que guardar");
        el.con(|c| c.escribir("de la reunión anterior"));
        assert!(el.hay_que_guardar_la_anterior(), "se perdería lo de la reunión anterior");
    }
}
