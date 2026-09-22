//! **Lo que solo se puede comprobar hablando con el Mac de verdad.**
//!
//! Los tres bloques de este archivo son lo que ningún test unitario puede afirmar: que el puente
//! de Swift transcribe, que los dos grifos de audio se abren, y que una frase que suena por los
//! altavoces acaba siendo texto. Viven fuera de `src/` porque leen archivos del kit y arrancan
//! procesos, y `src/stt/` y `src/capture/` tienen prohibido tocar el disco.
//!
//! **Por qué los tres están en UN archivo y no en tres.** Cada archivo de `tests/` es un binario
//! aparte, y cada binario vuelve a enlazar el crate entero **más la librería de Swift**. Con tres
//! archivos, el job de macOS de la integración continua pasó de 1 min 11 s a **7 min 32 s**, y
//! más de cinco de esos minutos eran enlazar lo mismo tres veces. Medido en la corrida
//! `35673597848`, no supuesto.
//!
//! **Y por eso hay un turno.** En un solo binario los tests corren en paralelo, y estos comparten
//! algo que no se puede compartir: los altavoces del Mac. Sin el turno, el audio que reproduce
//! uno entra por el tap que mide otro.
//!
//! **Cuándo miden y cuándo no.** Necesitan altavoces, permisos y el modelo del idioma. Si falta
//! algo, cada bloque comprueba **lo otro que sí se puede comprobar** —que el motivo de no poder
//! sea una frase que se pueda enseñar— y lo dice por la salida. Ninguna rama pasa por no hacer
//! nada; cuál corrió se ve siempre en el log.

#![cfg(target_os = "macos")]

use std::sync::{Mutex, MutexGuard};

// Los altavoces y el tap del sistema son uno solo para todo el Mac: los tests que los usan van
// de a uno. Sin esto, el `afplay` de un test aparece dentro de las mediciones de otro.
static TURNO: Mutex<()> = Mutex::new(());

fn turno() -> MutexGuard<'static, ()> {
    TURNO.lock().unwrap_or_else(|e| e.into_inner())
}

// =============================================================================================
// el puente de Swift transcribe
// =============================================================================================

// Al meter voz por un lado tiene que salir texto por el otro. Los tests de `src/stt/` prueban
// la frontera —que un idioma inventado no rompa nada, que el motor ausente no se confunda con
// silencio—; ninguno prueba lo único que importa.

use app_copiloto_consultor_lib::stt::{motor_de_la_casa, Disponibilidad, Fallo};

// Lee un WAV PCM de 16 bits mono. Veinte líneas en vez de una dependencia: el kit controla el
// formato de sus propios archivos, así que no hace falta un lector que entienda cuarenta.
fn leer_wav(ruta: &str) -> (Vec<f32>, u32) {
    let bytes = std::fs::read(ruta).unwrap_or_else(|e| panic!("no se pudo leer {ruta}: {e}"));
    assert_eq!(&bytes[0..4], b"RIFF", "{ruta} no es un WAV");
    let u32en = |i: usize| u32::from_le_bytes(bytes[i..i + 4].try_into().unwrap());
    let u16en = |i: usize| u16::from_le_bytes(bytes[i..i + 2].try_into().unwrap());

    let mut i = 12;
    let (mut hz, mut canales, mut bits) = (0u32, 0u16, 0u16);
    loop {
        let id = &bytes[i..i + 4];
        let largo = u32en(i + 4) as usize;
        let cuerpo = i + 8;
        if id == b"fmt " {
            canales = u16en(cuerpo + 2);
            hz = u32en(cuerpo + 4);
            bits = u16en(cuerpo + 14);
        } else if id == b"data" {
            assert_eq!((canales, bits), (1, 16), "{ruta}: se esperaba mono de 16 bits");
            let (pares, _) = bytes[cuerpo..cuerpo + largo].as_chunks::<2>();
            let muestras = pares.iter().map(|p| i16::from_le_bytes(*p) as f32 / 32_768.0).collect();
            return (muestras, hz);
        }
        i = cuerpo + largo + (largo & 1);
    }
}

fn probar(archivo: &str, idioma: &str, esperadas: &[&str]) {
    let (muestras, hz) = leer_wav(&format!("../docs/kit-de-prueba/audio/{archivo}"));
    let motor = motor_de_la_casa();
    let disponibilidad = motor.disponibilidad(idioma);
    println!("motor «{}» · {idioma} · {disponibilidad:?}", motor.nombre());

    match disponibilidad {
        Disponibilidad::Listo => {
            let arranque = std::time::Instant::now();
            let texto = motor
                .transcribir(idioma, &muestras, hz)
                .unwrap_or_else(|e| panic!("el motor estaba listo y aun así falló: {e:?}"));
            let ms = arranque.elapsed().as_millis();
            let segundos = muestras.len() as f32 / hz as f32;
            println!("  {segundos:.2}s de audio → {ms} ms · «{texto}»");
            assert!(!texto.trim().is_empty(), "el motor estaba listo y devolvió texto vacío");
            let bajo = texto.to_lowercase();
            for palabra in esperadas {
                assert!(
                    bajo.contains(&palabra.to_lowercase()),
                    "la transcripción no contiene «{palabra}»: «{texto}»"
                );
            }
        }
        otra => {
            // La otra mitad del contrato: sin motor no hay texto, y el motivo se puede enseñar.
            println!("  no se pudo transcribir aquí, y eso también se comprueba");
            let fallo = motor.transcribir(idioma, &muestras, hz).unwrap_err();
            match fallo {
                Fallo::NoDisponible(d) => assert_eq!(
                    formato(&d),
                    formato(&otra),
                    "el motor da un motivo al preguntar y otro al transcribir"
                ),
                Fallo::Motor(c) => panic!("error {c} del motor cuando ya había dicho que no estaba listo"),
            }
        }
    }
}

fn formato(d: &Disponibilidad) -> String {
    format!("{d:?}")
}

#[test]
fn transcribe_la_pregunta_en_espanol() {
    probar("pregunta-es.wav", "es-ES", &["limpieza de datos", "alcance"]);
}

#[test]
fn transcribe_la_pregunta_en_ingles() {
    probar("pregunta-en.wav", "en-US", &["data cleaning", "scope"]);
}

// El hallazgo del spike, convertido en test: el motor escribe las cifras con separador de miles
// («ISO27.001», «ISO 27,001») y el corpus las tiene sin él. La fase 4 tendrá que normalizarlas
// antes de buscar, y este test existe para que ese día no parezca un bug nuevo.
#[test]
fn las_cifras_llegan_con_separadores_del_idioma() {
    let (muestras, hz) = leer_wav("../docs/kit-de-prueba/audio/pregunta-es.wav");
    let motor = motor_de_la_casa();
    if !matches!(motor.disponibilidad("es-ES"), Disponibilidad::Listo) {
        println!("sin motor en esta máquina: no hay cifras que mirar");
        return;
    }
    let texto = motor.transcribir("es-ES", &muestras, hz).unwrap();
    let crudo = texto.to_lowercase();
    println!("cifras tal y como llegan: «{texto}»");
    assert!(
        !crudo.contains("iso 27001"),
        "el motor ya escribe las cifras sin separador: la normalización de la fase 4 sobra — \
         compruébalo antes de quitarla, y si sobra, borra este test"
    );
}

// =============================================================================================
// los dos grifos se abren
// =============================================================================================

// Afirma que los grifos **se abren** —o que, si no, dicen por qué con una frase legible—. No
// afirma que lleguen muestras: cuando no suena nada el sistema no llama al callback ni una vez,
// así que exigir muestras convertiría el silencio de una habitación en un fallo de la suite.

use app_copiloto_consultor_lib::capture::anillo::Anillo;
use app_copiloto_consultor_lib::capture::nativo::Grifo;
use std::sync::Arc;
use std::time::Duration;

fn escuchar(que: &str, abrir: impl FnOnce(Arc<Mutex<Anillo>>) -> Result<Grifo, String>) {
    let _turno = turno();
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

// =============================================================================================
// de la voz a la frase, de punta a punta
// =============================================================================================

// Suena una frase por los altavoces; el tap la capta; el detector la corta en turno; el motor la
// transcribe. Es el único sitio donde las cuatro piezas de la fase 3 se tocan, y la respuesta a
// la pregunta que ningún test unitario contesta: *¿funciona en el modo en que lo va a usar el
// usuario?*

use app_copiloto_consultor_lib::capture::Pista;
use app_copiloto_consultor_lib::escucha::{Escucha, Novedad, SinCorpus};
use std::sync::mpsc;
use std::time::Instant;

#[test]
fn una_frase_por_los_altavoces_acaba_siendo_texto() {
    let _turno = turno();
    let motor = motor_de_la_casa();
    let hay_motor = matches!(motor.disponibilidad("es-ES"), Disponibilidad::Listo);
    println!("motor «{}» · es-ES listo: {hay_motor}", motor.nombre());

    let (manda, recibe) = mpsc::channel();
    // Sin corpus: lo que este test comprueba es que una frase por los altavoces acaba siendo
    // texto. La ficha tiene su propio camino y sus propias pruebas.
    let escucha = Escucha::arrancar("es-ES", "es-ES", motor, std::sync::Arc::new(SinCorpus), move |n| {
        let _ = manda.send(n);
    });

    let estado = escucha.estado();
    println!(
        "pistas · micrófono abierto={} · sistema abierto={} ({})",
        estado.microfono.abierta,
        estado.sistema.abierta,
        estado.sistema.motivo.clone().unwrap_or_else(|| "sin motivo".into())
    );
    if !estado.sistema.abierta {
        let motivo = estado.sistema.motivo.unwrap_or_default();
        assert!(motivo.len() > 20, "el tap no abrió y el motivo «{motivo}» no explica nada");
        return;
    }

    // Suena la pregunta del cliente. `afplay` vuelve en cuanto termina de sonar.
    let arranque = Instant::now();
    let reproductor = std::process::Command::new("afplay")
        .arg("../docs/kit-de-prueba/audio/pregunta-es.wav")
        .spawn();
    let Ok(mut reproductor) = reproductor else {
        println!("este Mac no puede reproducir audio: no hay nada que escuchar");
        return;
    };
    let _ = reproductor.wait();
    println!("la frase sonó en {} ms", arranque.elapsed().as_millis());

    // El turno se cierra 320 ms después del último sonido y transcribirlo cuesta ~250 ms más, así
    // que con un segundo bastaría. La espera es de doce **y se corta en cuanto llega el turno del
    // cliente**, que es lo que este test viene a ver: una máquina cargada —recién compilando, o
    // una de integración continua— puede tardar mucho más que este Mac en reposo, y un test que
    // se rinde antes de tiempo falla por el reloj y no por el código. Pasó una vez dentro de un
    // `cargo test` completo: los seis segundos que había no alcanzaron.
    let hasta = Instant::now() + Duration::from_secs(12);
    let mut turnos = Vec::new();
    let mut sin_texto = Vec::new();
    while Instant::now() < hasta {
        if turnos.iter().any(|t: &app_copiloto_consultor_lib::stt::Turno| t.pista == Pista::Sistema) {
            break;
        }
        match recibe.recv_timeout(Duration::from_millis(300)) {
            Ok(Novedad::Turno(t)) => {
                println!(
                    "turno · {:?} · {}–{} ms · «{}»",
                    t.pista, t.desde_ms, t.hasta_ms, t.texto
                );
                turnos.push(t);
            }
            Ok(Novedad::SinTexto { pista, motivo, .. }) => {
                println!("turno sin texto · {pista:?} · {motivo}");
                sin_texto.push(motivo);
            }
            Ok(otra) => println!("· {otra:?}"),
            Err(_) => {}
        }
    }
    let llegado = escucha.estado().sistema.muestras_recibidas;
    escucha.cortar();
    println!("el tap recibió {llegado} muestras");

    // Una máquina sin altavoces de verdad —una de integración continua, por ejemplo— abre el tap
    // igual y no recibe ni una muestra. No es un fallo de la app y no se puede disimular como si
    // lo fuera: se dice, y se comprueba lo único que aquí sigue siendo cierto.
    if llegado == 0 {
        println!("no sonó nada por la salida de audio de esta máquina: no hay turnos que juzgar");
        assert!(turnos.is_empty(), "sin una sola muestra salieron turnos: {turnos:?}");
        return;
    }

    let del_cliente: Vec<_> = turnos.iter().filter(|t| t.pista == Pista::Sistema).collect();
    if !hay_motor {
        assert!(
            !sin_texto.is_empty(),
            "sin modelo instalado no salió ni un turno ni un motivo: la escucha se quedó muda en \
             silencio, que es la única forma de fallar que esta app no se permite"
        );
        return;
    }
    assert!(
        !del_cliente.is_empty(),
        "sonó una frase por los altavoces y no salió ningún turno del cliente (sin texto: {sin_texto:?})"
    );
    let todo = del_cliente.iter().map(|t| t.texto.to_lowercase()).collect::<Vec<_>>().join(" ");
    assert!(
        todo.contains("limpieza de datos") || todo.contains("alcance") || todo.contains("certificación"),
        "se transcribió algo que no se parece a la frase: «{todo}».\n\
         El tap oye TODO lo que suena en este Mac: si había música, una notificación o —como pasó \
         escribiendo esto— otro `afplay` a la vez, lo transcrito es la mezcla. Silencia el Mac y \
         vuelve a correrlo antes de buscar el fallo en el código."
    );
}
