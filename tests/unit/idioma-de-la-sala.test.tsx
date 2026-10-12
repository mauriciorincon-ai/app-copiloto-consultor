import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Cascara } from "@/App";
import { SpriteIconos } from "@/componentes/Iconos";
import { Sesion } from "@/pantallas/Sesion";
import { es } from "@/i18n/es";
import { preguntar } from "@/puente";
import { ESTADO_DE_LA_ESCUCHA, IDIOMAS_DE_PISTA, LA_VOZ_APAGADA, SALIDA_DE_AUDIO, VISTA_DEL_CLIENTE } from "@/contrato.generado";
import type { EstadoDeEscucha } from "@/cuaderno";

/**
 * **EL IDIOMA DE LA SALA VIENE DE LA SALA** (sprint 008, auditoría del S5, B9).
 *
 * En `IDIOMAS_DE_PISTA` la sala valía lo mismo que el cliente (`en-US`): una sala que no llegara de Rust caería al idioma
 * del cliente y ningún test lo distinguiría. La muestra de Rust ahora trae `es-ES` para la sala y `en-US` para el cliente.
 * Va en su propio archivo porque el idioma de cada pista vive en un módulo de la webview que recuerda lo elegido: en el
 * archivo de presencial, otro test ya había elegido uno.
 *
 * ¿Puede fallar? Sí: con la sala leída de `cliente`, el selector muestra `en-US` y es rojo.
 */
const respuestas = new Map<string, unknown>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: () => () => {},
  preguntar: vi.fn((comando: string) => Promise.resolve(respuestas.get(comando) ?? null)),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

const antesDeEmpezar: EstadoDeEscucha = { ...ESTADO_DE_LA_ESCUCHA, escuchando: false };

beforeEach(() => {
  cleanup();
  respuestas.clear();
  vi.mocked(preguntar).mockClear();
  respuestas.set("este_cliente", VISTA_DEL_CLIENTE);
  respuestas.set("estado_de_la_voz", { ...LA_VOZ_APAGADA, puede: true });
  respuestas.set("idiomas_de_pista", IDIOMAS_DE_PISTA);
});

describe("el idioma de la sala", () => {
  it("la muestra de Rust trae una sala distinta del cliente", () => {
    expect(IDIOMAS_DE_PISTA.sala).toBe("es-ES");
    expect(IDIOMAS_DE_PISTA.sala).not.toBe(IDIOMAS_DE_PISTA.cliente);
  });

  it("Sesión en presencial elige el de la sala, no el del cliente", async () => {
    document.documentElement.lang = "es";
    render(
      <Cascara>
        <SpriteIconos />
        <Sesion reunion={{ que: "ninguna" }} escucha={antesDeEmpezar} salida={SALIDA_DE_AUDIO} />
      </Cascara>,
    );
    await act(async () => {});
    fireEvent.click(screen.getByRole("button", { name: es.cuaderno.presencial.boton }));
    await act(async () => {});
    const selector = screen.getByRole("combobox", { name: es.cuaderno.presencial.idiomaDeLaSala }) as HTMLSelectElement;
    expect(selector.value, "la sala cayó al idioma del cliente").toBe("es-ES");
  });
});
