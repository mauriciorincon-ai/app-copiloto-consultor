import { createContext, useContext } from "react";
import { es } from "./es";
import { en } from "./en";
import type { Diccionario } from "./es";
import { CAMBIOS_DE_WINDOWS } from "./windows-cambios";

export type Idioma = "es" | "en";

const DICCIONARIOS: Record<Idioma, Diccionario> = { es, en };

/**
 * El idioma vive en `html[lang]`, igual que en la maqueta: un solo lugar de verdad que el
 * conmutador cambia y del que cuelga todo lo demás.
 */
export const IdiomaContext = createContext<Idioma>("es");

/**
 * **Cómo se escribe un texto en este sistema** (sprint 007). `null` en macOS: el diccionario sale tal cual, que es lo
 * que la maqueta fotografió. En Windows, la función que cambia las teclas de macOS por las de Windows
 * (`conTeclasDeEsteSistema`); la pone la cáscara cuando Rust dice qué sistema es.
 */
export const EsteSistemaContext = createContext<
  ((texto: string) => string) | null
>(null);

/** Cada texto, con las teclas de este sistema: en macOS, el mismo; en Windows, el que corresponde. */
export function useTeclas(): (texto: string) => string {
  return useContext(EsteSistemaContext) ?? IGUAL;
}

const IGUAL = (texto: string) => texto;

/** Recorre el diccionario cambiando cada texto con `teclas`; las funciones del diccionario cambian lo que devuelven. */
function reescribir(
  valor: unknown,
  teclas: (texto: string) => string,
): unknown {
  if (typeof valor === "string") return teclas(valor);
  if (typeof valor === "function") {
    const f = valor as (...a: unknown[]) => unknown;
    return (...a: unknown[]) => reescribir(f(...a), teclas);
  }
  if (Array.isArray(valor)) return valor.map((v) => reescribir(v, teclas));
  if (valor && typeof valor === "object") {
    return Object.fromEntries(
      Object.entries(valor).map(([clave, v]) => [clave, reescribir(v, teclas)]),
    );
  }
  return valor;
}

/** Pone `cambios` encima de `base`, clave por clave: lo que Windows dice distinto reemplaza al texto de macOS. */
function combinar(base: unknown, cambios: unknown): unknown {
  const esObjeto = (v: unknown): v is Record<string, unknown> =>
    typeof v === "object" && v !== null && !Array.isArray(v);
  if (!esObjeto(base) || !esObjeto(cambios)) return cambios ?? base;
  const salida: Record<string, unknown> = { ...base };
  for (const [clave, v] of Object.entries(cambios))
    salida[clave] = combinar(salida[clave], v);
  return salida;
}

/** El diccionario de Windows se arma una vez por función de teclas (la cáscara la guarda estable) y por idioma. */
const DE_WINDOWS = new WeakMap<object, Map<Idioma, Diccionario>>();

/** El diccionario de un idioma tal como se lee en Windows: sus cambios encima y las teclas de ese sistema. */
export function diccionarioDeWindows(
  idioma: Idioma,
  teclas: (texto: string) => string,
): Diccionario {
  let porIdioma = DE_WINDOWS.get(teclas);
  if (!porIdioma) DE_WINDOWS.set(teclas, (porIdioma = new Map()));
  let d = porIdioma.get(idioma);
  if (!d) {
    d = reescribir(
      combinar(DICCIONARIOS[idioma], CAMBIOS_DE_WINDOWS[idioma]),
      teclas,
    ) as Diccionario;
    porIdioma.set(idioma, d);
  }
  return d;
}

export function useT(): Diccionario {
  const idioma = useContext(IdiomaContext);
  const teclas = useContext(EsteSistemaContext);
  return teclas ? diccionarioDeWindows(idioma, teclas) : DICCIONARIOS[idioma];
}

export function useIdioma(): Idioma {
  return useContext(IdiomaContext);
}

export { es, en };
export type { Diccionario };
