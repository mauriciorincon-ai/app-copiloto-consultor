//! LO QUE SE SACA DE UNA PANTALLA LEÍDA — cifras, títulos y tus términos.
//!
//! Vision devuelve **todo** el texto de la ventana de la reunión: la diapositiva que comparte el
//! cliente, pero también los nombres de los participantes, el reloj, «Presentar ahora» y el chat.
//! Mandar todo eso al buscador sería preguntarle por la interfaz de Meet. Aquí se queda lo que
//! habla del **tema**, con tres reglas que se pueden explicar en una frase cada una:
//!
//! 1. **Títulos**: las líneas claramente más altas que el resto. En una diapositiva, el título es
//!    lo que el cliente quiere que se lea; en la interfaz de Meet no hay texto grande.
//! 2. **Cifras**: las líneas cortas con un número dentro —«Margen por canal: 23 %»—, que es lo que
//!    la orden del sprint pide leer. Las horas («14:03») no cuentan: son el reloj de la llamada.
//! 3. **Tus términos**: las palabras distintivas de TU corpus que aparezcan en cualquier línea
//!    —el nombre del cliente, el de tu método—. Es la misma lista que usa el disparador.
//!
//! **Todo esto es texto de un tercero** (regla 1 de la casa: «lo leído de la pantalla muere
//! siempre»). Vive en memoria, jamás se escribe en un log, y [`Refuerzo::olvidar`] pisa las letras
//! antes de soltarlas —el mismo trato que la ventana de turnos—.

use crate::disparo::normalizar;

/// Una línea de texto tal y como la devuelve Vision.
#[derive(Debug, Clone, PartialEq)]
pub struct LineaLeida {
    pub texto: String,
    /// 0..1 — cuánto se fía Vision de lo que leyó.
    pub confianza: f32,
    /// 0..1 — el alto de la línea respecto al alto del cuadro. Es lo que distingue un título.
    pub alto: f32,
}

/// Por debajo de esto, Vision está adivinando y lo dice. No se le hace caso.
pub const CONFIANZA_MINIMA: f32 = 0.5;

/// Una línea es título si es al menos esto más alta que la mediana de las líneas del cuadro…
const TITULO_SOBRE_LA_MEDIANA: f32 = 1.4;

/// …y además está cerca de la más alta. Sin esta segunda vara, en una ventana de Meet —donde la
/// interfaz pone muchas líneas pequeñas y baja la mediana— todo el cuerpo de la diapositiva pasaba
/// por título. Lo encontró su propio test.
const TITULO_BAJO_EL_MAS_ALTO: f32 = 0.75;

/// Techos: una pantalla con cuarenta cifras no aporta cuarenta pistas, aporta ruido.
const TITULOS: usize = 3;
const CIFRAS: usize = 5;
const TERMINOS: usize = 8;

/// Una cifra va con su contexto, no sola: «23» no busca nada, «Margen por canal: 23 %» sí. Pero una
/// línea larga es un párrafo, no una cifra.
const PALABRAS_DE_UNA_CIFRA: usize = 10;

/// Lo que la pantalla aporta a la búsqueda.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Refuerzo {
    pub titulos: Vec<String>,
    pub cifras: Vec<String>,
    pub terminos: Vec<String>,
}

impl Refuerzo {
    /// Todo junto, como texto de consulta. El buscador lo limpia igual que un turno hablado.
    pub fn consulta(&self) -> String {
        self.titulos
            .iter()
            .chain(&self.cifras)
            .chain(&self.terminos)
            .cloned()
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn vacio(&self) -> bool {
        self.titulos.is_empty() && self.cifras.is_empty() && self.terminos.is_empty()
    }

    /// **¿Esta pantalla pide ficha por sí sola?** Solo si trae una cifra o uno de tus términos: un
    /// título suelto —«Agenda», «Gracias»— no es una pregunta para nadie. Es la misma vara que el
    /// disparador aplica a un turno sin signo de interrogación.
    pub fn dispara(&self) -> bool {
        !self.cifras.is_empty() || !self.terminos.is_empty()
    }

    /// Cuántos bytes de texto de terceros viven aquí ahora mismo. Lo cuenta la pantalla de
    /// Honestidad junto al cuadro.
    pub fn bytes(&self) -> usize {
        self.titulos
            .iter()
            .chain(&self.cifras)
            .chain(&self.terminos)
            .map(String::len)
            .sum()
    }

    /// Olvida lo que leyó, **pisando las letras antes de soltarlas**.
    pub fn olvidar(&mut self) {
        for s in self
            .titulos
            .iter_mut()
            .chain(self.cifras.iter_mut())
            .chain(self.terminos.iter_mut())
        {
            // SEGURIDAD: se escriben ceros sobre bytes que ya eran UTF-8 válido; el cero también lo
            // es, así que la cadena sigue siendo válida en todo momento.
            unsafe { s.as_mut_vec() }.fill(0);
        }
        self.titulos.clear();
        self.cifras.clear();
        self.terminos.clear();
    }
}

/// De las líneas leídas al refuerzo. `vocabulario` son las palabras distintivas del corpus, ya
/// normalizadas (las mismas que usa el disparador).
pub fn extraer(lineas: &[LineaLeida], vocabulario: &[String]) -> Refuerzo {
    let fiables: Vec<&LineaLeida> = lineas
        .iter()
        .filter(|l| l.confianza >= CONFIANZA_MINIMA && !l.texto.trim().is_empty())
        .collect();
    let mut r = Refuerzo::default();
    if fiables.is_empty() {
        return r;
    }

    let mut altos: Vec<f32> = fiables.iter().map(|l| l.alto).collect();
    altos.sort_by(|a, b| a.total_cmp(b));
    // La mediana BAJA: con dos líneas, la alta no puede ser su propia vara.
    let mediana = altos[(altos.len() - 1) / 2];
    let vara =
        (mediana * TITULO_SOBRE_LA_MEDIANA).max(altos[altos.len() - 1] * TITULO_BAJO_EL_MAS_ALTO);

    // Los títulos, de más alto a más bajo.
    let mut candidatos: Vec<&&LineaLeida> = fiables.iter().filter(|l| l.alto >= vara).collect();
    candidatos.sort_by(|a, b| b.alto.total_cmp(&a.alto));
    for l in candidatos.into_iter().take(TITULOS) {
        empujar(&mut r.titulos, l.texto.trim());
    }

    for l in &fiables {
        if r.cifras.len() >= CIFRAS {
            break;
        }
        let palabras = l.texto.split_whitespace().count();
        if palabras <= PALABRAS_DE_UNA_CIFRA
            && tiene_cifra(&l.texto)
            && !r.titulos.contains(&l.texto.trim().to_string())
        {
            empujar(&mut r.cifras, l.texto.trim());
        }
    }

    if !vocabulario.is_empty() {
        for l in &fiables {
            for p in normalizar(&l.texto).split_whitespace() {
                if r.terminos.len() >= TERMINOS {
                    break;
                }
                if vocabulario.iter().any(|v| v == p) {
                    empujar(&mut r.terminos, p);
                }
            }
        }
    }
    r
}

fn empujar(lista: &mut Vec<String>, texto: &str) {
    if !lista.iter().any(|t| t == texto) {
        lista.push(texto.to_string());
    }
}

/// ¿Trae un número que no sea la hora? «14:03» y «9:30» son el reloj de la llamada o de la agenda;
/// «23 %», «9 semanas», «ISO 27001» y «$4.200» son cifras.
fn tiene_cifra(texto: &str) -> bool {
    texto.split_whitespace().any(|palabra| {
        let p = palabra.trim_matches(|c: char| !c.is_alphanumeric() && c != ':');
        p.chars().any(|c| c.is_ascii_digit()) && !es_hora(p)
    })
}

fn es_hora(p: &str) -> bool {
    let partes: Vec<&str> = p.split(':').collect();
    partes.len() == 2
        && (1..=2).contains(&partes[0].len())
        && partes[1].len() == 2
        && partes.iter().all(|s| s.chars().all(|c| c.is_ascii_digit()))
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn l(texto: &str, alto: f32) -> LineaLeida {
        LineaLeida {
            texto: texto.into(),
            confianza: 0.95,
            alto,
        }
    }

    /// Una ventana de Meet con una diapositiva compartida: la interfaz en letra pequeña y la
    /// diapositiva con su título grande.
    fn meet_con_diapositiva() -> Vec<LineaLeida> {
        vec![
            l("Rentabilidad por canal", 0.07),
            l("Margen por canal: 23 %", 0.035),
            l("Tienda propia · distribuidores · marketplace", 0.03),
            l("Laura Méndez", 0.018),
            l("14:03", 0.018),
            l("Presentar ahora", 0.018),
            l("abc-defg-hij", 0.018),
        ]
    }

    #[test]
    fn el_titulo_de_la_diapositiva_es_titulo_y_la_interfaz_no() {
        let r = extraer(&meet_con_diapositiva(), &[]);
        assert_eq!(r.titulos, vec!["Rentabilidad por canal"]);
        assert_eq!(
            r.cifras,
            vec!["Margen por canal: 23 %"],
            "el cuerpo con cifra es cifra, no título"
        );
        assert!(!r.titulos.iter().any(|t| t.contains("Presentar")));
    }

    #[test]
    fn la_hora_de_la_llamada_no_es_una_cifra() {
        let r = extraer(
            &[
                l("14:03", 0.02),
                l("9:30", 0.02),
                l("Plazo: 9 semanas", 0.02),
            ],
            &[],
        );
        assert_eq!(r.cifras, vec!["Plazo: 9 semanas"]);
    }

    #[test]
    fn un_parrafo_con_un_numero_no_es_una_cifra() {
        let largo = "En la segunda etapa del proyecto se integraron 4 fuentes de datos con el equipo del cliente";
        let r = extraer(&[l(largo, 0.02)], &[]);
        assert!(r.cifras.is_empty());
    }

    #[test]
    fn tus_terminos_se_reconocen_sin_tildes_ni_mayusculas() {
        let vocabulario = vec!["paramo".to_string(), "rentabilidad".to_string()];
        let r = extraer(
            &[l("PÁRAMO AZUL — revisión trimestral", 0.02)],
            &vocabulario,
        );
        assert_eq!(r.terminos, vec!["paramo"]);
        assert!(r.dispara());
    }

    #[test]
    fn lo_que_vision_adivina_no_cuenta() {
        let r = extraer(
            &[LineaLeida {
                texto: "Margen 23 %".into(),
                confianza: 0.2,
                alto: 0.05,
            }],
            &[],
        );
        assert!(r.vacio());
    }

    #[test]
    fn una_pantalla_sin_cifras_ni_terminos_no_dispara() {
        let r = extraer(
            &[l("Agenda", 0.08), l("Gracias por venir", 0.03)],
            &["paramo".into()],
        );
        assert!(
            !r.titulos.is_empty(),
            "el título se lee igual: refuerza la búsqueda"
        );
        assert!(!r.dispara(), "pero una agenda no le pregunta nada a nadie");
    }

    #[test]
    fn olvidar_pisa_las_letras() {
        let mut r = extraer(&meet_con_diapositiva(), &["rentabilidad".into()]);
        assert!(r.bytes() > 0);
        r.olvidar();
        assert!(r.vacio());
        assert_eq!(r.bytes(), 0);
    }
}
