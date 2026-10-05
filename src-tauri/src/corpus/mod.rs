//! Corpus del consultor — módulo **NO protegido**, y con razón.
//!
//! Aquí SÍ se abre disco: el corpus son los documentos PROPIOS del usuario, que se leen donde
//! están, y su índice vive en la carpeta de la app. Eso es exactamente lo que la regla del
//! efímero permite persistir. Por eso `corpus/` queda fuera de la lista de módulos protegidos
//! de `verify:ephemeral`: si estuviera dentro, el gate se volvería imposible de cumplir y la
//! tentación sería aflojarlo — y un gate aflojado deja de proteger lo que sí importa.
//!
//! La frontera es la del estándar 4-T: **lo del usuario** puede persistir; **lo de terceros**,
//! no. Nada de lo que entra aquí viene de la reunión.
//!
//! Se llena en la fase 4 del sprint 001 (ingesta → chunking por sección → BM25 con tantivy).

pub mod consulta;
pub mod evaluar;
pub mod indice;
pub mod leer;
pub mod seccion;
pub mod unidad;

use std::path::{Path, PathBuf};

use serde::Serialize;

pub use indice::{Hallazgo, Indice};
pub use unidad::Unidad;

/// Techo de documentos por carpeta. No es una limitación técnica: es un aviso. Quien apunta a su
/// carpeta de Descargas entera espera que la app se lo diga en vez de pasarse veinte minutos
/// leyendo facturas.
pub const TECHO_DE_DOCUMENTOS: usize = 2_000;

/// Hasta dónde baja por las subcarpetas.
const HONDURA: usize = 6;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "estado")]
pub enum Estado {
    Indexado { secciones: usize },
    /// La maqueta lo pinta en ámbar: el documento está ahí y la app dice por qué no lo leyó.
    SinLeer { motivo: String },
}

/// **El cliente de una ficha de cliente**, tal como el usuario lo escribió en el nombre del
/// archivo: lo que va tras el último «·» («Ficha de cliente · Páramo Azul» → «Páramo Azul»), o el
/// nombre entero si no lleva «·». Es el nombre que se dice en una reunión; el del archivo, no. Lo
/// usan la bóveda del API (lo tapa) y el diccionario (lo escribe bien) — auditoría del S2, A2 y A3.
pub fn cliente_de(nombre_del_archivo: &str) -> String {
    nombre_del_archivo.rsplit('·').next().unwrap_or(nombre_del_archivo).trim().to_string()
}

/// **Los clientes del corpus, por su nombre**: el de cada ficha de cliente que parezca un nombre.
/// Es lo único que la sesión le pasa a la bóveda del API y al diccionario (auditoría del S2, A2 y
/// A3), y lo que usa el kit del WER para medir la misma configuración que el usuario.
pub fn clientes(documentos: &[Documento]) -> Vec<String> {
    documentos
        .iter()
        .filter(|d| d.unidad == Some(Unidad::Cliente))
        .map(|d| cliente_de(&d.nombre))
        .filter(|n| parece_un_nombre_de_cliente(n))
        .collect()
}

/// Un documento que el corpus clasifica como ficha de cliente no siempre se llama como el cliente:
/// «Seguridad y manejo de datos del cliente» es de esa unidad y no es el nombre de nadie. Un nombre
/// de cliente es corto —hasta cuatro palabras— y no dice «cliente».
pub fn parece_un_nombre_de_cliente(n: &str) -> bool {
    let palabras = n.split_whitespace().count();
    let l = n.to_lowercase();
    (1..=4).contains(&palabras) && !l.contains("cliente") && !l.contains("client")
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Documento {
    pub ruta: String,
    pub nombre: String,
    pub unidad: Option<Unidad>,
    /// Los títulos de sus secciones eran una conjetura por la forma del texto (PDF).
    pub conjeturado: bool,
    #[serde(flatten)]
    pub estado: Estado,
    /// **La línea «Jurisdicción:» de una ficha de cliente** (ADR 017 §2), tal como la escribiste. Solo
    /// en memoria y solo para las fichas de cliente: no viaja a la pantalla del corpus ni al índice.
    #[serde(skip)]
    pub jurisdiccion: Option<String>,
    /// **El texto de una propuesta, plegado** (sin mayúsculas ni tildes), para saber de qué cliente es sin
    /// releerla (auditoría del S4, M9): «Preparar» buscaba el cliente releyendo cada PDF del disco, en el
    /// hilo principal y con el candado del corpus puesto. Solo memoria, solo propuestas, como el índice.
    #[serde(skip)]
    pub texto_plegado: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PorUnidad {
    pub unidad: String,
    pub documentos: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDelCorpus {
    /// `None` mientras el usuario no haya señalado carpeta: es el estado «vacío» de la maqueta.
    pub carpeta: Option<String>,
    pub documentos: usize,
    pub secciones: usize,
    pub por_unidad: Vec<PorUnidad>,
    pub sin_unidad: usize,
    pub ilegibles: usize,
    /// Documentos cuyas secciones se **conjeturaron** por la forma del texto (PDF), en vez de
    /// venir escritas. Se cuenta para que el usuario pueda juzgar cómo se leyó su corpus.
    pub conjeturados: usize,
    /// Dónde vive el índice, en claro, porque la pantalla de corpus lo enseña.
    pub donde_vive: Option<String>,
    pub bytes_del_indice: u64,
}

pub struct Corpus {
    indice: Indice,
    documentos: Vec<Documento>,
    carpeta: Option<PathBuf>,
    /// Las palabras distintivas del corpus, para el motivo «término tuyo» del disparador. Se
    /// calculan al indexar y no en cada turno: la escucha las pide veinticinco veces por segundo.
    vocabulario: Vec<String>,
}

impl Corpus {
    pub fn en(carpeta_del_indice: &Path) -> Result<Self, String> {
        Ok(Corpus {
            indice: Indice::en(carpeta_del_indice)?,
            documentos: Vec::new(),
            carpeta: None,
            vocabulario: Vec::new(),
        })
    }

    pub fn en_memoria() -> Result<Self, String> {
        Ok(Corpus {
            indice: Indice::en_memoria()?,
            documentos: Vec::new(),
            carpeta: None,
            vocabulario: Vec::new(),
        })
    }

    /// Indexa una carpeta entera. `avisar` recibe cada documento en cuanto se resuelve, para que
    /// la pantalla enseñe progreso de verdad y no una barra que se inventa el porcentaje.
    ///
    /// **Un documento que falla no detiene a los demás** — la maqueta lo promete con esas
    /// palabras. Cada fallo se queda en su propia fila, con su motivo en español llano.
    pub fn indexar(&mut self, carpeta: &Path, avisar: &dyn Fn(&Documento)) -> Result<usize, String> {
        if !carpeta.is_dir() {
            return Err("eso no es una carpeta".into());
        }
        self.indice.vaciar()?;
        self.documentos.clear();
        self.vocabulario.clear();
        self.carpeta = Some(carpeta.to_path_buf());
        let mut titulos: Vec<String> = Vec::new();

        for ruta in recorrer(carpeta, HONDURA) {
            if self.documentos.len() >= TECHO_DE_DOCUMENTOS {
                println!(
                    "[corpus] la carpeta trae más de {TECHO_DE_DOCUMENTOS} documentos legibles; se indexaron los primeros"
                );
                break;
            }
            let (doc, suyos) = self.indexar_uno(&ruta);
            titulos.extend(suyos);
            avisar(&doc);
            self.documentos.push(doc);
        }
        let nombres: Vec<String> = self.documentos.iter().map(|d| d.nombre.clone()).collect();
        self.vocabulario = crate::disparo::vocabulario(&nombres, &titulos);
        Ok(self.documentos.len())
    }

    /// Devuelve el documento y los títulos de sus secciones (que alimentan el vocabulario).
    fn indexar_uno(&self, ruta: &Path) -> (Documento, Vec<String>) {
        let nombre = ruta.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let como_texto = ruta.to_string_lossy().to_string();

        let leido = match leer::leer(ruta) {
            Ok(l) => l,
            Err(e) => {
                return (
                    Documento {
                        ruta: como_texto,
                        nombre,
                        unidad: None,
                        conjeturado: false,
                        estado: Estado::SinLeer { motivo: e.motivo() },
                        jurisdiccion: None,
                        texto_plegado: None,
                    },
                    Vec::new(),
                )
            }
        };

        let secciones = seccion::trocear(&leido.lineas);
        let plano: String = secciones.iter().map(|s| s.texto.as_str()).collect::<Vec<_>>().join(" ");
        let unidad = Unidad::clasificar(&nombre, &plano);
        let jurisdiccion = if unidad == Some(Unidad::Cliente) {
            leido.lineas.iter().find_map(|l| crate::jurisdiccion::de_la_linea(&l.texto))
        } else {
            None
        };

        let titulos: Vec<String> = secciones.iter().filter_map(|s| s.titulo.clone()).collect();
        // Línea a línea, como se buscaba antes en el disco: un nombre no se encuentra partido entre dos.
        let texto_plegado = (unidad == Some(Unidad::Propuesta))
            .then(|| leido.lineas.iter().map(|l| crate::propuestas::plegar(&l.texto)).collect::<Vec<_>>().join("\n"));
        match self.indice.meter(&como_texto, &nombre, unidad, leido.conjeturado, &secciones) {
            Ok(n) => (
                Documento {
                    ruta: como_texto,
                    nombre,
                    unidad,
                    conjeturado: leido.conjeturado,
                    estado: Estado::Indexado { secciones: n },
                    jurisdiccion,
                    texto_plegado,
                },
                titulos,
            ),
            Err(e) => (
                Documento {
                    ruta: como_texto,
                    nombre,
                    unidad,
                    conjeturado: leido.conjeturado,
                    estado: Estado::SinLeer { motivo: format!("no se pudo indexar: {e}") },
                    jurisdiccion,
                    texto_plegado: None,
                },
                Vec::new(),
            ),
        }
    }

    /// El índice, para las pruebas que necesitan meter secciones a mano sin pasar por archivos.
    #[cfg(test)]
    pub fn indice_para_pruebas(&self) -> &Indice {
        &self.indice
    }

    /// Las palabras distintivas del corpus. Vacío mientras no haya carpeta señalada.
    pub fn vocabulario(&self) -> &[String] {
        &self.vocabulario
    }

    /// ¿Está este nombre en tu corpus? En el nombre de un documento o en su texto (ADR 016, regla
    /// `nombre`).
    pub fn conoce(&self, nombre: &str) -> bool {
        let buscado = crate::propuestas::plegar(nombre);
        self.documentos.iter().any(|d| crate::propuestas::plegar(&d.nombre).contains(&buscado)) || self.indice.conoce(nombre)
    }

    pub fn buscar(&self, texto: &str, cuantos: usize) -> Result<Vec<Hallazgo>, String> {
        self.indice.buscar(texto, cuantos)
    }

    /// La búsqueda con **lo que hay en la pantalla** como contexto. Ver [`Indice::buscar_con_pantalla`].
    pub fn buscar_con_pantalla(
        &self,
        texto: &str,
        pantalla: &str,
        cuantos: usize,
    ) -> Result<Vec<Hallazgo>, String> {
        self.indice.buscar_con_pantalla(texto, pantalla, cuantos)
    }

    /// Lo que dice la línea «Jurisdicción:» de la ficha de `cliente` (su nombre, como lo da
    /// [`clientes`]). `None` si no hay ficha con ese nombre o no lo dice.
    pub fn jurisdiccion_de(&self, cliente: &str) -> Option<&str> {
        self.documentos
            .iter()
            .filter(|d| d.unidad == Some(Unidad::Cliente))
            .find(|d| cliente_de(&d.nombre) == cliente)
            .and_then(|d| d.jurisdiccion.as_deref())
    }

    pub fn documentos(&self) -> &[Documento] {
        &self.documentos
    }

    /// **Las secciones de un documento tuyo, enteras y en orden** (sprint 004, el banco del ensayo). El
    /// índice guarda trozos para buscar; el banco necesita cada sección con su título. Se relee de donde
    /// está, con el mismo lector y el mismo troceado que al indexar, y los trozos seguidos de un mismo
    /// título vuelven a ser uno. Solo documentos de tu corpus: una ruta cualquiera no se lee.
    pub fn secciones_de(&self, ruta: &str) -> Result<Vec<seccion::Seccion>, String> {
        if !self.documentos.iter().any(|d| d.ruta == ruta) {
            return Err("ese documento no está en tu corpus".into());
        }
        let leido = leer::leer(Path::new(ruta)).map_err(|e| e.motivo())?;
        let mut juntas: Vec<seccion::Seccion> = Vec::new();
        for s in seccion::trocear(&leido.lineas) {
            match juntas.last_mut() {
                Some(previa) if previa.titulo == s.titulo => {
                    previa.texto.push('\n');
                    previa.texto.push_str(&s.texto);
                }
                _ => juntas.push(s),
            }
        }
        Ok(juntas)
    }

    /// **Las propuestas de un cliente** (sprint 004): los documentos de la unidad «propuesta» que lo
    /// nombran, en el nombre del archivo o en su texto —como [`Corpus::conoce`]—. Ningún código une una
    /// propuesta con su cliente de otra forma; si hay varias, eliges tú, y si no hay ninguna, el ensayo
    /// lo dice («sin corpus para este cliente»).
    pub fn propuestas_de(&self, cliente: &str) -> Vec<&Documento> {
        let buscado = crate::propuestas::plegar(cliente);
        if buscado.trim().is_empty() {
            return Vec::new();
        }
        self.documentos
            .iter()
            .filter(|d| d.unidad == Some(Unidad::Propuesta) && matches!(d.estado, Estado::Indexado { .. }))
            .filter(|d| {
                crate::propuestas::plegar(&d.nombre).contains(&buscado)
                    || d.texto_plegado.as_deref().is_some_and(|t| t.contains(&buscado))
            })
            .collect()
    }

    /// **La ficha de un cliente**, por su nombre (el que da [`clientes`]).
    pub fn ficha_de(&self, cliente: &str) -> Option<&Documento> {
        self.documentos
            .iter()
            .filter(|d| d.unidad == Some(Unidad::Cliente))
            .find(|d| cliente_de(&d.nombre) == cliente)
    }

    pub fn estado(&self) -> EstadoDelCorpus {
        let ilegibles = self
            .documentos
            .iter()
            .filter(|d| matches!(d.estado, Estado::SinLeer { .. }))
            .count();
        let legibles = || self.documentos.iter().filter(|d| matches!(d.estado, Estado::Indexado { .. }));

        EstadoDelCorpus {
            carpeta: self.carpeta.as_ref().map(|c| c.to_string_lossy().to_string()),
            documentos: self.documentos.len(),
            secciones: self.indice.secciones(),
            por_unidad: Unidad::TODAS
                .into_iter()
                .map(|u| PorUnidad {
                    unidad: u.etiqueta().to_string(),
                    documentos: legibles().filter(|d| d.unidad == Some(u)).count(),
                })
                .collect(),
            sin_unidad: legibles().filter(|d| d.unidad.is_none()).count(),
            ilegibles,
            conjeturados: legibles().filter(|d| d.conjeturado).count(),
            donde_vive: self.indice.carpeta().map(|c| c.to_string_lossy().to_string()),
            bytes_del_indice: self.indice.carpeta().map(pesa).unwrap_or(0),
        }
    }
}

/// Lista los archivos legibles de una carpeta, ordenados, sin seguir enlaces simbólicos.
///
/// El orden importa: dos indexaciones de la misma carpeta tienen que recorrerla igual, o el
/// techo de documentos recortaría un conjunto distinto cada vez y nadie entendería por qué.
fn recorrer(carpeta: &Path, hondura: usize) -> Vec<PathBuf> {
    if hondura == 0 {
        return Vec::new();
    }
    let Ok(entradas) = std::fs::read_dir(carpeta) else {
        return Vec::new();
    };
    let mut archivos = Vec::new();
    let mut carpetas = Vec::new();
    for e in entradas.flatten() {
        let ruta = e.path();
        let nombre = e.file_name().to_string_lossy().to_string();
        // Lo oculto se queda fuera: ahí viven los archivos de sincronización de las nubes, no
        // los documentos del usuario.
        if nombre.starts_with('.') {
            continue;
        }
        match e.file_type() {
            Ok(t) if t.is_dir() => carpetas.push(ruta),
            Ok(t) if t.is_file() && leer::se_lee(&ruta) => archivos.push(ruta),
            _ => {}
        }
    }
    archivos.sort();
    carpetas.sort();
    for c in carpetas {
        archivos.extend(recorrer(&c, hondura - 1));
    }
    archivos
}

fn pesa(carpeta: &Path) -> u64 {
    let Ok(entradas) = std::fs::read_dir(carpeta) else {
        return 0;
    };
    entradas
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => pesa(&e.path()),
            _ => e.metadata().map(|m| m.len()).unwrap_or(0),
        })
        .sum()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **El ensayo lee la propuesta y la ficha del kit por título** (sprint 004): las seis secciones de la
    /// propuesta y las cuatro de la ficha, enteras, aunque «Historial» sea más corta que el mínimo del
    /// índice. Y la propuesta se encuentra por el cliente que nombra.
    /// **«Preparar» no relee tus propuestas** (auditoría del S4, M9): el cliente se busca en el texto que se
    /// leyó al indexar. Con el archivo ya borrado del disco, la propuesta se sigue encontrando por su texto.
    #[test]
    fn propuestas_de_no_relee_el_disco() {
        let d = std::env::temp_dir().join(format!("ag-corpus-propuestas-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let archivo = d.join("Propuesta comercial.md");
        std::fs::write(
            &archivo,
            "# Alcance\nPara Páramo Azul: perfilado y limpieza de tres fuentes de datos del negocio.\n\n\
             # Precio\nTarifa cerrada de cuatro semanas, con dos rondas de revisión incluidas.\n",
        )
        .unwrap();
        let mut c = Corpus::en_memoria().unwrap();
        c.indexar(&d, &|_| {}).unwrap();
        assert_eq!(c.propuestas_de("Páramo Azul").len(), 1, "no se encontró por su texto");
        std::fs::remove_file(&archivo).unwrap();
        assert_eq!(c.propuestas_de("Páramo Azul").len(), 1, "se volvió a leer el disco");
        assert!(c.propuestas_de("Sur del Valle").is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn el_ensayo_lee_la_propuesta_y_la_ficha_de_un_cliente_por_titulo() {
        let kit = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/kit-de-prueba/corpus");
        let mut c = Corpus::en_memoria().unwrap();
        c.indexar(&kit, &|_| {}).unwrap();
        let propuestas = c.propuestas_de("Páramo Azul");
        assert_eq!(propuestas.len(), 1, "{:?}", propuestas.iter().map(|d| &d.nombre).collect::<Vec<_>>());
        assert!(c.propuestas_de("Sur del Valle").is_empty(), "el caso no es una propuesta");
        assert!(c.propuestas_de("  ").is_empty());
        let titulos = |ruta: &str| c.secciones_de(ruta).unwrap().into_iter().map(|s| s.titulo.unwrap_or_default()).collect::<Vec<_>>();
        assert_eq!(titulos(&propuestas[0].ruta), ["Contexto", "Alcance", "Supuestos", "Entregables", "Precio", "Plazo de entrega"]);
        let ficha = c.ficha_de("Páramo Azul").expect("la ficha del kit");
        assert_eq!(titulos(&ficha.ruta), ["Quiénes son", "Quién decide", "Acuerdos previos", "Historial"]);
        assert!(c.secciones_de("/etc/hosts").is_err(), "una ruta que no es de tu corpus no se lee");
    }

    /// Cada test estrena carpeta. Compartirla por PID las hace pisarse entre ellas al correr en
    /// paralelo dentro del mismo binario — el mismo defecto que en la fase 3 obligó a serializar
    /// los tests de audio, y aquí se evita con un nombre distinto en vez de con un candado.
    fn carpeta_de_prueba(quien: &str) -> PathBuf {
        let c = std::env::temp_dir().join(format!("ag-corpus-{}-{quien}", std::process::id()));
        let _ = std::fs::remove_dir_all(&c);
        std::fs::create_dir_all(c.join("casos")).unwrap();
        std::fs::write(
            c.join("Propuesta Páramo Azul.md"),
            "# Alcance\nTres canales y una línea base de doce meses de histórico.\n\n# Precio\nTarifa cerrada por cuatro semanas de trabajo del equipo.\n",
        )
        .unwrap();
        std::fs::write(
            c.join("casos/Cooperativa Sur del Valle · cierre de caso.md"),
            "# Resultados\nLa implementación cerró con dos semanas de retraso y sin sobrecosto.\n",
        )
        .unwrap();
        // Ilegible: existe, pesa, y no suelta texto.
        std::fs::write(c.join("acta escaneada.pdf"), b"%PDF-1.4 roto").unwrap();
        // Ajeno: ni se mira.
        std::fs::write(c.join("hoja.xlsx"), b"cualquier cosa").unwrap();
        // Oculto: tampoco.
        std::fs::write(c.join(".sincroniza.md"), "# Nada\nnada de nada aquí dentro nunca.\n").unwrap();
        c
    }

    /// **La jurisdicción sale de la ficha del cliente, y de nada más** (ADR 017 §2). ¿Puede fallar?
    /// Sí: sin mirar la unidad, la propuesta que dice «Jurisdicción: Chile» le pondría bandera a un
    /// documento que no es un cliente (rojo en la bitácora).
    #[test]
    fn la_jurisdiccion_sale_de_la_ficha_del_cliente_y_de_nada_mas() {
        let c = std::env::temp_dir().join(format!("ag-corpus-{}-jurisdiccion", std::process::id()));
        let _ = std::fs::remove_dir_all(&c);
        std::fs::create_dir_all(&c).unwrap();
        std::fs::write(
            c.join("Ficha de cliente · Páramo Azul.md"),
            "# Quiénes son\nDistribuidora familiar de alimento para ganado, ciento veinte empleados.\nJurisdicción: Colombia\n\n# Quién decide\nLa gerente general firma.\n",
        )
        .unwrap();
        std::fs::write(
            c.join("Ficha de cliente · Sur del Valle.md"),
            "# Quiénes son\nCooperativa lechera de tres municipios del valle, con planta propia.\n",
        )
        .unwrap();
        std::fs::write(
            c.join("Propuesta Páramo Azul.md"),
            "# Alcance\nTres canales y una línea base de doce meses de histórico.\nJurisdicción: Chile\n",
        )
        .unwrap();
        let mut corpus = Corpus::en_memoria().unwrap();
        corpus.indexar(&c, &|_| {}).unwrap();
        assert_eq!(corpus.jurisdiccion_de("Páramo Azul"), Some("Colombia"));
        assert_eq!(corpus.jurisdiccion_de("Sur del Valle"), None, "una ficha sin la línea no tiene jurisdicción");
        let otros: Vec<_> = corpus.documentos().iter().filter(|d| d.unidad != Some(Unidad::Cliente)).collect();
        assert!(!otros.is_empty(), "la propuesta no se indexó: el caso no midió nada");
        assert!(otros.iter().all(|d| d.jurisdiccion.is_none()), "un documento que no es de un cliente tiene jurisdicción");
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn indexa_una_carpeta_entera_y_un_documento_roto_no_detiene_a_los_demas() {
        let c = carpeta_de_prueba("entera");
        let mut corpus = Corpus::en_memoria().unwrap();
        let vistos = std::sync::Mutex::new(Vec::new());
        corpus.indexar(&c, &|d| vistos.lock().unwrap().push(d.nombre.clone())).unwrap();

        let e = corpus.estado();
        assert_eq!(e.documentos, 3, "se esperaban los dos .md y el .pdf roto: {:?}", corpus.documentos());
        assert_eq!(e.ilegibles, 1);
        assert_eq!(vistos.lock().unwrap().len(), 3, "el aviso de progreso no llegó por cada documento");

        // Y los dos buenos SÍ se indexaron, que es lo que la promesa significa.
        let h = corpus.buscar("y la tarifa de esas semanas de trabajo", 3).unwrap();
        assert!(!h.is_empty(), "el documento roto se llevó por delante a los sanos");
        assert_eq!(h[0].seccion.as_deref(), Some("Precio"));
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn el_documento_ilegible_trae_su_motivo_en_espanol_llano() {
        let c = carpeta_de_prueba("motivo");
        let mut corpus = Corpus::en_memoria().unwrap();
        corpus.indexar(&c, &|_| {}).unwrap();
        let roto = corpus.documentos().iter().find(|d| d.nombre.contains("acta")).unwrap();
        match &roto.estado {
            Estado::SinLeer { motivo } => assert!(!motivo.is_empty()),
            otro => panic!("el .pdf roto se dio por indexado: {otro:?}"),
        }
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn baja_por_las_subcarpetas_y_no_mira_ni_lo_oculto_ni_lo_ajeno() {
        let c = carpeta_de_prueba("hondura");
        let mut corpus = Corpus::en_memoria().unwrap();
        corpus.indexar(&c, &|_| {}).unwrap();
        let nombres: Vec<&str> = corpus.documentos().iter().map(|d| d.nombre.as_str()).collect();
        assert!(nombres.iter().any(|n| n.contains("Cooperativa")), "no bajó a la subcarpeta");
        assert!(!nombres.iter().any(|n| n.contains("sincroniza")), "indexó un archivo oculto");
        assert!(!nombres.iter().any(|n| n.contains("hoja")), "abrió un formato que no lee");
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn el_estado_reparte_los_documentos_por_unidad() {
        let c = carpeta_de_prueba("unidades");
        let mut corpus = Corpus::en_memoria().unwrap();
        corpus.indexar(&c, &|_| {}).unwrap();
        let e = corpus.estado();
        let cuantos = |u: &str| e.por_unidad.iter().find(|p| p.unidad == u).unwrap().documentos;
        assert_eq!(cuantos("propuesta"), 1);
        assert_eq!(cuantos("caso"), 1);
        assert_eq!(e.por_unidad.len(), 5, "las cinco unidades salen siempre, aunque estén en cero");
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn sin_carpeta_senalada_el_corpus_esta_vacio_y_lo_dice() {
        let e = Corpus::en_memoria().unwrap().estado();
        assert_eq!(e.carpeta, None);
        assert_eq!(e.documentos, 0);
        assert_eq!(e.secciones, 0);
    }

    #[test]
    fn indexar_algo_que_no_es_carpeta_se_dice_en_vez_de_reventar() {
        let mut corpus = Corpus::en_memoria().unwrap();
        assert!(corpus.indexar(Path::new("/no/existe/esto"), &|_| {}).is_err());
    }
}
