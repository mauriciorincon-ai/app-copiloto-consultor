import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { ALTO_COMPACTA, ALTO_AMPLIADA, DESDE_AMPLIADA } from "@/asa";
import { hayTauri, llamar } from "@/puente";

/**
 * El asa mueve la banda entre dos alturas que son del DISEÑO, no del código. Si el producto
 * decidiera las suyas, el gate de fidelidad compararía la banda contra un tamaño que la maqueta
 * no dibuja — y lo haría en verde, porque el gate compara imágenes del mismo tamaño.
 */
const MAQUETA = "docs/diseno/assets/ghost.css";

function token(nombre: string): number {
  const m = readFileSync(MAQUETA, "utf8").match(new RegExp(`${nombre}:\\s*(\\d+)px`));
  if (!m) throw new Error(`la maqueta no declara ${nombre}`);
  return Number(m[1]);
}

describe("el asa: dos alturas, y son las de la maqueta", () => {
  it("las alturas del producto son los tokens del diseño", () => {
    expect(ALTO_COMPACTA).toBe(token("--banda-h"));
    expect(ALTO_AMPLIADA).toBe(token("--banda-h-ampliada"));
  });

  it("el umbral de «ampliada» cae entre las dos, no fuera", () => {
    expect(DESDE_AMPLIADA).toBeGreaterThan(ALTO_COMPACTA);
    expect(DESDE_AMPLIADA).toBeLessThan(ALTO_AMPLIADA);
  });
});

describe("el puente con lo nativo", () => {
  it("fuera de Tauri no hay nadie al otro lado, y se dice en vez de estallar", async () => {
    expect(hayTauri()).toBe(false);
    await expect(llamar("ajustar_banda", { alto: 200 })).resolves.toBe(false);
  });
});
