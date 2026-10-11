//! LAS PROPUESTAS — qué te propone guardar la app, por reglas publicadas (C9, ADR 016).
//!
//! **MÓDULO PROTEGIDO.** Lee cada turno de la reunión, también los del cliente, y por eso **no tiene
//! manera de escribir**: devuelve propuestas y nada más. Las guarda el cuaderno (`notas/`, también
//! protegido) y, al cerrar, las sella `bandeja.rs`, que ya solo ve líneas. `pnpm verify:ephemeral` lo
//! vigila.
//!
//! **Qué hace cumplir este archivo** (ADR 016 §1 y §2):
//! - las reglas son las del catálogo `data/propuestas/reglas.json`, y ninguna más;
//! - de tus turnos (micrófono **sin eco**), la frase en la que saltó la regla, como mucho 160 letras;
//! - del cliente (sistema, **o micrófono marcado como eco**, que es su voz), **jamás el turno**: un
//!   fragmento de como mucho 8 palabras, que la pantalla pone dentro de una plantilla («Dijeron
//!   «12 semanas»»). De una pregunta, ni eso: sus palabras clave;
//! - como mucho dos propuestas por turno, las de más peso primero.
//!
//! No hay modelo: son comparaciones de palabras, sin red y sin disco. Un turno largo se resuelve en
//! microsegundos, muy lejos del segundo que pide el kit.

pub mod catalogo;

use serde::{Deserialize, Serialize};

use crate::capture::Quien;
use crate::stt::Turno;

/// Tope de la frase tuya, en letras.
pub const TOPE_DE_LA_FRASE: usize = 160;
/// Tope del fragmento del cliente, en palabras (ADR 016 §2).
pub const TOPE_DEL_FRAGMENTO: usize = 8;
/// Tope de palabras clave de una pregunta del cliente.
pub const TOPE_DE_CLAVES: usize = 5;
/// Cuántas propuestas puede dar un solo turno.
pub const POR_TURNO: usize = 2;

/// Qué regla saltó. El orden es el peso: una contradicción con tu ficha pesa más que una cifra.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Regla {
    Choque,
    Compromiso,
    Cifra,
    Nombre,
    Pregunta,
}

impl Regla {
    pub const TODAS: [Regla; 5] = [Regla::Cifra, Regla::Compromiso, Regla::Choque, Regla::Nombre, Regla::Pregunta];

    pub fn id(self) -> &'static str {
        match self {
            Regla::Cifra => "cifra",
            Regla::Compromiso => "compromiso",
            Regla::Choque => "choque",
            Regla::Nombre => "nombre",
            Regla::Pregunta => "pregunta",
        }
    }
}

/// De quién salió, por la pista y el eco — nunca por la voz (regla dura 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum De {
    Tuyo,
    Cliente,
}

impl De {
    /// Un turno del micrófono marcado como eco es el cliente sonando por tus altavoces.
    ///
    /// **Y la sala no es de nadie** (ADR 020 §5): `None`. Hasta el sprint 005 esto era un `if` con
    /// `else De::Cliente`, y una tercera pista habría caído en el `else`: la sala entera propuesta como
    /// hechos «del cliente», incluido lo que dijiste tú.
    pub fn del_turno(turno: &Turno) -> Option<De> {
        match turno.pista.quien() {
            Quien::Tuyo if turno.eco => Some(De::Cliente),
            Quien::Tuyo => Some(De::Tuyo),
            Quien::Cliente => Some(De::Cliente),
            Quien::SinAtribuir => None,
        }
    }
}

/// Una propuesta. Lo que la pantalla enseña y, si dices que sí, lo que se guarda.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Propuesta {
    pub regla: Regla,
    pub de: De,
    /// Tuyo: la frase. Del cliente: el fragmento (≤ 8 palabras) o, de una pregunta, sus palabras
    /// clave separadas por « · ».
    pub texto: String,
    /// Solo `choque`: lo que dice tu ficha fijada («tres»).
    #[serde(default)]
    pub ficha: Option<String>,
    /// Solo `choque`: la sección de esa ficha («§3.2 Alcance»).
    #[serde(default)]
    pub seccion: Option<String>,
    /// «14:16», la hora del turno.
    pub hora: String,
}

/// Una ficha que fijaste, como la ven las reglas: su titular y su línea, que viven en memoria.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Fijada {
    pub titular: String,
    pub linea: String,
    pub seccion: Option<String>,
}

/// Lo que las reglas necesitan saber además del turno.
pub struct Contexto<'a> {
    pub fijadas: &'a [Fijada],
    /// ¿Está este nombre en tu corpus? Lo responde quien tiene el índice; aquí no hay disco.
    pub conoce: &'a dyn Fn(&str) -> bool,
}

/// De un turno, sus propuestas: como mucho [`POR_TURNO`], las de más peso primero.
pub fn proponer(turno: &Turno, ctx: &Contexto) -> Vec<Propuesta> {
    let texto = turno.texto.trim();
    if texto.is_empty() {
        return Vec::new();
    }
    // La sala no propone nada: no se sabe quién lo dijo.
    let Some(de) = De::del_turno(turno) else {
        return Vec::new();
    };
    let c = catalogo::catalogo();
    let fichas = palabras(texto);
    let mut todas: Vec<Propuesta> = Vec::new();
    let mut nueva = |regla: Regla, desde: usize, fragmento: String, ficha: Option<String>, seccion: Option<String>| {
        let texto = match de {
            De::Tuyo => frase(&fichas, desde),
            De::Cliente => fragmento,
        };
        todas.push(Propuesta { regla, de, texto, ficha, seccion, hora: turno.hora.clone() });
    };

    // choque: una cifra con su cosa que tu ficha dice con otro número.
    let dichas = pares(&fichas, c);
    'choques: for fijada in ctx.fijadas {
        let de_la_ficha = palabras(&format!("{} {}", fijada.titular, fijada.linea));
        for (valor_f, cosa_f, i_f) in pares(&de_la_ficha, c) {
            for &(valor, ref cosa, i) in &dichas {
                if *cosa == cosa_f && valor != valor_f {
                    nueva(
                        Regla::Choque,
                        i,
                        juntar(&fichas[i..=i + 1]),
                        Some(de_la_ficha[i_f].limpia.clone()),
                        fijada.seccion.clone(),
                    );
                    break 'choques;
                }
            }
        }
    }

    // compromiso: solo tuyos.
    if de == De::Tuyo {
        if let Some((i, n)) = buscar_frases(&fichas, &c.compromisos) {
            nueva(Regla::Compromiso, i, juntar(&fichas[i..i + n]), None, None);
        }
    }

    // cifra: número con unidad, moneda o fecha.
    if let Some((i, j)) = cifra(&fichas, c) {
        nueva(Regla::Cifra, i, juntar(&fichas[i..=j]), None, None);
    }

    // nombre: dos a cuatro palabras con mayúscula que tu corpus no conoce.
    if let Some((i, j)) = nombre(&fichas, c, ctx.conoce) {
        nueva(Regla::Nombre, i, juntar(&fichas[i..=j]), None, None);
    }

    // pregunta: solo del cliente, y solo sus palabras clave.
    if de == De::Cliente {
        if let Some((i, claves)) = pregunta(&fichas, c) {
            nueva(Regla::Pregunta, i, claves, None, None);
        }
    }

    todas.sort_by_key(|p| p.regla);
    let mut vistas: Vec<String> = Vec::new();
    todas.retain(|p| {
        let clave = plegar(&p.texto);
        if vistas.contains(&clave) {
            return false;
        }
        vistas.push(clave);
        true
    });
    for p in &mut todas {
        if p.de == De::Cliente && p.regla != Regla::Pregunta {
            p.texto = recortar_palabras(&p.texto, TOPE_DEL_FRAGMENTO);
        }
    }
    todas.truncate(POR_TURNO);
    todas
}

// ─── las palabras ────────────────────────────────────────────────────────────────────────────────

/// Una palabra del turno: como se dijo, limpia de signos, y plegada para comparar.
#[derive(Clone, Debug)]
struct Palabra {
    /// Tal cual, con sus signos («fuentes;»).
    cruda: String,
    /// Sin signos a los lados («fuentes»).
    limpia: String,
    /// Sin mayúsculas ni tildes («fuentes»).
    plegada: String,
    /// Empieza con mayúscula.
    mayuscula: bool,
    /// Termina una frase (. ? ! ;) o un inciso (,).
    corta: bool,
}

fn palabras(texto: &str) -> Vec<Palabra> {
    texto
        .split_whitespace()
        .filter_map(|cruda| {
            let limpia = cruda
                .trim_matches(|ch: char| !(ch.is_alphanumeric() || matches!(ch, '%' | '$' | '€' | '\'' | '’')))
                .trim_end_matches(['\'', '’'])
                .to_string();
            if limpia.is_empty() {
                return None;
            }
            Some(Palabra {
                mayuscula: limpia.chars().next().is_some_and(char::is_uppercase),
                plegada: plegar(&limpia),
                corta: cruda.ends_with(['.', '?', '!', ';', ',', ':']),
                cruda: cruda.to_string(),
                limpia,
            })
        })
        .collect()
}

/// Minúsculas, sin tildes y con el apóstrofo recto: como están escritas las del catálogo.
pub(crate) fn plegar(texto: &str) -> String {
    texto
        .chars()
        .flat_map(char::to_lowercase)
        .map(|ch| match ch {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            '’' => '\'',
            otro => otro,
        })
        .collect()
}

fn juntar(ps: &[Palabra]) -> String {
    ps.iter().map(|p| p.limpia.as_str()).collect::<Vec<_>>().join(" ")
}

/// La frase del turno que contiene la palabra `i`, tal como se dijo, como mucho [`TOPE_DE_LA_FRASE`].
fn frase(ps: &[Palabra], i: usize) -> String {
    let fin_de_frase = |p: &Palabra| p.cruda.ends_with(['.', '?', '!', ';']);
    let mut desde = i;
    while desde > 0 && !fin_de_frase(&ps[desde - 1]) {
        desde -= 1;
    }
    let mut hasta = i;
    while hasta + 1 < ps.len() && !fin_de_frase(&ps[hasta]) {
        hasta += 1;
    }
    let entera = ps[desde..=hasta].iter().map(|p| p.cruda.as_str()).collect::<Vec<_>>().join(" ");
    recortar_letras(&entera, TOPE_DE_LA_FRASE)
}

fn recortar_letras(texto: &str, tope: usize) -> String {
    if texto.chars().count() <= tope {
        return texto.to_string();
    }
    let mut dentro = String::new();
    for p in texto.split_whitespace() {
        if dentro.chars().count() + p.chars().count() + 2 > tope {
            break;
        }
        if !dentro.is_empty() {
            dentro.push(' ');
        }
        dentro.push_str(p);
    }
    dentro.push('…');
    dentro
}

fn recortar_palabras(texto: &str, tope: usize) -> String {
    texto.split_whitespace().take(tope).collect::<Vec<_>>().join(" ")
}

// ─── las reglas ──────────────────────────────────────────────────────────────────────────────────

/// El valor de una palabra que es un número: «12», «40.000», «1,5», «30%», «40k» o «cuatro».
fn numero(p: &Palabra, c: &catalogo::Catalogo) -> Option<f64> {
    if let Some(&n) = c.numeros.get(&p.plegada) {
        return Some(n);
    }
    let cifras = p.plegada.trim_end_matches(['%', 'k']).trim_start_matches(['$', '€']);
    if cifras.is_empty() || !cifras.chars().next().is_some_and(|ch| ch.is_ascii_digit()) {
        return None;
    }
    if !cifras.chars().all(|ch| ch.is_ascii_digit() || ch == '.' || ch == ',') {
        return None;
    }
    // «40.000» y «40,000» son miles; «1,5» y «1.5», decimales.
    let miles = cifras.split(['.', ',']).skip(1).all(|t| t.len() == 3);
    let normal = if miles { cifras.replace(['.', ','], "") } else { cifras.replace(',', ".") };
    normal.parse().ok()
}

/// La cosa que se cuenta, sin plural, para comparar «tres fuentes» con «cuatro fuentes».
fn raiz(plegada: &str) -> String {
    let n = plegada.chars().count();
    if n > 4 && plegada.ends_with("es") {
        plegada[..plegada.len() - 2].to_string()
    } else if (n > 3 && plegada.ends_with('s')) || (n > 4 && plegada.ends_with('e')) {
        plegada[..plegada.len() - 1].to_string()
    } else {
        plegada.to_string()
    }
}

/// Los pares «cifra · cosa» de un texto: (valor, raíz de la cosa, posición de la cifra).
fn pares(ps: &[Palabra], c: &catalogo::Catalogo) -> Vec<(f64, String, usize)> {
    let mut hay = Vec::new();
    for i in 0..ps.len().saturating_sub(1) {
        let Some(valor) = numero(&ps[i], c) else { continue };
        if ps[i].corta {
            continue;
        }
        let cosa = &ps[i + 1];
        let es_palabra = cosa.plegada.chars().count() >= 3 && cosa.plegada.chars().all(char::is_alphabetic);
        if es_palabra && !c.vacias.contains(&cosa.plegada) {
            hay.push((valor, raiz(&cosa.plegada), i));
        }
    }
    hay
}

/// Busca la primera de unas frases del catálogo, por palabras enteras: (desde, cuántas palabras).
fn buscar_frases(ps: &[Palabra], frases: &[Vec<String>]) -> Option<(usize, usize)> {
    for i in 0..ps.len() {
        for f in frases {
            if i + f.len() <= ps.len() && ps[i..i + f.len()].iter().zip(f).all(|(p, w)| p.plegada == *w) {
                return Some((i, f.len()));
            }
        }
    }
    None
}

/// Una cifra con su unidad, su moneda o su fecha: (desde, hasta), inclusive.
fn cifra(ps: &[Palabra], c: &catalogo::Catalogo) -> Option<(usize, usize)> {
    for i in 0..ps.len() {
        if numero(&ps[i], c).is_some() {
            let p = &ps[i].plegada;
            if p.ends_with('%') || p.ends_with('k') && p.len() > 1 || p.starts_with(['$', '€']) {
                return Some((i, i));
            }
            // la moneda delante: «USD 40.000», «US$ 40.000»
            if i > 0 && c.monedas.contains(&ps[i - 1].plegada) && !ps[i - 1].corta {
                let hasta = if i + 1 < ps.len() && c.es_unidad(&ps[i + 1].plegada) && !ps[i].corta { i + 1 } else { i };
                return Some((i - 1, hasta));
            }
            if !ps[i].corta {
                // la unidad detrás, de una o dos palabras: «12 semanas», «30 por ciento»
                if let Some(n) = c.unidad_en(&ps[i + 1..]) {
                    return Some((i, i + n));
                }
                // el día del mes: «15 de octubre»
                if i + 2 < ps.len() && ps[i + 1].plegada == "de" && c.meses.contains(&ps[i + 2].plegada) {
                    return Some((i, i + 2));
                }
            }
            // el mes delante: «October 15»
            if i > 0 && c.meses.contains(&ps[i - 1].plegada) && !ps[i - 1].corta {
                return Some((i - 1, i));
            }
        }
    }
    // una fecha dicha con palabras: «el viernes», «la próxima semana», «tomorrow»
    let (i, n) = buscar_frases(ps, &c.fechas)?;
    let antes = ["el", "este", "esta", "next", "this", "by", "on"];
    let desde = if i > 0 && antes.contains(&ps[i - 1].plegada.as_str()) && !ps[i - 1].corta { i - 1 } else { i };
    Some((desde, i + n - 1))
}

/// **Las cifras de un texto, con lo que cuentan** (sprint 004, el banco del ensayo, ADR 019 §2):
/// «tres fuentes», «cuatro semanas», «USD 40.000», «30 %», en el orden en que aparecen y sin repetir.
/// Son las mismas reglas que la regla `cifra` de las propuestas —un número con su unidad, su moneda o
/// su fecha— más el número con la cosa que cuenta detrás («dos rondas»). En inglés el modificador va
/// delante del nombre («two review rounds»): si a la cosa le sigue un plural, entra también.
pub fn cifras(texto: &str, ingles: bool) -> Vec<String> {
    let c = catalogo::catalogo();
    let ps = palabras(texto);
    let mut tramos: Vec<(usize, usize)> = Vec::new();
    let mut i = 0;
    while i < ps.len() {
        let Some((d, h)) = cifra(&ps[i..], c) else { break };
        tramos.push((i + d, i + h));
        i += h + 1;
    }
    for (_, _, pos) in pares(&ps, c) {
        let mut hasta = pos + 1;
        if ingles && !ps[hasta].corta && hasta + 1 < ps.len() {
            let sig = &ps[hasta + 1].plegada;
            let plural = sig.chars().count() > 3 && sig.ends_with('s') && sig.chars().all(char::is_alphabetic);
            if plural && !c.vacias.contains(sig) {
                hasta += 1;
            }
        }
        tramos.push((pos, hasta));
    }
    // Por orden de aparición; si dos empiezan en el mismo sitio, la más larga; y sin solapes.
    tramos.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)));
    let mut salida: Vec<String> = Vec::new();
    let mut fin: Option<usize> = None;
    for (desde, hasta) in tramos {
        if fin.is_some_and(|f| desde <= f) {
            continue;
        }
        let dicha = juntar(&ps[desde..=hasta]);
        if !salida.iter().any(|y| plegar(y) == plegar(&dicha)) {
            salida.push(dicha);
        }
        fin = Some(hasta);
    }
    salida
}

/// Dos a cuatro palabras seguidas con mayúscula que tu corpus no conoce: (desde, hasta).
fn nombre(ps: &[Palabra], c: &catalogo::Catalogo, conoce: &dyn Fn(&str) -> bool) -> Option<(usize, usize)> {
    let vale = |p: &Palabra| {
        p.mayuscula
            && p.limpia.chars().count() >= 2
            && p.limpia.chars().all(char::is_alphabetic)
            && !c.vacias.contains(&p.plegada)
            && !c.meses.contains(&p.plegada)
            && !c.es_fecha(&p.plegada)
            && !c.es_unidad(&p.plegada)
    };
    let mut i = 0;
    while i < ps.len() {
        if !vale(&ps[i]) {
            i += 1;
            continue;
        }
        let mut j = i;
        while j + 1 < ps.len() && !ps[j].corta && vale(&ps[j + 1]) {
            j += 1;
        }
        let largo = j - i + 1;
        if (2..=4).contains(&largo) && !conoce(&juntar(&ps[i..=j])) {
            return Some((i, j));
        }
        i = j + 1;
    }
    None
}

/// Una pregunta del cliente: (dónde empieza, sus palabras clave con « · »). Nunca la pregunta.
fn pregunta(ps: &[Palabra], c: &catalogo::Catalogo) -> Option<(usize, String)> {
    let mut desde = 0;
    for hasta in 0..ps.len() {
        let fin = ps[hasta].cruda.ends_with(['.', '?', '!', ';']) || hasta + 1 == ps.len();
        if !fin {
            continue;
        }
        let frase = &ps[desde..=hasta];
        let es_pregunta = ps[hasta].cruda.ends_with('?')
            || frase.iter().any(|p| p.cruda.contains('¿'))
            || frase
                .iter()
                .find(|p| !["y", "pero", "entonces", "and", "so", "but"].contains(&p.plegada.as_str()))
                .is_some_and(|p| c.interrogativos.contains(&p.plegada));
        if es_pregunta {
            let mut claves: Vec<String> = Vec::new();
            for p in frase {
                let clave = p.limpia.to_lowercase();
                let sirve = p.plegada.chars().count() >= 4
                    && p.plegada.chars().all(char::is_alphanumeric)
                    && !c.vacias.contains(&p.plegada)
                    && !c.interrogativos.contains(&p.plegada);
                if sirve && !claves.contains(&clave) {
                    claves.push(clave);
                }
                if claves.len() == TOPE_DE_CLAVES {
                    break;
                }
            }
            if !claves.is_empty() {
                return Some((desde, claves.join(" · ")));
            }
        }
        desde = hasta + 1;
    }
    None
}

#[cfg(test)]
mod pruebas {
    use crate::capture::Pista;

    /// **Las cifras para el ensayo** (sprint 004): con lo que cuentan, en orden y sin repetir, en los
    /// dos idiomas. Y en inglés, el modificador delante del plural.
    #[test]
    fn las_cifras_de_un_texto_con_lo_que_cuentan() {
        let es = "Tarifa cerrada de cuatro semanas. Incluye el taller de cierre y dos rondas de revisión. La entrega toma cuatro semanas.";
        assert_eq!(cifras(es, false), vec!["cuatro semanas", "dos rondas"]);
        assert_eq!(cifras("Cubre limpieza de tres fuentes: el ERP y el POS.", false), vec!["tres fuentes"]);
        assert_eq!(cifras("Un taller de tres horas con USD 40.000 de tope y un 30% de margen.", false), vec!["tres horas", "USD 40.000", "30%"]);
        let en = "Fixed fee for three weeks, including two review rounds. Covers two sources: the ERP.";
        assert_eq!(cifras(en, true), vec!["three weeks", "two review rounds", "two sources"]);
        assert!(cifras("Sin números aquí.", false).is_empty());
    }

    use super::*;

    fn turno(pista: Pista, eco: bool, texto: &str) -> Turno {
        Turno { pista, desde_ms: 0, hasta_ms: 1000, texto: texto.into(), hora: "14:16".into(), eco }
    }
    fn tuyo(texto: &str) -> Turno {
        turno(Pista::Microfono, false, texto)
    }
    fn cliente(texto: &str) -> Turno {
        turno(Pista::Sistema, false, texto)
    }
    fn nadie_conocido(_: &str) -> bool {
        false
    }
    fn todo_conocido(_: &str) -> bool {
        true
    }
    fn sin_fichas() -> Contexto<'static> {
        Contexto { fijadas: &[], conoce: &todo_conocido }
    }

    #[test]
    fn de_tus_turnos_la_frase_en_la_que_salto_la_regla() {
        let p = proponer(&tuyo("Bueno, vamos bien. La fecha real del tablero es de 12 semanas desde la firma. Lo demás igual."), &sin_fichas());
        assert_eq!(p.len(), 1, "{p:?}");
        assert_eq!(p[0].regla, Regla::Cifra);
        assert_eq!(p[0].de, De::Tuyo);
        assert_eq!(p[0].texto, "La fecha real del tablero es de 12 semanas desde la firma.");
    }

    #[test]
    fn un_compromiso_tuyo_se_propone_y_el_mismo_del_cliente_no() {
        let mio = proponer(&tuyo("Perfecto, te lo mando el lunes con la cotización."), &sin_fichas());
        assert_eq!(mio[0].regla, Regla::Compromiso, "{mio:?}");
        let suyo = proponer(&cliente("Perfecto, te lo mando cuando lo tenga listo."), &sin_fichas());
        assert!(suyo.iter().all(|p| p.regla != Regla::Compromiso), "{suyo:?}");
    }

    /// **El gate de la regla dura 1 en las propuestas.** Del cliente, jamás el turno: un fragmento de
    /// ≤ 8 palabras. Demostrado en rojo: con `De::Cliente => frase(&fichas, desde)` en `nueva`, este
    /// test falla, porque el texto del cliente pasa a ser su frase entera.
    #[test]
    fn del_cliente_jamas_el_turno_solo_un_fragmento_de_ocho_palabras() {
        let turnos = [
            "Nosotros necesitamos que el tablero esté listo en 12 semanas porque el comité se reúne en diciembre.",
            "Mira, el año pasado Andrea Villalba nos dijo que eso costaba USD 40.000 y nos pareció mucho.",
            "La verdad es que queremos incluir cuatro fuentes y no solo las tres que dice tu propuesta.",
            "Oye, ¿y la limpieza de datos de todas las tiendas de vereda eso está dentro del alcance o no?",
        ];
        let fijadas = [Fijada { titular: "Limpieza de datos: incluida, hasta tres fuentes".into(), linea: String::new(), seccion: Some("§3.2".into()) }];
        let ctx = Contexto { fijadas: &fijadas, conoce: &nadie_conocido };
        let mut vistas = 0;
        for t in turnos {
            for p in proponer(&cliente(t), &ctx) {
                vistas += 1;
                assert_eq!(p.de, De::Cliente);
                let n = p.texto.split_whitespace().filter(|w| *w != "·").count();
                assert!(n <= TOPE_DEL_FRAGMENTO, "más de 8 palabras: {p:?}");
                assert!(p.texto.chars().count() < t.chars().count() / 2, "demasiado del turno: {p:?}");
                assert!(!p.texto.contains(t.trim_end_matches(['.', '?'])), "el turno entero: {p:?}");
            }
        }
        assert!(vistas >= 4, "las reglas no saltaron: {vistas}");
    }

    /// **La sala no propone nada** (ADR 020 §5): ni como tuyo ni como del cliente, porque no se sabe
    /// quién lo dijo. La misma frase, del cliente, sí propone.
    ///
    /// ¿Puede fallar? Sí: con `Quien::SinAtribuir => Some(De::Cliente)` lo que dijiste tú en la mesa se
    /// propondría como un hecho del cliente (bitácora del sprint 005, fase 1).
    #[test]
    fn la_sala_no_propone_nada() {
        let frase = "Necesitamos que el tablero esté listo en 12 semanas porque el comité se reúne en diciembre.";
        assert!(!proponer(&cliente(frase), &sin_fichas()).is_empty(), "la frase de control ya no propone: el test no mide nada");
        assert_eq!(proponer(&turno(Pista::Sala, false, frase), &sin_fichas()), Vec::new(), "la sala propuso una nota");
        assert_eq!(De::del_turno(&turno(Pista::Sala, false, frase)), None);
        assert_eq!(De::del_turno(&turno(Pista::Sala, true, frase)), None, "el eco no le da dueño a la sala");
    }

    /// **El eco es del cliente.** Un turno del micrófono marcado como eco es su voz por tus altavoces:
    /// ni compromiso (que es solo tuyo) ni su frase. Demostrado en rojo: con `De::del_turno` sin mirar
    /// `turno.eco`, el compromiso sale como tuyo y con la frase entera.
    #[test]
    fn el_eco_es_la_voz_del_cliente() {
        let eco = turno(Pista::Microfono, true, "Sí, te lo mando mañana con las cifras de las 12 semanas del proyecto.");
        let p = proponer(&eco, &sin_fichas());
        assert!(!p.is_empty());
        assert!(p.iter().all(|p| p.de == De::Cliente && p.regla != Regla::Compromiso), "{p:?}");
        assert!(p.iter().all(|p| p.texto.split_whitespace().filter(|w| *w != "·").count() <= TOPE_DEL_FRAGMENTO), "{p:?}");
    }

    #[test]
    fn un_choque_con_tu_ficha_fijada_dice_lo_que_dice_la_ficha() {
        let fijadas = [Fijada {
            titular: "Limpieza de datos: incluida, hasta tres fuentes".into(),
            linea: "Cubre ERP, POS y Excel de canal; una cuarta fuente es adicional.".into(),
            seccion: Some("§3.2 Alcance".into()),
        }];
        let ctx = Contexto { fijadas: &fijadas, conoce: &todo_conocido };
        let p = proponer(&cliente("Nosotros vamos a necesitar cuatro fuentes como mínimo para el piloto."), &ctx);
        assert_eq!(p[0].regla, Regla::Choque, "{p:?}");
        assert_eq!(p[0].texto, "cuatro fuentes");
        assert_eq!(p[0].ficha.as_deref(), Some("tres"));
        assert_eq!(p[0].seccion.as_deref(), Some("§3.2 Alcance"));
        // la misma cifra no choca
        let igual = proponer(&cliente("Con tres fuentes nos alcanza por ahora."), &ctx);
        assert!(igual.iter().all(|p| p.regla != Regla::Choque), "{igual:?}");
    }

    #[test]
    fn un_nombre_que_tu_corpus_no_conoce_si_y_uno_que_conoce_no() {
        let t = cliente("Eso lo tiene que aprobar Andrea Villalba antes del comité.");
        let nuevo = proponer(&t, &Contexto { fijadas: &[], conoce: &nadie_conocido });
        assert!(nuevo.iter().any(|p| p.regla == Regla::Nombre && p.texto == "Andrea Villalba"), "{nuevo:?}");
        let conocido = proponer(&t, &Contexto { fijadas: &[], conoce: &todo_conocido });
        assert!(conocido.iter().all(|p| p.regla != Regla::Nombre), "{conocido:?}");
    }

    #[test]
    fn de_una_pregunta_del_cliente_solo_sus_palabras_clave() {
        let t = "¿Y la limpieza de datos, eso está dentro del alcance?";
        let p = proponer(&cliente(t), &sin_fichas());
        let q = p.iter().find(|p| p.regla == Regla::Pregunta).expect("la pregunta");
        assert_eq!(q.texto, "limpieza · datos · dentro · alcance");
        // las tuyas no se proponen: la pregunta que haces tú no es un hecho que guardar
        assert!(proponer(&tuyo(t), &sin_fichas()).iter().all(|p| p.regla != Regla::Pregunta));
    }

    #[test]
    fn cifras_fechas_y_monedas_en_los_dos_idiomas() {
        let casos = [
            ("Eso nos lleva unas 12 semanas.", "12 semanas"),
            ("El margen de canal es del 30%.", "30%"),
            ("El presupuesto es de USD 40.000 en total.", "USD 40.000"),
            ("Lo necesitamos para el 15 de octubre.", "15 de octubre"),
            ("Podemos verlo el viernes con el equipo.", "el viernes"),
            ("We need it within six weeks.", "six weeks"),
            ("Let's review it next week with finance.", "next week"),
            ("The kickoff is on October 15 at the latest.", "October 15"),
        ];
        for (dicho, esperado) in casos {
            let p = proponer(&cliente(dicho), &sin_fichas());
            assert!(p.iter().any(|p| p.regla == Regla::Cifra && p.texto == esperado), "«{dicho}» → {p:?}");
        }
    }

    #[test]
    fn como_mucho_dos_por_turno_y_sin_repetir() {
        let fijadas = [Fijada { titular: "Hasta tres fuentes".into(), ..Default::default() }];
        let ctx = Contexto { fijadas: &fijadas, conoce: &nadie_conocido };
        let p = proponer(&cliente("¿Cuatro fuentes en 12 semanas con Andrea Villalba el viernes?"), &ctx);
        assert_eq!(p.len(), POR_TURNO, "{p:?}");
        assert_eq!(p[0].regla, Regla::Choque);
        let tuyo = proponer(&tuyo("Te lo mando en 12 semanas."), &sin_fichas());
        assert_eq!(tuyo.len(), 1, "la misma frase por dos reglas es una propuesta: {tuyo:?}");
    }

    #[test]
    fn un_turno_sin_nada_no_propone_nada() {
        assert!(proponer(&cliente("Sí, claro, tiene sentido lo que dices."), &sin_fichas()).is_empty());
        assert!(proponer(&tuyo(""), &sin_fichas()).is_empty());
    }

    /// El kit pide la propuesta en ≤ 1 s tras el turno. Un turno de mil palabras con fichas fijadas
    /// tarda microsegundos; el umbral de 50 ms deja el segundo muy lejos incluso en el runner de CI.
    #[test]
    fn un_turno_largo_se_resuelve_muy_por_debajo_del_segundo() {
        let largo = "Necesitamos cuatro fuentes y 12 semanas para el viernes con Andrea Villalba. ".repeat(80);
        let fijadas = vec![Fijada { titular: "Hasta tres fuentes en 11 semanas".into(), linea: "Una cuarta fuente es adicional.".into(), seccion: None }; 20];
        let ctx = Contexto { fijadas: &fijadas, conoce: &nadie_conocido };
        let t = cliente(&largo);
        let antes = std::time::Instant::now();
        let p = proponer(&t, &ctx);
        let ms = antes.elapsed().as_millis();
        assert!(!p.is_empty());
        assert!(ms < 50, "{ms} ms");
    }
}
