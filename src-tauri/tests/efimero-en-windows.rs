//! **LA MITAD EN MARCHA DE LA REGLA 1, EN WINDOWS** (sprint 008, auditoría del S6, M11).
//!
//! En el Mac la corre `contra-el-mac-de-verdad`: una sesión completa con el motor de verdad y el disco inventariado
//! antes y después. Aquí, lo mismo con lo que Windows tiene: una sesión —el micrófono en silencio y la pregunta del
//! kit por el audio del sistema, sobre anillos que llena el test— con **whisper de verdad** (el modelo pequeño del
//! runner) y el disco inventariado. Ni un archivo nuevo o cambiado en la carpeta de la app fuera de `modelos/`, y
//! ninguno nuevo en la carpeta temporal que lleve lo que dijo el cliente (la «canaria»).
//!
//! Vive fuera de `src/` porque lee y escribe archivos, y `src/escucha/` y `src/stt/` tienen prohibido tocar el disco.
//! La corre `build-windows` en su runner efímero, con el modelo que ya baja para la medición (`AG_MODELO_WHISPER`);
//! jamás tu PC: copia el modelo a la carpeta de la app y deja la sesión corriendo unos segundos.
//!
//! ¿Puede fallar? Sí: un `std::fs::write` en cualquier punto de la sesión, o una escritura de la transcripción en la
//! carpeta de la app, es rojo con el nombre del archivo.

#![cfg(windows)]

use app_copiloto_consultor_lib::capture::anillo::HZ;
use app_copiloto_consultor_lib::capture::{Anillo, Pista};
use app_copiloto_consultor_lib::diccionario::Diccionario;
use app_copiloto_consultor_lib::escucha::{Escucha, Novedad, SinCorpus};
use app_copiloto_consultor_lib::modelo_whisper::{self, PEQUENO};
use app_copiloto_consultor_lib::stt::{self, Disponibilidad};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

type Huellas = BTreeMap<PathBuf, (u64, SystemTime)>;

fn inventario(raiz: &Path, salida: &mut Huellas) {
    let Ok(entradas) = std::fs::read_dir(raiz) else { return };
    for e in entradas.flatten() {
        let Ok(tipo) = e.file_type() else { continue };
        if tipo.is_dir() {
            inventario(&e.path(), salida);
        } else if let Ok(m) = e.metadata() {
            salida.insert(e.path(), (m.len(), m.modified().unwrap_or(UNIX_EPOCH)));
        }
    }
}

#[test]
#[ignore = "hardware: una sesión completa con whisper (el modelo pequeño del runner) y el inventario del disco; la corre build-windows"]
fn una_sesion_completa_con_whisper_no_deja_nada_en_el_disco() {
    // El modelo pequeño, donde la app lo busca. Esto SÍ escribe, y por eso va antes de tomar el inventario.
    let origen = std::env::var("AG_MODELO_WHISPER").expect("falta AG_MODELO_WHISPER: la sesión no midió nada");
    let destino = modelo_whisper::ruta(&PEQUENO).expect("sin LOCALAPPDATA");
    if modelo_whisper::listo(&PEQUENO).is_none() {
        std::fs::create_dir_all(destino.parent().unwrap()).unwrap();
        std::fs::copy(&origen, &destino).expect("no se pudo dejar el modelo donde la app lo busca");
    }
    let motor = stt::motor_de_la_casa();
    assert!(matches!(motor.disponibilidad("es-ES"), Disponibilidad::Listo), "whisper no ve el modelo pequeño");

    let local = PathBuf::from(std::env::var_os("LOCALAPPDATA").expect("sin LOCALAPPDATA"))
        .join(app_copiloto_consultor_lib::puerta::IDENTIFICADOR);
    let roaming = std::env::var_os("APPDATA")
        .map(|r| PathBuf::from(r).join(app_copiloto_consultor_lib::puerta::IDENTIFICADOR));
    let carpetas_de_la_app: Vec<PathBuf> = std::iter::once(local.clone()).chain(roaming).collect();
    let todas: Vec<PathBuf> = carpetas_de_la_app.iter().cloned().chain(std::iter::once(std::env::temp_dir())).collect();
    let tomar = || {
        let mut h = Huellas::new();
        for s in &todas {
            inventario(s, &mut h);
        }
        h
    };
    let antes = tomar();

    // La sesión: la pregunta del kit entra por el audio del sistema —lo que en Windows llena el loopback— con el
    // micrófono callado al lado, y se corta.
    let pregunta = app_copiloto_consultor_lib::habla::pcm::leer(include_bytes!(
        "../../docs/kit-de-prueba/audio/pregunta-es.wav"
    ))
    .expect("el WAV del kit se lee");
    let microfono = Arc::new(Mutex::new(Anillo::de_la_app()));
    let sistema = Arc::new(Mutex::new(Anillo::de_la_app()));
    let oido: Arc<Mutex<Vec<Novedad>>> = Arc::default();
    let apunta = oido.clone();
    let sesion = Escucha::sobre_anillos(
        vec![(Pista::Microfono, microfono.clone(), "es-ES".into()), (Pista::Sistema, sistema.clone(), "es-ES".into())],
        motor,
        Arc::new(SinCorpus),
        Arc::new(Diccionario::default()),
        move |n| apunta.lock().unwrap().push(n),
    );
    {
        let hz = HZ as usize;
        microfono.lock().unwrap().escribir(&vec![0.0; hz * 6 / 10 + pregunta.muestras.len() + hz]);
        let mut s = sistema.lock().unwrap();
        s.escribir(&vec![0.0; hz * 6 / 10]);
        s.escribir(&pregunta.muestras);
        s.escribir(&vec![0.0; hz]);
    }
    let del_cliente = |n: &Novedad| match n {
        Novedad::Turno(t) => t.pista == Pista::Sistema,
        Novedad::SinTexto { pista, .. } => *pista == Pista::Sistema,
        _ => false,
    };
    let hasta = Instant::now() + Duration::from_secs(90);
    while Instant::now() < hasta && !oido.lock().unwrap().iter().any(del_cliente) {
        std::thread::sleep(Duration::from_millis(50));
    }
    sesion.cortar();
    stt::whisper::soltar();
    assert!(
        oido.lock().unwrap().iter().any(del_cliente),
        "la sesión no llegó a procesar la pregunta del cliente: el inventario no mide nada"
    );
    let dicho: Vec<String> = oido
        .lock()
        .unwrap()
        .iter()
        .filter_map(|n| if let Novedad::Turno(t) = n { Some(t.texto.to_lowercase()) } else { None })
        .collect();

    let despues = tomar();
    let modelos = local.join(modelo_whisper::CARPETA);
    let mut fugas = Vec::new();
    for (ruta, huella) in &despues {
        if antes.get(ruta) == Some(huella) || ruta.starts_with(&modelos) {
            continue;
        }
        if carpetas_de_la_app.iter().any(|c| ruta.starts_with(c)) {
            fugas.push(format!("{} (en la carpeta de la app)", ruta.display()));
        } else if huella.0 <= 5_000_000 {
            // En la carpeta temporal, lo único que importa es si lleva lo que dijo el cliente.
            let texto = String::from_utf8_lossy(&std::fs::read(ruta).unwrap_or_default()).to_lowercase();
            if dicho.iter().any(|d| d.trim().len() >= 8 && texto.contains(d.trim())) {
                fugas.push(format!("{} (lleva la transcripción del cliente)", ruta.display()));
            }
        }
    }
    println!("[efímero] sesión de whisper: {} turno(s) del cliente; {} fuga(s)", dicho.len(), fugas.len());
    assert!(fugas.is_empty(), "la sesión dejó rastro en el disco (regla dura 1):\n{}", fugas.join("\n"));
}
