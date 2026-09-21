/**
 * Diccionario ES — cadenas VERBATIM de la maqueta aprobada en G-Diseño.
 *
 * Contrato §9 del design system: «en producto los pares `<span lang>` se sustituyen por el
 * diccionario i18n con las MISMAS cadenas que la maqueta — la maqueta es el primer diccionario».
 * `tests/unit/i18n-fiel-a-la-maqueta.test.ts` lo vigila: cada cadena de aquí debe existir tal
 * cual en `docs/diseno/`. Si el producto necesita decir algo que la maqueta no dice, primero se
 * escribe en la maqueta (es una decisión de diseño), no aquí.
 */
export const es = {
  banda: {
    // barra de estado
    escuchando: "Escuchando",
    dosPistas: "2 pistas",
    pantalla: "pantalla",
    protegidoMeet: "Meet · protegido",
    sinVerificarZoom: "Zoom · sin verificar",
    corpus: "143 documentos · 5 unidades",
    reunion: "Páramo Azul · 12 min",

    // esperando
    esperandoVoz: "Cuando el cliente pregunte, aquí aparece tu evidencia.",

    // buscando
    buscando: "Buscando en tu corpus…",
    oidoQuien: "cliente 14:02",
    oidoQue: "Y la limpieza de datos, ¿eso está dentro del alcance?",

    // ficha
    fichaTitular: "Limpieza de datos: incluida, hasta tres fuentes",
    fichaLinea:
      "Cubre perfilado y limpieza de ERP, POS y Excel de canal; una cuarta fuente es adicional.",
    fichaUnidad: "propuesta",
    fichaFuente: "Páramo Azul · §3.2 Alcance",
    fichaMas: "Marco · etapa 2 Preparación — Caso Sur del Valle: cuatro fuentes",

    // sin resultado
    sinResultado: "Nada en tu corpus sobre esto.",
    sinResultadoBuscar: "Buscar con otras palabras",
    sinResultadoAnotar: "Anotar para después",

    // sin verificar en este cliente
    sinVerificarTitulo: "Zoom: protección sin verificar",
    sinVerificarLinea: "Verificada solo en Meet · macOS 26.6.2 · 2026-09-20",
    sinVerificarVentana: "Comparte una ventana, no la pantalla",
    sinVerificarMonitor: "O usa un segundo monitor",
    sinVerificarNotas: "O pasa a modo solo notas",

    // transcript
    transcriptCab: "en vivo · solo en memoria",
    transcriptOcultar: "ocultar",
    transcriptCliente: "cliente",
    transcriptTu: "tú",

    // atajos
    fijar: "fijar",
    ayudame: "ayúdame",
    corta: "corta",
  },
};

/**
 * La FORMA del diccionario, no sus valores: obliga a que todo idioma tenga exactamente las
 * mismas claves, pero deja que cada uno diga lo suyo. Con `as const` a secas el tipo congelaba
 * también los textos y `en` no compilaba («"you" no es asignable a "tú"»).
 */
type Textos<T> = { [K in keyof T]: T[K] extends string ? string : Textos<T[K]> };

export type Diccionario = Textos<typeof es>;
