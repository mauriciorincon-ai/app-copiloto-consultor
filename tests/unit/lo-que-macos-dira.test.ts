import { existsSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { es } from "../../src/i18n/es";
import { en } from "../../src/i18n/en";

/**
 * Gate del texto de los permisos: **lo que la pantalla promete que macOS dirá tiene que ser lo que
 * macOS diga**.
 *
 * La pantalla de Permisos tiene una tarjeta titulada «Qué texto verás en macOS» y debajo, entre
 * comillas, la frase exacta del diálogo del sistema. Esa frase vive en dos sitios que nada obliga
 * a mantener iguales: el diccionario de la interfaz (`src/i18n/`) y el `Info.plist` que lee macOS.
 * Cambiar uno y olvidar el otro no rompe nada, no falla ningún test de los que había, y deja a la
 * app enseñando una promesa que el sistema no va a cumplir — en la única pantalla cuyo trabajo
 * entero es merecer confianza.
 *
 * Demo en rojo (regla 15): cambiar una palabra del `Info.plist` sin tocar el diccionario.
 *
 * Nace en la fase 3 del sprint 001, que es cuando la app empieza a pedir el micrófono de verdad.
 */
const PLIST = "src-tauri/Info.plist";
const IDIOMAS = { es: "src-tauri/lproj/es.lproj/InfoPlist.strings", en: "src-tauri/lproj/en.lproj/InfoPlist.strings" };

/** Las claves que esta app pide. Si algún día pide otra, se añade aquí y el gate la exige en todas partes. */
const CLAVES = [
  "NSMicrophoneUsageDescription",
  "NSAudioCaptureUsageDescription",
  "NSSpeechRecognitionUsageDescription",
] as const;

function delPlist(texto: string): Record<string, string> {
  const salida: Record<string, string> = {};
  const re = /<key>([^<]+)<\/key>\s*<string>([^<]*)<\/string>/g;
  for (const m of texto.matchAll(re)) salida[m[1]] = m[2];
  return salida;
}

function delStrings(texto: string): Record<string, string> {
  const salida: Record<string, string> = {};
  const re = /"([^"]+)"\s*=\s*"([^"]*)"\s*;/g;
  for (const m of texto.matchAll(re)) salida[m[1]] = m[2];
  return salida;
}

/** La maqueta escribe la frase entre comillas tipográficas; macOS la recibe sin ellas. */
const sinComillas = (s: string) => s.replace(/^[«“"']|[»”"']$/g, "").trim();

describe("lo que macOS dirá cuando pida un permiso", () => {
  const plist = delPlist(readFileSync(PLIST, "utf8"));
  const porIdioma = Object.fromEntries(
    Object.entries(IDIOMAS).map(([k, ruta]) => [k, delStrings(readFileSync(ruta, "utf8"))]),
  );

  it("el Info.plist declara las tres claves que esta app pide", () => {
    for (const clave of CLAVES) {
      expect(plist[clave], `falta ${clave} en ${PLIST}: macOS mata la app al pedir ese permiso`).toBeTruthy();
      expect(plist[clave].length, `${clave} está vacía`).toBeGreaterThan(40);
    }
  });

  it("los dos idiomas traducen exactamente las mismas claves", () => {
    for (const [idioma, ruta] of Object.entries(IDIOMAS)) {
      expect(existsSync(ruta), `falta ${ruta}`).toBe(true);
      expect(Object.keys(porIdioma[idioma]).sort()).toEqual([...CLAVES].sort());
    }
  });

  /**
   * El gate de verdad. La tarjeta «Qué texto verás en macOS» enseña esta frase palabra por
   * palabra; si el plist dice otra cosa, la pantalla miente.
   */
  it("la frase del micrófono es la misma en la pantalla y en el sistema", () => {
    expect(porIdioma.es.NSMicrophoneUsageDescription).toBe(sinComillas(es.cuaderno.textoMicrofono));
    expect(porIdioma.en.NSMicrophoneUsageDescription).toBe(sinComillas(en.cuaderno.textoMicrofono));
  });

  it("el inglés del plist es el mismo que el de su lproj", () => {
    for (const clave of CLAVES) {
      expect(plist[clave], `${clave} difiere entre Info.plist y en.lproj`).toBe(porIdioma.en[clave]);
    }
  });

  /**
   * Cada frase dice PARA QUÉ, no solo QUÉ. «Angel Ghost necesita el micrófono» no es una razón; es
   * lo que macOS ya sabe. La regla del vocabulario y la de la honestidad se juntan aquí: el
   * usuario decide con esta frase y con ninguna otra.
   */
  it("cada frase explica para qué, y promete lo mismo que la app", () => {
    for (const idioma of Object.keys(IDIOMAS)) {
      for (const clave of CLAVES) {
        const frase = porIdioma[idioma][clave];
        expect(frase, `${idioma}/${clave} no nombra la app`).toContain("Angel Ghost");
        expect(frase.split(" ").length, `${idioma}/${clave} es demasiado corta para explicar nada`).toBeGreaterThan(12);
      }
      const memoria = idioma === "es" ? "solo en memoria" : "in memory only";
      expect(porIdioma[idioma].NSMicrophoneUsageDescription).toContain(memoria);
      expect(porIdioma[idioma].NSAudioCaptureUsageDescription).toContain(memoria);
    }
  });
});
