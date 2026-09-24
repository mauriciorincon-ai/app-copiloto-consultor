import { createContext, useContext } from "react";
import { es } from "./es";
import { en } from "./en";
import type { Diccionario } from "./es";

export type Idioma = "es" | "en";

const DICCIONARIOS: Record<Idioma, Diccionario> = { es, en };

/**
 * El idioma vive en `html[lang]`, igual que en la maqueta: un solo lugar de verdad que el
 * conmutador cambia y del que cuelga todo lo demás.
 */
export const IdiomaContext = createContext<Idioma>("es");

export function useT(): Diccionario {
  return DICCIONARIOS[useContext(IdiomaContext)];
}

export function useIdioma(): Idioma {
  return useContext(IdiomaContext);
}

export { es, en };
export type { Diccionario };
