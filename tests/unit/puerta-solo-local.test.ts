import { readFileSync, readdirSync, statSync, existsSync } from "node:fs";
import { posix } from "node:path";
import { describe, expect, it } from "vitest";

// Las rutas del barrido van con `/` en cualquier sistema (sprint 009): se comparan con las de abajo, escritas con
// `/`, y en Windows `join` y `relative` de `node:path` devolvían barras invertidas.
const { join } = posix;

/**
 * GATE `puerta-solo-local` (sprint 003, fase 4, ADR 018 §8) — **la puerta de tu agente no sale de
 * este Mac**.
 *
 * La puerta local es el primer socket del producto, y ningún gate miraba los sockets Unix: el contador
 * de red busca `std::net`, `TcpStream`, `UdpSocket` y los clientes HTTP, y un
 * `std::os::unix::net::UnixListener` pasaba todos los barridos sin que nadie lo viera. Este gate
 * cierra ese hueco por los dos lados:
 *
 *   1. **Los sockets Unix solo pueden aparecer en la puerta** (`src-tauri/src/puerta/`) y en su cliente
 *      (`src-tauri/src/bin/ghost.rs`). Un socket en cualquier otro módulo sería otra puerta, sin su
 *      política, su llave ni su cierre en reunión.
 *   2. **Dentro de la puerta, nada que sepa salir de la máquina**: ni TCP, ni UDP, ni resolver un
 *      nombre, ni `libc` de red, ni un cliente HTTP, ni una URL. Un socket Unix no sabe salir del Mac;
 *      eso es lo que hace estructural la regla dura 9 para tu agente, y lo que sostiene el «el contador
 *      sigue igual» de la pantalla.
 *   3. **El socket nace en un solo sitio, dentro de la carpeta de la app**: un `bind` y solo uno, sobre
 *      la ruta que da `ruta_en`.
 *
 *   4. **En Windows (sprint 012), el named pipe es la puerta y solo de este equipo**: el API de pipes solo vive en la
 *      puerta, el nombre es siempre el de un pipe local (el punto: este PC, nunca otro servidor) y el pipe nace con
 *      `PIPE_REJECT_REMOTE_CLIENTS`, que deja fuera a cualquier cliente de otra máquina.
 *
 * ¿Puede fallar? Sí, y nació en rojo con un `TcpStream` plantado en `puerta/socket.rs` y con un
 * `UnixStream` plantado en `lib.rs` (bitácora del sprint 003, fase 4). Las agujas se arman por partes
 * para que este archivo no se cuente a sí mismo.
 */
const RUST = "src-tauri/src";
const PUERTA = "src-tauri/src/puerta";
const GHOST = "src-tauri/src/bin/ghost.rs";

/** Una línea que contenga esta marca NOMBRA la red sin abrirla (prosa, documentación). */
const CITA = "red:cita";

const UNIX: RegExp[] = [
  new RegExp("\\bUnix" + "(Listener|Stream|Datagram)\\b"),
  new RegExp("\\bstd::os::unix::" + "net\\b"),
];

const FUERA_DEL_MAC: [RegExp, string][] = [
  [new RegExp("\\bstd::" + "net\\b"), "std::net"],
  [
    new RegExp("\\b(Tcp" + "Stream|Tcp" + "Listener|Udp" + "Socket)\\b"),
    "TCP o UDP",
  ],
  [
    new RegExp(
      "\\b(ToSocket" +
        "Addrs|lookup_" +
        "host|getaddr" +
        "info|gethostby" +
        "name)\\b",
    ),
    "resolver un nombre",
  ],
  [
    new RegExp(
      "\\blibc::(" + "socket|connect|sendto|sendmsg|bind|listen|accept)\\b",
    ),
    "libc de red",
  ],
  [
    new RegExp(
      "\\b(req" +
        "west|hy" +
        "per|ure" +
        "q|isa" +
        "hc|socket" +
        "2|tungs" +
        "tenite)\\b",
    ),
    "un cliente de red",
  ],
  [new RegExp("https?:" + "//[\\w.-]"), "una URL"],
];

/** El API de named pipes de Win32 y el prefijo de un pipe en el código: solo la puerta puede usarlos. */
const PIPES: RegExp[] = [
  new RegExp("\\b(CreateNamed" + "PipeW?|ConnectNamed" + "Pipe|WaitNamed" + "PipeW?)\\b"),
  new RegExp(String.raw`\\\\[.?]\\` + "pi" + "pe" + String.raw`\\`),
];

function archivos(ruta: string): string[] {
  if (!existsSync(ruta)) return [];
  if (statSync(ruta).isFile()) return ruta.endsWith(".rs") ? [ruta] : [];
  return readdirSync(ruta).flatMap((n) => archivos(join(ruta, n)));
}

/** Las líneas de código, sin las que son solo comentario: la prosa puede NOMBRAR lo que el código no HACE. */
function codigo(f: string): [number, string][] {
  return readFileSync(f, "utf8")
    .split("\n")
    .map((l, i) => [i + 1, l] as [number, string])
    .filter(([, l]) => !l.trim().startsWith("//") && !l.includes(CITA));
}

const dePuerta = (f: string) => f.startsWith(PUERTA + "/") || f === GHOST;

describe("puerta-solo-local: la puerta de tu agente no sale de este Mac", () => {
  it("hay código que inspeccionar (si no, este gate no mide nada)", () => {
    expect(archivos(PUERTA).length).toBeGreaterThanOrEqual(3);
    expect(existsSync(GHOST)).toBe(true);
    const unix = archivos(PUERTA)
      .flatMap(codigo)
      .filter(([, l]) => UNIX.some((re) => re.test(l)));
    expect(
      unix.length,
      "la puerta ya no usa sockets Unix: este gate mira un sitio vacío",
    ).toBeGreaterThan(0);
  });

  it("los sockets Unix solo viven en la puerta y en ghost", () => {
    const fuera = archivos(RUST)
      .filter((f) => !dePuerta(f))
      .flatMap((f) =>
        codigo(f)
          .filter(([, l]) => UNIX.some((re) => re.test(l)))
          .map(
            ([n, l]) => `${f}:${n}  ${l.trim().slice(0, 90)}`,
          ),
      );
    expect(
      fuera,
      `sockets Unix fuera de la puerta:\n${fuera.join("\n")}`,
    ).toEqual([]);
  });

  it("dentro de la puerta nada sabe salir de la máquina", () => {
    const hallazgos = [...archivos(PUERTA), GHOST].flatMap((f) =>
      codigo(f).flatMap(([n, l]) =>
        FUERA_DEL_MAC.filter(([re]) => re.test(l)).map(
          ([, que]) =>
            `${f}:${n}  ${que}  ${l.trim().slice(0, 90)}`,
        ),
      ),
    );
    expect(
      hallazgos,
      `la puerta puede salir del Mac:\n${hallazgos.join("\n")}`,
    ).toEqual([]);
  });

  it("en Windows, el API de named pipes solo vive en la puerta", () => {
    const enLaPuerta = archivos(PUERTA)
      .flatMap(codigo)
      .filter(([, l]) => PIPES.some((re) => re.test(l)));
    expect(
      enLaPuerta.length,
      "la puerta de Windows ya no es un named pipe: este gate mira un sitio vacío",
    ).toBeGreaterThan(0);
    const fuera = archivos(RUST)
      .filter((f) => !dePuerta(f))
      .flatMap((f) =>
        codigo(f)
          .filter(([, l]) => PIPES.some((re) => re.test(l)))
          .map(([n, l]) => `${f}:${n}  ${l.trim().slice(0, 90)}`),
      );
    expect(fuera, `named pipes fuera de la puerta:\n${fuera.join("\n")}`).toEqual(
      [],
    );
  });

  it("en Windows, el pipe es de este equipo: nombre local y sin clientes de otra máquina", () => {
    const tubo = readFileSync(join(PUERTA, "windows.rs"), "utf8");
    const socket = readFileSync(join(PUERTA, "socket.rs"), "utf8");
    // El nombre: `\\.\pipe\…`, con el punto de este PC; nunca `\\servidor\pipe\…`.
    expect(socket).toContain(
      String.raw`PathBuf::from(format!(r"\\.\pipe\{tubo}"))`,
    );
    // El modo con el que nace, y que la creación lo usa.
    expect(tubo).toMatch(/pub const MODO: u32 = [^;]*PIPE_REJECT_REMOTE_CLIENTS;/);
    expect(tubo).toContain("CreateNamedPipeW(nombre.as_ptr(), APERTURA, MODO, 1,");
    expect(tubo).toMatch(
      /pub const APERTURA: u32 = [^;]*FILE_FLAG_FIRST_PIPE_INSTANCE;/,
    );
  });

  it("el socket nace en un solo sitio, en la carpeta de la app", () => {
    const bind = new RegExp("Unix" + "Listener::bind\\(");
    const binds = archivos(RUST).flatMap((f) =>
      codigo(f)
        .filter(([, l]) => bind.test(l))
        .map(([n, l]) => ({ f, n, l })),
    );
    expect(binds.map((b) => `${b.f}:${b.n}`)).toEqual([
      expect.stringContaining("src-tauri/src/puerta/socket.rs"),
    ]);
    expect(binds[0].l).toMatch(/bind\(&ruta\)/);
    const socket = readFileSync(join(PUERTA, "socket.rs"), "utf8");
    expect(socket).toMatch(/let ruta = ruta_en\(carpeta\);/);
  });
});
