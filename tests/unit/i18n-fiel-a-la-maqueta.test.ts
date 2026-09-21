import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { es } from "@/i18n/es";
import { en } from "@/i18n/en";

/**
 * Gate del DICCIONARIO (design-system.md §9): «la maqueta es el primer diccionario».
 *
 * Toda cadena visible del producto tiene que existir TAL CUAL en la maqueta aprobada en
 * G-Diseño. No es pedantería: si el producto puede inventar copy por su cuenta, el gate de
 * FIDELIDAD compara contra un documento que ya no describe lo que el usuario ve, y la maqueta
 * deja de ser un contrato para volverse un recuerdo.
 *
 * Cuando el producto necesite decir algo nuevo, se escribe PRIMERO en la maqueta —es una
 * decisión de diseño, va a la bitácora y a una mirada— y después aquí.
 *
 * ¿Puede fallar? Sí: ninguna regla previa compara `src/i18n/` con `docs/diseno/`. Demo en rojo
 * registrada en la bitácora del sprint.
 */
const MAQUETA = "docs/diseno";

function html(dir: string): string[] {
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    if (statSync(p).isDirectory()) return html(p);
    return n.endsWith(".html") ? [readFileSync(p, "utf8")] : [];
  });
}

/**
 * La maqueta escribe entidades HTML y parte el texto en `<span>`: se normaliza para comparar.
 *
 * **Lo que NO se normaliza, y es deliberado: los signos tipográficos.** La primera versión de
 * este gate convertía `’` en `'`, y eso le abrió un agujero: `the client's voice` con apóstrofo
 * recto pasaba en verde mientras la maqueta escribía `the client’s voice`. El producto renderiza
 * un glifo distinto, la pantalla deja de ser idéntica y este gate —cuyo trabajo es exactamente
 * eso— decía que sí. Lo cazó el gate de FIDELIDAD, comparando píxeles, tres pantallas después.
 *
 * Las entidades HTML sí se traducen (`&#8217;` ES el mismo carácter que `’`, escrito de otra
 * forma); lo que se compara después es carácter a carácter.
 */
function normaliza(s: string): string {
  return s
    .replace(/<[^>]+>/g, "")
    .replace(/&nbsp;/g, " ")
    .replace(/&amp;/g, "&")
    .replace(/&lt;/g, "<")
    .replace(/&gt;/g, ">")
    .replace(/&#8217;|&rsquo;/g, "\u2019")
    .replace(/&#8220;|&ldquo;/g, "\u201c")
    .replace(/&#8221;|&rdquo;/g, "\u201d")
    .replace(/\s+/g, " ")
    .trim();
}

function cadenas(obj: object, ruta = ""): [string, string][] {
  return Object.entries(obj).flatMap(([k, v]) =>
    typeof v === "string"
      ? [[`${ruta}${k}`, v] as [string, string]]
      : cadenas(v as object, `${ruta}${k}.`),
  );
}

describe("i18n fiel a la maqueta (design-system §9)", () => {
  const corpus = normaliza(html(MAQUETA).join("\n"));

  it("lee la maqueta (un gate que no lee nada no es un gate)", () => {
    expect(corpus.length).toBeGreaterThan(10_000);
  });

  it.each([
    ["es", es],
    ["en", en],
  ])("toda cadena del diccionario %s existe tal cual en la maqueta", (_idioma, dic) => {
    const ausentes = cadenas(dic)
      .filter(([, valor]) => !corpus.includes(normaliza(valor)))
      .map(([clave, valor]) => `${clave}: «${valor}»`);
    expect(
      ausentes,
      `cadenas que el producto inventó y la maqueta no dice:\n${ausentes.join("\n")}`,
    ).toEqual([]);
  });

  it("los dos idiomas tienen exactamente las mismas claves", () => {
    expect(cadenas(en).map(([k]) => k)).toEqual(cadenas(es).map(([k]) => k));
  });
});
