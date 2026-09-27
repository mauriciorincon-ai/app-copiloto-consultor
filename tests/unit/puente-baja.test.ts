import { afterEach, describe, expect, it, vi } from "vitest";

/**
 * La baja de un oyente de Tauri puede fallar —el oyente ya no está en su tabla— y el puente no
 * puede dejar esa promesa rechazada suelta: salía como «Unhandled rejection» en cada arranque de la
 * app en desarrollo (hallazgo de la corrida en vivo del radar, sprint 002, fase 4).
 */
let llamadas = 0;
let contestar: (() => void) | null = null;

/**
 * La baja es una función CORRIENTE, no un `vi.fn`: el espía de Vitest se engancha a la promesa que
 * devuelve para anotar cómo acabó, y con eso la da por atendida. La primera versión de este test
 * lo usaba y pasaba en verde con el defecto puesto —se vio al exigirle el rojo—.
 */
vi.mock("@tauri-apps/api/event", () => ({
  listen: () =>
    new Promise((resolver) => {
      const baja = () => {
        llamadas++;
        return Promise.reject(new TypeError("listeners[eventId].handlerId"));
      };
      contestar = () => resolver(baja);
    }),
}));

afterEach(() => {
  delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
  llamadas = 0;
});

describe("el puente se da de baja sin romper nada", () => {
  it("una baja que falla —el oyente se fue antes de que listen contestara— no queda suelta", async () => {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    const sueltas: unknown[] = [];
    const alRechazo = (e: unknown) => sueltas.push(e);
    process.on("unhandledRejection", alRechazo);
    try {
      const { escuchar } = await import("@/puente");
      const apagar = escuchar("radar", () => {});
      // Se desmonta ANTES de que `listen` conteste, como hace React al montar dos veces.
      apagar();
      await vi.waitFor(() => expect(contestar).not.toBeNull());
      contestar!();
      await vi.waitFor(() => expect(llamadas).toBe(1));
      await new Promise((r) => setTimeout(r, 20));
      expect(sueltas, "la baja fallida salió como promesa rechazada sin atrapar").toEqual([]);
    } finally {
      process.off("unhandledRejection", alRechazo);
    }
  });
});
