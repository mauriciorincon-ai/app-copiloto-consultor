//! LOS TRES LECTORES — de un archivo del usuario a líneas marcadas.
//!
//! Markdown, .docx y PDF. Cada uno sabe reconocer sus propios títulos, y ahí termina su trabajo:
//! el troceado vive en `seccion.rs` y es el mismo para los tres.
//!
//! **Los documentos no se copian.** Se leen donde están y no se escribe nada al lado — la
//! maqueta de corpus lo promete con esas palabras (*«no se copian: se leen donde están»*) y es
//! la clase de promesa que se rompe sin querer al añadir una cache.
//!
//! **Un documento que rompa el lector no puede detener a los demás.** La maqueta lo dice
//! literalmente: *«un documento ilegible no detiene a los otros 142»*. Como la librería de PDF
//! puede entrar en pánico con un archivo mal formado —y los PDF del mundo real lo están—, su
//! llamada va dentro de `catch_unwind`: el documento se marca ilegible y la indexación sigue.
//!
//! **Y esa promesa dependía de una línea del manifiesto que la anulaba.** `panic = "abort"` en el
//! perfil de release —lo traía la plantilla de Tauri— hace que un `panic!` no se desenrede: mata el
//! proceso. El test de esta promesa corre en debug, así que estuvo verde todo el sprint mientras en
//! el binario que se distribuye **un PDF roto cerraba la app**. Hallazgo A3 de la auditoría del
//! sprint 001. Hoy el perfil desenreda, lo vigila `el_perfil_de_release_desenreda` en este archivo,
//! y la promesa se comprobó corriendo la suite del corpus en release.

use std::path::Path;

use super::seccion::Linea;

/// Un PDF escaneado devuelve texto vacío o casi. Debajo de esto se declara ilegible, que es lo
/// que la maqueta pinta en ámbar («sin leer»), en vez de indexar un documento fantasma que luego
/// jamás aparece en ninguna búsqueda y nadie sabe por qué.
const MINIMO_LEGIBLE: usize = 40;

/// Una línea más larga que esto no es un título aunque lo parezca.
const TITULO_MAXIMO: usize = 70;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Leido {
    pub lineas: Vec<Linea>,
    /// `true` cuando los títulos son una **conjetura** por la forma del texto y no una marca del
    /// formato. Es el caso del PDF, siempre. La pantalla lo dice; la ficha no promete sección.
    pub conjeturado: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fallo {
    /// El archivo no se pudo abrir.
    NoSeAbre(String),
    /// Se abrió pero no soltó texto: un PDF escaneado, un .docx vacío.
    SinTexto,
    /// La extensión no es de las tres que la app lee.
    FormatoAjeno(String),
    /// La librería del formato se rompió con este archivo.
    Roto(String),
}

impl Fallo {
    /// El motivo, en español llano, para la fila de la pantalla de corpus. La app nunca enseña
    /// un error de librería: enseña qué pasó con el documento.
    pub fn motivo(&self) -> String {
        match self {
            Fallo::NoSeAbre(_) => "no se pudo abrir".into(),
            Fallo::SinTexto => "sin texto legible (¿escaneado?)".into(),
            Fallo::FormatoAjeno(e) => format!("no se lee «.{e}»"),
            Fallo::Roto(_) => "el archivo está dañado".into(),
        }
    }
}

/// ¿Es un archivo que la app sabe leer? Se usa al recorrer la carpeta, para no abrir de más.
pub fn se_lee(ruta: &Path) -> bool {
    matches!(extension(ruta).as_str(), "md" | "markdown" | "txt" | "docx" | "pdf")
}

fn extension(ruta: &Path) -> String {
    ruta.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase()
}

pub fn leer(ruta: &Path) -> Result<Leido, Fallo> {
    let leido = match extension(ruta).as_str() {
        "md" | "markdown" | "txt" => de_markdown(&texto_plano(ruta)?),
        "docx" => de_docx(ruta)?,
        "pdf" => de_pdf(ruta)?,
        otra => return Err(Fallo::FormatoAjeno(otra.to_string())),
    };
    let cuanto: usize = leido.lineas.iter().map(|l| l.texto.trim().chars().count()).sum();
    if cuanto < MINIMO_LEGIBLE {
        return Err(Fallo::SinTexto);
    }
    Ok(leido)
}

fn texto_plano(ruta: &Path) -> Result<String, Fallo> {
    std::fs::read(ruta)
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .map_err(|e| Fallo::NoSeAbre(e.to_string()))
}

// ---------------------------------------------------------------- Markdown

/// Markdown es el único de los tres donde el título es **explícito y del autor**. `#` y los
/// subrayados de la forma antigua (`===`, `---`).
pub fn de_markdown(texto: &str) -> Leido {
    let crudas: Vec<&str> = texto.lines().collect();
    let mut lineas = Vec::new();
    let mut i = 0;
    while i < crudas.len() {
        let l = crudas[i].trim();
        let siguiente = crudas.get(i + 1).map(|s| s.trim()).unwrap_or("");
        let subrayada = !l.is_empty()
            && siguiente.len() >= 3
            && (siguiente.chars().all(|c| c == '=') || siguiente.chars().all(|c| c == '-'));

        if let Some(resto) = l.strip_prefix('#') {
            lineas.push(Linea::titulo(resto.trim_start_matches('#').trim()));
        } else if subrayada {
            lineas.push(Linea::titulo(l));
            i += 1; // el subrayado no es contenido
        } else if !l.is_empty() {
            lineas.push(Linea::cuerpo(l));
        }
        i += 1;
    }
    Leido { lineas, conjeturado: false }
}

// ---------------------------------------------------------------- .docx

/// Un .docx es un zip con `word/document.xml` dentro. Se lee con las piezas —zip y un lector de
/// XML— y no con una librería de Word: menos superficie, y ningún escritor de .docx enlazado en
/// una app que jamás escribe .docx.
///
/// Reconoce títulos por **dos** caminos, y hacen falta los dos: `w:pStyle` (lo que escribe Word
/// de verdad) y **negrita con cuerpo mayor que el del texto** (lo que escriben los conversores;
/// se comprobó con uno del propio macOS, que no pone un solo `pStyle`).
fn de_docx(ruta: &Path) -> Result<Leido, Fallo> {
    use quick_xml::events::Event;
    use std::io::Read;

    let f = std::fs::File::open(ruta).map_err(|e| Fallo::NoSeAbre(e.to_string()))?;
    let mut zip = zip::ZipArchive::new(f).map_err(|e| Fallo::Roto(e.to_string()))?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .map_err(|e| Fallo::Roto(e.to_string()))?
        .read_to_string(&mut xml)
        .map_err(|e| Fallo::Roto(e.to_string()))?;

    let mut lector = quick_xml::Reader::from_str(&xml);
    lector.config_mut().trim_text(false);

    let mut lineas = Vec::new();
    let mut texto = String::new();
    let mut es_titulo = false;
    let mut negrita = false;
    let mut cuerpo_de_letra: Option<u32> = None;
    let mut dentro_de_w_t = false;
    let mut cuerpos: Vec<u32> = Vec::new();
    let mut crudas: Vec<(String, bool, Option<u32>, bool)> = Vec::new();

    loop {
        match lector.read_event() {
            Ok(Event::Start(e)) | Ok(Event::Empty(e)) => {
                let nombre = e.name();
                let local = nombre.local_name();
                match local.into_inner() {
                    "p" => {
                        texto.clear();
                        es_titulo = false;
                        negrita = false;
                        cuerpo_de_letra = None;
                    }
                    "pStyle" | "outlineLvl" => {
                        if let Some(v) = valor(&e) {
                            let v = v.to_lowercase();
                            if local.into_inner() == "outlineLvl" || v.starts_with("heading") || v.starts_with("titulo") || v.starts_with("ttulo") || v.starts_with("title") {
                                es_titulo = true;
                            }
                        }
                    }
                    "b" => negrita = valor(&e).as_deref() != Some("0"),
                    "sz" => {
                        if let Some(n) = valor(&e).and_then(|v| v.parse::<u32>().ok()) {
                            cuerpo_de_letra = Some(n);
                            cuerpos.push(n);
                        }
                    }
                    "t" => dentro_de_w_t = true,
                    _ => {}
                }
            }
            Ok(Event::Text(t)) if dentro_de_w_t => {
                texto.push_str(&t);
            }
            Ok(Event::End(e)) => match e.name().local_name().into_inner() {
                "t" => dentro_de_w_t = false,
                "p" => {
                    let t = texto.trim().to_string();
                    if !t.is_empty() {
                        crudas.push((t, es_titulo, cuerpo_de_letra, negrita));
                    }
                }
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(Fallo::Roto(e.to_string())),
            _ => {}
        }
    }

    // El cuerpo de letra más repetido es el del texto normal; lo que sea negrita y más grande que
    // eso es un título, aunque el conversor no haya escrito un solo `pStyle`.
    let normal = mas_repetido(&cuerpos);
    for (t, marcado, cuerpo, negrita) in crudas {
        let por_forma = negrita && matches!((cuerpo, normal), (Some(c), Some(n)) if c > n);
        lineas.push(if marcado || por_forma { Linea::titulo(t) } else { Linea::cuerpo(t) });
    }
    Ok(Leido { lineas, conjeturado: false })
}

fn valor(e: &quick_xml::events::BytesStart) -> Option<String> {
    e.attributes()
        .flatten()
        .find(|a| a.key.local_name().into_inner() == "val")
        .map(|a| a.value.into_owned())
}

fn mas_repetido(ns: &[u32]) -> Option<u32> {
    let mut cuenta: Vec<(u32, usize)> = Vec::new();
    for n in ns {
        match cuenta.iter_mut().find(|(v, _)| v == n) {
            Some((_, c)) => *c += 1,
            None => cuenta.push((*n, 1)),
        }
    }
    cuenta.into_iter().max_by_key(|(_, c)| *c).map(|(v, _)| v)
}

// ---------------------------------------------------------------- PDF

/// Un PDF no trae títulos: trae líneas con coordenadas. Lo que sale de la extracción es texto
/// plano, así que los títulos aquí son **una conjetura por la forma de la línea** — y el `Leido`
/// lo marca para que la pantalla no prometa lo que no sabe.
fn de_pdf(ruta: &Path) -> Result<Leido, Fallo> {
    let ruta = ruta.to_path_buf();
    let texto = std::panic::catch_unwind(move || pdf_extract::extract_text(&ruta))
        .map_err(|_| Fallo::Roto("la librería de PDF se rompió con este archivo".into()))?
        .map_err(|e| Fallo::Roto(e.to_string()))?;
    Ok(conjeturar_titulos(&texto))
}

/// La conjetura, escrita para poder discutirla: una línea es título si es **corta**, **no
/// termina como una frase**, tiene alguna letra, y **debajo lleva cuerpo**. Lo último es lo que
/// evita convertir en títulos las líneas de una lista o de una tabla.
pub fn conjeturar_titulos(texto: &str) -> Leido {
    let crudas: Vec<&str> = texto.lines().map(|l| l.trim_end()).collect();
    let parece = |l: &str| -> bool {
        let l = l.trim();
        !l.is_empty()
            && l.chars().count() <= TITULO_MAXIMO
            && !l.ends_with(['.', ',', ';', ':'])
            && l.chars().any(|c| c.is_alphabetic())
            && !l.starts_with(['-', '•', '*'])
    };

    let mut lineas = Vec::new();
    for (i, l) in crudas.iter().enumerate() {
        let l = l.trim();
        if l.is_empty() {
            continue;
        }
        let debajo_hay_cuerpo = crudas[i + 1..]
            .iter()
            .map(|s| s.trim())
            .find(|s| !s.is_empty())
            .is_some_and(|s| !parece(s));
        lineas.push(if parece(l) && debajo_hay_cuerpo {
            Linea::titulo(l)
        } else {
            Linea::cuerpo(l)
        });
    }
    Leido { lineas, conjeturado: true }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **El `catch_unwind` de `de_pdf` solo existe si el perfil de release DESENREDA.**
    ///
    /// Con `panic = "abort"` —que es lo que traía la plantilla de Tauri— un `panic!` no se
    /// desenreda: mata el proceso. Así que el módulo prometía que «un documento dañado no detiene a
    /// los otros», el test de la carpeta entera lo comprobaba **en debug**, y en el binario que se
    /// distribuye un solo PDF roto de la carpeta del usuario cerraba la app. Un gate que no puede
    /// fallar en el modo en que el usuario usa el programa no está probando ese modo.
    ///
    /// Esta comprobación es estática a propósito: correr la suite en release costaría otra
    /// compilación entera con LTO en cada PR para vigilar una línea de un archivo de configuración.
    /// Lo que hay que impedir es que esa línea vuelva, y eso se lee.
    ///
    /// Se ve en rojo devolviendo `panic = "abort"` a `[profile.release]` de `src-tauri/Cargo.toml`.
    #[test]
    fn el_perfil_de_release_desenreda() {
        let manifiesto =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml"))
                .expect("no se pudo leer el manifiesto");
        let release = manifiesto
            .split("[profile.release]")
            .nth(1)
            .expect("el manifiesto ya no tiene perfil de release: revisa este test");
        let release = release.split("\n[").next().unwrap_or(release);
        for linea in release.lines() {
            let l = linea.trim();
            assert!(
                l.starts_with('#') || !(l.starts_with("panic") && l.contains("abort")),
                "`panic = \"abort\"` volvió al perfil de release: con él, el `catch_unwind` de \
                 `de_pdf` no protege nada y un PDF roto de la carpeta del usuario cierra la app"
            );
        }
    }

    #[test]
    fn el_markdown_reconoce_las_dos_formas_de_titulo() {
        let l = de_markdown("# Alcance\nTres canales.\n\nPrecio\n======\nCerrado.\n");
        assert_eq!(l.lineas, vec![
            Linea::titulo("Alcance"),
            Linea::cuerpo("Tres canales."),
            Linea::titulo("Precio"),
            Linea::cuerpo("Cerrado."),
        ]);
        assert!(!l.conjeturado, "en markdown el título lo puso el autor, no se conjetura");
    }

    #[test]
    fn el_subrayado_no_se_cuela_como_contenido() {
        let l = de_markdown("Precio\n------\nCerrado.\n");
        assert!(!l.lineas.iter().any(|x| x.texto.contains("---")));
    }

    /// La conjetura del PDF, con sus dos filos: reconoce el título y NO convierte en título la
    /// última línea del documento (no tiene cuerpo debajo) ni las líneas de una lista.
    #[test]
    fn la_conjetura_del_pdf_pide_cuerpo_debajo() {
        let l = conjeturar_titulos("Alcance\nTres canales y una línea base.\nPrecio cerrado\n");
        assert_eq!(l.lineas[0], Linea::titulo("Alcance"));
        assert_eq!(l.lineas[1], Linea::cuerpo("Tres canales y una línea base."));
        assert_eq!(
            l.lineas[2],
            Linea::cuerpo("Precio cerrado"),
            "la última línea se tomó por título sin tener nada debajo"
        );
        assert!(l.conjeturado, "el PDF tiene que declarar que sus títulos son conjetura");
    }

    #[test]
    fn una_lista_no_es_una_pila_de_titulos() {
        let l = conjeturar_titulos("Entregables\n- Informe\n- Taller\nSe entregan en marzo.\n");
        let titulos: Vec<_> = l.lineas.iter().filter(|x| x.titulo).map(|x| &x.texto).collect();
        assert_eq!(titulos, vec!["Entregables"]);
    }

    #[test]
    fn se_lee_dice_que_si_a_las_tres_y_que_no_al_resto() {
        assert!(se_lee(Path::new("a.md")));
        assert!(se_lee(Path::new("a.DOCX")));
        assert!(se_lee(Path::new("a.pdf")));
        assert!(!se_lee(Path::new("a.key")));
        assert!(!se_lee(Path::new("a.xlsx")));
    }

    #[test]
    fn un_archivo_sin_texto_se_declara_ilegible_en_vez_de_indexarse_vacio() {
        let d = std::env::temp_dir().join("ag-corpus-vacio.md");
        std::fs::write(&d, "ok\n").unwrap();
        assert_eq!(leer(&d), Err(Fallo::SinTexto));
        let _ = std::fs::remove_file(&d);
    }

    #[test]
    fn el_motivo_se_dice_en_espanol_llano_y_no_con_el_error_de_la_libreria() {
        assert_eq!(Fallo::SinTexto.motivo(), "sin texto legible (¿escaneado?)");
        assert_eq!(Fallo::FormatoAjeno("key".into()).motivo(), "no se lee «.key»");
        assert!(!Fallo::Roto("EOF while parsing".into()).motivo().contains("EOF"));
    }
}
