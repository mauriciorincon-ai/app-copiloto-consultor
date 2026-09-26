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
 * Este gate lo convierte en otra cosa: **el cero no se mantiene por disciplina sino porque casi no
 * hay con qué romperlo.** Ni cliente HTTP, ni socket, ni `fetch` — y la única puerta que existe de
 * verdad, la que macOS usa para descargar su modelo de voz, está declarada línea a línea y contada.
 * «Casi» es la palabra exacta, y la frase de la pantalla dice lo mismo desde la fase 2 de la
 * auditoría: antes decía «no existe código capaz de abrir una conexión», que dejó de ser cierto en
 * la fase 3 sin que nadie volviera a mirar la frase.
 *
 * Cuatro barridos, por cuatro caminos distintos:
 *   1. **Las dependencias de Rust** — un cliente HTTP en `Cargo.toml` es una salida potencial
 *      aunque nadie la use todavía; el momento de discutirlo es cuando entra, no cuando se llama.
 *   2. **El código de Rust** — cualquier forma de abrir un socket.
 *   3. **El código del webview** — `fetch`, `WebSocket`, `sendBeacon` y compañía.
 *   4. **El puente de Swift** — añadido en la FASE 2 DE LA AUDITORÍA (hallazgo A8). Faltaba, y era
 *      el peor sitio donde faltar: `src-tauri/nativo/` es **el único archivo del producto capaz de
 *      abrir una conexión** (macOS descarga su propio modelo de voz), y era el único que este gate
 *      no leía. Así que la pantalla de Honestidad afirmaba «no existe código capaz de abrir una
 *      conexión» mientras el gate que respaldaba la frase miraba hacia otro lado.
 *
 * **Y por eso este gate dejó de ser un «no hay nada» para ser un NÚMERO:** la puerta a la red
 * existe, está declarada línea a línea con su ADR, y son exactamente [`PUERTAS_DECLARADAS`]. Si
 * aparece otra, esto falla y **la frase de la pantalla hay que volver a escribirla**, que es lo que
 * de verdad importa. Un «cero» absoluto que se sostiene sobre un barrido incompleto vale menos que
 * un número exacto que se puede comprobar.
 *
 * ¿Puede fallar? Sí, por los cuatro lados, y se demostró: ver la bitácora del sprint.
 *
 * **Cuándo se relaja:** cuando el adaptador de LLM se encienda (sprint 2, con su ADR «código
 * primero»), este gate no se borra — se estrecha a un único módulo permitido, y el contador pasa
 * a ser lo que mide ese único camino.
 */
const RUST = "src-tauri/src";
const CARGO = "src-tauri/Cargo.toml";
const WEBVIEW = "src";
const NATIVO = "src-tauri/nativo";

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
  // Por `libc`, que el crate ya usa para hablar con el núcleo. Faltaba y se vio en la fase 4 del
  // sprint 002: con un `libc::socket` plantado en el radar este gate seguía en verde. Es la puerta
  // de más abajo de todas, y la que no necesita ninguna dependencia nueva para abrirse.
  [/\blibc::(socket|connect|sendto|sendmsg|getaddrinfo|gethostbyname)\b/, "libc (socket, connect, DNS)"],
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

/** Formas de salir a la red desde Swift y Objective-C. No se parecen en nada a las de Rust. */
const SALIDAS_SWIFT: [RegExp, string][] = [
  [/\bURLSession\b/, "URLSession"],
  [/\bNSURLConnection\b/, "NSURLConnection"],
  [/\bNWConnection\b/, "NWConnection"],
  [/\bCFSocket/, "CFSocket"],
  [/\bdownloadAndInstall\b/, "downloadAndInstall"],
  [/\bassetInstallationRequest\b/, "assetInstallationRequest"],
];

/**
 * Una línea autorizada: lleva su permiso **y su ADR citado en la misma línea**. En la misma línea y
 * no encima, porque un comentario de bloque protege sin querer a lo que venga detrás.
 */
const DECLARADA = /verify-ephemeral:allow\b.*ADR/;

/**
 * **Cuántas puertas a la red tiene esta app.** Dos, y son las dos líneas con que macOS descarga su
 * propio modelo de reconocimiento a petición del usuario (ADR 006). No entra audio ni sale texto:
 * solo entra el modelo.
 *
 * Es un número y no un «ninguna» porque es la verdad, y porque un número obliga a volver aquí —y a
 * la frase de la pantalla de Honestidad— el día que aparezca la tercera.
 */
const PUERTAS_DECLARADAS = 2;

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

describe("contador de red: la única puerta que hay está declarada y contada", () => {
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

  it("el puente nativo tiene EXACTAMENTE las puertas a la red que la app declara", () => {
    const swift = archivos(NATIVO, [".swift", ".m", ".mm"]);
    expect(swift.length, `no se leyó nada de ${NATIVO}: este barrido no mide nada`).toBeGreaterThan(
      0,
    );

    const sinDeclarar: string[] = [];
    let declaradas = 0;
    for (const f of swift) {
      readFileSync(f, "utf8")
        .split("\n")
        .forEach((linea, i) => {
          if (linea.includes(CITA)) return;
          const toca = SALIDAS_SWIFT.filter(([re]) => re.test(linea));
          if (toca.length === 0) return;
          if (DECLARADA.test(linea)) {
            declaradas++;
            return;
          }
          sinDeclarar.push(
            `${relative(".", f)}:${i + 1}  ${toca.map(([, n]) => n).join(", ")}  ${linea.trim().slice(0, 90)}`,
          );
        });
    }

    expect(
      sinDeclarar,
      `salidas a la red en el puente nativo sin declarar:\n${sinDeclarar.join("\n")}`,
    ).toEqual([]);
    expect(
      declaradas,
      "cambió el número de puertas a la red del puente nativo: revisa el ADR 006 y, sobre todo, " +
        "lo que la pantalla de Honestidad afirma sobre la red",
    ).toBe(PUERTAS_DECLARADAS);
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
