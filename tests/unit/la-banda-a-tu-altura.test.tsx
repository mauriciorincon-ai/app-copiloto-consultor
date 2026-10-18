import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Cascara } from "@/App";
import { SpriteIconos } from "@/componentes/Iconos";
import { Banda } from "@/componentes/Banda";
import { Sesion } from "@/pantallas/Sesion";
import { desplazamientoAlArrastrar } from "@/asa";
import { TEXTOS_DE_WINDOWS } from "@/i18n/windows";
import { llamar, preguntar } from "@/puente";
import {
  ESTADO_DE_LA_ESCUCHA,
  LA_FRANJA_ABAJO,
  LA_FRANJA_ARRIBA,
  LA_FRANJA_BAJADA,
  LA_FRANJA_CERRADA,
  LA_VOZ_DICIENDO,
  PLATAFORMA_MACOS,
  PLATAFORMA_WINDOWS,
} from "@/contrato.generado";

/**
 * **LA BANDA A TU ALTURA Y SU «×»** (sprint 011, ADR 004 enmienda 3) — lo que la interfaz hace con las cuatro puertas de
 * Rust (`desplazar_banda`, `asentar_desplazamiento`, `cerrar_la_banda`, `mostrar_la_banda`).
 *
 * Se finge el PUENTE como en `la-banda-arriba.test.tsx`, y las respuestas salen de `src/contrato.generado.ts`, que escribe
 * Rust con su serde (regla 19). Lo que se vigila:
 *
 *  1. **En el Mac no cambia nada:** ni asa de altura, ni «×», ni la clase que les hace sitio, ni las filas de Sesión.
 *  2. **En Windows**, el «×» tiene nombre accesible y dice lo que hace (con la sesión en marcha, que la termina), y el asa
 *     de altura existe solo arriba.
 *  3. **El arrastre** sigue la pareja de siempre: mientras arrastras se mueve lo nuestro; al soltar, se guarda; y un clic
 *     sin mover no guarda nada. Nunca baja de 0.
 *  4. **Sesión** enseña la altura con «Volver arriba» y, con la banda cerrada, «Mostrar la banda».
 *
 * ¿Puede fallar? Sí: sin el `enWindows &&` de `Banda.tsx` el Mac vería el «×»; sin el `if (pedido !== inicial)` de
 * `useAsaDeAltura` un clic reacoplaría la reunión; y sin `franja.bandaAbierta` Sesión no ofrecería «Mostrar la banda».
 */
const respuestas = new Map<string, unknown>();
const oyentes = new Map<string, (dato: unknown) => void>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: vi.fn((evento: string, alOir: (dato: unknown) => void) => {
    oyentes.set(evento, alOir);
    return () => {};
  }),
  preguntar: vi.fn((comando: string) =>
    Promise.resolve(respuestas.get(comando) ?? null),
  ),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

const pedidos = (comando: string) =>
  vi.mocked(llamar).mock.calls.filter(([c]) => c === comando);

async function pinta(nodo: React.ReactNode, lang: "es" | "en" = "es") {
  document.documentElement.lang = lang;
  const r = render(
    <Cascara>
      <SpriteIconos />
      {nodo}
    </Cascara>,
  );
  await act(async () => {});
  return r;
}

const ventana = globalThis as unknown as Window;

/** La banda en Windows (o en el Mac) con la franja que diga Rust. */
function conRust(franja: unknown, plataforma: unknown) {
  respuestas.set("la_franja", franja);
  respuestas.set("plataforma_de_este_equipo", plataforma);
}

beforeEach(() => {
  cleanup();
  respuestas.clear();
  oyentes.clear();
  vi.mocked(preguntar).mockClear();
  vi.mocked(llamar).mockClear();
  Object.defineProperty(globalThis, "innerHeight", {
    value: 88,
    configurable: true,
  });
});

describe("el desplazamiento que pide el asa es una cuenta pura", () => {
  it("parte del de ahora, suma lo que bajó el puntero y nunca baja de 0", () => {
    expect(desplazamientoAlArrastrar(0, 100, 160)).toBe(60);
    expect(desplazamientoAlArrastrar(132, 500, 510)).toBe(142);
    expect(desplazamientoAlArrastrar(132, 500, 480)).toBe(112);
    expect(desplazamientoAlArrastrar(20, 500, 100)).toBe(0);
    expect(desplazamientoAlArrastrar(0, 100, 100.4)).toBe(0);
    expect(desplazamientoAlArrastrar(10, 100, 110.6)).toBe(21);
  });
});

describe("en el Mac la banda no cambia: ni asa de altura, ni «×», ni la clase que les hace sitio", () => {
  it("con la plataforma del Mac", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_MACOS);
    const { container } = await pinta(<Banda estado="esperando" />);
    expect(container.querySelector(".cerrar-b")).toBeNull();
    expect(container.querySelector(".asa-altura")).toBeNull();
    expect(container.querySelector(".banda")?.className).toBe("banda");
    expect(container.querySelector("[data-con-cerrar]")).toBeNull();
    expect(
      screen.queryByRole("button", { name: /Cerrar la banda/ }),
    ).toBeNull();
  });

  it("y Sesión no enseña la altura ni «Mostrar la banda», ni con la banda cerrada", async () => {
    conRust(LA_FRANJA_CERRADA, PLATAFORMA_MACOS);
    await pinta(
      <Sesion
        reunion={{ que: "ninguna" }}
        escucha={{ ...ESTADO_DE_LA_ESCUCHA, escuchando: false }}
        salida={{ salida: "auriculares" }}
      />,
    );
    const t = TEXTOS_DE_WINDOWS.es.banda;
    expect(screen.queryByText(t.alturaTitulo)).toBeNull();
    expect(screen.queryByText(t.cerradaTitulo)).toBeNull();
    expect(screen.queryByRole("button", { name: t.mostrar })).toBeNull();
  });
});

describe("en Windows la banda tiene su «×» y, arriba, su asa de altura", () => {
  it("el «×» tiene nombre accesible y le pide a Rust cerrar la banda", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_WINDOWS);
    const { container } = await pinta(<Banda estado="esperando" />);
    const cerrar = screen.getByRole("button", {
      name: TEXTOS_DE_WINDOWS.es.banda.cerrar,
    });
    expect(cerrar).toHaveAttribute("title", TEXTOS_DE_WINDOWS.es.banda.cerrar);
    expect(container.querySelector(".banda")).toHaveAttribute(
      "data-con-cerrar",
    );
    fireEvent.click(cerrar);
    expect(pedidos("cerrar_la_banda")).toEqual([["cerrar_la_banda"]]);
  });

  it("con la sesión en marcha el «×» lo dice: cierra y termina", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_WINDOWS);
    respuestas.set("estado_de_la_escucha", {
      ...ESTADO_DE_LA_ESCUCHA,
      escuchando: true,
    });
    await pinta(<Banda estado="esperando" />);
    const t = TEXTOS_DE_WINDOWS.es.banda;
    expect(
      await screen.findByRole("button", { name: t.cerrarYTerminar }),
    ).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: t.cerrar })).toBeNull();
  });

  it("en inglés, con las palabras de la Surface", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_WINDOWS);
    await pinta(<Banda estado="esperando" />, "en");
    expect(
      screen.getByRole("button", { name: "Close the band" }),
    ).toBeInTheDocument();
    expect(
      screen.getByTitle("Drag to choose how low the band sits"),
    ).toBeInTheDocument();
  });

  it("el asa de altura sale arriba y no abajo; el «×» sale en los dos", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_WINDOWS);
    const arriba = await pinta(<Banda estado="esperando" />);
    expect(arriba.container.querySelectorAll(".asa")).toHaveLength(1);
    expect(arriba.container.querySelector(".asa-altura")).not.toBeNull();
    cleanup();
    conRust(LA_FRANJA_ABAJO, PLATAFORMA_WINDOWS);
    const abajo = await pinta(<Banda estado="esperando" />);
    expect(abajo.container.querySelector(".asa-altura")).toBeNull();
    expect(abajo.container.querySelectorAll(".asa")).toHaveLength(1);
    expect(abajo.container.querySelector(".cerrar-b")).not.toBeNull();
  });

  it("la banda de voz, de 44 px, también los trae", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_WINDOWS);
    respuestas.set("estado_de_la_voz", LA_VOZ_DICIENDO);
    const { container } = await pinta(<Banda estado="esperando" />);
    expect(container.querySelector(".banda.voz")).toHaveAttribute(
      "data-con-cerrar",
    );
    expect(container.querySelector(".banda.voz .cerrar-b")).not.toBeNull();
    expect(container.querySelector(".banda.voz .asa-altura")).not.toBeNull();
  });
});

describe("el asa de altura: mientras arrastras se mueve lo nuestro; al soltar, se guarda", () => {
  it("arrastrar hacia abajo desplaza la banda y soltar guarda esa altura", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_WINDOWS);
    const { container } = await pinta(<Banda estado="esperando" />);
    const asa = container.querySelector(".asa-altura") as HTMLElement;
    fireEvent.pointerDown(asa, { screenY: 40 });
    fireEvent.pointerMove(ventana, { screenY: 100 });
    expect(pedidos("desplazar_banda")).toEqual([
      ["desplazar_banda", { desplazamiento: 60 }],
    ]);
    expect(
      pedidos("asentar_desplazamiento"),
      "se guardó antes de soltar",
    ).toEqual([]);
    fireEvent.pointerMove(ventana, { screenY: 172 });
    fireEvent.pointerUp(ventana, { screenY: 172 });
    expect(pedidos("desplazar_banda").at(-1)).toEqual([
      "desplazar_banda",
      { desplazamiento: 132 },
    ]);
    expect(pedidos("asentar_desplazamiento")).toEqual([
      ["asentar_desplazamiento", { desplazamiento: 132 }],
    ]);
  });

  it("un clic sin mover no guarda nada ni reacopla", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_WINDOWS);
    const { container } = await pinta(<Banda estado="esperando" />);
    const asa = container.querySelector(".asa-altura") as HTMLElement;
    fireEvent.pointerDown(asa, { screenY: 40 });
    fireEvent.pointerUp(ventana, { screenY: 40 });
    expect(pedidos("desplazar_banda")).toEqual([]);
    expect(
      pedidos("asentar_desplazamiento"),
      "un clic en el asa movió la reunión",
    ).toEqual([]);
  });

  it("parte de la altura de ahora, y subir más allá del borde no pasa de 0", async () => {
    conRust(LA_FRANJA_BAJADA, PLATAFORMA_WINDOWS);
    const { container } = await pinta(<Banda estado="esperando" />);
    const asa = container.querySelector(".asa-altura") as HTMLElement;
    fireEvent.pointerDown(asa, { screenY: 500 });
    fireEvent.pointerMove(ventana, { screenY: 510 });
    expect(pedidos("desplazar_banda").at(-1)).toEqual([
      "desplazar_banda",
      { desplazamiento: 142 },
    ]);
    fireEvent.pointerMove(ventana, { screenY: 100 });
    fireEvent.pointerUp(ventana, { screenY: 100 });
    expect(pedidos("asentar_desplazamiento")).toEqual([
      ["asentar_desplazamiento", { desplazamiento: 0 }],
    ]);
  });

  it("el asa de alto de siempre no se toca: sigue pidiendo `ajustar_banda`", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_WINDOWS);
    const { container } = await pinta(<Banda estado="esperando" />);
    const asaDeAlto = container.querySelector(
      ".asa:not(.posicion)",
    ) as HTMLElement;
    fireEvent.pointerDown(asaDeAlto, { screenY: 121 });
    fireEvent.pointerMove(ventana, { screenY: 171 });
    fireEvent.pointerUp(ventana, { screenY: 171 });
    expect(pedidos("ajustar_banda")).toEqual([
      ["ajustar_banda", { alto: 138 }],
    ]);
    expect(pedidos("desplazar_banda")).toEqual([]);
  });

  it("la banda de voz, tras cambiar de anatomía, sigue obedeciendo al asa (el oyente es de la ventana)", async () => {
    conRust(LA_FRANJA_ARRIBA, PLATAFORMA_WINDOWS);
    const { container } = await pinta(<Banda estado="esperando" />);
    expect(container.querySelector(".banda.voz")).toBeNull();
    // Rust enciende el modo solo audio: la banda cambia de árbol y el asa vieja desaparece con él.
    await act(async () => oyentes.get("voz")!(LA_VOZ_DICIENDO));
    const asa = container.querySelector(
      ".banda.voz .asa-altura",
    ) as HTMLElement;
    expect(asa, "la banda de voz no trae el asa de altura").not.toBeNull();
    fireEvent.pointerDown(asa, { screenY: 10 });
    fireEvent.pointerMove(ventana, { screenY: 70 });
    fireEvent.pointerUp(ventana, { screenY: 70 });
    expect(pedidos("asentar_desplazamiento")).toEqual([
      ["asentar_desplazamiento", { desplazamiento: 60 }],
    ]);
  });
});

describe("Sesión: la altura de la banda y «Mostrar la banda»", () => {
  const sesion = (
    <Sesion
      reunion={{ que: "ninguna" }}
      escucha={{ ...ESTADO_DE_LA_ESCUCHA, escuchando: false }}
      salida={{ salida: "auriculares" }}
    />
  );
  const t = TEXTOS_DE_WINDOWS.es.banda;

  it("pegada arriba dice dónde está y no ofrece «Volver arriba»", async () => {
    conRust({ ...LA_FRANJA_ARRIBA, avisoVisto: true }, PLATAFORMA_WINDOWS);
    await pinta(sesion);
    expect(screen.getByText(t.alturaTitulo)).toBeInTheDocument();
    expect(screen.getByText(t.pegadaArriba)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: t.volverArriba })).toBeNull();
    expect(screen.queryByText(t.cerradaTitulo)).toBeNull();
  });

  it("bajada, dice cuánto, y «Volver arriba» la devuelve (guarda 0)", async () => {
    conRust(LA_FRANJA_BAJADA, PLATAFORMA_WINDOWS);
    await pinta(sesion);
    expect(screen.getByText(t.alturaValor(132))).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: t.volverArriba }));
    expect(pedidos("asentar_desplazamiento")).toEqual([
      ["asentar_desplazamiento", { desplazamiento: 0 }],
    ]);
  });

  it("con la banda cerrada lo dice y «Mostrar la banda» se la pide a Rust", async () => {
    conRust(LA_FRANJA_CERRADA, PLATAFORMA_WINDOWS);
    await pinta(sesion);
    expect(screen.getByText(t.cerradaTitulo)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: t.mostrar }));
    expect(pedidos("mostrar_la_banda")).toEqual([["mostrar_la_banda"]]);
  });

  it("se entera por el evento: el «×» cierra la banda y Sesión ofrece volver a abrirla", async () => {
    conRust(LA_FRANJA_BAJADA, PLATAFORMA_WINDOWS);
    await pinta(sesion);
    expect(screen.queryByRole("button", { name: t.mostrar })).toBeNull();
    await act(async () => oyentes.get("franja")!(LA_FRANJA_CERRADA));
    expect(screen.getByRole("button", { name: t.mostrar })).toBeInTheDocument();
    await act(async () => oyentes.get("franja")!(LA_FRANJA_BAJADA));
    expect(screen.queryByRole("button", { name: t.mostrar })).toBeNull();
  });

  it("abajo no hay altura que elegir, pero la banda cerrada se puede mostrar", async () => {
    conRust({ ...LA_FRANJA_ABAJO, bandaAbierta: false }, PLATAFORMA_WINDOWS);
    await pinta(sesion);
    expect(screen.queryByText(t.alturaTitulo)).toBeNull();
    expect(screen.getByRole("button", { name: t.mostrar })).toBeInTheDocument();
  });
});

describe("los textos de Windows de la banda: en los dos idiomas, completos y sin una palabra del Mac", () => {
  const MAC =
    /macOS|\bMac\b|Llavero|Keychain|Touch ID|Finder|launchd|⌃|⌥|Ítems de inicio|Login Items/;
  const hojas = (b: (typeof TEXTOS_DE_WINDOWS)["es"]["banda"]) =>
    Object.values(b).map((v) => (typeof v === "function" ? v(132) : v));

  it("las dos listas dicen lo mismo y ninguna nombra el Mac", () => {
    const es = hojas(TEXTOS_DE_WINDOWS.es.banda);
    const en = hojas(TEXTOS_DE_WINDOWS.en.banda);
    expect(es).toHaveLength(en.length);
    for (const frase of [...es, ...en]) {
      expect(frase.length, "una frase vacía").toBeGreaterThan(3);
      expect(frase, `nombra el Mac: ${frase}`).not.toMatch(MAC);
    }
    for (const [i, frase] of es.entries())
      expect(frase, "sin traducir").not.toBe(en[i]);
  });
});
