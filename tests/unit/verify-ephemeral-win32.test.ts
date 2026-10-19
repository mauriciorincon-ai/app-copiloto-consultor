import { spawnSync } from "node:child_process";
import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { afterAll, describe, expect, it } from "vitest";

/**
 * **EL BARRIDO VE LA API DE WIN32 DE LOS NAMED PIPES, Y LO DEMUESTRA FALLANDO** (sprint 012, auditoría B6; regla 15).
 *
 * `scripts/verify-ephemeral.mjs` aprendió en el S12 cuatro patrones nuevos (`CreateNamedPipeW`, `CreateFileW`, `WriteFile`,
 * `ReadFile`): la puerta local de Windows es un named pipe, y su API no se parece a la de `std`. Un gate nuevo se demuestra
 * en rojo, y una demo que solo se corrió una vez se pierde: este test la repite en cada corrida, **sobre una copia vacía
 * de los módulos protegidos** (nunca sobre el repo), plantando la línea y mirando qué dice el script.
 *
 * ¿Puede fallar? Sí: quitar cualquiera de los cuatro patrones, o dejar de exigir el ADR en la marca, es rojo.
 */
const SCRIPT = resolve("scripts/verify-ephemeral.mjs");
const raices: string[] = [];

/** Una copia vacía de los módulos protegidos con un solo archivo plantado en `notas/`. */
function plantar(linea: string): { codigo: number | null; fuera: string } {
  const raiz = mkdtempSync(join(tmpdir(), "ag-verify-"));
  raices.push(raiz);
  const fuente = readFileSync(SCRIPT, "utf8");
  const bloque = fuente.slice(
    fuente.indexOf("const PROTEGIDOS = ["),
    fuente.indexOf("];", fuente.indexOf("const PROTEGIDOS = [")),
  );
  const rutas = [...bloque.matchAll(/^\s*"(src-tauri\/[^"]+)",?\s*$/gm)].map(
    (m) => m[1],
  );
  expect(rutas.length, "no se leyó la lista de PROTEGIDOS").toBeGreaterThan(10);
  for (const r of rutas) mkdirSync(join(raiz, r), { recursive: true });
  writeFileSync(
    join(raiz, "src-tauri/src/notas/planta.rs"),
    `fn planta() {\n    ${linea}\n}\n`,
  );
  const r = spawnSync(process.execPath, [SCRIPT], {
    cwd: raiz,
    encoding: "utf8",
  });
  return { codigo: r.status, fuera: `${r.stdout}${r.stderr}` };
}

afterAll(() => {
  for (const r of raices) rmSync(r, { recursive: true, force: true });
});

describe("verify:ephemeral ve la API de Win32 de los named pipes", () => {
  it("CONTROL: una copia vacía de los módulos protegidos pasa limpia", () => {
    const r = plantar("let _ = 1;");
    expect(r.codigo, r.fuera).toBe(0);
  });

  it.each([
    [
      "CreateNamedPipeW",
      "unsafe { CreateNamedPipeW(nombre, 0, 0, 1, 0, 0, 0, std::ptr::null()) };",
    ],
    [
      "CreateFileW",
      "unsafe { CreateFileW(nombre, 0, 0, std::ptr::null(), 3, 0, std::ptr::null_mut()) };",
    ],
    [
      "WriteFile",
      "unsafe { WriteFile(mango, bytes, 1, &mut n, std::ptr::null_mut()) };",
    ],
    [
      "ReadFile",
      "unsafe { ReadFile(mango, bytes, 1, &mut n, std::ptr::null_mut()) };",
    ],
  ])(
    "%s sin marca en un módulo protegido es rojo, y lo nombra",
    (api, linea) => {
      const r = plantar(linea);
      expect(r.codigo, `${api} sin marca pasó en verde:\n${r.fuera}`).toBe(1);
      expect(r.fuera).toContain("planta.rs");
      expect(r.fuera).toContain(api);
    },
  );

  it("con la marca Y el ADR en la misma línea, pasa; con la marca sin ADR, sigue en rojo", () => {
    const con = plantar(
      "unsafe { CreateFileW(n, 0, 0, std::ptr::null(), 3, 0, std::ptr::null_mut()) }; // verify-ephemeral:allow — ADR 018 §3: la puerta",
    );
    expect(con.codigo, con.fuera).toBe(0);
    const sin = plantar(
      "unsafe { CreateFileW(n, 0, 0, std::ptr::null(), 3, 0, std::ptr::null_mut()) }; // verify-ephemeral:allow",
    );
    expect(sin.codigo, "una marca sin ADR dejó pasar la línea").toBe(1);
  });
});
