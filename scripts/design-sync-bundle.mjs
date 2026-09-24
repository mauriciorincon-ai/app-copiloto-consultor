#!/usr/bin/env node
/**
 * Arma el bundle publicable del design system — `design-sync/` — DERIVÁNDOLO de la maqueta
 * aprobada en G-Diseño, no transcribiéndolo a mano.
 *
 * La jerarquía que manda (regla 16 de CLAUDE.md, `/design-sync`):
 *
 *   design-system.md   →   design-sync/   →   el proyecto en Claude Design
 *   (fuente de verdad)     (este bundle)      (vitrina; jamás se edita allá)
 *
 * **Por qué es un generador y no una carpeta escrita a mano.** El bundle tiene que ser espejo del
 * sistema, y un espejo copiado a mano se desvía en el primer sprint: alguien cambia un hex en
 * `ghost.css`, la tarjeta se queda con el viejo, y la vitrina enseña un sistema que la app ya no
 * usa. Aquí cada tarjeta se ARMA leyendo `docs/diseno/assets/ghost.css` y el HTML de la maqueta,
 * así que un cambio en el sistema cambia el bundle o el gate se pone en rojo
 * (`tests/unit/design-sync-espejo.test.ts`). Ningún hex, ninguna clase y ningún copy se escriben
 * dos veces.
 *
 * **Qué produce.** Tarjetas HTML AUTOCONTENIDAS —CSS en línea, sprite en línea, cero CDNs— cada
 * una con su primera línea exacta `<!-- @dsCard group="…" name="…" -->`, que es por donde Claude
 * Design las indexa: sin ella la tarjeta no aparece. Cada tarjeta enseña el componente en los DOS
 * temas, porque el sistema tiene dos y uno solo no es el sistema.
 *
 * Uso:  `node scripts/design-sync-bundle.mjs`   (escribe el bundle)
 *       `node scripts/design-sync-bundle.mjs --verificar`   (no escribe; dice si hay deriva)
 *
 * Publicar NO es trabajo de este script: lo hace `/design-sync`, que solo invoca el usuario y
 * siempre después del gate ⭐⭐ del cierre de ciclo.
 */
import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";
import { dirname, join } from "node:path";
import { JSDOM } from "jsdom";

const DISENO = "docs/diseno";
const BUNDLE = "design-sync";

// ─────────────────────────────────────────────────────────────────────────────
// CSS · parseo por bloques, para poder elegir reglas y reescribir el tema
// ─────────────────────────────────────────────────────────────────────────────

/** Parte una hoja en bloques de primer nivel. Cuenta llaves, así que las `@media` salen enteras. */
function bloques(css) {
  const out = [];
  let i = 0;
  while (i < css.length) {
    if (css.startsWith("/*", i)) {
      const fin = css.indexOf("*/", i);
      i = fin < 0 ? css.length : fin + 2;
      continue;
    }
    if (/\s/.test(css[i])) {
      i++;
      continue;
    }
    const abre = css.indexOf("{", i);
    if (abre < 0) break;
    const sel = css.slice(i, abre).trim();
    let hondo = 1;
    let j = abre + 1;
    while (j < css.length && hondo > 0) {
      if (css[j] === "{") hondo++;
      else if (css[j] === "}") hondo--;
      j++;
    }
    out.push({ sel, cuerpo: css.slice(abre + 1, j - 1), texto: css.slice(i, j) });
    i = j;
  }
  return out;
}

/**
 * El tema vive en `html[data-theme]`, y una tarjeta necesita los DOS a la vez. Se reescribe el
 * selector —no los valores— a un envoltorio: `.tema.oscuro` / `.tema.claro`. Mecánico a propósito:
 * si algún día el sistema añade una regla propia del tema claro, viaja sola.
 */
function aEnvoltorio(sel) {
  return sel
    .split(",")
    .map((s) =>
      s
        .trim()
        .replace(/^html\[data-theme="dark"\]/, ".tema.oscuro")
        .replace(/^html\[data-theme="light"\]/, ".tema.claro"),
    )
    .join(", ");
}

/** Reglas que toda tarjeta necesita aunque su fragmento no nombre la clase. */
const SIEMPRE = new Set([
  ":root",
  ":root, html[data-theme=\"dark\"]",
  "html[data-theme=\"light\"]",
  "*, *::before, *::after",
  "html",
  "body",
  "h1, h2, h3, p",
  "button",
  ":focus-visible",
  ".mono",
  "[hidden]",
  ".sr",
  ".ic",
  ".ic.s",
  ".ic.relleno",
  "html[data-lang=\"es\"] [lang=\"en\"]",
  "html[data-lang=\"en\"] [lang=\"es\"]",
]);

/** Las clases que un fragmento usa de verdad. */
function clasesDe(html) {
  const s = new Set();
  for (const m of html.matchAll(/class="([^"]*)"/g)) {
    for (const c of m[1].split(/\s+/)) if (c) s.add(c);
  }
  return s;
}

function clasesDelSelector(sel) {
  return [...sel.matchAll(/\.(-?[_a-zA-ZÀ-ÿ][\wÀ-ſ-]*)/g)].map((m) => m[1]);
}

/** El CSS que una tarjeta necesita: los tokens, lo base, y toda regla que nombre una clase suya. */
function cssPara(hojas, clases) {
  const trozos = [];
  for (const hoja of hojas) {
    for (const b of bloques(hoja)) {
      if (b.sel.startsWith("@media")) {
        // Reduced-motion es una regla del sistema, no de un componente: viaja siempre.
        const interesa =
          /prefers-reduced-motion/.test(b.sel) ||
          bloques(b.cuerpo).some((d) => clasesDelSelector(d.sel).some((c) => clases.has(c)));
        if (interesa) trozos.push(`@media ${b.sel.replace(/^@media\s*/, "")} {${b.cuerpo}}`);
        continue;
      }
      if (b.sel.startsWith("@")) continue;
      const suyas = clasesDelSelector(b.sel);
      const entra = SIEMPRE.has(b.sel) || (suyas.length > 0 && suyas.some((c) => clases.has(c)));
      if (entra) trozos.push(`${aEnvoltorio(b.sel)} {${b.cuerpo}}`);
    }
  }
  return trozos.join("\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// Sprite · solo los glifos que la tarjeta usa (D9: sprite propio, cero librerías)
// ─────────────────────────────────────────────────────────────────────────────

function simbolos(iconosJs) {
  const mapa = new Map();
  for (const m of iconosJs.matchAll(/<symbol id="([^"]+)"[\s\S]*?<\/symbol>/g)) mapa.set(m[1], m[0]);
  return mapa;
}

function spritePara(mapa, html) {
  const ids = [...new Set([...html.matchAll(/href="#(i-[\w-]+)"/g)].map((m) => m[1]))];
  const usados = ids.map((id) => mapa.get(id)).filter(Boolean);
  if (usados.length === 0) return "";
  return `<svg style="display:none" aria-hidden="true">\n${usados.join("\n")}\n</svg>`;
}

// ─────────────────────────────────────────────────────────────────────────────
// Maqueta · de dónde sale el markup de cada tarjeta
// ─────────────────────────────────────────────────────────────────────────────

const CHROME = ".mq-bar, .mq-nota, .mq-corte, .mq-choque, .mq-dock, .mq-h2, script, link";

function documento(pagina) {
  const dom = new JSDOM(readFileSync(join(DISENO, pagina), "utf8"));
  return dom.window.document;
}

/** La misma cuenta que hace `maqueta.js`: en un estado, lo que no es de ese estado no existe. */
function fijarEstado(doc, estado) {
  for (const el of [...doc.querySelectorAll("[data-en]")]) {
    if (!el.dataset.en.split(" ").includes(estado)) el.remove();
  }
  for (const el of [...doc.querySelectorAll("[data-transcript-en]")]) {
    el.dataset.transcript = el.dataset.transcriptEn.split(" ").includes(estado) ? "on" : "off";
  }
  for (const el of [...doc.querySelectorAll("[data-acople-en]")]) {
    const mapa = Object.fromEntries(el.dataset.acopleEn.split(" ").map((p) => p.split(":")));
    if (mapa[estado]) el.dataset.acople = mapa[estado];
    else delete el.dataset.acople;
  }
}

function limpiar(doc) {
  for (const el of [...doc.querySelectorAll(CHROME)]) el.remove();
}

/**
 * Qué dice cada estado de la banda, **leído de la tabla del `design-system.md` §9-quinquies**.
 *
 * La primera versión de esto tomaba la nota «qué mirar» de la maqueta, y era una mala idea que
 * solo se vio mirando la tarjeta: esas notas son de la SALA de diseño y hablan de cosas que la
 * tarjeta no tiene —«la reunión ya está recortada (línea azul)», «el recuadro ámbar sobre el
 * Dock»—, porque el marco de la sala se quita al publicar. Una nota que señala algo ausente es
 * peor que ninguna nota. La tabla del sistema, en cambio, describe el estado y nada más.
 */
function gramaticaDeLaBanda() {
  const md = readFileSync("design-system.md", "utf8");
  const seccion = md.split("## 9-quinquies")[1] ?? "";
  const mapa = new Map();
  for (const linea of seccion.split("\n")) {
    const celdas = linea.split("|").map((c) => c.trim());
    if (celdas.length !== 6) continue; // | estado | alto | izquierda | derecha |
    const [, estado, alto, izq, der] = celdas;
    if (!/^\d+$/.test(alto)) continue;
    // La tabla es markdown: su negrita llega como `**así**` y hay que traducirla, o la tarjeta
    // publica asteriscos. Lo vio la foto, no el conteo.
    const negrita = (t) => t.replace(/\*\*(.+?)\*\*/g, "<b>$1</b>");
    mapa.set(estado, `${alto} px · <b>izquierda:</b> ${negrita(izq)} · <b>derecha:</b> ${negrita(der)}`);
  }
  if (mapa.size === 0) throw new Error("design-system.md §9-quinquies ya no trae la tabla de estados");
  return mapa;
}

// ─────────────────────────────────────────────────────────────────────────────
// Las tarjetas
// ─────────────────────────────────────────────────────────────────────────────

/** `[estado de la maqueta, fila de la tabla del design-system]` — los seis de §9-quinquies. */
const ESTADOS_BANDA = [
  ["esperando", "esperando"],
  ["buscando", "buscando"],
  ["ficha", "ficha"],
  ["sin-resultado", "sin resultado"],
  ["sin-verificar", "sin verificar"],
  ["transcript", "transcript"],
];

const TARJETAS = [
  { grupo: "Fundamentos", nombre: "Tokens", archivo: "fundamentos/tokens.html", kit: 0 },
  { grupo: "Fundamentos", nombre: "Estados", archivo: "fundamentos/estados.html", kit: 1 },
  { grupo: "Fundamentos", nombre: "Anti-patrones", archivo: "fundamentos/anti-patrones.html", kit: 10 },
  { grupo: "Componentes", nombre: "Ficha de evidencia", archivo: "componentes/ficha-de-evidencia.html", kit: 2 },
  { grupo: "Componentes", nombre: "Contador de red", archivo: "componentes/contador-de-red.html", kit: 3 },
  { grupo: "Componentes", nombre: "Estado de permiso", archivo: "componentes/estado-de-permiso.html", kit: 4 },
  { grupo: "Componentes", nombre: "Bandera de jurisdicción", archivo: "componentes/bandera-de-jurisdiccion.html", kit: 5 },
  { grupo: "Componentes", nombre: "Estado de sesión", archivo: "componentes/estado-de-sesion.html", kit: 6 },
  { grupo: "Componentes", nombre: "Alerta del radar", archivo: "componentes/alerta-del-radar.html", kit: 7 },
  { grupo: "Componentes", nombre: "Primitivas", archivo: "componentes/primitivas.html", kit: 8 },
  { grupo: "Componentes", nombre: "Píldora de voz", archivo: "componentes/pildora-de-voz.html", kit: 9 },
  { grupo: "Componentes · S1", nombre: "La banda — seis estados", archivo: "s1/la-banda.html", banda: true },
  { grupo: "Componentes · S1", nombre: "«Todavía no»", archivo: "s1/todavia-no.html", pendiente: true },
];

/**
 * Los fragmentos de una tarjeta: `[{etiqueta, nota, html}]`, más su `regla` (la línea de canon).
 *
 * Las etiquetas y las reglas se toman como **HTML**, no como texto: la maqueta es bilingüe con los
 * dos idiomas dentro y el CSS esconde uno. Aplanarlas a texto pegaba las dos lenguas en una
 * ristra —«Alerta del radarRadar alert»—, y eso también se vio mirando la tarjeta, no contándola.
 */
function fragmentos(t) {
  if (t.kit !== undefined) {
    const doc = documento("kit.html");
    limpiar(doc);
    const sec = [...doc.querySelectorAll("section.k-sec")][t.kit];
    if (!sec) throw new Error(`kit.html no tiene la sección ${t.kit}`);
    const h2 = sec.querySelector("h2");
    const small = h2 ? h2.querySelector("small") : null;
    const regla = small ? small.innerHTML.trim() : "";
    if (h2) h2.remove();
    return { regla, piezas: [{ etiqueta: "", nota: "", html: sec.innerHTML.trim() }] };
  }
  if (t.banda) {
    const gramatica = gramaticaDeLaBanda();
    const piezas = ESTADOS_BANDA.map(([estado, fila]) => {
      const doc = documento("banda.html");
      fijarEstado(doc, estado);
      limpiar(doc);
      const banda = doc.querySelector("section.banda");
      if (!banda) throw new Error(`banda.html no dibuja el estado «${estado}»`);
      const nota = gramatica.get(fila);
      if (!nota) throw new Error(`el design-system no describe el estado «${fila}»`);
      return { etiqueta: fila, nota, html: banda.outerHTML };
    });
    return {
      regla:
        "<span lang=\"es\">Seis estados de contenido · la banda dice a la izquierda, de dónde sale " +
        "y qué teclas hay a la derecha · en la banda las acciones son TECLAS</span>" +
        "<span lang=\"en\">Six content states · the band speaks on the left, where it comes from " +
        "and which keys exist on the right · in the band, actions are KEYS</span>",
      piezas,
    };
  }
  if (t.pendiente) {
    const piezas = ["sesion.html", "idioma.html"].map((pagina) => {
      const doc = documento(pagina);
      limpiar(doc);
      const el = doc.querySelector(".tarjeta.pendiente");
      if (!el) throw new Error(`${pagina} ya no dibuja una tarjeta «todavía no»`);
      el.removeAttribute("style");
      // La etiqueta sale del título de la pantalla en la maqueta, no del nombre del archivo:
      // «sesion» sin tilde es un nombre de archivo, y esto se publica.
      const titulo = (doc.querySelector("title")?.textContent ?? "").split("—").pop().trim();
      return { etiqueta: titulo.replace(/^\d+\s*/, ""), nota: "", html: el.outerHTML };
    });
    return {
      regla:
        "<span lang=\"es\">Lo que aún no está construido se dice, con su sprint — nunca se " +
        "insinúa con un botón que no hace nada</span>" +
        "<span lang=\"en\">What is not built yet is stated, with its sprint — never hinted at " +
        "with a button that does nothing</span>",
      piezas,
    };
  }
  throw new Error(`la tarjeta «${t.nombre}» no dice de dónde sale`);
}

/**
 * El CSS propio de la tarjeta: el marco de exhibición, nada del sistema. Usa tokens, jamás
 * valores mágicos — salvo el ancho del marco de la banda, que es una decisión de exhibición
 * (la banda de verdad ocupa el ancho de la pantalla) y va declarada aquí.
 */
const MARCO = `
  body { margin: 0; padding: var(--s-6); }
  .ds-cab { width: 100%; max-width: 1180px; margin: 0 auto var(--s-5); }
  .ds-cab h1 { font-family: var(--font-ui); font-size: var(--t-h1); font-weight: 600; margin: 0 0 var(--s-2); }
  .ds-cab p { font-family: var(--font-ui); font-size: var(--t-ui); color: var(--ink-2); margin: 0; line-height: 1.5; max-width: 82ch; }
  .ds-cab p.ds-regla { color: var(--ink); margin-bottom: var(--s-2); }
  .ds-cab code { font-family: var(--font-mono); font-size: var(--t-mono); }
  /* 1180 px o lo que haya: la banda ocupa el ancho de la PANTALLA, y en un marco estrecho su
     ellipsis recorta la fuente y la fecha como si fuera un defecto del componente. */
  .ds-temas { width: 100%; max-width: 1180px; margin: 0 auto; display: flex; flex-direction: column; gap: var(--s-6); }
  .tema { padding: var(--s-5); border: var(--borde) solid var(--line); border-radius: var(--r-panel); background: var(--bg); color: var(--ink); }
  .tema > .ds-t { display: block; margin: 0 0 var(--s-4); font-family: var(--font-mono); font-size: 11px; letter-spacing: .1em; text-transform: uppercase; color: var(--ink-2); }
  .ds-pieza + .ds-pieza { margin-top: var(--s-6); }
  .ds-pieza > .ds-et { display: block; margin: 0 0 var(--s-2); font-family: var(--font-ui); font-size: var(--t-meta); font-weight: 600; color: var(--ink); }
  .ds-pieza > .ds-nota { display: block; margin: 0 0 var(--s-3); font-family: var(--font-ui); font-size: var(--t-meta); color: var(--ink-2); line-height: 1.5; max-width: 72ch; }
  .ds-marco-banda { border: var(--borde) solid var(--line); border-radius: var(--r-ficha); overflow: hidden; }
  @media (max-width: 720px) { body { padding: var(--s-4); } }
`;

function tarjeta(t, hojas, mapaIconos) {
  const { regla, piezas } = fragmentos(t);
  const cuerpo = piezas
    .map(
      (p) =>
        `      <div class="ds-pieza">\n` +
        (p.etiqueta ? `        <span class="ds-et">${p.etiqueta}</span>\n` : "") +
        (p.nota ? `        <span class="ds-nota">${p.nota}</span>\n` : "") +
        (t.banda ? `        <div class="ds-marco-banda">${p.html}</div>\n` : `        ${p.html}\n`) +
        `      </div>`,
    )
    .join("\n");

  const todo = piezas.map((p) => p.html).join("\n");
  const clases = clasesDe(todo);
  // El marco de exhibición también tiene clases, y su CSS va aparte: se declaran para que el
  // gate del espejo no las busque en la maqueta.
  const css = cssPara(hojas, clases) + "\n/* marco de exhibición del bundle */" + MARCO;
  const sprite = spritePara(mapaIconos, todo);

  const temas = [
    ["oscuro", "tema oscuro · primario"],
    ["claro", "tema claro · papel"],
  ]
    .map(
      ([clase, rotulo]) =>
        `    <section class="tema ${clase}">\n      <span class="ds-t">${rotulo}</span>\n${cuerpo}\n    </section>`,
    )
    .join("\n");

  return `<!-- @dsCard group="${t.grupo}" name="${t.nombre}" -->
<!doctype html>
<html lang="es" data-lang="es">
<head>
<meta charset="utf-8">
<title>${t.nombre} — Angel Ghost</title>
<style>
${css.trim()}
</style>
</head>
<body>
${sprite}
  <div class="ds-cab">
    <h1>${t.nombre}</h1>
${regla ? `    <p class="ds-regla">${regla}</p>\n` : ""}    <p>Angel Ghost · ${t.grupo} — derivado de <code>docs/diseno</code> y <code>design-system.md</code>. Los dos temas, las dos lenguas (se muestra <code>es</code>).</p>
  </div>
  <div class="ds-temas">
${temas}
  </div>
</body>
</html>
`;
}

// ─────────────────────────────────────────────────────────────────────────────
// El bundle
// ─────────────────────────────────────────────────────────────────────────────

export function armar() {
  const ghost = readFileSync(join(DISENO, "assets/ghost.css"), "utf8");
  const maqueta = readFileSync(join(DISENO, "assets/maqueta.css"), "utf8");
  const kitStyle = (readFileSync(join(DISENO, "kit.html"), "utf8").match(
    /<style>([\s\S]*?)<\/style>/,
  ) ?? [, ""])[1];
  const mapaIconos = simbolos(readFileSync(join(DISENO, "assets/iconos.js"), "utf8"));
  const hojas = [ghost, maqueta, kitStyle];

  const salida = new Map();
  for (const t of TARJETAS) {
    salida.set(join("components", t.archivo), tarjeta(t, hojas, mapaIconos));
  }

  const sistema = readFileSync("design-system.md", "utf8");
  const version = (sistema.match(/^version:\s*([\d.]+)/m) ?? [, "?"])[1];

  salida.set(
    "styles.css",
    `/* ============================================================================
   Angel Ghost — hoja del proyecto en Claude Design.
   ESPEJO de docs/diseno/assets/ghost.css (design-system.md v${version}).
   NO se edita aquí ni allá: se edita el sistema y se regenera con
   \`node scripts/design-sync-bundle.mjs\`.
   Las tarjetas no dependen de esta hoja —cada una lleva en línea lo que
   necesita, para que se abra sola— y esta es el sistema entero, para el
   proyecto.
   ============================================================================ */
${ghost}`,
  );

  salida.set(
    "project.json",
    JSON.stringify(
      {
        app: "copiloto-consultor",
        name: "Angel Ghost",
        designSystem: `design-system.md v${version}`,
        projectId: null,
        publishedFiles: 0,
        lastPublished: null,
        nota:
          "projectId en null: NADA se ha publicado todavía. El destino se decide con el usuario " +
          "la primera vez que él invoque /design-sync, que es en el cierre de ciclo y después " +
          "del gate ⭐⭐ — jamás se adivina ni se crea un proyecto por cuenta propia.",
        generadoPor: "scripts/design-sync-bundle.mjs",
      },
      null,
      2,
    ) + "\n",
  );

  const indice = TARJETAS.map((t) => `| ${t.grupo} | ${t.nombre} | \`components/${t.archivo}\` |`).join("\n");
  salida.set(
    "README.md",
    `# design-sync — el bundle publicable de Angel Ghost

Espejo en el repo de lo que se publica en Claude Design. **No se escribe a mano: se genera.**

\`\`\`
design-system.md   →   design-sync/   →   el proyecto en Claude Design
(fuente de verdad)     (este bundle)      (vitrina; jamás se edita allá)
\`\`\`

- **Regenerar:** \`node scripts/design-sync-bundle.mjs\`
- **Gate:** \`tests/unit/design-sync-espejo.test.ts\` — el bundle del repo tiene que ser el que el
  generador emite hoy. Si tocas \`ghost.css\`, la maqueta o el generador y no regeneras, el gate se
  pone en rojo con el archivo y la línea.
- **Publicar** no es trabajo de este bundle ni de este script: lo hace \`/design-sync\`, que **solo
  invoca el usuario**, en el cierre de ciclo y **después del gate ⭐⭐** — no se publica un sistema
  que el usuario no haya juzgado. El destino vive en \`project.json\` y hoy está en \`null\` porque
  nunca se ha publicado.

Cada tarjeta es **autocontenida** (CSS y sprite en línea, cero CDNs), lleva su primera línea
\`<!-- @dsCard … -->\` —por donde Claude Design la indexa— y enseña el componente en **los dos
temas**, con las dos lenguas dentro (se muestra el español).

| Grupo | Tarjeta | Archivo |
|---|---|---|
${indice}

Derivado de \`design-system.md\` **v${version}** y de la maqueta aprobada en G-Diseño
(\`docs/diseno/\`). Nace en el sprint 001, el primero con UI, por la regla 16 de \`CLAUDE.md\`:
el bundle se actualiza en el MISMO PR que toca la UI, para que el cierre de ciclo sea un delta
pequeño y nunca una reconstrucción.
`,
  );

  return salida;
}

const esMain = process.argv[1] && process.argv[1].endsWith("design-sync-bundle.mjs");
if (esMain) {
  const salida = armar();
  const verificar = process.argv.includes("--verificar");
  let distintos = 0;
  for (const [rel, contenido] of salida) {
    const ruta = join(BUNDLE, rel);
    const hay = existsSync(ruta) ? readFileSync(ruta, "utf8") : null;
    if (hay === contenido) continue;
    distintos++;
    if (verificar) {
      console.log(`· deriva en ${ruta}${hay === null ? " (no existe)" : ""}`);
      continue;
    }
    mkdirSync(dirname(ruta), { recursive: true });
    writeFileSync(ruta, contenido);
    console.log(`${hay === null ? "nuevo" : "al día"}  ${ruta}`);
  }
  if (verificar) {
    console.log(distintos === 0 ? "design-sync: al día" : `design-sync: ${distintos} archivo(s) con deriva`);
    process.exit(distintos === 0 ? 0 : 1);
  }
  console.log(`design-sync: ${salida.size} archivos, ${distintos} escritos`);
}
