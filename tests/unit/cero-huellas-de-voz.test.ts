// @vitest-environment node
import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * **Cero huellas de voz, cero emociones** (regla dura 4 · estándar 4-T · ADR 020 §2).
 *
 * La atribución de hablante se resuelve por pista, nunca por la voz. En el modo presencial (sprint 005) las dos voces
 * entran por el mismo micrófono, y la tentación de «separarlas» es la que esta regla prohíbe: ni huella de tu voz
 * («registrar solo la tuya» sigue siendo una huella), ni agrupar las voces de la sala (la diarización también deriva
 * rasgos de la voz), ni inferir el estado emocional de nadie.
 *
 * Este gate barre el código de la app —Rust, el puente de Swift y la webview— y sus dependencias (los dos lockfiles,
 * por nombre de paquete) buscando **nombres de API y de librería** de identificación de hablante, de rasgos de voz y
 * de emociones. No busca la idea: la prosa que cita la regla («no identifica a nadie por su voz») no salta; un
 * `import SoundAnalysis`, sí.
 *
 * ¿Puede fallar? Sí: su rojo plantó `import SoundAnalysis` en el puente de Swift, y un paquete `pyannote-rs` en el
 * `Cargo.lock` (bitácora del sprint 005, fase 1).
 */
const CODIGO = [
  "src-tauri/src",
  "src-tauri/nativo",
  "src",
  "src-tauri/Cargo.toml",
  "package.json",
];
const EXT = /\.(rs|swift|ts|tsx|js|toml|json|h|m|mm)$/;

/** Nombres de API y de librería, con límite de palabra: «frameCapacity» no es «ecapa». */
const PROHIBIDOS: [string, RegExp][] = [
  [
    "SoundAnalysis (Apple)",
    /\b(SoundAnalysis|SNClassifySoundRequest|SNAudioStreamAnalyzer|SNAudioFileAnalyzer)\b/,
  ],
  ["MFCC (rasgos de la voz)", /\bmfccs?\b/i],
  ["diarización", /\bdiari[sz]/i],
  [
    "identificación de hablante",
    /\bspeaker[_\s-]?(embedding|verification|identification|recogni[sz]|encoder|id)s?\b/i,
  ],
  ["huella de voz", /\bvoice[_\s-]?prints?\b/i],
  ["x-vector / ECAPA", /\b(x[-_]?vectors?|ecapa)\b/i],
  [
    "librería de hablantes",
    /\b(pyannote|resemblyzer|speechbrain|wespeaker|titanet)\b/i,
  ],
  [
    "emociones por la voz",
    /\b(speech|voice)[_\s-]?emotion\b|\bemotion[_\s-]?(recogni[sz]|detect|classif)/i,
  ],
  [
    "sentimiento (NaturalLanguage)",
    /\bsentimentScore\b|\bsentiment[_\s-]?(analy[sz]|classif)/i,
  ],
];

/** Los paquetes de los lockfiles, por nombre: los hashes pueden contener cualquier cosa. */
function paquetes(): string[] {
  const cargo = readFileSync("src-tauri/Cargo.lock", "utf8").matchAll(
    /^name = "([^"]+)"/gm,
  );
  const pnpm = readFileSync("pnpm-lock.yaml", "utf8").matchAll(
    /^ {2}'?(@?[^@\s'/]+(?:\/[^@\s']+)?)@/gm,
  );
  return [...[...cargo].map((m) => m[1]), ...[...pnpm].map((m) => m[1])];
}

function archivos(ruta: string): string[] {
  if (!existsSync(ruta)) return [];
  if (statSync(ruta).isFile()) return EXT.test(ruta) ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => archivos(join(ruta, n)));
}

describe("cero huellas de voz, cero emociones (regla dura 4)", () => {
  const lista = CODIGO.flatMap(archivos);

  it("hay código que mirar, y paquetes (un gate que no lee nada no es un gate)", () => {
    expect(lista.length).toBeGreaterThan(80);
    expect(lista.some((f) => f.endsWith(".swift"))).toBe(true);
    expect(paquetes().length).toBeGreaterThan(500);
  });

  it("ningún archivo de la app usa una API de identificación de hablante, de rasgos de voz ni de emociones", () => {
    const hallazgos: string[] = [];
    for (const f of lista) {
      readFileSync(f, "utf8")
        .split("\n")
        .forEach((linea, i) => {
          for (const [que, re] of PROHIBIDOS)
            if (re.test(linea))
              hallazgos.push(
                `${relative(".", f)}:${i + 1} · ${que} · ${linea.trim()}`,
              );
        });
    }
    expect(
      hallazgos,
      `la atribución va por pista, nunca por la voz:\n${hallazgos.join("\n")}`,
    ).toEqual([]);
  });

  it("ninguna dependencia es una librería de hablantes ni de emociones", () => {
    const hallazgos = paquetes().flatMap((p) =>
      PROHIBIDOS.filter(([, re]) => re.test(p.replace(/[-_]/g, " "))).map(
        ([que]) => `${p} · ${que}`,
      ),
    );
    expect(
      hallazgos,
      `dependencia prohibida por la regla dura 4:\n${hallazgos.join("\n")}`,
    ).toEqual([]);
  });
});
