import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * **EL LOTE DEL HITO H-1 ESTÁ A CERO** (sprint 008, «Listo para el gate»).
 *
 * Los medios y bajos de los sprints 5, 6 y 7 se aplazaron al lote del hito y el sprint 008 los pagó todos, cada uno en la
 * fila de su artefacto de origen. Este gate impide que alguno vuelva a `deuda`: una fila con ese estado en esos tres
 * artefactos es rojo con su identificador. **Un pago dice dónde** (`pagado · PR #N` o un commit) y **un descarte dice por
 * qué** (`descartado · <razón>` de verdad, no una palabra): sin eso, «pagado» sería una etiqueta.
 *
 * ¿Puede fallar? Sí: volver una fila a «deuda · lote del hito», o escribir «descartado · no» sin razón, es rojo (demo en
 * rojo del sprint 008).
 */
const ARTEFACTOS = [5, 6, 7].map((n) => `sprints/SPRINT_00${n}-auditoria.md`);
const FILA = /^(?:\|\s*|-\s+)\*\*([CAMB]\d{1,2})\*\*/;

/** El estado de la fila: lo que va tras el último «·» de estado, según la palabra clave. */
function estados(texto: string): Map<string, string> {
  const hallados = new Map<string, string>();
  for (const linea of texto.split("\n")) {
    const id = FILA.exec(linea)?.[1];
    if (!id) continue;
    const celdas = linea.split("|").map((c) => c.trim()).filter(Boolean);
    const ultima = celdas[celdas.length - 1] ?? "";
    hallados.set(id, linea.includes("deuda") ? `deuda: ${ultima}` : ultima);
  }
  return hallados;
}

describe("el lote del hito está a cero", () => {
  for (const ruta of ARTEFACTOS) {
    const texto = readFileSync(ruta, "utf8");
    const filas = estados(texto);

    it(`${ruta} — hay filas que mirar`, () => {
      expect(filas.size, "el lector no encontró hallazgos").toBeGreaterThan(15);
    });

    it(`${ruta} — ninguna fila está en «deuda»`, () => {
      const enDeuda = [...filas].filter(([, e]) => e.startsWith("deuda:")).map(([id]) => id);
      expect(enDeuda, `hallazgos que vuelven a «deuda» (el lote del hito se pagó en el sprint 008): ${enDeuda.join(" · ")}`).toEqual([]);
    });

    it(`${ruta} — cada medio o bajo dice cómo se pagó o por qué se descartó`, () => {
      const sinPago: string[] = [];
      for (const linea of texto.split("\n")) {
        const id = FILA.exec(linea)?.[1];
        if (!id || !/^[MB]/.test(id)) continue;
        const pagado = /pagad[oa]\s*·[^|]*?(?:PR\s*#\d+|`[0-9a-f]{7,}`)/i.test(linea);
        const descartado = /descartad[oa]\s*·\s*[^|]{25,}/i.test(linea);
        if (!pagado && !descartado) sinPago.push(id);
      }
      expect(
        sinPago,
        `hallazgos sin su pago (\`pagado · PR #N\` o un commit) ni su descarte con razón (≥ 25 caracteres): ${sinPago.join(" · ")}`,
      ).toEqual([]);
    });
  }
});
