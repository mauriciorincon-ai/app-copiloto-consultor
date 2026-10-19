//! LA PUERTA LOCAL EN WINDOWS, con un named pipe de verdad (sprint 012, ADR 018 y ADR 022 enmienda 4). La hermana de
//! `puerta.rs`, que corre solo en Unix: la misma política, el mismo hilo que atiende y vigila, y lo propio de Windows
//! —la DACL leída del pipe, que nadie se le adelante con el mismo nombre, y la instancia que se reutiliza—.
//!
//! **Ninguna prueba toca el Administrador de credenciales de verdad** (regla 22 y regla 25): la llave es un doble. Ni la
//! app: las operaciones son otro doble, con la reunión a voluntad. Un named pipe no es hardware ni un permiso: lo crea y
//! lo cierra el propio proceso, y no queda nada en el disco.
#![cfg(windows)]

use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use app_copiloto_consultor_lib::puerta::socket::{
    dacl_de, hay_puerta, limpiar_lo_que_quedo, pedir, ruta_de_la_puerta, ruta_en, sid_del_usuario, NoLlega, Puerta,
};
use app_copiloto_consultor_lib::puerta::{Cierre, Hecho, Llave, Motivo, NoAbre, Operaciones, Orden, Respuesta, Resultado};

/// La app, de mentira: reunión a voluntad y órdenes que se cuentan.
#[derive(Default)]
struct Doble {
    reunion: AtomicBool,
    hechas: AtomicUsize,
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
    fn avisar(&self) {}
}

/// El Administrador de credenciales, de mentira.
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
        self.token.lock().unwrap().clone().expect("no hay token en la llave de prueba")
    }
}

/// Una carpeta de prueba: su nombre da a cada prueba su propio pipe (no chocan entre sí ni con la app abierta).
fn carpeta(nombre: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ag-puerta-{nombre}-{}", std::process::id()))
}

fn abrir(nombre: &str) -> (Arc<Puerta>, Arc<LlaveDePrueba>, Arc<Doble>, PathBuf) {
    let d = carpeta(nombre);
    let (puerta, llave, doble) =
        (Arc::new(Puerta::con_vigia(Duration::from_millis(200))), Arc::new(LlaveDePrueba::default()), Arc::new(Doble::default()));
    puerta.abrir(&d, llave.clone(), doble.clone()).expect("la puerta no se abrió");
    (puerta, llave, doble, ruta_en(&d))
}

const UN_RATO: Duration = Duration::from_secs(5);

/// Un cliente a mano, como haría cualquiera: si la instancia aún está soltando la orden anterior, espera y reintenta
/// (`ghost` lo hace con `WaitNamedPipeW`).
fn a_mano(ruta: &std::path::Path) -> std::fs::File {
    for _ in 0..40 {
        match std::fs::OpenOptions::new().read(true).write(true).open(ruta) {
            Ok(f) => return f,
            Err(e) if e.raw_os_error() == Some(231) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => panic!("no se pudo abrir el pipe: {e}"),
        }
    }
    panic!("el pipe siguió ocupado 2 s")
}

fn esperar(que: impl Fn() -> bool) -> bool {
    for _ in 0..100 {
        if que() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    false
}

/// El pipe tiene el nombre de la app **y el SID de tu usuario** (otra sesión de Windows con la app abierta no le quita el
/// nombre: auditoría del S12, B11), y una carpeta de prueba le da uno propio.
#[test]
fn el_pipe_de_la_app_se_llama_como_la_app() {
    let app = ruta_en(&std::env::temp_dir().join("com.aiapps.copiloto-consultor"));
    let sid = sid_del_usuario().expect("el SID del usuario");
    assert_eq!(app.to_string_lossy(), format!(r"\\.\pipe\com.aiapps.copiloto-consultor.puerta.{sid}"));
    let prueba = ruta_en(&carpeta("nombre"));
    assert!(prueba.to_string_lossy().starts_with(r"\\.\pipe\com.aiapps.copiloto-consultor.puerta.ag-puerta-nombre-"));
}

/// **Cerrar y abrir de seguido funciona** (auditoría del S12, B2): el pipe cerrado tarda un instante en soltar su nombre
/// y abrir otra vez no dice «otro programa ya usa ese nombre» con la verdad al revés. ¿Puede fallar? Sí: sin los
/// reintentos de `enlazar`, alguna de las vueltas falla.
#[test]
fn cerrar_y_abrir_de_seguido_no_dice_que_el_nombre_esta_ocupado() {
    let d = carpeta("de-seguido");
    let (puerta, llave, doble) = (Arc::new(Puerta::default()), Arc::new(LlaveDePrueba::default()), Arc::new(Doble::default()));
    for vuelta in 0..6 {
        puerta.abrir(&d, llave.clone(), doble.clone()).unwrap_or_else(|e| panic!("la vuelta {vuelta} no abrió: {e:?}"));
        assert!(puerta.cerrar(Cierre::ATuMano));
    }
}

/// **El arranque no borra la llave de otra app viva** (auditoría del S12, M4): con un pipe vivo la llave no es una sobra;
/// sin pipe sí lo es, y se borra. ¿Puede fallar? Sí: sin el `existe` de `limpiar_lo_que_quedo`, la primera mitad es roja.
#[test]
fn el_arranque_no_borra_la_llave_de_otra_app_viva_y_si_la_que_quedo() {
    let (puerta, _llave, _doble, _ruta) = abrir("limpia");
    let ajena = LlaveDePrueba::default();
    ajena.guardar("la-de-la-otra-app").unwrap();
    assert!(!limpiar_lo_que_quedo(&carpeta("limpia"), &ajena));
    assert!(ajena.token.lock().unwrap().is_some(), "el arranque borró la llave de una app que tiene la puerta abierta");
    puerta.cerrar(Cierre::ATuMano);
    assert!(esperar(|| !hay_puerta(&ruta_en(&carpeta("limpia")))), "el pipe sobrevivió al cierre");
    assert!(!limpiar_lo_que_quedo(&carpeta("limpia"), &ajena));
    assert!(ajena.token.lock().unwrap().is_none(), "una llave que quedó de una caída no se borró");
}

/// **`ghost.exe` de verdad, con la puerta cerrada** (auditoría del S12, B8): sale con 3 sin conectarse a nada y **sin
/// tocar el Administrador de credenciales** (mira el pipe antes que la llave), y su ayuda habla de Windows. Si la app
/// tiene la puerta abierta en este equipo, no la usa: la prueba se salta.
#[test]
fn ghost_con_la_puerta_cerrada_sale_con_3_sin_tocar_la_llave_y_habla_de_windows() {
    let Some(ruta) = ruta_de_la_puerta() else { return };
    if hay_puerta(&ruta) {
        eprintln!("la app tiene la puerta abierta en este equipo: esta prueba no la toca");
        return;
    }
    let ghost = env!("CARGO_BIN_EXE_ghost");
    let salida = std::process::Command::new(ghost).args(["prefs", "leer"]).output().expect("ghost no arrancó");
    assert_eq!(salida.status.code(), Some(3), "con la puerta cerrada ghost sale con 3: {salida:?}");
    for idioma in ["en_US.UTF-8", "es_CO.UTF-8"] {
        let ayuda = std::process::Command::new(ghost).arg("--help").env("LANG", idioma).output().expect("ghost --help");
        let texto = String::from_utf8_lossy(&ayuda.stdout);
        assert!(
            !["Touch ID", "macOS", "Llavero", "Keychain"].iter().any(|m| texto.contains(m)),
            "ghost --help en Windows habla del Mac ({idioma}):\n{texto}"
        );
        assert!(texto.contains("Windows Hello"), "la ayuda no dice cuál es la identidad en Windows ({idioma})");
    }
}

/// **Nace cerrada**: sin pipe, y `ghost` lo sabe sin tocar la llave.
#[test]
fn nace_cerrada() {
    let ruta = ruta_en(&carpeta("nace"));
    let puerta = Puerta::default();
    assert!(!puerta.abierta());
    assert!(!hay_puerta(&ruta));
    assert_eq!(pedir(&ruta, "x", &Orden::Reindexar {}, UN_RATO), Err(NoLlega::Cerrada));
}

/// De punta a punta, con un pipe de verdad: abre, atiende con la llave y cierra sin dejar nada.
#[test]
fn abierta_atiende_con_la_llave_y_al_cerrar_no_queda_nada() {
    let (puerta, llave, doble, ruta) = abrir("punta");
    assert!(hay_puerta(&ruta), "la puerta abierta no se ve");
    let r = pedir(&ruta, &llave.token(), &Orden::Reindexar {}, UN_RATO).unwrap();
    assert!(matches!(r, Respuesta::Hecho { .. }), "{r:?}");
    let r = pedir(&ruta, "otra", &Orden::Reindexar {}, UN_RATO).unwrap();
    assert_eq!(r, Respuesta::Denegado { motivo: Motivo::LlaveErrada });
    assert_eq!(doble.hechas.load(Ordering::SeqCst), 1);

    let v = puerta.vista(None);
    assert_eq!(v.registro.len(), 2);
    assert_eq!(v.registro[1].resultado, Resultado::Hecho { cuenta: Some(28) });

    assert!(puerta.cerrar(Cierre::ATuMano));
    assert!(llave.token.lock().unwrap().is_none(), "la llave sobrevivió al cierre");
    assert!(esperar(|| !hay_puerta(&ruta)), "el pipe sobrevivió al cierre");
    assert_eq!(pedir(&ruta, "x", &Orden::Reindexar {}, UN_RATO), Err(NoLlega::Cerrada));
}

/// **La misma instancia atiende una orden tras otra**, aunque alguien conecte y se vaya sin decir nada: la puerta la
/// desconecta y la deja para la siguiente.
#[test]
fn la_instancia_se_reutiliza_entre_ordenes() {
    let (puerta, llave, _doble, ruta) = abrir("reutiliza");
    for _ in 0..3 {
        let r = pedir(&ruta, &llave.token(), &Orden::Reindexar {}, UN_RATO).unwrap();
        assert!(matches!(r, Respuesta::Hecho { .. }), "{r:?}");
    }
    // Uno que se conecta y se va sin mandar nada.
    drop(a_mano(&ruta));
    let r = pedir(&ruta, &llave.token(), &Orden::Reindexar {}, UN_RATO).unwrap();
    assert!(matches!(r, Respuesta::Hecho { .. }), "tras una conexión vacía la puerta dejó de atender: {r:?}");
    // Y uno que manda media línea y se va.
    {
        let mut a_medias = a_mano(&ruta);
        a_medias.write_all(br#"{"token":"#).unwrap();
    }
    let r = pedir(&ruta, &llave.token(), &Orden::Reindexar {}, UN_RATO).unwrap();
    assert!(matches!(r, Respuesta::Hecho { .. }), "tras una orden a medias la puerta dejó de atender: {r:?}");
    puerta.cerrar(Cierre::ATuMano);
}

/// **En reunión se deniega y se cierra sola.** Y no se vuelve a abrir sola.
#[test]
fn una_orden_en_reunion_se_deniega_y_cierra_la_puerta() {
    let (puerta, llave, doble, ruta) = abrir("reunion");
    doble.reunion.store(true, Ordering::SeqCst);
    let r = pedir(&ruta, &llave.token(), &Orden::LeerPrefs {}, UN_RATO).unwrap();
    assert_eq!(r, Respuesta::Denegado { motivo: Motivo::EnReunion });
    assert!(!puerta.abierta(), "la puerta siguió abierta en reunión");
    assert_eq!(puerta.vista(None).cerro, Some(Cierre::EnReunion));
    assert!(esperar(|| !hay_puerta(&ruta)), "el pipe siguió abierto en reunión");
    doble.reunion.store(false, Ordering::SeqCst);
    std::thread::sleep(Duration::from_millis(400));
    assert!(!puerta.abierta(), "la puerta se volvió a abrir sola");
}

/// **El vigía**: aunque nadie llame, en cuanto empieza una reunión la puerta se cierra y el pipe desaparece.
#[test]
fn el_vigia_la_cierra_al_empezar_una_reunion() {
    let (puerta, llave, doble, ruta) = abrir("vigia");
    doble.reunion.store(true, Ordering::SeqCst);
    assert!(esperar(|| !puerta.abierta()), "el vigía no cerró la puerta");
    assert!(esperar(|| !hay_puerta(&ruta)), "el pipe siguió abierto");
    assert!(llave.token.lock().unwrap().is_none(), "la llave siguió guardada");
    assert_eq!(puerta.vista(None).cerro, Some(Cierre::EnReunion));
}

#[test]
fn en_reunion_no_se_abre() {
    let d = carpeta("no-abre");
    let (puerta, doble) = (Arc::new(Puerta::default()), Arc::new(Doble::default()));
    doble.reunion.store(true, Ordering::SeqCst);
    let llave = Arc::new(LlaveDePrueba::default());
    assert_eq!(puerta.abrir(&d, llave.clone(), doble), Err(NoAbre::EnReunion));
    assert!(!hay_puerta(&ruta_en(&d)));
    assert!(llave.token.lock().unwrap().is_none());
}

/// Si la llave no se guarda, la puerta no se queda a medio abrir con un pipe sin llave.
#[test]
fn sin_llave_no_se_abre_ni_deja_pipe() {
    let d = carpeta("sin-llave");
    let llave = Arc::new(LlaveDePrueba { falla: true, ..Default::default() });
    let puerta = Arc::new(Puerta::default());
    assert_eq!(puerta.abrir(&d, llave, Arc::new(Doble::default())), Err(NoAbre::Llavero));
    assert!(!puerta.abierta());
    assert!(esperar(|| !hay_puerta(&ruta_en(&d))), "quedó un pipe sin llave");
}

/// **Solo tu usuario entra**: la DACL que tiene de verdad el pipe, leída desde fuera, trae una sola entrada y es la de
/// tu SID; ni «Everyone», ni los usuarios autenticados, ni los administradores, ni el anónimo. ¿Puede fallar? Sí: crear
/// el pipe sin atributos de seguridad (la DACL por defecto deja leer a Everyone) lo pone rojo.
#[test]
fn la_dacl_del_pipe_es_solo_del_usuario() {
    let (puerta, _llave, _doble, ruta) = abrir("dacl");
    let sddl = dacl_de(&ruta).expect("no se pudo leer la DACL del pipe");
    let sid = sid_del_usuario().unwrap();
    assert!(sddl.starts_with("D:P"), "la DACL hereda permisos: {sddl}");
    assert_eq!(sddl.matches('(').count(), 1, "la DACL trae más de una entrada: {sddl}");
    // Windows escribe la cuenta de administrador integrada (RID 500) con su alias `LA`, no con el SID: así la ve la CI, que
    // corre con esa cuenta. Es el mismo SID; cualquier otra cuenta se compara con su SID completo.
    let trustee = if sid.ends_with("-500") { "LA".to_string() } else { sid.clone() };
    assert!(sddl.starts_with("D:P(A;;") && sddl.ends_with(&format!(";;;{trustee})")), "la entrada no es la de tu SID: {sddl}");
    for ajeno in [";;;WD)", ";;;AU)", ";;;BU)", ";;;BA)", ";;;AN)", ";;;SY)"] {
        assert!(!sddl.contains(ajeno), "la DACL deja pasar a {ajeno}: {sddl}");
    }
    puerta.cerrar(Cierre::ATuMano);
}

/// **Nadie se adelanta**: con la puerta abierta, otro que intente crear un pipe con el mismo nombre no puede (y la app,
/// si el nombre ya estaba tomado, no abre). ¿Puede fallar? Sí: sin `FILE_FLAG_FIRST_PIPE_INSTANCE` o con más de una
/// instancia, la segunda apertura entra.
#[test]
fn nadie_ocupa_el_nombre_de_la_puerta() {
    let (puerta, _llave, _doble, _ruta) = abrir("ocupada");
    let otra = Arc::new(Puerta::default());
    let llave = Arc::new(LlaveDePrueba::default());
    assert_eq!(
        otra.abrir(&carpeta("ocupada"), llave.clone(), Arc::new(Doble::default())),
        Err(NoAbre::Socket),
        "una segunda puerta se abrió con el mismo nombre"
    );
    assert!(llave.token.lock().unwrap().is_none(), "la segunda puerta guardó una llave");
    puerta.cerrar(Cierre::ATuMano);
}
