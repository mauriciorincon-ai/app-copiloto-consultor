import { readdirSync, readFileSync, statSync, existsSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Gate de MAQUETA AUTOCONTENIDA (orden de diseño, entregable 2; regla 12 del CLAUDE.md):
 * `docs/diseno/` se abre con doble clic y no toca la red — cero CDNs, cero frameworks,
 * cero fuentes remotas, cero enlaces a producción (regla 17). Si un recurso externo
 * entrara, la maqueta rendería distinto sin conexión y el gate visual dejaría de ser local.
 *
 * Demo en rojo (regla 15): plantar `<script src="https://cdn...">` en un HTML lo pone rojo —
 * registrada en sprints/ETAPA-DISENO-implementation-log.md.
 */
const RAIZ = "docs/diseno";
const PROHIBIDO: Array<[RegExp, string]> = [
  [/https?:\/\//i, "URL absoluta (http/https)"],
  [/<script[^>]+src\s*=\s*["'](?!\.\/|assets\/|[a-z0-9_-]+\.js)/i, "script externo"],
  [/<link[^>]+href\s*=\s*["'](?!\.\/|assets\/|[a-z0-9_-]+\.(css|html))/i, "stylesheet externa"],
  [/@import\s+url\(/i, "@import remoto"],
  [/vercel[.]app|workers[.]dev|pages[.]dev/i, "dominio de despliegue (regla 17, cero enlaces)"],
];

function archivos(ruta: string): string[] {
  if (!existsSync(ruta)) return [];
  if (statSync(ruta).isFile()) return /\.(html|css|js)$/.test(ruta) ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => archivos(join(ruta, n)));
}

describe("maqueta autocontenida (docs/diseno)", () => {
  const lista = archivos(RAIZ);

  it("existe la maqueta y tiene archivos que inspeccionar", () => {
    expect(lista.length).toBeGreaterThan(0);
  });

  it("ningún HTML/CSS/JS de la maqueta referencia la red", () => {
    const hallazgos: string[] = [];
    for (const f of lista) {
      readFileSync(f, "utf8").split("\n").forEach((linea, i) => {
        if (linea.includes("autocontenida:cita")) return; // línea que CITA un patrón sin usarlo
        for (const [re, que] of PROHIBIDO) if (re.test(linea)) hallazgos.push(`${relative(".", f)}:${i + 1}  ${que}  →  ${linea.trim().slice(0, 100)}`);
      });
    }
    expect(hallazgos, `referencias externas:\n${hallazgos.join("\n")}`).toEqual([]);
  });
});
