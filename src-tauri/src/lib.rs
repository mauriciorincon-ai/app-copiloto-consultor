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
pub mod relleno;
pub mod stt;
pub mod ventana;

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, Manager};

/// Dónde anota el acople lo que encogió, para poder devolverlo **aunque esta sesión termine
/// mal**. En la carpeta de configuración de la app, nunca en el repo ni en un temporal del
/// sistema: un temporal lo barre macOS, y entonces la ventana de la reunión se queda encogida sin
/// que nadie sepa quién lo hizo.
fn huella(app: &tauri::AppHandle) -> PathBuf {
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

fn estado_ahora(app: &tauri::AppHandle) -> EstadoDelAcople {
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

/// Deja constancia de lo que pasó y **se lo cuenta a la banda**, que dibuja «acoplada» o «sin
/// acople» con ese dato. La banda pregunta al montarse y escucha a partir de ahí: preguntar sola
/// la dejaría sondeando cada dos segundos por algo que cambia tres veces en una reunión.
///
/// Lo que va al log son solo metadatos: cuántas ventanas y por qué no las demás. Ni títulos de
/// ventana, ni rutas, ni contenido — la Accessibility API los daría, y no se piden.
fn registrar_acople(app: &tauri::AppHandle, que: &str, informe: &acople::Informe) {
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
        .invoke_handler(tauri::generate_handler![
            abrir_banda,
            ajustar_banda,
            asentar_banda,
            cerrar_banda,
            estado_del_acople,
            acoplar,
            soltar_acople,
            pedir_permiso_de_acople,
            fondo_del_relleno
        ])
        .setup(|app| {
            // El invariante se comprueba ANTES de abrir nada y aborta el arranque si falla:
            // una banda que se abre sin su promesa es peor que una banda que no se abre.
            let declaradas = app.config().app.windows.clone();
            ventana::invariante_de_proteccion(&ventana::proteccion_declarada(&declaradas))
                .map_err(|e| format!("protección de captura: {e}"))?;

            // `setup` corre en el hilo principal, que es donde `NSScreen` se deja preguntar.
            app.manage(FondoDelRelleno(acople::fondo_de_escritorio()));

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
fn arrancar_el_acople(app: &tauri::AppHandle) {
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

fn acoplar_cuando_haya_a_quien(app: &tauri::AppHandle) {
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
