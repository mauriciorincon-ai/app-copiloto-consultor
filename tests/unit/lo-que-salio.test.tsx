import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Ia } from "@/pantallas/Ia";
import { IdiomaContext } from "@/i18n";
import { es } from "@/i18n/es";
import { ESTADO_DE_LA_IA_NADIE, LO_QUE_SALIO, LO_QUE_SALIO_DEL_BANCO } from "@/contrato.generado";

/**
 * **LO QUE SALIÓ AL API, EN PANTALLA** (B37, sprint 003, mirada 19 filas 4 y 5). El registro lo
 * guarda Rust desde la fase 0; aquí se comprueba que la pantalla lo enseña **tal cual lo emite Rust**
 * (la muestra sale de `src/contrato.generado.ts`): el texto exacto, lo reemplazado tachado junto a
 * su marcador, y el botón solo cuando algo salió.
 *
 * ¿Puede fallar? Sí: con el botón siempre visible, «sin nada que enseñar no hay botón» es rojo; y
 * sin el `<del>`, «lo reemplazado va tachado» también (bitácora del S3).
 */
const t = es.cuaderno;
const respuestas = new Map<string, unknown>();

vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: () => () => {},
  preguntar: vi.fn((comando: string) => Promise.resolve(respuestas.get(comando) ?? null)),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

async function pinta() {
  render(
    <IdiomaContext.Provider value="es">
      <Ia />
    </IdiomaContext.Provider>,
  );
  await act(async () => {});
}

beforeEach(() => {
  cleanup();
  respuestas.clear();
  respuestas.set("estado_de_la_ia", ESTADO_DE_LA_IA_NADIE);
  respuestas.set("bytes_a_la_red", "1,2 KB");
});

describe("lo que salió al API", () => {
  it("sin nada que enseñar no hay botón", async () => {
    respuestas.set("lo_que_salio_al_api", []);
    await pinta();
    expect(screen.queryByRole("button", { name: new RegExp(t.verLoQueSalio) })).toBeNull();
  });

  it("con una petición, el botón la cuenta y abre el texto exacto, con lo reemplazado tachado", async () => {
    respuestas.set("lo_que_salio_al_api", [LO_QUE_SALIO]);
    await pinta();
    fireEvent.click(screen.getByRole("button", { name: `${t.verLoQueSalio} · 1` }));
    expect(screen.getByText(t.loUltimoQueSalio)).toBeInTheDocument();
    const tapados = LO_QUE_SALIO.trozos.filter((tr) => tr.que === "tapado");
    expect(tapados.length).toBeGreaterThan(0);
    for (const tr of tapados) {
      if (tr.que !== "tapado") continue;
      const del = screen.getByText(tr.original);
      expect(del.tagName).toBe("DEL");
      expect(del.nextElementSibling?.textContent).toBe(tr.marcador);
    }
    expect(screen.getByText(`${t.redactarSugerencia} · ${LO_QUE_SALIO.sobre}`)).toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: t.volverQuienRedacta }));
    expect(screen.getByText(t.quienRedacta)).toBeInTheDocument();
  });

  /** Auditoría del S4, M21: lo que salió para el ensayo se rotula como lo que es, no como una sugerencia. */
  it("lo del ensayo dice «enriquecer el banco», y la tabla, «peticiones recientes»", async () => {
    respuestas.set("lo_que_salio_al_api", [LO_QUE_SALIO_DEL_BANCO, LO_QUE_SALIO]);
    await pinta();
    fireEvent.click(screen.getByRole("button", { name: `${t.verLoQueSalio} · 2` }));
    expect(screen.getByText(`${t.paraElBanco} · ${LO_QUE_SALIO_DEL_BANCO.sobre}`)).toBeInTheDocument();
    expect(screen.getByText(`${t.redactarSugerencia} · ${LO_QUE_SALIO.sobre}`)).toBeInTheDocument();
    expect(screen.getByText(`${t.las} 2 ${t.peticionesDeEstaReunion}`)).toBeInTheDocument();
    expect(screen.getByText(t.registroEnMemoria)).toBeInTheDocument();
  });
});
