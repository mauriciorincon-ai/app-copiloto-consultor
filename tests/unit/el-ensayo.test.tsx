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
import { en } from "@/i18n/en";
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
  ENSAYO_REPETIDAS,
  ENSAYO_SIN_SECCIONES,
  NO_EMPEZO_EN_REUNION,
  NO_EMPEZO_MICROFONO,
  NO_EMPEZO_NO_SE_SABE_SI_HAY_LLAMADA,
  NO_EMPEZO_VIDEOLLAMADA,
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

describe("auditoría del S4: Honestidad sabe que hay un ensayo", () => {
  /**
   * B28: con todo saltado no hay bytes ni micrófono, y la fila seguía siendo de un ensayo en memoria; la
   * pantalla lo adivinaba y la perdía. Lo dice Rust. B27: el contador es el del ensayo.
   */
  it("con el micrófono cerrado y 0 B, la fila del ensayo sigue y el contador es del ensayo", async () => {
    document.documentElement.lang = "es";
    const saltado = {
      ...ESTADO_DE_LA_ESCUCHA_EN_UN_ENSAYO,
      microfono: { abierta: false, motivo: null, bytes: 0 },
      bytesDelEnsayo: 0,
    };
    render(
      <Cascara>
        <SpriteIconos />
        <Honestidad bytes="0 B" escucha={saltado} />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.getByText(es.cuaderno.bufEnsayo).closest(".buffer")).toHaveTextContent("0 B");
    expect(screen.getByText(es.cuaderno.salieronEnEsteEnsayo)).toBeInTheDocument();
    expect(screen.queryByText(es.cuaderno.salieronDeTuEquipo)).toBeNull();
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

// ─── auditoría del S4 (Fase 2): lo que la pantalla del ensayo pagó ───────────────────────────────────

describe("auditoría del S4: las teclas", () => {
  /** M11: una tecla sostenida repite; una orden por pulsación, no por repetición. */
  it("Enter o S sostenidos mandan una sola orden", async () => {
    respuestas.set("estado_del_ensayo", { ...ENSAYO_PREGUNTANDO, fase: "respondiendo", leyendo: false });
    await pinta();
    for (const [tecla, comando] of [["Enter", "ensayo_listo"], ["s", "ensayo_saltar"]] as const) {
      fireEvent.keyDown(globalThis.window, { key: tecla });
      fireEvent.keyDown(globalThis.window, { key: tecla, repeat: true });
      fireEvent.keyDown(globalThis.window, { key: tecla, repeat: true });
      expect(pedidos(comando), tecla).toHaveLength(1);
    }
  });

  /** M16: Enter sobre un enlace del rail navega; no cierra tu respuesta. */
  it("Enter con el foco en un enlace no cierra la respuesta", async () => {
    respuestas.set("estado_del_ensayo", { ...ENSAYO_PREGUNTANDO, fase: "respondiendo", leyendo: false });
    await pinta();
    const enlace = document.createElement("a");
    enlace.href = "#notas";
    enlace.textContent = "Notas";
    document.body.appendChild(enlace);
    enlace.focus();
    fireEvent.keyDown(enlace, { key: "Enter" });
    expect(pedidos("ensayo_listo")).toHaveLength(0);
    enlace.remove();
  });
});

describe("auditoría del S4: preparar y empezar", () => {
  /** M13: lo elegido sobrevive a ver tu progreso. */
  it("volver del progreso conserva lo elegido", async () => {
    respuestas.set("preparar_el_ensayo", { ...PREPARACION_DEL_ENSAYO, cliente: "Sur del Valle", guardados: 4 });
    respuestas.set("progreso_del_ensayo", { ...PROGRESO_DEL_ENSAYO, cliente: "Sur del Valle" });
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("radio", { name: "12" }));
    });
    fireEvent.click(screen.getByRole("switch", { name: t.leerEnVozAlta }));
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.verTuProgreso }));
    });
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.volver }));
    });
    expect(pedidos("preparar_el_ensayo").at(-1)?.[1]).toMatchObject({ cliente: "Sur del Valle", tope: 12 });
    expect(screen.getByRole("switch", { name: t.leerEnVozAlta })).toHaveAttribute("aria-checked", "false");
  });

  /** A1: con una videollamada abierta y altavoces, el ensayo no empieza y dice por qué; B32: se anuncia. */
  it("con una videollamada abierta dice por qué no empieza, y se anuncia", async () => {
    respuestas.set("preparar_el_ensayo", PREPARACION_DEL_ENSAYO);
    rechazos.set("empezar_el_ensayo", NO_EMPEZO_VIDEOLLAMADA);
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.empezar }));
    });
    expect(screen.getByRole("alert")).toHaveTextContent(t.hayVideollamada);
    expect(screen.getByText(t.hayVideollamadaQue)).toBeInTheDocument();
    rechazos.set("empezar_el_ensayo", NO_EMPEZO_NO_SE_SABE_SI_HAY_LLAMADA);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.empezar }));
    });
    expect(screen.getByRole("alert")).toHaveTextContent(t.noSeSabeSiHayLlamada);
    rechazos.set("empezar_el_ensayo", NO_EMPEZO_EN_REUNION);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.empezar }));
    });
    expect(screen.getByRole("alert")).toHaveTextContent(t.hayReunion);
  });

  /** B26: si «sin corpus» lo dice `empezar` y no «preparar», la pantalla también lo enseña. */
  it("si empezar dice que no hay corpus, lo enseña", async () => {
    respuestas.set("preparar_el_ensayo", PREPARACION_DEL_ENSAYO);
    rechazos.set("empezar_el_ensayo", { que: "sin-corpus" });
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.empezar }));
    });
    expect(screen.getByText(t.nadaDeEsteCliente)).toBeInTheDocument();
  });

  /** B39: un cliente con solo su ficha no tiene propuesta que elegir. */
  it("sin propuestas no hay selector de propuesta", async () => {
    respuestas.set("preparar_el_ensayo", { ...PREPARACION_DEL_ENSAYO, propuestas: [], propuesta: null });
    await pinta();
    expect(screen.queryByRole("combobox", { name: t.propuesta })).toBeNull();
    expect(screen.getByRole("combobox", { name: t.cliente })).toBeInTheDocument();
  });
});

describe("auditoría del S4: la evaluada, el informe y el modelo", () => {
  /** M12 y B33: las cuentas de Rust; la marca de sección conjeturada. */
  it("la evaluada pinta lo que cuenta Rust y la sección conjeturada", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_EVALUADA);
    await pinta();
    expect(screen.getByText(`${ENSAYO_EVALUADA.usadas} ${t.de} 3`)).toBeInTheDocument();
    expect(screen.getByText(es.banda.seccionConjeturada)).toBeInTheDocument();
    cleanup();
    // Sin palabras transcritas, «—» y no un cero inventado.
    respuestas.set("estado_del_ensayo", { ...ENSAYO_EVALUADA, muletillas: null });
    await pinta();
    const cifra = screen.getByText(t.muletillas, { selector: ".q" }).closest(".cifra-e") as HTMLElement;
    expect(cifra.querySelector(".n")).toHaveTextContent("—");
  });

  it("un informe sin palabras dice «—» en las muletillas", async () => {
    const informe = ENSAYO_CERRADO.informe;
    if (!informe) throw new Error("la muestra del informe");
    respuestas.set("estado_del_ensayo", { ...ENSAYO_CERRADO, informe: { ...informe, muletillas: null, laQueMas: null } });
    await pinta();
    const cifra = screen.getByText(t.muletillas, { selector: ".q" }).closest(".cifra-e") as HTMLElement;
    expect(cifra.querySelector(".n")).toHaveTextContent("—");
  });

  /** B30: sin la respuesta del cuaderno no se sabe tu retención, y no se inventa. */
  it("sin saber tu retención, no la dice", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_CERRADO);
    await pinta();
    expect(screen.queryByText(new RegExp(t.seCifraAntes))).toBeNull();
  });

  /** B24: la tabla del informe dice «wpm» en inglés. */
  it("en inglés, la columna del ritmo dice wpm", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_CERRADO);
    await pinta();
    document.documentElement.lang = "en";
    cleanup();
    render(
      <Cascara>
        <SpriteIconos />
        <Ensayo ir={vi.fn()} />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.getByRole("columnheader", { name: en.ensayo.ppm })).toBeInTheDocument();
    expect(screen.queryByRole("columnheader", { name: "ppm" })).toBeNull();
    document.documentElement.lang = "es";
  });

  /** B22: «el modelo sumó 0» no es lo que pasó. */
  it("el modelo dice por qué no sumó: repetidas, o sin secciones", async () => {
    respuestas.set("estado_del_ensayo", ENSAYO_REPETIDAS);
    await pinta();
    expect(screen.getByText(`${t.noSumo} ${t.porQueNoSumo.repetidas}`)).toBeInTheDocument();
    cleanup();
    respuestas.set("estado_del_ensayo", ENSAYO_SIN_SECCIONES);
    await pinta();
    expect(screen.getByText(`${t.noSumo} ${t.porQueNoSumo["sin-secciones"]}`)).toBeInTheDocument();
  });
});

describe("auditoría del S4: borrar tus ensayos", () => {
  async function abreYPregunta() {
    respuestas.set("preparar_el_ensayo", { ...PREPARACION_DEL_ENSAYO, guardados: 4 });
    respuestas.set("progreso_del_ensayo", PROGRESO_DEL_ENSAYO);
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.verTuProgreso }));
    });
    fireEvent.click(screen.getByRole("button", { name: t.borrarLosDeEsteCliente }));
  }

  /** M15: el foco va a «Cancelar» al abrir, y vuelve al botón que la abrió al cerrar. */
  it("la pregunta recibe el foco, y al cancelar vuelve a su botón", async () => {
    await abreYPregunta();
    expect(screen.getByRole("button", { name: t.cancelar })).toHaveFocus();
    fireEvent.click(screen.getByRole("button", { name: t.cancelar }));
    expect(screen.getByRole("button", { name: t.borrarLosDeEsteCliente })).toHaveFocus();
  });

  /** M14: si borrar falla, la pantalla lo dice. */
  it("si borrar falla, lo dice y siguen en su sitio", async () => {
    rechazos.set("borrar_los_ensayos", "no se pudo");
    await abreYPregunta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.borrar }));
    });
    expect(screen.getByRole("alert")).toHaveTextContent(t.noSeBorraron);
    expect(screen.getByText(t.tuProgreso)).toBeInTheDocument();
  });

  /** B31: dos ensayos en el mismo minuto no repiten la `key` de React. */
  it("dos ensayos del mismo minuto no se pisan en la tabla", async () => {
    const quejas = vi.spyOn(console, "error").mockImplementation(() => {});
    const fila = PROGRESO_DEL_ENSAYO.filas[0];
    respuestas.set("preparar_el_ensayo", { ...PREPARACION_DEL_ENSAYO, guardados: 2 });
    respuestas.set("progreso_del_ensayo", { ...PROGRESO_DEL_ENSAYO, filas: [fila, fila] });
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.verTuProgreso }));
    });
    expect(quejas.mock.calls.filter(([m]) => String(m).includes("same key"))).toEqual([]);
    quejas.mockRestore();
  });
});

describe("auditoría del S4: los estilos son del sistema", () => {
  /** B14: ni un tamaño de letra en línea en la pantalla del ensayo; lo dice `.ayuda-e`. */
  it("Ensayo no escribe tamaños de letra a mano", async () => {
    const { readFileSync } = await import("node:fs");
    const fuente = readFileSync("src/pantallas/Ensayo.tsx", "utf8");
    expect(fuente.match(/fontSize\s*:/g) ?? []).toEqual([]);
  });
});
