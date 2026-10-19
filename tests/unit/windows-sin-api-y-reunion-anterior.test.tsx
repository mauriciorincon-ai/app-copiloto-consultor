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
import { Ia } from "@/pantallas/Ia";
import { Sesion } from "@/pantallas/Sesion";
import { CAMBIOS_DE_WINDOWS } from "@/i18n/windows-cambios";
import { TEXTOS_DE_WINDOWS } from "@/i18n/windows";
import { llamar, preguntar } from "@/puente";
import {
  EN_TU_MAC_VIGILADO,
  ESTADO_DE_LA_ESCUCHA,
  ESTADO_DE_LA_IA_CON_API,
  ESTADO_DE_LA_IA_EN_WINDOWS,
  LA_FRANJA_ARRIBA,
  PLATAFORMA_MACOS,
  PREPARACION_DEL_ENSAYO,
  PLATAFORMA_WINDOWS,
  VISTA_DEL_CLIENTE,
} from "@/contrato.generado";
import type { EstadoDeEscucha } from "@/cuaderno";

/**
 * **WINDOWS SIN API Y SIN GUARDAR SOLA** (sprint 013; ADR 023 enmienda 1; decisiones 1a y 2a del G-Plan).
 *
 *  1. **IA en Windows** no ofrece proveedor, clave ni costo; dice, con su motivo, que la sugerencia redactada es «no por
 *     ahora»; y los dos interruptores que necesitaban proveedor quedan apagados y bloqueados (el clic no llama a Rust). En
 *     el Mac, IA sigue como estaba.
 *  2. **Antes de empezar otra sesión**, en Windows, si la reunión anterior pide decidir, un cuadro «Guardar · Descartar ·
 *     Cancelar» va ANTES de cualquier arranque —los cuatro caminos de Sesión— y no se toca nada hasta que contestes. Sin
 *     nada que decidir, se empieza directo. En el Mac, ni se pregunta.
 *
 * Se finge el PUENTE como en `modo-transparente.test.tsx`; las respuestas de IA salen de `src/contrato.generado.ts`, que
 * escribe Rust con su serde (regla 19).
 *
 * ¿Puede fallar? Sí: sin el `!windows &&` de IA reaparece el proveedor; sin el `bloqueado` el clic llama a Rust; si algún
 * camino de arranque en Sesión llama a su comando sin pasar por `antes`, el cuadro no sale y el arranque va primero (el
 * orden de las llamadas lo delata); y sin el `esWindows` de `useAntesDeEmpezar` el Mac preguntaría.
 */
const tw = TEXTOS_DE_WINDOWS.es.reunionAnterior;
const twEn = TEXTOS_DE_WINDOWS.en.reunionAnterior;
const respuestas = new Map<string, unknown>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: vi.fn(() => () => {}),
  preguntar: vi.fn((comando: string) => {
    const r = respuestas.get(comando);
    if (typeof r === "function") return (r as () => Promise<unknown>)();
    return r instanceof Error ? Promise.reject(r) : Promise.resolve(r ?? null);
  }),
  llamar: vi.fn((comando: string) => {
    const r = respuestas.get(comando);
    if (typeof r === "function") return (r as () => Promise<unknown>)();
    return r instanceof Error ? Promise.reject(r) : Promise.resolve(true);
  }),
}));

/** Las llamadas a un comando, de las dos puertas del puente. */
const pedidos = (comando: string) =>
  [...vi.mocked(preguntar).mock.calls, ...vi.mocked(llamar).mock.calls].filter(
    ([c]) => c === comando,
  );

/** En qué orden se llamó (número global de invocación del primer pedido de cada comando; `null`: nunca). */
function ordenDe(comando: string): number | null {
  const ordenes = [
    ...vi
      .mocked(preguntar)
      .mock.calls.map(
        (c, i) =>
          [c[0], vi.mocked(preguntar).mock.invocationCallOrder[i]] as const,
      ),
    ...vi
      .mocked(llamar)
      .mock.calls.map(
        (c, i) =>
          [c[0], vi.mocked(llamar).mock.invocationCallOrder[i]] as const,
      ),
  ].filter(([c]) => c === comando);
  return ordenes.length > 0 ? Math.min(...ordenes.map(([, o]) => o)) : null;
}

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

const sesion = (escucha: Partial<EstadoDeEscucha> = {}) => (
  <Sesion
    reunion={{ que: "ninguna" }}
    escucha={{ ...ESTADO_DE_LA_ESCUCHA, escuchando: false, ...escucha }}
    salida={{ salida: "auriculares" }}
  />
);

/** Rust dice qué sistema es, y si la reunión anterior pide decidir. */
function conRust(plataforma: unknown, porDecidir: boolean | Error) {
  respuestas.set("plataforma_de_este_equipo", plataforma);
  respuestas.set("este_cliente", VISTA_DEL_CLIENTE);
  respuestas.set("reunion_por_decidir", porDecidir);
}

/** Una contestación de Rust que tarda: se resuelve cuando el test lo dice. */
function diferida() {
  let resolver!: (v: unknown) => void;
  const promesa = new Promise<unknown>((r) => {
    resolver = r;
  });
  return { promesa, resolver };
}

const clic = async (nombre: RegExp | string) => {
  await act(async () => {
    fireEvent.click(screen.getByRole("button", { name: nombre }));
  });
};

beforeEach(() => {
  cleanup();
  respuestas.clear();
  vi.mocked(preguntar).mockClear();
  vi.mocked(llamar).mockClear();
  respuestas.set("la_franja", LA_FRANJA_ARRIBA);
});

describe("IA en Windows: sin API, la sugerencia redactada es «no por ahora»", () => {
  it("no ofrece proveedor, clave ni costo, y dice por qué (en español)", async () => {
    respuestas.set("plataforma_de_este_equipo", PLATAFORMA_WINDOWS);
    respuestas.set("estado_de_la_ia", ESTADO_DE_LA_IA_EN_WINDOWS);
    const { container } = await pinta(<Ia />);
    for (const ausente of [
      "Proveedor externo",
      "Tu clave",
      "Costo",
      "API externo",
      "MLX",
    ]) {
      expect(container.textContent, ausente).not.toContain(ausente);
    }
    expect(container.querySelector("input[type=password]")).toBeNull();
    expect(container.textContent).toContain(
      "La sugerencia redactada no está en Windows por ahora",
    );
    expect(container.textContent).toContain("El API de pago salió de Windows");
    expect(container.textContent).toContain("Modelo local");
    expect(container.textContent).toContain("no por ahora");
    expect(container.textContent).not.toMatch(/Apple|macOS|\bMac\b/);
  });

  it("lo mismo en inglés", async () => {
    respuestas.set("plataforma_de_este_equipo", PLATAFORMA_WINDOWS);
    respuestas.set("estado_de_la_ia", ESTADO_DE_LA_IA_EN_WINDOWS);
    const { container } = await pinta(<Ia />, "en");
    for (const ausente of [
      "External provider",
      "Your key",
      "Cost",
      "External API",
    ]) {
      expect(container.textContent, ausente).not.toContain(ausente);
    }
    expect(container.textContent).toContain(
      "The written suggestion is not on Windows for now",
    );
    expect(container.textContent).toContain(
      "The paid API is gone from Windows",
    );
    expect(container.textContent).not.toMatch(/Apple|macOS|\bMac\b/);
  });

  it("«Redactar sugerencias» y «Enriquecer el banco» se ven apagados y bloqueados aunque Rust los recuerde encendidos", async () => {
    respuestas.set("plataforma_de_este_equipo", PLATAFORMA_WINDOWS);
    // Rust ya no lo manda encendido en Windows (`estado_de`); la pantalla se defiende aun así de un estado viejo.
    expect(
      ESTADO_DE_LA_IA_EN_WINDOWS.redactar ||
        ESTADO_DE_LA_IA_EN_WINDOWS.enriquecer,
    ).toBe(false);
    respuestas.set("estado_de_la_ia", {
      ...ESTADO_DE_LA_IA_EN_WINDOWS,
      redactar: true,
      enriquecer: true,
    });
    await pinta(<Ia />);
    for (const nombre of [/Redactar sugerencias/, /Enriquecer el banco/]) {
      const interruptor = screen.getByRole("switch", { name: nombre });
      expect(interruptor).toHaveAttribute("aria-checked", "false");
      expect(interruptor).toHaveAttribute("aria-disabled", "true");
      // Bloqueado se dice con texto y con motivo, no solo con menos color (regla 8; auditoría del S13, B-2).
      expect(interruptor).toHaveTextContent("no por ahora");
      expect(interruptor).toHaveAccessibleDescription(
        /La sugerencia redactada no está en Windows por ahora|no hay modelo|conexión|Microsoft/i,
      );
      await act(async () => {
        fireEvent.click(interruptor);
      });
    }
    expect(pedidos("redactar_sugerencias")).toEqual([]);
    expect(pedidos("enriquecer_el_banco")).toEqual([]);
  });

  it("la puerta local sigue en Windows", async () => {
    respuestas.set("plataforma_de_este_equipo", PLATAFORMA_WINDOWS);
    respuestas.set("estado_de_la_ia", ESTADO_DE_LA_IA_EN_WINDOWS);
    await pinta(<Ia />);
    expect(
      screen.getByRole("button", { name: /Puerta local/ }),
    ).toBeInTheDocument();
  });

  it("en el Mac, IA sigue con su proveedor, su clave, su costo y sus interruptores a mano", async () => {
    respuestas.set("plataforma_de_este_equipo", PLATAFORMA_MACOS);
    respuestas.set("estado_de_la_ia", ESTADO_DE_LA_IA_CON_API);
    respuestas.set("redactar_sugerencias", ESTADO_DE_LA_IA_CON_API);
    const { container } = await pinta(<Ia />);
    for (const presente of [
      "Proveedor externo",
      "Tu clave",
      "Costo",
      "API externo",
    ]) {
      expect(container.textContent, presente).toContain(presente);
    }
    const interruptor = screen.getByRole("switch", {
      name: /Redactar sugerencias/,
    });
    expect(interruptor).not.toHaveAttribute("aria-disabled");
    await act(async () => {
      fireEvent.click(interruptor);
    });
    expect(pedidos("redactar_sugerencias")).toHaveLength(1);
  });
});

describe("Sesión en Windows: antes de empezar otra, la reunión anterior decide", () => {
  /** Los cuatro caminos que empiezan una sesión, cada uno con su comando de Rust. */
  const CAMINOS: {
    nombre: string;
    /** Lo que Rust diría para que este camino exista (el radar vio algo, la NDA lo prohíbe). */
    preparar?: () => void;
    abrir: () => Promise<void>;
    comando: string;
  }[] = [
    {
      nombre: "Iniciar sesión",
      abrir: () => clic(/Iniciar sesión/),
      comando: "empezar_a_escuchar",
    },
    {
      nombre: "Solo notas",
      abrir: () => clic(/Solo notas/),
      comando: "empezar_solo_notas",
    },
    {
      nombre: "Empezar en presencial",
      abrir: async () => {
        await clic(/Presencial/);
        await clic(/Empezar en presencial/);
      },
      comando: "empezar_presencial",
    },
    {
      nombre: "Solo notas, en presencial",
      abrir: async () => {
        await clic(/Presencial/);
        await clic(/Solo notas/);
      },
      comando: "empezar_solo_notas",
    },
    {
      nombre: "Iniciar de todos modos (el radar vio algo)",
      preparar: () => respuestas.set("radar_de_tu_mac", EN_TU_MAC_VIGILADO),
      abrir: () => clic(/Iniciar de todos modos/),
      comando: "empezar_a_escuchar",
    },
    {
      nombre: "Iniciar en modo solo notas (la NDA lo prohíbe)",
      preparar: () =>
        respuestas.set("este_cliente", {
          ...VISTA_DEL_CLIENTE,
          nda: "lo-prohibe",
        }),
      abrir: () => clic(/Iniciar en modo solo notas/),
      comando: "empezar_solo_notas",
    },
  ];

  it.each(CAMINOS)(
    "«$nombre» con la reunión anterior abierta: pregunta primero y no empieza nada",
    async ({ abrir, comando, preparar }) => {
      conRust(PLATAFORMA_WINDOWS, true);
      preparar?.();
      await pinta(sesion());
      await abrir();
      const cuadro = screen.getByRole("alertdialog", { name: tw.titulo });
      expect(cuadro).toHaveTextContent(tw.detalle);
      for (const boton of [tw.guardar, tw.descartar, tw.cancelar]) {
        expect(screen.getByRole("button", { name: boton })).toBeInTheDocument();
      }
      expect(screen.getByRole("button", { name: tw.cancelar })).toHaveFocus();
      // Nada se tocó: ni se empezó, ni se guardó, ni se descartó.
      expect(pedidos(comando)).toEqual([]);
      expect(pedidos("guardar_la_reunion")).toEqual([]);
      expect(pedidos("cerrar_sin_guardar")).toEqual([]);
    },
  );

  it("Guardar: guarda y DESPUÉS empieza lo que pedías", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    await pinta(sesion());
    await clic(/Iniciar sesión/);
    await clic(tw.guardar);
    expect(pedidos("guardar_la_reunion")).toHaveLength(1);
    expect(pedidos("empezar_a_escuchar")).toHaveLength(1);
    expect(ordenDe("guardar_la_reunion")!).toBeLessThan(
      ordenDe("empezar_a_escuchar")!,
    );
    expect(pedidos("cerrar_sin_guardar")).toEqual([]);
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  it("Descartar: descarta y DESPUÉS empieza, sin guardar nada", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    await pinta(sesion());
    await clic(/Solo notas/);
    await clic(tw.descartar);
    expect(pedidos("cerrar_sin_guardar")).toHaveLength(1);
    expect(pedidos("empezar_solo_notas")).toHaveLength(1);
    expect(ordenDe("cerrar_sin_guardar")!).toBeLessThan(
      ordenDe("empezar_solo_notas")!,
    );
    expect(pedidos("guardar_la_reunion")).toEqual([]);
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  it("Cancelar: no cambia nada, no empieza nada y el cuadro se va", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    await pinta(sesion());
    await clic(/Iniciar sesión/);
    await clic(tw.cancelar);
    expect(screen.queryByRole("alertdialog")).toBeNull();
    for (const comando of [
      "empezar_a_escuchar",
      "guardar_la_reunion",
      "cerrar_sin_guardar",
    ]) {
      expect(pedidos(comando), comando).toEqual([]);
    }
    // Y se puede volver a pedir: pregunta otra vez.
    await clic(/Iniciar sesión/);
    expect(
      screen.getByRole("alertdialog", { name: tw.titulo }),
    ).toBeInTheDocument();
    expect(pedidos("reunion_por_decidir")).toHaveLength(2);
  });

  it("si guardar falla, la reunión sigue abierta, se dice y no se empezó nada", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    respuestas.set("guardar_la_reunion", new Error("el almacén no dejó"));
    await pinta(sesion());
    await clic(/Iniciar sesión/);
    await clic(tw.guardar);
    expect(screen.getByRole("alert")).toHaveTextContent(tw.noGuardo);
    expect(
      screen.getByRole("alertdialog", { name: tw.titulo }),
    ).toBeInTheDocument();
    expect(pedidos("empezar_a_escuchar")).toEqual([]);
  });

  it("sin nada que decidir se empieza directo, sin cuadro, y sin guardar ni descartar", async () => {
    conRust(PLATAFORMA_WINDOWS, false);
    await pinta(sesion());
    await clic(/Iniciar sesión/);
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(pedidos("reunion_por_decidir")).toHaveLength(1);
    expect(pedidos("empezar_a_escuchar")).toHaveLength(1);
    expect(pedidos("guardar_la_reunion")).toEqual([]);
    expect(pedidos("cerrar_sin_guardar")).toEqual([]);
  });

  it("si la pregunta misma falla, no se adivina: se pregunta", async () => {
    conRust(PLATAFORMA_WINDOWS, new Error("sin puente"));
    await pinta(sesion());
    await clic(/Iniciar sesión/);
    expect(
      screen.getByRole("alertdialog", { name: tw.titulo }),
    ).toBeInTheDocument();
    expect(pedidos("empezar_a_escuchar")).toEqual([]);
  });

  it("el cuadro habla en el idioma de la interfaz", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    await pinta(sesion(), "en");
    await clic(/Start session/);
    const cuadro = screen.getByRole("alertdialog", { name: twEn.titulo });
    expect(cuadro).toHaveTextContent(twEn.detalle);
    for (const boton of ["Save", "Discard", "Cancel"]) {
      expect(screen.getByRole("button", { name: boton })).toBeInTheDocument();
    }
  });

  it("en el Mac no se pregunta nada: se empieza directo y Rust sigue guardando solo, como siempre", async () => {
    conRust(PLATAFORMA_MACOS, true);
    await pinta(sesion());
    await clic(/Iniciar sesión/);
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(pedidos("reunion_por_decidir")).toEqual([]);
    expect(pedidos("empezar_a_escuchar")).toHaveLength(1);
  });

  it("el cuadro nombra las dos protecciones que Guardar toca, en los dos idiomas (regla 24)", () => {
    expect(tw.detalle).toMatch(/Administrador de credenciales/);
    expect(tw.detalle).toMatch(/Programador de tareas/);
    expect(twEn.detalle).toMatch(/Credential Manager/);
    expect(twEn.detalle).toMatch(/Task Scheduler/);
  });

  it("los dos idiomas tienen el mismo cuadro, completo", () => {
    expect(Object.keys(tw).sort()).toEqual(Object.keys(twEn).sort());
    for (const idioma of ["es", "en"] as const) {
      for (const [clave, texto] of Object.entries(
        TEXTOS_DE_WINDOWS[idioma].reunionAnterior,
      )) {
        expect(texto.trim(), `${idioma}.reunionAnterior.${clave}`).not.toBe("");
      }
    }
  });
});

describe("El cuadro, al detalle (auditoría del S13)", () => {
  it("cambiar de modo con el cuadro abierto lo cierra y «Guardar» ya no arranca la sesión del primer clic (M-3)", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    await pinta(sesion());
    await clic(/Iniciar sesión/);
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    await clic(/Presencial/);
    expect(screen.queryByRole("alertdialog")).toBeNull();
    for (const comando of [
      "empezar_a_escuchar",
      "empezar_presencial",
      "guardar_la_reunion",
      "cerrar_sin_guardar",
    ]) {
      expect(pedidos(comando), comando).toEqual([]);
    }
  });

  it("si cambias de modo mientras Rust contesta la pregunta, no aparece un cuadro de lo que ya no ves (M-3)", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    const d = diferida();
    respuestas.set("reunion_por_decidir", () => d.promesa);
    await pinta(sesion());
    await clic(/Iniciar sesión/);
    await clic(/Presencial/);
    await act(async () => {
      d.resolver(true);
    });
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(pedidos("empezar_a_escuchar")).toEqual([]);
  });

  it("doble clic en Guardar: guarda una vez, empieza una vez, y los tres botones esperan (M-4)", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    const d = diferida();
    respuestas.set("guardar_la_reunion", () => d.promesa);
    await pinta(sesion());
    await clic(/Iniciar sesión/);
    const guardar = screen.getByRole("button", { name: tw.guardar });
    await act(async () => {
      fireEvent.click(guardar);
      fireEvent.click(guardar);
    });
    expect(pedidos("guardar_la_reunion")).toHaveLength(1);
    for (const boton of [tw.guardar, tw.descartar, tw.cancelar]) {
      expect(screen.getByRole("button", { name: boton })).toHaveAttribute(
        "aria-disabled",
        "true",
      );
    }
    // Mientras vuela, ni Descartar ni Cancelar hacen nada.
    await clic(tw.descartar);
    await clic(tw.cancelar);
    expect(pedidos("cerrar_sin_guardar")).toEqual([]);
    expect(screen.getByRole("alertdialog")).toBeInTheDocument();
    await act(async () => {
      d.resolver(null);
    });
    expect(pedidos("empezar_a_escuchar")).toHaveLength(1);
    expect(screen.queryByRole("alertdialog")).toBeNull();
  });

  it("si Descartar es rechazado, se dice, el cuadro sigue y no se empezó nada; y se puede reintentar (M-4)", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    respuestas.set("cerrar_sin_guardar", new Error("el almacén no dejó"));
    await pinta(sesion());
    await clic(/Solo notas/);
    await clic(tw.descartar);
    expect(screen.getByRole("alert")).toHaveTextContent(tw.noDescarto);
    expect(
      screen.getByRole("alertdialog", { name: tw.titulo }),
    ).toBeInTheDocument();
    expect(pedidos("empezar_solo_notas")).toEqual([]);
    expect(
      screen.getByRole("button", { name: tw.descartar }),
    ).not.toHaveAttribute("aria-disabled");
    respuestas.delete("cerrar_sin_guardar");
    await clic(tw.descartar);
    expect(pedidos("empezar_solo_notas")).toHaveLength(1);
  });

  it("Esc cancela, Tab da la vuelta dentro del cuadro y el foco vuelve al botón que lo abrió (B-1)", async () => {
    conRust(PLATAFORMA_WINDOWS, true);
    await pinta(sesion());
    const abre = screen.getByRole("button", { name: /Iniciar sesión/ });
    abre.focus();
    await clic(/Iniciar sesión/);
    const cuadro = screen.getByRole("alertdialog", { name: tw.titulo });
    expect(cuadro).toHaveAttribute("aria-modal", "true");
    expect(cuadro).toHaveAccessibleDescription(tw.detalle);
    const guardar = screen.getByRole("button", { name: tw.guardar });
    const cancelar = screen.getByRole("button", { name: tw.cancelar });
    expect(cancelar).toHaveFocus();
    // Tab desde el último vuelve al primero; Mayús+Tab desde el primero va al último.
    fireEvent.keyDown(cancelar, { key: "Tab" });
    expect(guardar).toHaveFocus();
    fireEvent.keyDown(guardar, { key: "Tab", shiftKey: true });
    expect(cancelar).toHaveFocus();
    // Esc: el cuadro se va, no se empieza nada y el foco vuelve al botón de Start.
    fireEvent.keyDown(cancelar, { key: "Escape" });
    expect(screen.queryByRole("alertdialog")).toBeNull();
    expect(pedidos("empezar_a_escuchar")).toEqual([]);
    expect(abre).toHaveFocus();
  });
});

describe("Ensayo en Windows: no manda a «encenderlo en IA» (auditoría del S13, A-2)", () => {
  it("el banco enriquecido dice que está «no por ahora» y no manda a IA, en español", async () => {
    respuestas.set("plataforma_de_este_equipo", PLATAFORMA_WINDOWS);
    respuestas.set("preparar_el_ensayo", PREPARACION_DEL_ENSAYO);
    document.documentElement.lang = "es";
    const { container } = render(
      <Cascara>
        <SpriteIconos />
        <Ensayo ir={vi.fn()} />
      </Cascara>,
    );
    await act(async () => {});
    expect(container.textContent).toContain("no por ahora");
    expect(container.textContent).not.toContain("Se enciende en IA");
    expect(container.textContent).not.toMatch(/Enriquecer el banco: encendido/);
  });

  it("y en inglés", async () => {
    respuestas.set("plataforma_de_este_equipo", PLATAFORMA_WINDOWS);
    respuestas.set("preparar_el_ensayo", PREPARACION_DEL_ENSAYO);
    document.documentElement.lang = "en";
    const { container } = render(
      <Cascara>
        <SpriteIconos />
        <Ensayo ir={vi.fn()} />
      </Cascara>,
    );
    await act(async () => {});
    expect(container.textContent).toContain("not for now");
    expect(container.textContent).not.toContain("Turn it on in AI");
  });

  it("el motivo «nadie puede redactar» tampoco manda a IA en Windows, y el Mac conserva el suyo", () => {
    for (const idioma of ["es", "en"] as const) {
      const motivo =
        CAMBIOS_DE_WINDOWS[idioma].ensayo?.porQueNoSumo?.["sin-proveedor"];
      expect(motivo, idioma).toBeTruthy();
      expect(motivo, idioma).not.toMatch(/\bIA\b|\bAI\b/);
    }
  });
});
