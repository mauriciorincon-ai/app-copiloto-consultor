import { readdirSync, readFileSync, statSync, existsSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Gate de VOCABULARIO (regla dura 6 del CLAUDE.md · estándar 4-T): ningún copy, maqueta,
 * design system ni README de esta app dice «indetectable», «stealth», «trampa» ni
 * equivalentes. La app protege una ventana con un flag del sistema; no evade nada.
 *
 * Nace en la Etapa de Diseño porque la maqueta es un entregable versionado y público.
 * Demo en rojo (regla 15): plantar «indetectable» en docs/diseno/panel.html lo pone rojo —
 * registrada en sprints/ETAPA-DISENO-implementation-log.md. Cuando exista `src/`, el
 * barrido se extiende a él (mismo test, otra raíz).
 */
const RAICES = ["docs/diseno", "design-system.md", "README.md", "docs/MANUAL-DE-USO.md"];
const EXT = /\.(html|css|js|md|json|txt)$/;

// Términos vetados (es/en). Se comparan en minúsculas y sin tildes.
const VETADOS = [
  "indetectable", "undetectable", "stealth", "trampa", "tramposo", "cheat", "cheating",
  "enganar", "enganamos", "evadir deteccion", "evade detection", "anti-deteccion",
  "anti-detection", "sin que se den cuenta", "modo invisible", "invisible mode",
];

function normaliza(s: string): string {
  return s.normalize("NFD").replace(/[̀-ͯ]/g, "").toLowerCase();
}

function archivos(ruta: string): string[] {
  if (!existsSync(ruta)) return [];
  if (statSync(ruta).isFile()) return EXT.test(ruta) ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => archivos(join(ruta, n)));
}

describe("vocabulario vetado (regla dura 6)", () => {
  const lista = RAICES.flatMap(archivos);

  it("inspecciona al menos un archivo (un gate que no lee nada no es un gate)", () => {
    expect(lista.length).toBeGreaterThan(0);
  });

  it("ningún archivo usa vocabulario de «trampa» o «indetectable»", () => {
    const hallazgos: string[] = [];
    for (const f of lista) {
      readFileSync(f, "utf8").split("\n").forEach((linea, i) => {
        // Este propio test y las líneas que CITAN la regla se marcan con vocabulario:cita.
        if (linea.includes("vocabulario:cita")) return;
        const n = normaliza(linea);
        for (const t of VETADOS) if (n.includes(t)) hallazgos.push(`${relative(".", f)}:${i + 1}  «${t}»`);
      });
    }
    expect(hallazgos, `vocabulario vetado:\n${hallazgos.join("\n")}`).toEqual([]);
  });
});
