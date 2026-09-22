import type { Diccionario } from "./es";

/**
 * Diccionario EN — las MISMAS cadenas de la maqueta, en inglés. El tipo `Diccionario` obliga a
 * que las claves coincidan con las de `es`: si una falta, no compila. Y el gate del diccionario
 * comprueba que cada valor exista tal cual en `docs/diseno/`.
 */
export const en: Diccionario = {
  banda: {
    escuchando: "Listening · 2 tracks",
    protegido: "Meet · protected",
    sinVerificar: "Zoom · not verified",
    sinAcople: "not docked",

    esperando: "When the client asks, your evidence shows up here.",
    corpus: "143 documents · 5 units",
    reunion: "Páramo Azul · 12 min",

    buscando: "Searching your corpus…",

    nadaSobre: "Nothing in your corpus about",
    /** Las comillas del idioma: la maqueta escribe «…» en español y “…” en inglés. */
    comillaAbre: "“",
    comillaCierra: "”",
    /** Las cinco unidades del modelo de consultoría, como las escriben los chips de la maqueta. */
    unidades: {
      propuesta: "proposal",
      marco: "framework",
      caso: "case",
      cliente: "client",
      perfil: "profile",
    },
    maniobras: {
      credencial: "Say so plainly and offer to confirm it today.",
      cifra: "Do not improvise figures: offer the range from the comparable case.",
      plazo: "Give the timeline from the closest case and confirm it in writing.",
      referencia: "Offer a reference from the sector without naming the client yet.",
      contrato: "Do not opine on contracts live: note it down and answer in writing.",
      generica: "Turn the question around: what do they need it for?",
    },
    cercano: "closest match",
    cercanoLargo: "closest things you do have · none answers the question",
    buscarOtras: "Search with other words",
    anotarDespues: "Note it for later",
    otrasPalabras: "other words",
    anotar: "note it",

    sinVerificarTitulo: "Zoom: protection not verified",
    sinVerificarSalida: "→ Share a window, not the screen · the handle shows the other two ways out",
    sinVerificarCuando: "Verified only in Meet · macOS 26.6.2 · 2026-09-20",
    sinVerificarVentana: "Share a window, not the screen",
    sinVerificarMonitor: "Or use a second monitor",
    sinVerificarNotas: "Or switch to notes-only mode",
    yaVerifique: "I verified it in Zoom",
    soloNotas: "Notes-only mode",

    transcriptCab: "live · in memory only",
    ocultar: "hide",
    cliente: "client",
    tu: "you",

    fijar: "pin",
    ayudame: "help me",
    corta: "cut",
    transcript: "transcript",

    muestra: {
      oidoQuien: "client 14:02",
      oido: "And the data cleansing, is that within scope?",
      buscado: "ISO 27001 certification",
      oidoIso: "Are you ISO 27001 certified?",

      titular: "Data cleansing: included, up to three sources",
      linea: "Covers profiling and cleansing of ERP, POS and channel Excel; a fourth source is an add-on.",
      lineaLarga:
        "Covers profiling and cleansing of ERP, POS and channel Excel; a fourth source is an add-on, priced separately.",
      unidad: "proposal",
      fuente: "Páramo Azul · §3.2 Scope",

      acumulada1Unidad: "framework",
      acumulada1: "Stage 2 Preparation: profile before modelling",
      acumulada2Unidad: "case",
      acumulada2: "Sur del Valle: four sources in 9 weeks",

      cercana: "framework · §5.1 Data security and handling",
      cercana1Unidad: "framework",
      cercana1: "§5.1 Data security and handling",
      cercana2Unidad: "proposal",
      cercana2: "§6.3 Confidentiality and system access",
      cercana3Unidad: "case",
      cercana3: "Sur del Valle: the client’s internal audit",

      turno1: "And the data cleansing, is that within scope?",
      turno2: "Yes, cleansing the three sources we agreed on is included.",
      turno3: "What if we add the sales team’s Excel? We have it in Power BI.",
      hora1: "14:01",
      hora2: "14:02",
    },
  },
  /** THE NOTEBOOK — see the Spanish dictionary for the contract. */
  cuaderno: {
    marca: "Angel Ghost",
    marcaSub: "private notebook",
    navSesion: "Session",
    navPermisos: "Permissions",
    navCorpus: "Corpus",
    navNotas: "Notes",
    navHonestidad: "Honesty",
    navIdioma: "Language",
    navIa: "AI",
    sinSesion: "No session · 0 B",
    meetDetectado: "Meet detected · 0 B",

    todaviaNo: "Not yet",
    funciona: "Works",

    sesionTitulo: "Before you start",
    sesionSub: "Nothing turns on until you say so. This is what the app sees of your own Mac.",
    proteccionVerificada: "Protection verified",
    proteccionSinVerificar: "Zoom · not verified",
    proteccionDetalle:
      "Your panel does not show in the screen you share. Verified on your Mac (macOS 26.6.2) on 2026-09-20. In Zoom and Teams it is unverified.",
    dosPistas: "The two tracks",
    pistaMic: "Microphone — you",
    pistaSistema: "System audio — the client",
    pistaPantalla: "Screen — only when it changes",
    pistaAuriculares: "Headphones connected",
    esteCliente: "This client",
    fichaNdaRadar: "Client file, NDA and radar",
    noSeInventa: "Nothing is made up while it does not exist: no flag, no risk, no catalog.",
    queFuncionaHoy: "What works today",
    funcionaBanda: "The band, at the bottom, protected from capture",
    funcionaAcople: "The meeting makes room: it docks and returns on close",
    funcionaCorte: "Cut everything and wipe memory",
    iniciarSesion: "Start session",
    nadaSale: "cuts everything · sound is never stored · nothing leaves your machine",
    sinReunion: "No meeting open",
    tituloDeMuestra: "Páramo Azul — Profitability dashboard proposal",
    sinReunionVoz: "Open Zoom, Meet or Teams and it will show up here. Meanwhile, get ready.",

    permisosTitulo: "macOS permissions",
    permisosSub: "You grant them in the system, not here. The app works without them: it just does less.",
    permMic: "Microphone",
    permMicPara: "Your voice, to know when you speak. In memory only; never recorded.",
    permSistema: "System audio",
    permSistemaPara: "What plays on your Mac: the client’s voice. No bot in the meeting.",
    permPantalla: "Screen",
    permPantallaPara: "Reads figures and titles only when the screen changes. Images are not stored.",
    permAcople: "Dock the meeting window",
    permAcoplePara:
      "So the band does not cover the call: the meeting shrinks and both live side by side. macOS calls it “Accessibility”.",
    concedido: "Granted",
    sinConceder: "Not granted",
    concederEnMacos: "Grant in macOS",
    unSoloPermiso: "System audio and Screen are a single macOS permission: they are granted and lost together.",
    sinConcederNada: "What you can do now, granting nothing",
    indexar: "Index your corpus",
    escribirNotas: "Write notes and agreements",
    buscarAMano: "Search your evidence by hand",
    queTextoVeras: "What macOS will show you",
    textoMicrofono:
      "“Angel Ghost uses the microphone to know when you are speaking. Audio lives in memory only and is never recorded.”",
    claveMicrofono: "NSMicrophoneUsageDescription · es / en",

    honestidadTitulo: "Honesty",
    honestidadSub: "What lives in memory right now and what left your machine. Proven, not promised.",
    queViveEnMemoria: "What lives in memory now",
    bufMic: "Audio · microphone",
    bufSistema: "Audio · system",
    bufTranscript: "Transcript",
    bufFrame: "Last frame read",
    salieronDeTuEquipo: "left your machine in this meeting",
    modo: "Mode",
    modoLocal: "100 % local · API off",
    modoDetalle:
      "In this version there is no code able to open a connection: the zero is not kept by discipline.",
    piezasCola: "pieces: the other one does not exist yet.",
    loQueQuedara: "What will be left when you close",
    loQueQuedaraDetalle:
      "Notes, agreements and pinned cards come later. Today nothing is left because today nothing is written.",
    // ---- session · what phase 3 turned on ----
    funcionaEscucha: "It listens to both tracks and transcribes them on your Mac",
    altavocesInternos: "Internal speakers",
    avisoDelEco:
      "The microphone hears it too: marked as echo.",

    // ---- honesty · the buffers that now exist ----
    ringBuffer30: "ring buffer · last 30 s",
    ventana12: "window of 12 turns",

    // ---- language (sprint 1 subset) ----
    idiomaTitulo: "Language and transcript",
    idiomaSub:
      "Two tracks, the languages you tick, and a dictionary that is yours. The transcript works; you hardly ever look at it.",
    transcripcionEnVivo: "Live transcript",
    oculta: "hidden",
    visible: "visible",
    naceOculta:
      "It ships hidden on purpose. Reading what was just said is the fastest way to stop listening: your eyes go to the text and the conversation is left alone.",
    laMuestraCuando: "shows it when you need it",
    idiomaPorPista: "Language per track",
    tuMicrofono: "You · microphone",
    clienteSistema: "Client · system",
    modeloInstalado: "model installed",
    cincoIdiomas: "Five languages ready at a time, at most: macOS imposes it, not the app.",
    transcribeTuMac: "Your Mac transcribes, not a service",
    transcribeTuMacDetalle:
      "The macOS speech engine, inside your machine. No audio leaves to become text. The only time the app touches the network is when you ask to install a language model: macOS downloads it then, and nothing leaves here.",
    loQueTodaviaNo: "What does not exist yet",
    variosIdiomasPorPista: "Several languages at once, ticked per track",
    loQueFaltaDetalle:
      "Today each track listens to one language, proper names are transcribed as they sound, and both sets of turns die — yours and the client’s — on close and with the key.",
    diccionarioTecnico: "Technical dictionary: what is heard into what you want to read",
    conservarTusTurnos: "Keep what you said",
  },
};
