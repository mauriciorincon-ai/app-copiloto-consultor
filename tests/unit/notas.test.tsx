import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Notas } from "@/pantallas/Notas";
import { IdiomaContext } from "@/i18n";
import { es } from "@/i18n/es";
import { llamar, preguntar } from "@/puente";
import {
  LISTA_DE_REUNIONES,
  REUNION_GUARDADA,
  REUNION_GUARDADA_PARA_SIEMPRE,
  VISTA_DEL_CUADERNO,
} from "@/contrato.generado";

/**
 * **NOTAS DENTRO DE TAURI** (C9, sprint 003, fase 1) — lo que la pantalla hace con lo que Rust le da.
 *
 * Se finge el PUENTE, como en `la-ficha-llega-a-la-banda.test.tsx`, y las respuestas salen de
 * `src/contrato.generado.ts`, que escribe Rust con su serde (regla 19): lo que aquí se pinta es lo
 * que la parte nativa emite de verdad. Y al menos un camino cruza la costura de punta a punta:
 * Rust avisa «cuaderno» → la pantalla vuelve a preguntar → pinta lo nuevo.
 *
 * ¿Puede fallar? Sí: sin la pregunta antes de exportar, «exportar pregunta antes» es rojo (bitácora).
 */
const t = es.notas;
const oyentes = new Map<string, ((dato: unknown) => void)[]>();
const respuestas = new Map<string, unknown>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  // Darse de baja suelta SOLO al que se suscribió, como el puente de verdad: la primera versión de este
  // doble borraba el evento entero y el aviso «cuaderno» se quedaba sin oyente al desmontarse otro.
  escuchar: (evento: string, alOir: (dato: unknown) => void) => {
    oyentes.set(evento, [...(oyentes.get(evento) ?? []), alOir]);
    return () => oyentes.set(evento, (oyentes.get(evento) ?? []).filter((o) => o !== alOir));
  },
  preguntar: vi.fn((comando: string) => {
    const r = respuestas.get(comando);
    return r instanceof Error ? Promise.reject(r) : Promise.resolve(r ?? null);
  }),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

const pedidos = (comando: string) =>
  [...vi.mocked(preguntar).mock.calls, ...vi.mocked(llamar).mock.calls].filter(([c]) => c === comando);

async function pinta() {
  const r = render(
    <IdiomaContext.Provider value="es">
      <Notas />
    </IdiomaContext.Provider>,
  );
  await act(async () => {});
  return r;
}

beforeEach(() => {
  cleanup();
  oyentes.clear();
  respuestas.clear();
  vi.mocked(preguntar).mockClear();
  vi.mocked(llamar).mockClear();
  respuestas.set("reuniones_guardadas", LISTA_DE_REUNIONES);
});

describe("Notas dentro de Tauri", () => {
  it("con la reunión escuchando enseña «durante», y lo escrito va a Rust", async () => {
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, escuchando: true });
    await pinta();
    const nota = screen.getByLabelText(t.tuNota) as HTMLTextAreaElement;
    expect(nota.value).toBe(VISTA_DEL_CUADERNO.nota);
    expect(screen.getByText(VISTA_DEL_CUADERNO.fijadas[0].titular)).toBeInTheDocument();
    fireEvent.change(nota, { target: { value: "Piden la cuarta fuente. Y algo más." } });
    expect(pedidos("escribir_nota").at(-1)?.[1]).toEqual({ texto: "Piden la cuarta fuente. Y algo más." });
  });

  it("un acuerdo se anota con ↵ y la pantalla vuelve a preguntar", async () => {
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, escuchando: true });
    await pinta();
    const antes = pedidos("cuaderno_de_la_reunion").length;
    const campo = screen.getByPlaceholderText(t.escribeElAcuerdo);
    fireEvent.change(campo, { target: { value: "Entrega el viernes" } });
    await act(async () => {
      fireEvent.keyDown(campo, { key: "Enter" });
    });
    expect(pedidos("anotar_acuerdo").at(-1)?.[1]).toEqual({ texto: "Entrega el viernes" });
    expect(pedidos("cuaderno_de_la_reunion").length).toBeGreaterThan(antes);
  });

  it("⌃⌥P desde fuera: Rust avisa «cuaderno», la pantalla pregunta y pinta la fijada nueva", async () => {
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, escuchando: true, fijadas: [] });
    await pinta();
    expect(screen.queryByText(VISTA_DEL_CUADERNO.fijadas[0].titular)).toBeNull();
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, escuchando: true });
    await act(async () => {
      for (const o of oyentes.get("cuaderno") ?? []) o(undefined);
    });
    await act(async () => {});
    expect(screen.getByText(VISTA_DEL_CUADERNO.fijadas[0].titular)).toBeInTheDocument();
  });

  it("parada, enseña «al cerrar» con lo que muere contado y el archivo que va a nacer", async () => {
    respuestas.set("cuaderno_de_la_reunion", VISTA_DEL_CUADERNO);
    await pinta();
    expect(screen.getByText(t.seVaAGuardar)).toBeInTheDocument();
    expect(screen.getByText(`${VISTA_DEL_CUADERNO.turnosDelCliente} ${t.turnos}`)).toBeInTheDocument();
    expect(screen.getByText(`${VISTA_DEL_CUADERNO.lecturas} ${t.lecturas}`)).toBeInTheDocument();
    expect(screen.getByText(/→ reunion-2026-09-20-1402\.ghost/)).toBeInTheDocument();
    // La casilla apagada: la fila de tus turnos no está, y la frase lo dice.
    expect(screen.queryByText(t.tusTurnosEnTexto)).toBeNull();
    expect(screen.getByRole("switch", { name: t.conservarMisTurnos })).toHaveAttribute("aria-checked", "false");
  });

  it("si guardar falla, se dice y la nota se queda en pantalla", async () => {
    respuestas.set("cuaderno_de_la_reunion", VISTA_DEL_CUADERNO);
    respuestas.set("guardar_la_reunion", new Error("el Llavero no contestó"));
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: new RegExp(t.guardarCifrado) }));
    });
    expect(screen.getByRole("alert")).toHaveTextContent(t.noSeGuardo);
    expect(screen.getByText(t.seVaAGuardar)).toBeInTheDocument();
  });

  it("sin reunión abierta enseña el archivo, y la retención se elige con un clic", async () => {
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, abierta: false });
    respuestas.set("reuniones_guardadas", { carpeta: null, reuniones: [REUNION_GUARDADA, REUNION_GUARDADA_PARA_SIEMPRE] });
    await pinta();
    expect(screen.getAllByText(REUNION_GUARDADA.archivo).length).toBeGreaterThan(0);
    expect(screen.getAllByText(t.carpetaDeFabrica).length).toBeGreaterThan(0);
    fireEvent.click(screen.getByRole("radio", { name: "7 d" }));
    expect(pedidos("fijar_retencion").at(-1)?.[1]).toEqual({ retencion: "7d" });
    expect(screen.getByRole("radio", { name: /7 d/ })).toHaveAttribute("aria-checked", "true");
  });

  it("exportar pregunta antes, y solo el segundo botón exporta", async () => {
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, abierta: false });
    respuestas.set("reuniones_guardadas", { carpeta: null, reuniones: [REUNION_GUARDADA] });
    respuestas.set("exportar_reunion", true);
    await pinta();
    fireEvent.click(screen.getByRole("button", { name: new RegExp(t.exportarATexto) }));
    expect(pedidos("exportar_reunion")).toHaveLength(0);
    expect(screen.getByRole("alertdialog")).toHaveTextContent(t.exportarPregunta);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: new RegExp(t.exportarSinCifrado) }));
    });
    expect(pedidos("exportar_reunion").at(-1)?.[1]).toEqual({ archivo: REUNION_GUARDADA.archivo, idioma: "es" });
  });

  it("borrar pregunta antes, y «Cancelar» no borra nada", async () => {
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, abierta: false });
    respuestas.set("reuniones_guardadas", { carpeta: null, reuniones: [REUNION_GUARDADA] });
    await pinta();
    fireEvent.click(screen.getByRole("button", { name: new RegExp(t.borrarAhora) }));
    expect(pedidos("borrar_reunion")).toHaveLength(0);
    fireEvent.click(screen.getByRole("button", { name: t.cancelar }));
    expect(screen.queryByRole("alertdialog")).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: new RegExp(t.borrarAhora) }));
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: new RegExp(`^${t.borrar}$`) }));
    });
    expect(pedidos("borrar_reunion").at(-1)?.[1]).toEqual({ archivo: REUNION_GUARDADA.archivo });
  });

  it("mientras Rust no contesta no se pinta ninguna vista: ni un instante de «el archivo» antes de «durante»", async () => {
    respuestas.set("cuaderno_de_la_reunion", new Promise(() => {}));
    vi.mocked(preguntar).mockImplementationOnce(() => new Promise(() => {}));
    await pinta();
    expect(screen.queryByText(t.reunionesGuardadas)).toBeNull();
    expect(screen.queryByText(t.sinReuniones)).toBeNull();
    expect(screen.queryByLabelText(t.tuNota)).toBeNull();
  });

  it("sin ninguna reunión guardada lo dice, y dice dónde vivirán", async () => {
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, abierta: false });
    await pinta();
    expect(screen.getByText(t.sinReuniones)).toBeInTheDocument();
    expect(screen.getByText(t.carpetaDeFabrica)).toBeInTheDocument();
  });
});
