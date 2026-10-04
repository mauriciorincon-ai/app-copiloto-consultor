//! LA PUERTA LOCAL, con un socket de verdad (ADR 018). Vive aquí y no dentro de `puerta/` porque
//! necesita una carpeta en disco donde crear el socket, y `puerta/` es módulo protegido: ni disco ni
//! red salvo las líneas del socket, marcadas con su ADR.
//!
//! **Ninguna prueba toca el Llavero de verdad** (regla 22): la llave es un doble. Ni la app: las
//! operaciones son otro doble, con la reunión a voluntad.

use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use app_copiloto_consultor_lib::puerta::socket::{
    carpeta_de_la_app, limpiar_lo_que_quedo, pedir, ruta_en, NoLlega, Puerta, RUTA_MAXIMA,
};
use app_copiloto_consultor_lib::puerta::{Cierre, Hecho, Llave, Motivo, NoAbre, Operaciones, Orden, Respuesta, Resultado};


/// La app, de mentira: reunión a voluntad y órdenes que se cuentan.
#[derive(Default)]
struct Doble {
    reunion: AtomicBool,
    hechas: AtomicUsize,
    avisos: AtomicUsize,
}

impl Operaciones for Doble {
    fn en_reunion(&self) -> bool {
        self.reunion.load(Ordering::SeqCst)
    }
    fn hacer(&self, orden: &Orden) -> Result<Hecho, String> {
        self.hechas.fetch_add(1, Ordering::SeqCst);
        Ok(Hecho { datos: serde_json::json!({ "orden": orden.sin_contenido() }), cuenta: Some(28) })
    }
    fn hora(&self) -> String {
        "11:04".into()
    }
    fn avisar(&self) {
        self.avisos.fetch_add(1, Ordering::SeqCst);
    }
}

/// El Llavero, de mentira: **ninguna prueba toca el de verdad** (regla 22).
#[derive(Default)]
struct LlaveDePrueba {
    token: Mutex<Option<String>>,
    falla: bool,
}

impl Llave for LlaveDePrueba {
    fn guardar(&self, token: &str) -> Result<(), String> {
        if self.falla {
            return Err("bloqueado".into());
        }
        *self.token.lock().unwrap() = Some(token.to_string());
        Ok(())
    }
    fn borrar(&self) {
        *self.token.lock().unwrap() = None;
    }
}

impl LlaveDePrueba {
    fn token(&self) -> String {
        self.token.lock().unwrap().clone().expect("no hay token en el Llavero de prueba")
    }
}

fn carpeta(nombre: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ag-puerta-{nombre}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    app_copiloto_consultor_lib::almacen::carpeta_privada(&d).unwrap();
    d
}

fn abrir(nombre: &str) -> (Arc<Puerta>, Arc<LlaveDePrueba>, Arc<Doble>, PathBuf) {
    let d = carpeta(nombre);
    let (puerta, llave, doble) =
        (Arc::new(Puerta::con_vigia(Duration::from_millis(200))), Arc::new(LlaveDePrueba::default()), Arc::new(Doble::default()));
    puerta.abrir(&d, llave.clone(), doble.clone()).expect("la puerta no se abrió");
    (puerta, llave, doble, d)
}

const UN_RATO: Duration = Duration::from_secs(5);

fn esperar(que: impl Fn() -> bool) -> bool {
    for _ in 0..50 {
        if que() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

/// **Sin la carpeta en 700, no se abre** (auditoría del S3, B17): el socket nace con el umask antes del
/// `chmod` a 600, y lo que lo protege en ese instante es la carpeta. Si no está en 700, la puerta no crea
/// nada y no guarda token. Demostrado en rojo con el código de antes: se abría y dejaba el socket.
#[cfg(unix)]
#[test]
fn sin_la_carpeta_en_700_no_se_abre() {
    use std::os::unix::fs::PermissionsExt;
    let d = carpeta("floja");
    std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o755)).unwrap();
    let (puerta, llave, doble) =
        (Arc::new(Puerta::con_vigia(Duration::from_millis(200))), Arc::new(LlaveDePrueba::default()), Arc::new(Doble::default()));
    assert!(puerta.abrir(&d, llave.clone(), doble).is_err(), "la puerta se abrió en una carpeta en 755");
    assert!(!puerta.abierta());
    assert!(!ruta_en(&d).exists(), "quedó un socket en la carpeta floja");
    assert!(llave.token.lock().unwrap().is_none(), "se guardó un token sin abrir");
    let _ = std::fs::remove_dir_all(&d);
}

/// **Nace cerrada**: sin socket, sin token, y `ghost` lo sabe sin tocar el Llavero.
#[test]
fn nace_cerrada() {
    let d = carpeta("nace");
    let puerta = Puerta::default();
    assert!(!puerta.abierta());
    assert!(!ruta_en(&d).exists());
    assert_eq!(pedir(&ruta_en(&d), "x", &Orden::Reindexar {}, UN_RATO), Err(NoLlega::Cerrada));
    let v = puerta.vista(None);
    assert!(!v.abierta && v.cerro.is_none() && v.registro.is_empty());
}

/// De punta a punta, con un socket de verdad: abre, atiende con la llave y cierra sin dejar nada.
#[test]
fn abierta_atiende_con_la_llave_y_al_cerrar_no_queda_nada() {
    let (puerta, llave, doble, d) = abrir("punta");
    let ruta = ruta_en(&d);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(std::fs::metadata(&ruta).unwrap().permissions().mode() & 0o777, 0o600, "el socket no nace en 600");
    }
    let r = pedir(&ruta, &llave.token(), &Orden::Reindexar {}, UN_RATO).unwrap();
    assert!(matches!(r, Respuesta::Hecho { .. }), "{r:?}");
    let r = pedir(&ruta, "otra", &Orden::Reindexar {}, UN_RATO).unwrap();
    assert_eq!(r, Respuesta::Denegado { motivo: Motivo::LlaveErrada });
    assert_eq!(doble.hechas.load(Ordering::SeqCst), 1);

    let v = puerta.vista(None);
    assert_eq!(v.registro.len(), 2);
    assert_eq!(v.registro[1].resultado, Resultado::Hecho { cuenta: Some(28) });

    assert!(puerta.cerrar(Cierre::ATuMano));
    assert!(!ruta.exists(), "el socket sobrevivió al cierre");
    assert!(llave.token.lock().unwrap().is_none(), "el token sobrevivió al cierre");
    assert_eq!(pedir(&ruta, "x", &Orden::Reindexar {}, UN_RATO), Err(NoLlega::Cerrada));
    assert_eq!(puerta.vista(None).cerro, Some(Cierre::ATuMano));
}

/// **En reunión se deniega y se cierra sola.** Y no se vuelve a abrir sola.
#[test]
fn una_orden_en_reunion_se_deniega_y_cierra_la_puerta() {
    let (puerta, llave, doble, d) = abrir("reunion");
    doble.reunion.store(true, Ordering::SeqCst);
    let r = pedir(&ruta_en(&d), &llave.token(), &Orden::LeerPrefs {}, UN_RATO).unwrap();
    assert_eq!(r, Respuesta::Denegado { motivo: Motivo::EnReunion });
    assert!(!puerta.abierta(), "la puerta siguió abierta en reunión");
    assert_eq!(puerta.vista(None).cerro, Some(Cierre::EnReunion));
    doble.reunion.store(false, Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(400));
    assert!(!puerta.abierta(), "la puerta se volvió a abrir sola");
}

/// **El vigía**: aunque nadie llame, en cuanto empieza una reunión la puerta se cierra.
#[test]
fn el_vigia_la_cierra_al_empezar_una_reunion() {
    let (puerta, _llave, doble, d) = abrir("vigia");
    doble.reunion.store(true, Ordering::SeqCst);
    assert!(esperar(|| !puerta.abierta()), "el vigía no cerró la puerta");
    assert!(!ruta_en(&d).exists());
    assert_eq!(puerta.vista(None).cerro, Some(Cierre::EnReunion));
}

#[test]
fn en_reunion_no_se_abre() {
    let d = carpeta("no-abre");
    let (puerta, doble) = (Arc::new(Puerta::default()), Arc::new(Doble::default()));
    doble.reunion.store(true, Ordering::SeqCst);
    let llave = Arc::new(LlaveDePrueba::default());
    assert_eq!(puerta.abrir(&d, llave.clone(), doble), Err(NoAbre::EnReunion));
    assert!(!ruta_en(&d).exists());
    assert!(llave.token.lock().unwrap().is_none());
    assert_eq!(puerta.vista(None).no_abre, Some(NoAbre::EnReunion));
}

/// Si el Llavero no guarda el token, la puerta no se queda a medio abrir con un socket sin llave.
#[test]
fn sin_llavero_no_se_abre_ni_deja_socket() {
    let d = carpeta("sin-llavero");
    let llave = Arc::new(LlaveDePrueba { falla: true, ..Default::default() });
    let puerta = Arc::new(Puerta::default());
    assert_eq!(puerta.abrir(&d, llave, Arc::new(Doble::default())), Err(NoAbre::Llavero));
    assert!(!ruta_en(&d).exists());
    assert!(!puerta.abierta());
}

#[test]
fn una_ruta_demasiado_larga_no_se_abre_en_otra_parte() {
    let d = carpeta("larga").join("x".repeat(RUTA_MAXIMA));
    let puerta = Arc::new(Puerta::default());
    let r = puerta.abrir(&d, Arc::new(LlaveDePrueba::default()), Arc::new(Doble::default()));
    assert_eq!(r, Err(NoAbre::RutaLarga));
}

/// **Un token por apertura**: la llave de la apertura anterior ya no abre.
#[test]
fn cada_apertura_trae_su_llave() {
    let (puerta, llave, doble, d) = abrir("dos-llaves");
    let vieja = llave.token();
    puerta.cerrar(Cierre::ATuMano);
    puerta.abrir(&d, llave.clone(), doble).unwrap();
    assert_ne!(vieja, llave.token());
    let r = pedir(&ruta_en(&d), &vieja, &Orden::Reindexar {}, UN_RATO).unwrap();
    assert_eq!(r, Respuesta::Denegado { motivo: Motivo::LlaveErrada });
    puerta.cerrar(Cierre::ATuMano);
}

/// Una orden a medias no deja la puerta colgada: quien corta a mitad de línea recibe un fallo en el
/// registro, y la puerta sigue atendiendo a la siguiente.
#[test]
fn una_orden_a_medias_no_la_cuelga() {
    let (puerta, llave, _doble, d) = abrir("colgada");
    let ruta = ruta_en(&d);
    let mut a_medias = UnixStream::connect(&ruta).unwrap();
    a_medias.write_all(b"{\"token\":").unwrap();
    drop(a_medias);
    let r = pedir(&ruta, &llave.token(), &Orden::LeerPrefs {}, UN_RATO).unwrap();
    assert!(matches!(r, Respuesta::Hecho { .. }));
    let registro = puerta.vista(None).registro;
    assert_eq!(registro.len(), 2);
    assert_eq!((registro[1].orden.as_str(), &registro[1].resultado), ("ghost ?", &Resultado::Fallo));
    puerta.cerrar(Cierre::ATuMano);
}

#[test]
fn al_arrancar_se_borra_lo_que_dejo_una_caida() {
    let d = carpeta("caida");
    let ruta = ruta_en(&d);
    let viejo = UnixListener::bind(&ruta).unwrap();
    drop(viejo);
    let llave = LlaveDePrueba::default();
    llave.guardar("quedaba").unwrap();
    assert!(limpiar_lo_que_quedo(&d, &llave));
    assert!(!ruta.exists());
    assert!(llave.token.lock().unwrap().is_none());
    assert!(!limpiar_lo_que_quedo(&d, &llave), "sin socket, no hay nada que limpiar");
}

/// **La red no se entera**: una sesión entera de la puerta no suma un byte al contador. Lo que lo
/// hace estructural es el gate `puerta-solo-local` (un socket Unix no sabe salir del Mac); esto
/// comprueba que nadie haya metido la puerta por el camino que sí se cuenta.
#[test]
fn el_contador_de_red_no_se_mueve() {
    let antes = app_copiloto_consultor_lib::red::bytes();
    let (puerta, llave, _doble, d) = abrir("red");
    for orden in [Orden::Reindexar {}, Orden::LeerPrefs {}, Orden::Buscar { texto: "limpieza".into() }] {
        pedir(&ruta_en(&d), &llave.token(), &orden, UN_RATO).unwrap();
    }
    puerta.cerrar(Cierre::ATuMano);
    assert_eq!(app_copiloto_consultor_lib::red::bytes(), antes);
}

#[test]
fn la_carpeta_de_ghost_es_la_de_tauri() {
    assert_eq!(
        carpeta_de_la_app(Path::new("/Users/ana")),
        PathBuf::from("/Users/ana/Library/Application Support/com.aiapps.copiloto-consultor")
    );
}

/// **EL KIT POR LA PUERTA** (kit v2, sprint 003): `ghost kit docs/kit-de-prueba/preguntas.json` mide lo
/// mismo que la CI. Las treinta preguntas del kit cruzan el socket en una sola línea —dentro del tope
/// de 1 MiB—, la app las corre con el mismo `corpus::evaluar` que el test del retriever, y el informe
/// vuelve entero por la otra línea. Lo que se prueba aquí es el camino, no la nota: si una pregunta se
/// perdiera en la ida o un campo en la vuelta, el nDCG@5 por la puerta no sería el de la CI.
#[test]
fn el_kit_por_la_puerta_mide_lo_mismo_que_la_ci() {
    use app_copiloto_consultor_lib::corpus::evaluar::{evaluar, Kit};
    use app_copiloto_consultor_lib::corpus::Corpus;

    /// La app, con el corpus del kit indexado: hace el kit como `hacer_por_la_puerta` en `lib.rs`.
    struct ConElKit(Corpus);
    impl Operaciones for ConElKit {
        fn en_reunion(&self) -> bool {
            false
        }
        fn hacer(&self, orden: &Orden) -> Result<Hecho, String> {
            let Orden::Kit { kit } = orden else { return Err("solo el kit".into()) };
            let informe = evaluar(&self.0, kit)?;
            Ok(Hecho { cuenta: Some(informe.preguntas as u32), datos: serde_json::to_value(&informe).map_err(|e| e.to_string())? })
        }
        fn hora(&self) -> String {
            "11:20".into()
        }
        fn avisar(&self) {}
    }

    let raiz = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/kit-de-prueba");
    // Como `ghost`: el archivo lo lee el cliente, no la app (ADR 018 §4).
    let texto = std::fs::read_to_string(format!("{raiz}/preguntas.json")).expect("falta preguntas.json");
    let kit: Kit = serde_json::from_str(&texto).expect("preguntas.json no tiene la forma del kit");
    let mut corpus = Corpus::en_memoria().unwrap();
    corpus.indexar(Path::new(&format!("{raiz}/corpus")), &|_| {}).expect("no se indexó el kit");
    let directo = evaluar(&corpus, &kit).expect("el kit no se pudo evaluar");

    let d = carpeta("kit");
    let (puerta, llave) = (Arc::new(Puerta::default()), Arc::new(LlaveDePrueba::default()));
    puerta.abrir(&d, llave.clone(), Arc::new(ConElKit(corpus))).expect("la puerta no se abrió");
    let r = pedir(&ruta_en(&d), &llave.token(), &Orden::Kit { kit: kit.clone() }, Duration::from_secs(30)).unwrap();
    puerta.cerrar(Cierre::ATuMano);
    let _ = std::fs::remove_dir_all(&d);

    let Respuesta::Hecho { datos } = r else { panic!("la puerta no hizo el kit: {r:?}") };
    let por_la_puerta = &datos["ndcg5"];
    println!("[kit por la puerta] nDCG@5 {por_la_puerta} · directo {:.3} · preguntas {}", directo.ndcg5, directo.preguntas);
    assert_eq!(datos["preguntas"], serde_json::json!(kit.preguntas.len()), "se perdieron preguntas en el camino");
    assert_eq!(por_la_puerta.as_f64(), Some(directo.ndcg5), "por la puerta no mide lo mismo que la CI");
    assert_eq!(datos["rechazo"].as_f64(), directo.rechazo, "el rechazo no volvió igual");
    let fallos = datos["fallos"].as_array().map(Vec::len).unwrap_or(usize::MAX);
    assert_eq!(fallos, directo.fallos.len(), "los fallos no volvieron enteros");
}
