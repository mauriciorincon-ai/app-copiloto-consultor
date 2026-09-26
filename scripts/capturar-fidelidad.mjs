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
const DISENO = join(RAIZ, "docs/diseno");
const FIDELIDAD = join(RAIZ, "docs/fidelidad");

/**
 * Las filas de ABAJO que se dejan fuera de la comparación: la mordida del marco redondeado de
 * la página de referencia, que no es del producto. **Medidas, no estimadas** — ver el comentario
 * de la comparación, más abajo.
 */
const MARCO = 9;
const PUERTO = 4180;

/**
 * LOS ARTEFACTOS que este gate compara. Añadir uno es añadir una entrada, no tocar el motor.
 *
 * Nació cableado a la banda (fase 1) y se generalizó en la fase 2, cuando aparecieron las
 * pantallas del cuaderno. Cablearlo otra vez para tres pantallas más habría duplicado la parte
 * delicada —medir el ancho de la referencia, apagar la sala de diseño, restar los mapas de
 * bits—, y una copia de un gate es un gate que se arregla en un sitio y sigue roto en el otro.
 */
const ARTEFACTOS = [
  {
    id: "banda",
    titulo: "la banda",
    mirada: "la mirada 11",
    selectorMaqueta: ".banda",
    selectorProducto: "section.banda",
    /** Lo que se mide para detectar desbordes dentro del artefacto. */
    desbordes: ".banda, .banda .cuerpo-b, .banda .lado-b, .banda .ficha-b",
    encuadres: [
      { id: "esperando", maqueta: "banda.html", estado: "esperando", alto: 88, url: "ventana=banda&estado=esperando" },
      { id: "buscando", maqueta: "banda.html", estado: "buscando", alto: 88, url: "ventana=banda&estado=buscando" },
      { id: "ficha", maqueta: "banda.html", estado: "ficha", alto: 88, url: "ventana=banda&estado=ficha" },
      { id: "ficha-2", maqueta: "banda.html", estado: "ficha-2", alto: 200, url: "ventana=banda&estado=ficha&ampliada=1" },
      { id: "sin-resultado", maqueta: "banda.html", estado: "sin-resultado", alto: 88, url: "ventana=banda&estado=sin-resultado" },
      { id: "sin-resultado-2", maqueta: "banda.html", estado: "sin-resultado-2", alto: 200, url: "ventana=banda&estado=sin-resultado&ampliada=1" },
      { id: "sin-verificar", maqueta: "banda.html", estado: "sin-verificar", alto: 88, url: "ventana=banda&estado=sin-verificar&verificado=0" },
      { id: "sin-verificar-2", maqueta: "banda.html", estado: "sin-verificar-2", alto: 200, url: "ventana=banda&estado=sin-verificar&verificado=0&ampliada=1" },
      { id: "transcript", maqueta: "banda.html", estado: "transcript", alto: 200, url: "ventana=banda&estado=ficha&transcript=1" },
      { id: "flotante", maqueta: "banda.html", estado: "flotante", alto: 88, url: "ventana=banda&estado=ficha&acoplada=0" },
      // El modo solo audio (C15, sprint 002). 44 px: otra anatomía, no la misma banda con menos
      // cosas — sin cabecera, y con el contador de red bajado a la línea.
      { id: "voz", maqueta: "banda.html", estado: "voz", alto: 44, url: "ventana=banda&estado=voz" },
      { id: "voz-espera", maqueta: "banda.html", estado: "voz-espera", alto: 44, url: "ventana=banda&estado=voz-espera" },
      { id: "voz-sin", maqueta: "banda.html", estado: "voz-sin", alto: 44, url: "ventana=banda&estado=voz-sin" },
    ],
  },
  {
    id: "cuaderno",
    titulo: "el cuaderno",
    mirada: "las miradas 12 y 13",
    selectorMaqueta: ".ventana",
    selectorProducto: "main.ventana",
    desbordes: ".ventana .contenido, .ventana .rail",
    encuadres: [
      { id: "sesion", maqueta: "sesion.html", estado: "s1", alto: 640, url: "ventana=principal&pantalla=sesion" },
      { id: "permisos", maqueta: "permisos.html", estado: "s1", alto: 640, url: "ventana=principal&pantalla=permisos" },
      { id: "corpus", maqueta: "corpus.html", estado: "s1", alto: 640, url: "ventana=principal&pantalla=corpus" },
      { id: "honestidad", maqueta: "honestidad.html", estado: "s1", alto: 640, url: "ventana=principal&pantalla=honestidad" },
      { id: "idioma", maqueta: "idioma.html", estado: "s1", alto: 640, url: "ventana=principal&pantalla=idioma" },
    ],
  },
];

const TEMAS = ["dark", "light"];
const IDIOMAS = [
  { id: "es", locale: "es-CO" },
  { id: "en", locale: "en-US" },
];

/** El chrome de la sala de diseño, que jamás debe entrar en el encuadre. */
const SALA_DE_DISENO =
  ".mq-bar, .mq-nota, .mq-choque, .mq-corte, .mq-etiqueta, .mq-tabla-pos, .mq-hero, .mq-grupo";

console.log("── árbol del arnés ──────────────────────────────────────────");
console.log(`   referencia: ${DISENO}`);
console.log(`   producto  : http://localhost:${PUERTO}  (desde dist/, no desde el dev server)`);
console.log(`   escribe   : ${FIDELIDAD}`);
console.log(`   artefactos: ${ARTEFACTOS.map((a) => a.id).join(" · ")}`);
console.log("─────────────────────────────────────────────────────────────\n");

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

/**
 * Deja la página QUIETA antes de fotografiarla. Se aplica a los dos lados por igual.
 *
 * **Por qué existe:** una corrida devolvió **2,574 %** de divergencia en `sin-verificar-2 · light
 * · es` y la siguiente, sin tocar nada, **0,069 %**. Un gate que da dos respuestas distintas al
 * mismo código no es un gate: o deja pasar lo que debía parar, o para lo que debía pasar, y en
 * cualquiera de los dos casos deja de creerse. Las tres esperas de aquí quitan las tres fuentes
 * de ruido: tipografías que aún no habían cargado, transiciones a medio camino y un cuadro de
 * pintado sin terminar.
 */
async function asentar(pag) {
  await pag.addStyleTag({
    content: "*,*::before,*::after{animation:none!important;transition:none!important}",
  });
  await pag.evaluate(() => document.fonts.ready);
  await pag.evaluate(
    () => new Promise((listo) => requestAnimationFrame(() => requestAnimationFrame(listo))),
  );
}

const navegador = await chromium.launch();
const errores = [];
const desbordes = [];
const diferencias = [];
let hechas = 0;

for (const art of ARTEFACTOS) {
  const salida = join(FIDELIDAD, `s1-${art.id}`);
  rmSync(salida, { recursive: true, force: true });
  mkdirSync(join(salida, "maqueta"), { recursive: true });
  mkdirSync(join(salida, "producto"), { recursive: true });

  const altoMax = Math.max(...art.encuadres.map((e) => e.alto));

  // El ancho de la referencia no se supone: se MIDE. El artefacto de la maqueta vive dentro de
  // una página con márgenes y marcos, y capturar el producto a un ancho y la maqueta a otro
  // haría fallar la comparación por una razón que no tiene nada que ver con la fidelidad — y
  // peor, un acierto por casualidad se leería como que sí la tiene.
  // Se mide con la MISMA ventana ancha con la que después se fotografía: medir en una y
  // fotografiar en otra es cómo se cuela una diferencia de 36 px que nadie atribuye a la ventana.
  const ANCHO_DE_MEDIDA = 1600;
  const regla = await navegador.newPage();
  await regla.setViewportSize({ width: ANCHO_DE_MEDIDA, height: altoMax + 300 });
  await regla.goto(`file://${join(DISENO, art.encuadres[0].maqueta)}`);
  const caja = await (await regla.$(art.selectorMaqueta)).boundingBox();
  const ANCHO = Math.round(caja.width);
  await regla.close();
  console.log(`   ${art.id}: ancho de la referencia, medido: ${ANCHO} px`);

  for (const tema of TEMAS) {
    for (const idioma of IDIOMAS) {
      // La ventana del navegador es ANCHA a propósito, y el producto se estrecha después a su
      // tamaño real. La maqueta centra su artefacto dentro de una página con relleno lateral: si
      // el navegador mide justo lo que mide el artefacto, ese relleno lo ENCOGE (el cuaderno
      // salía a 924 px en vez de 960) y la comparación falla por la ventana, no por el producto.
      const ctx = await navegador.newContext({
        viewport: { width: ANCHO_DE_MEDIDA, height: altoMax + 300 },
        colorScheme: tema,
        locale: idioma.locale,
        deviceScaleFactor: 1,
      });
      const pag = await ctx.newPage();
      const anota = (m) => errores.push(`${art.id} ${tema}/${idioma.id}: ${m}`);
      pag.on("pageerror", (e) => anota(e.message));
      pag.on("console", (m) => { if (m.type() === "error") anota(m.text()); });

      // ---- LA MAQUETA ----
      for (const e of art.encuadres) {
        await pag.goto(`file://${join(DISENO, e.maqueta)}`);
        await pag.evaluate(([estado, tm, lg, sel]) => {
          document.documentElement.dataset.estado = estado;
          document.documentElement.dataset.theme = tm;
          document.documentElement.dataset.lang = lg;
          window.__mqApply();
          // TODO el chrome de la sala de diseño fuera del encuadre. La barra de estados es
          // `position: sticky`: al desplazarse para fotografiar un artefacto alto se le montaba
          // encima y la referencia del gate salía con media pantalla tapada por botones.
          for (const el of document.querySelectorAll(sel)) el.style.display = "none";
          window.scrollTo(0, 0);
        }, [e.estado, tema, idioma.id, SALA_DE_DISENO]);

        await asentar(pag);
        const candidatos = await pag.$$(art.selectorMaqueta);
        let pintado = null;
        for (const el of candidatos) if (await el.isVisible()) { pintado = el; break; }
        if (!pintado) { errores.push(`maqueta ${art.id}/${e.id}: ningún ${art.selectorMaqueta} visible`); continue; }
        await pintado.screenshot({ path: join(salida, "maqueta", `${e.id}--${tema}--${idioma.id}.png`) });
      }

      // ---- EL PRODUCTO ----
      for (const e of art.encuadres) {
        await pag.setViewportSize({ width: ANCHO, height: e.alto });
        await pag.goto(`http://localhost:${PUERTO}/?${e.url}`);
        await pag.waitForSelector(art.selectorProducto);

        const fuera = await pag.evaluate((sel) => {
          const out = [];
          for (const el of document.querySelectorAll(sel)) {
            const dv = el.scrollHeight - el.clientHeight;
            const dh = el.scrollWidth - el.clientWidth;
            if (dv > 1 || dh > 1) out.push(`${el.className.split(" ").join(".")} alto +${dv}px ancho +${dh}px`);
          }
          return out;
        }, art.desbordes);
        for (const f of fuera) desbordes.push(`${art.id} · ${e.id} · ${tema} · ${idioma.id}  ${f}`);

        await asentar(pag);
        const nodo = await pag.$(art.selectorProducto);
        await nodo.screenshot({ path: join(salida, "producto", `${e.id}--${tema}--${idioma.id}.png`) });
        hechas++;
      }
      await ctx.close();
    }
  }

  // ---- LA COMPARACIÓN ----
  // Las dos imágenes salen del MISMO Chromium, con el MISMO CSS, a la MISMA anchura: si el
  // producto es fiel, deben coincidir píxel a píxel. Comparar de verdad, y no «a ojo sobre una
  // hoja de contacto», es lo que convierte esto en un gate: un ojo cansado aprueba una pantalla
  // desplazada 3 px, y esa pantalla ya no obedece a la maqueta.
  const lienzo = await navegador.newPage();
  for (const e of art.encuadres) {
    for (const tm of TEMAS) {
      for (const lg of IDIOMAS) {
        const n = `${e.id}--${tm}--${lg.id}.png`;
        let a, b;
        try {
          a = readFileSync(join(salida, "maqueta", n)).toString("base64");
          b = readFileSync(join(salida, "producto", n)).toString("base64");
        } catch {
          diferencias.push({ artefacto: art.id, encuadre: `${e.id} · ${tm} · ${lg.id}`, medida: "falta un recorte" });
          continue;
        }
        const r = await lienzo.evaluate(async ([uno, dos, MARCO]) => {
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
          // **Las nueve filas de abajo no se comparan, y no es holgura: no son del producto.**
          // El recorte de la maqueta sale de una página de referencia cuyo marco redondeado le
          // muerde las esquinas inferiores. Es una cuña de 58 px repartida en nueve filas
          // —2,2,2,4,4,6,8,12,18, medidas—, idéntica en un encuadre de 88 px y en uno de 44, y
          // **siempre en las nueve últimas**. El contenido nunca llega ahí: la banda centra su
          // línea, y a 44 px el texto vive entre las filas 15 y 28.
          const alto = ia.height - MARCO;
          for (let y = 0; y < alto; y++) {
            for (let x = 0; x < ia.width; x++) {
              const i = (ia.width * y + x) << 2;
              // Tolerancia por canal: el antialias de una misma fuente puede variar un punto.
              if (
                Math.abs(pa[i] - pb[i]) > 8 ||
                Math.abs(pa[i + 1] - pb[i + 1]) > 8 ||
                Math.abs(pa[i + 2] - pb[i + 2]) > 8
              ) {
                distintos++;
              }
            }
          }
          return { distintos, porcentaje: (distintos / (ia.width * alto)) * 100 };
        }, [a, b, MARCO]);
        diferencias.push({ artefacto: art.id, encuadre: `${e.id} · ${tm} · ${lg.id}`, ...r });
      }
    }
  }
  await lienzo.close();

  // ---- hoja de contacto ----
  const filas = art.encuadres.flatMap((e) =>
    TEMAS.flatMap((tm) =>
      IDIOMAS.map((lg) => {
        const n = `${e.id}--${tm}--${lg.id}.png`;
        return `  <figure data-tema="${tm}" data-idioma="${lg.id}">
    <figcaption><b>${e.id}</b> · ${tm} · ${lg.id} · ${e.alto} px</figcaption>
    <div class="par">
      <div><span class="et">maqueta</span><img src="s1-${art.id}/maqueta/${n}" alt="${e.id} en la maqueta, tema ${tm}, idioma ${lg.id}"></div>
      <div><span class="et">producto</span><img src="s1-${art.id}/producto/${n}" alt="${e.id} en el producto, tema ${tm}, idioma ${lg.id}"></div>
    </div>
  </figure>`;
      }),
    ),
  ).join("\n");

  writeFileSync(
    join(FIDELIDAD, `S1-${art.id}.html`),
    `<!doctype html>
<html lang="es">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Angel Ghost · Gate de fidelidad S1 — ${art.titulo}</title>
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
<h1>Gate de fidelidad — ${art.titulo}</h1>
<p>Arriba la <b>maqueta</b> que aprobaste en ${art.mirada}; abajo lo <b>construido</b>, servido desde el build del producto. ${art.encuadres.length} encuadres × dos temas × dos idiomas.</p>
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
}

await navegador.close();
parar();

console.log(`\n✓ ${hechas} encuadres del producto y ${diferencias.length} comparaciones`);
for (const art of ARTEFACTOS) console.log(`✓ hoja de contacto: docs/fidelidad/S1-${art.id}.html`);

console.log("\n── desbordes en el producto ─────────────────────────────────");
if (desbordes.length === 0) console.log("   ninguno");
else desbordes.forEach((d) => console.log(`   ⚠ ${d}`));
console.log("─────────────────────────────────────────────────────────────");

console.log("── píxeles distintos: maqueta vs producto ───────────────────");
// **El suelo tenía dos partes, y solo una era del producto. El sprint 002 las separó.**
//
// Hasta entonces este umbral era 0,15 % y su comentario justificaba el suelo con «~73 px de marco
// redondeado sobre 103 000». Las dos mitades estaban mezcladas:
//
// 1. **La mordida del marco** — 58 px en las nueve filas de abajo, del ENCUADRE y no del producto.
//    Es **constante**: los mismos 58 px en una banda de 88 px y en una de 44. Medida en porcentaje
//    valía 0,056 % en la primera y **0,112 % en la segunda**, porque la mordida no cambia y el área
//    sí. Al llegar el modo solo audio, los seis encuadres de 44 px aparecieron pegados al umbral y
//    dos lo pasaron **sin que hubiera nada que arreglar en el producto**.
// 2. **El antialias del texto**, que sí escala con la cantidad de tinta y por tanto con el área.
//
// La 1 ya no se mide: `MARCO` la deja fuera. La 2 se mide en **porcentaje**, que es su unidad
// natural — medirla en píxeles absolutos tiene el defecto simétrico, y se comprobó: con un techo de
// 120 px las nueve pantallas del cuaderno (960 × 640, mucha más tinta) se ponían en rojo a 0,03 %.
//
// Con el marco fuera, el suelo real es **0,072 %** (honestidad en inglés, la pantalla con más
// texto) y un desplazamiento de verdad son **más del 2 %** — lo midió el transcript antes de
// arreglarlo. El umbral se queda en 0,15 % y ahora quiere decir lo mismo a cualquier alto. Un
// umbral generoso «por si acaso» es un gate que no puede fallar.
const UMBRAL = 0.15; // %
const fuera = diferencias.filter((d) => d.medida || d.porcentaje > UMBRAL);
for (const d of diferencias.slice().sort((x, y) => (y.distintos ?? 1e9) - (x.distintos ?? 1e9)).slice(0, 8)) {
  const cuanto = d.medida
    ? `medidas distintas: ${d.medida}`
    : `${d.porcentaje.toFixed(3)} %  (${String(d.distintos).padStart(5)} px)`;
  console.log(`   ${cuanto}  ${d.artefacto} · ${d.encuadre}`);
}
console.log(
  `   … ${diferencias.length} encuadres comparados; umbral ${UMBRAL} % (sin las ${MARCO} filas del marco)`,
);
if (fuera.length === 0) console.log("   ✓ ninguno pasa del umbral");
else
  fuera.forEach((d) =>
    console.log(
      `   ⚠ ${d.artefacto} · ${d.encuadre}: ${d.medida ?? `${d.porcentaje.toFixed(3)} % (${d.distintos} px)`}`,
    ),
  );
console.log("─────────────────────────────────────────────────────────────");

console.log("── errores de página ────────────────────────────────────────");
if (errores.length === 0) console.log("   ninguno");
else errores.forEach((e) => console.log(`   ✗ ${e}`));
console.log("─────────────────────────────────────────────────────────────");

process.exit(desbordes.length || errores.length || fuera.length ? 2 : 0);
