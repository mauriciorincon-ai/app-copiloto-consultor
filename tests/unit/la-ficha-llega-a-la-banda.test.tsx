import { act, render } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Banda } from "@/componentes/Banda";
import { IdiomaContext } from "@/i18n";
import { preguntar } from "@/puente";
import { es } from "@/i18n/es";
import {
  APARICION_DEL_ATAJO,
  ESTADO_DEL_CORPUS,
  NOVEDAD_APARECE_FICHA,
  NOVEDAD_APARECE_SIN_RESULTADO,
  NOVEDAD_TURNO,
  REUNION_DETECTADA,
  TURNO_DEL_CLIENTE,
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
/** Lo que cada comando contesta en este test. Lo que no esté aquí contesta `null`. */
const respuestas = new Map<string, unknown>();

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
  // **Cada comando responde lo suyo.** Un doble que contesta lo mismo a todo le daba la aparición
  // del atajo a `estado_de_la_escucha`, y la banda reventaba leyendo pistas donde había una ficha:
  // el arnés mentía sobre la forma del puente.
  preguntar: vi.fn((comando: string) => Promise.resolve(respuestas.get(comando) ?? null)),
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
  respuestas.clear();
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
    respuestas.set("pedir_ficha", APARICION_DEL_ATAJO);
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

/**
 * **A1: lo que la banda enseña dentro del producto es lo que hay.**
 *
 * Todo el sprint pintó dentro de la app los datos de la consultora inventada de la maqueta —
 * «Escuchando · 2 pistas» con el micrófono cerrado, «143 documentos» sin carpeta señalada,
 * «Páramo Azul · 12 min» sin reunión, «Meet · protegido» en una llamada de Zoom — y, lo peor, una
 * frase inventada **puesta en boca del cliente**, con hora. Nadie lo vio porque los tests corren
 * fuera de Tauri, que es justo donde esos datos SÍ van.
 */
describe("dentro del producto la banda no enseña la consultora de la maqueta", () => {
  it("sin reunión, sin corpus y sin escucha no dice «Escuchando», «143 documentos» ni «protegido»", async () => {
    await laBanda();
    const texto = banda().textContent ?? "";

    expect(texto, "dice que escucha con las dos pistas cerradas").not.toContain(es.banda.escuchando);
    expect(texto, "enseña el corpus de la maqueta").not.toContain("143");
    expect(texto, "promete protección sin reunión que proteger").not.toContain(es.banda.protegido);
    expect(texto, "enseña la reunión de la maqueta").not.toContain(es.banda.reunion);

    // Y lo que sí dice es lo que hay, que es el estado normal de una banda recién abierta.
    expect(texto).toContain(es.cuaderno.sinReunion);
    expect(texto).toContain(`0 ${es.banda.documentos}`);
  });

  it("la protección la decide el cliente detectado, no un valor por defecto", async () => {
    respuestas.set("reunion_abierta", REUNION_DETECTADA);
    await laBanda();
    if (REUNION_DETECTADA.que !== "detectada") throw new Error("la muestra dejó de ser detectada");
    expect(banda().textContent).toContain(
      `${REUNION_DETECTADA.cliente} · ${es.banda.protegidoSufijo}`,
    );
  });

  it("el corpus que enseña es el que el índice tiene", async () => {
    respuestas.set("estado_del_corpus", ESTADO_DEL_CORPUS);
    await laBanda();
    expect(banda().textContent).toContain(
      `${ESTADO_DEL_CORPUS.documentos} ${es.banda.documentos}`,
    );
  });

  it("en «buscando» no pone palabras en boca del cliente: sin turnos, sin frase", async () => {
    await laBanda();
    await emitir("escucha", NOVEDAD_TURNO);

    expect(banda().dataset.estado).toBe("buscando");
    expect(banda().textContent, "la frase inventada de la maqueta dentro del producto").not.toContain(
      es.banda.muestra.oido,
    );
  });

  it("y con turnos de verdad, enseña lo que el cliente dijo", async () => {
    respuestas.set("turnos_recientes", [TURNO_DEL_CLIENTE]);
    await laBanda();
    await emitir("escucha", NOVEDAD_TURNO);

    expect(banda().textContent).toContain(TURNO_DEL_CLIENTE.texto);
    expect(banda().textContent).toContain(`${es.banda.cliente} ${TURNO_DEL_CLIENTE.hora}`);
  });
});
