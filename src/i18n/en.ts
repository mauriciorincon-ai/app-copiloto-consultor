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

    nada: "Nothing in your corpus about “ISO 27001 certification”",
    maniobra: "Say so plainly and offer to confirm it today.",
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
};
