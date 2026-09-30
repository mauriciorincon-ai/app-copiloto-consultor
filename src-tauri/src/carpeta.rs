//! LA CARPETA DE TUS NOTAS — la capa que escribe (ADR 015 §2, §4, §6 y §8).
//!
//! Recibe el contenido **ya armado** por `notas/` —que decidió qué es tuyo— y hace lo que aquel no
//! puede: pedir la llave al Llavero, sellar, escribir, listar, abrir, exportar, borrar y barrer lo
//! vencido. No ve un solo turno del cliente: ni tiene cómo recibirlo.
//!
//! Dos reglas que viven aquí y no en otro sitio:
//! - **Jamás se reemplaza una llave que existe.** Si el Llavero no contesta, no se «crea una nueva»:
//!   hacerlo borraría la que abre todas tus reuniones guardadas. Se falla y se dice.
//! - **Solo se cifra con una llave que el Llavero ya devolvió.** Una llave recién creada se guarda y
//!   se vuelve a leer antes de usarla: cifrar con una llave que no quedó guardada es escribir un
//!   archivo que nadie podrá abrir nunca.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::llavero::{self, Servicio};
use crate::notas::cifrado::{self, Llave};
use crate::notas::{Contenido, Resumen};

/// El nombre de la carpeta, dentro de la de la app (`~/Library/Application Support/<app>/notas/`),
/// junto a la bandeja. **No en Documentos** (ADR 016, decisión A del usuario, 2026-09-27): ahí el
/// `sh` de launchd no puede borrar lo vencido, y la papelera de iCloud guarda 30 días lo que se borra.
pub const CARPETA: &str = "notas";
pub const EXTENSION: &str = "ghost";
/// La cuenta de la llave en el servicio «Angel Ghost · notas». Con versión: si un día cambia la forma
/// de la llave, la vieja sigue ahí para abrir lo viejo.
pub const CUENTA_DE_LA_LLAVE: &str = "llave-v1";

/// De dónde sale la llave. En la app, el Llavero; en los tests, la memoria.
pub trait Llaves {
    /// ¿Hay llave? `Err` si no se sabe (el Llavero no contestó): no es lo mismo que «no».
    fn existe(&self) -> Result<bool, String>;
    fn leer(&self) -> Result<Llave, String>;
    /// Guarda una llave nueva. Solo se llama cuando [`Llaves::existe`] dijo «no».
    fn crear(&self, llave: &Llave) -> Result<(), String>;
}

/// El Llavero de macOS, servicio «Angel Ghost · notas», en el llavero de inicio de sesión (ADR 015,
/// enmienda 2).
pub struct DelLlavero;

impl Llaves for DelLlavero {
    fn existe(&self) -> Result<bool, String> {
        llavero::existe(Servicio::Notas, CUENTA_DE_LA_LLAVE)
    }

    fn leer(&self) -> Result<Llave, String> {
        let mut texto = llavero::leer(Servicio::Notas, CUENTA_DE_LA_LLAVE)
            .ok_or("el Llavero no entregó la llave de tus notas")?;
        let llave = Llave::de_hex(&texto);
        // SEGURIDAD: ceros son UTF-8 válido.
        unsafe { texto.as_mut_vec() }.fill(0);
        llave.ok_or_else(|| "lo que hay en el Llavero no es una llave de notas".into())
    }

    fn crear(&self, llave: &Llave) -> Result<(), String> {
        let mut hex = llave.a_hex();
        let r = llavero::guardar(Servicio::Notas, CUENTA_DE_LA_LLAVE, &hex);
        // SEGURIDAD: ceros son UTF-8 válido.
        unsafe { hex.as_mut_vec() }.fill(0);
        r
    }
}

/// La llave de tus notas. Con `crear`, la primera vez se crea; sin él, si no hay, se dice.
pub fn la_llave(llaves: &dyn Llaves, crear: bool) -> Result<Llave, String> {
    if llaves.existe()? {
        return llaves.leer();
    }
    if !crear {
        return Err("no hay llave de notas en el Llavero de este Mac".into());
    }
    let nueva = Llave::nueva();
    llaves.crear(&nueva)?;
    let vuelta = llaves.leer()?;
    if !vuelta.igual(&nueva) {
        return Err("el Llavero devolvió otra llave que la que se acaba de guardar".into());
    }
    Ok(vuelta)
}

/// Una reunión guardada, como la lista la enseña: sin abrirla.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reunion {
    pub archivo: String,
    pub bytes: u64,
    /// Cuándo se guardó, en segundos Unix (la fecha del archivo).
    pub guardada: i64,
    /// Cuándo se borra sola, en segundos Unix; 0 es «siempre».
    pub vence: i64,
}

/// Lo que devuelve guardar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Guardada {
    pub archivo: String,
    pub bytes: u64,
    pub vence: i64,
}

/// El primer nombre (`base.ghost`, `base-2.ghost`…) que no existe en **ninguna** de `carpetas`. Las
/// notas y la bandeja de una reunión se llaman igual para que «Guardar» desde la bandeja encuentre su
/// reunión (ADR 016 §4), así que el nombre tiene que estar libre en las dos: una reunión sin nota deja
/// bandeja sin archivo de notas, y la siguiente del mismo cliente y día no puede tomar su nombre
/// (auditoría del S3, A1).
pub fn nombre_libre(base: &str, carpetas: &[&Path]) -> Option<String> {
    (1..1000)
        .map(|n| if n == 1 { format!("{base}.{EXTENSION}") } else { format!("{base}-{n}.{EXTENSION}") })
        .find(|nombre| nombre_valido(nombre) && carpetas.iter().all(|c| !c.join(nombre).exists()))
}

/// La carpeta, dondequiera que esté. En la app, `notas/` dentro de la carpeta de la app; en los
/// tests, una del temporal.
pub struct Carpeta {
    raiz: PathBuf,
}

impl Carpeta {
    pub fn en(raiz: PathBuf) -> Carpeta {
        Carpeta { raiz }
    }

    pub fn raiz(&self) -> &Path {
        &self.raiz
    }

    /// Sella y escribe. `base` es el nombre sin extensión (`notas::nombre_del_archivo`); si ya
    /// existe, se prueba `-2`, `-3`… La carpeta nace 700 y el archivo 600 (`almacen`).
    pub fn guardar(&self, llaves: &dyn Llaves, contenido: &Contenido, base: &str, vence: i64) -> Result<Guardada, String> {
        let archivo = self.nombre_para(base).ok_or("no queda un nombre libre para esta reunión")?;
        self.guardar_como(llaves, contenido, &archivo, vence)
    }

    /// Sella y escribe con este nombre exacto: el que la bandeja ya apuntó como su reunión al cerrar
    /// (ADR 016 §4), para que «Guardar» desde la bandeja encuentre el archivo.
    pub fn guardar_como(&self, llaves: &dyn Llaves, contenido: &Contenido, archivo: &str, vence: i64) -> Result<Guardada, String> {
        if !nombre_valido(archivo) {
            return Err("ese nombre no es el de una reunión".into());
        }
        let llave = la_llave(llaves, true)?;
        let mut claro = contenido.a_bytes();
        let sellado = cifrado::sellar(&llave, vence, &claro);
        claro.fill(0);
        crate::almacen::carpeta_privada(&self.raiz)?;
        crate::almacen::escribir(&self.raiz.join(archivo), &sellado)?;
        Ok(Guardada { archivo: archivo.to_string(), bytes: sellado.len() as u64, vence })
    }

    /// El nombre que tendría una reunión guardada ahora con esta `base`: el primero libre **en esta
    /// carpeta**. Una reunión que además deja bandeja pide el suyo con `nombre_libre` sobre las dos.
    pub fn nombre_para(&self, base: &str) -> Option<String> {
        nombre_libre(base, &[&self.raiz])
    }

    /// Las reuniones guardadas, la más reciente primero. Solo se lee la cabecera: **sin la llave**.
    /// Lo que no sea un archivo de notas de esta app no se lista.
    pub fn lista(&self) -> Vec<Reunion> {
        let Ok(entradas) = std::fs::read_dir(&self.raiz) else {
            return Vec::new();
        };
        let mut reuniones: Vec<Reunion> = entradas
            .flatten()
            .filter_map(|e| {
                let archivo = e.file_name().into_string().ok()?;
                if !nombre_valido(&archivo) {
                    return None;
                }
                let m = e.metadata().ok().filter(|m| m.is_file())?;
                let vence = cifrado::vence_de(&leer_cabecera(&e.path())?).ok()?;
                Some(Reunion { archivo, bytes: m.len(), guardada: segundos(m.modified().ok()?), vence })
            })
            .collect();
        reuniones.sort_by(|a, b| b.guardada.cmp(&a.guardada).then_with(|| a.archivo.cmp(&b.archivo)));
        reuniones
    }

    /// Abre una reunión guardada. Quien llama ya pidió el desbloqueo (ADR 015 §5).
    pub fn abrir(&self, llaves: &dyn Llaves, archivo: &str) -> Result<Contenido, String> {
        let ruta = self.ruta_de(archivo)?;
        let sellado = std::fs::read(&ruta).map_err(|e| format!("no se pudo leer la reunión: {e}"))?;
        let llave = la_llave(llaves, false)?;
        let mut claro = cifrado::abrir(&llave, &sellado).map_err(|e| format!("la reunión {e}"))?;
        let contenido = Contenido::de_bytes(&claro);
        claro.fill(0);
        contenido
    }

    /// Sella `claro` y lo escribe con **este** nombre, encima si ya existe. Es lo que usan la bandeja
    /// (que se llama como su reunión) y «Guardar» desde la bandeja (que vuelve a sellar la reunión).
    pub fn escribir_sellado(&self, llaves: &dyn Llaves, archivo: &str, claro: &[u8], vence: i64) -> Result<u64, String> {
        let ruta = self.ruta_de(archivo)?;
        let llave = la_llave(llaves, true)?;
        let sellado = cifrado::sellar(&llave, vence, claro);
        crate::almacen::carpeta_privada(&self.raiz)?;
        crate::almacen::escribir(&ruta, &sellado)?;
        Ok(sellado.len() as u64)
    }

    /// Abre un archivo sellado y devuelve lo de dentro, en claro, con su vencimiento. **Quien llama
    /// pisa los bytes** cuando termina.
    pub fn abrir_en_claro(&self, llaves: &dyn Llaves, archivo: &str) -> Result<(Vec<u8>, i64), String> {
        let ruta = self.ruta_de(archivo)?;
        // Sin el nombre: lleva el del cliente, y este error acaba en el log (auditoría del S3, B30).
        let sellado = std::fs::read(&ruta).map_err(|e| format!("no se pudo leer el archivo de la reunión: {e}"))?;
        let vence = cifrado::vence_de(&sellado).map_err(|e| format!("el archivo de la reunión {e}"))?;
        let llave = la_llave(llaves, false)?;
        let claro = cifrado::abrir(&llave, &sellado).map_err(|e| format!("el archivo de la reunión {e}"))?;
        Ok((claro, vence))
    }

    /// **Guardar desde la bandeja** (ADR 016 §4): la propuesta entra en su reunión, que se vuelve a
    /// sellar **con el mismo vencimiento**. Si la reunión no tenía nada tuyo, no hubo archivo: nace
    /// ahora, con el encabezado y el vencimiento que habría tenido.
    pub fn sumar_propuesta(
        &self,
        llaves: &dyn Llaves,
        archivo: &str,
        propuesta: crate::propuestas::Propuesta,
        encabezado: &crate::notas::Encabezado,
        vence_si_nace: i64,
    ) -> Result<(), String> {
        let ruta = self.ruta_de(archivo)?;
        let (mut contenido, vence) = if ruta.exists() {
            let (mut claro, vence) = self.abrir_en_claro(llaves, archivo)?;
            let c = Contenido::de_bytes(&claro);
            claro.fill(0);
            (c?, vence)
        } else {
            let c = Contenido {
                version: crate::notas::VERSION,
                encabezado: encabezado.clone(),
                nota: String::new(),
                acuerdos: Vec::new(),
                fijadas: Vec::new(),
                mis_turnos: Vec::new(),
                propuestas: Vec::new(),
            };
            (c, vence_si_nace)
        };
        contenido.propuestas.push(propuesta);
        let mut claro = contenido.a_bytes();
        let r = self.escribir_sellado(llaves, archivo, &claro, vence);
        claro.fill(0);
        r.map(|_| ())
    }

    /// Lo que vence en esta carpeta, para la tarea de launchd (`vencimiento`). «Siempre» no entra.
    pub fn pendientes(&self) -> Vec<crate::vencimiento::Pendiente> {
        self.lista()
            .into_iter()
            .filter(|r| r.vence != 0)
            .map(|r| crate::vencimiento::Pendiente { vence: r.vence, ruta: self.raiz.join(&r.archivo) })
            .collect()
    }

    /// «Borrar ahora».
    pub fn borrar(&self, archivo: &str) -> Result<(), String> {
        let ruta = self.ruta_de(archivo)?;
        std::fs::remove_file(&ruta).map_err(|e| format!("no se pudo borrar la reunión: {e}"))
    }

    /// Borra lo vencido a `ahora` (segundos Unix). Devuelve cuántas. Lo que no se deja leer, se deja:
    /// borrar un archivo que no se entiende no es cumplir una retención.
    pub fn barrer(&self, ahora: i64) -> usize {
        self.lista()
            .into_iter()
            .filter(|r| r.vence != 0 && r.vence <= ahora)
            .filter(|r| std::fs::remove_file(self.raiz.join(&r.archivo)).is_ok())
            .count()
    }

    /// Exportar a texto: el contenido, en claro, en `destino`. **Quita el cifrado**, y por eso se
    /// pregunta antes (la pantalla); aquí se hace. El archivo nace 600, como todo lo que escribe la app,
    /// pero **la carpeta la eligió el usuario y no se toca** (auditoría del S3, M5).
    pub fn exportar(&self, llaves: &dyn Llaves, archivo: &str, destino: &Path, idioma: &str) -> Result<(), String> {
        let contenido = self.abrir(llaves, archivo)?;
        let mut texto = a_texto(&contenido, idioma);
        let r = crate::almacen::escribir_en_carpeta_ajena(destino, texto.as_bytes());
        // SEGURIDAD: ceros son UTF-8 válido.
        unsafe { texto.as_mut_vec() }.fill(0);
        r
    }

    pub fn ruta_de(&self, archivo: &str) -> Result<PathBuf, String> {
        if !nombre_valido(archivo) {
            return Err("ese no es el nombre de una reunión guardada".into());
        }
        Ok(self.raiz.join(archivo))
    }
}

/// Un nombre de reunión: letras minúsculas, cifras y guiones, y `.ghost`. Nada de barras ni puntos
/// delante: lo que llega de la pantalla no puede salirse de la carpeta.
pub fn nombre_valido(archivo: &str) -> bool {
    let Some(base) = archivo.strip_suffix(&format!(".{EXTENSION}")) else {
        return false;
    };
    !base.is_empty()
        && base.len() <= 80
        && !base.starts_with('-')
        && base.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

fn leer_cabecera(ruta: &Path) -> Option<Vec<u8>> {
    use std::io::Read;
    let mut f = std::fs::File::open(ruta).ok()?;
    let mut cab = vec![0u8; cifrado::CABECERA];
    f.read_exact(&mut cab).ok()?;
    Some(cab)
}

fn segundos(t: std::time::SystemTime) -> i64 {
    t.duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as i64)
}

/// Segundos Unix de ahora.
pub fn ahora() -> i64 {
    segundos(std::time::SystemTime::now())
}

/// Lo que se exporta: Markdown, en el idioma de la interfaz. Los títulos se redactan en los dos
/// idiomas, no se traducen (regla bilingüe).
pub fn a_texto(c: &Contenido, idioma: &str) -> String {
    let en = idioma.starts_with("en");
    let (nota, acuerdos, propuestas, fijadas, turnos, min) = if en {
        ("Your note", "Agreements", "Suggestions you saved", "Pinned cards", "Your turns", "min")
    } else {
        ("Tu nota", "Acuerdos", "Propuestas que guardaste", "Fichas fijadas", "Tus turnos", "min")
    };
    let mut t = String::new();
    let titulo = match &c.encabezado.cliente {
        Some(cliente) => format!("# {cliente} · {} · {} {min}\n", c.encabezado.empezo, c.encabezado.minutos),
        None => format!("# {} · {} {min}\n", c.encabezado.empezo, c.encabezado.minutos),
    };
    t.push_str(&titulo);
    if !c.nota.trim().is_empty() {
        t.push_str(&format!("\n## {nota}\n\n{}\n", c.nota.trim_end()));
    }
    if !c.acuerdos.is_empty() {
        t.push_str(&format!("\n## {acuerdos}\n\n"));
        c.acuerdos.iter().for_each(|a| t.push_str(&format!("- {a}\n")));
    }
    if !c.propuestas.is_empty() {
        t.push_str(&format!("\n## {propuestas}\n\n"));
        c.propuestas.iter().for_each(|p| t.push_str(&format!("- {} · {}\n", p.hora, propuesta_en_texto(p, en))));
    }
    if !c.fijadas.is_empty() {
        t.push_str(&format!("\n## {fijadas}\n\n"));
        for f in &c.fijadas {
            let seccion = f.seccion.as_ref().map(|s| format!(" · {s}")).unwrap_or_default();
            t.push_str(&format!("- {} — {}{seccion}\n", f.titular, f.documento));
        }
    }
    if !c.mis_turnos.is_empty() {
        t.push_str(&format!("\n## {turnos}\n\n"));
        c.mis_turnos.iter().for_each(|m| t.push_str(&format!("- {} · {}\n", m.hora, m.texto)));
    }
    t
}

/// Una propuesta guardada, como se lee al exportar: tu frase, o del cliente **el hecho con su
/// plantilla, jamás su turno** (ADR 016 §2). Las plantillas son las de la pantalla (`src/propuesta.ts`
/// y el diccionario), redactadas en los dos idiomas.
fn propuesta_en_texto(p: &crate::propuestas::Propuesta, en: bool) -> String {
    use crate::propuestas::{De, Regla};
    if p.de == De::Tuyo {
        return p.texto.clone();
    }
    let t = &p.texto;
    match (p.regla, en) {
        (Regla::Choque, false) => match (&p.ficha, &p.seccion) {
            (Some(f), Some(s)) => format!("Dijeron «{t}»; tu ficha fijada dice «{f}» ({s})"),
            (Some(f), None) => format!("Dijeron «{t}»; tu ficha fijada dice «{f}»"),
            _ => format!("Dijeron «{t}»"),
        },
        (Regla::Choque, true) => match (&p.ficha, &p.seccion) {
            (Some(f), Some(s)) => format!("They said “{t}”; your pinned card says “{f}” ({s})"),
            (Some(f), None) => format!("They said “{t}”; your pinned card says “{f}”"),
            _ => format!("They said “{t}”"),
        },
        (Regla::Nombre, false) => format!("Mencionaron a «{t}», que no está en tu corpus"),
        (Regla::Nombre, true) => format!("They mentioned “{t}”, who is not in your corpus"),
        (Regla::Pregunta, false) => format!("Te preguntaron por: {t}"),
        (Regla::Pregunta, true) => format!("They asked about: {t}"),
        (_, false) => format!("Dijeron «{t}»"),
        (_, true) => format!("They said “{t}”"),
    }
}

/// La línea del log al guardar: **cuánto, nunca qué** (ADR 015 §9). Ni el nombre del archivo —lleva
/// el del cliente—, ni la ruta, ni una palabra de la nota.
pub fn linea_de_log(r: &Resumen, g: &Guardada) -> String {
    format!(
        "[notas] guardada y cifrada · {} párrafos · {} acuerdos · {} fijadas · {} turnos tuyos · {} bytes · {}",
        r.parrafos,
        r.acuerdos,
        r.fijadas,
        r.turnos,
        g.bytes,
        if g.vence == 0 { "sin vencimiento".to_string() } else { "con vencimiento".to_string() }
    )
}

/// El Llavero en memoria, con sus fallos a mano. Uno para todas las pruebas que guardan (carpeta,
/// reunión, bandeja): antes había una copia por archivo.
#[cfg(test)]
pub(crate) mod doble {
    use super::Llaves;
    use crate::notas::cifrado::Llave;
    use std::cell::RefCell;

    #[derive(Default)]
    pub(crate) struct EnMemoria {
        pub(crate) hex: RefCell<Option<String>>,
        pub(crate) no_contesta: bool,
        pub(crate) creadas: RefCell<usize>,
    }

    impl Llaves for EnMemoria {
        fn existe(&self) -> Result<bool, String> {
            if self.no_contesta {
                return Err("el Llavero no contestó (-25308)".into());
            }
            Ok(self.hex.borrow().is_some())
        }
        fn leer(&self) -> Result<Llave, String> {
            let h = self.hex.borrow().clone().ok_or("no hay")?;
            Llave::de_hex(&h).ok_or_else(|| "no es una llave".into())
        }
        fn crear(&self, llave: &Llave) -> Result<(), String> {
            *self.creadas.borrow_mut() += 1;
            *self.hex.borrow_mut() = Some(llave.a_hex());
            Ok(())
        }
    }
}

#[cfg(test)]
mod pruebas {
    use super::doble::EnMemoria;
    use super::*;
    use crate::notas::{Cuaderno, Encabezado, FichaFijada};

    fn carpeta(nombre: &str) -> Carpeta {
        let d = std::env::temp_dir().join(format!("ag-carpeta-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        Carpeta::en(d.join(CARPETA))
    }

    fn contenido(nota: &str) -> Contenido {
        let mut c = Cuaderno::nuevo(false);
        c.escribir(nota);
        c.acordar("Cuarta fuente: cotización aparte");
        c.ver(FichaFijada {
            titular: "Limpieza de datos: hasta tres fuentes".into(),
            documento: "Propuesta Páramo Azul".into(),
            seccion: Some("§3.2".into()),
            unidad: None,
            ..Default::default()
        });
        c.fijar_la_vigente();
        c.contenido(Encabezado { empezo: "2026-09-20 14:02".into(), minutos: 47, cliente: Some("Páramo Azul".into()) })
    }

    #[test]
    fn se_guarda_cifrado_y_se_vuelve_a_abrir() {
        let c = carpeta("ida");
        let llaves = EnMemoria::default();
        let g = c.guardar(&llaves, &contenido("Piden la cuarta fuente."), "paramo-azul-2026-09-20", 0).unwrap();
        assert_eq!(g.archivo, "paramo-azul-2026-09-20.ghost");
        let crudo = std::fs::read(c.raiz().join(&g.archivo)).unwrap();
        assert!(!String::from_utf8_lossy(&crudo).contains("cuarta fuente"), "la nota quedó en claro en el disco");
        assert_eq!(c.abrir(&llaves, &g.archivo).unwrap().nota, "Piden la cuarta fuente.");
        let _ = std::fs::remove_dir_all(c.raiz().parent().unwrap());
    }

    #[cfg(unix)]
    #[test]
    fn la_carpeta_nace_700_y_el_archivo_600() {
        use std::os::unix::fs::PermissionsExt;
        let c = carpeta("permisos");
        let g = c.guardar(&EnMemoria::default(), &contenido("x"), "reunion-2026-09-27-1402", 0).unwrap();
        let modo = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(modo(c.raiz()), 0o700, "la carpeta de tus notas la puede listar otra cuenta del Mac");
        assert_eq!(modo(&c.raiz().join(&g.archivo)), 0o600, "el archivo de la reunión lo puede leer otra cuenta");
        let _ = std::fs::remove_dir_all(c.raiz().parent().unwrap());
    }

    #[test]
    fn un_nombre_que_ya_existe_no_se_pisa() {
        let c = carpeta("repetido");
        let llaves = EnMemoria::default();
        let a = c.guardar(&llaves, &contenido("primera"), "paramo-azul-2026-09-20", 0).unwrap();
        let b = c.guardar(&llaves, &contenido("segunda"), "paramo-azul-2026-09-20", 0).unwrap();
        assert_eq!(b.archivo, "paramo-azul-2026-09-20-2.ghost");
        assert_eq!(c.abrir(&llaves, &a.archivo).unwrap().nota, "primera");
        assert_eq!(*llaves.creadas.borrow(), 1, "una sola llave para todas las reuniones");
        let _ = std::fs::remove_dir_all(c.raiz().parent().unwrap());
    }

    #[test]
    fn con_el_llavero_sin_contestar_no_se_crea_otra_llave() {
        let c = carpeta("sin-llavero");
        let llaves = EnMemoria { no_contesta: true, ..EnMemoria::default() };
        let e = c.guardar(&llaves, &contenido("x"), "reunion", 0).unwrap_err();
        assert!(e.contains("no contestó"), "{e}");
        assert_eq!(*llaves.creadas.borrow(), 0, "se creó una llave encima de la que había: tus reuniones, perdidas");
        assert!(c.lista().is_empty());
    }

    #[test]
    fn sin_llave_no_se_abre_ni_se_inventa_una() {
        let c = carpeta("sin-llave");
        let g = c.guardar(&EnMemoria::default(), &contenido("x"), "reunion", 0).unwrap();
        let otro_mac = EnMemoria::default();
        assert!(c.abrir(&otro_mac, &g.archivo).unwrap_err().contains("no hay llave"));
        assert_eq!(*otro_mac.creadas.borrow(), 0, "abrir no crea llaves");
        let _ = std::fs::remove_dir_all(c.raiz().parent().unwrap());
    }

    #[test]
    fn la_lista_no_necesita_la_llave_y_el_barrido_borra_solo_lo_vencido() {
        let c = carpeta("barrido");
        let llaves = EnMemoria::default();
        let ahora = ahora();
        let vencida = c.guardar(&llaves, &contenido("a"), "vencida", ahora - 1).unwrap();
        let futura = c.guardar(&llaves, &contenido("b"), "futura", ahora + 3_600).unwrap();
        let siempre = c.guardar(&llaves, &contenido("c"), "siempre", 0).unwrap();
        std::fs::write(c.raiz().join("otra-cosa.ghost"), b"no es de notas").unwrap();
        std::fs::write(c.raiz().join("apuntes.txt"), b"de otro").unwrap();

        let lista = c.lista();
        assert_eq!(lista.len(), 3, "{lista:?}");
        assert_eq!(c.barrer(ahora), 1);
        let quedan: Vec<String> = c.lista().into_iter().map(|r| r.archivo).collect();
        assert!(!quedan.contains(&vencida.archivo), "lo vencido sigue ahí");
        assert!(quedan.contains(&futura.archivo) && quedan.contains(&siempre.archivo));
        assert!(c.raiz().join("otra-cosa.ghost").exists(), "el barrido borró algo que no entiende");
        let _ = std::fs::remove_dir_all(c.raiz().parent().unwrap());
    }

    #[test]
    fn lo_que_llega_de_la_pantalla_no_sale_de_la_carpeta() {
        let c = carpeta("fuera");
        for malo in ["../preferencias.json", "/etc/passwd.ghost", "..ghost", ".ghost", "a/b.ghost", "-x.ghost", "Mayus.ghost"] {
            assert!(!nombre_valido(malo), "«{malo}» pasó por nombre de reunión");
            assert!(c.borrar(malo).is_err());
        }
        assert!(nombre_valido("paramo-azul-2026-09-20-2.ghost"));
    }

    fn guardada(regla: crate::propuestas::Regla, de: crate::propuestas::De, texto: &str) -> crate::propuestas::Propuesta {
        crate::propuestas::Propuesta { regla, de, texto: texto.into(), ficha: None, seccion: None, hora: "14:16".into() }
    }

    /// **Exportar quita el cifrado, nace 600 y lleva las propuestas que guardaste** (auditoría del S3,
    /// M6: el ADR 016 §3 las mete en el archivo y exportar las dejaba fuera). Del cliente, el hecho con
    /// su plantilla, jamás su turno. Demostrado en rojo con el código de antes: sin la sección.
    #[test]
    fn exportar_quita_el_cifrado_y_nace_600() {
        use crate::propuestas::{De, Regla};
        let c = carpeta("exportar");
        let llaves = EnMemoria::default();
        let mut con_propuestas = contenido("Piden la cuarta fuente.");
        con_propuestas.propuestas = vec![
            guardada(Regla::Compromiso, De::Tuyo, "Te lo mando el viernes con el detalle"),
            guardada(Regla::Cifra, De::Cliente, "12 semanas desde la firma"),
            crate::propuestas::Propuesta { ficha: Some("tres".into()), seccion: Some("§3.2 Alcance".into()), ..guardada(Regla::Choque, De::Cliente, "cuatro fuentes") },
            guardada(Regla::Nombre, De::Cliente, "Andrea Villalba"),
            guardada(Regla::Pregunta, De::Cliente, "limpieza · alcance"),
        ];
        let g = c.guardar(&llaves, &con_propuestas, "paramo-azul-2026-09-20", 0).unwrap();
        let destino = c.raiz().parent().unwrap().join("exportada.md");
        c.exportar(&llaves, &g.archivo, &destino, "es").unwrap();
        let texto = std::fs::read_to_string(&destino).unwrap();
        assert!(texto.starts_with("# Páramo Azul · 2026-09-20 14:02 · 47 min"), "{texto}");
        assert!(texto.contains("## Tu nota\n\nPiden la cuarta fuente."));
        assert!(texto.contains("- Cuarta fuente: cotización aparte"));
        assert!(texto.contains("- Limpieza de datos: hasta tres fuentes — Propuesta Páramo Azul · §3.2"));
        assert!(texto.contains("## Propuestas que guardaste\n\n"), "exportar deja fuera las propuestas guardadas:\n{texto}");
        for linea in [
            "- 14:16 · Te lo mando el viernes con el detalle",
            "- 14:16 · Dijeron «12 semanas desde la firma»",
            "- 14:16 · Dijeron «cuatro fuentes»; tu ficha fijada dice «tres» (§3.2 Alcance)",
            "- 14:16 · Mencionaron a «Andrea Villalba», que no está en tu corpus",
            "- 14:16 · Te preguntaron por: limpieza · alcance",
        ] {
            assert!(texto.contains(linea), "falta «{linea}»:\n{texto}");
        }
        let en = a_texto(&con_propuestas, "en");
        assert!(en.contains("## Agreements"));
        assert!(en.contains("## Suggestions you saved\n\n"), "{en}");
        for linea in [
            "- 14:16 · They said “12 semanas desde la firma”",
            "- 14:16 · They said “cuatro fuentes”; your pinned card says “tres” (§3.2 Alcance)",
            "- 14:16 · They mentioned “Andrea Villalba”, who is not in your corpus",
            "- 14:16 · They asked about: limpieza · alcance",
        ] {
            assert!(en.contains(linea), "falta «{linea}»:\n{en}");
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&destino).unwrap().permissions().mode() & 0o777, 0o600);
        }
        let _ = std::fs::remove_dir_all(c.raiz().parent().unwrap());
    }

    /// **Exportar no toca la carpeta que eliges** (auditoría del S3, M5): es tuya, no de la app. Con
    /// `almacen::escribir` quedaba en 700 —la carpeta compartida de tu equipo dejaba de serlo— y en una
    /// carpeta que no es tuya el `chmod` fallaba y la exportación entera también. Demostrado en rojo
    /// con el código de antes: 755 → 700.
    #[cfg(unix)]
    #[test]
    fn exportar_no_toca_la_carpeta_de_destino() {
        use std::os::unix::fs::PermissionsExt;
        let c = carpeta("destino");
        let llaves = EnMemoria::default();
        let g = c.guardar(&llaves, &contenido("x"), "paramo-azul-2026-09-20", 0).unwrap();
        let tuya = c.raiz().parent().unwrap().join("compartida");
        std::fs::create_dir_all(&tuya).unwrap();
        std::fs::set_permissions(&tuya, std::fs::Permissions::from_mode(0o755)).unwrap();
        c.exportar(&llaves, &g.archivo, &tuya.join("reunion.md"), "es").unwrap();
        let modo = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(modo(&tuya), 0o755, "exportar cambió los permisos de tu carpeta");
        assert_eq!(modo(&tuya.join("reunion.md")), 0o600, "el archivo exportado no nació 600");
        let _ = std::fs::remove_dir_all(c.raiz().parent().unwrap());
    }

    #[test]
    fn abrir_una_reunion_tarda_menos_de_medio_segundo() {
        // El presupuesto del plan (≤ 500 ms), con una reunión grande: una nota de 100 000 letras.
        let c = carpeta("rapido");
        let llaves = EnMemoria::default();
        let g = c.guardar(&llaves, &contenido(&"Una frase de la nota. ".repeat(4_500)), "grande", 0).unwrap();
        let t = std::time::Instant::now();
        c.abrir(&llaves, &g.archivo).unwrap();
        let ms = t.elapsed().as_millis();
        println!("[notas] abrir {} bytes: {ms} ms", g.bytes);
        assert!(ms <= 500, "abrir tardó {ms} ms");
        let _ = std::fs::remove_dir_all(c.raiz().parent().unwrap());
    }

    #[test]
    fn el_log_dice_cuanto_y_nunca_que() {
        let mut cu = Cuaderno::nuevo(false);
        cu.escribir("Páramo Azul pide la cuarta fuente");
        let g = Guardada { archivo: "paramo-azul-2026-09-20.ghost".into(), bytes: 2_048, vence: 1 };
        let linea = linea_de_log(&cu.resumen(), &g);
        for prohibido in ["paramo", "Páramo", "cuarta", ".ghost", "/"] {
            assert!(!linea.contains(prohibido), "el log lleva «{prohibido}»: {linea}");
        }
        assert!(linea.contains("1 párrafos") && linea.contains("2048 bytes"), "{linea}");
    }
}
