//! Los dos catálogos del radar, **dentro del binario**.
//!
//! `data/radar/programas.json` (coral y MDM) y `data/radar/avisos.json` (ámbar) se incluyen al
//! compilar y se leen una vez. No hay archivo que abrir en tiempo de ejecución ni servidor al que
//! preguntar: un catálogo nuevo llega con una versión nueva de la app. Cada fila lleva su fuente,
//! y el test de abajo no deja entrar una que no la tenga.

use super::{Bilingue, CatalogoDelRadar, Categoria, Programa};
use std::sync::OnceLock;

const PROGRAMAS: &str = include_str!("../../../data/radar/programas.json");
const AVISOS: &str = include_str!("../../../data/radar/avisos.json");

#[derive(serde::Deserialize)]
struct Programas {
    version: u32,
    fecha: String,
    programas: Vec<Fila>,
}

/// Una fila del catálogo de programas.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct Fila {
    pub id: String,
    pub nombre: String,
    pub categoria: Categoria,
    /// Los nombres de ejecutable que la delatan, tal y como el núcleo los da (el último tramo de
    /// la ruta). Se comparan enteros, sin mayúsculas: jamás por trozos, que es como «Teams» acabaría
    /// pareciéndose a «TeamViewer».
    #[serde(default)]
    pub procesos: Vec<String>,
    /// La fila no se reconoce por un proceso sino por la inscripción del Mac en un MDM.
    #[serde(default)]
    pub inscripcion: bool,
    pub ve: Bilingue,
    pub alcance: Bilingue,
    pub fuente: String,
}

impl Fila {
    pub fn programa(&self) -> Programa {
        Programa {
            nombre: self.nombre.clone(),
            categoria: self.categoria,
            nivel: self.categoria.nivel(),
            ve: self.ve.clone(),
            alcance: self.alcance.clone(),
            fuente: self.fuente.clone(),
        }
    }
}

#[derive(serde::Deserialize)]
struct Avisos {
    grabacion: Vec<FilaDeGrabacion>,
    bots: Vec<FilaDeBot>,
}

/// Las frases con que un cliente de videollamada avisa de que graba o transcribe.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct FilaDeGrabacion {
    pub cliente: String,
    pub frases: Vec<String>,
    pub fuente: String,
}

/// Un bot de notas y los nombres con que entra a la llamada.
#[derive(Clone, Debug, serde::Deserialize)]
pub struct FilaDeBot {
    pub nombre: String,
    pub patrones: Vec<String>,
    pub fuente: String,
}

fn de_programas() -> &'static Programas {
    static P: OnceLock<Programas> = OnceLock::new();
    P.get_or_init(|| serde_json::from_str(PROGRAMAS).expect("data/radar/programas.json no es válido"))
}

fn de_avisos() -> &'static Avisos {
    static A: OnceLock<Avisos> = OnceLock::new();
    A.get_or_init(|| serde_json::from_str(AVISOS).expect("data/radar/avisos.json no es válido"))
}

pub fn programas() -> &'static [Fila] {
    &de_programas().programas
}

/// La fila que se enseña cuando `profiles` dice que el Mac está inscrito en un MDM.
pub fn inscripcion() -> Option<Programa> {
    programas().iter().find(|f| f.inscripcion).map(Fila::programa)
}

pub fn rotulo() -> CatalogoDelRadar {
    let p = de_programas();
    CatalogoDelRadar { version: p.version, fecha: p.fecha.clone() }
}

pub fn grabacion() -> &'static [FilaDeGrabacion] {
    &de_avisos().grabacion
}

pub fn bots() -> &'static [FilaDeBot] {
    &de_avisos().bots
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use std::collections::HashSet;

    /// **Una fila sin fuente no entra.** Es la diferencia entre un catálogo y una lista de
    /// sospechas: si el radar le dice al usuario que un programa ve su cámara, alguien tiene que
    /// poder comprobar de dónde salió.
    #[test]
    fn cada_fila_trae_su_fuente_y_sus_dos_idiomas() {
        for f in programas() {
            assert!(f.fuente.starts_with("https://"), "«{}» sin fuente comprobable", f.id);
            for t in [&f.ve, &f.alcance] {
                assert!(!t.es.trim().is_empty() && !t.en.trim().is_empty(), "«{}» sin sus dos idiomas", f.id);
            }
            assert!(
                f.inscripcion != !f.procesos.is_empty(),
                "«{}» tiene que reconocerse por sus procesos o por la inscripción, una de las dos",
                f.id
            );
        }
        for g in grabacion() {
            assert!(g.fuente.starts_with("https://") && !g.frases.is_empty(), "{} sin fuente o sin frases", g.cliente);
        }
        for b in bots() {
            assert!(b.fuente.starts_with("https://") && !b.patrones.is_empty(), "{} sin fuente o sin patrones", b.nombre);
        }
    }

    /// Un ejecutable que delatara a dos programas pondría al radar a elegir, y elegiría al azar.
    #[test]
    fn ningun_proceso_delata_a_dos_programas() {
        let mut vistos = HashSet::new();
        let mut ids = HashSet::new();
        for f in programas() {
            assert!(ids.insert(f.id.clone()), "id repetido: {}", f.id);
            for p in &f.procesos {
                assert!(vistos.insert(p.to_lowercase()), "«{p}» está en dos filas");
            }
        }
    }

    #[test]
    fn el_catalogo_se_lee_y_tiene_de_las_cinco_clases() {
        let r = rotulo();
        assert!(r.version >= 1 && r.fecha.len() == 10, "rótulo raro: {r:?}");
        let clases: HashSet<_> = programas().iter().map(|f| f.categoria).collect();
        for c in [Categoria::Supervision, Categoria::Monitoreo, Categoria::AccesoRemoto, Categoria::Mdm] {
            assert!(clases.contains(&c), "el catálogo no trae ninguna fila de {c:?}");
        }
        assert!(inscripcion().is_some(), "falta la fila de la inscripción en un MDM");
    }
}
