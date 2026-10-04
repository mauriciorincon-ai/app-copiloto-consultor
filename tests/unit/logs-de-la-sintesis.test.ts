import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * GATE — **EL LOG DE LA SÍNTESIS ES METADATA, JAMÁS CONTENIDO** (sprint 002, fase 5; DoD
 * «Observabilidad»: ni un carácter de la sugerencia en logs).
 *
 * La síntesis tiene en las manos lo más delicado de la app a la vez: el turno del cliente, las
 * fichas del consultor, la sugerencia, el texto tapado que va al API y la clave del usuario. Todo
 * eso existe como variables con nombre, y un `println!` con una de ellas dentro es un archivo de
 * log con la reunión escrita. Este gate lee cada `println!` que lleva la marca `[sintesis]` y falla
 * si sus argumentos nombran alguna de esas variables. Lo que sí puede ir: quién redactó, cuánto
 * tardó, cuántos bytes salieron, la confianza y el motivo de un descarte.
 *
 * ¿Puede fallar? Sí: se vio en rojo con un `println!("[sintesis] {turno}")` plantado en `lib.rs`
 * (bitácora del sprint 002, fase 5).
 */
const RAIZ = "src-tauri/src";
const CONTENIDO = /\b(turno|linea|titular|texto|clave|tapado|cuerpo|json|peticion|respaldo|crudo)\b/;

function archivos(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? archivos(p) : p.endsWith(".rs") ? [p] : [];
  });
}

/** Cada `println!(…)` entero, aunque ocupe varias líneas, con su archivo y su línea. */
function impresiones(): { donde: string; texto: string }[] {
  const out: { donde: string; texto: string }[] = [];
  for (const f of archivos(RAIZ)) {
    const src = readFileSync(f, "utf8");
    let i = src.indexOf("println!(");
    while (i !== -1) {
      let hondo = 0;
      let j = i + "println!".length;
      for (; j < src.length; j++) {
        if (src[j] === "(") hondo++;
        else if (src[j] === ")" && --hondo === 0) break;
      }
      const linea = src.slice(0, i).split("\n").length;
      out.push({ donde: `${f}:${linea}`, texto: src.slice(i, j + 1) });
      i = src.indexOf("println!(", j);
    }
  }
  return out;
}

describe("el log de la síntesis", () => {
  const de_la_sintesis = impresiones().filter((p) => p.texto.includes("[sintesis]"));

  it("hay líneas que leer (un gate que no lee nada no es un gate)", () => {
    expect(de_la_sintesis.length).toBeGreaterThanOrEqual(5);
  });

  it("ninguna lleva el turno, las fichas, la sugerencia, la clave ni lo que salió", () => {
    const fuera = de_la_sintesis
      .filter((p) => {
        // Se miran los ARGUMENTOS y las llaves del formato, no la prosa entre comillas.
        const sinProsa = p.texto.replace(/"(?:[^"\\]|\\.)*"/g, (m) => (m.match(/\{[^}]*\}/g) ?? []).join(" "));
        return CONTENIDO.test(sinProsa);
      })
      .map((p) => `${p.donde}  ${p.texto.replace(/\s+/g, " ").slice(0, 120)}`);
    expect(fuera, `el log de la síntesis lleva contenido:\n${fuera.join("\n")}`).toEqual([]);
  });
});

/**
 * GATE — **EL LOG DEL ENSAYO ES METADATA, JAMÁS CONTENIDO** (sprint 004, fase 3; ADR 019, regla dura 1
 * (d): «término plantado en logs»). El ensayo tiene en las manos tu respuesta en texto, las preguntas,
 * el nombre del cliente y la propuesta; ninguna de esas variables puede ir dentro de un `println!` con
 * la marca `[ensayo]`. Lo que sí: cuántas preguntas, cuántos milisegundos, cuántas letras, el idioma.
 *
 * ¿Puede fallar? Sí: se vio en rojo con un `println!("[ensayo] {cliente}")` plantado en `lib.rs`
 * (bitácora del sprint 004, fase 3).
 */
const CONTENIDO_DEL_ENSAYO =
  /\b(pregunta|respuesta|texto|tramos?|cliente|propuesta|ficha|nombre|sobre|titular|frase|peticion|json|seccion|secciones|respaldo|jerga)\b/;

describe("el log del ensayo", () => {
  const del_ensayo = impresiones().filter((p) => p.texto.includes("[ensayo]"));

  it("hay líneas que leer (un gate que no lee nada no es un gate)", () => {
    expect(del_ensayo.length).toBeGreaterThanOrEqual(8);
  });

  it("ninguna lleva tu respuesta, las preguntas, el cliente ni la propuesta", () => {
    const fuera = del_ensayo
      .filter((p) => {
        const sinProsa = p.texto.replace(/"(?:[^"\\]|\\.)*"/g, (m) => (m.match(/\{[^}]*\}/g) ?? []).join(" "));
        return CONTENIDO_DEL_ENSAYO.test(sinProsa);
      })
      .map((p) => `${p.donde}  ${p.texto.replace(/\s+/g, " ").slice(0, 120)}`);
    expect(fuera, `el log del ensayo lleva contenido:\n${fuera.join("\n")}`).toEqual([]);
  });
});
