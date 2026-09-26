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
pub mod capture;
pub mod corpus;
pub mod corte;
/// El contrato con la interfaz, y el gate que lo compara. Solo se compila en `cargo test`: su
/// trabajo es escribir `src/contrato.generado.ts`, no viajar en el binario del usuario.
#[cfg(test)]
mod contrato;
pub mod diccionario;
pub mod disparo;
pub mod escucha;
pub mod ficha;
pub mod habla;
pub mod pantalla;
pub mod permisos;
pub mod red;
pub mod relleno;
pub mod sesion;
pub mod stt;
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

/// Permisos del archivo: **solo su dueño**. Es la regla 17-bis — un derivado no nace menos privado
/// que su fuente, y este desciende de los documentos del usuario.
#[cfg(unix)]
const PERMISOS_DEL_DICCIONARIO: u32 = 0o600;

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
    if let Some(padre) = ruta.parent() {
        std::fs::create_dir_all(padre).map_err(|e| format!("no se pudo crear {}: {e}", padre.display()))?;
    }
    if !ruta.exists() {
        nacer_cerrado(ruta, &diccionario::Diccionario::semilla().a_texto())?;
        println!("[diccionario] archivo nuevo con la semilla en {}", ruta.display());
    }
    if cerrar_permisos(ruta)? {
        println!("[diccionario] lo encontró abierto y lo dejó en {PERMISOS_DEL_DICCIONARIO:o}");
    }
    Ok(())
}

/// Crea el archivo **ya con sus permisos puestos**, no con los que le toquen y un apretón después.
///
/// La primera versión usaba `std::fs::write` y el propio gate del efímero lo delató en su traza:
/// «el archivo estaba en 644; se dejó en 600». Funcionaba, y aun así estaba mal: la regla 17-bis dice
/// que un derivado **nace** con permisos restrictivos, y entre el `write` y el `set_permissions` hay
/// una ventana —corta, pero real— en la que el archivo con la jerga del consultor es legible por
/// cualquier cuenta del Mac. Un `create_new` con su modo no tiene esa ventana, y encima falla si
/// alguien creó el archivo entre el `exists()` y aquí.
#[cfg(unix)]
fn nacer_cerrado(ruta: &std::path::Path, contenido: &str) -> Result<(), String> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut f = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(PERMISOS_DEL_DICCIONARIO)
        .open(ruta)
        .map_err(|e| format!("no se pudo crear el diccionario: {e}"))?;
    f.write_all(contenido.as_bytes())
        .map_err(|e| format!("no se pudo escribir el diccionario: {e}"))
}

#[cfg(not(unix))]
fn nacer_cerrado(ruta: &std::path::Path, contenido: &str) -> Result<(), String> {
    std::fs::write(ruta, contenido).map_err(|e| format!("no se pudo escribir el diccionario: {e}"))
}

/// Aprieta los permisos si los encuentra flojos. **Devuelve si hubo que repararlos**, y eso no es
/// un detalle de estilo: es lo que permite probar que el archivo nace cerrado en vez de nacer
/// abierto y cerrarse un instante después.
#[cfg(unix)]
fn cerrar_permisos(ruta: &std::path::Path) -> Result<bool, String> {
    use std::os::unix::fs::PermissionsExt;
    let md = std::fs::metadata(ruta).map_err(|e| e.to_string())?;
    if md.permissions().mode() & 0o777 == PERMISOS_DEL_DICCIONARIO {
        return Ok(false);
    }
    std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(PERMISOS_DEL_DICCIONARIO))
        .map_err(|e| format!("no se pudieron cerrar los permisos del diccionario: {e}"))?;
    Ok(true)
}

#[cfg(not(unix))]
fn cerrar_permisos(_ruta: &std::path::Path) -> Result<bool, String> {
    Ok(false)
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
    ventana::ajustar_banda(&app, alto)
}

/// El asa **al soltarla**: la reunión se vuelve a hacer sitio para la franja nueva.
///
/// Va aparte de [`ajustar_banda`] a propósito. Arrastrar dispara decenas de ajustes por segundo, y
/// cada acople son varias idas y vueltas a OTRO proceso por la Accessibility API: hacerlo en cada
/// cuadro convertiría el arrastre en un tirón y dejaría a la ventana de la reunión parpadeando.
/// Durante el arrastre se mueve lo nuestro; al soltar, lo ajeno.
#[tauri::command]
fn asentar_banda<R: tauri::Runtime>(app: tauri::AppHandle<R>, alto: u32) -> Result<(), String> {
    ventana::ajustar_banda(&app, alto)?;
    let franja = ventana::franja(&app, alto)?;
    let informe = acople::reacoplar(franja, &huella(&app));
    registrar_acople(&app, "reacople", &informe);
    Ok(())
}

/// Cierra la banda **y su relleno**, y devuelve a su sitio lo que el acople encogió. El relleno
/// jamás sobrevive a la banda, y la reunión jamás se queda recortada.
#[tauri::command]
fn cerrar_banda(app: tauri::AppHandle) {
    registrar_acople(&app, "soltar", &acople::soltar(&huella(&app)));
    ventana::cerrar_banda(&app);
}

/// Lo que la banda necesita para dibujar «acoplada» o «sin acople» — y lo que hace que esa
/// palabra sea un hecho comprobado, no una etiqueta fija.
#[derive(Clone, serde::Serialize)]
pub struct EstadoDelAcople {
    pub permiso: bool,
    pub acoplada: bool,
}

/// El nombre del evento con el que la banda se entera de que el acople cambió.
const EVENTO_ACOPLE: &str = "acople";

fn estado_ahora<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> EstadoDelAcople {
    EstadoDelAcople {
        permiso: acople::hay_permiso(),
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

/// Acopla la aplicación que está al frente. La **fase 2** la llamará al detectar la reunión; aquí
/// existe para poder verlo correr.
#[tauri::command]
fn acoplar(app: tauri::AppHandle) -> Result<bool, String> {
    let alto = ventana::alto_actual(&app).unwrap_or(ventana::ALTO_COMPACTA);
    let informe = acople::acoplar(ventana::franja(&app, alto)?, &huella(&app));
    registrar_acople(&app, "acoplar", &informe);
    Ok(informe.acoplada())
}

#[tauri::command]
fn soltar_acople(app: tauri::AppHandle) -> bool {
    let informe = acople::soltar(&huella(&app));
    registrar_acople(&app, "soltar", &informe);
    informe.acoplada()
}

/// Pide el permiso de Accesibilidad: macOS abre su diálogo y lleva a Ajustes del Sistema.
#[tauri::command]
fn pedir_permiso_de_acople() -> bool {
    acople::pedir_permiso()
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

/// La versión del catálogo de clientes de videollamada, para mostrarla al lado de lo que afirma.
#[tauri::command]
fn version_del_catalogo() -> &'static str {
    sesion::VERSION_CATALOGO
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
    println!(
        "[corpus] {cuantos} documentos · {} secciones · {} ilegibles · índice en {}",
        informe.secciones,
        informe.ilegibles,
        informe.donde_vive.as_deref().unwrap_or("memoria")
    );
    Ok(informe)
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
/// una de las siete piezas del corte— y hasta el sprint 002 no había forma de recuperarla sin
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
    idioma_del_consultor: String,
    idioma_del_cliente: String,
) -> Result<escucha::EstadoDeEscucha, String> {
    let mut guardada = estado.0.lock().map_err(|_| "la escucha quedó en mal estado")?;
    if let Some(vieja) = guardada.take() {
        vieja.cortar();
    }
    // Si la banda sigue en pantalla, esto no hace nada: `abrir_banda` es idempotente.
    let la_habian_cortado = app.get_webview_window(ventana::BANDA).is_none();
    match ventana::abrir_banda(&app, ventana::ALTO_COMPACTA) {
        // Que la banda no vuelva no impide escuchar, y callarlo sí sería un problema: el usuario
        // vería el transcript sin banda y no sabría por qué.
        Err(e) => println!("[ventanas] la banda no pudo volver: {e}"),
        // Se dice solo cuando de verdad volvió. Es la única traza de que M4 está cableado, y es
        // por donde se verificó en vivo: sin ella, «vuelve» sería una afirmación sin testigo.
        Ok(()) if la_habian_cortado => println!("[ventanas] la banda estaba cortada: vuelve"),
        Ok(()) => {}
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
    let jerga = diccionario_de_la_sesion(
        &ruta_del_diccionario(&app),
        &escucha::Buscador::vocabulario(&*buscador),
    );
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
            }
            let _ = mango.emit(EVENTO_ESCUCHA, novedad);
        },
    );
    let informe = nueva.estado();
    *guardada = Some(nueva);
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
}

/// Qué vive en memoria ahora mismo por culpa de la escucha. Lo pide la pantalla de Honestidad.
#[tauri::command]
fn estado_de_la_escucha(estado: tauri::State<'_, LaEscucha>) -> Option<escucha::EstadoDeEscucha> {
    estado.0.lock().ok()?.as_ref().map(|e| e.estado())
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
struct IdiomaDelMotor {
    codigo: String,
    disponibilidad: stt::Disponibilidad,
}

/// Lo que el motor de este Mac sabe hacer. **Se pregunta al sistema**, no se lleva una lista
/// escrita que quedaría desfasada con la siguiente versión de macOS.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct QueSabeTranscribir {
    motor: &'static str,
    techo: u32,
    idiomas: Vec<IdiomaDelMotor>,
    /// Si no hay motor, por qué — cerrado; Idioma lo pinta con su frase en los dos idiomas.
    motivo: Option<stt::PorQueNoHayMotor>,
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
    let d = diccionario_de_la_sesion(&ruta, &escucha::Buscador::vocabulario(el_corpus.inner()));
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
                corte::Pieza::ContadorDeRed => red::reiniciar(),
                corte::Pieza::Banda => ventana::cerrar_banda(app),
                corte::Pieza::Acople => {
                    registrar_acople(app, "kill-switch", &acople::soltar(&huella(app)))
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
    let informe = corte::Informe { piezas, bytes_en_red: red::bytes() };
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
        "[acople] {que}: permiso={} app=«{nombre}» ventanas={}",
        informe.permiso, informe.ventanas
    );
    for motivo in &informe.motivos {
        println!("[acople]   · {motivo}");
    }
    let _ = app.emit_to(ventana::BANDA, EVENTO_ACOPLE, estado_ahora(app));
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
            cerrar_banda,
            estado_del_acople,
            acoplar,
            soltar_acople,
            pedir_permiso_de_acople,
            fondo_del_relleno,
            reunion_abierta,
            version_del_catalogo,
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
            modo_solo_audio,
            estado_de_la_voz,
            estado_de_la_pantalla,
            lectura_automatica,
            leer_la_pantalla_ahora
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
            app.manage(ElCorpus::default());
            app.manage(LaPantalla::default());
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

            // El diccionario del consultor: se deja escrito con la semilla si no existía, para que
            // el usuario pueda ir a editarlo. Que falle no impide arrancar — la app funciona sin
            // diccionario y la sesión cae a la semilla— pero se dice.
            if let Err(e) = asegurar_el_diccionario(&ruta_del_diccionario(app.handle())) {
                println!("[diccionario] {e}");
            }

            ventana::abrir_banda(app.handle(), ventana::ALTO_COMPACTA)?;

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
    let a = ficha_vigente(&escucha_viva, &el_corpus, &la_pantalla);
    if a.is_none() {
        println!("[ficha] ⌃⌥A sin turno del cliente: todavía no hay nada que buscar");
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
        Ok(()) => println!("[habla] diciendo la ficha · {} letras · {idioma}", dicho.chars().count()),
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

/// `⌃⌥V` — **enciende o apaga el modo solo audio**, y con él la banda de 44 px.
///
/// Lo llaman la tecla global y nadie más. Al encender lee la ficha vigente, que es lo que el usuario
/// espera de haber pulsado la tecla: si no dijera nada hasta el turno siguiente, parecería que no
/// funcionó.
///
/// **Por qué la tecla es `⌃⌥V` y no el `⌃⌥A` que pedía la orden del sprint:** `⌃⌥A` ya es «ayúdame
/// con esto» desde el sprint 001 — está registrada [`el_atajo_de_ayuda`], dibujada en la banda y
/// escrita en el manual. El panel que el usuario aprobó en la etapa de diseño ya usaba `⌃⌥V`.
/// Desviación declarada en la bitácora.
#[tauri::command]
fn modo_solo_audio(app: tauri::AppHandle) -> habla::LaVoz {
    conmutar_el_modo(&app)
}

/// El trabajo de `⌃⌥V`, **hecho en Rust y no pedido a la banda por un evento**.
///
/// Los otros tres atajos emiten a la banda y la banda actúa, y aquí eso no sirve: este cambia el
/// ALTO de la ventana, y el `⌥⎋` puede haberla cerrado. Un modo que se enciende solo si queda una
/// banda que lo pida no es un modo, es una casualidad.
fn conmutar_el_modo<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> habla::LaVoz {
    let estado = app.state::<LaVozQueSale>();
    let el_corpus = app.state::<ElCorpus>();
    let escucha_viva = app.state::<LaEscucha>();
    let encendida = !estado.encendida.load(Ordering::Relaxed);
    estado.encendida.store(encendida, Ordering::Relaxed);

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
    let alto = if encendida { ventana::ALTO_VOZ } else { ventana::ALTO_COMPACTA };
    if let Err(e) = asentar_banda((*app).clone(), alto) {
        println!("[habla] la banda no pudo ir a {alto} px: {e}");
    }

    if encendida {
        // `⎋` se registra SOLO mientras el modo está encendido. Un Escape global permanente se lo
        // quitaría a la reunión —en Meet es la tecla de salir de pantalla completa— y a todas las
        // demás apps del Mac, para una función que existe unos segundos por ficha.
        con_el_callar(app, true);
        println!("[habla] ⌃⌥V: modo solo audio ENCENDIDO · banda a {} px", ventana::ALTO_VOZ);
        // La ficha vigente se rearma igual que en `pedir_ficha`: con el último turno del cliente.
        // Si no se ha oído nada todavía, no hay nada que decir y el modo queda encendido, esperando.
        match ficha_vigente(&escucha_viva, &el_corpus, &app.state::<LaPantalla>()) {
            Some(a) => decir_la_ficha(app, &a),
            None => println!("[habla] todavía no he oído nada del cliente: el modo queda a la espera"),
        }
    } else {
        con_el_callar(app, false);
        estado.voz.callar();
        println!("[habla] ⌃⌥V: modo solo audio APAGADO · banda a {} px", ventana::ALTO_COMPACTA);
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
/// El interruptor (`encendida`) vive fuera de la lectura y **sobrevive a las sesiones** mientras la
/// app esté abierta: si el usuario la apagó en una reunión con una NDA estricta, la siguiente sesión
/// no la vuelve a encender a sus espaldas. No se guarda en disco —es una preferencia de esta
/// sesión de la app, no del usuario— y arranca encendida, que es lo que la maqueta dibuja.
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
    let (al_leer, al_cambiar) = (app.clone(), app.clone());
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
    match app.global_shortcut().register(el_atajo_de_leer_la_pantalla()) {
        Ok(()) => println!("[pantalla] ⌃⌥L «léela ahora» registrado"),
        Err(e) => println!(
            "[pantalla] NO se pudo registrar ⌃⌥L ({e}): la lectura a petición no va a responder \
             a la tecla; la automática sigue en el interruptor de Sesión"
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
            if YA_SE_ACOPLO.load(Ordering::SeqCst)
                || !acople::hay_permiso()
                || !acople::hay_alguien_al_frente()
            {
                continue;
            }
            let alto = ventana::alto_actual(&mango).unwrap_or(ventana::ALTO_COMPACTA);
            let Ok(franja) = ventana::franja(&mango, alto) else {
                continue;
            };
            let informe = acople::acoplar(franja, &huella(&mango));
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
            el_atajo_de_callar(),
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
