import { test, expect, type Page } from "@playwright/test";

/**
 * REDUCED MOTION — que quien pide menos movimiento no reciba menos APP.
 *
 * La regla del kit nació de una reincidencia: **la forma del árbol jamás depende de
 * `useReducedMotion()`**. Si el hook decide QUÉ elementos se pintan, quien tiene activado
 * «reducir movimiento» acaba viendo una página distinta —a veces con piezas de menos— justo por
 * haber pedido el cinturón que debería cuidarle.
 *
 * Angel Ghost no usa ese hook: el movimiento vive en CSS (`@media (prefers-reduced-motion)`). Esta
 * prueba es lo que impide que eso cambie sin que nadie se entere, y lo comprueba **por
 * visibilidad real**, no por presencia en el DOM: un elemento con `opacity: 0` esperando una
 * animación que nunca va a llegar está en el árbol y no se ve, que es el fallo exacto que esta
 * prueba existe para cazar.
 */

const ENCUADRES = [
  { que: "sesión", url: "ventana=principal&pantalla=sesion", clave: ".titulo h1" },
  { que: "corpus", url: "ventana=principal&pantalla=corpus", clave: ".titulo h1" },
  { que: "honestidad", url: "ventana=principal&pantalla=honestidad", clave: ".contador .cifra" },
  { que: "banda · ficha", url: "ventana=banda&estado=ficha", clave: ".titular-b" },
  { que: "banda · sin resultado", url: "ventana=banda&estado=sin-resultado", clave: ".maniobra-b .t" },
];

/** Lo que de verdad se ve: en el árbol, con caja, y sin transparencia. */
async function seVe(pag: Page, selector: string): Promise<boolean> {
  return pag.evaluate((sel) => {
    const e = document.querySelector(sel);
    if (!e) return false;
    const caja = e.getBoundingClientRect();
    const estilo = getComputedStyle(e);
    return (
      caja.width > 0 &&
      caja.height > 0 &&
      estilo.visibility !== "hidden" &&
      estilo.display !== "none" &&
      Number(estilo.opacity) > 0.01
    );
  }, selector);
}

for (const modo of ["reduce", "no-preference"] as const) {
  test.describe(`con prefers-reduced-motion: ${modo}`, () => {
    test.use({ reducedMotion: modo });

    for (const e of ENCUADRES) {
      test(`«${e.que}» se ve de verdad`, async ({ page }) => {
        await page.goto(`/?${e.url}`);
        await page.evaluate(() => document.fonts.ready);
        expect(await seVe(page, e.clave), `${e.que}: ${e.clave} está en el árbol pero no se ve`).toBe(true);
      });
    }
  });
}

/**
 * Y el filo que la regla nombra: **el mismo árbol con y sin el cinturón**. Si alguna vez alguien
 * mete una condición de JavaScript sobre `prefers-reduced-motion`, esta prueba lo dice en la
 * misma corrida, comparando la estructura —no el estilo, que sí puede y debe cambiar.
 */
test("la FORMA del árbol no cambia con reduced motion", async ({ browser }) => {
  const forma = async (reducido: boolean) => {
    const ctx = await browser.newContext({ reducedMotion: reducido ? "reduce" : "no-preference" });
    const pag = await ctx.newPage();
    await pag.goto("/?ventana=principal&pantalla=corpus");
    await pag.evaluate(() => document.fonts.ready);
    // Solo la estructura: etiquetas y clases, sin estilos ni textos.
    const arbol = await pag.evaluate(() =>
      [...document.querySelectorAll("#root *")]
        .map((e) => `${e.tagName}.${e.getAttribute("class") ?? ""}`)
        .join("|"),
    );
    await ctx.close();
    return arbol;
  };
  expect(await forma(true)).toBe(await forma(false));
});
