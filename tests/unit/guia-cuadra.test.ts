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
 */
const GUIA = readFileSync("docs/GUIA-DE-PRUEBA.html", "utf8");
const PRUEBAS = [
  ...GUIA.matchAll(
    /<li (data-origen="(\w+)"[^>]*)><input type="checkbox" id="(\w+)">/g,
  ),
];

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
});
