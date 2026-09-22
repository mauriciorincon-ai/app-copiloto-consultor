//! DE LO QUE DIJO EL CLIENTE A UNA CONSULTA.
//!
//! Entre el transcriptor y BM25 hay tres desajustes, y los tres se arreglan aquí porque los tres
//! son del lenguaje, no del buscador.
//!
//! **1. El transcriptor escribe los números como se leen, no como están en el documento.** Lo
//! descubrió la fase 3 midiendo: SpeechAnalyzer escribe «ISO 27.001» y «ISO 27,001» —con
//! separador de millares, y cuál de los dos depende del idioma— donde el documento dice
//! «ISO 27001». Sin normalizar, la norma más citada de la consultoría no se encuentra jamás.
//!
//! **2. Las palabras vacías se llevan el puntaje.** «de», «la», «the», «of» aparecen en todas
//! las secciones: dejarlas dentro es preguntar por el idioma en vez de por el tema.
//!
//! **3. Los signos rompen el analizador de consultas de tantivy.** `¿`, `:`, `^`, `"` tienen
//! significado ahí dentro; una pregunta de verdad los trae casi siempre.
//!
//! La normalización es **solo de la consulta**. El índice guarda el texto del documento tal cual:
//! es del usuario, y la ficha lo cita literalmente.

/// Techo de palabras en la consulta. Un turno largo con treinta palabras de contenido hace una
/// consulta difusa que puntúa alto en casi todo; las primeras son las que llevan el tema.
const PALABRAS: usize = 24;

/// Debajo de esto una palabra no aporta tema (pero los números sí cuentan: «4», «27001»).
const CORTA: usize = 3;

const VACIAS: &[&str] = &[
    // español
    "el", "la", "los", "las", "un", "una", "unos", "unas", "de", "del", "al", "y", "o", "que",
    "en", "por", "para", "con", "sin", "sobre", "como", "mas", "pero", "sus", "nos", "les",
    "esta", "este", "esto", "estos", "estas", "ese", "esa", "eso", "muy", "ya", "hay", "son",
    "ser", "fue", "era", "han", "has", "hemos", "cuando", "donde", "porque", "cual", "cuales",
    "nosotros", "ustedes", "ellos", "ellas", "usted", "tambien", "hacer", "tiene", "tienen",
    // inglés
    "the", "and", "for", "with", "that", "this", "these", "those", "from", "are", "was", "were",
    "have", "has", "had", "you", "your", "our", "their", "they", "them", "what", "when", "where",
    "which", "would", "could", "should", "about", "into", "than", "then", "there", "here", "its",
    "can", "will", "not", "but", "all", "any", "how", "who", "why", "does", "did", "been",
];

/// Convierte un turno hablado en una consulta para tantivy.
pub fn limpiar(texto: &str) -> String {
    let mut salida: Vec<String> = Vec::new();
    for palabra in trocear(texto) {
        if salida.len() >= PALABRAS {
            break;
        }
        let p = palabra.to_lowercase();
        // Las vacías se comparan SIN tildes («cuál» es «cual»), pero lo que entra en la consulta
        // es la palabra con su tilde: el índice pliega los acentos por su cuenta, y hacerlo dos
        // veces aquí solo escondería si esa parte funciona.
        if VACIAS.contains(&sin_tildes(&p).as_str()) {
            continue;
        }
        let tiene_digito = p.chars().any(|c| c.is_ascii_digit());
        if p.chars().count() < CORTA && !tiene_digito {
            continue;
        }
        if !salida.contains(&p) {
            salida.push(p);
        }
    }
    salida.join(" ")
}

fn sin_tildes(p: &str) -> String {
    p.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            c => c,
        })
        .collect()
}

/// Parte en palabras quitando todo lo que no sea letra o dígito, **después** de arreglar los
/// números. «ISO27001» se parte además en «iso» y «27001»: el documento puede traer cualquiera
/// de las dos formas y las dos se preguntan.
fn trocear(texto: &str) -> Vec<String> {
    let texto = juntar_millares(texto);
    let mut salida = Vec::new();
    let mut actual = String::new();
    let mut ultima_era_digito = false;

    let cerrar = |actual: &mut String, salida: &mut Vec<String>| {
        if !actual.is_empty() {
            salida.push(std::mem::take(actual));
        }
    };
    for c in texto.chars() {
        if c.is_alphanumeric() {
            // El salto letra↔dígito abre palabra: «ISO27001» → «ISO» + «27001».
            if !actual.is_empty() && c.is_ascii_digit() != ultima_era_digito {
                cerrar(&mut actual, &mut salida);
            }
            ultima_era_digito = c.is_ascii_digit();
            actual.push(c);
        } else {
            cerrar(&mut actual, &mut salida);
        }
    }
    cerrar(&mut actual, &mut salida);
    salida
}

/// Quita el separador de millares: `27.001` y `27,001` pasan a `27001`.
///
/// La regla es estrecha a propósito — **dígitos, separador, exactamente tres dígitos** — para no
/// tocar los decimales: `3.5` sigue siendo `3.5` y `1,5 millones` sigue siendo `1,5`.
fn juntar_millares(texto: &str) -> String {
    let cs: Vec<char> = texto.chars().collect();
    let mut salida = String::with_capacity(texto.len());
    let mut i = 0;
    while i < cs.len() {
        let separador = matches!(cs[i], '.' | ',');
        let antes_digito = i > 0 && cs[i - 1].is_ascii_digit();
        let tres_detras = cs.get(i + 1..i + 4).is_some_and(|t| t.iter().all(|c| c.is_ascii_digit()));
        let y_no_sigue_otro = !cs.get(i + 4).is_some_and(|c| c.is_ascii_digit());
        if separador && antes_digito && tres_detras && y_no_sigue_otro {
            i += 1; // se salta el separador; los tres dígitos se copian solos
            continue;
        }
        salida.push(cs[i]);
        i += 1;
    }
    salida
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// El defecto que la fase 3 vio en vivo: el motor escribe la norma con separador de millares.
    /// Las tres formas tienen que acabar preguntando por lo mismo.
    #[test]
    fn la_norma_mas_citada_se_encuentra_como_sea_que_la_escriba_el_transcriptor() {
        for dicho in ["ISO 27.001", "ISO 27,001", "ISO 27001", "ISO27001"] {
            let q = limpiar(dicho);
            assert!(q.contains("27001"), "«{dicho}» quedó como «{q}»");
            assert!(q.contains("iso"), "«{dicho}» perdió la sigla: «{q}»");
        }
    }

    /// Y el filo contrario, que es el que se rompe si la regla se escribe ancha: un decimal no
    /// es un separador de millares.
    #[test]
    fn un_decimal_no_es_un_separador_de_millares() {
        assert_eq!(juntar_millares("crecimos 3.5 veces"), "crecimos 3.5 veces");
        assert_eq!(juntar_millares("1,5 millones"), "1,5 millones");
        assert_eq!(juntar_millares("son 1.500 euros"), "son 1500 euros");
        // Dos separadores seguidos: 1.500.000 → 1500000
        assert_eq!(juntar_millares("1.500.000"), "1500000");
    }

    #[test]
    fn las_palabras_vacias_de_los_dos_idiomas_se_van() {
        assert_eq!(limpiar("¿Y cuál es el precio de la propuesta?"), "precio propuesta");
        assert_eq!(limpiar("What is the price of the proposal?"), "price proposal");
    }

    /// Las vacías se reconocen con tilde y sin ella —«cuál» y «cual» son la misma— pero la
    /// palabra que SÍ lleva contenido entra en la consulta tal como se dijo: plegar el acento es
    /// trabajo del índice, y hacerlo también aquí taparía si esa parte funciona.
    #[test]
    fn la_tilde_no_salva_a_una_palabra_vacia_ni_se_le_quita_a_una_llena() {
        assert_eq!(limpiar("¿Cuál método?"), "método");
        assert_eq!(limpiar("¿Cual metodo?"), "metodo");
    }

    #[test]
    fn los_signos_no_llegan_al_analizador_de_consultas() {
        let q = limpiar("¿Cumplen \"ISO 27001\"^2 y GDPR: sí o no?");
        assert!(!q.contains(['¿', '"', '^', ':', '?']), "quedó un signo en «{q}»");
    }

    #[test]
    fn una_palabra_no_se_pregunta_dos_veces() {
        assert_eq!(limpiar("precio, precio y precio"), "precio");
    }

    #[test]
    fn un_turno_larguisimo_se_corta_por_el_techo() {
        let largo = (0..80).map(|i| format!("palabra{i}")).collect::<Vec<_>>().join(" ");
        assert_eq!(limpiar(&largo).split_whitespace().count(), PALABRAS);
    }

    #[test]
    fn un_turno_sin_contenido_no_deja_consulta() {
        assert_eq!(limpiar("¿y de la, o el?"), "");
        assert_eq!(limpiar("   "), "");
    }
}
