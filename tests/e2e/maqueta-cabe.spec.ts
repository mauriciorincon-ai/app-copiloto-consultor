import { test, expect } from "@playwright/test";
import { readdirSync } from "node:fs";
import { resolve } from "node:path";

/**
 * GATE — **NINGÚN ESTADO DE NINGUNA MAQUETA SE SALE DE SU VENTANA** (sprint 002, fase 3; fila 6 de la
 * mirada 17-quater, tomada con la opción recomendada).
 *
 * **Por qué existe.** El estado «vigilancia» de Sesión llevaba seis días cortado —la franja «Tu
 * protección propia sigue en pie» se veía a medias— y nadie lo vio: ni las pasadas de capturas, ni la
 * mirada, ni yo. Y al buscarlo aparecieron siete más, míos, en los estados «sprint 2»: se pasaban 5-20
 * px del área que desplaza y la última tarjeta quedaba pegada al borde. El recorrido que los encontró
 * medía mal la primera vez —el borde de la ventana y no el área que desplaza—, y por eso este gate
 * mide **las dos cosas**:
 *
 *  1. **el área que desplaza** (`.contenido`) no desborda más de 1 px — la misma tolerancia que el
 *     gate de fidelidad usa con el producto;
 *  2. **nada se sale del marco** (`.ventana` o `.banda`), salvo lo que un contenedor de dentro recorta
 *     a propósito (el «…» del transcript).
 *
 * Cubre toda maqueta que tenga estados y un marco: hoy, 164 casos (cada estado en los dos idiomas).
 * `index.html` y `kit.html` no tienen estados y quedan fuera, dicho aquí.
 *
 * ¿Puede fallar? Sí: nació en ROJO con «vigilancia», que es su demo de nacimiento. Y se vio fallar
 * también devolviendo a `.titulo .sub` su tope de 60ch (bitácora del sprint 002).
 */

/**
 * **La deuda, caso por caso.** Falla en los dos sentidos, como la de los campos sin lector: si
 * aparece un desborde nuevo no se puede colar, y si uno se paga y sigue aquí, también falla.
 */
const DEUDA: Record<string, string> = {
  "sesion.html · vigilancia · es":
    "hallazgo medio de la fase 3 del sprint 002 (docs/diseno/sesion.html:155-202): la franja «Tu protección propia sigue en pie» y sus botones, 48 px por debajo del área que desplaza, desde la Etapa de Diseño. Pago: fase 4, antes de construir el radar, con su mirada",
  "sesion.html · vigilancia · en": "el mismo hallazgo, en inglés: 13 px",
};

const DISENO = resolve("docs/diseno");
const PAGINAS = readdirSync(DISENO).filter((f) => f.endsWith(".html"));

test.use({ viewport: { width: 1440, height: 1400 } });

test("ningún estado de ninguna maqueta se sale de su ventana", async ({
  page,
}, info) => {
  // Las maquetas no dependen del proyecto de Playwright: con correrlo en uno basta.
  test.skip(
    info.project.name !== "ventana-principal",
    "se corre una vez, no una por proyecto",
  );
  // **Y se mide con las fuentes de macOS.** Las maquetas usan Avenir Next, Charter y Menlo, que un
  // runner de Linux no tiene: allí todo envuelve distinto y el gate dio 15 desbordes que en un Mac
  // no existen (su primera corrida en CI). Corre en el job `build-escritorio`, que es macOS, con su
  // propia conclusión.
  test.skip(
    process.platform !== "darwin",
    "las maquetas se miden con las fuentes de macOS: corre en build-escritorio",
  );
  test.setTimeout(300_000);

  const desbordes = new Map<string, string>();
  let casos = 0;
  for (const pagina of PAGINAS) {
    await page.goto("file://" + resolve(DISENO, pagina));
    const estados = await page.$$eval(".mq-bar button[data-estado]", (bs) =>
      bs.map((b) => (b as HTMLElement).dataset.estado as string),
    );
    for (const estado of estados) {
      for (const idioma of ["es", "en"]) {
        await page.click(`.mq-bar button[data-estado="${estado}"]`);
        await page.click(`.mq-bar button[data-lang-set="${idioma}"]`);
        const fallos = await page.evaluate(() => {
          const out: string[] = [];
          const marcos = [
            ...document.querySelectorAll(".ventana, .banda"),
          ].filter((v) => v.getBoundingClientRect().height > 0);
          for (const v of marcos) {
            const caja = v.getBoundingClientRect();
            for (const c of v.querySelectorAll(".contenido")) {
              const dv = c.scrollHeight - c.clientHeight;
              const dh = c.scrollWidth - c.clientWidth;
              if (dv > 1 || dh > 1)
                out.push(
                  `el área que desplaza +${dv} px de alto, +${dh} px de ancho`,
                );
            }
            for (const n of v.querySelectorAll("*")) {
              const b = n.getBoundingClientRect();
              if (!b.height || !b.width) continue;
              let p = n.parentElement;
              let recortado = false;
              while (p && p !== v) {
                const o = getComputedStyle(p);
                if (o.overflowX !== "visible" || o.overflowY !== "visible") {
                  recortado = true;
                  break;
                }
                p = p.parentElement;
              }
              if (recortado) continue;
              const abajo = b.bottom - caja.bottom;
              const derecha = b.right - caja.right;
              if (abajo > 1 || derecha > 1) {
                out.push(
                  `«${(n.textContent ?? "").trim().slice(0, 30)}» +${Math.round(abajo)} px abajo, +${Math.round(derecha)} px a la derecha`,
                );
                break;
              }
            }
          }
          return out;
        });
        casos++;
        if (fallos.length)
          desbordes.set(
            `${pagina} · ${estado} · ${idioma}`,
            fallos.join(" | "),
          );
      }
    }
  }

  expect(casos, "un gate que no mide nada no es un gate").toBeGreaterThan(100);
  const nuevos = [...desbordes]
    .filter(([k]) => !(k in DEUDA))
    .map(([k, v]) => `${k}: ${v}`);
  expect(
    nuevos,
    `estados que se salen de su ventana:\n${nuevos.join("\n")}`,
  ).toEqual([]);
  const pagados = Object.keys(DEUDA).filter((k) => !desbordes.has(k));
  expect(
    pagados,
    `DEUDA declara desbordes que ya caben: quítalos de la lista\n${pagados.join("\n")}`,
  ).toEqual([]);
});
