//! Corpus del consultor — módulo **NO protegido**, y con razón.
//!
//! Aquí SÍ se abre disco: el corpus son los documentos PROPIOS del usuario, que se leen donde
//! están, y su índice vive en la carpeta de la app. Eso es exactamente lo que la regla del
//! efímero permite persistir. Por eso `corpus/` queda fuera de la lista de módulos protegidos
//! de `verify:ephemeral`: si estuviera dentro, el gate se volvería imposible de cumplir y la
//! tentación sería aflojarlo — y un gate aflojado deja de proteger lo que sí importa.
//!
//! La frontera es la del estándar 4-T: **lo del usuario** puede persistir; **lo de terceros**,
//! no. Nada de lo que entra aquí viene de la reunión.
//!
//! Se llena en la fase 4 del sprint 001 (ingesta → chunking por sección → BM25 con tantivy).

pub mod consulta;
pub mod indice;
pub mod leer;
pub mod seccion;
pub mod unidad;

use std::path::{Path, PathBuf};

use serde::Serialize;

pub use indice::{Hallazgo, Indice};
pub use unidad::Unidad;

/// Techo de documentos por carpeta. No es una limitación técnica: es un aviso. Quien apunta a su
/// carpeta de Descargas entera espera que la app se lo diga en vez de pasarse veinte minutos
/// leyendo facturas.
pub const TECHO_DE_DOCUMENTOS: usize = 2_000;

/// Hasta dónde baja por las subcarpetas.
const HONDURA: usize = 6;

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "estado")]
pub enum Estado {
    Indexado { secciones: usize },
    /// La maqueta lo pinta en ámbar: el documento está ahí y la app dice por qué no lo leyó.
    SinLeer { motivo: String },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Documento {
    pub ruta: String,
    pub nombre: String,
    pub unidad: Option<Unidad>,
    /// Los títulos de sus secciones eran una conjetura por la forma del texto (PDF).
    pub conjeturado: bool,
    #[serde(flatten)]
    pub estado: Estado,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PorUnidad {
    pub unidad: String,
    pub documentos: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EstadoDelCorpus {
    /// `None` mientras el usuario no haya señalado carpeta: es el estado «vacío» de la maqueta.
    pub carpeta: Option<String>,
    pub documentos: usize,
    pub secciones: usize,
    pub por_unidad: Vec<PorUnidad>,
    pub sin_unidad: usize,
    pub ilegibles: usize,
    /// Documentos cuyas secciones se **conjeturaron** por la forma del texto (PDF), en vez de
    /// venir escritas. Se cuenta para que el usuario pueda juzgar cómo se leyó su corpus.
    pub conjeturados: usize,
    /// Dónde vive el índice, en claro, porque la pantalla de corpus lo enseña.
    pub donde_vive: Option<String>,
    pub bytes_del_indice: u64,
}

pub struct Corpus {
    indice: Indice,
    documentos: Vec<Documento>,
    carpeta: Option<PathBuf>,
    /// Las palabras distintivas del corpus, para el motivo «término tuyo» del disparador. Se
    /// calculan al indexar y no en cada turno: la escucha las pide veinticinco veces por segundo.
    vocabulario: Vec<String>,
}

impl Corpus {
    pub fn en(carpeta_del_indice: &Path) -> Result<Self, String> {
        Ok(Corpus {
            indice: Indice::en(carpeta_del_indice)?,
            documentos: Vec::new(),
            carpeta: None,
            vocabulario: Vec::new(),
        })
    }

    pub fn en_memoria() -> Result<Self, String> {
        Ok(Corpus {
            indice: Indice::en_memoria()?,
            documentos: Vec::new(),
            carpeta: None,
            vocabulario: Vec::new(),
        })
    }

    /// Indexa una carpeta entera. `avisar` recibe cada documento en cuanto se resuelve, para que
    /// la pantalla enseñe progreso de verdad y no una barra que se inventa el porcentaje.
    ///
    /// **Un documento que falla no detiene a los demás** — la maqueta lo promete con esas
    /// palabras. Cada fallo se queda en su propia fila, con su motivo en español llano.
    pub fn indexar(&mut self, carpeta: &Path, avisar: &dyn Fn(&Documento)) -> Result<usize, String> {
        if !carpeta.is_dir() {
            return Err("eso no es una carpeta".into());
        }
        self.indice.vaciar()?;
        self.documentos.clear();
        self.vocabulario.clear();
        self.carpeta = Some(carpeta.to_path_buf());
        let mut titulos: Vec<String> = Vec::new();

        for ruta in recorrer(carpeta, HONDURA) {
            if self.documentos.len() >= TECHO_DE_DOCUMENTOS {
                println!(
                    "[corpus] la carpeta trae más de {TECHO_DE_DOCUMENTOS} documentos legibles; se indexaron los primeros"
                );
                break;
            }
            let (doc, suyos) = self.indexar_uno(&ruta);
            titulos.extend(suyos);
            avisar(&doc);
            self.documentos.push(doc);
        }
        let nombres: Vec<String> = self.documentos.iter().map(|d| d.nombre.clone()).collect();
        self.vocabulario = crate::disparo::vocabulario(&nombres, &titulos);
        Ok(self.documentos.len())
    }

    /// Devuelve el documento y los títulos de sus secciones (que alimentan el vocabulario).
    fn indexar_uno(&self, ruta: &Path) -> (Documento, Vec<String>) {
        let nombre = ruta.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let como_texto = ruta.to_string_lossy().to_string();

        let leido = match leer::leer(ruta) {
            Ok(l) => l,
            Err(e) => {
                return (
                    Documento {
                        ruta: como_texto,
                        nombre,
                        unidad: None,
                        conjeturado: false,
                        estado: Estado::SinLeer { motivo: e.motivo() },
                    },
                    Vec::new(),
                )
            }
        };

        let secciones = seccion::trocear(&leido.lineas);
        let plano: String = secciones.iter().map(|s| s.texto.as_str()).collect::<Vec<_>>().join(" ");
        let unidad = Unidad::clasificar(&nombre, &plano);

        let titulos: Vec<String> = secciones.iter().filter_map(|s| s.titulo.clone()).collect();
        match self.indice.meter(&como_texto, &nombre, unidad, leido.conjeturado, &secciones) {
            Ok(n) => (
                Documento {
                    ruta: como_texto,
                    nombre,
                    unidad,
                    conjeturado: leido.conjeturado,
                    estado: Estado::Indexado { secciones: n },
                },
                titulos,
            ),
            Err(e) => (
                Documento {
                    ruta: como_texto,
                    nombre,
                    unidad,
                    conjeturado: leido.conjeturado,
                    estado: Estado::SinLeer { motivo: format!("no se pudo indexar: {e}") },
                },
                Vec::new(),
            ),
        }
    }

    /// El índice, para las pruebas que necesitan meter secciones a mano sin pasar por archivos.
    #[cfg(test)]
    pub fn indice_para_pruebas(&self) -> &Indice {
        &self.indice
    }

    /// Las palabras distintivas del corpus. Vacío mientras no haya carpeta señalada.
    pub fn vocabulario(&self) -> &[String] {
        &self.vocabulario
    }

    pub fn buscar(&self, texto: &str, cuantos: usize) -> Result<Vec<Hallazgo>, String> {
        self.indice.buscar(texto, cuantos)
    }

    pub fn documentos(&self) -> &[Documento] {
        &self.documentos
    }

    pub fn estado(&self) -> EstadoDelCorpus {
        let ilegibles = self
            .documentos
            .iter()
            .filter(|d| matches!(d.estado, Estado::SinLeer { .. }))
            .count();
        let legibles = || self.documentos.iter().filter(|d| matches!(d.estado, Estado::Indexado { .. }));

        EstadoDelCorpus {
            carpeta: self.carpeta.as_ref().map(|c| c.to_string_lossy().to_string()),
            documentos: self.documentos.len(),
            secciones: self.indice.secciones(),
            por_unidad: Unidad::TODAS
                .into_iter()
                .map(|u| PorUnidad {
                    unidad: u.etiqueta().to_string(),
                    documentos: legibles().filter(|d| d.unidad == Some(u)).count(),
                })
                .collect(),
            sin_unidad: legibles().filter(|d| d.unidad.is_none()).count(),
            ilegibles,
            conjeturados: legibles().filter(|d| d.conjeturado).count(),
            donde_vive: self.indice.carpeta().map(|c| c.to_string_lossy().to_string()),
            bytes_del_indice: self.indice.carpeta().map(pesa).unwrap_or(0),
        }
    }
}

/// Lista los archivos legibles de una carpeta, ordenados, sin seguir enlaces simbólicos.
///
/// El orden importa: dos indexaciones de la misma carpeta tienen que recorrerla igual, o el
/// techo de documentos recortaría un conjunto distinto cada vez y nadie entendería por qué.
fn recorrer(carpeta: &Path, hondura: usize) -> Vec<PathBuf> {
    if hondura == 0 {
        return Vec::new();
    }
    let Ok(entradas) = std::fs::read_dir(carpeta) else {
        return Vec::new();
    };
    let mut archivos = Vec::new();
    let mut carpetas = Vec::new();
    for e in entradas.flatten() {
        let ruta = e.path();
        let nombre = e.file_name().to_string_lossy().to_string();
        // Lo oculto se queda fuera: ahí viven los archivos de sincronización de las nubes, no
        // los documentos del usuario.
        if nombre.starts_with('.') {
            continue;
        }
        match e.file_type() {
            Ok(t) if t.is_dir() => carpetas.push(ruta),
            Ok(t) if t.is_file() && leer::se_lee(&ruta) => archivos.push(ruta),
            _ => {}
        }
    }
    archivos.sort();
    carpetas.sort();
    for c in carpetas {
        archivos.extend(recorrer(&c, hondura - 1));
    }
    archivos
}

fn pesa(carpeta: &Path) -> u64 {
    let Ok(entradas) = std::fs::read_dir(carpeta) else {
        return 0;
    };
    entradas
        .flatten()
        .map(|e| match e.file_type() {
            Ok(t) if t.is_dir() => pesa(&e.path()),
            _ => e.metadata().map(|m| m.len()).unwrap_or(0),
        })
        .sum()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// Cada test estrena carpeta. Compartirla por PID las hace pisarse entre ellas al correr en
    /// paralelo dentro del mismo binario — el mismo defecto que en la fase 3 obligó a serializar
    /// los tests de audio, y aquí se evita con un nombre distinto en vez de con un candado.
    fn carpeta_de_prueba(quien: &str) -> PathBuf {
        let c = std::env::temp_dir().join(format!("ag-corpus-{}-{quien}", std::process::id()));
        let _ = std::fs::remove_dir_all(&c);
        std::fs::create_dir_all(c.join("casos")).unwrap();
        std::fs::write(
            c.join("Propuesta Páramo Azul.md"),
            "# Alcance\nTres canales y una línea base de doce meses de histórico.\n\n# Precio\nTarifa cerrada por cuatro semanas de trabajo del equipo.\n",
        )
        .unwrap();
        std::fs::write(
            c.join("casos/Cooperativa Sur del Valle · cierre de caso.md"),
            "# Resultados\nLa implementación cerró con dos semanas de retraso y sin sobrecosto.\n",
        )
        .unwrap();
        // Ilegible: existe, pesa, y no suelta texto.
        std::fs::write(c.join("acta escaneada.pdf"), b"%PDF-1.4 roto").unwrap();
        // Ajeno: ni se mira.
        std::fs::write(c.join("hoja.xlsx"), b"cualquier cosa").unwrap();
        // Oculto: tampoco.
        std::fs::write(c.join(".sincroniza.md"), "# Nada\nnada de nada aquí dentro nunca.\n").unwrap();
        c
    }

    #[test]
    fn indexa_una_carpeta_entera_y_un_documento_roto_no_detiene_a_los_demas() {
        let c = carpeta_de_prueba("entera");
        let mut corpus = Corpus::en_memoria().unwrap();
        let vistos = std::sync::Mutex::new(Vec::new());
        corpus.indexar(&c, &|d| vistos.lock().unwrap().push(d.nombre.clone())).unwrap();

        let e = corpus.estado();
        assert_eq!(e.documentos, 3, "se esperaban los dos .md y el .pdf roto: {:?}", corpus.documentos());
        assert_eq!(e.ilegibles, 1);
        assert_eq!(vistos.lock().unwrap().len(), 3, "el aviso de progreso no llegó por cada documento");

        // Y los dos buenos SÍ se indexaron, que es lo que la promesa significa.
        let h = corpus.buscar("y la tarifa de esas semanas de trabajo", 3).unwrap();
        assert!(!h.is_empty(), "el documento roto se llevó por delante a los sanos");
        assert_eq!(h[0].seccion.as_deref(), Some("Precio"));
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn el_documento_ilegible_trae_su_motivo_en_espanol_llano() {
        let c = carpeta_de_prueba("motivo");
        let mut corpus = Corpus::en_memoria().unwrap();
        corpus.indexar(&c, &|_| {}).unwrap();
        let roto = corpus.documentos().iter().find(|d| d.nombre.contains("acta")).unwrap();
        match &roto.estado {
            Estado::SinLeer { motivo } => assert!(!motivo.is_empty()),
            otro => panic!("el .pdf roto se dio por indexado: {otro:?}"),
        }
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn baja_por_las_subcarpetas_y_no_mira_ni_lo_oculto_ni_lo_ajeno() {
        let c = carpeta_de_prueba("hondura");
        let mut corpus = Corpus::en_memoria().unwrap();
        corpus.indexar(&c, &|_| {}).unwrap();
        let nombres: Vec<&str> = corpus.documentos().iter().map(|d| d.nombre.as_str()).collect();
        assert!(nombres.iter().any(|n| n.contains("Cooperativa")), "no bajó a la subcarpeta");
        assert!(!nombres.iter().any(|n| n.contains("sincroniza")), "indexó un archivo oculto");
        assert!(!nombres.iter().any(|n| n.contains("hoja")), "abrió un formato que no lee");
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn el_estado_reparte_los_documentos_por_unidad() {
        let c = carpeta_de_prueba("unidades");
        let mut corpus = Corpus::en_memoria().unwrap();
        corpus.indexar(&c, &|_| {}).unwrap();
        let e = corpus.estado();
        let cuantos = |u: &str| e.por_unidad.iter().find(|p| p.unidad == u).unwrap().documentos;
        assert_eq!(cuantos("propuesta"), 1);
        assert_eq!(cuantos("caso"), 1);
        assert_eq!(e.por_unidad.len(), 5, "las cinco unidades salen siempre, aunque estén en cero");
        let _ = std::fs::remove_dir_all(&c);
    }

    #[test]
    fn sin_carpeta_senalada_el_corpus_esta_vacio_y_lo_dice() {
        let e = Corpus::en_memoria().unwrap().estado();
        assert_eq!(e.carpeta, None);
        assert_eq!(e.documentos, 0);
        assert_eq!(e.secciones, 0);
    }

    #[test]
    fn indexar_algo_que_no_es_carpeta_se_dice_en_vez_de_reventar() {
        let mut corpus = Corpus::en_memoria().unwrap();
        assert!(corpus.indexar(Path::new("/no/existe/esto"), &|_| {}).is_err());
    }
}
