import { act, cleanup, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Notas } from "@/pantallas/Notas";
import { Honestidad } from "@/pantallas/Honestidad";
import { IdiomaContext } from "@/i18n";
import {
  BANDEJA_ABIERTA,
  BANDEJA_CON_LLAVE,
  ESTADO_DE_LA_ESCUCHA,
  REUNION_GUARDADA,
  REUNION_GUARDADA_PARA_SIEMPRE,
  VISTA_DEL_CUADERNO,
} from "@/contrato.generado";

/**
 * GATE `envejecimiento` (sprint 004, fase 0) — **la matriz de envejecimiento** (regla 23 del kit,
 * v1.33.0; en esta app, regla 24): todo dato con fecha de cambio de estado se pinta en cada fecha en
 * que algo cambia —hoy, cada umbral, un segundo antes y un segundo después, y +100 días— y la pantalla
 * no puede enseñar un `NaN`, un `undefined`, un `Infinity` ni una cuenta negativa, ni quejarse en la
 * consola.
 *
 * Qué tiene fecha de cambio en esta app (inventario de la regla, en el `CLAUDE.md`):
 *   - **la bandeja**: abierta → vencida, con su ventana (3 h de fábrica, techo 24 h; «al cerrar» no
 *     escribe nada), en Notas y en Honestidad;
 *   - **la retención de las notas** (7 d · 30 d · 90 d · 1 año · siempre), en el archivo de Notas.
 * La fecha «consultado» de las jurisdicciones no cambia el estado de nada (se enseña tal cual) y queda
 * fuera, dicho en la regla.
 *
 * La fecha es una perilla del test (`vi.setSystemTime`), nunca el reloj de la máquina que corre la CI.
 *
 * ¿Puede fallar? Sí: nació en rojo quitando el `Math.max(0, …)` de los días que le quedan a una reunión
 * guardada (bitácora del sprint 004, fase 0).
 */
const respuestas = new Map<string, unknown>();
vi.mock("@/puente", () => ({
  hayTauri: () => true,
  escuchar: () => () => {},
  preguntar: vi.fn((comando: string) => Promise.resolve(respuestas.get(comando) ?? null)),
  llamar: vi.fn(() => Promise.resolve(true)),
}));

const HOY = new Date("2026-10-04T10:00:00Z").getTime();
const S = Math.floor(HOY / 1000);
const DIA = 86_400;
const HORA = 3_600;

/** Las fechas en que algo cambia, relativas a un vencimiento: un segundo antes, en el acto, un segundo después. */
const BORDES = [-1, 0, 1];
const MAS_100_DIAS = 100 * DIA;

const errores: string[] = [];

beforeEach(() => {
  cleanup();
  respuestas.clear();
  errores.length = 0;
  vi.useFakeTimers({ toFake: ["Date"] });
  vi.setSystemTime(HOY);
  vi.spyOn(console, "error").mockImplementation((...a) => void errores.push(a.map(String).join(" ")));
  vi.spyOn(console, "warn").mockImplementation((...a) => void errores.push(a.map(String).join(" ")));
});
afterEach(() => {
  vi.useRealTimers();
  vi.restoreAllMocks();
});

async function pinta(pantalla: React.ReactNode, en: number): Promise<string> {
  vi.setSystemTime(en * 1000);
  const r = render(<IdiomaContext.Provider value="es">{pantalla}</IdiomaContext.Provider>);
  await act(async () => {});
  const texto = r.container.textContent ?? "";
  r.unmount();
  return texto;
}

/** Lo que una pantalla con fechas jamás debe enseñar. */
function sano(texto: string, donde: string): string[] {
  const out: string[] = [];
  for (const [re, que] of [
    [/NaN/, "NaN"],
    [/undefined/, "undefined"],
    [/Infinity/, "Infinity"],
    [/(^|[\s(«])-\d/, "una cuenta negativa"],
    // Pegada al texto de al lado («KB-100 d»): textContent junta los nodos sin espacio.
    [/\p{L}-\d+\s?(d|h|min|días|days)\b/u, "una cuenta negativa pegada"],
  ] as const)
    if (re.test(texto)) out.push(`${donde}: ${que} → «${texto.slice(Math.max(0, texto.search(re) - 30), texto.search(re) + 30)}»`);
  return out;
}

describe("la matriz de envejecimiento: cada fecha en que algo cambia de estado se pinta sin rarezas", () => {
  it("la bandeja, en Notas y en Honestidad: hoy, cada ventana, cada borde y +100 días", async () => {
    const malos: string[] = [];
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, abierta: false });
    respuestas.set("reuniones_guardadas", { reuniones: [] });
    const ventanas: [string, number][] = [["3h", 3 * HORA], ["24h", 24 * HORA], ["fin", 0]];
    let casos = 0;
    for (const [ventana, dura] of ventanas) {
      const vence = S + dura;
      const fechas = [S, ...BORDES.map((b) => vence + b), vence + MAS_100_DIAS];
      for (const en of fechas) {
        for (const bandeja of [BANDEJA_ABIERTA, BANDEJA_CON_LLAVE]) {
          respuestas.set("la_bandeja", { ...bandeja, ventana, vence });
          respuestas.set("estado_de_la_bandeja", { noCorrio: en > vence, vence });
          const donde = `${ventana} · ${bandeja === BANDEJA_ABIERTA ? "abierta" : "con llave"} · ahora ${en - vence >= 0 ? "+" : ""}${en - vence} s`;
          malos.push(...sano(await pinta(<Notas />, en), `Notas · ${donde}`));
          malos.push(...sano(await pinta(<Honestidad bytes="0 B" escucha={ESTADO_DE_LA_ESCUCHA} />, en), `Honestidad · ${donde}`));
          casos += 2;
        }
      }
    }
    expect(casos).toBe(60);
    expect(malos).toEqual([]);
    expect(errores).toEqual([]);
  });

  it("la retención de las notas guardadas: cada plazo, cada borde, siempre y +100 días", async () => {
    const malos: string[] = [];
    respuestas.set("cuaderno_de_la_reunion", { ...VISTA_DEL_CUADERNO, abierta: false });
    let casos = 0;
    for (const dias of [7, 30, 90, 365]) {
      const vence = S + dias * DIA;
      for (const en of [S, ...BORDES.map((b) => vence + b), vence + MAS_100_DIAS]) {
        respuestas.set("reuniones_guardadas", {
          reuniones: [{ ...REUNION_GUARDADA, guardada: S, vence }, { ...REUNION_GUARDADA_PARA_SIEMPRE, guardada: S }],
        });
        malos.push(...sano(await pinta(<Notas />, en), `Notas · ${dias} d · ahora ${en - vence} s`));
        casos++;
      }
    }
    expect(casos).toBe(20);
    expect(malos).toEqual([]);
    expect(errores).toEqual([]);
  });
});
