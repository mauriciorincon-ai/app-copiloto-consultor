//! LA BANDEJA — lo que no decidiste en la reunión, esperando su ventana (ADR 016 §4).
//!
//! Al cerrar, las propuestas sin decidir se sellan aquí con su vencimiento: cierre + la ventana que
//! elegiste (al cerrar · 1 h · 3 h · fin del día · 24 h; techo 24 h). Con «al cerrar» no se escribe
//! nada. Vive en `~/Library/Application Support/<app>/bandeja/`, **no en Documentos**: la papelera
//! de iCloud guarda 30 días lo que se borra allí, y una lista que promete morir a las 3 h no puede
//! tener una copia que viva un mes. Y porque launchd sí puede borrar aquí, y en Documentos no (ADR
//! 016, «Hallazgo en vivo»).
//!
//! Este módulo escribe, y por eso **solo ve propuestas ya reducidas a una línea** (`propuestas/`
//! decidió qué se escribe de cada lado): ni un turno pasa por aquí. Usa la carpeta sellada de
//! `carpeta.rs` —el mismo formato `.ghost`, la misma llave— con su propio contenido.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::carpeta::{Carpeta, Llaves, Reunion};
use crate::notas::{pisar_propuesta, Encabezado};
use crate::propuestas::Propuesta;

/// La carpeta de la bandeja, dentro de la de la app.
pub const CARPETA: &str = "bandeja";
/// La versión del contenido de dentro.
pub const VERSION: u32 = 1;
/// El techo de la ventana: 24 h desde el cierre, elijas lo que elijas.
pub const TECHO: i64 = 86_400;

/// Cuánto espera la bandeja. Una preferencia tuya, como la retención.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ventana {
    /// Cero: no hay bandeja; lo que no decidiste muere al cerrar.
    #[serde(rename = "0")]
    AlCerrar,
    #[serde(rename = "1h")]
    UnaHora,
    #[default]
    #[serde(rename = "3h")]
    TresHoras,
    /// Las 23:59 del día en que cierras.
    #[serde(rename = "fin")]
    FinDelDia,
    #[serde(rename = "24h")]
    Dia,
}

impl Ventana {
    pub const TODAS: [Ventana; 5] = [Ventana::AlCerrar, Ventana::UnaHora, Ventana::TresHoras, Ventana::FinDelDia, Ventana::Dia];

    /// Cuándo vence una bandeja cerrada en `cierre`. `None` con «al cerrar»: no hay bandeja.
    /// `fin_del_dia` es el último segundo del día del cierre en la hora del Mac (lo calcula quien
    /// llama: este módulo no mira el reloj). Nunca pasa del techo de 24 h.
    pub fn vence(self, cierre: i64, fin_del_dia: i64) -> Option<i64> {
        let v = match self {
            Ventana::AlCerrar => return None,
            Ventana::UnaHora => cierre + 3_600,
            Ventana::TresHoras => cierre + 3 * 3_600,
            Ventana::FinDelDia => fin_del_dia,
            Ventana::Dia => cierre + TECHO,
        };
        Some(v.clamp(cierre + 1, cierre + TECHO))
    }
}

/// Lo que va dentro de un archivo de la bandeja, antes de cifrar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contenido {
    pub version: u32,
    /// El archivo de notas al que pertenecen: «Guardar» las lleva ahí.
    pub reunion: String,
    /// Si la reunión no tenía nada tuyo, su archivo no existe todavía y nace con esto.
    pub encabezado: Encabezado,
    pub vence_de_la_reunion: i64,
    /// Cuándo se cerró la reunión (segundos Unix): el techo de la ventana se cuenta desde aquí.
    pub cerro: i64,
    pub propuestas: Vec<Propuesta>,
}

impl Drop for Contenido {
    fn drop(&mut self) {
        self.propuestas.iter_mut().for_each(pisar_propuesta);
    }
}

pub struct Bandeja {
    carpeta: Carpeta,
}

impl Bandeja {
    pub fn en(raiz: PathBuf) -> Bandeja {
        Bandeja { carpeta: Carpeta::en(raiz) }
    }

    pub fn raiz(&self) -> &std::path::Path {
        self.carpeta.raiz()
    }

    /// Deja las propuestas sin decidir en la bandeja, con su vencimiento. Sin propuestas o con
    /// `vence` en `None` («al cerrar»), **no escribe nada** y devuelve `false`.
    pub fn dejar(&self, llaves: &dyn Llaves, c: &Contenido, vence: Option<i64>) -> Result<bool, String> {
        let Some(vence) = vence else { return Ok(false) };
        if c.propuestas.is_empty() {
            return Ok(false);
        }
        let mut claro = serde_json::to_vec(c).map_err(|e| e.to_string())?;
        let r = self.carpeta.escribir_sellado(llaves, &c.reunion, &claro, vence);
        claro.fill(0);
        r.map(|_| true)
    }

    /// Las bandejas que hay, sin abrirlas: nombre, tamaño y vencimiento.
    pub fn lista(&self) -> Vec<Reunion> {
        self.carpeta.lista()
    }

    /// Abre una. Quien llama decide si hace falta el desbloqueo (ADR 016 §4).
    pub fn abrir(&self, llaves: &dyn Llaves, archivo: &str) -> Result<(Contenido, i64), String> {
        let (mut claro, vence) = self.carpeta.abrir_en_claro(llaves, archivo)?;
        let c: Result<Contenido, String> = serde_json::from_slice(&claro).map_err(|e| format!("la bandeja no se lee: {e}"));
        claro.fill(0);
        let c = c?;
        if c.version != VERSION {
            return Err(format!("una bandeja de una versión que esta app no conoce ({})", c.version));
        }
        Ok((c, vence))
    }

    /// **Guardar**: la propuesta `indice` pasa a su reunión (en `notas`) y sale de la bandeja.
    pub fn guardar(&self, llaves: &dyn Llaves, notas: &Carpeta, archivo: &str, indice: usize) -> Result<(), String> {
        let (c, _) = self.abrir(llaves, archivo)?;
        let propuesta = c.propuestas.get(indice).cloned().ok_or("esa propuesta ya no está en la bandeja")?;
        notas.sumar_propuesta(llaves, &c.reunion, propuesta, &c.encabezado, c.vence_de_la_reunion)?;
        self.quitar(llaves, archivo, indice)
    }

    /// **No**: la propuesta `indice` sale de la bandeja y muere. Una bandeja vacía se borra.
    pub fn quitar(&self, llaves: &dyn Llaves, archivo: &str, indice: usize) -> Result<(), String> {
        let (mut c, vence) = self.abrir(llaves, archivo)?;
        if indice >= c.propuestas.len() {
            return Err("esa propuesta ya no está en la bandeja".into());
        }
        let mut fuera = c.propuestas.remove(indice);
        pisar_propuesta(&mut fuera);
        if c.propuestas.is_empty() {
            return self.carpeta.borrar(archivo);
        }
        let mut claro = serde_json::to_vec(&c).map_err(|e| e.to_string())?;
        let r = self.carpeta.escribir_sellado(llaves, archivo, &claro, vence);
        claro.fill(0);
        r.map(|_| ())
    }

    /// Lo vencido a `ahora`, borrado. Devuelve cuántas bandejas.
    pub fn barrer(&self, ahora: i64) -> usize {
        self.carpeta.barrer(ahora)
    }

    /// Lo que vence, para la tarea de launchd.
    pub fn pendientes(&self) -> Vec<crate::vencimiento::Pendiente> {
        self.carpeta.pendientes()
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::carpeta::doble::EnMemoria;
    use crate::propuestas::{De, Regla};

    fn carpeta(nombre: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("ag-bandeja-{nombre}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    fn propuesta(texto: &str) -> Propuesta {
        Propuesta { regla: Regla::Cifra, de: De::Cliente, texto: texto.into(), ficha: None, seccion: None, hora: "14:16".into() }
    }

    fn contenido(reunion: &str, textos: &[&str]) -> Contenido {
        Contenido {
            version: VERSION,
            reunion: reunion.into(),
            encabezado: Encabezado { empezo: "2026-09-27 14:02".into(), minutos: 47, cliente: None },
            vence_de_la_reunion: CIERRE + 90 * 86_400,
            cerro: CIERRE,
            propuestas: textos.iter().map(|t| propuesta(t)).collect(),
        }
    }

    const CIERRE: i64 = 1_790_517_600;

    #[test]
    fn la_ventana_tiene_su_techo_y_al_cerrar_es_cero() {
        let fin = CIERRE + 10 * 3_600;
        assert_eq!(Ventana::AlCerrar.vence(CIERRE, fin), None);
        assert_eq!(Ventana::UnaHora.vence(CIERRE, fin), Some(CIERRE + 3_600));
        assert_eq!(Ventana::TresHoras.vence(CIERRE, fin), Some(CIERRE + 3 * 3_600));
        assert_eq!(Ventana::FinDelDia.vence(CIERRE, fin), Some(fin));
        assert_eq!(Ventana::Dia.vence(CIERRE, fin), Some(CIERRE + TECHO));
        assert_eq!(Ventana::FinDelDia.vence(CIERRE, CIERRE + 30 * 3_600), Some(CIERRE + TECHO), "el techo es 24 h");
        assert_eq!(Ventana::default(), Ventana::TresHoras, "de fábrica, 3 h");
    }

    /// **«Al cerrar» no escribe nada**, ni la carpeta. Demostrado en rojo: con `let vence =
    /// vence.unwrap_or(c.cerro)`, la bandeja se escribe aunque la ventana sea cero.
    #[test]
    fn con_la_ventana_en_cero_no_se_escribe_nada() {
        let raiz = carpeta("cero");
        let b = Bandeja::en(raiz.clone());
        let llaves = EnMemoria::default();
        assert!(!b.dejar(&llaves, &contenido("reunion-2026-09-27-1402.ghost", &["12 semanas"]), Ventana::AlCerrar.vence(CIERRE, CIERRE)).unwrap());
        assert!(!b.dejar(&llaves, &contenido("reunion-2026-09-27-1402.ghost", &[]), Some(CIERRE + 60)).unwrap());
        assert!(!raiz.exists(), "con la ventana en cero se creó la bandeja");
    }

    #[test]
    fn la_bandeja_nace_cerrada_cifrada_y_con_su_vencimiento() {
        use std::os::unix::fs::PermissionsExt;
        let raiz = carpeta("nace");
        let b = Bandeja::en(raiz.clone());
        let llaves = EnMemoria::default();
        let vence = CIERRE + 3 * 3_600;
        assert!(b.dejar(&llaves, &contenido("reunion-2026-09-27-1402.ghost", &["12 semanas", "cuatro fuentes"]), Some(vence)).unwrap());
        let ruta = raiz.join("reunion-2026-09-27-1402.ghost");
        assert_eq!(std::fs::metadata(&ruta).unwrap().permissions().mode() & 0o777, 0o600);
        assert_eq!(std::fs::metadata(&raiz).unwrap().permissions().mode() & 0o777, 0o700);
        let bytes = std::fs::read(&ruta).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("semanas"), "la bandeja está en claro");
        assert_eq!(b.lista()[0].vence, vence);
        let (c, v) = b.abrir(&llaves, "reunion-2026-09-27-1402.ghost").unwrap();
        assert_eq!((c.propuestas.len(), v), (2, vence));
        let _ = std::fs::remove_dir_all(&raiz);
    }

    /// **Guardar desde la bandeja** lleva la propuesta a su reunión, que conserva su vencimiento, y
    /// la quita de la bandeja; la última se lleva el archivo de la bandeja.
    #[test]
    fn guardar_la_lleva_a_su_reunion_y_no_la_quita() {
        let raiz = carpeta("guardar");
        let b = Bandeja::en(raiz.join("bandeja"));
        let notas = Carpeta::en(raiz.join("notas"));
        let llaves = EnMemoria::default();
        let archivo = "reunion-2026-09-27-1402.ghost";
        b.dejar(&llaves, &contenido(archivo, &["12 semanas", "cuatro fuentes", "el viernes"]), Some(CIERRE + 3_600)).unwrap();

        // la reunión no tenía nada tuyo: su archivo nace con el vencimiento que habría tenido
        b.guardar(&llaves, &notas, archivo, 0).unwrap();
        let reunion = notas.lista();
        assert_eq!(reunion.len(), 1);
        assert_eq!(reunion[0].vence, CIERRE + 90 * 86_400);
        assert_eq!(notas.abrir(&llaves, archivo).unwrap().propuestas[0].texto, "12 semanas");

        // la segunda se suma a la misma reunión, sin cambiar su vencimiento
        b.guardar(&llaves, &notas, archivo, 0).unwrap();
        let c = notas.abrir(&llaves, archivo).unwrap();
        assert_eq!(c.propuestas.iter().map(|p| p.texto.as_str()).collect::<Vec<_>>(), ["12 semanas", "cuatro fuentes"]);
        assert_eq!(notas.lista()[0].vence, CIERRE + 90 * 86_400);

        // la bandeja se queda con la tercera y su vencimiento; «No» se la lleva, y el archivo con ella
        let (quedan, v) = b.abrir(&llaves, archivo).unwrap();
        assert_eq!((quedan.propuestas.len(), v), (1, CIERRE + 3_600));
        b.quitar(&llaves, archivo, 0).unwrap();
        assert!(b.lista().is_empty(), "una bandeja vacía se quedó en el disco");
        let _ = std::fs::remove_dir_all(&raiz);
    }

    #[test]
    fn lo_vencido_se_barre_y_lo_demas_se_queda_y_va_a_la_lista_de_launchd() {
        let raiz = carpeta("barrer");
        let b = Bandeja::en(raiz.clone());
        let llaves = EnMemoria::default();
        b.dejar(&llaves, &contenido("reunion-2026-09-27-1402.ghost", &["12 semanas"]), Some(CIERRE + 60)).unwrap();
        b.dejar(&llaves, &contenido("reunion-2026-09-27-1600.ghost", &["el viernes"]), Some(CIERRE + 3_600)).unwrap();
        assert_eq!(b.pendientes().len(), 2);
        assert_eq!(b.barrer(CIERRE + 61), 1);
        let quedan = b.pendientes();
        assert_eq!(quedan.len(), 1);
        assert!(quedan[0].ruta.ends_with("reunion-2026-09-27-1600.ghost"));
        let _ = std::fs::remove_dir_all(&raiz);
    }

    #[test]
    fn un_nombre_que_se_sale_de_la_carpeta_no_se_abre() {
        let b = Bandeja::en(carpeta("nombre"));
        assert!(b.abrir(&EnMemoria::default(), "../notas/x.ghost").is_err());
        assert!(b.quitar(&EnMemoria::default(), "/etc/passwd", 0).is_err());
    }
}
