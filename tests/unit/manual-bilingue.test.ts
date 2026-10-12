import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { CLAVES_DE_WINDOWS } from "../../src/plataforma";

/**
 * **EL MANUAL EN INGLÉS ES EL MANUAL, TRADUCIDO** (sprint 008, cierre del H-1; regla 20: bilingüe en todo).
 *
 * `docs/MANUAL-DE-USO.md` (es) y `docs/MANUAL-EN.md` (en) dicen lo mismo en dos idiomas. El test no lee el inglés: ata
 * su ESTRUCTURA al español, que es lo que una traducción que se desvía rompe primero —una sección que desaparece, una fila
 * de tabla que se pierde, una viñeta fusionada, una tecla que cambia—:
 *
 *  1. los mismos encabezados `#`, `##` y `###`, en el mismo orden y con el mismo nivel;
 *  2. por sección, el mismo número de filas de tabla, de viñetas y de fragmentos de código (`...`);
 *  3. las veinte funciones de «Features» en las dos lenguas (el conteo N = 20 del brochure sale de aquí);
 *  4. las quince filas de «Hoy en Windows» llevan las claves del ADR 022, en su orden, en las dos;
 *  5. los fragmentos de código son los mismos, salvo las teclas de Windows (`Alt+Mayús` → `Alt+Shift`).
 *
 * ¿Puede fallar? Sí: quitar una fila de una tabla, un `###`, o dejar `Alt+Mayús+Q` en el inglés, es rojo con la sección.
 * (Demo en rojo del sprint 008.)
 */
type Seccion = { nivel: number; titulo: string; filas: number; viñetas: number; codigo: string[] };

function leer(ruta: string): { secciones: Seccion[]; texto: string } {
  const texto = readFileSync(ruta, "utf8");
  const secciones: Seccion[] = [];
  for (const linea of texto.split("\n")) {
    const h = /^(#{1,3}) (.+)$/.exec(linea);
    if (h) {
      secciones.push({ nivel: h[1].length, titulo: h[2], filas: 0, viñetas: 0, codigo: [] });
      continue;
    }
    const actual = secciones.at(-1);
    if (!actual) continue;
    if (linea.startsWith("|")) actual.filas++;
    if (/^\s*[-*] /.test(linea)) actual.viñetas++;
    actual.codigo.push(...[...linea.matchAll(/`([^`]+)`/g)].map((m) => m[1]));
  }
  return { secciones, texto };
}

const ES = leer("docs/MANUAL-DE-USO.md");
const EN = leer("docs/MANUAL-EN.md");
/** Las teclas de Windows en español, tal como las escribe el manual en español. */
const alIngles = (c: string) => c.replaceAll("Mayús", "Shift");

describe("el manual en inglés sigue la estructura del manual en español", () => {
  it("tiene los mismos encabezados, en el mismo orden y nivel", () => {
    expect(EN.secciones.map((s) => s.nivel)).toEqual(ES.secciones.map((s) => s.nivel));
    expect(EN.secciones.length).toBeGreaterThan(25);
  });

  it("cada sección tiene las mismas filas de tabla, viñetas y fragmentos de código", () => {
    const diferencias: string[] = [];
    ES.secciones.forEach((es, i) => {
      const en = EN.secciones[i];
      if (!en) return;
      const donde = `«${es.titulo}» / «${en.titulo}»`;
      if (es.filas !== en.filas) diferencias.push(`${donde}: ${es.filas} filas de tabla contra ${en.filas}`);
      if (es.viñetas !== en.viñetas) diferencias.push(`${donde}: ${es.viñetas} viñetas contra ${en.viñetas}`);
      if (es.codigo.length !== en.codigo.length) diferencias.push(`${donde}: ${es.codigo.length} fragmentos de código contra ${en.codigo.length}`);
    });
    expect(diferencias, diferencias.join("\n")).toEqual([]);
  });

  it("los fragmentos de código son los mismos (teclas de Windows aparte)", () => {
    const diferencias: string[] = [];
    ES.secciones.forEach((es, i) => {
      const en = EN.secciones[i];
      if (!en) return;
      const a = es.codigo.map(alIngles).sort();
      const b = [...en.codigo].sort();
      a.forEach((c, k) => {
        if (c !== b[k]) diferencias.push(`«${es.titulo}»: «${c}» contra «${b[k]}»`);
      });
    });
    expect(diferencias.slice(0, 12), diferencias.join("\n")).toEqual([]);
    expect(EN.texto, "queda una tecla de Windows en español").not.toMatch(/Alt\+Mayús/);
  });

  it("las funciones de «Features» son las mismas en las dos lenguas (su cuenta es el N del brochure)", () => {
    const funciones = (m: typeof ES) => {
      const desde = m.secciones.findIndex((s) => s.nivel === 2 && /^(Features)$/.test(s.titulo));
      expect(desde, "no encontré «Features»").toBeGreaterThan(-1);
      const resto = m.secciones.slice(desde + 1);
      const hasta = resto.findIndex((s) => s.nivel === 2);
      return resto.slice(0, hasta === -1 ? undefined : hasta).filter((s) => s.nivel === 3);
    };
    expect(funciones(ES).length, "el manual no tiene funciones").toBeGreaterThan(0);
    expect(funciones(EN)).toHaveLength(funciones(ES).length);
  });

  it("«Hoy en Windows» lleva las quince claves del ADR 022 en las dos lenguas, en su orden", () => {
    for (const [lengua, m] of [["es", ES], ["en", EN]] as const) {
      const claves = [...m.texto.matchAll(/^\| `([a-z-]+)`\s*\|/gm)].map((x) => x[1]).filter((c) => (CLAVES_DE_WINDOWS as readonly string[]).includes(c));
      expect(claves, `la tabla de ${lengua}`).toEqual([...CLAVES_DE_WINDOWS]);
    }
  });

  it("el inglés no promete lo que el vocabulario de la casa prohíbe", () => {
    expect(EN.texto).not.toMatch(/undetectable|cheat|stealth|invisible mode|trick the|deceive/i);
  });
});
