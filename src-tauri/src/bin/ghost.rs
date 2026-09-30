//! `ghost` — la puerta local de Angel Ghost, del lado de tu agente (C16, ADR 018 §8).
//!
//! Un cliente fino: interpreta los argumentos (`puerta::cli`), mira si la puerta está abierta, lee
//! la llave del Llavero, manda la orden por el socket (`puerta::socket::pedir`) e imprime la
//! respuesta en JSON. La política vive en la app: aquí no se decide nada.
//!
//! **El orden importa, y lo vigila `puerta_antes_que_el_llavero`:** primero el socket, después el
//! Llavero. Con la puerta cerrada, `ghost` lo dice sin pedirle nada a macOS: ningún diálogo del
//! Llavero por una orden que no iba a entrar. `--version` y `--help` no tocan ni lo uno ni lo otro.

use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use app_copiloto_consultor_lib::corpus::evaluar::Kit;
use app_copiloto_consultor_lib::llavero::{self, Servicio};
use app_copiloto_consultor_lib::puerta::cli::{self, salida, Pedido};
use app_copiloto_consultor_lib::puerta::socket::{self, NoLlega};
use app_copiloto_consultor_lib::puerta::{Motivo, Orden, Respuesta, CUENTA};

/// Lo que espera la respuesta: reindexar una carpeta grande o esperar el dedo en Touch ID lleva su
/// tiempo, y una orden a medio hacer no se corta por impaciencia del cliente.
const ESPERA: Duration = Duration::from_secs(300);

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let idioma = cli::idioma(std::env::var("LANG").ok().as_deref());
    ExitCode::from(correr(&args, idioma) as u8)
}

fn correr(args: &[String], idioma: &str) -> i32 {
    let en = idioma == "en";
    let orden = match cli::interpretar(args) {
        Ok(Pedido::Version) => {
            println!("ghost {}", cli::VERSION);
            return salida::HECHO;
        }
        Ok(Pedido::Ayuda) => {
            print!("{}", cli::ayuda(idioma));
            return salida::HECHO;
        }
        Ok(Pedido::Orden(o)) => o,
        Ok(Pedido::Kit(archivo)) => match leer_el_kit(&archivo) {
            Ok(kit) => Orden::Kit { kit },
            Err(e) => {
                eprintln!("{e}");
                return salida::FALLO;
            }
        },
        Err(e) => {
            eprintln!("{e}");
            return salida::USO;
        }
    };

    let Some(casa) = std::env::var_os("HOME") else {
        eprintln!("{}", if en { "HOME is not set" } else { "falta HOME" });
        return salida::FALLO;
    };
    let ruta = socket::ruta_en(&socket::carpeta_de_la_app(Path::new(&casa)));

    // Primero la puerta, después el Llavero.
    if !ruta.exists() {
        return cerrada(en);
    }
    let Some(mut token) = llavero::leer(Servicio::Puerta, CUENTA) else {
        eprintln!(
            "{}",
            if en {
                "The door is open but ghost could not read its key from the Keychain (did you deny it?)."
            } else {
                "La puerta está abierta, pero ghost no pudo leer su llave del Llavero (¿la denegaste?)."
            }
        );
        return salida::FALLO;
    };
    let respuesta = socket::pedir(&ruta, &token, &orden, ESPERA);
    // SEGURIDAD: ceros sobre ASCII siguen siendo UTF-8 válido.
    unsafe { token.as_mut_vec() }.fill(0);

    match respuesta {
        Ok(Respuesta::Hecho { datos }) => {
            println!("{}", serde_json::to_string_pretty(&datos).unwrap_or_default());
            salida::HECHO
        }
        Ok(r @ Respuesta::Denegado { motivo }) => {
            println!("{}", serde_json::to_string(&r).unwrap_or_default());
            eprintln!("{}", por_que(motivo, en));
            salida::DENEGADO
        }
        Ok(r @ Respuesta::Fallo { .. }) => {
            println!("{}", serde_json::to_string(&r).unwrap_or_default());
            salida::FALLO
        }
        Err(NoLlega::Cerrada) => cerrada(en),
        Err(NoLlega::Fallo(e)) => {
            eprintln!("{e}");
            salida::FALLO
        }
    }
}

fn cerrada(en: bool) -> i32 {
    eprintln!(
        "{}",
        if en {
            "The door is closed. Open it in Angel Ghost → AI → Local door."
        } else {
            "La puerta está cerrada. Ábrela en Angel Ghost → IA → Puerta local."
        }
    );
    salida::CERRADA
}

fn por_que(motivo: Motivo, en: bool) -> &'static str {
    match (motivo, en) {
        (Motivo::LlaveErrada, false) => "Denegado: la llave no es la de esta apertura.",
        (Motivo::LlaveErrada, true) => "Denied: the key is not this opening's key.",
        (Motivo::EnReunion, false) => "Denegado: hay una reunión. La puerta se cerró sola; se vuelve a abrir a mano, en IA.",
        (Motivo::EnReunion, true) => "Denied: there is a meeting. The door closed itself; it is reopened by hand, in AI.",
        (Motivo::ElApiEsTuyo, false) => "Denegado: encender el API externo es decisión del usuario.",
        (Motivo::ElApiEsTuyo, true) => "Denied: turning on the external API is the user's call.",
        (Motivo::NoDelegable, false) => "Denegado: esa preferencia la decide el usuario.",
        (Motivo::NoDelegable, true) => "Denied: that preference is the user's call.",
        (Motivo::OrdenDesconocida, false) => "Denegado: la puerta no conoce esa orden.",
        (Motivo::OrdenDesconocida, true) => "Denied: the door does not know that order.",
    }
}

/// El kit lo lee `ghost`, no la app: la app no abre rutas que le dicte el agente (ADR 018 §4).
fn leer_el_kit(archivo: &Path) -> Result<Kit, String> {
    let texto = std::fs::read_to_string(archivo).map_err(|e| format!("no se pudo leer {}: {e}", archivo.display()))?;
    serde_json::from_str(&texto).map_err(|e| format!("{} no tiene la forma del kit: {e}", archivo.display()))
}
