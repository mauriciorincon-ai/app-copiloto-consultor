import { readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Gate del ARTEFACTO DE AUDITORÍA — **un hallazgo sin sitio no es deuda, es un rumor.**
 *
 * Lo que pasó en el sprint 001 y no puede volver a pasar: el auditor entregó 21 hallazgos medios y
 * bajos, y al escribirlos en el artefacto del repo se resumieron en una línea —«M3, M5, M6, M7, M8,
 * M12, M13, M14 y B1–B7: ver el detalle en la bitácora de ajustes»— con un puntero que además
 * estaba roto: la bitácora tampoco lo tenía. **Quince hallazgos desaparecieron**, y con ellos lo
 * único que los hace pagables: dónde están. El usuario los paga todos, incluidos los bajos, y para
 * eso tienen que existir.
 *
 * Este gate exige dos cosas a todo `sprints/SPRINT_NNN-auditoria.md`:
 *
 *   1. **Cada severidad declara su cuenta y la cuenta cuadra.** `## MEDIOS (14)` obliga a catorce
 *      filas, de `M1` a `M14`. Agrupar ocho identificadores en una frase deja de ser posible.
 *   2. **Cada hallazgo dice DÓNDE**: un `archivo:línea` o al menos un archivo del repo, en su fila
 *      o dentro de su sección. La única salida es marcarlo `irrecuperable` con su razón — y eso es
 *      una confesión escrita, no un silencio.
 *
 * **La primera regla nació de un agujero de este mismo gate**, encontrado al correrlo por primera
 * vez. La versión anterior buscaba los identificadores uno a uno (`**M12**`) y comprobaba que no
 * hubiera huecos entre los que encontrara: los ocho que el sprint 001 escondió dentro de una sola
 * negrita —`**M3, M5, M6, M7, M8, M12, M13, M14**`— **le pasaron por delante sin que los viera**,
 * porque para ese gate no existían y la numeración, entre los que sí veía, era continua. Contar
 * contra la cuenta que el documento declara de sí mismo no tiene ese escape: si dice catorce, hay
 * que enseñar catorce. Y por eso el encabezado sin cuenta —`## ALTOS`— también es rojo: un
 * documento que no dice cuántos tiene no se puede cuadrar con nada.
 *
 * ¿Puede fallar? Sí, y falla: escrito contra `SPRINT_001-auditoria.md` tal como estaba, este test
 * denunció los quince. Demo en rojo registrada en la bitácora del sprint 002.
 */
const SPRINTS = "sprints";

/** Las cuatro severidades del método, por la letra con que el auditor numera sus hallazgos. */
const SEVERIDADES: Record<string, string> = {
  C: "crítico",
  A: "alto",
  M: "medio",
  B: "bajo",
};

/** Un sitio es un archivo del repo, con o sin número de línea. */
const SITIO =
  /`[^`\n]*\.(rs|ts|tsx|mjs|js|json|html|md|css|swift|toml|yaml|yml)(:\d+)?[^`\n]*`/;
const IRRECUPERABLE = /irrecuperable/i;

function artefactos(): string[] {
  return readdirSync(SPRINTS)
    .filter((n) => /^SPRINT_\d+-auditoria\.md$/.test(n))
    .map((n) => join(SPRINTS, n));
}

/**
 * Lo que un hallazgo puede ser, y el trozo de texto que le pertenece:
 *
 *  · **fila de tabla o viñeta** que EMPIEZA por su identificador — su trozo es esa línea;
 *  · **sección propia** (`### C1 · …`) — su trozo llega hasta el encabezado siguiente. Un crítico
 *    merece prosa, y obligarlo a caber en una celda sería empujarlo a resumirse, que es exactamente
 *    el defecto que este gate persigue.
 */
function hallazgos(texto: string): Map<string, string> {
  const encontrados = new Map<string, string>();
  const lineas = texto.split("\n");
  for (let i = 0; i < lineas.length; i++) {
    const fila = /^(?:\|\s*|-\s+)\*\*([CAMB]\d{1,2})\*\*/.exec(lineas[i]);
    if (fila) {
      encontrados.set(fila[1], lineas[i]);
      continue;
    }
    const seccion = /^#{2,4} +\*{0,2}([CAMB]\d{1,2})\b/.exec(lineas[i]);
    if (!seccion) continue;
    let hasta = i + 1;
    while (hasta < lineas.length && !/^#{1,4} /.test(lineas[hasta])) hasta++;
    encontrados.set(seccion[1], lineas.slice(i, hasta).join("\n"));
  }
  return encontrados;
}

const rutas = artefactos();

describe("el artefacto de auditoría dice dónde está cada hallazgo", () => {
  it("hay artefactos que revisar (un gate sobre una carpeta vacía no es un gate)", () => {
    expect(rutas.length).toBeGreaterThan(0);
  });

  it.each(rutas)(
    "%s — cada severidad declara su cuenta y la cuenta cuadra",
    (ruta) => {
      const texto = readFileSync(ruta, "utf8");
      const conFila = [...hallazgos(texto).keys()];
      const problemas: string[] = [];

      const encabezados = [...texto.matchAll(/^## +(.+)$/gm)].map((m) => m[1]);
      const deSeveridad = encabezados.filter((h) =>
        /^(CR[IÍ]TICOS?|ALTOS?|MEDIOS?|BAJOS?)\b/i.test(h),
      );
      if (deSeveridad.length === 0)
        problemas.push("el documento no tiene encabezados de severidad");

      const declaradas = new Map<string, number>();
      for (const h of deSeveridad) {
        // Un encabezado puede declarar dos: «## MEDIOS (14) y BAJOS (7)».
        for (const [, palabra, cuenta] of h.matchAll(
          /(CR[IÍ]TICOS?|ALTOS?|MEDIOS?|BAJOS?)\s*(?:\((\d+)\))?/gi,
        )) {
          const letra = palabra[0].toUpperCase();
          if (cuenta === undefined) {
            problemas.push(
              `«${palabra}» no dice cuántos hay: un encabezado sin cuenta no se puede cuadrar`,
            );
            continue;
          }
          declaradas.set(letra, Number(cuenta));
        }
      }

      for (const [letra, cuantos] of declaradas) {
        const numeros = conFila
          .filter((i) => i[0] === letra)
          .map((i) => Number(i.slice(1)));
        for (let n = 1; n <= cuantos; n++) {
          if (!numeros.includes(n))
            problemas.push(
              `falta la fila de ${letra}${n} — ${SEVERIDADES[letra]} (se declararon ${cuantos})`,
            );
        }
        const sobran = numeros
          .filter((n) => n > cuantos)
          .map((n) => `${letra}${n}`);
        if (sobran.length > 0) {
          problemas.push(
            `filas fuera de la cuenta declarada (${cuantos}): ${sobran.join(" ")}`,
          );
        }
      }

      expect(
        problemas,
        `la cuenta que el documento declara de sí mismo no cuadra con sus filas — así se perdieron ` +
          `quince hallazgos del sprint 001:\n${problemas.join("\n")}`,
      ).toEqual([]);
    },
  );

  it.each(rutas)(
    "%s — cada hallazgo dice DÓNDE, o se declara irrecuperable",
    (ruta) => {
      const texto = readFileSync(ruta, "utf8");
      const sinSitio = [...hallazgos(texto).entries()]
        .filter(([, linea]) => !SITIO.test(linea) && !IRRECUPERABLE.test(linea))
        .map(([id]) => id)
        .sort();
      expect(
        sinSitio,
        `hallazgos sin archivo:línea y sin declararse irrecuperables. Un hallazgo sin sitio no se ` +
          `puede pagar ni heredar como deuda:\n${sinSitio.join(" · ")}`,
      ).toEqual([]);
    },
  );
});
