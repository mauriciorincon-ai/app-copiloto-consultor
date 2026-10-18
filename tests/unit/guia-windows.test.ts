import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { TEXTOS_DE_WINDOWS } from "../../src/i18n/windows";

/**
 * **LA GUÍA DE PRUEBA DE WINDOWS ES AUTOCONTENIDA, HABLA SOLO DE WINDOWS Y SUS 9 PARADAS SON LAS DEL GATE** (sprint 010).
 *
 * `docs/GUIA-DE-PRUEBA-WINDOWS.html` es la guía de quien prueba la app en un PC con Windows y nada más. Se escribe a mano y
 * no se genera de la guía general; por eso su **fuente única está declarada y atada**: las 9 paradas del gate son las
 * `data-gate="pc"` de `docs/GUIA-DE-PRUEBA.html`, con el MISMO id y los MISMOS minutos, y las 9 filas de la matriz de la
 * regla 24 son las del PC de la general, palabra por palabra. Si una cambia en un sitio, el gate lo dice.
 *
 * Lo que exige:
 *  1. **Cero menciones a otro sistema**, en ninguna prueba, matriz ni nota.
 *  2. **Autocontenida**: ni una URL, ni un `<link>`, ni un `<script src>`, ni `fetch`.
 *  3. **Prefijo propio de `localStorage`** (`ag-w11-`), distinto del de la guía general.
 *  4. **9 paradas del gate, ~20 min**, numeradas «parada N de 9» en el orden del documento.
 *  5. **Cada mirada de texto en matriz**: abre · qué mirar · qué respuesta espero.
 *  6. **La guía general la enlaza**, y las dos se encuentran.
 *
 * ¿Puede fallar? Sí: nombrar el otro sistema en una prueba, cambiar los minutos de una parada o quitar el enlace de la guía
 * general es rojo (demos del sprint 010).
 */
const WIN = readFileSync("docs/GUIA-DE-PRUEBA-WINDOWS.html", "utf8");
const GENERAL = readFileSync("docs/GUIA-DE-PRUEBA.html", "utf8");

const OTRO_SISTEMA =
  /macOS|\bMac\b|\bMacs\b|Llavero|Keychain|Touch ID|Finder|launchd|Time Machine|Spotlight|AirPlay|iCloud|\bApple\b|iPhone|AirPods|⌃|⌥|⎋|⌘/;

/** Lo que una persona lee: el texto, sin etiquetas ni el script. */
function texto(html: string): string {
  return html
    .replace(/<!--[\s\S]*?-->/g, " ")
    .replace(/<script[\s\S]*?<\/script>/g, " ")
    .replace(/<style[\s\S]*?<\/style>/g, " ")
    .replace(/<[^>]+>/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/\s+/g, " ")
    .trim();
}

/** Las paradas del gate de un archivo: su id y sus minutos, en el orden del documento. */
function paradasDelGate(html: string): { id: string; min: number }[] {
  return [
    ...html.matchAll(
      /<li[^>]*data-gate="pc"[^>]*data-min-gate="([\d.]+)"[^>]*><input type="checkbox" id="([a-z0-9]+)"/g,
    ),
  ].map((m) => ({ id: m[2], min: Number(m[1]) }));
}

describe("la guía de Windows habla solo de Windows", () => {
  it("ni la guía ni sus comentarios nombran otro sistema", () => {
    const culpables = WIN.split("\n")
      .map((l, i) => [i + 1, l] as const)
      .filter(([, l]) => OTRO_SISTEMA.test(l))
      .map(([i, l]) => `línea ${i}: ${l.trim().slice(0, 110)}`);
    expect(culpables, "la guía de Windows nombra otro sistema").toEqual([]);
  });

  it("CONTROL: la guía general sí lo nombra (el test no pasa de vacío)", () => {
    expect(OTRO_SISTEMA.test(GENERAL)).toBe(true);
  });
});

describe("la guía de Windows es autocontenida", () => {
  it("cero URLs, enlaces a hojas o scripts externos, ni fetch", () => {
    expect(WIN).not.toMatch(/https?:\/\//);
    expect(WIN).not.toMatch(/<link\b/i);
    expect(WIN).not.toMatch(/<script[^>]+\bsrc=/i);
    expect(WIN).not.toMatch(/\bfetch\s*\(|XMLHttpRequest|\bimport\s*\(/);
  });

  it("su prefijo de localStorage es propio y no el de la guía general", () => {
    const ns = (html: string) => /var NS = "([^"]+)"/.exec(html)?.[1];
    expect(ns(WIN)).toBe("ag-w11-");
    expect(ns(GENERAL)).not.toBe(ns(WIN));
  });

  it("cada casilla tiene su etiqueta o su nombre accesible, y ningún id se repite", () => {
    const ids = [...WIN.matchAll(/<input type="checkbox" id="([^"]+)"/g)].map(
      (m) => m[1],
    );
    expect(new Set(ids).size, "ids repetidos").toBe(ids.length);
    for (const id of ids) {
      const conEtiqueta =
        new RegExp(`<label for="${id}"`).test(WIN) ||
        new RegExp(`id="${id}" aria-label=`).test(WIN);
      expect(conEtiqueta, `la casilla ${id} no tiene etiqueta`).toBe(true);
    }
  });
});

describe("las 9 paradas del gate de Windows son las de la guía general", () => {
  const win = paradasDelGate(WIN);
  const general = paradasDelGate(GENERAL);

  it("son 9, con los mismos ids y los mismos minutos, en el mismo orden", () => {
    expect(win.length, "paradas del gate de Windows").toBe(9);
    expect(win).toEqual(general);
  });

  it("suman 18 min y, con los 2 de preparación, caben en ~20", () => {
    const suma = win.reduce((a, p) => a + p.min, 0);
    const preparacion = Number(
      /id="preparacion" data-min-gate="([\d.]+)"/.exec(WIN)?.[1],
    );
    expect(
      preparacion,
      "la preparación no declara sus minutos",
    ).toBeGreaterThan(0);
    expect(suma + preparacion, "el gate pasa de 20 min").toBeLessThanOrEqual(
      20,
    );
    expect(texto(WIN)).toMatch(/9 paradas · ~20 min/);
  });

  it("cada una dice «parada N de 9», en orden", () => {
    const etiquetas = [...WIN.matchAll(/parada (\d+) de 9/g)].map((m) =>
      Number(m[1]),
    );
    expect(etiquetas).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9]);
  });

  it("los ids del gate aparecen en el orden del documento", () => {
    const orden = win.map((p) => WIN.indexOf(`id="${p.id}"`));
    expect(orden).toEqual([...orden].sort((a, b) => a - b));
  });
});

describe("la matriz de la regla 24 es la del PC de la guía general", () => {
  /** Las celdas de las filas de la matriz, sin etiquetas. */
  const filas = (html: string, soloPc: boolean): string[][] => {
    const tabla = html.slice(html.indexOf('id="matriz-gate"'));
    const cuerpo = tabla.slice(
      tabla.indexOf("<tbody>"),
      tabla.indexOf("</tbody>"),
    );
    return [...cuerpo.matchAll(/<tr>([\s\S]*?)<\/tr>/g)]
      .map((m) =>
        [...m[1].matchAll(/<td>([\s\S]*?)<\/td>/g)].map((c) => texto(c[1])),
      )
      .filter((celdas) => !soloPc || celdas[1] === "PC")
      .map((celdas) => (soloPc ? celdas.slice(2) : celdas.slice(1)));
  };

  it("sus 9 filas dicen lo mismo, palabra por palabra, que las filas «PC» de la general", () => {
    const win = filas(WIN, false);
    const general = filas(GENERAL, true);
    expect(win.length, "filas de la matriz de Windows").toBe(9);
    expect(general.length, "filas del PC en la guía general").toBe(9);
    expect(win).toEqual(general);
  });

  it("pide el «sí» por acción antes de correr cada fila", () => {
    expect(texto(WIN)).toMatch(/tu «sí» por acción/);
  });
});

describe("cada mirada de texto va en una matriz de una fila", () => {
  const bloque = WIN.slice(WIN.indexOf("data-textos"));

  it("la tabla de textos tiene las columnas abre · qué mirar · qué respuesta espero", () => {
    for (const columna of [
      "Abre",
      "Qué mirar exactamente",
      "Qué respuesta espero",
    ]) {
      expect(bloque, columna).toContain(`<th scope="col">${columna}</th>`);
    }
  });

  it("cada fila trae sus tres celdas con texto, y hay al menos 8", () => {
    const filas = [...bloque.matchAll(/<tr data-texto>([\s\S]*?)<\/tr>/g)];
    expect(filas.length).toBeGreaterThanOrEqual(8);
    for (const [, fila] of filas) {
      const celdas = [...fila.matchAll(/<td>([\s\S]*?)<\/td>/g)].map((c) =>
        texto(c[1]),
      );
      expect(celdas.length, "celdas de la fila").toBe(4);
      // «Abre» puede ser corta («Language»); «qué mirar» y «qué respuesta espero» tienen que decir algo.
      expect(celdas[1].length, "la celda «abre» está vacía").toBeGreaterThan(3);
      for (const c of celdas.slice(2))
        expect(c.length, `celda vacía o muy corta: «${c}»`).toBeGreaterThan(10);
    }
  });
});

describe("lo que la guía manda correr se puede correr (auditoría del sprint 010, A2)", () => {
  // `AG_BANDEJA_DE_PRUEBA` solo la lee la compilación de desarrollo (`reunion.rs`, `#[cfg(debug_assertions)]`): contra el
  // instalador no hace nada y la bandeja dura 3 h. La guía lo mandaba sobre el instalador.
  it.each([
    ["docs/GUIA-DE-PRUEBA-WINDOWS.html", WIN],
    ["docs/GUIA-DE-PRUEBA.html", GENERAL],
  ])(
    "%s: cada AG_BANDEJA_DE_PRUEBA en Windows va con `pnpm tauri dev`",
    (_f, html) => {
      const t = texto(html);
      const sitios = [...t.matchAll(/\$env:AG_BANDEJA_DE_PRUEBA='1'/g)];
      expect(
        sitios.length,
        "no hay ninguna mención de PowerShell",
      ).toBeGreaterThan(0);
      for (const s of sitios)
        expect(
          t.slice(s.index!, s.index! + 60),
          "la variable de PowerShell va seguida de `pnpm tauri dev`",
        ).toContain("pnpm tauri dev");
    },
  );

  it("la guía de Windows ya no la manda sin el código (forma sin `$env:`)", () => {
    expect(texto(WIN)).not.toMatch(/AG_BANDEJA_DE_PRUEBA=1/);
  });
});

describe("la guía general y la de Windows se enlazan", () => {
  it("la guía general apunta a la de Windows", () => {
    expect(GENERAL).toContain('href="GUIA-DE-PRUEBA-WINDOWS.html"');
  });

  it("el manual de Windows nombra esta guía como su gate", () => {
    for (const f of ["docs/MANUAL-WINDOWS.md", "docs/MANUAL-WINDOWS-EN.md"]) {
      expect(readFileSync(f, "utf8"), f).toContain(
        "docs/GUIA-DE-PRUEBA-WINDOWS.html",
      );
    }
  });
});

describe("la banda a tu altura (sprint 011): sus paradas dicen los rótulos que la app pinta", () => {
  // La Surface está en inglés: la guía manda buscar rótulos exactos. Si `windows.ts` cambia uno y la guía no, quien
  // prueba busca un botón que no existe. Los rótulos se leen del diccionario, no se copian aquí.
  const en = TEXTOS_DE_WINDOWS.en.banda;
  const t = texto(WIN);

  it.each([
    ["asaDeAltura", en.asaDeAltura],
    ["cerrar", en.cerrar],
    ["cerrarYTerminar", en.cerrarYTerminar],
    ["alturaTitulo", en.alturaTitulo],
    ["pegadaArriba", en.pegadaArriba],
    ["volverArriba", en.volverArriba],
    ["bajar", en.bajar],
    ["subir", en.subir],
    ["cerradaTitulo", en.cerradaTitulo],
    ["mostrar", en.mostrar],
    ["alturaValor", en.alturaValor(123).replace("123", "N")],
  ])("la guía nombra «%s» tal cual: «%s»", (_clave, rotulo) => {
    expect(t, `la guía no nombra «${rotulo}»`).toContain(rotulo);
  });

  it("w20 y w21 existen, son ⭐ nuevas del S11 y NO están entre las 9 del gate (la guía general no se toca)", () => {
    for (const id of ["w20", "w21"]) {
      const li = new RegExp(
        `<li data-origen="nuevo" data-nace="s11" data-minimo><input type="checkbox" id="${id}">`,
      );
      expect(WIN, `${id} no es una parada ⭐ nueva del S11`).toMatch(li);
      expect(
        paradasDelGate(WIN).map((p) => p.id),
        `${id} entró al gate`,
      ).not.toContain(id);
    }
  });

  it("w16 es «Mejorado en S11» y dice la verdad: End session no devuelve la reunión acoplada", () => {
    expect(WIN).toMatch(
      /<li data-origen="mejorado" data-nace="s11"[^>]*data-gate="pc"[^>]*><input type="checkbox" id="w16">/,
    );
    expect(t).toMatch(/End session no la devuelve/);
    expect(t).not.toMatch(/vuelve a su sitio al terminar/);
  });

  it("el filtro «Lo que cambió» cuenta desde el S10 y llega al S11", () => {
    expect(WIN).toContain("Lo que cambió desde el S10");
    expect(WIN).toMatch(/n === "s10" \|\| n === "s11"/);
  });
});
