import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * **LAS LISTAS DE CHECKS REQUERIDOS SON LOS JOBS DE LA CI** (sprint 010; paga el M5 de la auditoría del sprint 009).
 *
 * El sprint 006 añadió `build-windows-arm` a la CI y a la ruleset `main-protegida`, pero la constitución, `/release-check` y
 * el anexo siguieron diciendo «cinco checks» sin él: `/release-check` no lo habría exigido. Este gate lee los jobs de
 * `ci.yml` y `un-pr-vivo.yml` y exige que cada uno aparezca en las tres listas. Un job nuevo sin su línea en ellas es rojo.
 *
 * ¿Puede fallar? Sí: quitar `build-windows-arm` de cualquiera de las tres, o añadir un job a la CI sin anotarlo, es rojo.
 */
const FLUJOS = [".github/workflows/ci.yml", ".github/workflows/un-pr-vivo.yml"];

/** Los jobs de los dos flujos: las claves de dos espacios bajo `jobs:`. */
const JOBS = FLUJOS.flatMap((f) => {
  const texto = readFileSync(f, "utf8");
  return [
    ...texto
      .slice(texto.indexOf("\njobs:\n"))
      .matchAll(/^ {2}([a-z][a-z0-9-]*):\s*$/gm),
  ].map((m) => m[1]!);
});

const LISTAS: [string, string, RegExp][] = [
  ["CLAUDE.md", "CLAUDE.md", /CI `quality[^`]*`/],
  [
    ".claude/commands/release-check.md (arriba)",
    ".claude/commands/release-check.md",
    /Todo lo demás lo corre la CI \([^)]*\)/,
  ],
  [
    ".claude/commands/release-check.md (casilla 9)",
    ".claude/commands/release-check.md",
    /Los \w+ checks \([^)]*\)/,
  ],
  [
    "docs/REGLAS-ANEXO.md",
    "docs/REGLAS-ANEXO.md",
    /\| `quality` · `e2e`[^|]*\|/,
  ],
];

describe("las listas de checks requeridos dicen los mismos jobs que la CI", () => {
  it("CONTROL: la CI tiene los seis jobs que se esperan (el lector no pasa de vacío)", () => {
    expect([...JOBS].sort()).toEqual(
      [
        "build-escritorio",
        "build-windows",
        "build-windows-arm",
        "e2e",
        "quality",
        "un-pr-vivo",
      ].sort(),
    );
  });

  it.each(LISTAS)("%s nombra todos los jobs", (_nombre, archivo, patron) => {
    const texto = readFileSync(archivo, "utf8").replace(/\s+/g, " ");
    const lista = texto.match(patron)?.[0];
    expect(lista, `${archivo}: no se halló la lista`).toBeTruthy();
    // Palabra completa: `build-windows` no se da por nombrado porque aparezca dentro de `build-windows-arm`.
    for (const job of JOBS)
      expect(
        new RegExp(`(?<![\\w-])${job}(?![\\w-])`).test(lista!),
        `${archivo}: falta \`${job}\``,
      ).toBe(true);
  });

  it("la casilla 9 dice «seis» porque son seis jobs", () => {
    const texto = readFileSync(".claude/commands/release-check.md", "utf8");
    expect(JOBS.length).toBe(6);
    expect(texto).toContain("Los seis checks");
  });
});
