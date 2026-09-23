import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { es } from "@/i18n/es";
import { en } from "@/i18n/en";

/**
 * Gate del DICCIONARIO (design-system.md §9): «la maqueta es el primer diccionario».
 *
 * Toda cadena visible del producto tiene que existir TAL CUAL en la maqueta aprobada en
 * G-Diseño. No es pedantería: si el producto puede inventar copy por su cuenta, el gate de
 * FIDELIDAD compara contra un documento que ya no describe lo que el usuario ve, y la maqueta
 * deja de ser un contrato para volverse un recuerdo.
 *
 * Cuando el producto necesite decir algo nuevo, se escribe PRIMERO en la maqueta —es una
 * decisión de diseño, va a la bitácora y a una mirada— y después aquí.
 *
 * ¿Puede fallar? Sí: ninguna regla previa compara `src/i18n/` con `docs/diseno/`. Demo en rojo
 * registrada en la bitácora del sprint.
 */
const MAQUETA = "docs/diseno";

function html(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    if (statSync(p).isDirectory()) return html(p);
    return n.endsWith(".html") ? [readFileSync(p, "utf8")] : [];
  });
}

/**
 * La maqueta escribe entidades HTML y parte el texto en `<span>`: se normaliza para comparar.
 *
 * **Lo que NO se normaliza, y es deliberado: los signos tipográficos.** La primera versión de
 * este gate convertía `’` en `'`, y eso le abrió un agujero: `the client's voice` con apóstrofo
 * recto pasaba en verde mientras la maqueta escribía `the client’s voice`. El producto renderiza
 * un glifo distinto, la pantalla deja de ser idéntica y este gate —cuyo trabajo es exactamente
 * eso— decía que sí. Lo cazó el gate de FIDELIDAD, comparando píxeles, tres pantallas después.
 *
 * Las entidades HTML sí se traducen (`&#8217;` ES el mismo carácter que `’`, escrito de otra
 * forma); lo que se compara después es carácter a carácter.
 */
function normaliza(s: string): string {
  return s
    .replace(/<[^>]+>/g, "")
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&#8217;|&rsquo;/g, "\u2019")
    .replace(/&#8220;|&ldquo;/g, "\u201c")
    .replace(/&#8221;|&rdquo;/g, "\u201d")
    .replace(/\s+/g, " ")
    .trim();
}

function cadenas(obj: object, ruta = ""): [string, string][] {
  return Object.entries(obj).flatMap(([k, v]) =>
    typeof v === "string"
      ? [[`${ruta}${k}`, v] as [string, string]]
      : cadenas(v as object, `${ruta}${k}.`),
  );
}

describe("i18n fiel a la maqueta (design-system §9)", () => {
  const corpus = normaliza(html(MAQUETA).join("\n"));

  it("lee la maqueta (un gate que no lee nada no es un gate)", () => {
    expect(corpus.length).toBeGreaterThan(10_000);
  });

  it.each([
    ["es", es],
    ["en", en],
  ])("toda cadena del diccionario %s existe tal cual en la maqueta", (_idioma, dic) => {
    const ausentes = cadenas(dic)
      .filter(([, valor]) => !corpus.includes(normaliza(valor)))
      .map(([clave, valor]) => `${clave}: «${valor}»`);
    expect(
      ausentes,
      `cadenas que el producto inventó y la maqueta no dice:\n${ausentes.join("\n")}`,
    ).toEqual([]);
  });

  it("los dos idiomas tienen exactamente las mismas claves", () => {
    expect(cadenas(en).map(([k]) => k)).toEqual(cadenas(es).map(([k]) => k));
  });

  /**
   * **Y EL AGUJERO DE CLASE QUE ESTE GATE TENÍA: solo miraba `i18n/`.**
   *
   * Un componente podía escribir copy a mano y este gate no se enteraba, porque compara el
   * diccionario con la maqueta y nunca leyó los componentes. Pasó: `Idioma.tsx` devolvía «sin
   * modelo» y «no lo reconoce» escritos en español dentro del código, así que la interfaz inglesa
   * los enseñaba en español (hallazgo A6 de la auditoría). Ocho encuadres del gate de FIDELIDAD en
   * inglés tampoco lo vieron: esos estados no se dan con los datos de muestra.
   *
   * Dos reglas, y la primera es la que caza este defecto de raíz: **ningún componente escribe a
   * mano una cadena que el diccionario ya traduce.** Si la palabra está traducida y alguien la
   * escribe igual dentro del código, esa copia no es bilingüe — da igual que la maqueta la diga en
   * español, porque la interfaz inglesa enseñará el español. No hay falsos positivos posibles: una
   * cadena igual a un valor del diccionario es siempre un defecto.
   *
   * La segunda es la de siempre, extendida a los componentes: **toda cadena de dos palabras que un
   * componente escriba a mano tiene que existir en la maqueta**, igual que las del diccionario. Lo
   * que no es copy se reconoce por su forma y se deja pasar sin lista de excepciones que mantener:
   *
   *   · **clases del design system** — cada palabra es un selector de `ghost.css` («btn mini»);
   *   · **valores de CSS** — llevan `var(…)` o una unidad («var(--borde) dashed var(--line-2)»);
   *   · **mensajes para quien programa** — viven en un `throw`, un `Error(` o un `console.`, y no
   *     los ve nunca un usuario.
   *
   * ¿Puede fallar? Sí: con el estado que tenía el repo al cerrar la construcción, esto estaba en
   * **rojo** por las dos cadenas de `Idioma.tsx`. Demo registrada en la bitácora.
   */
  const CLASES = new Set(
    [...readFileSync("docs/diseno/assets/ghost.css", "utf8").matchAll(/\.([a-z][a-z0-9-]*)/g)].map(
      (m) => m[1],
    ),
  );
  const DOS_PALABRAS = /[A-Za-zÁÉÍÓÚÑáéíóúñ]{2,}[ ][A-Za-zÁÉÍÓÚÑáéíóúñ]{2,}/;
  /**
   * Lo que el diccionario **traduce de verdad**: las claves cuyo español y cuyo inglés difieren.
   *
   * Las que coinciden en los dos idiomas quedan fuera a propósito. «Angel Ghost» y «Transcript» se
   * escriben igual en las dos lenguas, así que un `aria-label` con ese texto no rompe nada — y
   * meterlas aquí llenaba el informe de hallazgos que no lo son, que es la forma más rápida de que
   * un gate se acabe desactivando.
   */
  const TRADUCIDAS = new Map<string, string>(
    cadenas(es)
      .map(([clave, valor], i) => [clave, valor, cadenas(en)[i][1]] as const)
      .filter(([, esp, ing]) => esp !== ing)
      .flatMap(([clave, esp, ing]) => [
        [esp, clave] as [string, string],
        [ing, clave] as [string, string],
      ]),
  );

  function fuentes(dir: string): string[] {
    return readdirSync(dir).flatMap((n) => {
      const p = join(dir, n);
      if (statSync(p).isDirectory()) return fuentes(p);
      return /\.(ts|tsx)$/.test(n) && !p.includes("contrato.generado") ? [p] : [];
    });
  }

  it("ningún componente escribe copy que la maqueta no diga", () => {
    const cableadas: string[] = [];
    for (const f of fuentes("src")) {
      if (f.includes("i18n")) continue;
      const codigo = readFileSync(f, "utf8")
        .replace(/\/\*[\s\S]*?\*\//g, "")
        // Mensajes para quien programa: no los ve un usuario. Se quitan **enteros y antes de
        // partir en líneas**, porque una llamada a `Error(` puede ocupar cinco: mirando solo la
        // línea del literal, el mensaje de un `throw` de tres líneas se leía como copy de
        // producto. Se dejan sus saltos de línea para que el número de línea del informe siga
        // siendo el del archivo.
        .replace(/(?:throw new \w*Error|console\.\w+)\s*\([\s\S]*?\);/g, (t) =>
          "\n".repeat((t.match(/\n/g) ?? []).length),
        )
        .split("\n")
        .map((l) => l.replace(/\/\/.*$/, ""));
      codigo.forEach((linea, i) => {
        for (const m of linea.matchAll(/"([^"\n]{4,})"|'([^'\n]{4,})'/g)) {
          const valor = m[1] ?? m[2];
          // Lo que NO es copy, primero: una clase del design system puede llamarse igual que una
          // palabra del diccionario («unidad», «track») y no tiene nada que ver con ella.
          if (valor.split(/\s+/).every((w) => CLASES.has(w))) continue; // clases del DS
          if (/var\(|\d(px|rem|%)/.test(valor)) continue; // valores de CSS
          // **Dos palabras, las dos reglas.** Una palabra sola es casi siempre un identificador —
          // las cinco unidades del corpus se llaman igual que sus etiquetas en español («marco»,
          // «caso», «perfil»), y `Unidad` es un tipo, no copy. Lo que se pierde: una etiqueta
          // traducida de una sola palabra escrita a mano pasaría. Lo que se gana: que este gate
          // señale defectos y no ruido, que es la diferencia entre un gate vivo y uno desactivado.
          if (!DOS_PALABRAS.test(valor)) continue;
          // Regla 1: el diccionario ya lo traduce, y aquí está escrito a mano.
          const clave = TRADUCIDAS.get(valor);
          if (clave !== undefined) {
            cableadas.push(`${f}:${i + 1}  «${valor}» — el diccionario lo traduce en «${clave}»`);
            continue;
          }
          // Regla 2: prosa que nadie tradujo y la maqueta no dice.
          if (corpus.includes(normaliza(valor))) continue; // la maqueta lo dice
          cableadas.push(`${f}:${i + 1}  «${valor}»`);
        }
      });
    }
    expect(
      cableadas,
      `copy escrito a mano en un componente, que la maqueta no dice y el diccionario no traduce:\n${cableadas.join("\n")}`,
    ).toEqual([]);
  });
});
