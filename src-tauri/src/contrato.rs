//! EL CONTRATO CON LA INTERFAZ — lo que Rust emite, escrito por Rust.
//!
//! **Por qué existe.** El contrato entre la parte nativa y el webview se escribía dos veces a
//! mano: una en `#[derive(Serialize)]` y otra en los tipos de `src/*.ts`. Nadie comparaba las dos
//! copias, y el sprint 001 terminó su construcción con la ficha automática **sin llegar nunca a la
//! banda**: `escucha::Novedad` viaja etiquetada por dentro (`{"que":"aparece", …}`) y el webview
//! leía `{"Aparece": {…}}`, así que el campo era `undefined` en cada evento. Ni 87 tests unitarios
//! ni 66 e2e podían verlo, porque todos corren fuera de Tauri, donde esa suscripción no se monta.
//! Lo encontró la auditoría del sprint (hallazgo C1), leyendo los dos lados a la vez.
//!
//! **Cómo funciona.** Aquí se construye UNA muestra de cada cosa que cruza el puente y se
//! serializa **con el mismo serde que corre en producción**. De ahí sale
//! `src/contrato.generado.ts`: un archivo de TypeScript donde cada muestra lleva el valor que Rust
//! escribe de verdad y **el tipo que la interfaz declara**. Eso enciende dos gates:
//!
//! 1. **`cargo test`** falla si el archivo generado no es el que Rust escribiría hoy — el momento
//!    exacto en que alguien renombra un campo. Lo regenera `ACTUALIZA_CONTRATO=1 cargo test`.
//! 2. **`pnpm typecheck`** falla si ese valor no encaja en el tipo declarado. Son literales, así
//!    que TypeScript comprueba las tres direcciones: campo que falta, campo de más y campo con
//!    otro tipo.
//!
//! **¿Puede fallar?** Con el estado que tenía el repo al cerrar la construcción, el gate 2 estaba
//! en **rojo** — es la demo de la regla 15 y está registrada en la bitácora.
//!
//! **Lo que este gate NO cubre, dicho aquí para que no se lea como más de lo que es:** que alguien
//! *lea* los campos. Un campo puede encajar perfectamente y no tener un solo consumidor —la
//! auditoría contó diecisiete así. Eso es otra comprobación y vive en la deuda del sprint.

use serde_json::Value;

/// Una cosa que cruza el puente: cómo se llamará su constante, qué tipo la declara en TypeScript,
/// de qué módulo sale ese tipo, y el JSON que Rust produce de verdad.
pub struct Muestra {
    pub nombre: &'static str,
    pub tipo: &'static str,
    pub modulo: &'static str,
    pub valor: Value,
}

fn m<T: serde::Serialize>(
    nombre: &'static str,
    tipo: &'static str,
    modulo: &'static str,
    valor: &T,
) -> Muestra {
    Muestra {
        nombre,
        tipo,
        modulo,
        valor: serde_json::to_value(valor).expect("una muestra del contrato no se pudo serializar"),
    }
}

/// Todo lo que el webview recibe de lo nativo, con una muestra de cada forma.
///
/// **Los datos son sintéticos** («Páramo Azul», el mismo cliente inventado de la maqueta): este
/// archivo se versiona, y la regla 5 de la casa no admite datos reales en el repo.
pub fn muestras() -> Vec<Muestra> {
    use crate::capture::nativo::Salida;
    use crate::capture::Pista;
    use crate::corpus::{EstadoDelCorpus, PorUnidad};
    use crate::disparo::Motivo;
    use crate::escucha::{EstadoDeEscucha, EstadoDePista, Novedad};
    use crate::ficha::{Acumulada, Aparicion, Cercana, Ficha, Fuente, Respuesta};
    use crate::permisos::{Estado, Permisos};
    use crate::sesion::{Proteccion, Reunion};
    use crate::stt::{Disponibilidad, Turno};

    let turno = || Turno {
        pista: Pista::Sistema,
        desde_ms: 5_400,
        hasta_ms: 8_000,
        texto: "¿Y la limpieza de datos está dentro del alcance?".into(),
        hora: "14:02".into(),
        eco: false,
    };

    let ficha = || {
        Respuesta::Ficha(Box::new(Ficha {
            titular: "Limpieza de datos: incluida, hasta tres fuentes".into(),
            linea: "Cubre perfilado y limpieza de ERP, POS y Excel de canal.".into(),
            linea_larga: "Cubre perfilado y limpieza de ERP, POS y Excel de canal; una cuarta \
                          fuente se cotiza aparte."
                .into(),
            fuente: Fuente {
                documento: "Páramo Azul · Propuesta".into(),
                seccion: Some("§3.2 Alcance".into()),
                unidad: Some(crate::corpus::Unidad::Propuesta),
                conjeturada: false,
            },
            acumuladas: vec![Acumulada {
                unidad: Some(crate::corpus::Unidad::Caso),
                texto: "Sur del Valle: cuatro fuentes en 9 semanas".into(),
            }],
        }))
    };

    let sin_resultado = || Respuesta::SinResultado {
        buscado: "certificacion iso 27001".into(),
        cercanas: vec![Cercana {
            unidad: Some(crate::corpus::Unidad::Marco),
            texto: "§5.1 Seguridad y manejo de datos".into(),
        }],
        maniobra: "credencial".into(),
    };

    let aparicion = |respuesta: Respuesta, motivo: Motivo| Aparicion {
        respuesta,
        motivo,
        ms: 1_240,
        hora: "14:02".into(),
    };

    let pista_abierta = || EstadoDePista {
        abierta: true,
        motivo: None,
        bytes: 1_920_000,
        segundos: 30.0,
        muestras_recibidas: 480_000,
        hablando: false,
    };

    vec![
        // ---- el evento «escucha»: las cinco formas de una novedad -------------------------
        m("NOVEDAD_EMPIEZA", "Novedad", "./ficha", &Novedad::Empieza { pista: Pista::Microfono }),
        m("NOVEDAD_TURNO", "Novedad", "./ficha", &Novedad::Turno(turno())),
        m(
            "NOVEDAD_SIN_TEXTO",
            "Novedad",
            "./ficha",
            &Novedad::SinTexto {
                pista: Pista::Sistema,
                desde_ms: 5_400,
                hasta_ms: 8_000,
                motivo: "el motor no reconoció palabras en ese turno".into(),
            },
        ),
        m(
            "NOVEDAD_RUIDO",
            "Novedad",
            "./ficha",
            &Novedad::Ruido { pista: Pista::Microfono, duracion_ms: 140 },
        ),
        m(
            "NOVEDAD_APARECE_FICHA",
            "Novedad",
            "./ficha",
            &Novedad::Aparece(Box::new(aparicion(ficha(), Motivo::TerminoDelCorpus))),
        ),
        m(
            "NOVEDAD_APARECE_SIN_RESULTADO",
            "Novedad",
            "./ficha",
            &Novedad::Aparece(Box::new(aparicion(sin_resultado(), Motivo::Pregunta))),
        ),
        // ---- `pedir_ficha`, el camino del atajo ⌃⌥A ---------------------------------------
        m(
            "APARICION_DEL_ATAJO",
            "Aparicion",
            "./ficha",
            &aparicion(ficha(), Motivo::Atajo),
        ),
        // ---- `turnos_recientes`, el transcript de la banda --------------------------------
        m("TURNO_DEL_CLIENTE", "Turno", "./cuaderno", &turno()),
        // ---- la reunión, en sus tres formas ----------------------------------------------
        m("REUNION_NINGUNA", "Reunion", "./cuaderno", &Reunion::Ninguna),
        m(
            "REUNION_DETECTADA",
            "Reunion",
            "./cuaderno",
            &Reunion::Detectada {
                cliente: "Google Meet".into(),
                titulo: Some("Páramo Azul · seguimiento".into()),
                proteccion: Proteccion::Verificada,
            },
        ),
        m(
            "REUNION_NO_SE_PUEDE_SABER",
            "Reunion",
            "./cuaderno",
            &Reunion::NoSePuedeSaber { motivo: crate::sesion::PorQueNoSeVe::SinAccesibilidad },
        ),
        // ---- permisos, escucha, idioma, salida de audio -----------------------------------
        m(
            "PERMISOS",
            "Permisos",
            "./cuaderno",
            &Permisos {
                microfono: Estado::Concedido,
                audio: Estado::Concedido,
                pantalla: Estado::SinConceder,
                accesibilidad: Estado::NoSeSabe,
            },
        ),
        m(
            "ESTADO_DE_LA_ESCUCHA",
            "EstadoDeEscucha",
            "./cuaderno",
            &EstadoDeEscucha {
                escuchando: true,
                microfono: pista_abierta(),
                sistema: EstadoDePista {
                    abierta: false,
                    motivo: Some(crate::capture::PorQueNoAbrio::DispositivoOcupado),
                    bytes: 0,
                    segundos: 0.0,
                    muestras_recibidas: 0,
                    hablando: false,
                },
                turnos_en_memoria: 3,
                bytes_del_transcript: 2_048,
                motor: "apple-speechanalyzer",
            },
        ),
        m("DISPONIBILIDAD_LISTO", "Disponibilidad", "./cuaderno", &Disponibilidad::Listo),
        m(
            "DISPONIBILIDAD_SIN_MOTOR",
            "Disponibilidad",
            "./cuaderno",
            &Disponibilidad::SinMotor { motivo: crate::stt::PorQueNoHayMotor::SinTranscriptor },
        ),
        m("SALIDA_DE_AUDIO", "Salida", "./cuaderno", &Salida::Altavoces),
        m(
            "SALIDA_DE_AUDIO_OTRA",
            "Salida",
            "./cuaderno",
            &Salida::Otra { nombre: "AirPods Pro".into() },
        ),
        // El porqué cerrado cita el dispositivo: el nombre viaja aparte porque no se traduce.
        m(
            "SALIDA_DE_AUDIO_NO_SE_SABE",
            "Salida",
            "./cuaderno",
            &Salida::NoSeSabe {
                motivo: crate::capture::PorQueNoSeSabe::SinConexion,
                nombre: Some("Altavoz USB".into()),
            },
        ),
        // ---- el diccionario, en la pantalla de Idioma (mirada 17-bis) ------------------------
        m("ESTADO_DEL_DICCIONARIO", "EstadoDelDiccionario", "./cuaderno", &crate::EstadoDelDiccionario {
            terminos: 17,
            del_corpus: 12,
            en_tu_archivo: 5,
            ruta: "~/Library/Application Support/com.aiapps.copiloto-consultor/diccionario.yaml".into(),
        }),
        // ---- el corpus -------------------------------------------------------------------
        m(
            "ESTADO_DEL_CORPUS",
            "EstadoDelCorpus",
            "./cuaderno",
            &EstadoDelCorpus {
                carpeta: Some("~/Documentos/Consultoría".into()),
                documentos: 6,
                secciones: 31,
                por_unidad: vec![PorUnidad { unidad: "propuesta".into(), documentos: 2 }],
                sin_unidad: 1,
                ilegibles: 1,
                conjeturados: 2,
                donde_vive: Some("~/Library/…/Angel Ghost/corpus".into()),
                bytes_del_indice: 1_884_160,
            },
        ),
        // ---- el corte, que Honestidad enseña ANTES de que nadie pulse la tecla -------------
        // Entra en el sprint 002: le faltaba `rename_all` y llegaba como `bytes_en_red`, el mismo
        // defecto del C1 en el único payload que el gate no miraba.
        m("INFORME_DEL_CORTE", "InformeDelCorte", "./cuaderno", &crate::corte::Informe {
            piezas: crate::corte::TODAS
                .iter()
                .map(|p| (*p, crate::corte::suerte_en_este_sprint(*p)))
                .collect(),
            bytes_en_red: 0,
        }),
        // ---- la voz que sale, el modo solo audio (C15) del sprint 002 ---------------------
        // Las tres formas que la banda dibuja: apagada (banda a 88 px), diciendo la ficha, y
        // encendida sin poder hablar. La cuarta combinación —encendida, puede, y callada— es el
        // hueco que la mirada 16 no dibujó y que solo apareció al construir esto: está declarada en
        // la bitácora y pendiente de la mirada 16-bis.
        m("LA_VOZ_APAGADA", "LaVoz", "./cuaderno", &crate::habla::LaVoz::APAGADA),
        m("LA_VOZ_DICIENDO", "LaVoz", "./cuaderno", &crate::habla::LaVoz {
            encendida: true,
            puede: true,
            diciendo: true,
        }),
        m("LA_VOZ_SIN_AURICULARES", "LaVoz", "./cuaderno", &crate::habla::LaVoz {
            encendida: true,
            puede: false,
            diciendo: false,
        }),
        // ---- la lectura de pantalla (C8) del sprint 002 --------------------------------------
        // El evento «pantalla» y el comando `estado_de_la_pantalla`: la vista es la fila de Sesión y
        // los bytes en memoria, la de Honestidad. Las cinco vistas, porque las cinco se pintan.
        m("PANTALLA_LEYENDO", "EstadoDeLaPantalla", "./cuaderno", &crate::pantalla::EstadoDeLaPantalla {
            vista: crate::pantalla::Vista::Leyendo,
            bytes_en_memoria: 1_440_318,
        }),
        m("PANTALLA_APAGADA", "EstadoDeLaPantalla", "./cuaderno", &crate::pantalla::EstadoDeLaPantalla {
            vista: crate::pantalla::Vista::Apagada,
            bytes_en_memoria: 0,
        }),
        m("PANTALLA_SIN_PERMISO", "EstadoDeLaPantalla", "./cuaderno", &crate::pantalla::EstadoDeLaPantalla {
            vista: crate::pantalla::Vista::SinPermiso,
            bytes_en_memoria: 0,
        }),
        m("PANTALLA_ESPERANDO_LA_REUNION", "EstadoDeLaPantalla", "./cuaderno", &crate::pantalla::EstadoDeLaPantalla {
            vista: crate::pantalla::Vista::EsperandoLaReunion,
            bytes_en_memoria: 0,
        }),
        m("PANTALLA_NO_PUDO", "EstadoDeLaPantalla", "./cuaderno", &crate::pantalla::EstadoDeLaPantalla {
            vista: crate::pantalla::Vista::NoPudo,
            bytes_en_memoria: 0,
        }),
        // La ficha que pide la pantalla sola: el motivo nuevo cruza con el MISMO evento «escucha».
        m("NOVEDAD_APARECE_POR_PANTALLA", "Novedad", "./ficha", &Novedad::Aparece(Box::new(aparicion(
            ficha(),
            Motivo::Pantalla,
        )))),
        // `⌃⌥L` sin texto en la pantalla: la banda contesta igual, por el mismo evento.
        m("NOVEDAD_NADA_EN_PANTALLA", "Novedad", "./ficha", &Novedad::NadaEnPantalla {
            hora: "14:05".into(),
        }),
        // ---- el radar (C14) ----------------------------------------------------------------
        // El ámbar cruza por el evento «escucha», como las fichas: la banda lo pinta en su sitio.
        m("NOVEDAD_RADAR", "Novedad", "./ficha", &Novedad::Radar {
            grabando: true,
            bots: vec!["MinutaBot".into()],
            hora: "14:03".into(),
        }),
        // El coral cruza por su propio evento, «radar», y lo leen Sesión y la banda. Los nombres
        // son los de la maqueta, que son de ejemplo: el catálogo de verdad vive en `data/radar/`.
        m("EN_TU_MAC_VIGILADO", "EnTuMac", "./radar", &crate::radar::EnTuMac {
            programas: vec![
                crate::radar::Programa {
                    nombre: "ProctorLince".into(),
                    categoria: crate::radar::Categoria::Supervision,
                    nivel: crate::radar::Nivel::Invasivo,
                    ve: crate::radar::Bilingue {
                        es: "ve tu pantalla completa y tu cámara".into(),
                        en: "sees your full screen and your camera".into(),
                    },
                    alcance: crate::radar::Bilingue {
                        es: "Cámara, pantalla completa, apps abiertas; puede bloquear programas".into(),
                        en: "Camera, full screen, open apps; can block programs".into(),
                    },
                    fuente: "no cruza".into(),
                },
                crate::radar::Programa {
                    nombre: "MDM-Corp".into(),
                    categoria: crate::radar::Categoria::Mdm,
                    nivel: crate::radar::Nivel::Sabelo,
                    ve: crate::radar::Bilingue {
                        es: "puede instalar, borrar y leer la configuración".into(),
                        en: "can install, wipe and read configuration".into(),
                    },
                    alcance: crate::radar::Bilingue {
                        es: "Puede instalar, borrar y leer configuración. Normal en equipos de empresa".into(),
                        en: "Can install, wipe and read configuration. Normal on company machines".into(),
                    },
                    fuente: "no cruza".into(),
                },
            ],
            catalogo: crate::radar::CatalogoDelRadar { version: 1, fecha: "2026-09-26".into() },
        }),
        m("EN_TU_MAC_LIMPIO", "EnTuMac", "./radar", &crate::radar::EnTuMac {
            programas: vec![],
            catalogo: crate::radar::CatalogoDelRadar { version: 1, fecha: "2026-09-26".into() },
        }),
        // ---- el acople, que la banda dibuja en su cabecera --------------------------------
        m("ESTADO_DEL_ACOPLE", "EstadoDelAcople", "./acople", &crate::EstadoDelAcople {
            permiso: true,
            acoplada: true,
        }),
    ]
}

/// El archivo de TypeScript, tal y como tiene que quedar en `src/contrato.generado.ts`.
pub fn como_typescript() -> String {
    let muestras = muestras();

    // Los imports, agrupados por módulo y en el orden en que aparecen: un archivo generado que
    // cambia de orden entre corridas sería un diff falso en cada commit.
    let mut modulos: Vec<&'static str> = Vec::new();
    for mu in &muestras {
        if !modulos.contains(&mu.modulo) {
            modulos.push(mu.modulo);
        }
    }

    let mut salida = String::from(
        "/* ESTE ARCHIVO LO ESCRIBE RUST. No se edita a mano.\n\
         \x20*\n\
         \x20* Lo genera `src-tauri/src/contrato.rs` con el MISMO serde que corre en producción:\n\
         \x20* cada constante lleva el valor que la parte nativa emite de verdad y el tipo que\n\
         \x20* esta interfaz declara para él. Si los dos dejan de encajar, `pnpm typecheck` no\n\
         \x20* compila — y eso es justo lo que faltaba cuando la ficha automática no llegaba a la\n\
         \x20* banda (auditoría del sprint 001, hallazgo C1).\n\
         \x20*\n\
         \x20* Para regenerarlo tras cambiar un tipo de Rust:\n\
         \x20*\n\
         \x20*     cd src-tauri && ACTUALIZA_CONTRATO=1 cargo test contrato\n\
         \x20*/\n",
    );
    for modulo in modulos {
        let tipos: Vec<&str> = {
            let mut t: Vec<&str> = Vec::new();
            for mu in muestras.iter().filter(|mu| mu.modulo == modulo) {
                if !t.contains(&mu.tipo) {
                    t.push(mu.tipo);
                }
            }
            t
        };
        salida.push_str(&format!("import type {{ {} }} from \"{modulo}\";\n", tipos.join(", ")));
    }

    for mu in &muestras {
        let json = serde_json::to_string_pretty(&mu.valor).expect("el JSON ya estaba en memoria");
        // Dos espacios de sangría, como el resto del repo.
        let json = json.replace("\n  ", "\n    ").replace("\n}", "\n  }");
        salida.push_str(&format!("\nexport const {}: {} = {json};\n", mu.nombre, mu.tipo));
    }
    salida
}

#[cfg(test)]
mod tests {
    use super::*;

    fn donde_vive() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/contrato.generado.ts")
    }

    /// El gate: el archivo del repo es el que Rust escribiría hoy.
    ///
    /// Se ve en rojo renombrando cualquier campo de cualquier struct que cruce el puente — que es
    /// exactamente el cambio que este gate existe para no dejar pasar en silencio.
    #[test]
    fn el_contrato_del_repo_es_el_que_rust_emite() {
        let esperado = como_typescript();
        let ruta = donde_vive();

        if std::env::var("ACTUALIZA_CONTRATO").is_ok() {
            std::fs::write(&ruta, &esperado).expect("no se pudo escribir el contrato");
            println!("contrato regenerado en {}", ruta.display());
            return;
        }

        let hay = std::fs::read_to_string(&ruta).unwrap_or_default();
        if hay == esperado {
            return;
        }
        // **El mensaje enseña solo las líneas que cambiaron.** Volcar los dos archivos enteros
        // —que es lo que hace `assert_eq!`— deja un muro de cuatrocientas líneas escapadas donde
        // nadie encuentra el campo culpable: un gate cuyo fallo no se puede leer avisa a medias.
        let diferencias: Vec<String> = diferencias(&hay, &esperado);
        panic!(
            "`src/contrato.generado.ts` no es lo que la parte nativa emite hoy:\n{}\n\n\
             Regenéralo con `cd src-tauri && ACTUALIZA_CONTRATO=1 cargo test contrato` y mira qué \
             cambió: si un campo se renombró, la interfaz que lo leía dejó de leerlo.",
            diferencias.join("\n")
        );
    }

    /// Las líneas en que los dos archivos se separan, marcadas como un diff. Sin librerías: basta
    /// con recorrerlos a la par, porque este archivo lo escribe siempre el mismo generador y sus
    /// líneas no se mueven de sitio salvo que alguien cambie el contrato.
    fn diferencias(hay: &str, esperado: &str) -> Vec<String> {
        let (a, b): (Vec<&str>, Vec<&str>) = (hay.lines().collect(), esperado.lines().collect());
        let mut fuera = Vec::new();
        for i in 0..a.len().max(b.len()) {
            let (uno, otro) = (a.get(i).copied().unwrap_or(""), b.get(i).copied().unwrap_or(""));
            if uno != otro {
                fuera.push(format!("  línea {}:\n    - en el repo: {uno}\n    + Rust emite: {otro}", i + 1));
            }
            if fuera.len() == 12 {
                fuera.push("  … y más abajo siguen las diferencias".into());
                break;
            }
        }
        fuera
    }

    /// Que cada muestra siga siendo lo que dice ser. Un `Value::Null` aquí significa que alguien
    /// construyó la muestra mal y el gate de arriba compararía dos vacíos.
    #[test]
    fn ninguna_muestra_del_contrato_esta_vacia() {
        for mu in muestras() {
            assert!(
                mu.valor.is_object() || mu.valor.is_string(),
                "la muestra «{}» no serializó a nada útil: {:?}",
                mu.nombre,
                mu.valor
            );
        }
    }
}
