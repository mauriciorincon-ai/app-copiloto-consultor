import { act, render } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Banda } from "@/componentes/Banda";
import { IdiomaContext } from "@/i18n";
import { preguntar } from "@/puente";
import {
  APARICION_DEL_ATAJO,
  NOVEDAD_APARECE_FICHA,
  NOVEDAD_APARECE_SIN_RESULTADO,
  NOVEDAD_TURNO,
} from "@/contrato.generado";

/**
 * **EL CAMINO QUE NINGÚN TEST DE ESTE SPRINT ATRAVESABA.**
 *
 * Los 87 unitarios y los 66 e2e corren con `hayTauri() === false`, donde `useFicha` devuelve la
 * muestra de la maqueta y **no se suscribe a nada**. Dentro del producto es al revés: la ficha
 * llega por el evento `escucha`, y ahí el sprint tenía su defecto más caro — el webview leía
 * `{"Aparece":{…}}` y Rust emite `{"que":"aparece", …}`, así que la banda se quedaba en
 * «esperando» toda la reunión. La cobertura lo señalaba desde dos fases antes: este bloque de
 * `src/ficha.ts` era el único sin cubrir. Lo encontró la auditoría (C1).
 *
 * Por eso este test hace dos cosas que ningún otro hacía:
 *
 * 1. **Finge Tauri** —el objeto que `hayTauri()` busca y el `listen` del puente— para que la
 *    suscripción se monte de verdad.
 * 2. **No inventa el evento.** Los payloads salen de `src/contrato.generado.ts`, que lo escribe
 *    Rust con el serde de producción. Escribirlos a mano aquí sería repetir el defecto: el sprint
 *    ya tenía dos copias del contrato y ninguna comparada.
 */

/** Los oyentes que la banda registra, por nombre de evento. */
const oyentes = new Map<string, ((dato: unknown) => void)[]>();

/**
 * **Se finge el PUENTE, no la API de Tauri.** El puente es la frontera —y tiene sus propios tests
 * contra la API de verdad, en `acople.test.tsx`—; lo que aquí se prueba es lo que hay encima. Se
 * intentó primero por la otra vía, fingiendo `@tauri-apps/api/event`, y no sirvió: la banda
 * registra cinco suscripciones en el mismo instante y solo la primera llegaba al doble, así que
 * el test habría medido una de cinco creyendo medir las cinco. El seam bueno es el que la app
 * declara.
 */
vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: (evento: string, alOir: (dato: unknown) => void) => {
    oyentes.set(evento, [...(oyentes.get(evento) ?? []), alOir]);
    return () => oyentes.delete(evento);
  },
  // Sin turnos y sin ficha a mano: lo que se mide aquí es el evento automático.
  preguntar: vi.fn(() => Promise.resolve(null)),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

/** Lo que Rust haría: emitir el evento con su payload tal cual. */
async function emitir(evento: string, payload: unknown) {
  const oidos = oyentes.get(evento) ?? [];
  expect(oidos.length, `nadie escucha «${evento}»: el test emitiría al vacío`).toBeGreaterThan(0);
  await act(async () => {
    for (const oyente of oidos) oyente(payload);
  });
}

/** La banda dentro de Tauri, con la suscripción ya montada. */
async function laBanda() {
  const pintada = render(
    <IdiomaContext.Provider value="es">
      {/* «esperando» es lo que la URL pide; dentro del producto manda la ficha, y eso es
          justamente lo que este test comprueba. */}
      <Banda estado="esperando" ampliada />
    </IdiomaContext.Provider>,
  );
  // Los efectos de montaje ya registraron las suscripciones; este giro de bucle deja que las
  // preguntas al puente se resuelvan antes de la primera aserción.
  await act(async () => {});
  return pintada;
}

const banda = () => document.querySelector("section.banda") as HTMLElement;

beforeEach(() => {
  oyentes.clear();
  vi.mocked(preguntar).mockReset();
  // Sin turnos y sin ficha a mano, salvo donde el test diga otra cosa.
  vi.mocked(preguntar).mockResolvedValue(null);
});

describe("la ficha, dentro del producto", () => {
  it("una aparición del evento «escucha» se pinta en la banda", async () => {
    await laBanda();
    expect(banda().dataset.estado).toBe("esperando");

    await emitir("escucha", NOVEDAD_APARECE_FICHA);

    expect(banda().dataset.estado).toBe("ficha");
    if (NOVEDAD_APARECE_FICHA.clase !== "ficha") throw new Error("la muestra dejó de ser ficha");
    expect(banda().textContent).toContain(NOVEDAD_APARECE_FICHA.titular);
    expect(banda().textContent).toContain(NOVEDAD_APARECE_FICHA.fuente.documento);
  });

  it("cuando no hay nada, la banda dice qué se buscó en vez de quedarse esperando", async () => {
    await laBanda();
    await emitir("escucha", NOVEDAD_APARECE_SIN_RESULTADO);

    expect(banda().dataset.estado).toBe("sin-resultado");
    if (NOVEDAD_APARECE_SIN_RESULTADO.clase !== "sinResultado") {
      throw new Error("la muestra dejó de ser «sin resultado»");
    }
    expect(banda().textContent).toContain(NOVEDAD_APARECE_SIN_RESULTADO.buscado);
  });

  it("el turno del cliente abre el «buscando» antes de que haya respuesta", async () => {
    await laBanda();
    await emitir("escucha", NOVEDAD_TURNO);
    expect(banda().dataset.estado).toBe("buscando");

    // Y la respuesta lo cierra, aunque sea para decir que no hay nada.
    await emitir("escucha", NOVEDAD_APARECE_SIN_RESULTADO);
    expect(banda().dataset.estado).toBe("sin-resultado");
  });

  /** El camino del atajo era el ÚNICO que funcionaba en el binario del sprint, porque va por su
   *  propio evento y por `pedir_ficha`. Que siga funcionando se prueba aquí, con el payload que
   *  ese comando devuelve de verdad. */
  it("⌘⇧A pide la ficha y la pinta", async () => {
    vi.mocked(preguntar).mockResolvedValue(APARICION_DEL_ATAJO);
    await laBanda();

    await emitir("ficha", null);
    await act(async () => {});

    expect(banda().dataset.estado).toBe("ficha");
    if (APARICION_DEL_ATAJO.clase !== "ficha") throw new Error("la muestra dejó de ser ficha");
    expect(banda().textContent).toContain(APARICION_DEL_ATAJO.titular);
    expect(vi.mocked(preguntar)).toHaveBeenCalledWith("pedir_ficha");
  });

  it("tras el corte no queda ficha en pantalla", async () => {
    await laBanda();
    await emitir("escucha", NOVEDAD_APARECE_FICHA);
    expect(banda().dataset.estado).toBe("ficha");

    await emitir("corte", null);
    expect(banda().dataset.estado).toBe("esperando");
  });
});
