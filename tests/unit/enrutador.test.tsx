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

/** El alto de la ventana, que es lo que decide si la banda va compacta o ampliada. */
function alto(px: number) {
  Object.defineProperty(globalThis, "innerHeight", { value: px, configurable: true, writable: true });
}

describe("el enrutador de ventanas", () => {
  it("la banda dibuja la banda, con su sprite de iconos", () => {
    const { container } = pinta("?ventana=banda&estado=ficha");
    expect(container.querySelector("section.banda")).not.toBeNull();
    expect(container.querySelector("svg symbol")).not.toBeNull();
    expect(document.documentElement.dataset.ventana).toBe("banda");
  });

  it("la banda se dibuja ampliada o no según el ALTO DE LA VENTANA, no según un parámetro", () => {
    // Quien manda es la ventana: el asa la cambia de tamaño y la banda se entera midiendo. Si
    // esto dependiera de un parámetro, una banda ampliada podría acabar dibujada dentro de un
    // marco de 88 px —recortada por `overflow: hidden`— sin que nada se quejara.
    alto(88);
    expect(pinta("?ventana=banda&estado=ficha").container.querySelector("section.banda")?.className).toBe("banda");

    alto(200);
    expect(pinta("?ventana=banda&estado=ficha").container.querySelector("section.banda")?.className).toBe(
      "banda ampliada",
    );
  });

  it("el transcript se pide por parámetro (así se recorre el gate de fidelidad)", () => {
    alto(200);
    const { container } = pinta("?ventana=banda&estado=ficha&transcript=1");
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
