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
import type { LaFranja } from "./franja";
import type { Preparacion, VistaDelEnsayo, Progreso, NoEmpezo } from "./ensayo";
import type { VistaDelCuaderno, LineaDePropuesta, Propuesta, Regla, Ventana, VistaDeLaBandeja, EstadoDeLaBandeja, ReunionGuardada, ListaDeReuniones } from "./notas";
import type { VistaDelCliente, LaBandera, Nda } from "./jurisdiccion";
import type { VistaDeLaPuerta, Cierre, NoAbre, Motivo } from "./puerta";

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
    "bytesDelEnsayo": 0,
    "bytesDelTranscript": 2048,
    "ensayo": false,
    "escuchando": true,
    "microfono": {
      "abierta": true,
      "bytes": 1920000,
      "motivo": null
    },
    "presencial": false,
    "sistema": {
      "abierta": false,
      "bytes": 0,
      "motivo": "dispositivo-ocupado"
    },
    "soloNotas": false
  };

export const ESTADO_DE_LA_ESCUCHA_EN_UN_ENSAYO: EstadoDeEscucha = {
    "bytesDelEnsayo": 412,
    "bytesDelTranscript": 0,
    "ensayo": true,
    "escuchando": false,
    "microfono": {
      "abierta": true,
      "bytes": 1920000,
      "motivo": null
    },
    "presencial": false,
    "sistema": {
      "abierta": false,
      "bytes": 0,
      "motivo": null
    },
    "soloNotas": false
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
        "ensayo",
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
        "propuestas",
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
    "enriquecer": false,
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
      "externo": "groq",
      "hayClave": true
    },
    "enriquecer": true,
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

export const LA_FRANJA_ARRIBA: LaFranja = {
    "avisoVisto": false,
    "barra": 38.0,
    "borde": "arriba"
  };

export const LA_FRANJA_ABAJO: LaFranja = {
    "avisoVisto": true,
    "barra": 25.0,
    "borde": "abajo"
  };

export const PREPARACION_DEL_ENSAYO: Preparacion = {
    "cliente": "Páramo Azul",
    "clientes": [
      "Páramo Azul",
      "Sur del Valle"
    ],
    "cuentas": {
      "ficha": 1,
      "objeciones": 2,
      "propuesta": 5
    },
    "enriquecer": false,
    "guardados": 4,
    "idioma": "es",
    "propuesta": "/Users/consultor/Corpus/Propuestas/Rentabilidad por canal · Páramo Azul.md",
    "propuestas": [
      {
        "nombre": "Rentabilidad por canal · Páramo Azul",
        "ruta": "/Users/consultor/Corpus/Propuestas/Rentabilidad por canal · Páramo Azul.md"
      }
    ],
    "sinCorpus": false,
    "tope": 8,
    "topes": [
      5,
      8,
      12
    ],
    "transcribe": true
  };

export const PREPARACION_SIN_CORPUS: Preparacion = {
    "cliente": "Sur del Valle",
    "clientes": [
      "Páramo Azul",
      "Sur del Valle"
    ],
    "cuentas": {
      "ficha": 0,
      "objeciones": 0,
      "propuesta": 0
    },
    "enriquecer": true,
    "guardados": 0,
    "idioma": "en",
    "propuesta": null,
    "propuestas": [],
    "sinCorpus": true,
    "tope": 8,
    "topes": [
      5,
      8,
      12
    ],
    "transcribe": false
  };

export const ENSAYO_PREGUNTANDO: VistaDelEnsayo = {
    "banco": {
      "que": "apagado"
    },
    "cerrando": false,
    "cliente": "Páramo Azul",
    "evaluacion": null,
    "fase": "preguntando",
    "indice": 2,
    "informe": null,
    "leyendo": true,
    "muletillas": null,
    "pregunta": {
      "de": "propuesta",
      "fuente": null,
      "seccion": "Supuestos",
      "texto": "¿Qué pasa con el plazo si el ERP no entrega los datos limpios a tiempo?"
    },
    "respuesta": "",
    "total": 8,
    "transcurridoMs": 0,
    "usadas": 0
  };

export const ENSAYO_EVALUADA: VistaDelEnsayo = {
    "banco": {
      "que": "apagado"
    },
    "cerrando": false,
    "cliente": "Páramo Azul",
    "evaluacion": {
      "evidencia": [
        {
          "citada": true,
          "dichaPorTi": false,
          "fuente": {
            "conjeturada": false,
            "documento": "Páramo Azul",
            "seccion": "Supuestos",
            "unidad": "propuesta"
          },
          "titular": "El plazo corre desde la entrega de datos"
        },
        {
          "citada": true,
          "dichaPorTi": false,
          "fuente": {
            "conjeturada": false,
            "documento": "Sur del Valle",
            "seccion": "cierre",
            "unidad": "caso"
          },
          "titular": "Sur del Valle: tres semanas por datos"
        },
        {
          "citada": false,
          "dichaPorTi": true,
          "fuente": {
            "conjeturada": true,
            "documento": "Marco de trabajo",
            "seccion": "etapa 2",
            "unidad": "marco"
          },
          "titular": "Limpiar antes del tablero"
        }
      ],
      "muletillas": [
        {
          "frase": "o sea",
          "veces": 3
        },
        {
          "frase": "básicamente",
          "veces": 1
        }
      ],
      "ppm": 142,
      "tiempoMs": 72000
    },
    "fase": "evaluada",
    "indice": 2,
    "informe": null,
    "leyendo": false,
    "muletillas": 4,
    "pregunta": {
      "de": "propuesta",
      "fuente": null,
      "seccion": "Supuestos",
      "texto": "¿Qué pasa con el plazo si el ERP no entrega los datos limpios a tiempo?"
    },
    "respuesta": "El supuesto dos lo cubre: el plazo corre desde que el ERP entrega los datos.",
    "total": 8,
    "transcurridoMs": 72000,
    "usadas": 3
  };

export const ENSAYO_CERRADO: VistaDelEnsayo = {
    "banco": {
      "que": "apagado"
    },
    "cerrando": false,
    "cliente": "Páramo Azul",
    "evaluacion": null,
    "fase": "cerrado",
    "indice": 2,
    "informe": {
      "citadas": 14,
      "evidencia": 21,
      "filas": [
        {
          "citadas": 2,
          "evidencia": 3,
          "numero": 3,
          "ppm": 142,
          "saltada": false,
          "texto": "¿Qué pasa con el plazo si el ERP no entrega los datos limpios a tiempo?",
          "tiempoMs": 72000
        },
        {
          "citadas": 0,
          "evidencia": 0,
          "numero": 4,
          "ppm": null,
          "saltada": true,
          "texto": "¿Quién lo va a usar cuando ustedes se vayan?",
          "tiempoMs": null
        }
      ],
      "laQueMas": {
        "frase": "o sea",
        "veces": 5
      },
      "muletillas": 9,
      "ppmMedio": 138,
      "respondidas": 7,
      "saltadas": 1,
      "tiempoMedioMs": 58000
    },
    "leyendo": false,
    "muletillas": null,
    "pregunta": null,
    "respuesta": "",
    "total": 8,
    "transcurridoMs": 0,
    "usadas": 0
  };

export const ENSAYO_DEL_MODELO: VistaDelEnsayo = {
    "banco": {
      "cuantas": 2,
      "que": "sumadas"
    },
    "cerrando": false,
    "cliente": "Páramo Azul",
    "evaluacion": null,
    "fase": "respondiendo",
    "indice": 2,
    "informe": null,
    "leyendo": false,
    "muletillas": null,
    "pregunta": {
      "de": "modelo",
      "fuente": null,
      "seccion": "Quién decide",
      "texto": "¿Quién firma por parte de la gerencia general?"
    },
    "respuesta": "",
    "total": 8,
    "transcurridoMs": 0,
    "usadas": 0
  };

export const ENSAYO_OBJECION: VistaDelEnsayo = {
    "banco": {
      "porque": "nada-fundado",
      "que": "no-se-enriquecio"
    },
    "cerrando": false,
    "cliente": "Páramo Azul",
    "evaluacion": null,
    "fase": "respondiendo",
    "indice": 2,
    "informe": null,
    "leyendo": false,
    "muletillas": null,
    "pregunta": {
      "de": "objeciones",
      "fuente": "Kuznetsova",
      "seccion": null,
      "texto": "¿Estos números son correctos?"
    },
    "respuesta": "",
    "total": 8,
    "transcurridoMs": 0,
    "usadas": 0
  };

export const PROGRESO_DEL_ENSAYO: Progreso = {
    "antes": 0,
    "cliente": "Páramo Azul",
    "desdeElPrimero": {
      "evidencia": {
        "desde": 9,
        "hasta": 14
      },
      "muletillas": {
        "desde": 17,
        "hasta": 9
      },
      "ritmo": {
        "desde": 161,
        "hasta": 138
      },
      "tiempo": {
        "desde": 81000,
        "hasta": 58000
      }
    },
    "filas": [
      {
        "citadas": 9,
        "empezo": "2026-09-21 10:05",
        "evidencia": 21,
        "muletillas": 17,
        "ppmMedio": 161,
        "tiempoMedioMs": 81000
      },
      {
        "citadas": 11,
        "empezo": "2026-09-27 18:30",
        "evidencia": 21,
        "muletillas": 12,
        "ppmMedio": 150,
        "tiempoMedioMs": 69000
      },
      {
        "citadas": 12,
        "empezo": "2026-10-02 08:45",
        "evidencia": 21,
        "muletillas": 11,
        "ppmMedio": 143,
        "tiempoMedioMs": 62000
      },
      {
        "citadas": 14,
        "empezo": "2026-10-04 09:12",
        "evidencia": 21,
        "muletillas": 9,
        "ppmMedio": 138,
        "tiempoMedioMs": 58000
      }
    ]
  };

export const PROGRESO_DE_UN_ENSAYO: Progreso = {
    "antes": 0,
    "cliente": "Páramo Azul",
    "desdeElPrimero": {
      "evidencia": null,
      "muletillas": null,
      "ritmo": null,
      "tiempo": null
    },
    "filas": [
      {
        "citadas": 14,
        "empezo": "2026-10-04 09:12",
        "evidencia": 21,
        "muletillas": 9,
        "ppmMedio": 138,
        "tiempoMedioMs": 58000
      }
    ]
  };

export const PROGRESO_CON_MAS: Progreso = {
    "antes": 3,
    "cliente": "Páramo Azul",
    "desdeElPrimero": {
      "evidencia": {
        "desde": 9,
        "hasta": 14
      },
      "muletillas": {
        "desde": 17,
        "hasta": 9
      },
      "ritmo": {
        "desde": 150,
        "hasta": 138
      },
      "tiempo": {
        "desde": 81000,
        "hasta": 58000
      }
    },
    "filas": [
      {
        "citadas": 9,
        "empezo": "2026-09-21 10:05",
        "evidencia": 21,
        "muletillas": null,
        "ppmMedio": null,
        "tiempoMedioMs": 81000
      },
      {
        "citadas": 11,
        "empezo": "2026-09-27 18:30",
        "evidencia": 21,
        "muletillas": 12,
        "ppmMedio": 150,
        "tiempoMedioMs": 69000
      },
      {
        "citadas": 12,
        "empezo": "2026-10-02 08:45",
        "evidencia": 21,
        "muletillas": 11,
        "ppmMedio": 143,
        "tiempoMedioMs": 62000
      },
      {
        "citadas": 14,
        "empezo": "2026-10-04 09:12",
        "evidencia": 21,
        "muletillas": 9,
        "ppmMedio": 138,
        "tiempoMedioMs": 58000
      }
    ]
  };

export const NO_EMPEZO_EN_REUNION: NoEmpezo = {
    "que": "en-reunion"
  };

export const NO_EMPEZO_SIN_CORPUS: NoEmpezo = {
    "que": "sin-corpus"
  };

export const NO_EMPEZO_MICROFONO: NoEmpezo = {
    "porque": "sin-permiso-del-microfono",
    "que": "microfono"
  };

export const NO_EMPEZO_VIDEOLLAMADA: NoEmpezo = {
    "que": "videollamada"
  };

export const NO_EMPEZO_NO_SE_SABE_SI_HAY_LLAMADA: NoEmpezo = {
    "que": "no-se-sabe-si-hay-llamada"
  };

export const ENSAYO_REPETIDAS: VistaDelEnsayo = {
    "banco": {
      "porque": "repetidas",
      "que": "no-se-enriquecio"
    },
    "cerrando": false,
    "cliente": "Páramo Azul",
    "evaluacion": null,
    "fase": "preguntando",
    "indice": 2,
    "informe": null,
    "leyendo": true,
    "muletillas": null,
    "pregunta": {
      "de": "propuesta",
      "fuente": null,
      "seccion": "Supuestos",
      "texto": "¿Qué pasa con el plazo si el ERP no entrega los datos limpios a tiempo?"
    },
    "respuesta": "",
    "total": 8,
    "transcurridoMs": 0,
    "usadas": 0
  };

export const ENSAYO_SIN_SECCIONES: VistaDelEnsayo = {
    "banco": {
      "porque": "sin-secciones",
      "que": "no-se-enriquecio"
    },
    "cerrando": false,
    "cliente": "Páramo Azul",
    "evaluacion": null,
    "fase": "preguntando",
    "indice": 2,
    "informe": null,
    "leyendo": true,
    "muletillas": null,
    "pregunta": {
      "de": "propuesta",
      "fuente": null,
      "seccion": "Supuestos",
      "texto": "¿Qué pasa con el plazo si el ERP no entrega los datos limpios a tiempo?"
    },
    "respuesta": "",
    "total": 8,
    "transcurridoMs": 0,
    "usadas": 0
  };

export const LO_QUE_SALIO: LoQueSalio = {
    "caracteres": 84,
    "externo": "claude",
    "hora": "14:22",
    "para": "sugerencia",
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

export const LO_QUE_SALIO_DEL_BANCO: LoQueSalio = {
    "caracteres": 51,
    "externo": "claude",
    "hora": "09:12",
    "para": "banco",
    "sobre": "Propuesta Páramo Azul",
    "tapadas": 0,
    "trozos": [
      {
        "que": "texto",
        "texto": "S1 · Alcance: perfilado y limpieza de tres fuentes."
      }
    ],
    "usd": null
  };

export const IDIOMAS_DE_PISTA: IdiomasDePista = {
    "cliente": "en-US",
    "consultor": "es-ES",
    "sala": "en-US"
  };

export const VISTA_DEL_CUADERNO: VistaDelCuaderno = {
    "abierta": true,
    "acuerdos": [
      "Cuarta fuente: cotización aparte"
    ],
    "conservarMisTurnos": false,
    "escuchando": false,
    "fijadas": [
      {
        "documento": "Propuesta Páramo Azul",
        "seccion": "§3.2",
        "titular": "Limpieza de datos: hasta tres fuentes",
        "unidad": "propuesta"
      }
    ],
    "lecturas": 9,
    "lleno": false,
    "nota": "Piden la cuarta fuente (Excel de logística).\nFecha real: 12 semanas desde la firma.",
    "previsto": {
      "archivo": "reunion-2026-09-20-1402.ghost",
      "cliente": null,
      "fecha": "2026-09-20",
      "minutos": 47
    },
    "propuestas": [],
    "resumen": {
      "acuerdos": 1,
      "bytesAcuerdos": 33,
      "bytesFijadas": 64,
      "bytesNota": 84,
      "bytesTurnos": 0,
      "fijadas": 1,
      "parrafos": 2,
      "propuestas": 0,
      "sinDecidir": 0,
      "turnos": 0
    },
    "retencion": "90d",
    "sinProteger": false,
    "turnosDelCliente": 63,
    "ventana": "3h"
  };

export const VISTA_CON_PROPUESTAS: VistaDelCuaderno = {
    "abierta": true,
    "acuerdos": [],
    "conservarMisTurnos": false,
    "escuchando": true,
    "fijadas": [],
    "lecturas": 0,
    "lleno": true,
    "nota": "Piden la cuarta fuente.",
    "previsto": null,
    "propuestas": [
      {
        "de": "tuyo",
        "ficha": null,
        "hora": "14:16",
        "id": 1,
        "regla": "cifra",
        "seccion": null,
        "texto": "Fecha real del tablero: 12 semanas desde la firma."
      },
      {
        "de": "cliente",
        "ficha": "tres",
        "hora": "14:18",
        "id": 2,
        "regla": "choque",
        "seccion": "§3.2",
        "texto": "cuatro fuentes"
      }
    ],
    "resumen": {
      "acuerdos": 0,
      "bytesAcuerdos": 0,
      "bytesFijadas": 0,
      "bytesNota": 23,
      "bytesTurnos": 0,
      "fijadas": 0,
      "parrafos": 1,
      "propuestas": 0,
      "sinDecidir": 2,
      "turnos": 0
    },
    "retencion": "90d",
    "sinProteger": false,
    "turnosDelCliente": 12,
    "ventana": "0"
  };

export const CUADERNO_SIN_PROTEGER: VistaDelCuaderno = {
    "abierta": true,
    "acuerdos": [],
    "conservarMisTurnos": false,
    "escuchando": true,
    "fijadas": [],
    "lecturas": 0,
    "lleno": false,
    "nota": "Piden la cuarta fuente.",
    "previsto": null,
    "propuestas": [],
    "resumen": {
      "acuerdos": 0,
      "bytesAcuerdos": 0,
      "bytesFijadas": 0,
      "bytesNota": 23,
      "bytesTurnos": 0,
      "fijadas": 0,
      "parrafos": 1,
      "propuestas": 0,
      "sinDecidir": 0,
      "turnos": 0
    },
    "retencion": "90d",
    "sinProteger": true,
    "turnosDelCliente": 3,
    "ventana": "3h"
  };

export const PROPUESTA_EN_LA_BANDA: LineaDePropuesta = {
    "de": "tuyo",
    "ficha": null,
    "hora": "14:16",
    "regla": "cifra",
    "seccion": null,
    "texto": "Fecha real del tablero: 12 semanas desde la firma."
  };

export const SIN_PROPUESTA_EN_LA_BANDA: LineaDePropuesta = null;

export const PROPUESTA_CHOQUE: Propuesta = {
    "de": "cliente",
    "ficha": "tres",
    "hora": "14:18",
    "regla": "choque",
    "seccion": "§3.2",
    "texto": "cuatro fuentes"
  };

export const REGLA_CIFRA: Regla = "cifra";

export const REGLA_COMPROMISO: Regla = "compromiso";

export const REGLA_CHOQUE: Regla = "choque";

export const REGLA_NOMBRE: Regla = "nombre";

export const REGLA_PREGUNTA: Regla = "pregunta";

export const VENTANA_AL_CERRAR: Ventana = "0";

export const VENTANA_UNA_HORA: Ventana = "1h";

export const VENTANA_TRES_HORAS: Ventana = "3h";

export const VENTANA_FIN_DEL_DIA: Ventana = "fin";

export const VENTANA_UN_DIA: Ventana = "24h";

export const BANDEJA_ABIERTA: VistaDeLaBandeja = {
    "archivo": "reunion-2026-09-20-1402.ghost",
    "bytes": 4096,
    "guardadas": [
      {
        "de": "tuyo",
        "ficha": null,
        "hora": "14:16",
        "regla": "cifra",
        "seccion": null,
        "texto": "La cuarta fuente se cotiza aparte."
      }
    ],
    "mas": 0,
    "propuestas": [
      {
        "de": "tuyo",
        "ficha": null,
        "hora": "14:16",
        "regla": "cifra",
        "seccion": null,
        "texto": "Fecha real del tablero: 12 semanas desde la firma."
      },
      {
        "de": "cliente",
        "ficha": "tres",
        "hora": "14:18",
        "regla": "choque",
        "seccion": "§3.2",
        "texto": "cuatro fuentes"
      }
    ],
    "vence": 1790527268,
    "ventana": "3h"
  };

export const BANDEJA_CON_LLAVE: VistaDeLaBandeja = {
    "archivo": "reunion-2026-09-20-1402.ghost",
    "bytes": 4096,
    "guardadas": [],
    "mas": 1,
    "propuestas": null,
    "vence": 1790527268,
    "ventana": "fin"
  };

export const ESTADO_DE_LA_BANDEJA: EstadoDeLaBandeja = {
    "noCorrio": false,
    "vence": 1790527268
  };

export const SIN_BANDEJA_Y_LA_TAREA_NO_CORRIO: EstadoDeLaBandeja = {
    "noCorrio": true,
    "vence": null
  };

export const VISTA_DEL_CLIENTE: VistaDelCliente = {
    "bandera": {
      "bandera": {
        "consultado": "2026-09-17",
        "implica": {
          "en": "Handle it at the engagement level: contract clause + NDA check.",
          "es": "Resuélvelo en el encargo: cláusula del contrato + chequeo de NDA."
        },
        "nombre": {
          "en": "Colombia",
          "es": "Colombia"
        },
        "normas": [
          "CSJ AP1465-2018",
          "Ley 1581 art. 3"
        ],
        "pendiente": {
          "en": "CSJ AP1465-2018 was read in a secondary source (G-1)",
          "es": "la CSJ AP1465-2018 se leyó en una fuente secundaria (G-1)"
        },
        "regla": {
          "en": "Listening as a participant is lawful; ephemeral transcription is probably “processing”.",
          "es": "Escuchar como participante es lícito; la transcripción efímera probablemente es «tratamiento»."
        },
        "riesgo": "bajo-medio"
      },
      "que": "conocida"
    },
    "clausula": {
      "en": "Local AI assistance. During meetings under this engagement, the Consultant may use, on their own computer, an artificial intelligence assistant that listens to and transcribes the conversation only in that computer’s memory, to search the Consultant’s own documents. The assistant does not record audio or keep transcripts or screenshots, does not identify anyone by their voice and does not infer emotions. By default nothing leaves the computer; if the Consultant turns on an external provider, only text fragments are sent to it, in which the assistant replaces the names, emails, phone numbers and numbers it recognises; the provider does not train on them and may keep them for up to 30 days. Kept, encrypted and under the Consultant’s responsibility, are the Consultant’s notes, what the Consultant said if the Consultant so chooses, and the one-line suggestions about the meeting that the Consultant accepts; those not decided on delete themselves within 24 hours at most. If the Client prefers it not be used in a meeting, saying so is enough.",
      "es": "Asistencia de IA local. Durante las reuniones de este encargo, el Consultor puede usar en su propio equipo un asistente de inteligencia artificial que escucha y transcribe la conversación solo en la memoria del equipo, para buscar en sus propios documentos. El asistente no graba audio ni guarda transcripciones o capturas de pantalla, no identifica a nadie por su voz y no infiere emociones. Por defecto nada sale del equipo; si el Consultor activa un proveedor externo, solo le envía fragmentos de texto en los que el asistente reemplaza los nombres, correos, teléfonos y números que reconoce; el proveedor no entrena con ellos y puede conservarlos hasta 30 días. Se conservan, cifradas y bajo responsabilidad del Consultor, sus notas, lo que él mismo dijo si así lo elige y las propuestas de una línea sobre la reunión que él acepte; las que no decide se borran solas en 24 horas como mucho. Si el Cliente prefiere que no se use en una reunión, basta con decirlo."
    },
    "clientes": [
      "Páramo Azul",
      "Sur del Valle"
    ],
    "elegido": "Páramo Azul",
    "nda": "sin-revisar"
  };

export const VISTA_DEL_CLIENTE_SIN_ELEGIR: VistaDelCliente = {
    "bandera": null,
    "clausula": {
      "en": "Local AI assistance. During meetings under this engagement, the Consultant may use, on their own computer, an artificial intelligence assistant that listens to and transcribes the conversation only in that computer’s memory, to search the Consultant’s own documents. The assistant does not record audio or keep transcripts or screenshots, does not identify anyone by their voice and does not infer emotions. By default nothing leaves the computer; if the Consultant turns on an external provider, only text fragments are sent to it, in which the assistant replaces the names, emails, phone numbers and numbers it recognises; the provider does not train on them and may keep them for up to 30 days. Kept, encrypted and under the Consultant’s responsibility, are the Consultant’s notes, what the Consultant said if the Consultant so chooses, and the one-line suggestions about the meeting that the Consultant accepts; those not decided on delete themselves within 24 hours at most. If the Client prefers it not be used in a meeting, saying so is enough.",
      "es": "Asistencia de IA local. Durante las reuniones de este encargo, el Consultor puede usar en su propio equipo un asistente de inteligencia artificial que escucha y transcribe la conversación solo en la memoria del equipo, para buscar en sus propios documentos. El asistente no graba audio ni guarda transcripciones o capturas de pantalla, no identifica a nadie por su voz y no infiere emociones. Por defecto nada sale del equipo; si el Consultor activa un proveedor externo, solo le envía fragmentos de texto en los que el asistente reemplaza los nombres, correos, teléfonos y números que reconoce; el proveedor no entrena con ellos y puede conservarlos hasta 30 días. Se conservan, cifradas y bajo responsabilidad del Consultor, sus notas, lo que él mismo dijo si así lo elige y las propuestas de una línea sobre la reunión que él acepte; las que no decide se borran solas en 24 horas como mucho. Si el Cliente prefiere que no se use en una reunión, basta con decirlo."
    },
    "clientes": [
      "Páramo Azul",
      "Sur del Valle"
    ],
    "elegido": null,
    "nda": "sin-revisar"
  };

export const BANDERA_CONOCIDA: LaBandera = {
    "bandera": {
      "consultado": "2026-09-17",
      "implica": {
        "en": "Handle it at the engagement level: contract clause + NDA check.",
        "es": "Resuélvelo en el encargo: cláusula del contrato + chequeo de NDA."
      },
      "nombre": {
        "en": "Colombia",
        "es": "Colombia"
      },
      "normas": [
        "CSJ AP1465-2018",
        "Ley 1581 art. 3"
      ],
      "pendiente": {
        "en": "CSJ AP1465-2018 was read in a secondary source (G-1)",
        "es": "la CSJ AP1465-2018 se leyó en una fuente secundaria (G-1)"
      },
      "regla": {
        "en": "Listening as a participant is lawful; ephemeral transcription is probably “processing”.",
        "es": "Escuchar como participante es lícito; la transcripción efímera probablemente es «tratamiento»."
      },
      "riesgo": "bajo-medio"
    },
    "que": "conocida"
  };

export const BANDERA_CON_PENDIENTE: LaBandera = {
    "bandera": {
      "consultado": "2026-09-17",
      "implica": {
        "en": "Suggested: one-line notice to the client or notes-only mode.",
        "es": "Sugerido: aviso de una línea al cliente o modo solo notas."
      },
      "nombre": {
        "en": "USA · California (all-party)",
        "es": "EE. UU. · California (todas las partes)"
      },
      "normas": [
        "Cal. Penal Code § 632(a)"
      ],
      "pendiente": {
        "en": "there is no precedent on a participant who transcribes without recording (G-6)",
        "es": "no hay precedente sobre quien participa y transcribe sin grabar (G-6)"
      },
      "regla": {
        "en": "Requires all parties and punishes “eavesdrop upon or record” with a device, not just recording.",
        "es": "Exige a todas las partes y castiga «escuchar o grabar» con un aparato, no solo grabar."
      },
      "riesgo": "medio-alto"
    },
    "que": "conocida"
  };

export const BANDERA_SIN_VERIFICAR: LaBandera = {
    "bandera": {
      "consultado": "2026-09-17",
      "implica": {
        "en": "Check with a lawyer before the meeting: the app asserts nothing here.",
        "es": "Consúltalo con un abogado antes de la reunión: la app no afirma nada aquí."
      },
      "nombre": {
        "en": "USA · Missouri",
        "es": "EE. UU. · Missouri"
      },
      "normas": [
        "RCFP Reporter’s Recording Guide"
      ],
      "pendiente": {
        "en": "its statute is not in the report",
        "es": "su estatuto no está en el informe"
      },
      "regla": {
        "en": "The report lists it as mixed, without verifying its statute.",
        "es": "El informe lo nombra entre los de regla mixta, sin verificar su estatuto."
      },
      "riesgo": "sin-verificar"
    },
    "que": "conocida"
  };

export const BANDERA_FUERA_DEL_CATALOGO: LaBandera = {
    "escrita": "Bolivia",
    "que": "fuera-del-catalogo",
    "version": 1
  };

export const BANDERA_SIN_INDICAR: LaBandera = {
    "que": "sin-indicar"
  };

export const NDA_SIN_REVISAR: Nda = "sin-revisar";

export const NDA_NO_LO_PROHIBE: Nda = "no-lo-prohibe";

export const NDA_LO_PROHIBE: Nda = "lo-prohibe";

export const ESCUCHA_SOLO_NOTAS: EstadoDeEscucha = {
    "bytesDelEnsayo": 0,
    "bytesDelTranscript": 0,
    "ensayo": false,
    "escuchando": false,
    "microfono": {
      "abierta": false,
      "bytes": 0,
      "motivo": null
    },
    "presencial": false,
    "sistema": {
      "abierta": false,
      "bytes": 0,
      "motivo": null
    },
    "soloNotas": true
  };

export const VISTA_DE_LA_PUERTA_CERRADA: VistaDeLaPuerta = {
    "abierta": false,
    "cerro": null,
    "ghost": null,
    "noAbre": null,
    "registro": []
  };

export const VISTA_DE_LA_PUERTA_ABIERTA: VistaDeLaPuerta = {
    "abierta": true,
    "cerro": null,
    "ghost": "/Users/ana/app-copiloto-consultor/src-tauri/target/debug/ghost",
    "noAbre": null,
    "registro": [
      {
        "hora": "11:12",
        "orden": "ghost notas abrir",
        "resultado": {
          "que": "fallo"
        }
      },
      {
        "hora": "11:09",
        "orden": "ghost ia --encender-api",
        "resultado": {
          "motivo": "el-api-es-tuyo",
          "que": "denegado"
        }
      },
      {
        "hora": "11:04",
        "orden": "ghost corpus reindexar",
        "resultado": {
          "cuenta": 28,
          "que": "hecho"
        }
      }
    ]
  };

export const VISTA_DE_LA_PUERTA_EN_REUNION: VistaDeLaPuerta = {
    "abierta": false,
    "cerro": "en-reunion",
    "ghost": "/Users/ana/app-copiloto-consultor/src-tauri/target/debug/ghost",
    "noAbre": null,
    "registro": [
      {
        "hora": "14:02",
        "orden": "ghost corpus buscar",
        "resultado": {
          "motivo": "en-reunion",
          "que": "denegado"
        }
      },
      {
        "hora": "13:58",
        "orden": "ghost corpus buscar",
        "resultado": {
          "cuenta": 3,
          "que": "hecho"
        }
      }
    ]
  };

export const VISTA_DE_LA_PUERTA_SIN_GHOST: VistaDeLaPuerta = {
    "abierta": true,
    "cerro": null,
    "ghost": null,
    "noAbre": null,
    "registro": []
  };

export const PUERTA_CERRADA_A_MANO: Cierre = "a-tu-mano";

export const PUERTA_NO_ABRE_EN_REUNION: NoAbre = "en-reunion";

export const PUERTA_NO_ABRE_RUTA_LARGA: NoAbre = "ruta-larga";

export const PUERTA_NO_ABRE_LLAVERO: NoAbre = "llavero";

export const PUERTA_NO_ABRE_SOCKET: NoAbre = "socket";

export const PUERTA_LLAVE_ERRADA: Motivo = "llave-errada";

export const PUERTA_NO_DELEGABLE: Motivo = "no-delegable";

export const PUERTA_ORDEN_DESCONOCIDA: Motivo = "orden-desconocida";

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

export const LISTA_DE_REUNIONES: ListaDeReuniones = {
    "reuniones": []
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
