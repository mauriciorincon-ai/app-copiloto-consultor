//! EL MARCO EN LA MANO — la bandera de jurisdicción de un cliente (C11, ADR 017).
//!
//! **MÓDULO PROTEGIDO.** Puro: ni disco ni red, y `pnpm verify:ephemeral` lo comprueba. El catálogo va
//! dentro del binario (`data/jurisdicciones/`, como el del
//! radar y el de las propuestas), y la jurisdicción de un cliente sale de una línea de SU ficha
//! («Jurisdicción: Colombia»), que `corpus/` lee al indexar y le pasa aquí como texto.
//!
//! Dos reglas que viven aquí:
//! - **Lo no verificado no se afirma.** Cada fila del catálogo que tiene algo sin verificar lo dice en
//!   su `pendiente`, con el gap del informe legal-ético; la pantalla lo enseña.
//! - **No se adivina.** Una jurisdicción que el catálogo no trae se dice tal cual («“Bolivia” no está en
//!   el catálogo v1»), y no se busca la más parecida.
//!
//! Nada de esto es asesoría legal, y la pantalla lo dice siempre que hay bandera.

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::radar::Bilingue;

const CATALOGO: &str = include_str!("../../../data/jurisdicciones/catalogo.json");

/// El riesgo que la matriz del informe le da a la jurisdicción, con su palabra. Va del menor al mayor,
/// y **«sin verificar» es el mayor**: lo que no se sabe se trata con más cuidado, no con menos.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Riesgo {
    Bajo,
    BajoMedio,
    Medio,
    MedioAlto,
    SinVerificar,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub struct Fuente {
    pub norma: String,
    pub url: String,
}

/// Una fila del catálogo, como viene del archivo.
#[derive(Clone, Debug, Deserialize)]
pub struct Fila {
    pub id: String,
    pub nombre: Bilingue,
    pub alias: Vec<String>,
    pub riesgo: Riesgo,
    pub regla: Bilingue,
    pub implica: Bilingue,
    pub fuentes: Vec<Fuente>,
    /// Lo que el informe no pudo verificar de esta fila, con su gap.
    #[serde(default)]
    pub pendiente: Option<Bilingue>,
}

#[derive(Deserialize)]
struct Archivo {
    version: u32,
    consultado: String,
    clausula: Bilingue,
    jurisdicciones: Vec<Fila>,
}

pub struct Catalogo {
    pub version: u32,
    /// «2026-09-17»: la fecha del informe, la misma en todas las filas (el informe no fecha fila a fila).
    pub consultado: String,
    /// La cláusula modelo para la carta de encargo (ADR 017 §6).
    pub clausula: Bilingue,
    pub filas: Vec<Fila>,
}

impl Catalogo {
    fn de_texto(texto: &str) -> Result<Catalogo, String> {
        let a: Archivo = serde_json::from_str(texto).map_err(|e| format!("el catálogo de jurisdicciones no se lee: {e}"))?;
        let filas = a
            .jurisdicciones
            .into_iter()
            .map(|mut f| {
                f.alias = f.alias.iter().map(|x| normalizar(x)).collect();
                f
            })
            .collect();
        Ok(Catalogo { version: a.version, consultado: a.consultado, clausula: a.clausula, filas })
    }
}

/// El catálogo, leído una vez. El test `el_catalogo_se_lee_entero…` garantiza que se lee.
pub fn catalogo() -> &'static Catalogo {
    static C: OnceLock<Catalogo> = OnceLock::new();
    C.get_or_init(|| Catalogo::de_texto(CATALOGO).expect("data/jurisdicciones/catalogo.json no es válido"))
}

/// Sin mayúsculas, sin tildes y sin signos: «EE. UU.» → «ee uu», «**Jurisdicción**» → «jurisdiccion».
fn normalizar(texto: &str) -> String {
    crate::propuestas::plegar(texto)
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// **La línea de la ficha**: «Jurisdicción: Colombia» o «Jurisdiction: Florida», sin importar
/// mayúsculas, tildes ni el markdown alrededor («**Jurisdicción:** Colombia», «- Jurisdiction: FL»).
/// `None` si la línea no es esa.
pub fn de_la_linea(linea: &str) -> Option<String> {
    let (clave, valor) = linea.split_once(':')?;
    let clave = normalizar(clave);
    if clave != "jurisdiccion" && clave != "jurisdiction" {
        return None;
    }
    let valor = valor.trim().trim_matches(['*', '_']).trim();
    (!valor.is_empty()).then(|| valor.chars().take(80).collect())
}

/// La bandera, como la pinta la pantalla: sin alias ni URL, que se quedan en el catálogo.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Bandera {
    pub nombre: Bilingue,
    pub riesgo: Riesgo,
    pub regla: Bilingue,
    pub implica: Bilingue,
    /// Las normas, como las escribe la bandera: «CSJ AP1465-2018 · Ley 1581 art. 3».
    pub normas: Vec<String>,
    pub consultado: String,
    pub pendiente: Option<Bilingue>,
}

/// Lo que se sabe de la jurisdicción de un cliente. Las tres formas están en `kit.html` §5.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "que")]
pub enum LaBandera {
    /// En una caja: es mucho más grande que las otras dos formas, y serde la escribe igual.
    Conocida { bandera: Box<Bandera> },
    /// La ficha dice una jurisdicción que el catálogo no trae: se dice cuál y en qué versión, sin
    /// adivinar la más parecida.
    FueraDelCatalogo { escrita: String, version: u32 },
    /// La ficha no dice dónde está la contraparte.
    SinIndicar,
}

/// La bandera de lo que dice la ficha (`None`: no lo dice).
pub fn bandera(escrita: Option<&str>) -> LaBandera {
    let Some(escrita) = escrita else { return LaBandera::SinIndicar };
    let c = catalogo();
    match buscar(c, escrita) {
        Some(f) => LaBandera::Conocida {
            bandera: Box::new(Bandera {
                nombre: f.nombre.clone(),
                riesgo: f.riesgo,
                regla: f.regla.clone(),
                implica: f.implica.clone(),
                normas: f.fuentes.iter().map(|x| x.norma.clone()).collect(),
                consultado: c.consultado.clone(),
                pendiente: f.pendiente.clone(),
            }),
        },
        None => LaBandera::FueraDelCatalogo { escrita: escrita.to_string(), version: c.version },
    }
}

/// La fila de lo escrito: el valor entero o, si no, sus partes («EE. UU. · Florida», «Colombia y
/// California»). Si salen varias, **la más estricta**: el informe dice que, con las partes en sitios
/// distintos, suele aplicar la ley más estricta (§2.b).
fn buscar<'a>(c: &'a Catalogo, escrita: &str) -> Option<&'a Fila> {
    let fila = |k: &str| c.filas.iter().find(|f| f.alias.iter().any(|a| a == k));
    if let Some(f) = fila(&normalizar(escrita)) {
        return Some(f);
    }
    let mut partido = format!(" {} ", crate::propuestas::plegar(escrita));
    for separador in [" y ", " and ", " - ", ",", ";", "/", "·", "(", ")", "|", "&", "+"] {
        partido = partido.replace(separador, "\n");
    }
    partido.lines().map(normalizar).filter(|k| !k.is_empty()).filter_map(|k| fila(&k)).max_by_key(|f| f.riesgo)
}

/// La cláusula modelo, en los dos idiomas: se copia la de la carta de encargo, no la de la interfaz.
pub fn clausula() -> &'static Bilingue {
    &catalogo().clausula
}

/// La NDA de un cliente, tal como respondiste al chequeo (ADR 017 §4). Se guarda en tus preferencias;
/// «sin revisar» es no haber respondido, y no se guarda.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Nda {
    SinRevisar,
    NoLoProhibe,
    LoProhibe,
}

/// «Este cliente» en Sesión: a quién eliges, qué dice su ficha y qué respondiste de su NDA.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VistaDelCliente {
    /// Los clientes del corpus, por su nombre (`corpus::clientes`).
    pub clientes: Vec<String>,
    /// El de esta reunión. Vive en memoria: no se guarda en disco (ADR 017 §3).
    pub elegido: Option<String>,
    /// La bandera del elegido; `None` sin cliente elegido.
    pub bandera: Option<LaBandera>,
    pub nda: Nda,
    pub clausula: Bilingue,
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **El catálogo se lee entero y cada fila trae lo suyo.** ¿Puede fallar? Sí: una fila sin
    /// inglés, sin fuente, con una URL que no es `https://`, o «sin verificar» sin decir qué falta,
    /// y es rojo (bitácora).
    #[test]
    fn el_catalogo_se_lee_entero_y_cada_fila_trae_lo_suyo() {
        let c = catalogo();
        assert_eq!(c.version, 1);
        assert_eq!(c.consultado, "2026-09-17");
        assert_eq!(c.filas.len(), 27, "las diez de la matriz, los catorce estados de §2.b y los tres sin estatuto");
        assert!(c.clausula.es.len() > 200 && c.clausula.en.len() > 200, "la cláusula modelo está vacía");
        for f in &c.filas {
            for (que, b) in [("nombre", &f.nombre), ("regla", &f.regla), ("implica", &f.implica)] {
                assert!(!b.es.trim().is_empty() && !b.en.trim().is_empty(), "«{}»: su {que} no está en los dos idiomas", f.id);
            }
            assert!(!f.alias.is_empty(), "«{}» no tiene cómo escribirse en la ficha", f.id);
            assert!(!f.fuentes.is_empty(), "«{}» no tiene fuente", f.id);
            for x in &f.fuentes {
                assert!(!x.norma.trim().is_empty() && x.url.starts_with("https://"), "«{}»: fuente sin norma o sin URL https", f.id);
            }
            if f.riesgo == Riesgo::SinVerificar {
                assert!(f.pendiente.is_some(), "«{}» está sin verificar y no dice qué falta", f.id);
            }
            if let Some(p) = &f.pendiente {
                assert!(!p.es.trim().is_empty() && !p.en.trim().is_empty(), "«{}»: lo pendiente no está en los dos idiomas", f.id);
            }
        }
    }

    /// Dos filas no pueden compartir un alias: la misma línea de la ficha daría dos banderas.
    #[test]
    fn ningun_alias_se_repite() {
        let mut vistos = std::collections::HashMap::new();
        for f in &catalogo().filas {
            for a in &f.alias {
                if let Some(otra) = vistos.insert(a.clone(), f.id.clone()) {
                    panic!("«{a}» es alias de «{otra}» y de «{}»", f.id);
                }
            }
        }
    }

    #[test]
    fn la_linea_de_la_ficha_se_lee_con_su_markdown_y_sin_tildes() {
        assert_eq!(de_la_linea("Jurisdicción: Colombia").as_deref(), Some("Colombia"));
        assert_eq!(de_la_linea("jurisdiccion:colombia").as_deref(), Some("colombia"));
        assert_eq!(de_la_linea("**Jurisdicción:** Colombia").as_deref(), Some("Colombia"));
        assert_eq!(de_la_linea("- Jurisdiction: Florida").as_deref(), Some("Florida"));
        assert_eq!(de_la_linea("Jurisdicción del contrato: Chile"), None, "otra clave no es la jurisdicción");
        assert_eq!(de_la_linea("La gerente general firma."), None);
        assert_eq!(de_la_linea("Jurisdicción:   "), None, "vacía no es una jurisdicción");
    }

    /// **La bandera: la conocida, la de fuera y la que no se dijo; y con varias, la más estricta.**
    /// ¿Puede fallar? Sí: quedándose con la primera parte en vez de la más estricta, «Colombia y
    /// California» da Colombia (riesgo bajo-medio) y es rojo (bitácora).
    #[test]
    fn la_bandera_es_la_de_la_ficha_y_con_varias_la_mas_estricta() {
        let fila = |escrita: &str| match bandera(Some(escrita)) {
            LaBandera::Conocida { bandera } => (bandera.nombre.es, bandera.riesgo),
            otra => panic!("«{escrita}» no dio bandera: {otra:?}"),
        };
        assert_eq!(fila("Colombia"), ("Colombia".into(), Riesgo::BajoMedio));
        assert_eq!(fila("méxico").1, Riesgo::BajoMedio);
        assert_eq!(fila("EEUU"), ("EE. UU. (federal y estados de una parte)".into(), Riesgo::Bajo));
        assert_eq!(fila("EE. UU. · Florida").1, Riesgo::MedioAlto, "el estado manda sobre el país");
        assert_eq!(fila("Colombia y California").0, "EE. UU. · California (todas las partes)");
        assert_eq!(fila("Colombia, Missouri").1, Riesgo::SinVerificar, "lo que no se sabe es lo más estricto");
        match bandera(Some("Bolivia")) {
            LaBandera::FueraDelCatalogo { escrita, version } => assert_eq!((escrita.as_str(), version), ("Bolivia", 1)),
            otra => panic!("Bolivia no está en el catálogo y dio {otra:?}"),
        }
        assert_eq!(bandera(None), LaBandera::SinIndicar);
    }

    /// Lo no verificado viaja con la bandera: la pantalla no puede callarlo si no le llega.
    #[test]
    fn lo_pendiente_llega_a_la_pantalla() {
        let LaBandera::Conocida { bandera } = bandera(Some("Francia")) else { panic!("Francia está en el catálogo") };
        assert!(bandera.pendiente.is_some_and(|p| p.es.contains("G-9")));
        assert_eq!(bandera.normas, vec!["Code pénal art. 226-1".to_string()]);
    }
}
