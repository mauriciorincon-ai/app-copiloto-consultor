//! EL ÍNDICE — BM25 sobre las secciones, en los dos idiomas.
//!
//! ## Por qué dos campos por idioma y no uno
//!
//! Un stemmer es de un idioma. El español recorta «rentabilidad» a «rentabil» y encuentra
//! «rentable»; el inglés no sabe hacerlo, y al revés con «profitability»/«profitable». Una app
//! bilingüe por regla dura no puede elegir uno: **cada sección se indexa dos veces, una con cada
//! stemmer**, y la consulta pregunta a los dos. Cuesta el doble de índice sobre un corpus de
//! cientos de documentos —unos megabytes— y a cambio las dos mitades de los usuarios encuentran
//! sus cosas. La alternativa, adivinar el idioma del documento, falla justo en los corpus
//! mezclados, que son los de este usuario.
//!
//! ## El título pesa más que el cuerpo
//!
//! Una sección que se **llama** «Precio» responde mejor a una pregunta sobre precio que una que
//! lo menciona de pasada. `PESO_DEL_TITULO` lo dice con un número en vez de con una intuición.
//!
//! ## El índice es un derivado, y nace igual de privado que su fuente
//!
//! Dentro viven, en claro, trozos de los documentos del usuario. La regla de los derivados del
//! `CLAUDE.md` es explícita —*«un derivado JAMÁS nace menos privado que su fuente»*— con un
//! precedente caro detrás: un índice que nació legible para todo el mundo con datos reales
//! dentro, y 531 tests en verde que no lo vieron. Aquí la carpeta nace en 700 y se **repara al
//! abrir** si ya existía mal, porque la primera versión de la app pudo haberla creado floja.

use std::path::{Path, PathBuf};

use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::{Schema, TextFieldIndexing, TextOptions, Value, STORED, STRING};
use tantivy::tokenizer::{
    AsciiFoldingFilter, Language, LowerCaser, SimpleTokenizer, Stemmer, TextAnalyzer,
};
use tantivy::{doc, Index, IndexReader, IndexWriter, TantivyDocument};

use super::seccion::Seccion;
use super::unidad::Unidad;

/// Cuánto más pesa una palabra que está en el título de la sección.
const PESO_DEL_TITULO: f32 = 3.0;

/// **Cuánto pesa lo que hay en la pantalla**, cuando la pantalla entra a desempatar.
///
/// **Y cuándo entra: [`DESEMPATE`].** Los dos números salieron de medir, y la medida enseñó algo que
/// la intuición no: la pantalla **ayuda a lo que se ve y estorba a lo demás**. Con la pantalla
/// siempre dentro de la consulta, las treinta preguntas del kit v0 —que no tienen nada que ver con la
/// diapositiva que haya delante— bajaban de 0,823 a 0,764 con peso 0,5 y a 0,721 con peso 1,5.
/// Así que la pantalla **solo desempata**: si la pregunta sola ya tiene una sección que le saca
/// ventaja a la segunda, manda la pregunta.
///
/// La tabla, con el kit de pantalla (cinco frases vagas frente a su pantalla) y el kit v0 con la
/// PEOR de las cinco pantallas delante (mínimo 0,80):
///
/// | peso | desempate | kit de pantalla | kit v0, peor pantalla |
/// |------|-----------|-----------------|-----------------------|
/// | 0,5  | siempre   | 0,700           | 0,764 (bajo el mínimo) |
/// | 1,5  | siempre   | 0,800           | 0,721 (bajo el mínimo) |
/// | 1,5  | 2,0       | 0,800           | 0,747 (bajo el mínimo) |
/// | 1,0  | 1,25      | 0,700           | 0,807                 |
/// | **0,5** | **1,1** | **0,700**      | **0,819**             |
///
/// Se elige la fila que más ayuda sin que ninguna pantalla deje el kit v0 bajo su mínimo, y entre
/// las que empatan, la que menos lo toca. Sin pantalla, 0,823 como siempre (lo vigila un test).
pub const PESO_DE_LA_PANTALLA: f32 = 0.5;

/// **La pantalla solo desempata.** Si la primera sección de la pregunta sola le saca al menos un
/// 10 % a la segunda, la pregunta ya tiene respuesta y la pantalla no la toca. Ver la tabla de
/// [`PESO_DE_LA_PANTALLA`].
pub const DESEMPATE: f32 = 1.1;

/// Memoria del escritor. Por debajo de 15 MB tantivy se queja; más no hace falta para un corpus
/// de documentos de oficina.
const MEMORIA: usize = 15_000_000;

/// Permisos de la carpeta del índice: solo su dueño. Ver la nota de cabecera.
#[cfg(unix)]
pub const PERMISOS: u32 = 0o700;

#[derive(Debug, Clone, PartialEq)]
pub struct Hallazgo {
    pub documento: String,
    pub ruta: String,
    pub seccion: Option<String>,
    pub unidad: Option<Unidad>,
    pub texto: String,
    pub puntaje: f32,
    /// La sección era una conjetura del lector (PDF). La ficha no promete lo que no sabe.
    pub conjeturado: bool,
}

struct Campos {
    ruta: tantivy::schema::Field,
    documento: tantivy::schema::Field,
    seccion: tantivy::schema::Field,
    unidad: tantivy::schema::Field,
    texto: tantivy::schema::Field,
    conjeturado: tantivy::schema::Field,
    titulo_es: tantivy::schema::Field,
    titulo_en: tantivy::schema::Field,
    cuerpo_es: tantivy::schema::Field,
    cuerpo_en: tantivy::schema::Field,
}

pub struct Indice {
    indice: Index,
    campos: Campos,
    lector: IndexReader,
    esquema: Schema,
    carpeta: Option<PathBuf>,
}

fn analizado(tokenizador: &str) -> TextOptions {
    TextOptions::default().set_indexing_options(
        TextFieldIndexing::default()
            .set_tokenizer(tokenizador)
            .set_index_option(tantivy::schema::IndexRecordOption::WithFreqsAndPositions),
    )
}

fn esquema() -> (Schema, Campos) {
    let mut b = Schema::builder();
    let campos = Campos {
        ruta: b.add_text_field("ruta", STRING | STORED),
        documento: b.add_text_field("documento", STORED),
        seccion: b.add_text_field("seccion", STORED),
        unidad: b.add_text_field("unidad", STRING | STORED),
        texto: b.add_text_field("texto", STORED),
        conjeturado: b.add_text_field("conjeturado", STORED),
        titulo_es: b.add_text_field("titulo_es", analizado("es")),
        titulo_en: b.add_text_field("titulo_en", analizado("en")),
        cuerpo_es: b.add_text_field("cuerpo_es", analizado("es")),
        cuerpo_en: b.add_text_field("cuerpo_en", analizado("en")),
    };
    (b.build(), campos)
}

/// La cadena de análisis, y el acento en medio.
///
/// `AsciiFoldingFilter` va **antes** del stemmer y no es gratis: el stemmer español usa la tilde
/// para reconocer el sufijo «-ción», así que al plegarla pierde la familia
/// «implementación~implementar». A cambio gana todas las parejas que solo se diferencian en el
/// acento, y esas son las que de verdad ocurren: el transcriptor escribe sin tilde, el usuario
/// teclea sin tilde, y el documento la lleva.
///
/// **Medido antes de elegir**, sobre 23 parejas de lenguaje de consultoría (documento ~ lo que
/// diría el cliente): **17/23 con plegado contra 15/23 sin él**. Se pierden dos
/// («implementación~implementar», «implementación~implementan») y se ganan cuatro
/// («metodología», «implementación», «García», «auditoría» sin tilde). El detalle vive en el ADR
/// del corpus; el kit de evaluación (nDCG@5) es el instrumento para revisarlo si hiciera falta.
fn registrar_idiomas(indice: &Index) {
    for (nombre, idioma) in [("es", Language::Spanish), ("en", Language::English)] {
        let a = TextAnalyzer::builder(SimpleTokenizer::default())
            .filter(LowerCaser)
            .filter(AsciiFoldingFilter)
            .filter(Stemmer::new(idioma))
            .build();
        indice.tokenizers().register(nombre, a);
    }
}

impl Indice {
    /// El índice de verdad, en la carpeta de la app.
    pub fn en(carpeta: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(carpeta).map_err(|e| format!("no se pudo crear el índice: {e}"))?;
        asegurar_permisos(carpeta)?;
        let (esq, campos) = esquema();
        let indice = Index::open_in_dir(carpeta)
            .or_else(|_| Index::create_in_dir(carpeta, esq.clone()))
            .map_err(|e| format!("no se pudo abrir el índice: {e}"))?;
        // Después de crear: tantivy escribe sus archivos ahora, y heredan el umask si no se toca.
        asegurar_permisos(carpeta)?;
        Self::con(indice, esq, campos, Some(carpeta.to_path_buf()))
    }

    /// El mismo índice, en memoria. No es un apaño para los tests: es lo que usa el kit de
    /// evaluación, que no debe dejar nada en disco.
    pub fn en_memoria() -> Result<Self, String> {
        let (esq, campos) = esquema();
        let indice = Index::create_in_ram(esq.clone());
        Self::con(indice, esq, campos, None)
    }

    fn con(indice: Index, esquema: Schema, campos: Campos, carpeta: Option<PathBuf>) -> Result<Self, String> {
        registrar_idiomas(&indice);
        let lector = indice.reader().map_err(|e| e.to_string())?;
        Ok(Indice { indice, campos, lector, esquema, carpeta })
    }

    pub fn carpeta(&self) -> Option<&Path> {
        self.carpeta.as_deref()
    }

    /// Mete un documento entero. Si ya estaba indexado, **se reemplaza**: reindexar una carpeta
    /// dos veces no puede dejar cada sección por duplicado compitiendo consigo misma.
    pub fn meter(
        &self,
        ruta: &str,
        documento: &str,
        unidad: Option<Unidad>,
        conjeturado: bool,
        secciones: &[Seccion],
    ) -> Result<usize, String> {
        let mut w: IndexWriter = self.indice.writer(MEMORIA).map_err(|e| e.to_string())?;
        w.delete_term(tantivy::Term::from_field_text(self.campos.ruta, ruta));
        let c = &self.campos;
        for s in secciones {
            let titulo = s.titulo.clone().unwrap_or_default();
            w.add_document(doc!(
                c.ruta => ruta,
                c.documento => documento,
                c.seccion => titulo.as_str(),
                c.unidad => unidad.map(|u| u.etiqueta()).unwrap_or(""),
                c.texto => s.texto.as_str(),
                c.conjeturado => if conjeturado { "si" } else { "no" },
                c.titulo_es => titulo.as_str(),
                c.titulo_en => titulo.as_str(),
                c.cuerpo_es => s.texto.as_str(),
                c.cuerpo_en => s.texto.as_str(),
            ))
            .map_err(|e| e.to_string())?;
        }
        w.commit().map_err(|e| e.to_string())?;
        self.lector.reload().map_err(|e| e.to_string())?;
        Ok(secciones.len())
    }

    pub fn olvidar(&self, ruta: &str) -> Result<(), String> {
        let mut w: IndexWriter = self.indice.writer(MEMORIA).map_err(|e| e.to_string())?;
        w.delete_term(tantivy::Term::from_field_text(self.campos.ruta, ruta));
        w.commit().map_err(|e| e.to_string())?;
        self.lector.reload().map_err(|e| e.to_string())
    }

    pub fn vaciar(&self) -> Result<(), String> {
        let mut w: IndexWriter = self.indice.writer(MEMORIA).map_err(|e| e.to_string())?;
        w.delete_all_documents().map_err(|e| e.to_string())?;
        w.commit().map_err(|e| e.to_string())?;
        self.lector.reload().map_err(|e| e.to_string())
    }

    pub fn secciones(&self) -> usize {
        self.lector.searcher().num_docs() as usize
    }

    /// La búsqueda. `texto` es lo que dijo el cliente, tal cual salió del transcriptor.
    pub fn buscar(&self, texto: &str, cuantos: usize) -> Result<Vec<Hallazgo>, String> {
        self.buscar_con_pantalla(texto, "", cuantos)
    }

    /// **La búsqueda con la pantalla como contexto** (C8, sprint 002).
    ///
    /// Primero se busca la pregunta sola. Si tiene una respuesta clara ([`DESEMPATE`]), esa es la
    /// respuesta. Si no, se repite la MISMA consulta con la pantalla dentro: la pregunta como
    /// cláusula obligatoria y la pantalla como opcional ([`PESO_DE_LA_PANTALLA`]). Así la pantalla
    /// **solo reordena lo que la pregunta ya encontró**, nunca trae algo que la pregunta no pidiera,
    /// y no estropea las preguntas que no tienen nada que ver con lo que hay delante. No es una
    /// segunda búsqueda de la pantalla que luego haya que mezclar con una regla inventada: es la de
    /// la pregunta, desempatada.
    ///
    /// Con `pantalla` vacío es exactamente la búsqueda de siempre —la misma consulta, el mismo
    /// orden—, y eso lo vigila un test: el refuerzo no puede cambiar nada cuando no hay pantalla.
    pub fn buscar_con_pantalla(
        &self,
        texto: &str,
        pantalla: &str,
        cuantos: usize,
    ) -> Result<Vec<Hallazgo>, String> {
        use tantivy::query::{BooleanQuery, BoostQuery, Occur, Query};

        let consulta = super::consulta::limpiar(texto);
        let de_la_pantalla = super::consulta::limpiar(pantalla);
        if consulta.is_empty() && de_la_pantalla.is_empty() {
            return Ok(Vec::new());
        }
        if !consulta.is_empty() && !de_la_pantalla.is_empty() {
            let sola = self.buscar_con_pantalla(texto, "", cuantos.max(2))?;
            let clara = match (sola.first(), sola.get(1)) {
                (Some(a), Some(b)) => a.puntaje >= b.puntaje * DESEMPATE,
                _ => true,
            };
            if clara {
                return Ok(sola.into_iter().take(cuantos).collect());
            }
        }
        let c = &self.campos;
        let mut qp = QueryParser::for_index(
            &self.indice,
            vec![c.titulo_es, c.titulo_en, c.cuerpo_es, c.cuerpo_en],
        );
        qp.set_field_boost(c.titulo_es, PESO_DEL_TITULO);
        qp.set_field_boost(c.titulo_en, PESO_DEL_TITULO);
        let q: Box<dyn Query> = match (consulta.is_empty(), de_la_pantalla.is_empty()) {
            (false, true) => qp.parse_query(&consulta).map_err(|e| e.to_string())?,
            // Solo pantalla: es la ficha que la pantalla pide por sí sola. Sin nada con qué
            // compararla, el peso relativo no significa nada y no se aplica.
            (true, false) => qp.parse_query(&de_la_pantalla).map_err(|e| e.to_string())?,
            // La pregunta es OBLIGATORIA y la pantalla opcional: la pantalla solo reordena lo que
            // la pregunta ya encontró, jamás trae una sección que no tenga nada que ver con lo
            // preguntado. La primera versión las ponía a las dos como opcionales y su propio test
            // lo tumbó: «¿la tarifa es cerrada?» con una diapositiva de adopción de datos delante
            // devolvía el marco de adopción — la pantalla, con medio peso, pesaba más que la
            // pregunta porque traía tres palabras en el título de su sección.
            _ => Box::new(BooleanQuery::new(vec![
                (
                    Occur::Must,
                    qp.parse_query(&consulta).map_err(|e| e.to_string())?,
                ),
                (
                    Occur::Should,
                    Box::new(BoostQuery::new(
                        qp.parse_query(&de_la_pantalla).map_err(|e| e.to_string())?,
                        PESO_DE_LA_PANTALLA,
                    )),
                ),
            ])),
        };

        let buscador = self.lector.searcher();
        let top = buscador
            .search(&q, &TopDocs::with_limit(cuantos).order_by_score())
            .map_err(|e| e.to_string())?;

        let saca = |d: &TantivyDocument, campo| -> String {
            d.get_first(campo).and_then(|v| v.as_str()).unwrap_or("").to_string()
        };
        let mut salida = Vec::new();
        for (puntaje, dir) in top {
            let d: TantivyDocument = buscador.doc(dir).map_err(|e| e.to_string())?;
            let seccion = saca(&d, c.seccion);
            salida.push(Hallazgo {
                documento: saca(&d, c.documento),
                ruta: saca(&d, c.ruta),
                seccion: (!seccion.is_empty()).then_some(seccion),
                unidad: Unidad::de_etiqueta(&saca(&d, c.unidad)),
                texto: saca(&d, c.texto),
                puntaje,
                conjeturado: saca(&d, c.conjeturado) == "si",
            });
        }
        let _ = &self.esquema;
        Ok(salida)
    }
}

/// Deja la carpeta del índice en 700, la cree quien la cree. Se llama también al **abrir** una
/// que ya existía: una versión anterior de la app pudo haberla dejado floja, y descubrirlo no
/// sirve de nada si no se repara.
#[cfg(unix)]
pub fn asegurar_permisos(carpeta: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    let md = std::fs::metadata(carpeta).map_err(|e| e.to_string())?;
    let ahora = md.permissions().mode() & 0o777;
    if ahora != PERMISOS {
        std::fs::set_permissions(carpeta, std::fs::Permissions::from_mode(PERMISOS))
            .map_err(|e| format!("no se pudieron cerrar los permisos del índice: {e}"))?;
        println!("[corpus] la carpeta del índice estaba en {ahora:o}; se dejó en {PERMISOS:o}");
    }
    Ok(())
}

#[cfg(not(unix))]
pub fn asegurar_permisos(_carpeta: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::corpus::seccion::Seccion;

    fn s(titulo: &str, texto: &str) -> Seccion {
        Seccion { titulo: (!titulo.is_empty()).then(|| titulo.to_string()), texto: texto.into() }
    }

    fn con_dos_documentos() -> Indice {
        let i = Indice::en_memoria().unwrap();
        i.meter(
            "/c/propuesta.md",
            "Páramo Azul · Propuesta rentabilidad por canal",
            Some(Unidad::Propuesta),
            false,
            &[
                s("Alcance", "Tres canales y una línea base de doce meses."),
                s("Precio", "Tarifa cerrada por cuatro semanas de trabajo."),
            ],
        )
        .unwrap();
        i.meter(
            "/c/marco.md",
            "Adopción de datos en 4 etapas",
            Some(Unidad::Marco),
            false,
            &[s("Etapas", "El marco recorre cuatro etapas de adopción de datos.")],
        )
        .unwrap();
        i
    }

    #[test]
    fn encuentra_la_seccion_y_no_solo_el_documento() {
        let h = con_dos_documentos().buscar("cuánto cuesta y en cuántas semanas", 3).unwrap();
        assert_eq!(h[0].seccion.as_deref(), Some("Precio"));
        assert_eq!(h[0].unidad, Some(Unidad::Propuesta));
    }

    /// La razón de los dos stemmers, escrita con lo que el stemmer español hace DE VERDAD.
    ///
    /// La primera versión de este test afirmaba que «rentable» encuentra «rentabilidad». No es
    /// cierto —`rentabil` contra `rentabl`— y el spike que lo dio por bueno pasaba por la palabra
    /// «canal», que iba en la misma consulta. Lo que sí une es el número y la conjugación.
    #[test]
    fn el_stemmer_espanol_une_singular_y_plural() {
        let i = con_dos_documentos();
        let h = i.buscar("cuántos canales miden", 3).unwrap();
        assert!(!h.is_empty(), "«canales» no encontró «canal»");
        assert_eq!(h[0].seccion.as_deref(), Some("Alcance"));
    }

    /// Y lo que el plegado de acentos añade, que es el caso frecuente de verdad: el transcriptor
    /// y el teclado escriben sin tilde, y el documento la lleva.
    #[test]
    fn una_palabra_sin_tilde_encuentra_la_misma_palabra_con_tilde() {
        let i = Indice::en_memoria().unwrap();
        i.meter("/m.md", "Metodología", Some(Unidad::Marco), false, &[
            s("Metodología", "Nuestra metodología de adopción recorre cuatro etapas."),
        ])
        .unwrap();
        let h = i.buscar("cual es su metodologia de adopcion", 3).unwrap();
        assert!(!h.is_empty(), "«metodologia» sin tilde no encontró «metodología»");
    }

    /// Y la otra mitad de la regla bilingüe: el mismo índice responde en inglés.
    #[test]
    fn el_mismo_indice_responde_en_ingles() {
        let i = Indice::en_memoria().unwrap();
        i.meter(
            "/c/proposal.md",
            "Channel profitability proposal",
            Some(Unidad::Propuesta),
            false,
            &[s("Scope", "Three channels and a twelve month baseline.")],
        )
        .unwrap();
        let h = i.buscar("how many channels are we baselining", 3).unwrap();
        assert!(!h.is_empty(), "el stemmer inglés no encontró nada");
        assert_eq!(h[0].seccion.as_deref(), Some("Scope"));
    }

    /// Una sección que se **llama** como la pregunta gana a una que la menciona de pasada.
    #[test]
    fn el_titulo_pesa_mas_que_el_cuerpo() {
        let i = Indice::en_memoria().unwrap();
        i.meter("/a.md", "A", None, false, &[
            s("Notas", "Hablamos del precio, del precio y otra vez del precio."),
            s("Precio", "Cuatro semanas."),
        ])
        .unwrap();
        let h = i.buscar("precio", 2).unwrap();
        assert_eq!(h[0].seccion.as_deref(), Some("Precio"));
    }

    /// Reindexar la misma carpeta dos veces no puede dejar cada sección compitiendo consigo misma.
    #[test]
    fn reindexar_reemplaza_en_vez_de_duplicar() {
        let i = con_dos_documentos();
        let antes = i.secciones();
        i.meter("/c/marco.md", "Adopción de datos en 4 etapas", Some(Unidad::Marco), false, &[
            s("Etapas", "El marco recorre cuatro etapas de adopción de datos."),
        ])
        .unwrap();
        assert_eq!(i.secciones(), antes);
    }

    /// El refuerzo de la pantalla no puede cambiar NADA cuando no hay pantalla: ni el orden ni los
    /// puntajes. Si lo hiciera, el kit del sprint 001 dejaría de medir lo que medía.
    #[test]
    fn sin_pantalla_la_busqueda_es_la_de_siempre() {
        let i = con_dos_documentos();
        for pregunta in [
            "cuánto cuesta y en cuántas semanas",
            "cuántos canales miden",
            "etapas",
        ] {
            assert_eq!(
                i.buscar(pregunta, 3).unwrap(),
                i.buscar_con_pantalla(pregunta, "", 3).unwrap()
            );
        }
    }

    /// **Para qué sirve la pantalla**: una pregunta que vale LO MISMO para dos secciones se inclina
    /// hacia la que el cliente tiene delante. Las dos secciones son simétricas a propósito —mismo
    /// largo, misma palabra—, para que el empate sea de verdad y no dependa de cómo normaliza BM25.
    #[test]
    fn la_pantalla_desempata_hacia_lo_que_se_ve() {
        let i = Indice::en_memoria().unwrap();
        i.meter(
            "/p.md",
            "Propuesta",
            None,
            false,
            &[s("Plazo", "Son cuatro semanas de trabajo con el cliente.")],
        )
        .unwrap();
        i.meter(
            "/m.md",
            "Método",
            None,
            false,
            &[s("Etapas", "Son cuatro etapas de trabajo con el cliente.")],
        )
        .unwrap();
        let pregunta = "¿y lo de las cuatro?";
        let hacia = |pantalla: &str| {
            i.buscar_con_pantalla(pregunta, pantalla, 3).unwrap()[0]
                .seccion
                .clone()
        };
        assert_eq!(hacia("Etapas del método"), Some("Etapas".into()));
        assert_eq!(hacia("Plazo en semanas"), Some("Plazo".into()));
    }

    /// Y lo contrario: si la pregunta ya tiene una respuesta clara, **la pantalla no la toca** —ni
    /// siquiera reordena—. Es lo que protege a las preguntas que no tienen nada que ver con lo que
    /// hay delante (la tabla de [`PESO_DE_LA_PANTALLA`]).
    ///
    /// **La primera versión de este test era decorativo**, y lo destapó su demo en rojo: con la
    /// pantalla metida SIEMPRE en la consulta seguía en verde, porque su pantalla no tocaba ninguna
    /// sección que la pregunta encontrara — con o sin desempate daba lo mismo. Esta la elige para que
    /// SÍ lo cambiaría: con la pantalla dentro, «Etapas» sube de 0,9 a 7,2 puntos y se pone a la
    /// sombra de «Precio» (9,5). La pregunta sola dice «Precio» con ventaja clara, y tiene que quedar
    /// exactamente así — orden y puntajes—.
    #[test]
    fn con_una_respuesta_clara_la_pantalla_no_toca_nada() {
        let i = con_dos_documentos();
        let pregunta = "¿el precio de las cuatro semanas?";
        let pantalla = "Adopción de datos · cuatro etapas · inventario · etapas";
        let sola = i.buscar(pregunta, 3).unwrap();
        assert_eq!(sola[0].seccion.as_deref(), Some("Precio"));
        assert!(sola[0].puntaje >= sola[1].puntaje * DESEMPATE, "el ejemplo tiene que ser una respuesta clara");
        assert_eq!(sola, i.buscar_con_pantalla(pregunta, pantalla, 3).unwrap());
    }

    /// La pregunta manda sobre la pantalla: una diapositiva de otro tema no se lleva la respuesta
    /// de una pregunta clara. Es lo que [`PESO_DE_LA_PANTALLA`] < 1 protege.
    #[test]
    fn la_pregunta_manda_sobre_la_pantalla() {
        let i = con_dos_documentos();
        let h = i
            .buscar_con_pantalla("¿la tarifa es cerrada?", "Adopción de datos · etapas", 3)
            .unwrap();
        assert_eq!(h[0].seccion.as_deref(), Some("Precio"));
    }

    /// Solo pantalla: es la ficha que la pantalla pide por sí sola.
    #[test]
    fn la_pantalla_sola_tambien_busca() {
        let h = con_dos_documentos()
            .buscar_con_pantalla("", "Tarifa cerrada por semanas", 3)
            .unwrap();
        assert_eq!(h[0].seccion.as_deref(), Some("Precio"));
    }

    #[test]
    fn una_consulta_sin_palabras_utiles_no_devuelve_nada_en_vez_de_reventar() {
        assert!(con_dos_documentos().buscar("¿... ?", 3).unwrap().is_empty());
    }

    #[test]
    fn olvidar_y_vaciar_dejan_el_indice_como_dicen() {
        let i = con_dos_documentos();
        i.olvidar("/c/marco.md").unwrap();
        assert_eq!(i.secciones(), 2);
        i.vaciar().unwrap();
        assert_eq!(i.secciones(), 0);
    }

    /// El gate de los derivados. Un índice con el corpus del usuario en claro no puede nacer
    /// legible para las demás cuentas del Mac — ni quedarse así si ya existía flojo.
    #[cfg(unix)]
    #[test]
    fn el_indice_nace_en_700_y_se_repara_si_lo_encuentra_abierto() {
        use std::os::unix::fs::PermissionsExt;
        let c = std::env::temp_dir().join(format!("ag-indice-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&c);

        let i = Indice::en(&c).unwrap();
        let modo = |p: &Path| std::fs::metadata(p).unwrap().permissions().mode() & 0o777;
        assert_eq!(modo(&c), PERMISOS, "el índice nació abierto");
        drop(i);

        // Y ahora el caso que de verdad ocurre: la carpeta ya existe, mal.
        std::fs::set_permissions(&c, std::fs::Permissions::from_mode(0o755)).unwrap();
        let _ = Indice::en(&c).unwrap();
        assert_eq!(modo(&c), PERMISOS, "encontró la carpeta abierta y la dejó abierta");
        let _ = std::fs::remove_dir_all(&c);
    }
}
