import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { ALTO_COMPACTA, ALTO_AMPLIADA, DESDE_AMPLIADA, altoAlArrastrar } from "@/asa";
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

/**
 * **Arriba (sprint 004) el asa se arrastra hacia abajo para ampliar**: el borde clavado es el de
 * arriba. ¿Puede fallar? Sí: con la cuenta de abajo en los dos bordes, arrastrar hacia abajo la banda
 * de arriba la encoge (bitácora del sprint 004, fase 1).
 */
describe("el asa crece hacia donde hay sitio, en los dos bordes", () => {
  it("abajo, subir el puntero la amplía; arriba, bajarlo", () => {
    expect(altoAlArrastrar("abajo", 88, 800, 750)).toBe(138);
    expect(altoAlArrastrar("arriba", 88, 100, 150)).toBe(138);
    expect(altoAlArrastrar("arriba", 88, 100, 60)).toBe(ALTO_COMPACTA);
  });

  it("siempre entre las dos alturas del diseño", () => {
    for (const borde of ["arriba", "abajo"] as const) {
      expect(altoAlArrastrar(borde, 88, 500, borde === "arriba" ? 2000 : -2000)).toBe(ALTO_AMPLIADA);
      expect(altoAlArrastrar(borde, 200, 500, borde === "arriba" ? -2000 : 2000)).toBe(ALTO_COMPACTA);
    }
  });
});

describe("el puente con lo nativo", () => {
  it("fuera de Tauri no hay nadie al otro lado, y se dice en vez de estallar", async () => {
    expect(hayTauri()).toBe(false);
    await expect(llamar("ajustar_banda", { alto: 200 })).resolves.toBe(false);
  });
});
