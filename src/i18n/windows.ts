import { useIdioma, type Idioma } from "./index";
import type { ClaveDeWindows } from "../plataforma";

/**
 * LOS TEXTOS DE WINDOWS (sprint 006, ADR 022, decisión del usuario: «diccionario aparte»).
 *
 * Viven fuera de `es.ts` y `en.ts` a propósito: esos dos están atados a la maqueta por el gate
 * `i18n-fiel-a-la-maqueta`, y estos textos no existen en ninguna maqueta. Lo que dice cada uno está
 * escrito en el ADR 022 («Hoy en Windows») y lo comprueba `tests/unit/windows-honestidad-y-manual.test.ts`.
 *
 * **Estado: maquetados, no vistos.** Estos textos no los ha mirado el usuario en un PC con Windows.
 * Van al gate humano del hito, con su línea en el cierre.
 *
 * Las frases no prometen lo que no se ha verificado: la invisibilidad en Teams, Zoom y Meet sigue
 * «sin verificar» hasta la prueba del usuario (regla dura 6).
 */

export type TextosDeWindows = {
  titulo: string;
  intro: string;
  ya: string;
  todavia: string;
  noPorAhora: string;
  /** Lo que cada clave es, para quien la lee. */
  funciones: Record<ClaveDeWindows, { nombre: string; detalle: string }>;
  /** La frase de cada estado de la transcripción. `cifra` llega ya formateada con el separador del idioma. */
  whisper: {
    titulo: string;
    bajando: string;
    sinModelo: string;
    sinMedir: (modelo: string) => string;
    aTiempo: (modelo: string, ms: string) => string;
    lento: (modelo: string, ms: string) => string;
    medirOtra: string;
    bajarPequeno: string;
    sinReunion: string;
  };
  /** Idioma: el motor de voz es de la app, no del sistema (ADR 022). */
  idioma: {
    transcribeTuPc: string;
    transcribeTuPcDetalle: string;
    idiomasListos: string;
  };
  /** Notas y la bandeja: cuándo se borra lo guardado (ADR 022: el borrado con la app cerrada, todavía no). */
  notas: {
    cadaReunion: string;
    enLaBandejaDetalle: string;
  };
  /** Honestidad: qué abre la app y cuándo. */
  honestidad: {
    modoDetalle: string;
  };
  permisos: {
    microfonoPara: string;
    abrirAjustes: string;
    llegaDespues: string;
    antesTitulo: string;
    antesTexto: string;
  };
};

const ES: TextosDeWindows = {
  titulo: "Hoy en Windows",
  intro:
    "Angel Ghost ya se instala en Windows. Esto es lo que hace hoy en este PC y lo que todavía no, dicho tal cual.",
  ya: "Ya funciona",
  todavia: "Todavía no",
  noPorAhora: "No en Windows por ahora",
  funciones: {
    "banda-protegida": {
      nombre: "La banda protegida",
      detalle:
        "sí, sin verificar en Teams, Zoom ni Meet hasta que la pruebes en tu PC",
    },
    presencial: {
      nombre: "El modo presencial: la sala por el micrófono",
      detalle: "sí, con transcripción local (whisper)",
    },
    "corpus-y-ficha": { nombre: "Tu corpus y las fichas", detalle: "sí" },
    "notas-cifradas": {
      nombre: "Tus notas cifradas",
      detalle:
        "sí: se guardan cifradas con la llave en el Administrador de credenciales; exportarlas y abrir la bandeja esperan a Windows Hello",
    },
    videollamada: {
      nombre: "Las videollamadas: el audio del cliente",
      detalle: "llega en una versión próxima",
    },
    pantalla: {
      nombre: "Leer la pantalla de la reunión",
      detalle: "llega en una versión próxima",
    },
    "voz-al-oido": {
      nombre: "La ficha al oído",
      detalle: "llega en una versión próxima",
    },
    acople: {
      nombre: "Acoplarse a la ventana de la reunión",
      detalle: "llega en una versión próxima",
    },
    ensayo: {
      nombre: "Ensayar con la voz que lee las preguntas",
      detalle: "llega en una versión próxima",
    },
    sugerencia: {
      nombre: "La sugerencia redactada por IA",
      detalle: "llega en una versión próxima",
    },
    radar: {
      nombre: "El radar de tu equipo",
      detalle: "llega en una versión próxima",
    },
    atajos: {
      nombre: "Los atajos de teclado globales",
      detalle:
        "llega en una versión próxima; en muchos teclados Ctrl+Alt es AltGr",
    },
    "borrado-con-la-app-cerrada": {
      nombre: "Borrar tus notas con la app cerrada",
      detalle: "por ahora se borran con la app abierta",
    },
    desbloqueo: {
      nombre: "Abrir tus notas guardadas",
      detalle: "llega con Windows Hello, en una versión próxima",
    },
    "puerta-local": {
      nombre: "La puerta para Claude Code",
      detalle: "no en Windows por ahora",
    },
  },
  whisper: {
    titulo: "La transcripción en este PC",
    bajando: "Bajando el modelo de voz (1,6 GB, con su comprobación)…",
    sinModelo:
      "No hay modelo de voz. «Instalar» en Idioma lo baja: 1,6 GB, y se comprueba su huella antes de usarlo.",
    sinMedir: (m) => `Modelo ${m}: sin medir en esta sesión.`,
    aTiempo: (m, ms) =>
      `Modelo ${m}: 10 s de audio en ${ms} ms. El techo son 4 s.`,
    lento: (m, ms) =>
      `Modelo ${m}: tardó ${ms} ms, y pasa del techo de 4 s en este PC. El modelo pequeño es menos preciso, pero más rápido aquí.`,
    medirOtra: "Medir otra vez",
    bajarPequeno: "Bajar el modelo pequeño (0,5 GB)",
    sinReunion: "Bajar o medir no se puede con una reunión abierta.",
  },
  idioma: {
    transcribeTuPc: "Transcribe tu PC, no un servicio",
    transcribeTuPcDetalle:
      "El modelo de voz (whisper), dentro de tu equipo. Ningún audio sale para convertirse en texto. La única vez que toca la red es cuando pides instalar el modelo: lo baja Angel Ghost desde su origen y comprueba su huella antes de usarlo.",
    idiomasListos: "idiomas a la vez, como mucho: los que la app ofrece.",
  },
  notas: {
    cadaReunion:
      "Cada reunión que guardas vive eso, y se borra sola al vencer, en cuanto abras la app. Con la app cerrada, todavía no.",
    enLaBandejaDetalle: "Estas, cifradas, se borran al llegar a cero, en cuanto abras la app.",
  },
  honestidad: {
    modoDetalle:
      "La app abre una conexión solo si enciendes el API en IA, con tu clave, y solo con texto anonimizado. La otra la abre la propia app, una vez, cuando pides instalar el modelo de voz: 1,6 GB, comprobados antes de usarse.",
  },
  permisos: {
    microfonoPara: "Escuchar la sala o tu voz, con el micrófono de este PC",
    abrirAjustes: "Abrir Ajustes de Windows",
    llegaDespues: "Llega en una versión próxima",
    antesTitulo: "El micrófono en Windows",
    antesTexto:
      "Lo concedes o lo quitas tú en Ajustes → Privacidad y seguridad → Micrófono. La app no pide el permiso por su cuenta.",
  },
};

const EN: TextosDeWindows = {
  titulo: "On Windows today",
  intro:
    "Angel Ghost already installs on Windows. This is what it does today on this PC, and what it doesn't do yet, stated plainly.",
  ya: "Works now",
  todavia: "Not yet",
  noPorAhora: "Not on Windows for now",
  funciones: {
    "banda-protegida": {
      nombre: "The protected band",
      detalle:
        "yes, unverified on Teams, Zoom or Meet until you test it on your PC",
    },
    presencial: {
      nombre: "In-person mode: the room through the microphone",
      detalle: "yes, with local transcription (whisper)",
    },
    "corpus-y-ficha": {
      nombre: "Your corpus and the evidence cards",
      detalle: "yes",
    },
    "notas-cifradas": {
      nombre: "Your encrypted notes",
      detalle:
        "yes: they're saved encrypted with the key in Credential Manager; exporting them and opening the tray wait for Windows Hello",
    },
    videollamada: {
      nombre: "Video calls: the client's audio",
      detalle: "coming in a later version",
    },
    pantalla: {
      nombre: "Reading the meeting's screen",
      detalle: "coming in a later version",
    },
    "voz-al-oido": {
      nombre: "The card read to your ear",
      detalle: "coming in a later version",
    },
    acople: {
      nombre: "Docking to the meeting window",
      detalle: "coming in a later version",
    },
    ensayo: {
      nombre: "Rehearsing with a voice that reads the questions",
      detalle: "coming in a later version",
    },
    sugerencia: {
      nombre: "The suggestion written by AI",
      detalle: "coming in a later version",
    },
    radar: {
      nombre: "Your machine's radar",
      detalle: "coming in a later version",
    },
    atajos: {
      nombre: "Global keyboard shortcuts",
      detalle:
        "coming in a later version; on many keyboard layouts Ctrl+Alt is AltGr",
    },
    "borrado-con-la-app-cerrada": {
      nombre: "Deleting your notes with the app closed",
      detalle: "for now they are deleted with the app open",
    },
    desbloqueo: {
      nombre: "Opening your saved notes",
      detalle: "comes with Windows Hello, in a later version",
    },
    "puerta-local": {
      nombre: "The door for Claude Code",
      detalle: "not on Windows for now",
    },
  },
  whisper: {
    titulo: "Transcription on this PC",
    bajando: "Downloading the speech model (1.6 GB, with its checksum)…",
    sinModelo:
      "There is no speech model. «Install» in Language downloads it: 1.6 GB, and its checksum is verified before use.",
    sinMedir: (m) => `Model ${m}: not measured this session.`,
    aTiempo: (m, ms) =>
      `Model ${m}: 10 s of audio in ${ms} ms. The ceiling is 4 s.`,
    lento: (m, ms) =>
      `Model ${m}: took ${ms} ms, over the 4 s ceiling on this PC. The small model is less accurate, but faster here.`,
    medirOtra: "Measure again",
    bajarPequeno: "Download the small model (0.5 GB)",
    sinReunion:
      "Downloading or measuring isn't possible while a meeting is open.",
  },
  idioma: {
    transcribeTuPc: "Transcribes on your PC, not a service",
    transcribeTuPcDetalle:
      "The speech model (whisper) lives inside your machine. No audio leaves to become text. The only time it touches the network is when you ask to install a model: Angel Ghost downloads it from its origin and checks its checksum before using it.",
    idiomasListos: "languages ready at once, at most: the ones the app offers.",
  },
  notas: {
    cadaReunion:
      "Every meeting you save lives that long, and deletes itself when it expires, once you open the app. With the app closed, not yet.",
    enLaBandejaDetalle: "These, encrypted, delete at zero, once you open the app.",
  },
  honestidad: {
    modoDetalle:
      "The app opens a connection only if you turn on the API in AI, with your key, and only with anonymized text. The other one the app opens itself, once, when you ask to install the speech model: 1.6 GB, checked before use.",
  },
  permisos: {
    microfonoPara:
      "Listening to the room or your voice, with this PC's microphone",
    abrirAjustes: "Open Windows Settings",
    llegaDespues: "Coming in a later version",
    antesTitulo: "The microphone on Windows",
    antesTexto:
      "You grant or revoke it yourself in Settings → Privacy & security → Microphone. The app never asks for the permission on its own.",
  },
};

export const TEXTOS_DE_WINDOWS: Record<Idioma, TextosDeWindows> = {
  es: ES,
  en: EN,
};

/** Los textos de Windows en el idioma de la interfaz. */
export function useTextosDeWindows(): TextosDeWindows {
  return TEXTOS_DE_WINDOWS[useIdioma()];
}

/** Un número con el separador decimal del idioma, para las frases de arriba. */
export function cifraEnIdioma(n: number, idioma: Idioma): string {
  return new Intl.NumberFormat(idioma).format(n);
}
