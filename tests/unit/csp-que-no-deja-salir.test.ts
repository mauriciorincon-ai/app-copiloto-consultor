import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * Gate de la **CSP** — la tercera capa de «nada crudo sale del equipo», y el hallazgo M1.
 *
 * `tauri.conf.json` traía `"csp": null` desde el estampado: sin política, el webview puede pedir
 * cualquier cosa a cualquier sitio. Las dos primeras capas de la promesa son el lint que prohíbe
 * API de red en los módulos protegidos y el contador de puertas declaradas; **las dos viven en
 * Rust**, y ninguna mira lo que el webview pueda hacer por su cuenta — un `fetch` en un `.tsx`, un
 * `<img>` a un dominio de fuera, una fuente de Google en un CSS. Esta es la capa que cubre ese
 * lado, y es la única que el navegador aplica por su cuenta sin depender de que alguien se acuerde.
 *
 * Lo que se vigila aquí no es «hay una CSP» —eso sería un campo con texto— sino **que siga siendo
 * cerrada**:
 *
 *  1. existe, en producción y en desarrollo;
 *  2. `default-src` es `'self'` y nada más;
 *  3. **`connect-src` no deja salir a ningún sitio de fuera** — que es la directiva que sostiene la
 *     promesa del producto. El día que el API opt-in de la fase 5 necesite un dominio, este test
 *     falla y **obliga a nombrarlo**: un proveedor concreto, escrito, con su línea en el summary, en
 *     vez de un `https:` que abre la puerta a todos;
 *  4. ni `script-src` ni `object-src` admiten nada ejecutable de fuera.
 *
 * `'unsafe-inline'` en `style-src` está a propósito y no es un descuido: la interfaz usa atributos
 * `style` de React por todas partes —la maqueta se traslada así— y un atributo `style` inline no
 * pasa por nonce ni por hash. Es riesgo de estilo, no de ejecución, y queda escrito aquí para que
 * nadie tenga que adivinar si fue una decisión o una prisa. En `script-src` NO está, y ahí sí sería
 * grave: por eso el test lo prohíbe en producción.
 *
 * ¿Puede fallar? Sí, y falló: escrito contra el `"csp": null` del sprint 001 no pasa ni la primera
 * aserción. Y su demo de verdad es la de la bitácora: un `fetch` plantado a un dominio de fuera,
 * bloqueado con la app corriendo, con el control de que ese mismo dominio sí se alcanza con `curl`.
 */
const CONFIG = "src-tauri/tauri.conf.json";

/** Los orígenes que esta app puede nombrar, y por qué cada uno. */
const PERMITIDOS: Record<string, string> = {
  "'self'": "la propia app",
  "'none'": "nada",
  "'unsafe-inline'": "atributos style de React (solo en style-src)",
  "data:": "el fondo de escritorio del relleno, que llega como data URL",
  "ipc:": "el puente de Tauri",
  "http://ipc.localhost": "el puente de Tauri en macOS",
  "http://localhost:1420": "el servidor de Vite — SOLO en desarrollo",
  "ws://localhost:1420": "el recargado en caliente de Vite — SOLO en desarrollo",
};

/** Un origen que deja salir a internet. Es lo que este gate existe para que no aparezca. */
function haciaAfuera(fuente: string): boolean {
  if (fuente in PERMITIDOS) return false;
  return true;
}

function directivas(csp: string): Map<string, string[]> {
  const m = new Map<string, string[]>();
  for (const trozo of csp.split(";")) {
    const partes = trozo.trim().split(/\s+/).filter(Boolean);
    if (partes.length === 0) continue;
    m.set(partes[0], partes.slice(1));
  }
  return m;
}

const config = JSON.parse(readFileSync(CONFIG, "utf8")) as {
  app?: { security?: { csp?: string | null; devCsp?: string | null } };
};
const seguridad = config.app?.security ?? {};

describe("la CSP no deja salir nada del equipo", () => {
  it("existe, en producción y en desarrollo (el sprint 001 la tenía en null)", () => {
    for (const cual of ["csp", "devCsp"] as const) {
      const v = seguridad[cual];
      expect(typeof v, `${cual} es ${JSON.stringify(v)}: sin política, el webview puede pedir cualquier cosa`).toBe("string");
      expect((v as string).length).toBeGreaterThan(40);
    }
  });

  it.each(["csp", "devCsp"] as const)("%s — default-src es 'self' y nada más", (cual) => {
    const d = directivas(seguridad[cual] as string);
    expect(d.get("default-src"), `${cual} no cierra el caso por defecto`).toEqual(["'self'"]);
  });

  it.each(["csp", "devCsp"] as const)("%s — ninguna directiva nombra un sitio de fuera", (cual) => {
    const d = directivas(seguridad[cual] as string);
    const fugas: string[] = [];
    for (const [directiva, fuentes] of d) {
      for (const f of fuentes) {
        if (haciaAfuera(f)) fugas.push(`${directiva}: ${f}`);
      }
    }
    expect(
      fugas,
      `la app promete que nada sale del equipo y la CSP dejaría salir por:\n  ${fugas.join("\n  ")}\n` +
        `Si una fase necesita un dominio, nómbralo en PERMITIDOS con su razón — jamás un esquema suelto.`,
    ).toEqual([]);
  });

  it("connect-src existe en las dos y solo nombra el puente (y Vite en desarrollo)", () => {
    const prod = directivas(seguridad.csp as string).get("connect-src");
    expect(prod, "sin connect-src manda default-src, pero escribirlo es lo que hace el gate legible").toBeDefined();
    expect(prod).toEqual(["'self'", "ipc:", "http://ipc.localhost"]);
    const dev = directivas(seguridad.devCsp as string).get("connect-src") ?? [];
    expect(dev).toContain("http://localhost:1420");
  });

  it("script-src no admite inline en producción, y object-src está cerrado en las dos", () => {
    const prod = directivas(seguridad.csp as string);
    expect(prod.get("script-src")).toEqual(["'self'"]);
    for (const cual of ["csp", "devCsp"] as const) {
      const d = directivas(seguridad[cual] as string);
      expect(d.get("object-src"), `${cual} deja abierto object-src`).toEqual(["'none'"]);
      expect(d.get("base-uri"), `${cual} no fija base-uri`).toEqual(["'self'"]);
      expect(d.get("frame-ancestors"), `${cual} no cierra frame-ancestors`).toEqual(["'none'"]);
    }
  });
});
