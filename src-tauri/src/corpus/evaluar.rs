//! EL KIT DE EVALUACIÓN, como producto (ADR 018 §4).
//!
//! Hasta el sprint 003 vivía dentro del test de integración (`contra-el-mac-de-verdad.rs`), que medía
//! el kit del repo. La puerta local lo necesita para correr **las preguntas del usuario contra su
//! corpus**, así que sale aquí y el test y la puerta miden con el mismo código: si uno cambia, cambian
//! los dos.
//!
//! Dos medidas, las mismas de siempre:
//!
//! - **nDCG@5** sobre las preguntas que SÍ tienen respuesta, con relevancia binaria: la sección
//!   esperada vale 1 y todo lo demás 0.
//! - **El rechazo** sobre las que NO la tienen: cuántas contesta la app con «no está en tu corpus» en
//!   vez de aproximar una ficha. El fallo caro de esta app no es no encontrar; es encontrar cualquier
//!   cosa y ponerle una fuente debajo.

use std::time::Instant;

use serde::{Deserialize, Serialize};

use super::Corpus;
use crate::ficha::{armar, Respuesta};

/// Las preguntas: el formato de `docs/kit-de-prueba/preguntas.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Kit {
    pub preguntas: Vec<Caso>,
    #[serde(rename = "sinRespuesta", default)]
    pub sin_respuesta: Vec<String>,
}

/// Una pregunta y la sección que tendría que salir en los cinco primeros.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Caso {
    pub dice: String,
    pub espera: String,
}

/// Una pregunta que no encontró su sección en los cinco primeros.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fallo {
    pub dice: String,
    pub espera: String,
    pub trajo: Vec<String>,
}

/// Una pregunta sin respuesta a la que la app le puso una ficha en vez de callar.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Aproximada {
    pub dice: String,
    pub cito: String,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Informe {
    pub ndcg5: f64,
    /// `None` si el kit no trae preguntas sin respuesta: un rechazo de 0 de 0 no es un 100 %.
    pub rechazo: Option<f64>,
    pub preguntas: usize,
    pub sin_respuesta: usize,
    pub fallos: Vec<Fallo>,
    pub aproximadas: Vec<Aproximada>,
    /// De la pregunta a la ficha, en microsegundos, de menor a mayor: lo mismo que cronometra el
    /// producto —buscar y armar la ficha—, pregunta a pregunta, porque una media esconde los picos.
    pub latencias_us: Vec<u64>,
}

/// nDCG@5 con relevancia binaria. Con un solo documento relevante, el ideal es 1,0 y el descuento
/// sale del puesto en el que aparece.
pub fn ndcg_5(puestos: &[String], espera: &str) -> f64 {
    puestos
        .iter()
        .take(5)
        .position(|s| s == espera)
        .map(|i| 1.0 / ((i + 2) as f64).log2())
        .unwrap_or(0.0)
}

/// Corre el kit contra el corpus. Un kit sin preguntas con respuesta no mide nada, y se dice.
pub fn evaluar(corpus: &Corpus, kit: &Kit) -> Result<Informe, String> {
    if kit.preguntas.is_empty() {
        return Err("el kit no trae preguntas con respuesta: no hay nada que medir".into());
    }
    let mut suma = 0.0;
    let mut fallos = Vec::new();
    let mut latencias_us = Vec::with_capacity(kit.preguntas.len());
    for c in &kit.preguntas {
        let reloj = Instant::now();
        let hallazgos = corpus.buscar(&c.dice, 5)?;
        let _ = armar(&c.dice, &hallazgos);
        latencias_us.push(reloj.elapsed().as_micros() as u64);
        let puestos: Vec<String> = hallazgos.into_iter().map(|h| h.seccion.unwrap_or_default()).collect();
        let n = ndcg_5(&puestos, &c.espera);
        suma += n;
        if n == 0.0 {
            fallos.push(Fallo { dice: c.dice.clone(), espera: c.espera.clone(), trajo: puestos });
        }
    }
    latencias_us.sort_unstable();

    let mut rechazadas = 0;
    let mut aproximadas = Vec::new();
    for dice in &kit.sin_respuesta {
        match armar(dice, &corpus.buscar(dice, 3)?) {
            Respuesta::SinResultado { .. } => rechazadas += 1,
            Respuesta::Ficha(f) => aproximadas.push(Aproximada { dice: dice.clone(), cito: f.fuente.documento }),
        }
    }
    let rechazo = (!kit.sin_respuesta.is_empty()).then(|| rechazadas as f64 / kit.sin_respuesta.len() as f64);

    Ok(Informe {
        ndcg5: suma / kit.preguntas.len() as f64,
        rechazo,
        preguntas: kit.preguntas.len(),
        sin_respuesta: kit.sin_respuesta.len(),
        fallos,
        aproximadas,
        latencias_us,
    })
}

#[cfg(test)]
mod pruebas {
    use super::*;

    #[test]
    fn el_primer_puesto_vale_uno_y_fuera_de_los_cinco_cero() {
        let p = |v: &[&str]| v.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert_eq!(ndcg_5(&p(&["a", "b"]), "a"), 1.0);
        assert!((ndcg_5(&p(&["x", "a"]), "a") - 1.0 / 3f64.log2()).abs() < 1e-12);
        assert_eq!(ndcg_5(&p(&["1", "2", "3", "4", "5", "a"]), "a"), 0.0);
        assert_eq!(ndcg_5(&[], "a"), 0.0);
    }

    #[test]
    fn un_kit_sin_preguntas_no_mide_nada_y_lo_dice() {
        let corpus = Corpus::en_memoria().unwrap();
        let vacio = Kit { preguntas: vec![], sin_respuesta: vec!["¿algo?".into()] };
        assert!(evaluar(&corpus, &vacio).is_err());
    }

    /// El formato es el del kit del repo: si `preguntas.json` cambia de forma, la puerta deja de poder
    /// leerlo, y esto lo dice antes que el usuario.
    #[test]
    fn lee_el_kit_del_repo() {
        let kit: Kit =
            serde_json::from_str(include_str!("../../../docs/kit-de-prueba/preguntas.json")).expect("preguntas.json");
        assert!(kit.preguntas.len() >= 30);
        assert!(!kit.sin_respuesta.is_empty());
    }
}
