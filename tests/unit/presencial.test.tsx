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
import { TEXTOS_DE_PRESENCIAL } from "@/i18n/presencial-nuevo";
import { Sesion } from "@/pantallas/Sesion";
import { Honestidad } from "@/pantallas/Honestidad";
import { es } from "@/i18n/es";
import { llamar, preguntar } from "@/puente";
import {
  NOVEDAD_APARECE_FICHA,
  TURNO_DE_LA_SALA,
  ENTRADA_DEL_EQUIPO,
  ENTRADA_EXTERNA,
  EN_TU_MAC_LIMPIO,
  EN_TU_MAC_VIGILADO,
  ESTADO_DE_LA_ESCUCHA,
  ESTADO_DE_LA_ESCUCHA_PRESENCIAL,
  LA_VOZ_APAGADA,
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
const oyentes = new Map<string, ((dato: unknown) => void)[]>();
let enTauri = true;

vi.mock("@/puente", () => ({
  hayTauri: () => enTauri,
  // Los avisos que Rust manda (el radar, por ejemplo): el test los emite con `emitir`.
  escuchar: (evento: string, alOir: (dato: unknown) => void) => {
    oyentes.set(evento, [...(oyentes.get(evento) ?? []), alOir]);
    return () => oyentes.set(evento, (oyentes.get(evento) ?? []).filter((o) => o !== alOir));
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

const antesDeEmpezar: EstadoDeEscucha = {
  ...ESTADO_DE_LA_ESCUCHA,
  escuchando: false,
};
const AIRPODS: Salida = { salida: "otra", nombre: "AirPods Pro" };
/** Antes de encender nada, con voz para el idioma y una salida que no suena en la sala: la voz PUEDE hablar. */
const PUEDE_HABLAR = { ...LA_VOZ_APAGADA, puede: true };

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

async function emitir(evento: string, dato: unknown) {
  await act(async () => {
    for (const o of oyentes.get(evento) ?? []) o(dato);
  });
}

beforeEach(() => {
  cleanup();
  enTauri = true;
  oyentes.clear();
  respuestas.clear();
  vi.mocked(preguntar).mockClear();
  vi.mocked(llamar).mockClear();
  respuestas.set("este_cliente", VISTA_DEL_CLIENTE);
  respuestas.set("estado_de_la_voz", PUEDE_HABLAR);
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
    // La voz se pregunta a Rust al montarse la pantalla del modo: «Al oído» parte elegido cuando contesta que PUEDE.
    await act(async () => {});
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

  it("sin voz que pueda hablar, la banda viene marcada aunque la salida sean auriculares (M5 del S6)", async () => {
    // En Windows, sin voz del sistema para el idioma, «Al oído» partía elegido con la voz muda.
    respuestas.set("estado_de_la_voz", LA_VOZ_APAGADA);
    await pintaSesion(antesDeEmpezar, { salida: "auriculares" });
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    expect(radio(tp.laBanda)).toHaveAttribute("aria-checked", "true");
    expect(radio(tp.alOido)).toHaveAttribute("aria-checked", "false");
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

describe("el radar coral en presencial (auditoría del S5, M4)", () => {
  it("«Iniciar de todos modos» con presencial elegido NO empieza una reunión: vuelve a la sala", async () => {
    respuestas.set("radar_de_tu_mac", EN_TU_MAC_LIMPIO);
    await pintaSesion();
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    // Aparece un programa que vigila mientras ya elegiste presencial: el aviso ocupa la pantalla.
    await emitir("radar", EN_TU_MAC_VIGILADO);
    const seguir = screen.getByRole("button", { name: es.cuaderno.iniciarDeTodosModos });
    await act(async () => {
      fireEvent.click(seguir);
    });
    expect(pedidos("empezar_a_escuchar"), "abrió el audio del sistema y la pantalla aunque elegiste la sala").toHaveLength(0);
    expect(screen.getByRole("button", { name: tp.empezar }), "no volvió a la vista de la sala").toBeInTheDocument();
  });

  it("el control: en la reunión, «Iniciar de todos modos» sí empieza", async () => {
    respuestas.set("radar_de_tu_mac", EN_TU_MAC_LIMPIO);
    await pintaSesion();
    await emitir("radar", EN_TU_MAC_VIGILADO);
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: es.cuaderno.iniciarDeTodosModos }));
    });
    expect(pedidos("empezar_a_escuchar").length).toBeGreaterThan(0);
  });

  it("con la sala en marcha, el aviso y la fila del MDM salen arriba, como en la reunión", async () => {
    respuestas.set("radar_de_tu_mac", EN_TU_MAC_VIGILADO);
    await pintaSesion(ESTADO_DE_LA_ESCUCHA_PRESENCIAL);
    const [invasivo] = EN_TU_MAC_VIGILADO.programas;
    expect(screen.getByText(invasivo.nombre, { exact: false })).toBeInTheDocument();
    const mdm = es.cuaderno.radarClases.mdm;
    expect(screen.getByText(`${mdm.titulo} ${mdm.sufijo}`)).toBeInTheDocument();
  });
});

describe("sin voz para tu idioma, la sesión lo dice (auditoría del S5, B13)", () => {
  it("pediste la ficha al oído, empezó la sala y la voz no se encendió: una línea dice por qué", async () => {
    const lleva = (escucha: EstadoDeEscucha) => (
      <Cascara>
        <SpriteIconos />
        <Sesion reunion={{ que: "ninguna" }} escucha={escucha} salida={AIRPODS} />
      </Cascara>
    );
    document.documentElement.lang = "es";
    const { rerender } = render(lleva(antesDeEmpezar));
    await act(async () => {});
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    await act(async () => {});
    expect(radio(tp.alOido)).toHaveAttribute("aria-checked", "true");
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: tp.empezar }));
    });
    // La sala abrió, pero Rust no pudo encender la voz: no hay voz para el idioma del consultor.
    rerender(lleva(ESTADO_DE_LA_ESCUCHA_PRESENCIAL));
    await emitir("voz", { ...LA_VOZ_APAGADA, puede: false, encendida: false });
    expect(screen.getByText(TEXTOS_DE_PRESENCIAL.es.sinVoz("es-ES"), { exact: false })).toBeInTheDocument();
  });

  it("si la voz sí está, o pediste la banda, no se dice nada", async () => {
    document.documentElement.lang = "es";
    await pintaSesion(ESTADO_DE_LA_ESCUCHA_PRESENCIAL);
    expect(screen.queryByText(/No hay voz para leer/)).toBeNull();
  });
});

describe("el micrófono de la sala, nombrado (auditoría del S5, M7)", () => {
  it("antes de empezar nombra el micrófono; si es externo, avisa y dice dónde cambiarlo", async () => {
    const te = TEXTOS_DE_PRESENCIAL.es;
    respuestas.set("entrada_de_audio", ENTRADA_DEL_EQUIPO);
    await pintaSesion();
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    await act(async () => {});
    expect(screen.getByText(te.microfono(ENTRADA_DEL_EQUIPO.nombre))).toBeInTheDocument();
    expect(screen.queryByText(new RegExp(te.enElMac))).toBeNull();

    cleanup();
    respuestas.set("entrada_de_audio", ENTRADA_EXTERNA);
    await pintaSesion();
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    await act(async () => {});
    const aviso = screen.getByText(new RegExp(`${ENTRADA_EXTERNA.nombre} es un dispositivo externo`));
    expect(aviso.textContent).toContain(te.enElMac);
  });

  it("con la sala en marcha ya no se dice: se nombra antes de empezar", async () => {
    respuestas.set("entrada_de_audio", ENTRADA_EXTERNA);
    await pintaSesion(ESTADO_DE_LA_ESCUCHA_PRESENCIAL);
    expect(screen.queryByText(/es un dispositivo externo/)).toBeNull();
  });

  it("los dos idiomas tienen lo mismo, y no prometen nada que la app no hace", () => {
    for (const idioma of ["es", "en"] as const) {
      const t = TEXTOS_DE_PRESENCIAL[idioma];
      expect(t.microfono("X")).toContain("X");
      expect(t.externa("X", "Y")).toMatch(/X.*Y/);
      expect(t.externa("X", "Y")).not.toMatch(/indetectable|trampa|enga[ñn]|undetectable|cheat/i);
    }
    expect(Object.keys(TEXTOS_DE_PRESENCIAL.es)).toEqual(Object.keys(TEXTOS_DE_PRESENCIAL.en));
  });
});

describe("lo que Rust emite de la sala llega a la pantalla (auditoría del S5, B9)", () => {
  it("el turno de la sala, tal como Rust lo serializa, se pinta como «sala» en el transcript de la banda", async () => {
    respuestas.set("estado_de_la_escucha", ESTADO_DE_LA_ESCUCHA_PRESENCIAL);
    respuestas.set("turnos_recientes", [TURNO_DE_LA_SALA]);
    const { container } = render(
      <Cascara>
        <Banda estado="ficha-presencial" transcript ampliada />
      </Cascara>,
    );
    await act(async () => {});
    await emitir("escucha", NOVEDAD_APARECE_FICHA);
    const quien = container.querySelector(".transcript-b .turno .quien");
    expect(quien, "el turno de la sala no se pintó").not.toBeNull();
    expect(quien!.textContent!.startsWith(tb.sala)).toBe(true);
    expect(quien!.textContent).not.toContain(tb.cliente);
    expect(container.textContent).toContain(TURNO_DE_LA_SALA.texto);
  });
});

describe("el idioma de la sala (auditoría del S5, M5)", () => {
  it("ofrece lo que este equipo sabe transcribir, y el idioma del cliente no desaparece al elegir otro", async () => {
    respuestas.set("que_sabe_transcribir", {
      motor: "apple-speechanalyzer",
      techo: 5,
      idiomas: [
        { codigo: "es-ES", disponibilidad: { estado: "listo" } },
        { codigo: "en-US", disponibilidad: { estado: "listo" } },
        { codigo: "es-CO", disponibilidad: { estado: "sin-modelo" } },
      ],
      motivo: null,
    });
    await pintaSesion();
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    await act(async () => {});
    const selector = screen.getByRole("combobox", { name: tp.idiomaDeLaSala }) as HTMLSelectElement;
    expect([...selector.options].map((o) => o.value)).toEqual(expect.arrayContaining(["es-ES", "en-US", "es-CO"]));
    // Elegir otro no hace desaparecer a es-CO de la lista.
    await act(async () => {
      fireEvent.change(selector, { target: { value: "en-US" } });
    });
    expect([...selector.options].map((o) => o.value)).toContain("es-CO");
  });

  it("dice el estado del modelo del idioma elegido: sin modelo, y con modelo", async () => {
    respuestas.set("que_sabe_transcribir", {
      motor: "apple-speechanalyzer",
      techo: 5,
      idiomas: [
        { codigo: "es-ES", disponibilidad: { estado: "listo" } },
        { codigo: "en-US", disponibilidad: { estado: "sin-modelo" } },
      ],
      motivo: null,
    });
    await pintaSesion();
    fireEvent.click(screen.getByRole("button", { name: tp.boton }));
    await act(async () => {});
    const selector = screen.getByRole("combobox", { name: tp.idiomaDeLaSala });
    await act(async () => {
      fireEvent.change(selector, { target: { value: "en-US" } });
    });
    expect(screen.getByText(es.cuaderno.sinModelo)).toBeInTheDocument();
    await act(async () => {
      fireEvent.change(selector, { target: { value: "es-ES" } });
    });
    expect(screen.getByText(es.cuaderno.modeloInstalado)).toBeInTheDocument();
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

  /** Auditoría del S5, M1: «Escuchando · la sala» con el check verde solo si la única pista abrió. */
  it("si el micrófono no abrió, la cabecera no dice que escucha la sala", async () => {
    respuestas.set("estado_de_la_escucha", {
      ...ESTADO_DE_LA_ESCUCHA_PRESENCIAL,
      microfono: { abierta: false, bytes: 0, motivo: "sin-permiso-del-microfono" },
    });
    render(
      <Cascara>
        <Banda estado="esperando" />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.queryByText(tb.escuchandoLaSala)).toBeNull();
    expect(screen.getByText(/Escuchando · 0 pistas/)).toBeInTheDocument();
  });

  /** Auditoría del S5, M2: sin ventana de reunión que acoplar, «sin acople» no es una avería en la sala. */
  it("«sin acople» no sale en presencial, y sí en una reunión (el control)", async () => {
    respuestas.set("estado_de_la_escucha", ESTADO_DE_LA_ESCUCHA_PRESENCIAL);
    render(
      <Cascara>
        <Banda estado="esperando" acoplada={false} />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.queryByText(tb.sinAcople)).toBeNull();

    cleanup();
    respuestas.set("estado_de_la_escucha", ESTADO_DE_LA_ESCUCHA);
    render(
      <Cascara>
        <Banda estado="esperando" acoplada={false} />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.getByText(tb.sinAcople)).toBeInTheDocument();
  });

  /** Auditoría del S5, B6: un turno de la sala puede acabar en ficha, así que la banda dice «Buscando…» como con el cliente. */
  it("un turno de la sala abre el «Buscando…» de la banda, y el tuyo no", async () => {
    respuestas.set("estado_de_la_escucha", ESTADO_DE_LA_ESCUCHA_PRESENCIAL);
    render(
      <Cascara>
        <Banda estado="esperando" />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.queryByText(tb.buscando)).toBeNull();
    await emitir("escucha", { que: "turno", pista: "microfono", eco: false });
    expect(screen.queryByText(tb.buscando), "tu turno no busca").toBeNull();
    await emitir("escucha", { que: "turno", pista: "sala", eco: false });
    expect(screen.getByText(tb.buscando), "el turno de la sala no abrió el «Buscando…»").toBeInTheDocument();
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
