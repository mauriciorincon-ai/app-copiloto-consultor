// @vitest-environment node
import { readdirSync, readFileSync, statSync } from "node:fs";
import { posix } from "node:path";
import { describe, expect, it } from "vitest";

// Las rutas del barrido van con `/` en cualquier sistema (sprint 009): se comparan con las de abajo, escritas con
// `/`, y en Windows `join` y `relative` de `node:path` devolvían barras invertidas.
const { join } = posix;

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
 * **Y la webview, desde el sprint 008** (auditoría del S5, B6): los `.ts` y `.tsx` de `src/` tampoco comparan `pista === "sistema"`,
 * `"microfono"` ni `"sala"` sin decirlo a propósito. Dos sitios lo hacían sin que nadie lo hubiera decidido —con la sala,
 * su turno no abría el «Buscando…» de la banda— y se encontraron a mano. Cada sitio que sí compara lleva, en su línea o
 * en la de arriba, la marca `pista-decidida` con su porqué.
 *
 * ¿Puede fallar? Sí: su rojo plantó `if turno.pista == Pista::Sistema` en `disparo/mod.rs` (bitácora del
 * sprint 005, fase 1); y, en la webview, quitar la marca de cualquiera de los cuatro sitios decididos (sprint 008).
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
  const archivos = rust(RAIZ).filter((f) => f !== DONDE_VIVE);

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
                `${archivo}:${i + 1} · ${que} · ${linea.trim()}`,
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

/** Los archivos de la webview. */
function webview(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const ruta = join(dir, n);
    if (statSync(ruta).isDirectory()) return n === "i18n" ? [] : webview(ruta);
    return /\.(ts|tsx)$/.test(n) && !n.endsWith(".generado.ts") ? [ruta] : [];
  });
}

const COMPARA_PISTA = /\bpista\s*[!=]==?\s*"(?:sistema|microfono|sala)"|"(?:sistema|microfono|sala)"\s*[!=]==?\s*[A-Za-z_.]*pista\b/;
const DECIDIDA = "pista-decidida";

describe("en la webview, comparar con una pista concreta se decide a propósito (S5 B6)", () => {
  const archivos = webview("src");

  it("hay webview que mirar", () => {
    expect(archivos.length).toBeGreaterThan(20);
  });

  it("cada comparación con \"sistema\", \"microfono\" o \"sala\" lleva su marca `pista-decidida`", () => {
    const fallos: string[] = [];
    for (const archivo of archivos) {
      const lineas = readFileSync(archivo, "utf8").split("\n");
      lineas.forEach((linea, i) => {
        if (linea.trim().startsWith("//") || linea.trim().startsWith("*")) return;
        if (!COMPARA_PISTA.test(linea)) return;
        const decidida = linea.includes(DECIDIDA) || (lineas[i - 1] ?? "").includes(DECIDIDA);
        if (!decidida) fallos.push(`${archivo}:${i + 1} · ${linea.trim()}`);
      });
    }
    expect(
      fallos,
      `una comparación con una pista concreta decide en silencio qué pasa con la sala; si es a propósito, ` +
        `lleva la marca \`${DECIDIDA}\` con su porqué:\n${fallos.join("\n")}`,
    ).toEqual([]);
  });

  it("el gate de verdad ve las comparaciones: encuentra los cuatro sitios decididos", () => {
    const marcados = archivos.flatMap((a) =>
      readFileSync(a, "utf8").split("\n").filter((l) => COMPARA_PISTA.test(l) && !l.trim().startsWith("//")),
    );
    expect(marcados.length, "el lector se rompió: no encontró las comparaciones conocidas").toBeGreaterThanOrEqual(4);
  });
});
