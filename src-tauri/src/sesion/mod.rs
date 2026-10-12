//! DETECCIÓN DE REUNIÓN — qué videollamada tienes abierta, y si ahí la banda está verificada.
//!
//! La app no arranca sola: detectar sirve para **ofrecer**, nunca para encender. Lo que este
//! módulo produce es una frase que la pantalla de sesión muestra y un botón que espera.
//!
//! **Por qué es un catálogo y no una heurística.** Las videollamadas se reconocen por el
//! identificador de su aplicación, escrito en [`CATALOGO`], versionado en el repo, **sin consultar
//! a nadie por la red**. Adivinar por nombre —«algo que contenga *meeting*»— confundiría un
//! calendario con una llamada y, peor, lo haría de forma distinta en cada Mac.
//!
//! **Meet no es una aplicación, es una pestaña.** Zoom y Teams se detectan por su identificador y
//! ya está. Meet vive dentro de un navegador, y la única forma de distinguir «tiene Meet abierto»
//! de «tiene Chrome abierto» —que es siempre cierto— es mirar el **título de la ventana**. Eso
//! obliga a decir tres cosas en voz alta:
//!
//! - El título de una reunión **es información del cliente**. El estándar 4-T divide por de quién
//!   es, no por su formato: cae del mismo lado que el transcript. Vive en memoria mientras la
//!   pantalla lo muestra —la maqueta lo dibuja— y **no se escribe en ningún sitio**. Este módulo
//!   entra en la lista de los protegidos de `verify:ephemeral` por eso.
//! - Leer títulos **exige un permiso**. Sin él no decimos «no hay reunión», que sería mentira por
//!   omisión: decimos [`Reunion::NoSePuedeSaber`] con su motivo.
//! - El acople **no** lee títulos (ADR 004) y este módulo sí. Son dos usos distintos de la misma
//!   API y el que pide más tiene que justificarlo: sin el título no hay forma de saber si Chrome
//!   está en una llamada o en el correo.

use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

/// En Windows, la reunión se ve con los procesos y las ventanas del propio PC (sprint 007, ADR 022).
#[cfg(windows)]
#[path = "windows.rs"]
pub mod windows;

/// Qué es cada aplicación del catálogo para el detector.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Clase {
    /// Se detecta con que esté corriendo: si Zoom está abierto, hay llamada o la va a haber.
    Aplicacion,
    /// Solo cuenta si además una ventana suya delata la reunión. Chrome siempre está abierto.
    Navegador,
}

/// Hasta dónde llega la promesa de la banda **en este cliente de videollamada**.
///
/// No es un detalle: la promesa de invisibilidad es **graduada** por cliente y versión de macOS
/// (regla del vocabulario), y decir «protegido» donde solo hay «sin verificar» sería exactamente
/// la clase de afirmación que esta app no hace. La maqueta ya dibuja los dos estados.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum Proteccion {
    /// Comprobada por el usuario en su Mac, con fecha.
    Verificada,
    /// La bandera del sistema está puesta, pero nadie lo ha mirado en este cliente.
    SinVerificar,
}

/// Una entrada del catálogo de clientes de videollamada.
#[derive(Clone, Copy, Debug)]
pub struct Cliente {
    pub bundle: &'static str,
    pub nombre: &'static str,
    pub clase: Clase,
    pub proteccion: Proteccion,
}

/// **El catálogo, versión 1.** Crece por lista, no por ingenio.
pub const CATALOGO: &[Cliente] = &[
    Cliente { bundle: "us.zoom.xos", nombre: "Zoom", clase: Clase::Aplicacion, proteccion: Proteccion::SinVerificar },
    Cliente { bundle: "com.microsoft.teams2", nombre: "Microsoft Teams", clase: Clase::Aplicacion, proteccion: Proteccion::SinVerificar },
    Cliente { bundle: "com.microsoft.teams", nombre: "Microsoft Teams", clase: Clase::Aplicacion, proteccion: Proteccion::SinVerificar },
    Cliente { bundle: "com.google.Chrome", nombre: "Google Chrome", clase: Clase::Navegador, proteccion: Proteccion::Verificada },
    Cliente { bundle: "com.apple.Safari", nombre: "Safari", clase: Clase::Navegador, proteccion: Proteccion::SinVerificar },
    Cliente { bundle: "com.brave.Browser", nombre: "Brave", clase: Clase::Navegador, proteccion: Proteccion::SinVerificar },
    Cliente { bundle: "com.microsoft.edgemac", nombre: "Microsoft Edge", clase: Clase::Navegador, proteccion: Proteccion::SinVerificar },
    Cliente { bundle: "company.thebrowser.Browser", nombre: "Arc", clase: Clase::Navegador, proteccion: Proteccion::SinVerificar },
    Cliente { bundle: "org.mozilla.firefox", nombre: "Firefox", clase: Clase::Navegador, proteccion: Proteccion::SinVerificar },
];

/// La versión del catálogo, que se muestra en pantalla al lado de lo que afirma. Un catálogo sin
/// versión visible no se puede contrastar con nada.
pub const VERSION_CATALOGO: &str = "v1 · 2026-09-21";

/// **El catálogo de Windows** (sprint 007, ADR 022): la misma forma, otra clave. En Windows una aplicación se
/// reconoce por el nombre de su ejecutable, no por un identificador de paquete, y cada fila lleva la página
/// pública que lo nombra y la fecha en que se leyó (`data/clientes-windows.json`). **Nada verificado**: la banda
/// en Windows no la ha mirado nadie en Teams, Zoom ni Meet, y la fila lo dice.
const CLIENTES_WINDOWS: &str = include_str!("../../../data/clientes-windows.json");

/// El catálogo de Windows tal como viaja en el repo.
#[derive(Deserialize)]
struct CatalogoDeWindows {
    version: String,
    clientes: Vec<FilaDeWindows>,
}

/// Una fila del catálogo de Windows, con su fuente.
#[derive(Deserialize)]
pub struct FilaDeWindows {
    pub ejecutable: String,
    pub nombre: String,
    pub clase: Clase,
    pub proteccion: Proteccion,
    pub fuente: String,
    pub fecha: String,
}

fn de_windows() -> &'static (String, Vec<FilaDeWindows>, Vec<Cliente>) {
    static LEIDO: OnceLock<(String, Vec<FilaDeWindows>, Vec<Cliente>)> = OnceLock::new();
    LEIDO.get_or_init(|| {
        // El catálogo es del repo y lo comprueba su test: si no se lee, el binario no debería haber salido.
        let c: CatalogoDeWindows =
            serde_json::from_str(CLIENTES_WINDOWS).expect("data/clientes-windows.json no se lee");
        // `Cliente` es de cadenas estáticas (el catálogo del Mac es una constante). Las de Windows se leen una
        // vez por proceso y viven lo que el proceso: unas decenas de bytes.
        let clientes = c
            .clientes
            .iter()
            .map(|f| Cliente {
                bundle: Box::leak(f.ejecutable.clone().into_boxed_str()),
                nombre: Box::leak(f.nombre.clone().into_boxed_str()),
                clase: f.clase,
                proteccion: f.proteccion,
            })
            .collect();
        (c.version, c.clientes, clientes)
    })
}

/// El catálogo de Windows, leído del repo. Existe en todas las plataformas para que su test corra en el Mac.
pub fn catalogo_de_windows() -> &'static [Cliente] {
    &de_windows().2
}

/// Las filas del catálogo de Windows con su fuente y su fecha.
pub fn filas_de_windows() -> &'static [FilaDeWindows] {
    &de_windows().1
}

/// **El catálogo de esta plataforma**: el del Mac, igual que siempre, o el de Windows.
pub fn catalogo() -> &'static [Cliente] {
    if cfg!(windows) {
        catalogo_de_windows()
    } else {
        CATALOGO
    }
}

/// La versión del catálogo de esta plataforma.
pub fn version_del_catalogo() -> &'static str {
    if cfg!(windows) {
        &de_windows().0
    } else {
        VERSION_CATALOGO
    }
}

/// Lo que delata una pestaña de Meet en el título de una ventana. Se compara **sin distinguir
/// mayúsculas** y contra el título entero.
const SENALES_MEET: &[&str] = &["google meet", "meet.google.com"];

/// Una aplicación en ejecución, reducida a lo que el detector necesita ver.
#[derive(Clone, Debug)]
pub struct Programa {
    pub bundle: String,
    /// Títulos de sus ventanas. **Vacío** si no se pudieron leer (sin permiso), que es distinto
    /// de «no tiene ventanas» — ver [`Vista::titulos_legibles`].
    pub titulos: Vec<String>,
}

/// Lo que el sistema nos dejó ver. El booleano existe para no confundir «no hay reunión» con «no
/// puedo mirar»: la diferencia entre las dos es lo único que separa una app honesta de una que
/// dice que no cuando no sabe.
#[derive(Clone, Debug)]
pub struct Vista {
    pub programas: Vec<Programa>,
    pub titulos_legibles: bool,
}

/// El veredicto del detector.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "que", rename_all = "kebab-case")]
pub enum Reunion {
    /// Ninguna videollamada del catálogo está abierta.
    Ninguna,
    /// Hay una, y esto es lo que se sabe de ella.
    Detectada {
        /// Nombre del cliente de videollamada («Zoom», «Google Meet»).
        cliente: String,
        /// El título de la ventana, si se pudo leer. **Solo memoria, jamás disco.**
        titulo: Option<String>,
        proteccion: Proteccion,
    },
    /// Hay un navegador abierto pero no podemos mirar sus ventanas. **No es «no hay reunión».**
    NoSePuedeSaber { motivo: PorQueNoSeVe },
}

/// **Por qué no se puede saber si hay reunión**, en un conjunto cerrado (mirada 17-quater, kit
/// §8-ter). Hoy tiene un solo miembro, y es un enum igual: el día que aparezca un segundo, la
/// pantalla no compila hasta que alguien le escriba su frase en los dos idiomas.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PorQueNoSeVe {
    /// Sin el permiso de Accesibilidad no se leen los títulos de las ventanas del navegador.
    SinAccesibilidad,
}

fn del_catalogo(bundle: &str) -> Option<&'static Cliente> {
    del_catalogo_en(catalogo(), bundle)
}

fn del_catalogo_en<'a>(catalogo: &'a [Cliente], bundle: &str) -> Option<&'a Cliente> {
    catalogo.iter().find(|c| c.bundle.eq_ignore_ascii_case(bundle))
}

/// ¿Este título delata una reunión de Meet?
pub fn es_meet(titulo: &str) -> bool {
    let t = titulo.to_lowercase();
    SENALES_MEET.iter().any(|s| t.contains(s))
}

/// El veredicto, **puro**: entra lo que el sistema dejó ver, sale la frase de la pantalla.
///
/// El orden importa y es el del producto, no el del código: una aplicación nativa de videollamada
/// gana a una pestaña, porque si tienes Zoom abierto **y** Chrome, la reunión es la de Zoom.
pub fn clasificar(vista: &Vista) -> Reunion {
    clasificar_en(vista, catalogo())
}

/// El veredicto contra un catálogo dado: el de macOS o el de Windows. Puro.
pub fn clasificar_en(vista: &Vista, catalogo: &[Cliente]) -> Reunion {
    let mut navegadores = Vec::new();

    for p in &vista.programas {
        let Some(cliente) = del_catalogo_en(catalogo, &p.bundle) else { continue };
        match cliente.clase {
            Clase::Aplicacion => {
                return Reunion::Detectada {
                    cliente: cliente.nombre.to_string(),
                    titulo: None,
                    proteccion: cliente.proteccion,
                }
            }
            Clase::Navegador => navegadores.push((cliente, p)),
        }
    }

    for (cliente, p) in &navegadores {
        if let Some(t) = p.titulos.iter().find(|t| es_meet(t)) {
            return Reunion::Detectada {
                cliente: "Google Meet".to_string(),
                titulo: Some(t.clone()),
                proteccion: cliente.proteccion,
            };
        }
    }

    // Hay navegador, pero no podemos ver sus ventanas: no sabemos, y se dice.
    if !navegadores.is_empty() && !vista.titulos_legibles {
        return Reunion::NoSePuedeSaber { motivo: PorQueNoSeVe::SinAccesibilidad };
    }

    Reunion::Ninguna
}

/// **A quién se le preguntan los títulos de sus ventanas**, y a quién no.
///
/// Es la restricción de privacidad de este módulo hecha función, y por eso es pura y tiene su
/// test: sin ella, «leer títulos» significaría leer los de **todo el Mac** —el nombre de cada
/// documento abierto, cada conversación, cada expediente—, que es precisamente el poder que la
/// Accessibility API regala y que esta app no quiere.
///
/// Solo entran los **navegadores del catálogo**, y solo si se pueden leer. Zoom y Teams no: se
/// detectan por su identificador y su título no aporta nada.
pub fn a_quien_preguntar_titulos(apps: &[(i32, String)], legibles: bool) -> Vec<i32> {
    if !legibles {
        return Vec::new();
    }
    apps.iter()
        .filter(|(_, bundle)| {
            del_catalogo(bundle).is_some_and(|c| c.clase == Clase::Navegador)
        })
        .map(|(pid, _)| *pid)
        .collect()
}

/// Lo que el sistema deja ver ahora mismo.
#[cfg(target_os = "macos")]
pub fn mirar() -> Vista {
    use crate::acople::ax;

    // Leer títulos exige Accesibilidad — el mismo permiso que el acople. Si no lo hay, la
    // respuesta no es «no hay reunión», es «no puedo mirar», y eso viaja en la vista.
    let legibles = crate::acople::hay_permiso();
    let apps = ax::aplicaciones_en_ejecucion();
    let preguntables = a_quien_preguntar_titulos(&apps, legibles);

    let programas = apps
        .into_iter()
        .filter(|(_, bundle)| del_catalogo(bundle).is_some())
        .map(|(pid, bundle)| Programa {
            titulos: if preguntables.contains(&pid) {
                ax::titulos_de(pid)
            } else {
                Vec::new()
            },
            bundle,
        })
        .collect();

    Vista { programas, titulos_legibles: legibles }
}

/// En Windows, los procesos del PC y los títulos de las ventanas de los navegadores del catálogo. Windows no pide
/// permiso para leerlos, así que los títulos siempre se pueden leer.
#[cfg(windows)]
pub fn mirar() -> Vista {
    windows::mirar()
}

#[cfg(not(any(target_os = "macos", windows)))]
pub fn mirar() -> Vista {
    Vista { programas: Vec::new(), titulos_legibles: false }
}

/// El veredicto de ahora mismo.
pub fn ahora() -> Reunion {
    clasificar(&mirar())
}

/// **A qué ventana mira la lectura de pantalla** (C8): la app de la videollamada detectada y, si
/// es un navegador, las señales que su título tiene que llevar. Devuelve `(bundle, señales)`.
///
/// Sigue EXACTAMENTE el mismo orden que [`clasificar`] —primero las aplicaciones, luego Meet en un
/// navegador— para que la ventana que se lee sea la de la reunión que Sesión enseña, y no otra.
/// Con dos videollamadas abiertas a la vez, las dos pantallas eligen la misma.
pub fn objetivo_de(vista: &Vista) -> Option<(String, Vec<String>)> {
    for p in &vista.programas {
        if let Some(c) = del_catalogo(&p.bundle) {
            if c.clase == Clase::Aplicacion {
                return Some((p.bundle.clone(), Vec::new()));
            }
        }
    }
    vista.programas.iter().find_map(|p| {
        let c = del_catalogo(&p.bundle)?;
        (c.clase == Clase::Navegador && p.titulos.iter().any(|t| es_meet(t))).then(|| {
            (
                p.bundle.clone(),
                SENALES_MEET.iter().map(|s| s.to_string()).collect(),
            )
        })
    })
}

/// **El proceso de la videollamada que eligió [`objetivo_de`]** (sprint 004): el primero que corre con
/// ese identificador. Puro, para que el test vea que se elige por identificador y no por nombre.
pub fn pid_del_objetivo(apps: &[(i32, String)], bundle: &str) -> Option<i32> {
    apps.iter().find(|(_, b)| b.eq_ignore_ascii_case(bundle)).map(|(pid, _)| *pid)
}

/// **Cuál de las ventanas de un navegador es la de la reunión** (sprint 004): la primera cuyo título lleva
/// las señales que dio [`objetivo_de`], por su posición en la lista. Es lo único que el acople arriba recibe
/// de un título: un número. Las señales llegan de la detección y no se repiten aquí (auditoría del S4, B11):
/// cuando el catálogo crezca con otra reunión de navegador, esto la encuentra sin tocarlo.
pub fn indice_por_senales(titulos: &[(usize, String)], senales: &[String]) -> Option<usize> {
    titulos
        .iter()
        .find(|(_, t)| {
            let t = t.to_lowercase();
            senales.iter().any(|s| t.contains(s.as_str()))
        })
        .map(|(i, _)| *i)
}

/// **La ventana de la reunión, para el acople arriba** (ADR 004, enmienda 1): la de la videollamada que
/// Sesión enseña —el mismo orden que [`clasificar`]—. En Zoom o Teams, su ventana principal; en un
/// navegador, la de Meet, que se encuentra aquí, donde ya se leen títulos, y viaja como posición en la
/// lista. Sin reunión detectada, `None`: la banda flota arriba y no toca nada.
#[cfg(target_os = "macos")]
pub fn ventana_de_la_reunion() -> Option<crate::acople::Destino> {
    use crate::acople::ax;
    let (bundle, senales) = objetivo_de(&mirar())?;
    let pid = pid_del_objetivo(&ax::aplicaciones_en_ejecucion(), &bundle)?;
    let indice = if senales.is_empty() {
        ax::indice_de_la_principal(pid)?
    } else {
        indice_por_senales(&ax::titulos_con_indice(pid), &senales)?
    };
    Some(crate::acople::Destino { pid, app: ax::nombre_de(pid)?, indice })
}

#[cfg(not(target_os = "macos"))]
pub fn ventana_de_la_reunion() -> Option<crate::acople::Destino> {
    None
}

/// Una ventana de la videollamada, reducida a lo que decide cuál se lee.
#[derive(Clone, Debug)]
pub struct Candidata {
    pub ancho: i32,
    pub alto: i32,
    pub titulo: String,
}

/// **Cuál de las ventanas de la videollamada se lee** (sprint 007): la mayor de al menos 320×200 y, en un
/// navegador, la que lleva las señales de la reunión en el título. Es el mismo criterio que la captura del Mac
/// (`Pantalla.swift`), escrito en Rust para Windows. Pura.
pub fn la_de_la_reunion(candidatas: &[Candidata], senales: &[String]) -> Option<usize> {
    candidatas
        .iter()
        .enumerate()
        .filter(|(_, c)| c.ancho >= 320 && c.alto >= 200)
        .filter(|(_, c)| {
            let t = c.titulo.to_lowercase();
            senales.is_empty() || senales.iter().any(|s| t.contains(s.as_str()))
        })
        .max_by_key(|(_, c)| i64::from(c.ancho) * i64::from(c.alto))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **El catálogo de Windows dice de dónde sale cada fila** y no afirma nada que nadie haya mirado: cada fila trae
    /// una página pública con fecha, un ejecutable `.exe` sin repetir, y la banda «sin verificar».
    #[test]
    fn el_catalogo_de_windows_trae_su_fuente_y_nada_verificado() {
        let filas = filas_de_windows();
        assert!(filas.len() >= 6, "faltan clientes: {}", filas.len());
        let mut exes: Vec<String> = Vec::new();
        for f in filas {
            assert!(f.fuente.starts_with("https://"), "{} sin fuente pública", f.ejecutable);
            assert!(
                f.fecha.len() == 10 && f.fecha.starts_with("2026-"),
                "{}: la fecha de lectura no se puede contrastar",
                f.ejecutable
            );
            assert!(f.ejecutable.to_lowercase().ends_with(".exe"), "{} no es un ejecutable", f.ejecutable);
            assert_eq!(f.proteccion, Proteccion::SinVerificar, "{}: nadie lo ha mirado en Windows", f.nombre);
            exes.push(f.ejecutable.to_lowercase());
        }
        let antes = exes.len();
        exes.sort();
        exes.dedup();
        assert_eq!(antes, exes.len(), "hay un ejecutable repetido");
        assert!(version_del_catalogo().starts_with('v'));
    }

    /// **En Windows, Teams, Zoom y Meet se reconocen igual que en el Mac**, por el nombre del ejecutable y sin
    /// distinguir mayúsculas, y Meet por el título de la pestaña.
    #[test]
    fn en_windows_teams_zoom_y_meet_se_reconocen() {
        let w = catalogo_de_windows();
        let teams = clasificar_en(&vista(vec![prog("MS-TEAMS.EXE", &[])], true), w);
        assert!(matches!(&teams, Reunion::Detectada { cliente, .. } if cliente == "Microsoft Teams"), "{teams:?}");
        let zoom = clasificar_en(&vista(vec![prog("Zoom.exe", &[])], true), w);
        assert!(matches!(&zoom, Reunion::Detectada { cliente, .. } if cliente == "Zoom"), "{zoom:?}");
        let meet = clasificar_en(&vista(vec![prog("msedge.exe", &["Páramo Azul - Google Meet - Edge"])], true), w);
        let Reunion::Detectada { cliente, proteccion, .. } = meet else { panic!("tenía que detectar Meet") };
        assert_eq!(cliente, "Google Meet");
        assert_eq!(proteccion, Proteccion::SinVerificar, "Meet verificado en el Mac no vale para Windows");
        let correo = clasificar_en(&vista(vec![prog("chrome.exe", &["Correo"])], true), w);
        assert_eq!(correo, Reunion::Ninguna);
        // Un identificador de macOS no se reconoce en Windows, ni al revés: cada catálogo es de su plataforma.
        assert_eq!(clasificar_en(&vista(vec![prog("us.zoom.xos", &[])], true), w), Reunion::Ninguna);
        assert_eq!(clasificar_en(&vista(vec![prog("Zoom.exe", &[])], true), CATALOGO), Reunion::Ninguna);
    }

    /// **macOS no cambia**: `clasificar` con el catálogo de la plataforma es el de siempre.
    #[cfg(not(windows))]
    #[test]
    fn en_macos_el_catalogo_es_el_de_siempre() {
        assert_eq!(catalogo().len(), CATALOGO.len());
        assert_eq!(version_del_catalogo(), VERSION_CATALOGO);
        let v = vista(vec![prog("com.google.Chrome", &["Páramo Azul — Google Meet"])], true);
        assert_eq!(clasificar(&v), clasificar_en(&v, CATALOGO));
    }

    /// La ventana que se lee: la mayor que cabe, y en un navegador la de la reunión aunque no sea la mayor.
    #[test]
    fn se_lee_la_ventana_mayor_de_la_reunion() {
        let c = |ancho, alto, titulo: &str| Candidata { ancho, alto, titulo: titulo.into() };
        let zoom = [c(300, 900, "Zoom"), c(1280, 720, "Zoom Meeting"), c(1600, 150, "barra")];
        assert_eq!(la_de_la_reunion(&zoom, &[]), Some(1), "ni la estrecha ni la barra");
        let senales: Vec<String> = SENALES_MEET.iter().map(|s| s.to_string()).collect();
        let chrome = [c(1920, 1080, "Correo - Gmail"), c(1280, 800, "abc-defg-hij - Google Meet")];
        assert_eq!(la_de_la_reunion(&chrome, &senales), Some(1));
        assert_eq!(la_de_la_reunion(&chrome[..1], &senales), None, "sin Meet no se lee el correo");
        assert_eq!(la_de_la_reunion(&[], &[]), None);
    }

    /// El acople arriba se dirige a la reunión por identificador, y en un navegador a la ventana de Meet
    /// **por su posición real en la lista**, aunque haya ventanas sin título o con otro antes.
    #[test]
    fn arriba_el_acople_va_a_la_ventana_de_la_reunion_y_no_a_otra() {
        let apps = vec![(10, "com.apple.Safari".to_string()), (20, "com.google.Chrome".to_string())];
        assert_eq!(pid_del_objetivo(&apps, "com.google.Chrome"), Some(20));
        assert_eq!(pid_del_objetivo(&apps, "us.zoom.xos"), None, "sin Zoom en marcha no hay a quién acoplar");
        let titulos = vec![
            (0, "Correo — Bandeja de entrada".to_string()),
            (2, "Meet – abc-defg-hij · Google Meet".to_string()),
            (3, "Otra pestaña de meet.google.com".to_string()),
        ];
        let senales: Vec<String> = SENALES_MEET.iter().map(|s| s.to_string()).collect();
        assert_eq!(indice_por_senales(&titulos, &senales), Some(2), "la posición real, no la del título en la lista");
        assert_eq!(indice_por_senales(&titulos[..1], &senales), None);
        // Las señales son las que da la detección (auditoría del S4, B11): con las de otra reunión de
        // navegador de un catálogo de prueba, se encuentra la suya, no la de Meet.
        let otras = vec!["whereby.com".to_string()];
        let con_otra = vec![(1, "Meet – abc · Google Meet".to_string()), (4, "Sala de ventas · whereby.com".to_string())];
        assert_eq!(indice_por_senales(&con_otra, &otras), Some(4));
    }

    #[test]
    fn la_pantalla_mira_la_misma_reunion_que_sesion() {
        let meet = Vista {
            programas: vec![
                prog("com.apple.Safari", &["Correo"]),
                prog("com.google.Chrome", &["Meet – abc-defg-hij · Google Meet"]),
            ],
            titulos_legibles: true,
        };
        assert_eq!(
            objetivo_de(&meet),
            Some((
                "com.google.Chrome".into(),
                vec!["google meet".into(), "meet.google.com".into()]
            ))
        );
        let zoom_y_meet = Vista {
            programas: vec![
                prog("com.google.Chrome", &["Google Meet"]),
                prog("us.zoom.xos", &[]),
            ],
            titulos_legibles: true,
        };
        // Zoom gana, igual que en `clasificar`: las dos pantallas eligen la misma reunión.
        assert_eq!(
            objetivo_de(&zoom_y_meet),
            Some(("us.zoom.xos".into(), vec![]))
        );
        assert!(
            matches!(clasificar(&zoom_y_meet), Reunion::Detectada { cliente, .. } if cliente == "Zoom")
        );
        let nada = Vista {
            programas: vec![prog("com.google.Chrome", &["Correo"])],
            titulos_legibles: true,
        };
        assert_eq!(objetivo_de(&nada), None);
    }

    fn prog(bundle: &str, titulos: &[&str]) -> Programa {
        Programa {
            bundle: bundle.into(),
            titulos: titulos.iter().map(|s| s.to_string()).collect(),
        }
    }

    fn vista(programas: Vec<Programa>, legibles: bool) -> Vista {
        Vista { programas, titulos_legibles: legibles }
    }

    #[test]
    fn zoom_abierto_basta() {
        let r = clasificar(&vista(vec![prog("us.zoom.xos", &[])], true));
        assert_eq!(
            r,
            Reunion::Detectada {
                cliente: "Zoom".into(),
                titulo: None,
                proteccion: Proteccion::SinVerificar
            }
        );
    }

    #[test]
    fn meet_se_reconoce_por_el_titulo_de_la_pestana() {
        let r = clasificar(&vista(
            vec![prog("com.google.Chrome", &["Correo", "Páramo Azul — Google Meet"])],
            true,
        ));
        let Reunion::Detectada { cliente, titulo, proteccion } = r else {
            panic!("tenía que detectar Meet")
        };
        assert_eq!(cliente, "Google Meet");
        assert_eq!(titulo.as_deref(), Some("Páramo Azul — Google Meet"));
        assert_eq!(proteccion, Proteccion::Verificada);
    }

    /// El caso que hace inútil a un detector ingenuo: Chrome SIEMPRE está abierto.
    #[test]
    fn un_navegador_sin_pestana_de_reunion_no_es_una_reunion() {
        let r = clasificar(&vista(
            vec![prog("com.google.Chrome", &["Correo", "Documentación de Tauri"])],
            true,
        ));
        assert_eq!(r, Reunion::Ninguna);
    }

    /// La distinción que separa una app honesta de una que dice «no» cuando no sabe.
    #[test]
    fn sin_poder_leer_titulos_no_se_dice_que_no_hay_reunion() {
        let r = clasificar(&vista(vec![prog("com.google.Chrome", &[])], false));
        let Reunion::NoSePuedeSaber { motivo } = r else {
            panic!("sin permiso no se puede afirmar que no hay reunión")
        };
        assert_eq!(motivo, PorQueNoSeVe::SinAccesibilidad);
    }

    /// Sin navegadores abiertos, no poder leer títulos no cambia nada: no hay reunión y punto.
    #[test]
    fn sin_navegador_la_falta_de_permiso_no_ensucia_la_respuesta() {
        assert_eq!(clasificar(&vista(vec![prog("com.apple.Finder", &[])], false)), Reunion::Ninguna);
        assert_eq!(clasificar(&vista(vec![], false)), Reunion::Ninguna);
    }

    /// Si tienes Zoom Y Chrome con Meet, la reunión es la de Zoom: una aplicación de videollamada
    /// abierta pesa más que una pestaña.
    #[test]
    fn una_aplicacion_nativa_gana_a_una_pestana() {
        let r = clasificar(&vista(
            vec![
                prog("com.google.Chrome", &["Páramo Azul — Google Meet"]),
                prog("us.zoom.xos", &[]),
            ],
            true,
        ));
        let Reunion::Detectada { cliente, .. } = r else { panic!() };
        assert_eq!(cliente, "Zoom");
    }

    #[test]
    fn las_senales_de_meet_no_distinguen_mayusculas_ni_se_disparan_de_mas() {
        assert!(es_meet("Páramo Azul — GOOGLE MEET"));
        assert!(es_meet("meet.google.com/abc-defg-hij"));
        assert!(!es_meet("Reunión de equipo · Calendario"));
        assert!(!es_meet("Meet the team — blog"), "«meet» suelto no es Google Meet");
    }

    /// La protección es **graduada por cliente**: solo Meet está verificado, y afirmarlo de los
    /// demás sería la clase de promesa que esta app no hace.
    #[test]
    fn solo_el_cliente_verificado_se_declara_verificado() {
        let verificados: Vec<&str> = CATALOGO
            .iter()
            .filter(|c| c.proteccion == Proteccion::Verificada)
            .map(|c| c.nombre)
            .collect();
        assert_eq!(
            verificados,
            vec!["Google Chrome"],
            "alguien declaró verificado un cliente que nadie ha mirado"
        );
    }

    /// La restricción de privacidad del módulo: **solo se miran los títulos de los navegadores
    /// del catálogo**. Sin este filtro, «leer títulos» sería leer los de todo el Mac — cada
    /// documento abierto, cada conversación, cada expediente.
    #[test]
    fn solo_se_preguntan_titulos_a_los_navegadores_del_catalogo() {
        let apps = vec![
            (10, "com.google.Chrome".to_string()),
            (11, "us.zoom.xos".to_string()),
            (12, "com.apple.mail".to_string()),
            (13, "com.apple.Safari".to_string()),
            (14, "com.tuempresa.Expedientes".to_string()),
        ];
        assert_eq!(a_quien_preguntar_titulos(&apps, true), vec![10, 13]);
    }

    #[test]
    fn sin_permiso_no_se_le_preguntan_titulos_a_nadie() {
        let apps = vec![(10, "com.google.Chrome".to_string())];
        assert!(a_quien_preguntar_titulos(&apps, false).is_empty());
    }

    #[test]
    fn el_catalogo_no_tiene_identificadores_repetidos() {
        let mut vistos: Vec<&str> = CATALOGO.iter().map(|c| c.bundle).collect();
        let antes = vistos.len();
        vistos.sort_unstable();
        vistos.dedup();
        assert_eq!(antes, vistos.len(), "hay un identificador repetido en el catálogo");
    }

    #[test]
    fn el_catalogo_viaja_con_su_version_visible() {
        assert!(VERSION_CATALOGO.starts_with('v'), "la versión del catálogo no se puede contrastar");
        assert!(VERSION_CATALOGO.contains("2026"), "la versión no dice de cuándo es");
    }
}
