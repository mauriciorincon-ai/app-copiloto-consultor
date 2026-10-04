//! El ACOPLE — la banda no tapa la reunión, la reunión se hace sitio.
//!
//! Sin acople la banda se queda **encima** de la ventana de la videollamada: el consultor pierde
//! los 88 px de su reunión que la banda tapa. Con acople, la ventana de la reunión se hace sitio, y al
//! terminar vuelve a su tamaño. **Abajo** (el H1) se encoge hasta que su borde inferior queda justo
//! sobre la franja. **Arriba** (de fábrica desde el sprint 004, ADR 004 enmienda 1) baja su borde
//! superior hasta la banda y se encoge lo mismo, así que su borde inferior —con los controles de la
//! llamada— no se mueve.
//!
//! Tocar la ventana de OTRA aplicación es la operación más invasiva de toda la app, así que este
//! módulo se escribe con tres reglas explícitas:
//!
//! 1. **Abajo se encoge y no se mueve; arriba se mueve SOLO la ventana de la reunión.** Una ventana
//!    que se mueve sola es un susto, así que arriba no se toca la que esté al frente: solo la de la
//!    videollamada que nombra la detección ([`Destino`]), y sin reunión detectada la banda flota.
//! 2. **Se devuelve SIEMPRE**, y la devolución es idempotente: al cerrar, al apagar, y **al
//!    arrancar** si la vez anterior terminó en una caída. La huella vive en disco justo para eso.
//! 3. **Solo se devuelve lo que sigue como lo dejamos.** Si el usuario redimensionó esa ventana
//!    a mano mientras tanto, la huella ya no encaja y no se toca: su último gesto manda sobre
//!    nuestro registro.
//!
//! Lo que este módulo **no** hace, y es deliberado: no lee títulos de ventanas, ni contenido, ni
//! enumera el sistema. Abajo pregunta por la aplicación que está al frente, mide sus ventanas y les
//! cambia el alto; arriba recibe de la detección el PID y la posición de UNA ventana en la lista. La
//! Accessibility API permite mucho más; el ADR 004 y su enmienda lo declaran.
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
    pub fn fondo(&self) -> f64 {
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
    /// Encogerla a este alto (la posición no cambia). Abajo.
    Encoger { alto: f64 },
    /// Bajar su borde superior hasta `y` y dejarla en `alto`, con el borde inferior donde estaba.
    /// Arriba (sprint 004).
    BajarYEncoger { y: f64, alto: f64 },
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

/// La decisión sobre la ventana de la reunión frente a la franja **de arriba** (ADR 004, enmienda 1).
///
/// El nuevo borde superior es el inferior de la banda; el nuevo alto, el que deja el borde inferior de
/// la ventana donde estaba (ahí viven los controles de la llamada). Por debajo de
/// [`ALTO_MINIMO_UTIL`] no se toca: no se mutila.
pub fn decidir_arriba(ventana: Marco, franja: Marco) -> Decision {
    // Sin cruce horizontal, o entera por encima de la franja (otro monitor), la banda no la tapa.
    if ventana.derecha() <= franja.x || ventana.x >= franja.derecha() || ventana.fondo() <= franja.y {
        return Decision::NoSeCruzan;
    }
    if ventana.y + HOLGURA >= franja.fondo() {
        return Decision::YaCabe;
    }
    let y = franja.fondo();
    let alto = ventana.fondo() - y;
    if alto < ALTO_MINIMO_UTIL {
        return Decision::QuedariaInservible { alto_resultante: alto };
    }
    Decision::BajarYEncoger { y, alto }
}

/// La ventana de la reunión, **tal como la nombra la detección** (`sesion::ventana_de_la_reunion`):
/// el proceso, su nombre (cerrojo contra el reciclado de PID, como en la huella) y su posición en la
/// lista de ventanas de ese proceso. El acople no sabe por qué es esa: no lee títulos.
#[derive(Clone, Debug, PartialEq)]
pub struct Destino {
    pub pid: i32,
    pub app: String,
    pub indice: usize,
}

/// **Arriba, de una lista de ventanas sale UNA sola decisión**: la de la ventana del destino. Las demás
/// —aunque invadan la franja, como el editor de alguien que estaba delante— no se miran. Es la regla 1
/// del módulo hecha función, y por eso es pura y tiene su test.
pub fn plan_arriba(ventanas: &[(usize, Marco)], indice: usize, franja: Marco) -> Option<(Marco, Decision)> {
    ventanas
        .iter()
        .find(|(i, _)| *i == indice)
        .map(|(_, m)| (*m, decidir_arriba(*m, franja)))
}

/// Una escritura sobre la ventana de otro, en el orden en que se hace. Que el orden sea un dato y no
/// una costumbre del código es lo que deja probarlo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paso {
    /// Escribir el alto (`AXSize`; el ancho no se toca).
    Alto(f64),
    /// Escribir la esquina superior izquierda (`AXPosition`; solo arriba, solo la ventana acoplada).
    Mover { x: f64, y: f64 },
}

/// **Al acoplar arriba: primero el tamaño, después la posición.** Acortarla por abajo la deja dentro de
/// la pantalla; bajarla después la lleva hasta la banda. En el orden contrario, durante un instante su
/// borde inferior saldría de la pantalla y macOS la recolocaría a su manera.
pub fn pasos_del_acople_arriba(ventana: Marco, y: f64, alto: f64) -> [Paso; 2] {
    [Paso::Alto(alto), Paso::Mover { x: ventana.x, y }]
}

/// **Al devolver: primero la posición, después el tamaño** —sube y luego crece—, y solo lo que cambió.
/// Una huella del H1, que solo tocó el alto, se devuelve igual que entonces: un solo paso. Y una
/// ventana que ya está como era no recibe ninguno (la devolución es idempotente).
pub fn pasos_de_la_devolucion(actual: Marco, original: Marco) -> Vec<Paso> {
    let mut pasos = Vec::new();
    if (actual.x - original.x).abs() > HOLGURA || (actual.y - original.y).abs() > HOLGURA {
        pasos.push(Paso::Mover { x: original.x, y: original.y });
    }
    if (actual.alto - original.alto).abs() > HOLGURA {
        pasos.push(Paso::Alto(original.alto));
    }
    pasos
}

/// ¿Quedó acoplada arriba? Lo que importa es lo que se ve: que su borde superior ya no esté debajo de
/// la banda. Se mide sobre lo **leído** del sistema después de escribir, no sobre lo pedido.
pub fn quedo_bajo_la_franja(dejada: Marco, franja: Marco) -> bool {
    dejada.y + HOLGURA >= franja.fondo()
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

/// ¿La ventana **de verdad** cambió, o el sistema dijo que sí y la dejó igual?
///
/// Medido en vivo: una ventana de «Code» quedó anotada con `original 923 → dejada 923`. La
/// Accessibility API aceptó la escritura sin error y la aplicación mantuvo su tamaño —pasa con
/// ventanas en pantalla completa, en Split View o con tamaño fijo—. Sin esta comprobación el
/// acople se apunta una ventana que no encogió, la huella guarda una devolución que no hay que
/// hacer, y **la banda dice «acoplada» mientras sigue tapando la reunión**: justo la etiqueta
/// falsa que esta app existe para no poner. «Lo pedí» no es «pasó».
///
/// **Desde el sprint 004 mira la posición y el tamaño**: arriba la ventana también baja, y una
/// devolución que le devuelve el alto pero la deja abajo no ha devuelto nada.
pub fn hubo_cambio(antes: Marco, despues: Marco) -> bool {
    !casi_iguales(antes, despues)
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

/// Guarda la huella con **el escritor único** (`almacen::escribir`, auditoría del S3, B1): carpeta 700,
/// y el archivo nace 600 en un temporal que se renombra encima. Si la carpeta o un archivo viejo
/// estaban flojos, lo que queda es nuevo y cerrado. `restringir` se queda para reparar la carpeta.
pub fn guardar(ruta: &Path, pendiente: &Pendiente) -> std::io::Result<()> {
    crate::almacen::escribir(ruta, &serde_json::to_vec_pretty(pendiente)?).map_err(std::io::Error::other)?;
    if let Some(carpeta) = ruta.parent() {
        restringir(carpeta, MODO_CARPETA)?;
    }
    Ok(())
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
    /// Cuánto tardó, en milisegundos: el presupuesto del acople arriba es ≤ 300 ms (ADR 004, enmienda
    /// 1), y solo se puede afirmar midiéndolo en vivo. Lo escriben `acoplar_arriba` y `soltar`.
    pub ms: u64,
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
                // Abajo no se baja nada: `decidir` no produce `BajarYEncoger`, que es de `decidir_arriba`.
                Decision::YaCabe | Decision::NoSeCruzan | Decision::BajarYEncoger { .. } => {}
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

    /// Acopla **la ventana de la reunión** a la franja de arriba (ADR 004, enmienda 1): la baja hasta
    /// la banda y la encoge lo mismo, con su borde inferior donde estaba. Solo esa ventana, la que nombra
    /// la detección; ninguna otra se mira.
    pub fn acoplar_arriba(destino: &Destino, franja: Marco, huella: &Path) -> Informe {
        let reloj = std::time::Instant::now();
        let mut informe = acoplar_arriba_sin_medir(destino, franja, huella);
        informe.ms = reloj.elapsed().as_millis() as u64;
        informe
    }

    fn acoplar_arriba_sin_medir(destino: &Destino, franja: Marco, huella: &Path) -> Informe {
        if !ax::permiso_concedido() {
            return Informe {
                permiso: false,
                motivos: vec!["sin permiso de Accesibilidad: la banda flota".into()],
                ..Default::default()
            };
        }
        // Soltar primero, SIEMPRE, por lo mismo que abajo: acoplar dos veces guardaría como «original»
        // un sitio que ya era nuestro, y la ventana bajaría un poco más en cada pasada.
        soltar(huella);
        let app = destino.app.as_str();
        let mut informe = Informe {
            permiso: true,
            app: Some(app.to_string()),
            ..Default::default()
        };
        if !ax::sigue_siendo(destino.pid, app) {
            informe.motivos.push(format!("«{app}» ya no está: no se toca nada y la banda flota"));
            return informe;
        }
        let Some((marco, decision)) = plan_arriba(&ax::ventanas_de(destino.pid), destino.indice, franja) else {
            informe.motivos.push(format!("la ventana de la reunión de «{app}» ya no está: la banda flota"));
            return informe;
        };
        match decision {
            Decision::BajarYEncoger { y, alto } => {
                let pasos = pasos_del_acople_arriba(marco, y, alto);
                match ejecutar(destino.pid, destino.indice, marco, &pasos) {
                    Some(dejada) if hubo_cambio(marco, dejada) && quedo_bajo_la_franja(dejada, franja) => {
                        informe.ventanas = 1;
                        let pendiente = Pendiente {
                            version: VERSION_HUELLA,
                            huellas: vec![Huella { pid: destino.pid, app: app.to_string(), original: marco, dejada }],
                        };
                        if let Err(e) = guardar(huella, &pendiente) {
                            informe.motivos.push(format!("no se pudo anotar la huella del acople: {e}"));
                        }
                    }
                    // «Lo pedí» no es «pasó»: si macOS no la dejó bajar (pantalla completa, Spaces, una
                    // app que pelea su posición), se deshace lo que sí se hizo y la banda flota.
                    resultado => {
                        let ahora = resultado.or_else(|| {
                            ax::ventanas_de(destino.pid).into_iter().find(|(i, _)| *i == destino.indice).map(|(_, m)| m)
                        });
                        if let Some(ahora) = ahora {
                            ejecutar(destino.pid, destino.indice, ahora, &pasos_de_la_devolucion(ahora, marco));
                        }
                        informe.motivos.push(format!(
                            "«{app}» no dejó bajar su ventana (pantalla completa, Spaces o una app que pelea su sitio): se deshizo y la banda flota encima"
                        ));
                    }
                }
            }
            Decision::YaCabe => informe
                .motivos
                .push(format!("la ventana de la reunión de «{app}» ya empieza bajo la banda: no hace falta tocarla")),
            Decision::NoSeCruzan => informe
                .motivos
                .push(format!("la ventana de la reunión de «{app}» está en otro monitor: la banda no la tapa")),
            Decision::QuedariaInservible { alto_resultante } => informe.motivos.push(format!(
                "la ventana de la reunión de «{app}» quedaría en {alto_resultante:.0} px: se deja en paz y la banda flota encima"
            )),
            // `decidir_arriba` no produce `Encoger`, que es de abajo.
            Decision::Encoger { .. } => {}
        }
        informe
    }

    /// Hace los pasos **en su orden** sobre una ventana y devuelve cómo quedó, leído del sistema tras el
    /// último. Sin pasos, la ventana ya está como se quería y se devuelve tal cual. Si uno falla, se para:
    /// lo que siga dependía de él.
    fn ejecutar(pid: i32, indice: usize, desde: Marco, pasos: &[Paso]) -> Option<Marco> {
        let mut quedo = desde;
        for paso in pasos {
            quedo = match *paso {
                Paso::Alto(alto) => ax::encoger(pid, indice, alto)?,
                Paso::Mover { x, y } => ax::mover(pid, indice, x, y)?,
            };
        }
        Some(quedo)
    }

    /// Devuelve a su sitio y a su tamaño lo que este módulo tocó — el de esta sesión o el que dejó una
    /// caída anterior, que son el mismo camino: la huella no distingue, y por eso no hay dos
    /// funciones que puedan divergir. Desde el sprint 004, **primero la posición y después el tamaño**
    /// (`pasos_de_la_devolucion`); una huella del H1 solo tiene alto que devolver.
    pub fn soltar(huella: &Path) -> Informe {
        let reloj = std::time::Instant::now();
        let mut informe = soltar_sin_medir(huella);
        informe.ms = reloj.elapsed().as_millis() as u64;
        informe
    }

    fn soltar_sin_medir(huella: &Path) -> Informe {
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
                .find_map(|(i, m)| devolucion(m, h).map(|orig| (i, m, orig)));
            match devuelta {
                Some((i, actual, original)) => match ejecutar(h.pid, i, actual, &pasos_de_la_devolucion(actual, original)) {
                    Some(quedo) if !hubo_cambio(original, quedo) => informe.ventanas += 1,
                    // Mismo rasero que al acoplar: se cuenta lo que PASÓ, no lo que se pidió. Si
                    // la ventana no volvió a su sitio hay que decirlo — es la mitad de la
                    // promesa, y la que el usuario nota.
                    Some(quedo) => informe.motivos.push(format!(
                        "«{}» no volvió entera: quedó en ({:.0},{:.0}) con {:.0} px en vez de ({:.0},{:.0}) con {:.0}",
                        h.app, quedo.x, quedo.y, quedo.alto, original.x, original.y, original.alto
                    )),
                    None => informe
                        .motivos
                        .push(format!("«{}» rechazó devolver su ventana", h.app)),
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
    pub fn acoplar_arriba(_destino: &Destino, _franja: Marco, _huella: &Path) -> Informe {
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

    /// **La huella la escribe el escritor único** (auditoría del S3, B1): nace entera en un temporal 600 y
    /// se renombra encima, así que el archivo que queda es **otro** (otro inodo), no el viejo reescrito y
    /// apretado después. Demostrado en rojo con `fs::write`: el inodo era el mismo.
    #[cfg(unix)]
    #[test]
    fn la_huella_la_escribe_el_escritor_unico() {
        use std::os::unix::fs::MetadataExt;
        let base = std::env::temp_dir().join(format!("ag-acople-escritor-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        let ruta = ruta_de_la_huella(&base);
        guardar(&ruta, &Pendiente::default()).unwrap();
        let antes = std::fs::metadata(&ruta).unwrap().ino();
        guardar(&ruta, &Pendiente::default()).unwrap();
        assert_ne!(std::fs::metadata(&ruta).unwrap().ino(), antes, "la huella se reescribió en su sitio: no pasó por el almacén");
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

    // ---------------------------------------------------------------------------------------------
    // Arriba (sprint 004, ADR 004 enmienda 1)
    // ---------------------------------------------------------------------------------------------

    /// La franja de arriba en un MacBook con notch: bajo la barra de 38 pt.
    fn franja_arriba(alto: f64) -> Marco {
        Marco::nuevo(0.0, 38.0, 1512.0, alto)
    }

    /// Lo que macOS haría con cada paso, si obedece: el simulador de las pruebas de orden.
    fn aplicar(m: Marco, paso: Paso) -> Marco {
        match paso {
            Paso::Alto(alto) => Marco { alto, ..m },
            Paso::Mover { x, y } => Marco { x, y, ..m },
        }
    }

    /// La propiedad que importa, barrida como abajo: **la reunión baja hasta la banda y su borde
    /// inferior —con los controles de la llamada— no se mueve**, en los tres altos de la banda.
    #[test]
    fn arriba_la_reunion_baja_hasta_la_banda_y_su_borde_inferior_no_se_mueve() {
        let mut vistas = 0;
        for alto_banda in [44.0, 88.0, 200.0] {
            let f = franja_arriba(alto_banda);
            for y in [0.0, 38.0, 60.0, 120.0] {
                for alto in [400.0, 700.0, 944.0] {
                    let v = Marco::nuevo(100.0, y, 1200.0, alto);
                    if let Decision::BajarYEncoger { y: nuevo_y, alto: nuevo_alto } = decidir_arriba(v, f) {
                        vistas += 1;
                        assert_eq!(nuevo_y, f.fondo(), "no baja justo hasta la banda: y={y} alto={alto}");
                        assert_eq!(nuevo_y + nuevo_alto, v.fondo(), "su borde inferior se movió: y={y} alto={alto}");
                        assert!(nuevo_alto >= ALTO_MINIMO_UTIL, "se mutiló: {nuevo_alto}");
                    }
                }
            }
        }
        assert!(vistas >= 20, "el barrido casi no produjo decisiones: {vistas}");
    }

    #[test]
    fn arriba_lo_que_ya_empieza_bajo_la_banda_o_esta_en_otro_monitor_no_se_toca() {
        let f = franja_arriba(88.0);
        assert_eq!(decidir_arriba(Marco::nuevo(0.0, 126.0, 1200.0, 700.0), f), Decision::YaCabe);
        assert_eq!(decidir_arriba(Marco::nuevo(1600.0, 38.0, 900.0, 700.0), f), Decision::NoSeCruzan);
        // Un monitor encima del principal: la ventana acaba antes de la franja.
        assert_eq!(decidir_arriba(Marco::nuevo(0.0, -900.0, 1200.0, 800.0), f), Decision::NoSeCruzan);
        // Una ventana pequeña arriba del todo: bajarla la dejaría en 74 px.
        assert_eq!(
            decidir_arriba(Marco::nuevo(0.0, 38.0, 600.0, 200.0), f),
            Decision::QuedariaInservible { alto_resultante: 112.0 }
        );
    }

    /// **De una lista de ventanas sale UNA sola decisión, la del destino.** Las tres invaden la franja
    /// —el editor que estaba delante también—, y solo se decide sobre la que nombró la detección.
    #[test]
    fn arriba_de_una_lista_de_ventanas_sale_una_sola_decision() {
        let f = franja_arriba(88.0);
        let ventanas = [
            (0, Marco::nuevo(0.0, 38.0, 1512.0, 944.0)),
            (1, Marco::nuevo(200.0, 60.0, 900.0, 700.0)),
            (2, Marco::nuevo(100.0, 38.0, 1300.0, 900.0)),
        ];
        let (marco, decision) = plan_arriba(&ventanas, 2, f).expect("el destino está en la lista");
        assert_eq!(marco, ventanas[2].1, "decidió sobre otra ventana");
        assert!(matches!(decision, Decision::BajarYEncoger { .. }));
        assert_eq!(plan_arriba(&ventanas, 7, f), None, "un destino que ya no está no se cambia por otro");
    }

    /// **Primero el tamaño, después la posición**, y en ningún instante sale de la pantalla: tras el
    /// primer paso su borde inferior ya no llega más abajo que antes.
    #[test]
    fn al_acoplar_arriba_primero_se_encoge_y_despues_baja() {
        let f = franja_arriba(88.0);
        let v = Marco::nuevo(100.0, 38.0, 1300.0, 944.0);
        let Decision::BajarYEncoger { y, alto } = decidir_arriba(v, f) else { panic!("tenía que bajar") };
        let pasos = pasos_del_acople_arriba(v, y, alto);
        assert_eq!(pasos, [Paso::Alto(alto), Paso::Mover { x: 100.0, y }]);
        let tras_el_primero = aplicar(v, pasos[0]);
        assert!(tras_el_primero.fondo() <= v.fondo(), "el primer paso la saca por abajo");
        let dejada = pasos.iter().fold(v, |m, p| aplicar(m, *p));
        assert_eq!(dejada, Marco::nuevo(100.0, f.fondo(), 1300.0, alto));
        assert!(quedo_bajo_la_franja(dejada, f));
        assert!(!quedo_bajo_la_franja(v, f), "sin tocar, la banda la tapa");
    }

    /// **La devolución, entera y en su orden: primero sube, después crece.** Y es idempotente: una
    /// ventana que ya volvió no recibe ningún paso más.
    #[test]
    fn la_devolucion_primero_sube_despues_crece_y_es_idempotente() {
        let original = Marco::nuevo(100.0, 38.0, 1300.0, 944.0);
        let dejada = Marco::nuevo(100.0, 126.0, 1300.0, 856.0);
        let pasos = pasos_de_la_devolucion(dejada, original);
        assert_eq!(pasos, vec![Paso::Mover { x: 100.0, y: 38.0 }, Paso::Alto(944.0)]);
        let tras_subir = aplicar(dejada, pasos[0]);
        assert!(tras_subir.fondo() <= dejada.fondo(), "al subir primero no puede salirse por abajo");
        let vuelta = pasos.iter().fold(dejada, |m, p| aplicar(m, *p));
        assert_eq!(vuelta, original);
        assert!(!hubo_cambio(original, vuelta));
        assert_eq!(pasos_de_la_devolucion(vuelta, original), vec![], "devolver dos veces movió algo");
    }

    /// **Una huella del H1 se devuelve igual que entonces**: solo tocó el alto, así que la devolución
    /// es un paso de alto y ninguno de posición. El archivo de entonces se lee sin cambiar de versión.
    #[test]
    fn una_huella_del_h1_se_devuelve_como_entonces() {
        let base = std::env::temp_dir().join(format!("ag-acople-h1-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&base);
        std::fs::create_dir_all(&base).unwrap();
        let ruta = ruta_de_la_huella(&base);
        std::fs::write(
            &ruta,
            br#"{"version":1,"huellas":[{"pid":42,"app":"Google Chrome","original":{"x":0.0,"y":25.0,"ancho":1440.0,"alto":875.0},"dejada":{"x":0.0,"y":25.0,"ancho":1440.0,"alto":787.0}}]}"#,
        )
        .unwrap();
        let h = leer(&ruta).huellas.pop().expect("la huella del H1 ya no se lee");
        let actual = h.dejada;
        assert_eq!(devolucion(actual, &h), Some(h.original));
        assert_eq!(pasos_de_la_devolucion(actual, h.original), vec![Paso::Alto(875.0)]);
        let _ = std::fs::remove_dir_all(&base);
    }

    /// Arriba también manda tu gesto: si la moviste después de acoplarla, no se devuelve.
    #[test]
    fn arriba_no_se_devuelve_una_ventana_que_el_usuario_movio() {
        let h = Huella {
            pid: 42,
            app: "zoom.us".into(),
            original: Marco::nuevo(100.0, 38.0, 1300.0, 944.0),
            dejada: Marco::nuevo(100.0, 126.0, 1300.0, 856.0),
        };
        assert_eq!(devolucion(h.dejada, &h), Some(h.original));
        assert_eq!(devolucion(Marco::nuevo(100.0, 300.0, 1300.0, 856.0), &h), None, "la bajaste tú");
        assert_eq!(devolucion(Marco::nuevo(100.0, 126.0, 1300.0, 600.0), &h), None, "la encogiste tú");
    }

    /// Desde el sprint 004 «cambió» es posición o tamaño: devolverle el alto y dejarla abajo no es
    /// haberla devuelto.
    #[test]
    fn hubo_cambio_mira_tambien_la_posicion() {
        let a = Marco::nuevo(0.0, 38.0, 1440.0, 900.0);
        assert!(hubo_cambio(a, Marco::nuevo(0.0, 126.0, 1440.0, 900.0)));
        assert!(!hubo_cambio(a, Marco::nuevo(1.0, 39.0, 1440.0, 901.0)), "un punto de redondeo no es moverla");
    }

    /// **`AXPosition` se escribe en un solo sitio** (ADR 004, enmienda 1): `poner_posicion`, al que
    /// solo llama `ax::mover`, al que solo llama este módulo. Un segundo escritor —un «arreglo rápido»
    /// que recoloque ventanas desde otro sitio— cae aquí con su archivo.
    #[test]
    fn ax_mover_es_el_unico_que_escribe_la_posicion_de_una_ventana_ajena() {
        fn rs(dir: &Path) -> Vec<PathBuf> {
            std::fs::read_dir(dir)
                .unwrap()
                .flatten()
                .flat_map(|e| {
                    let p = e.path();
                    if p.is_dir() { rs(&p) } else if p.extension().is_some_and(|x| x == "rs") { vec![p] } else { vec![] }
                })
                .collect()
        }
        let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let aguja_atributo = ["\"AX", "Position\""].concat();
        let aguja_mover = ["ax::", "mover("].concat();
        let aguja_escribir = ["AXUIElementSet", "AttributeValue("].concat();
        for archivo in rs(&src) {
            let texto = std::fs::read_to_string(&archivo).unwrap();
            let nombre = archivo.strip_prefix(&src).unwrap().to_string_lossy().replace('\\', "/");
            let atributo = texto.matches(&aguja_atributo).count();
            let mover = texto.matches(&aguja_mover).count();
            match nombre.as_str() {
                "acople/ax.rs" => {
                    assert_eq!(atributo, 2, "ax.rs nombra AXPosition {atributo} veces: una para leer, otra para escribir");
                    // La escritura de la posición está dentro de `poner_posicion` y en ningún otro sitio.
                    let desde = texto.find("unsafe fn poner_posicion").expect("falta poner_posicion");
                    let cuerpo = &texto[desde..];
                    let fin = cuerpo.find("\n}\n").expect("poner_posicion sin cierre");
                    let escritura = ["CFString::new(", &aguja_atributo, ")"].concat();
                    assert!(cuerpo[..fin].contains(&escritura), "la escritura de AXPosition salió de poner_posicion");
                    assert_eq!(texto.matches(&escritura).count(), 1, "hay otra escritura de AXPosition en ax.rs");
                    // Dos escrituras de atributos en todo el crate: el alto y la posición.
                    assert_eq!(texto.matches(&aguja_escribir).count(), 3, "una declaración y dos llamadas, ni una más");
                }
                "acople/mod.rs" => assert!(mover >= 1, "el acople ya no mueve con ax::mover"),
                _ => {
                    assert_eq!(atributo, 0, "{nombre} nombra AXPosition: solo el acople lo toca");
                    assert_eq!(mover, 0, "{nombre} mueve ventanas ajenas: solo el acople lo hace");
                    assert_eq!(texto.matches(&aguja_escribir).count(), 0, "{nombre} escribe atributos de Accessibility");
                }
            }
        }
    }
}
