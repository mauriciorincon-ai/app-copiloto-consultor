import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * Gate de la VERSIÓN MÍNIMA — **lo que el binario enlaza manda sobre lo que el paquete promete.**
 *
 * `build.rs` enlaza los frameworks de Apple de forma FUERTE: si uno no existe en el macOS del
 * usuario, dyld aborta al arrancar, antes de que la app pueda decir «sin modelo». Desde el sprint
 * 002 enlaza `FoundationModels`, que solo existe en macOS 26, y `tauri.conf.json` seguía declarando
 * 14.2: en un Mac con 14 o 15 la app se instalaba y no abría (auditoría del S2, M1).
 *
 * Cada framework de la lista pide su versión; la declarada tiene que ser al menos la mayor.
 * ¿Puede fallar? Sí: con `14.2` en la configuración era rojo (bitácora del sprint 002).
 */
const EXIGE: Record<string, number> = {
  FoundationModels: 26,
  ScreenCaptureKit: 12.3,
  Speech: 10.15,
};

describe("la versión mínima de macOS", () => {
  it("es al menos la que exigen los frameworks que enlaza build.rs", () => {
    const build = readFileSync("src-tauri/build.rs", "utf8");
    const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8"));
    const declarada = Number.parseFloat(conf.bundle.macOS.minimumSystemVersion);
    const enlazados = [...build.matchAll(/framework=(\w+)/g)].map((m) => m[1]);
    const necesaria = Math.max(...enlazados.map((f) => EXIGE[f] ?? 0));
    expect(declarada, `enlaza ${enlazados.join(", ")}`).toBeGreaterThanOrEqual(necesaria);
  });
});
