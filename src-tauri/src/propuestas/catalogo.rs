//! El catálogo de las propuestas, **dentro del binario** (como el del radar).
//!
//! `data/propuestas/reglas.json` se incluye al compilar y se lee una vez. Las reglas que enseña la
//! pantalla salen de aquí, así que la lista que lees es la lista entera (ADR 016 §1). Todo se guarda
//! plegado —sin mayúsculas ni tildes—, que es como se compara.

use std::collections::{HashMap, HashSet};
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use super::{plegar, Palabra};

const REGLAS: &str = include_str!("../../../data/propuestas/reglas.json");

/// Una regla como la enseña la pantalla: su id, de quién la reconoce y su nombre en los dos idiomas.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FilaDeRegla {
    pub id: String,
    /// `ambos`, `tuyo` o `cliente`.
    pub de: String,
    pub nombre: crate::radar::Bilingue,
}

#[derive(Deserialize)]
struct Archivo {
    version: u32,
    fecha: String,
    reglas: Vec<FilaDeRegla>,
    compromisos: Vec<String>,
    numeros: HashMap<String, f64>,
    unidades: Vec<String>,
    monedas: Vec<String>,
    fechas: Vec<String>,
    meses: Vec<String>,
    interrogativos: Vec<String>,
    vacias: Vec<String>,
}

pub struct Catalogo {
    pub version: u32,
    pub fecha: String,
    pub reglas: Vec<FilaDeRegla>,
    /// Cada frase, en palabras plegadas.
    pub compromisos: Vec<Vec<String>>,
    pub numeros: HashMap<String, f64>,
    unidades: Vec<Vec<String>>,
    pub monedas: Vec<String>,
    pub fechas: Vec<Vec<String>>,
    pub meses: Vec<String>,
    pub interrogativos: HashSet<String>,
    pub vacias: HashSet<String>,
}

fn en_palabras(frases: Vec<String>) -> Vec<Vec<String>> {
    frases.iter().map(|f| plegar(f).split_whitespace().map(str::to_string).collect()).collect()
}

impl Catalogo {
    fn de_texto(texto: &str) -> Result<Catalogo, String> {
        let a: Archivo = serde_json::from_str(texto).map_err(|e| format!("el catálogo de propuestas no se lee: {e}"))?;
        Ok(Catalogo {
            version: a.version,
            fecha: a.fecha,
            reglas: a.reglas,
            compromisos: en_palabras(a.compromisos),
            numeros: a.numeros.into_iter().map(|(k, v)| (plegar(&k), v)).collect(),
            unidades: en_palabras(a.unidades),
            monedas: a.monedas.iter().map(|m| plegar(m)).collect(),
            fechas: en_palabras(a.fechas),
            meses: a.meses.iter().map(|m| plegar(m)).collect(),
            interrogativos: a.interrogativos.iter().map(|m| plegar(m)).collect(),
            vacias: a.vacias.iter().map(|m| plegar(m)).collect(),
        })
    }

    /// ¿Es esta palabra, sola, una unidad? («semanas», «%», «usd»)
    pub fn es_unidad(&self, plegada: &str) -> bool {
        self.unidades.iter().any(|u| u.len() == 1 && u[0] == plegada)
    }

    /// ¿Es esta palabra, sola, una fecha? («viernes», «tomorrow»)
    pub fn es_fecha(&self, plegada: &str) -> bool {
        self.fechas.iter().any(|f| f.len() == 1 && f[0] == plegada)
    }

    /// Si las palabras empiezan por una unidad, cuántas palabras ocupa (1 o 2).
    pub(super) fn unidad_en(&self, ps: &[Palabra]) -> Option<usize> {
        self.unidades
            .iter()
            .filter(|u| u.len() <= ps.len())
            .filter(|u| ps.iter().zip(u.iter()).all(|(p, w)| p.plegada == *w))
            .map(Vec::len)
            .max()
    }
}

/// El catálogo, leído una vez. Si no se lee, es un fallo de compilación del producto: lo caza el
/// test de abajo antes de que llegue a nadie.
pub fn catalogo() -> &'static Catalogo {
    static UNO: OnceLock<Catalogo> = OnceLock::new();
    UNO.get_or_init(|| Catalogo::de_texto(REGLAS).expect("data/propuestas/reglas.json"))
}

/// Las reglas, para la pantalla «Qué sabe reconocer».
pub fn reglas() -> Vec<FilaDeRegla> {
    catalogo().reglas.clone()
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::propuestas::Regla;

    #[test]
    fn el_catalogo_se_lee_y_trae_una_fila_por_regla_del_codigo() {
        let c = catalogo();
        assert_eq!(c.version, 1);
        assert!(!c.fecha.is_empty());
        let ids: Vec<&str> = c.reglas.iter().map(|r| r.id.as_str()).collect();
        let del_codigo: Vec<&str> = Regla::TODAS.iter().map(|r| r.id()).collect();
        assert_eq!(ids, del_codigo, "la pantalla enseñaría una lista distinta de la que aplica el código");
        for r in &c.reglas {
            assert!(!r.nombre.es.is_empty() && !r.nombre.en.is_empty(), "{} sin nombre en los dos idiomas", r.id);
            assert!(["ambos", "tuyo", "cliente"].contains(&r.de.as_str()), "{}: de «{}»", r.id, r.de);
        }
    }

    #[test]
    fn todo_se_guarda_plegado_para_compararse_igual() {
        let c = catalogo();
        assert!(c.fechas.iter().any(|f| f == &["manana".to_string()]));
        assert!(c.compromisos.iter().any(|f| f.join(" ") == "te lo envio"));
        assert_eq!(c.numeros.get("dieciseis"), Some(&16.0));
    }
}
