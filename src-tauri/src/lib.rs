//! Angel Ghost — núcleo nativo.
//!
//! La frontera que organiza este crate no es técnica, es la regla del efímero verificable:
//!
//! - **Protegidos** (`capture`, `stt`): RAM y nada más. `pnpm verify:ephemeral` barre estos
//!   directorios buscando API de disco y de red, y la CI se pone roja si aparece una.
//! - **Libres** (`corpus`): pueden abrir disco porque manejan lo que ES del usuario —sus
//!   documentos, su índice—, que la regla permite persistir.
//! - **Libres** (`ventana`, `acople`, `relleno`): geometría, ventanas y el rectángulo que la
//!   captura encuentra donde está la banda. No tocan datos de la reunión.
//!
//! Todo lo demás vive en la raíz del crate. El sprint 001 va llenando estos módulos por fases.

pub mod acople;
pub mod almacen;
pub mod bandeja;
pub mod capture;
pub mod carpeta;
pub mod corpus;
pub mod corte;
pub mod desbloqueo;
/// El contrato con la interfaz, y el gate que lo compara. Solo se compila en `cargo test`: su
/// trabajo es escribir `src/contrato.generado.ts`, no viajar en el binario del usuario.
#[cfg(test)]
mod contrato;
pub mod diccionario;
pub mod disparo;
pub mod ensayo;
pub mod ensayos;
pub mod escucha;
pub mod ficha;
pub mod habla;
pub mod hardware;
pub mod jurisdiccion;
pub mod llavero;
pub mod modo;
pub mod notas;
pub mod pantalla;
pub mod permisos;
pub mod prefs;
pub mod propuestas;
pub mod puerta;
pub mod radar;
pub mod red;
pub mod relleno;
pub mod reunion;
pub mod sesion;
pub mod sintesis;
pub mod stt;
pub mod vencimiento;
pub mod ventana;
pub mod voz;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

/// Dónde anota el acople lo que encogió, para poder devolverlo **aunque esta sesión termine
/// mal**. En la carpeta de configuración de la app, nunca en el repo ni en un temporal del
/// sistema: un temporal lo barre macOS, y entonces la ventana de la reunión se queda encogida sin
/// que nadie sepa quién lo hizo.
fn huella<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> PathBuf {
    let base = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| std::env::temp_dir());
    acople::ruta_de_la_huella(&base)
}

// ═══════════════════════════════════════════════════════ EL DICCIONARIO DEL CONSULTOR, EN DISCO
//
// **Por qué el archivo se lee y se escribe AQUÍ y no en `diccionario/`.** Ese módulo recibe cada
// turno del cliente y devuelve el turno corregido, así que tiene el transcript en las manos: está
// en la lista de `verify:ephemeral` y **no puede tocar disco**. La serialización vive allí, en dos
// funciones que van de `Diccionario` a `String` y al revés; el `fs::read_to_string`, el `fs::write`
// y los permisos viven aquí, en la capa que no ve un solo turno.
//
// El plan del sprint decía «`diccionario/` puede tocar disco». Esto es más fuerte y cuesta lo mismo:
// el módulo que toca la voz del cliente no tiene manera de escribirla, y no hace falta confiar en
// que nadie se equivoque al añadir la función siguiente.

/// Cómo se llama el archivo. En la carpeta de configuración de la app, al lado de la huella del
/// acople — es del usuario y él lo edita a mano.
const DICCIONARIO: &str = "diccionario.yaml";

fn ruta_del_diccionario<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| std::env::temp_dir())
        .join(DICCIONARIO)
}

/// Deja el archivo escrito **si no existía**, con la semilla, y cierra sus permisos siempre.
///
/// Se llama al arrancar y por una razón de producto: **un archivo que no existe no se puede
/// editar**. El manual le dice al usuario dónde está su diccionario, y si la app esperara a tener
/// algo que guardar para crearlo, el usuario iría a buscarlo y no encontraría nada.
///
/// Los permisos se aseguran **también cuando ya existía**, igual que hace el índice del corpus: una
/// versión anterior pudo dejarlo flojo, y descubrirlo no sirve de nada si no se repara.
pub fn asegurar_el_diccionario(ruta: &std::path::Path) -> Result<(), String> {
    // La carpeta, en 700 como la del resto de lo que escribe la app (auditoría del S3, B1).
    if let Some(padre) = ruta.parent() {
        almacen::carpeta_privada(padre)?;
    }
    if !ruta.exists() {
        nacer_cerrado(ruta, &diccionario::Diccionario::semilla().a_texto())?;
        println!("[diccionario] archivo nuevo con la semilla en {}", ruta.display());
    }
    if cerrar_permisos(ruta)? {
        println!("[diccionario] lo encontró abierto y lo dejó en {:o}", almacen::ARCHIVO);
    }
    Ok(())
}

/// Crea el archivo **ya con sus permisos puestos** (`almacen`, que desde el sprint 003 es el único
/// escritor de la app). La primera versión usaba `std::fs::write` y el propio gate del efímero lo
/// delató en su traza: «el archivo estaba en 644; se dejó en 600».
fn nacer_cerrado(ruta: &std::path::Path, contenido: &str) -> Result<(), String> {
    almacen::nacer_cerrado(ruta, contenido.as_bytes())
}

/// Aprieta los permisos si los encuentra flojos, y **devuelve si hubo que repararlos**.
fn cerrar_permisos(ruta: &std::path::Path) -> Result<bool, String> {
    almacen::cerrar_permisos(ruta, almacen::ARCHIVO)
}

/// El diccionario de **esta** sesión: lo que el usuario escribió en su archivo más los nombres
/// propios de su corpus.
///
/// Se lee al empezar la sesión y no una sola vez al arrancar, a propósito: el usuario edita el
/// archivo, pulsa «Iniciar sesión» y lo nuevo ya está puesto, sin cerrar la app.
///
/// **Un archivo torcido no detiene la reunión, pero se DICE.** Si no se puede leer o no se entiende,
/// se sigue con la semilla y el motivo va al log con su número de línea. Callarlo sería lo peor de
/// los dos mundos: el usuario creería que la app conoce su jerga y la app no la conocería.
pub fn diccionario_de_la_sesion(
    ruta: &std::path::Path,
    del_corpus: &[String],
) -> std::sync::Arc<diccionario::Diccionario> {
    let mut d = match std::fs::read_to_string(ruta) {
        Ok(texto) => match diccionario::Diccionario::de_texto(&texto) {
            Ok(d) => d,
            Err(e) => {
                println!("[diccionario] {} no se entiende ({e}): se sigue con la semilla", ruta.display());
                diccionario::Diccionario::semilla()
            }
        },
        Err(e) => {
            println!("[diccionario] no se pudo leer {} ({e}): se sigue con la semilla", ruta.display());
            diccionario::Diccionario::semilla()
        }
    };
    d.con_nombres_del_corpus(del_corpus);
    println!(
        "[diccionario] {} términos · {} de tu corpus",
        d.terminos().len(),
        d.cuantos_del_corpus()
    );
    std::sync::Arc::new(d)
}

/// **Lo que la pantalla de Idioma enseña del diccionario** (mirada 17-bis, opción a): de dónde
/// salen sus términos y dónde está el archivo, que es la única puerta para editarlo.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDelDiccionario {
    pub terminos: usize,
    /// Los nombres propios que entraron solos, desde el corpus del usuario.
    pub del_corpus: usize,
    /// Los que el usuario escribió en su archivo (la semilla incluida, que también es suya).
    pub en_tu_archivo: usize,
    /// La ruta ENTERA, con `~` por la carpeta del usuario: abreviarla con «…» escondería justo la
    /// carpeta que hay que encontrar.
    pub ruta: String,
}

impl EstadoDelDiccionario {
    pub fn de(d: &diccionario::Diccionario, ruta: &std::path::Path) -> Self {
        let terminos = d.terminos().len();
        let del_corpus = d.cuantos_del_corpus();
        let ruta = ruta.display().to_string();
        let ruta = match std::env::var("HOME") {
            Ok(casa) if !casa.is_empty() && ruta.starts_with(&casa) => format!("~{}", &ruta[casa.len()..]),
            _ => ruta,
        };
        Self { terminos, del_corpus, en_tu_archivo: terminos.saturating_sub(del_corpus), ruta }
    }
}

/// La ruta del fondo de escritorio, leída **una vez, en el hilo principal** (`NSScreen` lo exige).
/// Se guarda la RUTA y no la imagen: codificarla en base64 son megabytes vivos durante toda la
/// sesión para pintar una franja de 88 px que el relleno pide una sola vez.
struct FondoDelRelleno(Option<PathBuf>);

/// El asa mientras se arrastra: ajusta la banda y su relleno a la vez. El webview no cambia su
/// propio tamaño porque entonces el relleno podría quedarse atrás; la geometría de la franja vive
/// en un solo sitio.
#[tauri::command]
fn ajustar_banda(app: tauri::AppHandle, alto: u32) -> Result<(), String> {
    // **El alto ES el modo** (manual y título del asa): arrastrarla por encima de la línea de voz
    // apaga el modo solo audio. Antes la ventana crecía y la app seguía hablando y quedándose con
    // `⎋` de la reunión (auditoría del S2, M14).
    let voz = app.state::<LaVozQueSale>();
    if el_asa_apaga_el_modo(alto, voz.encendida.load(Ordering::Relaxed)) {
        apagar_el_modo(&app);
        println!("[habla] el asa apagó el modo solo audio");
    }
    ventana::ajustar_banda(&app, borde_de(&app), alto)
}

/// El modo solo audio solo se enciende si hay voz para el idioma en que se lee.
fn puede_encender_el_modo(hay_voz: bool) -> bool {
    hay_voz
}

/// Arrastrar el asa por encima de la banda de voz, con el modo encendido, lo apaga.
fn el_asa_apaga_el_modo(alto: u32, encendida: bool) -> bool {
    encendida && alto > ventana::ALTO_VOZ
}

/// Apaga el modo solo audio: devuelve `⎋`, calla lo que esté diciendo y avisa a la banda. Lo usan
/// `⌃⌥V` y el asa, para que las dos vías apaguen exactamente lo mismo.
fn apagar_el_modo<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> habla::LaVoz {
    let estado = app.state::<LaVozQueSale>();
    estado.encendida.store(false, Ordering::Relaxed);
    con_el_callar(app, false);
    estado.voz.callar();
    let ahora = estado.estado();
    let _ = app.emit_to(ventana::BANDA, EVENTO_VOZ, ahora);
    ahora
}

/// El asa **al soltarla**: la reunión se vuelve a hacer sitio para la franja nueva.
///
/// Va aparte de [`ajustar_banda`] a propósito. Arrastrar dispara decenas de ajustes por segundo, y
/// cada acople son varias idas y vueltas a OTRO proceso por la Accessibility API: hacerlo en cada
/// cuadro convertiría el arrastre en un tirón y dejaría a la ventana de la reunión parpadeando.
/// Durante el arrastre se mueve lo nuestro; al soltar, lo ajeno.
#[tauri::command]
fn asentar_banda<R: tauri::Runtime>(app: tauri::AppHandle<R>, alto: u32) -> Result<(), String> {
    let borde = borde_de(&app);
    ventana::ajustar_banda(&app, borde, alto)?;
    let franja = ventana::franja(&app, borde, alto)?;
    let informe = match borde {
        ventana::Borde::Abajo => acople::reacoplar(franja, &huella(&app)),
        // Arriba se reacopla **la ventana de la reunión**, y solo si había algo acoplado: soltar el asa
        // no es el momento de empezar a mover ventanas que no se habían tocado.
        ventana::Borde::Arriba if acople::leer(&huella(&app)).huellas.is_empty() => acople::Informe {
            permiso: true,
            motivos: vec!["no había nada acoplado que reajustar".into()],
            ..Default::default()
        },
        ventana::Borde::Arriba => acoplar_arriba(&app, franja),
    };
    registrar_acople(&app, "reacople", &informe);
    Ok(())
}

/// Acopla arriba **la ventana de la reunión que nombra la detección** (ADR 004, enmienda 1). Sin
/// reunión detectada no se toca nada: la banda flota arriba y el log lo dice.
fn acoplar_arriba<R: tauri::Runtime>(app: &tauri::AppHandle<R>, franja: acople::Marco) -> acople::Informe {
    match sesion::ventana_de_la_reunion() {
        Some(destino) => acople::acoplar_arriba(&destino, franja, &huella(app)),
        None => acople::Informe {
            permiso: acople::hay_permiso(),
            motivos: vec!["no hay reunión detectada: la banda flota arriba sin tocar ninguna ventana".into()],
            ..Default::default()
        },
    }
}

/// Acopla según el borde: abajo, la aplicación que está al frente (el H1); arriba, la reunión.
fn acoplar_segun_el_borde<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<acople::Informe, String> {
    let borde = borde_de(app);
    let alto = ventana::alto_actual(app).unwrap_or(ventana::ALTO_COMPACTA);
    let franja = ventana::franja(app, borde, alto)?;
    Ok(match borde {
        ventana::Borde::Abajo => acople::acoplar(franja, &huella(app)),
        ventana::Borde::Arriba => acoplar_arriba(app, franja),
    })
}

// ---------------------------------------------------------------------------------------------
// La banda, arriba o abajo (sprint 004, ADR 004 enmienda 1, ADR 002 enmienda 8)
// ---------------------------------------------------------------------------------------------

/// Dónde vive la banda ahora: lo que dicen las preferencias. Arriba si todavía no se han leído.
fn borde_de<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> ventana::Borde {
    app.try_state::<LasPreferencias>()
        .and_then(|lp| lp.actuales.lock().ok().map(|p| p.posicion_de_la_banda))
        .unwrap_or_default()
}

/// Lo que las tres ventanas necesitan saber de la franja: en qué borde está (la banda dibuja su asa y su
/// sombra con eso; Sesión, el interruptor), cuánto mide la barra de menús (el relleno sube su fondo eso
/// con la banda arriba) y si ya viste el aviso de la primera vez (Sesión).
#[derive(Clone, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaFranja {
    pub borde: ventana::Borde,
    pub barra: f64,
    pub aviso_visto: bool,
}

/// El nombre del evento con el que las tres ventanas se enteran de que la banda cambió de borde.
const EVENTO_FRANJA: &str = "franja";

fn la_franja_de<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> LaFranja {
    let aviso_visto = app
        .try_state::<LasPreferencias>()
        .and_then(|lp| lp.actuales.lock().ok().map(|p| p.aviso_de_arriba_visto))
        .unwrap_or(false);
    LaFranja { borde: borde_de(app), barra: ventana::barra(app), aviso_visto }
}

#[tauri::command]
fn la_franja(app: tauri::AppHandle) -> LaFranja {
    la_franja_de(&app)
}

/// Pone la banda en un borde: **suelta → recoloca → reacopla**. Primero se devuelve entera la ventana
/// que estuviera acoplada (el acople del otro borde ya no vale), luego la banda y su relleno cambian de
/// sitio juntos, y al final se acopla lo que toque en el borde nuevo. Se guarda en tus preferencias.
fn poner_la_banda<R: tauri::Runtime>(app: &tauri::AppHandle<R>, borde: ventana::Borde) {
    recordar(app, |p| p.posicion_de_la_banda = borde);
    registrar_acople(app, "soltar para cambiar de borde", &acople::soltar(&huella(app)));
    let alto = ventana::alto_actual(app).unwrap_or(ventana::ALTO_COMPACTA);
    if let Err(e) = ventana::ajustar_banda(app, borde, alto) {
        println!("[ventanas] la banda no pudo cambiar de borde: {e}");
    }
    println!("[ventanas] la banda va {}", if borde == ventana::Borde::Arriba { "arriba" } else { "abajo" });
    let _ = app.emit(EVENTO_FRANJA, la_franja_de(app));
    match acoplar_segun_el_borde(app) {
        Ok(informe) => registrar_acople(app, "acople en el borde nuevo", &informe),
        Err(e) => println!("[acople] no se pudo calcular la franja nueva: {e}"),
    }
}

/// «La banda: arriba · abajo», en Sesión. Va en otro hilo: el acople son idas y vueltas a otro proceso
/// por la Accessibility API, y el comando no tiene por qué esperarlas.
#[tauri::command]
fn fijar_posicion_de_la_banda(app: tauri::AppHandle, borde: ventana::Borde) -> LaFranja {
    let mango = app.clone();
    std::thread::spawn(move || poner_la_banda(&mango, borde));
    LaFranja { borde, ..la_franja_de(&app) }
}

/// «Entendido» en el aviso de la primera vez con la banda arriba: no vuelve a salir.
#[tauri::command]
fn entendido_el_aviso_de_arriba(app: tauri::AppHandle) -> LaFranja {
    recordar(&app, |p| p.aviso_de_arriba_visto = true);
    let franja = la_franja_de(&app);
    let _ = app.emit(EVENTO_FRANJA, franja.clone());
    franja
}

/// `⌃⌥B` — la banda, al otro borde (sprint 004). La octava letra con `⌃⌥`.
fn el_atajo_de_la_banda() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyB)
}

/// Lo que la banda necesita para dibujar «acoplada» o «sin acople» — y lo que hace que esa
/// palabra sea un hecho comprobado, no una etiqueta fija.
#[derive(Clone, serde::Serialize)]
pub struct EstadoDelAcople {
    // `permiso` salió en la auditoría del S2 (B10): nadie lo leía, y rellenarlo costaba una llamada
    // a la Accessibility API en cada consulta.
    pub acoplada: bool,
}

/// El nombre del evento con el que la banda se entera de que el acople cambió.
const EVENTO_ACOPLE: &str = "acople";

fn estado_ahora<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> EstadoDelAcople {
    EstadoDelAcople {
        // La verdad está en la huella, no en una variable nuestra: es el mismo archivo que usa
        // la devolución, así que la banda no puede decir «acoplada» mientras no hay nada que
        // devolver, ni al revés.
        acoplada: !acople::leer(&huella(app)).huellas.is_empty(),
    }
}

#[tauri::command]
fn estado_del_acople(app: tauri::AppHandle) -> EstadoDelAcople {
    estado_ahora(&app)
}

/// El fondo de escritorio para el relleno, como `data:` listo para CSS. `None` = negro, que es la
/// otra opción que el diseño aprobó y la única que no filtra nada.
///
/// Lo pide el relleno al montarse en vez de empujárselo por evento: un evento emitido antes de
/// que su webview registre el oyente se pierde en silencio, y la franja se quedaría negra sin que
/// nada lo dijera. Preguntando, el orden lo pone quien necesita la respuesta.
#[tauri::command]
fn fondo_del_relleno(estado: tauri::State<'_, FondoDelRelleno>) -> Option<String> {
    let ruta = estado.0.as_ref()?;
    match relleno::leer(ruta) {
        relleno::Fondo::Imagen(url) => Some(url),
        relleno::Fondo::Negro(motivo) => {
            println!("[relleno] negro: {motivo}");
            None
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Fase 2 — sesión, permisos, contador de red y el kill-switch
// ---------------------------------------------------------------------------------------------

/// ¿Qué videollamada hay abierta? Se consulta, no se vigila: la pantalla de sesión pregunta
/// cuando se muestra, y no hay nada corriendo en segundo plano mirando las ventanas del usuario.
#[tauri::command]
fn reunion_abierta() -> sesion::Reunion {
    sesion::ahora()
}

#[tauri::command]
fn permisos_de_macos() -> permisos::Permisos {
    permisos::leer()
}

/// Abre el panel de Ajustes del Sistema donde se concede un permiso. **No pide el permiso**: lo
/// concede el usuario en el sistema, que es lo que la maqueta decidió.
#[tauri::command]
fn abrir_ajustes_de(permiso: String) -> Result<(), String> {
    let url = permisos::ajustes_de(&permiso)
        .ok_or_else(|| format!("«{permiso}» no es un permiso que esta app pida"))?;
    tauri_plugin_opener::open_url(url, None::<&str>).map_err(|e| e.to_string())
}

/// Los bytes que han salido del equipo en esta reunión, ya formateados como los escribe la
/// maqueta. Se devuelve **leído**, no como constante de la interfaz: un contador escrito a mano
/// en el webview no es un contador.
#[tauri::command]
fn bytes_a_la_red() -> String {
    red::formatear(red::bytes())
}

// ---------------------------------------------------------------------------------------------
// Fase 3 — las dos pistas, los turnos y la transcripción local
// ---------------------------------------------------------------------------------------------

/// La escucha en marcha, si la hay. Vive en un `Mutex` porque empezar y cortar pueden llegar por
/// caminos distintos —un botón, una tecla global, el cierre de la app— y el único desenlace
/// inaceptable es que dos de ellos se pisen y dejen un grifo abierto sin nadie que lo cierre.
#[derive(Default)]
struct LaEscucha(std::sync::Mutex<Option<escucha::Escucha>>);

/// El nombre del evento con el que la banda se entera de que alguien habló.
const EVENTO_ESCUCHA: &str = "escucha";

/// EL CORPUS DEL USUARIO, vivo mientras la app esté abierta.
///
/// Se comparte con la escucha —que necesita buscar en él cuando el cliente pregunta— por
/// `Arc`, no copiándolo: el índice es uno solo y reindexar desde la pantalla tiene que verse en
/// la banda en el acto, sin reiniciar nada.
#[derive(Default, Clone)]
struct ElCorpus(std::sync::Arc<std::sync::Mutex<Option<corpus::Corpus>>>);

impl escucha::Buscador for ElCorpus {
    fn buscar(&self, texto: &str, cuantos: usize) -> Vec<corpus::Hallazgo> {
        self.0
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|c| c.buscar(texto, cuantos).unwrap_or_default()))
            .unwrap_or_default()
    }
    fn vocabulario(&self) -> Vec<String> {
        self.0
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|c| c.vocabulario().to_vec()))
            .unwrap_or_default()
    }
}

/// **EL CORPUS CON LA PANTALLA DELANTE** (C8): el buscador que usa la sesión.
///
/// Es el mismo corpus, y a cada búsqueda le añade como contexto lo que la pantalla que comparte el
/// cliente aporta ahora mismo (`pantalla::Refuerzo`), con menos peso que la pregunta y sin poder
/// traer nada que la pregunta no pidiera (`corpus::Indice::buscar_con_pantalla`). Sin lectura de
/// pantalla —apagada, sin permiso, sin reunión— el refuerzo está vacío y la búsqueda es
/// exactamente la de siempre; lo vigila un test del índice.
///
/// La copia del texto de la pantalla que se hace para buscar **se pisa al terminar**: es texto de
/// un tercero, y vive lo que dura la búsqueda.
#[derive(Clone)]
struct ConPantalla {
    corpus: ElCorpus,
    pantalla: std::sync::Arc<std::sync::Mutex<pantalla::Refuerzo>>,
}

impl escucha::Buscador for ConPantalla {
    fn buscar(&self, texto: &str, cuantos: usize) -> Vec<corpus::Hallazgo> {
        let mut contexto = self
            .pantalla
            .lock()
            .map(|r| r.consulta())
            .unwrap_or_default();
        let hallazgos = self
            .corpus
            .0
            .lock()
            .ok()
            .and_then(|g| {
                g.as_ref().map(|c| {
                    c.buscar_con_pantalla(texto, &contexto, cuantos)
                        .unwrap_or_default()
                })
            })
            .unwrap_or_default();
        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
        unsafe { contexto.as_mut_vec() }.fill(0);
        hallazgos
    }
    fn vocabulario(&self) -> Vec<String> {
        escucha::Buscador::vocabulario(&self.corpus)
    }
}

/// Dónde vive el índice: en la carpeta de datos de la app, **jamás en el repo ni al lado de los
/// documentos del usuario**. La pantalla de corpus enseña esta ruta, porque quien confía su
/// carpeta a una app tiene derecho a saber dónde acabó el derivado.
fn donde_va_el_indice(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    use tauri::Manager;
    app.path()
        .app_data_dir()
        .map(|d| d.join("corpus"))
        .map_err(|e| format!("no se supo dónde poner el índice: {e}"))
}

/// Abre el selector de carpetas y devuelve lo que el usuario eligió. `None` si canceló.
///
/// Va aparte de `indexar_corpus` porque son dos cosas distintas y una de ellas tarda: elegir es
/// instantáneo, indexar ciento cuarenta documentos no. Juntarlas dejaría la ventana congelada
/// desde el clic hasta el final, y la maqueta promete progreso «sin bloquear la ventana».
/// No es `async` a propósito: Tauri corre los comandos síncronos en su pool de hilos, así que
/// esperar aquí al usuario **no congela la ventana**, y a cambio no hace falta traerse un
/// runtime asíncrono entero para una llamada que ocurre una vez cada varios días.
#[tauri::command]
fn elegir_carpeta(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    app.dialog().file().blocking_pick_folder().map(|r| r.to_string())
}

/// Indexa la carpeta que el usuario señale. Los documentos **no se copian**: se leen donde están.
#[tauri::command]
fn indexar_corpus(
    app: tauri::AppHandle,
    estado: tauri::State<'_, ElCorpus>,
    carpeta: String,
) -> Result<corpus::EstadoDelCorpus, String> {
    let donde = donde_va_el_indice(&app)?;
    let mut guardado = estado.0.lock().map_err(|_| "el corpus quedó en mal estado")?;
    if guardado.is_none() {
        *guardado = Some(corpus::Corpus::en(&donde)?);
    }
    let c = guardado.as_mut().expect("acaba de crearse");
    let cuantos = c.indexar(std::path::Path::new(&carpeta), &|d| {
        // Metadata, jamás contenido: el nombre del archivo es del usuario y el log es un archivo.
        let _ = d;
    })?;
    let informe = c.estado();
    drop(guardado);
    println!(
        "[corpus] {cuantos} documentos · {} secciones · {} ilegibles · índice en {}",
        informe.secciones,
        informe.ilegibles,
        informe.donde_vive.as_deref().unwrap_or("memoria")
    );
    // La carpeta se recuerda para el arranque siguiente —la ruta, no su contenido— (auditoría del S3,
    // B29), y Corpus y Sesión se enteran de que el corpus cambió.
    recordar(&app, |p| p.carpeta_del_corpus = Some(carpeta.clone()));
    let _ = app.emit_to(ventana::PRINCIPAL, EVENTO_CORPUS, ());
    Ok(informe)
}

/// El evento con que Corpus y Sesión («Este cliente») se enteran de que el corpus cambió: una señal.
const EVENTO_CORPUS: &str = "corpus";

/// **La carpeta que recordabas, otra vez leída** (auditoría del S3, B29). En segundo plano: el arranque
/// no espera a indexar. Si la carpeta ya no está, se dice sin su ruta y no se olvida: puede ser un
/// disco que no está conectado. Si está en Documentos, Escritorio o Descargas, macOS puede preguntarte
/// la primera vez si Angel Ghost puede leerla.
fn reindexar_al_arrancar(app: &tauri::AppHandle, carpeta: String) {
    let app = app.clone();
    std::thread::spawn(move || {
        if !std::path::Path::new(&carpeta).is_dir() {
            println!("[corpus] la carpeta que recordabas no está: señálala otra vez en Corpus");
            return;
        }
        if let Err(e) = indexar_corpus(app.clone(), app.state::<ElCorpus>(), carpeta) {
            println!("[corpus] no se pudo volver a leer la carpeta al arrancar: {}", e.split(": ").last().unwrap_or(""));
        }
    });
}

/// Qué hay en el corpus ahora mismo. Lo pide la pantalla de Corpus.
#[tauri::command]
fn estado_del_corpus(estado: tauri::State<'_, ElCorpus>) -> Option<corpus::EstadoDelCorpus> {
    estado.0.lock().ok()?.as_ref().map(|c| c.estado())
}

/// Las piezas que el kill-switch cortaría, **leídas de `corte::TODAS`** y no escritas en la
/// interfaz. La pantalla de Honestidad las contaba a mano con dos constantes y lo declaraba en un
/// comentario; ahora pregunta. Es la misma doctrina del panel: medir, no afirmar.
#[tauri::command]
fn piezas_del_corte() -> corte::Informe {
    corte::Informe {
        piezas: corte::TODAS.iter().map(|p| (*p, corte::suerte_en_este_sprint(*p))).collect(),
        bytes_en_red: red::bytes(),
    }
}

/// Empieza a escuchar las dos pistas.
///
/// **No se llama sola al arrancar**, y es deliberado: la maqueta de la pantalla de Sesión dice
/// «nada se enciende hasta que tú lo digas», y encender el micrófono de alguien sin que lo pida
/// sería exactamente lo que esta app promete no hacer.
///
/// **Y es por donde la banda VUELVE tras el kill-switch** (hallazgo M4). `⌥⎋` cierra la banda —es
/// una de las piezas del corte— y hasta el sprint 002 no había forma de recuperarla sin
/// reiniciar la app. El sitio es este y no un botón nuevo: la banda es donde la ficha aparece, así
/// que empezar una sesión sin banda es empezar una sesión sin ningún sitio donde enseñar nada. El
/// invariante queda en un solo lado —hay escucha ⇒ hay banda— y no en cada lugar de la interfaz
/// que se acuerde de pedirla.
///
/// **Lo que NO se repone: el acople.** El corte lo suelta a propósito, y volver a encoger la
/// ventana de la reunión sin que nadie lo pida sería deshacer una pieza del kill-switch por la
/// puerta de atrás. La banda vuelve flotando y lo dice —«sin acople» sale de la huella, no de una
/// variable—, y el asa la vuelve a acoplar cuando el usuario quiera.
#[tauri::command]
fn empezar_a_escuchar(
    app: tauri::AppHandle,
    estado: tauri::State<'_, LaEscucha>,
    el_corpus: tauri::State<'_, ElCorpus>,
) -> Result<escucha::EstadoDeEscucha, String> {
    empezar(app, &estado, &el_corpus, modo::Modo::Normal)
}

/// **«Solo notas»** (ADR 017 §5): la reunión se abre —el cuaderno protegido y la banda— y **no se
/// captura nada**: ni micrófono, ni audio del sistema, ni transcripción, ni pantalla, ni radar ámbar.
/// Se llega desde Sesión, por la NDA de tu cliente o porque lo decides tú.
#[tauri::command]
fn empezar_solo_notas(
    app: tauri::AppHandle,
    estado: tauri::State<'_, LaEscucha>,
    el_corpus: tauri::State<'_, ElCorpus>,
) -> Result<escucha::EstadoDeEscucha, String> {
    empezar(app, &estado, &el_corpus, modo::Modo::SoloNotas)
}

/// El nombre del evento con el que la banda y Sesión se enteran de que la reunión empezó o terminó. Es
/// una **señal** sin dato: el oyente vuelve a preguntar el estado (auditoría del S3, B14; casilla 5).
const EVENTO_MODO: &str = "modo";

fn empezar(
    app: tauri::AppHandle,
    estado: &tauri::State<'_, LaEscucha>,
    el_corpus: &tauri::State<'_, ElCorpus>,
    modo: modo::Modo,
) -> Result<escucha::EstadoDeEscucha, String> {
    // **Los idiomas salen de las preferencias, que son la única fuente** (sprint 003). En la fase 0
    // los mandaba el webview desde su caché, y la casilla 6 lo cazó: después de reiniciar, Sesión
    // mandaba los de fábrica si el usuario no había pasado antes por Idioma. Sin parámetros, el
    // webview ya no tiene cómo mandar unos viejos.
    // **El ensayo y la reunión no conviven** (ADR 019 §6.5): empezar a escuchar corta el ensayo, y su
    // micrófono se cierra antes de que se abra el de la reunión.
    if soltar_el_ensayo(&app) {
        println!("[ensayo] cortado: empieza una reunión");
    }
    // **La puerta local se cierra primero** (ADR 018 §5): un agente no toca jamás una reunión, y la
    // reunión empieza aquí.
    cerrar_la_puerta_al_empezar(&app);
    let prefs::IdiomasDePista { consultor: idioma_del_consultor, cliente: idioma_del_cliente } =
        idiomas_guardados(&app.state::<LasPreferencias>());
    let mut guardada = estado.0.lock().map_err(|_| "la escucha quedó en mal estado")?;
    if let Some(vieja) = guardada.take() {
        vieja.cortar();
    }
    // Una reunión nueva: su costo, sus latencias y **su contador de red** empiezan de cero (el gasto
    // del mes, no). El contador dice «salieron de tu equipo en esta reunión», y solo lo ponía a cero
    // `⌥⎋`: una reunión heredaba los bytes de la anterior (auditoría del S2, B18).
    red::reiniciar();
    println!("[red] reunión nueva: el contador vuelve a 0 B");
    // La reunión se abre aquí: el cuaderno se protege de la captura hasta que la guardes o la
    // descartes, y si la anterior seguía abierta con algo tuyo, se guarda antes (ADR 015 §7 y §10).
    reunion::al_empezar(&app);
    {
        let s = app.state::<LaSintesis>();
        if let Ok(mut u) = s.reunion_usd.lock() {
            *u = 0.0;
        };
        if let Ok(mut l) = s.latencias.lock() {
            l.clear();
        };
        s.registro.vaciar();
    }
    // Si la banda sigue en pantalla, esto no hace nada: `abrir_banda` es idempotente.
    let la_habian_cortado = app.get_webview_window(ventana::BANDA).is_none();
    match ventana::abrir_banda(&app, borde_de(&app), ventana::ALTO_COMPACTA) {
        // Que la banda no vuelva no impide escuchar, y callarlo sí sería un problema: el usuario
        // vería el transcript sin banda y no sabría por qué.
        Err(e) => println!("[ventanas] la banda no pudo volver: {e}"),
        // Se dice solo cuando de verdad volvió. Es la única traza de que M4 está cableado, y es
        // por donde se verificó en vivo: sin ella, «vuelve» sería una afirmación sin testigo.
        Ok(()) if la_habian_cortado => println!("[ventanas] la banda estaba cortada: vuelve"),
        Ok(()) => {}
    }
    // **Arriba, el disparador del acople es la reunión** (ADR 004, enmienda 1): al empezar, la reunión
    // ya está abierta, así que se acopla su ventana si no lo estaba. Abajo no hace falta: al pulsar
    // «Iniciar sesión» la aplicación de delante somos nosotros, y el H1 acopla con su latido.
    if borde_de(&app) == ventana::Borde::Arriba && acople::leer(&huella(&app)).huellas.is_empty() {
        let mango = app.clone();
        std::thread::spawn(move || match acoplar_segun_el_borde(&mango) {
            Ok(informe) => registrar_acople(&mango, "al empezar la reunión", &informe),
            Err(e) => println!("[acople] no se pudo calcular la franja: {e}"),
        });
    }
    // **LA PUERTA DE LA CAPTURA** (ADR 017 §5). Todo lo que oye o mira la reunión —la pantalla, las
    // pistas, la transcripción— arranca DESPUÉS de esta línea, y en solo notas no se llega. Un test de
    // esta fuente vigila el orden (`pruebas_de_la_puerta_de_la_captura`).
    reunion::marcar_solo_notas(&app, !modo::abre_la_captura(modo));
    if !modo::abre_la_captura(modo) {
        parar_la_pantalla(&app);
        println!("[sesión] modo solo notas: ni pistas, ni transcripción, ni pantalla");
        let _ = app.emit(EVENTO_MODO, ());
        return Ok(escucha::EstadoDeEscucha::solo_notas());
    }
    let mango = app.clone();
    // **El modo solo audio lee en el idioma del CONSULTOR**, no del cliente: la ficha sale de los
    // documentos del usuario, así que está escrita en su idioma. Se fija aquí, que es el único sitio
    // donde la app se entera de cuál eligió.
    if let Ok(mut i) = app.state::<LaVozQueSale>().idioma.lock() {
        *i = idioma_del_consultor.clone();
    }
    // La lectura de pantalla arranca ANTES que la escucha, porque el buscador de la escucha lleva
    // dentro lo que la pantalla aporta: si arrancara después, los primeros turnos buscarían sin ella.
    let refuerzo = arrancar_la_pantalla(&app, el_corpus.inner().clone());
    let buscador = std::sync::Arc::new(ConPantalla {
        corpus: el_corpus.inner().clone(),
        pantalla: refuerzo,
    });
    // Los nombres propios salen del corpus DEL USUARIO, jamás de la reunión: es lo que hace que el
    // diccionario no sea un transcript persistido con otro nombre.
    // Solo los nombres de cliente, con sus mayúsculas y sus tildes: el vocabulario del disparador son
    // palabras sueltas sin tildes («valle», «manejo»), y como términos del diccionario estropeaban
    // el castellano corriente del cliente (auditoría del S2, A3).
    let jerga = diccionario_de_la_sesion(&ruta_del_diccionario(&app), &clientes_del_corpus(el_corpus.inner()));
    let nueva = escucha::Escucha::arrancar(
        &idioma_del_consultor,
        &idioma_del_cliente,
        stt::motor_de_la_casa(),
        buscador,
        jerga,
        move |novedad| {
            // Al log va **el hecho, nunca lo dicho**: quién habló y cuánto duró. El texto es del
            // cliente y un log es un archivo.
            match &novedad {
                escucha::Novedad::Turno(t) => println!(
                    "[escucha] turno de «{}» · {} ms · {} letras{}",
                    t.pista.etiqueta(),
                    t.duracion_ms(),
                    t.texto.chars().count(),
                    if t.eco { " · marcado como eco" } else { "" }
                ),
                escucha::Novedad::SinTexto { pista, motivo, .. } => {
                    println!("[escucha] turno de «{}» sin texto: {motivo}", pista.etiqueta())
                }
                escucha::Novedad::Aparece(a)
                    // El presupuesto del sprint es 4 s de fin de turno a ficha. Se dice cuando se
                    // pasa, en el momento, y no al final en una media que esconde los picos.
                    if a.ms > 4_000 => {
                        println!("[ficha] {} ms — por encima del presupuesto de 4 s", a.ms);
                    }
                _ => {}
            }
            // **EL OPT-IN AUTOMÁTICO DEL MODO SOLO AUDIO.** Aquí y en ningún otro sitio: este es el
            // instante en que un turno del cliente acaba de traer una ficha, que es exactamente
            // cuando el usuario pidió que se le hablara — *«que me hable de forma paralela»*.
            //
            // Es opt-in de verdad: `decir_la_ficha` empieza comprobando el interruptor, y con el
            // modo apagado se va sin hacer nada y sin escribir una línea. La app no habla si nadie
            // lo encendió.
            if let escucha::Novedad::Aparece(a) = &novedad {
                decir_la_ficha(&mango, a);
                sintetizar(&mango, a);
                // Para ⌃⌥P: la ficha que la banda enseña. No es tuya hasta que la fijas.
                reunion::ver(&mango, a);
            }
            // «Conservar mis turnos»: el cuaderno decide si el turno es tuyo (micrófono, sin eco).
            if let escucha::Novedad::Turno(t) = &novedad {
                reunion::oir(&mango, t);
                // Las propuestas (ADR 016): las reglas, con tus fijadas y tu corpus para los nombres.
                // `try_lock` y no `lock`: si el corpus se está indexando, este hilo es el de la escucha
                // y no puede esperar; ese turno no propone nombres, y ya está.
                let corpus = mango.state::<ElCorpus>();
                let conoce = |nombre: &str| match corpus.0.try_lock() {
                    Ok(c) => c.as_ref().is_none_or(|c| c.conoce(nombre)),
                    Err(_) => true,
                };
                reunion::proponer(&mango, t, &conoce);
            }
            let _ = mango.emit(EVENTO_ESCUCHA, novedad);
        },
    );
    let informe = nueva.estado();
    *guardada = Some(nueva);
    let _ = app.emit(EVENTO_MODO, ());
    Ok(informe)
}

#[tauri::command]
fn dejar_de_escuchar(app: tauri::AppHandle, estado: tauri::State<'_, LaEscucha>) {
    if let Ok(mut g) = estado.0.lock() {
        if let Some(e) = g.take() {
            e.cortar();
            println!("[escucha] parada a petición del usuario");
        }
    }
    // Sin sesión no se mira la pantalla: la lectura vive lo que vive la escucha.
    parar_la_pantalla(&app);
    // Lo que salió al API era de esta reunión: IA dice «se borra al cerrar», y se borra (B37).
    app.state::<LaSintesis>().registro.vaciar();
    avisar_a_la_ia(&app);
    // La reunión sigue abierta —«al cerrar»— hasta que la guardes o la descartes.
    reunion::al_terminar(&app);
    let _ = app.emit(EVENTO_MODO, ());
}

/// Qué vive en memoria ahora mismo por culpa de la escucha. Lo pide la pantalla de Honestidad. En solo
/// notas no hay escucha, y lo dice: la reunión está abierta y nada se captura.
#[tauri::command]
fn estado_de_la_escucha(app: tauri::AppHandle, estado: tauri::State<'_, LaEscucha>) -> Option<escucha::EstadoDeEscucha> {
    let viva = estado.0.lock().ok()?.as_ref().map(|e| e.estado());
    viva.or_else(|| reunion::solo_notas(&app).then(escucha::EstadoDeEscucha::solo_notas)).or_else(|| {
        // Un ensayo (sprint 004, fase 4): su micrófono y tus respuestas, en las mismas filas de Honestidad.
        let estado = app.state::<ElEnsayo>();
        let g = estado.0.lock().ok()?;
        let (_, e) = g.as_ref()?;
        let m = e.memoria();
        Some(escucha::EstadoDeEscucha::del_ensayo(e.escuchando(), m.microfono, m.respuestas))
    })
}

/// Los últimos turnos, para el transcript de la banda.
#[tauri::command]
fn turnos_recientes(estado: tauri::State<'_, LaEscucha>, cuantos: usize) -> Vec<stt::Turno> {
    estado
        .0
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|e| e.ultimos_turnos(cuantos)))
        .unwrap_or_default()
}

/// Un idioma tal y como lo enseña la pantalla de Idioma.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct IdiomaDelMotor {
    pub(crate) codigo: String,
    pub(crate) disponibilidad: stt::Disponibilidad,
}

/// Lo que el motor de este Mac sabe hacer. **Se pregunta al sistema**, no se lleva una lista
/// escrita que quedaría desfasada con la siguiente versión de macOS.
/// **Cruza la costura con muestra** desde la auditoría del S2 (M7): su `motivo` pasó de texto libre a
/// un enum cerrado en este sprint, y era el único camino por el que ese enum cruzaba sin fixture.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct QueSabeTranscribir {
    pub(crate) motor: &'static str,
    pub(crate) techo: u32,
    pub(crate) idiomas: Vec<IdiomaDelMotor>,
    /// Si no hay motor, por qué — cerrado; Idioma lo pinta con su frase en los dos idiomas.
    pub(crate) motivo: Option<stt::PorQueNoHayMotor>,
}

#[tauri::command]
fn que_sabe_transcribir() -> QueSabeTranscribir {
    let motor = stt::motor_de_la_casa();
    let codigos = motor.idiomas();
    let motivo = match motor.disponibilidad("es-ES") {
        stt::Disponibilidad::SinMotor { motivo } => Some(motivo),
        _ => None,
    };
    QueSabeTranscribir {
        motor: motor.nombre(),
        techo: motor.techo_de_idiomas(),
        idiomas: codigos
            .into_iter()
            .map(|codigo| {
                let disponibilidad = motor.disponibilidad(&codigo);
                IdiomaDelMotor { codigo, disponibilidad }
            })
            .collect(),
        motivo,
    }
}

/// Instala el modelo de un idioma. **Usa la red y tarda**: macOS descarga su propio modelo de
/// reconocimiento. Solo se llama desde el botón de la pantalla de Idioma; la app jamás descarga
/// nada por su cuenta.
#[tauri::command]
async fn instalar_idioma(codigo: String) -> stt::Disponibilidad {
    tauri::async_runtime::spawn_blocking(move || {
        println!("[idioma] el usuario pidió instalar el modelo de {codigo}");
        let d = stt::motor_de_la_casa().instalar(&codigo);
        println!("[idioma] {codigo}: {d:?}");
        d
    })
    .await
    .unwrap_or(stt::Disponibilidad::SinMotor { motivo: stt::PorQueNoHayMotor::NoContesta })
}

/// El diccionario tal y como lo usaría una sesión que empezara ahora: el archivo del usuario más los
/// nombres propios de su corpus. Se lee cada vez, porque el usuario edita el archivo con la app
/// abierta y la pantalla tiene que enseñar lo que hay, no lo que había.
#[tauri::command]
fn estado_del_diccionario(
    app: tauri::AppHandle,
    el_corpus: tauri::State<'_, ElCorpus>,
) -> EstadoDelDiccionario {
    let ruta = ruta_del_diccionario(&app);
    let d = diccionario_de_la_sesion(&ruta, &clientes_del_corpus(el_corpus.inner()));
    EstadoDelDiccionario::de(&d, &ruta)
}

/// Por dónde sale el sonido, y por tanto si el micrófono va a oír al cliente.
#[tauri::command]
fn salida_de_audio() -> capture::nativo::Salida {
    capture::nativo::salida_de_audio()
}

/// El kill-switch. Corta lo que existe y **declara lo que todavía no**, pieza por pieza.
#[tauri::command]
fn cortar_todo(app: tauri::AppHandle) -> corte::Informe {
    ejecutar_el_corte(&app)
}

fn ejecutar_el_corte<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> corte::Informe {
    // La escucha se corta ENTERA de una vez, porque las tres piezas que le tocan (las dos pistas
    // y el transcript) comparten grifos y candados: cortarlas por separado desde fuera obligaría a
    // exponer los tres por su cuenta y a confiar en que nadie cambie el orden. Se apunta aquí y se
    // marca abajo, pieza por pieza, para que el informe siga siendo el de siempre.
    let escuchaba = {
        let estado = app.state::<LaEscucha>();
        let cortada = estado.0.lock().ok().and_then(|mut g| g.take());
        match cortada {
            Some(e) => {
                e.cortar();
                true
            }
            None => false,
        }
    };

    // Se lee ANTES del bucle: la pieza `ContadorDeRed` lo pone a cero, y leído después el log
    // diría siempre «red 0 B» (auditoría del S2, B25).
    let bytes_al_cortar = red::bytes();
    let mut piezas = Vec::new();
    for pieza in corte::TODAS {
        let suerte = corte::suerte_en_este_sprint(*pieza);
        if suerte == corte::Suerte::Cortada {
            match pieza {
                // **Lo primero, porque es lo único que el cliente oye.** Y el modo se apaga: si
                // quedara encendido, el turno siguiente volvería a hablar solo, después de que el
                // usuario acabara de cortar todo delante de alguien.
                corte::Pieza::Voz => {
                    let voz = app.state::<LaVozQueSale>();
                    voz.voz.callar();
                    if voz.encendida.swap(false, Ordering::Relaxed) {
                        con_el_callar(app, false);
                        println!("[corte] el modo solo audio estaba encendido: callado y apagado");
                        let _ = app.emit_to(ventana::BANDA, EVENTO_VOZ, voz.estado());
                    }
                }
                // La sugerencia en camino: la época sube. Lo que vuelva de antes del corte se tira
                // sin enseñarlo, y una petición al API que aún no haya salido ya no sale. La
                // reunión, además, deja de sumar costo.
                corte::Pieza::Sugerencia => cortar_la_sugerencia(&app.state::<LaSintesis>()),
                corte::Pieza::ContadorDeRed => red::reiniciar(),
                corte::Pieza::Banda => ventana::cerrar_banda(app),
                corte::Pieza::Acople => {
                    registrar_acople(app, "kill-switch", &acople::soltar(&huella(app)))
                }
                // Del cuaderno, lo que salió de la captura: tus turnos y la ficha que la banda
                // enseñaba. Tu nota, tus acuerdos y tus fijadas se quedan (ADR 015 §7).
                corte::Pieza::TusTurnos => reunion::cortar(app),
                // Las propuestas sin decidir, y la línea de la banda con ellas (ADR 016 §4).
                corte::Pieza::Propuestas => reunion::cortar_las_propuestas(app),
                // El ensayo (ADR 019 §6.4): su micrófono se cierra y lo que dijiste se pisa.
                corte::Pieza::Ensayo => {
                    if soltar_el_ensayo(app) {
                        println!("[corte] había un ensayo: micrófono cerrado y tus respuestas pisadas");
                    }
                }
                // Ya cortadas arriba, todas a la vez.
                corte::Pieza::AudioDelMicrofono
                | corte::Pieza::AudioDelSistema
                | corte::Pieza::Transcript => {}
                // El cuadro de la reunión y lo leído de él: se pisan y el vigía se para.
                corte::Pieza::UltimoFrame => {
                    if parar_la_pantalla(app) {
                        println!("[corte] la lectura de pantalla estaba en marcha: parada, cuadro y texto pisados");
                    }
                }
            }
        }
        piezas.push((*pieza, suerte));
    }
    if escuchaba {
        println!("[corte] las dos pistas estaban abiertas: cerradas y vaciadas");
    }
    let informe = corte::Informe { piezas, bytes_en_red: bytes_al_cortar };
    println!(
        "[corte] ⌥⎋: {} de {} piezas cortadas · red {}",
        informe.cortadas(),
        corte::TODAS.len(),
        red::formatear(informe.bytes_en_red)
    );
    let _ = app.emit(EVENTO_CORTE, informe.clone());
    informe
}

/// El nombre del evento con el que las pantallas se enteran de que se cortó todo.
const EVENTO_CORTE: &str = "corte";

/// Deja constancia de lo que pasó y **se lo cuenta a la banda**, que dibuja «acoplada» o «sin
/// acople» con ese dato. La banda pregunta al montarse y escucha a partir de ahí: preguntar sola
/// la dejaría sondeando cada dos segundos por algo que cambia tres veces en una reunión.
///
/// Lo que va al log son solo metadatos: cuántas ventanas y por qué no las demás. Ni títulos de
/// ventana, ni rutas, ni contenido — la Accessibility API los daría, y no se piden.
fn registrar_acople<R: tauri::Runtime>(app: &tauri::AppHandle<R>, que: &str, informe: &acople::Informe) {
    let nombre = informe.app.as_deref().unwrap_or("—");
    println!(
        "[acople] {que}: permiso={} app=«{nombre}» ventanas={} · {} ms",
        informe.permiso, informe.ventanas, informe.ms
    );
    for motivo in &informe.motivos {
        println!("[acople]   · {motivo}");
    }
    let _ = app.emit_to(ventana::BANDA, EVENTO_ACOPLE, estado_ahora(app));
}

// ---------------------------------------------------------------------------------------------
// Sprint 004 — el ensayo (C18, ADR 019)
// ---------------------------------------------------------------------------------------------

/// **El ensayo**: en marcha, o recién terminado con su informe en memoria hasta que lo guardes o lo
/// cierres. Lleva un número para que lo que vuelva del modelo no caiga en un ensayo que ya no es.
#[derive(Default)]
struct ElEnsayo(std::sync::Mutex<Option<(u64, ensayo::Ensayo)>>);

static ENSAYOS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// El evento con que la pantalla Ensayo se entera de que algo cambió. **Una señal sin dato**, solo a la
/// ventana principal: lo que dijiste no viaja en ningún evento; se pide con `estado_del_ensayo`.
const EVENTO_ENSAYO: &str = "ensayo";

/// ¿Hay un micrófono de ensayo abierto? Un candado envenenado cuenta como sí.
fn ensayando<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> bool {
    app.try_state::<ElEnsayo>()
        .is_some_and(|e| e.0.lock().map(|g| g.as_ref().is_some_and(|(_, e)| e.escuchando())).unwrap_or(true))
}

/// El ensayo se va entero: micrófono cerrado, voz callada y lo que dijiste pisado. Devuelve si había uno.
fn soltar_el_ensayo<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> bool {
    let viejo = app.state::<ElEnsayo>().0.lock().ok().and_then(|mut g| g.take());
    let habia = viejo.is_some();
    if let Some((_, e)) = viejo {
        e.cortar();
    }
    if habia {
        let _ = app.emit_to(ventana::PRINCIPAL, EVENTO_ENSAYO, ());
    }
    habia
}

fn codigo_de(idioma: ensayo::banco::Idioma) -> &'static str {
    match idioma {
        ensayo::banco::Idioma::Es => "es-ES",
        ensayo::banco::Idioma::En => "en-US",
    }
}

/// **Lo que el ensayo necesita de la app**: la voz de siempre, el corpus de siempre y la ventana
/// principal. La voz se usa **sin el candado de los auriculares** (`habla::cabe_decirla`): ese candado
/// es de la reunión, donde el cliente oiría; en el ensayo no hay nadie, y el micrófono está sordo
/// mientras habla (ADR 019 §6.3).
struct MundoDeLaApp(tauri::AppHandle);

impl ensayo::Mundo for MundoDeLaApp {
    fn decir(&self, idioma: ensayo::banco::Idioma, texto: &str) -> bool {
        let voz = &self.0.state::<LaVozQueSale>().voz;
        let codigo = codigo_de(idioma);
        voz.hay_para(codigo) && voz.decir(codigo, texto).is_ok()
    }
    fn callar(&self) {
        self.0.state::<LaVozQueSale>().voz.callar();
    }
    fn diciendo(&self) -> bool {
        self.0.state::<LaVozQueSale>().voz.hablando()
    }
    fn evidencia(&self, pregunta: &str) -> Vec<ficha::Respaldo> {
        let hallazgos = escucha::Buscador::buscar(&*self.0.state::<ElCorpus>(), pregunta, ficha::TOP);
        match ficha::armar(pregunta, &hallazgos) {
            ficha::Respuesta::Ficha(f) => f.respaldo,
            ficha::Respuesta::SinResultado { .. } => Vec::new(),
        }
    }
    fn avisar(&self) {
        let _ = self.0.emit_to(ventana::PRINCIPAL, EVENTO_ENSAYO, ());
    }
}

/// Las secciones de la propuesta elegida y de la ficha del cliente, y el nombre del documento que se
/// enseña en «Ver lo que salió». Del corpus del usuario, jamás de una reunión.
fn secciones_del_ensayo(
    c: &corpus::Corpus,
    cliente: &str,
    propuesta: Option<&str>,
) -> (Vec<corpus::seccion::Seccion>, Vec<corpus::seccion::Seccion>, String) {
    let de = |ruta: &str| c.secciones_de(ruta).unwrap_or_default();
    let prop = propuesta.map(de).unwrap_or_default();
    let ficha = c.ficha_de(cliente).map(|d| de(&d.ruta)).unwrap_or_default();
    let nombre = propuesta
        .and_then(|r| c.documentos().iter().find(|d| d.ruta == r))
        .or_else(|| c.ficha_de(cliente))
        .map(|d| d.nombre.clone())
        .unwrap_or_default();
    (prop, ficha, nombre)
}

fn tope_valido(tope: usize) -> usize {
    let topes = &ensayo::banco::catalogo().topes;
    if topes.contains(&tope) {
        tope
    } else {
        topes.get(1).copied().unwrap_or(8)
    }
}

fn el_tuyo<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> ensayo::banco::Idioma {
    if idiomas_guardados(&app.state::<LasPreferencias>()).consultor.starts_with("en") {
        ensayo::banco::Idioma::En
    } else {
        ensayo::banco::Idioma::Es
    }
}

/// **«Preparar»**: con quién, con qué propuesta, cuántas preguntas y de dónde salen. Lee tus documentos
/// y nada más: ni micrófono ni red.
#[tauri::command]
fn preparar_el_ensayo(
    app: tauri::AppHandle,
    el_corpus: tauri::State<'_, ElCorpus>,
    cliente: Option<String>,
    propuesta: Option<String>,
    tope: usize,
) -> ensayo::Preparacion {
    let tope = tope_valido(tope);
    let enriquecer = app.state::<LaSintesis>().enriquecer.load(Ordering::Relaxed);
    let el_tuyo = el_tuyo(&app);
    let topes = ensayo::banco::catalogo().topes.clone();
    let vacia = |clientes: Vec<String>, cliente: Option<String>| ensayo::Preparacion {
        clientes,
        cliente,
        propuestas: Vec::new(),
        propuesta: None,
        topes: topes.clone(),
        tope,
        cuentas: ensayo::Cuentas::default(),
        idioma: el_tuyo,
        enriquecer,
        transcribe: true,
        sin_corpus: true,
        guardados: 0,
    };
    let Ok(g) = el_corpus.0.lock() else { return vacia(Vec::new(), None) };
    let Some(c) = g.as_ref() else { return vacia(Vec::new(), None) };
    let clientes = corpus::clientes(c.documentos());
    let elegido = cliente
        .filter(|n| clientes.contains(n))
        .or_else(|| reunion::cliente(&app).filter(|n| clientes.contains(n)))
        .or_else(|| clientes.first().cloned());
    let Some(elegido) = elegido else { return vacia(clientes, None) };
    let propuestas: Vec<ensayo::Elegible> = c
        .propuestas_de(&elegido)
        .into_iter()
        .map(|d| ensayo::Elegible { ruta: d.ruta.clone(), nombre: d.nombre.clone() })
        .collect();
    let propuesta = propuesta
        .filter(|r| propuestas.iter().any(|p| &p.ruta == r))
        .or_else(|| propuestas.first().map(|p| p.ruta.clone()));
    let (prop, ficha, _) = secciones_del_ensayo(c, &elegido, propuesta.as_deref());
    let idioma = ensayo::idioma_del_ensayo(&prop, &ficha, el_tuyo);
    let banco = if prop.is_empty() && ficha.is_empty() {
        Vec::new()
    } else {
        ensayo::banco::armar(&prop, &ficha, idioma, tope)
    };
    let guardados = reunion::los_ensayos_de(&app).del_cliente(&elegido).len();
    ensayo::Preparacion {
        clientes,
        cliente: Some(elegido),
        propuestas,
        propuesta,
        topes,
        tope,
        cuentas: ensayo::Cuentas::de(&banco),
        idioma,
        enriquecer,
        // Se pregunta al sistema, que no abre nada: si el modelo de ese idioma está listo.
        transcribe: matches!(stt::motor_de_la_casa().disponibilidad(codigo_de(idioma)), stt::Disponibilidad::Listo),
        sin_corpus: banco.is_empty(),
        // Por el nombre de los archivos: sin la llave y sin abrir ninguno (ADR 015, enmienda 4).
        guardados,
    }
}

/// **Empieza el ensayo** (ADR 019 §6). Abre **solo el micrófono**: ni el audio del sistema, ni la pantalla,
/// ni el radar ámbar, que viven en `empezar`. Lo vigila `pruebas_de_la_puerta_de_la_captura`.
#[tauri::command]
fn empezar_el_ensayo(
    app: tauri::AppHandle,
    el_corpus: tauri::State<'_, ElCorpus>,
    cliente: String,
    propuesta: Option<String>,
    tope: usize,
    voz: bool,
) -> Result<ensayo::VistaDelEnsayo, ensayo::NoEmpezo> {
    // **Excluyente con una reunión** (§6.5): con una sesión abierta —escuchando, en solo notas o
    // cerrándose— no se ensaya. Una videollamada abierta sin sesión no lo impide: no hay nada que oír.
    let escuchando = app.state::<LaEscucha>().0.lock().map(|g| g.is_some()).unwrap_or(true);
    let cuaderno = app.try_state::<reunion::ElCuaderno>().is_some_and(|c| c.abierta());
    if escuchando || cuaderno {
        return Err(ensayo::NoEmpezo::EnReunion);
    }
    let el_tuyo = el_tuyo(&app);
    let (prop, ficha, nombre) = {
        let g = el_corpus.0.lock().map_err(|_| ensayo::NoEmpezo::SinCorpus)?;
        let c = g.as_ref().ok_or(ensayo::NoEmpezo::SinCorpus)?;
        if !corpus::clientes(c.documentos()).contains(&cliente) {
            return Err(ensayo::NoEmpezo::SinCorpus);
        }
        let propuesta = propuesta.filter(|r| c.propuestas_de(&cliente).iter().any(|d| &d.ruta == r));
        secciones_del_ensayo(c, &cliente, propuesta.as_deref())
    };
    if prop.is_empty() && ficha.is_empty() {
        return Err(ensayo::NoEmpezo::SinCorpus);
    }
    let idioma = ensayo::idioma_del_ensayo(&prop, &ficha, el_tuyo);
    let preguntas = ensayo::banco::armar(&prop, &ficha, idioma, tope_valido(tope));
    // Con quién y cuándo: lo que nombra el ensayo si lo guardas (ADR 015, enmienda 4).
    let rotulo = ensayo::Rotulo {
        cliente,
        propuesta: if prop.is_empty() { String::new() } else { nombre.clone() },
        empezo: reunion::fecha_de_ahora(),
    };
    // **La puerta local se cierra** (§6.5): hay un micrófono abierto, como en reunión.
    cerrar_la_puerta_al_empezar(&app);
    // El informe de un ensayo anterior sin guardar se suelta: uno a la vez.
    soltar_el_ensayo(&app);
    let jerga = diccionario_de_la_sesion(&ruta_del_diccionario(&app), &clientes_del_corpus(el_corpus.inner()));
    let oido = ensayo::oido::Oido::del_microfono(codigo_de(idioma), stt::motor_de_la_casa(), jerga).map_err(|e| {
        println!("[ensayo] el micrófono no se abrió: {e}");
        let e = escucha::con_su_permiso(capture::Pista::Microfono, e, &permisos::leer());
        ensayo::NoEmpezo::Microfono { porque: e.porque }
    })?;
    let enriquecer = app.state::<LaSintesis>().enriquecer.load(Ordering::Relaxed);
    let banco = if enriquecer { ensayo::EstadoDelBanco::EnCamino } else { ensayo::EstadoDelBanco::Apagado };
    let n = preguntas.len();
    let id = ENSAYOS.fetch_add(1, Ordering::SeqCst) + 1;
    let e = ensayo::Ensayo::arrancar(preguntas, idioma, voz, rotulo, banco, oido, std::sync::Arc::new(MundoDeLaApp(app.clone())))
        .ok_or(ensayo::NoEmpezo::SinCorpus)?;
    // Metadata, jamás contenido: ni el cliente, ni la propuesta, ni las preguntas.
    println!(
        "[ensayo] empieza: {n} preguntas · {} · voz {} · enriquecer {}",
        codigo_de(idioma),
        if voz { "sí" } else { "no" },
        if enriquecer { "sí" } else { "no" }
    );
    let vista = e.vista();
    if let Ok(mut g) = app.state::<ElEnsayo>().0.lock() {
        *g = Some((id, e));
    }
    if enriquecer {
        enriquecer_el_ensayo(&app, id, prop, ficha, idioma, nombre);
    }
    let _ = app.emit_to(ventana::PRINCIPAL, EVENTO_ENSAYO, ());
    vista.ok_or(ensayo::NoEmpezo::SinCorpus)
}

/// **El acento del modelo** (ADR 019 §3), una vez por ensayo y en su propio hilo: el ensayo ya empezó con
/// el banco por reglas y no espera a nadie. Por el adaptador de siempre —`mock` → el API si lo
/// encendiste → el modelo del sistema—, con la bóveda, «Ver lo que salió», el contador de red y el tope
/// del mes, que el proveedor del API ya aplica por dentro.
fn enriquecer_el_ensayo(
    app: &tauri::AppHandle,
    id: u64,
    propuesta: Vec<corpus::seccion::Seccion>,
    ficha: Vec<corpus::seccion::Seccion>,
    idioma: ensayo::banco::Idioma,
    sobre: String,
) {
    let mango = app.clone();
    std::thread::spawn(move || {
        let s = mango.state::<LaSintesis>();
        let conocidos = clientes_del_corpus(&mango.state::<ElCorpus>());
        let peticion = ensayo::enriquecer::Peticion::nueva(&propuesta, &ficha, idioma);
        let resultado = match (peticion, proveedor_de_ahora(&s, conocidos, sobre)) {
            (None, _) => Err(ensayo::enriquecer::PorQueNo::NadaFundado),
            (_, None) => Err(ensayo::enriquecer::PorQueNo::SinProveedor),
            (Some(peticion), Some(proveedor)) => {
                let quien = proveedor.quien();
                let r = ensayo::enriquecer::enriquecer(proveedor, &peticion, sintesis::TECHO);
                if quien == sintesis::Quien::Api {
                    cobrar(&mango, &r.respuesta);
                    // Pasado el techo, el proveedor todavía puede contestar —y cobrar—: se espera aparte
                    // para sumarlo al tope del mes, sin enseñar nada (como la sugerencia, M10 del S2).
                    if let Some(tarde) = r.tarde {
                        let mango = mango.clone();
                        std::thread::spawn(move || {
                            if let Ok(Ok(mut respuesta)) = tarde.recv() {
                                cobrar(&mango, &respuesta);
                                // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
                                unsafe { respuesta.json.as_mut_vec() }.fill(0);
                                println!("[ensayo] el banco llegó pasado el techo: cobrado, no usado");
                            }
                        });
                    }
                }
                // Metadata, jamás contenido: quién, cuánto tardó, cuánto salió y cuántas sobrevivieron.
                let fuera = red::formatear(r.respuesta.bytes_fuera);
                match &r.enriquecido {
                    Ok(e) => println!(
                        "[ensayo] el banco: {quien:?} · {} ms · {fuera} fuera · {} sumadas, {} descartadas",
                        r.ms,
                        e.preguntas.len(),
                        e.descartadas
                    ),
                    Err(porque) => println!("[ensayo] el banco: {quien:?} · {} ms · {fuera} fuera · no se enriqueció: {porque:?}", r.ms),
                }
                r.enriquecido
            }
        };
        if let Ok(g) = mango.state::<ElEnsayo>().0.lock() {
            match g.as_ref() {
                Some((suyo, e)) if *suyo == id => e.con_lo_del_modelo(resultado),
                _ => println!("[ensayo] lo del modelo llegó para un ensayo que ya no está: se tira"),
            }
        }
        avisar_a_la_ia(&mango);
    });
}

fn con_el_ensayo(app: &tauri::AppHandle, f: impl FnOnce(&ensayo::Ensayo)) {
    if let Ok(g) = app.state::<ElEnsayo>().0.lock() {
        if let Some((_, e)) = g.as_ref() {
            f(e);
        }
    }
}

/// Enter: deja de leer · cierra la respuesta · la siguiente.
#[tauri::command]
fn ensayo_listo(app: tauri::AppHandle) {
    con_el_ensayo(&app, ensayo::Ensayo::listo);
}

/// R: vuelve a hacer la pregunta.
#[tauri::command]
fn ensayo_repetir(app: tauri::AppHandle) {
    con_el_ensayo(&app, ensayo::Ensayo::repetir);
}

/// S: salta la pregunta.
#[tauri::command]
fn ensayo_saltar(app: tauri::AppHandle) {
    con_el_ensayo(&app, ensayo::Ensayo::saltar);
}

/// Esc: termina el ensayo; el micrófono se cierra y queda el informe.
#[tauri::command]
fn ensayo_terminar(app: tauri::AppHandle) {
    con_el_ensayo(&app, ensayo::Ensayo::terminar);
}

/// «Sí lo dije» sobre una ficha de «Tenías y no usaste».
#[tauri::command]
fn ensayo_si_lo_dije(app: tauri::AppHandle, indice: usize) {
    con_el_ensayo(&app, |e| e.si_lo_dije(indice));
}

/// Lo que la pantalla Ensayo enseña. Solo para la ventana principal (`capabilities/default.json`).
#[tauri::command]
fn estado_del_ensayo(app: tauri::AppHandle) -> Option<ensayo::VistaDelEnsayo> {
    app.state::<ElEnsayo>().0.lock().ok()?.as_ref()?.1.vista()
}

/// «Cerrar sin guardar»: no queda nada.
#[tauri::command]
fn cerrar_el_ensayo(app: tauri::AppHandle) {
    if soltar_el_ensayo(&app) {
        println!("[ensayo] cerrado sin guardar: no queda nada");
    }
}

/// Lo que se guarda del ensayo terminado, con quién y cuándo. Suelta el candado antes de volver: lo que
/// venga después (el Llavero, el diálogo de guardar) no lo espera dentro.
fn el_ensayo_terminado(app: &tauri::AppHandle) -> Result<(ensayo::guardado::Guardado, ensayo::Rotulo), String> {
    let estado = app.state::<ElEnsayo>();
    let g = estado.0.lock().map_err(|_| "el ensayo no responde".to_string())?;
    let (_, e) = g.as_ref().ok_or("no hay ningún ensayo")?;
    let guardado = e.para_guardar().ok_or("el ensayo no ha terminado")?;
    let rotulo = e.rotulo().ok_or("no hay ningún ensayo")?;
    Ok((guardado, rotulo))
}

/// **«Guardar con tus notas»** (ADR 015, enmienda 4): se cifra con la llave de tus notas, **sin pedir nada**,
/// y vence con su retención. Guardado, el informe se va y la pantalla vuelve a «preparar»; si falla, se
/// queda entero y se puede volver a intentar. Con fecha de vencimiento, la tarea de launchd se pone al día.
#[tauri::command]
fn guardar_el_ensayo(app: tauri::AppHandle) -> Result<(), String> {
    let (guardado, rotulo) = el_ensayo_terminado(&app)?;
    let vence = reunion::preferencias(&app).retencion.segundos().map_or(0, |s| carpeta::ahora() + s);
    let base = notas::nombre_del_archivo(Some(&rotulo.cliente), &rotulo.empezo);
    let hecho = reunion::los_ensayos_de(&app).guardar(&carpeta::DelLlavero, &guardado, &base, vence).inspect_err(|e| {
        println!("[ensayo] no se pudo guardar: {}", reunion::sin_ruta(e));
    })?;
    println!("{}", ensayos::linea_de_log(&guardado, &hecho));
    soltar_el_ensayo(&app);
    reunion::poner_al_dia_el_vencimiento(&app);
    Ok(())
}

/// **«Exportar como texto»** el ensayo terminado: pide el desbloqueo de tus notas y después dónde. `false` si
/// cancelaste el diálogo. Síncrono a propósito, como `exportar_reunion`: esperar al usuario no congela la
/// ventana. El informe se queda: exportar no es guardar.
#[tauri::command]
fn exportar_el_ensayo(app: tauri::AppHandle, idioma: String) -> Result<bool, String> {
    use tauri_plugin_dialog::DialogExt;
    let (guardado, rotulo) = el_ensayo_terminado(&app)?;
    reunion::desbloquear(&app, desbloqueo::razon_de_los_ensayos(&idioma))?;
    let sugerido = format!("{}.md", notas::nombre_del_archivo(Some(&rotulo.cliente), &rotulo.empezo));
    let Some(destino) = app.dialog().file().set_file_name(&sugerido).blocking_save_file() else {
        return Ok(false);
    };
    let destino = destino.into_path().map_err(|e| format!("ese destino no es una ruta: {e}"))?;
    ensayos::exportar(&guardado, &destino, &idioma)?;
    println!("[ensayo] exportado a texto, sin cifrado");
    Ok(true)
}

/// **«Tu progreso con este cliente»**: abre tus ensayos de ese cliente con el desbloqueo de tus notas y
/// devuelve **solo las cifras**; tus respuestas no cruzan al webview.
#[tauri::command]
fn progreso_del_ensayo(app: tauri::AppHandle, cliente: String, idioma: String) -> Result<ensayo::guardado::Progreso, String> {
    reunion::desbloquear(&app, desbloqueo::razon_de_los_ensayos(&idioma))?;
    Ok(reunion::los_ensayos_de(&app).progreso(&carpeta::DelLlavero, &cliente))
}

/// «Borrar los ensayos de este cliente»: al momento y sin abrirlos. La tarea de launchd se pone al día.
#[tauri::command]
fn borrar_los_ensayos(app: tauri::AppHandle, cliente: String) -> Result<usize, String> {
    let n = reunion::los_ensayos_de(&app).borrar_del_cliente(&cliente).map_err(|e| reunion::sin_ruta(&e))?;
    println!("[ensayo] {n} ensayo(s) borrado(s) a mano");
    reunion::poner_al_dia_el_vencimiento(&app);
    Ok(n)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(atender_el_atajo).build())
        .invoke_handler(tauri::generate_handler![
            ajustar_banda,
            asentar_banda,
            estado_del_acople,
            la_franja,
            fijar_posicion_de_la_banda,
            enriquecer_el_banco,
            entendido_el_aviso_de_arriba,
            fondo_del_relleno,
            reunion_abierta,
            permisos_de_macos,
            abrir_ajustes_de,
            bytes_a_la_red,
            cortar_todo,
            empezar_a_escuchar,
            dejar_de_escuchar,
            estado_de_la_escucha,
            turnos_recientes,
            que_sabe_transcribir,
            estado_del_diccionario,
            instalar_idioma,
            salida_de_audio,
            elegir_carpeta,
            indexar_corpus,
            estado_del_corpus,
            piezas_del_corte,
            pedir_ficha,
            estado_de_la_voz,
            estado_de_la_pantalla,
            lectura_automatica,
            leer_la_pantalla_ahora,
            radar_de_tu_mac,
            abrir_lo_que_ve,
            estado_de_la_ia,
            redactar_sugerencias,
            api_externa,
            guardar_clave_del_api,
            borrar_clave_del_api,
            idiomas_de_pista,
            fijar_idioma_de_pista,
            lo_que_salio_al_api,
            cuaderno_de_la_reunion,
            escribir_nota,
            anotar_acuerdo,
            conservar_mis_turnos,
            guardar_la_reunion,
            cerrar_sin_guardar,
            reuniones_guardadas,
            exportar_reunion,
            borrar_reunion,
            fijar_retencion,
            mostrar_las_notas_en_finder,
            este_cliente,
            elegir_cliente,
            responder_nda,
            revisar_nda,
            empezar_solo_notas,
            ir_a_notas,
            guardar_propuesta,
            descartar_propuesta,
            fijar_ventana,
            la_bandeja,
            abrir_la_bandeja,
            decidir_en_la_bandeja,
            decidir_toda_la_bandeja,
            cambiar_la_ventana,
            estado_de_la_bandeja,
            la_puerta,
            abrir_la_puerta,
            cerrar_la_puerta,
            preparar_el_ensayo,
            empezar_el_ensayo,
            ensayo_listo,
            ensayo_repetir,
            ensayo_saltar,
            ensayo_terminar,
            ensayo_si_lo_dije,
            estado_del_ensayo,
            cerrar_el_ensayo,
            guardar_el_ensayo,
            exportar_el_ensayo,
            progreso_del_ensayo,
            borrar_los_ensayos
        ])
        .setup(|app| {
            // El invariante se comprueba ANTES de abrir nada y aborta el arranque si falla:
            // una banda que se abre sin su promesa es peor que una banda que no se abre.
            let declaradas = app.config().app.windows.clone();
            ventana::invariante_de_proteccion(&ventana::proteccion_declarada(&declaradas))
                .map_err(|e| format!("protección de captura: {e}"))?;

            // `setup` corre en el hilo principal, que es donde `NSScreen` se deja preguntar.
            app.manage(FondoDelRelleno(acople::fondo_de_escritorio()));
            app.manage(LaEscucha::default());
            app.manage(ElEnsayo::default());
            app.manage(ElCorpus::default());
            app.manage(LaPantalla::default());
            app.manage(ElRadar::default());
            app.manage(LaSintesis::default());
            // Lo que el usuario eligió la vez anterior (ADR 002, enmienda 2): antes de que ninguna
            // pantalla pregunte, para que ninguna enseñe los valores de fábrica un instante.
            {
                let ruta = ruta_de_las_preferencias(app.handle());
                let p = prefs::leer(&ruta);
                aplicar_las_preferencias(app.handle(), &p);
                let carpeta_del_corpus = p.carpeta_del_corpus.clone();
                app.manage(LasPreferencias { ruta, actuales: std::sync::Mutex::new(p) });
                if let Some(carpeta) = carpeta_del_corpus {
                    reindexar_al_arrancar(app.handle(), carpeta);
                }
            }
            // La puerta local nace cerrada en cada arranque (ADR 018 §5): no se recuerda. Si la vez
            // anterior la app se cayó con ella abierta, quedan su socket y su token: se borran.
            app.manage(LaPuerta::default());
            puerta::socket::limpiar_lo_que_quedo(&reunion::carpeta_de_la_app(app.handle()), &puerta::DelLlavero);
            // El cuaderno de la reunión (C9, ADR 015), y el barrido de lo que venció.
            app.manage(reunion::ElCuaderno::default());
            reunion::arrancar_el_barrido(app.handle());
            {
                let gasto = cargar_el_gasto(app.handle());
                println!("[sintesis] gasto del mes {}: USD {:.3}", gasto.mes, gasto.usd);
                if let Ok(mut g) = app.state::<LaSintesis>().mes.lock() {
                    *g = gasto;
                }
            }
            // La voz se pregunta al sistema aquí, una vez, y se deja dicho lo que hay: un Mac sin
            // voz para el idioma del usuario no puede usar el modo solo audio, y eso tiene que
            // verse en el arranque y no cuando el usuario pulse la tecla en mitad de una reunión.
            app.manage(LaVozQueSale::default());
            {
                let v = app.state::<LaVozQueSale>();
                println!(
                    "[habla] voz «{}» · ¿hay para es-ES? {} · ¿para en-US? {}",
                    v.voz.nombre(),
                    v.voz.hay_para("es-ES"),
                    v.voz.hay_para("en-US")
                );
            }

            registrar_el_kill_switch(app.handle());
            arrancar_el_radar(app.handle());

            // El diccionario del consultor: se deja escrito con la semilla si no existía, para que
            // el usuario pueda ir a editarlo. Que falle no impide arrancar — la app funciona sin
            // diccionario y la sesión cae a la semilla— pero se dice.
            if let Err(e) = asegurar_el_diccionario(&ruta_del_diccionario(app.handle())) {
                println!("[diccionario] {e}");
            }

            ventana::abrir_banda(app.handle(), borde_de(app.handle()), ventana::ALTO_COMPACTA)?;

            let mango = app.handle().clone();
            std::thread::spawn(move || {
                // El registro va CON RETRASO a propósito. macOS aplica el tamaño y la posición de
                // una ventana en el siguiente turno del hilo principal, así que leerlos justo
                // después de pedirlos devuelve los valores de la configuración, no los aplicados:
                // medido, decía «1440x88 en (15,242)» cuando la ventana acabó en «1470x88 en
                // (0,868)». Un registro que miente es peor que no tenerlo — se usa para decidir.
                std::thread::sleep(std::time::Duration::from_millis(600));
                ventana::registrar_geometria(&mango);
                registrar_lo_que_ve();
                arrancar_el_acople(&mango);
                acoplar_cuando_haya_a_quien(&mango);
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error al arrancar Angel Ghost");

    app.run(|mango, evento| {
        // La red de seguridad del cierre normal. Cerrar la banda ya suelta, pero salir de la app
        // por el menú, por ⌘Q o cerrando la última ventana NO pasa por ahí — y dejar la ventana
        // de una reunión encogida al salir es exactamente el fallo que este módulo existe para
        // no cometer. La huella cubre el caso que ni esto alcanza: la caída.
        if let tauri::RunEvent::Exit = evento {
            // Salir con los grifos abiertos dejaría el tap del sistema vivo en Core Audio hasta
            // que macOS lo recogiera. El `Drop` del grifo lo cierra; lo que hace falta es que
            // alguien suelte la escucha, y aquí es donde se sabe que ya no habrá otra ocasión.
            if let Some(e) = mango.state::<LaEscucha>().0.lock().ok().and_then(|mut g| g.take()) {
                e.cortar();
                println!("[escucha] cerrada al salir");
            }
            parar_la_pantalla(mango);
            registrar_acople(mango, "soltar al salir", &acople::soltar(&huella(mango)));
            // Lo tuyo sin guardar, se guarda: perderlo por salir es peor (ADR 015 §7).
            reunion::al_salir(mango);
            // La puerta se cierra al salir: su token sale del Llavero y su socket del disco.
            mango.state::<LaPuerta>().0.cerrar(puerta::Cierre::ATuMano);
        }
    });
}

/// Ya se acopló solo una vez en esta sesión. El acople automático es **una sola vez**: a partir
/// de ahí manda el usuario (y, desde la fase 2, la detección de la reunión).
static YA_SE_ACOPLO: AtomicBool = AtomicBool::new(false);

/// Lo primero al arrancar: **devolver lo que quedó de la vez anterior**.
///
/// Si la sesión pasada terminó en una caída, hay una ventana ajena encogida por nuestra culpa, y
/// deshacerlo va antes que cualquier otra cosa — antes incluso de saber si vamos a acoplar algo
/// hoy. Si no hay permiso, se pide una vez: macOS abre su propio diálogo y lleva a Ajustes del
/// Sistema; la respuesta real llega cuando el usuario vuelve, y hasta entonces la banda flota y
/// lo dice.
fn arrancar_el_acople<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let ruta = huella(app);
    let pendiente = acople::leer(&ruta);
    if !pendiente.huellas.is_empty() {
        println!(
            "[acople] la sesión anterior dejó {} ventana(s) encogida(s): se devuelven",
            pendiente.huellas.len()
        );
        registrar_acople(app, "devolver tras una caída", &acople::soltar(&ruta));
    }

    if !acople::hay_permiso() {
        println!("[acople] sin permiso de Accesibilidad: la banda flota. Se pide una vez.");
        acople::pedir_permiso();
    }
}

/// Deja en el log lo que la app VE del Mac al arrancar: qué videollamada hay y qué permisos
/// tiene. Es lo que vuelve contestable la pregunta «¿lo viste correr?» para dos módulos cuya
/// respuesta depende por completo de la máquina, y que por tanto ningún test puede afirmar.
///
/// **Metadatos y nada más**: el cliente de videollamada («Google Meet») y los cuatro estados de
/// permiso. **El título de la reunión no se escribe** — es información del cliente, vive en
/// memoria mientras la pantalla lo muestra, y un log es un archivo.
fn registrar_lo_que_ve() {
    let p = permisos::leer();
    println!(
        "[permisos] micrófono={:?} audio={:?} pantalla={:?} accesibilidad={:?} · cara={:?}",
        p.microfono,
        p.audio,
        p.pantalla,
        p.accesibilidad,
        permisos::cara(&p)
    );
    match sesion::ahora() {
        sesion::Reunion::Detectada { cliente, proteccion, .. } => println!(
            "[sesion] reunión detectada: «{cliente}» · protección {proteccion:?} · catálogo {}",
            sesion::VERSION_CATALOGO
        ),
        sesion::Reunion::Ninguna => println!("[sesion] ninguna videollamada del catálogo abierta"),
        sesion::Reunion::NoSePuedeSaber { motivo } => println!("[sesion] no se puede saber: {motivo:?}"),
    }
    println!("[red] salida acumulada: {}", red::formatear(red::bytes()));

    // El motor de transcripción y la salida de audio: las dos cosas de la fase 3 que dependen por
    // completo de la máquina y que ningún test puede afirmar.
    let que = que_sabe_transcribir();
    println!(
        "[stt] motor «{}» · {} idiomas soportados · techo {}{}",
        que.motor,
        que.idiomas.len(),
        que.techo,
        que.motivo.map(|m| format!(" · {}", m.en_el_log())).unwrap_or_default()
    );
    let listos: Vec<&str> = que
        .idiomas
        .iter()
        .filter(|i| matches!(i.disponibilidad, stt::Disponibilidad::Listo))
        .map(|i| i.codigo.as_str())
        .collect();
    println!("[stt] modelos instalados: {}", if listos.is_empty() { "ninguno".into() } else { listos.join(", ") });
    let salida = capture::nativo::salida_de_audio();
    match salida.puede_haber_eco() {
        Some(true) => println!("[audio] {salida:?} · el micrófono va a oír al cliente: se marcará el eco"),
        Some(false) => println!("[audio] {salida:?} · las dos pistas quedan limpias"),
        None => println!("[audio] {salida:?} · no se puede saber si habrá eco"),
    }
}

/// `⌥⎋` — la tecla del kill-switch, tal y como la dibuja la maqueta.
fn el_atajo() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::ALT), Code::Escape)
}

/// `⌃⌥T` — enseñar u ocultar el transcript, tal y como lo dibuja `idioma.html`.
///
/// Hasta la mirada 17-quater del sprint 002 era `⌘⇧T`, y chocaba dos veces: en los navegadores
/// reabre la última pestaña cerrada —y el navegador es donde vive la reunión de Meet— y en Zoom
/// pausa la pantalla compartida. El usuario pasó TODAS las teclas de la app a `⌃⌥` (Control +
/// Opción), que ni Zoom, ni Meet, ni Teams documentan. La que queda es VoiceOver, cuyas órdenes
/// empiezan por `⌃⌥`: lo dice el manual.
fn el_atajo_del_transcript() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyT)
}

/// `⌃⌥A` — «ayúdame con esto», el atajo que la maqueta dibuja en la banda.
///
/// Es la salida cuando el disparador automático no acierta, y por eso su camino es distinto: se
/// salta la espera entre fichas y la regla de no repetir. Si el usuario lo pulsa dos veces
/// seguidas es porque la primera no le sirvió, y contestarle con silencio sería lo peor posible
/// justo en el momento en que decidió pedir ayuda a mano.
fn el_atajo_de_ayuda() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyA)
}

/// El nombre del evento con el que la banda se entera de que hay que enseñar u ocultar el
/// transcript.
const EVENTO_TRANSCRIPT: &str = "transcript";

/// El nombre del evento con el que la banda recibe una ficha.
const EVENTO_FICHA: &str = "ficha";

/// La ficha a petición del usuario: `⌃⌥A`, o el botón de la banda ampliada.
///
/// Busca con **el último turno del cliente**, que es de lo que se estaba hablando. Sin turnos no
/// hay con qué buscar, y eso se dice en vez de devolver una ficha vacía.
#[tauri::command]
fn pedir_ficha(
    app: tauri::AppHandle,
    escucha_viva: tauri::State<'_, LaEscucha>,
    el_corpus: tauri::State<'_, ElCorpus>,
    la_pantalla: tauri::State<'_, LaPantalla>,
) -> Option<ficha::Aparicion> {
    // El cuerpo vive en `ficha_vigente` desde el sprint 002: `⌃⌥V` necesita **la misma** ficha para
    // decirla que esta enseña, y dos búsquedas escritas por separado acabarían encontrando cosas
    // distintas para la misma pregunta.
    //
    // **«Todavía no he oído nada» es una respuesta, no un error** (corrida en vivo de la fase 3 del
    // sprint 002). Devuelto como `Err`, la promesa del webview se rechazaba sin que nadie la
    // atendiera y la banda se quedaba en «Buscando en tu corpus…» para siempre: `⌃⌥A` pulsada antes
    // de que el cliente hablara dejaba la banda colgada. Ningún test lo vio porque su doble del
    // puente solo sabía resolver.
    let a = ficha_vigente(&escucha_viva, &el_corpus, &la_pantalla).or_else(|| ficha_de_la_nota(&app, &el_corpus));
    match &a {
        None => println!("[ficha] ⌃⌥A sin turno del cliente: todavía no hay nada que buscar"),
        Some(a) => {
            sintetizar(&app, a);
            // La ficha de ⌃⌥A también se fija con ⌃⌥P: es la que la banda enseña ahora.
            reunion::ver(&app, a);
        }
    }
    a
}

// ---------------------------------------------------------------------------------------------
// Fase 2 del sprint 002 — EL MODO SOLO AUDIO (C15): la voz que sale
// ---------------------------------------------------------------------------------------------

/// **EL MODO SOLO AUDIO, vivo mientras la app esté abierta.**
///
/// Nació en la mirada 3 de la etapa de diseño, con estas palabras del usuario: *«quisiera tener un
/// modo solo audio que me hable de forma paralela por si quiero ver completamente la pantalla y no
/// me interrumpa»*. Las dos mitades son los dos requisitos, y las dos están aquí: **hablar** (la voz
/// del sistema) y **devolver la pantalla** (la banda baja a 44 px).
///
/// El estado es mínimo a propósito — un interruptor, una voz y un idioma— porque la ficha **no se
/// guarda**. Se dice en el momento en que aparece y se suelta; guardarla habría obligado a añadir
/// una pieza más al kill-switch para volver a vaciarla, y la que sí hay que añadir es otra: la voz
/// tiene que **callarse** cuando el usuario pulsa `⌥⎋` delante de su cliente.
struct LaVozQueSale {
    voz: Box<dyn habla::Voz>,
    encendida: AtomicBool,
    /// En qué idioma se lee. **El del consultor**, no el del cliente: la ficha sale de los
    /// documentos del usuario, así que está escrita en su idioma. Lo fija `empezar_a_escuchar`; hasta
    /// entonces vale el de la interfaz por defecto.
    idioma: std::sync::Mutex<String>,
}

impl Default for LaVozQueSale {
    fn default() -> Self {
        Self {
            voz: habla::voz(),
            encendida: AtomicBool::new(false),
            idioma: std::sync::Mutex::new("es-ES".into()),
        }
    }
}

impl LaVozQueSale {
    fn idioma(&self) -> String {
        self.idioma.lock().map(|i| i.clone()).unwrap_or_else(|_| "es-ES".into())
    }

    /// Lo que la banda enseña. Se pregunta al sistema por dónde sale el sonido **cada vez**: los
    /// auriculares se conectan y se quitan en mitad de una reunión, que es justo cuando importa.
    fn estado(&self) -> habla::LaVoz {
        let idioma = self.idioma();
        habla::LaVoz {
            encendida: self.encendida.load(Ordering::Relaxed),
            puede: habla::puede_en_principio(
                &capture::nativo::salida_de_audio(),
                self.voz.hay_para(&idioma),
            ),
            diciendo: self.voz.hablando(),
        }
    }
}

/// El nombre del evento con el que la banda se entera de cómo está la voz.
const EVENTO_VOZ: &str = "voz";

/// **Dice la ficha en voz alta, si cabe decirla.**
///
/// Es el único camino por el que la app habla, y lo usan los dos disparadores: el atajo `⌃⌥V` —que
/// lee la ficha vigente al encender el modo— y la aparición automática al final de un turno del
/// cliente. Tener un solo camino es lo que hace que las cinco razones para callarse valgan para los
/// dos: dos copias de esta decisión acabarían callándose por motivos distintos.
///
/// Al log va **el hecho, jamás la ficha**: cuántas letras y por qué se calló. El titular es
/// contenido del corpus del usuario y un log es un archivo.
fn decir_la_ficha<R: tauri::Runtime>(app: &tauri::AppHandle<R>, a: &ficha::Aparicion) {
    let ficha::Respuesta::Ficha(f) = &a.respuesta else {
        // Una «sin resultado» no se lee. No es una decisión de ahorro: su titular son **las
        // palabras del cliente** («nada sobre "certificación ISO"»), y leérselas al usuario sería
        // sacar el transcript del cliente por el altavoz. La banda la pinta; la voz se calla.
        return;
    };
    let estado = app.state::<LaVozQueSale>();
    let idioma = estado.idioma();

    // `try_lock` y no `lock`, y la razón es un abrazo mortal real: este camino se llama DESDE los
    // hilos de la escucha, y `empezar_a_escuchar` tiene el candado de `LaEscucha` cogido mientras
    // los arranca. La ventana es de milisegundos y hace falta una ficha para entrar en ella, así que
    // no ocurriría casi nunca — «casi nunca» es exactamente la clase de fallo que aparece en una
    // reunión. Si el candado está ocupado se supone que **sí** hay alguien hablando, que es el lado
    // que calla.
    let alguien_hablando = match app.state::<LaEscucha>().0.try_lock() {
        Ok(g) => g
            .as_ref()
            .map(|e| {
                let s = e.estado();
                s.microfono.hablando || s.sistema.hablando
            })
            .unwrap_or(false),
        Err(_) => true,
    };

    let salida = capture::nativo::salida_de_audio();
    let momento = habla::Momento {
        modo_encendido: estado.encendida.load(Ordering::Relaxed),
        salida: &salida,
        hay_voz: estado.voz.hay_para(&idioma),
        alguien_hablando,
        ya_diciendo: estado.voz.hablando(),
    };
    if let Err(impedimento) = habla::cabe_decirla(&momento) {
        // El modo apagado es el estado normal de la app: decirlo en cada ficha llenaría el log de
        // una línea por turno sin informar de nada.
        if impedimento != habla::Impedimento::ModoApagado {
            println!("[habla] no se dice la ficha: {} · salida {salida:?}", impedimento.como_frase());
            let _ = app.emit_to(ventana::BANDA, EVENTO_VOZ, estado.estado());
        }
        return;
    }

    let dicho = habla::a_voz(&f.titular, &f.linea, &fuente_hablada(&f.fuente));
    match estado.voz.decir(&idioma, &dicho) {
        Ok(()) => {
            println!("[habla] diciendo la ficha · {} letras · {idioma}", dicho.chars().count());
            // **El presupuesto «la voz empieza ≤ 1 s tras la ficha», medido en vivo** (auditoría del
            // S2, B21): lo que tardó el sintetizador en sonar de verdad, al log. Solo el número.
            std::thread::spawn(|| {
                for _ in 0..100 {
                    if let Some(ms) = habla::apple::ms_hasta_sonar() {
                        println!("[habla] empezó a sonar a los {ms} ms (presupuesto 1000)");
                        return;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                println!("[habla] no empezó a sonar en 2 s");
            });
        }
        Err(e) => println!("[habla] el sintetizador no pudo: {e}"),
    }
    let _ = app.emit_to(ventana::BANDA, EVENTO_VOZ, estado.estado());
}

/// La fuente, tal y como se DICE — que no es como se escribe.
///
/// La banda pinta «propuesta · §3.2 Alcance», y el interpunto y el `§` leídos en voz alta son un
/// ruido: `AVSpeechSynthesizer` dice «párrafo tres punto dos» o se los come, según la voz. Se
/// cambian por palabras. Es el único sitio de la app donde la voz y la pantalla dicen lo mismo con
/// letras distintas, y por eso está aquí y no en el módulo protegido: es copy hablado.
fn fuente_hablada(f: &ficha::Fuente) -> String {
    let seccion = f.seccion.as_deref().unwrap_or("").replace('§', "");
    if seccion.trim().is_empty() {
        f.documento.clone()
    } else {
        format!("{}, {}", f.documento, seccion.trim())
    }
}

/// El trabajo de `⌃⌥V`, **hecho en Rust y no pedido a la banda por un evento**.
///
/// `⌃⌥T` y `⌃⌥A` emiten a la banda y la banda actúa, y aquí eso no sirve: este cambia el
/// ALTO de la ventana, y el `⌥⎋` puede haberla cerrado. Un modo que se enciende solo si queda una
/// banda que lo pida no es un modo, es una casualidad.
fn conmutar_el_modo<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> habla::LaVoz {
    let estado = app.state::<LaVozQueSale>();
    let el_corpus = app.state::<ElCorpus>();
    let escucha_viva = app.state::<LaEscucha>();
    // Apagar: la banda vuelve a su alto y el modo se apaga por la misma vía que el asa.
    if estado.encendida.load(Ordering::Relaxed) {
        let alto = ventana::ALTO_COMPACTA;
        if let Err(e) = asentar_banda((*app).clone(), alto) {
            println!("[habla] la banda no pudo ir a {alto} px: {e}");
        }
        println!("[habla] ⌃⌥V: modo solo audio APAGADO · banda a {alto} px");
        return apagar_el_modo(app);
    }
    // Sin voz para el idioma, el modo NO se enciende: bajar la banda a 44 px y quedarse con `⎋`
    // para no decir nada —y con «Conecta auriculares», que es el motivo equivocado— era peor que
    // no encenderlo (auditoría del S2, B14). El manual lo prometía así.
    let idioma = estado.idioma();
    if !puede_encender_el_modo(estado.voz.hay_para(&idioma)) {
        println!("[habla] sin voz para {idioma}: el modo no se enciende");
        return estado.estado();
    }
    estado.encendida.store(true, Ordering::Relaxed);

    // **El alto de la banda ES el modo.** Lo que el usuario pidió no era una voz: era recuperar la
    // pantalla mientras la app le habla, y eso son 44 px que vuelven a la reunión.
    //
    // Se usa `asentar_banda` y no `ajustar_banda`, que es la diferencia entre mover lo nuestro y
    // mover lo ajeno: `ajustar_banda` deja la banda y su relleno en su sitio —juntos, porque un
    // relleno que se quedara a 88 px dejaría una franja de escritorio a la vista de la captura— y
    // `asentar_banda` además **rehace el acople**, que es lo único que le devuelve esos 44 px a la
    // ventana de la reunión. Sin esa segunda mitad el modo bajaría la banda y el usuario no ganaría
    // un píxel de pantalla, que es justo lo que pidió. Es una llamada por encendido, no por cuadro:
    // el arrastre del asa aprendió lo caro que es hacerlo sesenta veces por segundo.
    let alto = ventana::ALTO_VOZ;
    if let Err(e) = asentar_banda((*app).clone(), alto) {
        println!("[habla] la banda no pudo ir a {alto} px: {e}");
    }

    // `⎋` se registra SOLO mientras el modo está encendido. Un Escape global permanente se lo
    // quitaría a la reunión —en Meet es la tecla de salir de pantalla completa— y a todas las
    // demás apps del Mac, para una función que existe unos segundos por ficha.
    con_el_callar(app, true);
    println!("[habla] ⌃⌥V: modo solo audio ENCENDIDO · banda a {alto} px");
    // La ficha vigente se rearma igual que en `pedir_ficha`: con el último turno del cliente.
    // Si no se ha oído nada todavía, no hay nada que decir y el modo queda encendido, esperando.
    match ficha_vigente(&escucha_viva, &el_corpus, &app.state::<LaPantalla>()) {
        Some(a) => decir_la_ficha(app, &a),
        None => println!("[habla] todavía no he oído nada del cliente: el modo queda a la espera"),
    }

    let ahora = estado.estado();
    let _ = app.emit_to(ventana::BANDA, EVENTO_VOZ, ahora);
    ahora
}

/// Cómo está la voz ahora mismo. Lo pregunta la banda al montarse; después escucha el evento.
#[tauri::command]
fn estado_de_la_voz(estado: tauri::State<'_, LaVozQueSale>) -> habla::LaVoz {
    estado.estado()
}

/// **La ficha vigente, rearmada con el último turno del cliente.**
///
/// Es el cuerpo que `pedir_ficha` tenía dentro, sacado para que lo compartan los dos que lo
/// necesitan: el atajo `⌃⌥A`, que la enseña, y el `⌃⌥V`, que la dice. Dos copias de esto acabarían
/// buscando distinto, y entonces la banda y la voz enseñarían fichas diferentes de la misma
/// pregunta — que es la peor manera posible de romper un modo que existe para no tener que mirar.
fn ficha_vigente(
    escucha_viva: &tauri::State<'_, LaEscucha>,
    el_corpus: &tauri::State<'_, ElCorpus>,
    la_pantalla: &LaPantalla,
) -> Option<ficha::Aparicion> {
    use escucha::Buscador;
    let ultimo = escucha_viva.0.lock().ok()?.as_ref().and_then(|e| {
        e.ultimos_turnos(6)
            .into_iter()
            .rev()
            .find(|t| t.pista == capture::Pista::Sistema && !t.eco)
    })?;
    let empezo = std::time::Instant::now();
    // Con la pantalla delante, igual que la ficha automática: `⌃⌥A` y el disparador no pueden
    // buscar distinto la misma pregunta.
    let buscador = ConPantalla {
        corpus: el_corpus.inner().clone(),
        pantalla: refuerzo_vigente(la_pantalla),
    };
    let hallazgos = buscador.buscar(&ultimo.texto, ficha::TOP);
    let respuesta = ficha::armar(&ultimo.texto, &hallazgos);
    let ms = empezo.elapsed().as_millis() as u64;
    println!("[ficha] a petición del usuario en {ms} ms · {} candidatas", hallazgos.len());
    Some(ficha::Aparicion { respuesta, motivo: disparo::Motivo::Atajo, ms, hora: ultimo.hora })
}

/// **`⌃⌥A` en solo notas** (ADR 017 §5): no hay turno del cliente con que buscar, así que busca con la
/// última línea de tu nota. Fuera de solo notas no hace nada: ahí `⌃⌥A` sigue siendo «lo último que
/// dijo el cliente», y sin turno se dice que todavía no hay nada que buscar.
fn ficha_de_la_nota(app: &tauri::AppHandle, el_corpus: &tauri::State<'_, ElCorpus>) -> Option<ficha::Aparicion> {
    use escucha::Buscador;
    if !reunion::solo_notas(app) {
        return None;
    }
    let linea = reunion::ultima_linea_de_la_nota(app)?;
    let empezo = std::time::Instant::now();
    let hallazgos = el_corpus.inner().buscar(&linea, ficha::TOP);
    let respuesta = ficha::armar(&linea, &hallazgos);
    let ms = empezo.elapsed().as_millis() as u64;
    println!("[ficha] ⌃⌥A en solo notas, con tu nota, en {ms} ms · {} candidatas", hallazgos.len());
    Some(ficha::Aparicion { respuesta, motivo: disparo::Motivo::Atajo, ms, hora: escucha::la_hora() })
}

/// `⌃⌥V` — **el modo solo audio**, tal y como lo dibuja la banda de 44 px.
///
/// No es `⌃⌥A`, que es lo que pedía la orden del sprint: `⌃⌥A` ya es «ayúdame con esto» desde el
/// sprint 001 y el panel aprobado en la etapa de diseño ya escribía `⌃⌥V`. Desviación declarada.
fn el_atajo_del_modo_de_voz() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyV)
}

/// `⎋` — **cállate**. Solo está registrada mientras el modo solo audio está encendido.
fn el_atajo_de_callar() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Shortcut};
    Shortcut::new(None, Code::Escape)
}

/// **Coger y soltar `⎋`, SIEMPRE EN OTRO HILO. Y esto no es una precaución: es un cuelgue real.**
///
/// `⌃⌥V` llega por el manejador de atajos globales, y **registrar un atajo desde dentro de ese
/// manejador bloquea el plugin**: se queda esperando un candado que tiene cogido el propio hilo que
/// lo llamó. Lo que se ve desde fuera es peor que un error — la app sigue viva, la ventana responde,
/// y **ningún atajo vuelve a funcionar nunca**. Incluido `⌥⎋`.
///
/// Se encontró corriendo la app, no probándola: los 253 tests de Rust y los 171 del webview estaban
/// verdes, y el registro se cortaba justo entre `[acople] reacople` y la línea siguiente. Es el
/// tercer filo de la regla 15 con nombre y apellido — *¿lo viste correr EN EL MODO en que el usuario
/// lo va a usar?*— y el modo, aquí, es «con el dedo en la tecla».
///
/// Un hilo suelto basta y no hace falta nada más: el manejador vuelve, suelta el candado, y el
/// registro ocurre unos microsegundos después. Nadie espera a nadie.
fn con_el_callar<R: tauri::Runtime>(app: &tauri::AppHandle<R>, coger: bool) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    let mango = app.clone();
    std::thread::spawn(move || {
        let atajo = el_atajo_de_callar();
        let g = mango.global_shortcut();
        if coger {
            // **Si no puede, lo dice**: el usuario pulsaría la tecla creyendo que calló a la app, y
            // la app seguiría hablándole encima del cliente.
            match g.register(atajo) {
                Ok(()) => println!(
                    "[habla] ⎋ registrada MIENTRAS dure el modo · OJO: durante estos segundos la \
                     tecla no le llega a la reunión"
                ),
                Err(e) => println!(
                    "[habla] NO se pudo registrar ⎋ ({e}): para callar la voz hay que apagar el \
                     modo con ⌃⌥V"
                ),
            }
        } else {
            // Que falle no rompe nada —la tecla seguiría cogida— pero se dice, porque a partir de
            // ahí la reunión dejaría de recibir Escapes sin ninguna razón visible.
            match g.unregister(atajo) {
                Ok(()) => println!("[habla] ⎋ devuelta al sistema"),
                Err(e) => {
                    println!("[habla] ⎋ NO se pudo devolver ({e}): la reunión seguirá sin recibirla")
                }
            }
        }
    });
}

// ---------------------------------------------------------------------------------------------
// Fase 3 del sprint 002 — LEER LA PANTALLA SOLO CUANDO CAMBIA (C8)
// ---------------------------------------------------------------------------------------------

/// **LA LECTURA DE PANTALLA**, viva lo que viva la sesión.
///
/// El interruptor (`encendida`) vive fuera de la lectura y **sobrevive a las sesiones**: si el
/// usuario la apagó en una reunión con una NDA estricta, la siguiente sesión no la vuelve a encender
/// a sus espaldas. Desde el sprint 003 **sobrevive también al reinicio** (`prefs.rs`): de fábrica
/// arranca encendida, que es lo que la maqueta dibuja, y después como la dejó el usuario.
struct LaPantalla {
    lectura: std::sync::Mutex<Option<pantalla::Lectura>>,
    encendida: AtomicBool,
}

impl Default for LaPantalla {
    fn default() -> Self {
        Self {
            lectura: std::sync::Mutex::new(None),
            encendida: AtomicBool::new(true),
        }
    }
}

/// El nombre del evento con el que Sesión y Honestidad se enteran de lo que hace la lectura.
const EVENTO_PANTALLA: &str = "pantalla";

/// Arranca la lectura de pantalla de una sesión y devuelve **lo que aporta a la búsqueda**, que el
/// buscador de la escucha lleva dentro.
///
/// La lectura no conoce ni la escucha ni el corpus: recibe funciones. Lo que hace con cada pantalla
/// nueva lo decide [`atender_la_pantalla`], aquí fuera.
fn arrancar_la_pantalla<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    el_corpus: ElCorpus,
) -> std::sync::Arc<std::sync::Mutex<pantalla::Refuerzo>> {
    parar_la_pantalla(app);
    let estado = app.state::<LaPantalla>();
    let (ojo, lector) = pantalla::apple::ojos();
    println!(
        "[pantalla] ojos «{}» · lectura automática {}",
        ojo.nombre(),
        if estado.encendida.load(Ordering::Relaxed) {
            "encendida"
        } else {
            "apagada"
        }
    );
    let (al_leer, al_cambiar, al_avisar) = (app.clone(), app.clone(), app.clone());
    let entorno = pantalla::Entorno {
        ojo,
        lector,
        objetivo: Box::new(|| {
            sesion::objetivo_de(&sesion::mirar())
                .map(|(bundle, senales)| pantalla::Objetivo { bundle, senales })
        }),
        vocabulario: Box::new(move || escucha::Buscador::vocabulario(&el_corpus)),
        al_leer: Box::new(move |r, origen| atender_la_pantalla(&al_leer, r, origen)),
        al_cambiar: Box::new(move |e| {
            println!("[pantalla] {:?}", e.vista);
            let _ = al_cambiar.emit(EVENTO_PANTALLA, e);
        }),
        // El radar ámbar (C14) viaja por el mismo canal que las fichas: la banda lo pinta en el
        // mismo sitio, y lo que llegue después lo sustituye.
        al_avisar: Box::new(move |a| {
            let _ = al_avisar.emit(
                EVENTO_ESCUCHA,
                escucha::Novedad::Radar {
                    grabando: a.grabando,
                    bots: a.bots.clone(),
                    hora: escucha::la_hora(),
                },
            );
        }),
    };
    let lectura = pantalla::Lectura::arrancar(entorno, estado.encendida.load(Ordering::Relaxed));
    let refuerzo = lectura.refuerzo();
    if let Ok(mut g) = estado.lectura.lock() {
        *g = Some(lectura);
    }
    refuerzo
}

/// Para la lectura, si la había. Devuelve si había una.
fn parar_la_pantalla<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> bool {
    let vieja = app
        .state::<LaPantalla>()
        .lectura
        .lock()
        .ok()
        .and_then(|mut g| g.take());
    match vieja {
        Some(l) => {
            l.cortar();
            true
        }
        None => false,
    }
}

/// Lo que la pantalla aporta ahora mismo, o un refuerzo vacío si no hay lectura.
fn refuerzo_vigente(
    la_pantalla: &LaPantalla,
) -> std::sync::Arc<std::sync::Mutex<pantalla::Refuerzo>> {
    la_pantalla
        .lectura
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|l| l.refuerzo()))
        .unwrap_or_default()
}

/// **Una pantalla nueva, y qué se hace con ella.**
///
/// - Si la leyó el vigía solo y trae una cifra o uno de tus términos, se le pide ficha a la escucha
///   —que la pasa por su disparador, con su espera— y **solo si hay ficha** se enseña.
/// - Si la pidió el usuario con su atajo, se responde siempre, como `⌃⌥A`.
///
/// El candado de la escucha se suelta ANTES de hablar y de avisar a la banda: `decir_la_ficha` lo
/// intenta coger por su cuenta, y con él cogido aquí, la voz se callaría la ficha de la pantalla.
fn atender_la_pantalla<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    refuerzo: &pantalla::Refuerzo,
    origen: pantalla::Origen,
) {
    // Para «Muere al cerrar»: una lectura más en esta reunión. Solo el número.
    reunion::contar_una_lectura(app);
    let mut consulta = refuerzo.consulta();
    let aparicion = {
        let escucha = app.state::<LaEscucha>();
        let guardada = escucha.0.lock();
        guardada.ok().and_then(|g| {
            g.as_ref().and_then(|e| match origen {
                pantalla::Origen::Sola if refuerzo.dispara() => e.por_pantalla(&consulta),
                pantalla::Origen::Sola => None,
                pantalla::Origen::Pedida if consulta.trim().is_empty() => None,
                pantalla::Origen::Pedida => e.pedida_por_pantalla(&consulta),
            })
        })
    };
    // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
    unsafe { consulta.as_mut_vec() }.fill(0);
    match aparicion {
        Some(a) => {
            decir_la_ficha(app, &a);
            // **Sin sugerencia**, ni con `⌃⌥L`: una ficha de la pantalla no responde a ninguna
            // pregunta del cliente, y la sugerencia tomaría su último turno, que puede ser de hace
            // minutos y de otra cosa (auditoría del S2, B5).
            reunion::ver(app, &a);
            let _ = app.emit(EVENTO_ESCUCHA, escucha::Novedad::Aparece(Box::new(a)));
        }
        None if origen == pantalla::Origen::Pedida => {
            println!("[pantalla] leída a petición: nada legible que buscar");
            let hora = escucha::la_hora();
            let _ = app.emit(EVENTO_ESCUCHA, escucha::Novedad::NadaEnPantalla { hora });
        }
        None => {}
    }
}

/// Qué hace la lectura de pantalla ahora mismo. Lo piden Sesión y Honestidad al montarse; después
/// escuchan el evento `pantalla`.
#[tauri::command]
fn estado_de_la_pantalla(
    la_pantalla: tauri::State<'_, LaPantalla>,
) -> pantalla::EstadoDeLaPantalla {
    la_pantalla
        .lectura
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|l| l.estado()))
        .unwrap_or(pantalla::EstadoDeLaPantalla {
            vista: if la_pantalla.encendida.load(Ordering::Relaxed) {
                pantalla::Vista::EsperandoLaReunion
            } else {
                pantalla::Vista::Apagada
            },
            bytes_en_memoria: 0,
        })
}

/// **El interruptor de Sesión.** Apagada, la lectura no captura ni un cuadro y olvida lo que había
/// leído; el atajo sigue leyendo cuando el usuario lo pide.
#[tauri::command]
fn lectura_automatica(
    app: tauri::AppHandle,
    la_pantalla: tauri::State<'_, LaPantalla>,
    encendida: bool,
) -> pantalla::EstadoDeLaPantalla {
    la_pantalla.encendida.store(encendida, Ordering::Relaxed);
    if let Ok(g) = la_pantalla.lectura.lock() {
        if let Some(l) = g.as_ref() {
            l.encender(encendida);
        }
    }
    println!(
        "[pantalla] lectura automática {}",
        if encendida { "ENCENDIDA" } else { "APAGADA" }
    );
    recordar(&app, |p| p.lectura_automatica = encendida);
    let estado = estado_de_la_pantalla(la_pantalla);
    let _ = app.emit(EVENTO_PANTALLA, estado);
    estado
}

/// `⌃⌥L` — leer la pantalla una vez, ahora. La tecla que Sesión dibuja al lado de «Leerla sola».
fn el_atajo_de_leer_la_pantalla() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyL)
}

/// **La lectura bajo demanda, sin región** (decisión del usuario, 2026-09-26): lee UNA vez la
/// ventana de la reunión, ahora, aunque la automática esté apagada. Es la salida para una NDA
/// estricta: nada se lee salvo cuando el consultor lo pide.
#[tauri::command]
fn leer_la_pantalla_ahora(la_pantalla: tauri::State<'_, LaPantalla>) -> bool {
    leer_una_vez(&la_pantalla)
}

/// Lo que hacen el comando y la tecla: los dos caminos, una sola decisión.
fn leer_una_vez(la_pantalla: &LaPantalla) -> bool {
    let hay = la_pantalla
        .lectura
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|l| l.leer_ahora()))
        .is_some();
    if !hay {
        println!("[pantalla] lectura pedida sin sesión: no hay reunión que leer");
    }
    hay
}

// ── LAS PREFERENCIAS (ADR 002, enmienda 2) ──────────────────────────────────────────────────────
//
// Lo que se elige una vez y se recuerda: `prefs.rs` dice qué y en qué formato; aquí se decide CUÁNDO
// se aplica (al arrancar, antes de que ninguna pantalla pregunte) y cuándo se guarda (cada vez que el
// usuario cambia algo, no al salir: una caída no se lleva lo que eligió).

struct LasPreferencias {
    ruta: PathBuf,
    actuales: std::sync::Mutex<prefs::Preferencias>,
}

fn ruta_de_las_preferencias<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> PathBuf {
    app.path().app_config_dir().unwrap_or_else(|_| std::env::temp_dir()).join(prefs::ARCHIVO)
}

/// Pone el estado de la app como lo dejó el usuario. **El API no se enciende solo si su clave ya no
/// está**: borrarla en «Acceso a Llaveros» también es una elección, y la de más peso.
fn aplicar_las_preferencias<R: tauri::Runtime>(app: &tauri::AppHandle<R>, p: &prefs::Preferencias) {
    let s = app.state::<LaSintesis>();
    s.redactar.store(p.redactar, Ordering::Relaxed);
    s.enriquecer.store(p.enriquecer_el_banco, Ordering::Relaxed);
    let encendida = p.api_encendida && sintesis::api::hay_clave(p.externo);
    if p.api_encendida && !encendida {
        println!("[prefs] el API estaba encendido pero ya no hay clave de {}: se queda apagado", p.externo.nombre());
    }
    if let Ok(mut a) = s.api.lock() {
        *a = ConfigDelApi { encendida, externo: p.externo };
    }
    app.state::<LaPantalla>().encendida.store(p.lectura_automatica, Ordering::Relaxed);
    println!(
        "[prefs] idiomas {} / {} · redactar {} · API {} ({}) · lectura automática {}",
        p.idiomas.consultor,
        p.idiomas.cliente,
        p.redactar,
        encendida,
        p.externo.nombre(),
        p.lectura_automatica
    );
}

/// Cambia una preferencia y la guarda **ya**. Que no se pueda guardar no deshace la elección —la
/// app sigue con ella hasta cerrarse— pero se dice.
fn recordar<R: tauri::Runtime>(app: &tauri::AppHandle<R>, cambio: impl FnOnce(&mut prefs::Preferencias)) {
    let Some(lp) = app.try_state::<LasPreferencias>() else { return };
    let Ok(mut p) = lp.actuales.lock() else { return };
    cambio(&mut p);
    if let Err(e) = prefs::guardar(&lp.ruta, &p) {
        println!("[prefs] no se pudieron guardar: {e}");
    }
}

/// El idioma de cada pista, como lo dejó el usuario.
fn idiomas_guardados(lp: &LasPreferencias) -> prefs::IdiomasDePista {
    lp.actuales.lock().map(|p| p.idiomas.clone()).unwrap_or_default()
}

/// El idioma de cada pista, para que Idioma lo enseñe. La escucha no lo pide: lo lee de aquí mismo.
#[tauri::command]
fn idiomas_de_pista(lp: tauri::State<'_, LasPreferencias>) -> prefs::IdiomasDePista {
    idiomas_guardados(&lp)
}

/// Idioma elige el de una pista. Solo `consultor` o `cliente`, y solo un código de idioma.
#[tauri::command]
fn fijar_idioma_de_pista(
    app: tauri::AppHandle,
    pista: String,
    idioma: String,
) -> Result<prefs::IdiomasDePista, String> {
    if !prefs::es_un_idioma(&idioma) {
        return Err(format!("«{idioma}» no es un código de idioma"));
    }
    match pista.as_str() {
        "consultor" => recordar(&app, |p| p.idiomas.consultor = idioma.clone()),
        "cliente" => recordar(&app, |p| p.idiomas.cliente = idioma.clone()),
        _ => return Err(format!("no hay pista «{pista}»")),
    }
    println!("[prefs] la pista «{pista}» escucha en {idioma}");
    Ok(idiomas_de_pista(app.state::<LasPreferencias>()))
}

// ── LA SÍNTESIS (C7) ────────────────────────────────────────────────────────────────────────────
//
// ADR 010 «síntesis, código primero» y ADR 011 «proveedores del modelo y minimización». Todo lo de
// aquí es la capa que decide CUÁNDO y CON QUIÉN; lo que el modelo ve y lo que se acepta de él vive en
// `sintesis/`, que es módulo protegido.

/// El tope del mes para el proveedor externo (ADR 011). Al llegar, la app vuelve sola a lo local.
const TOPE_DEL_MES_USD: f64 = 10.0;

/// El archivo donde persiste **la cifra** del gasto del mes —no el texto de nada—. Es un metadato de
/// costo, que la regla 1 de la casa permite guardar; nace con permisos de solo su dueño.
const COSTO_DEL_MES: &str = "costo-del-mes.json";

const EVENTO_IA: &str = "ia";

#[derive(Debug, Clone, Copy)]
struct ConfigDelApi {
    encendida: bool,
    externo: sintesis::api::Externo,
}

impl Default for ConfigDelApi {
    fn default() -> Self {
        Self { encendida: false, externo: sintesis::api::Externo::Claude }
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct GastoDelMes {
    mes: String,
    usd: f64,
}

#[derive(Default)]
struct LaSintesis {
    /// «Redactar sugerencias». **Nace apagado**: la app es entera sin ello (ADR 010).
    redactar: AtomicBool,
    /// «Enriquecer el banco» del ensayo (ADR 019 §3). **Nace apagado**: el ensayo es entero sin ello.
    enriquecer: AtomicBool,
    api: std::sync::Mutex<ConfigDelApi>,
    reunion_usd: std::sync::Mutex<f64>,
    mes: std::sync::Mutex<GastoDelMes>,
    /// Lo que tardaron las sugerencias de esta sesión, para la mediana que enseña IA.
    latencias: std::sync::Mutex<Vec<u64>>,
    /// Sube con cada corte: una sugerencia que vuelva de antes del corte se tira sin enseñarla, y
    /// una petición al API que no haya salido todavía ya no sale (la comparte con el adaptador).
    epoca: std::sync::Arc<std::sync::atomic::AtomicU64>,
    /// Una a la vez: si llega otra ficha mientras se redacta, esa se queda sin sugerencia.
    en_marcha: AtomicBool,
    /// Lo que salió al API en esta reunión, para que IA lo enseñe (auditoría del S2, B37). Solo en
    /// memoria: lo vacían el corte, el final de la sesión y la sesión siguiente.
    registro: sintesis::api::Registro,
}

/// **Lo que el corte le hace a la síntesis** (la pieza `Sugerencia`): la época sube —lo que vuelva
/// de antes se tira y lo que no salió ya no sale—, la reunión deja de sumar costo, y lo que salió al
/// API se olvida (B37). Fuera del bucle del corte para poder probarlo sin la app entera.
fn cortar_la_sugerencia(s: &LaSintesis) {
    s.epoca.fetch_add(1, Ordering::SeqCst);
    if let Ok(mut u) = s.reunion_usd.lock() {
        *u = 0.0;
    };
    s.registro.vaciar();
}

/// Lo que la pantalla IA enseña.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDeLaIa {
    redactar: bool,
    /// «Enriquecer el banco» del ensayo (sprint 004).
    enriquecer: bool,
    /// Quién redactaría ahora mismo. `None`: nadie puede.
    quien: Option<sintesis::Quien>,
    /// Por qué el modelo del sistema no puede. `None`: puede.
    sistema: Option<sintesis::PorQueNoRedacta>,
    api: EstadoDelApi,
    /// La mediana de las sugerencias de esta sesión, en milisegundos.
    latencia_ms: Option<u64>,
    reunion_usd: f64,
    mes_usd: f64,
    tope_usd: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDelApi {
    encendida: bool,
    externo: sintesis::api::Externo,
    hay_clave: bool,
}

fn ruta_del_costo<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> PathBuf {
    app.path().app_config_dir().unwrap_or_else(|_| std::env::temp_dir()).join(COSTO_DEL_MES)
}

/// «2026-09», en la hora del Mac. Sin traer una biblioteca de fechas para un año y un mes.
fn mes_de_hoy() -> String {
    // SEGURIDAD: `time` y `localtime_r` escriben en estructuras que viven en esta función.
    unsafe {
        let ahora = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&ahora, &mut tm);
        format!("{}-{:02}", tm.tm_year + 1900, tm.tm_mon + 1)
    }
}

/// El gasto del mes guardado; si es de otro mes, empieza en cero.
fn cargar_el_gasto<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> GastoDelMes {
    let hoy = mes_de_hoy();
    std::fs::read_to_string(ruta_del_costo(app))
        .ok()
        .and_then(|t| serde_json::from_str::<GastoDelMes>(&t).ok())
        .filter(|g| g.mes == hoy)
        .unwrap_or(GastoDelMes { mes: hoy, usd: 0.0 })
}

/// **El mes que cambia con la app abierta** (auditoría del S2, B3): el gasto de septiembre no puede
/// seguir bloqueando el API en octubre. Se pone a cero antes de LEER la cifra, no solo al sumar.
fn gasto_vigente(g: &mut GastoDelMes, hoy: &str) {
    if g.mes != hoy {
        *g = GastoDelMes { mes: hoy.to_string(), usd: 0.0 };
    }
}

/// Se escribe a un temporal —que nace cerrado— y se RENOMBRA encima: una caída entre borrar y crear
/// dejaba el mes en 0 y el tope sin efecto (auditoría del S2, M10). El renombrado es atómico.
fn guardar_el_gasto<R: tauri::Runtime>(app: &tauri::AppHandle<R>, gasto: &GastoDelMes) {
    if let Err(e) = escribir_el_gasto(&ruta_del_costo(app), gasto) {
        println!("[sintesis] no se pudo guardar el gasto del mes: {e}");
    }
}

fn escribir_el_gasto(ruta: &std::path::Path, gasto: &GastoDelMes) -> Result<(), String> {
    almacen::escribir(ruta, serde_json::to_string(gasto).unwrap_or_default().as_bytes())
}

/// Los clientes del corpus, por su nombre: lo que la bóveda tapa antes de que nada salga (ADR 011).
fn clientes_del_corpus(el_corpus: &ElCorpus) -> Vec<String> {
    el_corpus
        .0
        .lock()
        .ok()
        .and_then(|g| {
            g.as_ref().map(|c| corpus::clientes(c.documentos()))
        })
        .unwrap_or_default()
}

/// **Quién redacta ahora.** El que el usuario encendió manda: el API si lo encendió, tiene clave y
/// no llegó al tope; si no, el modelo del sistema si está. `AG_SINTESIS=mock` fuerza el `mock`
/// —el de la CI y el del kit—, dentro del adapter y no interceptando nada (ADR 011).
fn proveedor_de_ahora(
    s: &LaSintesis,
    conocidos: Vec<String>,
    sobre: String,
) -> Option<std::sync::Arc<dyn sintesis::Proveedor>> {
    use sintesis::Proveedor;
    if std::env::var("AG_SINTESIS").as_deref() == Ok("mock") {
        return Some(std::sync::Arc::new(sintesis::mock::Mock));
    }
    let api = *s.api.lock().ok()?;
    let bajo_el_tope = s
        .mes
        .lock()
        .map(|mut g| {
            gasto_vigente(&mut g, &mes_de_hoy());
            g.usd < TOPE_DEL_MES_USD
        })
        .unwrap_or(false);
    if api.encendida && bajo_el_tope && sintesis::api::hay_clave(api.externo) {
        return Some(std::sync::Arc::new(sintesis::api::Api {
            externo: api.externo,
            conocidos,
            registro: s.registro.clone(),
            sobre,
            vigencia: sintesis::api::Vigencia::desde_ahora(&s.epoca),
        }));
    }
    let sistema = sintesis::sistema::DelSistema;
    sistema.disponible().is_ok().then(|| std::sync::Arc::new(sistema) as std::sync::Arc<dyn sintesis::Proveedor>)
}

fn estado_de_la_ia_de<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> EstadoDeLaIa {
    use sintesis::Proveedor;
    let s = app.state::<LaSintesis>();
    let api = s.api.lock().map(|a| *a).unwrap_or_default();
    let hay_clave = sintesis::api::hay_clave(api.externo);
    let sistema = sintesis::sistema::DelSistema.disponible().err();
    let mes_usd = s
        .mes
        .lock()
        .map(|mut g| {
            gasto_vigente(&mut g, &mes_de_hoy());
            g.usd
        })
        .unwrap_or(0.0);
    let quien = if std::env::var("AG_SINTESIS").as_deref() == Ok("mock") {
        Some(sintesis::Quien::Mock)
    } else if api.encendida && hay_clave && mes_usd < TOPE_DEL_MES_USD {
        Some(sintesis::Quien::Api)
    } else if sistema.is_none() {
        Some(sintesis::Quien::Sistema)
    } else {
        None
    };
    let latencia_ms = s.latencias.lock().ok().and_then(|l| {
        let mut l = l.clone();
        l.sort_unstable();
        l.get(l.len() / 2).copied()
    });
    EstadoDeLaIa {
        redactar: s.redactar.load(Ordering::Relaxed),
        enriquecer: s.enriquecer.load(Ordering::Relaxed),
        quien,
        sistema,
        api: EstadoDelApi { encendida: api.encendida, externo: api.externo, hay_clave },
        latencia_ms,
        reunion_usd: s.reunion_usd.lock().map(|u| *u).unwrap_or(0.0),
        mes_usd,
        tope_usd: TOPE_DEL_MES_USD,
    }
}

fn avisar_a_la_ia<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> EstadoDeLaIa {
    let estado = estado_de_la_ia_de(app);
    let _ = app.emit(EVENTO_IA, &estado);
    estado
}

/// **Una ficha nueva, y quizá su sugerencia.** La ficha ya está en la banda cuando esto empieza:
/// la sugerencia llega después, debajo, o no llega. Nada de aquí la retrasa.
fn sintetizar<R: tauri::Runtime>(app: &tauri::AppHandle<R>, a: &ficha::Aparicion) {
    let s = app.state::<LaSintesis>();
    if !s.redactar.load(Ordering::Relaxed) {
        return;
    }
    let ficha::Respuesta::Ficha(f) = &a.respuesta else { return };
    if f.respaldo.is_empty() {
        return;
    }
    // Una ficha que trajo la PANTALLA no responde a ninguna pregunta del cliente: la sugerencia
    // tomaría el último turno que hubiera, que puede ser de hace diez minutos y de otra cosa.
    if a.motivo == disparo::Motivo::Pantalla {
        return;
    }
    if s.en_marcha.swap(true, Ordering::Relaxed) {
        println!("[sintesis] ya se redacta otra: esta ficha se queda sin sugerencia");
        return;
    }
    let respaldo = f.respaldo.clone();
    let epoca = s.epoca.load(Ordering::Relaxed);
    let mango = app.clone();
    std::thread::spawn(move || {
        let s = mango.state::<LaSintesis>();
        let turno = mango
            .state::<LaEscucha>()
            .0
            .lock()
            .ok()
            .and_then(|g| g.as_ref().map(|e| e.ultimos_turnos(12)))
            .and_then(|ts| ts.into_iter().rev().find(|t| t.pista == capture::Pista::Sistema && !t.eco))
            .map(|t| t.texto);
        let conocidos = clientes_del_corpus(&mango.state::<ElCorpus>());
        let sobre = respaldo.first().map(|r| r.titular.clone()).unwrap_or_default();
        let (Some(mut turno), Some(proveedor)) = (turno, proveedor_de_ahora(&s, conocidos, sobre)) else {
            s.en_marcha.store(false, Ordering::Relaxed);
            return;
        };
        let peticion = sintesis::Peticion::nueva(&turno, &respaldo);
        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
        unsafe { turno.as_mut_vec() }.fill(0);
        let Some(peticion) = peticion else {
            s.en_marcha.store(false, Ordering::Relaxed);
            return;
        };
        let quien = proveedor.quien();
        let r = sintesis::sugerir(proveedor, &peticion, sintesis::TECHO);
        drop(peticion);
        if quien == sintesis::Quien::Api {
            cobrar(&mango, &r.respuesta);
            // Pasado el techo, el proveedor todavía puede contestar —y cobrar—: se espera aparte
            // para sumar su costo al tope del mes, sin enseñar nada (auditoría del S2, M10).
            if let Some(tarde) = r.tarde {
                let mango = mango.clone();
                std::thread::spawn(move || {
                    if let Ok(Ok(mut respuesta)) = tarde.recv() {
                        cobrar(&mango, &respuesta);
                        // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
                        unsafe { respuesta.json.as_mut_vec() }.fill(0);
                        println!("[sintesis] llegó pasado el techo: cobrada, no enseñada");
                    }
                });
            }
        }
        if let Ok(mut l) = s.latencias.lock() {
            l.push(r.ms);
        }
        // Metadata, jamás contenido: quién, cuánto tardó, cuánto salió y si se aceptó.
        let fuera = red::formatear(r.respuesta.bytes_fuera);
        if s.epoca.load(Ordering::Relaxed) != epoca {
            println!("[sintesis] llegó después del corte: se tira sin enseñarla");
        } else {
            match r.sugerencia {
                Ok(sugerencia) => {
                    println!("[sintesis] {quien:?} · {} ms · {fuera} fuera · confianza {:?}", r.ms, sugerencia.confianza());
                    let _ = mango.emit(EVENTO_ESCUCHA, escucha::Novedad::Sugerencia(Box::new(sugerencia)));
                }
                Err(d) => println!("[sintesis] {quien:?} · {} ms · {fuera} fuera · descartada: {d:?}", r.ms),
            }
        }
        s.en_marcha.store(false, Ordering::Relaxed);
        avisar_a_la_ia(&mango);
    });
}

/// Suma lo que costó una respuesta del API a la reunión y al mes, y guarda el mes.
fn cobrar<R: tauri::Runtime>(app: &tauri::AppHandle<R>, respuesta: &sintesis::Respuesta) {
    if respuesta.tokens_entrada + respuesta.tokens_salida == 0 {
        return;
    }
    let s = app.state::<LaSintesis>();
    let externo = s.api.lock().map(|a| a.externo).unwrap_or(sintesis::api::Externo::Claude);
    let usd = externo.costo(respuesta.tokens_entrada, respuesta.tokens_salida);
    if let Ok(mut u) = s.reunion_usd.lock() {
        *u += usd;
    }
    if let Some(id) = respuesta.salida {
        s.registro.cobrar(id, usd);
    }
    let gasto = s.mes.lock().ok().map(|mut g| {
        gasto_vigente(&mut g, &mes_de_hoy());
        g.usd += usd;
        g.clone()
    });
    if let Some(g) = gasto {
        guardar_el_gasto(app, &g);
    }
}

#[tauri::command]
fn estado_de_la_ia(app: tauri::AppHandle) -> EstadoDeLaIa {
    estado_de_la_ia_de(&app)
}

/// **Lo que salió al API en esta reunión, texto incluido** (B37). Por comando y solo para la
/// ventana principal: el evento `ia` avisa de que hay algo nuevo, pero el texto no viaja a todas las
/// ventanas (el precedente es B11: el turno del cliente dejó de cruzar en los eventos).
#[tauri::command]
fn lo_que_salio_al_api(app: tauri::AppHandle) -> Vec<sintesis::api::LoQueSalio> {
    app.state::<LaSintesis>().registro.todas()
}

/// «Redactar sugerencias (además de mostrar la ficha)».
#[tauri::command]
fn redactar_sugerencias(app: tauri::AppHandle, si: bool) -> EstadoDeLaIa {
    app.state::<LaSintesis>().redactar.store(si, Ordering::Relaxed);
    println!("[sintesis] redactar sugerencias: {}", if si { "encendido" } else { "apagado" });
    recordar(&app, |p| p.redactar = si);
    avisar_a_la_ia(&app)
}

/// «Enriquecer el banco» (ADR 019 §3): el segundo interruptor de IA, junto a «Redactar sugerencias».
#[tauri::command]
fn enriquecer_el_banco(app: tauri::AppHandle, si: bool) -> EstadoDeLaIa {
    app.state::<LaSintesis>().enriquecer.store(si, Ordering::Relaxed);
    println!("[ensayo] enriquecer el banco: {}", if si { "encendido" } else { "apagado" });
    recordar(&app, |p| p.enriquecer_el_banco = si);
    avisar_a_la_ia(&app)
}

/// El proveedor externo: encenderlo exige su clave en el Llavero.
#[tauri::command]
fn api_externa(
    app: tauri::AppHandle,
    encendida: bool,
    externo: sintesis::api::Externo,
) -> Result<EstadoDeLaIa, String> {
    if encendida && !sintesis::api::hay_clave(externo) {
        return Err("sin-clave".into());
    }
    if let Ok(mut a) = app.state::<LaSintesis>().api.lock() {
        *a = ConfigDelApi { encendida, externo };
    }
    println!("[sintesis] API externo {} · {}", if encendida { "encendido" } else { "apagado" }, externo.nombre());
    recordar(&app, |p| {
        p.api_encendida = encendida;
        p.externo = externo;
    });
    Ok(avisar_a_la_ia(&app))
}

#[tauri::command]
fn guardar_clave_del_api(
    app: tauri::AppHandle,
    externo: sintesis::api::Externo,
    clave: String,
) -> Result<EstadoDeLaIa, String> {
    let mut clave = clave;
    let r = sintesis::api::guardar_clave(externo, &clave);
    // SEGURIDAD: ceros sobre UTF-8 válido siguen siendo UTF-8 válido.
    unsafe { clave.as_mut_vec() }.fill(0);
    r?;
    println!("[sintesis] clave de {} guardada en el Llavero", externo.nombre());
    Ok(avisar_a_la_ia(&app))
}

#[tauri::command]
fn borrar_clave_del_api(app: tauri::AppHandle, externo: sintesis::api::Externo) -> Result<EstadoDeLaIa, String> {
    sintesis::api::borrar_clave(externo)?;
    let se_apago = app
        .state::<LaSintesis>()
        .api
        .lock()
        .map(|mut a| apagar_si_usaba(&mut a, externo))
        .unwrap_or(false);
    // Y se recuerda apagado: si no, al guardar otra clave y reiniciar, el API se encendería solo
    // (casilla 6 del S3).
    if se_apago {
        recordar(&app, |p| p.api_encendida = false);
    }
    println!("[sintesis] clave de {} borrada del Llavero", externo.nombre());
    Ok(avisar_a_la_ia(&app))
}

/// Sin su clave, el proveedor que estaba encendido se apaga. Devuelve si se apagó, para recordarlo.
fn apagar_si_usaba(api: &mut ConfigDelApi, externo: sintesis::api::Externo) -> bool {
    if api.externo == externo && api.encendida {
        api.encendida = false;
        return true;
    }
    false
}

// ── EL RADAR (C14) ──────────────────────────────────────────────────────────────────────────────

/// Lo último que el radar coral vio en este Mac. Sesión lo pide al montarse; después escucha el
/// evento `radar`.
#[derive(Default)]
struct ElRadar(std::sync::Mutex<Option<radar::EnTuMac>>);

const EVENTO_RADAR: &str = "radar";

/// Cada cuánto se mira la lista de procesos. Una vuelta cuesta unos milisegundos (dos llamadas al
/// núcleo por proceso); diez segundos es lo bastante a menudo para que un programa que se abre en
/// mitad de la reunión se vea antes de que importe, y lo bastante poco para no notarse.
const RADAR_CADA: std::time::Duration = std::time::Duration::from_secs(10);

/// **El radar coral, en marcha desde que arranca la app** —no desde que empieza la sesión—: la
/// pantalla de Sesión tiene que poder decir ANTES de empezar que algo vigila este Mac, que es
/// cuando el usuario todavía puede decidir no empezar.
///
/// Avisa solo cuando cambia lo que hay. La primera vuelta avisa siempre, aunque no haya nada: así
/// Sesión y la banda saben que el radar miró, y no confunden «nada» con «todavía no».
fn arrancar_el_radar<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let mango = app.clone();
    std::thread::spawn(move || {
        let mdm = radar::mdm::inscrito();
        println!(
            "[radar] catálogo v{} · MDM: {}",
            radar::catalogo::rotulo().version,
            match mdm {
                Some(true) => "inscrito",
                Some(false) => "no",
                None => "no se pudo preguntar",
            }
        );
        let mdm = mdm.unwrap_or(false);
        let mut antes: Option<Vec<String>> = None;
        loop {
            let visto = radar::mirar_tu_mac(mdm);
            let nombres = visto.nombres();
            if antes.as_ref() != Some(&nombres) {
                // Cuántos y de qué nivel. Son programas de TU Mac, no de nadie más, pero el log
                // sigue siendo metadata: los nombres los enseña Sesión.
                let invasivos = visto.programas.iter().filter(|p| p.nivel == radar::Nivel::Invasivo).count();
                println!(
                    "[radar] en tu Mac: {invasivos} invasivos · {} sábelo",
                    visto.programas.len() - invasivos
                );
                if let Ok(mut g) = mango.state::<ElRadar>().0.lock() {
                    *g = Some(visto.clone());
                }
                let _ = mango.emit(EVENTO_RADAR, &visto);
                antes = Some(nombres);
            }
            std::thread::sleep(RADAR_CADA);
        }
    });
}

/// Lo que el radar vio en este Mac. Si el hilo todavía no dio su primera vuelta, se mira ahora
/// —sin preguntar por el MDM, que cuesta lanzar un programa y lo trae la vuelta siguiente—.
#[tauri::command]
fn radar_de_tu_mac(estado: tauri::State<'_, ElRadar>) -> radar::EnTuMac {
    estado
        .0
        .lock()
        .ok()
        .and_then(|g| g.clone())
        .unwrap_or_else(|| radar::mirar_tu_mac(false))
}

/// `⌃⌥R` — «qué ve» (mirada 17). Abre el cuaderno en Sesión, donde está la tabla de lo que
/// alcanza a ver cada programa, con su catálogo.
fn el_atajo_del_radar() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyR)
}

// ---------------------------------------------------------------------------------------------
// Sprint 003, fase 1 — TUS NOTAS (C9, ADR 015). La lógica vive en `reunion.rs`; aquí, los comandos
// que la pantalla de Notas llama, cada uno de una línea.
// ---------------------------------------------------------------------------------------------

/// El cuaderno de la reunión de ahora, para Notas (durante · al cerrar).
#[tauri::command]
fn cuaderno_de_la_reunion(app: tauri::AppHandle) -> Option<reunion::VistaDelCuaderno> {
    reunion::vista(&app)
}

/// Tu nota entera, tal como está en el campo.
#[tauri::command]
fn escribir_nota(app: tauri::AppHandle, texto: String) {
    reunion::escribir(&app, &texto);
}

/// Un acuerdo que escribiste y confirmaste con ↵.
#[tauri::command]
fn anotar_acuerdo(app: tauri::AppHandle, texto: String) -> bool {
    reunion::acordar(&app, &texto)
}

/// «Conservar mis turnos», que además se recuerda para las reuniones siguientes.
#[tauri::command]
fn conservar_mis_turnos(app: tauri::AppHandle, si: bool) {
    reunion::conservar_mis_turnos(&app, si);
}

/// «Guardar cifrado y cerrar». No devuelve lo guardado: la pantalla no lo leía (auditoría del S3, B13;
/// casilla 5). `carpeta::Guardada` sigue para la línea del log.
#[tauri::command]
fn guardar_la_reunion(app: tauri::AppHandle) -> Result<(), String> {
    reunion::guardar(&app).map(|_| ())
}

/// «Cerrar sin guardar».
#[tauri::command]
fn cerrar_sin_guardar(app: tauri::AppHandle) {
    reunion::descartar(&app);
}

/// Las reuniones guardadas, sin abrir ninguna: la lista solo lee cabeceras.
#[tauri::command]
fn reuniones_guardadas(app: tauri::AppHandle) -> reunion::ListaDeReuniones {
    reunion::lista(&app)
}

/// «Exportar sin cifrado»: pide el desbloqueo y después dónde. `false` si cancelaste el diálogo.
/// Síncrono a propósito, como `elegir_carpeta`: corre en el pool de Tauri y esperar al usuario no
/// congela la ventana.
#[tauri::command]
fn exportar_reunion(app: tauri::AppHandle, archivo: String, idioma: String) -> Result<bool, String> {
    reunion::exportar(&app, &archivo, &idioma)
}

/// «Borrar ahora».
#[tauri::command]
fn borrar_reunion(app: tauri::AppHandle, archivo: String) -> Result<(), String> {
    reunion::borrar(&app, &archivo)
}

/// «Cuánto viven tus notas».
#[tauri::command]
fn fijar_retencion(app: tauri::AppHandle, retencion: prefs::Retencion) {
    reunion::fijar_retencion(&app, retencion);
}

/// «Mostrar en Finder»: la carpeta de tus notas, o esa reunión seleccionada dentro de ella.
#[tauri::command]
fn mostrar_las_notas_en_finder(app: tauri::AppHandle, archivo: Option<String>) -> Result<(), String> {
    reunion::mostrar_en_finder(&app, archivo.as_deref())
}

/// «Anotar para después» en la banda: lo mismo que `⌃⌥N`.
#[tauri::command]
fn ir_a_notas(app: tauri::AppHandle) {
    reunion::ir_a_notas(&app);
}

// ---------------------------------------------------------------------------------------------
// Sprint 003, fase 3 — EL MARCO EN LA MANO: «Este cliente», su NDA y solo notas (ADR 017).
// ---------------------------------------------------------------------------------------------

/// «Este cliente», como Sesión lo pinta: los clientes del corpus, el elegido, su bandera, su NDA y la
/// cláusula modelo.
fn vista_del_cliente(app: &tauri::AppHandle, el_corpus: &ElCorpus) -> jurisdiccion::VistaDelCliente {
    let elegido = reunion::cliente(app);
    let (clientes, escrita) = el_corpus
        .0
        .lock()
        .ok()
        .and_then(|g| {
            g.as_ref().map(|c| {
                let escrita = elegido.as_deref().and_then(|e| c.jurisdiccion_de(e)).map(str::to_string);
                (corpus::clientes(c.documentos()), escrita)
            })
        })
        .unwrap_or_default();
    let nda = elegido
        .as_ref()
        .and_then(|e| app.state::<LasPreferencias>().actuales.lock().ok().and_then(|p| p.ndas.get(e).copied()))
        .unwrap_or(jurisdiccion::Nda::SinRevisar);
    jurisdiccion::VistaDelCliente {
        clientes,
        bandera: elegido.as_ref().map(|_| jurisdiccion::bandera(escrita.as_deref())),
        elegido,
        nda,
        clausula: jurisdiccion::clausula().clone(),
    }
}

#[tauri::command]
fn este_cliente(app: tauri::AppHandle, el_corpus: tauri::State<'_, ElCorpus>) -> jurisdiccion::VistaDelCliente {
    vista_del_cliente(&app, &el_corpus)
}

/// Elige el cliente de esta reunión, o ninguno. Solo uno del corpus: un nombre que no está ahí no se
/// elige (lo que llega de la pantalla no inventa clientes).
#[tauri::command]
fn elegir_cliente(
    app: tauri::AppHandle,
    el_corpus: tauri::State<'_, ElCorpus>,
    nombre: Option<String>,
) -> Result<jurisdiccion::VistaDelCliente, String> {
    if let Some(n) = &nombre {
        if !clientes_del_corpus(&el_corpus).contains(n) {
            return Err("ese cliente no está en tu corpus".into());
        }
    }
    reunion::elegir_cliente(&app, nombre);
    Ok(vista_del_cliente(&app, &el_corpus))
}

/// La respuesta al chequeo de NDA del cliente elegido: «Sí, lo prohíbe» o «No lo prohíbe». Se guarda.
#[tauri::command]
fn responder_nda(
    app: tauri::AppHandle,
    el_corpus: tauri::State<'_, ElCorpus>,
    prohibe: bool,
) -> Result<jurisdiccion::VistaDelCliente, String> {
    let elegido = reunion::cliente(&app).ok_or("elige primero el cliente")?;
    let nda = if prohibe { jurisdiccion::Nda::LoProhibe } else { jurisdiccion::Nda::NoLoProhibe };
    recordar(&app, |p| {
        p.ndas.insert(elegido, nda);
    });
    // Al log, la respuesta; el nombre del cliente, no.
    println!("[nda] respuesta guardada: {}", if prohibe { "lo prohíbe" } else { "no lo prohíbe" });
    Ok(vista_del_cliente(&app, &el_corpus))
}

/// «Revisar» y «Volver a revisar la NDA»: la respuesta se borra y la pregunta vuelve.
#[tauri::command]
fn revisar_nda(app: tauri::AppHandle, el_corpus: tauri::State<'_, ElCorpus>) -> jurisdiccion::VistaDelCliente {
    if let Some(elegido) = reunion::cliente(&app) {
        recordar(&app, |p| {
            p.ndas.remove(&elegido);
        });
        println!("[nda] respuesta borrada: vuelve la pregunta");
    }
    vista_del_cliente(&app, &el_corpus)
}

// ---------------------------------------------------------------------------------------------
// Sprint 003, fase 2 — LAS PROPUESTAS Y LA BANDEJA (ADR 016). La lógica vive en `reunion.rs`.
// ---------------------------------------------------------------------------------------------

/// «Guardar» una propuesta en Notas, durante la reunión.
#[tauri::command]
fn guardar_propuesta(app: tauri::AppHandle, id: u32) -> bool {
    reunion::guardar_propuesta(&app, id)
}

/// «No»: la propuesta muere en ese momento.
#[tauri::command]
fn descartar_propuesta(app: tauri::AppHandle, id: u32) -> bool {
    reunion::descartar_propuesta(&app, id)
}

/// La ventana de la bandeja, elegida en «al cerrar».
#[tauri::command]
fn fijar_ventana(app: tauri::AppHandle, ventana: bandeja::Ventana) {
    reunion::fijar_ventana(&app, ventana);
}

/// La bandeja que vence antes, abierta o cerrada con llave.
#[tauri::command]
fn la_bandeja(app: tauri::AppHandle) -> Option<reunion::VistaDeLaBandeja> {
    reunion::la_bandeja(&app)
}

/// «Abrir la bandeja»: pide el desbloqueo, como abrir una reunión. Síncrono a propósito, como
/// `exportar_reunion`: esperar a Touch ID no congela la ventana.
#[tauri::command]
fn abrir_la_bandeja(app: tauri::AppHandle, archivo: String, idioma: String) -> Result<(), String> {
    reunion::abrir_la_bandeja(&app, &archivo, &idioma)
}

/// «Guardar» o «No» sobre una propuesta de la bandeja.
#[tauri::command]
fn decidir_en_la_bandeja(app: tauri::AppHandle, archivo: String, indice: usize, guardar: bool) -> Result<(), String> {
    reunion::decidir_en_la_bandeja(&app, &archivo, indice, guardar)
}

/// «Guardar todas» o «Descartar todas».
#[tauri::command]
fn decidir_toda_la_bandeja(app: tauri::AppHandle, archivo: String, guardar: bool) -> Result<(), String> {
    reunion::decidir_toda_la_bandeja(&app, &archivo, guardar)
}

/// La ventana, cambiada desde la bandeja: la vuelve a sellar.
#[tauri::command]
fn cambiar_la_ventana(app: tauri::AppHandle, archivo: String, ventana: bandeja::Ventana) -> Result<(), String> {
    reunion::cambiar_la_ventana(&app, &archivo, ventana)
}

/// Lo que Honestidad dice de la bandeja, sin abrirla.
#[tauri::command]
fn estado_de_la_bandeja(app: tauri::AppHandle) -> reunion::EstadoDeLaBandeja {
    reunion::estado_de_la_bandeja(&app)
}

// ── LA PUERTA LOCAL PARA TU AGENTE (C16, ADR 018) ───────────────────────────────────────────────
//
// La política, el socket y la línea de órdenes viven en `puerta/`, módulo protegido. Aquí está lo que
// la puerta puede hacer con la app —la misma búsqueda, el mismo reindexado, las mismas preferencias
// y el mismo desbloqueo que usa la pantalla— y los tres comandos de IA, solo de la ventana principal.

/// La puerta. `Arc` porque el hilo que atiende la lleva consigo. Y **su propio desbloqueo**, que se
/// olvida al abrirla y al cerrarla: Touch ID una vez por apertura, nunca heredado del de la pantalla
/// (auditoría del S3, M4; decisión del usuario, 2026-09-28: «por apertura»).
#[derive(Default)]
struct LaPuerta(std::sync::Arc<puerta::socket::Puerta>, desbloqueo::Desbloqueo);

/// El nombre del evento con que IA se entera de que la puerta cambió. Solo a la ventana principal.
const EVENTO_PUERTA: &str = "puerta";

/// Lo que la puerta ve de la app.
struct LaAppParaLaPuerta(tauri::AppHandle);

impl puerta::Operaciones for LaAppParaLaPuerta {
    fn en_reunion(&self) -> bool {
        en_reunion(&self.0)
    }
    fn hacer(&self, orden: &puerta::Orden) -> Result<puerta::Hecho, String> {
        hacer_por_la_puerta(&self.0, orden)
    }
    fn hora(&self) -> String {
        hora_de_ahora()
    }
    fn avisar(&self) {
        let _ = self.0.emit_to(ventana::PRINCIPAL, EVENTO_PUERTA, vista_de_la_puerta(&self.0));
    }
}

/// **¿Hay reunión?** Escuchando, una reunión en solo notas, una videollamada detectada **o no se
/// puede saber** (ADR 018 §5): lo que no se sabe se trata como reunión. Un candado envenenado, también.
/// **Y un ensayo con el micrófono abierto** (ADR 019 §6.5): para la puerta cuenta igual, porque hay
/// un micrófono abierto.
fn en_reunion<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> bool {
    let escuchando = app.state::<LaEscucha>().0.lock().map(|g| g.is_some()).unwrap_or(true);
    let cuaderno = app.try_state::<reunion::ElCuaderno>().is_some_and(|c| c.abierta());
    escuchando || cuaderno || ensayando(app) || !matches!(sesion::ahora(), sesion::Reunion::Ninguna)
}

/// «11:04», la hora del Mac.
fn hora_de_ahora() -> String {
    // SEGURIDAD: `time` y `localtime_r` escriben en estructuras que viven en esta función.
    unsafe {
        let ahora = libc::time(std::ptr::null_mut());
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&ahora, &mut tm);
        format!("{:02}:{:02}", tm.tm_hour, tm.tm_min)
    }
}

const SIN_CORPUS: &str = "no hay corpus indexado en esta sesión de la app: señala tu carpeta en Corpus";

fn a_json<T: serde::Serialize>(v: &T) -> Result<serde_json::Value, String> {
    serde_json::to_value(v).map_err(|e| e.to_string())
}

/// Hace una orden que la política ya dejó pasar. Lo mismo que hace la pantalla, por el mismo camino.
fn hacer_por_la_puerta(app: &tauri::AppHandle, orden: &puerta::Orden) -> Result<puerta::Hecho, String> {
    use puerta::{Hecho, Orden};
    let el_corpus = app.state::<ElCorpus>();
    match orden {
        Orden::Buscar { texto } => {
            let g = el_corpus.0.lock().map_err(|_| "el corpus quedó en mal estado")?;
            let c = g.as_ref().ok_or(SIN_CORPUS)?;
            let respuesta = ficha::armar(texto, &c.buscar(texto, ficha::TOP)?);
            let cuenta = match &respuesta {
                ficha::Respuesta::Ficha(f) => 1 + f.acumuladas.len() as u32,
                ficha::Respuesta::SinResultado { .. } => 0,
            };
            Ok(Hecho { datos: a_json(&respuesta)?, cuenta: Some(cuenta) })
        }
        Orden::Reindexar {} => {
            let carpeta = el_corpus
                .0
                .lock()
                .map_err(|_| "el corpus quedó en mal estado")?
                .as_ref()
                .and_then(|c| c.estado().carpeta)
                .ok_or(SIN_CORPUS)?;
            let informe = indexar_corpus(app.clone(), app.state::<ElCorpus>(), carpeta)?;
            Ok(Hecho { cuenta: Some(informe.documentos as u32), datos: a_json(&informe)? })
        }
        Orden::Kit { kit } => {
            let g = el_corpus.0.lock().map_err(|_| "el corpus quedó en mal estado")?;
            let informe = corpus::evaluar::evaluar(g.as_ref().ok_or(SIN_CORPUS)?, kit)?;
            Ok(Hecho { cuenta: Some(informe.preguntas as u32), datos: a_json(&informe)? })
        }
        Orden::LeerPrefs {} => Ok(Hecho { datos: a_json(&preferencias_de_ahora(app))?, cuenta: None }),
        Orden::CambiarPref { clave, valor } => {
            cambiar_por_la_puerta(app, clave, valor)?;
            Ok(Hecho { datos: a_json(&preferencias_de_ahora(app))?, cuenta: None })
        }
        Orden::ListarNotas {} => {
            let lista = reunion::lista(app);
            Ok(Hecho { cuenta: Some(lista.reuniones.len() as u32), datos: a_json(&lista)? })
        }
        Orden::AbrirNota { archivo } => {
            let idioma = if idiomas_guardados(&app.state::<LasPreferencias>()).consultor.starts_with("en") {
                "en"
            } else {
                "es"
            };
            let desbloqueo = &app.state::<LaPuerta>().1;
            Ok(Hecho { datos: a_json(&reunion::abrir_con(app, desbloqueo, archivo, idioma)?)?, cuenta: None })
        }
        // La política lo deniega antes de llegar aquí (`puerta::decidir`); si llegara, tampoco.
        Orden::EncenderApi {} => Err("la puerta no enciende el API".into()),
    }
}

fn preferencias_de_ahora(app: &tauri::AppHandle) -> prefs::Preferencias {
    app.state::<LasPreferencias>().actuales.lock().map(|p| p.clone()).unwrap_or_default()
}

/// Una preferencia de la lista blanca, por el mismo camino que la pantalla que la cambia.
fn cambiar_por_la_puerta(app: &tauri::AppHandle, clave: &str, valor: &str) -> Result<(), String> {
    use puerta::Clave;
    let delegables: Vec<&str> = Clave::TODAS.into_iter().filter(|c| c.delegable()).map(Clave::nombre).collect();
    let Some(c) = Clave::de(clave).filter(|c| c.delegable()) else {
        return Err(format!("no hay preferencia «{clave}» que la puerta pueda cambiar: {}", delegables.join(" · ")));
    };
    fn de_la_lista<T: serde::de::DeserializeOwned>(v: &str) -> Result<T, serde_json::Error> {
        serde_json::from_value(serde_json::Value::String(v.to_string()))
    }
    match c {
        Clave::IdiomaConsultor => fijar_idioma_de_pista(app.clone(), "consultor".into(), valor.into()).map(|_| ()),
        Clave::IdiomaCliente => fijar_idioma_de_pista(app.clone(), "cliente".into(), valor.into()).map(|_| ()),
        Clave::Retencion => {
            let r: prefs::Retencion = de_la_lista(valor).map_err(|_| format!("«{valor}» no es una retención: 7d · 30d · 90d · 1a · siempre"))?;
            reunion::fijar_retencion(app, r);
            Ok(())
        }
        Clave::VentanaDeLaBandeja => {
            let v: bandeja::Ventana = de_la_lista(valor).map_err(|_| format!("«{valor}» no es una ventana: 0 · 1h · 3h · fin · 24h"))?;
            reunion::fijar_ventana(app, v);
            Ok(())
        }
        Clave::LecturaAutomatica => {
            let encendida = match valor {
                "true" | "si" | "sí" | "on" => true,
                "false" | "no" | "off" => false,
                _ => return Err(format!("«{valor}» no es sí ni no: true · false")),
            };
            lectura_automatica(app.clone(), app.state::<LaPantalla>(), encendida);
            Ok(())
        }
        // `filter(delegable)` los dejó fuera arriba, y la política los deniega antes.
        Clave::Redactar | Clave::Api | Clave::Proveedor | Clave::ConservarMisTurnos | Clave::Nda => {
            Err("eso lo decides tú".into())
        }
    }
}

/// La ruta de `ghost`, si está compilado junto a la app (`pnpm ghost`). En H1 no se instala en el PATH.
fn ruta_de_ghost() -> Option<String> {
    let g = std::env::current_exe().ok()?.with_file_name("ghost");
    g.is_file().then(|| g.to_string_lossy().into_owned())
}

fn vista_de_la_puerta<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> puerta::VistaDeLaPuerta {
    app.state::<LaPuerta>().0.vista(ruta_de_ghost())
}

/// IA pregunta por la puerta: abierta o cerrada, por qué, dónde está `ghost` y qué hizo tu agente.
#[tauri::command]
fn la_puerta(app: tauri::AppHandle) -> puerta::VistaDeLaPuerta {
    vista_de_la_puerta(&app)
}

/// **El único sitio donde la puerta se abre**: el conmutador de IA, a mano (ADR 018 §5). Lo vigila
/// `pruebas_de_la_puerta_local`.
#[tauri::command]
fn abrir_la_puerta(app: tauri::AppHandle) -> puerta::VistaDeLaPuerta {
    let carpeta = reunion::carpeta_de_la_app(&app);
    // Se intenta dejar en 700; si no queda, la puerta misma se niega a abrir (auditoría del S3, B17).
    if let Err(e) = almacen::carpeta_privada(&carpeta) {
        println!("[puerta] la carpeta de la app no quedó en 700: {}", e.split(": ").last().unwrap_or(""));
    }
    let ops = std::sync::Arc::new(LaAppParaLaPuerta(app.clone()));
    app.state::<LaPuerta>().1.olvidar();
    let _ = app.state::<LaPuerta>().0.abrir(&carpeta, std::sync::Arc::new(puerta::DelLlavero), ops);
    vista_de_la_puerta(&app)
}

#[tauri::command]
fn cerrar_la_puerta(app: tauri::AppHandle) -> puerta::VistaDeLaPuerta {
    app.state::<LaPuerta>().0.cerrar(puerta::Cierre::ATuMano);
    app.state::<LaPuerta>().1.olvidar();
    let vista = vista_de_la_puerta(&app);
    let _ = app.emit_to(ventana::PRINCIPAL, EVENTO_PUERTA, vista.clone());
    vista
}

/// «Iniciar sesión» y «Solo notas» cierran la puerta **antes** de abrir nada de la reunión.
fn cerrar_la_puerta_al_empezar<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    app.state::<LaPuerta>().1.olvidar();
    if app.state::<LaPuerta>().0.cerrar(puerta::Cierre::EnReunion) {
        let _ = app.emit_to(ventana::PRINCIPAL, EVENTO_PUERTA, vista_de_la_puerta(app));
    }
}

/// `⌃⌥↵` — guardar la última propuesta, la que la banda enseña (ADR 016 §3).
fn el_atajo_de_guardar_la_propuesta() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::Enter)
}

/// `⌃⌥N` — el cuaderno al frente, en tu nota.
fn el_atajo_de_anotar() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyN)
}

/// `⌃⌥P` — fijar la ficha que la banda enseña.
fn el_atajo_de_fijar() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::CONTROL | Modifiers::ALT), Code::KeyP)
}

/// El botón «Ver qué alcanza a ver» de la banda ampliada: lo mismo que `⌃⌥R`.
#[tauri::command]
fn abrir_lo_que_ve(app: tauri::AppHandle) {
    ir_a_lo_que_ve(&app);
}

fn ir_a_lo_que_ve<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let Some(v) = app.get_webview_window(ventana::PRINCIPAL) else {
        return;
    };
    let navego = v.url().map_err(|e| e.to_string()).and_then(|mut url| {
        url.set_query(Some("pantalla=sesion"));
        v.navigate(url).map_err(|e| e.to_string())
    });
    let _ = v.show();
    let _ = v.set_focus();
    match navego {
        Ok(()) => println!("[radar] el cuaderno se abre en Sesión"),
        Err(e) => println!("[radar] no se pudo abrir Sesión en el cuaderno: {e}"),
    }
}

fn atender_el_atajo<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    atajo: &tauri_plugin_global_shortcut::Shortcut,
    evento: tauri_plugin_global_shortcut::ShortcutEvent,
) {
    use tauri_plugin_global_shortcut::ShortcutState;
    if evento.state() != ShortcutState::Pressed {
        return;
    }
    if *atajo == el_atajo() {
        ejecutar_el_corte(app);
    } else if *atajo == el_atajo_del_transcript() {
        println!("[transcript] ⌃⌥T");
        let _ = app.emit_to(ventana::BANDA, EVENTO_TRANSCRIPT, ());
    } else if *atajo == el_atajo_de_ayuda() {
        println!("[ficha] ⌃⌥A");
        let _ = app.emit_to(ventana::BANDA, EVENTO_FICHA, ());
    } else if *atajo == el_atajo_del_modo_de_voz() {
        conmutar_el_modo(app);
    } else if *atajo == el_atajo_de_leer_la_pantalla() {
        println!("[pantalla] ⌃⌥L");
        leer_una_vez(&app.state::<LaPantalla>());
    } else if *atajo == el_atajo_del_radar() {
        println!("[radar] ⌃⌥R");
        ir_a_lo_que_ve(app);
    } else if *atajo == el_atajo_de_anotar() {
        reunion::ir_a_notas(app);
    } else if *atajo == el_atajo_de_fijar() {
        reunion::fijar(app);
    } else if *atajo == el_atajo_de_guardar_la_propuesta() {
        reunion::guardar_la_ultima(app);
    } else if *atajo == el_atajo_de_la_banda() {
        println!("[ventanas] ⌃⌥B");
        let (mango, borde) = (app.clone(), borde_de(app).otro());
        std::thread::spawn(move || poner_la_banda(&mango, borde));
    } else if *atajo == el_atajo_de_callar() {
        let estado = app.state::<LaVozQueSale>();
        estado.voz.callar();
        println!("[habla] ⎋: callada a petición del usuario");
        let _ = app.emit_to(ventana::BANDA, EVENTO_VOZ, estado.estado());
    }
}

/// Registra `⌥⎋`, y **si no puede, lo dice**.
///
/// Un atajo global puede estar cogido por el sistema, por la reunión o por otra app, y la
/// respuesta correcta no es morir: la app funciona igual, con el botón de la pantalla de
/// honestidad. Lo que no puede pasar es que falle en silencio — el usuario pulsaría la tecla en
/// mitad de una reunión creyendo que cortó, y no habría cortado nada. (Riesgo nº 7 del plan.)
fn registrar_el_kill_switch<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    use tauri_plugin_global_shortcut::GlobalShortcutExt;
    match app.global_shortcut().register(el_atajo()) {
        Ok(()) => println!("[corte] kill-switch ⌥⎋ registrado"),
        Err(e) => println!(
            "[corte] NO se pudo registrar ⌥⎋ ({e}): el kill-switch sigue en el botón de Honestidad, \
             pero la tecla no va a responder"
        ),
    }
    match app.global_shortcut().register(el_atajo_del_transcript()) {
        Ok(()) => println!("[transcript] ⌃⌥T registrado"),
        Err(e) => println!(
            "[transcript] NO se pudo registrar ⌃⌥T ({e}): el transcript no se va a poder abrir \
             con la tecla"
        ),
    }
    match app.global_shortcut().register(el_atajo_de_ayuda()) {
        Ok(()) => println!("[ficha] ⌃⌥A «ayúdame con esto» registrado"),
        Err(e) => println!(
            "[ficha] NO se pudo registrar ⌃⌥A ({e}): la ficha a petición sigue en el botón de la \
             banda ampliada, pero la tecla no va a responder"
        ),
    }
    match app.global_shortcut().register(el_atajo_del_modo_de_voz()) {
        Ok(()) => println!("[habla] ⌃⌥V «modo solo audio» registrado"),
        Err(e) => println!(
            "[habla] NO se pudo registrar ⌃⌥V ({e}): el modo solo audio no se va a poder encender \
             — y hoy no tiene otra puerta, así que queda apagado"
        ),
    }
    match app.global_shortcut().register(el_atajo_del_radar()) {
        Ok(()) => println!("[radar] ⌃⌥R «qué ve» registrado"),
        Err(e) => println!(
            "[radar] NO se pudo registrar ⌃⌥R ({e}): lo que alcanza a ver cada programa sigue en \
             Sesión y en el botón de la banda ampliada"
        ),
    }
    match app.global_shortcut().register(el_atajo_de_leer_la_pantalla()) {
        Ok(()) => println!("[pantalla] ⌃⌥L «léela ahora» registrado"),
        Err(e) => println!(
            "[pantalla] NO se pudo registrar ⌃⌥L ({e}): la lectura a petición no va a responder \
             a la tecla; la automática sigue en el interruptor de Sesión"
        ),
    }
    match app.global_shortcut().register(el_atajo_de_anotar()) {
        Ok(()) => println!("[notas] ⌃⌥N «anotar» registrado"),
        Err(e) => println!(
            "[notas] NO se pudo registrar ⌃⌥N ({e}): tu nota sigue en Notas, en el cuaderno, pero \
             la tecla no te va a llevar"
        ),
    }
    match app.global_shortcut().register(el_atajo_de_fijar()) {
        Ok(()) => println!("[notas] ⌃⌥P «fijar» registrado"),
        Err(e) => println!(
            "[notas] NO se pudo registrar ⌃⌥P ({e}): la ficha de la banda no se va a poder fijar \
             con la tecla"
        ),
    }
    match app.global_shortcut().register(el_atajo_de_guardar_la_propuesta()) {
        Ok(()) => println!("[propuestas] ⌃⌥↵ «guardar la propuesta» registrado"),
        Err(e) => println!(
            "[propuestas] NO se pudo registrar ⌃⌥↵ ({e}): las propuestas se siguen guardando en \
             Notas, pero la tecla no va a responder"
        ),
    }
    match app.global_shortcut().register(el_atajo_de_la_banda()) {
        Ok(()) => println!("[ventanas] ⌃⌥B «la banda, al otro borde» registrado"),
        Err(e) => println!(
            "[ventanas] NO se pudo registrar ⌃⌥B ({e}): la banda se sigue cambiando de borde en \
             Sesión, pero la tecla no va a responder"
        ),
    }
}

/// El acople automático de la fase 1: **esperar a que haya a quién acoplar, y hacerlo una vez.**
///
/// Dos intentos anteriores, los dos medidos en vivo y los dos fallidos, explican por qué esto es
/// un bucle y no un evento:
///
/// 1. **«Acopla lo que esté al frente», al arrancar.** Se saltaba a sí mismo: la aplicación de
///    delante somos nosotros, que acabamos de abrir la ventana principal. El log lo dijo en la
///    primera corrida — *«no hay ninguna otra aplicación al frente»*— y el mecanismo entero no se
///    ejecutó ni una vez.
/// 2. **Al perder el foco la ventana principal.** Funcionó una vez y no volvió a dispararse: si
///    la principal nunca llegó a tener el foco —el arranque con otra aplicación delante, el
///    reinicio del observador de `tauri dev`— el evento `Focused(false)` no llega nunca. Un
///    disparador que depende de un evento que puede no ocurrir no es un disparador.
///
/// Un latido acotado sí se puede afirmar: mira cada [`LATIDO`] durante [`ESPERA`], acopla en
/// cuanto haya alguien delante que no seamos nosotros, y **para**.
///
/// **Es andamio de la fase 1 y se declara como tal.** En el producto el disparador es la
/// detección de la reunión (fase 2), que llama a `acoplar` sabiendo a quién. Lo de debajo
/// —medir, encoger, anotar la huella, devolver— es lo mismo y no cambia.
const LATIDO: std::time::Duration = std::time::Duration::from_millis(1500);
const ESPERA: u32 = 20;

fn acoplar_cuando_haya_a_quien<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    let mango = app.clone();
    std::thread::spawn(move || {
        // El bucle deja UNA línea por motivo, no una por vuelta. Escrito sin esto dejaba veinte
        // veces seguidas la misma frase en el log: un latido que se narra a sí mismo ahoga
        // justamente lo que hay que leer, y el log de este módulo es el instrumento con el que
        // se contesta «¿por qué no se acopló?».
        let mut dicho: Option<String> = None;
        for _ in 0..ESPERA {
            std::thread::sleep(LATIDO);
            // Abajo espera a que haya alguien al frente (el H1); arriba, a que haya una reunión
            // (sprint 004): con la banda arriba solo se mueve la ventana de la videollamada.
            let hay_a_quien = match borde_de(&mango) {
                ventana::Borde::Abajo => acople::hay_alguien_al_frente(),
                ventana::Borde::Arriba => sesion::ventana_de_la_reunion().is_some(),
            };
            if YA_SE_ACOPLO.load(Ordering::SeqCst) || !acople::hay_permiso() || !hay_a_quien {
                continue;
            }
            let Ok(informe) = acoplar_segun_el_borde(&mango) else {
                continue;
            };
            if informe.acoplada() {
                YA_SE_ACOPLO.store(true, Ordering::SeqCst);
                registrar_acople(&mango, "al volver el usuario a su trabajo", &informe);
                return;
            }
            // Si no acopló nada —esa ventana ya cabía, o no se deja— se sigue esperando: el
            // intento no se gasta por haber pasado por delante algo que no estorbaba. Pero el
            // motivo solo se escribe cuando es NUEVO.
            let motivo = informe.motivos.join(" · ");
            if dicho.as_deref() != Some(motivo.as_str()) {
                registrar_acople(&mango, "intento de acople", &informe);
                dicho = Some(motivo);
            }
        }
        println!(
            "[acople] nadie a quien acoplar en {} s: la banda flota",
            ESPERA * LATIDO.as_secs() as u32 + ESPERA * LATIDO.subsec_millis() / 1000
        );
    });
}

#[cfg(test)]
mod pruebas_del_diccionario_en_disco {
    use super::*;

    fn temporal(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ag-dicc-{}-{}", nombre, std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d.join(DICCIONARIO)
    }

    /// **Un archivo que no existe no se puede editar.** El manual le dice al usuario dónde está su
    /// diccionario; si la app esperara a tener algo que guardar, él iría a buscarlo y no habría nada.
    #[test]
    fn el_archivo_nace_con_la_semilla_y_el_usuario_puede_leerlo() {
        let ruta = temporal("nace");
        asegurar_el_diccionario(&ruta).expect("no se pudo dejar el archivo escrito");
        let texto = std::fs::read_to_string(&ruta).unwrap();
        assert!(texto.contains("Power BI:"), "la semilla no llegó al archivo");
        assert!(texto.starts_with("# Tu diccionario técnico"), "sin cabecera nadie sabe qué editar");
        // Y lo que se escribió se vuelve a leer: el archivo del usuario es su propio contrato.
        let d = diccionario::Diccionario::de_texto(&texto).expect("la app escribió algo que no sabe leer");
        assert_eq!(d.corregir("con power by"), "con Power BI");
        let _ = std::fs::remove_dir_all(ruta.parent().unwrap());
    }

    /// **Regla 17-bis: un derivado no nace menos privado que su fuente.** Este desciende de los
    /// documentos del usuario, así que 600 — y si lo encuentra flojo, lo repara, que es la parte que
    /// el precedente del índice del corpus enseñó: descubrirlo no sirve de nada sin arreglarlo.
    #[cfg(unix)]
    #[test]
    fn el_archivo_nace_en_600_y_se_repara_si_lo_encuentra_abierto() {
        use std::os::unix::fs::PermissionsExt;
        let ruta = temporal("permisos");
        let modo = |p: &std::path::Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;

        asegurar_el_diccionario(&ruta).unwrap();
        assert_eq!(modo(&ruta), 0o600, "el diccionario nació legible por otras cuentas del Mac");
        // **Y nació así, no se apretó después.** La primera versión usaba `fs::write` y dejaba una
        // ventana en 644; lo delató la traza del propio gate del efímero. Si `cerrar_permisos` dice
        // que no tuvo que reparar nada, no hubo ventana.
        assert!(
            !cerrar_permisos(&ruta).unwrap(),
            "el archivo hubo que repararlo: nació abierto y se cerró un instante después"
        );

        std::fs::set_permissions(&ruta, std::fs::Permissions::from_mode(0o644)).unwrap();
        asegurar_el_diccionario(&ruta).unwrap();
        assert_eq!(modo(&ruta), 0o600, "lo encontró abierto y lo dejó abierto");
        let _ = std::fs::remove_dir_all(ruta.parent().unwrap());
    }

    /// **Y su carpeta nace en 700** (auditoría del S3, B1): el «escritor único» del `CLAUDE.md` promete
    /// carpetas 700, y esta se creaba con `create_dir_all` (el umask: 755). Demostrado en rojo con el
    /// código de antes.
    #[cfg(unix)]
    #[test]
    fn la_carpeta_del_diccionario_nace_en_700() {
        use std::os::unix::fs::PermissionsExt;
        let ruta = temporal("carpeta");
        asegurar_el_diccionario(&ruta).unwrap();
        let padre = ruta.parent().unwrap();
        assert_eq!(std::fs::metadata(padre).unwrap().permissions().mode() & 0o777, 0o700, "la carpeta del diccionario nació abierta");
        let _ = std::fs::remove_dir_all(padre);
    }

    /// Editar el archivo a mano tiene que servir para algo, y tiene que servir **sin reiniciar la
    /// app**: se lee al empezar la sesión.
    #[test]
    fn lo_que_el_usuario_escribe_a_mano_manda() {
        let ruta = temporal("a-mano");
        std::fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        std::fs::write(&ruta, "# lo mío\nCooperativa Sur del Valle: [cooperativa sur, coope sur]\n").unwrap();
        let d = diccionario_de_la_sesion(&ruta, &[]);
        assert_eq!(d.corregir("lo de coope sur"), "lo de Cooperativa Sur del Valle");
        // Y su archivo MANDA: la semilla no se le cuela por detrás.
        assert_eq!(d.corregir("con power by"), "con power by");
        let _ = std::fs::remove_dir_all(ruta.parent().unwrap());
    }

    /// **Un archivo torcido no detiene la reunión, y tampoco pasa en silencio.** Se sigue con la
    /// semilla; el motivo, con su número de línea, va al log.
    #[test]
    fn un_archivo_torcido_cae_a_la_semilla_en_vez_de_dejar_la_app_sin_jerga() {
        let ruta = temporal("torcido");
        std::fs::create_dir_all(ruta.parent().unwrap()).unwrap();
        std::fs::write(&ruta, "Power BI: power by\n").unwrap();  // sin corchetes
        let d = diccionario_de_la_sesion(&ruta, &[]);
        assert_eq!(d.corregir("con power by"), "con Power BI", "cayó a nada en vez de a la semilla");
        let _ = std::fs::remove_dir_all(ruta.parent().unwrap());
    }

    /// Y si el archivo no está —primer arranque, o el usuario lo borró— la sesión funciona igual.
    #[test]
    fn sin_archivo_la_sesion_arranca_con_la_semilla() {
        let d = diccionario_de_la_sesion(&temporal("ausente"), &["Páramo".into()]);
        assert_eq!(d.corregir("con power by"), "con Power BI");
        assert_eq!(d.cuantos_del_corpus(), 1, "los nombres del corpus entran igual");
    }
}

#[cfg(test)]
mod pruebas_de_las_teclas {
    use super::*;
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};

    /// **La decisión 7 de la mirada 17-quater, fijada: ninguna tecla de la app vuelve a `⌘⇧`.**
    ///
    /// Según la documentación de Zoom para Mac, `⌘⇧A` silencia el micrófono, `⌘⇧V` apaga la cámara
    /// y `⌘⇧T` pausa la pantalla compartida; y en los navegadores `⌘⇧T` reabre la última pestaña.
    /// Una tecla global se la quita a todos ellos mientras la app esté abierta. El usuario eligió
    /// `⌃⌥` para todas. Se quedan fuera, a propósito, `⌥⎋` (el kill-switch, aprobado en el sprint
    /// 001) y `⎋` (callar la voz, que solo existe mientras suena).
    #[test]
    fn las_teclas_de_la_app_son_control_opcion() {
        let control_opcion = Some(Modifiers::CONTROL | Modifiers::ALT);
        let esperadas = [
            (el_atajo_del_transcript(), Code::KeyT),
            (el_atajo_de_ayuda(), Code::KeyA),
            (el_atajo_del_modo_de_voz(), Code::KeyV),
            (el_atajo_de_leer_la_pantalla(), Code::KeyL),
            (el_atajo_del_radar(), Code::KeyR),
            (el_atajo_de_anotar(), Code::KeyN),
            (el_atajo_de_fijar(), Code::KeyP),
            (el_atajo_de_guardar_la_propuesta(), Code::Enter),
            (el_atajo_de_la_banda(), Code::KeyB),
        ];
        for (atajo, tecla) in esperadas {
            assert_eq!(atajo, Shortcut::new(control_opcion, tecla), "{tecla:?} no es ⌃⌥");
        }
        assert_eq!(el_atajo(), Shortcut::new(Some(Modifiers::ALT), Code::Escape));
        assert_eq!(el_atajo_de_callar(), Shortcut::new(None, Code::Escape));
    }

    /// Dos teclas iguales harían que una de las dos no respondiera nunca, y el registro no lo
    /// diría: el segundo `register` fallaría con un aviso en el log y nada más.
    #[test]
    fn ninguna_tecla_se_repite() {
        let todas = [
            el_atajo(),
            el_atajo_del_transcript(),
            el_atajo_de_ayuda(),
            el_atajo_del_modo_de_voz(),
            el_atajo_de_leer_la_pantalla(),
            el_atajo_del_radar(),
            el_atajo_de_callar(),
            el_atajo_de_anotar(),
            el_atajo_de_fijar(),
            el_atajo_de_guardar_la_propuesta(),
            el_atajo_de_la_banda(),
        ];
        for (i, a) in todas.iter().enumerate() {
            for b in &todas[i + 1..] {
                assert_ne!(a, b, "dos atajos comparten tecla");
            }
        }
    }
}

#[cfg(test)]
mod pruebas_del_estado_del_diccionario {
    use super::*;

    /// Lo que Idioma enseña cuadra: el total es la suma de las dos filas, y la ruta nunca llega con
    /// la carpeta del usuario escrita entera (es un dato personal que no hace falta enseñar).
    #[test]
    fn las_dos_filas_suman_el_total_y_la_ruta_empieza_por_la_casa() {
        let casa = std::env::var("HOME").unwrap_or_default();
        let ruta = PathBuf::from(&casa).join("Library/Application Support/x/diccionario.yaml");
        let mut d = diccionario::Diccionario::semilla();
        d.con_nombres_del_corpus(&["Páramo Azul".into(), "Sur del Valle".into()]);
        let e = EstadoDelDiccionario::de(&d, &ruta);
        assert_eq!(e.terminos, e.del_corpus + e.en_tu_archivo);
        assert!(e.del_corpus >= 1, "los nombres del corpus no entraron: {e:?}");
        if !casa.is_empty() {
            assert!(e.ruta.starts_with("~/Library/"), "{}", e.ruta);
        }
    }
}

#[cfg(test)]
mod pruebas_de_la_carpeta_del_corpus {
    /// **La carpeta del corpus se recuerda y se vuelve a leer al arrancar** (auditoría del S3, B29;
    /// decisión del usuario). `indexar_corpus` la guarda en las preferencias, y el arranque la reindexa
    /// en segundo plano. Se vigila la fuente: las dos cosas necesitan la app entera. ¿Puede fallar? Sí:
    /// sin el `recordar` en `indexar_corpus`, o sin la llamada en `setup`, es rojo (bitácora).
    #[test]
    fn la_carpeta_del_corpus_se_recuerda_y_se_relee_al_arrancar() {
        let fuente = include_str!("lib.rs");
        let cuerpo = |firma: &str| {
            let desde = fuente.find(firma).unwrap_or_else(|| panic!("no está {firma}"));
            let c = &fuente[desde..];
            c[..c.find("\n}\n").expect("su cierre")].to_string()
        };
        assert!(
            cuerpo("\nfn indexar_corpus(").contains(concat!("p.carpeta_del_corpus = ", "Some(")),
            "indexar no recuerda la carpeta"
        );
        assert!(
            fuente.contains(concat!("reindexar_al_", "arrancar(app.handle()")),
            "el arranque no vuelve a leer la carpeta recordada"
        );
    }
}

#[cfg(test)]
mod pruebas_de_los_clientes_del_corpus {
    use super::*;

    fn el_kit() -> ElCorpus {
        let mut c = corpus::Corpus::en_memoria().unwrap();
        let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba/corpus");
        c.indexar(std::path::Path::new(raiz), &|_| {}).expect("no se indexó el kit");
        ElCorpus(std::sync::Arc::new(std::sync::Mutex::new(Some(c))))
    }

    /// **Lo que reciben la bóveda y el diccionario es el nombre del cliente**, no el del archivo
    /// (auditoría del S2, A2 y A3): «Ficha de cliente · Páramo Azul» no aparece nunca en una frase
    /// del cliente, así que como «nombre conocido» la bóveda no tapaba nada.
    #[test]
    fn los_clientes_del_corpus_son_sus_nombres_y_la_boveda_los_tapa() {
        assert_eq!(clientes_del_corpus(&el_kit()), vec!["Páramo Azul".to_string()]);
        // Un cliente de una sola palabra, que la heurística de nombres propios no ve.
        let conocidos = vec![corpus::cliente_de("Ficha de cliente · Bancolombia")];
        let mut b = sintesis::anonimo::Boveda::nueva(&conocidos);
        let fuera = b.tapar("Bancolombia pide lo mismo");
        assert!(!fuera.contains("Bancolombia"), "{fuera}");
        assert!(fuera.contains("[CLIENTE_1]"), "{fuera}");
    }
}

#[cfg(test)]
mod pruebas_del_gasto_del_mes {
    use super::*;

    /// **El mes que cambia con la app abierta** (auditoría del S2, B3): el gasto del mes anterior no
    /// bloquea el API del nuevo.
    #[test]
    fn el_gasto_de_otro_mes_no_cuenta() {
        let mut g = GastoDelMes { mes: "2026-09".into(), usd: 12.0 };
        gasto_vigente(&mut g, "2026-09");
        assert_eq!(g.usd, 12.0, "el del mes en curso se conserva");
        gasto_vigente(&mut g, "2026-10");
        assert_eq!((g.mes.as_str(), g.usd), ("2026-10", 0.0));
    }

    /// **El archivo del gasto no desaparece nunca** (M10): se escribe a un temporal y se renombra
    /// encima. Cada escritura deja el archivo con lo último, cerrado (600) y sin temporal al lado.
    #[test]
    fn el_gasto_se_reemplaza_sin_desaparecer() {
        use std::os::unix::fs::PermissionsExt;
        let d = std::env::temp_dir().join(format!("ag-gasto-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let ruta = d.join(COSTO_DEL_MES);
        for usd in [0.5, 1.25] {
            escribir_el_gasto(&ruta, &GastoDelMes { mes: "2026-09".into(), usd }).unwrap();
            let leido: GastoDelMes = serde_json::from_str(&std::fs::read_to_string(&ruta).unwrap()).unwrap();
            assert_eq!(leido.usd, usd);
            assert_eq!(std::fs::metadata(&ruta).unwrap().permissions().mode() & 0o777, 0o600);
            assert!(!ruta.with_extension("json.tmp").exists(), "quedó el temporal");
        }
        let _ = std::fs::remove_dir_all(&d);
    }
}

#[cfg(test)]
mod pruebas_del_asa_y_la_voz {
    use super::*;

    /// **Arrastrar el asa apaga el modo** (auditoría del S2, M14): por encima de los 44 px de la
    /// banda de voz, con el modo encendido. Sin el modo, o dentro de la línea, no toca nada.
    #[test]
    fn el_asa_por_encima_de_la_banda_de_voz_apaga_el_modo() {
        assert!(el_asa_apaga_el_modo(ventana::ALTO_COMPACTA, true));
        assert!(el_asa_apaga_el_modo(ventana::ALTO_VOZ + 1, true));
        assert!(!el_asa_apaga_el_modo(ventana::ALTO_VOZ, true));
        assert!(!el_asa_apaga_el_modo(ventana::ALTO_COMPACTA, false));
    }

    /// Sin voz para el idioma, `⌃⌥V` no enciende el modo (auditoría del S2, B14).
    #[test]
    fn sin_voz_el_modo_no_se_enciende() {
        assert!(puede_encender_el_modo(true));
        assert!(!puede_encender_el_modo(false));
    }
}

#[cfg(test)]
mod pruebas_de_la_clave_borrada {
    use super::*;

    /// **Borrar la clave apaga el API y lo dice** (casilla 6 del S3): si no se apagara —y no se
    /// recordara apagado—, al guardar otra clave y reiniciar se encendería solo. ¿Puede fallar? Sí:
    /// con `apagar_si_usaba` devolviendo siempre `false`, es rojo (bitácora).
    #[test]
    fn borrar_la_clave_del_encendido_lo_apaga() {
        let mut api = ConfigDelApi { encendida: true, externo: sintesis::api::Externo::Groq };
        assert!(!apagar_si_usaba(&mut api, sintesis::api::Externo::Claude), "se apagó por la clave de otro");
        assert!(api.encendida);
        assert!(apagar_si_usaba(&mut api, sintesis::api::Externo::Groq));
        assert!(!api.encendida);
        assert!(!apagar_si_usaba(&mut api, sintesis::api::Externo::Groq), "ya estaba apagado: nada que recordar");
    }
}

#[cfg(test)]
mod pruebas_de_lo_que_salio {
    use super::*;

    /// **El corte olvida lo que salió al API** (B37): «se borra al cerrar» y también con `⌥⎋`. ¿Puede
    /// fallar? Sí: sin `registro.vaciar()` en `cortar_la_sugerencia`, es rojo (bitácora).
    #[test]
    fn el_corte_olvida_lo_que_salio() {
        let s = LaSintesis::default();
        let b = sintesis::anonimo::Boveda::nueva(&[]);
        s.registro.anotar(sintesis::api::LoQueSalio::de("texto", &b, sintesis::api::Externo::Claude, "Alcance"));
        let antes = s.epoca.load(Ordering::SeqCst);
        cortar_la_sugerencia(&s);
        assert_eq!(s.registro.cuantas(), 0, "lo que salió sobrevivió al corte");
        assert_eq!(s.epoca.load(Ordering::SeqCst), antes + 1);
    }
}

#[cfg(test)]
mod pruebas_del_informe_del_corte {
    /// **El log del corte dice los bytes que habían salido** (segunda pasada de la auditoría del
    /// S2, B25). La pieza `ContadorDeRed` pone el contador a cero DENTRO del bucle, así que leerlo
    /// después escribía siempre «red 0 B». `ejecutar_el_corte` necesita la app entera y no se puede
    /// llamar en un test; lo que se vigila es el orden, que es donde vivía el defecto.
    ///
    /// ¿Puede fallar? Sí: con `bytes_en_red: red::bytes()` en el informe, es rojo (bitácora).
    #[test]
    fn el_contador_se_lee_antes_de_ponerlo_a_cero() {
        let fuente = include_str!("lib.rs");
        let desde = fuente.find("fn ejecutar_el_corte").expect("la función del corte");
        let cuerpo = &fuente[desde..];
        let cuerpo = &cuerpo[..cuerpo.find("\n}\n").expect("su cierre")];
        let lee = cuerpo.find("red::bytes()").expect("el corte lee el contador");
        let bucle = cuerpo.find("for pieza in corte::TODAS").expect("el bucle de las piezas");
        assert!(lee < bucle, "el contador se lee después de que el corte lo ponga a cero");
        assert_eq!(cuerpo.matches("red::bytes()").count(), 1, "y se lee una sola vez");
    }
}

#[cfg(test)]
mod pruebas_de_la_puerta_de_la_captura {
    /// **En solo notas no arranca ninguna captura** (ADR 017 §5). `empezar` necesita la app entera y no
    /// se puede llamar en un test; lo que se vigila es lo mismo que en el informe del corte, el orden:
    /// la pantalla y las pistas arrancan DESPUÉS de la puerta del modo, y nadie más las arranca.
    ///
    /// ¿Puede fallar? Sí: con `arrancar_la_pantalla` antes de la puerta, es rojo (bitácora). Las agujas
    /// se arman con `concat!` para que este test no se cuente a sí mismo.
    #[test]
    fn la_captura_arranca_despues_de_la_puerta_y_solo_desde_empezar() {
        let fuente = include_str!("lib.rs");
        let pantalla = concat!("arrancar_la_", "pantalla(&");
        let pistas = concat!("escucha::Escucha::", "arrancar(");
        let desde = fuente.find("\nfn empezar(").expect("la función que arranca la reunión");
        let cuerpo = &fuente[desde..];
        let cuerpo = &cuerpo[..cuerpo.find("\n}\n").expect("su cierre")];
        let puerta = cuerpo.find("if !modo::abre_la_captura(modo)").expect("la puerta del modo");
        let p = cuerpo.find(pantalla).expect("empezar arranca la pantalla");
        let e = cuerpo.find(pistas).expect("empezar arranca las pistas");
        assert!(puerta < p, "la pantalla arranca antes de la puerta: en solo notas se leería la reunión");
        assert!(puerta < e, "las pistas arrancan antes de la puerta: en solo notas se escucharía");
        assert!(cuerpo[puerta..p].contains("return Ok("), "la puerta no devuelve: solo notas seguiría hasta la captura");
        assert_eq!(fuente.matches(pantalla).count(), 1, "otro sitio arranca la pantalla, sin pasar por la puerta");
        assert_eq!(fuente.matches(pistas).count(), 1, "otro sitio arranca las pistas, sin pasar por la puerta");
        // **El ensayo y la reunión no conviven** (ADR 019 §6.5): `empezar` suelta el ensayo —y su
        // micrófono— antes de abrir las pistas de la reunión.
        let ensayo = cuerpo.find(concat!("soltar_el_", "ensayo(&app)")).expect("empezar corta el ensayo");
        assert!(ensayo < e, "la reunión abre sus pistas con el micrófono del ensayo todavía abierto");
    }

    /// **El ensayo abre solo el micrófono** (ADR 019 §6.1): ni la pantalla, ni las dos pistas de la reunión,
    /// ni el audio del sistema, ni el radar. Y antes de abrirlo, con una reunión abierta se niega y la
    /// puerta local se cierra (§6.5).
    ///
    /// ¿Puede fallar? Sí: con la puerta cerrada después del micrófono, es rojo (bitácora, fase 3).
    #[test]
    fn el_ensayo_abre_solo_el_microfono_y_cierra_la_puerta_antes() {
        let fuente = include_str!("lib.rs");
        let desde = fuente.find(concat!("\nfn empezar_el_", "ensayo(")).expect("la función que arranca el ensayo");
        let cuerpo = &fuente[desde..];
        let cuerpo = &cuerpo[..cuerpo.find("\n}\n").expect("su cierre")];
        for prohibida in [
            concat!("arrancar_la_", "pantalla("),
            concat!("escucha::Escucha::", "arrancar("),
            concat!("del_", "sistema("),
            concat!("arrancar_el_", "radar("),
        ] {
            assert!(!cuerpo.contains(prohibida), "el ensayo llama a «{prohibida}»: solo abre el micrófono");
        }
        let micro = cuerpo.find(concat!("Oido::", "del_microfono(")).expect("el ensayo abre su micrófono");
        let puerta = cuerpo.find(concat!("cerrar_la_puerta_", "al_empezar(&app)")).expect("el ensayo cierra la puerta");
        let reunion = cuerpo.find(concat!("return Err(ensayo::NoEmpezo::", "EnReunion)")).expect("con reunión no se ensaya");
        assert!(puerta < micro, "la puerta local se cierra después de abrir el micrófono");
        assert!(reunion < micro, "se abre el micrófono antes de mirar si hay una reunión");
    }
}

#[cfg(test)]
mod pruebas_de_la_puerta_local {
    /// **La puerta nace cerrada y se abre en un solo sitio**: el conmutador de IA (ADR 018 §5). Ni el
    /// arranque, ni las preferencias, ni otro comando la abren. `abrir_la_puerta` necesita la app entera;
    /// lo que se vigila es el código. Las agujas se arman con `concat!` para no contarse a sí mismas.
    ///
    /// ¿Puede fallar? Sí: con la puerta abierta en `setup`, es rojo (bitácora).
    #[test]
    fn la_puerta_solo_se_abre_desde_su_conmutador() {
        let fuente = include_str!("lib.rs");
        let abrir = concat!("LaPuerta>().0.", "abrir(");
        assert_eq!(fuente.matches(abrir).count(), 1, "otro sitio abre la puerta local");
        let desde = fuente.find("\nfn abrir_la_puerta(").expect("el comando que la abre");
        let cuerpo = &fuente[desde..];
        let cuerpo = &cuerpo[..cuerpo.find("\n}\n").expect("su cierre")];
        assert!(cuerpo.contains(abrir), "la puerta se abre fuera de su conmutador");
    }

    /// **La puerta abre tus reuniones con SU desbloqueo, y lo olvida al abrirse** (auditoría del S3, M4).
    /// Con el de la pantalla, exportar por la mañana abría todas tus reuniones a tu agente por la tarde
    /// sin un Touch ID. ¿Puede fallar? Sí: con `reunion::abrir` y el desbloqueo del cuaderno, que era el
    /// código de antes, es rojo (bitácora).
    #[test]
    fn la_puerta_abre_con_su_desbloqueo_y_lo_olvida_al_abrirse() {
        let fuente = include_str!("lib.rs");
        let desde = fuente.find("Orden::AbrirNota { archivo } =>").expect("la orden de abrir una nota");
        let rama = &fuente[desde..desde + 600];
        assert!(rama.contains(concat!("app.state::<LaPuerta>()", ".1")), "la puerta abre con un desbloqueo que no es el suyo");
        assert!(rama.contains(concat!("reunion::abrir_", "con(app, desbloqueo")), "la puerta no le pasa su desbloqueo");
        let desde = fuente.find("\nfn abrir_la_puerta(").expect("el comando que la abre");
        let cuerpo = &fuente[desde..];
        let cuerpo = &cuerpo[..cuerpo.find("\n}\n").expect("su cierre")];
        assert!(cuerpo.contains(concat!("LaPuerta>().1.", "olvidar()")), "al abrirse, la puerta no olvida el desbloqueo anterior");
    }

    /// **«Iniciar sesión» y «Solo notas» la cierran antes que nada de la reunión**: antes de abrir el
    /// cuaderno, antes de la pantalla y antes de las pistas.
    ///
    /// ¿Puede fallar? Sí: con el cierre después de `reunion::al_empezar`, es rojo (bitácora).
    #[test]
    fn empezar_cierra_la_puerta_antes_que_nada() {
        let fuente = include_str!("lib.rs");
        let desde = fuente.find("\nfn empezar(").expect("la función que arranca la reunión");
        let cuerpo = &fuente[desde..];
        let cuerpo = &cuerpo[..cuerpo.find("\n}\n").expect("su cierre")];
        let cierre = cuerpo.find(concat!("cerrar_la_puerta_", "al_empezar(&app)")).expect("empezar ya no cierra la puerta");
        for despues in [
            concat!("reunion::al_", "empezar(&app)"),
            concat!("arrancar_la_", "pantalla(&"),
            concat!("escucha::Escucha::", "arrancar("),
        ] {
            let d = cuerpo.find(despues).unwrap_or_else(|| panic!("empezar ya no llama a {despues}"));
            assert!(cierre < d, "{despues} va antes de cerrar la puerta: el agente alcanzaría la reunión");
        }
    }
}
