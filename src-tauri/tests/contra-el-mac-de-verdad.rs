//! **Lo que solo se puede comprobar contra el Mac y el disco de verdad.**
//!
//! Los bloques de este archivo son lo que ningún test unitario puede afirmar: que el puente de
//! Swift transcribe, que los dos grifos de audio se abren, que una frase que suena por los
//! altavoces acaba siendo texto, y que una carpeta de documentos de verdad —Markdown y un PDF
//! hecho con las herramientas del propio macOS— acaba siendo una ficha con su fuente. Viven
//! fuera de `src/` porque leen archivos y arrancan procesos, y `src/stt/`, `src/capture/` y
//! `src/ficha/` tienen prohibido tocar el disco.
//!
//! **Por qué los tres están en UN archivo y no en tres.** Cada archivo de `tests/` es un binario
//! aparte, y cada binario vuelve a enlazar el crate entero **más la librería de Swift**. Con tres
//! archivos, el job de macOS de la integración continua pasó de 1 min 11 s a **7 min 32 s**, y
//! más de cinco de esos minutos eran enlazar lo mismo tres veces. Medido en la corrida
//! `35673597848`, no supuesto.
//!
//! **Y por eso hay un turno, que lo toman TODOS.** En un solo binario los tests corren en
//! paralelo, y aquí hay dos cosas que no se pueden compartir: los altavoces del Mac, y **el
//! disco mientras alguien lo está midiendo**. El gate del efímero inventaría el disco antes y
//! después de una sesión; cualquier archivo que otro test cree mientras tanto aparecería como
//! una fuga. Los del corpus no necesitan hardware, pero sí necesitan estarse quietos durante
//! esa medición, y cuesta menos de un segundo.
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
    let escucha = Escucha::arrancar(
        "es-ES",
        "es-ES",
        motor,
        std::sync::Arc::new(SinCorpus),
        // Sin jerga: este test mide el camino del audio, no la corrección del transcript.
        std::sync::Arc::new(app_copiloto_consultor_lib::diccionario::Diccionario::default()),
        move |n| {
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

// ═══════════════════════════════════════════════════ el corpus, contra archivos de verdad

use std::path::PathBuf;

use app_copiloto_consultor_lib::corpus::{Corpus, Unidad};
use app_copiloto_consultor_lib::ficha::{armar, Respuesta};

fn corpus_sintetico() -> PathBuf {
    let c = std::env::temp_dir().join(format!("ag-corpus-vivo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&c);
    std::fs::create_dir_all(c.join("casos")).unwrap();
    std::fs::write(
        c.join("Propuesta Páramo Azul · rentabilidad por canal.md"),
        "# Alcance\nCubre perfilado y limpieza de tres fuentes: ERP, POS y el Excel de canal.\n\n\
         # Precio\nTarifa cerrada. El precio incluye el taller de cierre y dos rondas de revisión.\n\n\
         # Plazo de entrega\nLa entrega completa toma cuatro semanas contadas desde la firma.\n",
    )
    .unwrap();
    std::fs::write(
        c.join("casos/Cooperativa Sur del Valle · cierre de caso.md"),
        "# Resultados\nLa implementación cerró con dos semanas de retraso y sin sobrecosto.\n",
    )
    .unwrap();
    std::fs::write(
        c.join("Adopción de datos en cuatro etapas.md"),
        "# Etapas\nEl marco recorre cuatro etapas de adopción de datos: inventario, calidad, gobierno y uso.\n",
    )
    .unwrap();
    c
}

#[test]
fn de_una_carpeta_de_verdad_a_una_ficha_con_su_fuente() {
    let _turno = turno();
    let carpeta = corpus_sintetico();
    let mut corpus = Corpus::en_memoria().unwrap();
    corpus.indexar(&carpeta, &|_| {}).unwrap();

    let e = corpus.estado();
    assert_eq!(e.documentos, 3, "{:?}", corpus.documentos());
    assert_eq!(e.ilegibles, 0);

    // Las unidades salen del nombre del archivo, que es como el usuario las guarda.
    let unidad = |trozo: &str| {
        corpus.documentos().iter().find(|d| d.nombre.contains(trozo)).unwrap().unidad
    };
    assert_eq!(unidad("Propuesta"), Some(Unidad::Propuesta));
    assert_eq!(unidad("Cooperativa"), Some(Unidad::Caso));
    assert_eq!(unidad("Adopción"), Some(Unidad::Marco));

    // Y la ficha: la pregunta del cliente, tal y como la escribiría el transcriptor.
    let pregunta = "¿En cuántas semanas hacen la entrega completa?";
    let hallazgos = corpus.buscar(pregunta, 3).unwrap();
    let Respuesta::Ficha(f) = armar(pregunta, &hallazgos) else {
        panic!("no encontró el plazo que sí está en el corpus")
    };
    assert_eq!(f.fuente.seccion.as_deref(), Some("Plazo de entrega"));
    assert_eq!(f.fuente.unidad, Some(Unidad::Propuesta));
    assert!(f.linea.contains("cuatro semanas"), "la línea no responde: {}", f.linea);

    let _ = std::fs::remove_dir_all(&carpeta);
}

/// El otro lado, y el que más se va a ver: el corpus no tiene nada de lo que preguntan. La app
/// **dice qué buscó** y ofrece una maniobra, en vez de enseñar la sección menos mala con su
/// fuente concreta debajo — que es el fallo más caro que esta app puede cometer.
#[test]
fn lo_que_no_esta_en_el_corpus_se_declara_en_vez_de_aproximarse() {
    let _turno = turno();
    let carpeta = corpus_sintetico();
    let mut corpus = Corpus::en_memoria().unwrap();
    corpus.indexar(&carpeta, &|_| {}).unwrap();

    let pregunta = "¿Ustedes tienen certificación ISO 27001?";
    let hallazgos = corpus.buscar(pregunta, 3).unwrap();
    match armar(pregunta, &hallazgos) {
        Respuesta::SinResultado { buscado, maniobra, .. } => {
            assert!(buscado.contains("27001"), "no dice qué buscó: «{buscado}»");
            assert_eq!(maniobra, "credencial");
        }
        Respuesta::Ficha(f) => panic!("aproximó una ficha sobre algo que no tiene: {f:?}"),
    }
    let _ = std::fs::remove_dir_all(&carpeta);
}

/// Un PDF de verdad, hecho con las herramientas del propio macOS. No trae títulos: lo que el
/// lector marca es conjetura, y el documento **tiene que declararlo** hasta la ficha.
#[test]
fn un_pdf_de_verdad_se_lee_y_declara_que_sus_secciones_son_conjetura() {
    let _turno = turno();
    let carpeta = std::env::temp_dir().join(format!("ag-pdf-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&carpeta);
    std::fs::create_dir_all(&carpeta).unwrap();

    let txt = carpeta.join("fuente.txt");
    std::fs::write(
        &txt,
        "Perfil profesional\nQuince años en consultoría de datos para retail y cooperativas.\n\
         Certificaciones\nNinguna certificación ISO vigente a la fecha.\n",
    )
    .unwrap();
    let pdf = carpeta.join("Mi perfil y trayectoria.pdf");
    let salida = std::process::Command::new("cupsfilter").arg(&txt).output().expect("cupsfilter");
    std::fs::write(&pdf, salida.stdout).unwrap();
    std::fs::remove_file(&txt).unwrap();

    let mut corpus = Corpus::en_memoria().unwrap();
    corpus.indexar(&carpeta, &|_| {}).unwrap();
    let doc = &corpus.documentos()[0];
    assert_eq!(doc.unidad, Some(Unidad::Perfil), "{doc:?}");
    assert!(doc.conjeturado, "un PDF declaró sus secciones como si alguien las hubiera escrito");

    let hallazgos = corpus.buscar("¿cuántos años de experiencia tienen en retail?", 3).unwrap();
    assert!(!hallazgos.is_empty(), "no encontró nada dentro del PDF");
    assert!(hallazgos[0].conjeturado);

    let _ = std::fs::remove_dir_all(&carpeta);
}

// ═══════════════════════════════════════════ el efímero, verificado EN MARCHA

use std::collections::BTreeMap;
use std::path::Path;

use app_copiloto_consultor_lib::disparo::{Contexto, Disparador};
use app_copiloto_consultor_lib::stt::Turno;
use app_copiloto_consultor_lib::voz::turno::{Suceso, Turnos};
use app_copiloto_consultor_lib::voz::vad::PorEnergia;

/// La frase que solo dice el cliente. No se parece a nada del corpus ni del código, para que
/// encontrarla en un archivo signifique una sola cosa.
const CANARIA: &str = "quetzalcoatlus-de-bolsillo-7731";

/// Lo único que una sesión puede dejar escrito, y por qué.
///
/// **Cada entrada de aquí es una promesa que se afloja**, así que se añaden de a una, nombradas, y el
/// summary del sprint las lista. Dos, al día del sprint 002.
struct Permitido {
    /// El índice del corpus: documentos DEL USUARIO, que la regla del efímero sí deja persistir.
    indice: PathBuf,
    /// El diccionario técnico del consultor (sprint 002, fase 1). Es del usuario: lo escribe él y la
    /// app lo relee al empezar cada sesión. **Lo que hace que no sea un transcript con otro nombre**
    /// es que sus entradas salen de dos sitios y de ninguno más: su archivo y los nombres de su
    /// corpus. Nunca de la reunión — la canaria de abajo lo comprueba archivo por archivo.
    diccionario: PathBuf,
}

impl Permitido {
    fn cubre(&self, ruta: &Path) -> bool {
        ruta.starts_with(&self.indice) || ruta == self.diccionario
    }
}

/// Carpetas que **escribe la herramienta, jamás la app**: el compilador, el gestor de paquetes,
/// git, el cubridor de tests, el empaquetador del frontend. Quedan fuera del inventario y esto no
/// es aflojar el gate, es apuntarlo: mientras estuvieron dentro, su veredicto dependía de **quién
/// más estuviera corriendo**. La primera vez que se corrió este gate en el `/release-check` del
/// S1 salió en rojo acusando a la sesión de dejar seis `.rlib` en `target/release/deps/` — los
/// había escrito un `pnpm tauri build` que compilaba al lado. Un gate que acusa al compilador se
/// acaba desactivando, y ese día la fuga de verdad pasa con él.
///
/// **Qué NO se pierde:** la app nunca escribe dentro de estas carpetas. Su carpeta de datos está
/// en `~/Library/Application Support`, su directorio de trabajo durante el test es `src-tauri/`
/// —que sigue vigilado entero— y una fuga con ruta relativa cae ahí, no en `target/`. La demo en
/// rojo de la fuga inyectada se repitió con esta exclusión puesta y siguió cazándola.
const DE_LA_HERRAMIENTA: [&str; 7] =
    ["target", "node_modules", ".git", "coverage", "dist", "playwright-report", "test-results"];

/// Lo que se sabe de un archivo **sin abrirlo**: cuánto ocupa y cuándo se escribió por última vez.
///
/// El sprint 001 inventariaba un **conjunto de rutas**, y con eso un archivo que CRECE es invisible:
/// añadirle una línea al final no le cambia la ruta, así que el inventario de antes y el de después
/// salían idénticos y el gate daba verde (hallazgo M9). Son las dos formas de dejar rastro —crear un
/// archivo y escribir en uno que ya estaba— y solo se miraba la primera.
///
/// **Y no se mira el CONTENIDO, a propósito.** El plan pedía un hash. Hashear lo que hay en
/// `~/Documents`, `~/Desktop` y `~/Downloads` significa **leer los documentos del usuario en cada
/// corrida del gate**, y un gate que abre los archivos privados para demostrar que la app no los
/// toca es un trato que esta casa no hace. Tamaño y fecha salen de la misma llamada a `metadata()`
/// que el inventario ya necesitaba para saber si algo es un archivo, cuestan cero lecturas, y cazan
/// las dos escrituras que un hash cazaría: la que engorda el archivo (cambia el tamaño) y la que lo
/// reescribe del mismo largo (cambia la fecha).
#[derive(PartialEq, Eq, Debug, Clone, Copy)]
struct Huella {
    bytes: u64,
    escrito: Option<std::time::SystemTime>,
}

/// Todo lo que cuelga de una carpeta, con su huella. Los enlaces no se siguen.
fn inventario(raiz: &Path) -> BTreeMap<PathBuf, Huella> {
    let mut salida = BTreeMap::new();
    let Ok(entradas) = std::fs::read_dir(raiz) else {
        return salida;
    };
    for e in entradas.flatten() {
        let ruta = e.path();
        if ruta.file_name().is_some_and(|n| DE_LA_HERRAMIENTA.iter().any(|d| n == *d)) {
            continue;
        }
        match e.file_type() {
            Ok(t) if t.is_dir() => salida.extend(inventario(&ruta)),
            Ok(t) if t.is_file() => {
                // Si la metadata no se deja leer, se apunta el archivo con la huella en blanco: la
                // ruta sigue contando como presencia. Dejarlo fuera sería un hueco silencioso.
                let m = e.metadata().ok();
                salida.insert(
                    ruta,
                    Huella {
                        bytes: m.as_ref().map(|m| m.len()).unwrap_or(0),
                        escrito: m.and_then(|m| m.modified().ok()),
                    },
                );
            }
            _ => {}
        }
    }
    salida
}

/// Dónde se mira. No es el disco entero —el sistema escribe sin parar y eso sería ruido— sino
/// **los sitios donde esta app podría escribir**: su propio árbol, su carpeta de datos, el
/// temporal del proceso y las carpetas del usuario donde un descuido dejaría un archivo a la
/// vista.
fn donde_se_mira(casa: &Path) -> Vec<PathBuf> {
    let hogar = std::env::var("HOME").map(PathBuf::from).unwrap_or_default();
    let mut sitios = vec![
        casa.to_path_buf(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".."),
        std::env::temp_dir(),
    ];
    // **Las tres carpetas de la app de verdad**, que hasta el sprint 002 no se miraban (hallazgo
    // M9). El test le da a la sesión una `casa` en el temporal, así que nada de lo que ESTE test
    // corre escribe ahí — y justo por eso hacía falta: si un día la app escribe con su ruta de
    // producción en vez de con la que se le pasa, el rastro cae aquí y en ningún otro sitio del
    // inventario.
    //
    // El nombre es el **identificador**, `com.aiapps.copiloto-consultor`, que es lo que macOS usa
    // de verdad; el plan del sprint lo escribió como «Angel Ghost», que es el nombre del producto
    // y una carpeta que no existe. Mirar donde no hay nada es la forma más fácil de que un gate
    // dé verde para siempre.
    for c in ["Application Support", "Caches", "Logs"] {
        let d = hogar.join("Library").join(c).join("com.aiapps.copiloto-consultor");
        if d.is_dir() {
            sitios.push(d);
        }
    }
    for c in ["Documents", "Desktop", "Downloads"] {
        let d = hogar.join(c);
        if d.is_dir() {
            sitios.push(d);
        }
    }
    sitios
}

/// La sesión. Devuelve lo que se dijo, para poder afirmar que de verdad pasó por dentro.
fn una_sesion_completa(casa: &Path, corpus_en: &Path) -> Vec<String> {
    let mut dicho = Vec::new();

    // 1 · El corpus del usuario, indexado. Esto SÍ escribe, y por eso está en `PERMITIDO`.
    let mut corpus = Corpus::en(&casa.join("corpus")).expect("el índice no se pudo abrir");
    corpus.indexar(corpus_en, &|_| {}).expect("no se pudo indexar el corpus sintético");
    assert!(corpus.estado().documentos > 0, "el corpus sintético quedó vacío");

    // 1-bis · El diccionario del consultor, que persiste y por tanto ESCRIBE. Entró en el inventario
    //     en el sprint 002 y tenía que entrar: la app lo deja escrito al arrancar y lo relee al
    //     empezar cada sesión, así que un gate que no lo ejerciera estaría midiendo una app distinta
    //     de la que el usuario usa. Y su corrección se aplica al turno de abajo, que es su sitio real.
    let ruta_dicc = casa.join("diccionario.yaml");
    app_copiloto_consultor_lib::asegurar_el_diccionario(&ruta_dicc)
        .expect("no se pudo dejar escrito el diccionario");
    let jerga = app_copiloto_consultor_lib::diccionario_de_la_sesion(&ruta_dicc, corpus.vocabulario());

    // 2 · Audio de verdad por el motor de verdad. Es el paso que el barrido estático no puede
    //     mirar: lo que Apple escriba por debajo, se escribe aquí.
    let motor = motor_de_la_casa();
    let (muestras, hz) = leer_wav(concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba/audio/pregunta-es.wav"));
    match motor.transcribir("es-ES", &muestras, hz) {
        Ok(texto) => {
            let texto = jerga.corregir(&texto);
            println!("[sesión] el motor devolvió {} letras", texto.chars().count());
            dicho.push(texto);
        }
        Err(e) => println!("[sesión] el motor no pudo transcribir ({e:?}): se sigue con la canaria"),
    }

    // 3 · El detector de turnos, con el mismo audio. Corta donde cortaría en una reunión.
    //
    // El `cerrar()` del final no es un detalle: el archivo del kit **termina justo después de la
    // frase**, sin el silencio que cierra un turno en una reunión de verdad. Sin él, este paso
    // contaba cero turnos y se quedaba de adorno — lo dijo su propia traza la primera vez que
    // corrió, y se arregló antes de dar el gate por bueno.
    let mut turnos = Turnos::nuevo(PorEnergia::nuevo());
    let mut cerrados = 0;
    for marco in muestras.chunks(320) {
        if let Some(Suceso::Termina { .. }) = turnos.marco(marco) {
            cerrados += 1;
        }
    }
    if let Some(Suceso::Termina { .. }) = turnos.cerrar() {
        cerrados += 1;
    }
    println!("[sesión] {cerrados} turnos cerrados por el detector");
    assert!(cerrados > 0, "el detector no vio ni un turno en el audio del kit: este paso no midió nada");

    // 4 · La canaria entra como turno del cliente y recorre disparador y ficha.
    let turno = Turno {
        pista: Pista::Sistema,
        desde_ms: 0,
        hasta_ms: 2_000,
        texto: format!("¿Y el alcance del {CANARIA} está dentro de la propuesta?"),
        hora: "14:02".into(),
        eco: false,
    };
    dicho.push(turno.texto.clone());

    let vocabulario = corpus.vocabulario().to_vec();
    let mut disparador = Disparador::nuevo();
    let motivo = disparador
        .mirar(&turno, &Contexto { ahora_ms: 2_000, vocabulario: &vocabulario })
        .expect("la pregunta del cliente no disparó: la sesión no probó el camino de la ficha");
    println!("[sesión] disparó por «{}»", motivo.etiqueta());

    let hallazgos = corpus.buscar(&turno.texto, 3).unwrap();
    match armar(&turno.texto, &hallazgos) {
        Respuesta::Ficha(f) => dicho.push(f.titular),
        Respuesta::SinResultado { buscado, .. } => dicho.push(buscado),
    }

    // 5 · El kill-switch sobre lo que guardó la última pregunta del cliente.
    disparador.reiniciar();
    dicho
}

/// Un corpus mínimo para la sesión del efímero: una propuesta y nada más.
fn corpus_para_el_efimero() -> PathBuf {
    let c = std::env::temp_dir().join(format!("ag-efimero-fuente-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&c);
    std::fs::create_dir_all(&c).unwrap();
    std::fs::write(
        c.join("Propuesta Páramo Azul.md"),
        "# Alcance\nCubre perfilado y limpieza de tres fuentes: ERP, POS y el Excel de canal.\n\n\
         # Plazo de entrega\nLa entrega completa toma cuatro semanas desde la firma.\n",
    )
    .unwrap();
    c
}

#[test]
fn una_sesion_completa_no_deja_nada_en_el_disco_salvo_el_indice_del_corpus() {
    let _turno = turno();
    let casa = std::env::temp_dir().join(format!("ag-efimero-casa-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&casa);
    std::fs::create_dir_all(&casa).unwrap();
    let fuente = corpus_para_el_efimero();
    let permitido =
        Permitido { indice: casa.join("corpus"), diccionario: casa.join("diccionario.yaml") };

    // El inventario se toma DESPUÉS de crear los fixtures: lo que se mide es lo que deja la
    // sesión, no lo que deja el test preparándola.
    let sitios = donde_se_mira(&casa);
    let antes: BTreeMap<PathBuf, Huella> = sitios.iter().flat_map(|s| inventario(s)).collect();
    println!("[efímero] {} archivos antes, en {} sitios", antes.len(), sitios.len());

    let dicho = una_sesion_completa(&casa, &fuente);

    let despues: BTreeMap<PathBuf, Huella> = sitios.iter().flat_map(|s| inventario(s)).collect();

    // **Las dos formas de dejar rastro.** Hasta el sprint 002 solo se miraba la primera —crear un
    // archivo—, y con eso una fuga que le añade una línea a un archivo que ya estaba pasaba con el
    // gate en verde: la ruta no cambia (hallazgo M9). Ahora un archivo que engorda o se reescribe
    // cuenta igual que uno recién creado.
    let tocados: Vec<(&PathBuf, &Huella, Option<&Huella>)> = despues
        .iter()
        // Los fixtures del propio test no cuentan: los creó el test, no la sesión.
        .filter(|(r, _)| !r.starts_with(&fuente))
        .filter_map(|(r, ahora)| match antes.get(r) {
            None => Some((r, ahora, None)),
            Some(a) if a != ahora => Some((r, ahora, Some(a))),
            Some(_) => None,
        })
        .collect();
    println!("[efímero] {} archivos tocados (creados o escritos)", tocados.len());

    let intrusos: Vec<String> = tocados
        .iter()
        .filter(|(r, ..)| !permitido.cubre(r))
        .map(|(r, ahora, antes)| match antes {
            None => format!("{} — nuevo, {} bytes", r.display(), ahora.bytes),
            Some(a) => format!(
                "{} — ya existía y la sesión escribió encima: {} → {} bytes",
                r.display(),
                a.bytes,
                ahora.bytes
            ),
        })
        .collect();
    assert!(
        intrusos.is_empty(),
        "la sesión dejó rastro en {} archivo(s) fuera del índice del corpus:\n  {}\n\
         (fuera del inventario, porque las escribe la herramienta y no la app: {})",
        intrusos.len(),
        intrusos.join("\n  "),
        DE_LA_HERRAMIENTA.join(" · ")
    );
    assert!(!tocados.is_empty(), "no se escribió NI el índice: la sesión no llegó a correr");

    // Y la canaria: lo que dijo el cliente no puede estar dentro de lo que sí se escribió.
    for (ruta, ..) in &tocados {
        let Ok(bytes) = std::fs::read(ruta) else { continue };
        let texto = String::from_utf8_lossy(&bytes);
        assert!(
            !texto.contains(CANARIA),
            "la frase del cliente acabó dentro de {}",
            ruta.display()
        );
    }
    assert!(dicho.iter().any(|d| d.contains(CANARIA)), "la canaria no llegó a recorrer la sesión");

    let _ = std::fs::remove_dir_all(&casa);
    let _ = std::fs::remove_dir_all(&fuente);
}

/// **LA CANARIA EN EL LOG** — la otra mitad del término plantado, que la DoD pedía y no existía.
///
/// El test de arriba vigila el DISCO: ni un archivo nuevo contiene lo que dijo el cliente. El log
/// es la otra salida por la que el texto de un tercero se escapa —a la consola, al `Console.app`
/// de macOS, al portapapeles de quien pega una traza en un mensaje— y nadie la miraba. Hallazgo A10
/// de la auditoría del sprint 001.
///
/// **Se corre la sesión en un proceso HIJO** y se lee su salida. Capturar `println!` desde dentro
/// del propio proceso obligaría a reemplazar la salida estándar por una de mentira, y entonces el
/// gate mediría el logger del test y no el del producto. El hijo es este mismo binario con el
/// filtro exacto de la sesión, así que no hay forma de que se llame a sí mismo en bucle.
///
/// Se ve en rojo poniendo `println!("{}", turno.texto)` en cualquier sitio del camino.
/// La sesión, **sin inventario del disco**, para que otro proceso pueda leer su log.
///
/// Es la misma `una_sesion_completa` que vigila el disco arriba; lo que aquí no se hace es recorrer
/// el temporal entero antes y después. Y esa diferencia es el arreglo de un fallo real: la primera
/// versión de la canaria lanzaba al hijo el test del disco, el hijo inventariaba `/var/folders`
/// mientras el padre creaba los fixtures de otro test, y **denunciaba como fuga de la sesión los
/// tres documentos del vecino**. Pasó en este Mac cinco veces seguidas y tumbó la integración
/// continua a la primera.
///
/// Dos procesos no comparten el mutex del turno, así que la respuesta no era un candado: era que el
/// hijo **no mire lo que no le toca**. Para leer un log hace falta la sesión, no el inventario.
#[test]
fn sesion_para_el_log() {
    let _turno = turno();
    let casa = std::env::temp_dir().join(format!("ag-log-casa-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&casa);
    std::fs::create_dir_all(&casa).unwrap();
    let fuente = corpus_para_el_efimero();

    let dicho = una_sesion_completa(&casa, &fuente);
    assert!(dicho.iter().any(|d| d.contains(CANARIA)), "la canaria no llegó a recorrer la sesión");

    let _ = std::fs::remove_dir_all(&casa);
    let _ = std::fs::remove_dir_all(&fuente);
}

#[test]
fn la_canaria_del_cliente_no_aparece_en_el_log() {
    // El turno porque el hijo usa el motor de voz, y los tests de audio de este binario usan los
    // altavoces y el tap del sistema: van de a uno, como todos los demás.
    let _turno = turno();
    let yo = std::env::current_exe().expect("no se supo cuál es este binario de pruebas");
    let hijo = std::process::Command::new(&yo)
        .args(["sesion_para_el_log", "--exact", "--nocapture", "--test-threads=1"])
        .output()
        .expect("no se pudo correr la sesión en un proceso hijo");

    let salida =
        format!("{}{}", String::from_utf8_lossy(&hijo.stdout), String::from_utf8_lossy(&hijo.stderr));

    // Que el hijo haya corrido DE VERDAD la sesión: si no, esto no mide nada y se vería verde.
    assert!(hijo.status.success(), "la sesión falló en el hijo:\n{salida}");
    // «disparó por» es el paso 4 de la sesión: la frase del cliente ya entró en el disparador y en
    // la ficha, que es justo el tramo donde podría escaparse al log. Sin esta comprobación, un hijo
    // que muriera al arrancar dejaría este gate en verde sin haber mirado nada.
    assert!(
        salida.contains("[sesión]") && salida.contains("disparó por"),
        "el hijo no llegó a correr la sesión entera: este gate no midió nada.\n{salida}"
    );

    let lineas: Vec<&str> = salida.lines().filter(|l| l.contains(CANARIA)).collect();
    assert!(
        lineas.is_empty(),
        "lo que dijo el cliente salió por el log, en {} línea(s):\n  {}",
        lineas.len(),
        lineas.join("\n  ")
    );
    println!(
        "[canaria] {} líneas de log revisadas · ni una con la frase del cliente",
        salida.lines().count()
    );
}

// ═══════════════════════════════════════════════ el KIT DE EVALUACIÓN v0 (nDCG@5 y la negativa)

// (bloque) Treinta preguntas contra el corpus sintético del kit, con umbrales declarados.
//
// **Por qué un número y no una impresión.** Cada decisión de la fase 4 —el plegado de acentos,
// el peso del título, el mínimo de términos— se puede defender con una anécdota. Con esto se
// puede defender con una medida, y sobre todo se puede *comparar*: cuando el sprint 2 quiera
// meter embeddings, la pregunta «¿hacen falta?» tiene por fin una respuesta que no depende de
// a quién se le pregunte.
//
// **Y la mitad que más importa son las cuatro últimas.** Las preguntas de `sinRespuesta` no
// están en el corpus, y la app tiene que DECIRLO. Un buscador que acierta 30 de 30 y además
// contesta con seguridad a lo que no sabe es peor que uno que acierta 25: el fallo caro de esta
// app no es no encontrar, es encontrar cualquier cosa y ponerle una fuente debajo.

/// Umbral del nDCG@5. Se fija **con la primera medición**, no antes: un umbral inventado o pasa
/// siempre o no pasa nunca, y en los dos casos deja de medir. Medido **0,823** con el corpus y
/// las preguntas del kit v0; el mínimo se deja dos centésimas por debajo para que el ruido de un
/// empate no tumbe la integración continua, y lo bastante cerca para que una regresión de
/// verdad se note. Se sube cuando el retriever mejore; bajarlo exige decirlo en la bitácora.
const NDCG_MINIMO: f64 = 0.80;

/// Las tres que fallan hoy, y por qué se dejan fallando: **ninguna comparte una sola palabra con
/// su sección**. «¿Por qué nos contrataron para esto?» contra una sección que habla de márgenes
/// y canales; «¿cómo les fue en lo de la cooperativa?» contra una que dice «la implementación
/// cerró con dos semanas de retraso». BM25 no puede resolverlas y reescribir las preguntas para
/// que las acierte convertiría el kit en un espejo. Son la evidencia que el sprint 2 necesita
/// para decidir si los embeddings hacen falta — y ahora esa pregunta tiene un número detrás.
const FALLOS_SEMANTICOS_CONOCIDOS: usize = 3;

/// Cuántas de las que NO están en el corpus tiene que rechazar. Aquí no hay margen: aproximar
/// una ficha sobre algo que no se tiene es el fallo que esta app existe para no cometer.
const RECHAZO_MINIMO: f64 = 1.0;

#[derive(serde::Deserialize)]
struct Kit {
    preguntas: Vec<Caso>,
    #[serde(rename = "sinRespuesta")]
    sin_respuesta: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Caso {
    dice: String,
    espera: String,
}

/// nDCG@5 con relevancia binaria: la sección esperada vale 1 y todo lo demás 0. Con un solo
/// documento relevante, el ideal es 1.0 y el descuento sale del puesto en el que aparece.
fn ndcg_5(puestos: &[String], espera: &str) -> f64 {
    puestos
        .iter()
        .take(5)
        .position(|s| s == espera)
        .map(|i| 1.0 / ((i + 2) as f64).log2())
        .unwrap_or(0.0)
}

#[test]
fn el_kit_de_evaluacion_mide_el_retriever_y_su_negativa() {
    let _turno = turno();
    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba");
    let kit: Kit = serde_json::from_str(
        &std::fs::read_to_string(format!("{raiz}/preguntas.json")).expect("falta preguntas.json"),
    )
    .expect("preguntas.json no se pudo leer");

    let mut corpus = Corpus::en_memoria().unwrap();
    corpus.indexar(Path::new(&format!("{raiz}/corpus")), &|_| {}).expect("no se indexó el kit");
    assert_eq!(corpus.estado().documentos, 6, "el corpus del kit cambió de tamaño");

    // ---- nDCG@5 sobre las treinta que SÍ están, y la LATENCIA de cada una
    let mut suma = 0.0;
    let mut fallos = Vec::new();
    let mut latencias = Vec::new();
    for c in &kit.preguntas {
        // Se cronometra lo mismo que cronometra el producto —buscar y armar la ficha—, y se
        // cronometra **por pregunta**, no el total: una media esconde los picos, y el presupuesto
        // del sprint es sobre el turno que el usuario está esperando, no sobre el promedio del día.
        let reloj = Instant::now();
        let hallazgos = corpus.buscar(&c.dice, 5).unwrap();
        let _ = armar(&c.dice, &hallazgos);
        latencias.push(reloj.elapsed().as_micros() as u64);
        let puestos: Vec<String> =
            hallazgos.into_iter().map(|h| h.seccion.unwrap_or_default()).collect();
        let n = ndcg_5(&puestos, &c.espera);
        suma += n;
        if n == 0.0 {
            fallos.push(format!("  «{}» → esperaba «{}», trajo {:?}", c.dice, c.espera, puestos));
        }
    }
    let ndcg = suma / kit.preguntas.len() as f64;
    latencias.sort_unstable();
    let percentil = |p: f64| latencias[((latencias.len() as f64 - 1.0) * p).round() as usize];
    let (mediana, p90, peor) = (percentil(0.5), percentil(0.9), *latencias.last().unwrap());

    // ---- y la negativa sobre las que NO están
    let mut rechazadas = 0;
    let mut aproximadas = Vec::new();
    for dice in &kit.sin_respuesta {
        let hallazgos = corpus.buscar(dice, 3).unwrap();
        match armar(dice, &hallazgos) {
            Respuesta::SinResultado { .. } => rechazadas += 1,
            Respuesta::Ficha(f) => {
                aproximadas.push(format!("  «{dice}» → citó «{}»", f.fuente.documento))
            }
        }
    }
    let rechazo = rechazadas as f64 / kit.sin_respuesta.len() as f64;

    println!("\n╭─ kit de evaluación v0 ─────────────────────────────");
    println!("│ nDCG@5          {ndcg:.3}   (mínimo {NDCG_MINIMO:.2})");
    println!("│ rechazo         {rechazo:.3}   (mínimo {RECHAZO_MINIMO:.2})");
    println!("│ preguntas       {}", kit.preguntas.len());
    println!("│ sin respuesta   {}", kit.sin_respuesta.len());
    println!("├─ de la pregunta a la ficha (µs) ───────────────────");
    println!("│ mediana {mediana}   p90 {p90}   peor {peor}");
    println!("│ presupuesto del sprint: {} µs de FIN DE TURNO a ficha,", PRESUPUESTO_US);
    println!("│ del que esto es el tramo determinista — sin captura ni STT.");
    println!("╰────────────────────────────────────────────────────");
    if !fallos.is_empty() {
        println!("las que no encontraron su sección en los cinco primeros:\n{}", fallos.join("\n"));
    }
    if !aproximadas.is_empty() {
        println!("las que se aproximaron en vez de callar:\n{}", aproximadas.join("\n"));
    }

    assert!(ndcg >= NDCG_MINIMO, "nDCG@5 {ndcg:.3} por debajo de {NDCG_MINIMO:.2}");
    assert!(
        fallos.len() <= FALLOS_SEMANTICOS_CONOCIDOS,
        "aparecieron {} fallos, {FALLOS_SEMANTICOS_CONOCIDOS} conocidos: alguno es nuevo",
        fallos.len()
    );
    assert!(rechazo >= RECHAZO_MINIMO, "rechazó {rechazadas} de {}", kit.sin_respuesta.len());
    // La mediana era lo que el plan pedía y el kit no daba (hallazgo A9). Se compara contra el
    // presupuesto entero a propósito: lo que hay que saber es cuánto de los 4 s se lleva la parte
    // que no depende del Mac ni del motor de voz. Hoy son microsegundos; el día que alguien meta
    // una espera en este camino, este número lo dirá antes que una reunión.
    assert!(
        mediana < PRESUPUESTO_US,
        "la mediana de la búsqueda son {mediana} µs y el presupuesto entero es {PRESUPUESTO_US}"
    );
    assert!(peor < PRESUPUESTO_US, "la peor pregunta del kit tardó {peor} µs");
}

/// El presupuesto del sprint, en microsegundos: **4 s del fin de turno a la ficha en la banda.**
/// Lo fija la orden. Aquí se mide solo el tramo determinista —buscar y armar—, y se dice.
const PRESUPUESTO_US: u64 = 4_000_000;

// =============================================================================================
// el disparador, medido: precisión y recall sobre una reunión marcada turno a turno
// =============================================================================================
//
// **Faltaba, y estaba en el plan del sprint** (hallazgo A9 de la auditoría): «P/R del disparo ±1
// turno». Sin esto, del disparador solo se sabía que sus reglas pasaban sus propios tests unitarios
// — que es otra cosa que saber cuántas veces se equivoca en una reunión entera, con la espera entre
// fichas y la negativa a repetir metidas en la cuenta.
//
// Los dos errores no cuestan lo mismo, y por eso se miden por separado:
//   · un **falso positivo** interrumpe al consultor con una ficha que nadie pidió;
//   · un **falso negativo** es una ficha que no llega — molesta menos y se arregla con `⌘⇧A`.
// De ahí que el umbral de precisión sea más alto que el de recall.

/// Precisión mínima: **ni un falso positivo** sobre el kit. Medida 1.000 en la primera corrida.
const PRECISION_MINIMA: f64 = 1.0;

/// Recall mínimo: **1.000, el mismo que se midió.**
///
/// Se dejó primero en 0,90 «por si un empate léxico cambia de lado», y la demo en rojo lo tumbó:
/// devolviendo `MINIMO_CON_SIGNO` a 3 —el defecto real que la fase 4 encontró y arregló—
/// «¿Tienen certificación?» deja de disparar, el recall baja a 0,909… **y el test seguía verde**.
/// Con once turnos que deben disparar, un umbral del 90 % regala uno. No hay ninguno regalable:
/// cada turno de este kit está marcado a mano porque la app tiene que acertarlo. Bajar este número
/// exige decirlo en la bitácora, como el del nDCG.
const RECALL_MINIMO: f64 = 1.0;

#[derive(serde::Deserialize)]
struct KitDelDisparo {
    turnos: Vec<TurnoMarcado>,
}

#[derive(serde::Deserialize)]
struct TurnoMarcado {
    dice: String,
    pista: String,
    #[serde(default)]
    eco: bool,
    ms: usize,
    dispara: bool,
    porque: String,
}

#[test]
fn el_kit_mide_el_disparador_turno_a_turno() {
    let _turno = turno();
    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba");
    let kit: KitDelDisparo = serde_json::from_str(
        &std::fs::read_to_string(format!("{raiz}/disparo.json")).expect("falta disparo.json"),
    )
    .expect("disparo.json no se pudo leer");

    // El vocabulario sale del corpus del kit, porque una de las reglas es «nombró algo tuyo».
    let mut corpus = Corpus::en_memoria().unwrap();
    corpus.indexar(Path::new(&format!("{raiz}/corpus")), &|_| {}).expect("no se indexó el kit");
    let vocabulario = corpus.vocabulario().to_vec();

    let mut disparador = Disparador::nuevo();
    let (mut ciertos, mut falsos_positivos, mut falsos_negativos) = (0, Vec::new(), Vec::new());

    for t in &kit.turnos {
        let turno = app_copiloto_consultor_lib::stt::Turno {
            pista: if t.pista == "microfono" { Pista::Microfono } else { Pista::Sistema },
            desde_ms: t.ms.saturating_sub(2_000),
            hasta_ms: t.ms,
            texto: t.dice.clone(),
            hora: "14:02".into(),
            eco: t.eco,
        };
        let ctx = Contexto { ahora_ms: t.ms, vocabulario: &vocabulario };
        let disparo = disparador.mirar(&turno, &ctx);
        match (disparo.is_some(), t.dispara) {
            (true, true) => ciertos += 1,
            (true, false) => falsos_positivos.push(format!("  «{}» — {}", t.dice, t.porque)),
            (false, true) => falsos_negativos.push(format!("  «{}» — {}", t.dice, t.porque)),
            (false, false) => {}
        }
    }

    let disparados = ciertos + falsos_positivos.len();
    let esperados = ciertos + falsos_negativos.len();
    let precision = if disparados == 0 { 1.0 } else { ciertos as f64 / disparados as f64 };
    let recall = if esperados == 0 { 1.0 } else { ciertos as f64 / esperados as f64 };

    println!("\n╭─ el disparador sobre el kit ───────────────────────");
    println!("│ turnos          {}", kit.turnos.len());
    println!("│ aciertos        {ciertos}");
    println!("│ precisión       {precision:.3}   (mínimo {PRECISION_MINIMA:.2})");
    println!("│ recall          {recall:.3}   (mínimo {RECALL_MINIMO:.2})");
    println!("╰────────────────────────────────────────────────────");
    if !falsos_positivos.is_empty() {
        println!("disparó y no debía:\n{}", falsos_positivos.join("\n"));
    }
    if !falsos_negativos.is_empty() {
        println!("no disparó y debía:\n{}", falsos_negativos.join("\n"));
    }

    assert!(
        precision >= PRECISION_MINIMA,
        "precisión {precision:.3}: el disparador interrumpe cuando no debe"
    );
    assert!(recall >= RECALL_MINIMO, "recall {recall:.3}: se está quedando callado cuando debería buscar");
}

// ═══════════════════════════════════════════ el WER, CON Y SIN DICCIONARIO (sprint 002, fase 1)
//
// **La deuda del sprint 001, pagada.** El kit de prueba prometía dos cosas que no entregó: un audio
// con mezcla de idiomas y un WER de la transcripción. Las dos están aquí, y juntas por una razón: el
// WER es la única forma de saber si el diccionario técnico **sirve o estorba**.
//
// Un corrector de jerga es fácil de escribir y fácil de auto-engañar. Con cinco términos elegidos y
// cinco frases de ejemplo, cualquier diccionario parece bueno. Lo que dice la verdad es medir el
// texto entero contra lo que se dijo de verdad, **con el diccionario puesto y sin él**, sobre el
// mismo audio. Si el número no baja, el módulo no vale; si sube, hace daño.

/// Palabras normalizadas para comparar: minúsculas, sin tildes, sin puntuación y sin separadores de
/// miles.
///
/// Los separadores importan y no es una concesión: el motor escribe «ISO 27.001» y la referencia dice
/// «ISO 27001». Es un hallazgo conocido del sprint 001 con su propio test, no un error de
/// transcripción, y contarlo como tal metería ruido fijo en las cuatro medidas.
fn palabras(texto: &str) -> Vec<String> {
    texto
        .split_whitespace()
        .map(|p| {
            p.chars()
                .flat_map(|c| c.to_lowercase())
                .map(|c| match c {
                    'á' | 'à' | 'ä' | 'â' => 'a',
                    'é' | 'è' | 'ë' | 'ê' => 'e',
                    'í' | 'ì' | 'ï' | 'î' => 'i',
                    'ó' | 'ò' | 'ö' | 'ô' => 'o',
                    'ú' | 'ù' | 'ü' | 'û' => 'u',
                    'ñ' => 'n',
                    c => c,
                })
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|p| !p.is_empty())
        .collect()
}

/// **Word Error Rate**: (sustituciones + inserciones + borrados) / palabras de la referencia.
///
/// Es la distancia de edición entre las dos listas de PALABRAS, no de letras. Se calcula entera —sin
/// el corte que usa el diccionario— porque aquí el número exacto es el resultado, no un sí/no.
fn wer(referencia: &[String], hipotesis: &[String]) -> f64 {
    if referencia.is_empty() {
        return if hipotesis.is_empty() { 0.0 } else { 1.0 };
    }
    let mut fila: Vec<usize> = (0..=hipotesis.len()).collect();
    for (i, r) in referencia.iter().enumerate() {
        let mut anterior = fila[0];
        fila[0] = i + 1;
        for (j, h) in hipotesis.iter().enumerate() {
            let costo = usize::from(r != h);
            let nuevo = (fila[j + 1] + 1).min(fila[j] + 1).min(anterior + costo);
            anterior = fila[j + 1];
            fila[j + 1] = nuevo;
        }
    }
    fila[hipotesis.len()] as f64 / referencia.len() as f64
}

#[derive(serde::Deserialize)]
struct AudioDelKit {
    archivo: String,
    idioma: String,
    dice: String,
    jerga: bool,
}

#[derive(serde::Deserialize)]
struct Transcripciones {
    audios: Vec<AudioDelKit>,
}

#[test]
fn el_wer_no_empeora_con_el_diccionario_y_mejora_donde_hay_jerga() {
    let _turno = turno();
    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba/audio");
    let kit: Transcripciones = serde_json::from_str(
        &std::fs::read_to_string(format!("{raiz}/transcripciones.json"))
            .expect("falta transcripciones.json"),
    )
    .expect("transcripciones.json no se pudo leer");

    let motor = motor_de_la_casa();
    // La jerga sale de la SEMILLA, no del archivo del usuario: el kit tiene que medir lo mismo en
    // esta máquina y en la integración continua, y el archivo del usuario es distinto en cada Mac.
    let jerga = app_copiloto_consultor_lib::diccionario::Diccionario::semilla();

    let mut medidos = 0;
    let mut peor_subida = 0.0_f64;
    let mut bajo_con_jerga = false;

    println!("┌─ WER del kit · con y sin diccionario ────────────────────────────────");
    for a in &kit.audios {
        if !matches!(motor.disponibilidad(&a.idioma), Disponibilidad::Listo) {
            // Nunca en silencio: sin modelo de ese idioma en esta máquina no hay nada que medir, y
            // eso se dice en vez de contar el audio como aprobado.
            println!("│ {:<16} sin modelo de {} en esta máquina: no se mide", a.archivo, a.idioma);
            continue;
        }
        let (muestras, hz) = leer_wav(&format!("{raiz}/{}", a.archivo));
        let Ok(crudo) = motor.transcribir(&a.idioma, &muestras, hz) else {
            println!("│ {:<16} el motor estaba listo y falló: no se mide", a.archivo);
            continue;
        };
        let corregido = jerga.corregir(&crudo);
        let referencia = palabras(&a.dice);
        let sin = wer(&referencia, &palabras(&crudo));
        let con = wer(&referencia, &palabras(&corregido));
        medidos += 1;
        peor_subida = peor_subida.max(con - sin);
        if a.jerga && con < sin {
            bajo_con_jerga = true;
        }
        println!(
            "│ {:<16} {:<6} sin {:.3} · con {:.3} · {}",
            a.archivo,
            a.idioma,
            sin,
            con,
            if con < sin {
                "MEJORA"
            } else if con > sin {
                "EMPEORA"
            } else {
                "igual"
            }
        );
        println!("│   oyó      «{crudo}»");
        if corregido != crudo {
            println!("│   corregido «{corregido}»");
        }
    }
    println!("└──────────────────────────────────────────────────────────────────────");

    if medidos == 0 {
        println!("sin modelos de voz en esta máquina: el WER no se pudo medir en ninguna pista");
        return;
    }

    // **El umbral del plan: «no empeora».** Es la mitad que importa de un corrector — el daño de
    // corregir de más no se ve en los ejemplos, se ve aquí.
    assert!(
        peor_subida <= 0.0,
        "el diccionario EMPEORÓ el WER en {peor_subida:.3}: está corrigiendo lo que no debe"
    );

    // Y la otra mitad: donde hay jerga, tiene que bajar. Un diccionario que nunca empeora nada
    // porque nunca corrige nada pasaría la aserción de arriba y no serviría para nada.
    assert!(
        bajo_con_jerga,
        "el diccionario no bajó el WER en ningún audio con jerga: no está haciendo su trabajo"
    );
}
