import { cleanup, render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Banda, type PropsBanda } from "@/componentes/Banda";
import { IdiomaContext, es, en, type Idioma } from "@/i18n";

/**
 * La banda contra su referencia (`docs/diseno/banda.html`, mirada 11).
 *
 * Lo que un test puede afirmar y una captura no: que cada estado dibuja **su** anatomía y **su**
 * copy, en los dos idiomas. Lo que una captura afirma y un test no —que se parece— es el gate de
 * FIDELIDAD, que se corre con la ventana real y lo aprueba el usuario.
 */
function pinta(props: PropsBanda, idioma: Idioma = "es") {
  // Varios `pinta` en un mismo test: sin esto los renders se ACUMULAN y `querySelector` devuelve
  // el primero, así que comparar «compacta» contra «ampliada» seguía leyendo la compacta y el
  // test fallaba culpando al componente. Es el mismo defecto que K3 (la limpieza vive en
  // `tests/setup.ts` y corre ENTRE tests, no dentro de uno).
  cleanup();
  return render(
    <IdiomaContext.Provider value={idioma}>
      <Banda {...props} />
    </IdiomaContext.Provider>,
  );
}

const banda = () => document.querySelector("section.banda") as HTMLElement;

describe("la banda", () => {
  it("es 88 px salvo que el asa la amplíe; el transcript la amplía porque vive a la derecha", () => {
    pinta({ estado: "ficha" });
    expect(banda().className).toBe("banda");

    pinta({ estado: "ficha", ampliada: true });
    expect(banda().className).toBe("banda ampliada");

    pinta({ estado: "ficha", transcript: true });
    expect(banda().className).toBe("banda ampliada");
  });

  it("la cabecera dice siempre el estado, el cliente y la red", () => {
    pinta({ estado: "esperando" });
    expect(screen.getByText(es.banda.escuchando)).toBeInTheDocument();
    expect(screen.getByText(es.banda.protegido)).toBeInTheDocument();
    expect(screen.getByText("0 B")).toBeInTheDocument();
  });

  it("sin permiso de acople lo dice con símbolo, texto y color", () => {
    pinta({ estado: "ficha", acoplada: false });
    const aviso = screen.getByText(es.banda.sinAcople).closest("span");
    expect(aviso?.className).toContain("warn");
    expect(aviso?.querySelector("svg use")?.getAttribute("href")).toBe("#i-alert");
  });

  it("«esperando» no dibuja un vacío gris: dice una frase y de cuánto corpus dispone", () => {
    pinta({ estado: "esperando" });
    expect(document.querySelector(".voz-b")?.textContent).toBe(es.banda.esperando);
    expect(screen.getByText(es.banda.corpus)).toBeInTheDocument();
    expect(screen.getByText(es.banda.reunion)).toBeInTheDocument();
  });

  it("«buscando» repite lo que oyó, con su pista, y no pone nada que se mueva", () => {
    pinta({ estado: "buscando" });
    expect(screen.getByText(es.banda.muestra.oido)).toBeInTheDocument();
    expect(
      document.querySelector(".oido .quien svg use")?.getAttribute("href"),
      "la pista del sistema es la que atribuye al cliente: jamás la voz",
    ).toBe("#i-sistema");
    expect(screen.getByText(es.banda.buscando)).toBeInTheDocument();
  });

  it("«ficha» pone la evidencia a la izquierda y su fuente a la derecha", () => {
    pinta({ estado: "ficha" });
    expect(document.querySelector(".ficha-b .titular-b")?.textContent).toBe(es.banda.muestra.titular);
    expect(document.querySelector(".ficha-b .linea-b")?.textContent).toBe(es.banda.muestra.linea);
    const fuente = document.querySelector(".lado-b .fuente-b");
    expect(fuente?.textContent).toContain(es.banda.muestra.unidad);
    expect(fuente?.textContent).toContain(es.banda.muestra.fuente);
  });

  it("al ampliarla, el «+2» se abre: las acumuladas se leen", () => {
    pinta({ estado: "ficha" });
    expect(document.querySelector(".mas-b")).toBeNull();

    pinta({ estado: "ficha", ampliada: true });
    expect(document.querySelectorAll(".mas-b .item")).toHaveLength(2);
    expect(screen.getByText(es.banda.muestra.acumulada1)).toBeInTheDocument();
    expect(document.querySelector(".linea-b")?.textContent).toBe(es.banda.muestra.lineaLarga);
  });

  it("en 88 px las acciones son teclas; los botones vuelven al ampliar", () => {
    pinta({ estado: "sin-resultado" });
    expect(document.querySelectorAll("button")).toHaveLength(0);
    expect(document.querySelectorAll(".lado-b kbd").length).toBeGreaterThan(0);

    pinta({ estado: "sin-resultado", ampliada: true });
    expect(document.querySelectorAll("button")).toHaveLength(2);
  });

  it("«sin resultado» no deja solo al consultor: dice qué buscó, cómo abordarlo y qué tiene cerca", () => {
    pinta({ estado: "sin-resultado" });
    expect(document.querySelector(".titular-b")?.textContent).toBe(es.banda.nada);
    expect(document.querySelector(".maniobra-b .t")?.textContent).toBe(es.banda.maniobra);
    expect(document.querySelector(".cercano-b .d")?.textContent).toBe(es.banda.muestra.cercana);
  });

  it("la maniobra NO puede parecer salida del modelo", () => {
    pinta({ estado: "sin-resultado" });
    const maniobra = document.querySelector(".maniobra-b") as HTMLElement;
    // `halo` y la chispa son, en este design system, las marcas de la síntesis de la IA. La
    // maniobra sale de un catálogo versionado y determinista: llevarlas sería mentir con estilo.
    expect(maniobra.className).not.toContain("halo");
    expect(maniobra.querySelector("svg use")?.getAttribute("href")).not.toBe("#i-chispa");
    expect(maniobra.closest(".sugerencia")).toBeNull();
  });

  it("al ampliar «sin resultado» entran la pregunta entera y las tres más cercanas", () => {
    pinta({ estado: "sin-resultado" });
    expect(document.querySelector(".oido")).toBeNull();

    pinta({ estado: "sin-resultado", ampliada: true });
    expect(screen.getByText(es.banda.muestra.oidoIso)).toBeInTheDocument();
    expect(document.querySelectorAll(".mas-b .item")).toHaveLength(3);
    expect(screen.getByText(es.banda.cercanoLargo)).toBeInTheDocument();
  });

  it("«sin verificar» avisa en ámbar y ofrece UNA salida en 88 px, TRES al ampliar", () => {
    pinta({ estado: "sin-verificar", verificado: false });
    expect(screen.getByText(es.banda.sinVerificar)).toBeInTheDocument();
    expect(document.querySelector(".aviso-b strong")?.textContent).toBe(es.banda.sinVerificarTitulo);
    expect(document.querySelectorAll(".aviso-b .salida")).toHaveLength(1);
    expect(document.querySelector("ul.vertical")).toBeNull();

    pinta({ estado: "sin-verificar", verificado: false, ampliada: true });
    expect(document.querySelectorAll("ul.vertical li")).toHaveLength(3);
  });

  it("el transcript solo aparece si se enciende, y cada turno lleva su PISTA, no un nombre", () => {
    pinta({ estado: "ficha" });
    expect(document.querySelector(".transcript-b")).toBeNull();

    pinta({ estado: "ficha", transcript: true });
    const turnos = [...document.querySelectorAll(".transcript-b .turno")];
    expect(turnos).toHaveLength(3);
    const pistas = turnos.map((t) => t.querySelector("svg use")?.getAttribute("href"));
    expect(pistas).toEqual(["#i-sistema", "#i-mic", "#i-sistema"]);
    expect(document.querySelector(".transcript-b .cab")?.textContent).toContain(
      es.banda.transcriptCab,
    );
  });

  it("habla inglés con las mismas clases", () => {
    pinta({ estado: "sin-resultado" }, "en");
    expect(document.querySelector(".titular-b")?.textContent).toBe(en.banda.nada);
    expect(document.querySelector(".maniobra-b .t")?.textContent).toBe(en.banda.maniobra);
    expect(screen.getByText(en.banda.escuchando)).toBeInTheDocument();
  });
});
