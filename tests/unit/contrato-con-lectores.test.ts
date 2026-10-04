import { readFileSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * Gate de los **CAMPOS DEL CONTRATO SIN CONSUMIDOR** — lo que el gate de contrato no puede ver.
 *
 * El sprint 001 nació el gate de la costura (`src-tauri/src/contrato.rs` → `src/contrato.generado.ts`)
 * porque un enum etiquetado de dos maneras distintas dejó la ficha automática sin llegar nunca a la
 * banda. Ese gate compara **la FORMA** de lo que cruza, y su propia documentación dice lo que no
 * cubre: *«que alguien LEA los campos. Un campo puede encajar perfectamente y no tener un solo
 * consumidor — la auditoría contó diecisiete así»*. Esto es ese otro lado.
 *
 * **Por qué merece un gate y no una casilla.** La comprobación existe en `/audita-sprint` (casilla 5)
 * y se corre a mano una vez por sprint, al final. Un campo huérfano es un motor probado que no es un
 * producto probado: *el defecto vive en el cable, no en la pieza*. Y los diecisiete del sprint 001
 * salieron a la luz **con el gate de la forma en verde y 153 tests pasando**.
 *
 * **La lista de abajo es la deuda, y está escrita porque el usuario los paga todos, hasta los bajos**
 * (regla 20 de la casa). Cada huérfano lleva su `archivo:línea` y **la fase en que se paga**: un
 * hallazgo sin sitio no es deuda, es un rumor.
 *
 * **El gate falla en los dos sentidos, y eso es a propósito:**
 *  · si aparece un huérfano nuevo, hay que cablearlo o declararlo — no se puede colar;
 *  · si uno se paga y se olvida borrarlo de la lista, también falla. Una lista de deuda que
 *    sobrevive a su deuda es peor que no tenerla: dice que queda trabajo donde no queda.
 *
 * ¿Puede fallar? Sí, y falla: quitarle una línea a `DEUDA` o cablear un campo sin sacarlo de ahí lo
 * pone en rojo por los dos lados. Su rojo está registrado en la bitácora del sprint 002.
 */
// Con los tres del sprint 002 (auditoría del S2, M6): la sugerencia, el estado de la IA, el radar y
// el acople vivían en archivos que este gate no leía, y para sus 32 campos no podía fallar.
// `notas.ts` desde la fase 1 del sprint 003 (C9): tus notas cruzan por cuatro comandos. `puerta.ts`
// desde la fase 4 (C16): la vista de la puerta cruza por tres comandos y un evento.
const DECLARACIONES = [
  "src/cuaderno.ts",
  "src/ficha.ts",
  "src/radar.ts",
  "src/ia.ts",
  "src/acople.ts",
  "src/notas.ts",
  "src/jurisdiccion.ts",
  "src/puerta.ts",
  "src/franja.ts",
];
const FIXTURE = "src/contrato.generado.ts";

/**
 * Campos que se leen por CLAVE CALCULADA —`p.ve[idioma]`—, que un acceso por nombre no ve. Cada uno
 * con su razón; la lista es corta a propósito.
 */
const LEIDOS_POR_CLAVE: Record<string, string> = {
  "Bilingue.es": "`p.ve[idioma]` y `p.alcance[idioma]` en la banda y Sesión: el idioma de la interfaz elige el campo",
  "Bilingue.en": "ídem",
  "IdiomasDePista.consultor":
    "`elegidos[cual]` en Idioma: la pista elige el campo. Desde el S3 «Iniciar sesión» no los manda (los lee Rust)",
};

/** Tipos que viven SOLO en la interfaz: no cruzan la costura y no les toca esta regla. */
const NO_CRUZAN = new Set(["LoQueLaBandaEnseña", "RadarEnLaBanda"]);

/** Los campos que eligen la variante de una unión. Se leen comparándolos, no accediendo. */
const DISCRIMINANTES = new Set(["que", "clase", "estado", "salida"]);

/**
 * **La deuda, campo por campo.** `tipo.campo` → por qué no tiene lector y cuándo se paga.
 *
 * Ninguno de estos se puede cablear sin escribir copy que **la maqueta no tiene**, y en esta casa el
 * copy nuevo pasa por una mirada del usuario antes de existir (regla 10). Por eso la fase de pago no
 * es «cuando haya tiempo» sino la fase de su mirada.
 */
const DEUDA: Record<string, string> = {
  // ── Pagada entera en la fase 3 del sprint 002 ──
  //
  // Los veintiún campos que esta lista llevaba —las miradas 17, 17-bis y 17-quater— se pagaron así:
  // diecisiete ganaron lector (Sesión, Idioma, Corpus, Honestidad y la banda) y **seis salieron del
  // contrato**: los cuatro que el usuario decidió no enseñar («4 fuera, 3 se ven») y dos que
  // cruzaban DOS VECES el mismo dato (`EstadoDeEscucha.motor` y `Disponibilidad.motivo`, que ya
  // cruzan por `QueSabeTranscribir`). Esos dos los cazó una lectura a mano, no este gate: los daba
  // por leídos porque compara por NOMBRE y otro tipo tenía un campo que se llamaba igual. Es la
  // limitación que su propia bitácora declara, vista en acción.

  // ── Y el último, pagado en la auditoría del S2 (B9) ──
  //
  // `InformeDelCorte.bytesEnRed` se declaraba aquí como «lo lee Rust, no se paga»: un tercer estado
  // que la regla 20 no tiene. Lo que lee solo Rust no cruza: `#[serde(skip)]`, y la lista queda vacía.

  // ── Sprint 003, fase 0: B37 cruzó antes que su vista; pagada en la fase 1 ──
  //
  // Siete campos esperaron aquí a la mirada 19 y los pagó la vista «Lo que salió» de IA. Los
  // veinticinco de las notas, que entraron en la fase 1 antes que su pantalla, los pagó la pantalla de
  // Notas en la misma fase; y los de `ContenidoDeReunion` salieron del contrato: el contenido
  // descifrado de una reunión no cruza al webview (la maqueta no tiene «abrir»). `hora`, `externo` y
  // `Trozo.texto` los lee la misma vista, aunque el gate no pueda distinguirlos por su nombre.

  // ── Sprint 003, fase 2: las propuestas cruzaron antes que su pantalla; pagada en la misma fase ──
  //
  // `propuestas` y `sinDecidir` los leen «durante» y la franja de «al cerrar» de Notas.
  // `bytesPropuestas` salió del contrato —y del cuaderno—: «Se va a guardar» no tiene fila de
  // propuestas en la maqueta (la franja dice «Las 3 que guardaste van en tu archivo»), y un campo que
  // nadie pinta no cruza. `EstadoDeLaBandeja.bandejas` nació y salió en la misma fase, por lo mismo.
};

/** Cada `export type X = … { … }`, con las variantes de una unión incluidas. */
function tipos(
  texto: string,
): { tipo: string; desde: number; cuerpo: string }[] {
  const salida: { tipo: string; desde: number; cuerpo: string }[] = [];
  for (const m of texto.matchAll(/export type (\w+)\s*=/g)) {
    let i = (m.index ?? 0) + m[0].length;
    for (;;) {
      const abre = texto.indexOf("{", i);
      if (abre < 0) break;
      // Un `;` antes de la llave significa que el tipo ya terminó: la llave es de otra cosa.
      if (texto.slice(i, abre).includes(";")) break;
      let hondo = 0;
      let j = abre;
      for (; j < texto.length; j++) {
        if (texto[j] === "{") hondo++;
        else if (texto[j] === "}" && --hondo === 0) break;
      }
      salida.push({
        tipo: m[1],
        desde: texto.slice(0, abre).split("\n").length,
        cuerpo: texto.slice(abre, j + 1),
      });
      // Si lo que sigue es `|`, es otra variante de la MISMA unión: se sigue leyendo. Entre la llave
      // y la barra puede haber una intersección cerrada —`({ que: "turno" } & Turno)`— y comentarios:
      // la primera versión se cortaba ahí y no veía las variantes de `Novedad` que seguían (M6).
      const resto = texto.slice(j + 1);
      if (!/^\s*\)?\s*(?:&\s*\w+\s*\)?)?\s*(?:(?:\/\/[^\n]*\n|\/\*[\s\S]*?\*\/)\s*)*\|/.test(resto)) break;
      i = j + 1 + resto.indexOf("|");
    }
  }
  return salida;
}

function campos(): Map<string, string> {
  const m = new Map<string, string>();
  for (const f of DECLARACIONES) {
    const texto = readFileSync(f, "utf8");
    for (const { tipo, desde, cuerpo } of tipos(texto)) {
      if (NO_CRUZAN.has(tipo)) continue;
      cuerpo.split("\n").forEach((linea, k) => {
        const t = linea.trim();
        if (t.startsWith("*") || t.startsWith("//") || t.startsWith("/*"))
          return;
        for (const c of linea.matchAll(/[{;,]?\s*([a-zA-Z]\w*)\??:/g)) {
          if (DISCRIMINANTES.has(c[1])) continue;
          const clave = `${tipo}.${c[1]}`;
          if (!m.has(clave)) m.set(clave, `${f}:${desde + k}`);
        }
      });
    }
  }
  return m;
}

function fuentes(ruta = "src"): string[] {
  if (statSync(ruta).isFile())
    return /\.tsx?$/.test(ruta) && ruta !== FIXTURE ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => fuentes(join(ruta, n)));
}

const CODIGO = fuentes().map((f) => ({
  f,
  lineas: readFileSync(f, "utf8").split("\n"),
}));

/**
 * Un LECTOR es un acceso a propiedad (`estado.bytes`) o una desestructuración que declara
 * (`const { bytes } = …`). **Enviar `{ carpeta }` como argumento NO es leer** — es escribir, y
 * confundir las dos cosas fue la primera versión de este gate: daba por consumido el único campo de
 * `EstadoDelCorpus` que nadie mira, porque lo veía viajando hacia Rust.
 */
function lector(campo: string): string | null {
  const acceso = new RegExp(`\\.${campo}\\b`);
  const desestructura = new RegExp(
    `(?:const|let)\\s*\\{[^}]*\\b${campo}\\b[^}]*\\}\\s*=`,
  );
  for (const { f, lineas } of CODIGO) {
    for (let i = 0; i < lineas.length; i++) {
      const t = lineas[i].trim();
      if (t.startsWith("*") || t.startsWith("//")) continue;
      if (acceso.test(lineas[i]) || desestructura.test(lineas[i]))
        return `${f}:${i + 1}`;
    }
  }
  return null;
}

const DECLARADOS = campos();
const HUERFANOS = [...DECLARADOS.keys()]
  .filter((c) => !(c in LEIDOS_POR_CLAVE))
  .filter((c) => !lector(c.split(".")[1]))
  .sort();

describe("todo campo del contrato tiene un lector, o está declarado como deuda con su sitio", () => {
  it("hay contrato que revisar (un gate sobre cero campos no es un gate)", () => {
    expect(DECLARADOS.size).toBeGreaterThan(40);
  });

  it("ningún huérfano se cuela sin declararse", () => {
    const sinDeclarar = HUERFANOS.filter((c) => !(c in DEUDA)).map(
      (c) => `${c}  (${DECLARADOS.get(c)})`,
    );
    expect(
      sinDeclarar,
      `campos que cruzan la costura y nadie lee, sin una línea en DEUDA que diga por qué y cuándo ` +
        `se pagan:\n  ${sinDeclarar.join("\n  ")}\n` +
        `Cablea el campo, sácalo del contrato, o escríbelo en DEUDA con su fase.`,
    ).toEqual([]);
  });

  it("ninguna deuda sobrevive a su pago", () => {
    const yaPagados = Object.keys(DEUDA)
      .filter((c) => !HUERFANOS.includes(c))
      .map((c) => `${c} — ya tiene lector: ${lector(c.split(".")[1])}`);
    expect(
      yaPagados,
      `DEUDA declara campos que YA tienen lector. Una lista de deuda que sobrevive a su deuda ` +
        `miente sobre lo que queda por hacer:\n  ${yaPagados.join("\n  ")}`,
    ).toEqual([]);
  });

  it("cada línea de la deuda dice dónde se paga (regla 20: nada por conteo)", () => {
    const mudas = Object.entries(DEUDA)
      .filter(([, razon]) => !/fase \d/.test(razon))
      .map(([c]) => c);
    expect(
      mudas,
      `deuda sin fase de pago asignada: ${mudas.join(" · ")}`,
    ).toEqual([]);
  });
});
