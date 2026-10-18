import { describe, expect, it } from "vitest";
import {
  PLATAFORMA_MACOS,
  PLATAFORMA_WINDOWS,
  PLATAFORMA_WINDOWS_SIN_MODELO,
} from "../../src/contrato.generado";
import {
  CLAVES_DE_WINDOWS,
  TECLAS_DE_MACOS,
  conTeclasDeEsteSistema,
  esWindows,
  estadoDeWhisper,
  porCuando,
  whisperEnEsteEquipo,
} from "../../src/plataforma";

/**
 * EL CONTRATO DE LA PLATAFORMA, de punta a punta (sprint 006, ADR 022; regla 19). Las muestras salen del MISMO serde
 * de Rust (`contrato.rs` → `contrato.generado.ts`) y aquí las leen los lectores que Honestidad usa.
 */
describe("lo que la app hace en este equipo", () => {
  it("en Windows cruzan las 15 funciones del ADR 022, en su orden", () => {
    expect(esWindows(PLATAFORMA_WINDOWS)).toBe(true);
    expect(PLATAFORMA_WINDOWS.funciones.map((f) => f.clave)).toEqual([
      ...CLAVES_DE_WINDOWS,
    ]);
    const g = porCuando(PLATAFORMA_WINDOWS);
    // Sprint 008, «Windows completo»: el acople, la sugerencia, el radar, el borrado con la app cerrada y el desbloqueo
    // pasan a «sí». Sprint 012: también la puerta local; están las 15.
    expect(g.si).toEqual([...CLAVES_DE_WINDOWS]);
    expect(g.si).toHaveLength(15);
    expect(g["no-por-ahora"]).toEqual([]);
    expect(Object.keys(g)).toEqual(["si", "no-por-ahora"]);
  });

  it("whisper se dice como está: medido, bajando o sin modelo", () => {
    expect(whisperEnEsteEquipo(PLATAFORMA_WINDOWS)).toEqual({
      que: "a-tiempo",
      modelo: "large-v3-turbo",
      ms: 2350,
    });
    expect(whisperEnEsteEquipo(PLATAFORMA_WINDOWS_SIN_MODELO)).toEqual({
      que: "bajando",
    });
    const sinBajar = {
      modelo: null,
      latenciaMs: null,
      techoMs: 4000,
      bajando: false,
    };
    expect(estadoDeWhisper(sinBajar)).toEqual({ que: "sin-modelo" });
    expect(estadoDeWhisper({ ...sinBajar, modelo: "small" })).toEqual({
      que: "sin-medir",
      modelo: "small",
    });
  });

  it("pasar de 4 s es «lento», y 4 s justos todavía no", () => {
    const base = { modelo: "large-v3-turbo", techoMs: 4000, bajando: false };
    expect(estadoDeWhisper({ ...base, latenciaMs: 4001 })).toEqual({
      que: "lento",
      modelo: "large-v3-turbo",
      ms: 4001,
      techoMs: 4000,
    });
    expect(estadoDeWhisper({ ...base, latenciaMs: 4000 })?.que).toBe(
      "a-tiempo",
    );
  });

  it("en macOS no cruza ninguna lista de Windows ni nada de whisper", () => {
    expect(esWindows(PLATAFORMA_MACOS)).toBe(false);
    expect(PLATAFORMA_MACOS.funciones).toEqual([]);
    expect(whisperEnEsteEquipo(PLATAFORMA_MACOS)).toBeNull();
  });
});

describe("las teclas de macOS salen del contrato, no de una copia (auditoría del S7, M7)", () => {
  it("son las once de Rust, cada acción una vez, y la sustitución va de la más larga a la más corta", () => {
    expect(TECLAS_DE_MACOS.map(([, accion]) => accion).sort()).toEqual(
      PLATAFORMA_MACOS.atajos.map((a) => a.accion).sort(),
    );
    expect(TECLAS_DE_MACOS).toHaveLength(PLATAFORMA_MACOS.atajos.length);
    // «⎋» está dentro de «⌥⎋»: si la corta fuera antes, el corte quedaría a medias.
    TECLAS_DE_MACOS.forEach(([teclas], i) => {
      for (const [despues] of TECLAS_DE_MACOS.slice(i + 1)) {
        expect(teclas).not.toBe(despues);
        expect(despues.includes(teclas)).toBe(false);
      }
    });
  });

  it("un texto con teclas de macOS se escribe con las de Windows, y el corte no se parte", () => {
    const texto = "Corta con ⌥⎋, calla la voz con ⎋ y oculta la banda con ⌃⌥B.";
    expect(conTeclasDeEsteSistema(texto, PLATAFORMA_MACOS, "es")).toBe(texto);
    const enWindows = conTeclasDeEsteSistema(texto, PLATAFORMA_WINDOWS, "es");
    expect(enWindows).toContain("Alt+Mayús+Q");
    expect(enWindows).not.toMatch(/[⌃⌥⎋]/);
  });
});
