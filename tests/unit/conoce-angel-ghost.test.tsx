import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { readFileSync } from "node:fs";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { Cascara } from "@/App";
import { Conoce } from "@/componentes/Conoce";

/**
 * **«CONOCE ANGEL GHOST»** (sprint 008): el botón de Honestidad que abre el recorrido visual en el navegador.
 *
 * Lo que se promete es poco y se comprueba entero: el recorrido es EL archivo del repositorio dentro de los recursos de
 * la app, el comando solo lo abre con `opener` y **no toca la red** (la orden de entrega: «el contador de salida no se
 * mueve al abrirlo»), solo la ventana principal puede llamarlo, y fuera de la app el botón no existe (la Honestidad
 * aprobada en la maqueta no cambia).
 *
 * ¿Puede fallar? Sí: meter un `ureq` o un `red::registrar_salida` en `abrir_conoce`, quitar el recurso de
 * `tauri.conf.json`, darle el comando a la banda o dejar el botón fuera de Tauri lo pone rojo, con el nombre de lo roto.
 */
const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...a: unknown[]) => invoke(...a) }));

const LIB = readFileSync("src-tauri/src/lib.rs", "utf8");
const cuerpo = (nombre: string) => {
  const desde = LIB.indexOf(`fn ${nombre}(`);
  expect(desde, `lib.rs no tiene fn ${nombre}`).toBeGreaterThan(-1);
  return LIB.slice(desde, LIB.indexOf("\n}\n", desde) + 3);
};

describe("el comando abre un archivo local y nada más", () => {
  it("no usa la red, no escribe nada y abre con el plugin opener", () => {
    const c = cuerpo("abrir_conoce");
    expect(c).toContain("tauri_plugin_opener::open_path(");
    expect(c).not.toMatch(/red::|ureq|reqwest|TcpStream|UdpSocket|open_url|fs::write|File::create|Command::new/);
  });

  it("el recurso es el archivo del repositorio, con el mismo nombre que el comando busca", () => {
    const conf = JSON.parse(readFileSync("src-tauri/tauri.conf.json", "utf8")) as { bundle: { resources: Record<string, string> } };
    expect(conf.bundle.resources["../docs/BROCHURE.html"]).toBe("BROCHURE.html");
    expect(cuerpo("ruta_del_recorrido")).toContain('join("BROCHURE.html")');
    expect(readFileSync("docs/BROCHURE.html", "utf8").length).toBeGreaterThan(50_000);
  });

  it("solo la ventana principal puede llamarlo: la banda y el relleno, no", () => {
    const permiso = "allow-abrir-conoce";
    expect(JSON.parse(readFileSync("src-tauri/capabilities/default.json", "utf8")).permissions).toContain(permiso);
    for (const otra of ["banda", "relleno"])
      expect(JSON.parse(readFileSync(`src-tauri/capabilities/${otra}.json`, "utf8")).permissions).not.toContain(permiso);
    expect(readFileSync("src-tauri/build.rs", "utf8")).toContain('"abrir_conoce"');
    expect(LIB).toMatch(/generate_handler!\[[\s\S]*abrir_conoce,/);
  });
});

describe("el botón de Honestidad", () => {
  beforeEach(() => invoke.mockReset());
  afterEach(() => {
    cleanup();
    delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    document.documentElement.lang = "";
  });

  it("fuera de la app no existe: la Honestidad de la maqueta no cambia", () => {
    render(
      <Cascara>
        <Conoce />
      </Cascara>,
    );
    expect(screen.queryByRole("button", { name: /recorrido|tour/i })).toBeNull();
  });

  it("dentro de la app, abre el recorrido por su comando, en los dos idiomas", async () => {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    invoke.mockResolvedValue(undefined);
    for (const [idioma, boton, titulo] of [
      ["es", "Abrir el recorrido", "Conoce Angel Ghost"],
      ["en", "Open the tour", "Get to know Angel Ghost"],
    ] as const) {
      cleanup();
      invoke.mockClear();
      document.documentElement.lang = idioma;
      render(
        <Cascara>
          <Conoce />
        </Cascara>,
      );
      expect(screen.getByRole("heading", { name: titulo })).toBeTruthy();
      fireEvent.click(screen.getByRole("button", { name: boton }));
      await waitFor(() => expect(invoke).toHaveBeenCalledWith("abrir_conoce", undefined));
      // La cáscara pregunta otras cosas al arrancar; el recorrido se abre UNA vez.
      expect(invoke.mock.calls.filter(([c]) => c === "abrir_conoce")).toHaveLength(1);
    }
  });

  it("si el sistema no pudo abrirlo, lo dice con símbolo y texto", async () => {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    invoke.mockImplementation(async (comando: string) => {
      if (comando === "abrir_conoce") throw "el recorrido no viaja en esta compilación";
      return undefined;
    });
    document.documentElement.lang = "es";
    render(
      <Cascara>
        <Conoce />
      </Cascara>,
    );
    fireEvent.click(screen.getByRole("button", { name: "Abrir el recorrido" }));
    expect(await screen.findByText("No se pudo abrir el recorrido en esta compilación.")).toBeTruthy();
  });
});
