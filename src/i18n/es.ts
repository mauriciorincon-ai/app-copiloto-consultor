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
    // ---- cabecera: estado de la sesión ----
    escuchando: "Escuchando · 2 pistas",
    protegido: "Meet · protegido",
    sinVerificar: "Zoom · sin verificar",
    sinAcople: "sin acople",

    // ---- esperando ----
    esperando: "Cuando el cliente pregunte, aquí aparece tu evidencia.",
    corpus: "143 documentos · 5 unidades",
    reunion: "Páramo Azul · 12 min",

    // ---- buscando ----
    buscando: "Buscando en tu corpus…",

    // ---- sin resultado: el veredicto, la maniobra y lo más cercano ----
    // La maniobra sale de un catálogo versionado elegido por reglas léxicas. NO es una
    // sugerencia del modelo y no puede parecerlo (design-system §9-quinquies).
    nada: "Nada en tu corpus sobre «certificación ISO 27001»",
    maniobra: "Dilo sin adornos y ofrece confirmarlo hoy mismo.",
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
    sinVerificarNotas: "O pasa a modo solo notas",
    yaVerifique: "Ya lo verifiqué en Zoom",
    soloNotas: "Modo solo notas",

    // ---- transcript ----
    transcriptCab: "en vivo · solo en memoria",
    ocultar: "ocultar",
    cliente: "cliente",
    tu: "tú",

    // ---- atajos ----
    fijar: "fijar",
    ayudame: "ayúdame",
    corta: "corta",
    transcript: "transcript",

    /**
     * MUESTRA SINTÉTICA «Páramo Azul» — la misma de la maqueta, con datos 100 % inventados.
     *
     * Vive aquí, y no en un módulo aparte, porque es texto bilingüe y el gate del diccionario lo
     * vigila igual que al resto: así no puede colarse contenido que la maqueta no dice. **Muere
     * en la fase 4**, cuando el corpus real alimente la banda; hasta entonces es lo que hace
     * posible el gate de FIDELIDAD, que compara la banda construida contra la maqueta.
     */
    muestra: {
      oidoQuien: "cliente 14:02",
      oido: "Y la limpieza de datos, ¿eso está dentro del alcance?",
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
    navSesion: "Sesión",
    navPermisos: "Permisos",
    navCorpus: "Corpus",
    navNotas: "Notas",
    navHonestidad: "Honestidad",
    navIdioma: "Idioma",
    navIa: "IA",
    sinSesion: "Sin sesión · 0 B",
    meetDetectado: "Meet detectado · 0 B",

    // ---- vocabulario de «todavía no» (§9-sexies) ----
    todaviaNo: "Todavía no",
    funciona: "Funciona",

    // ---- sesión ----
    sesionTitulo: "Antes de empezar",
    sesionSub: "Nada se enciende hasta que tú lo digas. Esto es lo que la app ve de tu propio Mac.",
    proteccionVerificada: "Protección verificada",
    proteccionSinVerificar: "Zoom · sin verificar",
    proteccionDetalle:
      "Tu panel no aparece en la pantalla que compartes. Verificado en tu Mac (macOS 26.6.2) el 2026-09-20. En Zoom y Teams está sin verificar.",
    dosPistas: "Las dos pistas",
    pistaMic: "Micrófono — tú",
    pistaSistema: "Audio del sistema — el cliente",
    pistaPantalla: "Pantalla — solo cuando cambia",
    pistaAuriculares: "Auriculares conectados",
    esteCliente: "Este cliente",
    fichaNdaRadar: "Ficha del cliente, NDA y radar",
    noSeInventa: "No se inventa nada mientras no exista: ni bandera, ni riesgo, ni catálogo.",
    queFuncionaHoy: "Qué funciona hoy",
    funcionaBanda: "La banda, abajo, protegida de la captura",
    funcionaAcople: "La reunión se hace sitio: se acopla y vuelve al cerrar",
    funcionaCorte: "Corta todo y vacía la memoria",
    iniciarSesion: "Iniciar sesión",
    nadaSale: "el sonido nunca se guarda · nada sale de tu equipo",
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
    permPantallaPara: "Lee cifras y títulos solo cuando la pantalla cambia. Las imágenes no se guardan.",
    permAcople: "Acoplar la ventana de la reunión",
    permAcoplePara:
      "Para que la banda no tape la llamada: la reunión se encoge y las dos conviven. macOS lo llama «Accesibilidad».",
    concedido: "Concedido",
    sinConceder: "Sin conceder",
    concederEnMacos: "Conceder en macOS",
    unSoloPermiso: "Audio del sistema y Pantalla son un solo permiso en macOS: se conceden y se caen juntos.",
    sinConcederNada: "Qué puedes hacer ya, sin conceder nada",
    indexar: "Indexar tu corpus",
    escribirNotas: "Escribir notas y acuerdos",
    buscarAMano: "Buscar tu evidencia a mano",
    queTextoVeras: "Qué texto verás en macOS",
    textoMicrofono:
      "«Angel Ghost usa el micrófono para saber cuándo hablas tú. El audio vive solo en memoria y no se graba.»",
    claveMicrofono: "NSMicrophoneUsageDescription · es / en",

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
    modoDetalle:
      "En esta versión no existe código capaz de abrir una conexión: el cero no se mantiene por disciplina.",
    piezasCola: "piezas: las otras cuatro todavía no existen.",
    loQueQuedara: "Lo que quedará cuando cierres",
    loQueQuedaraDetalle:
      "Notas, acuerdos y fichas fijadas llegan más adelante. Hoy no queda nada porque hoy no se escribe nada.",
  },
};

/**
 * La FORMA del diccionario, no sus valores: obliga a que todo idioma tenga exactamente las
 * mismas claves, pero deja que cada uno diga lo suyo. Con `as const` a secas el tipo congelaba
 * también los textos y `en` no compilaba («"you" no es asignable a "tú"»).
 */
type Textos<T> = { [K in keyof T]: T[K] extends string ? string : Textos<T[K]> };

export type Diccionario = Textos<typeof es>;
