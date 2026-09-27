import { act, cleanup, render } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Enrutador } from "@/App";
import { llamar } from "@/puente";

/**
 * **`⌃⌥T` AGRANDA LA BANDA** (casilla 6 del S3). El transcript vive en la columna derecha de la
 * banda ampliada; abrirlo cambiaba el dibujo a «ampliada» (200 px) dentro de una ventana que seguía
 * midiendo 88, y la banda salía recortada por abajo: justo lo que `enrutador.test.tsx` dice que
 * no puede pasar «sin que nada se quejara». Se pasa por el ENRUTADOR, no por el hook suelto, para
 * que quitar la llamada de `App.tsx` también sea rojo. ¿Puede fallar? Sí: sin
 * `useAltoDelTranscript` en el enrutador, cae el primero (bitácora del S3).
 */

const oyentes = new Map<string, ((dato: unknown) => void)[]>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: (evento: string, alOir: (dato: unknown) => void) => {
    oyentes.set(evento, [...(oyentes.get(evento) ?? []), alOir]);
    return () => oyentes.delete(evento);
  },
  preguntar: vi.fn(() => Promise.resolve(null)),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

/** Lo que hace Rust al oír `⌃⌥T`: emitir `transcript` a la banda. */
async function pulsarCtrlOptT() {
  const oidos = oyentes.get("transcript") ?? [];
  expect(oidos.length, "nadie escucha «transcript»").toBeGreaterThan(0);
  await act(async () => {
    for (const o of oidos) o(undefined);
  });
  await act(async () => {});
}

/** Lo que hace macOS cuando Rust cambia el tamaño de la ventana. */
async function laVentanaMide(px: number) {
  Object.defineProperty(globalThis, "innerHeight", { value: px, configurable: true, writable: true });
  await act(async () => {
    globalThis.dispatchEvent(new Event("resize"));
  });
}

const altosPedidos = () =>
  vi
    .mocked(llamar)
    .mock.calls.filter(([c]) => c === "ajustar_banda" || c === "asentar_banda")
    .map(([c, a]) => `${c}:${(a as { alto: number }).alto}`);

beforeEach(async () => {
  cleanup();
  oyentes.clear();
  vi.mocked(llamar).mockClear();
  Object.defineProperty(globalThis, "innerHeight", { value: 88, configurable: true, writable: true });
});

describe("⌃⌥T y el alto de la banda", () => {
  it("con la banda compacta, abrir el transcript la agranda como el asa y cerrarlo la devuelve", async () => {
    render(<Enrutador busqueda="?ventana=banda&estado=ficha" />);
    await act(async () => {});

    await pulsarCtrlOptT();
    expect(altosPedidos()).toEqual(["ajustar_banda:200", "asentar_banda:200"]);

    await laVentanaMide(200);
    await pulsarCtrlOptT();
    expect(altosPedidos()).toEqual([
      "ajustar_banda:200",
      "asentar_banda:200",
      "ajustar_banda:88",
      "asentar_banda:88",
    ]);
  });

  it("si la ampliaste tú con el asa, el transcript no la toca al abrir ni al cerrar", async () => {
    await laVentanaMide(200);
    render(<Enrutador busqueda="?ventana=banda&estado=ficha" />);
    await act(async () => {});

    await pulsarCtrlOptT();
    await pulsarCtrlOptT();
    expect(altosPedidos()).toEqual([]);
  });
});
