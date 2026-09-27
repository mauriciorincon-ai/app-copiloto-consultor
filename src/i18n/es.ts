/**
 * Diccionario ES — cadenas VERBATIM de la maqueta.
 *
 * Contrato §9 del design system: «en producto los pares `<span lang>` se sustituyen por el
 * diccionario i18n con las MISMAS cadenas que la maqueta — la maqueta es el primer diccionario».
 * `tests/unit/i18n-fiel-a-la-maqueta.test.ts` lo vigila: cada cadena de aquí debe existir tal
 * cual en `docs/diseno/`. Si el producto necesita decir algo que la maqueta no dice, primero se
 * escribe en la maqueta (es una decisión de diseño, va a una mirada), no aquí.
 *
 * La referencia de la banda es `docs/diseno/banda.html` (mirada 11).
 */
export const es = {
  banda: {
    /** El título del asa, para quien la pase el ratón. Estaba escrito en español en el componente y
     * salía igual en la interfaz inglesa (casilla 6 del S3). */
    asaAjustar: "arrastra para ajustar las dos a la vez",
    asaVolver: "arrastra para volver a la banda de 88 px",
    // ---- cabecera: estado de la sesión ----
    /**
     * LAS CINCO CADENAS DE ABAJO SON LO QUE LA MAQUETA DIBUJA, y dentro del producto **ya no se
     * pintan tal cual**: la banda las compone con lo que de verdad hay (hallazgo A1 de la
     * auditoría). Siguen existiendo porque son lo que el arnés del gate de FIDELIDAD fotografía
     * fuera de Tauri, y porque de ellas salen las piezas: los números y el nombre del cliente son
     * del sistema, las palabras son de la maqueta.
     */
    escuchando: "Escuchando · 2 pistas",
    protegido: "Meet · protegido",
    sinVerificar: "Zoom · sin verificar",
    sinAcople: "sin acople",

    /** Las piezas con las que se compone la cabecera. Cada una existe en la maqueta. */
    escuchandoPrefijo: "Escuchando",
    pista: "pista",
    pistas: "pistas",
    protegidoSufijo: "protegido",
    sinVerificarSufijo: "sin verificar",

    // ---- esperando ----
    esperando: "Cuando el cliente pregunte, aquí aparece tu evidencia.",
    corpus: "143 documentos · 5 unidades",
    reunion: "Páramo Azul · 12 min",
    /** Las piezas del contador del corpus. `unidades` ya es el objeto de las cinco etiquetas. */
    documento: "documento",
    documentos: "documentos",
    unidadPalabra: "unidad",
    unidadesPalabra: "unidades",

    // ---- buscando ----
    buscando: "Buscando en tu corpus…",

    // ---- sin resultado: el veredicto, la maniobra y lo más cercano ----
    // La maniobra sale de un catálogo versionado elegido por reglas léxicas. NO es una
    // sugerencia del modelo y no puede parecerlo (design-system §9-quinquies).
    /** Se compone con los términos que de verdad se buscaron: «Nada en tu corpus sobre «…»». */
    nadaSobre: "Nada en tu corpus sobre",
    /**
     * EL CATÁLOGO DE MANIOBRAS, bilingüe.
     *
     * Rust dice **cuál** maniobra (`credencial`, `cifra`, …) y el texto vive aquí, porque la
     * maniobra es voz de la app y la app es bilingüe por regla dura. Las siete están en
     * `docs/diseno/banda.html` —la maqueta es el primer diccionario— y en `design-system.md`: seis
     * aprobadas en la mirada 11 y el puente, construido en la auditoría del S2 (M15).
     */
    /** Las comillas del idioma: la maqueta escribe «…» en español y “…” en inglés. */
    comillaAbre: "«",
    comillaCierra: "»",
    /** Las cinco unidades del modelo de consultoría, como las escriben los chips de la maqueta. */
    unidades: {
      propuesta: "propuesta",
      marco: "marco",
      caso: "caso",
      cliente: "cliente",
      perfil: "perfil",
    },
    maniobras: {
      credencial: "Dilo sin adornos y ofrece confirmarlo hoy mismo.",
      cifra: "No improvises cifras: ofrece el rango del caso comparable.",
      plazo: "Da el plazo del caso más parecido y confírmalo por escrito.",
      referencia: "Ofrece una referencia del sector sin nombrar al cliente aún.",
      contrato: "No opines de contrato en vivo: anótalo y respóndelo por escrito.",
      generica: "Devuelve la pregunta: ¿para qué lo necesitan?",
      /** El puente (M15): la banda pone entre las dos partes el nombre de lo más cercano. */
      puente: "Lo más cercano que sí tienes es «…»: ofrécelo y pregunta para qué lo necesitan.",
    },
    puenteAntes: "Lo más cercano que sí tienes es",
    puenteDespues: ": ofrécelo y pregunta para qué lo necesitan.",
    cercano: "lo más cercano",
    cercanoLargo: "lo más cercano que sí tienes · ninguno responde la pregunta",
    buscarOtras: "Buscar con otras palabras",
    anotarDespues: "Anotar para después",
    otrasPalabras: "otras palabras",
    anotar: "anotar",

    // ---- sin verificar en este cliente ----
    sinVerificarTitulo: "Zoom: protección sin verificar",
    sinVerificarSalida: "→ Comparte una ventana, no la pantalla · el asa muestra las otras dos salidas",
    sinVerificarCuando: "Verificada solo en Meet · macOS 26.6.2 · 2026-09-20",
    sinVerificarVentana: "Comparte una ventana, no la pantalla",
    sinVerificarMonitor: "O usa un segundo monitor",
    sinVerificarNotas: "O corta la banda con ⌥⎋ mientras compartes",
    yaVerifique: "Ya lo verifiqué en Zoom",
    soloNotas: "Modo solo notas",

    // ---- transcript ----
    transcriptCab: "en vivo · solo en memoria",
    ocultar: "ocultar",
    cliente: "cliente",
    tu: "tú",

    // ---- atajos ----
    tePropongoGuardar: "Te propongo guardar:",
    fijada: "fijada",
    fijar: "fijar",
    ayudame: "ayúdame",
    corta: "corta",
    transcript: "transcript",

    /**
     * ---- el modo solo audio (C15, sprint 002) ----
     *
     * Las siete cadenas salen de `docs/diseno/banda.html`, estados «voz», «voz-espera» y
     * «voz-sin», y el gate del diccionario las compara una a una con la maqueta. Rust **no manda
     * ninguna**: manda tres booleanos. Si la parte nativa mandara «Conecta auriculares», ese texto
     * se podría cambiar sin que ninguna mirada lo viera nunca.
     */
    diciendoLaFicha: "Diciéndote la ficha…",
    callar: "callar",
    volver: "volver",
    /**
     * **El estado callado se DICE.** La primera propuesta de la mirada 16-bis enseñaba la línea
     * de la ficha recién leída y se callaba el estado; el veredicto del usuario fue «creo que sí
     * debería hacer evidente el estado». Es el mismo patrón «estado · por qué» de «Conecta
     * auriculares · el cliente te oiría», que él ya había aprobado.
     */
    callado: "Callado",
    esperandoElSiguienteTurno: "esperando el siguiente turno",
    conectaAuriculares: "Conecta auriculares",
    elClienteTeOiria: "el cliente te oiría",

    // ---- fase 3 del sprint 002: por qué llegó la ficha, y la lectura que no encontró nada ----
    /**
     * **Los seis motivos del disparo** (`Motivo` en Rust), con su frase en los dos idiomas: la
     * banda los pinta junto a la latencia («pregunta · 1,2 s»). Mirada 17-bis; «en pantalla» es de
     * la 17-quater y lleva su propio símbolo.
     */
    motivos: {
      pregunta: "pregunta",
      cifra: "cifra",
      terminoDelCorpus: "término tuyo",
      silencioLargo: "silencio",
      atajo: "lo pediste",
      pantalla: "en pantalla",
    },
    seccionConjeturada: "sección conjeturada",
    leiLaPantalla: "Leí la pantalla: no hay texto que buscar.",

    // ---- fase 4 del sprint 002: el radar (C14), mirada 17 y 17-ter ----------------------------
    /** Ámbar, «sábelo»: lo que la pantalla de la reunión dice de quién graba. */
    radarGrabadaYBot: "Reunión grabada · bot de notas presente",
    radarGrabada: "Reunión grabada",
    radarBot: "Bot de notas presente",
    /** «Meet muestra el aviso de grabación y «MinutaBot» aparece en la ventana de la reunión. …» */
    radarMuestraElAviso: "muestra el aviso de grabación",
    radarY: "y",
    radarEnLaLista: "aparece en la ventana de la reunión.",
    radarNoEsAngel: "Ese bot no es Angel Ghost, que nunca entra a la llamada.",
    radarAvisoNoBloqueo: "Aviso, no bloqueo.",
    radarLeidoDeTuPantalla: "leído de tu pantalla",
    /** Coral, «invasivo»: un programa de tu Mac que te mira. */
    radarTeMira: "Te está mirando un programa de tu Mac",
    radarCatalogo: "catálogo",
    radarEnTuEquipo: "en tu equipo, no en el del cliente",
    radarQueVe: "qué ve",
    radarSigueProtegida: "tu banda sigue protegida",
    radarEnRed: "en red",
    radarNadaPersiste: "nada persiste",
    radarVerQueVe: "Ver qué alcanza a ver",
    radarCortaTodo: "Corta todo",
    /** Cómo nombra la banda cada clase del catálogo, entre paréntesis detrás del programa. */
    radarClases: {
      supervision: "supervisión de exámenes",
      "anti-trampa": "anti-trampa con acceso al sistema",
      monitoreo: "monitoreo de empleados",
      "acceso-remoto": "acceso remoto",
      mdm: "gestión de dispositivos",
    },
    // ---- fase 5 del sprint 002: la sugerencia (C7), mirada 18 ----------------------------------
    sugerenciaEnTuMac: "sugerencia · en tu Mac",
    /** «Claude Haiku · API · confianza media». */
    api: "API",
    confianzas: {
      alta: "confianza alta",
      media: "confianza media",
      baja: "confianza baja",
    },

    /**
     * MUESTRA SINTÉTICA «Páramo Azul» — la misma de la maqueta, con datos 100 % inventados.
     *
     * Vive aquí, y no en un módulo aparte, porque es texto bilingüe y el gate del diccionario lo
     * vigila igual que al resto: así no puede colarse contenido que la maqueta no dice.
     *
     * **Decía «muere en la fase 4», y no murió:** la fase 4 trajo el corpus real y cinco de estas
     * cadenas siguieron pintándose DENTRO del producto —incluida una frase puesta en boca del
     * cliente— hasta la fase 2 de la auditoría (hallazgo A1). Ya no. Hoy solo se pintan **fuera de
     * Tauri**, que es donde el arnés del gate de FIDELIDAD fotografía la banda para compararla con
     * la maqueta, y dentro del producto no se miran. Morirán cuando muera ese gate, no antes.
     */
    muestra: {
      oidoQuien: "cliente 14:02",
      oido: "Y la limpieza de datos, ¿eso está dentro del alcance?",
      buscado: "certificación ISO 27001",
      oidoIso: "¿Ustedes tienen certificación ISO 27001?",

      titular: "Limpieza de datos: incluida, hasta tres fuentes",
      linea: "Cubre perfilado y limpieza de ERP, POS y Excel de canal; una cuarta fuente es adicional.",
      lineaLarga:
        "Cubre perfilado y limpieza de ERP, POS y Excel de canal; una cuarta fuente es adicional y se cotiza aparte.",
      unidad: "propuesta",
      fuente: "Páramo Azul · §3.2 Alcance",

      acumulada1Unidad: "marco",
      acumulada1: "Etapa 2 Preparación: perfilar antes de modelar",
      acumulada2Unidad: "caso",
      acumulada2: "Sur del Valle: cuatro fuentes en 9 semanas",

      cercana: "marco · §5.1 Seguridad y manejo de datos",
      cercana1Unidad: "marco",
      cercana1: "§5.1 Seguridad y manejo de datos",
      cercana2Unidad: "propuesta",
      cercana2: "§6.3 Confidencialidad y acceso a sistemas",
      cercana3Unidad: "caso",
      cercana3: "Sur del Valle: auditoría interna del cliente",

      turno1: "Y la limpieza de datos, ¿eso está dentro del alcance?",
      turno2: "Sí, la limpieza de las tres fuentes que acordamos está incluida.",
      turno3: "¿Y si sumamos el Excel de la fuerza comercial? Lo tenemos en Power BI.",
      hora1: "14:01",
      hora2: "14:02",
      /** La hora de «lo pediste · 14:05», el estado «pantalla · nada que leer». */
      hora3: "14:05",
      titularPdf: "Cuatro fuentes integradas en 9 semanas",
      lineaPdf: "El perfilado previo recortó a la mitad la limpieza; la cuarta fuente entró en la semana 6.",
      fuentePdf: "Sur del Valle · Resultados",
      titularPantalla: "Tres canales, y no se sabe cuál deja margen",
      lineaPantalla: "Mayorista, tiendas de vereda y venta directa: la propuesta mide el margen de cada uno.",
      fuentePantalla: "Páramo Azul · Contexto",
      /** El radar de la maqueta: «MinutaBot» y «ProctorLince» son inventados, como todo aquí. */
      radarBot: "MinutaBot",
      radarHora: "14:03",
      radarPrograma: "ProctorLince",
      radarVe: "ve tu pantalla completa y tu cámara",
      /** La sugerencia de la maqueta (mirada 18). */
      sugerenciaTitular: "Tres fuentes incluidas, la cuarta aparte",
      sugerenciaLinea: "Confirma que la limpieza incluye hasta tres fuentes; una cuarta es adicional y se cotiza aparte.",
      nombreApi: "Claude Haiku",
      redApi: "1,2 KB",
    },
  },
  /**
   * EL CUADERNO — las pantallas de la ventana principal (960 × 640).
   *
   * Referencia: `docs/diseno/{sesion,permisos,honestidad}.html`, estado **«así se ve hoy · sprint
   * 1»** (mirada 12, aprobada el 2026-09-21). Igual que con la banda, cada cadena de aquí existe
   * TAL CUAL en la maqueta y el gate del diccionario lo vigila.
   */
  cuaderno: {
    // ---- el rail, compartido por las tres ----
    marca: "Angel Ghost",
    marcaSub: "cuaderno privado",
    // ---- corpus (fase 4) ----
    corpusTitulo: "Corpus",
    corpusSub: "Tus documentos, indexados donde están. De aquí sale cada ficha que verás en una reunión.",
    senalarCarpeta: "Señalar una carpeta",
    noSeCopianAntes: "Tus documentos",
    noSeCopianFuerte: "no se copian",
    noSeCopianDespues: ": se leen donde están.",
    dondeVive: "Dónde vive el índice",
    soloTu: "Solo tú puedes leerlo",
    corpusPendiente: "Lo que todavía no existe",
    arrastrar: "Arrastrar y soltar documentos",
    releer: "Releer solo lo que cambies",
    leerEscaneado: "Leer lo escaneado",
    corpusNota: "Hoy la app recorre la carpeta entera, hasta 2 000 documentos. Lo escaneado se queda fuera con su motivo, y nunca se manda a un servicio.",
    sinUnidad: "sin unidad",
    sinUnidadNota: "no encaja, y no se fuerza",
    uProp: "alcance, precio, supuestos",
    uMarco: "tu método, tus etapas",
    uCaso: "qué pasó y qué costó",
    uCliente: "fichas y acuerdos previos",
    uPerfil: "tu trayectoria y cifras",
    sinLeer: "sin leer",
    conSeccionesConjeturadas: "con secciones conjeturadas",
    conjeturadas: "Un PDF no trae títulos, trae líneas: la app conjetura dónde empieza cada sección por la forma del texto. Se cuenta para que puedas juzgarlo.",

    navSesion: "Sesión",
    navPermisos: "Permisos",
    navCorpus: "Corpus",
    navNotas: "Notas",
    navHonestidad: "Honestidad",
    navIdioma: "Idioma",
    navIa: "IA",
    sinSesion: "Sin sesión",
    detectado: "detectado",

    // ---- vocabulario de «todavía no» (§9-sexies) ----
    todaviaNo: "Todavía no",
    funciona: "Funciona",

    // ---- sesión ----
    sesionTitulo: "Antes de empezar",
    sesionSub: "Nada se enciende hasta que tú lo digas. Esto es lo que la app ve de tu propio Mac.",
    proteccionVerificada: "Protección verificada",
    proteccionDetalle:
      "Tu panel no aparece en la pantalla que compartes. Verificado en tu Mac (macOS 26.6.2) el 2026-09-20. En Zoom y Teams está sin verificar.",
    dosPistas: "Las dos pistas",
    pistaMic: "Micrófono — tú",
    pistaSistema: "Audio del sistema — el cliente",
    pistaPantalla: "Pantalla — solo lee lo nuevo",
    pistaAuriculares: "Auriculares conectados",
    esteCliente: "Este cliente",
    fichaYNda: "Ficha del cliente y NDA",
    noSeInventa: "No se inventa nada mientras no exista: ni bandera, ni riesgo, ni catálogo.",
    queFuncionaHoy: "Qué funciona hoy",
    funcionaBanda: "La banda, abajo, protegida de la captura",
    funcionaAcople: "La reunión se hace sitio: se acopla y vuelve al cerrar",
    funcionaCorte: "Corta todo y vacía la memoria",
    iniciarSesion: "Iniciar sesión",
    terminarSesion: "Terminar sesión",
    nadaSale: "corta todo · el sonido nunca se guarda · nada sale de tu equipo",
    nadaSaleConApi: "corta todo · el sonido nunca se guarda · solo sale texto anonimizado al proveedor que encendiste",
    sinReunion: "Sin reunión abierta",
    tituloDeMuestra: "Páramo Azul — Propuesta tablero de rentabilidad",
    sinReunionVoz: "Abre Zoom, Meet o Teams y aparecerá aquí. Mientras tanto, prepara la reunión.",

    // ---- permisos ----
    permisosTitulo: "Permisos de macOS",
    permisosSub: "Tú los concedes en el sistema, no aquí. La app funciona sin ellos: solo hace menos.",
    permMic: "Micrófono",
    permMicPara: "Tu voz, para saber cuándo hablas tú. Solo en memoria; nunca se graba.",
    permSistema: "Audio del sistema",
    permSistemaPara: "Lo que suena en tu Mac: la voz del cliente. Sin bot en la reunión.",
    permPantalla: "Pantalla",
    permPantallaPara: "Lee cifras y títulos solo si hay algo nuevo, como otra diapositiva. Las imágenes no se guardan.",
    permAcople: "Acoplar la ventana de la reunión",
    permAcoplePara:
      "Para que la banda no tape la llamada: la reunión se encoge y las dos conviven. macOS lo llama «Accesibilidad».",
    concedido: "Concedido",
    sinConceder: "Sin conceder",
    concederEnMacos: "Conceder en macOS",
    sinConcederNada: "Qué puedes hacer ya, sin conceder nada",
    indexar: "Indexar tu corpus",
    escribirNotas: "Escribir notas y acuerdos",
    buscarAMano: "Buscar tu evidencia a mano",
    textoMicrofono:
      "«Angel Ghost usa el micrófono para saber cuándo hablas tú. El audio vive solo en memoria y no se graba.»",

    // ---- honestidad ----
    honestidadTitulo: "Honestidad",
    honestidadSub: "Qué vive en la memoria ahora mismo y qué salió de tu equipo. Se demuestra, no se promete.",
    queViveEnMemoria: "Qué vive en la memoria ahora",
    bufMic: "Audio · micrófono",
    bufSistema: "Audio · sistema",
    bufTranscript: "Transcript",
    bufFrame: "Último frame leído",
    salieronDeTuEquipo: "salieron de tu equipo en esta reunión",
    modo: "Modo",
    modoLocal: "100 % local · API apagado",
    /**
     * **Esta frase decía otra cosa hasta la fase 2 de la auditoría**, y es el hallazgo A8: decía
     * «no existe código capaz de abrir una conexión», que era verdad al escribirla y dejó de serlo
     * en la fase 3, cuando el puente de voz ganó la descarga del modelo de macOS. Nadie volvió a
     * mirar la frase — y estaba en la pantalla de Honestidad, el peor sitio posible para una
     * afirmación caducada. Lo que ahora dice es comprobable, y lo comprueba
     * `tests/unit/contador-de-red.test.ts` contando las puertas una a una.
     */
    modoDetalle:
      "La app abre una conexión solo si enciendes el API en IA, con tu clave, y solo con texto anonimizado. La otra la abre macOS cuando le pides instalar un modelo de voz.",
    piezasCola: "piezas: la otra todavía no existe.",
    loQueQuedara: "Lo que quedará cuando cierres",
    // ---- sesión · lo que la fase 3 puso a funcionar ----
    funcionaEscucha: "Escucha las dos pistas y las transcribe en tu Mac",
    altavocesInternos: "Altavoces internos",
    avisoDelEco:
      "El micrófono también lo oye: se marca como eco.",

    // ---- honestidad · los búferes que ya existen ----
    ringBuffer30: "ring buffer · últimos 30 s",
    ventana12: "ventana de 12 turnos",

    // ---- idioma (subconjunto del sprint 1) ----
    idiomaTitulo: "Idioma y transcripción",
    idiomaSub:
      "Dos pistas, un idioma elegido para cada una y un diccionario tuyo. La transcripción trabaja; casi nunca la miras.",
    transcripcionEnVivo: "Transcripción en vivo",
    oculta: "oculta",
    visible: "visible",
    naceOculta:
      "Nace oculta a propósito. Leer lo que acaban de decir es la forma más rápida de dejar de escuchar: los ojos se van al texto y la conversación se queda sola.",
    laMuestraCuando: "la muestra cuando la necesites",
    idiomaPorPista: "Idioma por pista",
    idiomaDeTuMicrofono: "Idioma de tu micrófono",
    idiomaDelCliente: "Idioma del cliente",
    tuMicrofono: "Tú · micrófono",
    clienteSistema: "Cliente · sistema",
    modeloInstalado: "modelo instalado",
    /**
     * LOS TRES MOTIVOS por los que un idioma no se puede transcribir, **en el diccionario**.
     *
     * Estaban escritos en español dentro de `Idioma.tsx`, así que la interfaz inglesa enseñaba
     * «sin modelo» y «no lo reconoce» (hallazgo A6). Y el gate del diccionario no podía verlo:
     * compara `i18n/` con la maqueta, no barre los componentes. Desde la fase 2 de la auditoría
     * hay un barrido que sí lo hace.
     */
    sinModelo: "sin modelo",
    noLoReconoce: "no lo reconoce",
    sinMotorDeVoz: "sin motor",
    /** El botón que faltaba (A7): sin él, un Mac sin el modelo no tenía cómo conseguirlo. */
    instalarModelo: "Instalar",
    instalando: "instalando…",
    cincoIdiomas: "Cinco idiomas listos a la vez, como mucho: lo impone macOS, no la app.",
    transcribeTuMac: "Transcribe tu Mac, no un servicio",
    transcribeTuMacDetalle:
      "El motor de voz de macOS, dentro de tu equipo. Ningún audio sale para convertirse en texto. La única vez que toca la red es cuando pides instalar el modelo de un idioma: lo descarga macOS —mientras dura, instalando…— y si no puede: no lo reconoce · sin motor.",
    loQueTodaviaNo: "Lo que todavía no existe",
    variosIdiomasPorPista: "Varios idiomas a la vez, marcados por pista",
    loQueFaltaDetalle:
      "Hoy cada pista escucha un idioma, y los turnos mueren los dos —el tuyo y el del cliente— al cerrar y con la tecla.",
    conservarTusTurnos: "Conservar lo que dijiste tú",
    // ---- fase 3 del sprint 002: LOS PORQUÉS (mirada 17-quater, `kit.html` §8-ter) ----
    /**
     * Cada estado que no funciona dice por qué con una frase CERRADA. Rust manda la clave; la frase
     * vive aquí, en los dos idiomas, y termina en una salida siempre que la haya. Las claves son las
     * de los enums de Rust tal cual (`kebab-case`), para que un porqué nuevo sin frase no compile.
     */
    noAbrio: "No abrió",
    porQueNoAbrio: {
      "sin-permiso-del-microfono": "macOS no dio permiso al micrófono. Concédelo en Permisos y vuelve a «Iniciar sesión».",
      "sin-permiso-del-audio": "macOS no dio permiso para el audio del sistema. Concédelo en Permisos y vuelve a «Iniciar sesión».",
      "dispositivo-ocupado": "macOS no dejó crear el tap: otra app tiene el dispositivo. Ciérrala y vuelve a «Iniciar sesión».",
      "formato-ilegible": "El dispositivo entrega un formato de audio que la app no sabe leer. Prueba con otro y vuelve a «Iniciar sesión».",
      "no-dejo": "macOS no dejó abrir esta pista. Vuelve a «Iniciar sesión»; si se repite, reinicia el Mac.",
    },
    escuchaAMedias: "Escucha y transcribe en tu Mac —",
    soloTuPista: "solo tu pista",
    soloLaDelCliente: "solo la del cliente",
    aMedias: "A medias",
    pantallaEspera: "Espera la reunión",
    pantallaApagada: "Apagada",
    pantallaApagadaPor: "No la lee sola. Pídesela cuando quieras: ⌃⌥L.",
    pantallaSinPermiso: "Sin permiso",
    pantallaSinPermisoPor: "macOS no ha concedido la grabación de pantalla: la app no lee nada de ella. Concédela en Permisos.",
    pantallaNoPudo: "No pudo",
    pantallaNoPudoPor: "macOS no entregó la ventana de la reunión. La app lo vuelve a intentar sola.",
    leerlaSola: "Leerla sola",
    leelaAhora: "léela ahora",
    altavozExterno:
      "Por HDMI, DisplayPort o AirPlay el sonido sale por un altavoz: el micrófono va a oír al cliente y el modo solo audio no habla.",
    siEsUnAltavoz: "Si es un altavoz, no uses el modo solo audio.",
    noSeSabe: "No se sabe",
    /** Las dos últimas citan el dispositivo: la pantalla antepone su nombre entre comillas. */
    porQueNoSeSabe: {
      "sin-salida": "Este Mac no dice por dónde sale el sonido.",
      "sin-conexion": "no dice cómo está conectado.",
      "sin-fuente": "no dice por dónde suena.",
    },
    noSePuedeSaber: "No se puede saber si hay reunión",
    porQueNoSeVe: {
      "sin-accesibilidad": "Sin el permiso de Accesibilidad la app no puede ver si tienes Meet abierto. Concédelo en Permisos.",
    },
    dosPermisos: "Audio del sistema y Pantalla son dos permisos de macOS, aunque Ajustes los enseña en el mismo panel.",
    antesDeQueMacos: "Antes de que macOS te pregunte",
    textoPantalla:
      "«Angel Ghost solo lee lo nuevo de tu pantalla compartida, para reconocer cifras y títulos. Las imágenes viven en memoria y no se guardan ni salen de tu equipo.»",
    fraseDeMacos:
      "macOS pregunta con su propia frase y no deja poner otra: «Angel Ghost quiere hacer una captura del contenido de la pantalla del sistema».",
    soloEnMemoriaElUltimo: "solo en memoria · el último",
    terceraCosa: "Y una tercera cosa, que no muere al instante: la bandeja",
    noCorrio: "La tarea de borrado no corrió con la app cerrada: revisa Ítems de inicio",
    botonCorta: "El botón corta",
    de: "de",
    piezasNingunaFuera: "piezas: ninguna queda fuera.",
    idiomasListos: "idiomas listos a la vez, como mucho: lo impone macOS, no la app.",
    sinMotorTitulo: "Sin motor de voz: nada se transcribe",
    porQueNoHayMotor: {
      "sin-transcriptor": "Este Mac no trae el transcriptor de macOS 26.",
      "sin-puente": "Esta copia de la app se construyó sin el transcriptor.",
      "no-contesta": "El transcriptor de macOS no contestó. Vuelve a abrir la app; si se repite, reinicia el Mac.",
    },
    laBandaSigue: "Sin él no hay turnos, y sin turnos no llegan fichas por lo que se dice, tampoco con ⌃⌥A; las que trae la pantalla, sí.",
    tuDiccionario: "Tu diccionario técnico",
    deTuCorpus: "De tu corpus — nombres, productos, títulos",
    enTuArchivo: "En tu archivo",
    jamasCompleta: "Jamás completa una frase ni adivina una palabra.",
    tuCarpeta: "Tu carpeta",
    secciones: "secciones",

    // ---- fase 5 del sprint 002: la pantalla IA y el modo de Honestidad (C7, mirada 18) ----------
    iaSub: "Local por defecto. El API es tuyo, opcional, y se ve lo que sale por él.",
    quienRedacta: "Quién redacta la sugerencia",
    enTuMac: "en tu Mac",
    nadie: "nadie",
    proveedor: "proveedor",
    estadoColumna: "estado",
    latencia: "latencia",
    modeloDelSistema: "Modelo del sistema (macOS 26)",
    apiExterno: "API externo",
    enUso: "en uso",
    respaldo: "respaldo",
    noDisponible: "no disponible",
    apagado: "apagado",
    encendido: "encendido",
    porQueNoRedacta: {
      "apple-intelligence-apagado": "Apple Intelligence está apagado en Ajustes.",
      "mac-no-compatible": "Este Mac no puede usar el modelo del sistema.",
      "no-disponible": "macOS no deja usar el modelo del sistema ahora mismo.",
      "modelo-descargandose": "macOS todavía está descargando el modelo.",
      "sin-puente": "Esta copia de la app se construyó sin el modelo del sistema.",
      "sin-clave": "Sin clave: el API no se puede encender.",
      "tope-del-mes": "Llegaste al tope del mes: vuelve sola a lo local.",
    },
    siFallaQuedan: "Si el modelo falla, quedan las fichas",
    siFallaDetalle:
      "La sugerencia es un acento. Buscar en tu corpus y mostrar la ficha con su fuente es código, no modelo: funciona aunque apagues esto entero.",
    redactarSugerencias: "Redactar sugerencias (además de mostrar la ficha)",
    proveedorExterno: "Proveedor externo",
    tuClave: "Tu clave",
    enTuLlavero: "se guarda en tu Llavero, nunca en un archivo",
    guardarEnLlavero: "Guardar en el Llavero",
    borrarLaClave: "Borrar la clave",
    costo: "Costo",
    estaReunion: "Esta reunión",
    esteMes: "Este mes",
    topeDelMes: "Tope del mes",
    alLlegarAlTope: "Al llegar al tope vuelve sola al modelo local. No se detiene la reunión.",
    /** Honestidad con el API encendido: «API encendido · Claude Haiku». */
    modoApi: "API encendido",

    // ---- fase 4 del sprint 002: el radar en Sesión, «software invasivo en tu Mac» ---------------
    vigilanciaTitulo: "Software invasivo corriendo en tu Mac",
    vigilanciaNoEs: "No es una grabación de la reunión: son programas que miran",
    vigilanciaTuEquipo: "tu equipo",
    vigilanciaQueMiran: "— tu pantalla, tu cámara, tus teclas o tus procesos. El radar mira",
    vigilanciaSoloTuMac: "solo tu Mac",
    vigilanciaJamas: ", jamás el computador de la contraparte. Es un aviso para que decidas, no un bloqueo.",
    queEncontro: "qué encontró",
    queAlcanzaAVer: "qué alcanza a ver",
    nivel: "nivel",
    catalogo: "catálogo",
    invasivo: "Invasivo",
    sabelo: "Sábelo",
    tuProteccionSigue: "Tu protección propia sigue en pie",
    panelProtegido: "Panel protegido de la captura",
    aLaRed: "a la red",
    nadaDeLaReunionEnDisco: "nada de la reunión se escribe en disco.",
    iniciarDeTodosModos: "Iniciar de todos modos",
    noIniciar: "No iniciar",
    // Lo que salió al API (B37, sprint 003, mirada 19): la vista que abre «Ver lo que salió».
    verLoQueSalio: "Ver lo que salió",
    volverQuienRedacta: "Quién redacta",
    seBorraAlCerrar: "se borra al cerrar",
    loUltimoQueSalio: "Lo último que salió",
    estoEs: "Esto es",
    todoLoQueSalio: "todo",
    loQueSalioN: "lo que salió:",
    caracteresTachado: "caracteres. Tachado, lo que se reemplazó en tu Mac antes de enviarlo.",
    salieron: "salieron",
    peticiones: "peticiones",
    las: "Las",
    peticionesDeEstaReunion: "peticiones de esta reunión",
    colHora: "hora",
    colPorQue: "por qué salió",
    colCaracteres: "caracteres",
    colAnonimizados: "anonimizados",
    redactarSugerencia: "redactar sugerencia",
    registroEnMemoria: "Este registro vive en memoria como todo lo demás: al cerrar la reunión desaparece. Lo que persiste del costo es la cifra del mes, no el texto.",
    /** Las tres peticiones de la maqueta (`ia.html`, «sprint 3 · lo que salió»). Solo fuera de Tauri. */
    muestraSalio: {
      sobre1: "alcance",
      sobre2: "plazos",
      sobre3: "caso previo",
      t1: "¿El alcance de",
      t2: "incluye la limpieza de datos?",
      t3: "pregunta por una cuarta fuente.",
    },
    /** Las clases del catálogo como las titula la tabla: la palabra en negrita y su añadido. */
    radarClases: {
      supervision: { titulo: "Supervisión de exámenes", sufijo: "(proctoring)" },
      "anti-trampa": { titulo: "Anti-trampa con acceso al sistema", sufijo: "" },
      monitoreo: { titulo: "Monitoreo de empleados", sufijo: "" },
      "acceso-remoto": { titulo: "Acceso remoto activo", sufijo: "" },
      mdm: { titulo: "Gestión de dispositivos", sufijo: "(MDM) de un tercero" },
    },
    /** Las cinco filas de la maqueta, con sus nombres de ejemplo. Solo fuera de Tauri. */
    muestraRadar: {
      supervision: { nombre: "ProctorLince (ejemplo)", alcance: "Cámara, pantalla completa, apps abiertas; puede bloquear programas" },
      "anti-trampa": { nombre: "GuardKernel (ejemplo)", alcance: "Procesos y memoria de todo el equipo, desde el núcleo del sistema" },
      monitoreo: { nombre: "VigilaAgente (ejemplo)", alcance: "Capturas cada pocos minutos, tiempo por app, a veces teclas" },
      "acceso-remoto": { nombre: "RemotoYa (ejemplo)", alcance: "Alguien podría estar viendo tu pantalla ahora mismo" },
      mdm: { nombre: "MDM-Corp (ejemplo)", alcance: "Puede instalar, borrar y leer configuración. Normal en equipos de empresa" },
    },
  },
  /** Tus notas (C9, sprint 003): la pantalla de Notas y la tarjeta de Honestidad. Referencia: `notas.html` y `honestidad.html`. */
  notas: {
    titulo: "Notas",
    sub: "Lo que escribes tú. Es lo único de la reunión que llega al día siguiente.",
    cerrando: "Cerrando…",
    tuNota: "Tu nota",
    clienteNoLaVe: "el cliente no la ve",
    notaAyuda: "⌃⌥N te trae aquí desde la reunión, con el cursor al final. Mientras escuchas, esta ventana no se ve al compartir pantalla.",
    fichasQueFijaste: "Fichas que fijaste",
    acuerdos: "Acuerdos",
    escribeElAcuerdo: "Escribe el acuerdo y pulsa ↵",
    acuerdosAyuda: "Los marcas tú. La app no decide qué fue un acuerdo.",
    sobrevive: "Esto es lo único que va a sobrevivir a la reunión",
    sobreviveDetalle: "Lo que escribes aquí, las fichas que fijas y los acuerdos que marcas. Todo lo demás —las dos voces, el transcript, lo que se leyó de la pantalla— vive en memoria y muere al cerrar.",
    seVaAGuardar: "Se va a guardar",
    tusNotas: "Tus notas",
    parrafos: "párrafos",
    fichasFijadas: "Fichas fijadas",
    tituloYFuente: "titular y fuente",
    marcadosPorTi: "marcados por ti",
    tusTurnosEnTexto: "Tus turnos, en texto",
    loQueDijisteTu: "lo que dijiste tú",
    conservarMisTurnos: "Conservar mis turnos",
    conservarEncendido: "Lo encendiste tú; nace apagado.",
    conservarApagado: "Nace apagado, y así está: esta reunión no guardó tus turnos. Si lo enciendes, se guardan desde la próxima.",
    audioNunca: "El audio no se guarda nunca",
    audioNuncaCola: ", tampoco el tuyo: lo que queda de ti es texto.",
    muereAlCerrar: "Muere al cerrar",
    vozDelCliente: "Voz del cliente",
    sinInterruptor: "sin interruptor que lo cambie",
    susTurnos: "Sus turnos del transcript",
    turnos: "turnos",
    loLeido: "Lo leído de la pantalla",
    lecturas: "lecturas",
    tuPropioAudio: "Tu propio audio",
    tambienMuere: "también muere",
    sinCasilla: "Esta columna no tiene casilla. Es la regla, no una preferencia.",
    min: "min",
    cerrarSinGuardar: "Cerrar sin guardar",
    guardarCifrado: "Guardar cifrado y cerrar",
    noSeGuardo: "No se pudo guardar",
    noSeGuardoDetalle: "Tu nota sigue aquí, entera, y la reunión sigue abierta. Vuelve a intentarlo.",
    intentarOtraVez: "Intentar otra vez",
    cifrado: "cifrado",
    donde: "Dónde",
    carpetaDeLaApp: "Carpeta privada de la app",
    mostrarEnFinder: "Mostrar en Finder",
    tamano: "Tamaño",
    llave: "Llave",
    enTuLlavero: "en tu Llavero, ligada a este Mac",
    retencion: "Retención",
    seBorraSoloEn: "se borra solo en",
    dias: "días",
    delCliente: "Del cliente",
    nadaDelCliente: "nada. Ni su voz, ni sus turnos, ni su pantalla",
    exportarATexto: "Exportar a texto",
    borrarAhora: "Borrar ahora",
    exportarQuita: "Exportar a texto quita el cifrado",
    exportarDetalle: "El archivo exportado es tuyo y se lee en cualquier parte — por eso mismo ya no lo protege nadie.",
    seAvisaAntes: "Se avisa antes, cada vez.",
    exportarPregunta: "¿Exportar sin cifrado?",
    exportarSinCifrado: "Exportar sin cifrado",
    cancelar: "Cancelar",
    noSeExporto: "No se exportó",
    noSeExportoDetalle: "Tus notas siguen cifradas y en su sitio.",
    cuantoViven: "Cuánto viven tus notas",
    unAnio: "1 año",
    siempre: "siempre",
    cadaReunion: "Cada reunión que guardas vive eso, y se borra sola al vencer aunque no abras la app.",
    reunionesGuardadas: "Reuniones guardadas",
    colArchivo: "archivo",
    colFecha: "fecha",
    colTamano: "tamaño",
    colSeBorraEn: "se borra en",
    sinReuniones: "Todavía no has guardado ninguna reunión",
    sinReunionesDetalle: "Cuando cierres una sesión con notas, «Guardar cifrado y cerrar» la deja aquí, cifrada.",
    tuyo: "Tuyo",
    tuyoDetalle: "— tu nota, tus acuerdos, tus fichas fijadas y, si lo enciendes, tus turnos. Texto, cifrado.",
    deCliente: "Del cliente",
    delClienteDetalle: "— su voz, su transcript y las capturas",
    mueren: "mueren",
    yElAudio: ". Y el audio, también el tuyo: lo que queda de ti es texto.",
    siguenAhi: "Tus notas siguen ahí",
    borrarPregunta: "¿Borrar esta reunión?",
    borrarDetalle: "No hay copia en otro sitio: se borra ahora y no vuelve.",
    borrar: "Borrar",
    tePropongo: "Te propongo guardar esto",
    guardada: "guardada",
    guardadas: "guardadas",
    guardar: "Guardar",
    no: "No",
    sonReglas: "Son reglas, y esta es la lista entera: cifras, plazos y fechas · tus compromisos · lo que choca con una ficha que fijaste · nombres que no están en tu corpus · preguntas que te hicieron.",
    soloSobreviveLoTuyo: "Solo sobrevive lo tuyo: tu nota, tus fichas fijadas, tus acuerdos y las propuestas que guardes",
    lleno: "Llegaste a 30 propuestas esperando: las nuevas no entran hasta que decidas alguna.",
    loDijisteTu: "lo dijiste tú ·",
    loDijoElCliente: "lo dijo el cliente ·",
    chocaConUnaFicha: "choca con una ficha que fijaste",
    reglaCifra: "cifra y fecha",
    reglaCompromiso: "compromiso",
    reglaNombre: "nombre",
    reglaPregunta: "pregunta",
    dijeron: "Dijeron «",
    cierraComilla: "»",
    tuFichaDice: "»; tu ficha fijada dice «",
    cierraConPunto: "».",
    mencionaron: "Mencionaron a «",
    noEstaEnTuCorpus: "», que no está en tu corpus",
    tePreguntaron: "Te preguntaron por:",
    esperaranEnLaBandeja: "propuestas sin decidir esperarán en la bandeja",
    muerenAlCerrar: "propuestas sin decidir mueren al cerrar",
    ventanaAlCerrar: "al cerrar",
    ventanaFinDelDia: "fin del día",
    las: "Las",
    queGuardasteVan: "que guardaste van en tu archivo.",
    enLaBandejaDetalle: "Estas, cifradas, se borran solas al llegar a cero, aunque no abras la app.",
    ceroDetalle: "Elegiste «al cerrar»: no hay bandeja y no se escribe nada.",
    laReunionCerro: "La reunión cerró.",
    teQuedan: "Te quedan",
    paraDecidirDeLas: "para decidir qué guardar de las",
    propuestasAlLlegar: "propuestas. Al llegar a cero, la bandeja se borra sola.",
    paraDecidirCerrada: "para decidir qué guardar. Al llegar a cero, la bandeja se borra sola.",
    h: "h",
    esperanTuDecision: "Esperan tu decisión",
    guardadaYaEnTuArchivo: "guardada · ya está en tu archivo",
    de: "de",
    guardarTodas: "Guardar todas",
    descartarTodas: "Descartar todas",
    yaMurioAlCerrar: "Ya murió, al cerrar",
    audioDosPistas: "Audio · las dos pistas",
    transcriptDelCliente: "Transcript del cliente",
    lecturasDePantalla: "Lecturas de pantalla",
    bandejaGuarda: "La bandeja guarda",
    frases: "frases",
    noLaReunion: ", no la reunión. Esas tres murieron en el instante de cerrar, sin ventana ni casilla.",
    cuantoQuieresEsperar: "Cuánto quieres esperar",
    esUnaVentana: "Es una ventana, no un archivo",
    alLlegarACero: "Al llegar a cero se borra sola",
    aunqueNuncaVuelvas: "aunque nunca vuelvas a abrir la app",
    elPrecio: ". Vuelve cinco horas después y ya no está: es el precio de que tenga fondo.",
    irATusReuniones: "Ir a tus reuniones",
    cerradaConLlave: "Cerrada con llave: es de una sesión anterior de la app",
    paraLeerla: "Para leerla, Touch ID, como para abrir una reunión. Una vez por sesión de la app.",
    abrirConTouchId: "Abrir con Touch ID",
    bandejasDetras: "Bandejas detrás de esta:",
    bandejaChip: "Bandeja ·",
    vencidaChip: "Bandeja · vencida",
    laBandejaVencio: "La bandeja venció a las",
    ySeBorroSola: "y se borró sola.",
    queNoDecidisteMurieron: "propuestas que no decidiste murieron;",
    lasMinuscula: "las",
    queGuardasteEstan: "que guardaste están en tu archivo.",
    nadaLoQueGuardaste: "Nada. Lo que guardaste sigue en la reunión del",
    cifradoConSuRetencion: ", cifrado, con su retención.",
    yaMurio: "Ya murió",
    laBandejaPunto: "La bandeja ·",
    propuestas: "propuestas",
    lasTresPrimeras: "Las tres primeras murieron al cerrar; la bandeja, al vencer. Ninguna tiene copia.",
    /** Lo que dibuja la maqueta, para el arnés de fidelidad. Solo fuera de Tauri. */
    muestra: {
      nota1: "Piden la cuarta fuente (Excel de logística). Recordarles que va como adicional — está en §3.2.",
      nota2: "Andrea insiste en tener el tablero para el cierre de trimestre. Fecha real: 12 semanas desde la firma.",
      fijada1: "Limpieza de datos: hasta tres fuentes",
      fijada2: "Sur del Valle: 11 semanas reales",
      acuerdo1: "Cuarta fuente: cotización aparte",
      propuestaTuya: "Fecha real del tablero: 12 semanas desde la firma.",
      choqueDicho: "cuatro fuentes",
      choqueFicha: "tres",
      aceptada: "La cuarta fuente se cotiza aparte.",
    },
  },
};

/**
 * La FORMA del diccionario, no sus valores: obliga a que todo idioma tenga exactamente las
 * mismas claves, pero deja que cada uno diga lo suyo. Con `as const` a secas el tipo congelaba
 * también los textos y `en` no compilaba («"you" no es asignable a "tú"»).
 */
type Textos<T> = { [K in keyof T]: T[K] extends string ? string : Textos<T[K]> };

export type Diccionario = Textos<typeof es>;
