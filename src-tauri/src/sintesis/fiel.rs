//! **¿Dice la línea lo que dice la ficha?** — la segunda mitad del grounding.
//!
//! Que la sugerencia cite `F2` no basta: la primera medida con el modelo del sistema (bitácora,
//! fase 6) citó fichas válidas y aun así dijo «el taller de cierre va aparte» cuando la propuesta
//! lo incluye, y pegó a la ficha de Calidad una frase de otro documento. Una cita válida con una
//! línea inventada es peor que ninguna: el consultor se la repite al cliente.
//!
//! La regla es léxica, determinista y deliberadamente estricta —descartar deja la ficha sola, que
//! es el camino de siempre—:
//!
//! 1. **Cada palabra con contenido de la línea sale de la ficha citada** (su titular y su línea, lo
//!    único que el modelo vio de ella). Se compara sin tildes ni mayúsculas y por raíz corta (las
//!    cinco primeras letras), para que «incluye» case con «incluida» y «fuentes» con «fuente».
//!    Ni una de sobra: la palabra que invierte el sentido suele ser una sola («aparte»).
//! 2. **Las cifras, exactas**, escritas con dígitos o con palabras («cuatro» es `4`).
//! 3. **Ninguna negación nueva, ninguna negación perdida:** si la línea niega y la ficha no, o si la
//!    frase de la ficha que la línea repite niega y la línea no.
//!
//! Lo que NO ve, declarado: un orden cambiado con las mismas palabras («A paga a B» por «B paga a
//! A») y un sinónimo legítimo («dentro» por «incluida»), que descarta de más. Lo primero es raro en
//! una frase de treinta palabras con cifras exactas; lo segundo cuesta una sugerencia, no una
//! falsedad.

use crate::ficha::Respaldo;

/// Palabras de cuatro letras o más que no afirman nada: artículos, auxiliares, conectores.
const VACIAS: &[&str] = &[
    // español
    "esta", "este", "esto", "estos", "estas", "estan", "estar", "sera", "seran", "seria", "sido",
    "para", "pero", "como", "cuando", "donde", "desde", "hasta", "entre", "sobre", "tiene", "tienen",
    "tengo", "tenemos", "hace", "hacen", "puede", "pueden", "cada", "todo", "toda", "todos", "todas",
    "otro", "otra", "otros", "otras", "mismo", "misma", "tambien", "porque", "segun", "sino", "solo",
    "solamente", "aqui", "ahora", "usted", "ustedes", "nuestro", "nuestra", "nuestros", "nuestras",
    "nosotros", "ellos", "ellas", "eso", "esos", "esas", "aquel", "cual", "cuales", "quien", "quienes",
    "cuanto", "cuantos", "cuanta", "cuantas", "haya", "habia", "hemos", "han", "sus", "unos", "unas",
    "muy", "mas", "menos", "bien", "asi", "luego", "pues", "dice", "decir",
    // inglés
    "that", "this", "these", "those", "with", "from", "have", "been", "were", "will", "would",
    "could", "should", "into", "about", "your", "their", "there", "they", "them", "what", "which",
    "also", "only", "just", "than", "then", "does", "each", "every", "some", "more", "most", "very",
    "here", "over", "under", "when", "where", "while", "because", "being", "such", "both", "yours",
];

/// Lo que la línea le dice al CONSULTOR («confirma que…», «recuérdale…»): no es una afirmación
/// sobre el proyecto, así que no tiene que salir de la ficha.
const AL_CONSULTOR: &[&str] = &[
    "confirma", "confirmale", "dile", "explica", "explicale", "aclara", "aclarale", "menciona",
    "recuerda", "recuerdale", "ofrece", "ofrecele", "propon", "proponle", "responde", "respondele",
    "senala", "indica", "comenta", "subraya", "puedes", "podrias", "conviene", "confirm", "tell",
    "explain", "clarify", "mention", "remind", "offer", "propose", "answer", "point", "note",
];

const NEGACIONES: &[&str] = &[
    "no", "sin", "ningun", "ninguna", "ninguno", "nunca", "jamas", "tampoco", "ni", "not", "never",
    "none", "without", "nor", "cannot",
];

const CIFRAS: &[(&str, &str)] = &[
    ("cero", "0"), ("uno", "1"), ("dos", "2"), ("tres", "3"), ("cuatro", "4"), ("cinco", "5"),
    ("seis", "6"), ("siete", "7"), ("ocho", "8"), ("nueve", "9"), ("diez", "10"), ("once", "11"),
    ("doce", "12"), ("quince", "15"), ("veinte", "20"), ("treinta", "30"), ("cuarenta", "40"),
    ("cincuenta", "50"), ("cien", "100"), ("zero", "0"), ("one", "1"), ("two", "2"), ("three", "3"),
    ("four", "4"), ("five", "5"), ("six", "6"), ("seven", "7"), ("eight", "8"), ("nine", "9"),
    ("ten", "10"), ("eleven", "11"), ("twelve", "12"), ("fifteen", "15"), ("twenty", "20"),
    ("thirty", "30"), ("forty", "40"), ("fifty", "50"), ("hundred", "100"),
];

/// Minúsculas, sin tildes, y «n't» como «not».
fn normalizar(texto: &str) -> String {
    texto
        .to_lowercase()
        .replace("n't", " not")
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' => 'a',
            'é' | 'è' | 'ë' => 'e',
            'í' | 'ì' | 'ï' => 'i',
            'ó' | 'ò' | 'ö' => 'o',
            'ú' | 'ù' | 'ü' => 'u',
            'ñ' => 'n',
            otro => otro,
        })
        .collect()
}

fn palabras(texto: &str) -> Vec<String> {
    texto.split(|c: char| !c.is_alphanumeric()).filter(|p| !p.is_empty()).map(String::from).collect()
}

fn cifra(p: &str) -> Option<String> {
    if p.chars().all(|c| c.is_ascii_digit()) {
        let sin_ceros = p.trim_start_matches('0');
        return Some(if sin_ceros.is_empty() { "0" } else { sin_ceros }.to_string());
    }
    CIFRAS.iter().find(|(w, _)| *w == p).map(|(_, d)| d.to_string())
}

fn niega(palabras: &[String]) -> bool {
    palabras.iter().any(|p| NEGACIONES.contains(&p.as_str()))
}

/// Dos palabras son la misma si comparten las cinco primeras letras —o todas, si una es más corta,
/// con un mínimo de cuatro—.
fn misma_raiz(a: &str, b: &str) -> bool {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let k = 5.min(a.len()).min(b.len());
    k >= 4 && a[..k] == b[..k]
}

/// **La regla.** `true` si la línea solo dice lo que dice la ficha.
pub fn dice_lo_que_la_ficha(linea: &str, ficha: &Respaldo) -> bool {
    cabe_en(linea, &format!("{}. {}", ficha.titular, ficha.linea))
}

/// El titular puede repetir además la pregunta del cliente («¿Cuántas fuentes?»). Si tampoco sale
/// de ahí, [`super::fundar`] no descarta la sugerencia —la afirmación es la línea—: le pone el
/// titular de la ficha.
pub fn titular_de_la_ficha_o_el_turno(titular: &str, ficha: &Respaldo, turno: &str) -> bool {
    cabe_en(titular, &format!("{}. {}. {}", ficha.titular, ficha.linea, turno))
}

fn cabe_en(linea: &str, fuente: &str) -> bool {
    let de_la_linea = palabras(&normalizar(linea));
    let texto_de_la_ficha = normalizar(fuente);
    let de_la_ficha = palabras(&texto_de_la_ficha);
    let cifras_de_la_ficha: Vec<String> = de_la_ficha.iter().filter_map(|p| cifra(p)).collect();

    // 1 y 2: cada palabra con contenido, y cada cifra, salen de la ficha.
    for p in &de_la_linea {
        if let Some(c) = cifra(p) {
            if !cifras_de_la_ficha.contains(&c) {
                return false;
            }
            continue;
        }
        let con_contenido = p.chars().count() >= 4
            && !VACIAS.contains(&p.as_str())
            && !AL_CONSULTOR.contains(&p.as_str())
            && !NEGACIONES.contains(&p.as_str());
        if con_contenido && !de_la_ficha.iter().any(|f| misma_raiz(p, f)) {
            return false;
        }
    }

    // 3: las negaciones. La frase de la ficha que la línea repite es la que más palabras comparte.
    let niega_la_linea = niega(&de_la_linea);
    if niega_la_linea && !niega(&de_la_ficha) {
        return false;
    }
    let repetida = texto_de_la_ficha
        .split(['.', ';', ':', '!', '?'])
        .map(palabras)
        .max_by_key(|frase| {
            de_la_linea
                .iter()
                .filter(|p| p.chars().count() >= 4 && frase.iter().any(|f| misma_raiz(p, f)))
                .count()
        })
        .unwrap_or_default();
    !(niega(&repetida) && !niega_la_linea)
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::ficha::Fuente;

    fn ficha(titular: &str, linea: &str) -> Respaldo {
        Respaldo {
            titular: titular.into(),
            linea: linea.into(),
            fuente: Fuente { documento: "Propuesta".into(), seccion: None, unidad: None, conjeturada: false },
        }
    }

    fn precio() -> Respaldo {
        ficha(
            "Precio",
            "Tarifa cerrada de cuatro semanas. Incluye el taller de cierre y dos rondas de revisión.",
        )
    }

    #[test]
    fn lo_que_la_ficha_dice_con_otras_palabras_pasa() {
        let f = precio();
        for buena in [
            "La tarifa es cerrada, de cuatro semanas.",
            "Incluye el taller de cierre y dos rondas de revisión.",
            "Confirma que el taller de cierre está incluido.",
            "Son 4 semanas con tarifa cerrada.",
        ] {
            assert!(dice_lo_que_la_ficha(buena, &f), "descartó «{buena}», que la ficha dice");
        }
        let en = ficha("Pricing", "Four-week fixed fee. It includes the closing workshop and two review rounds.");
        assert!(dice_lo_que_la_ficha("The fixed fee includes two review rounds.", &en));
        assert!(!dice_lo_que_la_ficha("The closing workshop is billed separately.", &en));
    }

    /// **Los casos de la primera medida con el modelo de verdad** (bitácora, fase 6): cita válida,
    /// línea que la ficha no dice.
    #[test]
    fn una_palabra_que_la_ficha_no_dice_descarta_la_linea() {
        let entregables = ficha(
            "Entregables",
            "Un tablero de margen por canal, el cuaderno de limpieza reproducible y un taller de cierre de tres horas.",
        );
        assert!(!dice_lo_que_la_ficha("El taller de cierre de tres horas va aparte.", &entregables));
        let calidad = ficha("Calidad", "Se acuerdan cinco reglas por fuente y se miden semanalmente.");
        assert!(!dice_lo_que_la_ficha(
            "La calidad se mide semanalmente y no hay penalidades por terminación anticipada.",
            &calidad
        ));
        let gobierno = ficha("Gobierno", "Define quién responde por cada fuente y cada cuánto se revisa.");
        assert!(!dice_lo_que_la_ficha("Cada fuente responde a su propia decisión.", &gobierno));
    }

    #[test]
    fn el_titular_puede_repetir_la_pregunta_pero_no_inventar() {
        let turno = "¿Cuántas rondas de revisión incluye?";
        assert!(titular_de_la_ficha_o_el_turno("¿Cuántas rondas?", &precio(), turno));
        assert!(titular_de_la_ficha_o_el_turno("Rondas de revisión", &precio(), turno));
        for inventado in ["Claro", "Fourth Source", "El taller va aparte"] {
            assert!(!titular_de_la_ficha_o_el_turno(inventado, &precio(), turno), "«{inventado}» pasó");
        }
    }

    #[test]
    fn una_cifra_distinta_descarta_la_linea_aunque_se_escriba_con_letras() {
        let f = precio();
        assert!(!dice_lo_que_la_ficha("La tarifa es cerrada de seis semanas.", &f));
        assert!(!dice_lo_que_la_ficha("Tarifa cerrada de 5 semanas.", &f));
        assert!(!dice_lo_que_la_ficha("Incluye tres rondas de revisión.", &f));
    }

    #[test]
    fn la_negacion_no_se_pone_ni_se_quita() {
        assert!(!dice_lo_que_la_ficha("No incluye el taller de cierre.", &precio()));
        assert!(!dice_lo_que_la_ficha("La tarifa no incluye revisión.", &precio()));
        let iso = ficha("Certificaciones", "No hay certificación ISO vigente. Las prácticas siguen la norma.");
        assert!(dice_lo_que_la_ficha("No hay certificación ISO vigente.", &iso));
        assert!(!dice_lo_que_la_ficha("Hay certificación ISO vigente.", &iso));
        // La otra frase de la ficha no niega: repetirla sin negación está bien.
        assert!(dice_lo_que_la_ficha("Las prácticas siguen la norma.", &iso));
    }
}
