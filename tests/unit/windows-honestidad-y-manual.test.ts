import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { PLATAFORMA_WINDOWS } from "../../src/contrato.generado";
import { CLAVES_DE_WINDOWS } from "../../src/plataforma";
import { TEXTOS_DE_WINDOWS } from "../../src/i18n/windows";

/**
 * «HOY EN WINDOWS» dice lo mismo en TRES sitios: el ADR 022 (la fuente), el diccionario de la app
 * (`src/i18n/windows.ts`, en los dos idiomas) y el manual (`docs/MANUAL-DE-USO.md`). Este test es la
 * comprobación de «pantalla = manual = ADR» del sprint 006: si una clave cambia de sitio, falla aquí.
 *
 * ¿Puede fallar? Sí: cambiar «sí» por «todavía no» en una sola fila del ADR, o quitar una clave del
 * manual, lo pone en rojo, con la clave y el sitio que no coincide.
 */

const ADR = readFileSync(
  "decisions/022-windows-detras-de-la-misma-frontera.md",
  "utf8",
);
const MANUAL = readFileSync("docs/MANUAL-DE-USO.md", "utf8");

/** Las filas de «Hoy en Windows» del ADR: clave → su estado en la frase de la tabla. */
function filasDelAdr(): Map<string, string> {
  const seccion = ADR.slice(ADR.indexOf("## Hoy en Windows"));
  const cuerpo = seccion.slice(0, seccion.indexOf("\n## ", 5));
  const filas = new Map<string, string>();
  for (const linea of cuerpo.split("\n")) {
    const m = /^\| `([a-z-]+)`\s*\|\s*(.+?)\s*\|$/.exec(linea);
    if (m) filas.set(m[1], m[2]);
  }
  return filas;
}

/** Del texto de la fila del ADR, el estado que dice: sí · no en Windows por ahora (desde el sprint 008, nada más). */
function estadoDelTexto(texto: string): "si" | "no-por-ahora" {
  if (texto.startsWith("sí")) return "si";
  if (texto.startsWith("no en Windows")) return "no-por-ahora";
  throw new Error(`no sé qué estado dice «${texto}»`);
}

/** Del detalle de un texto de la app, el mismo estado: el detalle empieza igual que la fila del ADR. */
function estadoDelDetalle(
  detalle: string,
  idioma: "es" | "en",
): "si" | "no-por-ahora" | "otro" {
  if (idioma === "es") {
    if (detalle.startsWith("sí")) return "si";
    if (detalle.startsWith("no en Windows")) return "no-por-ahora";
    return "otro";
  }
  if (detalle.startsWith("yes")) return "si";
  if (detalle.startsWith("not on Windows")) return "no-por-ahora";
  return "otro";
}

describe("«Hoy en Windows» dice lo mismo en el ADR, la app y el manual", () => {
  const filas = filasDelAdr();

  it("el ADR tiene las 15 claves, en el orden de la app", () => {
    expect([...filas.keys()]).toEqual([...CLAVES_DE_WINDOWS]);
  });

  it("cada clave tiene el MISMO estado en el ADR y en el contrato que emite Rust", () => {
    for (const f of PLATAFORMA_WINDOWS.funciones) {
      const enAdr = estadoDelTexto(filas.get(f.clave) ?? "");
      const enRust = f.cuando;
      expect(
        enAdr,
        `${f.clave}: el ADR dice una cosa y la app emite otra`,
      ).toBe(enRust);
    }
  });

  it("el español y el inglés de la app dicen el mismo estado que el ADR, clave por clave", () => {
    for (const clave of CLAVES_DE_WINDOWS) {
      const esperado = estadoDelTexto(filas.get(clave) ?? "");
      const es = TEXTOS_DE_WINDOWS.es.funciones[clave].detalle;
      const en = TEXTOS_DE_WINDOWS.en.funciones[clave].detalle;
      expect(estadoDelDetalle(es, "es"), `${clave} (es): «${es}»`).toBe(
        esperado,
      );
      expect(estadoDelDetalle(en, "en"), `${clave} (en): «${en}»`).toBe(
        esperado,
      );
    }
  });

  it("los dos idiomas tienen las mismas claves y ninguna vacía (paridad)", () => {
    for (const idioma of ["es", "en"] as const) {
      const t = TEXTOS_DE_WINDOWS[idioma];
      expect(Object.keys(t.funciones).sort()).toEqual(
        [...CLAVES_DE_WINDOWS].sort(),
      );
      for (const clave of CLAVES_DE_WINDOWS) {
        expect(
          t.funciones[clave].nombre.trim(),
          `${idioma}.${clave}.nombre`,
        ).not.toBe("");
        expect(
          t.funciones[clave].detalle.trim(),
          `${idioma}.${clave}.detalle`,
        ).not.toBe("");
      }
      for (const [k, v] of Object.entries(t)) {
        if (typeof v === "string")
          expect(v.trim(), `${idioma}.${k}`).not.toBe("");
      }
    }
  });

  it("el manual tiene su sección de Windows y nombra las 15 claves con el mismo estado", () => {
    const seccion = MANUAL.slice(MANUAL.indexOf("## Angel Ghost en Windows"));
    const cuerpo = seccion.slice(0, seccion.indexOf("\n## ", 5));
    expect(
      cuerpo.length,
      "el manual no tiene la sección de Windows",
    ).toBeGreaterThan(500);
    for (const clave of CLAVES_DE_WINDOWS) {
      const fila = cuerpo
        .split("\n")
        .find((l) => l.startsWith(`| \`${clave}\` |`));
      expect(fila, `el manual no tiene la fila de «${clave}»`).toBeDefined();
      const dicho = estadoDelTexto(fila!.split("|")[2]!.trim());
      expect(dicho, `${clave}: el manual dice una cosa y el ADR otra`).toBe(
        estadoDelTexto(filas.get(clave) ?? ""),
      );
    }
  });

  it("el manual dice que la invisibilidad está sin verificar, y no la promete", () => {
    const seccion = MANUAL.slice(MANUAL.indexOf("## Angel Ghost en Windows"));
    expect(seccion).toMatch(/sin verificar/);
    expect(seccion).toMatch(/Teams, Zoom ni Meet/);
  });
});

describe("el lenguaje de Windows cumple la regla 6 del vocabulario", () => {
  const vetadas = /indetectable|trampa|enga[ñn]ar|invisible|evadir|burlar/i;

  it("ni los textos de la app ni el manual de Windows usan las palabras vetadas", () => {
    const textos =
      JSON.stringify(TEXTOS_DE_WINDOWS.es) +
      JSON.stringify(TEXTOS_DE_WINDOWS.en);
    expect(textos).not.toMatch(vetadas);
    const seccion = MANUAL.slice(
      MANUAL.indexOf("## Angel Ghost en Windows"),
      MANUAL.indexOf("## Atajos de teclado"),
    );
    expect(seccion).not.toMatch(vetadas);
  });
});

describe("lo que Windows dice de la red, con la puerta única", () => {
  it("la frase de Honestidad en Windows nombra la única descarga que la app hace por su cuenta (ADR 022 §4)", () => {
    // Hallazgo A4 de la auditoría del S6: la frase no puede decir «macOS» ni dejar la puerta sin nombrar.
    expect(TEXTOS_DE_WINDOWS.es.honestidad.modoDetalle).toContain("instalar el modelo de voz");
    expect(TEXTOS_DE_WINDOWS.en.honestidad.modoDetalle).toContain("install the speech model");
  });
});

describe("la cuenta de turnos sin transcribir no atribuye (auditoría del S7, M3)", () => {
  it("no dice «tuyos» ni «your»: en la sala el turno que se tira puede ser del cliente", () => {
    for (const n of [1, 3]) {
      expect(TEXTOS_DE_WINDOWS.es.whisper.descartados(n)).not.toMatch(/tuy/i);
      expect(TEXTOS_DE_WINDOWS.en.whisper.descartados(n)).not.toMatch(/\byour\b/i);
    }
  });

  it("el manual cuenta lo mismo que el código: tuyos primero y, sin tuyos, el más viejo", () => {
    const manual = MANUAL.slice(MANUAL.indexOf("Si tu PC no da abasto"));
    const frase = manual.slice(0, manual.indexOf("\n- **"));
    expect(frase).toMatch(/primero los \*\*tuyos\*\*/);
    expect(frase).toMatch(/el turno más viejo/);
    expect(frase).not.toMatch(/nunca los del cliente/);
  });
});

/**
 * **NINGÚN TEXTO PROMETE «1,6 GB» SIN DECIR LO DE ARM64** (sprint 009, auditoría A3): en un Windows ARM64 «Instalar» baja
 * el modelo pequeño (0,5 GB), no el grande. Una frase de la app sobre sí misma que dijera «1,6 GB» a secas mentiría
 * justo en el equipo al que apunta el sprint. ¿Puede fallar? Sí: quitar «0,5 GB» de cualquiera de las cadenas es rojo.
 */
describe("los textos de Windows no prometen un tamaño que ARM64 no baja", () => {
  const cadenas = (valor: unknown, ruta: string): [string, string][] =>
    typeof valor === "string"
      ? [[ruta, valor]]
      : valor && typeof valor === "object"
        ? Object.entries(valor).flatMap(([k, v]) => cadenas(v, `${ruta}.${k}`))
        : [];

  it.each(["es", "en"] as const)("%s: toda cadena con «1,6 GB» dice también los 0,5 GB de ARM64", (idioma) => {
    const todas = cadenas(TEXTOS_DE_WINDOWS[idioma], idioma);
    const con16 = todas.filter(([, t]) => /1[,.]6 GB/.test(t));
    expect(con16.length, "ya no hay ninguna cadena con el tamaño del modelo grande: ¿se perdió la advertencia?").toBeGreaterThan(0);
    const sinArm = con16.filter(([, t]) => !/0[,.]5 GB/.test(t) || !/ARM64/.test(t)).map(([ruta]) => ruta);
    expect(sinArm, "cadenas que dicen 1,6 GB sin el caso de ARM64").toEqual([]);
  });
});
