import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";

/**
 * GATE DE LA REGLA DURA 9 — **EL RADAR MIRA SOLO ESTE MAC** (sprint 002, fase 4).
 *
 * La regla de la casa: la app jamás sondea, escanea ni actúa sobre el computador de la contraparte
 * —sería acceso abusivo y el espejo de lo que el radar existe para señalar—. El radar es el único
 * módulo cuyo TRABAJO es mirar «qué hay», y por eso es el que más cerca queda de cruzar esa línea:
 * un `ping` para saber si hay alguien en la red, un nombre que resolver, una conexión «solo para
 * comprobar». Este gate no deja que ninguna de esas cosas entre en silencio.
 *
 * **Por qué no basta con el gate del contador de red.** Ese barre el crate buscando sockets de la
 * biblioteca estándar y clientes HTTP, y aquí se repite; pero no ve las dos puertas que un módulo
 * que ya habla con el núcleo tiene a mano: **una llamada de `libc`** (`libc::socket`,
 * `libc::connect`, `libc::getaddrinfo`) y **un programa externo** (`ping`, `ssh`, `arp`, `nc`). Así
 * que aquí se invierte la pregunta: no «¿hay algo prohibido?», sino **«¿todo lo que llama está en
 * la lista?»**. La lista es corta y cada entrada dice por qué:
 *
 *  - `proc_listallpids` y `proc_pidpath`: la lista de procesos de ESTE Mac, al núcleo de este Mac.
 *  - `/usr/bin/profiles status`: la inscripción de este Mac en un MDM. Lee la configuración local.
 *
 * ¿Puede fallar? Sí, y por tres lados que el contador de red no ve: se vio en rojo con un
 * `libc::socket` plantado, con un `Command::new("ping")` y con un `extern "C"` propio (bitácora
 * del sprint 002, fase 4).
 */
const RADAR = "src-tauri/src/radar";

/** Lo que el radar puede pedirle a `libc`, y nada más. */
const LIBC_PERMITIDO = new Set([
  "proc_listallpids",
  "proc_pidpath",
  "PROC_PIDPATHINFO_MAXSIZE",
  "c_int",
  "c_void",
  "pid_t",
]);

/** Los únicos programas que el radar puede lanzar. */
const PROGRAMAS_PERMITIDOS = new Set(["/usr/bin/profiles"]);

/** Formas de hablar con otra máquina, o de salirse de la lista por la puerta de atrás. */
const PROHIBIDO: [RegExp, string][] = [
  [/\bstd::net\b/, "std::net"],
  [/\bTcpStream\b|\bUdpSocket\b|\bTcpListener\b/, "un socket de la biblioteca estándar"],
  [/\bToSocketAddrs\b|\blookup_host\b|\bgetaddrinfo\b|\bgethostbyname\b/, "resolver un nombre"],
  [/\bextern\s+"C"/, "declarar funciones de C propias (se saltarían la lista de libc)"],
  [/#\[link\b/, "enlazar una biblioteca propia"],
  [/\bdlopen\b|\bdlsym\b/, "cargar una biblioteca en marcha"],
  [/https?:\/\/[\w.-]/, "una URL en el código (las fuentes viven en el catálogo, no aquí)"],
  [/\b\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}\b/, "una dirección IP"],
];

/**
 * El código sin sus comentarios: la prosa puede NOMBRAR lo que el código no puede HACER.
 *
 * Solo se quitan las líneas que SON comentario. La primera versión cortaba todo lo que viniera
 * detrás de `//` en cualquier línea, y eso se comía también las URLs —`"https://…"` se quedaba en
 * `"https:`— así que la regla de las URLs no podía fallar nunca. Se vio al leer por qué pasaba en
 * verde con una URL en el propio catálogo.
 */
function codigo(texto: string): string[] {
  return texto.split("\n").map((l) => (l.trim().startsWith("//") ? "" : l));
}

function archivos(): [string, string[]][] {
  return readdirSync(RADAR)
    .filter((n) => n.endsWith(".rs"))
    .map((n) => [join(RADAR, n), codigo(readFileSync(join(RADAR, n), "utf8"))]);
}

describe("regla dura 9: el radar mira solo este Mac", () => {
  it("el módulo existe y tiene código que barrer", () => {
    const todo = archivos();
    expect(todo.length, "un gate que no lee nada no es un gate").toBeGreaterThanOrEqual(4);
    expect(todo.flatMap(([, l]) => l).join("\n")).toMatch(/proc_listallpids/);
  });

  it("cada llamada a libc está en la lista", () => {
    const fuera: string[] = [];
    for (const [f, lineas] of archivos()) {
      const texto = lineas.join("\n");
      // `use libc::{a, b, c};` y `libc::a` — las dos formas de nombrar algo de libc.
      for (const m of texto.matchAll(/use\s+libc::\{([^}]*)\}/g))
        for (const n of m[1]!.split(",").map((x) => x.trim()).filter(Boolean))
          if (!LIBC_PERMITIDO.has(n)) fuera.push(`${f}: libc::${n}`);
      for (const m of texto.matchAll(/\blibc::(\w+)/g))
        if (!LIBC_PERMITIDO.has(m[1]!)) fuera.push(`${f}: libc::${m[1]}`);
    }
    expect(fuera, `el radar llama a libc fuera de su lista:\n${fuera.join("\n")}`).toEqual([]);
  });

  it("solo lanza los programas de la lista, y con la ruta entera", () => {
    const fuera: string[] = [];
    for (const [f, lineas] of archivos()) {
      lineas.forEach((l, i) => {
        if (!/Command::new\s*\(/.test(l)) return;
        const m = l.match(/Command::new\s*\(\s*"([^"]*)"\s*\)/);
        if (!m || !PROGRAMAS_PERMITIDOS.has(m[1]!)) fuera.push(`${f}:${i + 1}  ${l.trim()}`);
      });
    }
    expect(fuera, `el radar lanza un programa fuera de su lista:\n${fuera.join("\n")}`).toEqual([]);
  });

  it("no tiene ninguna forma de hablar con otra máquina", () => {
    const fuera: string[] = [];
    for (const [f, lineas] of archivos())
      lineas.forEach((l, i) => {
        for (const [re, que] of PROHIBIDO) if (re.test(l)) fuera.push(`${f}:${i + 1}  ${que}  →  ${l.trim()}`);
      });
    expect(fuera, `el radar puede salir de este Mac:\n${fuera.join("\n")}`).toEqual([]);
  });

  it("está entre los módulos protegidos del efímero (ni disco ni red)", () => {
    const script = readFileSync("scripts/verify-ephemeral.mjs", "utf8");
    expect(script).toContain(`"${RADAR}"`);
  });
});
