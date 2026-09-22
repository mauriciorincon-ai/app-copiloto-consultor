//! ¿Transcribe de verdad el puente de Swift?
//!
//! Los tests de `src/stt/` prueban la frontera: que un idioma inventado no rompa nada, que el
//! motor ausente no se confunda con silencio. Ninguno prueba lo único que importa — que al meter
//! voz por un lado salga texto por el otro—, porque para eso hace falta audio y un modelo
//! instalado, y eso no es propiedad del código sino de la máquina.
//!
//! Este test vive fuera de `src/` a propósito: lee archivos (los del kit), y `src/stt/` tiene
//! prohibido tocar el disco. La frontera del efímero se respeta también cuando estorba.
//!
//! **Cuándo mide y cuándo no.** Si este Mac tiene el motor y el modelo del idioma, transcribe de
//! verdad y comprueba el texto. Si no los tiene, comprueba **el otro contrato**: que la app diga
//! por qué no puede, en vez de devolver una cadena vacía que la banda pintaría como un turno en
//! blanco. Las dos ramas afirman algo; ninguna pasa por no hacer nada. Cuál de las dos corrió se
//! ve en la salida, que el test imprime siempre.

use app_copiloto_consultor_lib::stt::{motor_de_la_casa, Disponibilidad, Fallo};

/// Lee un WAV PCM de 16 bits mono. Veinte líneas en vez de una dependencia: el kit controla el
/// formato de sus propios archivos, así que no hace falta un lector que entienda cuarenta.
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

/// El hallazgo del spike, convertido en test: el motor escribe las cifras con separador de miles
/// («ISO27.001», «ISO 27,001») y el corpus las tiene sin él. La fase 4 tendrá que normalizarlas
/// antes de buscar, y este test existe para que ese día no parezca un bug nuevo.
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
