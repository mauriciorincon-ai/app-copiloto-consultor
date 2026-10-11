import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Cascara } from "@/App";
import { SpriteIconos } from "@/componentes/Iconos";
import { Banda } from "@/componentes/Banda";
import { Sesion } from "@/pantallas/Sesion";
import { Honestidad } from "@/pantallas/Honestidad";
import { es } from "@/i18n/es";
import { llamar, preguntar } from "@/puente";
import {
  ESTADO_DE_LA_ESCUCHA,
  ESTADO_DE_LA_ESCUCHA_PRESENCIAL,
  LA_VOZ_ESPERANDO_A_LA_SALA,
  VISTA_DEL_CLIENTE,
} from "@/contrato.generado";
import type { EstadoDeEscucha, Salida } from "@/cuaderno";

/**
 * **EL MODO PRESENCIAL EN LA INTERFAZ** (sprint 005, fase 2, ADR 020) — Sesión, la banda y Honestidad con lo que
 * Rust les da, y lo que le piden.
 *
 * Se finge el PUENTE, como en `este-cliente.test.tsx`, y las respuestas salen de `src/contrato.generado.ts`, que
 * escribe Rust con su serde (regla 19): la escucha presencial y la voz que espera a la sala son muestras de Rust.
 *
 * ¿Puede fallar? Sí: con «Empezar en presencial» pidiendo siempre la ficha al oído, el caso de la banda es rojo; y
 * con la banda pintando el chip de la reunión en la sala, el de la cabecera (bitácora del sprint 005, fase 2).
 */
const tp = es.cuaderno.presencial;
const th = es.cuaderno.enPresencial;
const tb = es.banda;
const respuestas = new Map<string, unknown>();
let enTauri = true;

vi.mock("@/puente", () => ({
  hayTauri: () => enTauri,
  escuchar: () => () => {},
  preguntar: vi.fn((comando: string) =>
    Promise.resolve(respuestas.get(comando) ?? null),
  ),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

const pedidos = (comando: string) =>
  [...vi.mocked(preguntar).mock.calls, ...vi.mocked(llamar).mock.calls].filter(
    ([c]) => c === comando,
  );

const antesDeEmpezar: EstadoDeEscucha = {
  ...ESTADO_DE_LA_ESCUCHA,
  escuchando: false,
};
const AIRPODS: Salida = { salida: "otra", nombre: "AirPods Pro" };

async function pintaSesion(
  escucha: EstadoDeEscucha = antesDeEmpezar,
  salida: Salida = AIRPODS,
) {
  document.documentElement.lang = "es";
  const r = render(
    <Cascara>
      <SpriteIconos />
      <Sesion reunion={{ que: "ninguna" }} escucha={escucha} salida={salida} />
    </Cascara>,
  );
  await act(async () => {});
  return r;
}

const radio = (nombre: string) => screen.getByRole("radio", { name: nombre });

beforeEach(() => {
  cleanup();
  enTauri = true;
  respuestas.clear();
  vi.mocked(preguntar).mockClear();
  vi.mocked(llamar).mockClear();
  respuestas.set("este_cliente", VISTA_DEL_CLIENTE);
});

describe("Sesión en presencial", () => {
  it("«Presencial» enseña la sala sin empezar nada, y «reunión en este Mac» vuelve", async () => {
    await pintaSesion();
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    expect(
      screen.getByRole("heading", { name: tp.titulo }),
    ).toBeInTheDocument();
    expect(screen.getByText(tp.pcPregunta)).toBeInTheDocument();
    expect(screen.getByText(tp.loTuyoQueda)).toBeInTheDocument();
    // Elegir el modo no abre nada: ni la sala ni la reunión.
    expect(pedidos("empezar_presencial")).toHaveLength(0);
    expect(pedidos("empezar_a_escuchar")).toHaveLength(0);
    fireEvent.click(radio(tp.reunionEnEsteMac));
    expect(screen.queryByRole("heading", { name: tp.titulo })).toBeNull();
    expect(
      screen.getByRole("button", { name: es.cuaderno.iniciarSesion }),
    ).toBeInTheDocument();
  });

  it("«Empezar en presencial» pide la sala con la ficha al oído, y con la banda si la eliges", async () => {
    await pintaSesion();
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    expect(radio(tp.alOido)).toHaveAttribute("aria-checked", "true");
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: tp.empezar }));
    });
    expect(pedidos("empezar_presencial").at(-1)?.[1]).toEqual({ alOido: true });
    fireEvent.click(radio(tp.laBanda));
    expect(radio(tp.laBanda)).toHaveAttribute("aria-checked", "true");
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: tp.empezar }));
    });
    expect(pedidos("empezar_presencial").at(-1)?.[1]).toEqual({
      alOido: false,
    });
    expect(pedidos("empezar_a_escuchar")).toHaveLength(0);
  });

  it("con altavoces que la app reconoce, la banda viene marcada (ADR 020 §7)", async () => {
    await pintaSesion(antesDeEmpezar, { salida: "altavoces" });
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    expect(radio(tp.laBanda)).toHaveAttribute("aria-checked", "true");
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: tp.empezar }));
    });
    expect(pedidos("empezar_presencial").at(-1)?.[1]).toEqual({
      alOido: false,
    });
  });

  it("el idioma de la sala se elige ahí y Rust lo guarda como el de la pista «sala»", async () => {
    await pintaSesion();
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    fireEvent.change(
      screen.getByRole("combobox", { name: tp.idiomaDeLaSala }),
      { target: { value: "en-US" } },
    );
    expect(pedidos("fijar_idioma_de_pista").at(-1)?.[1]).toEqual({
      pista: "sala",
      idioma: "en-US",
    });
  });

  it("en marcha: la sala funciona, el sistema y la pantalla no se abren —sin «A medias»— y la ficha se cambia", async () => {
    respuestas.set("estado_de_la_voz", LA_VOZ_ESPERANDO_A_LA_SALA);
    await pintaSesion(ESTADO_DE_LA_ESCUCHA_PRESENCIAL);
    expect(
      screen.getByRole("heading", { name: tp.titulo }),
    ).toBeInTheDocument();
    expect(screen.getByText(tp.enMarcha)).toBeInTheDocument();
    expect(screen.getByText(es.cuaderno.funciona)).toBeInTheDocument();
    expect(screen.getAllByText(tp.noSeAbre)).toHaveLength(2);
    expect(screen.queryByText(es.cuaderno.aMedias)).toBeNull();
    // El idioma, fijo mientras dura; y nada que empezar.
    expect(
      screen.queryByRole("combobox", { name: tp.idiomaDeLaSala }),
    ).toBeNull();
    expect(screen.queryByRole("button", { name: tp.empezar })).toBeNull();
    expect(
      screen.getByRole("button", { name: es.cuaderno.terminarSesion }),
    ).toBeInTheDocument();
    // La voz encendida es «al oído»; elegir la banda se lo pide a Rust.
    expect(radio(tp.alOido)).toHaveAttribute("aria-checked", "true");
    await act(async () => {
      fireEvent.click(radio(tp.laBanda));
    });
    expect(pedidos("fijar_la_voz").at(-1)?.[1]).toEqual({ alOido: false });
  });
});

describe("la banda en presencial", () => {
  it("la cabecera dice la sala, sin el chip de protección de una videollamada", async () => {
    respuestas.set("estado_de_la_escucha", ESTADO_DE_LA_ESCUCHA_PRESENCIAL);
    render(
      <Cascara>
        <Banda estado="esperando" />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.getByText(tb.escuchandoLaSala)).toBeInTheDocument();
    expect(screen.getByText(tb.presencialLaSala)).toBeInTheDocument();
    expect(screen.queryByText(es.cuaderno.sinReunion)).toBeNull();
    expect(screen.queryByText(/Escuchando · \d/)).toBeNull();
  });

  it("al oído, con la sala hablando, la banda lo dice y ⌃⌥V la enseña", async () => {
    respuestas.set("estado_de_la_voz", LA_VOZ_ESPERANDO_A_LA_SALA);
    const { container } = render(
      <Cascara>
        <Banda estado="esperando" />
      </Cascara>,
    );
    await act(async () => {});
    const banda = container.querySelector("section.banda.voz");
    expect(banda).not.toBeNull();
    expect(banda).toHaveAttribute("data-estado", "voz-sala");
    expect(
      within(banda as HTMLElement).getByText(tb.laSalaHabla),
    ).toBeInTheDocument();
    expect(
      within(banda as HTMLElement).getByText(tb.teLaDigoCuandoCalle),
    ).toBeInTheDocument();
    expect(banda?.textContent).toContain(`⌃⌥V ${tb.verla}`);
    // No suena: `⎋ callar` solo existe mientras suena.
    expect(banda?.textContent).not.toContain(tb.callar);
  });

  it("el transcript de la sala no tiene dueño: cada turno dice «sala»", () => {
    enTauri = false;
    const { container } = render(
      <Cascara>
        <Banda estado="ficha-presencial" transcript ampliada />
      </Cascara>,
    );
    const quienes = [
      ...container.querySelectorAll(".transcript-b .turno .quien"),
    ].map((q) => q.textContent ?? "");
    expect(quienes.length).toBeGreaterThan(0);
    for (const q of quienes) {
      expect(q.startsWith(tb.sala)).toBe(true);
      expect(q).not.toContain(tb.cliente);
    }
    expect(container.querySelector(".transcript-b .turno.tu")).toBeNull();
  });
});

describe("Honestidad en presencial", () => {
  it("una pista, la sala; lo que no se abre, en 0 B; y al cerrar, tus turnos no", async () => {
    render(
      <Cascara>
        <Honestidad bytes="0 B" escucha={ESTADO_DE_LA_ESCUCHA_PRESENCIAL} />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.getByText(th.bufSala)).toBeInTheDocument();
    expect(screen.getByText(th.noSeAbre)).toBeInTheDocument();
    expect(screen.getByText(th.noSeLee)).toBeInTheDocument();
    expect(screen.getByText(th.transcript)).toBeInTheDocument();
    expect(screen.getByText(th.salieron)).toBeInTheDocument();
    expect(screen.getByText(th.tusTurnosNo)).toBeInTheDocument();
    expect(screen.queryByText(es.cuaderno.bufMic)).toBeNull();
    expect(screen.queryByText(es.notas.deCliente)).toBeNull();
  });
});
