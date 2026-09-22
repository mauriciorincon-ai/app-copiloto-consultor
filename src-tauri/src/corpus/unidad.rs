//! LAS CINCO UNIDADES del modelo de consultoría.
//!
//! La VISION dice que el corpus «entiende cinco unidades» y que *«la pregunta del cliente cae en
//! la unidad correcta»*. Eso obliga a clasificar cada documento al indexarlo, y a hacerlo **sin
//! modelo**: reglas léxicas en los dos idiomas de la app, sobre el nombre del archivo y el
//! principio del texto.
//!
//! Dos decisiones que se pagan aquí y se cobran en la pantalla:
//!
//! 1. **Existe una sexta respuesta: ninguna.** `Unidad::clasificar` devuelve `None` cuando nada
//!    marca lo suficiente, y la maqueta ya dibujó esa fila («sin unidad»). Forzar los cinco
//!    cajones significaría que un documento cae en el equivocado y la ficha cite una fuente que
//!    no es — mentir con más confianza que callar.
//! 2. **El nombre del archivo pesa más que el cuerpo.** Quien guarda «Propuesta Páramo Azul.pdf»
//!    ya clasificó el documento; el cuerpo solo confirma. Un caso que menciona veinte veces la
//!    palabra «propuesta» sigue siendo un caso.

use serde::{Deserialize, Serialize};

/// Cuánto tiene que sacar la unidad ganadora sobre la segunda para que se declare.
///
/// Con 1.0, un documento que reparte señales entre dos unidades se queda sin clasificar en vez de
/// caer en la que ganó por un punto. Es el mismo criterio que el resto de la app: el empate no se
/// resuelve inventando, se declara.
const VENTAJA: f32 = 1.0;

/// El nombre del archivo vale por tres apariciones en el cuerpo.
const PESO_DEL_NOMBRE: f32 = 3.0;

/// Cuánto texto se mira. Las señales de unidad viven al principio —portada, título, resumen—; el
/// resto del documento habla del tema, no de qué clase de documento es.
const CABEZA: usize = 1_500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Unidad {
    Propuesta,
    Marco,
    Caso,
    Cliente,
    Perfil,
}

impl Unidad {
    pub const TODAS: [Unidad; 5] = [
        Unidad::Propuesta,
        Unidad::Marco,
        Unidad::Caso,
        Unidad::Cliente,
        Unidad::Perfil,
    ];

    /// La palabra que la banda enseña en el chip de la ficha. En español, como el resto del
    /// índice: la unidad es una etiqueta del modelo de consultoría, no texto de interfaz.
    pub fn etiqueta(self) -> &'static str {
        match self {
            Unidad::Propuesta => "propuesta",
            Unidad::Marco => "marco",
            Unidad::Caso => "caso",
            Unidad::Cliente => "cliente",
            Unidad::Perfil => "perfil",
        }
    }

    pub fn de_etiqueta(s: &str) -> Option<Unidad> {
        Unidad::TODAS.into_iter().find(|u| u.etiqueta() == s)
    }

    /// Las marcas léxicas, en los dos idiomas. Una app bilingüe clasifica en los dos o solo
    /// clasifica los documentos de la mitad de sus usuarios.
    fn marcas(self) -> &'static [&'static str] {
        match self {
            Unidad::Propuesta => &[
                "propuesta", "proposal", "cotizacion", "quote", "oferta", "offer",
                "alcance", "scope", "entregables", "deliverables", "supuestos",
                "assumptions", "honorarios", "fees", "sow",
            ],
            Unidad::Marco => &[
                "marco", "framework", "metodologia", "methodology", "metodo", "method",
                "etapas", "stages", "fases", "phases", "modelo", "model", "playbook",
            ],
            Unidad::Caso => &[
                "caso", "case", "cierre", "closeout", "resultados", "results",
                "aprendizajes", "lessons", "retrospectiva", "retrospective",
                "implementacion", "implementation", "kickoff",
            ],
            Unidad::Cliente => &[
                "cliente", "client", "cuenta", "account", "acta", "minutes",
                "reunion", "meeting", "contacto", "contact", "nda", "acuerdo", "agreement",
            ],
            Unidad::Perfil => &[
                "perfil", "profile", "hoja de vida", "cv", "curriculum", "resume",
                "trayectoria", "track record", "bio", "biografia", "credenciales",
                "credentials", "certificaciones", "certifications",
            ],
        }
    }

    /// Clasifica un documento. `None` es una respuesta legítima, no una avería.
    pub fn clasificar(nombre: &str, texto: &str) -> Option<Unidad> {
        let nombre = normalizar(nombre);
        let cuerpo = normalizar(&texto.chars().take(CABEZA).collect::<String>());

        let mut puntajes: Vec<(Unidad, f32)> = Unidad::TODAS
            .into_iter()
            .map(|u| {
                let n: f32 = u
                    .marcas()
                    .iter()
                    .map(|m| veces(&nombre, m) as f32 * PESO_DEL_NOMBRE + veces(&cuerpo, m) as f32)
                    .sum();
                (u, n)
            })
            .collect();
        // Desempate estable por el orden de `TODAS`: dos corridas sobre el mismo documento tienen
        // que dar la misma unidad, y `sort_by` sin criterio total no lo garantiza.
        puntajes.sort_by(|a, b| b.1.total_cmp(&a.1));

        let (mejor, punta) = puntajes[0];
        let segunda = puntajes[1].1;
        if punta > 0.0 && punta - segunda >= VENTAJA {
            Some(mejor)
        } else {
            None
        }
    }
}

/// Minúsculas y sin tildes, para que «Metodología» y «metodologia» sean la misma marca. Es la
/// misma normalización que usa el detector de eco, y por la misma razón.
fn normalizar(texto: &str) -> String {
    texto
        .chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            c if c.is_alphanumeric() || c == ' ' => c,
            _ => ' ',
        })
        .collect()
}

/// Cuenta apariciones de `aguja` **como palabra**, no como subcadena: sin esto «casos» y
/// «casona» alimentarían por igual a la unidad «caso», y «marco» se dispararía dentro de
/// «marcó» o de un apellido.
fn veces(pajar: &str, aguja: &str) -> usize {
    if aguja.contains(' ') {
        return pajar.matches(aguja).count();
    }
    pajar.split_whitespace().filter(|p| *p == aguja).count()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_nombre_del_archivo_decide_cuando_el_cuerpo_habla_de_otra_cosa() {
        // Un CASO de cierre que menciona la propuesta original varias veces. Quien lo guardó ya
        // dijo qué es; el cuerpo no debería poder llevarle la contraria.
        let cuerpo = "Revisamos la propuesta inicial. La propuesta contemplaba tres canales. \
                      Frente a la propuesta, el alcance final fue mayor.";
        assert_eq!(
            Unidad::clasificar("Cooperativa Sur del Valle · cierre de caso.md", cuerpo),
            Some(Unidad::Caso)
        );
    }

    #[test]
    fn clasifica_en_los_dos_idiomas() {
        assert_eq!(
            Unidad::clasificar("Channel profitability proposal.docx", "Scope and deliverables."),
            Some(Unidad::Propuesta)
        );
        assert_eq!(
            Unidad::clasificar("Data adoption framework.md", "Four stages of adoption."),
            Some(Unidad::Marco)
        );
    }

    #[test]
    fn las_tildes_no_cambian_la_unidad() {
        assert_eq!(
            Unidad::clasificar("Metodología de adopción.md", ""),
            Unidad::clasificar("Metodologia de adopcion.md", "")
        );
    }

    /// La sexta respuesta. Un documento sin marcas se queda fuera de los cinco cajones y la
    /// pantalla lo dice: la maqueta dibujó esa fila a propósito.
    #[test]
    fn un_documento_sin_marcas_se_queda_sin_unidad() {
        assert_eq!(Unidad::clasificar("notas sueltas.md", "Ideas varias del martes."), None);
    }

    /// Y el empate también. Si dos unidades van igualadas, declarar la ganadora sería inventar.
    #[test]
    fn un_empate_se_declara_en_vez_de_resolverse() {
        // Una marca de cada una, las dos en el cuerpo: 1.0 contra 1.0.
        assert_eq!(Unidad::clasificar("documento.md", "El alcance y la metodologia."), None);
    }

    /// Sin esto, «casona» sumaba a «caso» y «marcó» a «marco».
    #[test]
    fn las_marcas_se_cuentan_como_palabras_y_no_como_trozos() {
        assert_eq!(veces("la casona del caso", "caso"), 1);
        assert_eq!(veces("marcos marco marcado", "marco"), 1);
    }

    #[test]
    fn la_etiqueta_va_y_vuelve() {
        for u in Unidad::TODAS {
            assert_eq!(Unidad::de_etiqueta(u.etiqueta()), Some(u));
        }
        assert_eq!(Unidad::de_etiqueta("propuestas"), None);
    }
}
