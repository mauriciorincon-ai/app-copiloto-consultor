//! ¿Se abren de verdad los dos grifos de audio?
//!
//! Igual que el del puente de Swift, este test vive fuera de `src/` porque necesita hablar con el
//! Mac de verdad: dispositivos, permisos y un tap del sistema no caben en un test unitario.
//!
//! **Lo que afirma y lo que no.** Afirma que los grifos **se abren** —o que, si no, dicen por qué
//! con una frase legible—. No afirma que lleguen muestras, y esa distinción es el hallazgo del
//! spike convertido en código: cuando no suena nada, el sistema no llama al callback ni una vez,
//! así que exigir muestras convertiría el silencio de una habitación en un fallo de la suite. Lo
//! que llega se imprime, para que quien corra esto en su Mac lo vea.

#![cfg(target_os = "macos")]

use app_copiloto_consultor_lib::capture::anillo::Anillo;
use app_copiloto_consultor_lib::capture::nativo::Grifo;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn escuchar(que: &str, abrir: impl FnOnce(Arc<Mutex<Anillo>>) -> Result<Grifo, String>) {
    let anillo = Arc::new(Mutex::new(Anillo::de_la_app()));
    match abrir(anillo.clone()) {
        Ok(grifo) => {
            println!("{que}: abierto · el dispositivo entrega {} Hz", grifo.hz_del_dispositivo);
            std::thread::sleep(Duration::from_millis(1_200));
            let dentro = anillo.lock().unwrap();
            println!(
                "  {} muestras recibidas · {:.2}s en el anillo · {} bytes",
                grifo.muestras_recibidas(),
                dentro.segundos(),
                dentro.bytes()
            );
            assert!(grifo.hz_del_dispositivo >= 8_000, "una frecuencia de {} Hz no es audio", grifo.hz_del_dispositivo);
        }
        Err(porque) => {
            println!("{que}: NO se pudo abrir — {porque}");
            assert!(
                porque.len() > 20 && !porque.contains("None") && !porque.contains("Err("),
                "el motivo «{porque}» no le dice nada a nadie: un grifo que no abre tiene que \
                 explicarse, o la pantalla de Sesión no tendrá qué enseñar"
            );
        }
    }
}

#[test]
fn el_microfono_se_abre_o_dice_por_que_no() {
    escuchar("micrófono", Grifo::del_microfono);
}

#[test]
fn el_audio_del_sistema_se_abre_o_dice_por_que_no() {
    escuchar("audio del sistema", Grifo::del_sistema);
}
