import { act, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Cascara } from "@/App";
import { REUNION_DETECTADA, REUNION_NINGUNA } from "@/contrato.generado";

/**
 * **LA BANDA SE ENTERA DE LA REUNIÓN AL EMPEZAR LA SESIÓN** (casilla 6 del S3). La banda no recibe
 * el foco, así que con la app abierta antes de la llamada se quedaba en «sin reunión» toda la sesión.
 * Ahora `useReunion` vuelve a preguntar cuando abre una pista (`escucha` · `empieza`) —y no antes: no
 * hay nada vigilando las ventanas en segundo plano (ADR 005)—.
 *
 * ¿Puede fallar? Sí: sin `alEmpezarLaSesion` en `useReunion`, es rojo (bitácora del sprint 003).
 */

const invoke = vi.fn();
const oyentes = new Map<string, (e: { payload: unknown }) => void>();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/api/event", () => ({
  listen: (evento: string, f: (e: { payload: unknown }) => void) => {
    oyentes.set(evento, f);
    return Promise.resolve(() => {});
  },
}));

beforeEach(() => {
  vi.resetModules();
  invoke.mockReset();
  oyentes.clear();
  (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
});

afterEach(() => {
  delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
});

describe("la reunión se vuelve a preguntar al empezar la sesión", () => {
  it("abierta antes de la llamada, la banda la ve en cuanto abre una pista", async () => {
    let hay = false;
    invoke.mockImplementation((c: string) =>
      Promise.resolve(c === "reunion_abierta" ? (hay ? REUNION_DETECTADA : REUNION_NINGUNA) : null),
    );
    const { useReunion } = await import("@/cuaderno");
    function Sonda() {
      const r = useReunion();
      return <span data-testid="r">{r.que}</span>;
    }
    render(
      <Cascara>
        <Sonda />
      </Cascara>,
    );
    await waitFor(() => expect(screen.getByTestId("r")).toHaveTextContent("ninguna"));
    hay = true;
    await waitFor(() => expect(oyentes.has("escucha")).toBe(true));
    await act(async () => oyentes.get("escucha")!({ payload: { que: "turno" } }));
    expect(screen.getByTestId("r")).toHaveTextContent("ninguna");
    await act(async () => oyentes.get("escucha")!({ payload: { que: "empieza" } }));
    await waitFor(() => expect(screen.getByTestId("r")).toHaveTextContent("detectada"));
  });
});
