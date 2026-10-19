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
    /** Sprint 010: lo que dice la pantalla y el botón mientras mide (con el modelo grande, cerca de medio minuto). */
    midiendo: string;
    midiendoBoton: string;
    /** Sprint 010: mientras baja el modelo pequeño desde Honestidad y lo mide después. */
    bajandoYMidiendo: string;
    bajarPequeno: string;
    sinReunion: string;
    /** Sprint 007: con dos pistas el cliente va primero; lo que no dio tiempo a transcribir, contado sin atribuirlo. */
    descartados: (n: number) => string;
    /**
     * Sprint 009: una línea por modelo, con lo que tardó en ESTE PC. Sprint 010: pista por pista; `cliente` es lo que
     * tardó la pista del cliente, ya formateado, o `null` si todavía no se midió. Los segundos de cada pista los ata
     * `windows-honestidad-y-manual` a sus archivos de audio.
     */
    modelos: {
      titulo: string;
      sinBajar: (m: string) => string;
      sinMedir: (m: string) => string;
      aTiempo: (m: string, ms: string, cliente: string | null) => string;
      lento: (m: string, ms: string, cliente: string | null) => string;
      /** Tu pista cabe en el techo y la del cliente no. */
      clienteLento: (m: string, ms: string, cliente: string) => string;
      enUso: string;
      porDefecto: string;
      regla: string;
    };
  };
  /** Sprint 007: un atajo que Windows no dejó registrar porque otra app ya tiene esa combinación. */
  atajoOcupado: (teclas: string) => string;
  /** Sprint 008: el aviso corto de la banda cuando alguna tecla no responde (el detalle vive en Honestidad). */
  teclasSinResponder: (n: number) => string;
  /**
   * Sprint 011: la banda a tu altura y su «×». Solo Windows, y sin maqueta (diccionario aparte, ADR 022): el asa de altura y
   * el «×» de la banda, y la fila de Sesión con la altura y «Mostrar la banda». Maquetados, no vistos: van al gate del hito.
   */
  banda: {
    /** El título del asa de arriba de la banda: la que elige a qué altura queda. */
    asaDeAltura: string;
    /** El nombre del «×» sin sesión, y el que dice lo que de verdad hace con la sesión en marcha. */
    cerrar: string;
    cerrarYTerminar: string;
    /** La fila de Sesión: la altura, y el botón que la devuelve al borde de arriba. */
    alturaTitulo: string;
    pegadaArriba: string;
    alturaValor: (px: number) => string;
    volverArriba: string;
    /** «Bajar» y «Subir»: la altura por pasos, sin arrastrar. El nombre accesible dice qué se baja. */
    bajar: string;
    subir: string;
    bajarNombre: string;
    subirNombre: string;
    /** La fila de Sesión con la banda cerrada, y el botón que la vuelve a abrir. */
    cerradaTitulo: string;
    mostrar: string;
  };
  /**
   * Sprint 012 (ADR 024): el modo transparente por cliente. Solo Windows y sin maqueta (diccionario aparte, ADR 022): el
   * interruptor y el aviso en «Este cliente», «El cliente consintió», la marca de la banda y las constancias en «Se va a
   * guardar» y en Honestidad. Maquetados, no vistos: van al gate del hito.
   */
  transparente: {
    titulo: string;
    ayuda: string;
    encendido: string;
    apagado: string;
    /** El aviso fijo en los dos idiomas: el título del recuadro que lo enseña. */
    loQueLeDices: string;
    consintio: string;
    consintioAyuda: string;
    sinReunion: string;
    anotada: string;
    noAnotada: string;
    decirlo: string;
    dicho: string;
    noCabe: string;
    /** La marca corta de la banda: solo dice que el modo está encendido. */
    marcaDeLaBanda: string;
    /** «Se va a guardar»: la fila de la constancia, con cuántas hay. */
    constancia: string;
    constanciaDonde: (n: number) => string;
    /** Honestidad, en «Tuyo»: lo que el modo añade a lo que se guarda. */
    tuyoMas: string;
  };
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
  /**
   * Sprint 014 (ADR 023 enmienda 2): el modelo local de la sugerencia, en IA. Solo Windows y sin maqueta (diccionario
   * aparte, ADR 022). Maquetados, no vistos: van al gate del hito.
   */
  ia: {
    /** La fila del modelo en «Quién redacta», sin él. */
    sinInstalar: string;
    bajando: string;
    fallo: string;
    /** El botón, con el tamaño real ya formateado con el separador del idioma. */
    instalar: (gb: string) => string;
    /** Debajo de la tabla, sin el modelo: qué hace, cómo se instala y cuándo no se puede. */
    sinModelo: string;
    /** Debajo de la tabla, con el modelo. */
    instalado: string;
    /** El aviso del interruptor «Redactar sugerencias» sin el modelo. */
    instalaPrimero: string;
  };
  /**
   * **La reunión anterior sigue abierta** (sprint 013, decisión 2a): el cuadro que Windows enseña antes de empezar otra
   * sesión. Guardar crea la llave de notas y la tarea de borrado la primera vez, y eso no se hace sin que lo digas.
   */
  reunionAnterior: {
    titulo: string;
    detalle: string;
    guardar: string;
    descartar: string;
    cancelar: string;
    /** El guardado falló: la reunión sigue abierta y no se empezó nada. */
    noGuardo: string;
    /** El descarte falló: la reunión sigue abierta y no se empezó nada. */
    noDescarto: string;
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

/** Lo que tardó cada pista de la calibración (sprint 010): la tuya siempre; la del cliente, si ya se midió, y si no, lo dice. */
const pistasEs = (ms: string, cliente: string | null) =>
  cliente === null
    ? `tu pista, 10 s en ${ms} ms; la del cliente, sin medir`
    : `tu pista, 10 s en ${ms} ms; la del cliente, 6 s en ${cliente} ms`;
const pistasEn = (ms: string, cliente: string | null) =>
  cliente === null
    ? `your track, 10 s in ${ms} ms; the client's, not measured`
    : `your track, 10 s in ${ms} ms; the client's, 6 s in ${cliente} ms`;

const ES: TextosDeWindows = {
  titulo: "Hoy en Windows",
  intro:
    "Angel Ghost se instala en Windows. Esto es lo que hace hoy en este PC, dicho tal cual; lo que no hiciera, aparecería aparte con su motivo.",
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
      detalle: "sí, con auriculares; por altavoces no habla",
    },
    acople: {
      nombre: "Acoplarse a la ventana de la reunión",
      detalle:
        "sí: la ventana de la reunión baja y se encoge para que la banda no la tape, y vuelve al terminar la sesión, al cerrar la banda con su ×, con el corte o al salir de la app; no pide permiso",
    },
    ensayo: {
      nombre: "Ensayar con la voz que lee las preguntas",
      detalle: "sí: la voz del sistema lee las preguntas",
    },
    sugerencia: {
      nombre: "La sugerencia redactada por IA",
      detalle:
        "sí, con el modelo local que instalas en IA (1,1 GB, una vez): redacta dentro de tu PC, sin abrir ninguna conexión; sin el modelo, la ficha sigue siendo la respuesta",
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
      detalle:
        "sí: con Windows Hello (tu PIN, tu huella o tu cara), una vez por sesión de la app",
    },
    "puerta-local": {
      nombre: "La puerta para Claude Code",
      detalle:
        "sí: un named pipe solo de tu usuario, con la llave en el Administrador de credenciales; se cierra sola en reunión o ensayo",
    },
  },
  atajoOcupado: (teclas) =>
    `${teclas} no responde: otra app de este PC ya tiene esa combinación. Lo mismo se hace desde la app.`,
  teclasSinResponder: (n) =>
    n === 1 ? "1 tecla no responde" : `${n} teclas no responden`,
  banda: {
    asaDeAltura: "Arrastra para elegir a qué altura queda la banda",
    cerrar: "Cerrar la banda",
    cerrarYTerminar: "Cerrar la banda y terminar la sesión",
    alturaTitulo: "Altura de la banda",
    pegadaArriba: "Pegada al borde de arriba",
    alturaValor: (px) => `${px} px bajo el borde de arriba`,
    volverArriba: "Volver arriba",
    bajar: "Bajar",
    subir: "Subir",
    bajarNombre: "Bajar la banda",
    subirNombre: "Subir la banda",
    cerradaTitulo: "La banda está cerrada",
    mostrar: "Mostrar la banda",
  },
  transparente: {
    titulo: "Modo transparente",
    ayuda:
      "Le cuentas al cliente, antes de empezar, que usas esta herramienta en tu equipo. Se recuerda por cliente. Nada sale de la app.",
    encendido: "Encendido",
    apagado: "Apagado",
    loQueLeDices: "Lo que le dices",
    consintio: "El cliente consintió",
    consintioAyuda:
      "Deja constancia en tus notas cifradas: cliente, jurisdicción, hora, idioma y este texto.",
    sinReunion: "Se anota con la reunión abierta: inicia la sesión primero.",
    anotada: "Anotado en tus notas cifradas.",
    noAnotada: "No se pudo anotar. Revisa que la reunión siga abierta.",
    decirlo: "Decírselo en voz alta",
    dicho: "Se lo estoy diciendo con la voz.",
    noCabe:
      "No cabe decirlo ahora (altavoces, alguien hablando o sin voz): léeselo tú.",
    marcaDeLaBanda: "Modo transparente",
    constancia: "Constancia del cliente",
    constanciaDonde: (n) =>
      n === 1 ? "1 · consintió el aviso" : `${n} · consintieron el aviso`,
    tuyoMas:
      "Y, si usas el modo transparente, la constancia de que el cliente consintió: cliente, jurisdicción, hora, idioma y el aviso que dijiste.",
  },
  whisper: {
    titulo: "La transcripción en este PC",
    bajando: "Bajando el modelo de voz, con su comprobación…",
    sinModelo:
      "No hay modelo de voz. «Instalar» en Idioma lo baja (1,6 GB; 0,5 GB en un PC ARM64) y se comprueba su huella antes de usarlo.",
    sinMedir: (m) => `Modelo ${m}: sin medir en esta sesión.`,
    aTiempo: (m, ms) =>
      `Modelo ${m}: 10 s de audio en ${ms} ms. El techo son 4 s.`,
    lento: (m, ms) =>
      `Modelo ${m}: tardó ${ms} ms, y pasa del techo de 4 s en este PC. El modelo pequeño es menos preciso, pero más rápido aquí.`,
    medirOtra: "Medir otra vez",
    midiendo:
      "Midiendo en este PC, modelo por modelo y pista por pista: unos segundos con el pequeño y cerca de medio minuto con el grande. Las cifras cambian al terminar.",
    midiendoBoton: "Midiendo…",
    bajandoYMidiendo:
      "Bajando el modelo pequeño (0,5 GB) y midiéndolo después en este PC. Las cifras cambian al terminar.",
    bajarPequeno: "Bajar el modelo pequeño (0,5 GB)",
    sinReunion:
      "Bajar o medir no se puede con una reunión, una sesión o un ensayo abiertos.",
    descartados: (n) =>
      n === 1
        ? "En la última reunión, 1 turno se quedó sin transcribir: este PC no daba abasto, y se atiende primero al cliente."
        : `En la última reunión, ${n} turnos se quedaron sin transcribir: este PC no daba abasto, y se atiende primero al cliente.`,
    modelos: {
      titulo: "Cada modelo de voz, medido en este PC, pista por pista",
      sinBajar: (m) => `${m}: sin bajar.`,
      sinMedir: (m) => `${m}: bajado, sin medir en esta sesión.`,
      aTiempo: (m, ms, cliente) =>
        `${m}: ${pistasEs(ms, cliente)}. Cabe en el techo de 4 s.`,
      lento: (m, ms, cliente) =>
        `${m}: ${pistasEs(ms, cliente)}. Pasa del techo de 4 s.`,
      clienteLento: (m, ms, cliente) =>
        `${m}: ${pistasEs(ms, cliente)}. Tu pista cabe en el techo de 4 s; la del cliente, no.`,
      enUso: "el que transcribe ahora",
      porDefecto: "el que baja «Instalar»",
      regla:
        "Se transcribe con el modelo mayor cuya pista tuya (10 s) cabe en 4 s en este PC; mientras uno no se haya medido, se da por bueno. La del cliente se mide detrás, con el modelo ya en marcha, como en una reunión.",
    },
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
      "La app abre una conexión solo cuando tú lo pides, una vez por modelo: al instalar el modelo de voz, 1,6 GB (0,5 GB en un PC ARM64), y al instalar el modelo local de la sugerencia, 1,1 GB; los dos se comprueban antes de usarse. En Windows no hay API: todo lo demás se queda en tu PC.",
  },
  ia: {
    sinInstalar: "sin instalar",
    bajando: "bajando, con su comprobación…",
    fallo: "la descarga falló",
    instalar: (gb) => `Instalar el modelo (${gb} GB)`,
    sinModelo:
      "Para redactar en Windows, instala el modelo local: Angel Ghost lo baja una vez desde su origen y comprueba su huella antes de usarlo. Después redacta dentro de tu PC, sin abrir ninguna conexión. No se instala con una reunión, una sesión o un ensayo abiertos; si la descarga se corta, vuelve a pedirla.",
    instalado:
      "El modelo local redacta dentro de tu PC, sin abrir ninguna conexión: el contador sigue en 0 B. «Enriquecer el banco» no lo usa por ahora.",
    instalaPrimero: "instala el modelo",
  },
  reunionAnterior: {
    titulo: "La reunión anterior sigue abierta",
    detalle:
      "Tiene notas o propuestas tuyas sin decidir. Si la guardas, queda cifrada en tu PC; la primera vez, Windows guarda la llave de tus notas en el Administrador de credenciales y la tarea de borrado en el Programador de tareas. Si la descartas, se tira. Cancelar no cambia nada y no empieza otra.",
    guardar: "Guardar",
    descartar: "Descartar",
    cancelar: "Cancelar",
    noGuardo:
      "No se pudo guardar la reunión anterior. Sigue abierta y no se empezó nada: vuelve a intentarlo o descártala.",
    noDescarto:
      "No se pudo descartar la reunión anterior. Sigue abierta y no se empezó nada: vuelve a intentarlo o cancela.",
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
    "Angel Ghost installs on Windows. This is what it does today on this PC, stated plainly; anything it didn't do would appear apart, with its reason.",
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
      detalle: "yes, with headphones; it doesn't speak through speakers",
    },
    acople: {
      nombre: "Docking to the meeting window",
      detalle:
        "yes: the meeting window moves down and shrinks so the band doesn't cover it, and comes back when you end the session, close the band with its ×, cut everything or quit the app; no permission needed",
    },
    ensayo: {
      nombre: "Rehearsing with a voice that reads the questions",
      detalle: "yes: the system voice reads the questions",
    },
    sugerencia: {
      nombre: "The suggestion written by AI",
      detalle:
        "yes, with the local model you install in AI (1.1 GB, once): it writes inside your PC without opening any connection; without the model, the card is still the answer",
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
      detalle:
        "yes: with Windows Hello (your PIN, fingerprint or face), once per app session",
    },
    "puerta-local": {
      nombre: "The door for Claude Code",
      detalle:
        "yes: a named pipe only your user can open, with the key in Credential Manager; it closes itself in a meeting or rehearsal",
    },
  },
  atajoOcupado: (teclas) =>
    `${teclas} doesn't respond: another app on this PC already has that combination. You can do the same from the app.`,
  teclasSinResponder: (n) =>
    n === 1
      ? "1 shortcut isn't responding"
      : `${n} shortcuts aren't responding`,
  banda: {
    asaDeAltura: "Drag to choose how low the band sits",
    cerrar: "Close the band",
    cerrarYTerminar: "Close the band and end the session",
    alturaTitulo: "Band height",
    pegadaArriba: "At the top edge",
    alturaValor: (px) => `${px} px below the top edge`,
    volverArriba: "Back to the top",
    bajar: "Lower",
    subir: "Raise",
    bajarNombre: "Lower the band",
    subirNombre: "Raise the band",
    cerradaTitulo: "The band is closed",
    mostrar: "Show the band",
  },
  transparente: {
    titulo: "Transparent mode",
    ayuda:
      "Before you start, you tell the client you use this tool on your own computer. It is remembered per client. Nothing leaves the app.",
    encendido: "On",
    apagado: "Off",
    loQueLeDices: "What you tell them",
    consintio: "The client consented",
    consintioAyuda:
      "Records it in your encrypted notes: client, jurisdiction, time, language and this text.",
    sinReunion:
      "It is recorded while the meeting is open: start the session first.",
    anotada: "Recorded in your encrypted notes.",
    noAnotada: "It couldn't be recorded. Check that the meeting is still open.",
    decirlo: "Say it out loud",
    dicho: "Saying it now with the voice.",
    noCabe:
      "It can't be said now (speakers, someone talking or no voice): read it to them yourself.",
    marcaDeLaBanda: "Transparent mode",
    constancia: "Client's consent record",
    constanciaDonde: (n) =>
      n === 1
        ? "1 · consented to the notice"
        : `${n} · consented to the notice`,
    tuyoMas:
      "And, if you use the transparent mode, the record that the client consented: client, jurisdiction, time, language and the notice you gave.",
  },
  whisper: {
    titulo: "Transcription on this PC",
    bajando: "Downloading the speech model, with its checksum…",
    sinModelo:
      "There is no speech model. «Install» in Language downloads it (1.6 GB; 0.5 GB on an ARM64 PC) and its checksum is verified before use.",
    sinMedir: (m) => `Model ${m}: not measured this session.`,
    aTiempo: (m, ms) =>
      `Model ${m}: 10 s of audio in ${ms} ms. The ceiling is 4 s.`,
    lento: (m, ms) =>
      `Model ${m}: took ${ms} ms, over the 4 s ceiling on this PC. The small model is less accurate, but faster here.`,
    medirOtra: "Measure again",
    midiendo:
      "Measuring on this PC, model by model and track by track: a few seconds with the small one and about half a minute with the large one. The figures change when it finishes.",
    midiendoBoton: "Measuring…",
    bajandoYMidiendo:
      "Downloading the small model (0.5 GB) and then measuring it on this PC. The figures change when it finishes.",
    bajarPequeno: "Download the small model (0.5 GB)",
    sinReunion:
      "Downloading or measuring isn't possible while a meeting, a session or a rehearsal is open.",
    descartados: (n) =>
      n === 1
        ? "In the last meeting, 1 turn wasn't transcribed: this PC couldn't keep up, and the client is served first."
        : `In the last meeting, ${n} turns weren't transcribed: this PC couldn't keep up, and the client is served first.`,
    modelos: {
      titulo: "Each speech model, measured on this PC, track by track",
      sinBajar: (m) => `${m}: not downloaded.`,
      sinMedir: (m) => `${m}: downloaded, not measured this session.`,
      aTiempo: (m, ms, cliente) =>
        `${m}: ${pistasEn(ms, cliente)}. Fits the 4 s ceiling.`,
      lento: (m, ms, cliente) =>
        `${m}: ${pistasEn(ms, cliente)}. Over the 4 s ceiling.`,
      clienteLento: (m, ms, cliente) =>
        `${m}: ${pistasEn(ms, cliente)}. Your track fits the 4 s ceiling; the client's does not.`,
      enUso: "the one transcribing now",
      porDefecto: "the one «Install» downloads",
      regla:
        "Transcription uses the largest model whose track of yours (10 s) fits in 4 s on this PC; until one has been measured, it is taken as fitting. The client's track is measured after yours, with the model already running, as in a meeting.",
    },
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
      "The app opens a connection only when you ask, once per model: to install the speech model, 1.6 GB (0.5 GB on an ARM64 PC), and to install the suggestion’s local model, 1.1 GB; both are checked before use. There is no API on Windows: everything else stays on your PC.",
  },
  ia: {
    sinInstalar: "not installed",
    bajando: "downloading, with its checksum…",
    fallo: "the download failed",
    instalar: (gb) => `Install the model (${gb} GB)`,
    sinModelo:
      "To write suggestions on Windows, install the local model: Angel Ghost downloads it once from its origin and checks its checksum before using it. Then it writes inside your PC, without opening any connection. It isn’t installed while a meeting, a session or a rehearsal is open; if the download is cut, ask for it again.",
    instalado:
      "The local model writes inside your PC, without opening any connection: the counter stays at 0 B. «Enrich the bank» doesn’t use it for now.",
    instalaPrimero: "install the model",
  },
  reunionAnterior: {
    titulo: "The previous meeting is still open",
    detalle:
      "It has notes or proposals of yours you haven't decided on. If you save it, it stays encrypted on your PC; the first time, Windows stores your notes key in Credential Manager and the deletion task in Task Scheduler. If you discard it, it is thrown away. Cancel changes nothing and starts nothing.",
    guardar: "Save",
    descartar: "Discard",
    cancelar: "Cancel",
    noGuardo:
      "The previous meeting could not be saved. It is still open and nothing was started: try again or discard it.",
    noDescarto:
      "The previous meeting could not be discarded. It is still open and nothing was started: try again or cancel.",
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
