import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { IDIOMAS_DE_PISTA } from "@/contrato.generado";

/**
 * **LO QUE ELIGES SE RECUERDA** (sprint 003, ADR 002 enmienda 2; deuda del S2), desde el lado del
 * cuaderno. Rust guarda y lee `preferencias.json` (sus tests en `prefs.rs`); lo que solo puede fallar
 * aquí es el cable:
 *
 * 1. al abrirse, el cuaderno PIDE los idiomas guardados y los enseña — si no los pide, cada arranque
 *    vuelve a español y el usuario lo descubre cuando no llega un turno;
 * 2. elegir un idioma se lo DICE a Rust — si no, se ve elegido y no se guarda.
 *
 * ¿Puede fallar? Sí: sin `pedirLosGuardados()` en el suscriptor, o sin el `llamar` de
 * `fijarIdiomaDePista`, es rojo (bitácora del sprint 003). La respuesta de Rust es la muestra del
 * contrato, la que el serializador real escribe.
 */

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));

let quitarTauri = () => {};

beforeEach(() => {
  vi.resetModules();
  invoke.mockReset();
  invoke.mockImplementation((comando: string) =>
    Promise.resolve(comando === "idiomas_de_pista" ? IDIOMAS_DE_PISTA : null),
  );
  (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  quitarTauri = () => {
    delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
  };
});

afterEach(() => quitarTauri());

describe("el idioma de cada pista se recuerda", () => {
  it("al abrir, el cuaderno enseña el que quedó guardado, no el de fábrica", async () => {
    const { useIdiomasDePista } = await import("@/cuaderno");
    function Sonda() {
      const i = useIdiomasDePista();
      return <span data-testid="cliente">{i.cliente}</span>;
    }
    render(<Sonda />);
    await waitFor(() => expect(screen.getByTestId("cliente")).toHaveTextContent("en-US"));
    expect(invoke).toHaveBeenCalledWith("idiomas_de_pista", undefined);
  });

  it("elegir uno se lo dice a Rust para la próxima vez", async () => {
    const { fijarIdiomaDePista, idiomasDeLasPistas } = await import("@/cuaderno");
    fijarIdiomaDePista("cliente", "en-US");
    expect(idiomasDeLasPistas().cliente).toBe("en-US");
    await waitFor(() =>
      expect(invoke).toHaveBeenCalledWith("fijar_idioma_de_pista", { pista: "cliente", idioma: "en-US" }),
    );
  });
});
