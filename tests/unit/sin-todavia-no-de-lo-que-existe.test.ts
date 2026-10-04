import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * Gate de las PROMESAS APLAZADAS QUE CADUCARON (auditoría del S3, M7 y B9; casilla 4 de `/audita-sprint`).
 *
 * Una pantalla que dice «Todavía no» de algo que existe miente igual que una que promete lo que falta.
 * El S3 construyó «Conservar mis turnos» y «Solo notas», y tres pantallas seguían diciendo «Todavía no».
 * Y lo que ya tiene sitio en el H2 (MLX, arrastrar documentos, releer lo que cambió, leer lo escaneado)
 * dice «En el H2», no «Todavía no». Este gate lee las fuentes de las pantallas.
 *
 * ¿Puede fallar? Sí: nació en rojo con las pantallas de antes (bitácora).
 */
const leer = (ruta: string) => readFileSync(ruta, "utf8");

/** La `<Fila …>` (o el bloque) que contiene `clave`, hasta su cierre. */
function bloqueCon(fuente: string, clave: string, cierre: string): string {
  const i = fuente.indexOf(clave);
  const j = i < 0 ? -1 : fuente.indexOf(cierre, i);
  return i < 0 || j < 0 ? "" : fuente.slice(i, j + cierre.length);
}

export function caducadas(f: { idioma: string; permisos: string; banda: string; corpus: string; ia: string }): string[] {
  const fuera: string[] = [];
  if (f.idioma.includes("conservarTusTurnos")) fuera.push("Idioma: «Conservar lo que dijiste tú — Todavía no» (existe en Notas)");
  if (/variosIdiomasPorPista\}<\/span>\s*<TodaviaNo/.test(f.idioma)) fuera.push("Idioma: varios idiomas por pista es H2");
  if (bloqueCon(f.permisos, "texto={t.escribirNotas}", "</Fila>").includes("<TodaviaNo")) {
    fuera.push("Permisos: «Escribir notas y acuerdos — Todavía no» (Solo notas funciona sin permisos)");
  }
  if (bloqueCon(f.permisos, "texto={t.buscarAMano}", "</Fila>").includes("<TodaviaNo")) {
    fuera.push("Permisos: «Buscar tu evidencia a mano — Todavía no» (en Solo notas, ⌃⌥A busca con tu nota, sin permisos)");
  }
  if (/title=\{tc\.todaviaNo\}>\s*<Ic id="i-nota"/.test(f.banda)) fuera.push("Banda: «Solo notas» con title «Todavía no» (se elige en Sesión)");
  if (bloqueCon(f.corpus, "t.corpusPendiente", "))}").includes("<TodaviaNo")) fuera.push("Corpus: lo que es H2 dice «Todavía no»");
  if (bloqueCon(f.ia, "MLX · Qwen 3 4B", "</tr>").includes("<TodaviaNo")) fuera.push("IA: MLX es H2 y dice «Todavía no»");
  return fuera;
}

describe("sin «Todavía no» de lo que existe (M7) ni de lo que ya es H2 (B9)", () => {
  const fuentes = {
    idioma: leer("src/pantallas/Idioma.tsx"),
    permisos: leer("src/pantallas/Permisos.tsx"),
    banda: leer("src/componentes/Banda.tsx"),
    corpus: leer("src/pantallas/Corpus.tsx"),
    ia: leer("src/pantallas/Ia.tsx"),
  };

  it("lee las cinco pantallas y encuentra cada bloque (un gate que no lee nada no es un gate)", () => {
    expect(bloqueCon(fuentes.permisos, "texto={t.escribirNotas}", "</Fila>")).not.toBe("");
    expect(bloqueCon(fuentes.permisos, "texto={t.buscarAMano}", "</Fila>")).not.toBe("");
    expect(bloqueCon(fuentes.corpus, "t.corpusPendiente", "))}")).not.toBe("");
    expect(bloqueCon(fuentes.ia, "MLX · Qwen 3 4B", "</tr>")).not.toBe("");
  });

  it("ninguna pantalla aplaza lo que existe ni esconde el horizonte de lo que falta", () => {
    expect(caducadas(fuentes)).toEqual([]);
  });

  it("el rojo: la fila de antes en Permisos se nombra", () => {
    const antes = '<Fila icono="i-nota" texto={t.escribirNotas} pendiente>\n  <TodaviaNo />\n</Fila>';
    expect(caducadas({ ...fuentes, permisos: antes })).toHaveLength(1);
  });
});
