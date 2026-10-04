import { test, expect } from "@playwright/test";
import { readdirSync } from "node:fs";
import { resolve } from "node:path";

/**
 * GATE — **la pasada de INTERACCIÓN de la maqueta** (kit v1.32.0, regla 22 del kit; en esta app, regla
 * 23 «los controles de la maqueta tienen su script»). La mitad estática —que cada botón de la barra
 * lleve algo que `maqueta.js` atiende— la mira `tests/unit/controladores-maqueta.test.ts`. Esta es la
 * otra mitad: **se hace clic de verdad en cada control de la sala de diseño y algo tiene que cambiar.**
 *
 *  - cada botón de estado deja ese estado puesto y pulsado, y **ningún par de estados se ve igual**
 *    (qué elementos quedan visibles, qué nota se enciende, cómo se acopla la reunión): un estado que
 *    se ve como otro es un botón que no enseña nada;
 *  - el tema cambia el fondo del producto (`--bg`), y el idioma cambia el texto de verdad.
 *
 * No depende de las fuentes (mide qué se ve, no cuánto ocupa): corre en el job `e2e`, una vez.
 *
 * ¿Puede fallar? Sí: nació en rojo con un botón de `posicion.html` repitiendo el estado de otro
 * (bitácora del sprint 004, fase 0).
 */
const DISENO = resolve("docs/diseno");
const PAGINAS = readdirSync(DISENO).filter((f) => f.endsWith(".html"));

test("cada control de la sala de diseño cambia algo al pulsarlo", async ({ page }, info) => {
  test.skip(info.project.name !== "ventana-principal", "se corre una vez, no una por proyecto");
  test.setTimeout(180_000);

  const fallos: string[] = [];
  let clics = 0;
  for (const pagina of PAGINAS) {
    await page.goto("file://" + resolve(DISENO, pagina));
    const estados = await page.$$eval(".mq-bar button[data-estado]", (bs) =>
      bs.map((b) => (b as HTMLElement).dataset.estado as string),
    );

    const vistos = new Map<string, string>();
    for (const estado of estados) {
      await page.click(`.mq-bar button[data-estado="${estado}"]`);
      clics++;
      const { puesto, pulsado, huella } = await page.evaluate((e) => {
        const html = document.documentElement;
        const visibles = [...document.querySelectorAll<HTMLElement>("[data-en]")]
          .map((el, i) => (el.hidden ? "" : String(i)))
          .filter(Boolean)
          .join(",");
        const nota = [...document.querySelectorAll(".mq-nota .n")].findIndex((n) => n.classList.contains("activa"));
        const acople = [...document.querySelectorAll<HTMLElement>("[data-acople-en]")].map((el) => el.dataset.acople ?? "-").join(",");
        const transcript = [...document.querySelectorAll<HTMLElement>("[data-transcript-en]")].map((el) => el.dataset.transcript).join(",");
        const boton = document.querySelector(`.mq-bar button[data-estado="${e}"]`);
        return {
          puesto: html.dataset.estado,
          pulsado: boton?.getAttribute("aria-pressed"),
          huella: `v:${visibles}|n:${nota}|a:${acople}|t:${transcript}`,
        };
      }, estado);
      if (puesto !== estado) fallos.push(`${pagina} · «${estado}»: el clic dejó puesto «${puesto}»`);
      if (pulsado !== "true") fallos.push(`${pagina} · «${estado}»: el botón no queda pulsado`);
      const igual = vistos.get(huella);
      if (igual) fallos.push(`${pagina} · «${estado}» se ve igual que «${igual}»: el botón no enseña nada nuevo`);
      else vistos.set(huella, estado);
    }

    const temas = await page.$$eval(".mq-bar button[data-theme-set]", (bs) => bs.map((b) => (b as HTMLElement).dataset.themeSet as string));
    const fondos = new Set<string>();
    for (const tema of temas) {
      await page.click(`.mq-bar button[data-theme-set="${tema}"]`);
      clics++;
      // El fondo de la sala es neutro a propósito; lo que cambia es la superficie del PRODUCTO, que
      // lee el token `--bg` del tema (`assets/ghost.css`).
      fondos.add(await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue("--bg").trim()));
    }
    if (temas.length > 1 && fondos.size < temas.length) fallos.push(`${pagina}: cambiar el tema no cambia el fondo del producto (--bg)`);

    const idiomas = await page.$$eval(".mq-bar button[data-lang-set]", (bs) => bs.map((b) => (b as HTMLElement).dataset.langSet as string));
    const textos = new Set<string>();
    for (const idioma of idiomas) {
      await page.click(`.mq-bar button[data-lang-set="${idioma}"]`);
      clics++;
      textos.add(await page.evaluate(() => document.body.innerText));
    }
    if (idiomas.length > 1 && textos.size < idiomas.length) fallos.push(`${pagina}: cambiar el idioma no cambia el texto`);
  }

  expect(clics, "no se pulsó ningún control: el recorrido no midió nada").toBeGreaterThan(100);
  expect(fallos).toEqual([]);
});
