//! Las tres ventanas de Angel Ghost.
//!
//! | Ventana | Qué es | Protegida de la captura |
//! |---|---|---|
//! | `principal` | 960 × 640, las pantallas del cuaderno | no |
//! | `banda` | ancho de pantalla × 88 (asa → 200), pegada al borde inferior | **sí, y es la única** |
//! | `relleno` | la misma geometría, SIN contenido, justo debajo de la banda | no, **a propósito** |
//!
//! **La regla que gobierna este módulo: el flag de protección vive SOLO en `tauri.conf.json`.**
//! Aquí no se escribe `content_protected` en ninguna parte y las ventanas se construyen con
//! `from_config`, que lo aplica. Así el riesgo nº 1 del sprint —copiar el constructor de la banda
//! para hacer el relleno y que el relleno herede el flag— deja de existir por construcción: no hay
//! nada que heredar en el constructor.
//!
//! Y el invariante no se confía a la revisión de código: [`invariante_de_proteccion`] lo comprueba
//! **antes de abrir nada**. Si alguna vez deja de cumplirse, la banda no se abre. Una banda que se
//! abre sin su promesa es peor que una banda que no se abre.

use tauri::utils::config::WindowConfig;
use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager, Runtime, WebviewWindowBuilder};

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

/// Abre la banda y su relleno, en ese orden inverso: **primero el relleno, después la banda**, para
/// que la banda quede encima. Las dos ocupan el ancho entero del monitor principal, pegadas al
/// borde inferior.
///
/// No abre nada si el invariante de protección no se cumple.
///
/// **Y es idempotente desde el sprint 002, que es lo que hace posible que la banda VUELVA.** El
/// kill-switch la cierra —es una de las siete piezas del corte— y hasta ahora no había manera de
/// recuperarla sin reiniciar la app: `abrir_banda` estaba escrita, registrada como comando y sin un
/// solo llamador (hallazgo M4). Volver a llamarla con la banda en pantalla habría sido un error,
/// porque `build()` no admite una etiqueta repetida, así que la reposición se mira **ventana por
/// ventana**: se abre la que falte y se deja en paz la que esté.
pub fn abrir_banda<R: Runtime>(app: &AppHandle<R>, alto: u32) -> Result<(), String> {
    let ventanas = app.config().app.windows.clone();
    invariante_de_proteccion(&proteccion_declarada(&ventanas))?;

    let (ancho, x, y) = geometria(app, alto)?;

    for etiqueta in [RELLENO, BANDA] {
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

/// Ajusta el alto de la banda **y el de su relleno**, manteniendo el borde inferior pegado a la
/// pantalla. Es lo que hace el asa.
///
/// Los dos se mueven en la misma llamada a propósito: si el relleno pudiera quedarse en 88 px
/// mientras la banda crece a 200, la franja de arriba dejaría de estar cubierta y la captura vería
/// lo que hay detrás — el mismo fallo que el relleno existe para evitar, por la puerta de al lado.
pub fn ajustar_banda<R: Runtime>(app: &AppHandle<R>, alto: u32) -> Result<(), String> {
    let alto = alto.clamp(ALTO_VOZ, ALTO_AMPLIADA);
    let (ancho, x, y) = geometria(app, alto)?;
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
pub fn franja<R: Runtime>(app: &AppHandle<R>, alto: u32) -> Result<crate::acople::Marco, String> {
    let (ancho, x, y) = geometria(app, alto)?;
    Ok(crate::acople::Marco::nuevo(x, y, ancho, f64::from(alto)))
}

/// El alto actual de la banda, leído de la ventana. `None` si la banda no está abierta.
pub fn alto_actual<R: Runtime>(app: &AppHandle<R>) -> Option<u32> {
    let v = app.get_webview_window(BANDA)?;
    let escala = v.scale_factor().unwrap_or(1.0);
    let t = v.outer_size().ok()?.to_logical::<f64>(escala);
    Some(t.height.round() as u32)
}

/// Ancho y esquina superior izquierda de la franja, para un alto dado.
fn geometria<R: Runtime>(app: &AppHandle<R>, alto: u32) -> Result<(f64, f64, f64), String> {
    let monitor = app
        .primary_monitor()
        .map_err(|e| format!("no se pudo leer el monitor principal: {e}"))?
        .ok_or_else(|| "no hay monitor principal".to_string())?;
    let escala = monitor.scale_factor();
    let tamano: LogicalSize<f64> = monitor.size().to_logical(escala);
    let origen: LogicalPosition<f64> = monitor.position().to_logical(escala);
    Ok((
        tamano.width,
        origen.x,
        origen.y + tamano.height - f64::from(alto),
    ))
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
