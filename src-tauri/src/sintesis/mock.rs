//! **`mock`, un proveedor de primera clase** (patrón de la casa: dentro del adapter, por env —
//! `AG_SINTESIS=mock`—, jamás interceptando la red). Es el de la CI: determinista, sin modelo y sin
//! red, y recorre el MISMO camino que los de verdad —`sugerir`, el techo, `fundar`—, así que cada
//! corrida prueba de paso el contrato entero.
//!
//! Qué redacta: la primera ficha, dicha como sugerencia. No es buena prosa y no lo pretende; es la
//! forma exacta del esquema, con una fuente que sí se le dio.

use super::{PorQueNoRedacta, Proveedor, Quien, Respuesta};

pub struct Mock;

impl Proveedor for Mock {
    fn quien(&self) -> Quien {
        Quien::Mock
    }
    fn nombre(&self) -> String {
        "mock".into()
    }
    fn disponible(&self) -> Result<(), PorQueNoRedacta> {
        Ok(())
    }
    fn redactar(&self, _instrucciones: &str, texto: &str) -> Result<Respuesta, String> {
        // **El banco del ensayo** (sprint 004, ADR 019 §3): la petición empieza por «Language: …».
        if let Some(idioma) = texto.lines().next().and_then(|l| l.strip_prefix("Language: ")) {
            return Ok(Respuesta { json: preguntas_del_ensayo(idioma.trim(), texto), ..Default::default() });
        }
        // La primera ficha es la línea que empieza por «F1 · titular — línea».
        let f1 = texto.lines().find(|l| l.starts_with("F1 · ")).ok_or("sin fichas")?;
        let (titular, linea) = f1["F1 · ".len()..].split_once(" — ").unwrap_or((f1, ""));
        let json = serde_json::json!({
            "titular": titular.split_whitespace().take(8).collect::<Vec<_>>().join(" "),
            "linea": if linea.is_empty() { titular } else { linea },
            "fuente": "F1",
            "confianza": "media",
        });
        Ok(Respuesta { json: json.to_string(), ..Default::default() })
    }
}

/// Para el ensayo, el mock propone **dos preguntas fundadas** —con las seis primeras palabras de las dos
/// primeras secciones— y **una que no se funda** (una sección que no existe), para que cada corrida del
/// kit recorra también el descarte.
fn preguntas_del_ensayo(idioma: &str, texto: &str) -> String {
    let secciones: Vec<(&str, &str)> = texto
        .lines()
        .skip(1)
        .filter_map(|l| {
            let (id, resto) = l.split_once(" · ")?;
            let (_, linea) = resto.split_once(" — ")?;
            (!linea.is_empty()).then_some((id, linea))
        })
        .collect();
    let en = idioma == "en";
    let mut preguntas: Vec<serde_json::Value> = secciones
        .iter()
        .take(2)
        .map(|(id, linea)| {
            let trozo = linea.split_whitespace().take(6).collect::<Vec<_>>().join(" ");
            let texto = if en { format!("What does “{trozo}” mean in practice?") } else { format!("¿Qué significa en la práctica «{trozo}»?") };
            serde_json::json!({ "texto": texto, "seccion": id })
        })
        .collect();
    preguntas.push(serde_json::json!({
        "texto": if en { "Do you guarantee results?" } else { "¿Garantizan los resultados?" },
        "seccion": "S99",
    }));
    serde_json::json!({ "preguntas": preguntas }).to_string()
}

#[cfg(test)]
mod pruebas {
    use super::super::{pruebas::peticion, sugerir, TECHO};
    use super::*;
    use std::sync::Arc;

    #[test]
    fn el_mock_recorre_el_contrato_entero_y_cita_la_primera_ficha() {
        let r = sugerir(Arc::new(Mock), &peticion(), TECHO);
        let s = r.sugerencia.expect("el mock no produjo una sugerencia fundada");
        assert_eq!(s.quien(), Quien::Mock);
        assert_eq!(r.respuesta.bytes_fuera, 0, "el mock no sale del Mac");
    }
}
