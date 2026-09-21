//! Angel Ghost — núcleo nativo.
//!
//! La frontera que organiza este crate no es técnica, es la regla del efímero verificable:
//!
//! - **Protegidos** (`capture`, `stt`): RAM y nada más. `pnpm verify:ephemeral` barre estos
//!   directorios buscando API de disco y de red, y la CI se pone roja si aparece una.
//! - **Libres** (`corpus`): pueden abrir disco porque manejan lo que ES del usuario —sus
//!   documentos, su índice—, que la regla permite persistir.
//! - **Libres** (`ventana`): geometría y ciclo de vida de las tres ventanas. No toca datos.
//!
//! Todo lo demás vive en la raíz del crate. El sprint 001 va llenando estos módulos por fases.

pub mod capture;
pub mod corpus;
pub mod stt;
pub mod ventana;

/// Abre la banda y su relleno. La **fase 2** la llamará al detectar una reunión; en la fase 1 se
/// llama al arrancar, que es lo que permite mirar la banda de verdad en el gate de fidelidad.
#[tauri::command]
fn abrir_banda(app: tauri::AppHandle, alto: u32) -> Result<(), String> {
    ventana::abrir_banda(&app, alto)
}

/// El asa: ajusta la banda y su relleno a la vez. El webview no cambia su propio tamaño porque
/// entonces el relleno podría quedarse atrás; la geometría de la franja vive en un solo sitio.
#[tauri::command]
fn ajustar_banda(app: tauri::AppHandle, alto: u32) -> Result<(), String> {
    ventana::ajustar_banda(&app, alto)
}

/// Cierra la banda **y su relleno**: el relleno jamás sobrevive a la banda.
#[tauri::command]
fn cerrar_banda(app: tauri::AppHandle) {
    ventana::cerrar_banda(&app);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![abrir_banda, ajustar_banda, cerrar_banda])
        .setup(|app| {
            // El invariante se comprueba ANTES de abrir nada y aborta el arranque si falla:
            // una banda que se abre sin su promesa es peor que una banda que no se abre.
            let declaradas = app.config().app.windows.clone();
            ventana::invariante_de_proteccion(&ventana::proteccion_declarada(&declaradas))
                .map_err(|e| format!("protección de captura: {e}"))?;

            ventana::abrir_banda(app.handle(), ventana::ALTO_COMPACTA)?;

            // El registro va CON RETRASO a propósito. macOS aplica el tamaño y la posición de una
            // ventana en el siguiente turno del hilo principal, así que leerlos justo después de
            // pedirlos devuelve los valores de la configuración, no los aplicados: medido, decía
            // «1440x88 en (15,242)» cuando la ventana acabó en «1470x88 en (0,868)». Un registro
            // que miente es peor que no tenerlo — se usa para decidir.
            let mango = app.handle().clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(600));
                ventana::registrar_geometria(&mango);
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error al arrancar Angel Ghost");
}
