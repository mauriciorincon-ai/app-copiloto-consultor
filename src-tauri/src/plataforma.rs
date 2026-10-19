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

/// Si una función está en Windows. Desde el sprint 008 («Windows completo») no queda ninguna «todavía no»: o está,
/// con lo que diga su frase, o no está planeada. Desde el sprint 012 estaban las 15; en el sprint 013 la sugerencia
/// redactada pasó a `NoPorAhora` (ADR 023, enmienda 1) y Honestidad la pinta con su motivo.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Cuando {
    /// Hoy, con lo que diga su frase (p. ej. «sin verificar»).
    Si,
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
    Funcion { clave: "videollamada", cuando: Cuando::Si },
    Funcion { clave: "pantalla", cuando: Cuando::Si },
    Funcion { clave: "voz-al-oido", cuando: Cuando::Si },
    Funcion { clave: "acople", cuando: Cuando::Si },
    Funcion { clave: "ensayo", cuando: Cuando::Si },
    // Sprint 013 (ADR 023, enmienda 1): el API salió de Windows y el modelo local que se midió abre una conexión de
    // telemetría a Microsoft. Sin modelo que no salga a la red, la sugerencia redactada queda «no por ahora».
    Funcion { clave: "sugerencia", cuando: Cuando::NoPorAhora },
    Funcion { clave: "radar", cuando: Cuando::Si },
    Funcion { clave: "atajos", cuando: Cuando::Si },
    Funcion { clave: "borrado-con-la-app-cerrada", cuando: Cuando::Si },
    Funcion { clave: "desbloqueo", cuando: Cuando::Si },
    // Sprint 012 (ADR 022 enmienda 4): un named pipe solo del usuario, con la llave en el Administrador de credenciales.
    Funcion { clave: "puerta-local", cuando: Cuando::Si },
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
    /// Los turnos tuyos que se quedaron sin transcribir en la última escucha porque el equipo no daba abasto (sprint
    /// 007: con dos pistas, el cliente va primero). Solo la cuenta.
    pub descartados: u32,
    /// Una línea por modelo, del mayor al menor (sprint 009): qué tardó cada uno EN ESTE PC, cuál se usa y cuál baja
    /// «Instalar». Honestidad las enseña todas, no solo la del modelo en uso.
    pub modelos: Vec<ModeloEnEsteEquipo>,
}

/// Un modelo de whisper tal como está en este PC.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModeloEnEsteEquipo {
    /// Cómo lo enseña la app («large-v3-turbo», «small»).
    pub nombre: &'static str,
    /// ¿Está entero en este PC?
    pub bajado: bool,
    /// Lo que tardó tu pista, el turno de 10 s de calibración, en ms; `None`: sin medir (o sin bajar). Es la cifra que
    /// decide el modelo (ADR 022, enmienda 3).
    pub latencia_ms: Option<u32>,
    /// Lo que tardó la pista del cliente (la pregunta del kit, unos 6 s), medida detrás de la tuya (sprint 010); `None`:
    /// sin medir (o sin bajar). Honestidad la enseña; la regla no la usa.
    pub latencia_cliente_ms: Option<u32>,
    /// ¿Es con el que se transcribe ahora?
    pub en_uso: bool,
    /// ¿Es el que baja «Instalar» en este equipo? En un PC ARM64 sale de la regla (ADR 022, enmienda 3).
    pub por_defecto: bool,
}

/// Lo que cruza a Honestidad.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plataforma {
    pub sistema: Sistema,
    /// En Windows, la lista del ADR 022; en macOS, vacía: ahí la app hace lo que dicen el manual y Honestidad.
    pub funciones: Vec<Funcion>,
    pub transcripcion: Option<Transcripcion>,
    /// Las teclas de este sistema y si el sistema las dejó registrar (sprint 007): la banda y los textos enseñan las
    /// de Windows en Windows, y Honestidad dice cuál tiene otra app.
    pub atajos: Vec<crate::atajos::Atajo>,
}

impl Plataforma {
    /// La de un sistema dado, con lo que su transcripción sepa.
    pub fn de(sistema: Sistema, transcripcion: Option<Transcripcion>) -> Self {
        let funciones = if sistema == Sistema::Windows { EN_WINDOWS.to_vec() } else { Vec::new() };
        let atajos = crate::atajos::para_el_contrato(sistema == Sistema::Windows);
        Self { sistema, funciones, transcripcion, atajos }
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
    let por_defecto = crate::stt::whisper::por_defecto();
    let modelos = [crate::modelo_whisper::GRANDE, crate::modelo_whisper::PEQUENO]
        .iter()
        .map(|m| {
            let bajado = crate::modelo_whisper::listo(m).is_some();
            ModeloEnEsteEquipo {
                nombre: m.nombre,
                bajado,
                latencia_ms: bajado.then(|| crate::stt::whisper::latencia_ms(m)).flatten(),
                latencia_cliente_ms: bajado.then(|| crate::stt::whisper::latencia_del_cliente_ms(m)).flatten(),
                en_uso: elegido == Some(*m),
                por_defecto: *m == por_defecto,
            }
        })
        .collect();
    Transcripcion {
        modelo: elegido.map(|m| m.nombre),
        latencia_ms: elegido.and_then(|m| crate::stt::whisper::latencia_ms(&m)),
        techo_ms: crate::stt::calibracion::TECHO_MS,
        bajando: crate::modelo_whisper::bajando(),
        descartados: crate::escucha::descartados(),
        modelos,
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
                Cuando::NoPorAhora => "no en Windows por ahora",
            };
            assert!(dice.starts_with(esperado), "{}: el ADR dice «{dice}» y aquí es {:?}", f.clave, f.cuando);
        }
    }

    /// En macOS no cruza ninguna lista de Windows ni nada de whisper; en Windows, las 15 (14 «sí» y 1 «no por ahora») y la transcripción. Desde
    /// el sprint 013 la única «no por ahora» es la sugerencia redactada: las otras 14 están.
    #[test]
    fn cada_sistema_cruza_lo_suyo() {
        let mac = Plataforma::de(Sistema::Macos, None);
        assert!(mac.funciones.is_empty());
        let win = Plataforma::de(Sistema::Windows, Some(la_transcripcion()));
        assert_eq!(win.funciones.len(), 15);
        assert_eq!(win.funciones.iter().filter(|f| f.cuando == Cuando::Si).count(), 14);
        let no_por_ahora: Vec<&str> = win.funciones.iter().filter(|f| f.cuando == Cuando::NoPorAhora).map(|f| f.clave).collect();
        assert_eq!(no_por_ahora, ["sugerencia"], "la única «no en Windows por ahora» es la sugerencia redactada");
        assert_eq!(win.transcripcion.unwrap().techo_ms, 4_000);
        let aqui = de_este_equipo();
        assert_eq!(aqui.sistema == Sistema::Windows, cfg!(windows));
    }
}
