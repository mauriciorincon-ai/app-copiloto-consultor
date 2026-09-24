import { execFileSync } from "node:child_process";
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Gate del ESPEJO (regla 16 de CLAUDE.md · `/design-sync`).
 *
 * `design-sync/` es el bundle publicable del design system y tiene un solo trabajo: ser espejo
 * del sistema. Un espejo que se copia a mano se desvía en el primer sprint —alguien cambia un hex
 * en `ghost.css`, la tarjeta se queda con el viejo, y la vitrina enseña un sistema que la app ya
 * no usa—, y lo peor es que nadie se entera: la vitrina no tiene tests, no tiene CI y nadie la
 * abre entre ciclo y ciclo.
 *
 * Aquí el bundle se genera (`scripts/design-sync-bundle.mjs`) y este gate exige que **el bundle
 * del repo sea el que el generador emite hoy**. Es el mismo trato que el contrato Rust→TS del
 * sprint: el que emite escribe, el repo guarda, el gate compara.
 *
 * ¿Puede fallar? Sí, y de dos maneras que pasan de verdad: tocar `ghost.css`, la maqueta o el
 * `design-system.md` sin regenerar, y editar una tarjeta a mano. Las dos demos en rojo están
 * registradas en la bitácora del sprint 001.
 */
const BUNDLE = "design-sync";

function tarjetas(dir = join(BUNDLE, "components")): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory()
      ? tarjetas(p)
      : p.endsWith(".html")
        ? [p]
        : [];
  });
}

describe("design-sync — el bundle es espejo del sistema", () => {
  it("el bundle del repo es el que el generador emite hoy", () => {
    let salida = "";
    let cayo = false;
    try {
      salida = execFileSync(
        "node",
        ["scripts/design-sync-bundle.mjs", "--verificar"],
        {
          encoding: "utf8",
        },
      );
    } catch (e) {
      cayo = true;
      salida = String((e as { stdout?: string }).stdout ?? e);
    }
    expect(
      cayo,
      `el bundle derivó del sistema. Regenéralo con \`node scripts/design-sync-bundle.mjs\`:\n${salida}`,
    ).toBe(false);
  });

  const fichas = tarjetas();

  it("hay tarjetas que mirar (un gate sobre una carpeta vacía no es un gate)", () => {
    expect(fichas.length).toBeGreaterThanOrEqual(10);
  });

  it.each(fichas)(
    "%s abre con la línea exacta que Claude Design indexa",
    (ruta) => {
      const primera = readFileSync(ruta, "utf8").split("\n")[0];
      // Sin esta línea —y con ESTA forma— la tarjeta sencillamente no aparece en el proyecto.
      expect(primera).toMatch(/^<!-- @dsCard group="[^"]+" name="[^"]+" -->$/);
    },
  );

  it.each(fichas)(
    "%s es autocontenida: cero CDNs, cero recursos externos",
    (ruta) => {
      const html = readFileSync(ruta, "utf8");
      const fuera = [...html.matchAll(/(?:src|href)="((?!#)[^"]*)"/g)].map(
        (m) => m[1],
      );
      expect(
        fuera,
        `una tarjeta que pide algo fuera no se puede publicar:\n${fuera.join("\n")}`,
      ).toEqual([]);
    },
  );

  it.each(fichas)("%s lleva dentro los tokens y los dos temas", (ruta) => {
    const html = readFileSync(ruta, "utf8");
    expect(html).toContain("--halo:");
    expect(html).toContain(".tema.claro");
    expect(html).toContain(".tema.oscuro");
    // Bilingüe, con el otro idioma escondido por CSS y no borrado: la app es bilingüe y su
    // vitrina también lo es.
    expect(html).toContain('[data-lang="es"] [lang="en"]');
  });

  it("el destino no se inventa: mientras nada se ha publicado, projectId va en null", () => {
    const p = JSON.parse(readFileSync(join(BUNDLE, "project.json"), "utf8"));
    expect(p.lastPublished === null).toBe(p.projectId === null);
    expect(p.publishedFiles).toBe(0);
  });
});
