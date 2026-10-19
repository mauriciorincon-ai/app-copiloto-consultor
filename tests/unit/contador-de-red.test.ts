import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { posix } from "node:path";
import { describe, expect, it } from "vitest";

// Las rutas del barrido van con `/` en cualquier sistema (sprint 009): se comparan con las de abajo, escritas con
// `/`, y en Windows `join` y `relative` de `node:path` devolvían barras invertidas.
const { join } = posix;

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
 * **Cuántas puertas a la red tiene esta app.** Tres:
 *
 * - las dos líneas con que macOS descarga su propio modelo de reconocimiento a petición del usuario
 *   (ADR 006) —no entra audio ni sale texto: solo entra el modelo—;
 * - **y la tercera, desde el sprint 002 (fase 5):** la sesión de `nativo/Red.swift` por la que va
 *   el proveedor externo de la síntesis (ADR 011). Nace apagada, necesita la clave del usuario, solo
 *   lleva texto anonimizado en el Mac y es efímera (sin caché en disco).
 *
 * Es un número y no un «ninguna» porque es la verdad, y porque un número obliga a volver aquí —y a
 * la frase de la pantalla de Honestidad— el día que aparezca otra. **La tercera obligó**: la frase de
 * Honestidad decía «la app no abre ninguna conexión» y se reescribió en la misma fase.
 */
const PUERTAS_DECLARADAS = 3;

/**
 * **Y las puertas escritas en Rust: una desde el sprint 006 (ADR 022 §4), dos del sprint 008 al 012 (ADR 023), y otra
 * vez una desde el sprint 013 (ADR 023, enmienda 1).** Windows no tiene puente de Swift: whisper necesita su modelo
 * (`modelo_whisper.rs`). La sugerencia por API salía por `sintesis/api.rs` y se retiró de Windows con el API entero.
 * La puerta vive en SU archivo, en UNA línea marcada con su ADR, y su cliente (`ureq`) solo puede declararse para
 * Windows. No se afloja nada: el resto de Rust sigue sin poder abrir un socket, y una línea de red más —en ese
 * archivo o en otro, `sintesis/api.rs` incluido— es rojo.
 */
const PUERTAS_RUST: [string, number][] = [["src-tauri/src/modelo_whisper.rs", 1]];
/** El único cliente de red que puede declararse, y solo en esta sección de `Cargo.toml`. */
const CLIENTE_PERMITIDO = { nombre: "ureq", seccion: "[target.'cfg(windows)'.dependencies]" };

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
            hallazgos.push(`${f}:${i + 1}  ${nombre}  ${linea.trim().slice(0, 90)}`);
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
    // Solo las líneas de dependencia: `nombre = ...` al principio de línea, con la sección en que viven.
    let seccion = "";
    const declaradas: { nombre: string; seccion: string }[] = [];
    for (const l of manifiesto.split("\n")) {
      if (/^\[.*\]\s*$/.test(l)) seccion = l.trim();
      const nombre = l.match(/^([A-Za-z0-9_-]+)\s*=/)?.[1];
      if (nombre) declaradas.push({ nombre, seccion });
    }
    const culpables = declaradas
      .filter((d) => CLIENTES.includes(d.nombre))
      .filter((d) => !(d.nombre === CLIENTE_PERMITIDO.nombre && d.seccion === CLIENTE_PERMITIDO.seccion))
      .map((d) => `${d.nombre} en ${d.seccion || "(sin sección)"}`);
    expect(
      culpables,
      `clientes de red en ${CARGO}: ${culpables.join(", ")} — necesitan ADR antes de entrar`,
    ).toEqual([]);
  });

  /**
   * **Ni por las features de `windows` o `windows-sys`** (auditoría del S6, B11). El gate de arriba mira los clientes de
   * red por su nombre de crate, pero las dos crates de Windows traen sockets, WinHTTP y WinINet detrás de features:
   * `Win32_Networking_*` (WinSock, WinHttp, WinInet), `Networking_*` y `Web_*` (WinRT), `Win32_NetworkManagement_*`. Una
   * línea en `Cargo.toml` abriría una puerta que ninguna búsqueda de `ureq` ve. Lo leído: cada feature declarada.
   */
  it("ninguna feature de windows o windows-sys trae red (B11 del S6)", () => {
    const manifiesto = readFileSync(CARGO, "utf8");
    const bloques = [...manifiesto.matchAll(/^(windows(?:-sys)?)\s*=\s*\{[^}]*features\s*=\s*\[([^\]]*)\]/gms)];
    expect(bloques.map((b) => b[1]).sort(), "no encontré las dos crates de Windows en Cargo.toml").toEqual(["windows", "windows-sys"]);
    const features = bloques.flatMap((b) => [...b[2].matchAll(/"([^"]+)"/g)].map((m) => `${b[1]}:${m[1]}`));
    expect(features.length, "no leí ninguna feature: el lector se rompió").toBeGreaterThan(10);
    const CON_RED = /:(Win32_Networking|Networking|Web(_|$)|Win32_NetworkManagement|Win32_Security_Authentication)/;
    expect(features.filter((f) => CON_RED.test(f)), "features de Windows con red: necesitan ADR antes de entrar").toEqual([]);
  });

  it("ningún archivo de Rust abre un socket, salvo las puertas declaradas (ADR 022 §4, ADR 023)", () => {
    const hallazgos = barrer(archivos(RUST, [".rs"]), SOCKETS_RUST);
    // `barrer` corta la línea a 90 caracteres: la marca se busca en la línea entera del archivo.
    const declarada = (h: string) => {
      const puerta = PUERTAS_RUST.find(([f]) => h.startsWith(`${f}:`));
      if (!puerta) return false;
      const n = Number(h.slice(puerta[0].length + 1).split(" ")[0]);
      return DECLARADA.test(readFileSync(puerta[0], "utf8").split("\n")[n - 1] ?? "");
    };
    const sinDeclarar = hallazgos.filter((h) => !declarada(h));
    expect(sinDeclarar, `salidas a la red en Rust:\n${sinDeclarar.join("\n")}`).toEqual([]);
    for (const [f, n] of PUERTAS_RUST) {
      const suyas = hallazgos.filter((h) => h.startsWith(`${f}:`) && declarada(h));
      expect(
        suyas.length,
        `cambió el número de líneas de red de ${f} (ADR 022 §4, ADR 023):\n${suyas.join("\n")}`,
      ).toBe(n);
    }
  });

  /**
   * **Ni lanza un programa que salga por él** (auditoría del S2, B13): un `Command::new("curl")` en
   * `lib.rs` pasaba todos los gates. La app lanza exactamente DOS programas, cada uno desde un solo
   * archivo y con la ruta entera: `/usr/bin/profiles`, desde el MDM del radar, que solo lee la
   * inscripción de ESTE Mac; y `/bin/launchctl`, desde `vencimiento/` (ADR 016), que registra la tarea
   * que borra lo vencido con la app cerrada. Ninguno sale a la red, y cualquier otro es rojo.
   */
  it("los dos programas que la app lanza son el del MDM del radar y el de launchd", () => {
    const PERMITIDOS: Record<string, string> = {
      "src-tauri/src/radar/mdm.rs": "/usr/bin/profiles",
      "src-tauri/src/vencimiento/mod.rs": "/bin/launchctl",
    };
    const lanzados: string[] = [];
    for (const f of archivos(RUST, [".rs"])) {
      readFileSync(f, "utf8")
        .split("\n")
        .forEach((linea, i) => {
          const t = linea.trim();
          if (t.startsWith("//")) return;
          for (const m of linea.matchAll(/Command::new\(\s*("[^"]*"|[^)]*)\)/g)) {
            const programa = m[1].replace(/"/g, "");
            if (PERMITIDOS[f] !== programa) lanzados.push(`${f}:${i + 1} lanza «${programa}»`);
          }
        });
    }
    expect(lanzados, `programas lanzados fuera de la lista:\n${lanzados.join("\n")}`).toEqual([]);
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
            `${f}:${i + 1}  ${toca.map(([, n]) => n).join(", ")}  ${linea.trim().slice(0, 90)}`,
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

  /**
   * **La puerta de la síntesis es efímera** (ADR 011). Una sesión de red por defecto guarda las
   * respuestas en la caché del disco, y la respuesta del proveedor lleva texto derivado del turno
   * del cliente: sería un archivo que sobrevive a la reunión. La sesión tiene que nacer `.ephemeral`.
   */
  it("la puerta de la síntesis no deja caché en el disco", () => {
    const red = readFileSync(join(NATIVO, "Red.swift"), "utf8");
    const sesiones = red.split("\n").filter((l) => /\bURLSession\s*\(/.test(l));
    expect(sesiones.length, "no se encontró la sesión: este test no mide nada").toBe(1);
    expect(sesiones[0]).toMatch(/configuration:\s*\.ephemeral\b/);
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
