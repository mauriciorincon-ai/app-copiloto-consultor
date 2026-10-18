//! Las tres ventanas de Angel Ghost.
//!
//! | Ventana | Qué es | Protegida de la captura |
//! |---|---|---|
//! | `principal` | 960 × 640, las pantallas del cuaderno | **con la reunión abierta** (desde el S3) |
//! | `banda` | ancho de pantalla × 88 (asa → 200), bajo la barra de menús o pegada abajo (sprint 004); arriba, a la altura que elijas (sprint 011) | **sí, siempre** |
//! | `relleno` | la misma geometría, SIN contenido, justo debajo de la banda | no, **jamás, a propósito** |
//!
//! **La regla que gobierna este módulo: la protección FIJA vive SOLO en `tauri.conf.json`** (la
//! banda sí, las otras dos no), y las ventanas se construyen con `from_config`, que la aplica. Así el
//! riesgo nº 1 del sprint 001 —copiar el constructor de la banda para hacer el relleno y que el
//! relleno herede el flag— deja de existir por construcción: no hay nada que heredar en el
//! constructor. **La parte que cambia en marcha es una sola** (sprint 003, ADR 015 §10): el cuaderno,
//! mientras la reunión está abierta, y pasa por una sola función, [`proteger_el_cuaderno`], que solo
//! sabe tocar la principal.
//!
//! Y el invariante no se confía a la revisión de código: [`invariante_de_proteccion`] lo comprueba
//! **antes de abrir nada**. Si alguna vez deja de cumplirse, la banda no se abre. Una banda que se
//! abre sin su promesa es peor que una banda que no se abre.

use tauri::utils::config::WindowConfig;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, WebviewWindowBuilder};

pub mod geometria;
pub use geometria::Borde;

pub const PRINCIPAL: &str = "principal";
pub const BANDA: &str = "banda";
pub const RELLENO: &str = "relleno";

/// Alturas de la banda, en píxeles lógicos.
///
/// **Fuente única:** `docs/diseno/assets/ghost.css` (`--banda-h`, `--banda-h-ampliada`,
/// `--banda-h-voz`). El gate de tokens del lado TypeScript las excluye a propósito —el webview
/// ocupa la ventana entera, no la dibuja— así que quien las vigila es el test de este módulo, que
/// lee la maqueta y falla si la ventana y el diseño se separan.
pub const ALTO_COMPACTA: u32 = 88;
pub const ALTO_AMPLIADA: u32 = 200;
pub const ALTO_VOZ: u32 = 44;

/// El invariante de la promesa entera de la app: **exactamente una ventana protegida de la
/// captura, y es la banda**.
///
/// Dos fallos que esto ataja, y los dos son silenciosos:
/// - el **relleno protegido**: la franja volvería a mostrar lo que hay detrás y nadie se entera,
///   porque desde el Mac del consultor todo se ve exactamente igual;
/// - la **banda desprotegida**: el cliente ve la banda entera y el producto no tiene sentido.
pub fn invariante_de_proteccion(ventanas: &[(&str, bool)]) -> Result<(), String> {
    let protegidas: Vec<&str> = ventanas
        .iter()
        .filter(|(_, protegida)| *protegida)
        .map(|(etiqueta, _)| *etiqueta)
        .collect();

    match protegidas.as_slice() {
        [BANDA] => Ok(()),
        [] => Err("ninguna ventana está protegida de la captura: la banda sería visible para el cliente".into()),
        [una] => Err(format!(
            "la ventana protegida es «{una}» y tiene que ser «{BANDA}»"
        )),
        varias => Err(format!(
            "hay {} ventanas protegidas ({}), y solo la banda puede estarlo",
            varias.len(),
            varias.join(", ")
        )),
    }
}

/// **La parte que cambia en marcha (sprint 003, ADR 015 §10): el cuaderno.** Desde que empieza una
/// sesión hasta que su reunión se guarda o se descarta —⌥⎋ incluido: tus notas siguen ahí—, la
/// ventana principal toma el flag de la banda. Qué debe estar protegido, ventana por ventana:
/// la banda siempre, el cuaderno con la reunión abierta, el relleno jamás.
pub fn debe_estar_protegida(etiqueta: &str, reunion_abierta: bool) -> bool {
    match etiqueta {
        BANDA => true,
        PRINCIPAL => reunion_abierta,
        _ => false,
    }
}

/// **La compilación de Windows más vieja que sabe sacar una ventana de la captura** (`WDA_EXCLUDEFROMCAPTURE`):
/// Windows 10 versión 2004, la 19041 (sprint 008, M6 del S6). En una anterior, la ventana «protegida» sale negra pero
/// presente o no se protege: la banda se vería. Es el mínimo que el manual y el ADR 022 declaran.
pub const COMPILACION_MINIMA_DE_WINDOWS: u32 = 19041;

/// ¿Una Windows de esta compilación puede proteger la banda? Pura.
pub fn la_proteccion_cabe_en(compilacion: u32) -> bool {
    compilacion >= COMPILACION_MINIMA_DE_WINDOWS
}

/// La frase cuando no cabe: qué compilación es, cuál hace falta y qué pasa.
pub fn por_que_no_cabe(compilacion: u32) -> String {
    format!(
        "Esta Windows (compilación {compilacion}) no puede ocultar la banda de la captura de pantalla: hace falta Windows 10 \
         versión 2004 (compilación {COMPILACION_MINIMA_DE_WINDOWS}) o posterior. Angel Ghost no abre la banda sin su protección."
    )
}

/// La compilación de Windows de este equipo, de `RtlGetVersion` (que no miente aunque el programa no declare versión).
#[cfg(windows)]
pub fn compilacion_de_windows() -> Option<u32> {
    use windows::Wdk::System::SystemServices::RtlGetVersion;
    use windows::Win32::System::SystemInformation::OSVERSIONINFOW;
    let mut v = OSVERSIONINFOW { dwOSVersionInfoSize: std::mem::size_of::<OSVERSIONINFOW>() as u32, ..Default::default() };
    // SAFETY: `v` vive durante la llamada y trae su tamaño puesto; `RtlGetVersion` solo escribe dentro de la estructura.
    unsafe { RtlGetVersion(&mut v) }.is_ok().then_some(v.dwBuildNumber)
}

/// Antes de abrir la banda: si esta Windows no puede protegerla, no se abre (y se dice por qué). En el Mac y en una
/// Windows que alcanza, no hace nada. Si no se pudo leer la versión, se deja pasar: la protección la comprueba
/// después `invariante_de_proteccion` con las ventanas de verdad.
pub fn exigir_la_version_de_windows() -> Result<(), String> {
    #[cfg(windows)]
    if let Some(c) = compilacion_de_windows() {
        if !la_proteccion_cabe_en(c) {
            return Err(por_que_no_cabe(c));
        }
    }
    Ok(())
}

/// Dice por qué la app no arranca, con un cuadro de Windows: una app que se cierra en silencio no le dice nada a
/// nadie. Solo informa (un botón «Aceptar»); no toca ninguna protección del sistema. Fuera de Windows, no hace nada.
pub fn avisar_que_no_arranca(motivo: &str) {
    #[cfg(windows)]
    {
        use windows::core::HSTRING;
        use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};
        // SAFETY: dos cadenas anchas que viven durante la llamada y ningún manejador de ventana (sin dueño).
        unsafe { MessageBoxW(None, &HSTRING::from(motivo), &HSTRING::from("Angel Ghost"), MB_OK | MB_ICONINFORMATION) };
    }
    #[cfg(not(windows))]
    let _ = motivo;
}

/// Lo que [`proteger_el_cuaderno`] cambia: una ventana, la principal, y nada más. Separado para que
/// el test lo aplique sobre el `tauri.conf.json` de verdad y compruebe el invariante en marcha.
pub fn lo_que_cambia(reunion_abierta: bool) -> [(&'static str, bool); 1] {
    [(PRINCIPAL, reunion_abierta)]
}

/// El invariante en marcha: cada ventana, como dice [`debe_estar_protegida`].
pub fn invariante_en_marcha(ventanas: &[(&str, bool)], reunion_abierta: bool) -> Result<(), String> {
    for (etiqueta, protegida) in ventanas {
        let debe = debe_estar_protegida(etiqueta, reunion_abierta);
        if *protegida != debe {
            return Err(match (*etiqueta, debe) {
                (RELLENO, _) => "el relleno está protegido: la franja volvería a enseñar lo que hay detrás".into(),
                (BANDA, _) => "la banda no está protegida: el cliente la vería".into(),
                (PRINCIPAL, true) => "hay una reunión abierta y el cuaderno no está protegido: tu nota se vería al compartir".into(),
                (PRINCIPAL, false) => "sin reunión abierta, el cuaderno sigue protegido: se vería negro al compartirlo".into(),
                (otra, _) => format!("«{otra}» no debería existir"),
            });
        }
    }
    Ok(())
}

/// Pone o quita el flag del cuaderno. **Solo sabe tocar la principal**: el riesgo nº 1 del sprint 001
/// —un relleno protegido— sigue sin camino. Devuelve el error si macOS no dejó cambiarlo: quien la
/// llama lo enseña en Notas, porque la consola no la lee nadie (auditoría del S3, B4).
pub fn proteger_el_cuaderno<R: Runtime>(app: &AppHandle<R>, reunion_abierta: bool) -> Result<(), String> {
    let mut resultado = Ok(());
    for (etiqueta, protegida) in lo_que_cambia(reunion_abierta) {
        if let Some(v) = app.get_webview_window(etiqueta) {
            match v.set_content_protected(protegida) {
                Ok(()) => println!(
                    "[ventanas] «{etiqueta}» {}",
                    if protegida { "protegida mientras la reunión está abierta" } else { "sin proteger: la reunión se cerró" }
                ),
                Err(e) => {
                    println!("[ventanas] «{etiqueta}»: no se pudo cambiar su protección ({e})");
                    resultado = Err(format!("«{etiqueta}»: {e}"));
                }
            }
        }
    }
    resultado
}

/// Las parejas `(etiqueta, protegida)` tal como las declara `tauri.conf.json`.
pub fn proteccion_declarada(ventanas: &[WindowConfig]) -> Vec<(&str, bool)> {
    ventanas
        .iter()
        .map(|v| (v.label.as_str(), v.content_protected))
        .collect()
}

fn config_de<'a>(ventanas: &'a [WindowConfig], etiqueta: &str) -> Result<&'a WindowConfig, String> {
    ventanas
        .iter()
        .find(|v| v.label == etiqueta)
        .ok_or_else(|| format!("«{etiqueta}» no está declarada en tauri.conf.json"))
}

/// **¿Esta plataforma necesita relleno?** Solo macOS (ADR 022 §3). Allí la ventana protegida deja un hueco en la
/// captura y el relleno lo tapa con tu fondo de escritorio. En Windows la ventana protegida se EXCLUYE de la captura
/// (`WDA_EXCLUDEFROMCAPTURE`) y se ve lo que hay detrás: si hubiera un relleno debajo, sería él lo que se capturase,
/// una franja negra donde estaba la banda. Allí no se crea.
pub const HAY_RELLENO: bool = cfg!(target_os = "macos");

/// Abre la banda y su relleno, en ese orden inverso: **primero el relleno, después la banda**, para
/// que la banda quede encima. Las dos ocupan el ancho entero del monitor principal, en el borde que
/// diga `borde` (sprint 004: arriba de fábrica, bajo la barra de menús; abajo como en el H1).
///
/// No abre nada si el invariante de protección no se cumple.
///
/// **Y es idempotente desde el sprint 002, que es lo que hace posible que la banda VUELVA.** El
/// kill-switch la cierra —es una de las piezas del corte— y hasta ahora no había manera de
/// recuperarla sin reiniciar la app: `abrir_banda` estaba escrita, registrada como comando y sin un
/// solo llamador (hallazgo M4). Volver a llamarla con la banda en pantalla habría sido un error,
/// porque `build()` no admite una etiqueta repetida, así que la reposición se mira **ventana por
/// ventana**: se abre la que falte y se deja en paz la que esté.
///
/// Desde el sprint 011 abre **a la altura que elegiste** (`desplazamiento`, solo arriba): una banda que vuelve tras el
/// «×» o tras el corte vuelve donde la dejaste, no pegada arriba.
pub fn abrir_banda<R: Runtime>(app: &AppHandle<R>, borde: Borde, alto: u32, desplazamiento: u32) -> Result<(), String> {
    exigir_la_version_de_windows()?;
    let ventanas = app.config().app.windows.clone();
    invariante_de_proteccion(&proteccion_declarada(&ventanas))?;

    let f = franja(app, borde, alto, desplazamiento)?;
    let (ancho, x, y) = (f.ancho, f.x, f.y);

    for etiqueta in [RELLENO, BANDA] {
        if etiqueta == RELLENO && !HAY_RELLENO {
            continue;
        }
        // Una por una y no «si falta alguna, las dos»: si alguna vez quedara el relleno sin su
        // banda, lo que hay que reponer es la banda, y abrir un segundo relleno encima del que ya
        // está sería dejar dos rectángulos opacos sobre la reunión.
        if app.get_webview_window(etiqueta).is_some() {
            continue;
        }
        let cfg = config_de(&ventanas, etiqueta)?;
        let ventana = WebviewWindowBuilder::from_config(app, cfg)
            .map_err(|e| format!("«{etiqueta}»: {e}"))?
            .build()
            .map_err(|e| format!("«{etiqueta}»: {e}"))?;

        ventana
            .set_size(LogicalSize::new(ancho, f64::from(alto)))
            .map_err(|e| format!("«{etiqueta}»: {e}"))?;
        ventana
            .set_position(LogicalPosition::new(x, y))
            .map_err(|e| format!("«{etiqueta}»: {e}"))?;

        // El relleno no intercepta nada: ni clics ni foco. Es un rectángulo y nada más.
        if etiqueta == RELLENO {
            ventana
                .set_ignore_cursor_events(true)
                .map_err(|e| format!("«{etiqueta}»: {e}"))?;
        }
    }

    Ok(())
}

/// Ajusta el alto de la banda **y el de su relleno**, manteniendo pegado a la pantalla el borde que
/// la toca: el superior arriba, el inferior abajo. Es lo que hace el asa, y lo que recoloca la banda
/// cuando cambias de borde (`⌃⌥B`).
///
/// Los dos se mueven en la misma llamada a propósito: si el relleno pudiera quedarse en 88 px
/// mientras la banda crece a 200, la franja de arriba dejaría de estar cubierta y la captura vería
/// lo que hay detrás — el mismo fallo que el relleno existe para evitar, por la puerta de al lado.
///
/// Y desde el sprint 011, **a la altura que elegiste**: es también lo que hace el asa de posición mientras arrastras
/// (`desplazamiento`, solo arriba; acotado con el alto nuevo, así que agrandar una banda bajada no la saca de la pantalla).
pub fn ajustar_banda<R: Runtime>(app: &AppHandle<R>, borde: Borde, alto: u32, desplazamiento: u32) -> Result<(), String> {
    let alto = alto.clamp(ALTO_VOZ, ALTO_AMPLIADA);
    let f = franja(app, borde, alto, desplazamiento)?;
    let (ancho, x, y) = (f.ancho, f.x, f.y);
    for etiqueta in [RELLENO, BANDA] {
        let Some(v) = app.get_webview_window(etiqueta) else { continue };
        v.set_size(LogicalSize::new(ancho, f64::from(alto)))
            .map_err(|e| format!("«{etiqueta}»: {e}"))?;
        v.set_position(LogicalPosition::new(x, y))
            .map_err(|e| format!("«{etiqueta}»: {e}"))?;
    }
    Ok(())
}

/// La franja que ocupa la banda, como rectángulo.
///
/// Sale en **puntos con origen arriba-izquierda**, que es exactamente el sistema en el que habla
/// la Accessibility API: los dos lados usan el mismo sin conversión de por medio. Una conversión
/// de más entre estos dos puntos valdría 2× en una pantalla Retina, y 2× de 88 px es media banda
/// — el tipo de error que se ve como «el acople recorta de más» y se busca en el sitio equivocado.
///
/// **Con el desplazamiento** (sprint 011): la franja que el acople mira es la de la banda donde está, así que la reunión
/// baja hasta el borde inferior de la banda esté a la altura que esté (`acople::decidir_arriba` usa `franja.fondo()`).
pub fn franja<R: Runtime>(
    app: &AppHandle<R>,
    borde: Borde,
    alto: u32,
    desplazamiento: u32,
) -> Result<crate::acople::Marco, String> {
    Ok(geometria::franja_desplazada(borde, &monitor_principal(app)?, alto, desplazamiento))
}

/// El desplazamiento que cabe en el monitor principal con este alto: lo que se guarda al soltar el asa de posición, para
/// que las preferencias digan la altura de verdad y no la que pidió un arrastre que se pasó.
pub fn desplazamiento_acotado<R: Runtime>(app: &AppHandle<R>, alto: u32, desplazamiento: u32) -> Result<u32, String> {
    Ok(geometria::desplazamiento_acotado(&monitor_principal(app)?, alto, desplazamiento))
}

/// Cuánto mide la barra de menús del monitor principal: lo que el relleno sube el fondo con la banda
/// arriba. Cero si no se puede leer el monitor (el relleno queda negro arriba, que no filtra nada).
pub fn barra<R: Runtime>(app: &AppHandle<R>) -> f64 {
    monitor_principal(app).map(|m| geometria::barra(&m)).unwrap_or(0.0)
}

/// El alto actual de la banda, leído de la ventana. `None` si la banda no está abierta.
pub fn alto_actual<R: Runtime>(app: &AppHandle<R>) -> Option<u32> {
    let v = app.get_webview_window(BANDA)?;
    let escala = v.scale_factor().unwrap_or(1.0);
    let t = v.outer_size().ok()?.to_logical::<f64>(escala);
    Some(t.height.round() as u32)
}

/// El monitor principal en puntos, con su área útil: lo único que la geometría pura necesita saber del
/// sistema. El área útil la da Tauri (`work_area`, el `visibleFrame` de macOS ya volteado a origen
/// arriba-izquierda), así que no hace falta FFI para saber cuánto mide la barra de menús.
fn monitor_principal<R: Runtime>(app: &AppHandle<R>) -> Result<geometria::Monitor, String> {
    let monitor = app
        .primary_monitor()
        .map_err(|e| format!("no se pudo leer el monitor principal: {e}"))?
        .ok_or_else(|| "no hay monitor principal".to_string())?;
    let escala = monitor.scale_factor();
    let tamano: LogicalSize<f64> = monitor.size().to_logical(escala);
    let origen: LogicalPosition<f64> = monitor.position().to_logical(escala);
    let util = monitor.work_area();
    let util_tamano: LogicalSize<f64> = util.size.to_logical(escala);
    let util_origen: LogicalPosition<f64> = util.position.to_logical(escala);
    Ok(geometria::Monitor {
        marco: crate::acople::Marco::nuevo(origen.x, origen.y, tamano.width, tamano.height),
        util: crate::acople::Marco::nuevo(util_origen.x, util_origen.y, util_tamano.width, util_tamano.height),
    })
}

/// Deja en el log la geometría REAL de cada ventana: etiqueta, posición y tamaño. Solo metadatos
/// —ni una cadena de contenido— y es lo que vuelve contestable la pregunta «¿lo viste correr?».
/// Sin esto, comprobar dónde acabó una ventana obliga a interrogar al sistema desde fuera, y el
/// sistema contesta cosas distintas según el momento (medido: la misma ventana reportada en x=0,
/// x=-84 y x=-1753 en lecturas seguidas).
pub fn registrar_geometria<R: Runtime>(app: &AppHandle<R>) {
    for etiqueta in [PRINCIPAL, BANDA, RELLENO] {
        let Some(v) = app.get_webview_window(etiqueta) else {
            println!("[ventanas] «{etiqueta}»: no existe");
            continue;
        };
        let escala = v.scale_factor().unwrap_or(1.0);
        let t = v.outer_size().map(|s| s.to_logical::<f64>(escala));
        let p = v.outer_position().map(|q| q.to_logical::<f64>(escala));
        match (t, p) {
            (Ok(t), Ok(p)) => println!(
                "[ventanas] «{etiqueta}»: {:.0}x{:.0} en ({:.0},{:.0}) escala {escala}",
                t.width, t.height, p.x, p.y
            ),
            _ => println!("[ventanas] «{etiqueta}»: geometría ilegible"),
        }
    }
}

/// Cierra la banda y su relleno. El relleno **muere con la banda**, siempre: una franja de relleno
/// sin banda encima es un rectángulo opaco tapando la reunión.
pub fn cerrar_banda<R: Runtime>(app: &AppHandle<R>) {
    for etiqueta in [BANDA, RELLENO] {
        if let Some(v) = app.get_webview_window(etiqueta) {
            let _ = v.close();
        }
    }
}

#[cfg(test)]
mod tests {
    /// **El mínimo de Windows para proteger la banda** (sprint 008, M6 del S6): 19041, Windows 10 versión 2004.
    #[test]
    fn la_proteccion_pide_windows_10_2004_o_mas() {
        assert!(!la_proteccion_cabe_en(18363), "Windows 10 1909 no puede ocultar la banda");
        assert!(!la_proteccion_cabe_en(19040));
        assert!(la_proteccion_cabe_en(19041));
        assert!(la_proteccion_cabe_en(22631), "Windows 11");
        let frase = por_que_no_cabe(18363);
        assert!(frase.contains("18363") && frase.contains("19041") && frase.contains("no abre la banda"), "{frase}");
        assert!(!frase.contains("sprint"), "una frase que llega al usuario no habla de sprints");
    }

    /// `abrir_banda` pregunta por la versión ANTES de tocar nada (en el Mac, la pregunta no hace nada).
    #[test]
    fn abrir_la_banda_pregunta_primero_por_la_version() {
        let fuente = include_str!("mod.rs");
        let cuerpo = &fuente[fuente.find("pub fn abrir_banda<").expect("abrir_banda")..];
        let pregunta = cuerpo.find("exigir_la_version_de_windows()?").expect("no pregunta por la versión");
        let invariante = cuerpo.find("invariante_de_proteccion(").expect("sin invariante");
        assert!(pregunta < invariante);
        assert!(exigir_la_version_de_windows().is_ok() || cfg!(windows));
    }

    use super::*;

    #[test]
    fn exactamente_una_ventana_protegida_y_es_la_banda() {
        assert!(invariante_de_proteccion(&[
            (PRINCIPAL, false),
            (RELLENO, false),
            (BANDA, true),
        ])
        .is_ok());
    }

    #[test]
    fn el_relleno_protegido_es_un_error_nombrado() {
        let e = invariante_de_proteccion(&[(RELLENO, true), (BANDA, true)]).unwrap_err();
        assert!(e.contains("2 ventanas protegidas"), "mensaje inútil: {e}");
        assert!(e.contains(RELLENO), "el mensaje no nombra al culpable: {e}");
    }

    #[test]
    fn la_banda_sin_proteger_es_un_error() {
        let e = invariante_de_proteccion(&[(PRINCIPAL, false), (BANDA, false)]).unwrap_err();
        assert!(e.contains("ninguna"), "mensaje inútil: {e}");
    }

    #[test]
    fn proteger_la_ventana_equivocada_es_un_error() {
        let e = invariante_de_proteccion(&[(PRINCIPAL, true), (BANDA, false)]).unwrap_err();
        assert!(e.contains(PRINCIPAL) && e.contains(BANDA), "mensaje inútil: {e}");
    }

    /// El invariante se comprueba sobre el archivo que se envía, no sobre un ejemplo de juguete.
    #[test]
    fn el_tauri_conf_del_repo_cumple_el_invariante() {
        let crudo = include_str!("../../tauri.conf.json");
        let json: serde_json::Value = serde_json::from_str(crudo).expect("tauri.conf.json ilegible");
        let ventanas = json["app"]["windows"]
            .as_array()
            .expect("tauri.conf.json no declara ventanas");

        let parejas: Vec<(String, bool)> = ventanas
            .iter()
            .map(|v| {
                (
                    v["label"].as_str().expect("ventana sin label").to_string(),
                    v["contentProtected"].as_bool().unwrap_or(false),
                )
            })
            .collect();
        let prestadas: Vec<(&str, bool)> =
            parejas.iter().map(|(l, p)| (l.as_str(), *p)).collect();

        invariante_de_proteccion(&prestadas).expect("tauri.conf.json rompe el invariante");
        assert_eq!(prestadas.len(), 3, "se esperaban tres ventanas");
    }

    /// El cuaderno, protegido con la reunión abierta: aplicado sobre el `tauri.conf.json` de verdad,
    /// con reunión y sin ella, el invariante en marcha se cumple. Si `lo_que_cambia` tocara el
    /// relleno, o se olvidara del cuaderno, cae aquí con su nombre.
    #[test]
    fn el_cuaderno_se_protege_con_la_reunion_abierta_y_el_relleno_jamas() {
        let json: serde_json::Value =
            serde_json::from_str(include_str!("../../tauri.conf.json")).expect("tauri.conf.json ilegible");
        let declaradas: Vec<(String, bool)> = json["app"]["windows"]
            .as_array()
            .expect("sin ventanas")
            .iter()
            .map(|v| (v["label"].as_str().unwrap().to_string(), v["contentProtected"].as_bool().unwrap_or(false)))
            .collect();
        for abierta in [true, false] {
            let mut en_marcha = declaradas.clone();
            for (etiqueta, protegida) in lo_que_cambia(abierta) {
                en_marcha.iter_mut().filter(|(l, _)| l == etiqueta).for_each(|(_, p)| *p = protegida);
            }
            let prestadas: Vec<(&str, bool)> = en_marcha.iter().map(|(l, p)| (l.as_str(), *p)).collect();
            invariante_en_marcha(&prestadas, abierta).unwrap_or_else(|e| panic!("reunión abierta={abierta}: {e}"));
        }
    }

    #[test]
    fn el_invariante_en_marcha_nombra_cada_fallo() {
        let ok = [(PRINCIPAL, true), (RELLENO, false), (BANDA, true)];
        assert!(invariante_en_marcha(&ok, true).is_ok());
        assert!(invariante_en_marcha(&[(RELLENO, true)], true).unwrap_err().contains("relleno"));
        assert!(invariante_en_marcha(&[(PRINCIPAL, false)], true).unwrap_err().contains("tu nota se vería"));
        assert!(invariante_en_marcha(&[(PRINCIPAL, true)], false).unwrap_err().contains("negro"));
        assert!(invariante_en_marcha(&[(BANDA, false)], false).unwrap_err().contains("la vería"));
    }

    /// **En Windows no nace el relleno** (sprint 006, ADR 022 §3): allí la banda se excluye de la captura y un relleno
    /// debajo sería lo capturado, una franja negra. En macOS sí nace: es lo que tapa el hueco. Y `abrir_banda` lo mira
    /// antes de construir nada.
    #[test]
    fn el_relleno_solo_nace_en_macos() {
        assert_eq!(HAY_RELLENO, cfg!(target_os = "macos"));
        let fuente = include_str!("mod.rs");
        let abrir = &fuente[fuente.find("pub fn abrir_banda").expect("abrir_banda")..];
        let abrir = &abrir[..abrir.find("\n}\n").expect("su cierre")];
        let salta = abrir.find("!HAY_RELLENO").expect("abrir_banda no mira si esta plataforma necesita relleno");
        assert!(salta < abrir.find(".build()").expect("construye"), "el relleno se construye antes de mirarlo");
    }

    /// La banda y su relleno **no tienen bordes**, y eso es lo único que impide cerrarlas por
    /// separado: macOS no le manda el cierre a una ventana sin bordes. Medido en vivo el
    /// 2026-09-27 con una sonda en `CloseRequested`: ⌘W y «Close All» sobre la banda no llegan
    /// (tres ventanas antes y después), y sobre la principal sí. Con bordes, ⌘W cerraría la banda
    /// sola y dejaría el relleno —un rectángulo opaco— encima de la reunión. Se quitan juntas, con ⌥⎋ (`cerrar_banda`).
    #[test]
    fn la_banda_y_su_relleno_no_tienen_bordes() {
        let json: serde_json::Value =
            serde_json::from_str(include_str!("../../tauri.conf.json")).expect("tauri.conf.json ilegible");
        for etiqueta in [BANDA, RELLENO] {
            let v = json["app"]["windows"]
                .as_array()
                .and_then(|vs| vs.iter().find(|v| v["label"] == etiqueta))
                .unwrap_or_else(|| panic!("«{etiqueta}» no está en tauri.conf.json"));
            assert_eq!(
                v["decorations"].as_bool(),
                Some(false),
                "«{etiqueta}» con bordes: ⌘W la cerraría sola y el relleno quedaría encima de la reunión"
            );
        }
    }

    /// El alto de la banda y el de la maqueta son el mismo número o el gate de fidelidad compara
    /// contra un diseño que ya no existe.
    #[test]
    fn las_alturas_de_la_banda_son_las_de_la_maqueta() {
        let css = include_str!("../../../docs/diseno/assets/ghost.css");
        for (token, valor) in [
            ("--banda-h", ALTO_COMPACTA),
            ("--banda-h-ampliada", ALTO_AMPLIADA),
            ("--banda-h-voz", ALTO_VOZ),
        ] {
            let aguja = format!("{token}: {valor}px");
            assert!(
                css.contains(&aguja),
                "la maqueta no declara «{aguja}»: la ventana y el diseño se separaron"
            );
        }
    }

    /// El alto declarado en `tauri.conf.json` para la banda y el relleno es el mismo, y es el
    /// compacto: si divergen, la franja deja de estar cubierta por el relleno.
    #[test]
    fn banda_y_relleno_nacen_con_la_misma_geometria() {
        let crudo = include_str!("../../tauri.conf.json");
        let json: serde_json::Value = serde_json::from_str(crudo).unwrap();
        let ventanas = json["app"]["windows"].as_array().unwrap();
        let alto = |etiqueta: &str| -> u64 {
            ventanas
                .iter()
                .find(|v| v["label"] == etiqueta)
                .unwrap_or_else(|| panic!("falta «{etiqueta}»"))["height"]
                .as_u64()
                .expect("alto no numérico")
        };
        assert_eq!(alto(BANDA), alto(RELLENO), "la banda y su relleno difieren de alto");
        assert_eq!(alto(BANDA), u64::from(ALTO_COMPACTA));
    }
}
