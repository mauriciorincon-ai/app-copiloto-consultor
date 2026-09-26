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
import type { Turno, Reunion, Permisos, EstadoDeEscucha, Disponibilidad, Salida, EstadoDelCorpus, InformeDelCorte, LaVoz } from "./cuaderno";
import type { EstadoDelAcople } from "./acople";

export const NOVEDAD_EMPIEZA: Novedad = {
    "pista": "microfono",
    "que": "empieza"
  };

export const NOVEDAD_TURNO: Novedad = {
    "desdeMs": 5400,
    "eco": false,
    "hastaMs": 8000,
    "hora": "14:02",
    "pista": "sistema",
    "que": "turno",
    "texto": "¿Y la limpieza de datos está dentro del alcance?"
  };

export const NOVEDAD_SIN_TEXTO: Novedad = {
    "desdeMs": 5400,
    "hastaMs": 8000,
    "motivo": "el motor no reconoció palabras en ese turno",
    "pista": "sistema",
    "que": "sin-texto"
  };

export const NOVEDAD_RUIDO: Novedad = {
    "duracionMs": 140,
    "pista": "microfono",
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
    "motivo": "hay un navegador abierto pero no se pueden leer sus ventanas",
    "que": "no-se-puede-saber"
  };

export const PERMISOS: Permisos = {
    "accesibilidad": "no-se-sabe",
    "microfono": "concedido",
    "pantalla": "sin-conceder"
  };

export const ESTADO_DE_LA_ESCUCHA: EstadoDeEscucha = {
    "bytesDelTranscript": 2048,
    "escuchando": true,
    "microfono": {
      "abierta": true,
      "bytes": 1920000,
      "hablando": false,
      "motivo": null,
      "muestrasRecibidas": 480000,
      "segundos": 30.0
    },
    "motor": "apple-speechanalyzer",
    "sistema": {
      "abierta": false,
      "bytes": 0,
      "hablando": false,
      "motivo": "este Mac no deja abrir el audio del sistema",
      "muestrasRecibidas": 0,
      "segundos": 0.0
    },
    "turnosEnMemoria": 3
  };

export const DISPONIBILIDAD_LISTO: Disponibilidad = {
    "estado": "listo"
  };

export const DISPONIBILIDAD_SIN_MOTOR: Disponibilidad = {
    "estado": "sin-motor",
    "motivo": "este Mac no trae el transcriptor"
  };

export const SALIDA_DE_AUDIO: Salida = {
    "salida": "altavoces"
  };

export const SALIDA_DE_AUDIO_OTRA: Salida = {
    "nombre": "AirPods Pro",
    "salida": "otra"
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
    "bytesEnRed": 0,
    "piezas": [
      [
        "voz",
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
        "aun-no-existe"
      ],
      [
        "transcript",
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

export const ESTADO_DEL_ACOPLE: EstadoDelAcople = {
    "acoplada": true,
    "permiso": true
  };
