import { test, expect, type Page } from "@playwright/test";

/**
 * EL RECORRIDO — que la webview arranque y se pueda caminar entera.
 *
 * **Por qué este archivo existe y por qué llega tarde.** El job `e2e` de la integración continua
 * corre `playwright test --pass-with-no-tests`, y durante cuatro fases no hubo ni una prueba: el
 * check estuvo verde sin ejecutar nada. Es la segunda pregunta de la regla de los gates —*¿lo
 * viste correr, alguna vez?*— respondida que no. Se paga aquí.
 *
 * **Qué puede y qué no puede afirmar un e2e de esta app.** Un navegador no ve una ventana nativa:
 * no puede decir nada de la protección de captura, del acople ni del audio. Lo que sí puede, y es
 * lo que hace, es comprobar que **la interfaz se pinta y se navega** en los dos tamaños de ventana
 * reales, en los dos temas y en los dos idiomas. Lo demás vive en el gate ⭐ de la guía de prueba,
 * y el resumen del sprint lo dice sin adornos.
 */

const PANTALLAS = ["sesion", "permisos", "corpus", "honestidad", "idioma"] as const;

async function abrir(pag: Page, busqueda: string) {
  await pag.goto(`/?${busqueda}`);
  await pag.evaluate(() => document.fonts.ready);
}

test.describe("el cuaderno", () => {
  test("las cinco pantallas se pintan y ninguna deja la ventana vacía", async ({ page }) => {
    for (const pantalla of PANTALLAS) {
      await abrir(page, `ventana=principal&pantalla=${pantalla}`);
      await expect(page.locator(".titulo h1")).toBeVisible();
      await expect(page.locator(".contenido")).not.toBeEmpty();
    }
  });

  test("el rail navega entre las cinco, y no ofrece las dos que no existen", async ({ page }) => {
    await abrir(page, "ventana=principal&pantalla=sesion");
    const rail = page.locator("nav.rail");
    await expect(rail.locator("a")).toHaveCount(PANTALLAS.length);
    // Las secciones que aún no existen están en el rail pero no son enlaces: el usuario ve que
    // van a llegar sin que una de ellas le lleve a una pantalla en blanco.
    await expect(rail.locator(".item.pendiente")).toHaveCount(2);

    await rail.getByRole("link", { name: /corpus/i }).click();
    await expect(page.locator(".titulo h1")).toHaveText("Corpus");
  });

  test("se camina con el teclado, sin tocar el ratón", async ({ page }) => {
    await abrir(page, "ventana=principal&pantalla=sesion");
    const alcanzados = new Set<string>();
    for (let i = 0; i < 25; i++) {
      await page.keyboard.press("Tab");
      const donde = await page.evaluate(() => {
        const e = document.activeElement;
        return e ? `${e.tagName}:${(e.textContent ?? "").trim().slice(0, 20)}` : "";
      });
      if (donde) alcanzados.add(donde);
    }
    expect(alcanzados.size).toBeGreaterThan(4);
  });
});

test.describe("la banda", () => {
  test("los cinco estados de contenido se pintan", async ({ page }) => {
    for (const estado of ["esperando", "buscando", "ficha", "sin-resultado", "sin-verificar"]) {
      await abrir(page, `ventana=banda&estado=${estado}`);
      await expect(page.locator("section.banda")).toHaveAttribute("data-estado", estado);
      await expect(page.locator(".cuerpo-b")).not.toBeEmpty();
    }
  });

  test("la ficha trae su fuente, que es lo que la hace creíble", async ({ page }) => {
    await abrir(page, "ventana=banda&estado=ficha");
    await expect(page.locator(".titular-b")).not.toBeEmpty();
    await expect(page.locator(".linea-b")).not.toBeEmpty();
    await expect(page.locator(".fuente-b")).not.toBeEmpty();
  });

  test("«sin resultado» dice qué buscó y cómo conducirse, en vez de callarse", async ({ page }) => {
    await abrir(page, "ventana=banda&estado=sin-resultado");
    await expect(page.locator(".titular-b")).toContainText(/certificación|certification/i);
    await expect(page.locator(".maniobra-b .t")).not.toBeEmpty();
    // La maniobra sale de un catálogo determinista: no puede llevar las marcas que en este
    // sistema significan «lo redactó un modelo».
    await expect(page.locator(".maniobra-b.halo")).toHaveCount(0);
    await expect(page.locator('.maniobra-b use[href="#i-chispa"]')).toHaveCount(0);
  });
});
