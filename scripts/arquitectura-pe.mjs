#!/usr/bin/env node
// Sprint 009 (La Surface): ¿el binario que sale de la CI es de la arquitectura que el nombre del artefacto promete?
//
// Un job `build-windows-arm` puede ponerse verde y subir un instalador x64 (un runner mal elegido, un `--target`
// olvidado, una caché de otra arquitectura): el artefacto se llamaría ARM64 y correría emulado justo donde se va a usar.
// Este gate lee el campo `Machine` de la cabecera PE de cada archivo y falla, NOMBRANDO lo que encontró, si no es la
// arquitectura esperada. Sin archivos que mirar también falla: un gate que no mira nada no es verde.
//
// Uso: node scripts/arquitectura-pe.mjs --espera arm64 <archivo.exe> [<archivo.dll> …]
import { openSync, readSync, closeSync, statSync } from "node:fs";
import { pathToFileURL } from "node:url";

const MAQUINAS = new Map([
  [0xaa64, "arm64"],
  [0x8664, "x64"],
  [0x014c, "x86"],
]);

/** La arquitectura de una cabecera PE (los primeros bytes de un .exe o una .dll), o `null` si no es un PE. */
export function arquitecturaDe(cabecera) {
  if (cabecera.length < 0x40 || cabecera[0] !== 0x4d || cabecera[1] !== 0x5a) return null; // «MZ»
  const pe = cabecera.readInt32LE(0x3c);
  if (pe < 0 || pe + 6 > cabecera.length) return null;
  if (cabecera.toString("latin1", pe, pe + 4) !== "PE\0\0") return null;
  return MAQUINAS.get(cabecera.readUInt16LE(pe + 4)) ?? `desconocida (0x${cabecera.readUInt16LE(pe + 4).toString(16)})`;
}

/** Lo que dice cada archivo: `{ archivo, encontrada }` o `{ archivo, error }`. */
export function mirar(archivos) {
  return archivos.map((archivo) => {
    try {
      if (!statSync(archivo).isFile()) return { archivo, error: "no es un archivo" };
      const fd = openSync(archivo, "r");
      try {
        const buf = Buffer.alloc(4096);
        const n = readSync(fd, buf, 0, buf.length, 0);
        const encontrada = arquitecturaDe(buf.subarray(0, n));
        return encontrada ? { archivo, encontrada } : { archivo, error: "no es un ejecutable de Windows (PE)" };
      } finally {
        closeSync(fd);
      }
    } catch (e) {
      return { archivo, error: `no se pudo leer: ${e.code ?? e.message}` };
    }
  });
}

function principal(argv) {
  const i = argv.indexOf("--espera");
  const espera = i >= 0 ? argv[i + 1] : undefined;
  const archivos = argv.filter((_, k) => k !== i && k !== i + 1);
  if (!espera || !["arm64", "x64", "x86"].includes(espera)) {
    console.error("uso: node scripts/arquitectura-pe.mjs --espera <arm64|x64|x86> <archivo> [<archivo> …]");
    return 2;
  }
  if (archivos.length === 0) {
    console.error("✕ arquitectura-pe: ningún archivo que mirar — un gate que no mira nada no es verde.");
    return 1;
  }
  let mal = 0;
  for (const r of mirar(archivos)) {
    if (r.error) {
      mal++;
      console.error(`✕ ${r.archivo}: ${r.error}`);
    } else if (r.encontrada !== espera) {
      mal++;
      console.error(`✕ ${r.archivo}: es ${r.encontrada} y se esperaba ${espera}`);
    } else {
      console.log(`✓ ${r.archivo}: ${r.encontrada}`);
    }
  }
  return mal ? 1 : 0;
}

if (import.meta.url === pathToFileURL(process.argv[1] ?? "").href) process.exit(principal(process.argv.slice(2)));
