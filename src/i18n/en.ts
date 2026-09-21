import type { Diccionario } from "./es";

/**
 * Diccionario EN — cadenas VERBATIM de la maqueta aprobada en G-Diseño.
 * El tipo `Diccionario` obliga a que ningún idioma se quede atrás: si `es` gana una clave,
 * esto no compila hasta que `en` la tenga. Bilingüe no es una intención, es el tipo.
 */
export const en: Diccionario = {
  banda: {
    escuchando: "Listening",
    dosPistas: "2 tracks",
    pantalla: "screen",
    protegidoMeet: "Meet · protected",
    sinVerificarZoom: "Zoom · not verified",
    corpus: "143 documents · 5 units",
    reunion: "Páramo Azul · 12 min",

    esperandoVoz: "When the client asks, your evidence shows up here.",

    buscando: "Searching your corpus…",
    oidoQuien: "client 14:02",
    oidoQue: "And the data cleansing, is that within scope?",

    fichaTitular: "Data cleansing: included, up to three sources",
    fichaLinea:
      "Covers profiling and cleansing of ERP, POS and channel Excel; a fourth source is an add-on.",
    fichaUnidad: "proposal",
    fichaFuente: "Páramo Azul · §3.2 Scope",
    fichaMas: "Framework · stage 2 Preparation — Sur del Valle case: four sources",

    sinResultado: "Nothing in your corpus about this.",
    sinResultadoBuscar: "Search with other words",
    sinResultadoAnotar: "Note it for later",

    sinVerificarTitulo: "Zoom: protection not verified",
    sinVerificarLinea: "Verified only in Meet · macOS 26.6.2 · 2026-09-20",
    sinVerificarVentana: "Share a window, not the screen",
    sinVerificarMonitor: "Or use a second monitor",
    sinVerificarNotas: "Or switch to notes-only mode",

    transcriptCab: "live · in memory only",
    transcriptOcultar: "hide",
    transcriptCliente: "client",
    transcriptTu: "you",

    fijar: "pin",
    ayudame: "help me",
    corta: "cut",
  },
};
