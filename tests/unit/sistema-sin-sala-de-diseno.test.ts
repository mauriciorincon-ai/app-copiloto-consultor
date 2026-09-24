import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * Gate de la FRONTERA entre el sistema y la sala de diseño.
 *
 * Desde el sprint 001 el producto **importa** `ghost.css` en vez de copiarlo: una sola fuente, y
 * la banda construida comparte literalmente el CSS de la maqueta. El precio de esa decisión es
 * que cualquier regla que entre a `ghost.css` **viaja al binario**. Y al lado vive
 * `maqueta.css`, que es el marco de la sala de diseño —barras de estado, escritorio de
 * referencia, tablas comparativas— y jamás debe llegar al producto.
 *
 * La frontera es el prefijo `.mq-`. Si aparece en `ghost.css`, la sala de diseño se coló.
 *
 * ¿Puede fallar? Sí: ningún otro gate mira los selectores de `ghost.css` (el de autocontenida
 * mira URLs; el de tokens, variables). Demo en rojo registrada en la bitácora.
 */
const SISTEMA = "docs/diseno/assets/ghost.css";
const SALA = "docs/diseno/assets/maqueta.css";

describe("el sistema no arrastra la sala de diseño al producto", () => {
  const sistema = readFileSync(SISTEMA, "utf8");

  it("los dos archivos existen y son distintos", () => {
    expect(sistema.length).toBeGreaterThan(1000);
    expect(readFileSync(SALA, "utf8")).not.toEqual(sistema);
  });

  it("`ghost.css` no declara un solo selector de la sala de diseño", () => {
    const intrusos = sistema
      .split("\n")
      .map((linea, i) => [i + 1, linea] as const)
      .filter(([, linea]) => /(^|[\s,>+~])\.mq-[a-z-]+/.test(linea))
      .map(([n, linea]) => `${SISTEMA}:${n}  ${linea.trim().slice(0, 90)}`);
    expect(
      intrusos,
      `el producto importa este archivo: lo de la sala de diseño viajaría al binario\n${intrusos.join("\n")}`,
    ).toEqual([]);
  });

  it("`maqueta.css` sí los tiene — si no, este gate estaría mirando al vacío", () => {
    expect(/\.mq-/.test(readFileSync(SALA, "utf8"))).toBe(true);
  });
});
