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
//!
//! **En Windows (sprint 008, ADR 016 enmienda 2): el Programador de tareas.** La misma tarea con otra forma: un
//! disparador por cada minuto que vence y «ejecutar lo antes posible si se perdió» en vez de `RunAtLoad`; lo que corre
//! es `powershell.exe` sobre la misma lista ([`guion_de_windows`]), **no el binario de la app**. La definición es el
//! XML del Programador ([`xml_de_windows`]), puro y probado aquí; se registra por COM (`ITaskService`), sin lanzar
//! ningún programa desde la app (`vencimiento/windows.rs`).

use std::path::{Path, PathBuf};

/// En Windows, la tarea se registra en el Programador de tareas.
#[cfg(windows)]
#[path = "windows.rs"]
mod windows;

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

/// Una hora del calendario, en la hora del equipo: lo que `StartCalendarInterval` entiende (sin el año) y lo que el
/// Programador de tareas de Windows entiende (con él).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Minuto {
    pub anio: i32,
    pub mes: u32,
    pub dia: u32,
    pub hora: u32,
    pub minuto: u32,
}

/// Dónde vive la tarea: su nombre, su definición y su lista. La app usa la suya; la prueba en vivo, otra. En macOS la
/// definición es el plist de launchd; en Windows, una copia del XML que se le dio al Programador de tareas, en la
/// carpeta de la app, para saber sin preguntarle si cambió (el campo conserva su nombre de siempre).
#[derive(Debug, Clone)]
pub struct Tarea {
    pub etiqueta: String,
    pub plist: PathBuf,
    pub lista: PathBuf,
}

/// El nombre de la copia del XML de la tarea de Windows, en la carpeta de la app.
pub const XML_DE_WINDOWS: &str = "vencimiento-tarea.xml";

impl Tarea {
    /// La de la app: `~/Library/LaunchAgents/<etiqueta>.plist` y la lista en `carpeta_de_la_app`. En Windows, el XML
    /// junto a la lista: el Programador de tareas guarda la suya, y esta es la que la app compara.
    pub fn de_la_app(casa: &Path, carpeta_de_la_app: &Path) -> Tarea {
        let plist = if cfg!(windows) {
            carpeta_de_la_app.join(XML_DE_WINDOWS)
        } else {
            casa.join("Library/LaunchAgents").join(format!("{ETIQUETA}.plist"))
        };
        Tarea { etiqueta: ETIQUETA.into(), plist, lista: carpeta_de_la_app.join(LISTA) }
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

/// **El barrido de Windows, entero** (sprint 008): lo mismo que [`BARRIDO`] en PowerShell, sobre la misma lista. Solo
/// borra archivos `.ghost` cuya hora ya pasó; una línea rota o de un archivo que ya no existe no hace nada; no escribe
/// la lista. La ruta de la lista va entre comillas simples de PowerShell, con las suyas dobladas.
pub fn guion_de_windows(lista: &str) -> String {
    format!(
        "$ErrorActionPreference = 'SilentlyContinue'\n\
         $lista = '{lista}'\n\
         if (-not (Test-Path -LiteralPath $lista -PathType Leaf)) {{ exit 0 }}\n\
         $ahora = [DateTimeOffset]::UtcNow.ToUnixTimeSeconds()\n\
         foreach ($linea in [IO.File]::ReadAllLines($lista)) {{\n\
         \x20 $p = $linea.Split([char]9, 2)\n\
         \x20 if ($p.Count -ne 2 -or -not $p[1].EndsWith('.ghost')) {{ continue }}\n\
         \x20 $vence = 0L\n\
         \x20 if ([long]::TryParse($p[0], [ref]$vence) -and $vence -le $ahora) {{ Remove-Item -LiteralPath $p[1] -Force }}\n\
         }}\n\
         exit 0",
        lista = lista.replace('\'', "''")
    )
}

/// Los argumentos de `powershell.exe` en la tarea: sin perfil, sin preguntar, sin ventana y con el guion codificado
/// (UTF-16 en base64, la forma que PowerShell documenta para `-EncodedCommand`), para que ninguna comilla de la ruta
/// toque el XML ni la línea de órdenes.
pub fn argumentos_de_windows(lista: &str) -> String {
    use base64::Engine;
    let utf16: Vec<u8> = guion_de_windows(lista).encode_utf16().flat_map(u16::to_le_bytes).collect();
    format!(
        "-NoProfile -NonInteractive -WindowStyle Hidden -EncodedCommand {}",
        base64::engine::general_purpose::STANDARD.encode(utf16)
    )
}

/// **La tarea de Windows en el XML del Programador de tareas** (esquema 1.2). Un disparador por cada minuto que vence,
/// sin repetir; «ejecutar lo antes posible si se perdió» (`StartWhenAvailable`) hace de `RunAtLoad`: un vencimiento
/// con el PC apagado se barre al volver. Corre como el usuario, solo con su sesión abierta y sin privilegios
/// (`InteractiveToken`, `LeastPrivilege`), cinco minutos como mucho, y no escribe nada. Puro.
pub fn xml_de_windows(tarea: &Tarea, pendientes: &[Pendiente], local: &dyn Fn(i64) -> Minuto) -> String {
    let mut minutos: Vec<Minuto> = en_la_lista(pendientes).iter().map(|p| local(minuto_de(p.vence))).collect();
    minutos.sort();
    minutos.dedup();
    let disparadores: String = minutos
        .iter()
        .map(|m| {
            format!(
                "    <TimeTrigger>\n      <StartBoundary>{:04}-{:02}-{:02}T{:02}:{:02}:00</StartBoundary>\n      <Enabled>true</Enabled>\n    </TimeTrigger>\n",
                m.anio, m.mes, m.dia, m.hora, m.minuto
            )
        })
        .collect();
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>Angel Ghost</Author>
    <Description>Angel Ghost: borra tus notas, tu bandeja y tus ensayos cuando vencen, aunque la app esté cerrada. Solo lee la lista de la app y solo borra archivos .ghost vencidos.</Description>
  </RegistrationInfo>
  <Triggers>
{disparadores}  </Triggers>
  <Principals>
    <Principal id="Author">
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <StartWhenAvailable>true</StartWhenAvailable>
    <RunOnlyIfNetworkAvailable>false</RunOnlyIfNetworkAvailable>
    <ExecutionTimeLimit>PT5M</ExecutionTimeLimit>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>powershell.exe</Command>
      <Arguments>{argumentos}</Arguments>
    </Exec>
  </Actions>
</Task>
"#,
        argumentos = xml(&argumentos_de_windows(&tarea.lista.display().to_string())),
    )
}

/// La definición de la tarea en este sistema: el plist de launchd o el XML del Programador de tareas.
pub fn definicion(tarea: &Tarea, pendientes: &[Pendiente], local: &dyn Fn(i64) -> Minuto) -> String {
    if cfg!(windows) {
        xml_de_windows(tarea, pendientes, local)
    } else {
        plist(tarea, pendientes, local)
    }
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
    /// La definición no cambió: el sistema ya tiene la tarea (launchd la carga sola al iniciar sesión; el
    /// Programador de tareas la guarda).
    SinCambios,
    /// Se escribió la lista y la definición, y se volvió a registrar la tarea.
    Registrada,
    /// No queda nada que vencer: la tarea, su definición y la lista se quitaron.
    Quitada,
}

/// Pone la tarea al día con lo que vence. Escribe la lista y la definición (600) solo si cambiaron, y
/// vuelve a registrar la tarea en el sistema. Sin nada que vencer, la quita.
pub fn al_dia(tarea: &Tarea, pendientes: &[Pendiente], local: &dyn Fn(i64) -> Minuto) -> Result<Hecho, String> {
    if en_la_lista(pendientes).is_empty() {
        if !tarea.plist.exists() && !tarea.lista.exists() {
            return Ok(Hecho::SinCambios);
        }
        quitar(tarea);
        let _ = std::fs::remove_file(&tarea.plist);
        let _ = std::fs::remove_file(&tarea.lista);
        return Ok(Hecho::Quitada);
    }
    let lista = lista_en_texto(pendientes);
    let nuevo = definicion(tarea, pendientes, local);
    let igual = std::fs::read_to_string(&tarea.plist).is_ok_and(|v| v == nuevo)
        && std::fs::read_to_string(&tarea.lista).is_ok_and(|v| v == lista);
    if igual {
        return Ok(Hecho::SinCambios);
    }
    crate::almacen::escribir(&tarea.lista, lista.as_bytes())?;
    registrar(tarea, &nuevo)?;
    Ok(Hecho::Registrada)
}

/// launchd: el plist va a `~/Library/LaunchAgents` y se vuelve a cargar, con reintentos.
#[cfg(not(windows))]
fn registrar(tarea: &Tarea, plist: &str) -> Result<(), String> {
    crate::almacen::escribir_en_carpeta_ajena(&tarea.plist, plist.as_bytes())?;
    launchctl(&["bootout", &format!("gui/{}/{}", uid(), tarea.etiqueta)]);
    let dominio = format!("gui/{}", uid());
    let ruta = tarea.plist.display().to_string();
    if !con_reintentos(INTENTOS, &|| std::thread::sleep(ENTRE_INTENTOS), || launchctl(&["bootstrap", &dominio, &ruta])) {
        return Err("launchd no aceptó la tarea de vencimiento".into());
    }
    Ok(())
}

#[cfg(not(windows))]
fn quitar(tarea: &Tarea) {
    launchctl(&["bootout", &format!("gui/{}/{}", uid(), tarea.etiqueta)]);
}

/// Windows: la copia del XML en la carpeta de la app (600 dentro de 700) y la tarea en el Programador de tareas, que
/// sustituye a la anterior con el mismo nombre.
#[cfg(windows)]
fn registrar(tarea: &Tarea, xml: &str) -> Result<(), String> {
    crate::almacen::escribir(&tarea.plist, xml.as_bytes())?;
    windows::registrar(&tarea.etiqueta, xml)
}

#[cfg(windows)]
fn quitar(tarea: &Tarea) {
    windows::quitar(&tarea.etiqueta);
}

/// Cuántas veces se intenta registrar la tarea y cuánto se espera entre una y otra. `bootout` no
/// espera a que launchd suelte la tarea, y un `bootstrap` inmediato puede encontrarla todavía dentro
/// (auditoría del S3, B27).
pub const INTENTOS: u32 = 3;
pub const ENTRE_INTENTOS: std::time::Duration = std::time::Duration::from_millis(300);

/// Hace `hacer` hasta `intentos` veces, con `esperar` entre una y otra. Puro, para probarlo sin
/// launchd: quien llama le pasa el ejecutor.
pub fn con_reintentos(intentos: u32, esperar: &dyn Fn(), mut hacer: impl FnMut() -> bool) -> bool {
    for n in 1..=intentos {
        if hacer() {
            return true;
        }
        if n < intentos {
            esperar();
        }
    }
    false
}

/// El segundo programa que lanza la app, con la ruta entera y solo desde aquí. Devuelve si salió bien.
#[cfg(not(windows))]
fn launchctl(argumentos: &[&str]) -> bool {
    crate::hardware::vigilar("launchd");
    std::process::Command::new("/bin/launchctl")
        .args(argumentos)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
}

#[cfg(unix)]
fn uid() -> u32 {
    // SEGURIDAD: `getuid` no recibe nada y no puede fallar.
    unsafe { libc::getuid() }
}


/// El último segundo del día de `instante` en la hora del Mac: «fin del día» de la bandeja (ADR 016
/// §4), las 23:59 del día en que cierras.
pub fn fin_del_dia(instante: i64) -> i64 {
    let l = crate::reloj::local(instante);
    let pasado = i64::from(l.hora) * 3_600 + i64::from(l.minuto) * 60 + i64::from(l.segundo);
    instante + (86_399 - pasado)
}

/// La hora del equipo de un instante Unix (`reloj.rs`, ADR 022).
pub fn hora_del_mac(instante: i64) -> Minuto {
    let l = crate::reloj::local(instante);
    Minuto { anio: l.anio, mes: l.mes, dia: l.dia, hora: l.hora, minuto: l.minuto }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// UTC, para que las pruebas no dependan de la zona del runner (el mismo algoritmo civil de `reloj.rs`).
    fn utc(t: i64) -> Minuto {
        let l = crate::reloj::en_utc(t);
        Minuto { anio: l.anio, mes: l.mes, dia: l.dia, hora: l.hora, minuto: l.minuto }
    }

    fn tarea() -> Tarea {
        Tarea { etiqueta: ETIQUETA.into(), plist: "/x/p.plist".into(), lista: "/Users/u/Library/Application Support/app/vencimientos".into() }
    }

    fn p(vence: i64, ruta: &str) -> Pendiente {
        Pendiente { vence, ruta: ruta.into() }
    }

    // 2026-09-27 14:00:00 UTC
    const HOY: i64 = 1_790_517_600;

    /// **El registro se reintenta** (auditoría del S3, B27): si launchd todavía no soltó la tarea, el
    /// `bootstrap` falla; con tres intentos y una espera entre ellos, entra. Con un ejecutor falso, sin
    /// tocar launchd. Demostrado en rojo con un solo intento.
    #[test]
    fn el_registro_se_reintenta_antes_de_rendirse() {
        use std::cell::Cell;
        let (veces, esperas) = (Cell::new(0), Cell::new(0));
        let espera = || esperas.set(esperas.get() + 1);
        let ok = con_reintentos(INTENTOS, &espera, || {
            veces.set(veces.get() + 1);
            veces.get() == 3
        });
        assert!(ok, "falló dos veces y a la tercera tenía que entrar");
        assert_eq!((veces.get(), esperas.get()), (3, 2));
        let (veces, esperas) = (Cell::new(0), Cell::new(0));
        let espera = || esperas.set(esperas.get() + 1);
        assert!(!con_reintentos(INTENTOS, &espera, || {
            veces.set(veces.get() + 1);
            false
        }));
        assert_eq!((veces.get(), esperas.get()), (3, 2), "tras el último intento no se espera");
    }

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

    /// **La tarea de Windows** (sprint 008): un disparador por minuto que vence, con el año; lo perdido se corre al
    /// volver; corre como el usuario y sin privilegios; lanza `powershell.exe`, no el binario de la app; y no escribe
    /// nada.
    #[test]
    fn el_xml_de_windows_tiene_un_disparador_por_minuto_y_no_depende_de_la_app() {
        let t = Tarea { etiqueta: ETIQUETA.into(), plist: "C:/a/x.xml".into(), lista: r"C:\Users\Ana O'Neil\AppData\Roaming\app\vencimientos".into() };
        let x = xml_de_windows(
            &t,
            &[p(HOY + 3 * 3_600 + 10, "/b/a.ghost"), p(HOY + 3 * 3_600 + 20, "/b/b.ghost"), p(HOY + 90 * 86_400, "/n/c.ghost"), p(0, "/n/d.ghost")],
            &utc,
        );
        assert_eq!(x.matches("<TimeTrigger>").count(), 2, "{x}");
        assert!(x.contains("<StartBoundary>2026-09-27T17:01:00</StartBoundary>"), "{x}");
        assert!(x.contains("<StartBoundary>2026-12-26T14:00:00</StartBoundary>"), "{x}");
        assert!(x.contains("<StartWhenAvailable>true</StartWhenAvailable>"));
        assert!(x.contains("<LogonType>InteractiveToken</LogonType>") && x.contains("<RunLevel>LeastPrivilege</RunLevel>"));
        assert!(x.contains("<Command>powershell.exe</Command>"), "la tarea no depende del binario de la app");
        assert!(!x.contains("angel-ghost.exe") && !x.contains("copiloto-consultor.exe"));
        assert!(x.contains("-EncodedCommand "), "la ruta no viaja en claro por la línea de órdenes");
        assert!(!x.contains("O'Neil"), "la ruta va dentro del guion codificado");
    }

    /// El guion de Windows hace lo mismo que el de `sh`: solo `.ghost`, solo vencido, y no escribe la lista. Una comilla
    /// simple en la ruta no lo rompe.
    #[test]
    fn el_guion_de_windows_solo_borra_ghost_vencidos_de_la_lista() {
        let g = guion_de_windows(r"C:\Users\Ana O'Neil\vencimientos");
        assert!(g.contains(r"$lista = 'C:\Users\Ana O''Neil\vencimientos'"), "{g}");
        assert!(g.contains("EndsWith('.ghost')") && g.contains("$vence -le $ahora"));
        assert!(g.contains("Remove-Item -LiteralPath $p[1] -Force"));
        assert!(!g.contains("Set-Content") && !g.contains("Out-File") && !g.contains("WriteAll"), "el guion no escribe nada");
        // El codificado es el guion en UTF-16: se puede leer de vuelta.
        use base64::Engine;
        let a = argumentos_de_windows("C:/l");
        let b64 = a.rsplit(' ').next().unwrap();
        let bytes = base64::engine::general_purpose::STANDARD.decode(b64).unwrap();
        let u16s: Vec<u16> = bytes.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        assert_eq!(String::from_utf16(&u16s).unwrap(), guion_de_windows("C:/l"));
    }

    /// La definición es la de este sistema: el plist en el Mac, el XML en Windows.
    #[test]
    fn cada_sistema_tiene_su_definicion() {
        let d = definicion(&tarea(), &[p(HOY, "/b/a.ghost")], &utc);
        assert_eq!(d.contains("<plist"), !cfg!(windows));
        assert_eq!(d.contains("<Task "), cfg!(windows));
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
