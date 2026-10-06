//! LAS NOTAS — lo tuyo, lo único que sobrevive a la reunión (C9, ADR 015).
//!
//! **MÓDULO PROTEGIDO.** El cuaderno recibe los turnos del micrófono para «Conservar mis turnos», y
//! entre ellos llegan los marcados como **eco**: el micrófono oyendo al cliente por los altavoces. Es
//! la voz del cliente. Por eso este módulo, que decide qué es tuyo, **no tiene manera de escribir**:
//! arma el contenido, lo cifra (`cifrado`) y entrega bytes. Los escribe `carpeta.rs`, que no ve un
//! solo turno. Es el patrón del diccionario (ADR 002, enmienda 1). `pnpm verify:ephemeral` lo vigila.
//!
//! **Qué entra y qué no** está en la tabla del ADR 015 §1. Lo que este archivo hace cumplir:
//! - de los turnos, **solo** los del micrófono y **sin eco**, y solo con la casilla encendida;
//! - de las fichas, **solo las que fijas tú**: la que la banda enseña («la vigente») vive aquí para
//!   que ⌃⌥P tenga qué fijar, y muere con el corte — la eligió el disparador por las palabras del
//!   cliente, y su lista sería la huella de lo que dijo;
//! - ⌥⎋ corta tus turnos, la vigente y las propuestas sin decidir; tu nota, tus acuerdos, tus fijadas
//!   y las propuestas que guardaste **sobreviven**;
//! - las propuestas (ADR 016) esperan aquí tu sí: sin él, no entran al archivo.

pub mod cifrado;

use serde::{Deserialize, Serialize};

use crate::capture::Quien;
use crate::corpus::Unidad;
use crate::propuestas::{self, Propuesta};
use crate::stt::Turno;

/// La versión del contenido (el JSON de dentro). La del sobre es `cifrado::VERSION`.
pub const VERSION: u32 = 1;

/// Tope de la nota, en letras. Muy por encima de cualquier reunión (una hora dictando son ~9 000
/// palabras): está para que un pegado accidental de un documento entero no acabe cifrado a disco.
pub const TOPE_DE_LA_NOTA: usize = 200_000;
/// Tope de un acuerdo, en letras. Un acuerdo es una línea.
pub const TOPE_DEL_ACUERDO: usize = 500;
/// Cuántas propuestas pueden esperar a la vez (ADR 016 §1). Al llegar, las nuevas no entran.
pub const TOPE_DE_PROPUESTAS: usize = 30;

/// Una ficha que fijaste: cómo se llamaba y de dónde salía. **No** el texto del documento: ese sigue
/// en tu corpus, y copiarlo aquí sería guardar dos veces lo mismo.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FichaFijada {
    pub titular: String,
    pub documento: String,
    pub seccion: Option<String>,
    pub unidad: Option<Unidad>,
    /// La línea de la ficha, **solo en memoria**: la regla `choque` compara con ella las cifras que
    /// se dicen (ADR 016). No va al archivo ni a la pantalla: es texto de tu documento, que ya está en
    /// tu corpus.
    #[serde(skip)]
    pub linea: String,
}

/// Un turno tuyo, en texto. La hora es la del reloj, «14:02», como en el transcript.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnoPropio {
    pub hora: String,
    pub texto: String,
}

/// Cuándo y cuánto: lo que el cuaderno no sabe por sí solo y le dice quien lo guarda.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Encabezado {
    /// «2026-09-27 14:02», en la hora del Mac.
    pub empezo: String,
    pub minutos: u32,
    /// El cliente elegido en «Este cliente» (fase 3). Sin él, nada: la app no lo adivina.
    pub cliente: Option<String>,
}

/// Lo que va dentro del archivo, antes de cifrar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contenido {
    pub version: u32,
    #[serde(flatten)]
    pub encabezado: Encabezado,
    pub nota: String,
    pub acuerdos: Vec<String>,
    pub fijadas: Vec<FichaFijada>,
    pub mis_turnos: Vec<TurnoPropio>,
    /// Las propuestas que guardaste (ADR 016 §3). Un archivo de antes de la fase 2 no las trae y se
    /// lee como una lista vacía.
    #[serde(default)]
    pub propuestas: Vec<Propuesta>,
}

impl Contenido {
    pub fn a_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("el contenido de las notas siempre se serializa")
    }

    pub fn de_bytes(bytes: &[u8]) -> Result<Contenido, String> {
        let c: Contenido = serde_json::from_slice(bytes).map_err(|e| format!("contenido ilegible: {e}"))?;
        if c.version != VERSION {
            return Err(format!("contenido de una versión que esta app no conoce ({})", c.version));
        }
        Ok(c)
    }
}

impl Drop for Contenido {
    fn drop(&mut self) {
        pisar(&mut self.nota);
        self.acuerdos.iter_mut().for_each(pisar);
        self.mis_turnos.iter_mut().for_each(|t| pisar(&mut t.texto));
        self.propuestas.iter_mut().for_each(pisar_propuesta);
    }
}

/// Cuánto hay de cada cosa. Es lo que enseña «Se va a guardar» y lo único que va al log.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Resumen {
    /// Párrafos de la nota: bloques separados por una línea en blanco o un salto.
    pub parrafos: usize,
    pub acuerdos: usize,
    pub fijadas: usize,
    pub turnos: usize,
    /// Las propuestas que guardaste, y las que esperan tu decisión.
    pub propuestas: usize,
    pub sin_decidir: usize,
    /// Bytes en claro de cada parte, para las cifras de «Se va a guardar».
    pub bytes_nota: usize,
    pub bytes_acuerdos: usize,
    pub bytes_fijadas: usize,
    pub bytes_turnos: usize,
}

/// Una propuesta que espera tu decisión, con el número con que la pantalla la nombra.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnEspera {
    pub id: u32,
    #[serde(flatten)]
    pub propuesta: Propuesta,
}

/// El cuaderno de la reunión, en memoria.
#[derive(Debug, Default)]
pub struct Cuaderno {
    nota: String,
    acuerdos: Vec<String>,
    fijadas: Vec<FichaFijada>,
    mis_turnos: Vec<TurnoPropio>,
    conservar_mis_turnos: bool,
    /// La ficha que la banda enseña ahora. No es tuya todavía: lo es si la fijas.
    vigente: Option<FichaFijada>,
    /// Las propuestas que esperan tu sí, en el orden en que llegaron.
    en_espera: Vec<EnEspera>,
    /// Las que guardaste: entran al archivo.
    guardadas: Vec<Propuesta>,
    /// Lo que ya se propuso en esta reunión, para no repetirlo aunque lo descartaras: **una huella**
    /// de regla y texto plegados, nunca el texto (auditoría del S3, M1: «nombre:andrea villalba»
    /// sobrevivía al corte). La llave de la huella nace al azar con cada cuaderno, y ⌥⎋ la vacía.
    propuestas_vistas: std::collections::HashSet<u64>,
    vistas_con: std::hash::RandomState,
    siguiente: u32,
    /// Se llegó al tope de propuestas y alguna no entró. La pantalla lo dice.
    lleno: bool,
}

impl Cuaderno {
    pub fn nuevo(conservar_mis_turnos: bool) -> Cuaderno {
        Cuaderno {
            nota: String::new(),
            acuerdos: Vec::new(),
            fijadas: Vec::new(),
            mis_turnos: Vec::new(),
            conservar_mis_turnos,
            vigente: None,
            en_espera: Vec::new(),
            guardadas: Vec::new(),
            propuestas_vistas: std::collections::HashSet::new(),
            vistas_con: std::hash::RandomState::new(),
            siguiente: 1,
            lleno: false,
        }
    }

    /// Tu nota entera, tal como está en el campo. Se reemplaza, no se añade: el campo es la verdad.
    pub fn escribir(&mut self, texto: &str) {
        let recortado: String = texto.chars().take(TOPE_DE_LA_NOTA).collect();
        pisar(&mut self.nota);
        self.nota = recortado;
    }

    pub fn nota(&self) -> &str {
        &self.nota
    }

    /// Un acuerdo que escribiste. Vacío no es un acuerdo; el mismo dos veces seguidas, tampoco.
    pub fn acordar(&mut self, texto: &str) -> bool {
        let t: String = texto.trim().chars().take(TOPE_DEL_ACUERDO).collect();
        if t.is_empty() || self.acuerdos.last().is_some_and(|u| *u == t) {
            return false;
        }
        self.acuerdos.push(t);
        true
    }

    pub fn quitar_acuerdo(&mut self, indice: usize) -> bool {
        if indice >= self.acuerdos.len() {
            return false;
        }
        let mut quitado = self.acuerdos.remove(indice);
        pisar(&mut quitado);
        true
    }

    pub fn acuerdos(&self) -> &[String] {
        &self.acuerdos
    }

    /// La banda enseña esta ficha. Solo la ficha: un «sin resultado» no se puede fijar.
    pub fn ver(&mut self, ficha: FichaFijada) {
        self.vigente = Some(ficha);
    }

    /// ⌃⌥P: fija la ficha que la banda enseña. `None` si no hay ninguna; la misma no se fija dos veces.
    pub fn fijar_la_vigente(&mut self) -> Option<&FichaFijada> {
        let v = self.vigente.clone()?;
        if !self.fijadas.contains(&v) {
            self.fijadas.push(v);
        }
        self.fijadas.iter().find(|f| **f == *self.vigente.as_ref().expect("se acaba de leer"))
    }

    pub fn soltar_fijada(&mut self, indice: usize) -> bool {
        if indice >= self.fijadas.len() {
            return false;
        }
        self.fijadas.remove(indice);
        true
    }

    pub fn fijadas(&self) -> &[FichaFijada] {
        &self.fijadas
    }

    /// Un turno recién transcrito. Entra **solo** si es tuyo: pista del micrófono, sin eco, y con
    /// «Conservar mis turnos» encendido. Devuelve si entró.
    ///
    /// El eco es la regla que importa: con altavoces, el micrófono oye al cliente, y el turno que llega
    /// por tu pista con la marca de eco **es la voz del cliente**. Aunque la casilla esté encendida.
    pub fn oir(&mut self, turno: &Turno) -> bool {
        // Tuyo es solo lo de tu pista, sin eco. **La sala nunca** (ADR 020 §5): no se sabe de quién es.
        let es_tuyo = match turno.pista.quien() {
            Quien::Tuyo => !turno.eco,
            Quien::Cliente | Quien::SinAtribuir => false,
        };
        if !self.conservar_mis_turnos || !es_tuyo {
            return false;
        }
        let texto = turno.texto.trim();
        if texto.is_empty() {
            return false;
        }
        self.mis_turnos.push(TurnoPropio { hora: turno.hora.clone(), texto: texto.to_string() });
        true
    }

    /// Encender o apagar «Conservar mis turnos». Apagarlo **tira los que había**: si ya no quieres
    /// que queden, no quedan.
    pub fn conservar_mis_turnos(&mut self, si: bool) {
        self.conservar_mis_turnos = si;
        if !si {
            self.tirar_mis_turnos();
        }
    }

    pub fn conserva_mis_turnos(&self) -> bool {
        self.conservar_mis_turnos
    }

    /// ⌥⎋, la pieza `TusTurnos`. Mueren tus turnos (salen de la captura) y la ficha vigente (la
    /// eligió el cliente con sus palabras). Tu nota, tus acuerdos y tus fijadas siguen: «Tus notas
    /// siguen ahí».
    pub fn cortar(&mut self) {
        self.tirar_mis_turnos();
        self.vigente = None;
    }

    /// ⌥⎋, la pieza `Propuestas` (ADR 016 §4): mueren las que esperaban tu decisión, porque salen de
    /// los turnos. Las que guardaste siguen, como tus notas. Y se olvida qué se propuso (M1): lo que
    /// se diga después del corte es captura nueva.
    pub fn cortar_las_propuestas(&mut self) {
        self.tirar_las_que_esperan();
        self.propuestas_vistas.clear();
    }

    /// ¿Hay algo tuyo que guardar?
    pub fn vacio(&self) -> bool {
        self.nota.trim().is_empty()
            && self.acuerdos.is_empty()
            && self.fijadas.is_empty()
            && self.mis_turnos.is_empty()
            && self.guardadas.is_empty()
    }

    /// ¿Queda algo que decidir al cerrar? Lo tuyo, o propuestas esperando (ADR 016 §4): una reunión
    /// así no se cierra sola al parar.
    pub fn hay_que_decidir(&self) -> bool {
        !self.vacio() || !self.en_espera.is_empty()
    }

    // ---- las propuestas (ADR 016) ----

    /// Las fijadas como las ven las reglas: titular, línea y sección.
    pub fn para_las_reglas(&self) -> Vec<propuestas::Fijada> {
        self.fijadas
            .iter()
            .map(|f| propuestas::Fijada { titular: f.titular.clone(), linea: f.linea.clone(), seccion: f.seccion.clone() })
            .collect()
    }

    /// Llegan propuestas de un turno. Entran las que no se propusieron ya en esta reunión, hasta el
    /// tope. Devuelve cuántas entraron.
    pub fn proponer(&mut self, nuevas: Vec<Propuesta>) -> usize {
        let mut entraron = 0;
        for p in nuevas {
            let mut texto = format!("{}:{}", p.regla.id(), propuestas::plegar(&p.texto));
            let clave = std::hash::BuildHasher::hash_one(&self.vistas_con, &texto);
            pisar(&mut texto);
            if self.propuestas_vistas.contains(&clave) {
                continue;
            }
            if self.en_espera.len() >= TOPE_DE_PROPUESTAS {
                self.lleno = true;
                continue;
            }
            self.propuestas_vistas.insert(clave);
            self.en_espera.push(EnEspera { id: self.siguiente, propuesta: p });
            self.siguiente += 1;
            entraron += 1;
        }
        entraron
    }

    pub fn en_espera(&self) -> &[EnEspera] {
        &self.en_espera
    }

    pub fn guardadas(&self) -> &[Propuesta] {
        &self.guardadas
    }

    /// ¿Se quedó alguna fuera por el tope?
    pub fn lleno(&self) -> bool {
        self.lleno
    }

    /// «Guardar»: la propuesta pasa a tu archivo.
    pub fn guardar_propuesta(&mut self, id: u32) -> bool {
        let Some(i) = self.en_espera.iter().position(|e| e.id == id) else { return false };
        let e = self.en_espera.remove(i);
        self.guardadas.push(e.propuesta);
        true
    }

    /// ⌃⌥↵: guarda la última que llegó. Devuelve la guardada.
    pub fn guardar_la_ultima(&mut self) -> Option<&Propuesta> {
        let id = self.en_espera.last()?.id;
        self.guardar_propuesta(id);
        self.guardadas.last()
    }

    /// «No»: muere en ese momento.
    pub fn descartar_propuesta(&mut self, id: u32) -> bool {
        let Some(i) = self.en_espera.iter().position(|e| e.id == id) else { return false };
        let mut e = self.en_espera.remove(i);
        pisar_propuesta(&mut e.propuesta);
        true
    }

    /// Al cerrar: las que no decidiste salen del cuaderno (a la bandeja, o a morir si la ventana es
    /// «al cerrar»).
    pub fn sin_decidir(&mut self) -> Vec<Propuesta> {
        self.en_espera.drain(..).map(|e| e.propuesta).collect()
    }

    fn tirar_las_que_esperan(&mut self) {
        self.en_espera.iter_mut().for_each(|e| pisar_propuesta(&mut e.propuesta));
        self.en_espera.clear();
    }

    pub fn resumen(&self) -> Resumen {
        let fijadas_bytes = self
            .fijadas
            .iter()
            .map(|f| f.titular.len() + f.documento.len() + f.seccion.as_ref().map_or(0, |s| s.len()))
            .sum();
        Resumen {
            parrafos: self.nota.split('\n').filter(|l| !l.trim().is_empty()).count(),
            acuerdos: self.acuerdos.len(),
            fijadas: self.fijadas.len(),
            turnos: self.mis_turnos.len(),
            propuestas: self.guardadas.len(),
            sin_decidir: self.en_espera.len(),
            bytes_nota: self.nota.len(),
            bytes_acuerdos: self.acuerdos.iter().map(String::len).sum(),
            bytes_fijadas: fijadas_bytes,
            bytes_turnos: self.mis_turnos.iter().map(|t| t.texto.len()).sum(),
        }
    }

    /// Lo que va al archivo. No vacía el cuaderno: eso lo hace [`Cuaderno::olvidar`] cuando el
    /// archivo ya está escrito, para que un fallo al guardar no se lleve la nota.
    pub fn contenido(&self, encabezado: Encabezado) -> Contenido {
        Contenido {
            version: VERSION,
            encabezado,
            nota: self.nota.clone(),
            acuerdos: self.acuerdos.clone(),
            fijadas: self.fijadas.clone(),
            mis_turnos: if self.conservar_mis_turnos { self.mis_turnos.clone() } else { Vec::new() },
            propuestas: self.guardadas.clone(),
        }
    }

    /// Guardado o descartado: el cuaderno vuelve a empezar, con las letras pisadas.
    pub fn olvidar(&mut self) {
        let conservar = self.conservar_mis_turnos;
        pisar(&mut self.nota);
        self.acuerdos.iter_mut().for_each(pisar);
        self.tirar_mis_turnos();
        self.tirar_las_que_esperan();
        self.guardadas.iter_mut().for_each(pisar_propuesta);
        *self = Cuaderno::nuevo(conservar);
    }

    fn tirar_mis_turnos(&mut self) {
        self.mis_turnos.iter_mut().for_each(|t| pisar(&mut t.texto));
        self.mis_turnos.clear();
    }
}

impl Drop for Cuaderno {
    fn drop(&mut self) {
        pisar(&mut self.nota);
        self.acuerdos.iter_mut().for_each(pisar);
        self.mis_turnos.iter_mut().for_each(|t| pisar(&mut t.texto));
        self.en_espera.iter_mut().for_each(|e| pisar_propuesta(&mut e.propuesta));
        self.guardadas.iter_mut().for_each(pisar_propuesta);
    }
}

/// Una propuesta se pisa entera: su texto y lo que dice la ficha.
pub fn pisar_propuesta(p: &mut Propuesta) {
    pisar(&mut p.texto);
    if let Some(f) = p.ficha.as_mut() {
        pisar(f);
    }
}

/// Las letras se pisan con ceros antes de soltarlas, como el resto de la casa.
fn pisar(s: &mut String) {
    // SEGURIDAD: ceros son UTF-8 válido, así que el `String` sigue siendo un `String`.
    unsafe { s.as_mut_vec() }.fill(0);
}

/// Una fecha y hora del reloj del Mac. La calcula quien llama: este módulo no mira el reloj.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fecha {
    pub anio: i32,
    pub mes: u32,
    pub dia: u32,
    pub hora: u32,
    pub minuto: u32,
}

impl Fecha {
    /// «2026-09-27 14:02».
    pub fn como_texto(&self) -> String {
        format!("{}-{:02}-{:02} {:02}:{:02}", self.anio, self.mes, self.dia, self.hora, self.minuto)
    }
}

/// **La última línea de tu nota**, con la que `⌃⌥A` busca en modo solo notas (ADR 017 §5): la última
/// que no esté en blanco, sin viñeta delante. `None` si la nota está vacía.
pub fn ultima_linea(nota: &str) -> Option<&str> {
    nota.lines()
        .map(|l| l.trim().trim_start_matches(['-', '*', '•']).trim())
        .rfind(|l| !l.is_empty())
}

/// El nombre del archivo, sin la extensión: `paramo-azul-2026-09-20`, o
/// `reunion-2026-09-27-1402` si no se eligió cliente. Lo que dibuja la maqueta.
pub fn nombre_del_archivo(cliente: Option<&str>, f: &Fecha) -> String {
    let dia = format!("{}-{:02}-{:02}", f.anio, f.mes, f.dia);
    match cliente.map(slug).filter(|s| !s.is_empty()) {
        Some(c) => format!("{c}-{dia}"),
        None => format!("reunion-{dia}-{:02}{:02}", f.hora, f.minuto),
    }
}

/// «Páramo Azul» → «paramo-azul». Minúsculas, sin tildes, y guiones entre palabras. Lo que no sea
/// letra o cifra se va: el nombre de un archivo no es sitio para barras ni puntos.
pub fn slug(texto: &str) -> String {
    let mut salida = String::new();
    let mut guion = false;
    for c in texto.chars().flat_map(char::to_lowercase) {
        let c = match c {
            'á' | 'à' | 'ä' | 'â' | 'ã' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' | 'õ' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            'ç' => 'c',
            otro => otro,
        };
        if c.is_ascii_alphanumeric() {
            if guion && !salida.is_empty() {
                salida.push('-');
            }
            salida.push(c);
            guion = false;
        } else {
            guion = true;
        }
    }
    salida.chars().take(60).collect::<String>().trim_end_matches('-').to_string()
}

#[cfg(test)]
mod pruebas {
    use crate::capture::Pista;

    /// ⌃⌥A en solo notas busca con esto: la última línea con algo, sin su viñeta.
    #[test]
    fn la_ultima_linea_de_la_nota() {
        assert_eq!(ultima_linea("Piden la cuarta fuente.\n- ETL con tres fuentes\n\n  "), Some("ETL con tres fuentes"));
        assert_eq!(ultima_linea("una sola"), Some("una sola"));
        assert_eq!(ultima_linea("   \n\n"), None);
    }

    use super::*;

    fn turno(pista: Pista, eco: bool, texto: &str) -> Turno {
        Turno { pista, desde_ms: 0, hasta_ms: 1_000, texto: texto.into(), hora: "14:02".into(), eco }
    }

    fn ficha(titular: &str) -> FichaFijada {
        FichaFijada {
            titular: titular.into(),
            documento: "Propuesta Páramo Azul".into(),
            seccion: Some("Alcance".into()),
            unidad: Some(Unidad::Propuesta),
            ..Default::default()
        }
    }

    #[test]
    fn de_los_turnos_solo_entran_los_tuyos_y_sin_eco() {
        let mut c = Cuaderno::nuevo(true);
        assert!(c.oir(&turno(Pista::Microfono, false, "Te envío la cotización el lunes.")));
        assert!(!c.oir(&turno(Pista::Sistema, false, "Del cliente")), "un turno del cliente entró al archivo");
        assert!(
            !c.oir(&turno(Pista::Microfono, true, "El cliente, oído por tus altavoces")),
            "un turno con eco es la voz del cliente y entró al archivo"
        );
        assert_eq!(c.resumen().turnos, 1);
        let dentro = String::from_utf8(c.contenido(encabezado()).a_bytes()).unwrap();
        assert!(dentro.contains("cotización"));
        assert!(!dentro.contains("Del cliente") && !dentro.contains("oído por tus altavoces"));
    }

    #[test]
    fn con_la_casilla_apagada_no_entra_ni_un_turno_y_apagarla_tira_los_que_habia() {
        let mut c = Cuaderno::nuevo(false);
        assert!(!c.oir(&turno(Pista::Microfono, false, "Mío, pero no lo pedí")));
        c.conservar_mis_turnos(true);
        assert!(c.oir(&turno(Pista::Microfono, false, "Ahora sí")));
        c.conservar_mis_turnos(false);
        assert_eq!(c.resumen().turnos, 0, "apagar la casilla tiene que tirar los que ya había");
        assert!(c.contenido(encabezado()).mis_turnos.is_empty());
    }

    #[test]
    fn el_corte_se_lleva_tus_turnos_y_la_vigente_pero_no_tus_notas() {
        let mut c = Cuaderno::nuevo(true);
        c.escribir("Piden la cuarta fuente.");
        c.acordar("Cuarta fuente: cotización aparte");
        c.ver(ficha("Limpieza de datos: hasta tres fuentes"));
        c.fijar_la_vigente();
        c.ver(ficha("Sur del Valle: 11 semanas reales"));
        c.oir(&turno(Pista::Microfono, false, "Lo reviso y te escribo."));

        c.cortar();

        assert_eq!(c.nota(), "Piden la cuarta fuente.");
        assert_eq!(c.acuerdos().len(), 1);
        assert_eq!(c.fijadas().len(), 1, "las fijadas son tuyas y sobreviven al corte");
        assert_eq!(c.resumen().turnos, 0, "tus turnos salen de la captura y mueren con el corte");
        assert!(c.fijar_la_vigente().is_none(), "la ficha vigente la eligió el cliente: muere con el corte");
    }

    #[test]
    fn fijar_es_de_la_ficha_que_se_ve_y_una_sola_vez() {
        let mut c = Cuaderno::nuevo(false);
        assert!(c.fijar_la_vigente().is_none(), "sin ficha en la banda no hay qué fijar");
        c.ver(ficha("Limpieza de datos: hasta tres fuentes"));
        assert!(c.fijar_la_vigente().is_some());
        c.fijar_la_vigente();
        assert_eq!(c.fijadas().len(), 1);
        assert!(c.soltar_fijada(0));
        assert!(c.fijadas().is_empty());
    }

    #[test]
    fn un_acuerdo_es_una_linea_escrita() {
        let mut c = Cuaderno::nuevo(false);
        assert!(!c.acordar("   "));
        assert!(c.acordar("  Cuarta fuente: cotización aparte "));
        assert!(!c.acordar("Cuarta fuente: cotización aparte"), "el mismo dos veces seguidas");
        assert_eq!(c.acuerdos(), ["Cuarta fuente: cotización aparte"]);
        assert!(c.quitar_acuerdo(0));
        assert!(!c.quitar_acuerdo(0));
    }

    #[test]
    fn vacio_es_no_tener_nada_tuyo() {
        let mut c = Cuaderno::nuevo(false);
        assert!(c.vacio());
        c.ver(ficha("Una ficha que solo apareció"));
        assert!(c.vacio(), "una ficha que solo apareció no es tuya");
        c.escribir("  \n ");
        assert!(c.vacio());
        c.escribir("algo");
        assert!(!c.vacio());
        c.olvidar();
        assert!(c.vacio());
    }

    #[test]
    fn el_contenido_va_y_vuelve() {
        let mut c = Cuaderno::nuevo(true);
        c.escribir("Uno\n\nDos");
        c.acordar("Acuerdo");
        c.ver(ficha("Titular"));
        c.fijar_la_vigente();
        c.oir(&turno(Pista::Microfono, false, "Mío"));
        let dentro = c.contenido(encabezado());
        let vuelta = Contenido::de_bytes(&dentro.a_bytes()).unwrap();
        assert_eq!(vuelta, dentro);
        assert_eq!(c.resumen().parrafos, 2);
        let mut otra = dentro.clone();
        otra.version = 9;
        assert!(Contenido::de_bytes(&otra.a_bytes()).is_err());
    }

    #[test]
    fn el_nombre_es_el_de_la_maqueta() {
        let f = Fecha { anio: 2026, mes: 9, dia: 20, hora: 14, minuto: 2 };
        assert_eq!(nombre_del_archivo(Some("Páramo Azul"), &f), "paramo-azul-2026-09-20");
        assert_eq!(nombre_del_archivo(None, &f), "reunion-2026-09-20-1402");
        assert_eq!(nombre_del_archivo(Some("  ../../etc "), &f), "etc-2026-09-20");
        assert_eq!(nombre_del_archivo(Some("¡!"), &f), "reunion-2026-09-20-1402");
        assert_eq!(slug("Sur del Valle S.A.S."), "sur-del-valle-s-a-s");
    }

    fn encabezado() -> Encabezado {
        Encabezado { empezo: "2026-09-27 14:02".into(), minutos: 47, cliente: None }
    }

    fn propuesta(texto: &str) -> Propuesta {
        Propuesta {
            regla: propuestas::Regla::Cifra,
            de: propuestas::De::Cliente,
            texto: texto.into(),
            ficha: None,
            seccion: None,
            hora: "14:16".into(),
        }
    }

    /// **Proponer no es guardar.** Solo entra al archivo lo que dijiste que sí. Demostrado en rojo:
    /// con `propuestas: self.en_espera…` en `contenido`, las que esperan acaban cifradas en el archivo.
    #[test]
    fn solo_las_que_guardas_entran_al_archivo() {
        let mut c = Cuaderno::nuevo(false);
        assert_eq!(c.proponer(vec![propuesta("12 semanas"), propuesta("USD 40.000"), propuesta("el viernes")]), 3);
        let ids: Vec<u32> = c.en_espera().iter().map(|e| e.id).collect();
        assert!(c.guardar_propuesta(ids[0]));
        assert!(c.descartar_propuesta(ids[1]));
        let dentro = c.contenido(encabezado());
        assert_eq!(dentro.propuestas.iter().map(|p| p.texto.as_str()).collect::<Vec<_>>(), ["12 semanas"]);
        assert_eq!(c.resumen().propuestas, 1);
        assert_eq!(c.resumen().sin_decidir, 1);
        // y un archivo de antes de la fase 2, sin el campo, se lee como una lista vacía
        let mut sin = serde_json::to_value(&dentro).unwrap();
        sin.as_object_mut().unwrap().remove("propuestas");
        let vuelta = Contenido::de_bytes(&serde_json::to_vec(&sin).unwrap()).unwrap();
        assert!(vuelta.propuestas.is_empty());
    }

    #[test]
    fn la_misma_propuesta_no_se_repite_ni_despues_de_descartarla_y_hay_tope() {
        let mut c = Cuaderno::nuevo(false);
        c.proponer(vec![propuesta("12 semanas")]);
        let id = c.en_espera()[0].id;
        c.descartar_propuesta(id);
        assert_eq!(c.proponer(vec![propuesta("12 Semanas")]), 0, "descartada una vez, no vuelve");
        let muchas: Vec<Propuesta> = (0..40).map(|i| propuesta(&format!("{i} días"))).collect();
        assert_eq!(c.proponer(muchas), TOPE_DE_PROPUESTAS);
        assert!(c.lleno());
    }

    /// **El corte no deja copia de lo que se propuso** (auditoría del S3, M1). La memoria de «ya lo
    /// propuse» guardaba `regla:texto` en claro —«nombre:andrea villalba»—, y ni ⌥⎋ ni «No» la
    /// tocaban. Ahora guarda solo una huella con llave al azar por cuaderno y el corte la vacía: lo
    /// que se diga después es captura nueva. Demostrado en rojo con el código de antes: la lista
    /// seguía con el nombre del cliente tras el corte.
    #[test]
    fn el_corte_no_deja_copia_de_lo_que_se_propuso() {
        let mut c = Cuaderno::nuevo(false);
        c.proponer(vec![Propuesta { regla: propuestas::Regla::Nombre, texto: "Andrea Villalba".into(), ..propuesta("") }]);
        c.cortar();
        c.cortar_las_propuestas();
        assert!(c.propuestas_vistas.is_empty(), "lo que se propuso sobrevive al corte: {:?}", c.propuestas_vistas);
        assert_eq!(c.proponer(vec![Propuesta { regla: propuestas::Regla::Nombre, texto: "Andrea Villalba".into(), ..propuesta("") }]), 1);
    }

    #[test]
    fn la_ultima_se_guarda_con_una_tecla() {
        let mut c = Cuaderno::nuevo(false);
        assert!(c.guardar_la_ultima().is_none());
        c.proponer(vec![propuesta("12 semanas"), propuesta("el viernes")]);
        assert_eq!(c.guardar_la_ultima().map(|p| p.texto.clone()).as_deref(), Some("el viernes"));
        assert_eq!(c.en_espera().len(), 1);
    }

    /// **⌥⎋ se lleva las que esperan y deja las guardadas** (ADR 016 §4). Demostrado en rojo: sin
    /// `tirar_las_que_esperan` en `cortar_las_propuestas` (la pieza `Propuestas` del corte desde la
    /// fase 2), las propuestas del cliente sobreviven al corte.
    #[test]
    fn el_corte_se_lleva_las_que_esperan_y_deja_las_guardadas() {
        let mut c = Cuaderno::nuevo(false);
        c.proponer(vec![propuesta("12 semanas"), propuesta("cuatro fuentes")]);
        c.guardar_propuesta(c.en_espera()[0].id);
        c.cortar();
        c.cortar_las_propuestas();
        assert!(c.en_espera().is_empty(), "las que esperaban sobrevivieron al corte");
        assert_eq!(c.guardadas().len(), 1);
        assert!(!c.vacio(), "una propuesta guardada es tuya");
    }

    #[test]
    fn una_reunion_con_propuestas_sin_decidir_tiene_algo_que_decidir() {
        let mut c = Cuaderno::nuevo(false);
        assert!(!c.hay_que_decidir());
        c.proponer(vec![propuesta("12 semanas")]);
        assert!(c.vacio());
        assert!(c.hay_que_decidir());
        assert_eq!(c.sin_decidir().len(), 1);
        assert!(!c.hay_que_decidir());
    }

    #[test]
    fn las_reglas_ven_la_linea_de_la_fijada_y_el_archivo_no() {
        let mut c = Cuaderno::nuevo(false);
        c.ver(FichaFijada { linea: "una cuarta fuente es adicional".into(), ..ficha("Hasta tres fuentes") });
        c.fijar_la_vigente();
        assert_eq!(c.para_las_reglas()[0].linea, "una cuarta fuente es adicional");
        let json = String::from_utf8(c.contenido(encabezado()).a_bytes()).unwrap();
        assert!(!json.contains("cuarta"), "la línea del documento acabó en el archivo: {json}");
    }
}
