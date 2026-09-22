//! LA FICHA — titular de ocho palabras, una línea y la fuente.
//!
//! *«Es lo primero que verás funcionar»*, dice la VISION. Y la forma exacta la fijó la maqueta:
//! un titular corto, una línea de evidencia y de dónde sale.
//!
//! **Todo lo que la ficha dice es texto del usuario.** El titular sale del título de su sección o
//! de la primera frase de su documento; la línea, de la frase de esa sección que más responde a
//! lo preguntado. La app **no redacta**: recorta y cita. Es la diferencia entre esta app y la
//! categoría con la que se la va a confundir, y es también lo que hace que este sprint pueda
//! entregar la ficha con cero LLM.
//!
//! **Cuándo NO hay ficha.** BM25 siempre devuelve algo: sobre un corpus de propuestas, cualquier
//! pregunta encuentra la sección «menos mala». Enseñarla como respuesta sería el fallo más caro
//! posible — una fuente concreta debajo de una respuesta equivocada se cree. Por eso el umbral no
//! es un puntaje (que no es comparable entre consultas) sino algo que se le puede explicar al
//! usuario: **la sección tiene que contener de verdad las palabras que se buscaron**.

pub mod maniobra;

use crate::corpus::{Hallazgo, Unidad};
use crate::disparo::Motivo;

/// Palabras que puede tener el titular. Lo fija la VISION y lo dibuja la maqueta.
pub const PALABRAS_DEL_TITULAR: usize = 8;

/// Cuántos hallazgos mira la ficha: el primero es la respuesta y los otros dos, las acumuladas
/// que aparecen al ampliar la banda.
pub const TOP: usize = 3;

/// Cuántas palabras distintas de la consulta tiene que traer la sección para que se considere
/// una respuesta y no la «menos mala». Con una sola palabra en común, una propuesta cualquiera
/// gana a la nada.
const TERMINOS_MINIMOS: usize = 2;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fuente {
    pub documento: String,
    pub seccion: Option<String>,
    pub unidad: Option<Unidad>,
    /// La sección era una conjetura del lector, no un título que alguien escribiera (PDF).
    pub conjeturada: bool,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Acumulada {
    pub unidad: Option<Unidad>,
    pub texto: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Ficha {
    pub titular: String,
    pub linea: String,
    /// La misma línea con más contexto, para la banda ampliada.
    pub linea_larga: String,
    pub fuente: Fuente,
    pub acumuladas: Vec<Acumulada>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Cercana {
    pub unidad: Option<Unidad>,
    pub texto: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase", tag = "clase")]
pub enum Respuesta {
    Ficha(Box<Ficha>),
    /// Ni una sección del corpus responde. Se dice **qué se buscó** —para que se vea en el acto
    /// si la app entendió mal—, lo más cercano que sí hay, y una maniobra del catálogo.
    SinResultado {
        buscado: String,
        cercanas: Vec<Cercana>,
        /// **Cuál** maniobra, no su texto: el copy es bilingüe y vive en el diccionario.
        maniobra: String,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Aparicion {
    #[serde(flatten)]
    pub respuesta: Respuesta,
    pub motivo: Motivo,
    /// Del fin de turno a la ficha, en milisegundos. El presupuesto del sprint es 4 000.
    pub ms: u64,
    pub hora: String,
}

/// Arma la ficha a partir de lo que devolvió el índice.
///
/// `pregunta` es el turno del cliente tal cual; se usa para elegir la línea y para decidir si
/// esto es una respuesta o un «no tengo nada».
pub fn armar(pregunta: &str, hallazgos: &[Hallazgo]) -> Respuesta {
    let consulta = crate::corpus::consulta::limpiar(pregunta);
    let terminos: Vec<&str> = consulta.split_whitespace().collect();

    let responde = |h: &Hallazgo| -> bool {
        let necesarios = TERMINOS_MINIMOS.min(terminos.len().max(1));
        cuantos_coinciden(&terminos, &format!("{} {}", h.seccion.clone().unwrap_or_default(), h.texto))
            >= necesarios
    };

    match hallazgos.iter().find(|h| responde(h)) {
        Some(mejor) => {
            let acumuladas = hallazgos
                .iter()
                .filter(|h| h.ruta != mejor.ruta || h.seccion != mejor.seccion)
                .take(TOP - 1)
                .map(|h| Acumulada { unidad: h.unidad, texto: recortar(&titular_de(h), PALABRAS_DEL_TITULAR) })
                .collect();
            Respuesta::Ficha(Box::new(Ficha {
                titular: recortar(&titular_de(mejor), PALABRAS_DEL_TITULAR),
                linea: recortar(&linea_de(mejor, &terminos), 14),
                linea_larga: recortar(&linea_de(mejor, &terminos), 30),
                fuente: Fuente {
                    documento: mejor.documento.clone(),
                    seccion: mejor.seccion.clone(),
                    unidad: mejor.unidad,
                    conjeturada: mejor.conjeturado,
                },
                acumuladas,
            }))
        }
        None => {
            Respuesta::SinResultado {
                buscado: consulta,
                cercanas: hallazgos
                    .iter()
                    .take(TOP)
                    .map(|h| Cercana {
                        unidad: h.unidad,
                        texto: recortar(&titular_de(h), PALABRAS_DEL_TITULAR),
                    })
                    .collect(),
                maniobra: maniobra::elegir(pregunta).id.to_string(),
            }
        }
    }
}

/// El titular sale del **título de la sección** cuando lo hay, y si no de la primera frase del
/// texto. Nunca se redacta.
fn titular_de(h: &Hallazgo) -> String {
    match &h.seccion {
        Some(s) if !s.trim().is_empty() => s.clone(),
        // Sin sección —un PDF sin estructura— el titular es la primera frase, que es lo más
        // parecido a un título que el documento ofrece.
        _ => primera_frase(&h.texto),
    }
}

/// La línea es la **frase de la sección que más palabras de la pregunta trae**. No la primera:
/// la primera suele ser una introducción, y la que responde está tres frases más abajo.
fn linea_de(h: &Hallazgo, terminos: &[&str]) -> String {
    partir_en_frases(&h.texto)
        .into_iter()
        .max_by_key(|f| cuantos_coinciden(terminos, f))
        .unwrap_or_else(|| h.texto.clone())
}

fn primera_frase(texto: &str) -> String {
    partir_en_frases(texto).into_iter().next().unwrap_or_default()
}

fn partir_en_frases(texto: &str) -> Vec<String> {
    let mut salida = Vec::new();
    let mut actual = String::new();
    let cs: Vec<char> = texto.chars().collect();
    for (i, &c) in cs.iter().enumerate() {
        actual.push(c);
        let entre_digitos = c == '.'
            && i > 0
            && cs[i - 1].is_ascii_digit()
            && cs.get(i + 1).is_some_and(|s| s.is_ascii_digit());
        if matches!(c, '.' | '!' | '?' | '\n') && !entre_digitos {
            let t = actual.trim().to_string();
            if !t.is_empty() {
                salida.push(t);
            }
            actual.clear();
        }
    }
    let t = actual.trim().to_string();
    if !t.is_empty() {
        salida.push(t);
    }
    salida
}

/// Cuántas palabras **distintas** de la consulta aparecen en el texto. Distintas importa: una
/// sección que repite «precio» diez veces no responde mejor que una que dice «precio» y «plazo».
fn cuantos_coinciden(terminos: &[&str], texto: &str) -> usize {
    let plano = sin_tildes(&texto.to_lowercase());
    let palabras: Vec<&str> = plano.split_whitespace().collect();
    let mut vistos: Vec<&str> = Vec::new();
    for t in terminos {
        let t_plano = sin_tildes(t);
        if vistos.contains(t) {
            continue;
        }
        // Prefijo y no igualdad: el índice hace stemming y aquí no hay stemmer, así que
        // «canales» tiene que poder reconocer «canal» sin arrastrar media librería.
        if palabras.iter().any(|p| {
            let p = sin_tildes(&p.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase());
            p.starts_with(&t_plano) || t_plano.starts_with(&p) && p.chars().count() >= 4
        }) {
            vistos.push(t);
        }
    }
    vistos.len()
}

fn sin_tildes(p: &str) -> String {
    p.chars()
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            c => c,
        })
        .collect()
}

/// Recorta a `cuantas` palabras. Si recorta, lo dice con puntos suspensivos: media frase sin
/// marca se lee como una frase entera que dice otra cosa.
fn recortar(texto: &str, cuantas: usize) -> String {
    let palabras: Vec<&str> = texto.split_whitespace().collect();
    if palabras.len() <= cuantas {
        return palabras.join(" ");
    }
    format!("{}…", palabras[..cuantas].join(" "))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn h(documento: &str, seccion: Option<&str>, texto: &str, puntaje: f32) -> Hallazgo {
        Hallazgo {
            documento: documento.into(),
            ruta: format!("/c/{documento}.md"),
            seccion: seccion.map(str::to_string),
            unidad: Some(Unidad::Propuesta),
            texto: texto.into(),
            puntaje,
            conjeturado: false,
        }
    }

    #[test]
    fn el_titular_sale_del_documento_y_cabe_en_ocho_palabras() {
        let r = armar("¿cuál es el precio y el plazo?", &[h(
            "Propuesta Páramo Azul",
            Some("Precio y plazo de entrega para el proyecto completo de canales"),
            "El precio es cerrado. El plazo de entrega es de cuatro semanas.",
            9.0,
        )]);
        let Respuesta::Ficha(f) = r else { panic!("no armó ficha") };
        assert!(f.titular.split_whitespace().count() <= PALABRAS_DEL_TITULAR, "titular: {}", f.titular);
        assert!(f.titular.ends_with('…'), "recortó sin decirlo: {}", f.titular);
        assert!(f.titular.starts_with("Precio y plazo"));
    }

    /// La línea que responde suele estar debajo de la introducción, no encima.
    #[test]
    fn la_linea_es_la_frase_que_responde_y_no_la_primera() {
        let r = armar("¿cuántas semanas de plazo?", &[h(
            "Propuesta",
            Some("Plazo"),
            "Este apartado describe el enfoque general del trabajo. \
             La entrega completa toma cuatro semanas desde la firma.",
            9.0,
        )]);
        let Respuesta::Ficha(f) = r else { panic!("no armó ficha") };
        assert!(f.linea.contains("cuatro semanas"), "eligió la introducción: {}", f.linea);
    }

    /// El fallo más caro de todos: una fuente concreta debajo de una respuesta equivocada.
    #[test]
    fn una_seccion_que_no_responde_no_se_enseña_como_si_respondiera() {
        // BM25 devuelve algo, pero no tiene nada que ver con lo preguntado.
        let r = armar("¿tienen certificación ISO 27001?", &[h(
            "Propuesta",
            Some("Alcance"),
            "Tres canales y una línea base de doce meses.",
            2.0,
        )]);
        match r {
            Respuesta::SinResultado { buscado, cercanas, .. } => {
                assert!(buscado.contains("27001"), "no dice qué buscó: {buscado}");
                assert_eq!(cercanas.len(), 1, "se perdió lo más cercano que sí hay");
            }
            Respuesta::Ficha(f) => panic!("enseñó como respuesta una sección que no responde: {f:?}"),
        }
    }

    #[test]
    fn sin_resultado_trae_su_maniobra_del_catalogo() {
        let r = armar("¿tienen certificación ISO 27001?", &[]);
        let Respuesta::SinResultado { maniobra, cercanas, .. } = r else { panic!("armó ficha de la nada") };
        assert_eq!(maniobra, "credencial");
        assert!(cercanas.is_empty());
    }

    #[test]
    fn las_acumuladas_son_las_otras_dos_y_no_repiten_la_primera() {
        let r = armar("precio del plazo de entrega", &[
            h("A", Some("Precio"), "El precio del plazo de entrega es cerrado.", 9.0),
            h("B", Some("Plazo"), "El plazo de entrega y su precio se pactan.", 5.0),
            h("C", Some("Otro"), "También el precio del plazo de entrega aparece.", 3.0),
            h("D", Some("Cuarto"), "Y el precio del plazo de entrega otra vez.", 1.0),
        ]);
        let Respuesta::Ficha(f) = r else { panic!("no armó ficha") };
        assert_eq!(f.acumuladas.len(), TOP - 1);
        assert!(!f.acumuladas.iter().any(|a| a.texto == f.titular));
    }

    /// Un PDF sin estructura no tiene título de sección. La ficha sigue existiendo, con el
    /// titular sacado de la primera frase, y **declara que la sección era conjetura**.
    #[test]
    fn un_documento_sin_secciones_da_ficha_igual_y_lo_declara() {
        let mut sin = h("Acta escaneada", None, "El precio acordado fue cerrado. Nada más.", 8.0);
        sin.conjeturado = true;
        let Respuesta::Ficha(f) = armar("¿cuál fue el precio acordado?", &[sin]) else {
            panic!("no armó ficha")
        };
        assert_eq!(f.fuente.seccion, None);
        assert!(f.fuente.conjeturada);
        assert!(f.titular.starts_with("El precio acordado"));
    }

    #[test]
    fn la_linea_larga_trae_mas_contexto_que_la_corta() {
        let largo = "El plazo de entrega comprometido para la totalidad del alcance descrito en \
                     este documento es de cuatro semanas contadas desde la firma del contrato.";
        let Respuesta::Ficha(f) = armar("¿cuál es el plazo de entrega?", &[h("A", Some("Plazo"), largo, 9.0)])
        else {
            panic!("no armó ficha")
        };
        assert!(f.linea_larga.chars().count() > f.linea.chars().count());
        assert!(f.linea.ends_with('…'));
    }

    /// Sin esto, «canales» no reconocería «canal» y la ficha se declararía sin resultado sobre
    /// un documento que sí responde. El índice hace stemming; este contador no puede quedarse
    /// atrás.
    #[test]
    fn el_contador_de_terminos_aguanta_el_singular_y_el_plural() {
        assert_eq!(cuantos_coinciden(&["canales", "precio"], "El canal y su precio"), 2);
        assert_eq!(cuantos_coinciden(&["metodologia"], "Nuestra metodología"), 1);
    }
}
