import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Gate de la PROTECCIÓN DE CAPTURA — el riesgo nº 1 del sprint 001.
 *
 * La banda lleva el flag que la borra de cualquier grabación o «compartir pantalla». Su relleno
 * ocupa el mismo rectángulo y **no debe llevarlo**: es lo único que la captura encuentra ahí. Si
 * el relleno lo hereda —copiando el constructor de la banda, que es lo natural— la franja vuelve a
 * mostrar lo que hay detrás **y nadie se entera**: desde el Mac del consultor las dos versiones se
 * ven exactamente igual. El producto entero pierde sentido en silencio.
 *
 * Dos aserciones, por dos caminos distintos:
 *  1. `tauri.conf.json` declara **exactamente una** ventana protegida, y es la banda.
 *  2. **Ningún `.rs` enciende el flag por código.** El flag vive solo en la configuración y las
 *     ventanas se construyen con `from_config`: así no hay nada que heredar en un constructor.
 *
 * ¿Puede fallar? Sí, por los dos lados, y se demostró: ver la bitácora del sprint.
 * (El lado Rust tiene su propio test del invariante en `src-tauri/src/ventana/mod.rs`; este mira
 * lo que aquel no puede ver: el árbol de fuentes entero.)
 */
const CONFIG = "src-tauri/tauri.conf.json";
const FUENTES_RUST = "src-tauri/src";
const BANDA = "banda";

/**
 * Lo que se busca es la LLAMADA que enciende el flag: `content_protected(…)` o
 * `set_content_protected(…)`, con paréntesis. Nombrarlo en prosa (la documentación del módulo lo
 * hace) o **leer** el campo no cuenta — y leerlo es justo lo que hace el test del invariante del
 * lado Rust, que sería absurdo prohibir.
 */
const ENCENDIDO = /(?:\.set_)?content_protected\s*\(/;

type Ventana = { label?: string; contentProtected?: boolean };

function rust(ruta: string): string[] {
  if (!existsSync(ruta)) return [];
  if (statSync(ruta).isFile()) return ruta.endsWith(".rs") ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => rust(join(ruta, n)));
}

describe("protección de captura: exactamente una ventana, y es la banda", () => {
  const config = JSON.parse(readFileSync(CONFIG, "utf8")) as {
    app?: { windows?: Ventana[] };
  };
  const ventanas = config.app?.windows ?? [];

  it("el archivo declara ventanas que inspeccionar", () => {
    expect(ventanas.length).toBeGreaterThan(0);
    expect(ventanas.every((v) => typeof v.label === "string")).toBe(true);
  });

  it("solo la banda lleva el flag de protección", () => {
    const protegidas = ventanas.filter((v) => v.contentProtected === true).map((v) => v.label);
    expect(
      protegidas,
      `protegidas: [${protegidas.join(", ")}] — tiene que ser exactamente ["${BANDA}"]`,
    ).toEqual([BANDA]);
  });

  it("el relleno existe, ocupa el alto de la banda y NO está protegido", () => {
    const banda = ventanas.find((v) => v.label === BANDA);
    const relleno = ventanas.find((v) => v.label === "relleno");
    expect(relleno, "el relleno no está declarado: la franja mostraría lo que haya detrás").toBeDefined();
    expect(relleno?.contentProtected ?? false).toBe(false);
    expect((relleno as Record<string, unknown>)?.height).toBe(
      (banda as Record<string, unknown>)?.height,
    );
  });

  it("ningún archivo de `src-tauri/src` enciende el flag por código", () => {
    const hallazgos: string[] = [];
    for (const f of rust(FUENTES_RUST)) {
      readFileSync(f, "utf8").split("\n").forEach((linea, i) => {
        if (linea.includes("proteccion:cita")) return; // línea que CITA el nombre sin encenderlo
        if (ENCENDIDO.test(linea)) hallazgos.push(`${relative(".", f)}:${i + 1}  ${linea.trim().slice(0, 100)}`);
      });
    }
    expect(
      hallazgos,
      `el flag tiene que vivir solo en ${CONFIG}; encendido por código en:\n${hallazgos.join("\n")}`,
    ).toEqual([]);
  });
});
