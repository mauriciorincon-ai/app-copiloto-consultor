fn main() {
    tauri_build::build();
    #[cfg(target_os = "macos")]
    compilar_el_puente_de_swift();
}

/// **Los archivos de Swift que forman el puente.** Se compilan JUNTOS, en una sola librería y un
/// solo módulo: los dos exportan símbolos de C y ninguno importa al otro, así que partirlos en dos
/// librerías solo añadiría un `-l` más y una manera nueva de que falte la mitad.
///
/// - `Transcriptor.swift` — la voz que ENTRA (`SpeechAnalyzer`), sprint 001.
/// - `Habla.swift` — la voz que SALE (`AVSpeechSynthesizer`), sprint 002.
#[cfg(target_os = "macos")]
const EL_PUENTE: &[&str] = &["nativo/Transcriptor.swift", "nativo/Habla.swift"];

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
///   compila, arranca y funciona; lo único que pierde es la transcripción, y lo dice en la
///   pantalla de Idioma con ese motivo exacto.
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
