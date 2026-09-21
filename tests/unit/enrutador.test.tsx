import { cleanup, render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Enrutador } from "@/App";

/**
 * Las tres ventanas comparten webview y se reparten por etiqueta. Dos cosas que comprobar aquí,
 * y la segunda es de producto, no de código:
 *
 *  - cada etiqueta dibuja lo suyo;
 *  - **el relleno no dibuja NADA**. Es lo único que una captura de pantalla completa encuentra
 *    donde vive la banda: si algún día alguien le mete contenido «solo para depurar», eso es
 *    exactamente lo que vería el cliente. No hay aviso posible: desde el Mac del consultor el
 *    relleno está tapado por la banda y no se ve nunca.
 */
function pinta(busqueda: string) {
  cleanup();
  return render(<Enrutador busqueda={busqueda} />);
}

describe("el enrutador de ventanas", () => {
  it("la banda dibuja la banda, con su sprite de iconos", () => {
    const { container } = pinta("?ventana=banda&estado=ficha");
    expect(container.querySelector("section.banda")).not.toBeNull();
    expect(container.querySelector("svg symbol")).not.toBeNull();
    expect(document.documentElement.dataset.ventana).toBe("banda");
  });

  it("el asa y el transcript se piden por parámetro (así se recorre el gate de fidelidad)", () => {
    const { container } = pinta("?ventana=banda&estado=ficha&ampliada=1&transcript=1");
    expect(container.querySelector("section.banda")?.className).toBe("banda ampliada");
    expect(container.querySelector(".transcript-b")).not.toBeNull();
  });

  it("sin acople se pide igual, y la banda lo dice", () => {
    const { container } = pinta("?ventana=banda&estado=ficha&acoplada=0");
    expect(container.querySelector(".cliente-b.warn")).not.toBeNull();
  });

  it("EL RELLENO NO DIBUJA NADA: ni texto, ni banda, ni iconos", () => {
    const { container } = pinta("?ventana=relleno");
    expect(container.querySelector(".relleno-franja")).not.toBeNull();
    expect(container.textContent).toBe("");
    expect(container.querySelector("section.banda")).toBeNull();
    expect(container.querySelector("svg")).toBeNull();
    expect(container.querySelector("img")).toBeNull();
  });

  it("sin etiqueta, la principal", () => {
    const { container } = pinta("");
    expect(container.querySelector(".principal-vacia")).not.toBeNull();
    expect(container.querySelector("section.banda")).toBeNull();
  });
});
