//! EL CATÁLOGO DE MANIOBRAS — qué sugerir cuando el corpus no tiene nada.
//!
//! Seis maneras de conducirse, escritas por personas y versionadas aquí, elegidas por reglas
//! léxicas sobre lo que preguntó el cliente. El texto es **literalmente** el de
//! `design-system.md` §9-quinquies, aprobado en la mirada 11: este módulo no lo reescribe, lo
//! ejecuta. Si el catálogo cambia allí, cambia aquí, y el test que los compara palabra por
//! palabra es lo que impide que se separen en silencio.
//!
//! **Las maniobras hablan de cómo conducirse, jamás del negocio del usuario.** Por eso pueden ser
//! fijas y por eso no son una funcionalidad de IA: cero tokens, cero red. Cuando llegue la
//! síntesis con modelo (sprint 2), este catálogo es su fallback permanente — lo pide la regla del
//! código primero.
//!
//! **Deuda declarada y aprobada por el usuario (mirada 11, segunda vuelta):** la sexta maniobra
//! —la genérica— *«deja solo al consultor»*, y es la que más se va a disparar. El requisito
//! escrito para el sprint 2 es «sin inventar respuesta, pero a medida de la situación», y el
//! camino determinista está en `design-system.md` §10.

/// Una maniobra del catálogo: el texto que ve el usuario y la razón por la que es esa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Maniobra {
    pub texto: &'static str,
    pub porque: &'static str,
}

/// Las marcas que eligen cada maniobra, sin tildes: se comparan contra texto ya normalizado.
const CATALOGO: &[(&[&str], Maniobra)] = &[
    (
        &["certificacion", "certificado", "certificados", "iso", "acreditado", "acreditacion",
          "licencia", "certification", "certified", "accredited", "license", "licence"],
        Maniobra {
            texto: "Dilo sin adornos y ofrece confirmarlo hoy mismo.",
            porque: "una credencial se tiene o no; dudar cuesta más que el «no»",
        },
    ),
    (
        &["cuanto", "cuanta", "precio", "precios", "costo", "costos", "coste", "descuento",
          "tarifa", "tarifas", "presupuesto", "price", "cost", "discount", "fee", "fees",
          "budget", "rate"],
        Maniobra {
            texto: "No improvises cifras: ofrece el rango del caso comparable.",
            porque: "un número dicho al aire se vuelve compromiso",
        },
    ),
    (
        &["cuando", "plazo", "plazos", "semanas", "meses", "entrega", "cronograma", "when",
          "timeline", "weeks", "months", "delivery", "deadline", "schedule"],
        Maniobra {
            texto: "Da el plazo del caso más parecido y confírmalo por escrito.",
            porque: "anclar en un caso real es defendible; una fecha inventada, no",
        },
    ),
    (
        &["referencia", "referencias", "referencias", "trabajado", "clientes", "sector",
          "reference", "references", "worked", "customers", "clients"],
        Maniobra {
            texto: "Ofrece una referencia del sector sin nombrar al cliente aún.",
            porque: "nombrar clientes sin permiso es un problema, no una venta",
        },
    ),
    (
        &["contrato", "contractual", "clausula", "clausulas", "penalidad", "penalizacion",
          "nda", "confidencialidad", "contract", "clause", "penalty", "liability"],
        Maniobra {
            texto: "No opines de contrato en vivo: anótalo y respóndelo por escrito.",
            porque: "lo contractual no se improvisa en una llamada",
        },
    ),
];

/// La sexta: la que sale cuando ninguna marca aparece.
pub const GENERICA: Maniobra = Maniobra {
    texto: "Devuelve la pregunta: ¿para qué lo necesitan?",
    porque: "la pregunta real suele ser otra — y da tiempo",
};

/// Elige la maniobra. El orden del catálogo decide los empates, y es el mismo del design system:
/// una pregunta que mezcla precio y plazo se responde por el precio, que es lo que compromete.
pub fn elegir(pregunta: &str) -> Maniobra {
    let normalizada = normalizar(pregunta);
    let palabras: Vec<&str> = normalizada.split_whitespace().collect();
    for (marcas, maniobra) in CATALOGO {
        if palabras.iter().any(|p| marcas.contains(p)) {
            return *maniobra;
        }
    }
    GENERICA
}

/// Cuántas maniobras hay, para que la pantalla pueda decir «1 de 6» y para que el test que
/// compara con el design system sepa cuántas buscar.
pub const CUANTAS: usize = 6;

pub fn todas() -> Vec<Maniobra> {
    CATALOGO.iter().map(|(_, m)| *m).chain(std::iter::once(GENERICA)).collect()
}

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
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn cada_marca_lleva_a_su_maniobra() {
        assert!(elegir("¿Tienen certificación ISO 27001?").texto.starts_with("Dilo sin adornos"));
        assert!(elegir("¿Y cuánto costaría eso?").texto.starts_with("No improvises cifras"));
        assert!(elegir("¿En cuántas semanas lo entregan?").texto.starts_with("Da el plazo"));
        assert!(elegir("¿Con qué clientes del sector han trabajado?").texto.starts_with("Ofrece una referencia"));
        assert!(elegir("¿Qué pasa con la cláusula de penalidad?").texto.starts_with("No opines de contrato"));
    }

    #[test]
    fn elige_igual_en_ingles() {
        assert!(elegir("Are you ISO certified?").texto.starts_with("Dilo sin adornos"));
        assert!(elegir("What is the cost?").texto.starts_with("No improvises cifras"));
        assert!(elegir("How many weeks for delivery?").texto.starts_with("Da el plazo"));
    }

    #[test]
    fn lo_que_no_encaja_cae_en_la_generica() {
        assert_eq!(elegir("¿Y ustedes dónde quedan?"), GENERICA);
        assert_eq!(elegir(""), GENERICA);
    }

    /// Una pregunta que mezcla dos marcas se resuelve por el orden del catálogo, y ese orden es
    /// el del design system. Sin esto, dos corridas podrían dar maniobras distintas.
    #[test]
    fn un_empate_lo_resuelve_el_orden_del_catalogo_y_no_el_azar() {
        let mezcla = "¿Cuánto cuesta y en cuántas semanas lo entregan?";
        assert_eq!(elegir(mezcla), elegir(mezcla));
        assert!(elegir(mezcla).texto.starts_with("No improvises cifras"));
    }

    #[test]
    fn son_seis_y_ninguna_se_repite() {
        let t = todas();
        assert_eq!(t.len(), CUANTAS);
        for (i, a) in t.iter().enumerate() {
            for b in t.iter().skip(i + 1) {
                assert_ne!(a.texto, b.texto);
            }
        }
    }

    /// El catálogo vive en dos sitios —el design system, que el usuario aprobó, y este módulo—
    /// y **tienen que decir lo mismo**. Este test es lo único que impide que se separen sin que
    /// nadie lo note: si alguien reescribe una maniobra aquí, el design system deja de describir
    /// la app.
    #[test]
    fn el_catalogo_dice_lo_mismo_que_el_design_system() {
        // Lee el design system EN TIEMPO DE TEST para comprobar que el catálogo no se separó de
        // lo que el usuario aprobó. La app en marcha jamás abre este archivo.
        let donde = concat!(env!("CARGO_MANIFEST_DIR"), "/../design-system.md");
        let ds = std::fs::read_to_string(donde).expect("falta design-system.md"); // verify-ephemeral:allow — ADR 008 «el corpus, el disparo y la ficha»: lectura de test, nunca en marcha
        for m in todas() {
            assert!(
                ds.contains(m.texto),
                "la maniobra «{}» no está en design-system.md — el catálogo se separó de lo que el usuario aprobó",
                m.texto
            );
            assert!(ds.contains(m.porque), "falta la razón de «{}» en el design system", m.texto);
        }
    }
}
