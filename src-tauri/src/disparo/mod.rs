//! EL DISPARADOR — cuándo la app se molesta en buscar.
//!
//! La VISION lo dice en una línea: *«una pregunta, un término tuyo, una cifra o un silencio
//! disparan la ficha; y un atajo global "ayúdame con esto" por si algo falla»*. Los cinco motivos
//! de `Motivo` son esos, y no hay un sexto.
//!
//! Tres decisiones que marcan el carácter de la app:
//!
//! **Dispara el cliente, no el consultor.** La ficha existe para responder a lo que preguntan
//! desde el otro lado. Si la propia voz del usuario disparara, la app le contestaría a él mismo
//! en mitad de su frase. La única excepción es el atajo: ahí el usuario pide en persona.
//!
//! **Un turno marcado como eco jamás dispara.** Con altavoces internos el micrófono oye al
//! cliente, y ese mismo turno llega por las dos pistas. Sin esta regla la app buscaría dos veces
//! lo mismo y enseñaría la ficha dos veces.
//!
//! **Hay una espera entre disparos.** Una pregunta larga puede llegar troceada en tres turnos, y
//! sin espera la banda parpadearía tres fichas en cuatro segundos. No es cosmética: una ficha que
//! se va antes de leerse es peor que ninguna.
//!
//! Todo esto es código: ni un token de modelo. La regla del código primero pide exactamente esto:
//! el disparo es determinista, y la única pieza con modelo de la app —la sugerencia, `sintesis/`—
//! llega después de la ficha y no decide cuándo aparece.

use crate::capture::Quien;
use crate::stt::Turno;

pub mod tolerancia;
pub use tolerancia::{FichaVista, Tolerancia};

/// Cuánto calla la app entre dos fichas. Medido contra el ritmo de una conversación: por debajo
/// la banda parpadea, por encima se pierde la pregunta siguiente.
pub const ESPERA_MS: usize = 6_000;

/// Un silencio del cliente más largo que esto, tras haber hablado, también es una petición de
/// ayuda: es el hueco en el que el consultor tiene que decir algo.
pub const SILENCIO_MS: usize = 4_000;

/// Menos palabras que esto no llevan pregunta: «sí», «ajá», «claro».
const MINIMO_DE_PALABRAS: usize = 3;

/// Con signo de interrogación bastan dos. «¿Tienen certificación?» es una pregunta entera y con
/// el techo de tres se caía — lo encontró su propio test, no una reunión.
const MINIMO_CON_SIGNO: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Motivo {
    /// El cliente preguntó algo.
    Pregunta,
    /// Dijo una cifra: un precio, un plazo, una norma.
    Cifra,
    /// Nombró algo que está en el corpus del usuario.
    TerminoDelCorpus,
    /// Lleva callado desde que preguntó.
    SilencioLargo,
    /// El usuario lo pidió con ⌃⌥A. Siempre gana: es la salida cuando lo demás falla.
    Atajo,
    /// **La pantalla que comparte el cliente cambió** y trae una cifra o uno de tus términos. Nace en
    /// el sprint 002 con la lectura de pantalla (C8): la ficha llega sin que nadie pregunte.
    Pantalla,
}

impl Motivo {
    /// Lo que la banda enseña como razón. En español; el inglés lo pone la interfaz.
    pub fn etiqueta(self) -> &'static str {
        match self {
            Motivo::Pregunta => "pregunta",
            Motivo::Cifra => "cifra",
            Motivo::TerminoDelCorpus => "término tuyo",
            Motivo::SilencioLargo => "silencio",
            Motivo::Atajo => "lo pediste",
            Motivo::Pantalla => "en pantalla",
        }
    }
}

/// Las palabras con las que se pregunta, en los dos idiomas. Sin tildes: se comparan contra texto
/// ya normalizado, igual que en el clasificador de unidades.
const INTERROGATIVOS: &[&str] = &[
    "que", "cual", "cuales", "cuanto", "cuanta", "cuantos", "cuantas", "como", "cuando", "donde",
    "quien", "quienes", "porque", "acaso", "tienen", "tiene", "pueden", "puede", "podrian",
    "hacen", "manejan", "ofrecen", "incluye", "incluyen", "cubren", "existe", "hay",
    "what", "which", "how", "when", "where", "who", "whom", "why", "whose",
];

/// **Los auxiliares del inglés preguntan solo si ABREN la frase** («Do you…», «Is there…», «And can we…»). En medio
/// son una afirmación: «We have the same problem», «That is exactly what…», «Access is requested…». Hasta el sprint
/// 005 contaban en cualquiera de las tres primeras palabras, como los interrogativos, y el kit v4 lo cazó en la sala
/// inglesa: tres frases dichas sin pregunta disparaban como preguntas, en presencial y en una reunión.
const AUXILIARES: &[&str] = &["does", "do", "can", "could", "would", "will", "are", "is", "have", "has", "any"];

/// Lo que puede ir delante de un auxiliar sin quitarle la pregunta: «And is that included?».
const CONJUNCIONES: &[&str] = &["and", "so", "but", "or", "then"];

/// Estado del disparador entre turnos. Vive en la escucha.
#[derive(Debug, Default)]
pub struct Disparador {
    ultimo_ms: Option<usize>,
    /// Lo último que se buscó, ya normalizado: una pregunta repetida no vuelve a disparar.
    ultima_consulta: String,
    /// **La tolerancia a tu voz**, para los turnos de la sala (ADR 020 §3). De fábrica, la regla de la casa.
    tolerancia: Tolerancia,
    /// La última ficha con resultado, que la tolerancia mira (C1 y C2). Se olvida con el corte.
    ficha_vista: Option<FichaVista>,
}

/// Lo que el disparador necesita saber del mundo, sin depender de él.
pub struct Contexto<'a> {
    /// Reloj de la escucha, en milisegundos.
    pub ahora_ms: usize,
    /// Palabras distintivas del corpus del usuario. Vacío mientras no haya corpus.
    pub vocabulario: &'a [String],
}

impl Disparador {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Un disparador con otra tolerancia para la sala: el kit v4 recorre así las candidatas.
    pub fn con_tolerancia(tolerancia: Tolerancia) -> Self {
        Self { tolerancia, ..Self::default() }
    }

    /// La tolerancia que aplica a la sala.
    pub fn tolerancia(&self) -> &Tolerancia {
        &self.tolerancia
    }

    /// **Salió una ficha con resultado.** La escucha lo dice aquí para que la tolerancia de la sala sepa cuándo y
    /// con qué palabras (C1 y C2). En una reunión se apunta igual y no cambia nada: solo la sala la mira.
    pub fn vio_ficha(&mut self, ms: usize, titular: &str, linea: &str) {
        self.ficha_vista = Some(FichaVista::nueva(ms, titular, linea));
    }

    /// El turno que acaba de cerrarse, ¿pide ficha?
    pub fn mirar(&mut self, turno: &Turno, ctx: &Contexto) -> Option<Motivo> {
        match turno.pista.quien() {
            // El consultor no se dispara a sí mismo.
            Quien::Tuyo => None,
            // Y un eco es el consultor oyendo al cliente por sus altavoces: ya llegó por su pista.
            Quien::Cliente if turno.eco => None,
            Quien::Cliente => {
                let motivo = self.por_que(&turno.texto, ctx)?;
                self.aceptar(&turno.texto, ctx.ahora_ms, motivo)
            }
            // **La sala** (ADR 020): no se sabe quién habló, así que dispara con las reglas de
            // siempre y después pasa por la tolerancia a tu voz.
            Quien::SinAtribuir => {
                let motivo = self.por_que(&turno.texto, ctx)?;
                if !self.tolerancia.deja(motivo, &turno.texto, ctx.ahora_ms, self.ficha_vista.as_ref()) {
                    return None;
                }
                self.aceptar(&turno.texto, ctx.ahora_ms, motivo)
            }
        }
    }

    /// El atajo del usuario. Se salta la espera y la repetición: si lo pide dos veces seguidas es
    /// porque la primera no le sirvió, y responderle con silencio sería lo peor que podría pasar
    /// justo en el momento en que decidió pedir ayuda a mano.
    pub fn a_mano(&mut self, ahora_ms: usize) -> Motivo {
        self.ultimo_ms = Some(ahora_ms);
        self.ultima_consulta.clear();
        Motivo::Atajo
    }

    /// El cliente preguntó y lleva callado desde entonces.
    ///
    /// **Se le pasa lo último que dijo, y no una cadena vacía como hasta el sprint 002**, porque el
    /// texto es lo que hace funcionar la guardia contra repetidos: si esa frase ya trajo su ficha,
    /// el silencio que viene detrás no tiene que traer la misma otra vez. Con la cadena vacía que
    /// había aquí, `aceptar` se saltaba la comparación —una consulta vacía no se compara con
    /// nada— **y además pisaba la última consulta con «»**, así que la pregunta siguiente del
    /// cliente, aunque fuera idéntica a la anterior, volvía a disparar. Nunca se notó porque este
    /// método no tuvo un llamador hasta que se cableó.
    ///
    /// **Se le pasa el turno entero desde el sprint 005**, no solo su texto: el silencio que sigue a la sala pasa
    /// por la tolerancia (que puede apagarlo), y el que sigue a tu voz no es un silencio del cliente.
    pub fn por_silencio(&mut self, turno: &Turno, ctx: &Contexto) -> Option<Motivo> {
        if ctx.ahora_ms.checked_sub(turno.hasta_ms)? < SILENCIO_MS {
            return None;
        }
        match turno.pista.quien() {
            Quien::Tuyo => return None,
            Quien::Cliente => {}
            Quien::SinAtribuir => {
                let vista = self.ficha_vista.as_ref();
                if !self.tolerancia.deja(Motivo::SilencioLargo, &turno.texto, ctx.ahora_ms, vista) {
                    return None;
                }
            }
        }
        self.aceptar(&turno.texto, ctx.ahora_ms, Motivo::SilencioLargo)
    }

    /// **La pantalla nueva pide ficha.** Pasa por la misma guardia que un turno —la espera entre
    /// fichas y la negativa a repetir—, así que una diapositiva que se queda veinte minutos en
    /// pantalla trae UNA ficha, no una cada vez que alguien mueve el ratón. `texto` es lo que la
    /// pantalla aporta a la consulta, y es lo que se compara para no repetir.
    ///
    /// El reloj es el de la sesión en milisegundos, el mismo que usan los turnos.
    pub fn por_pantalla(&mut self, texto: &str, ahora_ms: usize) -> Option<Motivo> {
        self.aceptar(texto, ahora_ms, Motivo::Pantalla)
    }

    fn aceptar(&mut self, texto: &str, ahora_ms: usize, motivo: Motivo) -> Option<Motivo> {
        if let Some(antes) = self.ultimo_ms {
            // **El reloj puede VOLVER ATRÁS.** `ahora_ms` es el reloj de la pista, y cuando su
            // anillo da la vuelta entera la escucha se reengancha al presente y ese reloj vuelve a
            // empezar en cero. Con la resta saturada que había aquí, «0 − 60 000» daba 0: la espera
            // entre fichas se cumplía para siempre y **el disparador quedaba muerto el resto de la
            // sesión**, sin que nada lo dijera. Un reloj que retrocede no es «hace un instante»: es
            // otro reloj, y lo que midió el anterior ya no sirve para comparar. Hallazgo A5 de la
            // auditoría del sprint.
            if ahora_ms >= antes && ahora_ms - antes < ESPERA_MS {
                return None;
            }
        }
        let consulta = crate::corpus::consulta::limpiar(texto);
        if !consulta.is_empty() && consulta == self.ultima_consulta {
            return None;
        }
        self.ultimo_ms = Some(ahora_ms);
        self.ultima_consulta = consulta;
        Some(motivo)
    }

    /// Olvida lo que oyó. **Pisa las letras antes de soltarlas**, igual que la ventana de
    /// turnos: `ultima_consulta` guarda palabras del cliente, y soltar una `String` deja su
    /// contenido en el montón hasta que otra cosa lo pise. Es la misma pieza «Transcript» del
    /// kill-switch, en su otro escondite.
    pub fn reiniciar(&mut self) {
        // SEGURIDAD: se escriben ceros sobre bytes que ya eran UTF-8 válido; el cero también lo
        // es, así que la cadena sigue siendo válida en todo momento.
        unsafe { self.ultima_consulta.as_mut_vec() }.fill(0);
        self.ultima_consulta.clear();
        self.ultimo_ms = None;
        self.ficha_vista = None;
    }
}

/// Las reglas léxicas, aparte del estado para poder probarlas solas.
impl Disparador {
    fn por_que(&self, texto: &str, ctx: &Contexto) -> Option<Motivo> {
        let palabras: Vec<String> = normalizar(texto).split_whitespace().map(str::to_string).collect();
        // El signo de cierre es la señal más fiable: el transcriptor lo escribe cuando oye
        // entonación de pregunta, y no depende del vocabulario. Por eso se conforma con menos
        // palabras que el resto de las reglas.
        if texto.contains('?') && palabras.len() >= MINIMO_CON_SIGNO {
            return Some(Motivo::Pregunta);
        }
        if palabras.len() < MINIMO_DE_PALABRAS {
            return None;
        }
        // Un interrogativo, pero solo si abre la frase: «no sé QUE hacer» no es una pregunta.
        if palabras.iter().take(3).any(|p| INTERROGATIVOS.contains(&p.as_str())) {
            return Some(Motivo::Pregunta);
        }
        // Y un auxiliar del inglés, solo si es lo primero (tras una conjunción, si la hay).
        let primera = palabras.iter().find(|p| !CONJUNCIONES.contains(&p.as_str()));
        if primera.is_some_and(|p| AUXILIARES.contains(&p.as_str())) {
            return Some(Motivo::Pregunta);
        }
        if !ctx.vocabulario.is_empty()
            && palabras.iter().any(|p| ctx.vocabulario.iter().any(|v| v == p))
        {
            return Some(Motivo::TerminoDelCorpus);
        }
        if palabras.iter().any(|p| p.chars().any(|c| c.is_ascii_digit())) {
            return Some(Motivo::Cifra);
        }
        None
    }
}

pub(crate) fn normalizar(texto: &str) -> String {
    texto
        .chars()
        .flat_map(|c| c.to_lowercase())
        .map(|c| match c {
            'á' | 'à' | 'ä' | 'â' => 'a',
            'é' | 'è' | 'ë' | 'ê' => 'e',
            'í' | 'ì' | 'ï' | 'î' => 'i',
            'ó' | 'ò' | 'ö' | 'ô' => 'o',
            'ú' | 'ù' | 'ü' | 'û' => 'u',
            'ñ' => 'n',
            c if c.is_alphanumeric() => c,
            _ => ' ',
        })
        .collect()
}

/// Las palabras distintivas del corpus, sacadas de los nombres de documento y de los títulos de
/// sección. Es lo que convierte «¿y el Páramo Azul?» en un disparo: nadie más dice eso.
///
/// Se queda con las largas y deja fuera las que dice todo el mundo — si «propuesta» disparara, la
/// app saltaría en cada frase de una reunión de consultoría.
pub fn vocabulario(nombres: &[String], titulos: &[String]) -> Vec<String> {
    const COMUNES: &[&str] = &[
        "propuesta", "proposal", "marco", "framework", "caso", "case", "cliente", "client",
        "perfil", "profile", "alcance", "scope", "precio", "price", "resumen", "summary",
        "anexo", "annex", "introduccion", "introduction", "conclusiones", "conclusions",
        "documento", "document", "seccion", "section", "notas", "notes", "informe", "report",
    ];
    let mut salida: Vec<String> = Vec::new();
    for texto in nombres.iter().chain(titulos.iter()) {
        for p in normalizar(texto).split_whitespace() {
            if p.chars().count() >= 4 && !COMUNES.contains(&p) && !salida.iter().any(|v| v == p) {
                salida.push(p.to_string());
            }
        }
    }
    salida
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::capture::Pista;

    fn turno(texto: &str) -> Turno {
        Turno {
            pista: Pista::Sistema,
            desde_ms: 0,
            hasta_ms: 2_000,
            texto: texto.into(),
            hora: "14:02".into(),
            eco: false,
        }
    }

    fn ctx(ahora_ms: usize) -> Contexto<'static> {
        Contexto { ahora_ms, vocabulario: &[] }
    }

    /// Lo que el cliente dijo, cerrado en `hasta_ms`: lo que mira el silencio.
    fn dicho_en(texto: &str, hasta_ms: usize) -> Turno {
        Turno { hasta_ms, ..turno(texto) }
    }

    #[test]
    fn una_pregunta_del_cliente_dispara() {
        let mut d = Disparador::nuevo();
        assert_eq!(d.mirar(&turno("¿Ustedes tienen certificación?"), &ctx(0)), Some(Motivo::Pregunta));
    }

    #[test]
    fn dispara_igual_en_ingles() {
        let mut d = Disparador::nuevo();
        assert_eq!(d.mirar(&turno("How long does the rollout take"), &ctx(0)), Some(Motivo::Pregunta));
    }

    /// La ficha responde al cliente. Si la voz del propio consultor disparara, la app le
    /// contestaría a él en mitad de su frase.
    #[test]
    fn el_consultor_no_se_dispara_a_si_mismo() {
        let mut d = Disparador::nuevo();
        let mio = Turno { pista: Pista::Microfono, ..turno("¿Y cuánto cuesta esto?") };
        assert_eq!(d.mirar(&mio, &ctx(0)), None);
    }

    /// Con altavoces el mismo turno llega por las dos pistas. Sin esta regla se buscaría dos veces.
    #[test]
    fn un_turno_marcado_como_eco_no_dispara() {
        let mut d = Disparador::nuevo();
        let eco = Turno { eco: true, ..turno("¿Ustedes tienen certificación?") };
        assert_eq!(d.mirar(&eco, &ctx(0)), None);
    }

    /// **El reloj de una pista vuelve a cero** cuando su anillo da la vuelta y la escucha se
    /// reengancha al presente. Antes de este arreglo, el disparador comparaba el reloj nuevo con el
    /// viejo, la resta saturada daba cero y la espera entre fichas se cumplía para siempre: a
    /// partir de ese instante la app no volvía a buscar nada en toda la reunión, callada.
    ///
    /// Se ve en rojo devolviendo la comparación a `ahora_ms.saturating_sub(antes) < ESPERA_MS`.
    #[test]
    fn un_reloj_que_vuelve_atras_no_deja_al_disparador_muerto() {
        let mut d = Disparador::nuevo();
        // Un minuto de reunión: la primera pregunta dispara.
        assert_eq!(
            d.mirar(&turno("¿Ustedes tienen certificación?"), &ctx(60_000)),
            Some(Motivo::Pregunta)
        );
        // Y aquí el anillo dio la vuelta: la pista se reengancha y su reloj arranca de cero.
        assert_eq!(
            d.mirar(&turno("¿Y en cuántas semanas hacen la entrega?"), &ctx(500)),
            Some(Motivo::Pregunta),
            "tras el reenganche el disparador se quedó mudo el resto de la sesión"
        );
        // Lo que NO cambia es la espera de verdad: con el reloj nuevo ya en marcha, dos preguntas
        // seguidas siguen siendo una sola ficha.
        assert_eq!(d.mirar(&turno("¿Y el soporte está incluido?"), &ctx(1_200)), None);
    }

    #[test]
    fn una_cifra_dispara_aunque_no_sea_pregunta() {
        let mut d = Disparador::nuevo();
        assert_eq!(d.mirar(&turno("Nos hablaron de 40 semanas"), &ctx(0)), Some(Motivo::Cifra));
    }

    #[test]
    fn un_termino_del_corpus_dispara_y_pesa_mas_que_la_cifra() {
        let vocab = vocabulario(&["Páramo Azul · propuesta".into()], &[]);
        let mut d = Disparador::nuevo();
        let c = Contexto { ahora_ms: 0, vocabulario: &vocab };
        assert_eq!(d.mirar(&turno("Lo del paramo azul de 2024"), &c), Some(Motivo::TerminoDelCorpus));
    }

    /// Si «propuesta» disparara, la app saltaría en cada frase de una reunión de consultoría.
    #[test]
    fn el_vocabulario_deja_fuera_las_palabras_que_dice_todo_el_mundo() {
        let v = vocabulario(&["Propuesta Páramo Azul".into()], &["Alcance".into()]);
        assert!(v.contains(&"paramo".to_string()));
        assert!(!v.contains(&"propuesta".to_string()), "«propuesta» dispararía en cada frase");
        assert!(!v.contains(&"alcance".to_string()));
        assert!(v.contains(&"azul".to_string()), "«azul» es distintiva y debería entrar");
    }

    #[test]
    fn un_gruñido_no_es_una_pregunta() {
        let mut d = Disparador::nuevo();
        assert_eq!(d.mirar(&turno("Ajá, claro"), &ctx(0)), None);
        assert_eq!(d.mirar(&turno("Sí"), &ctx(0)), None);
        assert_eq!(d.mirar(&turno("¿Sí?"), &ctx(0)), None, "un signo no convierte un gruñido en pregunta");
    }

    /// Dos palabras y un signo son una pregunta entera. El techo de tres palabras se la comía, y
    /// lo encontró este test y no una reunión.
    #[test]
    fn una_pregunta_de_dos_palabras_con_signo_dispara() {
        let mut d = Disparador::nuevo();
        assert_eq!(
            d.mirar(&turno("¿Tienen certificación?"), &ctx(0)),
            Some(Motivo::Pregunta)
        );
    }

    /// «No sé QUE hacer» no es una pregunta aunque lleve un interrogativo dentro.
    #[test]
    fn un_interrogativo_a_mitad_de_frase_no_convierte_la_frase_en_pregunta() {
        let mut d = Disparador::nuevo();
        assert_eq!(d.mirar(&turno("Todavía no tenemos claro que camino tomar"), &ctx(0)), None);
    }

    /// Una pregunta larga llega troceada. Sin espera, la banda parpadearía tres fichas seguidas.
    #[test]
    fn dos_disparos_seguidos_se_convierten_en_uno() {
        let mut d = Disparador::nuevo();
        assert!(d.mirar(&turno("¿Tienen certificación ISO?"), &ctx(0)).is_some());
        assert_eq!(d.mirar(&turno("¿O algo parecido, digo yo?"), &ctx(1_500)), None);
        assert!(d.mirar(&turno("¿Y el plazo de entrega cuál sería?"), &ctx(ESPERA_MS + 10)).is_some());
    }

    /// Y la misma pregunta repetida tampoco, aunque haya pasado la espera: sería la misma ficha.
    #[test]
    fn la_misma_pregunta_repetida_no_vuelve_a_disparar() {
        let mut d = Disparador::nuevo();
        assert!(d.mirar(&turno("¿Tienen certificación ISO 27001?"), &ctx(0)).is_some());
        assert_eq!(d.mirar(&turno("¿Tienen certificación ISO 27001?"), &ctx(ESPERA_MS + 10)), None);
    }

    /// El atajo es la salida cuando lo demás falla: no lo frena ni la espera ni la repetición.
    #[test]
    fn el_atajo_se_salta_la_espera_porque_es_la_salida_de_emergencia() {
        let mut d = Disparador::nuevo();
        assert!(d.mirar(&turno("¿Tienen certificación?"), &ctx(0)).is_some());
        assert_eq!(d.a_mano(100), Motivo::Atajo);
        assert_eq!(d.a_mano(200), Motivo::Atajo);
    }

    #[test]
    fn el_silencio_largo_dispara_solo_cuando_de_verdad_es_largo() {
        let mut d = Disparador::nuevo();
        assert_eq!(d.por_silencio(&dicho_en("de 40 semanas nos hablaron", 0), &ctx(SILENCIO_MS - 1)), None);
        assert_eq!(
            d.por_silencio(&dicho_en("de 40 semanas nos hablaron", 0), &ctx(SILENCIO_MS)),
            Some(Motivo::SilencioLargo)
        );
    }

    /// **El silencio que sigue a una pregunta ya contestada no vuelve a contestarla.** El turno
    /// dispara su ficha; cuatro segundos después el cliente sigue callado y el silencio mira ese
    /// mismo turno — la única cosa que dijo. Sin pasarle el texto, esto era una segunda ficha
    /// idéntica en la banda, y a la tercera vuelta del reloj una tercera.
    #[test]
    fn el_silencio_no_repite_la_ficha_que_la_pregunta_ya_trajo() {
        let mut d = Disparador::nuevo();
        let dicho = "¿Ustedes tienen certificación ISO 27001?";
        assert!(d.mirar(&turno(dicho), &ctx(2_000)).is_some());
        // Pasada la espera entre fichas, que es lo único que frenaba al silencio antes.
        assert_eq!(d.por_silencio(&dicho_en(dicho, 2_000), &ctx(2_000 + ESPERA_MS + 1)), None);
    }

    /// Y al contrario: lo que el cliente dijo **sin** que disparara —una frase sin pregunta, sin
    /// cifra y sin término del corpus— es exactamente el caso para el que existe el silencio.
    #[test]
    fn el_silencio_dispara_justo_por_lo_que_no_disparo_solo() {
        let mut d = Disparador::nuevo();
        let dicho = "Nosotros veníamos trabajando con otro proveedor";
        assert_eq!(d.mirar(&turno(dicho), &ctx(2_000)), None, "esa frase no debería disparar sola");
        assert_eq!(
            d.por_silencio(&dicho_en(dicho, 2_000), &ctx(2_000 + SILENCIO_MS)),
            Some(Motivo::SilencioLargo)
        );
    }

    /// **Un auxiliar del inglés pregunta solo si abre la frase** (kit v4, sprint 005). ¿Puede fallar? Sí: con los
    /// auxiliares otra vez entre los interrogativos de las tres primeras palabras, «We have the same problem» dispara
    /// (bitácora del sprint 005, fase 1).
    #[test]
    fn los_auxiliares_del_ingles_preguntan_solo_al_abrir_la_frase() {
        for afirmacion in [
            "We have the same problem with the rural stores.",
            "That is exactly what the margin by channel dashboard measures.",
            "Access is requested in writing, read only, and revoked on closing day.",
        ] {
            assert_eq!(Disparador::nuevo().mirar(&turno(afirmacion), &ctx(0)), None, "«{afirmacion}» no es una pregunta");
        }
        for pregunta in [
            "Do you have ISO certification",
            "And is data cleaning within the scope",
            "Can we start with the scope",
            "So does that include the workshop",
        ] {
            assert_eq!(Disparador::nuevo().mirar(&turno(pregunta), &ctx(0)), Some(Motivo::Pregunta), "«{pregunta}» es una pregunta");
        }
    }

    /// **La tolerancia solo toca la sala.** El mismo turno y la misma regla: del cliente dispara, de la sala no. Y
    /// tu silencio no es un silencio del cliente.
    ///
    /// ¿Puede fallar? Sí: con la tolerancia aplicada a `Quien::Cliente`, una reunión perdería fichas por una regla
    /// pensada para la sala (bitácora del sprint 005, fase 1).
    #[test]
    fn la_tolerancia_solo_toca_la_sala() {
        let regla = Tolerancia { solo_pregunta_o_cifra: true, silencio: false, ..Tolerancia::SIN_FILTRO };
        let vocabulario = vec!["paramo".to_string()];
        let c = Contexto { ahora_ms: 2_000, vocabulario: &vocabulario };
        let dicho = "Lo del Páramo Azul lo vemos luego";

        let mut reunion = Disparador::con_tolerancia(regla.clone());
        assert_eq!(reunion.mirar(&turno(dicho), &c), Some(Motivo::TerminoDelCorpus), "la tolerancia de la sala calló al cliente");
        let mut sala = Disparador::con_tolerancia(regla.clone());
        assert_eq!(sala.mirar(&Turno { pista: Pista::Sala, ..turno(dicho) }, &c), None, "la sala disparó con un término suelto");

        let callado = "Nosotros veníamos trabajando con otro proveedor";
        let mut reunion = Disparador::con_tolerancia(regla.clone());
        assert_eq!(reunion.por_silencio(&dicho_en(callado, 2_000), &ctx(2_000 + SILENCIO_MS)), Some(Motivo::SilencioLargo));
        let mut sala = Disparador::con_tolerancia(regla);
        let de_la_sala = Turno { pista: Pista::Sala, ..dicho_en(callado, 2_000) };
        assert_eq!(sala.por_silencio(&de_la_sala, &ctx(2_000 + SILENCIO_MS)), None, "el silencio apagado disparó en la sala");
        let mut tuyo = Disparador::nuevo();
        let de_tu_pista = Turno { pista: Pista::Microfono, ..dicho_en(callado, 2_000) };
        assert_eq!(tuyo.por_silencio(&de_tu_pista, &ctx(2_000 + SILENCIO_MS)), None, "tu silencio disparó como si fuera del cliente");
    }

    /// **La ficha vista se olvida con el corte**, con el resto de lo que el disparador recuerda.
    #[test]
    fn el_corte_olvida_la_ficha_vista() {
        let mut d = Disparador::con_tolerancia(Tolerancia { espera_tras_ficha_ms: 60_000, ..Tolerancia::SIN_FILTRO });
        d.vio_ficha(1_000, "Plazo de entrega", "cuatro semanas");
        let sala = |texto: &str| Turno { pista: Pista::Sala, ..turno(texto) };
        assert_eq!(d.mirar(&sala("¿Y el precio total?"), &ctx(20_000)), None, "C1 no calló tras la ficha");
        d.reiniciar();
        assert_eq!(d.mirar(&sala("¿Y el precio total?"), &ctx(20_000)), Some(Motivo::Pregunta), "tras el corte la sala sigue callada por una ficha olvidada");
    }
}
