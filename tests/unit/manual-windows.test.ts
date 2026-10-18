import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { PLATAFORMA_WINDOWS } from "../../src/contrato.generado";

/**
 * **EL MANUAL DE WINDOWS ES COMPLETO, HABLA SOLO DE WINDOWS Y DICE LO MISMO EN LOS DOS IDIOMAS** (sprint 010).
 *
 * `docs/MANUAL-WINDOWS.md` y `docs/MANUAL-WINDOWS-EN.md` son los manuales de quien usa la app en un PC con Windows y nunca
 * vio otro sistema. Este gate exige cuatro cosas:
 *
 *  1. **Cero menciones a otro sistema**: ni el Mac, ni su llavero, ni sus teclas, ni sus avisos.
 *  2. **Los dos idiomas tienen la misma forma**: los mismos encabezados y las mismas filas en cada tabla.
 *  3. **Los atajos son los del contrato** que genera Rust: la tabla del manual no puede decir una tecla que la app no
 *     registra, ni callar una que sí.
 *  4. **La tabla de las 30 funcionalidades de la visión** tiene sus 30 filas, cada una con la clave de «Hoy en Windows» que
 *     la cubre (o «sin clave») y un estado en tres palabras —hecho · hecho sin verificar · fuera—, **atada a
 *     `EN_WINDOWS`** (las 15 claves de `plataforma.rs`): una clave que no existe, una clave sin fila o una función «no por
 *     ahora» que se declara hecha, es rojo. Si la planeadora (solo lectura) está en este equipo, también se comprueba que la
 *     visión tiene esas mismas 30.
 *
 * ¿Puede fallar? Sí: nombrar el Mac en una sola línea, quitar la fila 17, declarar «hecho» la puerta local o inventar una
 * clave son rojo (demos del sprint 010).
 */
const ES = readFileSync("docs/MANUAL-WINDOWS.md", "utf8");
const EN = readFileSync("docs/MANUAL-WINDOWS-EN.md", "utf8");

/** Lo que no puede aparecer: el otro sistema de escritorio de la app, con su llavero, sus teclas y sus avisos. */
const OTRO_SISTEMA =
  /macOS|\bMac\b|\bMacs\b|Llavero|Keychain|Touch ID|Finder|launchd|Time Machine|Spotlight|AirPlay|iCloud|\bApple\b|iPhone|⌃|⌥|⎋|⌘/;

const ESTADOS: Record<string, "hecho" | "sin-verificar" | "fuera"> = {
  hecho: "hecho",
  done: "hecho",
  "hecho sin verificar": "sin-verificar",
  "done, unverified": "sin-verificar",
  fuera: "fuera",
  out: "fuera",
};

type Fila = {
  n: number;
  funcionalidad: string;
  claves: string[];
  estado: "hecho" | "sin-verificar" | "fuera";
};

/** Las filas de la tabla de las 30: `| N | Funcionalidad | Clave | Estado | Qué falta |`. */
function filasDe30(texto: string): Fila[] {
  const filas: Fila[] = [];
  for (const linea of texto.split("\n")) {
    const celdas = linea.split("|").map((c) => c.trim());
    if (celdas.length < 7 || !/^\d+$/.test(celdas[1])) continue;
    const estado = ESTADOS[celdas[4]];
    if (!estado)
      throw new Error(
        `fila ${celdas[1]}: el estado «${celdas[4]}» no es uno de los tres`,
      );
    const claves =
      celdas[3] === "sin clave" || celdas[3] === "no key"
        ? []
        : [...celdas[3].matchAll(/`([a-z-]+)`/g)].map((m) => m[1]);
    filas.push({
      n: Number(celdas[1]),
      funcionalidad: celdas[2],
      claves,
      estado,
    });
  }
  return filas;
}

const CLAVES = PLATAFORMA_WINDOWS.funciones.map((f) => f.clave as string);
const NO_POR_AHORA = PLATAFORMA_WINDOWS.funciones
  .filter((f) => f.cuando === "no-por-ahora")
  .map((f) => f.clave as string);

describe("el manual de Windows habla solo de Windows", () => {
  it.each([
    ["ES", ES],
    ["EN", EN],
  ])("%s: ninguna línea nombra otro sistema", (_, texto) => {
    const culpables = texto
      .split("\n")
      .map((l, i) => [i + 1, l] as const)
      .filter(([, l]) => OTRO_SISTEMA.test(l))
      .map(([i, l]) => `línea ${i}: ${l.slice(0, 110)}`);
    expect(culpables, "el manual de Windows nombra otro sistema").toEqual([]);
  });

  it("es un manual completo, no un capítulo: tiene las secciones de uso de punta a punta", () => {
    for (const texto of [ES, EN]) {
      const secciones = [...texto.matchAll(/^## (.+)$/gm)].map((m) => m[1]);
      expect(
        secciones.length,
        "pocas secciones para un manual entero",
      ).toBeGreaterThanOrEqual(15);
    }
  });
});

describe("el manual de Windows dice lo mismo en español y en inglés", () => {
  it("los mismos encabezados, en el mismo orden", () => {
    const cuenta = (t: string, nivel: string) =>
      [...t.matchAll(new RegExp(`^${nivel} `, "gm"))].length;
    expect(cuenta(EN, "##"), "encabezados ##").toBe(cuenta(ES, "##"));
    expect(cuenta(EN, "###"), "encabezados ###").toBe(cuenta(ES, "###"));
  });

  it("las mismas filas en cada tabla y las mismas viñetas", () => {
    const filas = (t: string) =>
      t.split("\n").filter((l) => /^\|/.test(l)).length;
    const viñetas = (t: string) =>
      t.split("\n").filter((l) => /^(- |\d+\. )/.test(l)).length;
    expect(filas(EN), "filas de tabla").toBe(filas(ES));
    expect(viñetas(EN), "viñetas y pasos").toBe(viñetas(ES));
  });

  it("los atajos de los dos manuales son los mismos", () => {
    const teclas = (t: string) =>
      [...t.matchAll(/`Alt\+(?:Mayús|Shift)\+([A-Z])`/g)].map((m) => m[1]);
    expect(new Set(teclas(EN))).toEqual(new Set(teclas(ES)));
  });
});

describe("los atajos del manual son los que la app registra en Windows", () => {
  const delContrato = PLATAFORMA_WINDOWS.atajos.map((a) => a.teclas);
  const letras = new Set(
    delContrato
      .filter((t) => t.startsWith("Alt+Shift+"))
      .map((t) => t.slice(-1)),
  );

  /** La tabla de atajos de un manual: desde su cabecera hasta el siguiente encabezado. */
  const tablaDe = (texto: string, cabecera: RegExp, siguiente: string) =>
    texto.slice(texto.search(cabecera), texto.indexOf(siguiente));

  it.each([
    [
      "EN",
      EN,
      /\|\s*Key\s*\|\s*What it does\s*\|/,
      "## Your corpus",
      /`Alt\+Shift\+([A-Z])`/g,
    ],
    [
      "ES",
      ES,
      /\|\s*Tecla\s*\|\s*Qué hace\s*\|/,
      "## Tu corpus",
      /`Alt\+Mayús\+([A-Z])`/g,
    ],
  ])(
    "%s: la tabla de atajos dice exactamente las teclas del contrato",
    (_, texto, cabecera, siguiente, patron) => {
      const tabla = tablaDe(texto, cabecera, siguiente);
      const teclas = [...tabla.matchAll(patron)].map((m) => m[1]);
      expect(
        new Set(teclas),
        "las teclas del manual y las del contrato",
      ).toEqual(letras);
      expect(teclas.length, "una tecla repetida o de más").toBe(letras.size);
      // Esc: el contrato la trae y la tabla la dice (solo existe con el modo solo audio encendido).
      expect(delContrato, "el contrato ya no trae Esc").toContain("Esc");
      expect(tabla, "la tabla no dice Esc").toMatch(/`Esc`/);
    },
  );
});

describe("las 30 funcionalidades de la visión, en Windows", () => {
  const es = filasDe30(ES);
  const en = filasDe30(EN);

  it("hay 30 filas, numeradas de 1 a 30, en los dos idiomas", () => {
    for (const filas of [es, en]) {
      expect(filas.map((f) => f.n)).toEqual(
        Array.from({ length: 30 }, (_, i) => i + 1),
      );
    }
  });

  it("español e inglés dicen la misma clave y el mismo estado en cada fila", () => {
    for (let i = 0; i < 30; i++) {
      expect(en[i].claves, `fila ${i + 1}: claves`).toEqual(es[i].claves);
      expect(en[i].estado, `fila ${i + 1}: estado`).toBe(es[i].estado);
    }
  });

  it("cada clave existe en `EN_WINDOWS` (plataforma.rs) y cada clave de `EN_WINDOWS` tiene su fila", () => {
    // Las dos tablas, no solo la española: una clave inventada en inglés tiene que ser roja por su propia regla.
    const usadas = new Set([...es, ...en].flatMap((f) => f.claves));
    const inventadas = [...usadas].filter((c) => !CLAVES.includes(c));
    const sinFila = CLAVES.filter((c) => !usadas.has(c));
    expect(inventadas, "claves que `plataforma.rs` no tiene").toEqual([]);
    expect(sinFila, "claves de `plataforma.rs` sin ninguna fila").toEqual([]);
    expect(
      CLAVES.length,
      "el contrato ya no trae las 15 de «Hoy en Windows»",
    ).toBe(15);
  });

  it("el estado es coherente con lo que `plataforma.rs` dice: lo «no por ahora» está fuera, y solo eso lo cubre una clave", () => {
    for (const f of es) {
      const sonNoPorAhora =
        f.claves.length > 0 && f.claves.every((c) => NO_POR_AHORA.includes(c));
      if (f.claves.length > 0) {
        expect(
          f.estado === "fuera",
          `fila ${f.n} («${f.funcionalidad}»): una clave «sí» no puede estar «fuera», y una «no por ahora» no puede estar hecha`,
        ).toBe(sonNoPorAhora);
      }
    }
  });

  it("lo «fuera» sin clave dice su motivo y lo «hecho sin verificar» dice qué falta", () => {
    const motivo = (texto: string) =>
      texto
        .split("\n")
        .filter((l) => /^\| \d+ \|/.test(l))
        .map((l) => l.split("|")[5]?.trim() ?? "");
    for (const texto of [ES, EN]) {
      for (const m of motivo(texto))
        expect(m.length, `motivo vacío o muy corto: «${m}»`).toBeGreaterThan(
          15,
        );
    }
  });

  const VISION = [
    "C:/Code/hr01-develop-ai-apps/portafolio/copiloto-consultor/VISION.md",
    `${process.env.HOME ?? ""}/Code/hr01-develop-ai-apps/portafolio/copiloto-consultor/VISION.md`,
  ].find(existsSync);
  it.skipIf(!VISION)(
    "la visión de la planeadora (solo lectura; solo corre donde está la planeadora, la CI no la tiene) tiene esas mismas 30 funcionalidades",
    () => {
      const texto = readFileSync(VISION as string, "utf8");
      const desde = texto.indexOf("## Lo que podrás hacer");
      const hasta = texto.indexOf("## Qué la hace distinta");
      const viñetas = texto
        .slice(desde, hasta)
        .split("\n")
        .filter((l) => /^- \*\*/.test(l));
      expect(
        viñetas.length,
        "la visión ya no tiene 30 funcionalidades: la tabla del manual hay que revisarla",
      ).toBe(30);
    },
  );
});
