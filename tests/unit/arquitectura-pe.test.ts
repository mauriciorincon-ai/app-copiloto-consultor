// @vitest-environment node
/**
 * El gate `scripts/arquitectura-pe.mjs` (sprint 009, La Surface): el artefacto que la CI llama ARM64 tiene que ser
 * ARM64. Se prueba con cabeceras PE sintéticas —ningún binario real entra en el repo— y se prueba que el job de la CI
 * lo llama sobre los binarios que construye (un gate que nadie ejecuta no es un gate).
 */
import { spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { arquitecturaDe, mirar } from "../../scripts/arquitectura-pe.mjs";

const ARM64 = 0xaa64;
const X64 = 0x8664;
const X86 = 0x014c;

/** Una cabecera PE mínima: «MZ», el desplazamiento a «PE\0\0» en 0x3C y el campo Machine justo después. */
function pe(machine: number): Buffer {
  const b = Buffer.alloc(256);
  b.write("MZ", 0, "latin1");
  b.writeInt32LE(0x80, 0x3c);
  b.write("PE\0\0", 0x80, "latin1");
  b.writeUInt16LE(machine, 0x84);
  return b;
}

const carpeta = mkdtempSync(join(tmpdir(), "arquitectura-pe-"));
function archivo(nombre: string, contenido: Buffer | string): string {
  const ruta = join(carpeta, nombre);
  writeFileSync(ruta, contenido);
  return ruta;
}

function gate(...args: string[]) {
  return spawnSync(process.execPath, ["scripts/arquitectura-pe.mjs", ...args], {
    encoding: "utf8",
  });
}

describe("arquitectura-pe: lee la arquitectura de la cabecera PE", () => {
  it("distingue ARM64, x64 y x86", () => {
    expect(arquitecturaDe(pe(ARM64))).toBe("arm64");
    expect(arquitecturaDe(pe(X64))).toBe("x64");
    expect(arquitecturaDe(pe(X86))).toBe("x86");
  });

  it("lo que no es un PE no tiene arquitectura", () => {
    expect(arquitecturaDe(Buffer.from("#!/bin/sh\necho hola\n"))).toBeNull();
    expect(arquitecturaDe(Buffer.alloc(8))).toBeNull();
    const sinPE = pe(ARM64);
    sinPE.write("XX", 0x80, "latin1"); // «MZ» sí, pero la firma «PE» no
    expect(arquitecturaDe(sinPE)).toBeNull();
    const fuera = pe(ARM64);
    fuera.writeInt32LE(0x7fffffff, 0x3c); // el desplazamiento cae fuera de lo leído
    expect(arquitecturaDe(fuera)).toBeNull();
  });

  it("una máquina que no conoce la dice, no la inventa", () => {
    expect(arquitecturaDe(pe(0x01c4))).toMatch(/^desconocida \(0x1c4\)$/);
  });

  it("mira archivos de verdad y nombra los que no se pueden mirar", () => {
    const bien = archivo("bien.exe", pe(ARM64));
    const texto = archivo("texto.exe", "no soy un ejecutable");
    const r = mirar([bien, texto, join(carpeta, "no-existe.exe")]);
    expect(r[0]).toEqual({ archivo: bien, encontrada: "arm64" });
    expect(r[1].error).toMatch(/no es un ejecutable/);
    expect(r[2].error).toMatch(/no se pudo leer/);
  });
});

describe("arquitectura-pe: el gate falla nombrando lo que encontró (su rojo, permanente)", () => {
  it("pasa con binarios ARM64", () => {
    const a = archivo("a.exe", pe(ARM64));
    const b = archivo("b.dll", pe(ARM64));
    const r = gate("--espera", "arm64", a, b);
    expect(r.status, r.stderr).toBe(0);
    expect(r.stdout).toContain("arm64");
  });

  it("FALLA con un binario x64 y dice «x64»", () => {
    const bueno = archivo("bueno.exe", pe(ARM64));
    const malo = archivo("malo.exe", pe(X64));
    const r = gate("--espera", "arm64", bueno, malo);
    expect(r.status).toBe(1);
    expect(r.stderr).toContain("malo.exe");
    expect(r.stderr).toContain("es x64 y se esperaba arm64");
    expect(r.stderr).not.toContain("bueno.exe: es");
  });

  it("FALLA sin archivos que mirar: un gate que no mira nada no es verde", () => {
    const r = gate("--espera", "arm64");
    expect(r.status).toBe(1);
    expect(r.stderr).toContain("ningún archivo que mirar");
  });

  it("FALLA con un archivo que no existe, no lo salta", () => {
    const r = gate("--espera", "arm64", join(carpeta, "fantasma.exe"));
    expect(r.status).toBe(1);
    expect(r.stderr).toContain("fantasma.exe");
  });

  it("sin --espera o con una arquitectura inventada, no corre", () => {
    expect(gate(archivo("c.exe", pe(ARM64))).status).toBe(2);
    expect(gate("--espera", "riscv", archivo("d.exe", pe(ARM64))).status).toBe(
      2,
    );
  });
});

describe("arquitectura-pe: el job de ARM de la CI lo ejecuta sobre lo que construye", () => {
  const ci = readFileSync(".github/workflows/ci.yml", "utf8");
  // El bloque del job: desde su nombre hasta el siguiente job (dos espacios de sangría) o el final del archivo.
  const job =
    /^ {2}build-windows-arm:\n([\s\S]*?)(?=^ {2}[a-z][\w-]*:\n|(?![\s\S]))/m.exec(
      ci,
    )?.[1] ?? "";

  it("existe, corre en un runner ARM de verdad y no cambia el nombre de los demás", () => {
    expect(job, "no encontré el job build-windows-arm").not.toBe("");
    expect(job).toMatch(/runs-on:\s*windows-11-arm\b/);
    for (const existente of [
      "quality",
      "e2e",
      "build-escritorio",
      "build-windows",
    ]) {
      expect(ci, `desapareció el job ${existente}`).toMatch(
        new RegExp(`^ {2}${existente}:\\n`, "m"),
      );
    }
  });

  it("mira la arquitectura de la app, de ghost y de los instaladores que sube", () => {
    expect(job).toMatch(/node scripts\/arquitectura-pe\.mjs --espera arm64/);
    expect(job).toContain("app-copiloto-consultor.exe");
    expect(job).toContain("ghost.exe");
    // El gate va ANTES de subir el artefacto: un instalador x64 no llega a llamarse ARM64.
    expect(job.indexOf("arquitectura-pe.mjs")).toBeLessThan(
      job.indexOf("upload-artifact"),
    );
    expect(job).toContain("angel-ghost-windows-arm64");
  });

  // La primera corrida del job murió por `"Falta $componente: …"`: en PowerShell `$componente:` se lee como una variable con
  // ámbito y el script entero no se parsea. En Ubuntu no hay quien lo ejecute, así que se vigila la forma.
  it("ningún bloque de PowerShell del workflow escribe `$variable:` sin llaves dentro de una cadena", () => {
    const bloques = [
      ...ci.matchAll(/shell: pwsh\n\s+run: \|\n((?: {10,}.*\n|\n)+)/g),
    ].map((m) => m[1]);
    expect(
      bloques.length,
      "no encontré ningún bloque de PowerShell: el lector se rompió",
    ).toBeGreaterThanOrEqual(3);
    const ambitos =
      /\$(?!env:|global:|script:|local:|private:|using:|true\b|false\b|null\b)[A-Za-z_]\w*:(?!:)/;
    const mal = bloques.flatMap((b) =>
      b
        .split("\n")
        .filter((l) => ambitos.test(l))
        .map((l) => l.trim()),
    );
    expect(mal, "usa ${variable}: dentro de las cadenas").toEqual([]);
  });

  it("corre los tests sin hardware, con el centinela armado (regla 25)", () => {
    expect(job).toMatch(/AG_SIN_HARDWARE:\s*"1"/);
    expect(job).toMatch(/cargo test --locked/);
  });
});
