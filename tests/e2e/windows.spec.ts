import { test, expect } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";

/**
 * ANGEL GHOST EN WINDOWS (sprint 006, ADR 022), en el navegador con la muestra del contrato.
 *
 * `?plataforma=windows` pinta lo que Rust diría en un PC con Windows (`plataformaFuera`): el bloque
 * «Hoy en Windows» de Honestidad y los permisos de Windows en Permisos. Sin el parámetro, la app se ve
 * como en macOS, y eso también se comprueba: en macOS no aparece ni una línea de Windows.
 *
 * Los textos se leen en español (PUERTO_E2E=3417). El inglés lo cubre la paridad del test unitario
 * `windows-honestidad-y-manual` y, para la banda y las pantallas, `windows-sin-mac`.
 *
 * Sprint 007: la videollamada. Las cinco funciones nuevas en «Ya funciona», la banda y Sesión sin el Mac, y las
 * teclas de Windows en vez de `⌃⌥`.
 */

// La app elige el idioma por el del navegador (src/App.tsx): estos textos se leen en español.
test.use({ locale: "es-ES", reducedMotion: "reduce" });

test("en Windows, Honestidad enseña «Hoy en Windows» con sus tres grupos y la transcripción medida", async ({
  page,
}) => {
  await page.goto("/?ventana=principal&pantalla=honestidad&plataforma=windows");
  await expect(
    page.getByRole("heading", { name: "Hoy en Windows" }),
  ).toBeVisible();
  await expect(page.getByText("Ya funciona")).toBeVisible();
  await expect(page.getByRole("heading", { name: "Todavía no" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "No en Windows por ahora" })).toBeVisible();
  // La banda sin verificar se dice con su frase, no solo con un color. Desde el sprint 007 lo dicen dos filas: la
  // banda protegida y la videollamada, porque las dos dependen de lo que el usuario compruebe en su PC.
  await expect(
    page.getByText(/sin verificar en Teams, Zoom ni Meet/),
  ).toHaveCount(2);
  // La transcripción: el modelo medido, con la cifra y el techo de 4 s.
  await expect(
    page.getByText(/large-v3-turbo: 10 s de audio en 2\.?350 ms/),
  ).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Medir otra vez" }),
  ).toBeVisible();
  // Con un modelo rápido no hay escalón: el botón del modelo pequeño solo aparece si pasa de 4 s.
  await expect(
    page.getByRole("button", { name: /Bajar el modelo pequeño/ }),
  ).toHaveCount(0);
});

test("en Windows, «Ya funciona» lleva las cinco funciones de la videollamada", async ({ page }) => {
  await page.goto("/?ventana=principal&pantalla=honestidad&plataforma=windows");
  const ya = page.locator("section, div").filter({ has: page.getByText("Ya funciona", { exact: true }) }).last();
  for (const nombre of [
    "Las videollamadas: el audio del cliente",
    "Leer la pantalla de la reunión",
    "La ficha al oído",
    "Ensayar con la voz que lee las preguntas",
    "Los atajos de teclado globales",
  ]) {
    await expect(ya.getByText(nombre)).toBeVisible();
  }
  // Y las que siguen sin llegar no se cuelan en «Ya funciona»: el acople y la sugerencia dicen que llegan después.
  await expect(page.getByText("llega en una versión próxima").first()).toBeVisible();
});

test("en macOS, Honestidad no enseña nada de Windows", async ({ page }) => {
  await page.goto("/?ventana=principal&pantalla=honestidad");
  await expect(
    page.getByRole("heading", { name: "Qué vive en la memoria ahora" }),
  ).toBeVisible();
  await expect(page.getByText("Hoy en Windows")).toHaveCount(0);
});

test("en Windows, Permisos enseña el micrófono de Windows y no inventa los demás", async ({
  page,
}) => {
  await page.goto("/?ventana=principal&pantalla=permisos&plataforma=windows");
  // La muestra de la maqueta trae el micrófono concedido: sin botón, con su chip. Los otros tres no existen aún.
  await expect(page.getByRole("button", { name: "Abrir Ajustes de Windows" })).toHaveCount(0);
  await expect(page.getByText("Conceder en macOS")).toHaveCount(0);
  // Sprint 007: el audio del sistema y la pantalla de la reunión funcionan sin ningún permiso; solo acoplarse llega después.
  await expect(page.getByText("No pide permiso en Windows")).toHaveCount(2);
  await expect(page.getByText("Llega en una versión próxima")).toHaveCount(1);
  await expect(page.getByText("El micrófono en Windows")).toBeVisible();
  await expect(
    page.getByText("Ajustes → Privacidad y seguridad → Micrófono"),
  ).toBeVisible();
});

const PANTALLAS = [
  {
    que: "honestidad en Windows",
    url: "ventana=principal&pantalla=honestidad&plataforma=windows",
  },
  {
    que: "permisos en Windows",
    url: "ventana=principal&pantalla=permisos&plataforma=windows",
  },
];

for (const tema of ["dark", "light"] as const) {
  for (const p of PANTALLAS) {
    test(`«${p.que}» · tema ${tema} · sin hallazgos de axe`, async ({
      page,
    }) => {
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

/**
 * Hallazgo A4 de la auditoría del S6: en Windows, Idioma y Honestidad no pueden decir «macOS» ni «Mac»: el motor
 * de voz y el modelo los baja la app, no el sistema (ADR 022). Este test es el criterio de aceptación de ese ajuste.
 */
for (const p of [
  { que: "idioma", url: "ventana=principal&pantalla=idioma&plataforma=windows" },
  { que: "honestidad", url: "ventana=principal&pantalla=honestidad&plataforma=windows" },
]) {
  test(`en Windows, «${p.que}» no dice «macOS» ni «Mac»`, async ({ page }) => {
    await page.goto(`/?${p.url}`);
    await expect(page.locator("h1").first()).toBeVisible();
    await expect(page.getByText(/macOS|\bMac\b/)).toHaveCount(0);
  });
}

/**
 * Hallazgo A6 de la auditoría del S6: Notas y la bandeja prometían que lo guardado se borra «aunque no abras la app».
 * En Windows esa tarea no existe (ADR 022: el borrado con la app cerrada, todavía no), así que la frase no puede salir.
 */
for (const url of [
  "ventana=principal&pantalla=notas&estado=archivo&plataforma=windows",
  "ventana=principal&pantalla=notas&estado=al-cerrar-bandeja&plataforma=windows",
]) {
  test(`en Windows, Notas no promete el borrado con la app cerrada (${url.split("estado=")[1].split("&")[0]})`, async ({ page }) => {
    await page.goto(`/?${url}`);
    await expect(page.locator("h1").first()).toBeVisible();
    await expect(page.getByText(/aunque no abras la app/)).toHaveCount(0);
  });
}

/** El contrario del test de arriba: en macOS esa frase sí sale en las mismas vistas. Si no saliera, el cero de arriba no
 * demostraría nada. */
for (const url of [
  "ventana=principal&pantalla=notas&estado=archivo",
  "ventana=principal&pantalla=notas&estado=al-cerrar-bandeja",
]) {
  test(`en macOS, la frase del borrado sí sale en ${url.split("estado=")[1]} (control del cero de Windows)`, async ({ page }) => {
    await page.goto(`/?${url}`);
    await expect(page.locator("h1").first()).toBeVisible();
    await expect(page.getByText(/aunque no abras la app/).first()).toBeVisible();
  });
}

/**
 * **La banda y Sesión en Windows (sprint 007).** La banda no promete invisibilidad donde nadie la verificó (regla dura 6) ni
 * nombra el Mac, y las teclas son las de Windows. El control: en macOS las mismas pantallas siguen diciendo lo de siempre.
 */
const BANDA_SIN_VERIFICAR = "ventana=banda&estado=sin-verificar";

test("en Windows, la banda sin verificar lo dice sin nombrar el Mac y con la tecla de Windows", async ({ page }) => {
  await page.setViewportSize({ width: 900, height: 220 });
  await page.goto(`/?${BANDA_SIN_VERIFICAR}&plataforma=windows`);
  await expect(page.getByText("Sin verificar en Teams, Zoom ni Meet · se verifica en tu PC")).toBeVisible();
  await expect(page.getByText(/Verificada solo en Meet|macOS|\bMac\b/)).toHaveCount(0);
  await expect(page.getByText(/O corta la banda con Alt\+Mayús\+Q mientras compartes/)).toBeVisible();
  await expect(page.getByText(/⌃|⌥|⎋/)).toHaveCount(0);
});

test("en Windows, la banda con una ficha pinta las teclas de Windows", async ({ page }) => {
  await page.setViewportSize({ width: 900, height: 220 });
  await page.goto("/?ventana=banda&estado=ficha&plataforma=windows");
  await expect(page.locator("kbd", { hasText: "Alt+Mayús+F" })).toBeVisible();
  await expect(page.locator("kbd", { hasText: "Alt+Mayús+C" })).toBeVisible();
  await expect(page.getByText(/⌃|⌥|⎋/)).toHaveCount(0);
});

test("en macOS, la banda sigue con sus teclas y su verificación (control de los dos de arriba)", async ({ page }) => {
  await page.setViewportSize({ width: 900, height: 220 });
  await page.goto(`/?${BANDA_SIN_VERIFICAR}`);
  await expect(page.getByText(/Verificada solo en Meet · macOS 26\.6\.2/)).toBeVisible();
  await expect(page.getByText(/O corta la banda con ⌥⎋ mientras compartes/)).toBeVisible();
  await expect(page.getByText(/Alt\+Mayús/)).toHaveCount(0);
});

test("en Windows, Sesión no habla del Mac y dice que la protección está sin verificar", async ({ page }) => {
  await page.goto("/?ventana=principal&pantalla=sesion&plataforma=windows");
  await expect(page.locator("h1").first()).toBeVisible();
  await expect(page.getByText(/tu propio PC/)).toBeVisible();
  await expect(page.getByText(/sin verificar: se verifica con una videollamada real en tu PC/)).toBeVisible();
  await expect(page.getByText(/macOS|\bMac\b/)).toHaveCount(0);
});

for (const tema of ["dark", "light"] as const) {
  for (const p of [
    { que: "banda sin verificar en Windows", url: `${BANDA_SIN_VERIFICAR}&plataforma=windows` },
    { que: "sesión en Windows", url: "ventana=principal&pantalla=sesion&plataforma=windows" },
  ]) {
    test(`«${p.que}» · tema ${tema} · sin hallazgos de axe`, async ({ page }) => {
      await page.setViewportSize({ width: 900, height: 700 });
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
