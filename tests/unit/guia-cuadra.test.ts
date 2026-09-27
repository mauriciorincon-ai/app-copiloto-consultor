import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * Gate de la GUÍA DE PRUEBA — **lo que la cabecera promete es lo que el cuerpo tiene.**
 *
 * La guía declara sus cuentas en prosa («Gate mínimo ⭐: 60 pruebas», «9 paradas», «deja fuera 51»)
 * y las pruebas llevan sus marcas en atributos (`data-minimo`, `data-corto`). Nadie las cruzaba: al
 * añadir o mover una prueba, la cabecera seguía diciendo la cifra vieja, y un gate que «son 33» y
 * tiene 35 se corre mal sin que nadie lo note. Además el ⭐⭐ es un RECORRIDO: sus paradas van
 * numeradas en el orden del documento, sin huecos, y cada una dice «de M» con la misma M.
 *
 * ¿Puede fallar? Sí: basta añadir una prueba ⭐ sin tocar la cabecera, o numerar dos paradas
 * iguales. Demo en rojo en la bitácora del sprint 002, fase 6.
 *
 * **Desde el sprint 003** mira además lo que el ⭐⭐ de la v4 no podía decir de sí mismo: **cuánto
 * dura** (cada parada y la preparación llevan `data-min`, y la suma no pasa del techo de ~20 min que
 * la cabecera declara), **de qué sprint es cada ⭐** (`data-nace`, el desglose S1 · S2 · S3), **cuántas
 * filas tiene el bloque de textos diferidos**, y que **el chip de cada prueba diga lo mismo que su
 * origen** con el namespace del sprint de la guía: al heredar una guía hay que pasar todos los «Nuevo»
 * a «S2» a secas, y uno olvidado se lee como algo que cambió. Rojos en la bitácora del sprint 003, fase 5.
 */
const GUIA = readFileSync("docs/GUIA-DE-PRUEBA.html", "utf8");
const PRUEBAS = [
  ...GUIA.matchAll(
    /<li (data-origen="(\w+)"[^>]*)><input type="checkbox" id="(\w+)">/g,
  ),
];

const FILAS = [...GUIA.matchAll(/<tr data-texto><td><input type="checkbox" id="(\w+)"/g)];

/** «Sprint 003» → 3: el sprint de la guía, de su cabecera. */
const SPRINT = Number(/Sprint 0*(\d+) · /.exec(GUIA)?.[1]);

/** «1,5» → 1.5: los minutos se escriben en castellano. */
const minutos = (t: string) => Number(t.replace(",", "."));

function declarada(patron: RegExp): number {
  const m = patron.exec(GUIA);
  expect(m, `la cabecera no declara ${patron}`).not.toBeNull();
  return Number(m![1]);
}

describe("la guía de prueba cuadra", () => {
  it("el gate mínimo ⭐ declara las pruebas que marca", () => {
    const marcadas = PRUEBAS.filter((p) => p[1].includes("data-minimo")).length;
    expect(declarada(/Gate mínimo ⭐: (\d+) pruebas/)).toBe(marcadas);
  });

  it("el gate corto ⭐⭐ declara sus paradas, y las que deja fuera son las ⭐ que no entran", () => {
    const minimo = PRUEBAS.filter((p) => p[1].includes("data-minimo")).length;
    const corto = PRUEBAS.filter((p) => p[1].includes("data-corto"));
    expect(declarada(/Gate corto ⭐⭐: (\d+) paradas/)).toBe(corto.length);
    expect(declarada(/Deja fuera (\d+) pruebas/)).toBe(minimo - corto.length);
    // Toda parada ⭐⭐ es también ⭐: el corto es una lente sobre el mínimo, no otro conjunto.
    expect(
      corto.filter((p) => !p[1].includes("data-minimo")).map((p) => p[3]),
    ).toEqual([]);
  });

  it("las paradas son un recorrido: 1..M en el orden del documento, una por prueba marcada", () => {
    const paradas = [...GUIA.matchAll(/⭐⭐ Parada (\d+) de (\d+)/g)];
    const total = PRUEBAS.filter((p) => p[1].includes("data-corto")).length;
    expect(paradas.map((p) => Number(p[1]))).toEqual(
      Array.from({ length: total }, (_, i) => i + 1),
    );
    expect(new Set(paradas.map((p) => p[2]))).toEqual(new Set([String(total)]));
  });

  it("ningún id se repite y todo origen es uno de los tres", () => {
    const ids = PRUEBAS.map((p) => p[3]);
    expect(ids.filter((id, i) => ids.indexOf(id) !== i)).toEqual([]);
    expect(
      PRUEBAS.filter(
        (p) => !["nuevo", "mejorado", "heredada"].includes(p[2]),
      ).map((p) => p[3]),
    ).toEqual([]);
  });

  it("el ⭐⭐ cabe en su techo: preparación y paradas suman lo que dice la cabecera, y no más de 20 min", () => {
    const corto = PRUEBAS.filter((p) => p[1].includes("data-corto"));
    const sinMinutos = corto.filter((p) => !/data-min="[\d.]+"/.test(p[1])).map((p) => p[3]);
    expect(sinMinutos, "paradas sin data-min").toEqual([]);
    const preparacion = /id="preparacion" data-min="([\d.]+)"/.exec(GUIA);
    expect(preparacion, "la preparación no dice sus minutos").not.toBeNull();
    const suma =
      corto.reduce((n, p) => n + Number(/data-min="([\d.]+)"/.exec(p[1])![1]), 0) +
      Number(preparacion![1]);
    const cabecera = /Gate corto ⭐⭐: \d+ paradas · ~([\d,]+) min/.exec(GUIA);
    expect(cabecera, "la cabecera no dice los minutos del ⭐⭐").not.toBeNull();
    expect(suma).toBe(minutos(cabecera![1]));
    expect(suma, "el ⭐⭐ pasa del techo de ~20 min").toBeLessThanOrEqual(20);
  });

  it("cada parada dice los minutos de su atributo", () => {
    const paradas = [
      ...GUIA.matchAll(
        /<li ([^>]*data-min="([\d.]+)"[^>]*)>[\s\S]*?⭐⭐ Parada \d+ de \d+ · ~([\d,]+) min/g,
      ),
    ];
    expect(paradas.length).toBe(PRUEBAS.filter((p) => p[1].includes("data-corto")).length);
    expect(
      paradas.filter((p) => Number(p[2]) !== minutos(p[3])).map((p) => p[1]),
    ).toEqual([]);
  });

  it("el desglose del ⭐ por sprint cuadra con de dónde nace cada prueba", () => {
    const sinNacer = PRUEBAS.filter((p) => !/data-nace="s\d"/.test(p[1])).map((p) => p[3]);
    expect(sinNacer, "pruebas sin data-nace").toEqual([]);
    const m = /Gate mínimo ⭐: (\d+) pruebas · ~\d+ min \(S1 (\d+) · S2 (\d+) · S3 (\d+)\)/.exec(GUIA);
    expect(m, "la cabecera no trae el desglose S1 · S2 · S3").not.toBeNull();
    const minimo = PRUEBAS.filter((p) => p[1].includes("data-minimo"));
    const de = (s: string) => minimo.filter((p) => p[1].includes(`data-nace="${s}"`)).length;
    expect([de("s1"), de("s2"), de("s3")]).toEqual([Number(m![2]), Number(m![3]), Number(m![4])]);
    expect(de("s1") + de("s2") + de("s3")).toBe(Number(m![1]));
  });

  it("el bloque de textos diferidos declara sus filas, y ningún id se repite con las pruebas", () => {
    expect(declarada(/Textos diferidos: (\d+) filas/)).toBe(FILAS.length);
    expect(FILAS.length, "sin filas, el bloque de textos no es un bloque").toBeGreaterThan(0);
    const ids = [...PRUEBAS.map((p) => p[3]), ...FILAS.map((f) => f[1])];
    expect(ids.filter((id, i) => ids.indexOf(id) !== i)).toEqual([]);
  });

  it("el chip de cada prueba dice su origen, con el sprint de la guía y su namespace", () => {
    expect(Number.isInteger(SPRINT) && SPRINT > 0, "la cabecera no dice su sprint").toBe(true);
    expect(GUIA).toContain(`var NS = "ag-s${SPRINT}-";`);
    const chip = (i: RegExpMatchArray) => {
      const desde = i.index! + i[0].length;
      return /<span class="origen origen-(\w+)">([^<]*)<\/span>/.exec(GUIA.slice(desde))!;
    };
    const mal = PRUEBAS.filter((p) => {
      const [, clase, texto] = chip(p);
      if (clase !== p[2]) return true;
      if (p[2] === "nuevo") return !texto.startsWith(`Nuevo · S${SPRINT}`);
      if (p[2] === "mejorado") return !texto.startsWith(`Mejorado en S${SPRINT}`);
      return /Nuevo|Mejorado/.test(texto);
    }).map((p) => p[3]);
    expect(mal, "chips que no dicen su origen").toEqual([]);
  });
});
