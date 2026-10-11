import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { parse } from "yaml";

/**
 * Regla 28 (kit v1.40.0) — PAREJAS DE VERSIONES ENTRE DOS ECOSISTEMAS se vigilan con un test entre lockfiles.
 * Perfil escritorio: cada paquete npm de Tauri y su crate de Rust van en la misma versión MENOR, o
 * `pnpm tauri build` se niega («Found version mismatched Tauri packages»). Dependabot solo mueve npm y
 * `cargo check` no lo ve: todo verde y el binario no se construye (Angel Ghost S4, /release-check).
 * Sin `src-tauri/Cargo.lock` (perfil web) el test pasa y lo dice: aquí no hay parejas que vigilar.
 *
 * Plantilla del kit v1.40.0, que nació de este test (sprint 005, fase 0). El origen, en esta casa (auditoría del
 * S4, `/release-check`, 2026-10-04): dependabot subió `@tauri-apps/api` a 2.12 y `@tauri-apps/plugin-opener` a 2.7
 * en el lote de npm (#11), y los crates se quedaron en 2.11 y 2.5: dependabot no vigila cargo (el techo de dos PRs,
 * `dependabot-config.test.ts`). La CI no construye el binario —solo `cargo check`—, así que todo salió verde y el
 * binario no se podía hacer. Este test lee los dos lockfiles, que es lo que se instala de verdad, y nació en rojo con
 * ese estado (regla 15; su rojo, en la bitácora del sprint 004).
 */
const CARGO_LOCK = "src-tauri/Cargo.lock";
const hayEscritorio = existsSync(CARGO_LOCK);

type Importador = Record<string, Record<string, { version: string }> | undefined>;
function npmVersiones(): Record<string, string> {
  const lock = parse(readFileSync("pnpm-lock.yaml", "utf8")) as { importers: Record<string, Importador> };
  const raiz = lock.importers["."] ?? {};
  const npm: Record<string, string> = {};
  for (const grupo of ["dependencies", "devDependencies"]) {
    for (const [nombre, d] of Object.entries(raiz[grupo] ?? {})) npm[nombre] = d.version.split("(")[0] ?? d.version;
  }
  return npm;
}
function cargoVersiones(): Record<string, string> {
  const cargo: Record<string, string> = {};
  for (const bloque of readFileSync(CARGO_LOCK, "utf8").split("[[package]]")) {
    const nombre = /^name = "([^"]+)"$/m.exec(bloque)?.[1];
    const version = /^version = "([^"]+)"$/m.exec(bloque)?.[1];
    if (nombre && version) cargo[nombre] = version;
  }
  return cargo;
}
/** El crate que corresponde a cada paquete npm de Tauri; la CLI no tiene crate. */
function crateDe(paquete: string): string | null {
  const resto = paquete.replace("@tauri-apps/", "");
  if (resto === "cli") return null;
  if (resto === "api") return "tauri";
  return resto.startsWith("plugin-") ? `tauri-${resto}` : null;
}
const menor = (v: string) => v.split(".").slice(0, 2).join(".");

describe("Tauri a la par: cada paquete npm y su crate en la misma versión menor (regla 28)", () => {
  if (!hayEscritorio) {
    it("sin src-tauri/Cargo.lock no hay parejas que vigilar (perfil web): pasa y lo dice", () => {
      expect(hayEscritorio).toBe(false);
    });
    return;
  }
  const npm = npmVersiones();
  const cargo = cargoVersiones();
  const parejas = Object.keys(npm)
    .filter((p) => p.startsWith("@tauri-apps/"))
    .map((p) => ({ paquete: p, crate: crateDe(p) }))
    .filter((x): x is { paquete: string; crate: string } => x.crate !== null);

  it("hay parejas que comparar (si no, este test no mide nada)", () => {
    expect(parejas.map((p) => p.crate)).toContain("tauri");
  });

  it.each(parejas)("$paquete ↔ $crate", ({ paquete, crate }) => {
    const enRust = cargo[crate];
    expect(enRust, `${crate} no está en ${CARGO_LOCK}`).toBeDefined();
    expect(
      menor(enRust ?? ""),
      `${paquete} está en ${npm[paquete]} y ${crate} en ${enRust}: «pnpm tauri build» no construye así. Sube el crate con «cargo update -p ${crate}».`,
    ).toBe(menor(npm[paquete] ?? ""));
  });
});
