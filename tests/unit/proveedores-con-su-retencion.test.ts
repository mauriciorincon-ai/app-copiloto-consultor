import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { EXTERNOS } from "@/ia";
import { es } from "@/i18n/es";
import { en } from "@/i18n/en";

/**
 * Gate de la RETENCIÓN DE CADA PROVEEDOR (auditoría del S3, M3; regla dura 2; ADR 011, enmienda).
 *
 * La regla dura 2 deja salir texto anonimizado solo «bajo proveedor con no-retención». Eso no se puede
 * afirmar sin haber leído qué hace cada proveedor con lo que recibe. Cada proveedor externo que la app
 * ofrece (`EXTERNOS`) tiene su fila en la tabla del ADR 011, con la fecha en que se leyó y la fuente.
 * Un proveedor nuevo sin fila pone esto en rojo.
 *
 * ¿Puede fallar? Sí: nació en rojo, sin tabla. Y su demo está abajo: un proveedor inventado no tiene fila.
 *
 * **Y la decisión del usuario se cumple (2026-09-29):** con la tabla delante, cada proveedor «se queda»
 * o «sale». El que se queda dice en IA, en los dos idiomas, cuánto guarda (`cuaderno.retencion<Nombre>`);
 * el que sale no vuelve ni a la lista de la app, ni al tipo de Rust, ni a la maqueta de IA. Nació en rojo:
 * la decisión no estaba escrita y Gemini seguía en las tres.
 */
const ADR = "decisions/011-proveedores-del-modelo-y-minimizacion.md";

function tabla(adr: string): string[] {
  const enmienda = adr.split(/^## Enmienda \(\d{4}-\d{2}-\d{2}\) — cuánto guarda cada proveedor/m)[1] ?? "";
  return enmienda.split("\n").filter((l) => l.startsWith("| **"));
}

export function sinFila(nombres: string[], filas: string[]): string[] {
  return nombres.filter(
    (nombre) =>
      !filas.some(
        (f) => f.startsWith(`| **${nombre}**`) && /\b\d{4}-\d{2}-\d{2}\b/.test(f) && /https:\/\/\S+/.test(f),
      ),
  );
}

type Veredicto = "se queda" | "sale";

/** La tabla de la decisión: `| Nombre | se queda… | … |` o `| Nombre | sale | … |`. */
export function decisiones(adr: string): Map<string, Veredicto> {
  const bloque = adr.split(/^\*\*Decisión del usuario \(\d{4}-\d{2}-\d{2}\)/m)[1] ?? "";
  const d = new Map<string, Veredicto>();
  for (const m of bloque.matchAll(/^\| ([A-Z]\w+) \| (se queda|sale)\b/gm)) d.set(m[1], m[2] as Veredicto);
  return d;
}

/** Dónde sigue viviendo un proveedor que salió. */
export function dondeSigue(nombre: string, lugares: Record<string, string>): string[] {
  return Object.entries(lugares)
    .filter(([, texto]) => new RegExp(`\\b${nombre}\\b`, "i").test(texto))
    .map(([lugar]) => lugar);
}

function lugaresDeLaApp(): Record<string, string> {
  const api = readFileSync("src-tauri/src/sintesis/api.rs", "utf8");
  const botones = [
    ...readFileSync("docs/diseno/ia.html", "utf8").matchAll(/<button[^>]*>(?:<svg.*?<\/svg>)?([^<]*)<\/button>/g),
  ].map((m) => m[1]);
  return {
    "EXTERNOS (src/ia.ts)": EXTERNOS.map((e) => `${e.id} ${e.nombre}`).join(" · "),
    "enum Externo (sintesis/api.rs)": api.match(/pub enum Externo \{[^}]*\}/)?.[0] ?? "",
    "botones de docs/diseno/ia.html": botones.join(" · "),
  };
}

describe("proveedores-con-su-retencion: cada proveedor externo dice cuánto guarda, con fuente y fecha", () => {
  const adr = readFileSync(ADR, "utf8");
  const filas = tabla(adr);

  it("el ADR 011 tiene la tabla (un gate que no lee nada no es un gate)", () => {
    expect(filas.length).toBeGreaterThanOrEqual(EXTERNOS.length);
  });

  it("cada proveedor que la app ofrece tiene fila con fecha y URL", () => {
    expect(sinFila(EXTERNOS.map((e) => e.nombre), filas)).toEqual([]);
  });

  it("la decisión del usuario está escrita, y cada proveedor que la app ofrece se queda", () => {
    const d = decisiones(adr);
    expect(d.size).toBeGreaterThanOrEqual(EXTERNOS.length);
    expect(EXTERNOS.filter((e) => d.get(e.nombre) !== "se queda").map((e) => e.nombre)).toEqual([]);
  });

  it("el que sale no vuelve ni a la app, ni a Rust, ni a la maqueta", () => {
    const lugares = lugaresDeLaApp();
    // Los tres sitios se leyeron de verdad: si una expresión deja de encontrarlos, esto lo dice.
    expect(lugares["enum Externo (sintesis/api.rs)"]).toContain("Claude");
    expect(lugares["botones de docs/diseno/ia.html"]).toContain("Claude");
    const salen = [...decisiones(adr)].filter(([, v]) => v === "sale").map(([n]) => n);
    expect(salen.length).toBeGreaterThan(0);
    expect(salen.flatMap((n) => dondeSigue(n, lugares).map((l) => `${n} en ${l}`))).toEqual([]);
  });

  it("el que se queda dice en IA, en los dos idiomas, cuánto guarda", () => {
    const falta = EXTERNOS.flatMap((e) =>
      (
        [
          ["es", es],
          ["en", en],
        ] as const
      )
        .filter(([, d]) => !String((d.cuaderno as Record<string, unknown>)[`retencion${e.nombre}`] ?? "").trim())
        .map(([lengua]) => `${e.nombre} (${lengua})`),
    );
    expect(falta).toEqual([]);
  });

  it("el rojo: un proveedor sin fila se nombra", () => {
    expect(sinFila(["Mistral"], filas)).toEqual(["Mistral"]);
    expect(sinFila(["Claude"], ["| **Claude** | 30 días | no | acuerdo | no | sin fuente | sin fecha |"])).toEqual(["Claude"]);
  });

  it("el rojo: el que salió y sigue en un sitio se nombra con el sitio", () => {
    expect(dondeSigue("Gemini", { lista: "claude Claude · gemini Gemini · groq Groq", rust: "Claude, Groq" })).toEqual([
      "lista",
    ]);
    expect(decisiones("**Decisión del usuario (2026-09-29):**\n\n| Mistral | sale | x |\n| Claude | se queda, con aviso | y |")).toEqual(
      new Map<string, Veredicto>([
        ["Mistral", "sale"],
        ["Claude", "se queda"],
      ]),
    );
  });
});
