//! El ACOPLE — la banda no tapa la reunión, la reunión se hace sitio.
//!
//! Sin acople la banda se queda **encima** de la ventana de la videollamada: el consultor pierde
//! los 88 px de abajo de su reunión. Con acople, la ventana de la reunión se **encoge** hasta que
//! su borde inferior queda justo sobre la franja, y al terminar vuelve a su tamaño.
//!
//! Tocar la ventana de OTRA aplicación es la operación más invasiva de toda la app, así que este
//! módulo se escribe con tres reglas explícitas:
//!
//! 1. **Se encoge, nunca se mueve.** No cambiamos la posición de nada: solo el alto. Una ventana
//!    que se mueve sola es un susto; una que se acorta por abajo es una ventana que cabe.
//! 2. **Se devuelve SIEMPRE**, y la devolución es idempotente: al cerrar, al apagar, y **al
//!    arrancar** si la vez anterior terminó en una caída. La huella vive en disco justo para eso.
//! 3. **Solo se devuelve lo que sigue como lo dejamos.** Si el usuario redimensionó esa ventana
//!    a mano mientras tanto, la huella ya no encaja y no se toca: su último gesto manda sobre
//!    nuestro registro.
//!
//! Lo que este módulo **no** hace, y es deliberado: no lee títulos de ventanas, ni contenido, ni
//! enumera el sistema. Pregunta por la aplicación que está al frente, mide sus ventanas y les
//! cambia el alto. La Accessibility API permite mucho más; el ADR de esta fase lo declara.
//!
//! La geometría de aquí abajo es **pura**: entra un rectángulo, sale una decisión. Es la parte
//! que se puede probar sin permisos, sin ventanas ajenas y sin macOS — y es donde viven los dos
//! errores que de verdad duelen (encoger una ventana hasta dejarla inservible, y devolver un
//! tamaño a la ventana equivocada).

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
pub mod ax;

/// Un rectángulo en coordenadas globales de pantalla, origen arriba-izquierda y **puntos**
/// (no píxeles físicos) — que es el sistema que usan tanto la Accessibility API como la geometría
/// lógica de Tauri. Los dos lados hablan el mismo idioma a propósito: una conversión de más entre
/// medias es una diferencia de 2× en una pantalla Retina, y eso es media banda.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Marco {
    pub x: f64,
    pub y: f64,
    pub ancho: f64,
    pub alto: f64,
}

impl Marco {
    pub fn nuevo(x: f64, y: f64, ancho: f64, alto: f64) -> Self {
        Self { x, y, ancho, alto }
    }
    fn fondo(&self) -> f64 {
        self.y + self.alto
    }
    fn derecha(&self) -> f64 {
        self.x + self.ancho
    }
}

/// Holgura en puntos. Por debajo de esto dos rectángulos son el mismo: los gestores de ventanas
/// redondean, y exigir igualdad exacta convertiría «devolver la ventana» en una lotería.
const HOLGURA: f64 = 2.0;

/// Por debajo de este alto, una ventana deja de servir para una videollamada. Si acoplarla la
/// dejaría más baja que esto, **no se toca**: la banda flota encima y se dice. Preferimos una
/// banda que estorba a una reunión que no se puede usar.
pub const ALTO_MINIMO_UTIL: f64 = 240.0;

/// Qué hacer con una ventana, y **por qué** — el motivo viaja en el valor porque es lo que se
/// escribe en el log cuando el acople no hace nada. «No pasó nada» sin motivo es indistinguible
/// de un fallo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Decision {
    /// Encogerla a este alto (la posición no cambia).
    Encoger { alto: f64 },
    /// No se cruza con la franja: la banda no la tapa.
    NoSeCruzan,
    /// Su borde inferior ya está por encima de la franja.
    YaCabe,
    /// Encogerla la dejaría inservible.
    QuedariaInservible { alto_resultante: f64 },
}

/// La decisión sobre UNA ventana frente a la franja de la banda.
pub fn decidir(ventana: Marco, franja: Marco) -> Decision {
    // Sin cruce horizontal la banda no la tapa aunque llegue más abajo: monitor de al lado,
    // ventana estrecha pegada a un borde.
    if ventana.derecha() <= franja.x || ventana.x >= franja.derecha() {
        return Decision::NoSeCruzan;
    }
    if ventana.fondo() <= franja.y + HOLGURA {
        return Decision::YaCabe;
    }
    let alto = franja.y - ventana.y;
    if alto < ALTO_MINIMO_UTIL {
        return Decision::QuedariaInservible {
            alto_resultante: alto,
        };
    }
    Decision::Encoger { alto }
}

/// Lo que hay que saber para deshacer un acople: quién era, cómo estaba y cómo la dejamos.
///
/// **No guarda el título de la ventana ni nada de su contenido**, a propósito: el nombre de una
/// reunión es información del cliente, y el estándar 4-T no distingue por formato sino por de
/// quién es. Con el nombre de la aplicación y dos rectángulos basta para devolverla.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Huella {
    /// PID del proceso dueño de la ventana.
    pub pid: i32,
    /// Nombre de la aplicación («Google Chrome»). Solo sirve de cerrojo contra el reciclado de
    /// PID: un PID libre se reasigna, y devolverle un tamaño a la ventana equivocada sería el
    /// peor fallo posible de este módulo.
    pub app: String,
    /// Cómo estaba antes de que la tocáramos.
    pub original: Marco,
    /// Cómo la dejamos, **leído del sistema después de escribir**, no calculado. Muchas
    /// aplicaciones acotan el tamaño que se les pide; si guardáramos lo pedido en vez de lo
    /// conseguido, la comprobación de la devolución no encajaría nunca y la ventana se quedaría
    /// encogida para siempre.
    pub dejada: Marco,
}

/// Lo que se escribe en disco entre una sesión y la siguiente.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Pendiente {
    pub version: u32,
    pub huellas: Vec<Huella>,
}

pub const VERSION_HUELLA: u32 = 1;

fn casi_iguales(a: Marco, b: Marco) -> bool {
    (a.x - b.x).abs() <= HOLGURA
        && (a.y - b.y).abs() <= HOLGURA
        && (a.ancho - b.ancho).abs() <= HOLGURA
        && (a.alto - b.alto).abs() <= HOLGURA
}

/// ¿La ventana **de verdad** cambió de alto, o el sistema dijo que sí y la dejó igual?
///
/// Medido en vivo: una ventana de «Code» quedó anotada con `original 923 → dejada 923`. La
/// Accessibility API aceptó la escritura sin error y la aplicación mantuvo su tamaño —pasa con
/// ventanas en pantalla completa, en Split View o con tamaño fijo—. Sin esta comprobación el
/// acople se apunta una ventana que no encogió, la huella guarda una devolución que no hay que
/// hacer, y **la banda dice «acoplada» mientras sigue tapando la reunión**: justo la etiqueta
/// falsa que esta app existe para no poner. «Lo pedí» no es «pasó».
pub fn hubo_cambio(antes: Marco, despues: Marco) -> bool {
    (antes.alto - despues.alto).abs() > HOLGURA
}

/// El tamaño al que devolver una ventana, **solo si sigue exactamente como la dejamos**.
///
/// Este `None` es la regla 3 del módulo hecha código: si el usuario la redimensionó a mano
/// mientras tanto, su gesto gana. Vale igual para la devolución normal y para la de después de
/// una caída — son el mismo camino, y por eso solo hay una función.
pub fn devolucion(actual: Marco, huella: &Huella) -> Option<Marco> {
    casi_iguales(actual, huella.dejada).then_some(huella.original)
}

// ---------------------------------------------------------------------------------------------
// La huella en disco
// ---------------------------------------------------------------------------------------------

/// Dónde vive la huella: la carpeta de soporte de la app, nunca el repo ni un temporal del
/// sistema. Es un archivo de estado de la app sobre ventanas del propio usuario — ni audio, ni
/// transcript, ni pantalla — así que no lo alcanza la prohibición del efímero; y aun así nace
/// restringido, porque un derivado jamás nace menos privado que su fuente.
pub fn ruta_de_la_huella(soporte: &Path) -> PathBuf {
    soporte.join("acople.json")
}

/// Permisos con los que nacen la carpeta y el archivo. No es ceremonia: el nombre de la
/// aplicación acoplada y la geometría de sus ventanas dicen con qué trabajó el usuario y cuándo.
#[cfg(unix)]
pub const MODO_ARCHIVO: u32 = 0o600;
#[cfg(unix)]
pub const MODO_CARPETA: u32 = 0o700;

/// Guarda la huella, creando la carpeta si hace falta, y **repara los permisos si el archivo o la
/// carpeta ya existían mal** — que es el caso que de verdad ocurre: el primero se crea bien y el
/// que quedó flojo es el de la versión anterior.
pub fn guardar(ruta: &Path, pendiente: &Pendiente) -> std::io::Result<()> {
    if let Some(carpeta) = ruta.parent() {
        std::fs::create_dir_all(carpeta)?;
        restringir(carpeta, MODO_CARPETA)?;
    }
    std::fs::write(ruta, serde_json::to_vec_pretty(pendiente)?)?;
    restringir(ruta, MODO_ARCHIVO)
}

#[cfg(unix)]
fn restringir(ruta: &Path, modo: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(ruta, std::fs::Permissions::from_mode(modo))
}

#[cfg(not(unix))]
fn restringir(_ruta: &Path, _modo: u32) -> std::io::Result<()> {
    Ok(())
}

/// Lee la huella pendiente. Un archivo ausente, ilegible o de otra versión **no es un error**:
/// significa que no hay nada que devolver. Fallar aquí abortaría el arranque por un archivo de
/// conveniencia.
pub fn leer(ruta: &Path) -> Pendiente {
    let Ok(crudo) = std::fs::read(ruta) else {
        return Pendiente::default();
    };
    match serde_json::from_slice::<Pendiente>(&crudo) {
        Ok(p) if p.version == VERSION_HUELLA => p,
        _ => Pendiente::default(),
    }
}

pub fn olvidar(ruta: &Path) {
    let _ = std::fs::remove_file(ruta);
}

// ---------------------------------------------------------------------------------------------
// La maniobra completa — solo macOS, porque la Accessibility API lo es
// ---------------------------------------------------------------------------------------------

/// Qué pasó, en datos. Viaja al log y a la banda (que dibuja «acoplada» o «sin acople»).
///
/// Los **motivos** no son decoración: el caso normal de este módulo es no hacer nada —no hay
/// permiso, la ventana ya cabe, nadie al frente— y «no pasó nada» sin motivo es indistinguible
/// de un fallo. Es lo que se mira cuando el usuario dice «no se acopló».
#[derive(Clone, Debug, Default, Serialize)]
pub struct Informe {
    /// ¿Concedió el usuario Accesibilidad?
    pub permiso: bool,
    /// Nombre de la aplicación sobre la que se actuó.
    pub app: Option<String>,
    /// Cuántas ventanas se encogieron (o se devolvieron, en `soltar`).
    pub ventanas: usize,
    /// Por qué no se tocó lo que no se tocó.
    pub motivos: Vec<String>,
}

impl Informe {
    pub fn acoplada(&self) -> bool {
        self.ventanas > 0
    }
}

#[cfg(target_os = "macos")]
pub use nativo::*;

#[cfg(target_os = "macos")]
mod nativo {
    use super::*;

    pub fn hay_permiso() -> bool {
        ax::permiso_concedido()
    }

    /// ¿Hay alguna aplicación al frente que no seamos nosotros? Se pregunta sin acoplar y sin
    /// escribir nada: es el latido de un bucle de espera, y un bucle que deja una línea de log
    /// por vuelta ahoga lo que sí hay que leer.
    pub fn hay_alguien_al_frente() -> bool {
        ax::aplicacion_al_frente().is_some()
    }

    /// Pide el permiso de Accesibilidad. Devuelve si YA estaba concedido: el diálogo de macOS
    /// lleva a Ajustes del Sistema y la respuesta real llega cuando el usuario vuelve.
    pub fn pedir_permiso() -> bool {
        ax::pedir_permiso()
    }

    /// Acopla la aplicación que está al frente a la franja de la banda.
    pub fn acoplar(franja: Marco, huella: &Path) -> Informe {
        if !ax::permiso_concedido() {
            return Informe {
                permiso: false,
                motivos: vec!["sin permiso de Accesibilidad: la banda flota".into()],
                ..Default::default()
            };
        }
        // Soltar primero, SIEMPRE. Sin esto, acoplar dos veces guardaría como «original» un
        // tamaño que ya era nuestro, y la ventana encogería un poco más en cada pasada hasta
        // desaparecer — un fallo que solo se nota después de un rato y ya no se puede deshacer.
        let previo = soltar(huella);
        let Some((pid, app)) = ax::aplicacion_al_frente() else {
            return Informe {
                permiso: true,
                motivos: vec!["no hay ninguna otra aplicación al frente".into()],
                ventanas: previo.ventanas,
                ..Default::default()
            };
        };
        acoplar_a(pid, &app, franja, huella)
    }

    /// Vuelve a acoplar **la misma aplicación** a una franja nueva. Es lo que corre cuando el asa
    /// cambia el alto de la banda: mientras se arrastra, la aplicación al frente somos nosotros,
    /// así que preguntar «¿quién está delante?» soltaría la reunión justo al agrandar la banda.
    pub fn reacoplar(franja: Marco, huella: &Path) -> Informe {
        if !ax::permiso_concedido() {
            return Informe {
                permiso: false,
                ..Default::default()
            };
        }
        let objetivo = leer(huella)
            .huellas
            .first()
            .map(|h| (h.pid, h.app.clone()));
        let previo = soltar(huella);
        match objetivo {
            Some((pid, app)) => acoplar_a(pid, &app, franja, huella),
            None => Informe {
                permiso: true,
                ventanas: previo.ventanas,
                motivos: vec!["no había nada acoplado que reajustar".into()],
                ..Default::default()
            },
        }
    }

    fn acoplar_a(pid: i32, app: &str, franja: Marco, huella: &Path) -> Informe {
        let mut informe = Informe {
            permiso: true,
            app: Some(app.to_string()),
            ..Default::default()
        };
        let mut huellas = Vec::new();

        for (indice, marco) in ax::ventanas_de(pid) {
            match decidir(marco, franja) {
                Decision::Encoger { alto } => match ax::encoger(pid, indice, alto) {
                    Some(dejada) if hubo_cambio(marco, dejada) => {
                        huellas.push(Huella {
                            pid,
                            app: app.to_string(),
                            original: marco,
                            dejada,
                        });
                        informe.ventanas += 1;
                    }
                    Some(_) => informe.motivos.push(format!(
                        "«{app}» aceptó el cambio de alto y se quedó igual (pantalla completa, Split View o tamaño fijo): la banda flota encima"
                    )),
                    None => informe
                        .motivos
                        .push(format!("«{app}» rechazó el cambio de alto de una ventana")),
                },
                Decision::YaCabe | Decision::NoSeCruzan => {}
                Decision::QuedariaInservible { alto_resultante } => informe.motivos.push(format!(
                    "una ventana de «{app}» quedaría en {alto_resultante:.0} px: se deja en paz y la banda flota encima"
                )),
            }
        }

        if huellas.is_empty() {
            if informe.motivos.is_empty() {
                informe
                    .motivos
                    .push(format!("ninguna ventana de «{app}» llega hasta la franja"));
            }
            olvidar(huella);
        } else if let Err(e) = guardar(
            huella,
            &Pendiente {
                version: VERSION_HUELLA,
                huellas,
            },
        ) {
            // No es fatal para ESTA sesión (el acople ya está hecho y se deshace al cerrar),
            // pero sí para la siguiente si esta termina mal: sin huella, nadie devuelve nada.
            informe
                .motivos
                .push(format!("no se pudo anotar la huella del acople: {e}"));
        }
        informe
    }

    /// Devuelve a su tamaño lo que este módulo encogió — el de esta sesión o el que dejó una
    /// caída anterior, que son el mismo camino: la huella no distingue, y por eso no hay dos
    /// funciones que puedan divergir.
    pub fn soltar(huella: &Path) -> Informe {
        let pendiente = leer(huella);
        let mut informe = Informe {
            permiso: ax::permiso_concedido(),
            app: pendiente.huellas.first().map(|h| h.app.clone()),
            ..Default::default()
        };
        if pendiente.huellas.is_empty() {
            return informe;
        }
        if !informe.permiso {
            informe.motivos.push(
                "hay ventanas por devolver pero ya no tenemos permiso de Accesibilidad".into(),
            );
            return informe;
        }

        for h in &pendiente.huellas {
            if !ax::sigue_siendo(h.pid, &h.app) {
                informe.motivos.push(format!(
                    "«{}» ya no está: su ventana no se toca",
                    h.app
                ));
                continue;
            }
            // Se busca por GEOMETRÍA, no por índice: entre el acople y la devolución el usuario
            // pudo abrir o cerrar pestañas y ventanas, y el índice 2 de antes ya no es el de
            // ahora. La huella reconoce lo que dejó.
            let devuelta = ax::ventanas_de(h.pid)
                .into_iter()
                .find_map(|(i, m)| devolucion(m, h).map(|orig| (i, orig)));
            match devuelta {
                Some((i, original)) => match ax::encoger(h.pid, i, original.alto) {
                    Some(quedo) if !hubo_cambio(original, quedo) => informe.ventanas += 1,
                    // Mismo rasero que al acoplar: se cuenta lo que PASÓ, no lo que se pidió. Si
                    // la ventana no volvió a su sitio hay que decirlo — es la mitad de la
                    // promesa, y la que el usuario nota.
                    Some(quedo) => informe.motivos.push(format!(
                        "«{}» no volvió a su alto: quedó en {:.0} px en vez de {:.0}",
                        h.app, quedo.alto, original.alto
                    )),
                    None => informe
                        .motivos
                        .push(format!("«{}» rechazó devolver el alto", h.app)),
                },
                // Medido en vivo: aquí caen DOS casos y el mensaje solo nombraba uno. Si el
                // usuario redimensionó esa ventana, manda él. Pero si la cerró, no hay nada que
                // devolver — y decir «cambió de tamaño» mandaba a buscar un fallo donde no lo
                // había. Lo que sabemos de verdad es que ninguna ventana de esa aplicación
                // coincide con la huella; eso es lo que se dice.
                None => informe.motivos.push(format!(
                    "ninguna ventana de «{}» coincide con la huella (la redimensionaron o la cerraron): no se toca",
                    h.app
                )),
            }
        }
        olvidar(huella);
        informe
    }

    /// La ruta del fondo de escritorio, para el relleno. **Se llama desde el hilo principal**
    /// (`NSScreen` lo exige y el tipo lo obliga).
    pub fn fondo_de_escritorio() -> Option<PathBuf> {
        ax::fondo_de_escritorio()
    }
}

/// Fuera de macOS no hay Accessibility API, y esta app es solo de macOS. Los stubs existen para
/// que la lógica pura —que es donde están los tests que importan— compile y se pruebe en
/// cualquier sitio, sin `cfg` salpicados por el resto del crate.
#[cfg(not(target_os = "macos"))]
mod nativo_ausente {
    use super::*;

    pub fn hay_permiso() -> bool {
        false
    }
    pub fn hay_alguien_al_frente() -> bool {
        false
    }
    pub fn pedir_permiso() -> bool {
        false
    }
    pub fn acoplar(_franja: Marco, _huella: &Path) -> Informe {
        Informe::default()
    }
    pub fn reacoplar(_franja: Marco, _huella: &Path) -> Informe {
        Informe::default()
    }
    pub fn soltar(_huella: &Path) -> Informe {
        Informe::default()
    }
    pub fn fondo_de_escritorio() -> Option<PathBuf> {
        None
    }
}

#[cfg(not(target_os = "macos"))]
pub use nativo_ausente::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// Una franja de banda como la real: ancho de pantalla, 88 px, pegada abajo en 1440×900.
    fn franja(alto: f64) -> Marco {
        Marco::nuevo(0.0, 900.0 - alto, 1440.0, alto)
    }

    #[test]
    fn una_ventana_que_llega_al_fondo_se_encoge_hasta_justo_encima_de_la_franja() {
        let f = franja(88.0);
        let v = Marco::nuevo(100.0, 50.0, 1000.0, 850.0); // fondo = 900, la pantalla entera
        let Decision::Encoger { alto } = decidir(v, f) else {
            panic!("tenía que encogerse: {:?}", decidir(v, f))
        };
        let resultado = Marco::nuevo(v.x, v.y, v.ancho, alto);
        assert!(
            resultado.fondo() <= f.y,
            "sigue metiéndose en la franja: fondo {} vs franja {}",
            resultado.fondo(),
            f.y
        );
    }

    /// La propiedad que importa, barrida: **después de encoger, ninguna ventana invade la
    /// franja**. Un caso suelto se ajusta a mano sin querer; una rejilla, no.
    #[test]
    fn ninguna_decision_de_encoger_deja_la_ventana_dentro_de_la_franja() {
        for alto_banda in [44.0, 88.0, 200.0] {
            let f = franja(alto_banda);
            for y in [0.0, 25.0, 100.0, 300.0, 600.0, 820.0] {
                for alto in [120.0, 300.0, 500.0, 900.0] {
                    let v = Marco::nuevo(200.0, y, 800.0, alto);
                    if let Decision::Encoger { alto: nuevo } = decidir(v, f) {
                        assert!(nuevo >= ALTO_MINIMO_UTIL, "se encogió por debajo del mínimo");
                        // SIN holgura: `decidir` calcula el alto exacto, así que el borde
                        // inferior tiene que caer EXACTAMENTE sobre la franja. Con `+ HOLGURA`
                        // —la tolerancia del propio código— este barrido se tragaba un error de
                        // 1 px sin pestañear: comprobado plantándolo.
                        assert!(
                            v.y + nuevo <= f.y,
                            "franja invadida por {:.0} px: y={y} alto={alto} banda={alto_banda}",
                            v.y + nuevo - f.y
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn una_ventana_que_ya_cabe_no_se_toca() {
        assert_eq!(
            decidir(Marco::nuevo(0.0, 0.0, 1440.0, 700.0), franja(88.0)),
            Decision::YaCabe
        );
    }

    #[test]
    fn una_ventana_en_el_monitor_de_al_lado_no_se_toca() {
        let v = Marco::nuevo(1500.0, 0.0, 900.0, 900.0);
        assert_eq!(decidir(v, franja(88.0)), Decision::NoSeCruzan);
    }

    /// El caso que protege al usuario de nosotros: una ventana pequeña abajo del todo. Encogerla
    /// la dejaría en 62 px — inservible. Se declara y se deja en paz.
    #[test]
    fn una_ventana_pequena_abajo_no_se_mutila() {
        let v = Marco::nuevo(0.0, 750.0, 600.0, 150.0);
        assert_eq!(
            decidir(v, franja(88.0)),
            Decision::QuedariaInservible {
                alto_resultante: 62.0
            }
        );
    }

    /// «Lo pedí» no es «pasó». Se encontró en vivo: «Code» aceptó la escritura y se quedó igual.
    #[test]
    fn una_ventana_que_no_cambio_de_alto_no_cuenta_como_acoplada() {
        let antes = Marco::nuevo(0.0, 0.0, 1440.0, 923.0);
        assert!(!hubo_cambio(antes, antes), "sin cambio no hay acople");
        assert!(
            !hubo_cambio(antes, Marco::nuevo(0.0, 0.0, 1440.0, 924.0)),
            "un punto de redondeo no es haber encogido"
        );
        assert!(hubo_cambio(antes, Marco::nuevo(0.0, 0.0, 1440.0, 835.0)));
    }

    #[test]
    fn se_devuelve_la_ventana_que_sigue_como_la_dejamos() {
        let h = Huella {
            pid: 42,
            app: "Google Chrome".into(),
            original: Marco::nuevo(0.0, 0.0, 1440.0, 900.0),
            dejada: Marco::nuevo(0.0, 0.0, 1440.0, 812.0),
        };
        assert_eq!(devolucion(h.dejada, &h), Some(h.original));
        // Un punto de redondeo del gestor de ventanas no puede costar la devolución.
        assert_eq!(
            devolucion(Marco::nuevo(0.0, 1.0, 1440.0, 811.0), &h),
            Some(h.original)
        );
    }

    #[test]
    fn no_se_devuelve_una_ventana_que_el_usuario_movio_mientras_tanto() {
        let h = Huella {
            pid: 42,
            app: "Google Chrome".into(),
            original: Marco::nuevo(0.0, 0.0, 1440.0, 900.0),
            dejada: Marco::nuevo(0.0, 0.0, 1440.0, 812.0),
        };
        assert_eq!(devolucion(Marco::nuevo(300.0, 40.0, 700.0, 500.0), &h), None);
    }

    /// **Gate de la regla 17-bis (a):** la huella nace 600 en una carpeta 700 — y si ya existían
    /// flojas, se reparan al guardar. Se demostró en rojo en el mismo commit (bitácora).
    #[cfg(unix)]
    #[test]
    fn la_huella_nace_privada_y_repara_lo_que_encuentre_flojo() {
        use std::os::unix::fs::PermissionsExt;

        let base = std::env::temp_dir().join(format!("ag-acople-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let ruta = ruta_de_la_huella(&base);

        // Alguien dejó la carpeta y el archivo abiertos en una versión anterior.
        std::fs::create_dir_all(&base).unwrap();
        std::fs::set_permissions(&base, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::write(&ruta, b"{}").unwrap();
        std::fs::set_permissions(&ruta, std::fs::Permissions::from_mode(0o644)).unwrap();

        guardar(&ruta, &Pendiente::default()).unwrap();

        // Los permisos se afirman con el NÚMERO LITERAL, no con `MODO_ARCHIVO`. Escrito con la
        // constante, el test se lee igual de bien y **no puede fallar**: cambiar la constante a
        // 0o644 cambia también lo que el test espera, y pasa en verde con la huella abierta —
        // comprobado, es lo que hacía este test en su primera versión. Un gate que se mide contra
        // sí mismo es decorado (regla 15, tercera pregunta: ¿puede fallar siquiera?).
        let modo = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(modo(&ruta), 0o600, "la huella quedó legible por otros");
        assert_eq!(modo(&base), 0o700, "la carpeta quedó abierta");
        assert_eq!((MODO_ARCHIVO, MODO_CARPETA), (0o600, 0o700));

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn una_huella_de_otra_version_se_ignora_en_vez_de_romper_el_arranque() {
        let base = std::env::temp_dir().join(format!("ag-acople-v-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let ruta = ruta_de_la_huella(&base);
        std::fs::create_dir_all(&base).unwrap();

        std::fs::write(&ruta, br#"{"version":99,"huellas":[{"pid":1}]}"#).unwrap();
        assert!(leer(&ruta).huellas.is_empty());

        std::fs::write(&ruta, b"esto no es json").unwrap();
        assert!(leer(&ruta).huellas.is_empty());

        assert!(leer(&base.join("no-existe.json")).huellas.is_empty());

        let _ = std::fs::remove_dir_all(&base);
    }

    #[test]
    fn la_huella_va_y_vuelve_entera() {
        let base = std::env::temp_dir().join(format!("ag-acople-rt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let ruta = ruta_de_la_huella(&base);
        let p = Pendiente {
            version: VERSION_HUELLA,
            huellas: vec![Huella {
                pid: 7,
                app: "zoom.us".into(),
                original: Marco::nuevo(1.0, 2.0, 3.0, 4.0),
                dejada: Marco::nuevo(1.0, 2.0, 3.0, 3.0),
            }],
        };
        guardar(&ruta, &p).unwrap();
        assert_eq!(leer(&ruta).huellas, p.huellas);
        olvidar(&ruta);
        assert!(leer(&ruta).huellas.is_empty());
        let _ = std::fs::remove_dir_all(&base);
    }

    /// El título de una ventana es información del cliente. Que la huella no tenga dónde
    /// guardarlo es una propiedad del tipo, y se afirma aquí para que añadir el campo «para
    /// depurar» tenga que pasar por encima de un test con su motivo escrito.
    #[test]
    fn la_huella_no_tiene_sitio_para_el_titulo_de_una_ventana() {
        let serializada = serde_json::to_string(&Huella {
            pid: 1,
            app: "x".into(),
            original: Marco::nuevo(0.0, 0.0, 1.0, 1.0),
            dejada: Marco::nuevo(0.0, 0.0, 1.0, 1.0),
        })
        .unwrap();
        for prohibido in ["titulo", "title", "url", "documento"] {
            assert!(
                !serializada.contains(prohibido),
                "la huella guarda «{prohibido}»: eso es contenido de la reunión"
            );
        }
    }
}
