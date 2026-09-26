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

  /**
   * La fase 4 cambió de dónde sale este texto: el titular ya no es una cadena fija, se **compone
   * con los términos que de verdad se buscaron**, y la maniobra llega del catálogo por su
   * identificador. Lo que la pantalla enseña es lo mismo; lo que cambió es que ahora puede ser
   * verdad.
   */
  it("«sin resultado» no deja solo al consultor: dice qué buscó, cómo abordarlo y qué tiene cerca", () => {
    pinta({ estado: "sin-resultado" });
    expect(document.querySelector(".titular-b")?.textContent).toBe(
      `${es.banda.nadaSobre} ${es.banda.comillaAbre}${es.banda.muestra.buscado}${es.banda.comillaCierra}`,
    );
    expect(document.querySelector(".maniobra-b .t")?.textContent).toBe(es.banda.maniobras.credencial);
    expect(document.querySelector(".cercano-b .d")?.textContent).toBe(
      `${es.banda.unidades.marco} · ${es.banda.muestra.cercana1}`,
    );
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

  /**
   * **EL MODO SOLO AUDIO (C15) — los tres estados de 44 px.**
   *
   * Referencia: `docs/diseno/banda.html`, estados «voz» y «voz-sin» (mirada 16, aprobada el
   * 2026-09-26) y «voz-espera» (mirada 16-bis, pendiente).
   *
   * Lo que estos tests afirman y una captura no: que a 44 px **desaparece la cabecera y NO
   * desaparece el contador de red**. Es la primera de las tres decisiones que el usuario aprobó, y
   * es la que se rompería sin darse cuenta: basta con que alguien mueva el «0 B» al `cab-b`, donde
   * están los otros dos chips, para que una promesa dura de la app deje de verse en el modo en que
   * el usuario **no está mirando la pantalla**.
   */
  it("a 44 px desaparece la cabecera y NO el contador de red", () => {
    pinta({ estado: "voz" });
    expect(banda().className).toBe("banda voz");
    expect(banda().querySelector(".cab-b")).toBeNull();
    // El «0 B» sobrevive a la cabecera, en la línea.
    expect(banda().querySelector(".lado-b .red.cero")?.textContent).toContain("0 B");
  });

  it("«hablando» dice qué está diciendo y de dónde sale, con las dos teclas", () => {
    pinta({ estado: "voz" });
    const linea = banda().querySelector(".linea-b") as HTMLElement;
    expect(linea.textContent).toContain(es.banda.diciendoLaFicha);
    expect(linea.querySelector("svg use")?.getAttribute("href")).toBe("#i-voz");
    expect(linea.querySelector(".fuente-b")).not.toBeNull();
    const teclas = [...banda().querySelectorAll(".tecla")].map((k) => k.textContent?.trim());
    expect(teclas).toEqual([`⎋ ${es.banda.callar}`, `⌘⇧V ${es.banda.volver}`]);
  });

  /**
   * **El candado, dibujado.** Es el estado que existe porque el sonido de esta app saldría por
   * donde el cliente oye. Los tres portadores de la regla 8 se comprueban de uno en uno: el glifo
   * de los auriculares TACHADOS, la frase, y el ámbar. Que el icono diga lo contrario del estado
   * ya pasó una vez en este mismo estado —`relleno` le comía la barra y dibujaba unos auriculares
   * conectados— y no lo vio ningún número: lo vio mirar la imagen.
   */
  it("«sin auriculares» avisa con símbolo, texto y color, y NO ofrece callar", () => {
    pinta({ estado: "voz-sin" });
    const linea = banda().querySelector(".linea-b") as HTMLElement;
    expect(linea.className).toContain("warn");
    expect(linea.querySelector("svg use")?.getAttribute("href")).toBe("#i-auriculares-off");
    expect(linea.textContent).toContain(es.banda.conectaAuriculares);
    expect(linea.textContent).toContain(es.banda.elClienteTeOiria);
    // No hay nada que callar: la tecla `⎋` ni siquiera está registrada en este estado.
    const teclas = [...banda().querySelectorAll(".tecla")].map((k) => k.textContent?.trim());
    expect(teclas).toEqual([`⌘⇧V ${es.banda.volver}`]);
  });

  /**
   * **El estado callado SE DICE.** Es el cambio que pidió la mirada 16-bis: la primera propuesta
   * enseñaba la línea de la ficha recién leída y se callaba el estado, y el veredicto fue «creo que
   * sí debería hacer evidente el estado». Una banda que no dice en qué está obliga a mirarla para
   * averiguarlo — lo contrario de un modo que existe para no mirar.
   *
   * Lo que se comprueba aquí y una captura no: que el verbo del estado «hablando» **no** se quedó
   * puesto, que el del callado sí está, y que detrás sigue la fuente de lo último que se leyó.
   */
  it("«callado» dice el estado y de dónde salió lo último que leyó", () => {
    pinta({ estado: "voz-espera" });
    const linea = banda().querySelector(".linea-b") as HTMLElement;
    expect(linea.textContent).not.toContain(es.banda.diciendoLaFicha);
    expect(linea.textContent).toContain(es.banda.callado);
    expect(linea.textContent).toContain(es.banda.esperandoElSiguienteTurno);
    expect(linea.querySelector(".fuente-b")).not.toBeNull();
    const teclas = [...banda().querySelectorAll(".tecla")].map((k) => k.textContent?.trim());
    expect(teclas).toEqual([`⌘⇧V ${es.banda.volver}`]);
  });

  it("el modo solo audio habla inglés, los tres estados", () => {
    pinta({ estado: "voz" }, "en");
    expect(banda().querySelector(".linea-b")?.textContent).toContain(en.banda.diciendoLaFicha);
    pinta({ estado: "voz-sin" }, "en");
    expect(banda().querySelector(".linea-b")?.textContent).toContain(en.banda.conectaAuriculares);
    expect(banda().querySelector(".linea-b")?.textContent).toContain(en.banda.elClienteTeOiria);
    pinta({ estado: "voz-espera" }, "en");
    expect(banda().querySelector(".linea-b")?.textContent).toContain(en.banda.callado);
    expect(banda().querySelector(".linea-b")?.textContent).toContain(
      en.banda.esperandoElSiguienteTurno,
    );
  });

  it("habla inglés con las mismas clases", () => {
    pinta({ estado: "sin-resultado" }, "en");
    expect(document.querySelector(".titular-b")?.textContent).toBe(
      `${en.banda.nadaSobre} ${en.banda.comillaAbre}${en.banda.muestra.buscado}${en.banda.comillaCierra}`,
    );
    expect(document.querySelector(".maniobra-b .t")?.textContent).toBe(en.banda.maniobras.credencial);
    expect(screen.getByText(en.banda.escuchando)).toBeInTheDocument();
  });
});

/**
 * **La composición tiene que dar EXACTAMENTE la línea de la maqueta.**
 *
 * Desde el arreglo del hallazgo A1, la banda no pinta las cadenas de la maqueta: las compone con
 * los números que de verdad hay («143 documentos · 5 unidades» = documentos del índice + unidades
 * con documentos). Fuera de Tauri los números son los de muestra, así que la línea tiene que salir
 * carácter a carácter igual — si no, el gate de FIDELIDAD compara contra un documento que ya no
 * describe el producto, y esa diferencia es de 0,0x % de píxeles: pasa por debajo del umbral sin
 * que nadie la vea.
 */
describe("la banda compone los contadores sin apartarse de la maqueta", () => {
  it.each([
    ["es", es],
    ["en", en],
  ])("el contador del corpus da la línea de la maqueta en %s", (idioma, dic) => {
    pinta({ estado: "esperando" }, idioma as Idioma);
    const meta = [...banda().querySelectorAll(".meta-b")].map((n) => n.textContent?.trim());
    expect(meta).toContain(dic.banda.corpus);
  });

  it.each([
    ["es", es],
    ["en", en],
  ])("la cabecera dice lo que la maqueta dice en %s", (idioma, dic) => {
    pinta({ estado: "esperando" }, idioma as Idioma);
    expect(banda().querySelector(".marca-min")?.textContent?.trim()).toBe(dic.banda.escuchando);
    expect(banda().querySelector(".cliente-b")?.textContent?.trim()).toBe(dic.banda.protegido);
  });
});
