import { describe, expect, it } from "vitest";
import {
  PLATAFORMA_MACOS,
  PLATAFORMA_WINDOWS,
  PLATAFORMA_WINDOWS_SIN_MODELO,
} from "../../src/contrato.generado";
import {
  CLAVES_DE_WINDOWS,
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
    // Sprint 007: la videollamada, la pantalla, la voz al oído, el ensayo con voz y los atajos pasan a «sí».
    expect(g.si).toEqual([
      "banda-protegida",
      "presencial",
      "corpus-y-ficha",
      "notas-cifradas",
      "videollamada",
      "pantalla",
      "voz-al-oido",
      "ensayo",
      "atajos",
    ]);
    expect(g["todavia-no"]).toEqual([
      "acople",
      "sugerencia",
      "radar",
      "borrado-con-la-app-cerrada",
      "desbloqueo",
    ]);
    expect(g["no-por-ahora"]).toEqual(["puerta-local"]);
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
