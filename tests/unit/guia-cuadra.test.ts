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

/** De qué sprint nace una prueba, de su `data-nace`: «s7» → 7. Los sprints salen de los datos de la guía (auditoría del S7, B10). */
const NACE = (p: RegExpMatchArray): number => Number(/data-nace="s(\d+)"/.exec(p[1])?.[1]);

/** Las ⭐ del ciclo H2: las que nacen del sprint 4 en adelante. */
const DEL_H2 = PRUEBAS.filter((p) => NACE(p) >= 4 && p[1].includes("data-minimo"));

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
    const sinNacer = PRUEBAS.filter((p) => !/data-nace="s\d+"/.test(p[1])).map((p) => p[3]);
    expect(sinNacer, "pruebas sin data-nace").toEqual([]);
    // El desglose se lee de la cabecera y se cruza con los sprints que hay en los datos: ni la lista de sprints ni su
    // número están escritos aquí (auditoría del S7, B10: `s[4-7]` y «S1 … S7» se rompían con cada sprint nuevo).
    const m = /Gate mínimo ⭐: (\d+) pruebas · ~\d+ min \(((?:S\d+ \d+(?: · )?)+)\)/.exec(GUIA);
    expect(m, "la cabecera no trae el desglose «S1 n · S2 n · …»").not.toBeNull();
    const declaradoPorSprint = new Map(
      [...m![2].matchAll(/S(\d+) (\d+)/g)].map((x) => [Number(x[1]), Number(x[2])] as const),
    );
    const minimo = PRUEBAS.filter((p) => p[1].includes("data-minimo"));
    const delosDatos = new Map<number, number>();
    for (const p of PRUEBAS) delosDatos.set(NACE(p), delosDatos.get(NACE(p)) ?? 0); // un sprint sin ⭐ queda en 0, visible
    for (const p of minimo) delosDatos.set(NACE(p), (delosDatos.get(NACE(p)) ?? 0) + 1);
    // S5 no tiene bloque de pruebas: el sprint 005 saltó la directiva del 2026-10-06 y la guía lo declara en
    // su cabecera de versión. Que la cabecera lo diga en cero es lo que lo hace visible, no un olvido.
    const desglose = (mapa: Map<number, number>) => [...mapa].sort((a, b) => a[0] - b[0]);
    for (const [sprint, cuantas] of delosDatos) {
      expect(declaradoPorSprint.get(sprint) ?? 0, `S${sprint}`).toBe(cuantas);
    }
    for (const [sprint] of declaradoPorSprint) {
      expect(delosDatos.has(sprint) || declaradoPorSprint.get(sprint) === 0, `S${sprint} figura en la cabecera y no en los datos`).toBe(true);
    }
    expect(desglose(delosDatos).reduce((n, [, c]) => n + c, 0)).toBe(Number(m![1]));
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
    const m = /Acumulado del ciclo H2: (\d+) ⭐ \(((?:S\d+ \d+(?: · )?)+)\) · candidatas al ⭐⭐ del H2: (\d+) sí · (\d+) no/.exec(
      GUIA,
    );
    expect(m, "la cabecera no dice el acumulado del H2 y sus candidatas").not.toBeNull();
    const si = DEL_H2.filter((p) => p[1].includes('data-candidata="si"')).length;
    // Los sprints del desglose salen de los datos, no de una lista escrita aquí (auditoría del S7, B10).
    const declarado = new Map(
      [...m![2].matchAll(/S(\d+) (\d+)/g)].map((x) => [Number(x[1]), Number(x[2])] as const),
    );
    const real = new Map<number, number>();
    for (const p of DEL_H2) real.set(NACE(p), (real.get(NACE(p)) ?? 0) + 1);
    expect([...declarado].sort((a, b) => a[0] - b[0])).toEqual([...real].sort((a, b) => a[0] - b[0]));
    expect([m![1], m![3], m![4]].map(Number)).toEqual([DEL_H2.length, si, DEL_H2.length - si]);
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

  /**
   * **El gate del H-1: dos sentadas, una por máquina, cada una dentro de su techo** (sprint 008). Es una lente sobre las ⭐
   * que ya existen (`data-gate="mac|pc"` y `data-min-gate`), no una lista aparte: las paradas se caminan de arriba abajo, en el
   * orden del documento, y cada una dice «parada N de M · ~X min» con los minutos de su atributo. La preparación de cada
   * máquina cuenta. La cabecera declara las dos cuentas, y la matriz de la regla 24 está en la guía.
   *
   * ¿Puede fallar? Sí: pasar una parada de 4 min a 6 sube el Mac por encima de 20; repetir un número de parada, o cambiar
   * los minutos del atributo sin cambiar la etiqueta, también es rojo (demo en rojo del sprint 008).
   */
  describe("el gate del H-1 cuadra por máquina", () => {
    const TECHO_POR_MAQUINA = 20;
    const delGate = (maq: string) => PRUEBAS.filter((p) => p[1].includes(`data-gate="${maq}"`));
    const minutosDe = (p: RegExpMatchArray) => Number(/data-min-gate="([\d.]+)"/.exec(p[1])?.[1]);
    const preparacion = (maq: string) => Number(new RegExp(`id="preparacion-${maq}" data-min-gate="([\\d.]+)"`).exec(GUIA)?.[1]);
    const declarado = /Gate del H-1: Mac (\d+) paradas · ~([\d,]+) min · PC (\d+) paradas · ~([\d,]+) min/.exec(GUIA);

    it("la cabecera declara las dos mitades", () => {
      expect(declarado, "la cabecera no trae «Gate del H-1: Mac N paradas · ~X min · PC M paradas · ~Y min»").not.toBeNull();
    });

    it.each([["mac", 1, 2], ["pc", 3, 4]] as const)("%s: sus paradas suman lo que dice la cabecera y no pasan de 20 min", (maq, iN, iMin) => {
      const paradas = delGate(maq);
      expect(paradas.length, `sin paradas de ${maq}`).toBeGreaterThan(0);
      const sinMinutos = paradas.filter((p) => !Number.isFinite(minutosDe(p))).map((p) => p[3]);
      expect(sinMinutos, "paradas sin data-min-gate").toEqual([]);
      const prep = preparacion(maq);
      expect(Number.isFinite(prep), `la preparación de ${maq} no dice sus minutos`).toBe(true);
      const suma = paradas.reduce((n, p) => n + minutosDe(p), 0) + prep;
      expect(Number(declarado![iN]), "el número de paradas de la cabecera").toBe(paradas.length);
      expect(suma, "la suma de las paradas más la preparación").toBe(minutos(declarado![iMin]));
      expect(suma, `el gate de ${maq} pasa de ${TECHO_POR_MAQUINA} min`).toBeLessThanOrEqual(TECHO_POR_MAQUINA);
    });

    it.each(["mac", "pc"])("%s: las paradas son un recorrido 1..M, cada una dice sus minutos, y son ⭐ que ya existían", (maq) => {
      const paradas = delGate(maq);
      const etiquetas = [...GUIA.matchAll(new RegExp(`Gate H-1 · ${maq === "mac" ? "Mac" : "PC"} · parada (\\d+) de (\\d+) · ~([\\d,]+) min`, "g"))];
      expect(etiquetas.map((e) => Number(e[1]))).toEqual(paradas.map((_, i) => i + 1));
      expect(new Set(etiquetas.map((e) => e[2]))).toEqual(new Set([String(paradas.length)]));
      // El orden de las etiquetas es el de las paradas del documento, y cada una lleva los minutos de su atributo.
      etiquetas.forEach((e, i) => expect(minutos(e[3]), `parada ${i + 1}`).toBe(minutosDe(paradas[i])));
      // Una lente sobre las ⭐: toda parada del gate es ⭐.
      expect(paradas.filter((p) => !p[1].includes("data-minimo")).map((p) => p[3])).toEqual([]);
    });

    it("las dos mitades no comparten una prueba, y cada parada es de una sola máquina", () => {
      const comunes = delGate("mac").filter((p) => delGate("pc").some((q) => q[3] === p[3]));
      expect(comunes.map((p) => p[3])).toEqual([]);
    });

    it("la matriz de la regla 24 está en la guía, con sus seis columnas y cubre las dos máquinas", () => {
      const m = /<table id="matriz-gate">([\s\S]*?)<\/table>/.exec(GUIA);
      expect(m, "la guía no trae la matriz de la regla 24 (id=\"matriz-gate\")").not.toBeNull();
      const encabezados = [...m![1].matchAll(/<th scope="col">([^<]*)<\/th>/g)].map((x) => x[1]);
      expect(encabezados).toEqual(["#", "Máquina", "Qué se toca", "Para qué", "Aviso que vas a ver", "Cómo se deshace"]);
      const filas = [...m![1].matchAll(/<tr><td>(\d+)<\/td><td>(Mac|PC)<\/td>/g)];
      expect(filas.length, "la matriz tiene pocas filas").toBeGreaterThanOrEqual(8);
      expect(new Set(filas.map((f) => f[2]))).toEqual(new Set(["Mac", "PC"]));
      expect(filas.map((f) => Number(f[1]))).toEqual(filas.map((_, i) => i + 1));
    });
  });
});
