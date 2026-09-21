#!/usr/bin/env node
// verify-ephemeral — gate del estándar 4-T «captura de terceros» (kit v1.27.1).
// Corre en CI (job build-escritorio) y en /release-check cuando CLAUDE.md declara
// `captura_terceros: true`. Falla (exit 1) si algún módulo que toca audio, transcript o
// pantalla de terceros usa API de DISCO o de RED. Es estático y determinista: crece con el
// código (el S1 añade la verificación en RUNTIME: sesión completa ⇒ cero archivos nuevos
// fuera de la carpeta de notas). Sin este script, la CI falla: un gate saltado se ve igual
// que uno verde.
import { readdirSync, readFileSync, statSync, existsSync } from "node:fs";
import { join, relative } from "node:path";

// Módulos protegidos: TODO lo que vive aquí es efímero (RAM) por definición.
//
// `sesion` se añadió en el sprint 001, fase 2, y conviene decir por qué: no toca audio ni
// pantalla, pero **lee títulos de ventana** para distinguir «tienes una reunión de Meet» de
// «tienes Chrome abierto», que es siempre cierto. El título de una reunión es información del
// cliente — el estándar 4-T divide por DE QUIÉN es, no por su formato— así que cae del mismo lado
// que el transcript y vive bajo la misma regla: memoria mientras la pantalla lo muestra, y nada
// más. Sin esta línea, el módulo que maneja nombres de reuniones sería el único sin vigilancia.
const PROTEGIDOS = [
  "src-tauri/src/capture",
  "src-tauri/src/stt",
  "src-tauri/src/screen",
  "src-tauri/src/sesion",
  "src/capture",
];
// API prohibida dentro de los protegidos (Rust y TS). Se puede ampliar; jamás recortar sin ADR.
const PROHIBIDO = [
  /std::fs\b/, /tokio::fs\b/, /File::create\b/, /OpenOptions\b/, /\bfs::write\b/, /\bwrite_all\b/,
  /std::net\b/, /TcpStream\b/, /UdpSocket\b/, /\breqwest\b/, /\bhyper\b/, /tauri_plugin_fs\b/,
  /tauri_plugin_store\b/, /tauri_plugin_http\b/, /rusqlite\b/, /sqlx\b/,
  /\bfetch\(/, /XMLHttpRequest\b/, /WebSocket\b/, /localStorage\b/, /indexedDB\b/, /writeFile\b/,
];
const ALLOW = /verify-ephemeral:allow\b/; // línea explícitamente autorizada (exige ADR citado en la misma línea)

function archivos(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? archivos(p) : /\.(rs|ts|tsx|js|mjs)$/.test(n) ? [p] : [];
  });
}

let hallazgos = 0, inspeccionados = 0;
for (const dir of PROTEGIDOS) {
  const lista = archivos(dir);
  inspeccionados += lista.length;
  for (const f of lista) {
    readFileSync(f, "utf8").split("\n").forEach((linea, i) => {
      if (ALLOW.test(linea)) return;
      for (const re of PROHIBIDO) if (re.test(linea)) { hallazgos++; console.error(`✕ ${relative(".", f)}:${i + 1}  ${re}  →  ${linea.trim()}`); }
    });
  }
}
const existentes = PROTEGIDOS.filter((d) => existsSync(d));
console.log(`verify:ephemeral — módulos protegidos presentes: ${existentes.length ? existentes.join(", ") : "ninguno aún"} · archivos inspeccionados: ${inspeccionados}`);
if (hallazgos) { console.error(`✕ ${hallazgos} uso(s) de disco/red en módulos efímeros. Regla dura 1 (estándar 4-T).`); process.exit(1); }
console.log("✓ cero API de disco o red en los módulos efímeros (verificación estática; la de runtime llega con el S1)");
