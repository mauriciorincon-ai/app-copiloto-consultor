import { readdirSync, readFileSync, statSync, existsSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Gate de ICONOGRAFÍA (design-system.md §6 y §8 · decisión D9): la maqueta —y el producto
 * cuando exista— usa el sprite SVG propio. **Cero emojis.**
 *
 * Nace en la Fase 5 porque el recorrido se escribió declarando «cero emojis» en su propia
 * portada… con cuatro ⭐ dentro (uno en `panel.html`, tres en `posicion.html`). Un anti-patrón
 * que solo vive en un documento no es un gate: se cuela por la puerta de al lado.
 *
 * ¿Puede fallar? Sí: ninguna regla previa mira los caracteres de `docs/diseno/**` (el barrido de
 * vocabulario mira palabras; el de autocontenida, URLs). Demo en rojo registrada en
 * sprints/ETAPA-DISENO-implementation-log.md.
 */
/**
 * ALCANCE, declarado (no aflojado): solo lo que se RENDERIZA — el markup y los estilos de la
 * maqueta. Los `.md` quedan fuera a propósito: `design-system.md` y el README de diseño narran
 * el método, y el método escribe sus gates con ⭐. La regla prohíbe emojis como ICONOGRAFÍA, no
 * como notación en prosa sobre el método.
 */
const RAICES = ["docs/diseno", "src", "src-tauri/src"];
const EXT = /\.(html|css|js|ts|tsx|rs)$/;

/**
 * EXCEPCIÓN NOMINAL (declarada, no aflojada): `✓` (U+2713). El design system NOMBRA su propio
 * símbolo en prosa —«positivo = tinte 15 % + borde sólido + ✓ en círculo relleno»— en un
 * comentario de `ghost.css` y en el texto del kit. Ese carácter tipografiado no es el glifo que
 * se pinta: el glifo sigue siendo `#i-check-circle` del sprite. Se permite ESE carácter, no su
 * bloque: `✅` (U+2705), del mismo bloque Dingbats, sigue poniendo el gate en rojo.
 */
const EXCEPCIONES = ["✓"];

/**
 * Rangos de emoji (pictográficos y de presentación). NO se veta todo lo no-ASCII: la maqueta es
 * bilingüe y usa tipografía real — tildes, «comillas», guiones largos, flechas tipográficas (→),
 * símbolos de teclas (⌘ ⇧ ⌥ ⎋ ↵) y el punto medio (·) son parte del diseño, no iconografía.
 */
const EMOJI =
  /[\u{1F000}-\u{1FAFF}\u{2600}-\u{27BF}\u{2B00}-\u{2BFF}\u{FE0F}\u{1F1E6}-\u{1F1FF}]/gu;

function archivos(ruta: string): string[] {
  if (!existsSync(ruta)) return [];
  if (statSync(ruta).isFile()) return EXT.test(ruta) ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => archivos(join(ruta, n)));
}

describe("iconografía: cero emojis (design-system §6/§8)", () => {
  const lista = RAICES.flatMap(archivos);

  it("inspecciona al menos un archivo (un gate que no lee nada no es un gate)", () => {
    expect(lista.length).toBeGreaterThan(0);
  });

  it("ningún archivo de la maqueta usa emojis como iconografía", () => {
    const hallazgos: string[] = [];
    for (const f of lista) {
      readFileSync(f, "utf8")
        .split("\n")
        .forEach((linea, i) => {
          let limpia = linea;
          for (const ex of EXCEPCIONES) limpia = limpia.split(ex).join("·");
          const m = limpia.match(EMOJI);
          if (m) hallazgos.push(`${relative(".", f)}:${i + 1}  ${[...new Set(m)].join(" ")}`);
        });
    }
    expect(
      hallazgos,
      `emojis en la maqueta (usa el sprite de assets/iconos.js):\n${hallazgos.join("\n")}`,
    ).toEqual([]);
  });
});
