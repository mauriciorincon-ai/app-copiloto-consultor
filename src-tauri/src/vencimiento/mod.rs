//! EL VENCIMIENTO — lo que vence se borra, aunque no abras la app (ADR 016 §5).
//!
//! Tres capas, y esta es la tercera: con la app cerrada, **launchd**. Una tarea programada al minuto
//! de cada vencimiento (`StartCalendarInterval`) y una pasada al iniciar sesión (`RunAtLoad`). Lo que
//! corre es `/bin/sh` sobre una lista «segundos · ruta», **no el binario de la app**: funciona aunque
//! la borres. Entre vencimientos no corre nada; solo hay una hora anotada en launchd.
//!
//! Lo que vive aquí:
//! - la **lista** (`vencimientos`, 600, en la carpeta de la app), que la app regenera a partir de las
//!   cabeceras de los archivos —la fuente de verdad— y que el barrido **lee y no escribe**;
//! - el **plist**, generado de forma pura y probado con `plutil`;
//! - el **barrido**, entero, en [`BARRIDO`], probado con `/bin/sh` de verdad (`tests/`);
//! - y el registro con `/bin/launchctl`: el **segundo programa** que lanza la app, solo desde aquí
//!   (lo cuenta `contador-de-red`).

use std::path::{Path, PathBuf};

/// El nombre de la tarea en launchd.
pub const ETIQUETA: &str = "com.aiapps.copiloto-consultor.vencimiento";
/// La app a la que Ajustes del Sistema atribuye la tarea («Angel Ghost», no «sh»).
pub const APP: &str = "com.aiapps.copiloto-consultor";
/// El nombre de la lista, en la carpeta de la app.
pub const LISTA: &str = "vencimientos";

/// **El barrido, entero.** `$1` es la lista. Solo borra archivos `.ghost` cuya hora ya pasó; una
/// línea de un archivo que ya no existe no hace nada. No escribe la lista: la app y launchd no se
/// pisan. El tabulador sale de `printf` para que no haya un tabulador invisible dentro del plist.
pub const BARRIDO: &str = r#"ahora=$(/bin/date +%s)
[ -f "$1" ] || exit 0
tab=$(printf '\t')
while IFS="$tab" read -r vence ruta; do
  case "$ruta" in *.ghost) ;; *) continue ;; esac
  [ "$vence" -le "$ahora" ] 2>/dev/null && /bin/rm -f -- "$ruta"
done < "$1"
exit 0"#;

/// Algo que vence: un archivo de notas o de la bandeja, y cuándo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pendiente {
    /// Segundos Unix. 0 es «siempre» y no entra en la lista.
    pub vence: i64,
    pub ruta: PathBuf,
}

/// Una hora del calendario, en la hora del Mac: lo que `StartCalendarInterval` entiende.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Minuto {
    pub mes: u32,
    pub dia: u32,
    pub hora: u32,
    pub minuto: u32,
}

/// Dónde vive la tarea: su nombre, su plist y su lista. La app usa la suya; la prueba en vivo, otra.
#[derive(Debug, Clone)]
pub struct Tarea {
    pub etiqueta: String,
    pub plist: PathBuf,
    pub lista: PathBuf,
}

impl Tarea {
    /// La de la app: `~/Library/LaunchAgents/<etiqueta>.plist` y la lista en `carpeta_de_la_app`.
    pub fn de_la_app(casa: &Path, carpeta_de_la_app: &Path) -> Tarea {
        Tarea {
            etiqueta: ETIQUETA.into(),
            plist: casa.join("Library/LaunchAgents").join(format!("{ETIQUETA}.plist")),
            lista: carpeta_de_la_app.join(LISTA),
        }
    }
}

/// Lo que entra en la lista: solo lo que vence, solo `.ghost`, y ninguna ruta que la parta en dos.
pub fn en_la_lista(pendientes: &[Pendiente]) -> Vec<&Pendiente> {
    let mut v: Vec<&Pendiente> = pendientes
        .iter()
        .filter(|p| p.vence > 0)
        .filter(|p| p.ruta.extension().is_some_and(|e| e == "ghost"))
        .filter(|p| p.ruta.to_str().is_some_and(|r| !r.contains(['\n', '\t', '\r'])))
        .collect();
    v.sort_by_key(|p| p.vence);
    v
}

/// La lista, en texto: una línea «segundos<TAB>ruta» por archivo.
pub fn lista_en_texto(pendientes: &[Pendiente]) -> String {
    en_la_lista(pendientes)
        .iter()
        .map(|p| format!("{}\t{}\n", p.vence, p.ruta.display()))
        .collect()
}

/// El minuto en que la tarea tiene que correr para un vencimiento: **el siguiente** al vencimiento,
/// porque el barrido compara «vence ≤ ahora» y launchd arranca en el segundo 0.
pub fn minuto_de(vence: i64) -> i64 {
    (vence + 59).div_euclid(60) * 60
}

/// El plist de la tarea. `local` pasa un instante Unix a la hora del Mac (la app usa `localtime_r`;
/// las pruebas, UTC).
pub fn plist(tarea: &Tarea, pendientes: &[Pendiente], local: &dyn Fn(i64) -> Minuto) -> String {
    let mut minutos: Vec<Minuto> = en_la_lista(pendientes).iter().map(|p| local(minuto_de(p.vence))).collect();
    minutos.sort();
    minutos.dedup();
    let calendario: String = minutos
        .iter()
        .map(|m| {
            format!(
                "\t\t<dict>\n\t\t\t<key>Month</key>\n\t\t\t<integer>{}</integer>\n\t\t\t<key>Day</key>\n\t\t\t<integer>{}</integer>\n\t\t\t<key>Hour</key>\n\t\t\t<integer>{}</integer>\n\t\t\t<key>Minute</key>\n\t\t\t<integer>{}</integer>\n\t\t</dict>\n",
                m.mes, m.dia, m.hora, m.minuto
            )
        })
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{etiqueta}</string>
	<key>ProgramArguments</key>
	<array>
		<string>/bin/sh</string>
		<string>-c</string>
		<string>{barrido}</string>
		<string>angel-ghost-vencimiento</string>
		<string>{lista}</string>
	</array>
	<key>StartCalendarInterval</key>
	<array>
{calendario}	</array>
	<key>RunAtLoad</key>
	<true/>
	<key>ProcessType</key>
	<string>Background</string>
	<key>AssociatedBundleIdentifiers</key>
	<array>
		<string>{app}</string>
	</array>
</dict>
</plist>
"#,
        etiqueta = xml(&tarea.etiqueta),
        barrido = xml(BARRIDO),
        lista = xml(&tarea.lista.display().to_string()),
        app = APP,
    )
}

fn xml(texto: &str) -> String {
    texto.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// **¿Corrió la tarea mientras la app estaba cerrada?** No se adivina: se mide (ADR 016 §5). Si la
/// tarea estaba instalada y al arrancar hay algo que venció hace más de 2 minutos, no corrió
/// —desactivada en Ítems de inicio, o launchd no la cargó— y Honestidad lo dice.
pub fn no_corrio(pendientes: &[Pendiente], ahora: i64, instalada: bool) -> bool {
    instalada && en_la_lista(pendientes).iter().any(|p| p.vence + 120 < ahora)
}

/// Lo que hizo [`al_dia`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hecho {
    /// El plist no cambió: launchd ya tiene la tarea (la carga sola al iniciar sesión).
    SinCambios,
    /// Se escribió la lista y el plist, y se volvió a registrar la tarea.
    Registrada,
    /// No queda nada que vencer: la tarea, el plist y la lista se quitaron.
    Quitada,
}

/// Pone la tarea al día con lo que vence. Escribe la lista y el plist (600) solo si cambiaron, y
/// vuelve a registrar la tarea en launchd. Sin nada que vencer, la quita.
pub fn al_dia(tarea: &Tarea, pendientes: &[Pendiente], local: &dyn Fn(i64) -> Minuto) -> Result<Hecho, String> {
    if en_la_lista(pendientes).is_empty() {
        if !tarea.plist.exists() && !tarea.lista.exists() {
            return Ok(Hecho::SinCambios);
        }
        launchctl(&["bootout", &format!("gui/{}/{}", uid(), tarea.etiqueta)]);
        let _ = std::fs::remove_file(&tarea.plist);
        let _ = std::fs::remove_file(&tarea.lista);
        return Ok(Hecho::Quitada);
    }
    let lista = lista_en_texto(pendientes);
    let nuevo = plist(tarea, pendientes, local);
    let igual = std::fs::read_to_string(&tarea.plist).is_ok_and(|v| v == nuevo)
        && std::fs::read_to_string(&tarea.lista).is_ok_and(|v| v == lista);
    if igual {
        return Ok(Hecho::SinCambios);
    }
    crate::almacen::escribir(&tarea.lista, lista.as_bytes())?;
    crate::almacen::escribir_en_carpeta_ajena(&tarea.plist, nuevo.as_bytes())?;
    launchctl(&["bootout", &format!("gui/{}/{}", uid(), tarea.etiqueta)]);
    if !launchctl(&["bootstrap", &format!("gui/{}", uid()), &tarea.plist.display().to_string()]) {
        return Err("launchd no aceptó la tarea de vencimiento".into());
    }
    Ok(Hecho::Registrada)
}

/// El segundo programa que lanza la app, con la ruta entera y solo desde aquí. Devuelve si salió bien.
fn launchctl(argumentos: &[&str]) -> bool {
    std::process::Command::new("/bin/launchctl")
        .args(argumentos)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

fn uid() -> u32 {
    // SEGURIDAD: `getuid` no recibe nada y no puede fallar.
    unsafe { libc::getuid() }
}

/// El último segundo del día de `instante` en la hora del Mac: «fin del día» de la bandeja (ADR 016
/// §4), las 23:59 del día en que cierras.
pub fn fin_del_dia(instante: i64) -> i64 {
    // SEGURIDAD: `localtime_r` escribe en una estructura que vive en esta función.
    let pasado = unsafe {
        let t = instante as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        i64::from(tm.tm_hour) * 3_600 + i64::from(tm.tm_min) * 60 + i64::from(tm.tm_sec)
    };
    instante + (86_399 - pasado)
}

/// La hora del Mac de un instante Unix, con `localtime_r`: sin una biblioteca de fechas.
pub fn hora_del_mac(instante: i64) -> Minuto {
    // SEGURIDAD: `localtime_r` escribe en una estructura que vive en esta función.
    unsafe {
        let t = instante as libc::time_t;
        let mut tm: libc::tm = std::mem::zeroed();
        libc::localtime_r(&t, &mut tm);
        Minuto { mes: (tm.tm_mon + 1) as u32, dia: tm.tm_mday as u32, hora: tm.tm_hour as u32, minuto: tm.tm_min as u32 }
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// UTC, para que las pruebas no dependan de la zona del runner.
    fn utc(t: i64) -> Minuto {
        let dia = t.div_euclid(86_400);
        let s = t.rem_euclid(86_400);
        // días desde 1970-01-01 → mes y día (algoritmo civil de Howard Hinnant)
        let z = dia + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        Minuto { mes: m as u32, dia: d as u32, hora: (s / 3_600) as u32, minuto: (s % 3_600 / 60) as u32 }
    }

    fn tarea() -> Tarea {
        Tarea { etiqueta: ETIQUETA.into(), plist: "/x/p.plist".into(), lista: "/Users/u/Library/Application Support/app/vencimientos".into() }
    }

    fn p(vence: i64, ruta: &str) -> Pendiente {
        Pendiente { vence, ruta: ruta.into() }
    }

    // 2026-09-27 14:00:00 UTC
    const HOY: i64 = 1_790_517_600;

    #[test]
    fn la_lista_lleva_solo_lo_que_vence_y_solo_archivos_ghost() {
        let texto = lista_en_texto(&[
            p(HOY + 3_600, "/n/Angel Ghost/reunion-2026-09-27-1402.ghost"),
            p(0, "/n/Angel Ghost/para-siempre.ghost"),
            p(HOY, "/n/otro.txt"),
            p(HOY, "/n/mal\nnombre.ghost"),
            p(HOY - 10, "/b/bandeja/reunion.ghost"),
        ]);
        assert_eq!(texto, format!("{}\t/b/bandeja/reunion.ghost\n{}\t/n/Angel Ghost/reunion-2026-09-27-1402.ghost\n", HOY - 10, HOY + 3_600));
    }

    /// «Fin del día» son las 23:59:59 del día de cierre en la hora del Mac, sea la zona que sea.
    #[test]
    fn el_fin_del_dia_es_el_ultimo_segundo_del_dia_del_mac() {
        for instante in [HOY, HOY + 9 * 3_600 + 59 * 60 + 59, HOY + 13 * 3_600] {
            let fin = fin_del_dia(instante);
            assert!(fin >= instante && fin - instante < 86_400, "{instante} → {fin}");
            let m = hora_del_mac(fin);
            assert_eq!((m.hora, m.minuto), (23, 59), "{instante} → {fin}");
            assert_ne!(hora_del_mac(fin + 1).dia, m.dia, "el segundo siguiente ya es mañana");
        }
    }

    #[test]
    fn la_tarea_corre_en_el_minuto_siguiente_al_vencimiento() {
        assert_eq!(minuto_de(HOY), HOY);
        assert_eq!(minuto_de(HOY + 1), HOY + 60);
        assert_eq!(minuto_de(HOY + 59), HOY + 60);
    }

    #[test]
    fn el_plist_tiene_una_hora_por_vencimiento_y_ninguna_repetida() {
        let x = plist(
            &tarea(),
            &[p(HOY + 3 * 3_600 + 10, "/b/a.ghost"), p(HOY + 3 * 3_600 + 20, "/b/b.ghost"), p(HOY + 90 * 86_400, "/n/c.ghost"), p(0, "/n/d.ghost")],
            &utc,
        );
        assert_eq!(x.matches("<key>Minute</key>").count(), 2, "{x}");
        assert!(x.contains("<integer>17</integer>\n\t\t\t<key>Minute</key>\n\t\t\t<integer>1</integer>"), "3 h y 10 y 20 s después, las dos a las 17:01: {x}");
        assert!(x.contains("<key>RunAtLoad</key>\n\t<true/>"));
        assert!(x.contains("<string>/bin/sh</string>"), "la tarea no depende del binario de la app");
        assert!(!x.contains("copiloto-consultor.app"), "la tarea no depende del binario de la app");
        assert!(x.contains(&format!("<string>{APP}</string>")));
        assert!(x.contains("Application Support/app/vencimientos"));
        assert!(!x.contains("StandardOutPath") && !x.contains("StandardErrorPath"), "la tarea no escribe nada");
    }

    #[test]
    fn el_barrido_va_escapado_dentro_del_plist() {
        let x = plist(&tarea(), &[p(HOY, "/b/a.ghost")], &utc);
        assert!(x.contains("[ -f &quot;$1&quot; ] || exit 0"), "{x}");
        assert!(!x.contains("< \"$1\""), "un «<» sin escapar rompe el XML");
    }

    #[test]
    fn que_la_tarea_no_corrio_se_mide() {
        let vencido = [p(HOY - 600, "/b/a.ghost")];
        assert!(no_corrio(&vencido, HOY, true), "venció hace 10 min con la tarea instalada: no corrió");
        assert!(!no_corrio(&vencido, HOY, false), "sin la tarea, no hay nada que acusar");
        assert!(!no_corrio(&[p(HOY - 60, "/b/a.ghost")], HOY, true), "hace un minuto: puede estar corriendo");
        assert!(!no_corrio(&[p(0, "/n/siempre.ghost")], HOY, true));
    }

    #[test]
    fn la_hora_del_mac_es_la_de_localtime() {
        let m = hora_del_mac(HOY);
        assert!((1..=12).contains(&m.mes) && (1..=31).contains(&m.dia) && m.hora < 24 && m.minuto < 60);
    }
}
