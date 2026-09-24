import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useAcoplada, EVENTO_ACOPLE, type EstadoDelAcople } from "@/acople";
import { Relleno } from "@/componentes/Relleno";
import { escuchar, hayTauri, preguntar } from "@/puente";

/**
 * EL ACOPLE Y EL RELLENO, desde el lado del webview.
 *
 * Lo que se prueba aquí no es la Accessibility API —eso vive en Rust y tiene sus tests— sino las
 * dos cosas que solo pueden fallar en este lado:
 *
 * 1. Que la banda diga «acoplada» **por haberlo comprobado**, no por defecto. Es la clase de
 *    etiqueta que, dejada fija, se lee como una promesa cumplida cuando no lo está.
 * 2. Que el relleno se quede NEGRO ante cualquier fallo del fondo de escritorio. Un relleno que
 *    no se puede pintar no puede quedarse transparente: eso es la franja descubierta.
 */

const invoke = vi.fn();
const listen = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: (...a: unknown[]) => listen(...a) }));

/** Finge que estamos dentro de Tauri y devuelve cómo deshacerlo. */
function conTauri() {
  (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
  return () => {
    delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
  };
}

function Sonda({ forzado }: { forzado?: boolean }) {
  const acoplada = useAcoplada(forzado);
  return <span data-testid="estado">{acoplada ? "acoplada" : "sin acople"}</span>;
}

beforeEach(() => {
  invoke.mockReset();
  listen.mockReset();
  listen.mockResolvedValue(() => {});
});

describe("«acoplada» es un hecho comprobado, no una etiqueta", () => {
  it("fuera de Tauri no pregunta a nadie y se queda en el estado normal del producto", async () => {
    expect(hayTauri()).toBe(false);
    render(<Sonda />);
    expect(screen.getByTestId("estado")).toHaveTextContent("acoplada");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("dentro de Tauri pregunta, y si la respuesta es «no», lo dice", async () => {
    const fuera = conTauri();
    invoke.mockResolvedValue({ permiso: false, acoplada: false } satisfies EstadoDelAcople);

    render(<Sonda />);
    await waitFor(() => expect(screen.getByTestId("estado")).toHaveTextContent("sin acople"));
    expect(invoke).toHaveBeenCalledWith("estado_del_acople", undefined);
    fuera();
  });

  it("se entera de los cambios sin sondear: escucha el evento de lo nativo", async () => {
    const fuera = conTauri();
    invoke.mockResolvedValue({ permiso: true, acoplada: true } satisfies EstadoDelAcople);
    let avisar: ((e: { payload: EstadoDelAcople }) => void) | null = null;
    listen.mockImplementation(async (_n: string, cb: (e: { payload: EstadoDelAcople }) => void) => {
      avisar = cb;
      return () => {};
    });

    render(<Sonda />);
    await waitFor(() => expect(avisar).not.toBeNull());
    expect(listen).toHaveBeenCalledWith(EVENTO_ACOPLE, expect.any(Function));

    avisar!({ payload: { permiso: true, acoplada: false } });
    await waitFor(() => expect(screen.getByTestId("estado")).toHaveTextContent("sin acople"));
    fuera();
  });

  /**
   * El parámetro de URL existe para que el gate de fidelidad pueda recorrer el encuadre «sin
   * acople» en un navegador. Si además disparara la pregunta a lo nativo, el arnés de capturas
   * estaría fotografiando un estado que no eligió.
   */
  it("el valor forzado manda y calla la pregunta", () => {
    const fuera = conTauri();
    render(<Sonda forzado={false} />);
    expect(screen.getByTestId("estado")).toHaveTextContent("sin acople");
    expect(invoke).not.toHaveBeenCalled();
    fuera();
  });
});

describe("el relleno: negro ante cualquier fallo, jamás transparente", () => {
  it("sin fondo de escritorio se queda negro y sin nada dentro", () => {
    const { container } = render(<Relleno />);
    const franja = container.querySelector(".relleno-franja");
    expect(franja).toBeInTheDocument();
    expect(franja).toHaveAttribute("aria-hidden", "true");
    expect(container.querySelector(".relleno-fondo")).toBeNull();
  });

  it("con fondo, lo encuadra al tamaño de la PANTALLA y lo sube por el alto de la franja", async () => {
    const fuera = conTauri();
    invoke.mockResolvedValue("data:image/jpeg;base64,AAA");
    // Una pantalla de 1440×900 con la franja de 88 px abajo: la imagen se pinta entera y se
    // sube 812 px, que es justo lo que deja asomar el trozo que estaría debajo de la banda.
    Object.defineProperty(globalThis.screen, "width", { value: 1440, configurable: true });
    Object.defineProperty(globalThis.screen, "height", { value: 900, configurable: true });
    Object.defineProperty(globalThis, "innerHeight", { value: 88, configurable: true });

    const { container } = render(<Relleno />);
    await waitFor(() => expect(container.querySelector(".relleno-fondo")).not.toBeNull());
    const fondo = container.querySelector(".relleno-fondo") as HTMLElement;
    expect(fondo.style.width).toBe("1440px");
    expect(fondo.style.height).toBe("900px");
    expect(fondo.style.top).toBe("-812px");
    expect(fondo.style.backgroundImage).toContain("data:image/jpeg;base64,AAA");
    fuera();
  });

  it("si lo nativo contesta que no hay fondo, sigue negro", async () => {
    const fuera = conTauri();
    invoke.mockResolvedValue(null);
    const { container } = render(<Relleno />);
    await waitFor(() => expect(invoke).toHaveBeenCalledWith("fondo_del_relleno", undefined));
    expect(container.querySelector(".relleno-fondo")).toBeNull();
    fuera();
  });
});

describe("el puente: preguntar y escuchar fuera de Tauri", () => {
  it("preguntar devuelve null en vez de estallar", async () => {
    await expect(preguntar("estado_del_acople")).resolves.toBeNull();
  });

  it("escuchar devuelve SIEMPRE una baja, para que quien limpia no tenga que comprobar nada", () => {
    const baja = escuchar("lo-que-sea", () => {});
    expect(typeof baja).toBe("function");
    expect(() => baja()).not.toThrow();
  });
});

afterEach(() => {
  delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
});
