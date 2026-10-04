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
import { Ensayo } from "@/pantallas/Ensayo";
import { Honestidad } from "@/pantallas/Honestidad";
import { es } from "@/i18n/es";
import { escuchar, llamar, preguntar } from "@/puente";
import { diaCorto, entreComillas, flecha, reloj } from "@/ensayo";
import {
  ENSAYO_CERRADO,
  ESTADO_DE_LA_ESCUCHA_EN_UN_ENSAYO,
  PROGRESO_CON_MAS,
  PROGRESO_DE_UN_ENSAYO,
  PROGRESO_DEL_ENSAYO,
  VISTA_DEL_CUADERNO,
  ENSAYO_DEL_MODELO,
  ENSAYO_EVALUADA,
  ENSAYO_OBJECION,
  ENSAYO_PREGUNTANDO,
  NO_EMPEZO_EN_REUNION,
  NO_EMPEZO_MICROFONO,
  PREPARACION_DEL_ENSAYO,
  PREPARACION_SIN_CORPUS,
} from "@/contrato.generado";

/**
 * **EL ENSAYO, DENTRO DE TAURI** (C18, sprint 004, fase 3, ADR 019) — la pantalla con lo que Rust le da.
 *
 * Se finge el PUENTE, como en `la-banda-arriba.test.tsx`, y las respuestas salen de
 * `src/contrato.generado.ts`, que escribe Rust con su serde (regla 19). Lo que se prueba es el cable:
 * que «Empezar» manda lo elegido, que las teclas de la ventana llegan a Rust, que «Sí lo dije» dice cuál,
 * y que un ensayo que no empezó dice por qué.
 *
 * ¿Puede fallar? Sí: con Enter sin llamar a `ensayo_listo`, o con el porqué de la reunión pintado como
 * el del micrófono, es rojo (bitácora, fase 3).
 */
const t = es.ensayo;
const respuestas = new Map<string, unknown>();
const rechazos = new Map<string, unknown>();
const oyentes = new Map<string, (dato: unknown) => void>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: vi.fn((evento: string, alOir: (dato: unknown) => void) => {
    oyentes.set(evento, alOir);
    return () => {};
  }),
  preguntar: vi.fn((comando: string) =>
    rechazos.has(comando)
      ? Promise.reject(rechazos.get(comando))
      : Promise.resolve(respuestas.get(comando) ?? null),
  ),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

const pedidos = (comando: string) =>
  [...vi.mocked(preguntar).mock.calls, ...vi.mocked(llamar).mock.calls].filter(
    ([c]) => c === comando,
  );

async function pinta() {
  document.documentElement.lang = "es";
  const ir = vi.fn();
  const r = render(
    <Cascara>
      <SpriteIconos />
      <Ensayo ir={ir} />
    </Cascara>,
  );
  await act(async () => {});
  return { ...r, ir };
}

beforeEach(() => {
  cleanup();
  respuestas.clear();
  rechazos.clear();
  oyentes.clear();
  vi.mocked(preguntar).mockClear();
  vi.mocked(llamar).mockClear();
  vi.mocked(escuchar).mockClear();
});

describe("preparar", () => {
  it("enseña de dónde salen las preguntas y empieza con lo elegido", async () => {
    respuestas.set("preparar_el_ensayo", PREPARACION_DEL_ENSAYO);
    await pinta();
    expect(screen.getByText(t.tuPropuesta).parentElement).toHaveTextContent(
      "5",
    );
    expect(screen.getByText(t.idiomaDelEnsayo.es)).toBeInTheDocument();
    // La voz, apagada a mano antes de empezar.
    const voz = screen.getByRole("switch", { name: t.leerEnVozAlta });
    expect(voz).toHaveAttribute("aria-checked", "true");
    fireEvent.click(voz);
    expect(voz).toHaveAttribute("aria-checked", "false");
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.empezar }));
    });
    const [[, args]] = pedidos("empezar_el_ensayo");
    expect(args).toEqual({
      cliente: "Páramo Azul",
      propuesta: PREPARACION_DEL_ENSAYO.propuesta,
      tope: 8,
      voz: false,
    });
  });

  it("cambiar el tope vuelve a preguntar a Rust con el nuevo", async () => {
    respuestas.set("preparar_el_ensayo", PREPARACION_DEL_ENSAYO);
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("radio", { name: "12" }));
    });
    expect(pedidos("preparar_el_ensayo").at(-1)?.[1]).toMatchObject({
      tope: 12,
    });
  });

  it("si no empieza, dice por qué: la reunión no es el micrófono", async () => {
    respuestas.set("preparar_el_ensayo", PREPARACION_DEL_ENSAYO);
    rechazos.set("empezar_el_ensayo", NO_EMPEZO_EN_REUNION);
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.empezar }));
    });
    expect(screen.getByText(t.hayReunion)).toBeInTheDocument();
    expect(screen.queryByText(t.microfonoNoAbrio)).toBeNull();

    rechazos.set("empezar_el_ensayo", NO_EMPEZO_MICROFONO);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.empezar }));
    });
    expect(screen.getByText(t.microfonoSinPermiso)).toBeInTheDocument();
  });

  it("si este Mac no transcribe el idioma de la propuesta, lo dice antes de empezar", async () => {
    respuestas.set("preparar_el_ensayo", { ...PREPARACION_DEL_ENSAYO, transcribe: false });
    await pinta();
    expect(screen.getByText(t.sinTranscripcion)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: t.empezar })).toBeEnabled();
    cleanup();
    respuestas.set("preparar_el_ensayo", PREPARACION_DEL_ENSAYO);
    await pinta();
    expect(screen.queryByText(t.sinTranscripcion)).toBeNull();
  });

  it("sin corpus de ese cliente no inventa preguntas: lleva a Corpus", async () => {
    respuestas.set("preparar_el_ensayo", PREPARACION_SIN_CORPUS);
    const { ir } = await pinta();
    expect(screen.getByText(t.nadaDeEsteCliente)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: t.empezar })).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: t.irACorpus }));
    expect(ir).toHaveBeenCalledWith("corpus");
  });
});

describe("ensayando: las teclas son de esta ventana", () => {
  it("Enter, R, S y Esc llegan a Rust; con ⌘, no", async () => {
    respuestas.set("estado_del_ensayo", {
      ...ENSAYO_PREGUNTANDO,
      fase: "respondiendo",
      leyendo: false,
    });
    await pinta();
    expect(screen.getByText(t.teEscucho)).toBeInTheDocument();
    for (const [tecla, comando] of [
      ["Enter", "ensayo_listo"],
      ["r", "ensayo_repetir"],
      ["S", "ensayo_saltar"],
      ["Escape", "ensayo_terminar"],
    ] as const) {
      fireEvent.keyDown(globalThis.window, { key: tecla });
      expect(pedidos(comando), tecla).toHaveLength(1);
    }
    // Con ⌘ no es nuestra tecla.
    fireEvent.keyDown(globalThis.window, { key: "r", metaKey: true });
    expect(pedidos("ensayo_repetir")).toHaveLength(1);
  });

  it("la pregunta dice de dónde sale: una objeción lleva su fuente, la del modelo su marca", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_OBJECION);
    await pinta();
    expect(screen.getByText(t.chips.objeciones)).toBeInTheDocument();
    expect(screen.getByText("Kuznetsova")).toBeInTheDocument();
    expect(
      screen.getByText(`${t.noSumo} ${t.porQueNoSumo["nada-fundado"]}`),
    ).toBeInTheDocument();
    cleanup();
    respuestas.set("estado_del_ensayo", ENSAYO_DEL_MODELO);
    await pinta();
    expect(screen.getByText(t.chips.modelo)).toBeInTheDocument();
    expect(
      screen.getByText(`${t.sumoAntes} 2 ${t.sumoDespues}`),
    ).toBeInTheDocument();
  });

  it("al leer la pregunta, el micrófono espera y lo dice", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_PREGUNTANDO);
    await pinta();
    expect(screen.getByText(t.teLaEstaLeyendo)).toBeInTheDocument();
    expect(screen.getByText(t.microfonoEspera)).toBeInTheDocument();
  });

  it("vuelve a preguntar con cada señal `ensayo`: lo que dijiste no viaja en el evento", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_PREGUNTANDO);
    await pinta();
    const antes = pedidos("estado_del_ensayo").length;
    await act(async () => oyentes.get("ensayo")?.(null));
    expect(pedidos("estado_del_ensayo").length).toBe(antes + 1);
  });
});

describe("la evaluada y el informe", () => {
  it("sin puntaje: lo que usaste, lo que tenías, y «Sí lo dije» dice cuál", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_EVALUADA);
    await pinta();
    expect(screen.getByText(t.usaste)).toBeInTheDocument();
    expect(screen.getByText(t.teniasYNo)).toBeInTheDocument();
    // La tercera ficha la marcó el usuario: va con las usadas, con su botón pulsado para deshacerlo.
    const pulsado = screen.getByRole("button", {
      name: t.siLoDije,
      pressed: true,
    });
    fireEvent.click(pulsado);
    expect(pedidos("ensayo_si_lo_dije").at(-1)?.[1]).toEqual({ indice: 2 });
    expect(screen.getByText(`3 ${t.de} 3`)).toBeInTheDocument();
    expect(screen.getByText(`142 ${t.ppm}`)).toBeInTheDocument();
    expect(
      screen.getByText("«o sea» ×3 · «básicamente» ×1"),
    ).toBeInTheDocument();
    // Enter en la evaluada pasa a la siguiente; R y S no hacen nada.
    fireEvent.keyDown(globalThis.window, { key: "r" });
    expect(pedidos("ensayo_repetir")).toHaveLength(0);
    fireEvent.click(screen.getByRole("button", { name: t.siguiente }));
    expect(pedidos("ensayo_listo")).toHaveLength(1);
  });

  it("el informe cuenta, enseña cuatro filas y cierra sin guardar", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_CERRADO);
    await pinta();
    expect(
      screen.getByText(`${t.terminado} 7 ${t.respondidas}, 1 ${t.saltada}`),
    ).toBeInTheDocument();
    expect(screen.getByText(`14 ${t.de} 21`)).toBeInTheDocument();
    expect(screen.getByText(`${t.laQueMas} «o sea» ×5`)).toBeInTheDocument();
    // Las teclas ya no son del ensayo: terminó.
    fireEvent.keyDown(globalThis.window, { key: "Enter" });
    expect(pedidos("ensayo_listo")).toHaveLength(0);
    fireEvent.click(screen.getByRole("button", { name: t.cerrarSinGuardar }));
    expect(pedidos("cerrar_el_ensayo")).toHaveLength(1);
  });
});

// ─── fase 4: lo que queda (ADR 015, enmienda 4) ─────────────────────────────────────────────────────

describe("el informe se guarda, se exporta y dice su retención", () => {
  it("«Guardar con tus notas» guarda y vuelve a «preparar» diciéndolo", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_CERRADO);
    respuestas.set("preparar_el_ensayo", { ...PREPARACION_DEL_ENSAYO, guardados: 5 });
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.guardar }));
    });
    expect(pedidos("guardar_el_ensayo")).toHaveLength(1);
    // Rust soltó el informe y avisa: ya no hay ensayo, y la pantalla vuelve a «preparar».
    respuestas.set("estado_del_ensayo", null);
    await act(async () => oyentes.get("ensayo")?.(null));
    expect(screen.getByText(t.guardadoConTusNotas)).toBeInTheDocument();
    expect(screen.getByText(t.guardados).parentElement).toHaveTextContent("5");
  });

  it("si guardar falla, el informe sigue entero y lo dice", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_CERRADO);
    rechazos.set("guardar_el_ensayo", "el Llavero no contestó");
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.guardar }));
    });
    expect(screen.getByText(t.noSeGuardo)).toBeInTheDocument();
    expect(screen.getByText(t.noSeGuardoQue)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: t.guardar })).toBeEnabled();
  });

  it("«Exportar como texto» pide el idioma de la interfaz; si falla, lo dice sin tocar el informe", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_CERRADO);
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.exportar }));
    });
    expect(pedidos("exportar_el_ensayo").at(-1)?.[1]).toEqual({ idioma: "es" });
    expect(screen.queryByText(t.noSeExporto)).toBeNull();
    rechazos.set("exportar_el_ensayo", "cancelado");
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.exportar }));
    });
    expect(screen.getByText(t.noSeExporto)).toBeInTheDocument();
    expect(pedidos("guardar_el_ensayo")).toHaveLength(0);
  });

  it("la línea de la retención es la de tus notas, y «siempre» no dice «se borra»", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_CERRADO);
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, retencion: "30d" });
    await pinta();
    expect(screen.getByText(`${t.seCifraAntes} 30 d${t.seCifraDespues}`)).toBeInTheDocument();
    cleanup();
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, retencion: "siempre" });
    await pinta();
    expect(screen.getByText(t.seQueda)).toBeInTheDocument();
  });
});

describe("tu progreso con un cliente", () => {
  async function abre(progreso: unknown) {
    respuestas.set("preparar_el_ensayo", { ...PREPARACION_DEL_ENSAYO, guardados: 4 });
    respuestas.set("progreso_del_ensayo", progreso);
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.verTuProgreso }));
    });
  }

  it("sin ensayos guardados no hay camino al progreso", async () => {
    respuestas.set("preparar_el_ensayo", { ...PREPARACION_DEL_ENSAYO, guardados: 0 });
    await pinta();
    expect(screen.queryByRole("button", { name: t.verTuProgreso })).toBeNull();
  });

  it("pide el progreso de ese cliente y enseña las cifras con su flecha, sin puntaje", async () => {
    await abre(PROGRESO_DEL_ENSAYO);
    expect(pedidos("progreso_del_ensayo").at(-1)?.[1]).toEqual({ cliente: "Páramo Azul", idioma: "es" });
    expect(screen.getByText(t.tuProgreso)).toBeInTheDocument();
    expect(screen.getByText("21 sep")).toBeInTheDocument();
    expect(screen.getByText(`9 ${t.de} 21`)).toBeInTheDocument();
    expect(screen.getByText(`↑ ${t.chipEvidencia} 9 → 14`)).toBeInTheDocument();
    expect(screen.getByText(`↓ ${t.chipRitmo} 161 → 138 ${t.ppm}`)).toBeInTheDocument();
    expect(screen.getByText(`↓ ${t.chipMuletillas} 17 → 9`)).toBeInTheDocument();
    expect(screen.getByText(`↓ ${t.chipTiempo} 1:21 → 0:58`)).toBeInTheDocument();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.volver }));
    });
    expect(screen.getByRole("button", { name: t.empezar })).toBeInTheDocument();
  });

  it("con un solo ensayo no hay «desde el primero»; con más de los que caben, los cuenta", async () => {
    await abre(PROGRESO_DE_UN_ENSAYO);
    expect(screen.queryByText(t.desdeElPrimero)).toBeNull();
    cleanup();
    await abre(PROGRESO_CON_MAS);
    expect(screen.getByText(`${t.yMas} 3 ${t.mas}`)).toBeInTheDocument();
    // El primero no tenía ritmo ni muletillas: «—», y la flecha sale del primero que sí.
    expect(screen.getAllByText("—").length).toBeGreaterThanOrEqual(2);
    expect(screen.getByText(`↓ ${t.chipRitmo} 150 → 138 ${t.ppm}`)).toBeInTheDocument();
  });

  it("si el desbloqueo no se da, tus ensayos siguen cifrados y la pantalla lo dice", async () => {
    respuestas.set("preparar_el_ensayo", { ...PREPARACION_DEL_ENSAYO, guardados: 4 });
    rechazos.set("progreso_del_ensayo", "cancelado");
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.verTuProgreso }));
    });
    expect(screen.getByText(t.noSeAbrieron)).toBeInTheDocument();
    expect(screen.queryByText(t.tuProgreso)).toBeNull();
  });

  it("borrar los de este cliente pregunta antes y manda cuál", async () => {
    await abre(PROGRESO_DEL_ENSAYO);
    fireEvent.click(screen.getByRole("button", { name: t.borrarLosDeEsteCliente }));
    expect(screen.getByText(t.borrarPregunta)).toBeInTheDocument();
    expect(pedidos("borrar_los_ensayos")).toHaveLength(0);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.borrar }));
    });
    expect(pedidos("borrar_los_ensayos").at(-1)?.[1]).toEqual({ cliente: "Páramo Azul" });
    expect(screen.queryByText(t.tuProgreso)).toBeNull();
  });
});

describe("Honestidad mientras ensayas", () => {
  it("el micrófono es el del ensayo y tus respuestas tienen su fila, contadas en la RAM", async () => {
    document.documentElement.lang = "es";
    render(
      <Cascara>
        <SpriteIconos />
        <Honestidad bytes="0 B" escucha={ESTADO_DE_LA_ESCUCHA_EN_UN_ENSAYO} />
      </Cascara>,
    );
    await act(async () => {});
    const tc = es.cuaderno;
    const fila = screen.getByText(tc.bufEnsayo).closest(".buffer");
    expect(fila).toHaveTextContent("412 B");
    expect(screen.getByText(tc.bufMic).closest(".buffer")).toHaveTextContent("1,8 MB");
    expect(screen.getByText(tc.bufSistema).closest(".buffer")).toHaveTextContent("0 B");
    expect(screen.getByText(/^RAM ·/)).toHaveTextContent("RAM · 1,8 MB");
    expect(screen.getByText(es.notas.tuyoDetalle, { exact: false })).toBeInTheDocument();
  });

  /** Terminado, el micrófono ya no ocupa nada y tus respuestas siguen en memoria hasta que guardes o
   *  cierres: la fila sigue, y la RAM es exactamente lo suyo (con el anillo de 1,8 MB al lado, 412 B se
   *  perdían en el redondeo y quitarlos de la suma no se notaba). */
  it("terminado, la RAM es exactamente tus respuestas", async () => {
    document.documentElement.lang = "es";
    const terminado = {
      ...ESTADO_DE_LA_ESCUCHA_EN_UN_ENSAYO,
      microfono: { abierta: false, motivo: null, bytes: 0 },
    };
    render(
      <Cascara>
        <SpriteIconos />
        <Honestidad bytes="0 B" escucha={terminado} />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.getByText(es.cuaderno.bufEnsayo).closest(".buffer")).toHaveTextContent("412 B");
    expect(screen.getByText(/^RAM ·/)).toHaveTextContent("RAM · 412 B");
  });
});

describe("los formatos", () => {
  it("el día del progreso y la flecha", () => {
    expect(diaCorto("2026-09-21 10:05", es.ensayo.meses, "es")).toBe("21 sep");
    expect(diaCorto("2026-10-02 08:45", ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"], "en")).toBe("Oct 02");
    expect(diaCorto("ayer", es.ensayo.meses, "es")).toBe("ayer");
    expect([flecha({ desde: 9, hasta: 14 }), flecha({ desde: 161, hasta: 138 }), flecha({ desde: 9, hasta: 9 })]).toEqual(["↑", "↓", "="]);
  });

  it("el reloj y las comillas", () => {
    expect([reloj(0), reloj(42_000), reloj(72_400), reloj(-5)]).toEqual([
      "0:00",
      "0:42",
      "1:12",
      "0:00",
    ]);
    expect(entreComillas("o sea", "es")).toBe("«o sea»");
    expect(entreComillas("I mean", "en")).toBe("“I mean”");
  });
});
