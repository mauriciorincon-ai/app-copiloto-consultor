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
// `voz` y `src-tauri/nativo` se añadieron en el sprint 001, fase 3. `voz` parte la voz del
// cliente en turnos: no la guarda, pero la tiene entera en las manos, que es lo mismo desde el
// lado de la regla. Y `nativo/` es el puente de Swift hacia el transcriptor — sin esa línea, el
// único archivo del producto que llama a una API de descarga de Apple sería el único sin barrer,
// y la vigilancia se habría detenido justo en la frontera del lenguaje.
const PROTEGIDOS = [
  "src-tauri/src/capture",
  "src-tauri/src/stt",
  "src-tauri/src/voz",
  // El disparador guarda la última pregunta del CLIENTE para no repetir ficha, y la ficha se
  // arma con sus palabras. Los dos manejan contenido de terceros: ni disco ni red.
  "src-tauri/src/disparo",
  "src-tauri/src/ficha",
  "src-tauri/src/screen",
  "src-tauri/src/sesion",
  "src-tauri/nativo",
  "src/capture",
];
// API prohibida dentro de los protegidos (Rust y TS). Se puede ampliar; jamás recortar sin ADR.
const PROHIBIDO = [
  /std::fs\b/, /tokio::fs\b/, /File::create\b/, /OpenOptions\b/, /\bfs::write\b/, /\bwrite_all\b/,
  /std::net\b/, /TcpStream\b/, /UdpSocket\b/, /\breqwest\b/, /\bhyper\b/, /tauri_plugin_fs\b/,
  /tauri_plugin_store\b/, /tauri_plugin_http\b/, /rusqlite\b/, /sqlx\b/,
  /\bfetch\(/, /XMLHttpRequest\b/, /WebSocket\b/, /localStorage\b/, /indexedDB\b/, /writeFile\b/,
  // Swift y Objective-C (fase 3): el puente del transcriptor vive en Swift y su API de disco y de
  // red no se parece en nada a la de Rust. Sin estas líneas el barrido leía el archivo y no veía
  // nada, que es la peor forma de pasar: verde por no saber mirar.
  /\bFileManager\b/, /\bURLSession\b/, /\bNSURLConnection\b/, /contentsOf:/, /\bwrite\(to:/,
  /\bNWConnection\b/, /\bCFSocket/, /\bNSFileHandle\b/, /\bUserDefaults\b/,
  // La descarga del modelo de reconocimiento que hace macOS. Es legítima y necesaria, y por eso
  // NO se prohíbe a secas: se obliga a que la línea lleve su marca y su ADR. Una puerta a la red
  // en un módulo efímero puede existir; lo que no puede es existir sin que se vea.
  /\bdownloadAndInstall\b/, /\bassetInstallationRequest\b/,
];
const ALLOW = /verify-ephemeral:allow\b/; // línea explícitamente autorizada (exige ADR citado en la misma línea)

function archivos(dir) {
  if (!existsSync(dir)) return [];
  return readdirSync(dir).flatMap((n) => {
    const p = join(dir, n);
    return statSync(p).isDirectory() ? archivos(p) : /\.(rs|ts|tsx|js|mjs|swift|m|mm)$/.test(n) ? [p] : [];
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
