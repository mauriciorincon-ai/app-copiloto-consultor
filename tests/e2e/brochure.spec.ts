import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { pathToFileURL } from "node:url";
import { resolve } from "node:path";

/**
 * EL BROCHURE, EN UN NAVEGADOR DE VERDAD (`docs/BROCHURE.html`, sprint 008). Lo que `tests/unit/brochure.test.ts` no puede ver:
 *
 *   · **Con `prefers-reduced-motion` la experiencia es COMPLETA** — el hero invisible del piloto de app-habla pasó doce e2e y
 *     Lighthouse 100/100 porque ningún gate miraba esa rama. Aquí se afirma visibilidad REAL (opacidad en toda la cadena, caja,
 *     y las rayas ya puestas).
 *   · **La apertura por lectura**: la tarjeta se abre al llegar a ella AUN SIN DETENERTE, el toque manda, y al cerrar por
 *     arriba lo que se lee no se mueve (trampa T7: se mide la deriva de un ancla visible, no se resta a ciegas).
 *   · **Lo cerrado, fuera del árbol de accesibilidad** (axe no lo ve): se lee el árbol real.
 *   · axe en los dos temas y los dos idiomas con todo abierto, y que nada desborde la página en ningún ancho.
 *   · Que abrir el archivo no pide nada a la red.
 *
 * Se abre el ARCHIVO (`file://`), como lo abre quien da doble clic: sin servidor.
 *
 * ¿Puede fallar? Sí. Con `.revela { opacity: 1 }` quitado del bloque de movimiento reducido, el primer grupo cae con el nombre
 * del elemento; con `visibility: hidden` quitado de `.detalle-interior`, cae el del árbol de accesibilidad; con la compensación
 * del cierre restada a ciegas, cae el de la deriva.
 */
const ARCHIVO = pathToFileURL(resolve("docs/BROCHURE.html")).href;

const abrir = async (pag: Page, idioma: "es" | "en" = "es") => {
  await pag.goto(`${ARCHIVO}?idioma=${idioma}`);
  await pag.waitForLoadState("load");
};

/** Lo que de verdad se ve: con caja y sin transparencia en TODA su cadena (la opacidad no se hereda en el estilo calculado). */
async function seVe(pag: Page, selector: string): Promise<boolean> {
  return pag.evaluate((sel) => {
    const el = Array.from(document.querySelectorAll(sel)).find(
      (e) =>
        (e as HTMLElement).offsetParent !== null ||
        getComputedStyle(e).position === "fixed",
    );
    if (!el) return false;
    const r = el.getBoundingClientRect();
    if (r.width < 1 || r.height < 1) return false;
    let o = 1;
    for (let n: Element | null = el; n; n = n.parentElement)
      o *= Number(getComputedStyle(n).opacity);
    return o > 0.99;
  }, selector);
}

test.describe("con movimiento reducido, el brochure se ve entero", () => {
  test.use({ reducedMotion: "reduce", viewport: { width: 1100, height: 900 } });

  test("cada escena está en su pose final: titular, pregunta, banda, rayas, contador y pie", async ({
    page,
  }) => {
    await abrir(page);
    for (const [que, sel] of [
      [
        "el titular de la portada",
        ".portada h1:not([style*='display: none']) .palabra",
      ],
      ["la línea de apoyo", ".portada .promesa"],
      ["la pregunta de la escena 2", ".e2 .escribe"],
      ["la banda de la escena 2", ".e2 .vitrina-fig"],
      ["el clímax", ".e4 .libro li.queda"],
      ["el contador de bytes", ".e4 .cero"],
      ["la escena de Windows", ".e5 .vitrina-fig"],
      ["el pie", "footer .conteo"],
    ] as const)
      expect(
        await seVe(page, sel),
        `${que} (${sel}) no se ve con movimiento reducido`,
      ).toBe(true);
    // La pregunta no queda recortada, y la raya de cada línea que muere ya está puesta.
    expect(
      await page
        .locator(".e2 .escribe")
        .first()
        .evaluate((e) => getComputedStyle(e).clipPath),
    ).toBe("none");
    const rayas = await page
      .locator(".e4 .raya")
      .evaluateAll((rs) => rs.map((r) => getComputedStyle(r).transform));
    expect(rayas).toHaveLength(3);
    for (const t of rayas)
      expect(
        t === "none" || t === "matrix(1, 0, 0, 1, 0, 0)",
        `raya en ${t}`,
      ).toBe(true);
  });

  test("las tarjetas se abren igual al llegar (es contenido, no decoración) y sin transición", async ({
    page,
  }) => {
    await abrir(page);
    await page.locator(".tarjeta").first().scrollIntoViewIfNeeded();
    await page.evaluate(() =>
      window.scrollBy({ top: 200, behavior: "instant" }),
    );
    await expect(
      page.locator(".tarjeta").first().locator(".tarjeta-boton"),
    ).toHaveAttribute("aria-expanded", "true");
    const durac = await page
      .locator(".detalle")
      .first()
      .evaluate((e) => getComputedStyle(e).transitionDuration);
    expect(durac).toMatch(/^0s$/);
  });
});

test.describe("con movimiento normal, todo llega a verse al recorrer la página", () => {
  test.use({
    reducedMotion: "no-preference",
    viewport: { width: 1100, height: 900 },
  });

  test("tras bajar de punta a punta, ningún bloque queda invisible (la red determinista de la entrada)", async ({
    page,
  }) => {
    await abrir(page);
    const alto = await page.evaluate(
      () => document.documentElement.scrollHeight,
    );
    for (let y = 0; y < alto; y += 600) {
      await page.evaluate(
        (yy) => window.scrollTo({ top: yy, behavior: "instant" }),
        y,
      );
      await page.waitForTimeout(60);
    }
    await page.waitForTimeout(900);
    const invisibles = await page
      .locator(".revela")
      .evaluateAll((els) =>
        els
          .filter((e) => !e.classList.contains("visto"))
          .map((e) => e.className),
      );
    expect(invisibles, "bloques .revela que no llegaron a mostrarse").toEqual(
      [],
    );
    // El contador del pie terminó en su número, y el texto accesible lo dijo completo desde el principio.
    await expect(page.locator("#conteo")).toHaveText("20");
  });

  test("abrir el archivo no pide NADA a la red", async ({ page }) => {
    const pedidas: string[] = [];
    page.on("request", (r) => {
      if (!r.url().startsWith("file:") && !r.url().startsWith("data:"))
        pedidas.push(r.url());
    });
    await abrir(page);
    await page.evaluate(() =>
      window.scrollTo({
        top: document.documentElement.scrollHeight,
        behavior: "instant",
      }),
    );
    await page.waitForTimeout(500);
    expect(pedidas).toEqual([]);
  });
});

test.describe("la apertura por lectura", () => {
  test.use({ reducedMotion: "reduce", viewport: { width: 1100, height: 900 } });

  const estado = (pag: Page, n: number) =>
    pag
      .locator(".tarjeta")
      .nth(n)
      .locator(".tarjeta-boton")
      .getAttribute("aria-expanded");

  test("las tarjetas llegan cerradas, y se abre la que cruza la línea de los dos tercios AUN SIN DETENERTE", async ({
    page,
  }) => {
    await abrir(page);
    expect(
      await Promise.all([0, 1, 2, 3, 4, 5].map((i) => estado(page, i))),
    ).toEqual(Array(6).fill("false"));
    // Bajada continua, a paso de rueda y sin parar: la primera se abre al cruzar la línea, no al reposo.
    const y0 = await page
      .locator(".tarjeta")
      .first()
      .evaluate((e) => e.getBoundingClientRect().top + window.scrollY);
    let abiertaEn = -1;
    for (let y = Math.max(0, y0 - 900); y < y0 + 100; y += 60) {
      await page.evaluate(
        (yy) => window.scrollTo({ top: yy, behavior: "instant" }),
        y,
      );
      if ((await estado(page, 0)) === "true") {
        abiertaEn = y;
        break;
      }
      await page.waitForTimeout(20);
    }
    expect(
      abiertaEn,
      "la primera tarjeta no se abrió al bajar",
    ).toBeGreaterThan(-1);
    const top = await page
      .locator(".tarjeta")
      .first()
      .evaluate((e) => e.getBoundingClientRect().top);
    expect(
      top,
      "se abrió con su cabecera entre el borde de arriba y los dos tercios",
    ).toBeGreaterThanOrEqual(0);
    expect(top).toBeLessThanOrEqual(900 * (2 / 3) + 1);
  });

  test("subiendo no se abre nada, y el toque manda para el resto de la visita", async ({
    page,
  }) => {
    await abrir(page);
    // Baja hasta el fondo (se abren y se cierran las que pasan) y vuelve a subir: nada se reabre contra la lectura.
    const alto = await page.evaluate(
      () => document.documentElement.scrollHeight,
    );
    await page.evaluate(
      (a) => window.scrollTo({ top: a, behavior: "instant" }),
      alto,
    );
    await page.waitForTimeout(100);
    const antesDeSubir = await Promise.all(
      [0, 1, 2, 3, 4, 5].map((i) => estado(page, i)),
    );
    await page.evaluate(() => window.scrollTo({ top: 0, behavior: "instant" }));
    await page.waitForTimeout(100);
    const alSubir = await Promise.all(
      [0, 1, 2, 3, 4, 5].map((i) => estado(page, i)),
    );
    for (let i = 0; i < 6; i++)
      if (antesDeSubir[i] === "false")
        expect(alSubir[i], `la tarjeta ${i + 1} se abrió subiendo`).toBe(
          "false",
        );
    // El toque: la abre, y bajar o subir no la cierra.
    const boton = page.locator(".tarjeta").nth(1).locator(".tarjeta-boton");
    await boton.scrollIntoViewIfNeeded();
    await boton.click();
    const quedo = await estado(page, 1);
    await page.evaluate(() =>
      window.scrollBy({ top: 3000, behavior: "instant" }),
    );
    await page.waitForTimeout(100);
    await page.evaluate(() =>
      window.scrollBy({ top: -6000, behavior: "instant" }),
    );
    await page.waitForTimeout(100);
    expect(
      await estado(page, 1),
      "lo que decidiste con el dedo lo cambió el recorrido",
    ).toBe(quedo);
  });

  test("al cerrarse una tarjeta por arriba, lo que se lee no se mueve (deriva cero, medida sobre un ancla visible)", async ({
    page,
  }) => {
    await abrir(page);
    // La ancla: el primer bloque (ancla, tarjeta o función) con `top >= 0` (jamás un contenedor: su techo está por encima de lo que se
    // recoge y «deriva» aunque lo visible no se mueva). Se mide la página asentada.
    const PASO = 140;
    // Con `top >= PASO` la ancla sigue a la vista después del paso: lo que está dentro de la tarjeta que se cierra (que solo se
    // cierra cuando salió entera) no puede ser ancla, porque «deriva» aunque lo que se lee no se mueva.
    const ancla = () =>
      page.evaluate((paso) => {
        const els = Array.from(
          document.querySelectorAll("[data-ancla], .tarjeta, .feature"),
        );
        // Las funciones de una tarjeta cerrada tienen caja (están recortadas, no ausentes) pero no se ven: no son ancla.
        const a = els.findIndex(
          (e) =>
            e.getBoundingClientRect().top >= paso &&
            getComputedStyle(e).visibility === "visible",
        );
        return a < 0 ? null : { i: a, top: els[a].getBoundingClientRect().top };
      }, PASO);
    const topDe = (i: number) =>
      page.evaluate(
        (k) =>
          document.querySelectorAll("[data-ancla], .tarjeta, .feature")[k].getBoundingClientRect()
            .top,
        i,
      );
    const y0 = await page
      .locator(".tarjeta")
      .first()
      .evaluate((e) => e.getBoundingClientRect().top + window.scrollY);
    await page.evaluate(
      (y) => window.scrollTo({ top: y - 600, behavior: "instant" }),
      y0,
    );
    let cierres = 0;
    for (let k = 0; k < 40; k++) {
      const antes = await ancla();
      if (!antes) break;
      const abiertasAntes = await page
        .locator(".tarjeta[data-abierta]")
        .count();
      await page.evaluate(
        (p) => window.scrollBy({ top: p, behavior: "instant" }),
        PASO,
      );
      await page.waitForTimeout(60); // un cuadro para que el rAF del recorrido corra
      const despues = await topDe(antes.i);
      expect(
        Math.abs(antes.top - despues - PASO),
        `paso ${k}: la ancla se movió ${antes.top - despues} px y el scroll fue ${PASO}`,
      ).toBeLessThanOrEqual(1);
      const abiertasDespues = await page
        .locator(".tarjeta[data-abierta]")
        .count();
      if (
        abiertasDespues < abiertasAntes + 1 &&
        abiertasDespues < abiertasAntes
      )
        cierres++;
    }
    expect(
      cierres,
      "el recorrido no llegó a cerrar ninguna tarjeta por arriba: la prueba no midió nada",
    ).toBeGreaterThan(0);
  });
});

test.describe("el árbol de accesibilidad y el idioma", () => {
  test.use({ reducedMotion: "reduce", viewport: { width: 1100, height: 900 } });

  test("lo cerrado no está en el árbol; al abrir la tarjeta, sí", async ({
    page,
  }) => {
    await abrir(page);
    const antes = await page.locator("body").ariaSnapshot();
    expect(antes).toContain("Qué hace");
    expect(antes, "una tarjeta cerrada recita su detalle").not.toContain(
      "Tu corpus, indexado",
    );
    expect(antes).not.toContain("Nada se graba, y se puede comprobar");
    await page.locator("#b-g2").click();
    const despues = await page.locator("body").ariaSnapshot();
    expect(despues).toContain("Tu corpus, indexado");
    expect(despues, "la otra tarjeta sigue cerrada").not.toContain(
      "Escucha las dos pistas",
    );
  });

  test("el idioma se cambia en el propio archivo, y el otro queda fuera del árbol", async ({
    page,
  }) => {
    await abrir(page, "es");
    await expect(page.locator("html")).toHaveAttribute("lang", "es");
    expect(await page.locator("body").ariaSnapshot()).not.toContain(
      "Your own experience",
    );
    await page.getByRole("button", { name: "EN", exact: true }).click();
    await expect(page.locator("html")).toHaveAttribute("lang", "en");
    await expect(page).toHaveTitle("Angel Ghost — Get to know it");
    const en = await page.locator("body").ariaSnapshot();
    expect(en).toContain(
      "Your own experience, in the meeting, at the right moment — and only you see it.",
    );
    expect(en).not.toContain("Tu propia experiencia");
    await expect(page.getByRole("button", { name: "EN", exact: true })).toHaveAttribute(
      "aria-pressed",
      "true",
    );
  });

  test("sigue el idioma del navegador al abrir", async ({ browser }) => {
    const ctx = await browser.newContext({ locale: "en-US" });
    const p = await ctx.newPage();
    await p.goto(ARCHIVO);
    await expect(p.locator("html")).toHaveAttribute("data-idioma", "en");
    await ctx.close();
  });
});

for (const tema of ["dark", "light"] as const)
  for (const idioma of ["es", "en"] as const)
    test.describe(`calidad · ${tema} · ${idioma}`, () => {
      test.use({
        reducedMotion: "reduce",
        colorScheme: tema,
        viewport: { width: 1100, height: 900 },
      });

      test("axe, con todas las tarjetas y los detalles abiertos, sin hallazgos", async ({
        page,
      }) => {
        await abrir(page, idioma);
        for (const b of await page.locator(".tarjeta-boton").all())
          await b.click();
        for (const d of await page.locator("details").all())
          await d.evaluate((e) => ((e as HTMLDetailsElement).open = true));
        await page.waitForTimeout(200);
        const r = await new AxeBuilder({ page }).analyze();
        expect(
          r.violations.map(
            (v) =>
              `${v.id}: ${v.nodes
                .map((n) => n.target.join(" "))
                .slice(0, 3)
                .join(" | ")}`,
          ),
        ).toEqual([]);
      });

      for (const ancho of [360, 390, 768, 1280])
        test(`nada desborda la página a ${ancho} px`, async ({ page }) => {
          await page.setViewportSize({ width: ancho, height: 800 });
          await abrir(page, idioma);
          for (const b of await page.locator(".tarjeta-boton").all())
            await b.click();
          const [sw, cw] = await page.evaluate(() => [
            document.documentElement.scrollWidth,
            document.documentElement.clientWidth,
          ]);
          expect(sw, `scrollWidth ${sw} contra ${cw}`).toBeLessThanOrEqual(cw);
          // Las muestras se deslizan DENTRO de su marco: nada que no sea de ese marco se sale de la pantalla.
          const fuera = await page.evaluate(() =>
            Array.from(document.querySelectorAll("body *"))
              .filter(
                (e) =>
                  !e.closest(".vitrina") &&
                  e.getBoundingClientRect().right >
                    document.documentElement.clientWidth + 1 &&
                  getComputedStyle(e).display !== "none",
              )
              .map((e) => e.tagName + "." + e.className),
          );
          expect(fuera).toEqual([]);
        });
    });
