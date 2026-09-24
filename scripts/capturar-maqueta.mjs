/**
 * Arnés de CAPTURAS de la maqueta — para mirar los artefactos visuales como imagen antes de
 * presentarlos, y para el gate de FIDELIDAD del sprint (la referencia contra la que se compara).
 *
 * Vive en el repo a propósito: en la Etapa de Diseño este arnés vivió en el scratchpad y se
 * perdió al terminar; volver a escribirlo costó tiempo y, peor, las corridas anteriores no
 * quedaron reproducibles. (Regla 17-bis: un arnés es superficie.)
 *
 * Declara su árbol al arrancar y solo lee `docs/diseno/`: no toca datos del usuario, no abre red
 * (la maqueta es autocontenida y su propio gate lo vigila).
 *
 * Uso:  node scripts/capturar-maqueta.mjs <pagina.html> [--temas dark,light] [--idiomas es,en]
 * Ej.:  node scripts/capturar-maqueta.mjs banda.html
 *
 * LO QUE IMPRIME ES EL GATE. Los desbordes se acumulan y se imprimen JUNTOS al final: en la
 * Etapa de Diseño se imprimían antes de cada estado y `tail` los escondió durante cuatro fases.
 */
import { chromium } from "@playwright/test";
import { mkdirSync, existsSync, rmSync } from "node:fs";
import { resolve, join } from "node:path";
import process from "node:process";

const RAIZ = resolve("docs/diseno");
const SALIDA = resolve("capturas");

const [, , pagina = "banda.html", ...resto] = process.argv;
const opt = (nombre, porDefecto) => {
  const i = resto.indexOf(`--${nombre}`);
  return i === -1 ? porDefecto : resto[i + 1];
};
const temas = opt("temas", "dark,light").split(",");
const idiomas = opt("idiomas", "es,en").split(",");

const archivo = join(RAIZ, pagina);
if (!existsSync(archivo)) {
  console.error(`✗ no existe ${archivo}`);
  process.exit(1);
}

console.log("── árbol del arnés ──────────────────────────────────────────");
console.log(`   lee    : ${RAIZ}`);
console.log(`   escribe: ${SALIDA}  (ignorado por git)`);
console.log(`   página : ${pagina} · temas ${temas.join("/")} · idiomas ${idiomas.join("/")}`);
console.log("─────────────────────────────────────────────────────────────\n");

rmSync(SALIDA, { recursive: true, force: true });
mkdirSync(SALIDA, { recursive: true });
mkdirSync(join(SALIDA, "solo"), { recursive: true });

const navegador = await chromium.launch();
const pag = await navegador.newPage({ viewport: { width: 1260, height: 1000 }, deviceScaleFactor: 2 });

const errores = [];
pag.on("pageerror", (e) => errores.push(`pageerror: ${e.message}`));
pag.on("console", (m) => { if (m.type() === "error") errores.push(`console: ${m.text()}`); });

await pag.goto(`file://${archivo}`);

const estados = await pag.$$eval(".mq-bar [data-estado]", (bs) => bs.map((b) => b.dataset.estado));
if (estados.length === 0) { console.error("✗ la página no declara estados en su barra"); process.exit(1); }

const desbordes = [];
const sinRecorte = [];
const hechas = [];

for (const estado of estados) {
  for (const tema of temas) {
    for (const idioma of idiomas) {
      await pag.evaluate(([e, t, l]) => {
        document.documentElement.dataset.estado = e;
        document.documentElement.dataset.theme = t;
        document.documentElement.dataset.lang = l;
        window.__mqApply();
      }, [estado, tema, idioma]);

      // Desborde: contenido que no cabe en su caja (la banda tiene `overflow:hidden` — lo que
      // no cabe DESAPARECE en silencio, que es exactamente el fallo que hay que cazar).
      const fuera = await pag.evaluate(() => {
        const out = [];
        for (const el of document.querySelectorAll(".banda, .panel, .pildora, .banda .cuerpo-b, .banda .lado-b, .banda .ficha-b")) {
          if (el.offsetParent === null && el.hidden) continue;
          const dv = el.scrollHeight - el.clientHeight;
          const dh = el.scrollWidth - el.clientWidth;
          if (dv > 1 || dh > 1) out.push(`${el.className.split(" ").join(".")} → alto +${dv}px · ancho +${dh}px`);
        }
        return out;
      });
      for (const f of fuera) desbordes.push(`${estado} · ${tema} · ${idioma}  ${f}`);

      const nombre = `${pagina.replace(".html", "")}--${estado}--${tema}--${idioma}.png`;
      const caja = await pag.$(".mq-escritorio");
      await (caja ?? pag).screenshot({ path: join(SALIDA, nombre) });
      hechas.push(nombre);

      // Segundo encuadre: SOLO el artefacto de producto, sin el escritorio de referencia.
      // Es el que se compara contra la ventana construida en el gate de FIDELIDAD.
      //
      // Se recorren TODOS los candidatos, no el primero: una página puede declarar dos formas
      // del mismo artefacto (banda compacta y ampliada) y enseñar una u otra según el estado.
      // La primera versión de este arnés tomaba `$(sel)` —el primero del DOM— y para los tres
      // estados ampliados no escribía nada, sin decirlo. Un encuadre que falta en silencio es
      // el mismo fallo que un gate que no corre: por eso ahora se cuenta y se reporta.
      // El recorte es la REFERENCIA del gate de fidelidad: solo producto. Se apaga el chrome
      // de la sala de diseño, que está posicionado encima y se colaría en la imagen.
      await pag.evaluate((sel) => {
        for (const el of document.querySelectorAll(sel)) el.style.display = "none";
        window.scrollTo(0, 0);
      }, ".mq-bar, .mq-nota, .mq-choque, .mq-corte, .mq-etiqueta, .mq-tabla-pos, .mq-hero, .mq-grupo");
      let recortado = false;
      for (const sel of [".banda", ".panel", ".pildora"]) {
        for (const el of await pag.$$(sel)) {
          if (!(await el.isVisible())) continue;
          await el.screenshot({ path: join(SALIDA, "solo", nombre) });
          recortado = true;
          break;
        }
        if (recortado) break;
      }
      if (!recortado) sinRecorte.push(`${estado} · ${tema} · ${idioma}`);
      await pag.evaluate((sel) => {
        for (const el of document.querySelectorAll(sel)) el.style.display = "";
      }, ".mq-bar, .mq-nota, .mq-choque, .mq-corte, .mq-etiqueta, .mq-tabla-pos, .mq-hero, .mq-grupo");
    }
  }
}

await navegador.close();

console.log(`✓ ${hechas.length} capturas en ${SALIDA}\n`);

console.log("── desbordes ────────────────────────────────────────────────");
if (desbordes.length === 0) console.log("   ninguno");
else desbordes.forEach((d) => console.log(`   ⚠ ${d}`));
console.log("─────────────────────────────────────────────────────────────");

console.log("── estados sin recorte del artefacto ────────────────────────");
if (sinRecorte.length === 0) console.log("   ninguno: los", hechas.length, "estados dejaron su recorte");
else sinRecorte.forEach((s) => console.log(`   ⚠ ${s}: no se encontró artefacto visible que recortar`));
console.log("─────────────────────────────────────────────────────────────");

console.log("── errores de la página ─────────────────────────────────────");
if (errores.length === 0) console.log("   ninguno");
else errores.forEach((e) => console.log(`   ✗ ${e}`));
console.log("─────────────────────────────────────────────────────────────");

process.exit(desbordes.length || errores.length || sinRecorte.length ? 2 : 0);
