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
    /** Sprint 007: con dos pistas el cliente va primero; lo que no dio tiempo a transcribir, contado sin atribuirlo. */
    descartados: (n: number) => string;
  };
  /** Sprint 007: un atajo que Windows no dejó registrar porque otra app ya tiene esa combinación. */
  atajoOcupado: (teclas: string) => string;
  /** Sprint 008: el aviso corto de la banda cuando alguna tecla no responde (el detalle vive en Honestidad). */
  teclasSinResponder: (n: number) => string;
  /** Idioma: el motor de voz es de la app, no del sistema (ADR 022). */
  idioma: {
    transcribeTuPc: string;
    transcribeTuPcDetalle: string;
    idiomasListos: string;
    /** Sprint 008: el «Instalar» acabó y el modelo no está (la descarga falló o se paró). */
    instalarFallo: string;
  };
  /** Honestidad: qué abre la app y cuándo. */
  honestidad: {
    modoDetalle: string;
  };
  permisos: {
    microfonoPara: string;
    abrirAjustes: string;
    /** El audio del sistema, la pantalla (sprint 007) y el acople (sprint 008) no piden ningún permiso en Windows. */
    sinPermiso: string;
    antesTitulo: string;
    antesTexto: string;
  };
};

const ES: TextosDeWindows = {
  titulo: "Hoy en Windows",
  intro:
    "Angel Ghost se instala en Windows. Esto es lo que hace hoy en este PC y lo que no, dicho tal cual.",
  ya: "Ya funciona",
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
        "sí: se guardan cifradas con la llave en el Administrador de credenciales; exportarlas y abrir la bandeja piden Windows Hello",
    },
    videollamada: {
      nombre: "Las videollamadas: el audio del cliente",
      detalle:
        "sí: tu micrófono y el audio del sistema, dos pistas; sin verificar en Teams, Zoom ni Meet hasta que la pruebes en tu PC",
    },
    pantalla: {
      nombre: "Leer la pantalla de la reunión",
      detalle:
        "sí: la ventana de la reunión, leída solo cuando cambia; necesita el reconocimiento de texto de tu idioma en Windows",
    },
    "voz-al-oido": {
      nombre: "La ficha al oído",
      detalle:
        "sí, con auriculares; por altavoces no habla",
    },
    acople: {
      nombre: "Acoplarse a la ventana de la reunión",
      detalle:
        "sí: la ventana de la reunión baja y se encoge para que la banda no la tape, y vuelve al cerrar; no pide permiso",
    },
    ensayo: {
      nombre: "Ensayar con la voz que lee las preguntas",
      detalle:
        "sí: la voz del sistema lee las preguntas",
    },
    sugerencia: {
      nombre: "La sugerencia redactada por IA",
      detalle:
        "sí, con el API y tu clave: en Windows no hay modelo del sistema; sin API, la ficha sigue siendo la respuesta",
    },
    radar: {
      nombre: "El radar de tu equipo",
      detalle:
        "sí: los programas de este PC que te miran, con su catálogo de Windows, y si el PC está gestionado (MDM)",
    },
    atajos: {
      nombre: "Los atajos de teclado globales",
      detalle:
        "sí: Alt+Mayús y una tecla; ninguno con Ctrl+Alt, que en muchos teclados es AltGr",
    },
    "borrado-con-la-app-cerrada": {
      nombre: "Borrar tus notas con la app cerrada",
      detalle:
        "sí: una tarea del Programador de tareas las borra al vencer, aunque no abras la app",
    },
    desbloqueo: {
      nombre: "Abrir tus notas guardadas",
      detalle: "sí: con Windows Hello (tu PIN, tu huella o tu cara), una vez por sesión de la app",
    },
    "puerta-local": {
      nombre: "La puerta para Claude Code",
      detalle: "no en Windows por ahora",
    },
  },
  atajoOcupado: (teclas) =>
    `${teclas} no responde: otra app de este PC ya tiene esa combinación. Lo mismo se hace desde la app.`,
  teclasSinResponder: (n) => (n === 1 ? "1 tecla no responde" : `${n} teclas no responden`),
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
    sinReunion: "Bajar o medir no se puede con una reunión, una sesión o un ensayo abiertos.",
    descartados: (n) =>
      n === 1
        ? "En la última reunión, 1 turno se quedó sin transcribir: este PC no daba abasto, y se atiende primero al cliente."
        : `En la última reunión, ${n} turnos se quedaron sin transcribir: este PC no daba abasto, y se atiende primero al cliente.`,
  },
  idioma: {
    transcribeTuPc: "Transcribe tu PC, no un servicio",
    transcribeTuPcDetalle:
      "El modelo de voz (whisper), dentro de tu equipo. Ningún audio sale para convertirse en texto. La única vez que toca la red es cuando pides instalar el modelo: lo baja Angel Ghost desde su origen y comprueba su huella antes de usarlo.",
    idiomasListos: "idiomas a la vez, como mucho: los que la app ofrece.",
    instalarFallo:
      "No se pudo bajar el modelo. Revisa la conexión y vuelve a intentarlo (si empezó una reunión o un ensayo, la descarga se para).",
  },
  honestidad: {
    modoDetalle:
      "La app abre una conexión solo si enciendes el API en IA, con tu clave, y solo con texto anonimizado. La otra la abre la propia app, una vez, cuando pides instalar el modelo de voz: 1,6 GB, comprobados antes de usarse.",
  },
  permisos: {
    microfonoPara: "Escuchar la sala o tu voz, con el micrófono de este PC",
    abrirAjustes: "Abrir Ajustes de Windows",
    sinPermiso: "No pide permiso en Windows",
    antesTitulo: "El micrófono en Windows",
    antesTexto:
      "El micrófono lo concedes o lo quitas tú en Ajustes → Privacidad y seguridad → Micrófono; la app no pide el permiso por su cuenta. El audio del sistema, la pantalla de la reunión y el acople no piden ninguno.",
  },
};

const EN: TextosDeWindows = {
  titulo: "On Windows today",
  intro:
    "Angel Ghost installs on Windows. This is what it does today on this PC, and what it doesn't, stated plainly.",
  ya: "Works now",
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
        "yes: they're saved encrypted with the key in Credential Manager; exporting them and opening the tray ask for Windows Hello",
    },
    videollamada: {
      nombre: "Video calls: the client's audio",
      detalle:
        "yes: your microphone and the system audio, two tracks; unverified on Teams, Zoom or Meet until you test it on your PC",
    },
    pantalla: {
      nombre: "Reading the meeting's screen",
      detalle:
        "yes: the meeting window, read only when it changes; needs Windows text recognition for your language",
    },
    "voz-al-oido": {
      nombre: "The card read to your ear",
      detalle:
        "yes, with headphones; it doesn't speak through speakers",
    },
    acople: {
      nombre: "Docking to the meeting window",
      detalle:
        "yes: the meeting window moves down and shrinks so the band doesn't cover it, and comes back on close; no permission needed",
    },
    ensayo: {
      nombre: "Rehearsing with a voice that reads the questions",
      detalle:
        "yes: the system voice reads the questions",
    },
    sugerencia: {
      nombre: "The suggestion written by AI",
      detalle:
        "yes, with the API and your key: Windows has no system model; without the API, the card is still the answer",
    },
    radar: {
      nombre: "Your machine's radar",
      detalle:
        "yes: the programs on this PC that watch you, with its Windows catalog, and whether the PC is managed (MDM)",
    },
    atajos: {
      nombre: "Global keyboard shortcuts",
      detalle:
        "yes: Alt+Shift and a key; none with Ctrl+Alt, which is AltGr on many keyboards",
    },
    "borrado-con-la-app-cerrada": {
      nombre: "Deleting your notes with the app closed",
      detalle:
        "yes: a Task Scheduler task deletes them when they expire, even if you never open the app",
    },
    desbloqueo: {
      nombre: "Opening your saved notes",
      detalle: "yes: with Windows Hello (your PIN, fingerprint or face), once per app session",
    },
    "puerta-local": {
      nombre: "The door for Claude Code",
      detalle: "not on Windows for now",
    },
  },
  atajoOcupado: (teclas) =>
    `${teclas} doesn't respond: another app on this PC already has that combination. You can do the same from the app.`,
  teclasSinResponder: (n) => (n === 1 ? "1 shortcut isn't responding" : `${n} shortcuts aren't responding`),
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
      "Downloading or measuring isn't possible while a meeting, a session or a rehearsal is open.",
    descartados: (n) =>
      n === 1
        ? "In the last meeting, 1 turn wasn't transcribed: this PC couldn't keep up, and the client is served first."
        : `In the last meeting, ${n} turns weren't transcribed: this PC couldn't keep up, and the client is served first.`,
  },
  idioma: {
    transcribeTuPc: "Transcribes on your PC, not a service",
    transcribeTuPcDetalle:
      "The speech model (whisper) lives inside your machine. No audio leaves to become text. The only time it touches the network is when you ask to install a model: Angel Ghost downloads it from its origin and checks its checksum before using it.",
    idiomasListos: "languages ready at once, at most: the ones the app offers.",
    instalarFallo:
      "The model couldn't be downloaded. Check the connection and try again (if a meeting or a rehearsal started, the download stops).",
  },
  honestidad: {
    modoDetalle:
      "The app opens a connection only if you turn on the API in AI, with your key, and only with anonymized text. The other one the app opens itself, once, when you ask to install the speech model: 1.6 GB, checked before use.",
  },
  permisos: {
    microfonoPara:
      "Listening to the room or your voice, with this PC's microphone",
    abrirAjustes: "Open Windows Settings",
    sinPermiso: "Needs no permission on Windows",
    antesTitulo: "The microphone on Windows",
    antesTexto:
      "You grant or revoke the microphone yourself in Settings → Privacy & security → Microphone; the app never asks for the permission on its own. The system audio, the meeting screen and docking need none.",
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
