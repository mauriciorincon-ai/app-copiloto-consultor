/**
 * LAS MUESTRAS DEL BROCHURE (`docs/BROCHURE.html`): capturas de la app CORRIENDO, no dibujos (banco de técnicas del kit §7,
 * veredicto de Innmobiliaria: «interfaces que nada que ver con la app real»). Sirve el build, abre cada estado con el
 * navegador en el idioma y el tema pedidos, lo pasa a WebP con el propio navegador y **reescribe en el HTML** el `src` o el
 * `srcset` de cada elemento que lleva `data-muestra="<estado>--<tema>--<idioma>"`. Re-ejecutable: el HTML sigue siendo un
 * solo archivo autocontenido (el brochure abre con doble clic y sin internet).
 *
 * Todo es sintético: los clientes y las fichas son los del kit de muestra (Páramo Azul, Sur del Valle).
 *
 * Uso: pnpm build && node scripts/capturar-brochure.mjs     (puerto 4186, jamás el 3000)
 */
import { chromium } from "@playwright/test";
import { spawn } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";

const HTML = resolve("docs/BROCHURE.html");
const PUERTO = 4186;

/** Qué abre cada muestra y a qué tamaño. La banda se recorta a su alto real; el cuaderno entra entero. */
const ESTADOS = {
  "banda-ficha": { url: "ventana=banda&estado=ficha", ancho: 1180, alto: 88 },
  "notas-al-cerrar": { url: "ventana=principal&pantalla=notas&estado=al-cerrar", ancho: 960, alto: 640 },
  "sesion-windows": { url: "ventana=principal&pantalla=sesion&plataforma=windows", ancho: 960, alto: 640 },
};
const CALIDAD = 0.72;

const html = readFileSync(HTML, "utf8");
const pedidas = [...new Set([...html.matchAll(/data-muestra="([^"]+)"/g)].map((m) => m[1]))];
if (pedidas.length === 0) throw new Error("el HTML no pide ninguna muestra (data-muestra)");

const servidor = spawn("pnpm", ["preview", "--port", String(PUERTO), "--strictPort"], { stdio: "ignore" });
const esperar = async () => {
  for (let i = 0; i < 60; i++) {
    try {
      if ((await fetch(`http://localhost:${PUERTO}/`)).ok) return;
    } catch {}
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error("el servidor del build no arrancó");
};

try {
  await esperar();
  const navegador = await chromium.launch();
  const resultado = new Map();
  for (const id of pedidas) {
    const [estado, tema, idioma] = id.split("--");
    const e = ESTADOS[estado];
    if (!e || !["dark", "light"].includes(tema) || !["es", "en"].includes(idioma)) throw new Error(`muestra desconocida: ${id}`);
    const contexto = await navegador.newContext({
      locale: idioma === "en" ? "en-US" : "es-CO",
      colorScheme: tema,
      reducedMotion: "reduce",
    });
    const pagina = await contexto.newPage();
    await pagina.setViewportSize({ width: e.ancho, height: e.alto });
    await pagina.goto(`http://localhost:${PUERTO}/?${e.url}`);
    await pagina.evaluate(() => document.fonts.ready);
    const png = await pagina.screenshot();
    await pagina.close();
    // El navegador codifica el WebP: no hay dependencia nueva y sale lo mismo en cualquier máquina con Chromium.
    const lienzo = await contexto.newPage();
    const uri = await lienzo.evaluate(
      async ([base64, calidad]) => {
        const img = new Image();
        await new Promise((ok, mal) => {
          img.onload = ok;
          img.onerror = mal;
          img.src = `data:image/png;base64,${base64}`;
        });
        const c = document.createElement("canvas");
        c.width = img.width;
        c.height = img.height;
        c.getContext("2d").drawImage(img, 0, 0);
        return c.toDataURL("image/webp", calidad);
      },
      [png.toString("base64"), CALIDAD],
    );
    await contexto.close();
    if (!uri.startsWith("data:image/webp")) throw new Error(`el navegador no codificó WebP para ${id}`);
    resultado.set(id, uri);
    console.log(`${id}: ${(uri.length / 1024).toFixed(1)} KB`);
  }
  await navegador.close();

  const nuevo = html.replace(
    /(data-muestra="([^"]+)"[^>]*?\s(?:src|srcset)=")[^"]*(")/g,
    (_m, antes, id, despues) => `${antes}${resultado.get(id)}${despues}`,
  );
  writeFileSync(HTML, nuevo);
  const total = [...resultado.values()].reduce((s, u) => s + u.length, 0);
  console.log(`${resultado.size} muestras escritas en docs/BROCHURE.html · ${(total / 1024).toFixed(0)} KB`);
} finally {
  servidor.kill();
}
