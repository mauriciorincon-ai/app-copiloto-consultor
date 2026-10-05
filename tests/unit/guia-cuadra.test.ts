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
 *
 * **Desde el sprint 004** la guía lleva dos ciclos: el ⭐⭐ del H1 sigue entero para su Acto 2, y las ⭐ que
 * nacen en el H2 forman su acumulado. Cada una de esas **declara su candidatura al ⭐⭐ del H2** (sí o no, y
 * por qué): sin la línea, el ⭐⭐ del cierre del ciclo se arma a ciegas. Y las formas y textos del H2 que nadie
 * ha visto van en su propia tabla, contada aparte de la del H1. Rojos en la bitácora del sprint 004, fase 5.
 */
const GUIA = readFileSync("docs/GUIA-DE-PRUEBA.html", "utf8");
const PRUEBAS = [
  ...GUIA.matchAll(
    /<li (data-origen="(\w+)"[^>]*)><input type="checkbox" id="(\w+)">/g,
  ),
];

const FILAS = [...GUIA.matchAll(/<tr data-texto><td><input type="checkbox" id="(\w+)"/g)];
const FILAS_H2 = [
  ...GUIA.matchAll(/<tr data-texto data-ciclo="h2"><td><input type="checkbox" id="(\w+)"/g),
];

/** Las ⭐ del ciclo H2: las que nacen del sprint 4 al 6. */
const DEL_H2 = PRUEBAS.filter(
  (p) => /data-nace="s[4-6]"/.test(p[1]) && p[1].includes("data-minimo"),
);

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

  // Las cifras de la cabecera sobre el origen (auditoría del S3, B25; casilla 6 a): «29 pruebas nuevas» y
  // «33 de las 72 heredadas reescritas» eran ciertas, pero nadie las contaba. Rojo cambiando una cifra.
  it("la cabecera cuenta bien las nuevas y las heredadas reescritas", () => {
    const de = (origen: string) => PRUEBAS.filter((p) => p[2] === origen).length;
    expect(declarada(/(\d+) pruebas nuevas/)).toBe(de("nuevo"));
    const m = /(\d+) de las (\d+) heredadas reescritas/.exec(GUIA);
    expect(m, "la cabecera no dice cuántas heredadas se reescribieron").not.toBeNull();
    expect(Number(m![1])).toBe(de("mejorado"));
    expect(Number(m![2])).toBe(de("mejorado") + de("heredada"));
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
    const m =
      /Gate mínimo ⭐: (\d+) pruebas · ~\d+ min \(S1 (\d+) · S2 (\d+) · S3 (\d+) · S4 (\d+)\)/.exec(GUIA);
    expect(m, "la cabecera no trae el desglose S1 · S2 · S3 · S4").not.toBeNull();
    const minimo = PRUEBAS.filter((p) => p[1].includes("data-minimo"));
    const de = (s: string) => minimo.filter((p) => p[1].includes(`data-nace="${s}"`)).length;
    expect([de("s1"), de("s2"), de("s3"), de("s4")]).toEqual(
      [m![2], m![3], m![4], m![5]].map(Number),
    );
    expect(de("s1") + de("s2") + de("s3") + de("s4")).toBe(Number(m![1]));
  });

  it("el bloque de textos diferidos declara sus filas, y ningún id se repite con las pruebas", () => {
    expect(declarada(/Textos diferidos: (\d+) filas/)).toBe(FILAS.length);
    expect(FILAS.length, "sin filas, el bloque de textos no es un bloque").toBeGreaterThan(0);
    expect(declarada(/Formas y textos del H2: (\d+) filas/)).toBe(FILAS_H2.length);
    const ids = [...PRUEBAS.map((p) => p[3]), ...FILAS.map((f) => f[1]), ...FILAS_H2.map((f) => f[1])];
    expect(ids.filter((id, i) => ids.indexOf(id) !== i)).toEqual([]);
  });

  // Sprint 004: el ⭐⭐ del H2 se arma al cierre del ciclo con las candidatas que cada ⭐ declara al nacer.
  // ¿Puede fallar? Sí: una ⭐ nueva sin su línea, una línea que dice «sí» con el atributo en «no», o la
  // cabecera con la cuenta vieja. Rojo en la bitácora del sprint 004, fase 5.
  it("cada ⭐ del H2 declara su candidatura al ⭐⭐ del H2, con su porqué, y la cabecera las cuenta", () => {
    expect(DEL_H2.length, "el ciclo H2 ya tiene ⭐").toBeGreaterThan(0);
    const linea = (p: RegExpMatchArray) => {
      const resto = GUIA.slice(p.index! + p[0].length);
      const li = resto.slice(0, resto.indexOf("</li>"));
      return /<p class="candidata">Candidata al ⭐⭐ del H2: <strong>(sí|no)<\/strong>\. ([^<]{30,})<\/p>/.exec(li);
    };
    const mal = DEL_H2.filter((p) => {
      const atributo = /data-candidata="(si|no)"/.exec(p[1])?.[1];
      const l = linea(p);
      return !atributo || !l || (l[1] === "sí" ? "si" : "no") !== atributo;
    }).map((p) => p[3]);
    expect(mal, "⭐ del H2 sin candidatura, o con la línea y el atributo en desacuerdo").toEqual([]);
    // Solo una ⭐ es candidata: el ⭐⭐ es una lente sobre el mínimo.
    const sueltas = PRUEBAS.filter(
      (p) => p[1].includes("data-candidata") && !p[1].includes("data-minimo"),
    ).map((p) => p[3]);
    expect(sueltas, "candidata sin ser ⭐").toEqual([]);
    const m = /Acumulado del ciclo H2: (\d+) ⭐ \(S4 (\d+)\) · candidatas al ⭐⭐ del H2: (\d+) sí · (\d+) no/.exec(
      GUIA,
    );
    expect(m, "la cabecera no dice el acumulado del H2 y sus candidatas").not.toBeNull();
    const si = DEL_H2.filter((p) => p[1].includes('data-candidata="si"')).length;
    expect([m![1], m![2], m![3], m![4]].map(Number)).toEqual([
      DEL_H2.length,
      DEL_H2.filter((p) => p[1].includes('data-nace="s4"')).length,
      si,
      DEL_H2.length - si,
    ]);
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
