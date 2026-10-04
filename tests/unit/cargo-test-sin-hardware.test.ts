import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * GATE `cargo-test-sin-hardware` (sprint 004, fase 0) — **la regla 25 del kit, «el comando de pruebas
 * por defecto no toca hardware»**, que en esta app vive dentro de la regla 22 («las protecciones del
 * Mac se enseñan ANTES de tocarlas»).
 *
 * El gate de verdad es el centinela en marcha (`src-tauri/src/hardware.rs`): la CI corre `cargo test` a
 * secas con `AG_SIN_HARDWARE=1` y cualquier entrada nativa abierta aborta el binario con su nombre.
 * Este archivo vigila lo que el centinela no puede ver por sí solo:
 *
 *   1. **Las marcas dicen una de tres cosas**: `hardware:` (lo corre la CI con `--include-ignored`),
 *      `en vivo:` (necesita al usuario delante; se llama `en_vivo_*` y la CI lo salta) o `medición`
 *      (no es una prueba, es una corrida para fijar un umbral). Un `#[ignore]` sin motivo es un test
 *      que nadie corre jamás.
 *   2. **En `contra-el-mac-de-verdad.rs`, todo test que llegue a una entrada nativa lleva su marca**,
 *      aunque llegue por un ayudante (`probar`, `escuchar`, `una_sesion_completa`…): la búsqueda sigue
 *      los ayudantes del archivo hasta que no aparezca ninguno nuevo.
 *   3. **El centinela está en cada entrada nativa**, y la CI la arma en el paso a secas y corre lo
 *      marcado en otro paso.
 *
 * ¿Puede fallar? Sí: nació en rojo quitando el `#[ignore]` del test del micrófono (bitácora del
 * sprint 004, fase 0). Y el centinela nació en rojo de verdad: su primera corrida tumbó un test de la
 * voz que llevaba desde el S2 encolando una frase en los altavoces del Mac.
 */
const RAIZ = "src-tauri";
const CONTRA = "src-tauri/tests/contra-el-mac-de-verdad.rs";

/** Las entradas nativas: tocarlas desde un test es tocar el Mac. Las agujas se arman por partes. */
const AGUJAS: string[] = [
  "Grifo::del_" + "microfono",
  "Grifo::del_" + "sistema",
  "motor_de_" + "la_casa(",
  "habla::" + "voz()",
  "llavero::",
  "guardar_" + "clave(",
  "desbloqueo::" + "pedir",
  "/bin/" + "launchctl",
  "pedir_" + "permiso(",
  "ax::" + "encoger(",
  "ax::" + "mover(",
  "Lectura::" + "arrancar(",
  "\"sesion_para_" + "el_log\"",
  "donde_se_" + "mira(",
];

/** Dónde vive cada entrada, y cuántas veces tiene que aparecer el centinela en ese archivo. */
const CENTINELAS: [string, number][] = [
  ["src-tauri/src/capture/nativo.rs", 2],
  ["src-tauri/src/habla/apple.rs", 1],
  ["src-tauri/src/stt/apple.rs", 2],
  ["src-tauri/src/pantalla/apple.rs", 1],
  ["src-tauri/src/acople/ax.rs", 2],
  ["src-tauri/src/llavero.rs", 5],
  ["src-tauri/src/desbloqueo.rs", 1],
  ["src-tauri/src/vencimiento/mod.rs", 1],
];

function rustDe(dir: string): string[] {
  const out: string[] = [];
  for (const n of readdirSync(dir)) {
    const p = join(dir, n);
    if (n === "target" || n === "gen") continue;
    if (statSync(p).isDirectory()) out.push(...rustDe(p));
    else if (n.endsWith(".rs")) out.push(p);
  }
  return out;
}

/** El texto de un archivo de Rust sin comentarios ni literales, con el mismo largo (para los cuerpos). */
function sinTexto(fuente: string): string {
  const s = fuente.split("");
  let i = 0;
  const blanco = (a: number, b: number) => {
    for (let k = a; k < b; k++) if (s[k] !== "\n") s[k] = " ";
  };
  while (i < s.length) {
    const c = s[i];
    const d = s[i + 1];
    if (c === "/" && d === "/") {
      const fin = fuente.indexOf("\n", i);
      const f = fin < 0 ? s.length : fin;
      blanco(i, f);
      i = f;
    } else if (c === "/" && d === "*") {
      const f = fuente.indexOf("*/", i + 2) + 2;
      blanco(i, f);
      i = f;
    } else if (c === "r" && (d === "#" || d === '"') && !/\w/.test(s[i - 1] ?? "")) {
      const m = /^r(#*)"/.exec(fuente.slice(i));
      if (!m) {
        i++;
        continue;
      }
      const cierre = '"' + m[1];
      const f = fuente.indexOf(cierre, i + m[0].length) + cierre.length;
      blanco(i, f);
      i = f;
    } else if (c === '"') {
      let k = i + 1;
      while (k < s.length && fuente[k] !== '"') k += fuente[k] === "\\" ? 2 : 1;
      blanco(i, k + 1);
      i = k + 1;
    } else if (c === "'") {
      // Un carácter ('a', '\n', '{') o una vida ('static): solo lo primero se borra.
      const m = /^'(\\.|[^\\'])'/.exec(fuente.slice(i)) ?? /^'\\u\{[0-9a-fA-F]+\}'/.exec(fuente.slice(i));
      if (m) {
        blanco(i, i + m[0].length);
        i += m[0].length;
      } else i++;
    } else i++;
  }
  return s.join("");
}

type Funcion = { nombre: string; atributos: string; cuerpo: string; linea: number };

/** Cada `fn` del archivo, con los atributos que la preceden y su cuerpo (texto original). */
function funciones(fuente: string): Funcion[] {
  const limpio = sinTexto(fuente);
  const out: Funcion[] = [];
  const re = /((?:[ \t]*#\[[^\n]*\][ \t]*\n)*)[ \t]*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?(?:unsafe\s+)?fn\s+(\w+)/g;
  let m: RegExpExecArray | null;
  while ((m = re.exec(limpio))) {
    const abre = limpio.indexOf("{", m.index + m[0].length);
    const pc = limpio.indexOf(";", m.index + m[0].length);
    if (abre < 0 || (pc >= 0 && pc < abre)) continue; // una firma sin cuerpo (trait, extern)
    let prof = 0;
    let k = abre;
    for (; k < limpio.length; k++) {
      if (limpio[k] === "{") prof++;
      else if (limpio[k] === "}" && --prof === 0) break;
    }
    out.push({
      nombre: m[2]!,
      atributos: fuente.slice(m.index, m.index + (m[1] ?? "").length),
      cuerpo: fuente.slice(abre, k + 1),
      linea: fuente.slice(0, m.index).split("\n").length + (m[1] ?? "").split("\n").length - 1,
    });
  }
  return out;
}

const MOTIVO = /#\[ignore\s*=\s*"([^"]*)"\]/;

describe("cargo test a secas no toca el Mac (regla 25 del kit · regla 22 de la app)", () => {
  const archivos = rustDe(RAIZ);

  it("cada #[ignore] dice por qué: hardware, en vivo o medición, y lo «en vivo» se llama en_vivo_*", () => {
    const malos: string[] = [];
    let cuantos = 0;
    for (const a of archivos) {
      for (const f of funciones(readFileSync(a, "utf8"))) {
        const ignorado = /#\[ignore/.test(f.atributos);
        const motivo = MOTIVO.exec(f.atributos)?.[1] ?? "";
        if (ignorado) cuantos++;
        if (ignorado && !/^(hardware:|en vivo:|medición)/.test(motivo))
          malos.push(`${a}:${f.linea} ${f.nombre}: «${motivo || "sin motivo"}»`);
        if (motivo.startsWith("en vivo:") !== f.nombre.startsWith("en_vivo_"))
          malos.push(`${a}:${f.linea} ${f.nombre}: «en vivo» y el prefijo en_vivo_ van juntos`);
      }
    }
    expect(cuantos, "no encontré ningún #[ignore]: el lector de Rust se rompió").toBeGreaterThan(10);
    expect(malos).toEqual([]);
  });

  it("en contra-el-mac-de-verdad, todo test que llega a una entrada nativa lleva su marca", () => {
    const fs = funciones(readFileSync(CONTRA, "utf8"));
    expect(fs.length, "no encontré funciones en el archivo").toBeGreaterThan(30);
    // Los ayudantes que tocan el Mac se vuelven agujas, hasta que no aparezca ninguno nuevo.
    const agujas = [...AGUJAS];
    for (let cambio = true; cambio; ) {
      cambio = false;
      for (const f of fs) {
        const llamada = `${f.nombre}(`;
        if (/#\[test\]/.test(f.atributos) || agujas.includes(llamada)) continue;
        if (agujas.some((a) => f.cuerpo.includes(a))) {
          agujas.push(llamada);
          cambio = true;
        }
      }
    }
    const sinMarca = fs
      .filter((f) => /#\[test\]/.test(f.atributos) && !/#\[ignore/.test(f.atributos))
      .filter((f) => agujas.some((a) => f.cuerpo.includes(a)))
      .map((f) => `${f.nombre} (línea ${f.linea}) llega a ${agujas.find((a) => f.cuerpo.includes(a))}`);
    expect(sinMarca).toEqual([]);
  });

  it("el centinela está en cada entrada nativa, y aborta", () => {
    const hw = readFileSync(`${RAIZ}/src/hardware.rs`, "utf8");
    expect(hw).toContain("std::process::abort()");
    expect(hw).toContain('pub const VARIABLE: &str = "AG_SIN_HARDWARE"');
    for (const [archivo, n] of CENTINELAS) {
      const veces = readFileSync(archivo, "utf8").split("crate::hardware::vigilar(").length - 1;
      expect(veces, `${archivo}: ${veces} centinelas, se esperaban ${n}`).toBe(n);
    }
  });

  it("la CI corre a secas con el centinela armado, y lo marcado en otro paso; en_vivo_ se salta", () => {
    const ci = readFileSync(".github/workflows/ci.yml", "utf8");
    const aSecas = /env: \{ AG_SIN_HARDWARE: "1" \}\n\s+run: cargo test --locked\n/;
    expect(ci).toMatch(aSecas);
    expect(ci).toContain("run: cargo test --locked -- --include-ignored --skip en_vivo_");
    const pkg = JSON.parse(readFileSync("package.json", "utf8")) as { scripts: Record<string, string> };
    expect(pkg.scripts["verify:ephemeral:runtime"]).toContain("--include-ignored");
  });
});
