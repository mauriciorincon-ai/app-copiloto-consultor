import { existsSync, readdirSync, readFileSync } from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import {
  brochureExportSchema,
  versionCompatible,
} from "./brochure-export.schema";

/**
 * **EL EXPORT DEL BROCHURE** (`docs/brochure-export.json`, contrato v1.0.0 del portafolio): lo que la vitrina de
 * hoja-de-vida lee de esta app sin abrir el repositorio.
 *
 * Dos cosas, y las dos importan:
 *
 *   1. **Se valida con el MISMO esquema que lo va a leer** (`brochure-export.schema.ts`, copia literal del de la
 *      vitrina): un export que pasa aquí y revienta allá no le sirve a nadie.
 *   2. **Lo que el repo puede recontar, se recuenta.** El esquema valida la forma, no si es verdad: no sabe si hay
 *      20 secciones en el manual o 21 ADR. Un export con `fuente: "medido"` y una cifra vieja miente con
 *      etiqueta de honesto (app-ds lo vivió en su S6: «8 ADR» con 14, todo «medido»).
 *
 * Lo que no se recuenta, dicho: `pruebas_automaticas` y las cifras de los kits salen de una corrida de la CI con
 * su commit escrito en el detalle; recontarlas aquí sería correr la CI dentro de un test.
 *
 * ¿Puede fallar? Sí, y se vio fallar al nacer (PR «content: brochure-export inicial»): con `total` en 21, el
 * esquema de la vitrina nombró «total declarado 21 ≠ 20 features en los grupos»; y con una funcionalidad apuntando
 * a una sección que el manual no tiene, el cuadre con el manual la nombró.
 */
const EXPORT = "docs/brochure-export.json";
const MANUAL = "docs/MANUAL-DE-USO.md";

const crudo = readFileSync(EXPORT, "utf8");
const exportado = JSON.parse(crudo) as {
  _schema: Record<string, unknown>;
  app: {
    nombre: string;
    ciclo: string;
    version_repo: string;
    sprints_cerrados: number;
  };
  promesa: { tagline: string; intro: string; para_quien: string };
  funcionalidades: {
    total: number;
    fuente_del_conteo: string;
    grupos: {
      nombre: string;
      linea: string;
      features: { id: string; seccion_manual: string }[];
    }[];
  };
  metricas: {
    clave: string;
    etiqueta: string;
    unidad: string;
    valor: number;
    fuente: string;
    detalle: string;
  }[];
  stack: { nombre: string; papel: string }[];
};

const features = exportado.funcionalidades.grupos.flatMap((g) => g.features);
const metrica = (clave: string) =>
  exportado.metricas.find((m) => m.clave === clave);

/** Las secciones de «## Features» del manual, sin su origen («· Nuevo · Sprint 005», «· desde Sprint 001»). */
function seccionesDelManual(): string[] {
  const texto = readFileSync(MANUAL, "utf8");
  const desde = texto.indexOf("\n## Features\n");
  expect(desde, "el manual no tiene «## Features»").toBeGreaterThan(-1);
  const resto = texto.slice(desde + "\n## Features\n".length);
  const hasta = resto.search(/^## /m);
  return [...resto.slice(0, hasta).matchAll(/^### (.+)$/gm)].map((m) =>
    m[1].split(" · ")[0].trim(),
  );
}

/**
 * **Lo que la ficha técnica de CV Viva deriva de este export, y sus topes: el contrato entero.** CV Viva (hoja-de-vida)
 * arma con este archivo la ficha técnica de la app (`src/lib/vitrina/ficha-tecnica/armar.ts`), y su contrato
 * (`docs/contrato-ficha-tecnica/plantilla.ficha-tecnica.json`, v1.3.1; `fichaTecnicaSchema`) recorta TODO lo que
 * toma de aquí:
 *
 *   - de la app, el nombre (≤ 60), el ciclo (≤ 16) y la versión (≤ 20);
 *   - de la promesa, el tagline (≤ 80), la intro y el «para quién» (1–400);
 *   - el stack, de 1 a 8 piezas, cada una con su nombre (≤ 40) y su papel (≤ 160);
 *   - los grupos de funcionalidades, de 2 a 10, con su nombre (≤ 40) y su línea (≤ 120);
 *   - y de cada métrica, porque cualquiera puede ser una «cifra destacada», la clave (≤ 60), la etiqueta (≤ 60), la
 *     unidad (≤ 40) y el detalle (1–400).
 *
 * Con uno fuera de su tope la ficha no se arma y el build de CV Viva falla, y CV Viva no edita lo que exporta otra
 * app: los topes se vigilan aquí, en origen. El `diferencial` no viaja a la ficha y no tiene tope.
 *
 * ¿Puede fallar? Sí, y nació en rojo tres veces, a pedido de CV Viva: primero nombró los tres detalles que pasaban
 * —`pruebas_automaticas (475)`, `bytes_que_salen_del_equipo (563)`, `costo_operacion_mensual (515)`—; al cubrir la
 * promesa, `promesa.intro (528 de 400)` (PR #17); y al cerrar el contrato entero, `stack (9 de 8)`.
 */
type Tope = [campo: string, medida: number, minimo: number, maximo: number];
const largo = (campo: string, texto: string, maximo: number, minimo = 1): Tope => [
  campo,
  texto.length,
  minimo,
  maximo,
];
const TOPES_DE_LA_FICHA: Tope[] = [
  largo("app.nombre", exportado.app.nombre, 60),
  largo("app.ciclo", exportado.app.ciclo, 16),
  largo("app.version_repo", exportado.app.version_repo, 20),
  largo("promesa.tagline", exportado.promesa.tagline, 80),
  largo("promesa.intro", exportado.promesa.intro, 400),
  largo("promesa.para_quien", exportado.promesa.para_quien, 400),
  ["stack", exportado.stack.length, 1, 8],
  ...exportado.stack.flatMap((p, i) => [
    largo(`stack[${i}].nombre`, p.nombre, 40),
    largo(`stack[${i}].papel`, p.papel, 160),
  ]),
  ["funcionalidades.grupos", exportado.funcionalidades.grupos.length, 2, 10],
  ...exportado.funcionalidades.grupos.flatMap((g, i) => [
    largo(`funcionalidades.grupos[${i}].nombre`, g.nombre, 40),
    largo(`funcionalidades.grupos[${i}].linea`, g.linea, 120),
  ]),
  ...exportado.metricas.flatMap((m) => [
    largo(`metricas.${m.clave}.clave`, m.clave, 60),
    largo(`metricas.${m.clave}.etiqueta`, m.etiqueta, 60),
    largo(`metricas.${m.clave}.unidad`, m.unidad, 40, 0),
    largo(`metricas.${m.clave}.detalle`, m.detalle, 400),
  ]),
];

describe("brochure-export.json: el contrato de la vitrina", () => {
  // Los topes de la ficha van en esta prueba, no en una nueva, a propósito: `pruebas_automaticas` cuenta pruebas, y
  // una de más la dejaría desfasada.
  it("lo acepta el esquema con que lo lee la vitrina, en la versión que sabe leer, y cabe en la ficha técnica de CV Viva", () => {
    const r = brochureExportSchema.safeParse(exportado);
    expect(
      r.success,
      r.success ? "" : JSON.stringify(r.error.issues, null, 2),
    ).toBe(true);
    expect(
      versionCompatible(
        (exportado as { schema_version: string }).schema_version,
      ),
    ).toBe(true);
    const fuera = TOPES_DE_LA_FICHA.filter(
      ([, medida, minimo, maximo]) => medida < minimo || medida > maximo,
    ).map(([campo, medida, minimo, maximo]) =>
      medida > maximo
        ? `${campo} (${medida} de ${maximo})`
        : `${campo} (${medida}, mínimo ${minimo})`,
    );
    expect(
      fuera,
      `no caben en la ficha técnica de CV Viva: ${fuera.join(", ")}`,
    ).toEqual([]);
  });

  it("el total es el conteo del manual: una funcionalidad por sección de «Features», ninguna de más", () => {
    const secciones = seccionesDelManual();
    expect(
      secciones.length,
      "el manual no tiene secciones de funcionalidades",
    ).toBeGreaterThan(0);
    expect(exportado.funcionalidades.fuente_del_conteo).toBe(MANUAL);
    expect(
      features.map((f) => f.seccion_manual).sort(),
      "cada sección del manual tiene su funcionalidad aquí, y cada funcionalidad su sección",
    ).toEqual([...secciones].sort());
    expect(exportado.funcionalidades.total).toBe(secciones.length);
    expect(
      new Set(features.map((f) => f.id)).size,
      "dos funcionalidades con el mismo id",
    ).toBe(features.length);
  });

  it("las cuatro métricas obligatorias están, y los bytes que salen son una medida", () => {
    for (const clave of [
      "funcionalidades",
      "pruebas_automaticas",
      "bytes_que_salen_del_equipo",
      "costo_operacion_mensual",
    ]) {
      expect(metrica(clave), `falta la métrica «${clave}»`).toBeDefined();
    }
    expect(metrica("bytes_que_salen_del_equipo")?.fuente).toBe("medido");
    expect(metrica("funcionalidades")?.valor).toBe(
      exportado.funcionalidades.total,
    );
  });

  it("lo que se recuenta desde el repo cuadra: versión, sprints cerrados y ADR", () => {
    const pkg = JSON.parse(readFileSync("package.json", "utf8")) as {
      version: string;
    };
    expect(exportado.app.version_repo).toBe(pkg.version);
    // Hasta el sprint 5 el cierre era un `summary` con `status: closed`; desde el 6 es el cierre ejecutivo, con `closed: <fecha>`
    // en su frontmatter (`/cierre-sprint`: «un solo archivo»).
    const cerrados = readdirSync("sprints")
      .filter((n) => /^SPRINT_\d{3}-(summary|cierre)\.md$/.test(n))
      .filter((n) => {
        const texto = readFileSync(join("sprints", n), "utf8");
        return /^status: closed$/m.test(texto) || /\bclosed: \d{4}-\d{2}-\d{2}\b/.test(texto.slice(0, 600));
      });
    expect(exportado.app.sprints_cerrados).toBe(cerrados.length);
    const adrs = readdirSync("decisions").filter((n) =>
      /^\d{3}-.+\.md$/.test(n),
    );
    expect(metrica("decisiones_registradas")?.valor).toBe(adrs.length);
  });

  it("los archivos y las pruebas que citan las métricas existen", () => {
    const rutas = new Set(
      exportado.metricas.flatMap((m) =>
        [
          ...m.detalle.matchAll(
            /\b(?:src-tauri|src|tests|docs|decisions)\/[\w./-]+\.(?:rs|ts|html|md|json)\b/g,
          ),
        ].map((r) => r[0]),
      ),
    );
    expect(
      rutas.size,
      "ninguna métrica cita un archivo: este barrido no mide nada",
    ).toBeGreaterThan(0);
    for (const ruta of rutas)
      expect(existsSync(ruta), `${ruta} no existe`).toBe(true);
    expect(readFileSync("src-tauri/tests/puerta.rs", "utf8")).toContain(
      "fn el_contador_de_red_no_se_mueve(",
    );
  });

  it("cero enlaces: ni una URL en todo el export", () => {
    expect(crudo).not.toMatch(
      /https?:\/\/|www\.|vercel[.]app|workers[.]dev|pages[.]dev/,
    );
  });
});

/**
 * **Las copias, contra sus originales**, cuando esos repos están en el Mac: el esquema (de hoja-de-vida) y el bloque
 * `_schema` (de app-ds, que se copia tal cual). En la CI esos repos no están y estas dos pruebas se saltan: es una
 * comprobación local, dicha aquí para que nadie la cuente como gate de la CI.
 */
const VITRINA = join(
  homedir(),
  "Code/app-hoja-de-vida/src/lib/vitrina/schemas.ts",
);
const APP_DS = join(homedir(), "Code/app-ds/docs/brochure-export.json");

describe("las copias del contrato, contra sus originales (solo en local)", () => {
  it.skipIf(!existsSync(VITRINA))(
    "el esquema es el de la vitrina, sin tocar",
    () => {
      const copia = readFileSync(
        "tests/unit/brochure-export.schema.ts",
        "utf8",
      );
      expect(copia.slice(copia.indexOf('import { z } from "zod";'))).toBe(
        readFileSync(VITRINA, "utf8"),
      );
    },
  );

  it.skipIf(!existsSync(APP_DS))(
    "el bloque _schema es el de app-ds, tal cual",
    () => {
      const original = JSON.parse(readFileSync(APP_DS, "utf8")) as {
        _schema: unknown;
      };
      expect(exportado._schema).toEqual(original._schema);
    },
  );
});
