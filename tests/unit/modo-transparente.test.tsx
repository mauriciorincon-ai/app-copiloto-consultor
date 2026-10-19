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
import { Notas } from "@/pantallas/Notas";
import { TEXTOS_DE_WINDOWS } from "@/i18n/windows";
import { llamar, preguntar } from "@/puente";
import {
  ESTADO_DE_LA_ESCUCHA,
  LA_FRANJA_ARRIBA,
  LA_VOZ_APAGADA,
  LA_VOZ_DICIENDO,
  LISTA_DE_REUNIONES,
  PLATAFORMA_MACOS,
  PLATAFORMA_WINDOWS,
  VISTA_DEL_CLIENTE,
  VISTA_DEL_CLIENTE_TRANSPARENTE,
  VISTA_DEL_CUADERNO,
} from "@/contrato.generado";
import type { EstadoDeEscucha } from "@/cuaderno";

/**
 * **EL MODO TRANSPARENTE EN LA INTERFAZ** (sprint 012, ADR 024) — lo que la webview hace con las cuatro puertas de Rust
 * (`fijar_modo_transparente`, `el_cliente_consintio`, `decir_el_aviso`, `modo_transparente`). Solo Windows, y sin maqueta.
 *
 * Se finge el PUENTE como en `este-cliente.test.tsx`, y las respuestas salen de `src/contrato.generado.ts`, que escribe
 * Rust con su serde (regla 19). Lo que se vigila:
 *
 *  1. **En el Mac no cambia nada:** ni el interruptor, ni el aviso, ni la marca de la banda, ni la fila de «Se va a guardar»,
 *     aunque Rust mande el modo encendido.
 *  2. **El interruptor** pide a Rust el cambio, y el aviso fijo se enseña en los dos idiomas, el de la interfaz primero.
 *  3. **«El cliente consintió»** no llama a Rust sin una reunión abierta; con ella, deja la constancia y lo dice; si Rust
 *     dice que no, lo dice también.
 *  4. **La voz** solo se ofrece con el modo solo audio encendido, y dice si cupo o no.
 *  5. **La banda solo sabe si el modo está encendido:** nunca pregunta por el cliente ni puede encenderlo.
 *  6. **«Se va a guardar»** cuenta las constancias, y solo con alguna.
 *
 * ¿Puede fallar? Sí: sin el `esWindows(plataforma) &&` de Sesión el Mac vería el interruptor; sin el `if (!abierta)` de
 * `consintio` se pediría una constancia sin reunión; sin `voz.encendida` la voz se ofrecería con el modo apagado; y si la
 * banda llamara a `este_cliente` el último caso de (5) es rojo.
 */
const tt = TEXTOS_DE_WINDOWS.es.transparente;
const ttEn = TEXTOS_DE_WINDOWS.en.transparente;
const respuestas = new Map<string, unknown>();
const oyentes = new Map<string, (dato: unknown) => void>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: vi.fn((evento: string, alOir: (dato: unknown) => void) => {
    oyentes.set(evento, alOir);
    return () => {};
  }),
  preguntar: vi.fn((comando: string) => {
    const r = respuestas.get(comando);
    return r instanceof Error ? Promise.reject(r) : Promise.resolve(r ?? null);
  }),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

const pedidos = (comando: string) =>
  [...vi.mocked(preguntar).mock.calls, ...vi.mocked(llamar).mock.calls].filter(
    ([c]) => c === comando,
  );

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

/** La línea de estado del modo transparente: Sesión tiene otros `role="status"`, y este es solo el suyo. */
const estadoDelAviso = (c: HTMLElement) => c.querySelector(".transparente [role=status]") as HTMLElement;

const sesion = (escucha: Partial<EstadoDeEscucha> = {}) => (
  <Sesion
    reunion={{ que: "ninguna" }}
    escucha={{ ...ESTADO_DE_LA_ESCUCHA, escuchando: false, ...escucha }}
    salida={{ salida: "auriculares" }}
  />
);

/** La plataforma y el cliente que diga Rust. */
function conRust(plataforma: unknown, cliente: unknown = VISTA_DEL_CLIENTE) {
  respuestas.set("plataforma_de_este_equipo", plataforma);
  respuestas.set("este_cliente", cliente);
}

beforeEach(() => {
  cleanup();
  respuestas.clear();
  oyentes.clear();
  vi.mocked(preguntar).mockClear();
  vi.mocked(llamar).mockClear();
  respuestas.set("la_franja", LA_FRANJA_ARRIBA);
});

describe("en el Mac el modo transparente no se pinta, ni siquiera si Rust lo manda encendido", () => {
  it("Sesión no tiene interruptor ni aviso", async () => {
    conRust(PLATAFORMA_MACOS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    const { container } = await pinta(sesion());
    expect(
      screen.queryByRole("switch", { name: new RegExp(tt.titulo) }),
    ).toBeNull();
    expect(container.textContent).not.toContain(tt.loQueLeDices);
    expect(container.textContent).not.toContain(
      VISTA_DEL_CLIENTE_TRANSPARENTE.aviso.es,
    );
    expect(container.querySelector(".transparente")).toBeNull();
  });

  it("la banda no lleva marca ni pregunta por el modo", async () => {
    conRust(PLATAFORMA_MACOS);
    respuestas.set("modo_transparente", true);
    const { container } = await pinta(<Banda estado="esperando" />);
    expect(container.querySelector("[data-transparente]")).toBeNull();
    expect(pedidos("modo_transparente")).toEqual([]);
  });

  it("«Se va a guardar» no cuenta constancias", async () => {
    conRust(PLATAFORMA_MACOS);
    respuestas.set("reuniones_guardadas", LISTA_DE_REUNIONES);
    respuestas.set("cuaderno_de_la_reunion", {
      ...VISTA_DEL_CUADERNO,
      resumen: { ...VISTA_DEL_CUADERNO.resumen, consentimientos: 2 },
    });
    await pinta(<Notas />);
    expect(screen.queryByText(tt.constancia)).toBeNull();
  });
});

describe("Sesión en Windows: el interruptor y el aviso fijo en los dos idiomas", () => {
  it("sin cliente elegido no hay interruptor: el modo es de un cliente", async () => {
    conRust(PLATAFORMA_WINDOWS, {
      ...VISTA_DEL_CLIENTE,
      elegido: null,
      bandera: null,
    });
    await pinta(sesion());
    expect(
      screen.queryByRole("switch", { name: new RegExp(tt.titulo) }),
    ).toBeNull();
  });

  it("apagado: se ofrece, no enseña el aviso, y encenderlo se lo pide a Rust", async () => {
    conRust(PLATAFORMA_WINDOWS);
    respuestas.set("fijar_modo_transparente", VISTA_DEL_CLIENTE_TRANSPARENTE);
    const { container } = await pinta(sesion());
    const interruptor = screen.getByRole("switch", {
      name: new RegExp(tt.titulo),
    });
    expect(interruptor).toHaveAttribute("aria-checked", "false");
    expect(interruptor).toHaveTextContent(tt.apagado);
    expect(container.textContent).not.toContain(
      VISTA_DEL_CLIENTE_TRANSPARENTE.aviso.es,
    );
    await act(async () => {
      fireEvent.click(interruptor);
    });
    expect(pedidos("fijar_modo_transparente").at(-1)?.[1]).toEqual({
      encendido: true,
    });
    expect(
      screen.getByRole("switch", { name: new RegExp(tt.titulo) }),
    ).toHaveAttribute("aria-checked", "true");
    expect(
      screen.getByRole("switch", { name: new RegExp(tt.titulo) }),
    ).toHaveTextContent(tt.encendido);
  });

  it("encendido: el aviso sale en español e inglés, cada uno con su lang, y el de la interfaz primero", async () => {
    conRust(PLATAFORMA_WINDOWS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    for (const lang of ["es", "en"] as const) {
      cleanup();
      const { container } = await pinta(sesion(), lang);
      const avisos = [...container.querySelectorAll(".transparente p[lang]")];
      expect(
        avisos.map((p) => p.getAttribute("lang")),
        `orden del aviso · ${lang}`,
      ).toEqual(lang === "es" ? ["es", "en"] : ["en", "es"]);
      expect(avisos[0].textContent).toBe(
        VISTA_DEL_CLIENTE_TRANSPARENTE.aviso[lang],
      );
      expect(avisos[1].textContent).toBe(
        VISTA_DEL_CLIENTE_TRANSPARENTE.aviso[lang === "es" ? "en" : "es"],
      );
      expect(
        screen.getByText(TEXTOS_DE_WINDOWS[lang].transparente.loQueLeDices),
      ).toBeInTheDocument();
    }
  });

  it("apagarlo también se lo pide a Rust", async () => {
    conRust(PLATAFORMA_WINDOWS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    respuestas.set("fijar_modo_transparente", VISTA_DEL_CLIENTE);
    await pinta(sesion());
    await act(async () => {
      fireEvent.click(
        screen.getByRole("switch", { name: new RegExp(tt.titulo) }),
      );
    });
    expect(pedidos("fijar_modo_transparente").at(-1)?.[1]).toEqual({
      encendido: false,
    });
    expect(screen.queryByText(tt.loQueLeDices)).toBeNull();
  });
});

describe("«El cliente consintió»", () => {
  const boton = () =>
    screen.getByRole("button", { name: new RegExp(tt.consintio) });

  it("sin reunión abierta se ve apagado, lo dice y no llama a Rust", async () => {
    conRust(PLATAFORMA_WINDOWS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    await pinta(sesion({ escuchando: false, soloNotas: false }));
    expect(boton()).toHaveAttribute("aria-disabled", "true");
    expect(screen.getByText(tt.sinReunion)).toBeInTheDocument();
    await act(async () => {
      fireEvent.click(boton());
    });
    expect(pedidos("el_cliente_consintio")).toEqual([]);
  });

  it("con la sesión en marcha deja la constancia y lo dice", async () => {
    conRust(PLATAFORMA_WINDOWS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    respuestas.set("el_cliente_consintio", true);
    const { container } = await pinta(sesion({ escuchando: true }));
    expect(boton()).not.toHaveAttribute("aria-disabled");
    expect(screen.getByText(tt.consintioAyuda)).toBeInTheDocument();
    await act(async () => {
      fireEvent.click(boton());
    });
    expect(pedidos("el_cliente_consintio")).toHaveLength(1);
    expect(estadoDelAviso(container)).toHaveTextContent(tt.anotada);
  });

  it("en solo notas no se ofrece: nada se captura, y no hay aviso que dar", async () => {
    conRust(PLATAFORMA_WINDOWS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    const { container } = await pinta(sesion({ escuchando: false, soloNotas: true }));
    expect(container.querySelector(".transparente")).toBeNull();
    expect(pedidos("el_cliente_consintio")).toEqual([]);
  });

  it("si Rust dice que no (la reunión se cerró), la pantalla no finge que se anotó", async () => {
    conRust(PLATAFORMA_WINDOWS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    respuestas.set(
      "el_cliente_consintio",
      new Error("no hay una reunión abierta donde dejar la constancia"),
    );
    const { container } = await pinta(sesion({ escuchando: true }));
    await act(async () => {
      fireEvent.click(boton());
    });
    expect(estadoDelAviso(container)).toHaveTextContent(tt.noAnotada);
    expect(screen.queryByText(tt.anotada)).toBeNull();
  });

  it("en inglés, los textos son los del diccionario inglés", async () => {
    conRust(PLATAFORMA_WINDOWS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    respuestas.set("el_cliente_consintio", true);
    const { container } = await pinta(sesion({ escuchando: true }), "en");
    await act(async () => {
      fireEvent.click(
        screen.getByRole("button", { name: new RegExp(ttEn.consintio) }),
      );
    });
    expect(estadoDelAviso(container)).toHaveTextContent(ttEn.anotada);
  });
});

describe("el aviso al oído: solo con el modo solo audio encendido", () => {
  it("con la voz apagada no se ofrece", async () => {
    conRust(PLATAFORMA_WINDOWS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    respuestas.set("estado_de_la_voz", LA_VOZ_APAGADA);
    await pinta(sesion({ escuchando: true }));
    expect(
      screen.queryByRole("button", { name: new RegExp(tt.decirlo) }),
    ).toBeNull();
  });

  it("con la voz encendida se ofrece, y dice si cupo", async () => {
    conRust(PLATAFORMA_WINDOWS, VISTA_DEL_CLIENTE_TRANSPARENTE);
    respuestas.set("estado_de_la_voz", LA_VOZ_DICIENDO);
    respuestas.set("decir_el_aviso", true);
    const { container } = await pinta(sesion({ escuchando: true }));
    await act(async () => {
      fireEvent.click(
        screen.getByRole("button", { name: new RegExp(tt.decirlo) }),
      );
    });
    expect(pedidos("decir_el_aviso")).toHaveLength(1);
    expect(estadoDelAviso(container)).toHaveTextContent(tt.dicho);

    respuestas.set("decir_el_aviso", false);
    await act(async () => {
      fireEvent.click(
        screen.getByRole("button", { name: new RegExp(tt.decirlo) }),
      );
    });
    expect(estadoDelAviso(container)).toHaveTextContent(tt.noCabe);
  });
});

describe("la banda solo sabe si el modo está encendido", () => {
  it("lleva la marca en Windows cuando Rust dice que sí, y se la quita cuando avisa que no", async () => {
    conRust(PLATAFORMA_WINDOWS);
    respuestas.set("modo_transparente", true);
    const { container } = await pinta(<Banda estado="esperando" />);
    expect(container.querySelector(".banda")).toHaveAttribute(
      "data-transparente",
    );
    expect(screen.getByText(tt.marcaDeLaBanda)).toBeInTheDocument();
    respuestas.set("modo_transparente", false);
    await act(async () => oyentes.get("transparente")!(undefined));
    expect(container.querySelector("[data-transparente]")).toBeNull();
    expect(screen.queryByText(tt.marcaDeLaBanda)).toBeNull();
  });

  it("la banda de voz, de 44 px, también la lleva", async () => {
    conRust(PLATAFORMA_WINDOWS);
    respuestas.set("modo_transparente", true);
    respuestas.set("estado_de_la_voz", LA_VOZ_DICIENDO);
    const { container } = await pinta(<Banda estado="esperando" />);
    expect(container.querySelector(".banda.voz")).toHaveAttribute(
      "data-transparente",
    );
    expect(container.querySelector(".banda.voz")).toHaveTextContent(
      tt.marcaDeLaBanda,
    );
  });

  it("sin el modo, ni marca", async () => {
    conRust(PLATAFORMA_WINDOWS);
    respuestas.set("modo_transparente", false);
    const { container } = await pinta(<Banda estado="esperando" />);
    expect(container.querySelector("[data-transparente]")).toBeNull();
  });

  it("no pregunta por el cliente ni puede encender el modo ni anotar: solo `modo_transparente`", async () => {
    conRust(PLATAFORMA_WINDOWS);
    respuestas.set("modo_transparente", true);
    await pinta(<Banda estado="esperando" />);
    for (const ajena of [
      "este_cliente",
      "fijar_modo_transparente",
      "el_cliente_consintio",
      "decir_el_aviso",
    ]) {
      expect(pedidos(ajena), `la banda llamó a ${ajena}`).toEqual([]);
    }
    expect(pedidos("modo_transparente").length).toBeGreaterThan(0);
  });
});

describe("«Se va a guardar» cuenta las constancias", () => {
  const cuaderno = (n: number) => ({
    ...VISTA_DEL_CUADERNO,
    resumen: { ...VISTA_DEL_CUADERNO.resumen, consentimientos: n },
  });

  it("con una o varias, la fila sale con su cuenta", async () => {
    conRust(PLATAFORMA_WINDOWS);
    respuestas.set("reuniones_guardadas", LISTA_DE_REUNIONES);
    respuestas.set("cuaderno_de_la_reunion", cuaderno(1));
    await pinta(<Notas />);
    expect(screen.getByText(tt.constancia)).toBeInTheDocument();
    expect(screen.getByText(tt.constanciaDonde(1))).toBeInTheDocument();
    cleanup();
    respuestas.set("cuaderno_de_la_reunion", cuaderno(2));
    await pinta(<Notas />);
    expect(screen.getByText(tt.constanciaDonde(2))).toBeInTheDocument();
  });

  it("sin ninguna, no hay fila: una reunión sin modo transparente se guarda como siempre", async () => {
    conRust(PLATAFORMA_WINDOWS);
    respuestas.set("reuniones_guardadas", LISTA_DE_REUNIONES);
    respuestas.set("cuaderno_de_la_reunion", cuaderno(0));
    await pinta(<Notas />);
    expect(screen.queryByText(tt.constancia)).toBeNull();
  });
});
