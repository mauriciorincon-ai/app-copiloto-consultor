import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  within,
} from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Ia } from "@/pantallas/Ia";
import { IdiomaContext } from "@/i18n";
import { es } from "@/i18n/es";
import { llamar, preguntar } from "@/puente";
import { comandoDeGhost, type VistaDeLaPuerta } from "@/puerta";
import {
  ESTADO_DE_LA_IA_NADIE,
  VISTA_DE_LA_PUERTA_ABIERTA,
  VISTA_DE_LA_PUERTA_CERRADA,
  VISTA_DE_LA_PUERTA_EN_REUNION,
  VISTA_DE_LA_PUERTA_SIN_GHOST,
} from "@/contrato.generado";

/**
 * **LA PUERTA LOCAL EN IA** (C16, sprint 003, fase 4, ADR 018). Se finge el puente y las respuestas
 * salen de `src/contrato.generado.ts`, que escribe Rust con su serde (regla 19).
 *
 * ¿Puede fallar? Sí: con el conmutador llamando siempre a `abrir_la_puerta`, «abierta, el conmutador
 * la cierra» es rojo; y con la franja de la reunión aunque la puerta esté abierta, «se cerró sola…»
 * también (bitácora del S3, fase 4).
 */
const t = es.puerta;
const respuestas = new Map<string, unknown>();
let alOirLaPuerta: ((v: VistaDeLaPuerta) => void) | null = null;

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: (evento: string, alOir: (v: unknown) => void) => {
    if (evento === "puerta")
      alOirLaPuerta = alOir as (v: VistaDeLaPuerta) => void;
    return () => {};
  },
  preguntar: vi.fn((comando: string) =>
    Promise.resolve(respuestas.get(comando) ?? null),
  ),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

const pedidos = (comando: string) =>
  [...vi.mocked(preguntar).mock.calls, ...vi.mocked(llamar).mock.calls].filter(
    ([c]) => c === comando,
  );

async function pinta(busqueda = "?vista=puerta") {
  render(
    <IdiomaContext.Provider value="es">
      <Ia busqueda={busqueda} />
    </IdiomaContext.Provider>,
  );
  await act(async () => {});
}

beforeEach(() => {
  cleanup();
  respuestas.clear();
  alOirLaPuerta = null;
  vi.mocked(preguntar).mockClear();
  respuestas.set("estado_de_la_ia", ESTADO_DE_LA_IA_NADIE);
  respuestas.set("bytes_a_la_red", "0 B");
  respuestas.set("lo_que_salio_al_api", []);
  respuestas.set("la_puerta", VISTA_DE_LA_PUERTA_CERRADA);
});

describe("la puerta local en IA", () => {
  it("se entra por «Puerta local · cerrada» y se vuelve a «Quién redacta»", async () => {
    await pinta("");
    fireEvent.click(
      screen.getByRole("button", { name: `${t.puertaLocal} · ${t.cerrada}` }),
    );
    expect(screen.getByRole("heading", { name: t.titulo })).toBeInTheDocument();
    expect(screen.getByText(t.vacio)).toBeInTheDocument();
    fireEvent.click(
      screen.getByRole("button", { name: es.cuaderno.volverQuienRedacta }),
    );
    expect(screen.getByText(es.cuaderno.quienRedacta)).toBeInTheDocument();
  });

  it("cerrada, el conmutador la abre; abierta, enseña el comando y lo copia", async () => {
    const writeText = vi.fn(() => Promise.resolve());
    Object.defineProperty(navigator, "clipboard", {
      value: { writeText },
      configurable: true,
    });
    respuestas.set("abrir_la_puerta", VISTA_DE_LA_PUERTA_ABIERTA);
    await pinta();
    const conmutador = screen.getByRole("switch", { name: t.titulo });
    expect(conmutador).toHaveAttribute("aria-checked", "false");
    await act(async () => {
      fireEvent.click(conmutador);
    });
    expect(pedidos("abrir_la_puerta")).toHaveLength(1);
    expect(pedidos("cerrar_la_puerta")).toHaveLength(0);
    expect(screen.getByRole("switch", { name: t.titulo })).toHaveAttribute(
      "aria-checked",
      "true",
    );
    const comando = comandoDeGhost(VISTA_DE_LA_PUERTA_ABIERTA.ghost ?? "");
    expect(screen.getByText(comando)).toBeInTheDocument();
    expect(screen.getByText(t.daselo)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: t.copiar }));
    expect(writeText).toHaveBeenCalledWith(comando);
  });

  it("abierta, el conmutador la cierra", async () => {
    respuestas.set("la_puerta", VISTA_DE_LA_PUERTA_ABIERTA);
    respuestas.set("cerrar_la_puerta", VISTA_DE_LA_PUERTA_CERRADA);
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("switch", { name: t.titulo }));
    });
    expect(pedidos("cerrar_la_puerta")).toHaveLength(1);
    expect(pedidos("abrir_la_puerta")).toHaveLength(0);
  });

  it("sin ghost compilado, dice cómo compilarlo en vez de un comando que no existe", async () => {
    respuestas.set("la_puerta", VISTA_DE_LA_PUERTA_SIN_GHOST);
    await pinta();
    expect(screen.getByText(t.faltaGhost)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: t.copiar })).toBeNull();
  });

  it("se cerró sola por una reunión: la franja lo dice, y el registro lo denegado con su motivo", async () => {
    respuestas.set("la_puerta", VISTA_DE_LA_PUERTA_EN_REUNION);
    await pinta();
    expect(screen.getByRole("status")).toHaveTextContent(t.seCerroSola);
    const lineas = screen
      .getByText(t.registro)
      .closest(".tarjeta") as HTMLElement;
    expect(
      within(lineas).getByText(t.motivo["en-reunion"]),
    ).toBeInTheDocument();
    expect(within(lineas).getByText(t.denegado)).toBeInTheDocument();
    // la de antes de la reunión, hecha, con su cuenta
    expect(within(lineas).getByText("3")).toBeInTheDocument();
  });

  it("si no se abrió, la franja dice por qué, antes que nada", async () => {
    respuestas.set("la_puerta", {
      ...VISTA_DE_LA_PUERTA_EN_REUNION,
      noAbre: "llavero",
    });
    await pinta();
    const franja = screen.getByRole("status");
    expect(franja).toHaveTextContent(t.noSeAbrio);
    expect(franja).toHaveTextContent(t.noAbre.llavero);
    expect(franja).not.toHaveTextContent(t.seCerroSola);
  });

  it("abierta no enseña la franja de la reunión aunque se hubiera cerrado sola antes", async () => {
    respuestas.set("la_puerta", {
      ...VISTA_DE_LA_PUERTA_ABIERTA,
      cerro: "en-reunion",
    });
    await pinta();
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("el registro enseña lo hecho, lo denegado y lo fallido, cada uno con símbolo y palabra", async () => {
    respuestas.set("la_puerta", VISTA_DE_LA_PUERTA_ABIERTA);
    await pinta();
    const tarjeta = screen
      .getByText(t.registro)
      .closest(".tarjeta") as HTMLElement;
    expect(
      within(tarjeta).getByText("ghost corpus reindexar"),
    ).toBeInTheDocument();
    expect(within(tarjeta).getByText("28")).toBeInTheDocument();
    expect(
      within(tarjeta).getByText(t.motivo["el-api-es-tuyo"]),
    ).toBeInTheDocument();
    expect(within(tarjeta).getByText(t.fallo)).toBeInTheDocument();
  });

  it("cuando Rust avisa que la puerta cambió, la vista se pone al día sola", async () => {
    respuestas.set("la_puerta", VISTA_DE_LA_PUERTA_ABIERTA);
    await pinta();
    expect(screen.getByRole("switch", { name: t.titulo })).toHaveAttribute(
      "aria-checked",
      "true",
    );
    await act(async () => {
      alOirLaPuerta?.(VISTA_DE_LA_PUERTA_EN_REUNION);
    });
    expect(screen.getByRole("switch", { name: t.titulo })).toHaveAttribute(
      "aria-checked",
      "false",
    );
    expect(screen.getByRole("status")).toHaveTextContent(t.seCerroSola);
  });
});

describe("el comando para Claude Code", () => {
  it("va entre comillas simples, también con una comilla dentro de la ruta", () => {
    expect(comandoDeGhost("/Users/ana/mi app/ghost")).toBe(
      "'/Users/ana/mi app/ghost' --help",
    );
    expect(comandoDeGhost("/Users/o'neil/ghost")).toBe(
      "'/Users/o'\\''neil/ghost' --help",
    );
  });
});
