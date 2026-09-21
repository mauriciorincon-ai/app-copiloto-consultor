import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { join, relative } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * GATE DEL CONTADOR DE RED (B2) — el cero tiene que ser estructural, no una coincidencia.
 *
 * La pantalla de honestidad enseña «0 B salieron de tu equipo en esta reunión». Ese cero vale
 * exactamente lo que valga la razón por la que es cero. Si es «nadie llamó al contador», no vale
 * nada: mañana alguien añade una llamada y el cero sigue ahí, porque el contador solo lo suma
 * quien se acuerda de sumarlo.
 *
 * Este gate lo convierte en otra cosa: **en el sprint 001 no existe código capaz de abrir una
 * conexión.** No hay cliente HTTP, no hay socket, no hay `fetch`. El cero no se mantiene por
 * disciplina sino porque no hay con qué romperlo.
 *
 * Tres barridos, por tres caminos distintos:
 *   1. **Las dependencias de Rust** — un cliente HTTP en `Cargo.toml` es una salida potencial
 *      aunque nadie la use todavía; el momento de discutirlo es cuando entra, no cuando se llama.
 *   2. **El código de Rust** — cualquier forma de abrir un socket.
 *   3. **El código del webview** — `fetch`, `WebSocket`, `sendBeacon` y compañía.
 *
 * ¿Puede fallar? Sí, por los tres lados, y se demostró: ver la bitácora del sprint.
 *
 * **Cuándo se relaja:** cuando el adaptador de LLM se encienda (sprint 2, con su ADR «código
 * primero»), este gate no se borra — se estrecha a un único módulo permitido, y el contador pasa
 * a ser lo que mide ese único camino.
 */
const RUST = "src-tauri/src";
const CARGO = "src-tauri/Cargo.toml";
const WEBVIEW = "src";

/** Una línea que contenga esta marca NOMBRA la red sin abrirla (prosa, documentación). */
const CITA = "red:cita";

/** Formas de abrir una conexión desde Rust. Se buscan como CÓDIGO, no como palabra. */
const SOCKETS_RUST: [RegExp, string][] = [
  [/\bstd::net::/, "std::net"],
  [/\bTcpStream\s*::/, "TcpStream"],
  [/\bTcpListener\s*::/, "TcpListener"],
  [/\bUdpSocket\s*::/, "UdpSocket"],
  [/\breqwest\s*::/, "reqwest"],
  [/\bureq\s*::/, "ureq"],
  [/\bhyper\s*::/, "hyper"],
  [/\bisahc\s*::/, "isahc"],
  [/\bsocket2\s*::/, "socket2"],
];

/** Clientes HTTP y de red que no pueden aparecer como dependencia directa. */
const CLIENTES = ["reqwest", "ureq", "hyper", "isahc", "surf", "curl", "socket2", "tungstenite"];

/** Formas de salir a la red desde el webview. */
const SALIDAS_WEB: [RegExp, string][] = [
  [/(?<![.\w])fetch\s*\(/, "fetch()"],
  [/\bXMLHttpRequest\b/, "XMLHttpRequest"],
  [/new\s+WebSocket\s*\(/, "new WebSocket()"],
  [/\bsendBeacon\s*\(/, "navigator.sendBeacon()"],
  [/new\s+EventSource\s*\(/, "new EventSource()"],
];

function archivos(ruta: string, ext: string[]): string[] {
  if (!existsSync(ruta)) return [];
  if (statSync(ruta).isFile()) return ext.some((e) => ruta.endsWith(e)) ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => archivos(join(ruta, n), ext));
}

function barrer(rutas: string[], patrones: [RegExp, string][]): string[] {
  const hallazgos: string[] = [];
  for (const f of rutas) {
    readFileSync(f, "utf8")
      .split("\n")
      .forEach((linea, i) => {
        if (linea.includes(CITA)) return;
        for (const [re, nombre] of patrones) {
          if (re.test(linea)) {
            hallazgos.push(`${relative(".", f)}:${i + 1}  ${nombre}  ${linea.trim().slice(0, 90)}`);
          }
        }
      });
  }
  return hallazgos;
}

describe("contador de red: en el sprint 001 no hay con qué romper el cero", () => {
  it("hay código que inspeccionar (si no, este gate no mide nada)", () => {
    expect(archivos(RUST, [".rs"]).length).toBeGreaterThan(5);
    expect(archivos(WEBVIEW, [".ts", ".tsx"]).length).toBeGreaterThan(5);
  });

  it("ninguna dependencia directa de Rust es un cliente de red", () => {
    const manifiesto = readFileSync(CARGO, "utf8");
    // Solo las líneas de dependencia: `nombre = ...` al principio de línea.
    const declaradas = manifiesto
      .split("\n")
      .map((l) => l.match(/^([A-Za-z0-9_-]+)\s*=/)?.[1])
      .filter((n): n is string => Boolean(n));
    const culpables = declaradas.filter((d) => CLIENTES.includes(d));
    expect(
      culpables,
      `clientes de red en ${CARGO}: ${culpables.join(", ")} — necesitan ADR antes de entrar`,
    ).toEqual([]);
  });

  it("ningún archivo de Rust abre un socket", () => {
    const hallazgos = barrer(archivos(RUST, [".rs"]), SOCKETS_RUST);
    expect(hallazgos, `salidas a la red en Rust:\n${hallazgos.join("\n")}`).toEqual([]);
  });

  it("ningún archivo del webview sale a la red", () => {
    const hallazgos = barrer(archivos(WEBVIEW, [".ts", ".tsx"]), SALIDAS_WEB);
    expect(hallazgos, `salidas a la red en el webview:\n${hallazgos.join("\n")}`).toEqual([]);
  });

  it("el contador tiene un solo camino de entrada", () => {
    const red = readFileSync(join(RUST, "red.rs"), "utf8");
    // `fetch_add` es la única mutación del contador, y vive dentro de `registrar_salida`.
    const sumas = red.split("\n").filter((l) => l.includes("fetch_add"));
    expect(sumas.length, "el contador se suma desde más de un sitio").toBe(1);
    // Y nadie más en el crate puede tocar el estático: es privado al módulo.
    expect(red).toMatch(/static SALIDA: AtomicU64/);
    expect(red).not.toMatch(/pub static SALIDA/);
  });
});
