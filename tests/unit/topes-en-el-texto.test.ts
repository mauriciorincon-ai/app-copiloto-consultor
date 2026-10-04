import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { es } from "@/i18n/es";
import { en } from "@/i18n/en";

/**
 * Gate de los TOPES QUE EL TEXTO REPITE (auditoría del S3, B11; casilla 6 a de `/audita-sprint`).
 *
 * «Llegaste a 30 propuestas» fija en la interfaz el tope que decide Rust (`TOPE_DE_PROPUESTAS`). Si uno
 * cambia y el otro no, la pantalla miente sobre cuándo deja de proponer. El texto es de la maqueta, así
 * que no se interpola: se comprueba.
 *
 * ¿Puede fallar? Sí: su demo cambia el tope y el texto deja de cuadrar.
 */
export function topeEnRust(fuente: string, nombre: string): number {
  const m = new RegExp(`pub const ${nombre}: usize = ([\\d_]+);`).exec(fuente);
  if (!m) throw new Error(`${nombre} no está en la fuente`);
  return Number(m[1].replace(/_/g, ""));
}

export function dicenElTope(tope: number, textos: string[]): boolean {
  return textos.every((t) => new RegExp(`(^|\\D)${tope}(\\D|$)`).test(t));
}

describe("topes-en-el-texto: el número que la pantalla dice es el que Rust aplica", () => {
  const tope = topeEnRust(readFileSync("src-tauri/src/notas/mod.rs", "utf8"), "TOPE_DE_PROPUESTAS");

  it("«lleno», en los dos idiomas, dice TOPE_DE_PROPUESTAS", () => {
    expect(dicenElTope(tope, [es.notas.lleno, en.notas.lleno])).toBe(true);
  });

  it("el rojo: con otro tope, el texto ya no cuadra", () => {
    expect(dicenElTope(tope + 5, [es.notas.lleno, en.notas.lleno])).toBe(false);
    expect(topeEnRust("pub const TOPE_DE_PROPUESTAS: usize = 1_000;", "TOPE_DE_PROPUESTAS")).toBe(1000);
  });
});
