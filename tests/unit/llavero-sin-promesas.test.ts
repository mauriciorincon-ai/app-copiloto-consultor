import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * Gate de la PROMESA DEL LLAVERO (auditoría del S3, A2; ADR 015, enmienda 2).
 *
 * `SecItemAdd` sin `kSecUseDataProtectionKeychain` guarda en el llavero de archivo, el de inicio de
 * sesión, y ese llavero ignora `kSecAttrAccessible` (Apple, TN3137): el `WhenUnlockedThisDeviceOnly`
 * del código no ata la llave a este Mac. Viaja con Time Machine y con el Asistente de migración.
 * Mientras el puente Swift no active el llavero de protección de datos **en código** —un comentario no
 * cuenta—, ningún texto de la app puede prometer lo contrario.
 *
 * ¿Puede fallar? Sí: nació en rojo, con seis textos que decían «ligada a este Mac», «ni por copia de
 * seguridad» o «cambias de Mac… no se pueden abrir». El día de la firma (G-Release, H2), con
 * `kSecUseDataProtectionKeychain: true`, el gate deja de exigir y los textos pueden volver a prometerlo.
 */
const PUENTE = "src-tauri/nativo/Llavero.swift";

// «la llave no viaja», desde la segunda pasada de la casilla 4 (2026-10-03): el §4 del ADR 015 lo
// seguía diciendo sin tachar, y el gate no lo cazaba.
const PROMESAS =
  /ligada a este Mac|tied to this Mac|ni por copia de seguridad|no viajan? en copias de seguridad|no salen? de\s+este Mac|no viaja a otro Mac|cambias de Mac|la llave no viaja|the key (does not|doesn't) travel/i;

/** Los textos que prometen algo sobre la llave. Del ADR, solo lo que va antes de la enmienda 2. */
const TEXTOS = [
  "docs/MANUAL-DE-USO.md",
  "docs/BLUEPRINT.html",
  "docs/GUIA-DE-PRUEBA.html",
  "docs/diseno/notas.html",
  "src/i18n/es.ts",
  "src/i18n/en.ts",
  "src-tauri/src/llavero.rs",
  "src-tauri/src/carpeta.rs",
  PUENTE,
  "decisions/015-las-notas-y-su-cifrado.md",
];

/** Lo que se lee: sin comentarios si es Swift (para saber qué hace el código), sin lo tachado (`~~…~~`,
 *  la historia que se corrigió a la vista) y, en el ADR, hasta la enmienda que lo corrige. */
function texto(ruta: string): string {
  let t = readFileSync(ruta, "utf8");
  if (ruta.startsWith("decisions/")) t = t.split(/^## Enmienda 2\b/m)[0];
  return t.replace(/~~[\s\S]*?~~/g, "");
}

function usaElLlaveroDeProteccionDeDatos(fuente: string): boolean {
  const codigo = fuente
    .split("\n")
    .filter((l) => !l.trim().startsWith("//"))
    .join("\n");
  return /kSecUseDataProtectionKeychain\s+as\s+String\]?\s*[:=]\s*true/.test(codigo);
}

export function promesasSinRespaldo(textos: Record<string, string>, puente: string): string[] {
  if (usaElLlaveroDeProteccionDeDatos(puente)) return [];
  return Object.entries(textos).flatMap(([ruta, t]) =>
    t
      .split("\n")
      .map((l, i) => [l, i + 1] as const)
      .filter(([l]) => PROMESAS.test(l))
      .map(([l, n]) => `${ruta}:${n} · ${l.trim().slice(0, 120)}`),
  );
}

describe("llavero-sin-promesas: ningún texto ata la llave a este Mac mientras el código no lo haga", () => {
  it("el puente Swift todavía usa el llavero de archivo (si esto cambia, el gate deja de exigir)", () => {
    expect(usaElLlaveroDeProteccionDeDatos(readFileSync(PUENTE, "utf8"))).toBe(false);
  });

  it("ningún texto promete que la llave no sale de este Mac", () => {
    const textos = Object.fromEntries(TEXTOS.map((r) => [r, texto(r)]));
    expect(promesasSinRespaldo(textos, readFileSync(PUENTE, "utf8"))).toEqual([]);
  });

  it("el rojo: una promesa plantada se ve, y un comentario en Swift no la respalda", () => {
    const plantado = { "docs/MANUAL-DE-USO.md": "La llave vive en el Llavero, ligada a este Mac." };
    const comentario = "// kSecUseDataProtectionKeychain as String: true\nq[kSecAttrAccessible as String] = x";
    expect(promesasSinRespaldo(plantado, comentario)).toHaveLength(1);
    expect(promesasSinRespaldo(plantado, "q[kSecUseDataProtectionKeychain as String] = true")).toEqual([]);
  });
});
