import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

/**
 * AXE sobre cada pantalla, en los dos temas y con el cinturón de movimiento puesto.
 *
 * **El tema importa y no es decorativo.** El usuario de esta app tiene daltonismo leve y el
 * design system obliga a que todo estado sea símbolo + texto + color. El contraste se mide aquí,
 * en los dos temas, porque una paleta que cumple AA en oscuro puede no cumplirlo en claro.
 *
 * **Y con reduced motion**, porque es la combinación que el kit exige explícitamente: lo que se
 * esconde esperando una animación que no va a llegar no lo ve un axe en condiciones normales.
 */

const PANTALLAS = [
  { que: "sesión", url: "ventana=principal&pantalla=sesion" },
  { que: "permisos", url: "ventana=principal&pantalla=permisos" },
  { que: "corpus", url: "ventana=principal&pantalla=corpus" },
  { que: "honestidad", url: "ventana=principal&pantalla=honestidad" },
  { que: "idioma", url: "ventana=principal&pantalla=idioma" },
  { que: "banda · ficha", url: "ventana=banda&estado=ficha" },
  { que: "banda · sin resultado", url: "ventana=banda&estado=sin-resultado" },
  { que: "banda · sin verificar", url: "ventana=banda&estado=sin-verificar&verificado=0" },
];

test.use({ reducedMotion: "reduce" });

for (const tema of ["dark", "light"] as const) {
  for (const p of PANTALLAS) {
    test(`«${p.que}» · tema ${tema} · sin hallazgos de axe`, async ({ page }) => {
      await page.goto(`/?${p.url}`);
      await page.evaluate((t) => {
        document.documentElement.dataset.theme = t;
      }, tema);
      await page.evaluate(() => document.fonts.ready);

      const { violations } = await new AxeBuilder({ page })
        .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
        .analyze();

      expect(
        violations.map((v) => `${v.id}: ${v.nodes.length} · ${v.help}`),
        `axe encontró ${violations.length} incumplimiento(s) en «${p.que}» (${tema})`,
      ).toEqual([]);
    });
  }
}
