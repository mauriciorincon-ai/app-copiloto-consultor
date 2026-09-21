import { render, screen } from "@testing-library/react";
import { describe, expect, it, beforeEach } from "vitest";
import { Cascara } from "@/App";
import { useT, useIdioma } from "@/i18n";

/**
 * La cáscara fija tema e idioma en `<html>` — el mismo contrato que la maqueta (design-system
 * §9: «`html[data-theme]` sigue siendo el conmutador de tema»). Si esto se rompe, los tokens
 * dejan de resolver y el tema claro deja de existir sin que nada más se queje.
 */
function Sonda() {
  const t = useT();
  return (
    <>
      <span data-testid="idioma">{useIdioma()}</span>
      <span data-testid="cadena">{t.banda.escuchando}</span>
    </>
  );
}

describe("cáscara: tema e idioma viven en <html>", () => {
  beforeEach(() => {
    document.documentElement.removeAttribute("data-theme");
    document.documentElement.lang = "";
  });

  it("respeta el tema declarado en el documento", () => {
    document.documentElement.dataset.theme = "light";
    render(<Cascara />);
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("sin tema declarado cae en oscuro, que es el primario del design system", () => {
    render(<Cascara />);
    expect(document.documentElement.dataset.theme).toBe("dark");
  });

  it("sirve el diccionario del idioma del documento", () => {
    document.documentElement.lang = "en";
    render(
      <Cascara>
        <Sonda />
      </Cascara>,
    );
    expect(screen.getByTestId("idioma")).toHaveTextContent("en");
    expect(screen.getByTestId("cadena")).toHaveTextContent("Listening");
  });

  it("en español sirve la cadena de la maqueta en español", () => {
    document.documentElement.lang = "es";
    render(
      <Cascara>
        <Sonda />
      </Cascara>,
    );
    expect(screen.getByTestId("cadena")).toHaveTextContent("Escuchando");
  });
});
