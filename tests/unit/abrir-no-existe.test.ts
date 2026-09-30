import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * Gate del «ABRIR» QUE NO EXISTE (auditoría del S3, M8; ADR 015, enmienda 3).
 *
 * En *Notas* una reunión guardada se **exporta** o se **borra**; el contenido descifrado no cruza a la
 * pantalla (`contrato-con-lectores`). Abrirla solo se puede por la puerta local, con `ghost`. El manual,
 * la interfaz y la guía hablaban de «abrir una reunión» y mandaban al usuario a buscar un botón que no
 * está. Una línea que nombra `ghost` sí puede decir «abrir».
 *
 * ¿Puede fallar? Sí: nació en rojo con los textos de antes (bitácora).
 */
const TEXTOS = ["docs/MANUAL-DE-USO.md", "src/i18n/es.ts", "src/i18n/en.ts", "docs/GUIA-DE-PRUEBA.html", "docs/diseno/notas.html"];
const ABRIR = /abrir (una|la) reuni[oó]n|ábrela|como abrir en|opening a meeting|open a (saved )?meeting/i;

/** El comando `ghost` cerca de la frase (no la extensión `.ghost` de un archivo): ahí «abrir» sí existe. */
const GHOST = /(?<![.\w-])ghost\b/;

function conGhostCerca(linea: string, en: number): boolean {
  return GHOST.test(linea.slice(Math.max(0, en - 120), en + 120));
}

export function prometenAbrir(textos: Record<string, string>): string[] {
  return Object.entries(textos).flatMap(([ruta, t]) =>
    t.split("\n").flatMap((l, i) =>
      [...l.matchAll(new RegExp(ABRIR.source, "gi"))]
        .filter((m) => !conGhostCerca(l, m.index ?? 0))
        .map((m) => `${ruta}:${i + 1} · ${m[0]}`),
    ),
  );
}

describe("abrir-no-existe: en la pantalla una reunión guardada se exporta; abrir es de ghost", () => {
  const textos = Object.fromEntries(TEXTOS.map((r) => [r, readFileSync(r, "utf8")]));

  it("ningún texto manda a abrir una reunión en la pantalla", () => {
    expect(prometenAbrir(textos)).toEqual([]);
  });

  it("el rojo: una promesa plantada se nombra, y la línea de ghost no", () => {
    expect(prometenAbrir({ "x.md": "Vuelve a Notas y ábrela." })).toHaveLength(1);
    expect(prometenAbrir({ "x.md": "`ghost notas abrir` abre una reunión: abrir una reunión por la puerta." })).toEqual([]);
    expect(prometenAbrir({ "x.md": "El archivo paramo-azul.ghost está en Finder. Vuelve a Notas y ábrela." })).toHaveLength(1);
  });
});
