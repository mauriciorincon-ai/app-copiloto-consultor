import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Cascara } from "@/App";
import { SpriteIconos } from "@/componentes/Iconos";
import { Banda } from "@/componentes/Banda";
import { Relleno } from "@/componentes/Relleno";
import { Sesion } from "@/pantallas/Sesion";
import { es } from "@/i18n/es";
import { escuchar, llamar, preguntar } from "@/puente";
import { EVENTO_FRANJA, type LaFranja } from "@/franja";
import { ESTADO_DE_LA_ESCUCHA, LA_FRANJA_ABAJO, LA_FRANJA_ARRIBA } from "@/contrato.generado";

/**
 * **LA BANDA ARRIBA, DENTRO DE TAURI** (C1', sprint 004, ADR 004 enmienda 1) — lo que las tres ventanas
 * hacen con la franja que Rust les da.
 *
 * Se finge el PUENTE, como en `este-cliente.test.tsx`, y las respuestas salen de
 * `src/contrato.generado.ts`, que escribe Rust con su serde (regla 19): la banda dibuja su asa en el
 * borde que da a la reunión, el relleno sube el fondo lo que mide la barra de menús, y Sesión enseña el
 * interruptor y el aviso de la primera vez.
 *
 * ¿Puede fallar? Sí: con el relleno calculando siempre «abajo», el caso de arriba da `-812px` en vez de
 * `-38px`; y con «Entendido» sin llamar a Rust, el aviso vuelve en el arranque siguiente (bitácora).
 */
const t = es.cuaderno;
const respuestas = new Map<string, unknown>();
const oyentes = new Map<string, (dato: unknown) => void>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: vi.fn((evento: string, alOir: (dato: unknown) => void) => {
    oyentes.set(evento, alOir);
    return () => {};
  }),
  preguntar: vi.fn((comando: string) => Promise.resolve(respuestas.get(comando) ?? null)),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

const pedidos = (comando: string) =>
  [...vi.mocked(preguntar).mock.calls, ...vi.mocked(llamar).mock.calls].filter(([c]) => c === comando);

async function pinta(nodo: React.ReactNode) {
  document.documentElement.lang = "es";
  const r = render(
    <Cascara>
      <SpriteIconos />
      {nodo}
    </Cascara>,
  );
  await act(async () => {});
  return r;
}

const sesion = (
  <Sesion
    reunion={{ que: "ninguna" }}
    escucha={{ ...ESTADO_DE_LA_ESCUCHA, escuchando: false }}
    salida={{ salida: "auriculares" }}
  />
);

beforeEach(() => {
  cleanup();
  respuestas.clear();
  oyentes.clear();
  vi.mocked(preguntar).mockClear();
  vi.mocked(llamar).mockClear();
  vi.mocked(escuchar).mockClear();
});

describe("la banda dibuja su asa en el borde que da a la reunión", () => {
  it("arriba: la banda lo dice en `data-borde`, aunque la URL pidiera abajo", async () => {
    respuestas.set("la_franja", LA_FRANJA_ARRIBA);
    const { container } = await pinta(<Banda estado="esperando" borde="abajo" />);
    expect(container.querySelector(".banda")).toHaveAttribute("data-borde", "arriba");
  });

  it("se entera de ⌃⌥B sin sondear: escucha el evento de la franja", async () => {
    respuestas.set("la_franja", LA_FRANJA_ARRIBA);
    const { container } = await pinta(<Banda estado="esperando" />);
    expect(oyentes.has(EVENTO_FRANJA)).toBe(true);
    await act(async () => oyentes.get(EVENTO_FRANJA)!(LA_FRANJA_ABAJO));
    expect(container.querySelector(".banda")).toHaveAttribute("data-borde", "abajo");
  });
});

describe("el relleno sube el fondo lo que mide la barra de menús", () => {
  function pantalla(alto: number) {
    Object.defineProperty(globalThis.screen, "width", { value: 1512, configurable: true });
    Object.defineProperty(globalThis.screen, "height", { value: 982, configurable: true });
    Object.defineProperty(globalThis, "innerHeight", { value: alto, configurable: true });
  }

  it("arriba, -barra con cualquier alto del asa; abajo, la pantalla menos el alto", async () => {
    respuestas.set("fondo_del_relleno", "data:image/jpeg;base64,AAA");
    respuestas.set("la_franja", LA_FRANJA_ARRIBA);
    for (const alto of [88, 200]) {
      cleanup();
      pantalla(alto);
      const { container } = await pinta(<Relleno />);
      expect((container.querySelector(".relleno-fondo") as HTMLElement).style.top).toBe("-38px");
    }
    cleanup();
    respuestas.set("la_franja", LA_FRANJA_ABAJO);
    pantalla(88);
    const { container } = await pinta(<Relleno />);
    expect((container.querySelector(".relleno-fondo") as HTMLElement).style.top).toBe("-894px");
  });
});

describe("Sesión: la banda, arriba o abajo, y el aviso de la primera vez", () => {
  it("la fila «La banda» marca el borde de Rust y elegir el otro se lo pide a Rust", async () => {
    respuestas.set("la_franja", { ...LA_FRANJA_ARRIBA, avisoVisto: true } satisfies LaFranja);
    await pinta(sesion);
    const grupo = screen.getByRole("radiogroup", { name: t.laBanda });
    expect(screen.getByRole("radio", { name: t.bandaArriba })).toHaveAttribute("aria-checked", "true");
    expect(screen.getByRole("radio", { name: t.bandaAbajo })).toHaveAttribute("aria-checked", "false");
    expect(grupo.closest(".fila")).toHaveTextContent("⌃⌥B");
    fireEvent.click(screen.getByRole("radio", { name: t.bandaAbajo }));
    expect(pedidos("fijar_posicion_de_la_banda")).toEqual([["fijar_posicion_de_la_banda", { borde: "abajo" }]]);
  });

  /** Auditoría del S4, B25: pulsar el borde que ya está elegido no suelta ni vuelve a acoplar la reunión. */
  it("pulsar el borde que ya está elegido no le pide nada a Rust", async () => {
    respuestas.set("la_franja", { ...LA_FRANJA_ARRIBA, avisoVisto: true } satisfies LaFranja);
    await pinta(sesion);
    fireEvent.click(screen.getByRole("radio", { name: t.bandaArriba }));
    expect(pedidos("fijar_posicion_de_la_banda")).toEqual([]);
  });

  /** Auditoría del S4, M10: con un ensayo terminado sin guardar, Sesión lo dice antes de «Iniciar sesión». */
  it("con un ensayo sin guardar, Sesión avisa antes de descartarlo", async () => {
    respuestas.set("la_franja", { ...LA_FRANJA_ARRIBA, avisoVisto: true } satisfies LaFranja);
    await pinta(sesion);
    expect(screen.queryByText(t.ensayoSinGuardar, { exact: false })).toBeNull();
    expect(screen.getByText(t.nadaSale)).toBeInTheDocument();
    cleanup();
    await pinta(
      <Sesion reunion={{ que: "ninguna" }} escucha={{ ...ESTADO_DE_LA_ESCUCHA, escuchando: false }} salida={{ salida: "auriculares" }} ensayoSinGuardar />,
    );
    const aviso = screen.getByRole("status");
    expect(aviso).toHaveTextContent(t.ensayoSinGuardar);
    expect(aviso).toHaveTextContent(t.ensayoSinGuardarQue);
    // En una línea, en el sitio de la promesa: con dos, la pantalla se salía de su ventana (maqueta-cabe).
    expect(screen.queryByText(t.nadaSale)).toBeNull();
  });

  it("el aviso sale la primera vez con la banda arriba, y «Entendido» se lo dice a Rust", async () => {
    respuestas.set("la_franja", LA_FRANJA_ARRIBA);
    await pinta(sesion);
    expect(screen.getByRole("status")).toHaveTextContent(t.avisoArribaTitulo);
    fireEvent.click(screen.getByRole("button", { name: t.entendido }));
    expect(screen.queryByText(t.avisoArribaTitulo)).toBeNull();
    expect(pedidos("entendido_el_aviso_de_arriba")).toHaveLength(1);
  });

  it("ni con el aviso ya visto, ni con la banda abajo", async () => {
    respuestas.set("la_franja", LA_FRANJA_ABAJO);
    await pinta(sesion);
    expect(screen.queryByText(t.avisoArribaTitulo)).toBeNull();
    respuestas.set("la_franja", { ...LA_FRANJA_ABAJO, avisoVisto: false } satisfies LaFranja);
    cleanup();
    await pinta(sesion);
    expect(screen.queryByText(t.avisoArribaTitulo)).toBeNull();
  });
});

describe("el asa: soltarla sin arrastrar no mueve la reunión", () => {
  it("un clic no reacopla; un arrastre sí, al soltar", async () => {
    Object.defineProperty(globalThis, "innerHeight", { value: 88, configurable: true });
    respuestas.set("la_franja", LA_FRANJA_ARRIBA);
    const { container } = await pinta(<Banda estado="esperando" />);
    const asa = container.querySelector(".asa") as HTMLElement;
    fireEvent.pointerDown(asa, { screenY: 121 });
    fireEvent.pointerUp(globalThis as unknown as Window, { screenY: 121 });
    expect(pedidos("asentar_banda"), "un clic en el asa movió la reunión").toEqual([]);
    fireEvent.pointerDown(asa, { screenY: 121 });
    fireEvent.pointerMove(globalThis as unknown as Window, { screenY: 171 });
    fireEvent.pointerUp(globalThis as unknown as Window, { screenY: 171 });
    expect(pedidos("asentar_banda")).toEqual([["asentar_banda", { alto: 138 }]]);
  });
});
