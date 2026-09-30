import { act, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { ESTADO_DE_LA_IA_CON_API, ESTADO_DE_LA_IA_NADIE } from "@/contrato.generado";

/**
 * **IA SE PONE AL DÍA AL VOLVER A LA VENTANA** (casilla 6 del S3): apagar Apple Intelligence en
 * Ajustes no emite nada que la app oiga; la pantalla tiene que volver a preguntar al recuperar el
 * foco, como Permisos. ¿Puede fallar? Sí: sin el `focus` en `useIa`, es rojo (bitácora del S3).
 */

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn().mockResolvedValue(() => {}) }));

beforeEach(() => {
  vi.resetModules();
  invoke.mockReset();
  (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
});

afterEach(() => {
  delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
});

describe("IA vuelve a preguntar al recuperar el foco", () => {
  it("cambia fuera de la app, y al volver dice lo que hay ahora", async () => {
    let apagado = false;
    invoke.mockImplementation(() => Promise.resolve(apagado ? ESTADO_DE_LA_IA_NADIE : ESTADO_DE_LA_IA_CON_API));
    const { useIa } = await import("@/ia");
    function Sonda() {
      const [e] = useIa();
      return <span data-testid="q">{e.quien ?? "nadie"}</span>;
    }
    render(<Sonda />);
    await waitFor(() => expect(screen.getByTestId("q")).toHaveTextContent("api"));
    apagado = true;
    await act(async () => {
      globalThis.dispatchEvent(new Event("focus"));
    });
    await waitFor(() => expect(screen.getByTestId("q")).toHaveTextContent("nadie"));
  });
});
