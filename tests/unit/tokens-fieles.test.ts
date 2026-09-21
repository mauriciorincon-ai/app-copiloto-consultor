import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * Gate de TOKENS (design-system.md §9 — «contrato con el código futuro»).
 *
 * El contrato dice que los tokens pasan **tal cual** de la maqueta al producto y que los
 * componentes no llevan valores mágicos. Una frase así solo vale si algo la vigila: sin gate,
 * el día que alguien afine un color en el producto la maqueta deja de ser la fuente de verdad
 * y el gate de FIDELIDAD pasa a comparar contra un documento desactualizado.
 *
 * Compara token por token los tres bloques de `ghost.css` (`:root`, dark, light) contra
 * `src/index.css`. No exige que el archivo sea idéntico —el producto añade el puente a
 * Tailwind y su base— sino que **todo token de la maqueta exista en el producto con el mismo
 * valor**.
 *
 * ¿Puede fallar? Sí: ninguna regla previa mira `src/index.css` (eslint no lee CSS; los otros
 * gates miran `docs/diseno/`). Demo en rojo registrada en la bitácora del sprint.
 */
const MAQUETA = "docs/diseno/assets/ghost.css";
const PRODUCTO = "src/index.css";

/**
 * Tokens que el producto NO hereda: son medidas de la VENTANA, que en producto las fija Tauri.
 * El webview ocupa la ventana entera, así que su CSS no las necesita — y exigirlas aquí sería
 * un gate imposible de cumplir. Quedan vigiladas por el otro lado: `src-tauri/` lee las alturas
 * de la banda desde `ghost.css` y falla si se separan (fase 1 del sprint 001).
 */
const NO_APLICAN = new Set([
  "--panel-w",
  "--panel-h",
  "--panel-h-transcript",
  "--panel-h-max",
  "--principal-w",
  "--principal-h",
  "--banda-h",
  "--banda-h-ampliada",
  "--banda-h-voz",
]);

/** Extrae `--token: valor` de los bloques de tokens (los de `:root` y `[data-theme]`). */
function tokens(css: string, archivo: string): Map<string, string> {
  const mapa = new Map<string, string>();
  const bloques = css.matchAll(/(^|\n)(:root[^{]*|html\[data-theme="[a-z]+"\][^{]*)\{([^}]*)\}/g);
  for (const b of bloques) {
    const selector = b[2].trim();
    for (const linea of b[3].split("\n")) {
      const m = linea.match(/^\s*(--[a-z0-9-]+)\s*:\s*([^;]+);/i);
      if (!m) continue;
      const [, nombre, valor] = m;
      if (NO_APLICAN.has(nombre)) continue;
      // El tema se distingue por selector: `--bg` vale distinto en oscuro y en claro.
      const clave = `${selector.includes("light") ? "light" : selector.includes("dark") ? "dark" : "base"} ${nombre}`;
      mapa.set(clave, valor.trim());
    }
  }
  if (mapa.size === 0) throw new Error(`${archivo}: no se encontró ningún token (¿cambió la forma del archivo?)`);
  return mapa;
}

describe("tokens fieles a la maqueta (design-system §9)", () => {
  const deLaMaqueta = tokens(readFileSync(MAQUETA, "utf8"), MAQUETA);
  const delProducto = tokens(readFileSync(PRODUCTO, "utf8"), PRODUCTO);

  it("lee tokens de los dos lados (un gate que no lee nada no es un gate)", () => {
    expect(deLaMaqueta.size).toBeGreaterThan(40);
    expect(delProducto.size).toBeGreaterThan(40);
  });

  it("todo token de la maqueta existe en el producto con el MISMO valor", () => {
    const divergencias: string[] = [];
    for (const [clave, valor] of deLaMaqueta) {
      const suyo = delProducto.get(clave);
      if (suyo === undefined) divergencias.push(`${clave}: falta en ${PRODUCTO}`);
      else if (suyo !== valor) divergencias.push(`${clave}: maqueta «${valor}» ≠ producto «${suyo}»`);
    }
    expect(
      divergencias,
      `los tokens del producto se separaron de la maqueta:\n${divergencias.join("\n")}`,
    ).toEqual([]);
  });
});
