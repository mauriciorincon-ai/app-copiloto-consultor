import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { es } from "@/i18n/es";
import { en } from "@/i18n/en";

/**
 * Gate de las REGLAS PUBLICADAS (auditoría del S3, M11; ADR 016 §1 y su enmienda 3).
 *
 * «Qué sabe reconocer» promete «esta es la lista entera». La lista vive en `data/propuestas/reglas.json`
 * y la aplica Rust; la frase de la pantalla es de la maqueta (el diccionario tiene que ser fiel a ella).
 * Este gate las ata: la frase es exactamente su prefijo más el `corto` de cada regla del catálogo, en su
 * orden, unidos con « · » y con punto final. Si una regla se añade, se quita o se renombra sin tocar la
 * frase —o al revés—, es rojo.
 *
 * ¿Puede fallar? Sí: nació en rojo (el catálogo no tenía `corto`) y su demo quita una regla.
 */
type Regla = { id: string; corto?: { es: string; en: string } };
const CATALOGO = JSON.parse(readFileSync("data/propuestas/reglas.json", "utf8")) as { reglas: Regla[] };

const PREFIJO = { es: "Son reglas, y esta es la lista entera: ", en: "These are rules, and this is the whole list: " };

export function fraseEsperada(reglas: Regla[], idioma: "es" | "en"): string {
  return `${PREFIJO[idioma]}${reglas.map((r) => r.corto?.[idioma] ?? `(${r.id} sin corto)`).join(" · ")}.`;
}

describe("reglas-publicadas: la frase de la pantalla es la lista del catálogo", () => {
  it("el catálogo trae las cinco reglas, cada una con su «corto» en los dos idiomas", () => {
    expect(CATALOGO.reglas.length).toBeGreaterThanOrEqual(5);
    expect(CATALOGO.reglas.filter((r) => !r.corto?.es || !r.corto?.en).map((r) => r.id)).toEqual([]);
  });

  it.each([
    ["es", es.notas.sonReglas],
    ["en", en.notas.sonReglas],
  ] as const)("en %s, «sonReglas» sale del catálogo", (idioma, frase) => {
    expect(frase).toBe(fraseEsperada(CATALOGO.reglas, idioma));
  });

  it("el rojo: sin una regla, la frase ya no cuadra", () => {
    expect(es.notas.sonReglas).not.toBe(fraseEsperada(CATALOGO.reglas.slice(1), "es"));
  });
});
