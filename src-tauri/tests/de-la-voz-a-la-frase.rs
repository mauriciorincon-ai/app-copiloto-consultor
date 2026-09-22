//! **La prueba entera, de punta a punta, con el Mac de verdad.**
//!
//! Suena una frase por los altavoces; el tap del sistema la capta; el detector la corta en turno;
//! el motor la transcribe. Es el único sitio donde las cuatro piezas de la fase 3 se tocan, y es
//! la respuesta a la pregunta que ningún test unitario puede contestar: *¿esto funciona en el modo
//! en que lo va a usar el usuario?*
//!
//! **Cuándo mide.** Necesita altavoces, permiso de captura de audio y el modelo del idioma. Si
//! falta algo, lo dice y comprueba lo otro que sí se puede comprobar — que el motivo de no poder
//! sea una frase que se pueda enseñar—. Qué rama corrió se ve siempre en la salida.

#![cfg(target_os = "macos")]

use app_copiloto_consultor_lib::capture::Pista;
use app_copiloto_consultor_lib::escucha::{Escucha, Novedad};
use app_copiloto_consultor_lib::stt::{motor_de_la_casa, Disponibilidad};
use std::sync::mpsc;
use std::time::{Duration, Instant};

#[test]
fn una_frase_por_los_altavoces_acaba_siendo_texto() {
    let motor = motor_de_la_casa();
    let hay_motor = matches!(motor.disponibilidad("es-ES"), Disponibilidad::Listo);
    println!("motor «{}» · es-ES listo: {hay_motor}", motor.nombre());

    let (manda, recibe) = mpsc::channel();
    let escucha = Escucha::arrancar("es-ES", "es-ES", motor, move |n| {
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

    // El turno se cierra 320 ms después del último sonido, y transcribirlo cuesta ~250 ms más.
    let hasta = Instant::now() + Duration::from_secs(6);
    let mut turnos = Vec::new();
    let mut sin_texto = Vec::new();
    while Instant::now() < hasta {
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
        "se transcribió algo que no se parece a la frase: «{todo}»"
    );
}
