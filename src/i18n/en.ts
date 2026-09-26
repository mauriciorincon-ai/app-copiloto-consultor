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

    escuchandoPrefijo: "Listening",
    pista: "track",
    pistas: "tracks",
    protegidoSufijo: "protected",
    sinVerificarSufijo: "not verified",

    esperando: "When the client asks, your evidence shows up here.",
    corpus: "143 documents · 5 units",
    reunion: "Páramo Azul · 12 min",
    documento: "document",
    documentos: "documents",
    unidadPalabra: "unit",
    unidadesPalabra: "units",

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

    // ---- el modo solo audio (C15, sprint 002) ----
    diciendoLaFicha: "Reading you the card…",
    callar: "stop",
    volver: "back",
    /**
     * **El estado callado se DICE.** La primera propuesta de la mirada 16-bis enseñaba la línea
     * de la ficha recién leída y se callaba el estado; el veredicto del usuario fue «creo que sí
     * debería hacer evidente el estado». Es el mismo patrón «estado · por qué» de «Conecta
     * auriculares · el cliente te oiría», que él ya había aprobado.
     */
    callado: "Silent",
    esperandoElSiguienteTurno: "waiting for the next turn",
    conectaAuriculares: "Plug in headphones",
    elClienteTeOiria: "the client would hear you",

    // ---- phase 3 of sprint 002: why the card came, and the read that found nothing ----
    motivos: {
      pregunta: "question",
      cifra: "figure",
      terminoDelCorpus: "your term",
      silencioLargo: "silence",
      atajo: "you asked",
      pantalla: "on screen",
    },
    seccionConjeturada: "guessed section",
    leiLaPantalla: "I read the screen: there is no text to search.",

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
      hora3: "14:05",
      titularPdf: "Four sources integrated in 9 weeks",
      lineaPdf: "Up-front profiling halved the cleansing; the fourth source came in at week 6.",
      fuentePdf: "Sur del Valle · Results",
      titularPantalla: "Three channels, and no one knows which one makes margin",
      lineaPantalla: "Wholesale, village shops and direct sales: the proposal measures the margin of each.",
      fuentePantalla: "Páramo Azul · Context",
    },
  },
  /** THE NOTEBOOK — see the Spanish dictionary for the contract. */
  cuaderno: {
    marca: "Angel Ghost",
    marcaSub: "private notebook",
    // ---- corpus (fase 4) ----
    corpusTitulo: "Corpus",
    corpusSub: "Your documents, indexed where they live. Every card you will see in a meeting comes from here.",
    senalarCarpeta: "Point to a folder",
    noSeCopianAntes: "Your documents are",
    noSeCopianFuerte: "not copied",
    noSeCopianDespues: ": they are read where they live.",
    dondeVive: "Where the index lives",
    soloTu: "Only you can read it",
    corpusPendiente: "What does not exist yet",
    arrastrar: "Drag and drop documents",
    releer: "Re-read only what you change",
    leerEscaneado: "Read what is scanned",
    corpusNota: "Today the app walks the whole folder, up to 2,000 documents. Scanned files stay out with their reason, and are never sent to a service.",
    sinUnidad: "no unit",
    sinUnidadNota: "fits none, and is not forced",
    uProp: "scope, price, assumptions",
    uMarco: "your method, your stages",
    uCaso: "what happened, what it cost",
    uCliente: "briefs and past agreements",
    uPerfil: "your track record",
    sinLeer: "unread",
    conSeccionesConjeturadas: "with guessed sections",
    conjeturadas: "A PDF carries no headings, only lines: the app guesses where each section starts from the shape of the text. It is counted so you can judge it.",

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
    pistaPantalla: "Screen — reads only what is new",
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
    permPantallaPara: "Reads figures and titles only if something is new, like another slide. Images are not stored.",
    permAcople: "Dock the meeting window",
    permAcoplePara:
      "So the band does not cover the call: the meeting shrinks and both live side by side. macOS calls it “Accessibility”.",
    concedido: "Granted",
    sinConceder: "Not granted",
    concederEnMacos: "Grant in macOS",
    sinConcederNada: "What you can do now, granting nothing",
    indexar: "Index your corpus",
    escribirNotas: "Write notes and agreements",
    buscarAMano: "Search your evidence by hand",
    textoMicrofono:
      "“Angel Ghost uses the microphone to know when you are speaking. Audio lives in memory only and is never recorded.”",

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
      "In this version the app opens no connection: the only one there is belongs to macOS, when you ask it to install a speech model.",
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
    sinModelo: "no model",
    noLoReconoce: "not recognised",
    sinMotorDeVoz: "no engine",
    instalarModelo: "Install",
    instalando: "installing…",
    cincoIdiomas: "Five languages ready at a time, at most: macOS imposes it, not the app.",
    transcribeTuMac: "Your Mac transcribes, not a service",
    transcribeTuMacDetalle:
      "The macOS speech engine, inside your machine. No audio leaves to become text. The only time it touches the network is when you ask to install a language model: macOS downloads it —while it runs, installing…— and if it cannot: not recognised · no engine.",
    loQueTodaviaNo: "What does not exist yet",
    variosIdiomasPorPista: "Several languages at once, ticked per track",
    loQueFaltaDetalle:
      "Today each track listens to one language, and both sets of turns die — yours and the client’s — on close and with the key.",
    conservarTusTurnos: "Keep what you said",
    // ---- phase 3 of sprint 002: THE WHYS (look 17-quater, `kit.html` §8-ter) ----
    noAbrio: "Did not open",
    porQueNoAbrio: {
      "sin-permiso-del-microfono": "macOS did not grant the microphone. Grant it in Permissions and press “Start session” again.",
      "sin-permiso-del-audio": "macOS did not grant system audio. Grant it in Permissions and press “Start session” again.",
      "dispositivo-ocupado": "macOS would not create the tap: another app holds the device. Close it and press “Start session” again.",
      "formato-ilegible": "The device delivers an audio format the app cannot read. Try another one and press “Start session” again.",
      "no-dejo": "macOS would not open this track. Press “Start session” again; if it repeats, restart the Mac.",
    },
    escuchaAMedias: "Listens and transcribes on your Mac —",
    soloTuPista: "your track only",
    soloLaDelCliente: "the client’s track only",
    aMedias: "Partly",
    pantallaEspera: "Waiting for the meeting",
    pantallaApagada: "Off",
    pantallaApagadaPor: "It does not read it on its own. Ask for it whenever you want: ⌃⌥L.",
    pantallaSinPermiso: "No permission",
    pantallaSinPermisoPor: "macOS has not granted screen recording: the app reads nothing from it. Grant it in Permissions.",
    pantallaNoPudo: "Could not",
    pantallaNoPudoPor: "macOS did not hand over the meeting window. The app tries again on its own.",
    leerlaSola: "Read it on its own",
    leelaAhora: "read it now",
    siEsUnAltavoz: "If it is a speaker, do not use audio-only mode.",
    noSeSabe: "Unknown",
    porQueNoSeSabe: {
      "sin-salida": "This Mac does not say where the sound goes out.",
      "sin-conexion": "does not say how it is connected.",
      "sin-fuente": "does not say where it plays.",
    },
    noSePuedeSaber: "Cannot tell whether there is a meeting",
    porQueNoSeVe: {
      "sin-accesibilidad": "Without the Accessibility permission the app cannot see whether you have Meet open. Grant it in Permissions.",
    },
    dosPermisos: "System audio and Screen are two macOS permissions, even though Settings shows them in the same pane.",
    antesDeQueMacos: "Before macOS asks you",
    textoPantalla:
      "“Angel Ghost only reads what is new on your shared screen, to recognise figures and titles. Images live in memory and are neither stored nor sent anywhere.”",
    fraseDeMacos:
      "macOS asks with its own sentence and allows no other: “Angel Ghost would like to capture the contents of the system display.”",
    soloEnMemoriaElUltimo: "in memory only · the last one",
    botonCorta: "The button cuts",
    de: "of",
    piezasNingunaFuera: "pieces: none is left out.",
    idiomasListos: "languages ready at a time, at most: macOS imposes it, not the app.",
    sinMotorTitulo: "No speech engine: nothing is transcribed",
    porQueNoHayMotor: {
      "sin-transcriptor": "This Mac does not have the macOS 26 transcriber.",
      "sin-puente": "This copy of the app was built without the transcriber.",
      "no-contesta": "The macOS transcriber did not answer. Reopen the app; if it repeats, restart the Mac.",
    },
    laBandaSigue: "The band still works with ⌃⌥A and your corpus; what does not arrive is the automatic card.",
    tuDiccionario: "Your technical dictionary",
    deTuCorpus: "From your corpus — names, products, titles",
    enTuArchivo: "In your file",
    jamasCompleta: "It never completes a sentence or guesses a word.",
    tuCarpeta: "Your folder",
    secciones: "sections",
  },
};
