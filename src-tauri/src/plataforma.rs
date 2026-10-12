//! **Qué hace la app en este equipo, dicho por la app** (sprint 006, ADR 022).
//!
//! La lista «Hoy en Windows» del ADR 022 vive aquí como dato: Honestidad la pinta, el manual la repite y
//! `tests/unit/windows-honestidad-y-manual.test.ts` comprueba que los tres dicen lo mismo. Lo que una plataforma aún no
//! hace se dice con su motivo; no se finge (regla dura 6: la promesa, graduada por plataforma).
//!
//! En Windows también cruza lo que la transcripción sabe de este PC: qué modelo hay, lo que tardó en la calibración
//! y si se está bajando. En macOS no cruza nada de eso: el motor es el del sistema y su latencia la midió el ADR 006.

use serde::Serialize;

/// De qué sistema es esta app.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Sistema {
    Macos,
    Windows,
    Otro,
}

/// Cuándo llega una función a Windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cuando {
    /// Hoy, con lo que diga su frase (p. ej. «sin verificar»).
    Si,
    /// En el sprint 007.
    TodaviaNo,
    /// No está planeada por ahora.
    NoPorAhora,
}

/// Una función de la app y cuándo está en Windows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct Funcion {
    pub clave: &'static str,
    pub cuando: Cuando,
}

/// **Hoy en Windows**, en el orden del ADR 022. Una función nueva entra aquí, en el ADR, en Honestidad y en el manual
/// a la vez; el test de los cuatro lo exige.
pub const EN_WINDOWS: [Funcion; 15] = [
    Funcion { clave: "banda-protegida", cuando: Cuando::Si },
    Funcion { clave: "presencial", cuando: Cuando::Si },
    Funcion { clave: "corpus-y-ficha", cuando: Cuando::Si },
    Funcion { clave: "notas-cifradas", cuando: Cuando::Si },
    Funcion { clave: "videollamada", cuando: Cuando::TodaviaNo },
    Funcion { clave: "pantalla", cuando: Cuando::TodaviaNo },
    Funcion { clave: "voz-al-oido", cuando: Cuando::TodaviaNo },
    Funcion { clave: "acople", cuando: Cuando::TodaviaNo },
    Funcion { clave: "ensayo", cuando: Cuando::TodaviaNo },
    Funcion { clave: "sugerencia", cuando: Cuando::TodaviaNo },
    Funcion { clave: "radar", cuando: Cuando::TodaviaNo },
    Funcion { clave: "atajos", cuando: Cuando::TodaviaNo },
    Funcion { clave: "borrado-con-la-app-cerrada", cuando: Cuando::TodaviaNo },
    Funcion { clave: "desbloqueo", cuando: Cuando::TodaviaNo },
    Funcion { clave: "puerta-local", cuando: Cuando::NoPorAhora },
];

/// Lo que la transcripción sabe de este PC (solo Windows).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transcripcion {
    /// El modelo con el que se transcribiría ahora («large-v3-turbo», «small»), o `None` si no hay ninguno entero.
    pub modelo: Option<&'static str>,
    /// Lo que ese modelo tardó en los 10 s de calibración, en ms; `None` si todavía no se midió en esta sesión.
    pub latencia_ms: Option<u32>,
    /// El techo del fin de turno a la ficha (4 s): si la latencia lo pasa, el modelo es lento para este PC.
    pub techo_ms: u32,
    /// Se está bajando un modelo ahora mismo.
    pub bajando: bool,
}

/// Lo que cruza a Honestidad.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plataforma {
    pub sistema: Sistema,
    /// En Windows, la lista del ADR 022; en macOS, vacía: ahí la app hace lo que dicen el manual y Honestidad.
    pub funciones: Vec<Funcion>,
    pub transcripcion: Option<Transcripcion>,
}

impl Plataforma {
    /// La de un sistema dado, con lo que su transcripción sepa.
    pub fn de(sistema: Sistema, transcripcion: Option<Transcripcion>) -> Self {
        let funciones = if sistema == Sistema::Windows { EN_WINDOWS.to_vec() } else { Vec::new() };
        Self { sistema, funciones, transcripcion }
    }
}

/// La de este equipo.
pub fn de_este_equipo() -> Plataforma {
    if cfg!(target_os = "macos") {
        Plataforma::de(Sistema::Macos, None)
    } else if cfg!(windows) {
        Plataforma::de(Sistema::Windows, Some(la_transcripcion()))
    } else {
        Plataforma::de(Sistema::Otro, None)
    }
}

/// Lo que la transcripción de whisper sabe ahora.
fn la_transcripcion() -> Transcripcion {
    let elegido = crate::stt::whisper::el_modelo().map(|(m, _)| m);
    Transcripcion {
        modelo: elegido.map(|m| m.nombre),
        latencia_ms: elegido.and_then(|m| crate::stt::whisper::latencia_ms(&m)),
        techo_ms: crate::stt::calibracion::TECHO_MS,
        bajando: crate::modelo_whisper::bajando(),
    }
}

#[cfg(test)]
mod pruebas {
    use super::*;

    /// **La lista es la del ADR 022**, clave por clave y en su orden: si una cambia en un sitio, cambia en los dos.
    #[test]
    fn la_lista_es_la_del_adr() {
        let adr = include_str!("../../decisions/022-windows-detras-de-la-misma-frontera.md");
        let tabla: Vec<&str> = adr
            .lines()
            .skip_while(|l| !l.starts_with("## Hoy en Windows"))
            .skip(1)
            .take_while(|l| !l.starts_with("## "))
            .filter_map(|l| l.strip_prefix("| `")?.split('`').next())
            .collect();
        let aqui: Vec<&str> = EN_WINDOWS.iter().map(|f| f.clave).collect();
        assert_eq!(aqui, tabla);
        for f in EN_WINDOWS {
            let fila = adr.lines().find(|l| l.starts_with(&format!("| `{}`", f.clave))).unwrap();
            let dice = fila.split('|').nth(2).unwrap().trim();
            let esperado = match f.cuando {
                Cuando::Si => "sí",
                Cuando::TodaviaNo => "todavía no (S7)",
                Cuando::NoPorAhora => "no en Windows por ahora",
            };
            assert!(dice.starts_with(esperado), "{}: el ADR dice «{dice}» y aquí es {:?}", f.clave, f.cuando);
        }
    }

    /// En macOS no cruza ninguna lista de Windows ni nada de whisper; en Windows, las 15 y la transcripción.
    #[test]
    fn cada_sistema_cruza_lo_suyo() {
        let mac = Plataforma::de(Sistema::Macos, None);
        assert!(mac.funciones.is_empty());
        let win = Plataforma::de(Sistema::Windows, Some(la_transcripcion()));
        assert_eq!(win.funciones.len(), 15);
        assert_eq!(win.transcripcion.unwrap().techo_ms, 4_000);
        let aqui = de_este_equipo();
        assert_eq!(aqui.sistema == Sistema::Windows, cfg!(windows));
    }
}
