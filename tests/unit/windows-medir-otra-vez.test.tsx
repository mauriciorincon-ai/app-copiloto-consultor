import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { HoyEnWindows } from "@/componentes/HoyEnWindows";
import { SpriteIconos } from "@/componentes/Iconos";
import { IdiomaContext } from "@/i18n";
import { preguntar } from "@/puente";
import { PLATAFORMA_WINDOWS } from "@/contrato.generado";
import type { Plataforma } from "@/plataforma";

/**
 * **«MEDIR OTRA VEZ» DICE QUE ESTÁ MIDIENDO** (sprint 010, visto en vivo en la Surface).
 *
 * Con el modelo grande, medir las dos pistas tarda cerca de medio minuto. Lo que había: el botón solo se apagaba, y
 * el usuario creyó que no hacía nada. Ahora, mientras dura, una línea con símbolo, texto y color (regla dura 8) lo
 * dice, el botón cambia a «Midiendo…» con `aria-busy`, y la región `status` lo anuncia a los lectores de pantalla.
 * Al terminar, todo vuelve. ¿Puede fallar? Sí: quitar la línea o el cambio del botón es rojo.
 */
vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: () => () => {},
  preguntar: vi.fn(() => Promise.resolve(null)),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

function pinta(lengua: "es" | "en") {
  cleanup();
  return render(
    <IdiomaContext.Provider value={lengua}>
      <SpriteIconos />
      <HoyEnWindows plataforma={PLATAFORMA_WINDOWS} />
    </IdiomaContext.Provider>,
  );
}

/** La medida que Rust todavía no ha devuelto: el test decide cuándo termina. */
let terminar: (p: Plataforma | null) => void = () => {};

beforeEach(() => {
  vi.mocked(preguntar).mockReset();
  vi.mocked(preguntar).mockImplementation((orden: string) =>
    orden === "calibrar_transcripcion"
      ? new Promise((r) => {
          terminar = r as (p: Plataforma | null) => void;
        })
      : Promise.resolve(null),
  );
});

afterEach(() => cleanup());

describe("mientras mide, Honestidad lo dice", () => {
  it.each([
    [
      "es",
      "Medir otra vez",
      "Midiendo…",
      /Midiendo en este PC, modelo por modelo y pista por pista/,
    ],
    [
      "en",
      "Measure again",
      "Measuring…",
      /Measuring on this PC, model by model and track by track/,
    ],
  ] as const)(
    "%s: la línea, el botón y el aviso a los lectores de pantalla",
    async (lengua, medir, midiendo, linea) => {
      pinta(lengua);
      expect(screen.queryByText(linea)).toBeNull();
      fireEvent.click(screen.getByRole("button", { name: medir }));

      const enMarcha = await screen.findByText(linea);
      expect(enMarcha.closest("p")?.className).toContain("halo");
      expect(enMarcha.closest("p")?.querySelector("svg")).not.toBeNull();
      expect(screen.getByRole("status").textContent).toMatch(linea);
      const boton = screen.getByRole("button", { name: midiendo });
      expect(boton).toBeDisabled();
      expect(boton.getAttribute("aria-busy")).toBe("true");

      await act(async () => terminar(null));
      expect(screen.queryByText(linea)).toBeNull();
      expect(screen.getByRole("button", { name: medir })).not.toBeDisabled();
    },
  );
});
