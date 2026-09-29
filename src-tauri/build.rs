/// **Los comandos de la app, declarados** (auditoría del S2, M5). Sin esta lista, Tauri deja que
/// CUALQUIER ventana llame a cualquier comando: la banda —la que pinta texto de terceros encima de la
/// reunión— podía guardar la clave del API, y el relleno —que no puede tener contenido— también.
/// Con ella, cada ventana solo puede lo que su capability (`capabilities/*.json`) le permite, y un
/// permiso mal escrito rompe la compilación. Tiene que ser la misma lista que `generate_handler!`:
/// lo vigila `tests/unit/capabilities.test.ts`.
const COMANDOS: &[&str] = &[
        "ajustar_banda",
        "asentar_banda",
        "estado_del_acople",
        "fondo_del_relleno",
        "reunion_abierta",
        "permisos_de_macos",
        "abrir_ajustes_de",
        "bytes_a_la_red",
        "cortar_todo",
        "empezar_a_escuchar",
        "dejar_de_escuchar",
        "estado_de_la_escucha",
        "turnos_recientes",
        "que_sabe_transcribir",
        "estado_del_diccionario",
        "instalar_idioma",
        "salida_de_audio",
        "elegir_carpeta",
        "indexar_corpus",
        "estado_del_corpus",
        "piezas_del_corte",
        "pedir_ficha",
        "estado_de_la_voz",
        "estado_de_la_pantalla",
        "lectura_automatica",
        "leer_la_pantalla_ahora",
        "radar_de_tu_mac",
        "abrir_lo_que_ve",
        "estado_de_la_ia",
        "redactar_sugerencias",
        "api_externa",
        "guardar_clave_del_api",
        "borrar_clave_del_api",
        "idiomas_de_pista",
        "fijar_idioma_de_pista",
        "lo_que_salio_al_api",
        "cuaderno_de_la_reunion",
        "escribir_nota",
        "anotar_acuerdo",
        "conservar_mis_turnos",
        "guardar_la_reunion",
        "cerrar_sin_guardar",
        "reuniones_guardadas",
        "exportar_reunion",
        "borrar_reunion",
        "fijar_retencion",
        "mostrar_las_notas_en_finder",
        "este_cliente",
        "elegir_cliente",
        "responder_nda",
        "revisar_nda",
        "empezar_solo_notas",
        "ir_a_notas",
        // Sprint 003, fase 2: las propuestas y la bandeja (ADR 016). Solo la ventana principal.
        "guardar_propuesta",
        "descartar_propuesta",
        "fijar_ventana",
        "la_bandeja",
        "abrir_la_bandeja",
        "decidir_en_la_bandeja",
        "decidir_toda_la_bandeja",
        "cambiar_la_ventana",
        "estado_de_la_bandeja",
        // Sprint 003, fase 4: la puerta local para tu agente (ADR 018 §7). Solo la ventana principal.
        "la_puerta",
        "abrir_la_puerta",
        "cerrar_la_puerta",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(COMANDOS)),
    )
    .expect("tauri-build no pudo generar el manifiesto de comandos");
    #[cfg(target_os = "macos")]
    compilar_el_puente_de_swift();
}

/// **Los archivos de Swift que forman el puente.** Se compilan JUNTOS, en una sola librería y un
/// solo módulo: los tres exportan símbolos de C y ninguno importa a otro, así que partirlos en
/// varias librerías solo añadiría `-l` de más y maneras nuevas de que falte un trozo.
///
/// - `Transcriptor.swift` — la voz que ENTRA (`SpeechAnalyzer`), sprint 001.
/// - `Habla.swift` — la voz que SALE (`AVSpeechSynthesizer`), sprint 002.
/// - `Pantalla.swift` — la pantalla que se LEE (`ScreenCaptureKit` + `Vision`), sprint 002.
#[cfg(target_os = "macos")]
const EL_PUENTE: &[&str] = &[
    "nativo/Transcriptor.swift",
    "nativo/Habla.swift",
    "nativo/Pantalla.swift",
    "nativo/Sintesis.swift",
    "nativo/Red.swift",
    "nativo/Llavero.swift",
    // El desbloqueo de las notas guardadas (sprint 003, fase 1, ADR 015 §5).
    "nativo/Desbloqueo.swift",
    // La bandeja, fuera de las copias de Time Machine (auditoría del S3, M2; ADR 016, enmienda 2).
    "nativo/Copias.swift",
];

/// Compila el puente de Swift y lo deja listo para enlazar dentro del binario.
///
/// **Por qué un puente y no una FFI directa.** `SpeechAnalyzer` (macOS 26) es un `actor` de Swift
/// con secuencias asíncronas; no hay selectores de Objective-C que mandar como sí los hay para la
/// Accessibility API del acople. Se probó antes de comprometerse: el enlace estático de una
/// librería Swift dentro de un binario de Rust funciona con las **Command Line Tools** solas, sin
/// Xcode completo (bitácora, fase 3a).
///
/// **Los dos fallos posibles se tratan distinto, y esa asimetría es el gate.**
///
/// - `swiftc` **no está** (un Mac sin herramientas de desarrollo): se avisa y se sigue. La app
///   compila, arranca y funciona; lo que pierde es la transcripción, la voz y la lectura de
///   pantalla, y lo dice en Idioma y en Sesión con ese motivo exacto.
/// - `swiftc` **está y falla**: se rompe la compilación. Ese es el caso peligroso —un error en el
///   Swift, una API que cambió— y el único desenlace inaceptable sería un binario verde sin
///   transcripción y sin nadie enterado. Es la misma lección que el kill-switch: lo que no existe
///   se declara; lo que se rompió se grita.
#[cfg(target_os = "macos")]
fn compilar_el_puente_de_swift() {
    use std::process::Command;

    for archivo in EL_PUENTE {
        println!("cargo:rerun-if-changed={archivo}");
    }

    let salida = std::env::var("OUT_DIR").expect("OUT_DIR");
    let biblioteca = format!("{salida}/libagstt.a");

    let swiftc = Command::new("swiftc")
        .args(["-emit-library", "-static", "-O", "-module-name", "agstt", "-o", &biblioteca])
        .args(EL_PUENTE)
        .output();

    match swiftc {
        Ok(salida_del_compilador) if salida_del_compilador.status.success() => {
            println!("cargo:rustc-link-search=native={salida}");
            println!("cargo:rustc-link-lib=static=agstt");
            // El runtime de Swift no viaja dentro de la librería estática: vive en el sistema.
            println!("cargo:rustc-link-search=native=/usr/lib/swift");
            println!("cargo:rustc-link-arg=-L/usr/lib/swift");
            println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
            println!("cargo:rustc-link-lib=framework=Speech");
            println!("cargo:rustc-link-lib=framework=AVFoundation");
            println!("cargo:rustc-link-lib=framework=ScreenCaptureKit");
            println!("cargo:rustc-link-lib=framework=Vision");
            println!("cargo:rustc-link-lib=framework=CoreGraphics");
            // La síntesis (C7, sprint 002, fase 5): el modelo del sistema.
            println!("cargo:rustc-link-lib=framework=FoundationModels");
            println!("cargo:rustc-link-lib=framework=Security");
            // Touch ID o la contraseña del Mac, para abrir tus notas guardadas (ADR 015 §5).
            println!("cargo:rustc-link-lib=framework=LocalAuthentication");
            println!("cargo:rustc-cfg=puente_de_swift");
        }
        Ok(fallo) => {
            let quejas = String::from_utf8_lossy(&fallo.stderr);
            for linea in quejas.lines() {
                println!("cargo:warning={linea}");
            }
            panic!(
                "swiftc está instalado y no pudo compilar el puente ({}). El puente es parte del \
                 producto: un binario sin él sería una app que no escucha, no habla, y no lo dice. \
                 Arriba están las quejas del compilador.",
                EL_PUENTE.join(" + ")
            );
        }
        Err(e) => {
            println!(
                "cargo:warning=sin swiftc ({e}): la app compila, pero la transcripción local y el \
                 modo solo audio quedan apagados, y lo dirán en la pantalla de Idioma y en el log"
            );
        }
    }
    println!("cargo:rustc-check-cfg=cfg(puente_de_swift)");
}
