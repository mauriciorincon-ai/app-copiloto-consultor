import { describe, expect, it } from "vitest";
import { ventanaActual, VENTANAS } from "@/ventanas";

/**
 * Las tres ventanas cargan el mismo webview y se distinguen por su etiqueta. Fuera de Tauri no
 * hay etiqueta: si la resolución no tuviera un defecto sensato, abrir el proyecto en el navegador
 * —o un e2e— mostraría una banda suelta sin contexto.
 */
describe("qué ventana soy", () => {
  it("las tres del plan, ni una más", () => {
    expect([...VENTANAS]).toEqual(["principal", "banda", "relleno"]);
  });

  it("fuera de Tauri manda el parámetro `ventana`", () => {
    expect(ventanaActual("?ventana=banda")).toBe("banda");
    expect(ventanaActual("?ventana=relleno")).toBe("relleno");
  });

  it("sin parámetro, o con uno inventado, la principal", () => {
    expect(ventanaActual("")).toBe("principal");
    expect(ventanaActual("?ventana=lo-que-sea")).toBe("principal");
  });
});
