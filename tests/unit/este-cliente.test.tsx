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
import { es } from "@/i18n/es";
import { llamar, preguntar } from "@/puente";
import {
  BANDERA_FUERA_DEL_CATALOGO,
  BANDERA_SIN_INDICAR,
  ESCUCHA_SOLO_NOTAS,
  ESTADO_DE_LA_ESCUCHA,
  NDA_LO_PROHIBE,
  VISTA_DEL_CLIENTE,
} from "@/contrato.generado";
import type { EstadoDeEscucha } from "@/cuaderno";

/**
 * **«ESTE CLIENTE» DENTRO DE TAURI** (C11, sprint 003, fase 3, ADR 017) — lo que Sesión hace con lo
 * que Rust le da, y lo que le pide.
 *
 * Se finge el PUENTE, como en `notas.test.tsx`, y las respuestas salen de `src/contrato.generado.ts`,
 * que escribe Rust con su serde y con el catálogo de verdad (regla 19).
 *
 * ¿Puede fallar? Sí: sin la pregunta antes de guardar la respuesta, «Revisar abre la pregunta…» es
 * rojo; y con «Solo notas» llamando a `empezar_a_escuchar`, el último caso lo es (bitácora).
 */
const t = es.cliente;
const respuestas = new Map<string, unknown>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: () => () => {},
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

const antesDeEmpezar: EstadoDeEscucha = {
  ...ESTADO_DE_LA_ESCUCHA,
  escuchando: false,
};

async function pinta(escucha: EstadoDeEscucha = antesDeEmpezar) {
  document.documentElement.lang = "es";
  const r = render(
    <Cascara>
      <SpriteIconos />
      <Sesion
        reunion={{ que: "ninguna" }}
        escucha={escucha}
        salida={{ salida: "auriculares" }}
      />
    </Cascara>,
  );
  await act(async () => {});
  return r;
}

beforeEach(() => {
  cleanup();
  respuestas.clear();
  vi.mocked(preguntar).mockClear();
  vi.mocked(llamar).mockClear();
  respuestas.set("este_cliente", VISTA_DEL_CLIENTE);
});

describe("«Este cliente» dentro de Tauri", () => {
  it("elegir otro cliente se lo pide a Rust y pinta la bandera que vuelve", async () => {
    respuestas.set("elegir_cliente", {
      ...VISTA_DEL_CLIENTE,
      elegido: "Sur del Valle",
      bandera: BANDERA_SIN_INDICAR,
    });
    await pinta();
    await act(async () => {
      fireEvent.change(
        screen.getByRole("combobox", { name: es.cuaderno.esteCliente }),
        {
          target: { value: "Sur del Valle" },
        },
      );
    });
    expect(pedidos("elegir_cliente").at(-1)?.[1]).toEqual({
      nombre: "Sur del Valle",
    });
    expect(screen.getByText(t.noIndicada)).toBeInTheDocument();
    expect(screen.getByText(t.indicaDonde)).toBeInTheDocument();
  });

  it("una jurisdicción que el catálogo no trae se dice tal cual, sin adivinar", async () => {
    respuestas.set("este_cliente", {
      ...VISTA_DEL_CLIENTE,
      bandera: BANDERA_FUERA_DEL_CATALOGO,
    });
    await pinta();
    expect(
      screen.getByText(new RegExp(`«Bolivia» ${t.noEstaEnElCatalogo} v1`)),
    ).toBeInTheDocument();
    expect(screen.getByText(t.escribela)).toBeInTheDocument();
  });

  it("«Revisar» abre la pregunta, y «Sí, lo prohíbe» la guarda y lleva a solo notas", async () => {
    respuestas.set("responder_nda", {
      ...VISTA_DEL_CLIENTE,
      nda: NDA_LO_PROHIBE,
    });
    await pinta();
    expect(screen.queryByText(t.pregunta)).toBeNull();
    fireEvent.click(screen.getByRole("button", { name: t.revisar }));
    const pregunta = screen.getByRole("group", { name: "NDA" });
    expect(within(pregunta).getByText(t.pregunta)).toBeInTheDocument();
    // sin respuesta previa no hay nada que borrar
    expect(pedidos("revisar_nda")).toHaveLength(0);
    await act(async () => {
      fireEvent.click(
        within(pregunta).getByRole("button", { name: t.siLoProhibe }),
      );
    });
    expect(pedidos("responder_nda").at(-1)?.[1]).toEqual({ prohibe: true });
    // el estado aprobado «NDA prohíbe transcribir»: no bloquea, propone solo notas
    expect(screen.getByText(t.ndaProhibe)).toBeInTheDocument();
    expect(screen.getByText(t.queQuedaApagado)).toBeInTheDocument();
    await act(async () => {
      fireEvent.click(
        screen.getByRole("button", { name: new RegExp(t.iniciarSoloNotas) }),
      );
    });
    expect(pedidos("empezar_solo_notas")).toHaveLength(1);
    expect(pedidos("empezar_a_escuchar")).toHaveLength(0);
  });

  it("«Volver a revisar la NDA» borra la respuesta y vuelve a preguntar", async () => {
    respuestas.set("este_cliente", {
      ...VISTA_DEL_CLIENTE,
      nda: NDA_LO_PROHIBE,
    });
    respuestas.set("revisar_nda", VISTA_DEL_CLIENTE);
    await pinta();
    await act(async () => {
      fireEvent.click(screen.getByRole("button", { name: t.volverARevisar }));
    });
    expect(pedidos("revisar_nda")).toHaveLength(1);
    expect(screen.getByRole("group", { name: "NDA" })).toBeInTheDocument();
  });

  it("la cláusula enseña las dos versiones y copia la que pulsas, no la de la interfaz", async () => {
    const writeText = vi.fn(() => Promise.resolve());
    Object.defineProperty(navigator, "clipboard", {
      value: { writeText },
      configurable: true,
    });
    await pinta();
    fireEvent.click(
      screen.getByRole("button", { name: new RegExp(t.clausulaDeEncargo) }),
    );
    expect(screen.getByText(t.plantilla)).toBeInTheDocument();
    expect(screen.getByText(VISTA_DEL_CLIENTE.clausula.es)).toHaveAttribute(
      "lang",
      "es",
    );
    expect(screen.getByText(VISTA_DEL_CLIENTE.clausula.en)).toHaveAttribute(
      "lang",
      "en",
    );
    const copiar = screen.getAllByRole("button", { name: t.copiar });
    await act(async () => {
      fireEvent.click(copiar[1]);
    });
    expect(writeText).toHaveBeenCalledWith(VISTA_DEL_CLIENTE.clausula.en);
    expect(
      screen.getByRole("button", { name: new RegExp(t.copiada) }),
    ).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: t.volver }));
    expect(screen.getByText(es.cuaderno.dosPistas)).toBeInTheDocument();
  });

  it("«Solo notas» empieza la reunión sin captura, y en marcha se termina como cualquier otra", async () => {
    await pinta();
    await act(async () => {
      fireEvent.click(
        screen.getByRole("button", { name: new RegExp(`^${t.soloNotas}$`) }),
      );
    });
    expect(pedidos("empezar_solo_notas")).toHaveLength(1);
    expect(pedidos("empezar_a_escuchar")).toHaveLength(0);
    cleanup();
    await pinta(ESCUCHA_SOLO_NOTAS);
    expect(screen.getByText(t.modoSoloNotas)).toBeInTheDocument();
    await act(async () => {
      fireEvent.click(
        screen.getByRole("button", {
          name: new RegExp(es.cuaderno.terminarSesion),
        }),
      );
    });
    expect(pedidos("dejar_de_escuchar")).toHaveLength(1);
  });
});

describe("la banda en solo notas", () => {
  it("lo dice en la cabecera y no afirma que escucha", async () => {
    respuestas.set("estado_de_la_escucha", ESCUCHA_SOLO_NOTAS);
    document.documentElement.lang = "es";
    render(
      <Cascara>
        <SpriteIconos />
        <Banda estado="esperando" />
      </Cascara>,
    );
    await act(async () => {});
    expect(screen.getByText(es.banda.soloNotasCab)).toBeInTheDocument();
    expect(screen.getByText(es.banda.esperandoSoloNotas)).toBeInTheDocument();
    expect(
      screen.queryByText(new RegExp(es.banda.escuchandoPrefijo)),
    ).toBeNull();
  });
});
