import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * EL BROCHURE (`docs/BROCHURE.html`, orden `ENTREGA-brochure-orden.md`, sprint 008): una página que se recorre como un sitio
 * web, de lo general a lo particular. Este archivo comprueba lo que se puede comprobar sin un navegador de verdad —el
 * conteo contra el manual en los DOS idiomas, que sea autocontenido, que obedezca al design system y que no prometa lo vetado—;
 * lo que se ve (movimiento reducido, árbol de accesibilidad, axe, desbordes) lo mira `tests/e2e/brochure.spec.ts`.
 *
 * ¿Puede fallar? Sí: quitar una `.feature`, cambiar un título por uno que el manual no tiene, dejar un texto sin su
 * traducción, colar un `http://` o un token del tema fuera de los de la app pone una de estas pruebas en rojo con el nombre
 * de lo que no cuadra.
 */
const RUTA = "docs/BROCHURE.html";
const html = readFileSync(RUTA, "utf8");
const doc = new DOMParser().parseFromString(html, "text/html");

/** Las secciones `###` de «## Features» de un manual, sin su origen («· Nuevo · Sprint 005», «· since Sprint 001»). */
function seccionesDe(
  ruta: string,
  features: string,
  siguiente: RegExp,
): string[] {
  const texto = readFileSync(ruta, "utf8");
  const desde = texto.indexOf(features);
  expect(desde, `${ruta} no tiene «${features.trim()}»`).toBeGreaterThan(-1);
  const resto = texto.slice(desde + features.length);
  const hasta = resto.search(siguiente);
  return [...resto.slice(0, hasta).matchAll(/^### (.+)$/gm)].map((m) =>
    m[1].split(" · ")[0].trim(),
  );
}
const MANUAL_ES = seccionesDe(
  "docs/MANUAL-DE-USO.md",
  "\n## Features\n",
  /^## /m,
);
const MANUAL_EN = seccionesDe("docs/MANUAL-EN.md", "\n## Features\n", /^## /m);

const features = [...doc.querySelectorAll(".feature")];
const texto = (el: Element, idioma: "es" | "en") =>
  [...el.querySelectorAll(`[lang="${idioma}"]`)]
    .map((x) => x.textContent?.trim() ?? "")
    .join(" ");

describe("el brochure cubre las funcionalidades del manual, todas y en los dos idiomas", () => {
  it("hay una .feature por sección «###» de Features del manual (su cuenta es el N): agrupar sí, omitir jamás", () => {
    expect(MANUAL_ES.length, "el manual no tiene funcionalidades").toBeGreaterThan(0);
    expect(features.length).toBe(MANUAL_ES.length);
    const es = features.map((f) => texto(f.querySelector("h4")!, "es"));
    const en = features.map((f) => texto(f.querySelector("h4")!, "en"));
    expect([...es].sort(), "los títulos en español son los del manual").toEqual(
      [...MANUAL_ES].sort(),
    );
    expect(
      [...en].sort(),
      "los títulos en inglés son los del manual en inglés",
    ).toEqual([...MANUAL_EN].sort());
    expect(
      new Set(features.map((f) => f.getAttribute("data-id"))).size,
      "dos .feature con el mismo id",
    ).toBe(features.length);
  });

  it("el pie declara el mismo N, y el contador lo dice completo desde el primer byte", () => {
    const cuenta = doc.getElementById("conteo")!;
    expect(Number(cuenta.getAttribute("data-n"))).toBe(features.length);
    expect(cuenta.textContent).toBe(String(features.length));
    const dicho = cuenta.parentElement!.querySelector(".sr")!;
    expect(dicho.textContent, "el texto accesible del contador").toBe(
      String(features.length),
    );
  });

  it("una tarjeta por grupo del export, con las .feature de cada grupo, y entre todas suman las del manual", () => {
    const exportado = JSON.parse(readFileSync("docs/brochure-export.json", "utf8")) as {
      funcionalidades: { grupos: { features: unknown[] }[] };
    };
    const tarjetas = [...doc.querySelectorAll(".tarjeta")];
    expect(tarjetas.length).toBe(exportado.funcionalidades.grupos.length);
    expect(tarjetas.map((t) => t.querySelectorAll(".feature").length)).toEqual(
      exportado.funcionalidades.grupos.map((g) => g.features.length),
    );
    expect(
      tarjetas.reduce((n, t) => n + t.querySelectorAll(".feature").length, 0),
    ).toBe(MANUAL_ES.length);
  });

  it("todo texto tiene su par: cada elemento con lang=es tiene su lang=en en el mismo sitio", () => {
    const sin: string[] = [];
    for (const es of doc.body.querySelectorAll('[lang="es"]')) {
      const par = es.nextElementSibling;
      const ok =
        par?.getAttribute("lang") === "en" ||
        es.previousElementSibling?.getAttribute("lang") === "en";
      if (!ok) sin.push(`${es.tagName}: ${(es.textContent ?? "").slice(0, 50)}`);
    }
    expect(sin, "textos en español sin su traducción").toEqual([]);
    expect(doc.body.querySelectorAll('[lang="en"]').length).toBe(
      doc.body.querySelectorAll('[lang="es"]').length,
    );
  });

  it("el export de CV Viva y el brochure cuentan lo mismo", () => {
    const exportado = JSON.parse(
      readFileSync("docs/brochure-export.json", "utf8"),
    ) as {
      funcionalidades: {
        total: number;
        grupos: {
          nombre: string;
          linea: string;
          features: { nombre: string }[];
        }[];
      };
    };
    expect(exportado.funcionalidades.total).toBe(features.length);
    const tarjetas = [...doc.querySelectorAll(".tarjeta")];
    expect(exportado.funcionalidades.grupos.length).toBe(tarjetas.length);
    exportado.funcionalidades.grupos.forEach((g, i) => {
      expect(
        texto(tarjetas[i].querySelector(".titulo")!, "es"),
        `la tarjeta ${i + 1}`,
      ).toBe(g.nombre);
      expect(texto(tarjetas[i].querySelector(".linea")!, "es")).toBe(g.linea);
      expect(g.features.map((f) => f.nombre)).toEqual(
        [...tarjetas[i].querySelectorAll(".feature h4")].map((h) =>
          texto(h, "es"),
        ),
      );
    });
  });
});

describe("el brochure es autocontenido y no ofrece nada que no sea suyo", () => {
  it("cero red y cero enlaces: ni una URL, ni un script, ni una hoja de estilos de fuera", () => {
    expect(html).not.toMatch(/https?:\/\/|www\.|\/\/[a-z0-9-]+\.[a-z]{2,}\//i);
    expect(html).not.toMatch(
      /vercel[.]app|workers[.]dev|pages[.]dev|github[.]io/,
    );
    expect(
      doc.querySelectorAll("script[src], link, iframe, embed, object, form"),
    ).toHaveLength(0);
    expect(html).not.toMatch(
      /\bfetch\s*\(|XMLHttpRequest|WebSocket|sendBeacon|@import|\bimport\s*\(/,
    );
    // Los únicos href son fragmentos del sprite de iconos (<use href="#i-…">): ningún enlace de verdad.
    const hrefs = [...doc.querySelectorAll("[href], [src], [srcset]")]
      .flatMap((e) => ["href", "src", "srcset"].map((a) => e.getAttribute(a)))
      .filter((v): v is string => !!v)
      .filter((v) => !v.startsWith("#") && !v.startsWith("data:"));
    expect(hrefs, "href, src o srcset que no son del archivo").toEqual([]);
    expect(doc.querySelectorAll("a")).toHaveLength(0);
  });

  it("las muestras son capturas de la app (WebP incrustado), tres estados en dos temas y dos idiomas", () => {
    const pedidas = [...doc.querySelectorAll("[data-muestra]")].map((e) =>
      e.getAttribute("data-muestra")!,
    );
    // Los estados que el script sabe capturar, por sus llaves; cada uno se pide en dos temas y dos idiomas, sin repetirse.
    const guion = readFileSync("scripts/capturar-brochure.mjs", "utf8");
    const delGuion = [...guion.matchAll(/^\s*"([a-z-]+)": \{ url:/gm)].map((m) => m[1]);
    expect(delGuion.length, "el script no declara estados").toBeGreaterThan(0);
    expect(new Set(pedidas).size, "una muestra repetida").toBe(pedidas.length);
    expect(pedidas.length).toBe(delGuion.length * 2 * 2);
    const estados = new Set(pedidas.map((p) => p.split("--")[0]));
    expect([...estados].sort()).toEqual([...delGuion].sort());
    for (const e of estados)
      expect(guion, `el script no sabe capturar «${e}»`).toContain(`"${e}"`);
    for (const el of doc.querySelectorAll("[data-muestra]")) {
      const dato = el.getAttribute("src") ?? el.getAttribute("srcset") ?? "";
      expect(
        dato.startsWith("data:image/webp;base64,"),
        `${el.getAttribute("data-muestra")} sin su captura`,
      ).toBe(true);
      expect(
        dato.length,
        `${el.getAttribute("data-muestra")} vacía`,
      ).toBeGreaterThan(2000);
    }
  });

  it("sin emojis (el design system los prohíbe) y con los iconos del sprite de la maqueta", () => {
    expect(html).not.toMatch(/\p{Extended_Pictographic}/u);
    const sprite = readFileSync("docs/diseno/assets/iconos.js", "utf8");
    // Se comparan los símbolos ya parseados: así no importa cómo cada archivo cierra sus etiquetas.
    const maqueta = new DOMParser().parseFromString(
      sprite.match(/`(<svg[\s\S]*?<\/svg>)`/)![1],
      "text/html",
    );
    const delSprite = new Map(
      [...maqueta.querySelectorAll("symbol")].map((x) => [x.id, x.innerHTML]),
    );
    const propios = [...doc.querySelectorAll("symbol")];
    expect(propios.length).toBeGreaterThan(5);
    for (const s of propios)
      expect(s.innerHTML, `el icono ${s.id} no es el de la maqueta`).toBe(
        delSprite.get(s.id),
      );
    for (const u of doc.querySelectorAll("use"))
      expect(
        doc.getElementById(u.getAttribute("href")!.slice(1)),
        u.outerHTML,
      ).not.toBeNull();
  });
});

describe("el brochure obedece el design system de la app", () => {
  /** Los tokens de un bloque de CSS, nombre → valor. */
  const tokens = (css: string) =>
    new Map([...css.matchAll(/(--[a-z0-9-]+):\s*([^;]+);/g)].map((m) => [m[1], m[2].trim()]));
  const app = readFileSync("src/index.css", "utf8");
  const oscuroApp = tokens(
    app.slice(app.indexOf(':root, html[data-theme="dark"]'), app.indexOf('html[data-theme="light"]')),
  );
  const claroApp = tokens(
    app.slice(app.indexOf('html[data-theme="light"] {'), app.indexOf("@theme inline")),
  );
  const estilo = doc.querySelector("style")!.textContent!;
  const iClaro = estilo.indexOf("@media (prefers-color-scheme: light)");
  const oscuro = tokens(estilo.slice(0, iClaro));
  const claro = tokens(estilo.slice(iClaro, estilo.indexOf("* { box-sizing")));
  const DE_COLOR = ["--bg", "--surface", "--surface-2", "--line", "--line-2", "--ink", "--ink-2", "--ink-3", "--halo", "--halo-tint", "--ok", "--ok-tint", "--warn", "--warn-tint", "--err", "--err-tint", "--mute"];

  it("sus colores son los tokens de la app, token por token, en los dos temas (oscuro primario)", () => {
    for (const t of DE_COLOR) {
      expect(oscuro.get(t), `${t} oscuro`).toBe(oscuroApp.get(t));
      expect(claro.get(t), `${t} claro`).toBe(claroApp.get(t));
    }
  });

  it("las tipografías, los radios y la curva son los de la app", () => {
    const deLaApp = tokens(app.slice(0, app.indexOf(':root, html[data-theme="dark"]')));
    for (const t of ["--r-chip", "--r-ficha", "--r-panel", "--r-tecla", "--ease"])
      expect(oscuro.get(t), t).toBe(deLaApp.get(t));
    // Las familias de la app van todas, y en su orden; el brochure se abre también en un PC, así que suma las de Windows.
    for (const t of ["--font-evidencia", "--font-ui", "--font-mono"]) {
      const familias = (v: string) => v.split(",").map((f) => f.trim());
      const propias = familias(oscuro.get(t)!);
      const quedan = familias(deLaApp.get(t)!).filter((f) => !propias.includes(f));
      expect(quedan, `${t}: familias de la app que faltan`).toEqual([]);
    }
  });

  it("el tema oscuro es el de fábrica y el claro entra por prefers-color-scheme (no puede seguir el selector de la app)", () => {
    expect(estilo.indexOf("color-scheme: dark")).toBeLessThan(iClaro);
    expect(doc.querySelector('meta[name="color-scheme"]')?.getAttribute("content")).toBe("dark light");
    expect(estilo).not.toMatch(/data-theme/);
  });

  it("ningún texto usa --ink-3, que el design system reserva al ornamento", () => {
    expect(estilo.match(/[^-]color:\s*var\(--ink-3\)/g) ?? []).toEqual([]);
  });
});

describe("el brochure solo dice lo real, con el vocabulario del producto", () => {
  const visible = doc.body.textContent!.replace(/\s+/g, " ");

  it("declara en su cabecera que es INICIAL y que se sella con el gate del H-1", () => {
    expect(texto(doc.querySelector(".barra")!, "es")).toMatch(/inicial.*H-1/i);
    expect(texto(doc.querySelector(".barra")!, "en")).toMatch(/first version.*H-1/i);
    const export_ = JSON.parse(readFileSync("docs/brochure-export.json", "utf8")) as { app: { estado: string } };
    expect(export_.app.estado).toBe("inicial");
  });

  it("incluye Windows y no promete lo que nadie ha verificado", () => {
    expect(visible).toMatch(/Windows/);
    expect(texto(doc.querySelector(".e5")!, "es")).toMatch(/en el gate del hito/);
    // La protección de la captura, graduada por plataforma (regla dura 6): verificada solo donde la corrida en vivo lo comprobó.
    expect(visible).toMatch(/verificado en Google Meet sobre macOS 26\.6\.2/);
    expect(visible).toMatch(/verified in Google Meet on macOS 26\.6\.2/);
  });

  it("nada de roadmap ni de lo que la visión marca como futuro", () => {
    for (const futuro of ["MLX", "iPhone", "embeddings", "próximamente", "coming soon", "roadmap"])
      expect(visible.toLowerCase(), futuro).not.toContain(futuro.toLowerCase());
  });

  it("el llamado a la acción es «lista de espera», sin enlace, y no hay canal de descarga", () => {
    const espera = doc.querySelector(".espera")!;
    expect(texto(espera, "es")).toMatch(/Lista de espera/);
    expect(texto(espera, "en")).toMatch(/Waiting list/);
    expect(espera.querySelectorAll("a, button")).toHaveLength(0);
    expect(visible).not.toMatch(/descargar (la app|el instalador)|download the (app|installer)/i);
  });

  it("las cifras de «lo fino» salen del disco: funcionalidades del manual, ADR de decisions/, y 0 B", () => {
    const filas = [...doc.querySelectorAll(".cifras dd")].map((d) => d.textContent!.trim());
    const adrs = existsSync("decisions") ? (readFileSync("package.json", "utf8"), 0) : 0;
    void adrs;
    expect(filas[0]).toBe(String(MANUAL_ES.length));
    expect(filas[2]).toBe("0");
    // El número de ADR lo cuenta tests/unit/brochure-export.test.ts contra decisions/; aquí, que el brochure dice el mismo.
    const exportado = JSON.parse(readFileSync("docs/brochure-export.json", "utf8")) as { metricas: { clave: string; valor: number }[] };
    expect(Number(filas[1])).toBe(exportado.metricas.find((m) => m.clave === "decisiones_registradas")!.valor);
  });
});

describe("el movimiento: solo lo declarado, y con movimiento reducido la pieza entera", () => {
  const estilo = doc.querySelector("style")!.textContent!;

  it("el bloque de movimiento reducido existe y deja visible lo que el otro anima", () => {
    const reducido = estilo.slice(estilo.indexOf("@media (prefers-reduced-motion: reduce)"));
    expect(reducido).toMatch(/\.revela \{ opacity: 1; transform: none; \}/);
    expect(reducido).toMatch(/\.escribe \{ clip-path: none; \}/);
    expect(reducido).toMatch(/\.libro \.raya \{ transform: scaleX\(1\)/);
    expect(reducido).toMatch(/\.palabra[^{]*\{ animation: none; filter: none; \}/);
  });

  it("lo oculto por animación vive bajo `.js` o dentro del bloque sin preferencia: sin JavaScript todo se ve", () => {
    const sinPreferencia = estilo.slice(
      estilo.indexOf("@media (prefers-reduced-motion: no-preference)"),
      estilo.indexOf("@media (prefers-reduced-motion: reduce)"),
    );
    for (const regla of sinPreferencia.matchAll(/([^{}]+)\{[^{}]*opacity: 0[^{}]*\}/g)) {
      const selector = regla[1].trim();
      if (/^@keyframes/.test(selector) || /^[0-9%]+$/.test(selector)) continue;
      // Un selector de keyframes (`from`) no es una regla; el resto debe empezar por `.js `, ser un pseudo (::after) o un hijo de la tarjeta.
      expect(selector, `«${selector}» esconde algo sin depender de .js`).toMatch(/^\.js |::after|\.detalle-cuerpo|^from$/);
    }
    expect(html).toMatch(/<noscript><style>[\s\S]*\.detalle[\s\S]*<\/style><\/noscript>/);
  });

  it("el titular, que es el candidato LCP, nunca nace con opacidad 0", () => {
    const enfocar = estilo.match(/@keyframes enfocar \{([^}]*\{[^}]*\}[^}]*)\}/)?.[0] ?? "";
    expect(enfocar).toContain("filter: blur");
    expect(enfocar).not.toMatch(/opacity/);
  });

  it("el patrón de la tarjeta es <h3><button>, y lo cerrado sale del árbol de accesibilidad", () => {
    for (const t of doc.querySelectorAll(".tarjeta")) {
      const cabeza = t.querySelector(".tarjeta-cabeza")!;
      expect(cabeza.tagName).toBe("H3");
      const boton = cabeza.firstElementChild!;
      expect(boton.tagName).toBe("BUTTON");
      expect(boton.getAttribute("aria-expanded")).toBe("false");
      const region = doc.getElementById(boton.getAttribute("aria-controls")!)!;
      expect(region.getAttribute("role")).toBe("region");
      expect(region.getAttribute("aria-labelledby")).toBe(boton.id);
    }
    expect(estilo).toMatch(/\.detalle-interior \{[^}]*visibility: hidden/);
    expect(estilo).toMatch(/\.tarjeta\[data-abierta\] \.detalle-interior \{ visibility: visible; \}/);
  });
});
