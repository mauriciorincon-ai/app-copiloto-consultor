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
//! auditoría contó diecisiete así. Eso es otra comprobación y vive en
//! `tests/unit/contrato-con-lectores.test.ts` (auditoría del S2, M6).

use serde_json::Value;

/// Una cosa que cruza el puente: cómo se llamará su constante, qué tipo la declara en TypeScript,
/// de qué módulo sale ese tipo, y el JSON que Rust produce de verdad.
pub struct Muestra {
    pub nombre: &'static str,
    pub tipo: &'static str,
    pub modulo: &'static str,
    pub valor: Value,
}

/// La ficha fijada de las muestras de notas: la de la maqueta (`notas.html`, «durante»).
fn ficha_fijada() -> crate::notas::FichaFijada {
    crate::notas::FichaFijada {
        titular: "Limpieza de datos: hasta tres fuentes".into(),
        documento: "Propuesta Páramo Azul".into(),
        seccion: Some("§3.2".into()),
        unidad: Some(crate::corpus::Unidad::Propuesta),
        ..Default::default()
    }
}

/// Una propuesta tuya, la de la maqueta: una cifra con su fecha, dicha por ti.
fn propuesta_tuya() -> crate::propuestas::Propuesta {
    crate::propuestas::Propuesta {
        regla: crate::propuestas::Regla::Cifra,
        de: crate::propuestas::De::Tuyo,
        texto: "Fecha real del tablero: 12 semanas desde la firma.".into(),
        ficha: None,
        seccion: None,
        hora: "14:16".into(),
    }
}

/// Un choque con una ficha fijada: la única forma con `ficha` y `seccion`.
fn propuesta_choque() -> crate::propuestas::Propuesta {
    crate::propuestas::Propuesta {
        regla: crate::propuestas::Regla::Choque,
        de: crate::propuestas::De::Cliente,
        texto: "cuatro fuentes".into(),
        ficha: Some("tres".into()),
        seccion: Some("§3.2".into()),
        hora: "14:18".into(),
    }
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

/// La puerta como nace: cerrada, sin `ghost` compilado al lado y sin nada en el registro.
fn puerta_cerrada() -> crate::puerta::VistaDeLaPuerta {
    crate::puerta::VistaDeLaPuerta { abierta: false, cerro: None, no_abre: None, ghost: None, registro: Vec::new() }
}

fn entrada(hora: &str, orden: &str, resultado: crate::puerta::Resultado) -> crate::puerta::Entrada {
    crate::puerta::Entrada { hora: hora.into(), orden: orden.into(), resultado }
}

fn denegado(motivo: crate::puerta::Motivo) -> crate::puerta::Resultado {
    crate::puerta::Resultado::Denegado { motivo }
}

/// La sugerencia de la maqueta (mirada 18), fundada en la ficha de «Páramo Azul · §3.2 Alcance».
fn sugerencia() -> crate::sintesis::Sugerencia {
    use crate::ficha::{Fuente, Respaldo};
    let respaldo = vec![Respaldo {
        titular: "Limpieza de datos: incluida, hasta tres fuentes".into(),
        linea: "Una cuarta fuente es adicional y se cotiza aparte.".into(),
        fuente: Fuente {
            documento: "Páramo Azul".into(),
            seccion: Some("§3.2 Alcance".into()),
            unidad: Some(crate::corpus::Unidad::Propuesta),
            conjeturada: false,
        },
    }];
    let peticion = crate::sintesis::Peticion::nueva("¿Y si sumamos el Excel de la fuerza comercial?", &respaldo)
        .expect("la muestra tiene turno y ficha");
    let crudo = crate::sintesis::Crudo {
        titular: "Tres fuentes incluidas, la cuarta aparte".into(),
        linea: "Confirma que la limpieza incluye hasta tres fuentes; una cuarta es adicional y se cotiza aparte.".into(),
        fuente: "F1".into(),
        confianza: "media".into(),
    };
    crate::sintesis::fundar(&crudo, &peticion, crate::sintesis::Quien::Sistema, "Modelo del sistema", 1_400)
        .expect("la muestra cita su ficha")
}

/// «Preparar» de la maqueta: Páramo Azul, su propuesta, ocho preguntas (5 · 1 · 2).
fn ensayo_preparado() -> crate::ensayo::Preparacion {
    crate::ensayo::Preparacion {
        clientes: vec!["Páramo Azul".into(), "Sur del Valle".into()],
        cliente: Some("Páramo Azul".into()),
        propuestas: vec![crate::ensayo::Elegible {
            ruta: "/Users/consultor/Corpus/Propuestas/Rentabilidad por canal · Páramo Azul.md".into(),
            nombre: "Rentabilidad por canal · Páramo Azul".into(),
        }],
        propuesta: Some("/Users/consultor/Corpus/Propuestas/Rentabilidad por canal · Páramo Azul.md".into()),
        topes: vec![5, 8, 12],
        tope: 8,
        cuentas: crate::ensayo::Cuentas { propuesta: 5, ficha: 1, objeciones: 2 },
        idioma: crate::ensayo::banco::Idioma::Es,
        enriquecer: false,
        transcribe: true,
        sin_corpus: false,
        guardados: 4,
    }
}

/// «6 · tu progreso» de la maqueta: cuatro ensayos con Páramo Azul, del 21 sep al 04 oct.
fn ensayo_progreso() -> crate::ensayo::guardado::Progreso {
    use crate::ensayo::guardado::{Cambio, DesdeElPrimero, FilaDelProgreso, Progreso};
    let fila = |empezo: &str, citadas, ppm, muletillas, s: u64| FilaDelProgreso {
        empezo: empezo.into(),
        citadas,
        evidencia: 21,
        ppm_medio: Some(ppm),
        muletillas: Some(muletillas),
        tiempo_medio_ms: Some(s * 1000),
    };
    Progreso {
        cliente: "Páramo Azul".into(),
        filas: vec![
            fila("2026-09-21 10:05", 9, 161, 17, 81),
            fila("2026-09-27 18:30", 11, 150, 12, 69),
            fila("2026-10-02 08:45", 12, 143, 11, 62),
            fila("2026-10-04 09:12", 14, 138, 9, 58),
        ],
        antes: 0,
        desde_el_primero: DesdeElPrimero {
            evidencia: Some(Cambio { desde: 9, hasta: 14 }),
            ritmo: Some(Cambio { desde: 161, hasta: 138 }),
            muletillas: Some(Cambio { desde: 17, hasta: 9 }),
            tiempo: Some(Cambio { desde: 81_000, hasta: 58_000 }),
        },
    }
}

/// La pregunta 3 de 8 de la maqueta, en cada fase.
fn ensayo_vista(fase: crate::ensayo::sesion::Fase) -> crate::ensayo::VistaDelEnsayo {
    use crate::ensayo::evaluacion::{Evaluacion, Evidencia, Muletilla};
    use crate::ensayo::sesion::{Fase, Fila, Informe};
    use crate::ficha::Fuente;
    let fuente = |documento: &str, seccion: &str, unidad| Fuente {
        documento: documento.into(),
        seccion: Some(seccion.into()),
        unidad: Some(unidad),
        conjeturada: false,
    };
    let evaluacion = Evaluacion {
        evidencia: vec![
            Evidencia { titular: "El plazo corre desde la entrega de datos".into(), fuente: fuente("Páramo Azul", "Supuestos", crate::corpus::Unidad::Propuesta), citada: true, dicha_por_ti: false },
            Evidencia { titular: "Sur del Valle: tres semanas por datos".into(), fuente: fuente("Sur del Valle", "cierre", crate::corpus::Unidad::Caso), citada: true, dicha_por_ti: false },
            // El lector de PDF conjeturó esta sección: la evaluada la marca (auditoría del S4, B33).
            Evidencia { titular: "Limpiar antes del tablero".into(), fuente: Fuente { conjeturada: true, ..fuente("Marco de trabajo", "etapa 2", crate::corpus::Unidad::Marco) }, citada: false, dicha_por_ti: true },
        ],
        tiempo_ms: 72_000,
        ppm: Some(142),
        muletillas: vec![Muletilla { frase: "o sea".into(), veces: 3 }, Muletilla { frase: "básicamente".into(), veces: 1 }],
        palabras: 170,
    };
    let informe = Informe {
        respondidas: 7,
        saltadas: 1,
        citadas: 14,
        evidencia: 21,
        ppm_medio: Some(138),
        muletillas: Some(9),
        la_que_mas: Some(Muletilla { frase: "o sea".into(), veces: 5 }),
        tiempo_medio_ms: Some(58_000),
        filas: vec![
            Fila { numero: 3, texto: "¿Qué pasa con el plazo si el ERP no entrega los datos limpios a tiempo?".into(), citadas: 2, evidencia: 3, tiempo_ms: Some(72_000), ppm: Some(142), saltada: false },
            Fila { numero: 4, texto: "¿Quién lo va a usar cuando ustedes se vayan?".into(), citadas: 0, evidencia: 0, tiempo_ms: None, ppm: None, saltada: true },
        ],
    };
    crate::ensayo::VistaDelEnsayo {
        fase,
        cliente: "Páramo Azul".into(),
        indice: 2,
        total: 8,
        pregunta: (fase != Fase::Cerrado).then(|| crate::ensayo::PreguntaEnPantalla {
            texto: "¿Qué pasa con el plazo si el ERP no entrega los datos limpios a tiempo?".into(),
            de: crate::ensayo::banco::De::Propuesta,
            seccion: Some("Supuestos".into()),
            fuente: None,
        }),
        leyendo: fase == Fase::Preguntando,
        cerrando: false,
        respuesta: if fase == Fase::Evaluada { "El supuesto dos lo cubre: el plazo corre desde que el ERP entrega los datos.".into() } else { String::new() },
        transcurrido_ms: if fase == Fase::Evaluada { 72_000 } else { 0 },
        usadas: if fase == Fase::Evaluada { evaluacion.citadas() } else { 0 },
        muletillas: (fase == Fase::Evaluada).then(|| evaluacion.total_de_muletillas()),
        evaluacion: (fase == Fase::Evaluada).then_some(evaluacion),
        banco: crate::ensayo::EstadoDelBanco::Apagado,
        informe: (fase == Fase::Cerrado).then_some(informe),
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
            // No cruza la costura (`serde(skip)`): es lo que ve el modelo de la síntesis.
            respaldo: vec![],
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
        de_ms: None,
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
        // La sala, en presencial (sprint 005, ADR 020 §2): un turno sin dueño. La banda lo pinta «Sala».
        m("TURNO_DE_LA_SALA", "Turno", "./cuaderno", &Turno { pista: Pista::Sala, ..turno() }),
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
                solo_notas: false,
                presencial: false,
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
                bytes_del_ensayo: 0,
                ensayo: false,
                motor: "apple-speechanalyzer",
            },
        ),
        // Presencial (sprint 005, ADR 020): la fila del micrófono es la sala, y la del sistema va cerrada sin motivo
        // porque no se pidió; Sesión no puede pintarla «A medias».
        m(
            "ESTADO_DE_LA_ESCUCHA_PRESENCIAL",
            "EstadoDeEscucha",
            "./cuaderno",
            &EstadoDeEscucha {
                escuchando: true,
                solo_notas: false,
                presencial: true,
                microfono: pista_abierta(),
                sistema: EstadoDePista::cerrada(),
                turnos_en_memoria: 4,
                bytes_del_transcript: 1_536,
                bytes_del_ensayo: 0,
                ensayo: false,
                motor: "apple-speechanalyzer",
            },
        ),
        // Un ensayo en marcha (sprint 004, fase 4): solo el micrófono, y tus respuestas en texto.
        m("ESTADO_DE_LA_ESCUCHA_EN_UN_ENSAYO", "EstadoDeEscucha", "./cuaderno", &EstadoDeEscucha::del_ensayo(true, 1_920_000, 412)),
        m("DISPONIBILIDAD_LISTO", "Disponibilidad", "./cuaderno", &Disponibilidad::Listo),
        m(
            "DISPONIBILIDAD_SIN_MOTOR",
            "Disponibilidad",
            "./cuaderno",
            &Disponibilidad::SinMotor { motivo: crate::stt::PorQueNoHayMotor::SinTranscriptor },
        ),
        m("SALIDA_DE_AUDIO", "Salida", "./cuaderno", &Salida::Altavoces),
        m("SALIDA_DE_AUDIO_ALTAVOZ_EXTERNO", "Salida", "./cuaderno", &Salida::AltavozExterno { nombre: "Monitor LG".into() }),
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
        // encendida sin poder hablar. La cuarta combinación —encendida, puede, y callada— la
        // dibuja la mirada 16-bis (aprobada el 2026-09-26): «Callado · esperando el siguiente
        // turno».
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
        // ---- la síntesis (C7) ----------------------------------------------------------------
        // La sugerencia cruza por «escucha», detrás de su ficha. Se construye con `fundar` porque
        // es la ÚNICA manera: sus campos son privados (ADR 010, la regla dura que no compila).
        m("NOVEDAD_SUGERENCIA", "Novedad", "./ficha", &Novedad::Sugerencia(Box::new(sugerencia()))),
        m("ESTADO_DE_LA_IA_NADIE", "EstadoDeLaIa", "./ia", &crate::EstadoDeLaIa {
            redactar: false,
            enriquecer: false,
            quien: None,
            sistema: Some(crate::sintesis::PorQueNoRedacta::AppleIntelligenceApagado),
            api: crate::EstadoDelApi {
                encendida: false,
                externo: crate::sintesis::api::Externo::Claude,
                hay_clave: false,
            },
            latencia_ms: None,
            reunion_usd: 0.0,
            mes_usd: 0.0,
            tope_usd: 10.0,
        }),
        m("ESTADO_DE_LA_IA_CON_API", "EstadoDeLaIa", "./ia", &crate::EstadoDeLaIa {
            redactar: true,
            enriquecer: true,
            quien: Some(crate::sintesis::Quien::Api),
            sistema: None,
            api: crate::EstadoDelApi {
                encendida: true,
                externo: crate::sintesis::api::Externo::Groq,
                hay_clave: true,
            },
            latencia_ms: Some(1_400),
            reunion_usd: 0.031,
            mes_usd: 0.84,
            tope_usd: 10.0,
        }),
        // ---- el acople, que la banda dibuja en su cabecera --------------------------------
        // Cada motivo de «nadie redacta», uno por uno: son grafías kebab de varias palabras, y un
        // cambio de `rename_all` solo lo vería una muestra de CADA variante (auditoría del S2, B2).
        m("POR_QUE_NO_REDACTA_APAGADO", "PorQueNoRedacta", "./ia", &crate::sintesis::PorQueNoRedacta::AppleIntelligenceApagado),
        m("POR_QUE_NO_REDACTA_NO_COMPATIBLE", "PorQueNoRedacta", "./ia", &crate::sintesis::PorQueNoRedacta::MacNoCompatible),
        m("POR_QUE_NO_REDACTA_NO_DISPONIBLE", "PorQueNoRedacta", "./ia", &crate::sintesis::PorQueNoRedacta::NoDisponible),
        m("POR_QUE_NO_REDACTA_DESCARGANDO", "PorQueNoRedacta", "./ia", &crate::sintesis::PorQueNoRedacta::ModeloDescargandose),
        m("POR_QUE_NO_REDACTA_SIN_PUENTE", "PorQueNoRedacta", "./ia", &crate::sintesis::PorQueNoRedacta::SinPuente),
        m("POR_QUE_NO_REDACTA_SIN_CLAVE", "PorQueNoRedacta", "./ia", &crate::sintesis::PorQueNoRedacta::SinClave),
        m("POR_QUE_NO_REDACTA_TOPE", "PorQueNoRedacta", "./ia", &crate::sintesis::PorQueNoRedacta::TopeDelMes),
        m("ESTADO_DEL_ACOPLE", "EstadoDelAcople", "./acople", &crate::EstadoDelAcople { acoplada: true }),
        // ---- la banda, arriba o abajo (sprint 004, ADR 004 enmienda 1) --------------------------
        m("LA_FRANJA_ARRIBA", "LaFranja", "./franja", &crate::LaFranja {
            borde: crate::ventana::Borde::Arriba,
            barra: 38.0,
            aviso_visto: false,
        }),
        m("LA_FRANJA_ABAJO", "LaFranja", "./franja", &crate::LaFranja {
            borde: crate::ventana::Borde::Abajo,
            barra: 25.0,
            aviso_visto: true,
        }),
        // ---- el ensayo (sprint 004, ADR 019) — los estados de `ensayo.html` ---------------------
        m("PREPARACION_DEL_ENSAYO", "Preparacion", "./ensayo", &ensayo_preparado()),
        m("PREPARACION_SIN_CORPUS", "Preparacion", "./ensayo", &crate::ensayo::Preparacion {
            clientes: vec!["Páramo Azul".into(), "Sur del Valle".into()],
            cliente: Some("Sur del Valle".into()),
            propuestas: Vec::new(),
            propuesta: None,
            topes: vec![5, 8, 12],
            tope: 8,
            cuentas: crate::ensayo::Cuentas::default(),
            idioma: crate::ensayo::banco::Idioma::En,
            enriquecer: true,
            transcribe: false,
            sin_corpus: true,
            guardados: 0,
        }),
        m("ENSAYO_PREGUNTANDO", "VistaDelEnsayo", "./ensayo", &ensayo_vista(crate::ensayo::sesion::Fase::Preguntando)),
        m("ENSAYO_EVALUADA", "VistaDelEnsayo", "./ensayo", &ensayo_vista(crate::ensayo::sesion::Fase::Evaluada)),
        m("ENSAYO_CERRADO", "VistaDelEnsayo", "./ensayo", &ensayo_vista(crate::ensayo::sesion::Fase::Cerrado)),
        m("ENSAYO_DEL_MODELO", "VistaDelEnsayo", "./ensayo", &{
            let mut v = ensayo_vista(crate::ensayo::sesion::Fase::Respondiendo);
            v.pregunta = Some(crate::ensayo::PreguntaEnPantalla {
                texto: "¿Quién firma por parte de la gerencia general?".into(),
                de: crate::ensayo::banco::De::Modelo,
                seccion: Some("Quién decide".into()),
                fuente: None,
            });
            v.banco = crate::ensayo::EstadoDelBanco::Sumadas { cuantas: 2 };
            v
        }),
        m("ENSAYO_OBJECION", "VistaDelEnsayo", "./ensayo", &{
            let mut v = ensayo_vista(crate::ensayo::sesion::Fase::Respondiendo);
            v.pregunta = Some(crate::ensayo::PreguntaEnPantalla {
                texto: "¿Estos números son correctos?".into(),
                de: crate::ensayo::banco::De::Objeciones,
                seccion: None,
                fuente: Some("Kuznetsova".into()),
            });
            v.banco = crate::ensayo::EstadoDelBanco::NoSeEnriquecio { porque: crate::ensayo::enriquecer::PorQueNo::NadaFundado };
            v
        }),
        // ---- lo que queda de tus ensayos (sprint 004, fase 4, ADR 015 enmienda 4) -----------------
        m("PROGRESO_DEL_ENSAYO", "Progreso", "./ensayo", &ensayo_progreso()),
        m("PROGRESO_DE_UN_ENSAYO", "Progreso", "./ensayo", &{
            let mut p = ensayo_progreso();
            p.filas.drain(..3);
            p.desde_el_primero = crate::ensayo::guardado::DesdeElPrimero::default();
            p
        }),
        m("PROGRESO_CON_MAS", "Progreso", "./ensayo", &{
            let mut p = ensayo_progreso();
            p.antes = 3;
            p.filas[0].ppm_medio = None;
            p.filas[0].muletillas = None;
            p.desde_el_primero.ritmo = Some(crate::ensayo::guardado::Cambio { desde: 150, hasta: 138 });
            p
        }),
        m("NO_EMPEZO_EN_REUNION", "NoEmpezo", "./ensayo", &crate::ensayo::NoEmpezo::EnReunion),
        m("NO_EMPEZO_SIN_CORPUS", "NoEmpezo", "./ensayo", &crate::ensayo::NoEmpezo::SinCorpus),
        m("NO_EMPEZO_MICROFONO", "NoEmpezo", "./ensayo", &crate::ensayo::NoEmpezo::Microfono {
            porque: crate::capture::PorQueNoAbrio::SinPermisoDelMicrofono,
        }),
        // Con una videollamada abierta, no por altavoces que la app reconoce (auditoría del S4, A1).
        m("NO_EMPEZO_VIDEOLLAMADA", "NoEmpezo", "./ensayo", &crate::ensayo::NoEmpezo::Videollamada),
        m("NO_EMPEZO_NO_SE_SABE_SI_HAY_LLAMADA", "NoEmpezo", "./ensayo", &crate::ensayo::NoEmpezo::NoSeSabeSiHayLlamada),
        // Lo que el modelo no sumó, con sus dos porqués nuevos (auditoría del S4, B22).
        m("ENSAYO_REPETIDAS", "VistaDelEnsayo", "./ensayo", &{
            let mut v = ensayo_vista(crate::ensayo::sesion::Fase::Preguntando);
            v.banco = crate::ensayo::EstadoDelBanco::NoSeEnriquecio { porque: crate::ensayo::enriquecer::PorQueNo::Repetidas };
            v
        }),
        m("ENSAYO_SIN_SECCIONES", "VistaDelEnsayo", "./ensayo", &{
            let mut v = ensayo_vista(crate::ensayo::sesion::Fase::Preguntando);
            v.banco = crate::ensayo::EstadoDelBanco::NoSeEnriquecio { porque: crate::ensayo::enriquecer::PorQueNo::SinSecciones };
            v
        }),
        // Lo que el motor de este Mac sabe hacer (auditoría del S2, M7): con motor, y sin él por cada
        // uno de sus tres porqués, que son grafías kebab de varias palabras.
        // ---- lo que salió al API (auditoría del S2, B37) ----------------------------------------
        // Construida con la bóveda de verdad sobre un texto con un cliente y una persona: los
        // trozos, los marcadores y la cuenta son los que el serializador real escribe.
        m("LO_QUE_SALIO", "LoQueSalio", "./ia", &{
            let mut b = crate::sintesis::anonimo::Boveda::nueva(&["Páramo Azul".to_string()]);
            let tapado = b.tapar("El alcance de Páramo Azul incluye tres fuentes. Andrea Villalba pregunta por una cuarta.");
            let mut s = crate::sintesis::api::LoQueSalio::de(&tapado, &b, crate::sintesis::api::Externo::Claude, "Alcance");
            s.hora = "14:22".into();
            s.usd = Some(0.004);
            s
        }),
        // Lo del ensayo, rotulado como lo que es (auditoría del S4, M21).
        m("LO_QUE_SALIO_DEL_BANCO", "LoQueSalio", "./ia", &{
            let b = crate::sintesis::anonimo::Boveda::nueva(&["Páramo Azul".to_string()]);
            let mut s = crate::sintesis::api::LoQueSalio::de("S1 · Alcance: perfilado y limpieza de tres fuentes.", &b, crate::sintesis::api::Externo::Claude, "Propuesta Páramo Azul")
                .para(crate::sintesis::api::Para::Banco);
            s.hora = "09:12".into();
            s
        }),
        // ---- las preferencias que se recuerdan (sprint 003, ADR 002 enmienda 2) -----------------
        // El idioma de cada pista cruza a la ventana principal al abrirse: lo que el usuario eligió
        // la vez anterior. El cliente en inglés, para que la muestra no sea la de fábrica. **Y la sala**
        // (sprint 005, ADR 020 §6), elegida: sin elegir no cruza y vale la del cliente.
        m("IDIOMAS_DE_PISTA", "IdiomasDePista", "./cuaderno", &crate::prefs::IdiomasDePista {
            consultor: "es-ES".into(),
            cliente: "en-US".into(),
            sala: Some("en-US".into()),
        }),
        // ---- tus notas (C9, sprint 003, fase 1, ADR 015) -------------------------------------
        // Tres comandos que la pantalla de Notas lee: el cuaderno de ahora (durante · al cerrar), la
        // lista de reuniones guardadas y lo que devuelve guardar. Con datos de la maqueta. El contenido
        // descifrado de una reunión NO cruza: la maqueta no tiene «abrir» dentro de la app —se lee
        // exportándola, con su aviso—, y lo que no cruza no se declara.
        m("VISTA_DEL_CUADERNO", "VistaDelCuaderno", "./notas", &{
            let mut c = crate::notas::Cuaderno::nuevo(false);
            c.escribir("Piden la cuarta fuente (Excel de logística).\nFecha real: 12 semanas desde la firma.");
            c.acordar("Cuarta fuente: cotización aparte");
            c.ver(ficha_fijada());
            c.fijar_la_vigente();
            crate::reunion::VistaDelCuaderno {
                nota: c.nota().to_string(),
                acuerdos: c.acuerdos().to_vec(),
                fijadas: c.fijadas().to_vec(),
                resumen: c.resumen(),
                conservar_mis_turnos: false,
                abierta: true,
                escuchando: false,
                previsto: Some(crate::reunion::Previsto {
                    fecha: "2026-09-20".into(),
                    minutos: 47,
                    cliente: None,
                    archivo: "reunion-2026-09-20-1402.ghost".into(),
                }),
                turnos_del_cliente: 63,
                lecturas: 9,
                retencion: crate::prefs::Retencion::Dias90,
                propuestas: c.en_espera().to_vec(),
                lleno: c.lleno(),
                ventana: crate::bandeja::Ventana::TresHoras,
                sin_proteger: false,
            }
        }),
        // ---- las propuestas y la bandeja (sprint 003, fase 2, ADR 016) ---------------------------
        // Durante: el cuaderno con dos esperando (la maqueta «sprint 3 · durante, con propuestas»),
        // una tuya y un choque con su ficha, que son las dos formas con campos opcionales.
        m("VISTA_CON_PROPUESTAS", "VistaDelCuaderno", "./notas", &{
            let mut c = crate::notas::Cuaderno::nuevo(false);
            c.escribir("Piden la cuarta fuente.");
            c.proponer(vec![propuesta_tuya(), propuesta_choque()]);
            crate::reunion::VistaDelCuaderno {
                nota: c.nota().to_string(),
                acuerdos: Vec::new(),
                fijadas: Vec::new(),
                resumen: c.resumen(),
                conservar_mis_turnos: false,
                abierta: true,
                escuchando: true,
                previsto: None,
                turnos_del_cliente: 12,
                lecturas: 0,
                retencion: crate::prefs::Retencion::Dias90,
                propuestas: c.en_espera().to_vec(),
                lleno: true,
                ventana: crate::bandeja::Ventana::AlCerrar,
                sin_proteger: false,
            }
        }),
        // macOS no dejó proteger el cuaderno (auditoría del S3, B4): la única forma con el campo en `true`.
        m("CUADERNO_SIN_PROTEGER", "VistaDelCuaderno", "./notas", &{
            let mut c = crate::notas::Cuaderno::nuevo(false);
            c.escribir("Piden la cuarta fuente.");
            crate::reunion::VistaDelCuaderno {
                nota: c.nota().to_string(),
                acuerdos: Vec::new(),
                fijadas: Vec::new(),
                resumen: c.resumen(),
                conservar_mis_turnos: false,
                abierta: true,
                escuchando: true,
                previsto: None,
                turnos_del_cliente: 3,
                lecturas: 0,
                retencion: crate::prefs::Retencion::Dias90,
                propuestas: Vec::new(),
                lleno: false,
                ventana: crate::bandeja::Ventana::TresHoras,
                sin_proteger: true,
            }
        }),
        // La línea de la banda: la última propuesta, o nada. Y las cinco reglas, cada una con su grafía.
        m("PROPUESTA_EN_LA_BANDA", "LineaDePropuesta", "./notas", &Some(propuesta_tuya())),
        m("SIN_PROPUESTA_EN_LA_BANDA", "LineaDePropuesta", "./notas", &None::<crate::propuestas::Propuesta>),
        m("PROPUESTA_CHOQUE", "Propuesta", "./notas", &propuesta_choque()),
        m("REGLA_CIFRA", "Regla", "./notas", &crate::propuestas::Regla::Cifra),
        m("REGLA_COMPROMISO", "Regla", "./notas", &crate::propuestas::Regla::Compromiso),
        m("REGLA_CHOQUE", "Regla", "./notas", &crate::propuestas::Regla::Choque),
        m("REGLA_NOMBRE", "Regla", "./notas", &crate::propuestas::Regla::Nombre),
        m("REGLA_PREGUNTA", "Regla", "./notas", &crate::propuestas::Regla::Pregunta),
        m("VENTANA_AL_CERRAR", "Ventana", "./notas", &crate::bandeja::Ventana::AlCerrar),
        m("VENTANA_UNA_HORA", "Ventana", "./notas", &crate::bandeja::Ventana::UnaHora),
        m("VENTANA_TRES_HORAS", "Ventana", "./notas", &crate::bandeja::Ventana::TresHoras),
        m("VENTANA_FIN_DEL_DIA", "Ventana", "./notas", &crate::bandeja::Ventana::FinDelDia),
        m("VENTANA_UN_DIA", "Ventana", "./notas", &crate::bandeja::Ventana::Dia),
        // La bandeja abierta (recién cerrada, con una guardada desde aquí) y cerrada con llave.
        m("BANDEJA_ABIERTA", "VistaDeLaBandeja", "./notas", &crate::reunion::VistaDeLaBandeja {
            archivo: "reunion-2026-09-20-1402.ghost".into(),
            vence: 1_790_527_268,
            bytes: 4_096,
            propuestas: Some(vec![propuesta_tuya(), propuesta_choque()]),
            guardadas: vec![crate::propuestas::Propuesta { texto: "La cuarta fuente se cotiza aparte.".into(), ..propuesta_tuya() }],
            mas: 0,
            ventana: crate::bandeja::Ventana::TresHoras,
        }),
        m("BANDEJA_CON_LLAVE", "VistaDeLaBandeja", "./notas", &crate::reunion::VistaDeLaBandeja {
            archivo: "reunion-2026-09-20-1402.ghost".into(),
            vence: 1_790_527_268,
            bytes: 4_096,
            propuestas: None,
            guardadas: Vec::new(),
            mas: 1,
            ventana: crate::bandeja::Ventana::FinDelDia,
        }),
        m("ESTADO_DE_LA_BANDEJA", "EstadoDeLaBandeja", "./notas", &crate::reunion::EstadoDeLaBandeja {
            vence: Some(1_790_527_268),
            no_corrio: false,
        }),
        m("SIN_BANDEJA_Y_LA_TAREA_NO_CORRIO", "EstadoDeLaBandeja", "./notas", &crate::reunion::EstadoDeLaBandeja {
            vence: None,
            no_corrio: true,
        }),
        // «Este cliente», la bandera y la NDA (ADR 017). Las banderas salen del catálogo de verdad:
        // si una fila cambia, la muestra cambia con ella y la pantalla se entera en el contrato.
        m("VISTA_DEL_CLIENTE", "VistaDelCliente", "./jurisdiccion", &crate::jurisdiccion::VistaDelCliente {
            clientes: vec!["Páramo Azul".into(), "Sur del Valle".into()],
            elegido: Some("Páramo Azul".into()),
            bandera: Some(crate::jurisdiccion::bandera(Some("Colombia"))),
            nda: crate::jurisdiccion::Nda::SinRevisar,
            clausula: crate::jurisdiccion::clausula().clone(),
        }),
        m("VISTA_DEL_CLIENTE_SIN_ELEGIR", "VistaDelCliente", "./jurisdiccion", &crate::jurisdiccion::VistaDelCliente {
            clientes: vec!["Páramo Azul".into(), "Sur del Valle".into()],
            elegido: None,
            bandera: None,
            nda: crate::jurisdiccion::Nda::SinRevisar,
            clausula: crate::jurisdiccion::clausula().clone(),
        }),
        m("BANDERA_CONOCIDA", "LaBandera", "./jurisdiccion", &crate::jurisdiccion::bandera(Some("Colombia"))),
        m("BANDERA_CON_PENDIENTE", "LaBandera", "./jurisdiccion", &crate::jurisdiccion::bandera(Some("California"))),
        m("BANDERA_SIN_VERIFICAR", "LaBandera", "./jurisdiccion", &crate::jurisdiccion::bandera(Some("Missouri"))),
        m("BANDERA_FUERA_DEL_CATALOGO", "LaBandera", "./jurisdiccion", &crate::jurisdiccion::bandera(Some("Bolivia"))),
        m("BANDERA_SIN_INDICAR", "LaBandera", "./jurisdiccion", &crate::jurisdiccion::bandera(None)),
        m("NDA_SIN_REVISAR", "Nda", "./jurisdiccion", &crate::jurisdiccion::Nda::SinRevisar),
        m("NDA_NO_LO_PROHIBE", "Nda", "./jurisdiccion", &crate::jurisdiccion::Nda::NoLoProhibe),
        m("NDA_LO_PROHIBE", "Nda", "./jurisdiccion", &crate::jurisdiccion::Nda::LoProhibe),
        m("ESCUCHA_SOLO_NOTAS", "EstadoDeEscucha", "./cuaderno", &crate::escucha::EstadoDeEscucha::solo_notas()),
        // La puerta local (C16, ADR 018): cerrada, abierta con lo que hizo tu agente y cerrada sola por una
        // reunión. El registro, de la más reciente a la más antigua, con las tres formas de un resultado.
        m("VISTA_DE_LA_PUERTA_CERRADA", "VistaDeLaPuerta", "./puerta", &puerta_cerrada()),
        m("VISTA_DE_LA_PUERTA_ABIERTA", "VistaDeLaPuerta", "./puerta", &crate::puerta::VistaDeLaPuerta {
            abierta: true,
            ghost: Some("/Users/ana/app-copiloto-consultor/src-tauri/target/debug/ghost".into()),
            registro: vec![
                entrada("11:12", "ghost notas abrir", crate::puerta::Resultado::Fallo),
                entrada("11:09", "ghost ia --encender-api", denegado(crate::puerta::Motivo::ElApiEsTuyo)),
                entrada("11:04", "ghost corpus reindexar", crate::puerta::Resultado::Hecho { cuenta: Some(28) }),
            ],
            ..puerta_cerrada()
        }),
        m("VISTA_DE_LA_PUERTA_EN_REUNION", "VistaDeLaPuerta", "./puerta", &crate::puerta::VistaDeLaPuerta {
            cerro: Some(crate::puerta::Cierre::EnReunion),
            ghost: Some("/Users/ana/app-copiloto-consultor/src-tauri/target/debug/ghost".into()),
            registro: vec![
                entrada("14:02", "ghost corpus buscar", denegado(crate::puerta::Motivo::EnReunion)),
                entrada("13:58", "ghost corpus buscar", crate::puerta::Resultado::Hecho { cuenta: Some(3) }),
            ],
            ..puerta_cerrada()
        }),
        m("VISTA_DE_LA_PUERTA_SIN_GHOST", "VistaDeLaPuerta", "./puerta", &crate::puerta::VistaDeLaPuerta {
            abierta: true,
            ..puerta_cerrada()
        }),
        m("PUERTA_CERRADA_A_MANO", "Cierre", "./puerta", &crate::puerta::Cierre::ATuMano),
        m("PUERTA_NO_ABRE_EN_REUNION", "NoAbre", "./puerta", &crate::puerta::NoAbre::EnReunion),
        m("PUERTA_NO_ABRE_RUTA_LARGA", "NoAbre", "./puerta", &crate::puerta::NoAbre::RutaLarga),
        m("PUERTA_NO_ABRE_LLAVERO", "NoAbre", "./puerta", &crate::puerta::NoAbre::Llavero),
        m("PUERTA_NO_ABRE_SOCKET", "NoAbre", "./puerta", &crate::puerta::NoAbre::Socket),
        m("PUERTA_LLAVE_ERRADA", "Motivo", "./puerta", &crate::puerta::Motivo::LlaveErrada),
        m("PUERTA_NO_DELEGABLE", "Motivo", "./puerta", &crate::puerta::Motivo::NoDelegable),
        m("PUERTA_ORDEN_DESCONOCIDA", "Motivo", "./puerta", &crate::puerta::Motivo::OrdenDesconocida),
        // `reuniones_guardadas` devuelve una lista; se ata el elemento, con vencimiento y sin él.
        m("REUNION_GUARDADA", "ReunionGuardada", "./notas", &crate::carpeta::Reunion {
            archivo: "paramo-azul-2026-09-20.ghost".into(),
            bytes: 22_528,
            guardada: 1_789_900_000,
            vence: 1_797_676_000,
        }),
        m("REUNION_GUARDADA_PARA_SIEMPRE", "ReunionGuardada", "./notas", &crate::carpeta::Reunion {
            archivo: "reunion-2026-09-27-1402.ghost".into(),
            bytes: 3_104,
            guardada: 1_790_500_000,
            vence: 0,
        }),
        m("LISTA_DE_REUNIONES", "ListaDeReuniones", "./notas", &crate::reunion::ListaDeReuniones {
            reuniones: Vec::new(),
        }),
        m("QUE_SABE_TRANSCRIBIR", "QueSabeTranscribir", "./cuaderno", &crate::QueSabeTranscribir {
            motor: "apple-speechanalyzer",
            techo: 5,
            idiomas: vec![
                crate::IdiomaDelMotor { codigo: "es-ES".into(), disponibilidad: Disponibilidad::Listo },
                crate::IdiomaDelMotor { codigo: "en-US".into(), disponibilidad: Disponibilidad::SinModelo },
            ],
            motivo: None,
        }),
        m("QUE_SABE_TRANSCRIBIR_SIN_TRANSCRIPTOR", "QueSabeTranscribir", "./cuaderno", &crate::QueSabeTranscribir {
            motor: "ninguno",
            techo: 0,
            idiomas: vec![],
            motivo: Some(crate::stt::PorQueNoHayMotor::SinTranscriptor),
        }),
        m("QUE_SABE_TRANSCRIBIR_SIN_PUENTE", "QueSabeTranscribir", "./cuaderno", &crate::QueSabeTranscribir {
            motor: "ninguno",
            techo: 0,
            idiomas: vec![],
            motivo: Some(crate::stt::PorQueNoHayMotor::SinPuente),
        }),
        m("QUE_SABE_TRANSCRIBIR_NO_CONTESTA", "QueSabeTranscribir", "./cuaderno", &crate::QueSabeTranscribir {
            motor: "ninguno",
            techo: 0,
            idiomas: vec![],
            motivo: Some(crate::stt::PorQueNoHayMotor::NoContesta),
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
    /// construyó la muestra mal y el gate de arriba compararía dos vacíos — salvo en las muestras
    /// cuyo `null` ES el mensaje: «la banda ya no tiene propuesta que enseñar» (ADR 016 §3).
    #[test]
    fn ninguna_muestra_del_contrato_esta_vacia() {
        const NULAS_A_PROPOSITO: &[&str] = &["SIN_PROPUESTA_EN_LA_BANDA"];
        for mu in muestras() {
            assert!(
                mu.valor.is_object() || mu.valor.is_string() || (mu.valor.is_null() && NULAS_A_PROPOSITO.contains(&mu.nombre)),
                "la muestra «{}» no serializó a nada útil: {:?}",
                mu.nombre,
                mu.valor
            );
        }
    }
}
