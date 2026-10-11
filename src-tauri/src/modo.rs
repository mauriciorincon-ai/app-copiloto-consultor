//! EL MODO DE LA SESIÓN — reunión, solo notas o presencial (C11, ADR 017 §5; C19, ADR 020).
//!
//! Una sola pregunta, y pura para poder demostrarla en rojo: **¿qué abre este modo?** La captura es
//! todo lo que oye o mira la sesión: micrófono, audio del sistema, transcripción, lectura de pantalla
//! (la automática y `⌃⌥L`) y el radar ámbar, que lee la ventana de la reunión. `empezar` (en `lib.rs`)
//! la obedece antes de arrancar nada, y un test de su fuente vigila que la obedezca.
//!
//! - En **reunión** se abre todo: las dos pistas, la pantalla, el acople, tus turnos si lo pediste y las
//!   propuestas.
//! - En **solo notas** no se abre nada de eso, ni una vez: la NDA de tu cliente puede prohibir «grabar o
//!   transcribir por cualquier medio», o puedes decidirlo tú. Siguen tus notas, fijar, `⌃⌥A` (que busca
//!   con la última línea de tu nota), el radar coral y `⌥⎋`.
//! - En **presencial** (sprint 005) se abre **una** pista, la sala, por el micrófono. Ni el audio del
//!   sistema, ni la pantalla —ni con ella `⌃⌥L` ni el radar ámbar—, ni el acople: no hay ventana de
//!   reunión en este Mac que acoplar. Y ni tus turnos ni las propuestas: en la sala no se sabe quién habló.
//!
//! **La pantalla se abre con un permiso que solo crea [`que_abre`]** ([`PermisoDePantalla`]): arrancarla
//! en presencial no es un `if` que alguien pueda olvidar, es una llamada que no tiene con qué hacerse.

/// No cruza a la pantalla: el evento `modo` es una señal sin dato (auditoría del S3, B14).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Modo {
    Normal,
    SoloNotas,
    /// La sala, por el micrófono del Mac (ADR 020).
    Presencial,
}

/// Qué pistas se abren.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuePistas {
    /// Las dos de una reunión: el micrófono (tú) y el audio del sistema (el cliente).
    Reunion,
    /// Una: la sala, por el micrófono.
    Sala,
}

/// **El permiso de abrir la pantalla.** Su campo es privado: fuera de este módulo no se puede fabricar,
/// y `arrancar_la_pantalla` no arranca sin uno. Solo [`que_abre`] lo da, y solo en reunión.
#[derive(Debug, PartialEq, Eq)]
pub struct PermisoDePantalla(());

/// Lo que abre un modo. Lo lee `empezar`, y lo que no está aquí no se abre.
#[derive(Debug, PartialEq, Eq)]
pub struct Apertura {
    /// Las pistas que se abren. `None` en solo notas.
    pub pistas: Option<QuePistas>,
    /// La lectura de pantalla, y con ella `⌃⌥L` y el radar ámbar. Solo en reunión.
    pub pantalla: Option<PermisoDePantalla>,
    /// Si se puede acoplar la ventana de la reunión (o la que esté al frente, abajo).
    pub acople: bool,
    /// Si «Conservar mis turnos» puede guardar algo. La preferencia no se toca: vuelve a valer en la
    /// siguiente reunión.
    pub tus_turnos: bool,
    /// Si las reglas pueden proponer notas.
    pub propuestas: bool,
}

/// Qué abre cada modo.
pub fn que_abre(modo: Modo) -> Apertura {
    match modo {
        Modo::Normal => Apertura {
            pistas: Some(QuePistas::Reunion),
            pantalla: Some(PermisoDePantalla(())),
            acople: true,
            tus_turnos: true,
            propuestas: true,
        },
        Modo::SoloNotas => Apertura {
            pistas: None,
            pantalla: None,
            // En solo notas la reunión sigue en tu pantalla y la banda se acopla como siempre (ADR 017 §5).
            acople: true,
            tus_turnos: false,
            propuestas: false,
        },
        Modo::Presencial => Apertura {
            pistas: Some(QuePistas::Sala),
            pantalla: None,
            acople: false,
            tus_turnos: false,
            propuestas: false,
        },
    }
}

/// ¿Abre este modo la captura? Lo que preguntaba `empezar` hasta el sprint 005.
pub fn abre_la_captura(modo: Modo) -> bool {
    que_abre(modo).pistas.is_some()
}

/// Por qué un modo no puede empezar.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NoEmpieza {
    /// La NDA de este cliente prohíbe grabar o transcribir por cualquier medio, y este modo transcribe.
    LaNdaLoProhibe,
}

/// **La NDA, comprobada en Rust** (ADR 020 §1). Hasta el sprint 005 solo la miraba la pantalla de
/// Sesión (`Sesion.tsx`): un comando llamado por otro camino habría escuchado igual. Con la NDA que lo
/// prohíbe solo se empieza en solo notas.
pub fn puede_empezar(modo: Modo, nda: Option<crate::jurisdiccion::Nda>) -> Result<(), NoEmpieza> {
    let prohibe = nda == Some(crate::jurisdiccion::Nda::LoProhibe);
    if prohibe && abre_la_captura(modo) {
        return Err(NoEmpieza::LaNdaLoProhibe);
    }
    Ok(())
}

#[cfg(test)]
mod pruebas {
    use super::*;
    use crate::jurisdiccion::Nda;

    /// **Solo notas no abre la captura.** ¿Puede fallar? Sí: con `SoloNotas => true` es rojo
    /// (bitácora), y `lib.rs` arrancaría micrófono, sistema y pantalla en una reunión cuya NDA lo
    /// prohíbe.
    #[test]
    fn solo_notas_no_abre_la_captura() {
        assert!(!abre_la_captura(Modo::SoloNotas));
        assert!(abre_la_captura(Modo::Normal));
        assert!(abre_la_captura(Modo::Presencial));
    }

    /// **Lo que presencial NO abre, una cosa por aserción** (ADR 020 §1): ni el audio del sistema (abre
    /// la sala, no la reunión), ni la pantalla, ni el acople, ni tus turnos, ni las propuestas.
    ///
    /// ¿Puede fallar? Sí: con `pantalla: Some(PermisoDePantalla(()))` en presencial es rojo (bitácora del
    /// sprint 005, fase 1), y `empezar` arrancaría la lectura de una pantalla que en la sala nadie comparte.
    #[test]
    fn presencial_abre_la_sala_y_nada_mas() {
        let a = que_abre(Modo::Presencial);
        assert_eq!(a.pistas, Some(QuePistas::Sala), "presencial abre la sala, no las dos pistas de una reunión");
        assert!(a.pantalla.is_none(), "presencial no lee la pantalla (ni ⌃⌥L, ni el radar ámbar)");
        assert!(!a.acople, "presencial no acopla ninguna ventana ni escribe AXPosition");
        assert!(!a.tus_turnos, "presencial no conserva tus turnos: la sala no tiene dueño");
        assert!(!a.propuestas, "presencial no propone notas: no se sabe quién lo dijo");
    }

    /// La reunión abre todo, como hasta el sprint 005: el modo nuevo no le quitó nada.
    #[test]
    fn la_reunion_abre_lo_de_siempre() {
        let a = que_abre(Modo::Normal);
        assert_eq!(a.pistas, Some(QuePistas::Reunion));
        assert!(a.pantalla.is_some() && a.acople && a.tus_turnos && a.propuestas);
        let n = que_abre(Modo::SoloNotas);
        assert!(n.pistas.is_none() && n.pantalla.is_none() && !n.tus_turnos && !n.propuestas);
    }

    /// **Con la NDA que lo prohíbe solo se empieza en solo notas** — también en presencial, que transcribe.
    ///
    /// ¿Puede fallar? Sí: sin la comprobación de `abre_la_captura`, presencial empezaría con la NDA que lo
    /// prohíbe (bitácora del sprint 005, fase 1).
    #[test]
    fn la_nda_que_lo_prohibe_solo_deja_solo_notas() {
        let prohibe = Some(Nda::LoProhibe);
        assert_eq!(puede_empezar(Modo::Presencial, prohibe), Err(NoEmpieza::LaNdaLoProhibe));
        assert_eq!(puede_empezar(Modo::Normal, prohibe), Err(NoEmpieza::LaNdaLoProhibe));
        assert_eq!(puede_empezar(Modo::SoloNotas, prohibe), Ok(()));
        for m in [Modo::Normal, Modo::SoloNotas, Modo::Presencial] {
            assert_eq!(puede_empezar(m, Some(Nda::NoLoProhibe)), Ok(()));
            assert_eq!(puede_empezar(m, None), Ok(()), "sin respuesta no se bloquea: Sesión pregunta");
        }
    }
}
