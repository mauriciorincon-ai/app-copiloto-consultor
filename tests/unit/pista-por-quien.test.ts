// @vitest-environment node
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * **De quién es un turno se pregunta con `Pista::quien()`, nunca comparando pistas** (sprint 005, ADR 020).
 *
 * Hasta el sprint 005 unos veinticinco sitios de Rust preguntaban `pista == Pista::Sistema` o
 * `pista != Pista::Microfono`. Con dos pistas daba igual; con la tercera —la sala del modo presencial,
 * que no tiene dueño— esa comparación decide sola: «no es del sistema, luego es tuyo», o «no es tuyo,
 * luego es del cliente». La sala habría acabado guardada como tus turnos o propuesta como hechos del
 * cliente, y el compilador no habría dicho nada. Con `quien()` y su `match` sin comodín, sí lo dice.
 *
 * Este gate prohíbe, fuera de `src-tauri/src/capture/mod.rs` (donde vive `quien()`), tres formas de
 * comparar con una pista concreta: `== Pista::X` / `!= Pista::X` (en cualquier orden), `matches!(…, Pista::X)`
 * y un brazo de `match` `Pista::X =>`. Nombrar una pista para CONSTRUIR algo (un turno de prueba, la
 * pista que se abre) sigue permitido.
 *
 * ¿Puede fallar? Sí: su rojo plantó `if turno.pista == Pista::Sistema` en `disparo/mod.rs` (bitácora del
 * sprint 005, fase 1).
 */
const RAIZ = "src-tauri/src";
const DONDE_VIVE = "src-tauri/src/capture/mod.rs";

function rust(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const ruta = join(dir, n);
    if (statSync(ruta).isDirectory()) return rust(ruta);
    return n.endsWith(".rs") ? [ruta] : [];
  });
}

const PISTA = String.raw`(?:crate::)?(?:capture::)?Pista::(?:Microfono|Sistema|Sala)\b`;
const PROHIBIDAS: [string, RegExp][] = [
  [
    "comparación ==/!= con una pista",
    new RegExp(String.raw`(?:==|!=)\s*${PISTA}|${PISTA}\s*(?:==|!=)`),
  ],
  ["matches! con una pista", new RegExp(String.raw`matches!\([^;]*${PISTA}`)],
  [
    "brazo de match sobre una pista",
    new RegExp(String.raw`${PISTA}\s*(?:\|\s*${PISTA}\s*)*=>`),
  ],
];

describe("de quién es un turno se pregunta con quien(), no comparando pistas", () => {
  const archivos = rust(RAIZ).filter((f) => relative(".", f) !== DONDE_VIVE);

  it("hay Rust que mirar (un gate sobre una carpeta vacía no es un gate)", () => {
    expect(archivos.length).toBeGreaterThan(50);
  });

  it("fuera de capture/mod.rs nadie compara con Pista::Sistema, Pista::Microfono ni Pista::Sala", () => {
    const fallos: string[] = [];
    for (const archivo of archivos) {
      readFileSync(archivo, "utf8")
        .split("\n")
        .forEach((linea, i) => {
          if (linea.trim().startsWith("//")) return;
          for (const [que, re] of PROHIBIDAS) {
            if (re.test(linea))
              fallos.push(
                `${relative(".", archivo)}:${i + 1} · ${que} · ${linea.trim()}`,
              );
          }
        });
    }
    expect(
      fallos,
      `pregunta con \`pista.quien()\` (Tuyo · Cliente · SinAtribuir) o con \`pista.fuente()\`; comparar con una pista ` +
        `concreta decide en silencio qué pasa con la sala:\n${fallos.join("\n")}`,
    ).toEqual([]);
  });
});
