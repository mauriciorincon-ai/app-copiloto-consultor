import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Cascara } from "@/App";
import { Banda } from "@/componentes/Banda";
import { HoyEnWindows } from "@/componentes/HoyEnWindows";
import { PLATAFORMA_MACOS, PLATAFORMA_WINDOWS } from "@/contrato.generado";
import { plataformaInicial, type Plataforma } from "@/plataforma";

/**
 * **LAS RAMAS DE WINDOWS QUE NINGUNA MUESTRA ALCANZABA** (sprint 008, auditoría del S7, M10, M11 y M12): una tecla que
 * el sistema no dejó registrar, los turnos que no dio tiempo a transcribir, la descarga en curso y el primer cuadro
 * dentro de Tauri. Las muestras del contrato son las del caso feliz; aquí se parten a propósito.
 *
 * ¿Puede fallar? Sí: quitar `atajosSinRegistrar` de la banda o de Honestidad, o el `> 0` de los descartados, es rojo.
 */
const sinRegistrar = (
  accion: Plataforma["atajos"][number]["accion"],
): Plataforma => ({
  ...PLATAFORMA_WINDOWS,
  atajos: PLATAFORMA_WINDOWS.atajos.map((a) =>
    a.accion === accion ? { ...a, registrado: false } : a,
  ),
});

function pinta(nodo: React.ReactNode, idioma: "es" | "en") {
  cleanup();
  document.documentElement.lang = idioma;
  return render(<Cascara>{nodo}</Cascara>);
}

afterEach(() => {
  cleanup();
  document.documentElement.lang = "";
});

describe("una tecla que el sistema no dejó registrar se dice", () => {
  it("Honestidad nombra la tecla, en los dos idiomas, con su símbolo y su texto", () => {
    pinta(<HoyEnWindows plataforma={sinRegistrar("corte")} />, "es");
    expect(screen.getByText(/Alt\+Mayús\+Q no responde/)).toBeTruthy();
    pinta(<HoyEnWindows plataforma={sinRegistrar("corte")} />, "en");
    expect(screen.getByText(/Alt\+Shift\+Q doesn't respond/)).toBeTruthy();
  });

  it("con todas registradas, Honestidad no dice nada de teclas", () => {
    pinta(<HoyEnWindows plataforma={PLATAFORMA_WINDOWS} />, "es");
    expect(screen.queryByText(/no responde:/)).toBeNull();
  });

  it("la banda lo dice también, contado, y en macOS o con todas registradas no aparece", () => {
    pinta(
      <Banda estado="esperando" plataforma={sinRegistrar("corte")} />,
      "es",
    );
    expect(screen.getByText("1 tecla no responde")).toBeTruthy();
    pinta(<Banda estado="esperando" plataforma={PLATAFORMA_WINDOWS} />, "es");
    expect(screen.queryByText(/no responde/)).toBeNull();
    pinta(<Banda estado="esperando" plataforma={PLATAFORMA_MACOS} />, "es");
    expect(screen.queryByText(/no responde/)).toBeNull();
    const dos: Plataforma = {
      ...sinRegistrar("corte"),
      atajos: sinRegistrar("corte").atajos.map((a) =>
        a.accion === "callar" ? { ...a, registrado: false } : a,
      ),
    };
    pinta(<Banda estado="esperando" plataforma={dos} />, "en");
    expect(screen.getByText("2 shortcuts aren't responding")).toBeTruthy();
  });
});

describe("los turnos sin transcribir y la descarga en curso se dicen", () => {
  const con = (
    t: Partial<NonNullable<Plataforma["transcripcion"]>>,
  ): Plataforma => ({
    ...PLATAFORMA_WINDOWS,
    transcripcion: { ...PLATAFORMA_WINDOWS.transcripcion!, ...t },
  });

  it("descartados > 0 aparece, sin atribuirlos, y en 0 no", () => {
    pinta(<HoyEnWindows plataforma={con({ descartados: 3 })} />, "es");
    const frase = screen.getByText(/3 turnos se quedaron sin transcribir/);
    expect(frase.textContent).not.toMatch(/tuy/i);
    pinta(<HoyEnWindows plataforma={con({ descartados: 0 })} />, "es");
    expect(screen.queryByText(/sin transcribir/)).toBeNull();
  });

  it("la descarga en curso se enseña y esconde los botones de medir", () => {
    pinta(<HoyEnWindows plataforma={con({ bajando: true })} />, "es");
    expect(screen.getByText(/Bajando el modelo de voz/)).toBeTruthy();
    expect(screen.queryByText("Medir otra vez")).toBeNull();
  });
});

/**
 * **UNA LÍNEA POR MODELO** (sprint 009, ADR 022 enmienda 3): Honestidad enseña lo que tardó CADA modelo de voz en este
 * PC, no solo el que se usa. Cuatro estados —sin bajar, sin medir, cabe, pasa del techo— y cada uno con símbolo, texto y
 * color (regla dura 8). ¿Puede fallar? Sí: pintar solo el modelo en uso, o dejar fuera «pasa del techo», es rojo.
 */
describe("Honestidad enseña una línea por modelo de voz, medida en este PC", () => {
  const modelo = (
    m: Partial<NonNullable<Plataforma["transcripcion"]>["modelos"][number]>,
  ) => ({
    nombre: "large-v3-turbo",
    bajado: true,
    latenciaMs: null as number | null,
    latenciaClienteMs: null as number | null,
    enUso: false,
    porDefecto: false,
    ...m,
  });
  const con = (modelos: ReturnType<typeof modelo>[]): Plataforma => ({
    ...PLATAFORMA_WINDOWS,
    transcripcion: { ...PLATAFORMA_WINDOWS.transcripcion!, modelos },
  });
  const lineas = () =>
    screen.getByTestId("modelos-de-voz").querySelectorAll("p.estado");

  it("pinta los dos modelos, cada uno con su cifra, en los dos idiomas", () => {
    const p = con([
      modelo({ nombre: "large-v3-turbo", latenciaMs: 5100, enUso: false }),
      modelo({
        nombre: "small",
        latenciaMs: 1800,
        enUso: true,
        porDefecto: true,
      }),
    ]);
    pinta(<HoyEnWindows plataforma={p} />, "es");
    expect(lineas()).toHaveLength(2);
    expect(
      screen.getByText(
        /large-v3-turbo: tu pista, 10 s en 5.?100 ms\. Pasa del techo de 4 s\./,
      ),
    ).toBeTruthy();
    expect(
      screen.getByText(
        /small: tu pista, 10 s en 1.?800 ms\. Cabe en el techo de 4 s\. \(el que transcribe ahora · el que baja «Instalar»\)/,
      ),
    ).toBeTruthy();
    pinta(<HoyEnWindows plataforma={p} />, "en");
    expect(
      screen.getByText(
        /large-v3-turbo: your track, 10 s in 5,100 ms\. Over the 4 s ceiling\./,
      ),
    ).toBeTruthy();
    expect(
      screen.getByText(
        /small: .*Fits the 4 s ceiling\. \(the one transcribing now · the one «Install» downloads\)/,
      ),
    ).toBeTruthy();
  });

  it("cada estado lleva su símbolo y su color, no solo uno de los dos", () => {
    pinta(
      <HoyEnWindows
        plataforma={con([
          modelo({ nombre: "large-v3-turbo", latenciaMs: 9000 }),
          modelo({ nombre: "small", bajado: false }),
        ])}
      />,
      "es",
    );
    const [lento, sinBajar] = Array.from(lineas());
    expect(lento.className).toContain("warn");
    expect(sinBajar.className).toContain("mute");
    // El símbolo: cada línea trae su icono además del color.
    expect(lento.querySelector("svg")).not.toBeNull();
    expect(sinBajar.querySelector("svg")).not.toBeNull();
    expect(screen.getByText("small: sin bajar.")).toBeTruthy();
  });

  // Sprint 010: Honestidad, pista por pista. La tuya decide «cabe» o «pasa», como la regla; la del cliente se enseña
  // al lado y, si pasa del techo aunque la tuya quepa, se dice aparte y en ámbar. ¿Puede fallar? Sí: callar la pista del
  // cliente, o pintar «cabe» en verde cuando la del cliente pasa de 4 s, es rojo.
  it("enseña las dos pistas de cada modelo medido, en los dos idiomas", () => {
    const p = con([
      modelo({
        nombre: "large-v3-turbo",
        latenciaMs: 9144,
        latenciaClienteMs: 16858,
      }),
      modelo({
        nombre: "small",
        latenciaMs: 1632,
        latenciaClienteMs: 1552,
        enUso: true,
      }),
    ]);
    pinta(<HoyEnWindows plataforma={p} />, "es");
    expect(
      screen.getByText(
        /large-v3-turbo: tu pista, 10 s en 9.?144 ms; la del cliente, 6 s en 16.?858 ms\. Pasa del techo de 4 s\./,
      ),
    ).toBeTruthy();
    expect(
      screen.getByText(
        /small: tu pista, 10 s en 1.?632 ms; la del cliente, 6 s en 1.?552 ms\. Cabe en el techo de 4 s\. \(el que transcribe ahora\)/,
      ),
    ).toBeTruthy();
    pinta(<HoyEnWindows plataforma={p} />, "en");
    expect(
      screen.getByText(
        /small: your track, 10 s in 1,632 ms; the client's, 6 s in 1,552 ms\. Fits the 4 s ceiling\./,
      ),
    ).toBeTruthy();
  });

  it("si tu pista cabe y la del cliente no, lo dice aparte y en ámbar", () => {
    pinta(
      <HoyEnWindows
        plataforma={con([
          modelo({
            nombre: "large-v3-turbo",
            latenciaMs: 3900,
            latenciaClienteMs: 4200,
          }),
          modelo({ nombre: "small", bajado: false }),
        ])}
      />,
      "es",
    );
    const [linea] = Array.from(lineas());
    expect(linea.className).toContain("warn");
    expect(linea.querySelector("svg")).not.toBeNull();
    expect(
      screen.getByText(
        /large-v3-turbo: tu pista, 10 s en 3.?900 ms; la del cliente, 6 s en 4.?200 ms\. Tu pista cabe en el techo de 4 s; la del cliente, no\./,
      ),
    ).toBeTruthy();
  });

  it("bajado y sin cifra dice «sin medir»; una cifra justo en el techo cabe", () => {
    pinta(
      <HoyEnWindows
        plataforma={con([
          modelo({ latenciaMs: null }),
          modelo({ nombre: "small", latenciaMs: 4000 }),
        ])}
      />,
      "es",
    );
    expect(
      screen.getByText("large-v3-turbo: bajado, sin medir en esta sesión."),
    ).toBeTruthy();
    expect(
      screen.getByText(/small: tu pista, 10 s en 4.?000 ms\. Cabe en el techo/),
    ).toBeTruthy();
  });

  // Auditoría del S9 (A1/B5): el resumen de arriba y la línea del modelo decían lo mismo con las mismas palabras, y un
  // e2e que buscaba esa frase encontró dos elementos. Con las líneas por modelo, el resumen de «a tiempo» y «sin medir»
  // no se repite; sin ellas (muestra vacía) sí se pinta, para que el estado no se pierda.
  it("la cifra del modelo en uso sale UNA vez, no en el resumen y otra vez en su línea", () => {
    pinta(<HoyEnWindows plataforma={PLATAFORMA_WINDOWS} />, "es");
    expect(
      screen.getAllByText(/large-v3-turbo: tu pista, 10 s en 2\.?350 ms/),
    ).toHaveLength(1);
    expect(screen.queryByText(/El techo son 4 s/)).toBeNull();
    pinta(<HoyEnWindows plataforma={con([])} />, "es");
    expect(
      screen.getByText(
        /Modelo large-v3-turbo: 10 s de audio en 2\.?350 ms\. El techo son 4 s/,
      ),
    ).toBeTruthy();
  });

  it("sin modelos en la muestra no pinta el bloque (macOS y el primer cuadro no cambian)", () => {
    pinta(<HoyEnWindows plataforma={con([])} />, "es");
    expect(screen.queryByTestId("modelos-de-voz")).toBeNull();
    pinta(<HoyEnWindows plataforma={PLATAFORMA_MACOS} />, "es");
    expect(screen.queryByTestId("modelos-de-voz")).toBeNull();
  });
});

describe("el primer cuadro dentro de Tauri ya sabe en qué sistema está", () => {
  const WEBVIEW2 =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36 Edg/130.0";
  const WKWEBVIEW =
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)";

  it("en Windows parte de Windows, con sus teclas y sin cifras de muestra", () => {
    const p = plataformaInicial(PLATAFORMA_MACOS, WEBVIEW2, true);
    expect(p.sistema).toBe("windows");
    expect(p.atajos.find((a) => a.accion === "corte")?.teclas).toBe(
      "Alt+Shift+Q",
    );
    expect(p.transcripcion).toBeNull();
    expect(p.funciones).toEqual([]);
  });

  it("en macOS, o fuera de Tauri, no cambia nada", () => {
    expect(plataformaInicial(PLATAFORMA_MACOS, WKWEBVIEW, true)).toBe(
      PLATAFORMA_MACOS,
    );
    expect(plataformaInicial(PLATAFORMA_MACOS, WEBVIEW2, false)).toBe(
      PLATAFORMA_MACOS,
    );
  });
});

describe("en Windows la protección se dice pedida, no verificada (auditoría del S7, B9; regla dura 6)", () => {
  it("las tres frases que decían «protegido» dicen lo que se pidió, en los dos idiomas", async () => {
    const { diccionarioDeWindows } = await import("@/i18n");
    for (const idioma of ["es", "en"] as const) {
      const d = diccionarioDeWindows(idioma, (t) => t);
      for (const frase of [
        d.cuaderno.panelProtegido,
        d.notas.clienteNoLaVe,
        d.cliente.panelProtegido,
      ]) {
        expect(frase, frase).toMatch(/pedida|requested/);
      }
    }
  });
});

describe("«Bajando…» se refresca solo, y el modelo pequeño no se ofrece si ya es el que suena (auditoría del S6, B6 y B7)", () => {
  const con = (
    t: Partial<NonNullable<Plataforma["transcripcion"]>>,
  ): Plataforma => ({
    ...PLATAFORMA_WINDOWS,
    transcripcion: { ...PLATAFORMA_WINDOWS.transcripcion!, ...t },
  });

  it("el refresco corre mientras dura la descarga y se para al acabar", async () => {
    const { renderHook } = await import("@testing-library/react");
    const { useRefrescoMientras } = await import("@/plataforma");
    const { vi } = await import("vitest");
    vi.useFakeTimers();
    try {
      const refrescar = vi.fn();
      const { rerender, unmount } = renderHook(
        ({ activo }) => useRefrescoMientras(activo, refrescar, 1000),
        {
          initialProps: { activo: true },
        },
      );
      vi.advanceTimersByTime(3500);
      expect(refrescar).toHaveBeenCalledTimes(3);
      rerender({ activo: false });
      vi.advanceTimersByTime(5000);
      expect(
        refrescar,
        "al acabar la descarga ya no pregunta",
      ).toHaveBeenCalledTimes(3);
      unmount();
    } finally {
      vi.useRealTimers();
    }
  });

  it("lento con el grande: ofrece el pequeño · lento con el pequeño puesto: no", () => {
    pinta(
      <HoyEnWindows
        plataforma={con({ modelo: "large-v3-turbo", latenciaMs: 6200 })}
      />,
      "es",
    );
    expect(screen.getByText("Bajar el modelo pequeño (0,5 GB)")).toBeTruthy();
    pinta(
      <HoyEnWindows plataforma={con({ modelo: "small", latenciaMs: 5100 })} />,
      "es",
    );
    expect(screen.getByText(/pasa del techo de 4 s/)).toBeTruthy();
    expect(screen.queryByText("Bajar el modelo pequeño (0,5 GB)")).toBeNull();
  });
});
