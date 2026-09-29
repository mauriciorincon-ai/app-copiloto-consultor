import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { EXTERNOS } from "@/ia";

/**
 * Gate de la RETENCIÓN DE CADA PROVEEDOR (auditoría del S3, M3; regla dura 2; ADR 011, enmienda).
 *
 * La regla dura 2 deja salir texto anonimizado solo «bajo proveedor con no-retención». Eso no se puede
 * afirmar sin haber leído qué hace cada proveedor con lo que recibe. Cada proveedor externo que la app
 * ofrece (`EXTERNOS`) tiene su fila en la tabla del ADR 011, con la fecha en que se leyó y la fuente.
 * Un proveedor nuevo sin fila pone esto en rojo.
 *
 * ¿Puede fallar? Sí: nació en rojo, sin tabla. Y su demo está abajo: un proveedor inventado no tiene fila.
 */
const ADR = "decisions/011-proveedores-del-modelo-y-minimizacion.md";

function tabla(adr: string): string[] {
  const enmienda = adr.split(/^## Enmienda \(\d{4}-\d{2}-\d{2}\) — cuánto guarda cada proveedor/m)[1] ?? "";
  return enmienda.split("\n").filter((l) => l.startsWith("| **"));
}

export function sinFila(nombres: string[], filas: string[]): string[] {
  return nombres.filter(
    (nombre) =>
      !filas.some(
        (f) => f.startsWith(`| **${nombre}**`) && /\b\d{4}-\d{2}-\d{2}\b/.test(f) && /https:\/\/\S+/.test(f),
      ),
  );
}

describe("proveedores-con-su-retencion: cada proveedor externo dice cuánto guarda, con fuente y fecha", () => {
  const filas = tabla(readFileSync(ADR, "utf8"));

  it("el ADR 011 tiene la tabla (un gate que no lee nada no es un gate)", () => {
    expect(filas.length).toBeGreaterThanOrEqual(EXTERNOS.length);
  });

  it("cada proveedor que la app ofrece tiene fila con fecha y URL", () => {
    expect(sinFila(EXTERNOS.map((e) => e.nombre), filas)).toEqual([]);
  });

  it("el rojo: un proveedor sin fila se nombra", () => {
    expect(sinFila(["Mistral"], filas)).toEqual(["Mistral"]);
    expect(sinFila(["Claude"], ["| **Claude** | 30 días | no | acuerdo | no | sin fuente | sin fecha |"])).toEqual(["Claude"]);
  });
});
