//! **El eco**: cuando el micrófono oye al cliente por los altavoces y la app lo toma por el
//! consultor.
//!
//! No estaba en el plan. Apareció al correr la fase entera de punta a punta por primera vez: sonó
//! una pregunta del cliente por los altavoces del portátil, el tap del sistema la capturó —bien— y
//! el micrófono **también**, así que el mismo turno salió por las dos pistas. La app promete que
//! el micrófono es el consultor y el audio del sistema es el cliente, y con altavoces esa promesa
//! es falsa: le atribuye al consultor palabras que no dijo. En una herramienta cuyo trabajo es
//! recordarle a alguien lo que dijo, eso no es un detalle.
//!
//! La respuesta canónica de la industria es un cancelador de eco acústico, que es un filtro
//! adaptativo caro de escribir y de justificar. La respuesta de esta casa es la barata que además
//! se puede leer: **si lo que dice el micrófono se parece mucho a lo que el sistema dijo a la vez,
//! es eco**. Dos condiciones, las dos necesarias:
//!
//! - **se solapan en el tiempo** — al menos la mitad del turno del micrófono cae dentro de uno del
//!   sistema. Sola no basta: interrumpir a alguien también solapa;
//! - **dicen casi lo mismo** — al menos el 60 % de los trozos de tres letras del micrófono
//!   aparecen en el del sistema. Sola tampoco basta: repetir lo que acaba de decir el cliente es
//!   una cosa normal en una conversación, y pasa desplazada en el tiempo, no encima.
//!
//! Se comparan trozos de tres letras y no palabras enteras por una razón que se vio en los datos
//! reales: el transcriptor escribió «certificaciones» en una pista y «certificación» en la otra, y
//! un comparador de palabras exactas habría dicho que no se parecen en nada.
//!
//! **Y lo que hace con el eco es marcarlo, no borrarlo.** Un turno que desaparece sin explicación
//! es la clase de silencio que esta app no se permite. La banda sabrá que ese turno es un reflejo
//! y no lo atribuirá a nadie; la pantalla de Sesión, mientras tanto, dice lo único que de verdad
//! resuelve el problema: ponte los auriculares.

use std::collections::HashSet;

/// Cuánto del turno del micrófono tiene que caer dentro de uno del sistema.
const SOLAPE_MINIMO: f32 = 0.5;

/// Cuánto se tienen que parecer los textos.
const PARECIDO_MINIMO: f32 = 0.6;

/// Un turno visto desde aquí: cuándo fue y qué dijo. Sin pista y sin quién, que aquí no hacen
/// falta — este módulo no sabe de hablantes, solo de coincidencias.
#[derive(Clone, Copy, Debug)]
pub struct Tramo<'a> {
    pub desde_ms: usize,
    pub hasta_ms: usize,
    pub texto: &'a str,
}

/// ¿Es `mio` un reflejo de alguno de los `ajenos`?
pub fn es_eco(mio: &Tramo, ajenos: &[Tramo]) -> bool {
    ajenos.iter().any(|otro| {
        solape(mio, otro) >= SOLAPE_MINIMO && parecido(mio.texto, otro.texto) >= PARECIDO_MINIMO
    })
}

/// Qué fracción del tramo propio cae dentro del ajeno.
fn solape(mio: &Tramo, otro: &Tramo) -> f32 {
    let duracion = mio.hasta_ms.saturating_sub(mio.desde_ms);
    if duracion == 0 {
        return 0.0;
    }
    let inicio = mio.desde_ms.max(otro.desde_ms);
    let fin = mio.hasta_ms.min(otro.hasta_ms);
    fin.saturating_sub(inicio) as f32 / duracion as f32
}

/// Qué fracción de los trozos de tres letras del primero aparece en el segundo.
///
/// No es simétrico a propósito: la pregunta es «¿está lo que dijo el micrófono dentro de lo que
/// dijo el sistema?», y el turno del sistema suele ser más largo porque el micrófono solo capta
/// parte del eco.
pub fn parecido(mio: &str, otro: &str) -> f32 {
    let mios = trozos(mio);
    if mios.is_empty() {
        return 0.0;
    }
    let ajenos = trozos(otro);
    let comunes = mios.intersection(&ajenos).count();
    comunes as f32 / mios.len() as f32
}

fn trozos(texto: &str) -> HashSet<[char; 3]> {
    let limpio: Vec<char> = normalizar(texto).chars().collect();
    limpio.windows(3).map(|v| [v[0], v[1], v[2]]).collect()
}

/// Minúsculas, sin tildes, sin signos, con un solo espacio entre palabras.
///
/// Las tildes se quitan porque el motor no siempre las pone igual en las dos pistas cuando el eco
/// llega más flojo; los signos, porque una pista acaba en interrogación y la otra en punto — se
/// vio literalmente así en la primera prueba de punta a punta.
pub fn normalizar(texto: &str) -> String {
    let mut salida = String::with_capacity(texto.len());
    let mut hueco = false;
    for c in texto.chars() {
        let c = sin_tilde(c.to_lowercase().next().unwrap_or(c));
        if c.is_alphanumeric() {
            if hueco && !salida.is_empty() {
                salida.push(' ');
            }
            hueco = false;
            salida.push(c);
        } else {
            hueco = true;
        }
    }
    salida
}

fn sin_tilde(c: char) -> char {
    match c {
        'á' | 'à' | 'ä' | 'â' => 'a',
        'é' | 'è' | 'ë' | 'ê' => 'e',
        'í' | 'ì' | 'ï' | 'î' => 'i',
        'ó' | 'ò' | 'ö' | 'ô' => 'o',
        'ú' | 'ù' | 'ü' | 'û' => 'u',
        'ñ' => 'n',
        otro => otro,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **Los dos casos reales**, copiados tal cual de la primera prueba de punta a punta: sonó
    /// `pregunta-es.wav` por los altavoces del MacBook y estos son los textos que salieron de las
    /// dos pistas. No son ejemplos inventados para que el test pase; son el motivo de que este
    /// módulo exista.
    const DEL_SISTEMA: &str =
        "¿Tienen certificación ISO27.001 y la limpieza de datos eso está dentro del alcance.";
    const ECO_1: &str = "Tienen certificaciones o 27";
    const ECO_2: &str = "Y la limpieza de datos, eso está dentro del alcance?";

    fn del_sistema() -> Vec<Tramo<'static>> {
        vec![Tramo { desde_ms: 740, hasta_ms: 6_100, texto: DEL_SISTEMA }]
    }

    #[test]
    fn el_eco_de_la_prueba_real_se_reconoce() {
        let uno = Tramo { desde_ms: 880, hasta_ms: 2_920, texto: ECO_1 };
        let dos = Tramo { desde_ms: 3_420, hasta_ms: 6_240, texto: ECO_2 };
        assert!(
            es_eco(&uno, &del_sistema()),
            "«{ECO_1}» no se reconoció como eco (parecido {:.2})",
            parecido(ECO_1, DEL_SISTEMA)
        );
        assert!(
            es_eco(&dos, &del_sistema()),
            "«{ECO_2}» no se reconoció como eco (parecido {:.2})",
            parecido(ECO_2, DEL_SISTEMA)
        );
    }

    /// **El gate de este módulo, por el otro lado.** Interrumpir a alguien solapa en el tiempo
    /// igual que el eco, y confundir una interrupción con un reflejo borraría de la pista del
    /// consultor justo lo que dijo cuando más importaba.
    ///
    /// Se ve en rojo quitando la condición del parecido y dejando solo la del solape.
    #[test]
    fn interrumpir_no_es_hacer_eco() {
        let interrupcion = Tramo {
            desde_ms: 2_000,
            hasta_ms: 3_000,
            texto: "Perdona, sí, eso lo cubre la propuesta que te mandé",
        };
        assert!(
            !es_eco(&interrupcion, &del_sistema()),
            "una interrupción del consultor se tomó por eco (parecido {:.2})",
            parecido(interrupcion.texto, DEL_SISTEMA)
        );
    }

    /// Y por el otro extremo: repetir lo que el cliente acaba de decir es normal, y pasa DESPUÉS,
    /// no encima. Sin la condición del solape, la app borraría al consultor cada vez que resume.
    ///
    /// Se ve en rojo quitando la condición del solape y dejando solo la del parecido.
    #[test]
    fn repetir_despues_lo_que_dijo_el_cliente_no_es_eco() {
        let repeticion = Tramo {
            desde_ms: 6_500,
            hasta_ms: 9_000,
            texto: "la limpieza de datos, eso está dentro del alcance, sí",
        };
        assert!(
            solape(&repeticion, &del_sistema()[0]) < SOLAPE_MINIMO,
            "el caso de prueba no está construido como dice su nombre"
        );
        assert!(!es_eco(&repeticion, &del_sistema()));
    }

    #[test]
    fn sin_nada_con_que_comparar_no_hay_eco() {
        let solo = Tramo { desde_ms: 0, hasta_ms: 1_000, texto: "hola" };
        assert!(!es_eco(&solo, &[]));
    }

    #[test]
    fn un_turno_sin_texto_no_se_parece_a_nada() {
        assert_eq!(parecido("", "lo que sea"), 0.0);
        assert_eq!(parecido("ab", "abc"), 0.0, "menos de tres letras no da ni un trozo");
    }

    #[test]
    fn normalizar_quita_tildes_signos_y_mayusculas() {
        assert_eq!(
            normalizar("¿Está DENTRO del alcance?"),
            "esta dentro del alcance"
        );
        assert_eq!(normalizar("ISO27.001"), "iso27 001");
        assert_eq!(normalizar("  ...  "), "");
    }

    #[test]
    fn lo_identico_se_parece_del_todo() {
        assert!((parecido(DEL_SISTEMA, DEL_SISTEMA) - 1.0).abs() < 1e-6);
    }
}
