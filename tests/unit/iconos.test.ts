import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { SPRITE, idsDelSprite } from "@/componentes/Iconos";

/**
 * Gate de ICONOS: **todo icono que el producto pide existe en el sprite de la maqueta**.
 *
 * `<use href="#i-lo-que-sea">` con un id que no existe no lanza ningún error: dibuja **nada**. El
 * estado se queda sin su símbolo y pasa a ser solo texto y color — justo lo que la regla del
 * daltonismo leve prohíbe— y no hay traza de ello en ninguna consola. Un renombre en la maqueta
 * bastaría; por eso esto se comprueba en vez de confiarse.
 *
 * ¿Puede fallar? Sí: ninguna otra regla cruza `src/componentes/` con el sprite.
 */
const COMPONENTES = "src/componentes";

function iconosPedidos(): { archivo: string; id: string }[] {
  return readdirSync(COMPONENTES)
    .filter((n) => n.endsWith(".tsx"))
    .flatMap((n) => {
      const texto = readFileSync(join(COMPONENTES, n), "utf8");
      return [...texto.matchAll(/id=(?:"|\{`?)(i-[a-z0-9-]+)/g)].map((m) => ({
        archivo: n,
        id: m[1],
      }));
    });
}

describe("iconos: el producto no pide símbolos que no existen", () => {
  it("el sprite se extrajo de la maqueta y trae símbolos", () => {
    expect(SPRITE.startsWith("<svg")).toBe(true);
    expect(idsDelSprite().length).toBeGreaterThan(30);
  });

  it("todo icono que piden los componentes está en el sprite", () => {
    const disponibles = new Set(idsDelSprite());
    const pedidos = iconosPedidos();
    expect(pedidos.length, "ningún componente pide iconos: ¿cambió la forma del código?").toBeGreaterThan(5);

    const huerfanos = pedidos
      .filter(({ id }) => !disponibles.has(id))
      .map(({ archivo, id }) => `${archivo} pide «${id}», que el sprite no declara`);
    expect(
      huerfanos,
      `un <use> sin símbolo dibuja NADA, en silencio:\n${huerfanos.join("\n")}`,
    ).toEqual([]);
  });
});

/**
 * Gate hermano: **`relleno` solo vale sobre un símbolo que decide su propio `fill`.**
 *
 * `.ic.relleno` es una regla de CSS que pone `fill: currentColor; stroke: none` sobre el `<svg>`. Los
 * símbolos del sprite son de dos familias, y la diferencia no se ve en el nombre:
 *
 *  · **Los que se dibujan solos** —`i-check-circle`, `i-alert`, `i-rayo`…— llevan sus propios
 *    `fill="currentColor"` y `stroke="var(--bg)"` en cada hijo. Los atributos del elemento le ganan
 *    a lo heredado, así que `relleno` no les hace nada: ya estaban rellenos por diseño.
 *  · **Los de trazo** —`i-voz`, `i-doc`, `i-candado`, `i-reloj`, `i-auriculares-off`— no llevan
 *    atributos: heredan del CSS. Con `relleno`, sus líneas abiertas **desaparecen** y sus formas
 *    cerradas se convierten en una mancha.
 *
 * **Y no es un problema estético.** `i-voz` son cinco líneas verticales: relleno, no dibuja NADA, y
 * el estado se queda en texto y color, que es justo lo que la regla del daltonismo leve prohíbe.
 * `i-auriculares-off` es peor: relleno pierde la barra tachada y se convierte en unos auriculares
 * **conectados** — el icono dice lo contrario del estado que acompaña. Ninguno de los dos casos da
 * error en ninguna consola.
 *
 * *(Origen: sprint 002, mirada 16. Los dos los cometí maquetando el modo solo audio y los encontró
 * la pasada de capturas leída como imagen, no un número. Al escribir este gate aparecieron siete
 * más, de sprints anteriores, en cinco pantallas ya aprobadas.)*
 *
 * ¿Puede fallar? Sí, y falló: con los nueve usos que había al escribirlo.
 */
const DONDE_SE_USA = ["docs/diseno", "design-sync", "src"];

/** Los hijos dibujables de cada símbolo, con sus atributos tal cual. */
function simbolos(sprite: string): Map<string, string[]> {
  const m = new Map<string, string[]>();
  for (const s of sprite.matchAll(/<symbol id="(i-[a-z0-9-]+)"[^>]*>([\s\S]*?)<\/symbol>/g)) {
    m.set(s[1], [...s[2].matchAll(/<(path|circle|rect|ellipse|polygon|polyline|line)\b[^>]*>/g)].map((c) => c[0]));
  }
  return m;
}

/**
 * Un símbolo aguanta `relleno` si **todos** sus hijos declaran su propio `fill`. Basta con que uno
 * herede para que la regla del CSS lo borre o lo convierta en mancha.
 */
function aguantaRelleno(hijos: string[]): boolean {
  return hijos.length > 0 && hijos.every((h) => /\sfill="/.test(h));
}

function archivos(ruta: string): string[] {
  if (!existsSync(ruta)) return [];
  if (statSync(ruta).isFile()) return /\.(html|tsx)$/.test(ruta) ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => archivos(join(ruta, n)));
}

/** Cada sitio donde se pide un icono RELLENO, en las dos sintaxis: la maqueta y React. */
function usosConRelleno(): { donde: string; id: string }[] {
  const salida: { donde: string; id: string }[] = [];
  for (const f of DONDE_SE_USA.flatMap(archivos)) {
    readFileSync(f, "utf8").split("\n").forEach((linea, i) => {
      for (const m of linea.matchAll(/class="ic[^"]*\brelleno\b[^"]*"\s*>\s*<use href="#(i-[a-z0-9-]+)"/g)) {
        salida.push({ donde: `${f}:${i + 1}`, id: m[1] });
      }
      for (const m of linea.matchAll(/<Ic\b([^>]*)\/?>/g)) {
        const props = m[1];
        const id = /id="(i-[a-z0-9-]+)"/.exec(props);
        if (id && /\brelleno\b/.test(props)) salida.push({ donde: `${f}:${i + 1}`, id: id[1] });
      }
    });
  }
  return salida;
}

describe("un icono relleno tiene que dibujar algo", () => {
  const defs = simbolos(SPRITE);

  it("el sprite se dejó leer (un gate sobre cero símbolos no es un gate)", () => {
    expect(defs.size).toBeGreaterThan(30);
    expect(aguantaRelleno(defs.get("i-check-circle") ?? [])).toBe(true);
    expect(aguantaRelleno(defs.get("i-voz") ?? [])).toBe(false);
  });

  it("ningún icono de trazo se pide con `relleno`", () => {
    const rotos = usosConRelleno()
      .filter(({ id }) => defs.has(id) && !aguantaRelleno(defs.get(id) as string[]))
      .map(({ donde, id }) => `${donde}  ${id}`);
    expect(
      rotos,
      `estos iconos son de TRAZO y se piden rellenos: sus líneas desaparecen y lo que queda es una ` +
        `mancha o nada. El estado se queda en texto y color, que la regla del daltonismo leve ` +
        `prohíbe.\n  ${rotos.join("\n  ")}\n` +
        `Quita \`relleno\` de esos sitios, o dale al símbolo sus propios atributos \`fill\`.`,
    ).toEqual([]);
  });
});
