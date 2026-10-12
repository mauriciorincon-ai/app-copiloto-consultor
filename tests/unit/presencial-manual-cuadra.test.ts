import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * **LAS CIFRAS DEL MANUAL DE PRESENCIAL SALEN DE SUS ARCHIVOS** (sprint 008, auditoría del S5, B12 y M8).
 *
 * El manual repetía a mano los valores de `data/presencial/reglas.json` (15 s y 15 s), la espera de la sala (4 s, que vive
 * en `habla/mod.rs`) y las cifras de la prueba del kit. El ADR 020 dice que cambiar una regla es una línea de
 * `reglas.json`, y ningún gate ataba el manual a ese archivo: la cifra del manual envejecía sola.
 *
 * ¿Puede fallar? Sí: con `esperaTrasFichaMs` en 20 000 sin tocar el manual, o con el 4 s de Rust en 5, es rojo con el
 * valor y el sitio.
 */
const MANUAL = readFileSync("docs/MANUAL-DE-USO.md", "utf8");
const SECCION = (() => {
  const i = MANUAL.indexOf("### Presencial: la app escucha la sala");
  expect(i, "no encontré la sección Presencial del manual").toBeGreaterThan(-1);
  return MANUAL.slice(i, MANUAL.indexOf("\n### ", i + 10));
})();
const REGLAS = JSON.parse(readFileSync("data/presencial/reglas.json", "utf8")) as {
  tolerancia: { esperaTrasFichaMs: number; topeDeTurnoMs: number };
};
const KIT = JSON.parse(readFileSync("docs/kit-de-prueba/presencial.json", "utf8")) as {
  salas: { turnos: { quien: string; espera: string }[] }[];
};
const segundos = (ms: number) => `${ms / 1000} s`;

describe("el manual de Presencial dice lo que dicen sus archivos", () => {
  it("la espera tras una ficha y el tope de un turno son los de reglas.json", () => {
    const { esperaTrasFichaMs, topeDeTurnoMs } = REGLAS.tolerancia;
    expect(SECCION, `la espera tras una ficha (${segundos(esperaTrasFichaMs)})`).toContain(
      `espera **${segundos(esperaTrasFichaMs)}** antes de dar otra`,
    );
    expect(SECCION, `el tope de un turno (${segundos(topeDeTurnoMs)})`).toContain(
      `ningún turno de la sala dura más de ${segundos(topeDeTurnoMs)}`,
    );
  });

  it("la espera a que la sala calle es la de habla/mod.rs", () => {
    const rust = readFileSync("src-tauri/src/habla/mod.rs", "utf8");
    const segs = /pub const ESPERA_DE_LA_SALA: std::time::Duration = std::time::Duration::from_secs\((\d+)\)/.exec(rust)?.[1];
    expect(segs, "no leí ESPERA_DE_LA_SALA").toBeTruthy();
    expect(SECCION).toContain(`Si en ${segs} s nadie calla`);
  });

  it("la prueba del kit no promete lo que no midió: sus turnos del cliente que piden ficha, 5 fichas, 16 que sobraban a 5", () => {
    // Los 14 salen del kit: los turnos del cliente que debían traer ficha.
    const piden = KIT.salas.flatMap((s) => s.turnos).filter((x) => x.quien === "cliente" && x.espera === "ficha").length;
    expect(SECCION).toContain(`de ${piden} turnos del cliente que pedían ficha salieron 5`);
    expect(SECCION).toContain("bajaron de 16 a 5");
    expect(SECCION).not.toMatch(/las mismas que sin\s+reglas/);
    expect(SECCION).not.toMatch(/casi todas por tu voz/);
  });

  it("la sección nombra dónde cambiar el micrófono con el nombre del Mac en inglés, y que ⎋ es de la app", () => {
    expect(SECCION).toContain("System Settings → Sound → Input");
    expect(SECCION).toContain("`⎋` es de la app");
  });
});
