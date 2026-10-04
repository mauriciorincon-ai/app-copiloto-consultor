import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * Gate del MANIFIESTO DE COMANDOS (auditoría del S2, M5 y B8) — **cada ventana puede solo lo suyo.**
 *
 * Sin manifiesto, Tauri deja que cualquier ventana llame a cualquier comando: la banda —encima de
 * la reunión, pintando texto de terceros— podía guardar la clave del API, y el relleno —que no
 * puede tener contenido— también. Y seis comandos estaban registrados sin un solo llamador.
 *
 * Lo que este gate exige:
 *  1. la lista de `build.rs` es exactamente la de `generate_handler!` (una sin la otra no protege);
 *  2. todo comando registrado se invoca desde `src/` (sin comandos muertos) y toda ventana lo
 *     permite solo si existe;
 *  3. el relleno puede UNO; la banda, nada de la clave, el API, el corpus ni la sesión;
 *  4. la banda puede todo lo que de verdad llama: se sigue, función por función, cada cosa que
 *     `Banda.tsx` importa hasta el literal del comando.
 *
 * ¿Puede fallar? Sí: con `allow-guardar-clave-del-api` en la banda, o quitándole `allow-pedir-ficha`,
 * es rojo (bitácora del sprint 002).
 */
const LIB = readFileSync("src-tauri/src/lib.rs", "utf8");
const BUILD = readFileSync("src-tauri/build.rs", "utf8");

function registrados(): string[] {
  const i = LIB.indexOf("generate_handler![");
  const j = LIB.indexOf("]", i);
  return LIB.slice(i + "generate_handler![".length, j)
    .split(",")
    .map((c) => c.trim())
    .filter(Boolean);
}

function delManifiesto(): string[] {
  const i = BUILD.indexOf("const COMANDOS");
  const j = BUILD.indexOf("];", i);
  return [...BUILD.slice(i, j).matchAll(/"([a-z_]+)"/g)].map((m) => m[1]);
}

function permite(ventana: "default" | "banda" | "relleno"): string[] {
  const cap = JSON.parse(
    readFileSync(`src-tauri/capabilities/${ventana}.json`, "utf8"),
  );
  return (cap.permissions as string[])
    .filter((p) => p.startsWith("allow-"))
    .map((p) => p.slice("allow-".length).replace(/-/g, "_"));
}

/** Los comandos que alcanza una función exportada de un módulo de `src/`: los literales de su cuerpo. */
function comandosDe(modulo: string, funcion: string): string[] {
  const texto = readFileSync(`src/${modulo}.ts`, "utf8");
  const m = new RegExp(`export function ${funcion}\\b[\\s\\S]*?\\n}\\n`).exec(
    texto,
  );
  if (!m) return [];
  return [...m[0].matchAll(/"([a-z]+(?:_[a-z]+)+)"/g)].map((x) => x[1]);
}

/** Lo que `Banda.tsx` importa de cada módulo que habla con Rust, función por función. */
function loQueLlamaLaBanda(): string[] {
  const banda = readFileSync("src/componentes/Banda.tsx", "utf8");
  const app = readFileSync("src/App.tsx", "utf8");
  const salida = new Set<string>();
  const imports = [
    ...(banda + app).matchAll(
      /import\s*\{([^}]*)\}\s*from\s*"\.\.?\/(cuaderno|ficha|turnos|asa|radar|acople|ia|notas|franja)"/g,
    ),
  ];
  for (const [, nombres, modulo] of imports) {
    for (const n of nombres
      .split(",")
      .map((x) => x.trim().replace(/^type\s+.*/, ""))) {
      if (!n) continue;
      for (const c of comandosDe(modulo, n)) salida.add(c);
      // Un hook que la banda usa puede devolver funciones que llaman comandos (`pedir` en `useFicha`).
    }
  }
  return [...salida].filter((c) => registrados().includes(c)).sort();
}

const SENSIBLES = [
  "guardar_clave_del_api",
  "borrar_clave_del_api",
  "api_externa",
  "redactar_sugerencias",
  "indexar_corpus",
  "elegir_carpeta",
  "instalar_idioma",
  "empezar_a_escuchar",
  "abrir_ajustes_de",
  // La banda no se cambia de borde a sí misma: lo eliges tú en Sesión o con ⌃⌥B (sprint 004).
  "fijar_posicion_de_la_banda",
  "entendido_el_aviso_de_arriba",
  // El acento del ensayo usa el proveedor externo, como redactar: solo desde IA (sprint 004).
  "enriquecer_el_banco",
  // El ensayo entero, solo desde la ventana principal (sprint 004, ADR 019): abre el micrófono y su
  // estado lleva tus respuestas en texto. La banda y el relleno no ensayan.
  "preparar_el_ensayo",
  "empezar_el_ensayo",
  "ensayo_listo",
  "ensayo_repetir",
  "ensayo_saltar",
  "ensayo_terminar",
  "ensayo_si_lo_dije",
  "estado_del_ensayo",
  "cerrar_el_ensayo",
  // Lo que queda de tus ensayos (fase 4, ADR 015 enmienda 4): abre tus ensayos con el desbloqueo de tus
  // notas, los exporta en claro o los borra. Solo la ventana principal.
  "guardar_el_ensayo",
  "exportar_el_ensayo",
  "progreso_del_ensayo",
  "borrar_los_ensayos",
];

describe("cada ventana puede solo lo suyo", () => {
  it("el manifiesto de build.rs es la lista de generate_handler!", () => {
    expect(delManifiesto().sort()).toEqual(registrados().sort());
    expect(registrados().length).toBeGreaterThan(20);
  });

  it("ningún comando registrado está muerto, y ninguna ventana permite uno que no exista", () => {
    const fuentes = [
      "cuaderno",
      "ficha",
      "turnos",
      "asa",
      "radar",
      "acople",
      "ia",
      "notas",
      "jurisdiccion",
      "puerta",
      "franja",
      "ensayo",
      "componentes/Relleno",
    ]
      .map((m) =>
        readFileSync(`src/${m}.${m.includes("/") ? "tsx" : "ts"}`, "utf8"),
      )
      .join("\n");
    const muertos = registrados().filter((c) => !fuentes.includes(`"${c}"`));
    expect(muertos, "comandos registrados que nadie llama").toEqual([]);
    for (const v of ["default", "banda", "relleno"] as const) {
      expect(
        permite(v).filter((c) => !registrados().includes(c)),
        `${v} permite comandos inexistentes`,
      ).toEqual([]);
    }
    const nadie = registrados().filter(
      (c) =>
        !["default", "banda", "relleno"].some((v) =>
          permite(v as "banda").includes(c),
        ),
    );
    expect(nadie, "comandos que ninguna ventana puede llamar").toEqual([]);
  });

  // Desde el sprint 004 el relleno puede DOS, y los dos solo leen: su fondo, y en qué borde está la
  // franja (para subir ese fondo lo que mide la barra de menús con la banda arriba). Ninguno le da
  // contenido.
  it("el relleno puede dos que solo leen, y la banda nada de la clave, el API, el corpus ni la sesión", () => {
    expect(permite("relleno")).toEqual(["fondo_del_relleno", "la_franja"]);
    expect(permite("banda").filter((c) => SENSIBLES.includes(c))).toEqual([]);
  });

  it("la banda puede todo lo que de verdad llama", () => {
    const llama = loQueLlamaLaBanda();
    expect(llama).toContain("pedir_ficha");
    expect(
      llama.filter((c) => !permite("banda").includes(c)),
      "la banda llama y no tiene permiso",
    ).toEqual([]);
  });
});
