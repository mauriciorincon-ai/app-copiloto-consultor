import { useIdioma, type Idioma } from "./index";

/**
 * «CONOCE ANGEL GHOST» — la entrada al recorrido visual (sprint 008, cierre del H-1).
 *
 * Vive fuera de `es.ts` y `en.ts` a propósito, como los textos de Windows: esos dos están atados a la maqueta por el
 * gate `i18n-fiel-a-la-maqueta`, y esta entrada no existe en ninguna maqueta. **Estado: maquetado, no visto.** La frase va
 * al gate humano del hito, con su línea en el cierre.
 *
 * Lo que dice es literal: abre un archivo local en el navegador y no toca la red (el contador de Honestidad no se mueve).
 */
export type TextosDeConoce = {
  titulo: string;
  detalle: string;
  boton: string;
  /** Si esta compilación no trae el recorrido, o el sistema no pudo abrirlo. */
  noSePudo: string;
};

const TEXTOS: Record<Idioma, TextosDeConoce> = {
  es: {
    titulo: "Conoce Angel Ghost",
    detalle:
      "Un recorrido visual, de lo general a lo particular. Se abre en tu navegador desde un archivo que viaja con la app: no toca la red.",
    boton: "Abrir el recorrido",
    noSePudo: "No se pudo abrir el recorrido en esta compilación.",
  },
  en: {
    titulo: "Get to know Angel Ghost",
    detalle:
      "A visual tour, from the general to the particular. It opens in your browser from a file that ships with the app: it does not touch the network.",
    boton: "Open the tour",
    noSePudo: "The tour could not be opened in this build.",
  },
};

export function useTextosDeConoce(): TextosDeConoce {
  return TEXTOS[useIdioma()];
}
