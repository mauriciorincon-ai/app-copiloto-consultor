import { readFileSync } from "node:fs";
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
 *
 * Sprint 008: Windows completo. «Todavía no» desaparece: el acople, la sugerencia, el radar, el borrado con la app
 * cerrada y el desbloqueo pasan a «Ya funciona»; la IA dice que en Windows no hay modelo del sistema; el radar pinta su
 * tecla de Windows.
 */

// La app elige el idioma por el del navegador (src/App.tsx): estos textos se leen en español.
test.use({ locale: "es-ES", reducedMotion: "reduce" });

test("en Windows, Honestidad enseña «Hoy en Windows» con sus dos grupos y la transcripción medida", async ({
  page,
}) => {
  await page.goto("/?ventana=principal&pantalla=honestidad&plataforma=windows");
  await expect(
    page.getByRole("heading", { name: "Hoy en Windows" }),
  ).toBeVisible();
  await expect(page.getByText("Ya funciona")).toBeVisible();
  // Sprint 008: ninguna función queda «todavía no».
  await expect(page.getByRole("heading", { name: "Todavía no" })).toHaveCount(
    0,
  );
  await expect(
    page.getByRole("heading", { name: "No en Windows por ahora" }),
  ).toBeVisible();
  // La banda sin verificar se dice con su frase, no solo con un color. Desde el sprint 007 lo dicen dos filas: la
  // banda protegida y la videollamada, porque las dos dependen de lo que el usuario compruebe en su PC.
  await expect(
    page.getByText(/sin verificar en Teams, Zoom ni Meet/),
  ).toHaveCount(2);
  // La transcripción: una línea por modelo (sprint 009), cada una con su cifra en este PC y el techo de 4 s. El grande
  // está bajado, medido y en uso; el pequeño, sin bajar. Dentro de su bloque, para que un texto repetido en otra parte
  // de la pantalla no la confunda (la auditoría del S9 cazó ese choque: A1).
  const modelos = page.getByTestId("modelos-de-voz");
  await expect(modelos.locator("p.estado")).toHaveCount(2);
  await expect(
    modelos.getByText(
      /large-v3-turbo: tu pista, 10 s en 2\.?350 ms; la del cliente, 6 s en 2\.?900 ms\. Cabe en el techo de 4 s/,
    ),
  ).toBeVisible();
  await expect(modelos.getByText(/el que transcribe ahora/)).toHaveCount(1);
  await expect(modelos.getByText("small: sin bajar.")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Medir otra vez" }),
  ).toBeVisible();
  // Con un modelo rápido no hay escalón: el botón del modelo pequeño solo aparece si pasa de 4 s.
  await expect(
    page.getByRole("button", { name: /Bajar el modelo pequeño/ }),
  ).toHaveCount(0);
});

test("en Windows, «Ya funciona» lleva la videollamada (S7) y las cinco del S8", async ({
  page,
}) => {
  await page.goto("/?ventana=principal&pantalla=honestidad&plataforma=windows");
  const ya = page
    .locator("section, div")
    .filter({ has: page.getByText("Ya funciona", { exact: true }) })
    .last();
  for (const nombre of [
    "Las videollamadas: el audio del cliente",
    "Leer la pantalla de la reunión",
    "La ficha al oído",
    "Ensayar con la voz que lee las preguntas",
    "Los atajos de teclado globales",
    "Acoplarse a la ventana de la reunión",
    "La sugerencia redactada por IA",
    "El radar de tu equipo",
    "Borrar tus notas con la app cerrada",
    "Abrir tus notas guardadas",
  ]) {
    await expect(ya.getByText(nombre)).toBeVisible();
  }
  // Y nada promete una versión próxima.
  await expect(page.getByText(/versión próxima/i)).toHaveCount(0);
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
  await expect(
    page.getByRole("button", { name: "Abrir Ajustes de Windows" }),
  ).toHaveCount(0);
  await expect(page.getByText("Conceder en macOS")).toHaveCount(0);
  // Sprint 007: el audio del sistema y la pantalla de la reunión funcionan sin ningún permiso; desde el sprint 008, acoplarse
  // tampoco pide ninguno.
  await expect(page.getByText("No pide permiso en Windows")).toHaveCount(3);
  await expect(page.getByText(/versión próxima/i)).toHaveCount(0);
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
  {
    que: "idioma",
    url: "ventana=principal&pantalla=idioma&plataforma=windows",
  },
  {
    que: "honestidad",
    url: "ventana=principal&pantalla=honestidad&plataforma=windows",
  },
  // Sprint 007: el título «Permisos de macOS» y «Mostrar en Finder» los cazó la lectura de las capturas.
  {
    que: "permisos",
    url: "ventana=principal&pantalla=permisos&plataforma=windows",
  },
  { que: "notas", url: "ventana=principal&pantalla=notas&plataforma=windows" },
]) {
  test(`en Windows, «${p.que}» no dice «macOS» ni «Mac»`, async ({ page }) => {
    await page.goto(`/?${p.url}`);
    await expect(page.locator("h1").first()).toBeVisible();
    await expect(page.getByText(/macOS|\bMac\b|Finder/)).toHaveCount(0);
  });
}

/**
 * Hallazgo A6 de la auditoría del S6: Notas y la bandeja prometían que lo guardado se borra «aunque no abras la app», y en
 * Windows esa tarea no existía. **Desde el sprint 008 existe** (el Programador de tareas, ADR 016 enmienda 4): la frase es
 * verdad en los dos sistemas, y sale en los dos.
 */
for (const plataforma of ["", "&plataforma=windows"]) {
  for (const estado of ["archivo", "al-cerrar-bandeja"]) {
    test(`Notas promete el borrado con la app cerrada (${estado}${plataforma ? ", Windows" : ", macOS"})`, async ({
      page,
    }) => {
      await page.goto(
        `/?ventana=principal&pantalla=notas&estado=${estado}${plataforma}`,
      );
      await expect(page.locator("h1").first()).toBeVisible();
      await expect(
        page.getByText(/aunque no abras la app/).first(),
      ).toBeVisible();
    });
  }
}

/** La IA en Windows (sprint 008, ADR 023): sin modelo del sistema, lo dice con la verdad de Windows y no nombra a Apple. */
test("en Windows, IA dice que no hay modelo del sistema y que la sugerencia va con el API", async ({
  page,
}) => {
  await page.goto("/?ventana=principal&pantalla=ia&plataforma=windows");
  await expect(page.locator("h1").first()).toBeVisible();
  await expect(
    page.getByText(/En Windows no hay modelo del sistema/).first(),
  ).toBeVisible();
  await expect(page.getByText(/Apple|macOS|\bMac\b/)).toHaveCount(0);
});

/** El radar en la banda de Windows (sprint 008): dice que te mira un programa de tu PC, con las teclas de Windows y sin
 * nombrar el Mac. (Su tecla, `Alt+Mayús+U`, la comprueban el contrato y `atajos.rs`: la banda compacta no la pinta.) */
test("en Windows, la banda pinta el radar de tu PC", async ({ page }) => {
  await page.setViewportSize({ width: 900, height: 220 });
  await page.goto("/?ventana=banda&estado=radar-invasivo&plataforma=windows");
  await expect(
    page.getByText("Te está mirando un programa de tu PC"),
  ).toBeVisible();
  await expect(page.getByText(/Alt\+Mayús\+Q corta/)).toBeVisible();
  await expect(page.getByText(/⌃|⌥|⎋|macOS|\bMac\b/)).toHaveCount(0);
});

/**
 * **La banda y Sesión en Windows (sprint 007).** La banda no promete invisibilidad donde nadie la verificó (regla dura 6) ni
 * nombra el Mac, y las teclas son las de Windows. El control: en macOS las mismas pantallas siguen diciendo lo de siempre.
 */
const BANDA_SIN_VERIFICAR = "ventana=banda&estado=sin-verificar";

test("en Windows, la banda sin verificar lo dice sin nombrar el Mac y con la tecla de Windows", async ({
  page,
}) => {
  await page.setViewportSize({ width: 900, height: 220 });
  await page.goto(`/?${BANDA_SIN_VERIFICAR}&plataforma=windows`);
  await expect(
    page.getByText(
      "Sin verificar en Teams, Zoom ni Meet · se verifica en tu PC",
    ),
  ).toBeVisible();
  await expect(
    page.getByText(/Verificada solo en Meet|macOS|\bMac\b/),
  ).toHaveCount(0);
  await expect(
    page.getByText(/O corta la banda con Alt\+Mayús\+Q mientras compartes/),
  ).toBeVisible();
  await expect(page.getByText(/⌃|⌥|⎋/)).toHaveCount(0);
});

test("en Windows, la banda con una ficha pinta las teclas de Windows", async ({
  page,
}) => {
  await page.setViewportSize({ width: 900, height: 220 });
  await page.goto("/?ventana=banda&estado=ficha&plataforma=windows");
  await expect(page.locator("kbd", { hasText: "Alt+Mayús+F" })).toBeVisible();
  await expect(page.locator("kbd", { hasText: "Alt+Mayús+C" })).toBeVisible();
  await expect(page.getByText(/⌃|⌥|⎋/)).toHaveCount(0);
});

test("en macOS, la banda sigue con sus teclas y su verificación (control de los dos de arriba)", async ({
  page,
}) => {
  await page.setViewportSize({ width: 900, height: 220 });
  await page.goto(`/?${BANDA_SIN_VERIFICAR}`);
  await expect(
    page.getByText(/Verificada solo en Meet · macOS 26\.6\.2/),
  ).toBeVisible();
  await expect(
    page.getByText(/O corta la banda con ⌥⎋ mientras compartes/),
  ).toBeVisible();
  await expect(page.getByText(/Alt\+Mayús/)).toHaveCount(0);
});

test("en Windows, Sesión no habla del Mac y dice que la protección está sin verificar", async ({
  page,
}) => {
  await page.goto("/?ventana=principal&pantalla=sesion&plataforma=windows");
  await expect(page.locator("h1").first()).toBeVisible();
  await expect(page.getByText(/tu propio PC/)).toBeVisible();
  await expect(
    page.getByText(
      /sin verificar: se verifica con una videollamada real en tu PC/,
    ),
  ).toBeVisible();
  await expect(page.getByText(/macOS|\bMac\b/)).toHaveCount(0);
});

for (const tema of ["dark", "light"] as const) {
  for (const p of [
    {
      que: "banda sin verificar en Windows",
      url: `${BANDA_SIN_VERIFICAR}&plataforma=windows`,
    },
    {
      que: "sesión en Windows",
      url: "ventana=principal&pantalla=sesion&plataforma=windows",
    },
    {
      que: "banda arriba en Windows",
      url: "ventana=banda&estado=ficha&borde=arriba&plataforma=windows",
    },
    {
      que: "banda arriba en Windows · solo audio",
      url: "ventana=banda&estado=voz&borde=arriba&plataforma=windows",
    },
    {
      que: "sesión en Windows · banda bajada",
      url: "ventana=principal&pantalla=sesion&plataforma=windows&desplazamiento=132",
    },
    {
      que: "sesión en Windows · banda cerrada",
      url: "ventana=principal&pantalla=sesion&plataforma=windows&banda=cerrada",
    },
  ]) {
    test(`«${p.que}» · tema ${tema} · sin hallazgos de axe`, async ({
      page,
    }) => {
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

/**
 * **LAS LÍNEAS DE CADA MODELO SE PARTEN, NO DESBORDAN** (sprint 010, visto en vivo en la Surface): con las dos pistas,
 * la línea de un modelo pasa de 140 caracteres y `.estado` es un chip de una línea (`white-space: nowrap` en
 * `ghost.css`). Honestidad sacaba una barra de desplazamiento horizontal y cortaba el final de la frase. ¿Puede fallar?
 * Sí: quitar el `whiteSpace: "normal"` de las líneas de `modelos-de-voz` lo pone rojo, con el ancho que se sale.
 */
test("en Windows, cada línea de modelo cabe en su bloque y Honestidad no se desborda a lo ancho", async ({
  page,
}) => {
  await page.goto("/?ventana=principal&pantalla=honestidad&plataforma=windows");
  const modelos = page.getByTestId("modelos-de-voz");
  await expect(modelos.locator("p.estado")).toHaveCount(2);
  const caja = await modelos.boundingBox();
  for (const linea of await modelos.locator("p.estado").all()) {
    const b = await linea.boundingBox();
    expect(
      b && caja && b.x + b.width <= caja.x + caja.width + 1,
      `una línea de modelo se sale de su bloque`,
    ).toBe(true);
  }
  const desbordados = await page.evaluate(() =>
    [...document.querySelectorAll<HTMLElement>("body *")]
      .filter((e) => {
        const x = getComputedStyle(e).overflowX;
        return (
          (x === "auto" || x === "scroll") && e.scrollWidth > e.clientWidth + 1
        );
      })
      .map(
        (e) =>
          `${e.tagName.toLowerCase()}.${e.className}: ${e.scrollWidth} > ${e.clientWidth}`,
      ),
  );
  expect(desbordados, "Honestidad tiene desplazamiento horizontal").toEqual([]);
});

/**
 * **EL ARNÉS ENTERO, SOBRE EL BUILD REAL, EN WINDOWS, NO HABLA DEL MAC** (sprint 010). El gate de vitest recorre las mismas
 * URLs sobre el código; este las recorre sobre lo que se distribuye: el build servido, con el navegador de verdad. La lista
 * se lee de `scripts/capturar-fidelidad.mjs` (una sola fuente), los encuadres de la banda en el proyecto de la banda y los
 * demás en los de la ventana principal. ¿Puede fallar? Sí: un «Touch ID» en Notas «bandeja-llave» es rojo, con la URL.
 */
const DEL_MAC =
  /macOS|\bMac\b|Llavero|Keychain|Touch ID|Finder|launchd|Time Machine|AirPlay|System Settings|Ajustes del Sistema|Ítems de inicio|Items de inicio|Login Items|Background Items|⌥|⌃/;
const URLS_DEL_ARNES = [
  ...new Set(
    [
      ...readFileSync("scripts/capturar-fidelidad.mjs", "utf8").matchAll(
        /url: "([^"]+)"/g,
      ),
    ].map((m) => m[1]),
  ),
];

test("en Windows, las URLs del arnés de fidelidad no dicen nada del Mac", async ({
  page,
}, info) => {
  const deLaBanda = info.project.name === "banda";
  const mias = URLS_DEL_ARNES.filter(
    (u) => u.includes("ventana=banda") === deLaBanda,
  );
  expect(
    mias.length,
    "el arnés no tiene encuadres para este proyecto",
  ).toBeGreaterThan(5);
  const culpables: string[] = [];
  for (const url of mias) {
    await page.goto(`/?${url}&plataforma=windows`);
    await page.evaluate(() => document.fonts.ready);
    const texto = await page.evaluate(() => document.body.innerText);
    const frase = DEL_MAC.exec(texto);
    if (frase) culpables.push(`${url}: «${frase[0]}»`);
  }
  expect(culpables, "en Windows el build dice del Mac").toEqual([]);
});

/**
 * **LA BANDA A TU ALTURA (sprint 011).** Con la banda arriba, en Windows, trae el asa de altura y el «×»; Sesión dice a qué
 * altura está, cómo volverla arriba y, cerrada, cómo traerla. Todo es de Windows: en macOS no hay ni asa, ni «×», ni filas, y
 * con la banda abajo no hay asa ni fila de altura. Los textos se leen en español; los de inglés los ata `windows-sin-mac`.
 * ¿Puede fallar? Sí: pintar el asa o el «×» sin mirar la plataforma, o dejar de ofrecer «Mostrar la banda», es rojo.
 */
const sinDesbordeAncho = async (page: import("@playwright/test").Page) => {
  const { ancho, visible } = await page.evaluate(() => ({
    ancho: document.documentElement.scrollWidth,
    visible: document.documentElement.clientWidth,
  }));
  expect(ancho, "la banda se sale a lo ancho").toBeLessThanOrEqual(visible);
};

for (const alto of [
  { que: "88 px", estado: "ficha", cerrar: "Cerrar la banda" },
  {
    que: "44 px, solo audio",
    estado: "voz",
    cerrar: "Cerrar la banda y terminar la sesión",
  },
]) {
  test(`en Windows, la banda de arriba (${alto.que}) trae el asa de altura y el «×», y no se desborda`, async ({
    page,
  }) => {
    await page.setViewportSize({ width: 900, height: 220 });
    await page.goto(
      `/?ventana=banda&estado=${alto.estado}&borde=arriba&plataforma=windows`,
    );
    const asa = page.locator(".asa-altura");
    await expect(asa).toHaveCount(1);
    await expect(asa).toHaveAttribute(
      "title",
      "Arrastra para elegir a qué altura queda la banda",
    );
    await expect(page.getByRole("button", { name: alto.cerrar })).toBeVisible();
    await sinDesbordeAncho(page);
  });
}

test("en Windows, con la banda abajo no hay asa de altura, y el «×» sigue", async ({
  page,
}) => {
  await page.setViewportSize({ width: 900, height: 220 });
  await page.goto(
    "/?ventana=banda&estado=ficha&borde=abajo&plataforma=windows",
  );
  await expect(page.locator(".asa-altura")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Cerrar la banda" }),
  ).toBeVisible();
});

test("en macOS, la banda de arriba no trae ni asa de altura ni «×» (control de los de arriba)", async ({
  page,
}) => {
  await page.setViewportSize({ width: 900, height: 220 });
  await page.goto("/?ventana=banda&estado=ficha&borde=arriba");
  await expect(page.locator(".asa-altura")).toHaveCount(0);
  await expect(page.locator(".cerrar-b")).toHaveCount(0);
  await expect(page.locator("[data-con-cerrar]")).toHaveCount(0);
});

test("en Windows, Sesión dice a qué altura está la banda y cómo volverla arriba", async ({
  page,
}) => {
  await page.goto("/?ventana=principal&pantalla=sesion&plataforma=windows");
  await expect(page.getByText("Altura de la banda")).toBeVisible();
  await expect(page.getByText("Pegada al borde de arriba")).toBeVisible();
  await expect(page.getByRole("button", { name: "Volver arriba" })).toHaveCount(
    0,
  );

  await page.goto(
    "/?ventana=principal&pantalla=sesion&plataforma=windows&desplazamiento=132",
  );
  await expect(page.getByText("132 px bajo el borde de arriba")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Volver arriba" }),
  ).toBeVisible();
});

test("en Windows, con la banda cerrada Sesión lo dice y ofrece «Mostrar la banda»", async ({
  page,
}) => {
  await page.goto(
    "/?ventana=principal&pantalla=sesion&plataforma=windows&banda=cerrada",
  );
  await expect(page.getByText("La banda está cerrada")).toBeVisible();
  await expect(
    page.getByRole("button", { name: "Mostrar la banda" }),
  ).toBeVisible();

  await page.goto("/?ventana=principal&pantalla=sesion&plataforma=windows");
  await expect(page.getByText("La banda está cerrada")).toHaveCount(0);
});

test("en macOS, Sesión no habla de la altura ni del cierre de la banda", async ({
  page,
}) => {
  await page.goto(
    "/?ventana=principal&pantalla=sesion&desplazamiento=132&banda=cerrada",
  );
  await expect(page.locator("h1").first()).toBeVisible();
  await expect(page.getByText("Altura de la banda")).toHaveCount(0);
  await expect(page.getByText("La banda está cerrada")).toHaveCount(0);
  await expect(
    page.getByRole("button", { name: "Mostrar la banda" }),
  ).toHaveCount(0);
});
