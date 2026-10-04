// Gate de CONTROLADORES (kit v1.32.0, regla 22 del kit; en esta app, regla 23 «los controles de la
// maqueta tienen su script»): todo control dibujado en la maqueta tiene su script cargado. Origen
// (kit): en la Etapa de Diseño de Big-D la ficha del nivel 2 no cargó su script desde la mirada 2 y
// cuatro miradas no lo vieron — una captura de un panel cerrado «mide bien».
//
// Endurecido para esta casa (sprint 004, fase 0): la sala de diseño de cada página es la barra
// `.mq-bar`, y `assets/maqueta.js` solo atiende tres cosas: `data-estado`, `data-theme-set` y
// `data-lang-set`. Un botón de la barra sin ninguna de las tres se dibuja y no hace nada. La otra
// mitad —que cada clic CAMBIE algo de verdad— la mira `tests/e2e/maqueta-interaccion.spec.ts`.
//
// ¿Puede fallar? Sí: nació en rojo quitando el `<script src="assets/maqueta.js">` de `ia.html`
// (bitácora del sprint 004, fase 0).
import { describe, expect, it } from "vitest";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join, dirname, resolve } from "node:path";

const DIR = resolve(process.cwd(), "docs/diseno");
const paginas = existsSync(DIR) ? readdirSync(DIR).filter((f) => f.endsWith(".html")) : [];

const CONTROL = /<(button|select|input|details)\b[^>]*>|\brole="(button|switch|tab|slider)"|\bdata-(accion|controlador|paso|lang-set)=/g;
const SCRIPT = /<script\b[^>]*\bsrc="([^"]+)"[^>]*>|<script\b(?![^>]*\bsrc=)[^>]*>([\s\S]*?)<\/script>/g;
/** Lo único que `assets/maqueta.js` sabe atender en un botón de la barra. */
const ATENDIDO = /\bdata-(estado|theme-set|lang-set)="[^"]+"/;

describe("maqueta: controladores con script cargado", () => {
  it("hay maqueta que mirar", () => expect(paginas.length).toBeGreaterThan(10));

  for (const pagina of paginas) {
    it(`${pagina}: si dibuja controles, carga al menos un script y todo src existe`, () => {
      const html = readFileSync(join(DIR, pagina), "utf8");
      const controles = html.match(CONTROL) ?? [];
      const scripts = [...html.matchAll(SCRIPT)];
      const srcs = scripts.map((m) => m[1]).filter((s): s is string => Boolean(s));
      for (const src of srcs) {
        if (/^https?:/.test(src)) throw new Error(`${pagina}: script externo prohibido (${src}) — la maqueta es autocontenida`);
        expect(existsSync(join(dirname(join(DIR, pagina)), src)), `${pagina}: falta el script ${src}`).toBe(true);
      }
      // <details> se abre solo; los demás controles exigen un script (externo existente o inline no vacío).
      const interactivos = controles.filter((c) => !c.startsWith("<details"));
      if (interactivos.length > 0) {
        const inline = scripts.some((m) => (m[2] ?? "").trim().length > 0);
        expect(srcs.length > 0 || inline, `${pagina}: ${interactivos.length} control(es) dibujado(s) y ningún script cargado`).toBe(true);
      }
    });

    it(`${pagina}: la sala de diseño carga maqueta.js y cada botón de su barra hace algo que maqueta.js atiende`, () => {
      const html = readFileSync(join(DIR, pagina), "utf8");
      const barra = /<div class="mq-bar"[\s\S]*?\n<\/div>/.exec(html)?.[0];
      if (!barra) return; // sin sala de diseño (no la hay en ninguna página hoy, pero no se inventa)
      expect(html, `${pagina}: tiene barra y no carga assets/maqueta.js`).toMatch(/<script src="assets\/maqueta\.js"><\/script>/);
      const sordos = (barra.match(/<button\b[^>]*>/g) ?? []).filter((b) => !ATENDIDO.test(b));
      expect(sordos, `${pagina}: botones de la barra que maqueta.js no atiende`).toEqual([]);
    });
  }
});
