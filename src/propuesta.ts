import type { Diccionario } from "./i18n/es";
import type { Propuesta, Regla } from "./notas";

/**
 * **CÓMO SE LEE UNA PROPUESTA** (ADR 016 §2) — una sola vez, para Notas, la bandeja y la banda.
 *
 * Qué se guarda de cada lado lo decidió Rust (`propuestas/`, módulo protegido): de tus turnos, tu
 * frase; del cliente, un fragmento de ocho palabras como mucho, o las palabras clave de su pregunta.
 * Aquí solo se viste: del cliente, un hecho en una línea con su plantilla («Dijeron «12 semanas»»),
 * jamás su turno. Las plantillas son las de `docs/diseno/notas.html` («sprint 3 · las cinco reglas»),
 * en trozos, porque el gate del diccionario compara cada cadena tal cual con la maqueta.
 */
type Textos = Diccionario["notas"];

const ICONOS: Record<Regla, string> = {
  cifra: "i-reloj",
  compromiso: "i-nota",
  choque: "i-alert",
  nombre: "i-doc",
  pregunta: "i-chispa",
};

export function iconoDe(p: Propuesta): string {
  return ICONOS[p.regla];
}

/** La línea que se lee: tu frase, o el hecho que dejó el cliente. */
export function textoDe(p: Propuesta, t: Textos): string {
  if (p.de === "tuyo") return p.texto;
  switch (p.regla) {
    case "choque":
      return p.ficha
        ? `${t.dijeron}${p.texto}${t.tuFichaDice}${p.ficha}${t.cierraConPunto}`
        : `${t.dijeron}${p.texto}${t.cierraComilla}`;
    case "nombre":
      return `${t.mencionaron}${p.texto}${t.noEstaEnTuCorpus}`;
    case "pregunta":
      return `${t.tePreguntaron} ${p.texto}`;
    default:
      return `${t.dijeron}${p.texto}${t.cierraComilla}`;
  }
}

/** De dónde salió: «lo dijiste tú · 14:16», «lo dijo el cliente · 14:21», «choca con una ficha que fijaste». */
export function origenDe(p: Propuesta, t: Textos): string {
  if (p.regla === "choque") return t.chocaConUnaFicha;
  return `${p.de === "tuyo" ? t.loDijisteTu : t.loDijoElCliente} ${p.hora}`;
}

/** Qué regla saltó, tras el origen: «· cifra y fecha», o la sección de la ficha con que choca. */
export function etiquetaDe(p: Propuesta, t: Textos): string | null {
  switch (p.regla) {
    case "cifra":
      return t.reglaCifra;
    case "compromiso":
      return t.reglaCompromiso;
    case "nombre":
      return t.reglaNombre;
    case "pregunta":
      return t.reglaPregunta;
    case "choque":
      return p.seccion;
  }
}
