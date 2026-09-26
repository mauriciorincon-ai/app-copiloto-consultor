//! **LA VOZ QUE SALE** — el modo solo audio (C15). **MÓDULO PROTEGIDO.**
//!
//! `voz/` es la voz que ENTRA: parte en turnos lo que se oye. Esto es lo contrario, y por eso no
//! puede llamarse igual. El plan del sprint decía `voz/` y está declarado como desviación en la
//! bitácora: dos módulos con el mismo nombre para las dos direcciones del sonido es la clase de
//! confusión que se descubre tarde.
//!
//! **De dónde viene esta funcionalidad.** No estaba en la VISION: nació en la mirada 3 de la etapa
//! de diseño, con estas palabras del usuario — *«quisiera tener un modo solo audio que me hable de
//! forma paralela por si quiero ver completamente la pantalla y no me interrumpa»*. Las dos mitades
//! de esa frase son los dos requisitos: **hablar** y **devolver la pantalla**. La segunda es la que
//! hace que la banda baje a 44 px, y es la que el usuario nombró al aprobar la mirada 16: *«amplio
//! margen para la pantalla de reunión»*.
//!
//! **Por qué está protegido, siendo texto del propio usuario.** Porque `AVSpeechSynthesizer` tiene
//! `write(_:toBufferCallback:)`: una manera de convertir la ficha en **audio en un archivo**. Nada
//! de este módulo la usa y nada puede usarla sin que `pnpm verify:ephemeral` lo vea. Un WAV con la
//! evidencia del consultor leída en voz alta sería una grabación de la reunión con otro nombre.
//!
//! **Y la decisión que gobierna el módulo entero: la voz no decide cuándo hablar.** Lo decide
//! [`cabe_decirla`], que es una función pura sobre un [`Momento`]. La razón es la regla 15 en su
//! forma más simple: lo que depende de un altavoz, de un hilo y de un Mac no se puede probar; lo
//! que depende de cinco booleanos, sí — y las cinco maneras de callarse son justo lo que no se
//! puede permitir equivocar.

pub mod apple;

use crate::capture::nativo::Salida;

/// Lo que cualquier voz tiene que saber hacer. Igual que `stt::Motor`: la app no conoce a
/// `AVSpeechSynthesizer`, conoce a quien sabe decir una frase.
pub trait Voz: Send + Sync {
    fn nombre(&self) -> &'static str;
    /// ¿Hay voz en este Mac para este idioma?
    fn hay_para(&self, idioma: &str) -> bool;
    /// Empieza a decirlo. **Vuelve enseguida**: el sonido sigue después.
    fn decir(&self, idioma: &str, texto: &str) -> Result<(), String>;
    /// Corta lo que esté diciendo y tira lo que quede en la cola.
    fn callar(&self);
    /// ¿Está diciendo algo ahora mismo?
    fn hablando(&self) -> bool;
}

/// **La voz muda, de primera clase.** Es la que corre en la integración continua, donde no hay
/// altavoz ni voces instaladas, y la que queda en un Mac compilado sin el puente de Swift. No
/// finge: `hay_para` es `false` y quien pregunte por qué recibe el motivo en español.
pub struct Muda {
    motivo: String,
}

impl Muda {
    pub fn por(motivo: impl Into<String>) -> Self {
        Self { motivo: motivo.into() }
    }

    pub fn motivo(&self) -> &str {
        &self.motivo
    }
}

impl Voz for Muda {
    fn nombre(&self) -> &'static str {
        "muda"
    }
    fn hay_para(&self, _idioma: &str) -> bool {
        false
    }
    fn decir(&self, _idioma: &str, _texto: &str) -> Result<(), String> {
        Err(self.motivo.clone())
    }
    fn callar(&self) {}
    fn hablando(&self) -> bool {
        false
    }
}

/// **Lo que la banda enseña del modo solo audio.**
///
/// Tres booleanos y ninguna frase, a propósito: las frases son copy y el copy vive en `src/i18n/`,
/// donde el gate del diccionario las compara una a una con `docs/diseno/banda.html`. Si Rust
/// mandara «Conecta auriculares», ese texto se podría cambiar aquí sin que ninguna mirada lo viera.
///
/// | Campo | Qué decide en la banda |
/// |---|---|
/// | `encendida` | la banda vive a 44 px en vez de 88 |
/// | `puede` | cuál de los dos estados se pinta: «Diciéndote la ficha…» o «Conecta auriculares» |
/// | `diciendo` | si está sonando ahora mismo |
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaVoz {
    pub encendida: bool,
    /// **Puede** decir la ficha: hay voz para el idioma y el sonido no saldría por donde el cliente
    /// oye. Es una capacidad, no un instante — que alguien esté hablando ahora mismo no la cambia.
    pub puede: bool,
    pub diciendo: bool,
}

impl LaVoz {
    /// La voz apagada, que es como nace la app.
    pub const APAGADA: Self = Self { encendida: false, puede: false, diciendo: false };
}

/// Por qué la voz no dice la ficha. **Cinco, y ninguna se colapsa en «no se puede».**
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Impedimento {
    /// El modo solo audio está apagado. No es un fallo: es el estado normal.
    ModoApagado,
    /// El sonido saldría por los altavoces del Mac. **Es el único que la banda dibuja**, y lo
    /// dibuja con las palabras que el usuario aprobó: «Conecta auriculares · el cliente te oiría».
    TeOiriaElCliente,
    /// Este Mac no tiene ninguna voz para ese idioma.
    SinVozParaEseIdioma,
    /// Alguien está hablando **en la reunión**. La app no habla encima de nadie: ni del cliente, ni
    /// del consultor.
    AlguienEstaHablando,
    /// Ya está diciendo otra ficha. Encolar la siguiente dejaría a la app hablando sola dos veces
    /// seguidas mientras la reunión sigue.
    YaEstaDiciendoOtra,
}

impl Impedimento {
    /// Para el log, que es el único sitio donde estas frases se escriben. **No cruza a la
    /// interfaz**: allí el copy sale de `src/i18n/`, contra la maqueta.
    pub fn como_frase(&self) -> &'static str {
        match self {
            Impedimento::ModoApagado => "el modo solo audio está apagado",
            Impedimento::TeOiriaElCliente => "el sonido saldría por los altavoces y el cliente te oiría",
            Impedimento::SinVozParaEseIdioma => "este Mac no tiene voz para ese idioma",
            Impedimento::AlguienEstaHablando => "alguien está hablando en la reunión",
            Impedimento::YaEstaDiciendoOtra => "ya está diciendo otra ficha",
        }
    }
}

/// Lo que hace falta saber para decidir si se puede hablar. Todo lo que la app sabe del instante,
/// y nada más — ni el texto, ni la ficha, ni quién la pidió.
#[derive(Clone, Debug)]
pub struct Momento<'a> {
    pub modo_encendido: bool,
    /// Por dónde sale el sonido de este Mac.
    pub salida: &'a Salida,
    /// ¿Hay voz instalada para el idioma en que se va a leer?
    pub hay_voz: bool,
    /// ¿Habla alguien **en la reunión** ahora mismo? Cualquiera de las dos pistas.
    pub alguien_hablando: bool,
    /// ¿Está la app diciendo ya otra cosa?
    pub ya_diciendo: bool,
}

/// **EL CANDADO DE LOS AURICULARES** — y las otras cuatro razones para callarse.
///
/// El orden de las comprobaciones es el orden de la gravedad, porque el primero que falle es el que
/// se registra: primero lo que rompería la promesa del producto, después lo que solo sería
/// inoportuno.
///
/// ### La decisión que hubo que tomar, y por qué está escrita aquí
///
/// `Salida::puede_haber_eco()` tiene **tres** respuestas, no dos: `Some(true)` con los altavoces
/// internos, `Some(false)` con auriculares por el conector, y **`None` con cualquier dispositivo
/// externo** — porque desde Core Audio un USB o un Bluetooth puede ser un casco o un altavoz de
/// mesa y **no se distinguen**. Eso incluye los AirPods, que es como la mayoría de la gente hace
/// una videollamada.
///
/// Las dos salidas eran malas: negarse con `None` deja C15 inservible para el caso normal (una
/// funcionalidad que nunca corre es peor que una limitación declarada); hablar con `None` acepta que
/// algún día el sonido salga por un altavoz de mesa.
///
/// **Se habla.** Tres razones, en orden de peso:
///
/// 1. **La app ya trazó esta línea y el usuario la aprobó.** El aviso de eco de la pantalla de
///    Sesión se enciende SOLO con los altavoces internos (`Sesion.tsx`: `salida.salida ===
///    "altavoces"`), y esa pantalla pasó la mirada 13. Trazarla distinta aquí sería que la misma
///    app respondiera dos cosas a la misma pregunta.
/// 2. **Un dispositivo externo en una videollamada es un casco**, porque es lo que el consultor se
///    pone para no oírse. No es una certeza, y por eso no se escribe como tal en ningún sitio.
/// 3. **Quien lo enciende es el usuario, con una tecla, sabiendo por dónde le suena el Mac.** El
///    modo no se enciende solo nunca.
///
/// **Y lo que falta para cerrarlo bien está nombrado:** la frase precisa —«no sé si "AirPods Pro"
/// es un casco»— necesita `Salida.nombre`, que es uno de los campos del contrato sin lector, con su
/// sitio pedido para la mirada 17. Hasta entonces el log lo dice con el nombre del dispositivo y el
/// manual lo declara como limitación. Cambiar la decisión es **una línea de esta función**.
pub fn cabe_decirla(m: &Momento) -> Result<(), Impedimento> {
    if !m.modo_encendido {
        return Err(Impedimento::ModoApagado);
    }
    // El único caso en que la app SABE que el cliente oiría. Los demás se deciden arriba.
    if m.salida.puede_haber_eco() == Some(true) {
        return Err(Impedimento::TeOiriaElCliente);
    }
    if !m.hay_voz {
        return Err(Impedimento::SinVozParaEseIdioma);
    }
    if m.alguien_hablando {
        return Err(Impedimento::AlguienEstaHablando);
    }
    if m.ya_diciendo {
        return Err(Impedimento::YaEstaDiciendoOtra);
    }
    Ok(())
}

/// **Lo que la banda puede afirmar sin mirar el instante.** `puede` es la capacidad, no la
/// oportunidad: si el cliente está hablando la app se calla, y la banda sigue diciendo que el modo
/// funciona — porque funciona. Pintar «Conecta auriculares» cada vez que alguien abre la boca haría
/// parpadear la banda entre dos estados durante toda la reunión.
pub fn puede_en_principio(salida: &Salida, hay_voz: bool) -> bool {
    salida.puede_haber_eco() != Some(true) && hay_voz
}

/// **LO QUE SE LEE DE UNA FICHA, y en qué orden.**
///
/// Titular, línea y fuente — lo mismo que la banda pinta, y en el mismo orden, porque el usuario
/// pidió el modo para **no** tener que mirar: si la voz dijera otra cosa que la pantalla, tendría
/// que comprobar cuál de las dos va bien.
///
/// **La fuente se lee al final y siempre.** Es la mitad del valor de una ficha: una frase sin saber
/// de dónde sale no es evidencia, es una sugerencia. Y es lo único que el usuario no puede
/// reconstruir de oído.
///
/// Los puntos entre las tres partes no son decorativos: `AVSpeechSynthesizer` hace una pausa en
/// cada uno, y sin ellos las tres frases salen pegadas y hay que oírlas dos veces.
pub fn a_voz(titular: &str, linea: &str, fuente: &str) -> String {
    [titular.trim(), linea.trim(), fuente.trim()]
        .iter()
        .filter(|t| !t.is_empty())
        .map(|t| t.trim_end_matches('.'))
        .collect::<Vec<_>>()
        .join(". ")
        + "."
}

/// La voz de este Mac, o la muda con su razón.
pub fn voz() -> Box<dyn Voz> {
    apple::voz()
}

#[cfg(test)]
mod pruebas {
    use super::*;

    fn momento<'a>(salida: &'a Salida) -> Momento<'a> {
        Momento {
            modo_encendido: true,
            salida,
            hay_voz: true,
            alguien_hablando: false,
            ya_diciendo: false,
        }
    }

    /// El camino feliz, que es el que tiene que existir para que los otros signifiquen algo.
    #[test]
    fn con_auriculares_y_nadie_hablando_la_ficha_se_dice() {
        assert_eq!(cabe_decirla(&momento(&Salida::Auriculares)), Ok(()));
    }

    /// **El candado.** Es la razón de ser del estado «sin auriculares» de la banda.
    #[test]
    fn con_los_altavoces_internos_se_calla_y_dice_por_que() {
        assert_eq!(
            cabe_decirla(&momento(&Salida::Altavoces)),
            Err(Impedimento::TeOiriaElCliente)
        );
    }

    /// La decisión declarada arriba, escrita como test para que cambiarla cueste cambiar esto —
    /// que es lo que obliga a leer el por qué antes de moverla.
    #[test]
    fn con_un_dispositivo_externo_se_habla_igual_que_sesion_no_avisa_del_eco() {
        let airpods = Salida::Otra { nombre: "AirPods Pro".into() };
        assert_eq!(cabe_decirla(&momento(&airpods)), Ok(()));
        // Y la misma respuesta que da la pantalla de Sesión al mismo dispositivo: ningún aviso.
        assert_ne!(airpods.puede_haber_eco(), Some(true));
    }

    /// Sin saber por dónde suena el Mac tampoco hay aviso: es el mismo `None`, y la coherencia con
    /// Sesión vale también aquí.
    #[test]
    fn sin_saber_por_donde_suena_se_habla_y_el_log_lo_dice() {
        let a_ciegas = Salida::NoSeSabe {
            motivo: crate::capture::PorQueNoSeSabe::SinFuente,
            nombre: Some("Altavoz USB".into()),
        };
        assert_eq!(cabe_decirla(&momento(&a_ciegas)), Ok(()));
    }

    #[test]
    fn el_modo_apagado_no_es_un_fallo_pero_tiene_nombre() {
        let mut m = momento(&Salida::Auriculares);
        m.modo_encendido = false;
        assert_eq!(cabe_decirla(&m), Err(Impedimento::ModoApagado));
    }

    #[test]
    fn sin_voz_para_el_idioma_no_se_inventa_otra() {
        let mut m = momento(&Salida::Auriculares);
        m.hay_voz = false;
        assert_eq!(cabe_decirla(&m), Err(Impedimento::SinVozParaEseIdioma));
    }

    /// **La app no habla encima de nadie.** Es la segunda mitad de la petición del usuario: «que no
    /// me interrumpa». Hablar sobre el cliente es peor que no hablar.
    #[test]
    fn mientras_alguien_habla_en_la_reunion_la_app_se_calla() {
        let mut m = momento(&Salida::Auriculares);
        m.alguien_hablando = true;
        assert_eq!(cabe_decirla(&m), Err(Impedimento::AlguienEstaHablando));
    }

    /// **Y no se oye a sí misma por el otro lado**: dos fichas encoladas dejarían a la app hablando
    /// sola mientras la reunión sigue. El tap del sistema ya excluye nuestro propio proceso desde el
    /// sprint 001; esto es el candado del lado de la decisión.
    #[test]
    fn no_encola_una_segunda_ficha_encima_de_la_que_esta_diciendo() {
        let mut m = momento(&Salida::Auriculares);
        m.ya_diciendo = true;
        assert_eq!(cabe_decirla(&m), Err(Impedimento::YaEstaDiciendoOtra));
    }

    /// **El orden importa y se prueba.** Con los altavoces Y alguien hablando, lo que se registra
    /// es el candado — que es lo que el usuario tiene que arreglar; lo otro se arregla solo.
    #[test]
    fn el_impedimento_que_se_cuenta_es_el_mas_grave() {
        let mut m = momento(&Salida::Altavoces);
        m.alguien_hablando = true;
        m.ya_diciendo = true;
        assert_eq!(cabe_decirla(&m), Err(Impedimento::TeOiriaElCliente));
    }

    /// Los cinco impedimentos tienen frase, y ninguna está vacía: un motivo vacío en un log es un
    /// motivo que nadie va a poder seguir.
    #[test]
    fn los_cinco_impedimentos_se_pueden_escribir_en_el_log() {
        for i in [
            Impedimento::ModoApagado,
            Impedimento::TeOiriaElCliente,
            Impedimento::SinVozParaEseIdioma,
            Impedimento::AlguienEstaHablando,
            Impedimento::YaEstaDiciendoOtra,
        ] {
            assert!(!i.como_frase().is_empty(), "{i:?} no tiene frase");
        }
    }

    /// La capacidad NO parpadea con los turnos. Es lo que impide que la banda cambie de estado
    /// cada vez que alguien abre la boca.
    #[test]
    fn la_capacidad_no_depende_de_quien_este_hablando() {
        assert!(puede_en_principio(&Salida::Auriculares, true));
        assert!(!puede_en_principio(&Salida::Altavoces, true));
        assert!(!puede_en_principio(&Salida::Auriculares, false));
    }

    #[test]
    fn la_ficha_se_lee_titular_linea_y_fuente_en_ese_orden() {
        let dicho = a_voz(
            "Limpieza de datos: incluida, hasta tres fuentes",
            "Cubre perfilado y limpieza de ERP, POS y Excel de canal.",
            "propuesta · §3.2 Alcance",
        );
        assert_eq!(
            dicho,
            "Limpieza de datos: incluida, hasta tres fuentes. Cubre perfilado y limpieza de ERP, \
             POS y Excel de canal. propuesta · §3.2 Alcance."
        );
    }

    /// Una ficha sin sección no deja un hueco con dos puntos seguidos: el sintetizador los lee
    /// como una pausa larga y suena a avería.
    #[test]
    fn una_parte_vacia_no_deja_una_pausa_sola() {
        assert_eq!(a_voz("Titular", "", "propuesta"), "Titular. propuesta.");
        assert!(!a_voz("Titular", "", "propuesta").contains(".."));
    }

    /// La voz muda **siempre existe** y siempre dice por qué. Es la que corre en la CI.
    #[test]
    fn la_muda_no_finge_y_trae_su_motivo() {
        let m = Muda::por("en la CI no hay altavoz");
        assert!(!m.hay_para("es-ES"));
        assert!(!m.hablando());
        assert_eq!(m.decir("es-ES", "algo"), Err("en la CI no hay altavoz".into()));
        m.callar(); // no se rompe, y no hace nada
        assert!(!m.motivo().is_empty());
    }

    /// La voz de este Mac contesta algo enseñable, tenga o no altavoz debajo — igual que el motor
    /// de transcripción. Sin esto, la CI probaría solo la muda.
    #[test]
    fn siempre_hay_una_voz_y_siempre_tiene_nombre() {
        let v = voz();
        println!("[habla] voz de este Mac: «{}»", v.nombre());
        assert!(!v.nombre().is_empty());
        println!("[habla] ¿hay voz para es-ES? {}", v.hay_para("es-ES"));
        println!("[habla] ¿hay voz para en-US? {}", v.hay_para("en-US"));
        // Un idioma inventado no puede tener voz, y no puede romper nada.
        assert!(!v.hay_para("xx-ZZ"), "«xx-ZZ» no puede tener voz");
    }
}
