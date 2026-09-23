import { act, cleanup, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Idioma } from "@/pantallas/Idioma";
import { SpriteIconos } from "@/componentes/Iconos";
import { IdiomaContext, es, en, type Idioma as Lengua } from "@/i18n";
import { preguntar } from "@/puente";
import { DEL_CLIENTE, DEL_CONSULTOR, type QueSabeTranscribir } from "@/cuaderno";

/**
 * **EL IDIOMA QUE NO TIENE MODELO** — hallazgos A6 y A7 de la auditoría.
 *
 * Es el estado más probable de todos: un Mac en español no trae el modelo de inglés, y el cliente
 * habla inglés. Lo que había: la pantalla decía «sin modelo» —en español, también en la interfaz
 * inglesa, porque la cadena estaba escrita dentro del componente— y no ofrecía nada que hacer. El
 * comando que instala el modelo existía **sin un solo llamador**, así que desde dentro de la app la
 * transcripción del cliente era inalcanzable, mientras tres comentarios del código hablaban de un
 * botón que no existía.
 *
 * Los ocho encuadres en inglés del gate de FIDELIDAD tampoco podían verlo: con los datos de muestra
 * los dos modelos están instalados, así que ese texto no se pinta nunca.
 */
vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: () => () => {},
  preguntar: vi.fn(() => Promise.resolve(null)),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

function transcribeCon(delCliente: QueSabeTranscribir["idiomas"][number]["disponibilidad"]) {
  return {
    motor: "apple-speechanalyzer",
    techo: 5,
    idiomas: [
      { codigo: DEL_CONSULTOR, disponibilidad: { estado: "listo" as const } },
      { codigo: DEL_CLIENTE, disponibilidad: delCliente },
    ],
    motivo: null,
  } satisfies QueSabeTranscribir;
}

function pinta(transcribe: QueSabeTranscribir, lengua: Lengua = "es") {
  cleanup();
  return render(
    <IdiomaContext.Provider value={lengua}>
      <SpriteIconos />
      <Idioma transcribe={transcribe} />
    </IdiomaContext.Provider>,
  );
}

beforeEach(() => {
  vi.mocked(preguntar).mockReset();
  vi.mocked(preguntar).mockResolvedValue(null);
});

describe("el idioma sin modelo", () => {
  it("lo dice en el idioma de la interfaz, no en español siempre", () => {
    pinta(transcribeCon({ estado: "sin-modelo" }), "es");
    expect(screen.getByText(es.cuaderno.sinModelo)).toBeInTheDocument();

    pinta(transcribeCon({ estado: "sin-modelo" }), "en");
    expect(screen.getByText(en.cuaderno.sinModelo)).toBeInTheDocument();
    expect(screen.queryByText(es.cuaderno.sinModelo)).toBeNull();
  });

  it("ofrece instalarlo, y al pulsar se lo pide a macOS con ese idioma", async () => {
    pinta(transcribeCon({ estado: "sin-modelo" }));
    const boton = screen.getByRole("button", { name: new RegExp(es.cuaderno.instalarModelo) });

    await act(async () => boton.click());

    expect(vi.mocked(preguntar)).toHaveBeenCalledWith("instalar_idioma", { codigo: DEL_CLIENTE });
  });

  it("mientras macOS descarga, el botón lo dice y no se puede volver a pulsar", async () => {
    // La descarga tarda: se deja la promesa sin resolver para mirar el estado intermedio, que es el
    // que el usuario va a ver durante minutos.
    let resolver: (v: unknown) => void = () => {};
    vi.mocked(preguntar).mockReturnValue(new Promise((r) => (resolver = r)));
    pinta(transcribeCon({ estado: "sin-modelo" }));

    await act(async () => {
      screen.getByRole("button", { name: new RegExp(es.cuaderno.instalarModelo) }).click();
    });

    const esperando = screen.getByRole("button", { name: new RegExp(es.cuaderno.instalando) });
    expect(esperando).toBeDisabled();

    // Y cuando termina, las DOS pistas tienen su modelo: la del consultor ya lo tenía.
    await act(async () => resolver({ estado: "listo" }));
    expect(screen.getAllByText(es.cuaderno.modeloInstalado)).toHaveLength(2);
    expect(screen.queryByRole("button", { name: new RegExp(es.cuaderno.instalando) })).toBeNull();
  });

  it("si el Mac no conoce el idioma NO ofrece instalar nada: no hay nada que descargar", () => {
    pinta(transcribeCon({ estado: "idioma-desconocido" }));
    expect(screen.getByText(es.cuaderno.noLoReconoce)).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: new RegExp(es.cuaderno.instalarModelo) })).toBeNull();
  });

  it("sin motor de voz tampoco, y lo dice sin enseñar la frase que Rust escribe para el log", () => {
    pinta(transcribeCon({ estado: "sin-motor", motivo: "este Mac no trae el transcriptor" }));
    expect(screen.getByText(es.cuaderno.sinMotorDeVoz)).toBeInTheDocument();
    expect(screen.queryByText("este Mac no trae el transcriptor")).toBeNull();
    expect(screen.queryByRole("button", { name: new RegExp(es.cuaderno.instalarModelo) })).toBeNull();
  });
});
