/* ESTE ARCHIVO LO ESCRIBE RUST. No se edita a mano.
 *
 * Lo genera `src-tauri/src/contrato.rs` con el MISMO serde que corre en producción:
 * cada constante lleva el valor que la parte nativa emite de verdad y el tipo que
 * esta interfaz declara para él. Si los dos dejan de encajar, `pnpm typecheck` no
 * compila — y eso es justo lo que faltaba cuando la ficha automática no llegaba a la
 * banda (auditoría del sprint 001, hallazgo C1).
 *
 * Para regenerarlo tras cambiar un tipo de Rust:
 *
 *     cd src-tauri && ACTUALIZA_CONTRATO=1 cargo test contrato
 */
import type { Novedad, Aparicion } from "./ficha";
import type { Turno, Reunion, Permisos, EstadoDeEscucha, Disponibilidad, Salida, EstadoDelDiccionario, EstadoDelCorpus, InformeDelCorte, LaVoz, EstadoDeLaPantalla, IdiomasDePista, QueSabeTranscribir } from "./cuaderno";
import type { EnTuMac } from "./radar";
import type { EstadoDeLaIa, PorQueNoRedacta, LoQueSalio } from "./ia";
import type { EstadoDelAcople } from "./acople";
import type { VistaDelCuaderno, ReunionGuardada, Guardada, ContenidoDeReunion } from "./notas";

export const NOVEDAD_EMPIEZA: Novedad = {
    "que": "empieza"
  };

export const NOVEDAD_TURNO: Novedad = {
    "eco": false,
    "pista": "sistema",
    "que": "turno"
  };

export const NOVEDAD_SIN_TEXTO: Novedad = {
    "que": "sin-texto"
  };

export const NOVEDAD_RUIDO: Novedad = {
    "que": "ruido"
  };

export const NOVEDAD_APARECE_FICHA: Novedad = {
    "acumuladas": [
      {
        "texto": "Sur del Valle: cuatro fuentes en 9 semanas",
        "unidad": "caso"
      }
    ],
    "clase": "ficha",
    "fuente": {
      "conjeturada": false,
      "documento": "Páramo Azul · Propuesta",
      "seccion": "§3.2 Alcance",
      "unidad": "propuesta"
    },
    "hora": "14:02",
    "linea": "Cubre perfilado y limpieza de ERP, POS y Excel de canal.",
    "lineaLarga": "Cubre perfilado y limpieza de ERP, POS y Excel de canal; una cuarta fuente se cotiza aparte.",
    "motivo": "terminoDelCorpus",
    "ms": 1240,
    "que": "aparece",
    "titular": "Limpieza de datos: incluida, hasta tres fuentes"
  };

export const NOVEDAD_APARECE_SIN_RESULTADO: Novedad = {
    "buscado": "certificacion iso 27001",
    "cercanas": [
      {
        "texto": "§5.1 Seguridad y manejo de datos",
        "unidad": "marco"
      }
    ],
    "clase": "sinResultado",
    "hora": "14:02",
    "maniobra": "credencial",
    "motivo": "pregunta",
    "ms": 1240,
    "que": "aparece"
  };

export const APARICION_DEL_ATAJO: Aparicion = {
    "acumuladas": [
      {
        "texto": "Sur del Valle: cuatro fuentes en 9 semanas",
        "unidad": "caso"
      }
    ],
    "clase": "ficha",
    "fuente": {
      "conjeturada": false,
      "documento": "Páramo Azul · Propuesta",
      "seccion": "§3.2 Alcance",
      "unidad": "propuesta"
    },
    "hora": "14:02",
    "linea": "Cubre perfilado y limpieza de ERP, POS y Excel de canal.",
    "lineaLarga": "Cubre perfilado y limpieza de ERP, POS y Excel de canal; una cuarta fuente se cotiza aparte.",
    "motivo": "atajo",
    "ms": 1240,
    "titular": "Limpieza de datos: incluida, hasta tres fuentes"
  };

export const TURNO_DEL_CLIENTE: Turno = {
    "desdeMs": 5400,
    "eco": false,
    "hastaMs": 8000,
    "hora": "14:02",
    "pista": "sistema",
    "texto": "¿Y la limpieza de datos está dentro del alcance?"
  };

export const REUNION_NINGUNA: Reunion = {
    "que": "ninguna"
  };

export const REUNION_DETECTADA: Reunion = {
    "cliente": "Google Meet",
    "proteccion": "Verificada",
    "que": "detectada",
    "titulo": "Páramo Azul · seguimiento"
  };

export const REUNION_NO_SE_PUEDE_SABER: Reunion = {
    "motivo": "sin-accesibilidad",
    "que": "no-se-puede-saber"
  };

export const PERMISOS: Permisos = {
    "accesibilidad": "no-se-sabe",
    "audio": "concedido",
    "microfono": "concedido",
    "pantalla": "sin-conceder"
  };

export const ESTADO_DE_LA_ESCUCHA: EstadoDeEscucha = {
    "bytesDelTranscript": 2048,
    "escuchando": true,
    "microfono": {
      "abierta": true,
      "bytes": 1920000,
      "motivo": null
    },
    "sistema": {
      "abierta": false,
      "bytes": 0,
      "motivo": "dispositivo-ocupado"
    }
  };

export const DISPONIBILIDAD_LISTO: Disponibilidad = {
    "estado": "listo"
  };

export const DISPONIBILIDAD_SIN_MOTOR: Disponibilidad = {
    "estado": "sin-motor"
  };

export const SALIDA_DE_AUDIO: Salida = {
    "salida": "altavoces"
  };

export const SALIDA_DE_AUDIO_ALTAVOZ_EXTERNO: Salida = {
    "nombre": "Monitor LG",
    "salida": "altavoz-externo"
  };

export const SALIDA_DE_AUDIO_OTRA: Salida = {
    "nombre": "AirPods Pro",
    "salida": "otra"
  };

export const SALIDA_DE_AUDIO_NO_SE_SABE: Salida = {
    "motivo": "sin-conexion",
    "nombre": "Altavoz USB",
    "salida": "no-se-sabe"
  };

export const ESTADO_DEL_DICCIONARIO: EstadoDelDiccionario = {
    "delCorpus": 12,
    "enTuArchivo": 5,
    "ruta": "~/Library/Application Support/com.aiapps.copiloto-consultor/diccionario.yaml",
    "terminos": 17
  };

export const ESTADO_DEL_CORPUS: EstadoDelCorpus = {
    "bytesDelIndice": 1884160,
    "carpeta": "~/Documentos/Consultoría",
    "conjeturados": 2,
    "documentos": 6,
    "dondeVive": "~/Library/…/Angel Ghost/corpus",
    "ilegibles": 1,
    "porUnidad": [
      {
        "documentos": 2,
        "unidad": "propuesta"
      }
    ],
    "secciones": 31,
    "sinUnidad": 1
  };

export const INFORME_DEL_CORTE: InformeDelCorte = {
    "piezas": [
      [
        "voz",
        "cortada"
      ],
      [
        "sugerencia",
        "cortada"
      ],
      [
        "audio-del-microfono",
        "cortada"
      ],
      [
        "audio-del-sistema",
        "cortada"
      ],
      [
        "ultimo-frame",
        "cortada"
      ],
      [
        "transcript",
        "cortada"
      ],
      [
        "tus-turnos",
        "cortada"
      ],
      [
        "contador-de-red",
        "cortada"
      ],
      [
        "banda",
        "cortada"
      ],
      [
        "acople",
        "cortada"
      ]
    ]
  };

export const LA_VOZ_APAGADA: LaVoz = {
    "diciendo": false,
    "encendida": false,
    "puede": false
  };

export const LA_VOZ_DICIENDO: LaVoz = {
    "diciendo": true,
    "encendida": true,
    "puede": true
  };

export const LA_VOZ_SIN_AURICULARES: LaVoz = {
    "diciendo": false,
    "encendida": true,
    "puede": false
  };

export const PANTALLA_LEYENDO: EstadoDeLaPantalla = {
    "bytesEnMemoria": 1440318,
    "vista": "leyendo"
  };

export const PANTALLA_APAGADA: EstadoDeLaPantalla = {
    "bytesEnMemoria": 0,
    "vista": "apagada"
  };

export const PANTALLA_SIN_PERMISO: EstadoDeLaPantalla = {
    "bytesEnMemoria": 0,
    "vista": "sin-permiso"
  };

export const PANTALLA_ESPERANDO_LA_REUNION: EstadoDeLaPantalla = {
    "bytesEnMemoria": 0,
    "vista": "esperando-la-reunion"
  };

export const PANTALLA_NO_PUDO: EstadoDeLaPantalla = {
    "bytesEnMemoria": 0,
    "vista": "no-pudo"
  };

export const NOVEDAD_APARECE_POR_PANTALLA: Novedad = {
    "acumuladas": [
      {
        "texto": "Sur del Valle: cuatro fuentes en 9 semanas",
        "unidad": "caso"
      }
    ],
    "clase": "ficha",
    "fuente": {
      "conjeturada": false,
      "documento": "Páramo Azul · Propuesta",
      "seccion": "§3.2 Alcance",
      "unidad": "propuesta"
    },
    "hora": "14:02",
    "linea": "Cubre perfilado y limpieza de ERP, POS y Excel de canal.",
    "lineaLarga": "Cubre perfilado y limpieza de ERP, POS y Excel de canal; una cuarta fuente se cotiza aparte.",
    "motivo": "pantalla",
    "ms": 1240,
    "que": "aparece",
    "titular": "Limpieza de datos: incluida, hasta tres fuentes"
  };

export const NOVEDAD_NADA_EN_PANTALLA: Novedad = {
    "hora": "14:05",
    "que": "nada-en-pantalla"
  };

export const NOVEDAD_RADAR: Novedad = {
    "bots": [
      "MinutaBot"
    ],
    "grabando": true,
    "hora": "14:03",
    "que": "radar"
  };

export const EN_TU_MAC_VIGILADO: EnTuMac = {
    "catalogo": {
      "fecha": "2026-09-26",
      "version": 1
    },
    "programas": [
      {
        "alcance": {
          "en": "Camera, full screen, open apps; can block programs",
          "es": "Cámara, pantalla completa, apps abiertas; puede bloquear programas"
        },
        "categoria": "supervision",
        "nivel": "invasivo",
        "nombre": "ProctorLince",
        "ve": {
          "en": "sees your full screen and your camera",
          "es": "ve tu pantalla completa y tu cámara"
        }
      },
      {
        "alcance": {
          "en": "Can install, wipe and read configuration. Normal on company machines",
          "es": "Puede instalar, borrar y leer configuración. Normal en equipos de empresa"
        },
        "categoria": "mdm",
        "nivel": "sabelo",
        "nombre": "MDM-Corp",
        "ve": {
          "en": "can install, wipe and read configuration",
          "es": "puede instalar, borrar y leer la configuración"
        }
      }
    ]
  };

export const EN_TU_MAC_LIMPIO: EnTuMac = {
    "catalogo": {
      "fecha": "2026-09-26",
      "version": 1
    },
    "programas": []
  };

export const NOVEDAD_SUGERENCIA: Novedad = {
    "confianza": "media",
    "ficha": {
      "fuente": {
        "conjeturada": false,
        "documento": "Páramo Azul",
        "seccion": "§3.2 Alcance",
        "unidad": "propuesta"
      },
      "titular": "Limpieza de datos: incluida, hasta tres fuentes"
    },
    "linea": "Confirma que la limpieza incluye hasta tres fuentes; una cuarta es adicional y se cotiza aparte.",
    "nombre": "Modelo del sistema",
    "que": "sugerencia",
    "quien": "sistema",
    "titular": "Tres fuentes incluidas, la cuarta aparte"
  };

export const ESTADO_DE_LA_IA_NADIE: EstadoDeLaIa = {
    "api": {
      "encendida": false,
      "externo": "claude",
      "hayClave": false
    },
    "latenciaMs": null,
    "mesUsd": 0.0,
    "quien": null,
    "redactar": false,
    "reunionUsd": 0.0,
    "sistema": "apple-intelligence-apagado",
    "topeUsd": 10.0
  };

export const ESTADO_DE_LA_IA_CON_API: EstadoDeLaIa = {
    "api": {
      "encendida": true,
      "externo": "gemini",
      "hayClave": true
    },
    "latenciaMs": 1400,
    "mesUsd": 0.84,
    "quien": "api",
    "redactar": true,
    "reunionUsd": 0.031,
    "sistema": null,
    "topeUsd": 10.0
  };

export const POR_QUE_NO_REDACTA_APAGADO: PorQueNoRedacta = "apple-intelligence-apagado";

export const POR_QUE_NO_REDACTA_NO_COMPATIBLE: PorQueNoRedacta = "mac-no-compatible";

export const POR_QUE_NO_REDACTA_NO_DISPONIBLE: PorQueNoRedacta = "no-disponible";

export const POR_QUE_NO_REDACTA_DESCARGANDO: PorQueNoRedacta = "modelo-descargandose";

export const POR_QUE_NO_REDACTA_SIN_PUENTE: PorQueNoRedacta = "sin-puente";

export const POR_QUE_NO_REDACTA_SIN_CLAVE: PorQueNoRedacta = "sin-clave";

export const POR_QUE_NO_REDACTA_TOPE: PorQueNoRedacta = "tope-del-mes";

export const ESTADO_DEL_ACOPLE: EstadoDelAcople = {
    "acoplada": true
  };

export const LO_QUE_SALIO: LoQueSalio = {
    "caracteres": 84,
    "externo": "claude",
    "hora": "14:22",
    "sobre": "Alcance",
    "tapadas": 2,
    "trozos": [
      {
        "que": "texto",
        "texto": "El alcance de "
      },
      {
        "marcador": "[CLIENTE_1]",
        "original": "Páramo Azul",
        "que": "tapado"
      },
      {
        "que": "texto",
        "texto": " incluye tres fuentes. "
      },
      {
        "marcador": "[PERSONA_1]",
        "original": "Andrea Villalba",
        "que": "tapado"
      },
      {
        "que": "texto",
        "texto": " pregunta por una cuarta."
      }
    ],
    "usd": 0.004
  };

export const IDIOMAS_DE_PISTA: IdiomasDePista = {
    "cliente": "en-US",
    "consultor": "es-ES"
  };

export const VISTA_DEL_CUADERNO: VistaDelCuaderno = {
    "abierta": true,
    "acuerdos": [
      "Cuarta fuente: cotización aparte"
    ],
    "conservarMisTurnos": false,
    "fijadas": [
      {
        "documento": "Propuesta Páramo Azul",
        "seccion": "§3.2",
        "titular": "Limpieza de datos: hasta tres fuentes",
        "unidad": "propuesta"
      }
    ],
    "nota": "Piden la cuarta fuente (Excel de logística).\nFecha real: 12 semanas desde la firma.",
    "resumen": {
      "acuerdos": 1,
      "bytesAcuerdos": 33,
      "bytesFijadas": 64,
      "bytesNota": 84,
      "bytesTurnos": 0,
      "fijadas": 1,
      "parrafos": 2,
      "turnos": 0
    },
    "retencion": "90d"
  };

export const REUNION_GUARDADA: ReunionGuardada = {
    "archivo": "paramo-azul-2026-09-20.ghost",
    "bytes": 22528,
    "guardada": 1789900000,
    "vence": 1797676000
  };

export const REUNION_GUARDADA_PARA_SIEMPRE: ReunionGuardada = {
    "archivo": "reunion-2026-09-27-1402.ghost",
    "bytes": 3104,
    "guardada": 1790500000,
    "vence": 0
  };

export const REUNION_GUARDADA_AHORA: Guardada = {
    "archivo": "paramo-azul-2026-09-20.ghost",
    "bytes": 22528,
    "vence": 1797676000
  };

export const CONTENIDO_DE_REUNION: ContenidoDeReunion = {
    "acuerdos": [
      "Cuarta fuente: cotización aparte"
    ],
    "cliente": "Páramo Azul",
    "empezo": "2026-09-20 14:02",
    "fijadas": [
      {
        "documento": "Propuesta Páramo Azul",
        "seccion": "§3.2",
        "titular": "Limpieza de datos: hasta tres fuentes",
        "unidad": "propuesta"
      }
    ],
    "minutos": 47,
    "misTurnos": [
      {
        "hora": "14:05",
        "texto": "Te envío la cotización el lunes."
      }
    ],
    "nota": "Piden la cuarta fuente.",
    "version": 1
  };

export const QUE_SABE_TRANSCRIBIR: QueSabeTranscribir = {
    "idiomas": [
      {
        "codigo": "es-ES",
        "disponibilidad": {
          "estado": "listo"
        }
      },
      {
        "codigo": "en-US",
        "disponibilidad": {
          "estado": "sin-modelo"
        }
      }
    ],
    "motivo": null,
    "motor": "apple-speechanalyzer",
    "techo": 5
  };

export const QUE_SABE_TRANSCRIBIR_SIN_TRANSCRIPTOR: QueSabeTranscribir = {
    "idiomas": [],
    "motivo": "sin-transcriptor",
    "motor": "ninguno",
    "techo": 0
  };

export const QUE_SABE_TRANSCRIBIR_SIN_PUENTE: QueSabeTranscribir = {
    "idiomas": [],
    "motivo": "sin-puente",
    "motor": "ninguno",
    "techo": 0
  };

export const QUE_SABE_TRANSCRIBIR_NO_CONTESTA: QueSabeTranscribir = {
    "idiomas": [],
    "motivo": "no-contesta",
    "motor": "ninguno",
    "techo": 0
  };
