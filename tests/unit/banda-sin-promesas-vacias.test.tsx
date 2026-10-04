import { readFileSync } from "node:fs";
import { cleanup, render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { Cascara } from "@/App";
import { Banda, type EstadoBanda } from "@/componentes/Banda";

/**
 * Gate de la BANDA SIN PROMESAS VACÍAS (auditoría del S2, M12) — la superficie que el consultor usa
 * delante del cliente no anuncia lo que no hace.
 *
 * Lo que se encontró: `⌃⌥P fijar` y `⌃⌥N anotar` pintados como teclas y sin registrar, y cuatro
 * botones sin manejador («Buscar con otras palabras», «Anotar para después», «Ya lo verifiqué en
 * Zoom», «Modo solo notas»). Ninguna prueba lo veía, porque ninguna preguntaba si lo pintado
 * funciona. Aquí, en cada estado de la banda, compacta y ampliada:
 *
 *  1. toda tecla `⌃⌥X` pintada está REGISTRADA en `lib.rs`, o va marcada `pendiente` (apagada y con
 *     «todavía no» para el lector de pantalla);
 *  2. todo `<button>` tiene manejador o está `disabled`.
 *
 * ¿Puede fallar? Sí: con la banda de antes de la auditoría, las dos reglas eran rojas (bitácora).
 */
const LIB = readFileSync("src-tauri/src/lib.rs", "utf8");
const REGISTRADAS = new Set(
  [
    ...LIB.matchAll(
      /Shortcut::new\(Some\(Modifiers::CONTROL \| Modifiers::ALT\), Code::Key([A-Z])\)/g,
    ),
  ].map((m) => `⌃⌥${m[1]}`),
);

const ESTADOS: EstadoBanda[] = [
  "esperando",
  "buscando",
  "ficha",
  "sin-resultado",
  "sin-verificar",
  "ficha-pantalla",
  "radar",
  "radar-invasivo",
  "sugerencia-local",
  "sugerencia-api",
  "voz",
];

/** El manejador que React le puso al botón, si alguno. */
function tieneManejador(b: HTMLButtonElement): boolean {
  const clave = Object.keys(b).find((k) => k.startsWith("__reactProps$"));
  const props = clave
    ? (b as unknown as Record<string, { onClick?: unknown }>)[clave]
    : undefined;
  return typeof props?.onClick === "function";
}

describe("la banda no promete lo que no hace", () => {
  // Siete desde el sprint 003: ⌃⌥N (anotar) y ⌃⌥P (fijar) dejaron de ser «todavía no». Ocho desde el
  // sprint 004: ⌃⌥B lleva la banda al otro borde.
  it("las teclas que se leen en el código son las ocho de la app", () => {
    expect([...REGISTRADAS].sort()).toEqual([
      "⌃⌥A",
      "⌃⌥B",
      "⌃⌥L",
      "⌃⌥N",
      "⌃⌥P",
      "⌃⌥R",
      "⌃⌥T",
      "⌃⌥V",
    ]);
  });

  for (const estado of ESTADOS) {
    for (const ampliada of [false, true]) {
      it(`«${estado}»${ampliada ? " ampliada" : ""}: teclas registradas o pendientes, y botones que hacen algo`, () => {
        cleanup();
        const { container } = render(
          <Cascara>
            <Banda estado={estado} ampliada={ampliada} />
          </Cascara>,
        );
        const teclasFalsas = [...container.querySelectorAll("kbd")]
          .map((k) => ({ k, texto: (k.textContent ?? "").trim() }))
          .filter(({ texto }) => /^⌃⌥[A-Z]$/.test(texto))
          .filter(
            ({ k, texto }) =>
              !REGISTRADAS.has(texto) && !k.closest(".pendiente, button:disabled"),
          )
          .map(({ texto }) => texto);
        expect(teclasFalsas, "teclas pintadas que no hacen nada").toEqual([]);
        const botonesMuertos = [...container.querySelectorAll("button")]
          .filter((b) => !b.disabled && !tieneManejador(b))
          .map((b) => (b.textContent ?? "").trim());
        expect(botonesMuertos, "botones sin manejador ni disabled").toEqual([]);
      });
    }
  }
});
