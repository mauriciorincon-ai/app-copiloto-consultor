import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * GATE de los TOKENS VETADOS COMO TEXTO (regla 5b del CLAUDE.md, `design-system.md` §7.2).
 *
 * `--ink-3` no alcanza AA sobre la superficie: es para separadores y ornamento, jamás para texto. La
 * regla 5b pide que eso **falle en test, no en axe al final**, y el `design-system.md` daba el barrido
 * por nacido en el S1. No existía: lo encontró la auditoría del `CLAUDE.md` del cierre del ciclo H1
 * (sprint 003). Nadie había escrito texto en `--ink-3`, pero nada lo impedía.
 *
 * La lista de clases NO se escribe aquí: se lee de §7.2, para que el sistema y el gate no se separen.
 * Además, cualquier `color` que apunte a `var(--ink-3)` —en CSS o en un `style` de React— cae igual.
 *
 * ¿Puede fallar? Sí: con un `style={{ color: "var(--ink-3)" }}` o un `className="text-ink-3"` plantados
 * en una pantalla (bitácora del sprint 003, fase 5).
 */
const SISTEMA = readFileSync("design-system.md", "utf8");
const SECCION = SISTEMA.split("### 7.2")[1]?.split("### 7.3")[0] ?? "";

/** `text-ink-3`, `text-surface*` → expresiones por palabra entera; el `*` es «y lo que siga». */
const VETADAS = [...SECCION.matchAll(/`(text-[\w-]+\*?)`/g)].map((m) => {
  const t = m[1];
  return t.endsWith("*")
    ? new RegExp(`(?<![\\w-])${t.slice(0, -1)}[\\w-]*`)
    : new RegExp(`(?<![\\w-])${t}(?![\\w-])`);
});

/** `color: var(--ink-3)` en CSS, o `color: "var(--ink-3)"` en un objeto de estilo. */
const COLOR_VETADO = /(?<![\w-])color\s*:\s*["'`]?\s*var\(--ink-3\)/;

function archivos(ruta: string): string[] {
  if (statSync(ruta).isFile()) return /\.(tsx?|css)$/.test(ruta) ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => archivos(join(ruta, n)));
}

describe("tokens vetados como texto (regla 5b)", () => {
  it("la lista sale del design-system (si §7.2 se queda sin clases, este gate no mide nada)", () => {
    expect(VETADAS.map(String)).toContain(String(/(?<![\w-])text-ink-3(?![\w-])/));
    expect(VETADAS.length).toBeGreaterThanOrEqual(4);
  });

  it("ningún texto de src/ usa un token vetado", () => {
    const hallazgos = archivos("src").flatMap((f) =>
      readFileSync(f, "utf8")
        .split("\n")
        .map((l, i) => [i + 1, l] as const)
        .filter(([, l]) => VETADAS.some((re) => re.test(l)) || COLOR_VETADO.test(l))
        .map(([n, l]) => `${relative(".", f)}:${n}  ${l.trim().slice(0, 90)}`),
    );
    expect(hallazgos, `texto en un token vetado:\n${hallazgos.join("\n")}`).toEqual([]);
  });
});
