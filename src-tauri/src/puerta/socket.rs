//! EL SOCKET DE LA PUERTA (ADR 018 §1 y §3). Parte del módulo protegido: las únicas líneas que
//! tocan el disco o escriben en el socket llevan su marca y el ADR en la misma línea.
//!
//! **Un socket Unix en la carpeta privada de la app (700), en 600, y solo mientras la puerta está
//! abierta.** Lo alcanza el usuario de este Mac y nadie más: ni otra cuenta ni, por construcción, otra
//! máquina (regla dura 9). Una conexión, una orden: una línea JSON de ida, una de vuelta.
//!
//! **Un solo hilo por apertura**, que atiende y vigila: espera conexiones sin bloquearse y, entre una y
//! otra, mira cada [`VIGIA`](super::VIGIA) si empezó una reunión. Al cerrar la puerta el hilo lo ve en
//! su siguiente vuelta y se va: no hay que despertarlo con una conexión falsa, ni queda un hilo colgado
//! de un `accept` si alguien borró el socket por fuera.

use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{
    resolver, Cierre, Entrada, Llave, Motivo, NoAbre, Operaciones, Orden, Registro, Respuesta, Resultado, Token,
    VistaDeLaPuerta, ESPERA, IDENTIFICADOR, SOCKET, TOPE_DE_LINEA, VIGIA,
};

/// Lo más larga que macOS admite la ruta de un socket: `sun_path` mide 104 bytes, con el cero final.
pub const RUTA_MAXIMA: usize = 103;

/// Cada cuánto mira el hilo si hay conexión o si la puerta se cerró.
const PASO: Duration = Duration::from_millis(100);

/// Lo más larga que puede ser una respuesta: una reunión guardada entera cabe; una sin fin, no.
const TOPE_DE_RESPUESTA: u64 = 16 << 20;

/// El socket, dentro de la carpeta de la app.
pub fn ruta_en(carpeta_de_la_app: &Path) -> PathBuf {
    carpeta_de_la_app.join(SOCKET)
}

/// La carpeta de la app a partir de la carpeta del usuario, como la calcula Tauri en macOS. Es la que
/// usa `ghost`, que no tiene Tauri.
pub fn carpeta_de_la_app(casa: &Path) -> PathBuf {
    casa.join("Library").join("Application Support").join(IDENTIFICADOR)
}

/// **La puerta**, con su estado: cerrada o abierta (con su token y su época), por qué se cerró, por
/// qué no se abrió y el registro de esta sesión de la app.
pub struct Puerta {
    dentro: Mutex<Dentro>,
    vigia: Duration,
}

#[derive(Default)]
struct Dentro {
    abierta: Option<Abierta>,
    /// Sube en cada apertura: el hilo de una apertura vieja se reconoce y se va.
    epoca: u64,
    cerro: Option<Cierre>,
    no_abre: Option<NoAbre>,
    registro: Registro,
}

struct Abierta {
    ruta: PathBuf,
    epoca: u64,
    llave: Arc<dyn Llave>,
}

impl Default for Puerta {
    fn default() -> Self {
        Puerta::con_vigia(VIGIA)
    }
}

impl Puerta {
    /// Una puerta cerrada, con el vigía que se le diga (las pruebas lo quieren corto).
    pub fn con_vigia(vigia: Duration) -> Self {
        Puerta { dentro: Mutex::new(Dentro::default()), vigia }
    }

    pub fn abierta(&self) -> bool {
        self.dentro.lock().map(|d| d.abierta.is_some()).unwrap_or(false)
    }

    /// Lo que IA enseña. `ghost` es la ruta del binario, si está compilado junto a la app.
    pub fn vista(&self, ghost: Option<String>) -> VistaDeLaPuerta {
        let d = self.dentro.lock().expect("la puerta quedó en mal estado");
        VistaDeLaPuerta {
            abierta: d.abierta.is_some(),
            cerro: d.cerro,
            no_abre: d.no_abre,
            ghost,
            registro: d.registro.entradas(),
        }
    }

    /// **Abre la puerta**: el socket en 600, un token nuevo en el Llavero y el hilo que atiende y
    /// vigila. En reunión no se abre. Abrirla abierta no hace nada.
    pub fn abrir(
        self: &Arc<Self>,
        carpeta_de_la_app: &Path,
        llave: Arc<dyn Llave>,
        ops: Arc<dyn Operaciones>,
    ) -> Result<(), NoAbre> {
        let hecho = self.intentar_abrir(carpeta_de_la_app, llave, ops.clone());
        if let Ok(mut d) = self.dentro.lock() {
            d.no_abre = hecho.err();
        }
        ops.avisar();
        hecho
    }

    fn intentar_abrir(
        self: &Arc<Self>,
        carpeta: &Path,
        llave: Arc<dyn Llave>,
        ops: Arc<dyn Operaciones>,
    ) -> Result<(), NoAbre> {
        if self.abierta() {
            return Ok(());
        }
        if ops.en_reunion() {
            return Err(NoAbre::EnReunion);
        }
        let ruta = ruta_en(carpeta);
        if ruta.as_os_str().len() > RUTA_MAXIMA {
            println!("[puerta] la ruta del socket pasa de {RUTA_MAXIMA} bytes: no se abre");
            return Err(NoAbre::RutaLarga);
        }
        // El socket nace con el umask y se aprieta a 600 un instante después: en ese instante lo
        // protege la carpeta. Si no está en 700, no se crea nada (auditoría del S3, B17).
        if !carpeta_en_700(carpeta) {
            println!("[puerta] la carpeta de la app no está en 700: no se abre");
            return Err(NoAbre::Socket);
        }
        quitar(&ruta);
        let escucha = UnixListener::bind(&ruta).map_err(|e| {
            println!("[puerta] no se pudo crear el socket: {e}");
            NoAbre::Socket
        })?;
        if let Err(e) = cerrar_permisos(&ruta) {
            println!("[puerta] el socket no quedó en 600 ({e}): no se abre");
            quitar(&ruta);
            return Err(NoAbre::Socket);
        }
        let token = Token::nuevo();
        if let Err(e) = llave.guardar(token.como_texto()) {
            println!("[puerta] el Llavero no guardó el token ({e}): no se abre");
            quitar(&ruta);
            return Err(NoAbre::Llavero);
        }
        let token = Arc::new(token);
        let epoca = {
            let mut d = self.dentro.lock().map_err(|_| NoAbre::Socket)?;
            d.epoca += 1;
            d.abierta = Some(Abierta { ruta, epoca: d.epoca, llave });
            d.cerro = None;
            d.epoca
        };
        let puerta = self.clone();
        std::thread::spawn(move || servir(puerta, escucha, token, epoca, ops));
        println!("[puerta] abierta");
        Ok(())
    }

    /// **Cierra la puerta**: borra el token del Llavero y el socket. Devuelve si estaba abierta.
    pub fn cerrar(&self, por: Cierre) -> bool {
        self.cerrar_la(None, por)
    }

    /// Cierra la apertura `epoca` (o cualquiera, con `None`): el hilo de una apertura vieja no puede
    /// cerrar la nueva.
    ///
    /// **El socket y el token se van ANTES de que la puerta diga «cerrada»**, dentro del mismo candado.
    /// Al revés —anunciar primero y borrar después— hubo un instante en que IA decía «cerrada» con el
    /// socket todavía en el disco y el token en el Llavero; la CI lo cazó en un runner lento (bitácora,
    /// fase 4).
    fn cerrar_la(&self, epoca: Option<u64>, por: Cierre) -> bool {
        {
            let Ok(mut d) = self.dentro.lock() else { return false };
            match &d.abierta {
                Some(a) if epoca.is_none_or(|e| e == a.epoca) => {
                    a.llave.borrar();
                    quitar(&a.ruta);
                    d.abierta = None;
                    d.cerro = Some(por);
                }
                _ => return false,
            }
        }
        println!("[puerta] cerrada ({})", if por == Cierre::EnReunion { "sola: hay reunión" } else { "a mano" });
        true
    }

    fn sigue(&self, epoca: u64) -> bool {
        self.dentro.lock().map(|d| d.abierta.as_ref().is_some_and(|a| a.epoca == epoca)).unwrap_or(false)
    }

    fn apuntar(&self, e: Entrada) {
        if let Ok(mut d) = self.dentro.lock() {
            d.registro.apuntar(e);
        }
    }
}

/// Lo que queda de una caída con la puerta abierta: el socket, y con él el token del Llavero. Se
/// mira el socket primero, para no preguntarle nada al Llavero en el arranque normal.
pub fn limpiar_lo_que_quedo(carpeta_de_la_app: &Path, llave: &dyn Llave) -> bool {
    let ruta = ruta_en(carpeta_de_la_app);
    if !ruta.exists() {
        return false;
    }
    quitar(&ruta);
    llave.borrar();
    println!("[puerta] quedaba un socket de una sesión anterior: borrado, y su token con él");
    true
}

fn quitar(ruta: &Path) {
    let _ = std::fs::remove_file(ruta); // verify-ephemeral:allow — ADR 018 §1: el socket se borra al cerrar la puerta
}

#[cfg(unix)]
fn carpeta_en_700(carpeta: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(carpeta).is_ok_and(|m| m.is_dir() && m.permissions().mode() & 0o777 == 0o700) // verify-ephemeral:allow — ADR 018 §1: la carpeta del socket, en 700, antes de crearlo
}

#[cfg(not(unix))]
fn carpeta_en_700(_carpeta: &Path) -> bool {
    false
}

#[cfg(unix)]
fn cerrar_permisos(ruta: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(0o600)) // verify-ephemeral:allow — ADR 018 §1: el socket, en 600
}

/// El hilo de una apertura: atiende de una en una y, entre conexiones, vigila la reunión.
fn servir(puerta: Arc<Puerta>, escucha: UnixListener, token: Arc<Token>, epoca: u64, ops: Arc<dyn Operaciones>) {
    if let Err(e) = escucha.set_nonblocking(true) {
        println!("[puerta] el socket no se dejó esperar sin bloquear ({e}): se cierra");
        puerta.cerrar_la(Some(epoca), Cierre::ATuMano);
        ops.avisar();
        return;
    }
    let mut sin_vigilar = Duration::ZERO;
    while puerta.sigue(epoca) {
        match escucha.accept() {
            Ok((conexion, _)) => {
                atender(&puerta, conexion, &token, &*ops);
                ops.avisar();
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(PASO);
                sin_vigilar += PASO;
                if sin_vigilar >= puerta.vigia {
                    sin_vigilar = Duration::ZERO;
                    if ops.en_reunion() && puerta.cerrar_la(Some(epoca), Cierre::EnReunion) {
                        ops.avisar();
                    }
                }
            }
            Err(_) => std::thread::sleep(PASO),
        }
    }
}

/// Una conexión: lee la línea, la resuelve, contesta y apunta. Una orden en reunión cierra la puerta.
fn atender(puerta: &Puerta, mut conexion: UnixStream, token: &Token, ops: &dyn Operaciones) {
    // En macOS la conexión hereda el «sin bloquear» de quien escucha.
    let _ = conexion.set_nonblocking(false);
    let _ = conexion.set_read_timeout(Some(ESPERA));
    let _ = conexion.set_write_timeout(Some(ESPERA));
    let (respuesta, entrada) = match leer_linea(&conexion) {
        Some(linea) => resolver(token, &linea, ops),
        None => (
            Respuesta::Fallo { error: "la orden no llegó entera".into() },
            Entrada { hora: ops.hora(), orden: "ghost ?".into(), resultado: Resultado::Fallo },
        ),
    };
    // **Primero el estado, después la respuesta**: quien recibe «denegado · en reunión» encuentra la puerta
    // ya cerrada y la orden ya en el registro. Al revés, había un instante en que no.
    puerta.apuntar(entrada);
    if respuesta == (Respuesta::Denegado { motivo: Motivo::EnReunion }) {
        puerta.cerrar(Cierre::EnReunion);
    }
    let mut linea = serde_json::to_string(&respuesta).unwrap_or_default();
    linea.push('\n');
    let _ = conexion.write_all(linea.as_bytes()); // verify-ephemeral:allow — ADR 018 §3: la respuesta vuelve por el socket
}

/// Una línea, de [`TOPE_DE_LINEA`] como mucho. `None` si no llegó entera o pasa del tope.
fn leer_linea(conexion: &UnixStream) -> Option<String> {
    let mut linea = String::new();
    BufReader::new(conexion.take(TOPE_DE_LINEA as u64 + 1)).read_line(&mut linea).ok()?;
    (linea.ends_with('\n') && linea.len() <= TOPE_DE_LINEA).then_some(linea)
}

/// Por qué una orden no llegó a la puerta.
#[derive(Debug, PartialEq, Eq)]
pub enum NoLlega {
    /// No hay socket, o no contesta: la puerta está cerrada.
    Cerrada,
    Fallo(String),
}

/// **El lado de `ghost`**: manda una orden y espera la respuesta. `espera` es generosa: reindexar o
/// esperar a que el usuario ponga el dedo en Touch ID lleva su tiempo.
pub fn pedir(ruta: &Path, token: &str, orden: &Orden, espera: Duration) -> Result<Respuesta, NoLlega> {
    let mut conexion = UnixStream::connect(ruta).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound | std::io::ErrorKind::ConnectionRefused => NoLlega::Cerrada,
        _ => NoLlega::Fallo(e.to_string()),
    })?;
    conexion.set_read_timeout(Some(espera)).map_err(|e| NoLlega::Fallo(e.to_string()))?;
    let mut linea = serde_json::json!({ "token": token, "orden": orden }).to_string();
    linea.push('\n');
    conexion.write_all(linea.as_bytes()).map_err(|e| NoLlega::Fallo(e.to_string()))?; // verify-ephemeral:allow — ADR 018 §3: la orden va por el socket
    let mut vuelta = String::new();
    BufReader::new((&conexion).take(TOPE_DE_RESPUESTA))
        .read_line(&mut vuelta)
        .map_err(|e| NoLlega::Fallo(e.to_string()))?;
    if vuelta.is_empty() {
        return Err(NoLlega::Cerrada);
    }
    serde_json::from_str(&vuelta).map_err(|e| NoLlega::Fallo(format!("la puerta contestó algo que no se entiende: {e}")))
}
