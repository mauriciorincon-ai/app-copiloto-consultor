//! EL KILL-SWITCH — `⌥⎋`: corta todo y vacía la memoria.
//!
//! Es la pieza que hace verificable la promesa del efímero. No sirve de nada prometer que nada se
//! guarda si el usuario no tiene **una tecla** que lo compruebe delante de su cliente: la
//! honestidad de esta app no es un texto legal, es un botón que deja la cifra en `0 B` a la
//! vista.
//!
//! **El problema de un kill-switch a medio construir.** En la fase 2 de este sprint la mitad de lo
//! que hay que cortar todavía no existía —ni audio, ni transcript, ni lectura de pantalla—. La
//! tentación era escribir el corte de lo que había y «ya lo ampliaremos». Y eso falla en silencio:
//! cuando la fase 3 añadiera los búferes de audio, nadie recordaría volver aquí, el kill-switch
//! seguiría en verde y **dejaría el audio dentro**.
//!
//! No pasó, y no pasó por cómo está escrito este archivo: al añadir las pistas en la fase 3, el
//! compilador no dejó compilar hasta resolverlas. La cuenta pasó de tres piezas cortadas a seis, y
//! la lectura de pantalla fue la última declarada como inexistente, hasta que llegó con C8 en la
//! fase 3 del sprint 002. Desde entonces se cortan las ocho.
//!
//! Por eso el corte no es una lista de acciones sino una lista de **piezas** ([`Pieza`]), y cada
//! una tiene que estar en uno de dos sitios: cortada, o declarada como que aún no existe.
//!
//! **Y quien lo obliga es el compilador, no un test.** Primera versión de este módulo: un test
//! comprobaba `TODAS.len() == 7`. Al plantarle una pieza nueva **pasó en verde** — claro: el 7 lo
//! había escrito yo, así que el test se medía contra mi propio número, no contra el código. Lo
//! que sí no se puede esquivar son dos `match` **sin comodín** ([`Pieza::orden`] y
//! [`suerte_en_este_sprint`]): añadir una variante y no resolverla **no compila**. Los tests de
//! abajo comprueban lo que al compilador se le escapa — que [`TODAS`] y `orden` digan lo mismo.

use serde::Serialize;

/// Todo lo que el kill-switch tiene que dejar en cero. **Añadir una variante aquí obliga a
/// resolverla**, y ese es el punto.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Pieza {
    /// El contador de bytes que salieron: vuelve a `0 B`.
    ContadorDeRed,
    /// La ventana de la reunión vuelve a su tamaño.
    Acople,
    /// La banda y su relleno desaparecen de la pantalla.
    Banda,
    /// Búfer circular del micrófono: se cierra el grifo y se pisan las muestras con ceros.
    AudioDelMicrofono,
    /// Búfer circular del audio del sistema: el tap se destruye y el anillo se pisa.
    AudioDelSistema,
    /// El último fotograma leído de la pantalla y el texto que se sacó de él. Nace en el sprint 002
    /// con C8: se para el vigía y los dos se pisan con ceros.
    UltimoFrame,
    /// La ventana de turnos transcritos: se sobrescriben las letras antes de soltarlas.
    Transcript,
    /// **La voz que sale**: se corta lo que esté diciendo, se tira lo que quede en la cola y el modo
    /// solo audio se apaga. Nace en el sprint 002 con C15.
    ///
    /// Es la pieza más visible de todas y la única que se OYE. Si el usuario pulsa la tecla delante
    /// de su cliente y la app sigue leyéndole una ficha en voz alta, no hay informe que arregle eso:
    /// el kill-switch habría fallado en el único sitio donde el cliente puede notarlo.
    Voz,
}

/// Todas las piezas, en el orden en que se cortan: **primero lo que sigue entrando**.
///
/// El orden no es decorativo. Cerrar la banda antes de parar las pistas dejaría al audio
/// entrando sin nada en pantalla que lo dijera; vaciar un búfer que todavía recibe muestras lo
/// deja con muestras nuevas un instante después. Se corta el grifo, luego se vacía el vaso.
pub const TODAS: &[Pieza] = &[
    // La voz va PRIMERA, antes incluso que los grifos, y es el único sitio donde el orden se decide
    // por lo que el cliente percibe: es lo único de esta lista que se oye desde el otro lado de la
    // llamada. Cerrar un grifo tarda milisegundos y no se nota; una frase a medio decir, sí.
    Pieza::Voz,
    Pieza::AudioDelMicrofono,
    Pieza::AudioDelSistema,
    Pieza::UltimoFrame,
    Pieza::Transcript,
    Pieza::ContadorDeRed,
    Pieza::Banda,
    Pieza::Acople,
];

impl Pieza {
    /// El puesto de esta pieza en [`TODAS`].
    ///
    /// **Su único trabajo es obligar al compilador.** Es un `match` sin comodín: quien añada una
    /// variante a [`Pieza`] y no la nombre aquí no consigue compilar el crate, y por tanto no
    /// consigue dejar una pieza sin cortar. Es el gate de este módulo; los tests solo comprueban
    /// que la lista y este `match` cuenten la misma historia.
    pub const fn orden(self) -> usize {
        match self {
            Pieza::Voz => 0,
            Pieza::AudioDelMicrofono => 1,
            Pieza::AudioDelSistema => 2,
            Pieza::UltimoFrame => 3,
            Pieza::Transcript => 4,
            Pieza::ContadorDeRed => 5,
            Pieza::Banda => 6,
            Pieza::Acople => 7,
        }
    }
}

/// Qué pasó con cada pieza.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Suerte {
    /// Se cortó o se vació.
    Cortada,
    /// Todavía no existe en el producto. **No es un fallo y no se disimula**: la pantalla de
    /// honestidad enumera lo que hay, y en este sprint lo que hay es poco.
    AunNoExiste,
}

/// Lo que el usuario ve después de pulsar la tecla — y, antes de pulsarla, lo que el corte HARÍA.
///
/// **Le faltaba `rename_all` y nadie lo había notado**: llegaba al webview como `bytes_en_red`
/// mientras TypeScript habría esperado `bytesEnRed`. Es el mismo defecto que el C1 del sprint 001,
/// en el único sitio donde el gate del contrato no llegaba —porque esta struct no estaba en
/// `contrato.rs`—, y sobrevivió porque los cuatro suscriptores del evento `corte` lo usan como
/// **señal** y ninguno lee el payload. Desde el sprint 002 está en el contrato y se lee.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Informe {
    pub piezas: Vec<(Pieza, Suerte)>,
    /// Los bytes que quedaron en el contador. Es `0` siempre; se reporta para que la pantalla
    /// muestre un número **leído**, no una constante escrita en la interfaz.
    pub bytes_en_red: u64,
}

impl Informe {
    pub fn cortadas(&self) -> usize {
        self.piezas.iter().filter(|(_, s)| *s == Suerte::Cortada).count()
    }
}

/// Qué le toca a cada pieza **en este sprint**. Es la única función que hay que tocar cuando una
/// pieza empiece a existir.
pub fn suerte_en_este_sprint(pieza: Pieza) -> Suerte {
    match pieza {
        Pieza::ContadorDeRed
        | Pieza::Banda
        | Pieza::Acople
        | Pieza::AudioDelMicrofono
        | Pieza::AudioDelSistema
        | Pieza::Transcript
        // La voz existe desde la fase 2 del sprint 002 y se corta de verdad: `stopSpeaking` tira la
        // frase en curso y la cola entera, y el modo queda apagado.
        | Pieza::Voz
        // La lectura de pantalla existe desde la fase 3 del sprint 002 (C8): se para el vigía y se
        // pisan el cuadro de la reunión y el texto leído de él. Fue la última en llegar, y hasta
        // entonces se declaró como inexistente en vez de disimularse.
        | Pieza::UltimoFrame => Suerte::Cortada,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [`TODAS`] y [`Pieza::orden`] tienen que contar la misma historia: cada pieza en su puesto,
    /// sin huecos y sin repetidos.
    ///
    /// Esto **no** es el gate —el gate es que `orden` sea un `match` sin comodín, y lo cobra el
    /// compilador—. Esto es lo que al compilador se le escapa: que alguien dé a una pieza nueva
    /// un puesto que ya estaba ocupado, o uno que se sale de la lista.
    #[test]
    fn la_lista_y_los_puestos_dicen_lo_mismo() {
        for (i, pieza) in TODAS.iter().enumerate() {
            assert_eq!(
                pieza.orden(),
                i,
                "«{pieza:?}» está en el puesto {i} de TODAS pero dice ser el {}",
                pieza.orden()
            );
        }
        let mut puestos: Vec<usize> = TODAS.iter().map(|p| p.orden()).collect();
        puestos.sort_unstable();
        puestos.dedup();
        assert_eq!(puestos.len(), TODAS.len(), "dos piezas comparten puesto");
        assert_eq!(
            puestos.last().copied(),
            Some(TODAS.len() - 1),
            "hay un puesto fuera de la lista: una pieza se quedaría sin cortar"
        );
    }

    /// Se corta el grifo antes de vaciar el vaso: las pistas y la pantalla primero, la banda y el
    /// acople al final. Vaciar un búfer que todavía recibe muestras lo deja con muestras nuevas
    /// un instante después.
    #[test]
    fn se_corta_lo_que_sigue_entrando_antes_que_lo_que_solo_se_ve() {
        let pos = |p: Pieza| TODAS.iter().position(|q| *q == p).unwrap();
        for entrante in [Pieza::AudioDelMicrofono, Pieza::AudioDelSistema, Pieza::UltimoFrame] {
            assert!(
                pos(entrante) < pos(Pieza::Banda),
                "{entrante:?} se corta después de cerrar la banda: entraría a ciegas"
            );
        }
        assert!(pos(Pieza::Acople) == TODAS.len() - 1, "el acople se devuelve al final");
    }

    /// En este sprint se corta lo que existe, y lo que no existe **se dice**. Un kill-switch que
    /// informara «8 de 8 cortadas» teniendo piezas sin construir sería una mentira cómoda.
    ///
    /// La cuenta ha cambiado tres veces y **las tres las obligó el compilador**: en la fase 3 del
    /// sprint 001, de 3 y 4 a 6 y 1, cuando las pistas de audio y el transcript pasaron de
    /// declararse a cortarse de verdad; en la fase 2 del sprint 002, a 7 y 1, con la voz que sale; y
    /// en la fase 3 del sprint 002, a **8 y 0**, con la lectura de pantalla. Es la primera vez que
    /// el kill-switch corta todas sus piezas, y la primera en que `AunNoExiste` no le toca a nadie.
    #[test]
    fn en_este_sprint_se_cortan_las_ocho() {
        let cortadas = TODAS.iter().filter(|p| suerte_en_este_sprint(**p) == Suerte::Cortada).count();
        let futuras = TODAS.iter().filter(|p| suerte_en_este_sprint(**p) == Suerte::AunNoExiste).count();
        assert_eq!((cortadas, futuras), (8, 0));
    }
}
