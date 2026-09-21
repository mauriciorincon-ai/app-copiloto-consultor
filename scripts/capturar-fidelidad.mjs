/**
 * GATE DE FIDELIDAD (sprint 001, fase 1) — la banda construida contra la maqueta.
 *
 * Captura el MISMO encuadre dos veces: la banda de `docs/diseno/banda.html` (la referencia que el
 * usuario aprobó en la mirada 11) y la banda del PRODUCTO (React + Tauri, servida desde el build),
 * en los dos temas y los dos idiomas. Deja las imágenes en `docs/fidelidad/s1/` y una hoja de
 * contacto para mirarlas en pareja.
 *
 * QUÉ PRUEBA Y QUÉ NO — y esto importa más que las imágenes:
 *   · SÍ prueba que el producto dibuja lo mismo que la maqueta: mismas clases, mismo CSS (es el
 *     mismo archivo), mismo copy, en los cuatro cruces de tema e idioma.
 *   · NO prueba que la ventana nativa esté protegida de la captura. **No puede**: una ventana
 *     protegida no sale en una captura de pantalla, ese es justamente el punto. Lo que se
 *     fotografía aquí es el webview, no la ventana. Verificar la protección en una llamada real
 *     es una parada ⭐ del gate de prueba, y el summary lo dice sin adornos.
 *
 * El tema y el idioma NO se fuerzan por parámetro: se emulan los del sistema
 * (`prefers-color-scheme`, `navigator.language`), que es el camino que el código recorre de
 * verdad. Un gate que prueba un atajo no prueba el producto.
 *
 * Uso: pnpm build && node scripts/capturar-fidelidad.mjs
 */
import { chromium } from "@playwright/test";
import { spawn } from "node:child_process";
import { mkdirSync, rmSync, writeFileSync, readdirSync, readFileSync } from "node:fs";
import { resolve, join } from "node:path";
import process from "node:process";

const RAIZ = resolve(".");
const MAQUETA = join(RAIZ, "docs/diseno/banda.html");
const SALIDA = join(RAIZ, "docs/fidelidad/s1");
const PUERTO = 4180;
const ANCHO = 1180; // el escritorio de referencia de la maqueta: así las dos imágenes se comparan

/** Los diez encuadres de `banda.html`, con su traducción a las props del producto. */
const ENCUADRES = [
  { id: "esperando", alto: 88, url: "estado=esperando" },
  { id: "buscando", alto: 88, url: "estado=buscando" },
  { id: "ficha", alto: 88, url: "estado=ficha" },
  { id: "ficha-2", alto: 200, url: "estado=ficha&ampliada=1" },
  { id: "sin-resultado", alto: 88, url: "estado=sin-resultado" },
  { id: "sin-resultado-2", alto: 200, url: "estado=sin-resultado&ampliada=1" },
  { id: "sin-verificar", alto: 88, url: "estado=sin-verificar&verificado=0" },
  { id: "sin-verificar-2", alto: 200, url: "estado=sin-verificar&verificado=0&ampliada=1" },
  { id: "transcript", alto: 200, url: "estado=ficha&transcript=1" },
  { id: "flotante", alto: 88, url: "estado=ficha&acoplada=0" },
];
const TEMAS = ["dark", "light"];
const IDIOMAS = [
  { id: "es", locale: "es-CO" },
  { id: "en", locale: "en-US" },
];

console.log("── árbol del arnés ──────────────────────────────────────────");
console.log(`   referencia: ${MAQUETA}`);
console.log(`   producto  : http://localhost:${PUERTO}  (desde dist/, no desde el dev server)`);
console.log(`   escribe   : ${SALIDA}`);
console.log("─────────────────────────────────────────────────────────────\n");

rmSync(SALIDA, { recursive: true, force: true });
mkdirSync(join(SALIDA, "maqueta"), { recursive: true });
mkdirSync(join(SALIDA, "producto"), { recursive: true });

const servidor = spawn("pnpm", ["preview", "--port", String(PUERTO), "--strictPort"], {
  cwd: RAIZ,
  stdio: "ignore",
});
const parar = () => servidor.kill("SIGTERM");
process.on("exit", parar);
process.on("SIGINT", () => { parar(); process.exit(130); });

async function esperarServidor() {
  for (let i = 0; i < 60; i++) {
    try {
      const r = await fetch(`http://localhost:${PUERTO}/`);
      if (r.ok) return;
    } catch { /* todavía no levanta */ }
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error("`pnpm preview` no levantó: ¿corriste `pnpm build`?");
}
await esperarServidor();

const navegador = await chromium.launch();

// El ancho de la referencia no se supone: se MIDE. La banda de la maqueta vive dentro del
// escritorio de referencia y los bordes de ese marco le quitan un par de píxeles; capturar el
// producto a 1180 y la maqueta a 1178 haría que la comparación fallara por una razón que no
// tiene nada que ver con la fidelidad — y peor, se habría leído como que sí la tiene.
const regla = await navegador.newPage();
await regla.setViewportSize({ width: ANCHO + 80, height: 900 });
await regla.goto(`file://${MAQUETA}`);
const ANCHO_REAL = Math.round((await (await regla.$(".banda")).boundingBox()).width);
await regla.close();
console.log(`   ancho de la referencia, medido: ${ANCHO_REAL} px\n`);

const errores = [];
const desbordes = [];
let hechas = 0;

for (const tema of TEMAS) {
  for (const idioma of IDIOMAS) {
    const ctx = await navegador.newContext({
      viewport: { width: ANCHO_REAL, height: 400 },
      colorScheme: tema === "dark" ? "dark" : "light",
      locale: idioma.locale,
      deviceScaleFactor: 1,
    });
    const pag = await ctx.newPage();
    pag.on("pageerror", (e) => errores.push(`producto ${tema}/${idioma.id}: ${e.message}`));
    pag.on("console", (m) => { if (m.type() === "error") errores.push(`producto ${tema}/${idioma.id}: ${m.text()}`); });

    // ---- LA MAQUETA ----
    await pag.goto(`file://${MAQUETA}`);
    for (const e of ENCUADRES) {
      await pag.evaluate(([estado, t, l, sel]) => {
        document.documentElement.dataset.estado = estado;
        document.documentElement.dataset.theme = t;
        document.documentElement.dataset.lang = l;
        window.__mqApply();
        // TODO el chrome de la sala de diseño fuera del encuadre. La barra de estados es
        // `position: sticky`: al desplazarse para fotografiar la banda ampliada se le montaba
        // encima y la referencia del gate salía con media banda tapada por botones.
        for (const el of document.querySelectorAll(sel)) el.style.display = "none";
        window.scrollTo(0, 0);
      }, [e.id, tema, idioma.id, ".mq-bar, .mq-nota, .mq-choque, .mq-corte, .mq-etiqueta, .mq-tabla-pos, .mq-hero, .mq-grupo"]);
      const visibles = await pag.$$(".banda");
      let pintada = null;
      for (const el of visibles) if (await el.isVisible()) { pintada = el; break; }
      if (!pintada) { errores.push(`maqueta ${e.id}: ningún .banda visible`); continue; }
      await pintada.screenshot({ path: join(SALIDA, "maqueta", `${e.id}--${tema}--${idioma.id}.png`) });
    }

    // ---- EL PRODUCTO ----
    for (const e of ENCUADRES) {
      await pag.setViewportSize({ width: ANCHO_REAL, height: e.alto });
      await pag.goto(`http://localhost:${PUERTO}/?ventana=banda&${e.url}`);
      await pag.waitForSelector("section.banda");
      const banda = await pag.$("section.banda");

      const fuera = await pag.evaluate(() => {
        const out = [];
        for (const el of document.querySelectorAll(".banda, .banda .cuerpo-b, .banda .lado-b, .banda .ficha-b")) {
          const dv = el.scrollHeight - el.clientHeight;
          const dh = el.scrollWidth - el.clientWidth;
          if (dv > 1 || dh > 1) out.push(`${el.className.split(" ").join(".")} alto +${dv}px ancho +${dh}px`);
        }
        return out;
      });
      for (const f of fuera) desbordes.push(`${e.id} · ${tema} · ${idioma.id}  ${f}`);

      await banda.screenshot({ path: join(SALIDA, "producto", `${e.id}--${tema}--${idioma.id}.png`) });
      hechas++;
    }
  }
}
// ---- LA COMPARACIÓN ----
// Las dos imágenes salen del MISMO Chromium, con el MISMO CSS, a la MISMA anchura: si el
// producto es fiel, deben coincidir píxel a píxel. Comparar de verdad, y no «a ojo sobre una
// hoja de contacto», es lo que convierte esto en un gate: un ojo cansado aprueba una banda
// desplazada 3 px, y esa banda ya no obedece a la maqueta.
//
// La comparación se hace DENTRO del navegador (canvas), que ya está abierto: así no entra una
// dependencia nueva solo para restar dos mapas de bits.
const lienzo = await navegador.newPage();
const diferencias = [];
for (const e of ENCUADRES) {
  for (const t of TEMAS) {
    for (const l of IDIOMAS) {
      const n = `${e.id}--${t}--${l.id}.png`;
      const a = readFileSync(join(SALIDA, "maqueta", n)).toString("base64");
      const b = readFileSync(join(SALIDA, "producto", n)).toString("base64");
      const r = await lienzo.evaluate(async ([uno, dos]) => {
        const carga = (b64) =>
          new Promise((ok, mal) => {
            const i = new Image();
            i.onload = () => ok(i);
            i.onerror = mal;
            i.src = "data:image/png;base64," + b64;
          });
        const [ia, ib] = await Promise.all([carga(uno), carga(dos)]);
        if (ia.width !== ib.width || ia.height !== ib.height) {
          return { medida: `${ia.width}x${ia.height} vs ${ib.width}x${ib.height}` };
        }
        const pinta = (img) => {
          const c = document.createElement("canvas");
          c.width = img.width;
          c.height = img.height;
          c.getContext("2d").drawImage(img, 0, 0);
          return c.getContext("2d").getImageData(0, 0, img.width, img.height).data;
        };
        const [pa, pb] = [pinta(ia), pinta(ib)];
        let distintos = 0;
        for (let i = 0; i < pa.length; i += 4) {
          // Tolerancia por canal: el antialias de una misma fuente puede variar un punto.
          if (
            Math.abs(pa[i] - pb[i]) > 8 ||
            Math.abs(pa[i + 1] - pb[i + 1]) > 8 ||
            Math.abs(pa[i + 2] - pb[i + 2]) > 8
          ) {
            distintos++;
          }
        }
        return { porcentaje: (distintos / (pa.length / 4)) * 100 };
      }, [a, b]);
      diferencias.push({ encuadre: `${e.id} · ${t} · ${l.id}`, ...r });
    }
  }
}
await navegador.close();
parar();

// ---- hoja de contacto ----
const filas = ENCUADRES.flatMap((e) =>
  TEMAS.flatMap((t) =>
    IDIOMAS.map((l) => {
      const n = `${e.id}--${t}--${l.id}.png`;
      return `  <figure data-tema="${t}" data-idioma="${l.id}">
    <figcaption><b>${e.id}</b> · ${t} · ${l.id} · ${e.alto} px</figcaption>
    <div class="par">
      <div><span class="et">maqueta</span><img src="s1/maqueta/${n}" alt="banda ${e.id} en la maqueta, tema ${t}, idioma ${l.id}"></div>
      <div><span class="et">producto</span><img src="s1/producto/${n}" alt="banda ${e.id} en el producto, tema ${t}, idioma ${l.id}"></div>
    </div>
  </figure>`;
    }),
  ),
).join("\n");

writeFileSync(
  join(RAIZ, "docs/fidelidad/S1-banda.html"),
  `<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Angel Ghost · Gate de fidelidad S1 — la banda</title>
<style>
  body { margin: 0; padding: 22px; background: #0d0e10; color: #cfd3d8; font: 14px/1.6 "Avenir Next", system-ui, sans-serif; }
  h1 { font-family: Charter, Georgia, serif; font-size: 26px; color: #e5e7eb; margin: 0 0 6px; }
  p { max-width: 900px; color: #9aa2ad; margin: 0 0 10px; }
  p b { color: #e5e7eb; }
  .aviso { border-left: 3px solid #f0b64a; background: #1a1400; padding: 10px 14px; margin: 16px 0 24px; max-width: 900px; }
  figure { margin: 0 0 26px; }
  figcaption { font-family: Menlo, monospace; font-size: 11px; letter-spacing: .06em; text-transform: uppercase; color: #7dd3fc; margin-bottom: 6px; }
  figcaption b { color: #e5e7eb; }
  .par > div { margin-bottom: 6px; }
  .et { display: inline-block; font-family: Menlo, monospace; font-size: 10.5px; color: #6b7280; margin-bottom: 2px; }
  img { display: block; width: 100%; max-width: ${ANCHO}px; border: 1px solid #2b2f37; }
  .filtros { position: sticky; top: 0; background: #14161a; margin: -22px -22px 20px; padding: 10px 22px; border-bottom: 1px solid #23262c; display: flex; gap: 8px; align-items: center; font-size: 12px; }
  .filtros button { font: inherit; color: #cfd3d8; background: #1c1f25; border: 1px solid #2b2f37; border-radius: 5px; padding: 3px 9px; cursor: pointer; }
  .filtros button[aria-pressed="true"] { background: #7dd3fc; border-color: #7dd3fc; color: #0b1016; font-weight: 600; }
</style>
</head>
<body>
<div class="filtros" role="toolbar" aria-label="Filtros">
  <span>ver</span>
  <button data-f="todo" aria-pressed="true">todo</button>
  <button data-f="dark">oscuro</button>
  <button data-f="light">claro</button>
  <button data-f="es">es</button>
  <button data-f="en">en</button>
</div>
<h1>Gate de fidelidad — la banda</h1>
<p>Arriba la <b>maqueta</b> que aprobaste en la mirada 11 (<code>docs/diseno/banda.html</code>); abajo la <b>banda construida</b>, servida desde el build del producto. Diez encuadres × dos temas × dos idiomas.</p>
<p>Comparten literalmente el mismo CSS —el producto <b>importa</b> <code>ghost.css</code> en vez de copiarlo— y el mismo diccionario, que un gate verifica contra la maqueta cadena por cadena. Lo que buscas aquí es lo que ningún test puede afirmar: si se <b>lee</b> igual de bien.</p>
<div class="aviso"><b>Lo que estas imágenes NO prueban:</b> que la ventana esté protegida de la captura. No pueden — una ventana protegida no aparece en una captura de pantalla, que es exactamente el punto. Lo fotografiado es el webview, no la ventana nativa. Comprobar la protección en una llamada real con la pantalla compartida es una parada ⭐ del gate de prueba.</div>
${filas}
<script>
  document.querySelector(".filtros").addEventListener("click", (ev) => {
    const b = ev.target.closest("button"); if (!b) return;
    const f = b.dataset.f;
    document.querySelectorAll(".filtros button").forEach((x) => x.setAttribute("aria-pressed", String(x === b)));
    document.querySelectorAll("figure").forEach((fig) => {
      fig.hidden = f !== "todo" && fig.dataset.tema !== f && fig.dataset.idioma !== f;
    });
  });
</script>
</body>
</html>
`,
);

console.log(`✓ ${hechas} encuadres del producto y ${readdirSync(join(SALIDA, "maqueta")).length} de la maqueta`);
console.log(`✓ hoja de contacto: docs/fidelidad/S1-banda.html\n`);
console.log("── desbordes en el producto ─────────────────────────────────");
if (desbordes.length === 0) console.log("   ninguno");
else desbordes.forEach((d) => console.log(`   ⚠ ${d}`));
console.log("─────────────────────────────────────────────────────────────");
console.log("── píxeles distintos: maqueta vs producto ───────────────────");
// El suelo no es cero y la razón está medida: la banda de la maqueta vive dentro del escritorio
// de referencia, cuyo marco redondeado le muerde la última fila de píxeles. Son ~73 px sobre
// 103 000 (0,07 %), todos en `y = 85..87`, y son del ENCUADRE, no del producto. El umbral se
// pone en el doble de eso: cualquier desplazamiento real de texto pasa del 2 % (lo midió el
// transcript antes de arreglarlo), así que 0,15 % separa el artefacto del defecto sin holgura
// de sobra. Un umbral generoso «por si acaso» es un gate que no puede fallar.
const UMBRAL = 0.15; // %
const fuera = diferencias.filter((d) => d.medida || d.porcentaje > UMBRAL);
for (const d of diferencias.slice().sort((x, y) => (y.porcentaje ?? 100) - (x.porcentaje ?? 100)).slice(0, 6)) {
  console.log(`   ${d.medida ? `medidas distintas: ${d.medida}` : `${d.porcentaje.toFixed(3)} %`}  ${d.encuadre}`);
}
console.log(`   … ${diferencias.length} encuadres comparados; umbral ${UMBRAL} %`);
if (fuera.length === 0) console.log("   ✓ ninguno pasa del umbral");
else fuera.forEach((d) => console.log(`   ⚠ ${d.encuadre}: ${d.medida ?? d.porcentaje.toFixed(3) + " %"}`));
console.log("─────────────────────────────────────────────────────────────");

console.log("── errores de página ────────────────────────────────────────");
if (errores.length === 0) console.log("   ninguno");
else errores.forEach((e) => console.log(`   ✗ ${e}`));
console.log("─────────────────────────────────────────────────────────────");

process.exit(desbordes.length || errores.length || fuera.length ? 2 : 0);
