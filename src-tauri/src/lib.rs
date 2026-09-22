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
pub mod disparo;
pub mod escucha;
pub mod ficha;
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

/// La ruta del fondo de escritorio, leída **una vez, en el hilo principal** (`NSScreen` lo exige).
/// Se guarda la RUTA y no la imagen: codificarla en base64 son megabytes vivos durante toda la
/// sesión para pintar una franja de 88 px que el relleno pide una sola vez.
struct FondoDelRelleno(Option<PathBuf>);

/// Abre la banda y su relleno. La **fase 2** la llamará al detectar una reunión; en la fase 1 se
/// llama al arrancar, que es lo que permite mirar la banda de verdad en el gate de fidelidad.
#[tauri::command]
fn abrir_banda(app: tauri::AppHandle, alto: u32) -> Result<(), String> {
    ventana::abrir_banda(&app, alto)
}

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
fn asentar_banda(app: tauri::AppHandle, alto: u32) -> Result<(), String> {
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
struct EstadoDelAcople {
    permiso: bool,
    acoplada: bool,
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

/// Los documentos, uno a uno, con su unidad y su estado.
#[tauri::command]
fn documentos_del_corpus(estado: tauri::State<'_, ElCorpus>) -> Vec<corpus::Documento> {
    estado
        .0
        .lock()
        .ok()
        .and_then(|g| g.as_ref().map(|c| c.documentos().to_vec()))
        .unwrap_or_default()
}

/// Empieza a escuchar las dos pistas.
///
/// **No se llama sola al arrancar**, y es deliberado: la maqueta de la pantalla de Sesión dice
/// «nada se enciende hasta que tú lo digas», y encender el micrófono de alguien sin que lo pida
/// sería exactamente lo que esta app promete no hacer.
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
    let mango = app.clone();
    let nueva = escucha::Escucha::arrancar(
        &idioma_del_consultor,
        &idioma_del_cliente,
        stt::motor_de_la_casa(),
        std::sync::Arc::new(el_corpus.inner().clone()),
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
            let _ = mango.emit(EVENTO_ESCUCHA, novedad);
        },
    );
    let informe = nueva.estado();
    *guardada = Some(nueva);
    Ok(informe)
}

#[tauri::command]
fn dejar_de_escuchar(estado: tauri::State<'_, LaEscucha>) {
    if let Ok(mut g) = estado.0.lock() {
        if let Some(e) = g.take() {
            e.cortar();
            println!("[escucha] parada a petición del usuario");
        }
    }
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
    /// Si no hay motor, por qué. En español, para enseñarlo tal cual.
    motivo: Option<String>,
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
    .unwrap_or(stt::Disponibilidad::SinMotor {
        motivo: "la instalación se interrumpió".into(),
    })
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
                corte::Pieza::ContadorDeRed => red::reiniciar(),
                corte::Pieza::Banda => ventana::cerrar_banda(app),
                corte::Pieza::Acople => {
                    registrar_acople(app, "kill-switch", &acople::soltar(&huella(app)))
                }
                // Ya cortadas arriba, todas a la vez.
                corte::Pieza::AudioDelMicrofono
                | corte::Pieza::AudioDelSistema
                | corte::Pieza::Transcript => {}
                corte::Pieza::UltimoFrame => {}
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
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(atender_el_atajo).build())
        .invoke_handler(tauri::generate_handler![
            abrir_banda,
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
            instalar_idioma,
            salida_de_audio,
            indexar_corpus,
            estado_del_corpus,
            documentos_del_corpus,
            pedir_ficha
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

            registrar_el_kill_switch(app.handle());

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
        "[permisos] micrófono={:?} pantalla={:?} accesibilidad={:?} · cara={:?}",
        p.microfono,
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
        sesion::Reunion::NoSePuedeSaber { motivo } => println!("[sesion] no se puede saber: {motivo}"),
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
        que.motivo.map(|m| format!(" · {m}")).unwrap_or_default()
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

/// `⌘⇧T` — enseñar u ocultar el transcript, tal y como lo dibuja `idioma.html`.
///
/// **Y una advertencia que no se puede callar:** en Chrome, Safari y Firefox esta combinación
/// vuelve a abrir la última pestaña cerrada. Registrarla globalmente se la quita al navegador
/// mientras Angel Ghost esté abierto — y el navegador es donde vive la reunión de Meet. Se
/// registra porque es lo que el diseño aprobó, se deja dicho aquí y en el log, y la decisión de
/// cambiarla es del usuario (riesgo nº 7 del plan: atajo configurable).
fn el_atajo_del_transcript() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyT)
}

/// `⌘⇧A` — «ayúdame con esto», el atajo que la maqueta dibuja en la banda.
///
/// Es la salida cuando el disparador automático no acierta, y por eso su camino es distinto: se
/// salta la espera entre fichas y la regla de no repetir. Si el usuario lo pulsa dos veces
/// seguidas es porque la primera no le sirvió, y contestarle con silencio sería lo peor posible
/// justo en el momento en que decidió pedir ayuda a mano.
fn el_atajo_de_ayuda() -> tauri_plugin_global_shortcut::Shortcut {
    use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
    Shortcut::new(Some(Modifiers::SUPER | Modifiers::SHIFT), Code::KeyA)
}

/// El nombre del evento con el que la banda se entera de que hay que enseñar u ocultar el
/// transcript.
const EVENTO_TRANSCRIPT: &str = "transcript";

/// El nombre del evento con el que la banda recibe una ficha.
const EVENTO_FICHA: &str = "ficha";

/// La ficha a petición del usuario: `⌘⇧A`, o el botón de la banda ampliada.
///
/// Busca con **el último turno del cliente**, que es de lo que se estaba hablando. Sin turnos no
/// hay con qué buscar, y eso se dice en vez de devolver una ficha vacía.
#[tauri::command]
fn pedir_ficha(
    escucha_viva: tauri::State<'_, LaEscucha>,
    el_corpus: tauri::State<'_, ElCorpus>,
) -> Result<ficha::Aparicion, String> {
    use escucha::Buscador;
    let ultimo = escucha_viva
        .0
        .lock()
        .map_err(|_| "la escucha quedó en mal estado")?
        .as_ref()
        .and_then(|e| {
            e.ultimos_turnos(6)
                .into_iter()
                .rev()
                .find(|t| t.pista == capture::Pista::Sistema && !t.eco)
        })
        .ok_or("todavía no he oído nada del cliente")?;

    let empezo = std::time::Instant::now();
    let buscador = el_corpus.inner().clone();
    let hallazgos = buscador.buscar(&ultimo.texto, ficha::TOP);
    let respuesta = ficha::armar(&ultimo.texto, &hallazgos);
    let ms = empezo.elapsed().as_millis() as u64;
    println!("[ficha] a petición del usuario en {ms} ms · {} candidatas", hallazgos.len());
    Ok(ficha::Aparicion { respuesta, motivo: disparo::Motivo::Atajo, ms, hora: ultimo.hora })
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
        println!("[transcript] ⌘⇧T");
        let _ = app.emit_to(ventana::BANDA, EVENTO_TRANSCRIPT, ());
    } else if *atajo == el_atajo_de_ayuda() {
        println!("[ficha] ⌘⇧A");
        let _ = app.emit_to(ventana::BANDA, EVENTO_FICHA, ());
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
        Ok(()) => println!(
            "[transcript] ⌘⇧T registrado · OJO: mientras Angel Ghost esté abierto, el navegador \
             deja de reabrir la última pestaña cerrada con esa tecla"
        ),
        Err(e) => println!(
            "[transcript] NO se pudo registrar ⌘⇧T ({e}): el transcript no se va a poder abrir \
             con la tecla"
        ),
    }
    match app.global_shortcut().register(el_atajo_de_ayuda()) {
        Ok(()) => println!("[ficha] ⌘⇧A «ayúdame con esto» registrado"),
        Err(e) => println!(
            "[ficha] NO se pudo registrar ⌘⇧A ({e}): la ficha a petición sigue en el botón de la \
             banda ampliada, pero la tecla no va a responder"
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
