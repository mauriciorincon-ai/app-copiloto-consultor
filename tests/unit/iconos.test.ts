import { readFileSync, readdirSync } from "node:fs";
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
