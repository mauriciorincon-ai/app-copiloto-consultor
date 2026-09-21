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
};

/**
 * La FORMA del diccionario, no sus valores: obliga a que todo idioma tenga exactamente las
 * mismas claves, pero deja que cada uno diga lo suyo. Con `as const` a secas el tipo congelaba
 * también los textos y `en` no compilaba («"you" no es asignable a "tú"»).
 */
type Textos<T> = { [K in keyof T]: T[K] extends string ? string : Textos<T[K]> };

export type Diccionario = Textos<typeof es>;
