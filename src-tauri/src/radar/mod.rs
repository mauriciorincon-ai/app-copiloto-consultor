//! EL RADAR (C14) — quién graba la reunión y qué programas de **tu** Mac te miran.
//!
//! **MÓDULO PROTEGIDO.** Dos mitades, y las dos miran solo este equipo (regla dura 9 de la casa:
//! la app jamás sondea, escanea ni actúa sobre el computador de la contraparte):
//!
//! - **Ámbar, «sábelo»** ([`avisos`]): el aviso de grabación de Meet, Zoom o Teams y los bots de
//!   notas de la lista de participantes. Se leen del texto que la lectura de pantalla YA sacó de la
//!   ventana de la reunión —el radar no captura nada por su cuenta— y por eso este módulo tiene en
//!   las manos texto de un tercero: ni disco, ni red, ni log, y las copias se pisan al soltarse.
//! - **Coral, «invasivo»** ([`procesos`]): programas de este Mac que miran tu pantalla, tu cámara,
//!   tus teclas o tus procesos —supervisión de exámenes, monitoreo de empleados, acceso remoto—,
//!   cotejados contra un catálogo versionado y con fuente. La lista de procesos se le pide al
//!   núcleo de ESTE Mac; no hay otra máquina en la ecuación.
//!
//! **El catálogo viaja dentro del binario** (`data/radar/*.json`, incluido al compilar): el radar
//! no lee un archivo al arrancar ni consulta a nadie para actualizarse. Una versión nueva del
//! catálogo es una versión nueva de la app, con su diff y su revisión.
//!
//! **Qué NO hace, y lo vigila `tests/unit/radar-solo-este-mac.test.ts`:** no abre sockets, no
//! resuelve nombres, no lanza programas que hablen con otras máquinas. La única llamada al sistema
//! que sale del proceso es `/usr/bin/profiles` para saber si este Mac está inscrito en un MDM, y
//! está en la lista de ese gate con su razón. Tampoco bloquea ni cierra nada: **avisa**, y el
//! usuario decide.

pub mod avisos;
pub mod catalogo;
pub mod mdm;
pub mod procesos;

pub use avisos::Aviso;

/// De qué clase es un programa del catálogo. Cerrado: la pantalla lo nombra en dos idiomas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Categoria {
    /// Supervisión de exámenes (proctoring) y navegadores de examen que bloquean el equipo.
    Supervision,
    /// Anti-trampa con acceso al núcleo del sistema.
    AntiTrampa,
    /// Monitoreo de empleados: capturas periódicas, tiempo por app, a veces teclas.
    Monitoreo,
    /// Acceso remoto: alguien puede ver la pantalla y manejar el Mac.
    AccesoRemoto,
    /// Gestión de dispositivos de un tercero. Normal en equipos de empresa: «sábelo».
    Mdm,
}

impl Categoria {
    pub fn nivel(self) -> Nivel {
        match self {
            Categoria::Mdm => Nivel::Sabelo,
            _ => Nivel::Invasivo,
        }
    }
}

/// Los dos niveles del radar (mirada 3-bis): **sábelo** es información, **invasivo** es un
/// programa que mira tu equipo. Cada uno con su símbolo, su palabra y su color (regla 8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Nivel {
    Invasivo,
    Sabelo,
}

/// Un texto del catálogo en los dos idiomas de la app.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Bilingue {
    pub es: String,
    pub en: String,
}

/// Un programa del catálogo que **está corriendo en este Mac**.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Programa {
    /// El nombre del catálogo («TeamViewer»), no el del ejecutable.
    pub nombre: String,
    pub categoria: Categoria,
    pub nivel: Nivel,
    /// Lo que la banda dice que ve, en una frase corta: «ve tu pantalla completa y tu cámara».
    pub ve: Bilingue,
    /// Lo que Sesión escribe en la columna «qué alcanza a ver».
    pub alcance: Bilingue,
    /// La fuente de la fila. Se queda en Rust: vive en el catálogo versionado, que es donde se
    /// comprueba, y ninguna pantalla la pinta.
    #[serde(skip)]
    pub fuente: String,
}

/// Con qué catálogo se cotejó: Sesión escribe «v1 · 2026-09-26» y la banda «catálogo v1».
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
// camelCase aunque hoy todos sus campos sean de una palabra: el primero que no lo sea
// cruzaría en snake_case, que es el defecto C1 del sprint 001.
#[serde(rename_all = "camelCase")]
pub struct CatalogoDelRadar {
    pub version: u32,
    pub fecha: String,
}

/// **Lo que el radar vio en tu Mac**, invasivos primero.
#[derive(Clone, Debug, PartialEq, serde::Serialize)]
// camelCase aunque hoy todos sus campos sean de una palabra: el primero que no lo sea
// cruzaría en snake_case, que es el defecto C1 del sprint 001.
#[serde(rename_all = "camelCase")]
pub struct EnTuMac {
    pub programas: Vec<Programa>,
    pub catalogo: CatalogoDelRadar,
}

impl EnTuMac {
    /// Los nombres, para saber si algo cambió entre dos vueltas sin comparar textos enteros.
    pub fn nombres(&self) -> Vec<String> {
        self.programas.iter().map(|p| p.nombre.clone()).collect()
    }
}

/// **Una vuelta del radar coral**: los procesos de este Mac contra el catálogo, más la inscripción
/// en un MDM si ya se sabe (se pregunta aparte, porque cuesta lanzar un programa).
pub fn mirar_tu_mac(inscrito_en_un_mdm: bool) -> EnTuMac {
    let rutas = procesos::listar();
    let mut programas = procesos::cotejar(&rutas, catalogo::programas());
    if inscrito_en_un_mdm {
        if let Some(p) = catalogo::inscripcion() {
            if !programas.iter().any(|q| q.categoria == Categoria::Mdm) {
                programas.push(p);
            }
        }
    }
    programas.sort_by_key(|p| p.nivel);
    EnTuMac {
        programas,
        catalogo: catalogo::rotulo(),
    }
}
