import { test, expect, type Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import { en } from "../../src/i18n/en";
import { TEXTOS_DE_WINDOWS } from "../../src/i18n/windows";
import { TEXTOS_DE_PRESENCIAL } from "../../src/i18n/presencial-nuevo";
import { CAMBIOS_DE_WINDOWS } from "../../src/i18n/windows-cambios";
import { CLAVES_DE_WINDOWS } from "../../src/plataforma";

/**
 * **LA APP EN INGLÉS, DE PUNTA A PUNTA** (sprint 008, cierre del H-1; regla 20: un idioma por sprint, el otro al cierre del hito).
 *
 * Hasta ahora los e2e corrían «lo que el navegador diga» y `windows.spec.ts` en español. Este archivo fija el inglés: el
 * navegador es `en-US` y **cada aserción sale del diccionario `en`** (y de `windows.ts` y `presencial-nuevo.ts`), no de un
 * literal escrito aquí: si un texto cambia, cambia en un sitio. Cubre el recorrido, las funciones de Windows, el modo
 * presencial y la accesibilidad (axe) de lo que S5 a S8 pintan, en los dos temas. Es el único archivo que Playwright corre en
 * el proyecto `en-US`; los demás ya no se repiten en inglés porque el recorrido y el axe de abajo lo son.
 *
 * ¿Puede fallar? Sí: con un texto del diccionario inglés que cambie sin que cambie la pantalla (o la pantalla en español),
 * o con un «Mac» en la pantalla de Windows, es rojo con la cadena.
 */
test.use({ locale: "en-US", reducedMotion: "reduce" });

const PANTALLAS = ["sesion", "ensayo", "permisos", "corpus", "notas", "honestidad", "idioma", "ia"] as const;
const NAVEGACION = [
  en.cuaderno.navSesion,
  en.cuaderno.navEnsayo,
  en.cuaderno.navPermisos,
  en.cuaderno.navCorpus,
  en.cuaderno.navNotas,
  en.cuaderno.navHonestidad,
  en.cuaderno.navIdioma,
  en.cuaderno.navIa,
];
const W = TEXTOS_DE_WINDOWS.en;
const P = TEXTOS_DE_PRESENCIAL.en;

async function abrir(page: Page, busqueda: string) {
  await page.goto(`/?${busqueda}`);
  await page.evaluate(() => document.fonts.ready);
}

test.describe("el cuaderno, en inglés", () => {
  test("la app arranca en inglés y el rail trae las ocho pantallas con sus nombres del diccionario", async ({ page }) => {
    await abrir(page, "ventana=principal&pantalla=sesion");
    expect(await page.evaluate(() => document.documentElement.lang)).toBe("en");
    const rail = page.locator("nav.rail");
    await expect(rail.locator("a")).toHaveCount(PANTALLAS.length);
    for (const nombre of NAVEGACION) await expect(rail.getByRole("link", { name: nombre, exact: true })).toBeVisible();
  });

  test("cada pantalla se pinta con su título y ninguna deja la ventana vacía", async ({ page }) => {
    for (const pantalla of PANTALLAS) {
      await abrir(page, `ventana=principal&pantalla=${pantalla}`);
      await expect(page.locator(".titulo h1")).toBeVisible();
      await expect(page.locator(".contenido")).not.toBeEmpty();
    }
  });
});

test.describe("Windows, en inglés", () => {
  test("«Hoy en Windows» tiene sus dos grupos y las quince funciones con el nombre del diccionario", async ({ page }) => {
    await abrir(page, "ventana=principal&pantalla=honestidad&plataforma=windows");
    await expect(page.getByRole("heading", { name: W.titulo })).toBeVisible();
    await expect(page.getByText(W.ya, { exact: true })).toBeVisible();
    await expect(page.getByText(W.noPorAhora, { exact: true })).toBeVisible();
    for (const clave of CLAVES_DE_WINDOWS) {
      await expect(page.getByText(W.funciones[clave].nombre, { exact: true }).first(), clave).toBeVisible();
    }
  });

  test("ninguna pantalla de Windows nombra el Mac, ni en inglés", async ({ page }) => {
    for (const pantalla of ["sesion", "permisos", "honestidad", "idioma", "ia", "notas"]) {
      await abrir(page, `ventana=principal&pantalla=${pantalla}&plataforma=windows`);
      await expect(page.locator("h1").first()).toBeVisible();
      await expect(page.getByText(/macOS|\bMac\b|Finder|Keychain|Touch ID|Apple/), pantalla).toHaveCount(0);
    }
  });

  test("Permisos dice que Windows no pide permiso, y lleva el botón de Ajustes de Windows", async ({ page }) => {
    await abrir(page, "ventana=principal&pantalla=permisos&plataforma=windows");
    await expect(page.getByText(W.permisos.sinPermiso).first()).toBeVisible();
  });

  test("la banda de Windows dice «not verified» y trae las teclas de Windows", async ({ page }) => {
    await page.setViewportSize({ width: 1180, height: 200 });
    await abrir(page, "ventana=banda&estado=sin-verificar&plataforma=windows");
    await expect(page.getByText(CAMBIOS_DE_WINDOWS.en.banda?.sinVerificarCuando ?? "no está en el diccionario").first()).toBeVisible();
    await expect(page.getByText(/⌃|⌥|⎋|macOS|\bMac\b/)).toHaveCount(0);
    await expect(page.getByText(/Alt\+Shift\+Q/).first()).toBeVisible();
  });
});

test.describe("el modo presencial, en inglés", () => {
  test("Sesión en presencial enseña la sala, con lo que no guarda y su botón", async ({ page }) => {
    await abrir(page, "ventana=principal&pantalla=sesion&estado=presencial");
    await expect(page.getByRole("heading", { name: en.cuaderno.presencial.titulo })).toBeVisible();
    await expect(page.getByText(en.cuaderno.presencial.noSabeCuales)).toBeVisible();
    await expect(page.getByRole("button", { name: en.cuaderno.presencial.empezar })).toBeVisible();
    await expect(page.getByText(en.cuaderno.presencial.idiomaDeLaSala).first()).toBeVisible();
  });

  test("con la sesión en marcha, dice «Running» y ofrece terminar", async ({ page }) => {
    await abrir(page, "ventana=principal&pantalla=sesion&estado=presencial-marcha");
    await expect(page.getByText(en.cuaderno.presencial.enMarcha).first()).toBeVisible();
    await expect(page.getByRole("button", { name: en.cuaderno.terminarSesion })).toBeVisible();
  });

  test("Notas, al cerrar una sesión presencial, desactiva «Keep my turns» y nombra la sala", async ({ page }) => {
    await abrir(page, "ventana=principal&pantalla=notas&estado=al-cerrar-presencial");
    await expect(page.getByText(en.cuaderno.presencial.noSabeCuales)).toBeVisible();
    await expect(page.getByText(en.cuaderno.enPresencial.bufSala)).toBeVisible();
    await expect(page.getByRole("switch", { name: en.notas.conservarMisTurnos })).toHaveAttribute("aria-disabled", "true");
  });

  test("los textos nuevos de la sala existen en inglés y no prometen nada vetado", async () => {
    expect(P.sinVoz("en-US")).toMatch(/no voice/i);
    expect(P.externa("AirPods", P.enElMac)).toContain("System Settings");
    for (const frase of [P.sinVoz("x"), P.externa("x", "y")]) expect(frase).not.toMatch(/undetectable|cheat|trick/i);
  });
});

/** Accesibilidad de lo que los sprints 5 a 8 pintan, en inglés y en los dos temas. */
const ESTADOS_AXE = [
  { que: "presencial", url: "ventana=principal&pantalla=sesion&estado=presencial" },
  { que: "presencial en marcha", url: "ventana=principal&pantalla=sesion&estado=presencial-marcha" },
  { que: "sesión en Windows", url: "ventana=principal&pantalla=sesion&plataforma=windows" },
  { que: "Honestidad en Windows", url: "ventana=principal&pantalla=honestidad&plataforma=windows" },
  { que: "Permisos en Windows", url: "ventana=principal&pantalla=permisos&plataforma=windows" },
  { que: "IA en Windows", url: "ventana=principal&pantalla=ia&plataforma=windows" },
  { que: "Idioma en Windows", url: "ventana=principal&pantalla=idioma&plataforma=windows" },
  { que: "Notas al cerrar", url: "ventana=principal&pantalla=notas&estado=al-cerrar-bandeja" },
  { que: "Notas al cerrar, presencial", url: "ventana=principal&pantalla=notas&estado=al-cerrar-presencial" },
];
for (const tema of ["dark", "light"] as const) {
  for (const e of ESTADOS_AXE) {
    test(`«${e.que}» en inglés · tema ${tema} · sin hallazgos de axe`, async ({ page }) => {
      await page.emulateMedia({ colorScheme: tema });
      await abrir(page, e.url);
      await expect(page.locator("h1").first()).toBeVisible();
      // Las mismas etiquetas que `a11y.spec.ts`: WCAG 2.0 y 2.1, A y AA.
      const { violations } = await new AxeBuilder({ page }).withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"]).analyze();
      expect(
        violations.map((v) => `${v.id}: ${v.nodes.length} · ${v.help}`),
        `axe encontró ${violations.length} incumplimiento(s) en «${e.que}» (${tema})`,
      ).toEqual([]);
    });
  }
}
