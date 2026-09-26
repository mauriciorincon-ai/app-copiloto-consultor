#!/usr/bin/env node
// verify-ephemeral — gate del estándar 4-T «captura de terceros» (kit v1.27.1).
//
// ESTE ARCHIVO ES LA MITAD ESTÁTICA. Lee el código y prohíbe API de disco y de red en los
// módulos protegidos. Necesario, y NO suficiente: no ve lo que escriben las librerías de Apple
// por debajo, ni un temporal que nazca dentro del puente de Swift, ni un log que se lleve una
// frase del cliente. La otra mitad mira el DISCO —inventario antes y después de una sesión
// completa, con una canaria que solo dice el cliente— y vive en
// `src-tauri/tests/contra-el-mac-de-verdad.rs`. Se corre con `pnpm verify:ephemeral:runtime`, y
// en la integración continua la arrastra `cargo test`. Decirlo aquí no es cortesía: sin esta
// nota, un verde de este script se lee como «la promesa está verificada», y no lo está.
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
// `escucha` se añadió en la FASE 2 DE LA AUDITORÍA del sprint 001, y es el hallazgo A4: su
// cabecera decía «**MÓDULO PROTEGIDO.** … y `pnpm verify:ephemeral` lo comprueba» **y no estaba en
// esta lista**. Es el módulo de mayor superficie de los tres —copia el audio del turno, mantiene la
// ventana de transcript y guarda la última pregunta del cliente—, así que un `fs::write` ahí pasaba
// el gate en verde. De ahí sale también la comprobación de abajo: que la cabecera y esta lista no
// puedan volver a decir cosas distintas.
const PROTEGIDOS = [
  "src-tauri/src/capture",
  "src-tauri/src/escucha",
  "src-tauri/src/stt",
  "src-tauri/src/voz",
  // El disparador guarda la última pregunta del CLIENTE para no repetir ficha, y la ficha se
  // arma con sus palabras. Los dos manejan contenido de terceros: ni disco ni red.
  "src-tauri/src/disparo",
  // `diccionario` se añadió en el sprint 002, fase 1, y es el caso más interesante de la lista:
  // **el diccionario PERSISTE** —es del consultor, como sus notas— y aun así el módulo está aquí.
  // Recibe cada turno del cliente y devuelve el turno corregido, así que tiene el transcript en las
  // manos; lo que hace es serializarse a un `String` y dejar que `lib.rs` escriba el archivo, que es
  // la capa que no ve un solo turno. El plan del sprint decía «`diccionario/` puede tocar disco»;
  // esto es más fuerte y cuesta lo mismo.
  "src-tauri/src/diccionario",
  // `habla` se añadió en el sprint 002, fase 2, y es el caso raro de la lista: lo que dice en voz
  // alta es texto del CORPUS DEL USUARIO, no del cliente, así que por la frontera del ADR 002 no le
  // tocaría. Está aquí por la API: `AVSpeechSynthesizer` trae `write(_:toBufferCallback:)`, que
  // convierte lo que va a decir en **búferes de audio** — es decir, una manera de dejar en un
  // archivo la evidencia del consultor leída en voz alta. Eso sería una grabación de la reunión con
  // otro nombre. El módulo no la usa y desde aquí no puede empezar a usarla en silencio.
  "src-tauri/src/habla",
  "src-tauri/src/ficha",
  // `pantalla` es la ranura que el sprint 001 reservó como `screen` y dejó vacía; el sprint 002, fase
  // 3, la llenó con el nombre en español que usa el resto de la casa. Es el módulo de más superficie
  // de la lista después de `escucha`: tiene en las manos cuadros de la ventana de la reunión y el
  // texto que Vision leyó de ellos. El gate de abajo lo cazó en su primera corrida: la cabecera ya
  // decía «MÓDULO PROTEGIDO» y la ranura seguía llamándose `screen`.
  "src-tauri/src/pantalla",
  // `radar` (sprint 002, fase 4): su mitad ámbar recibe las líneas que Vision leyó de la ventana de
  // la reunión —nombres de participantes incluidos— para buscar el aviso de grabación y los bots.
  // Texto de un tercero en las manos: ni disco ni red. Y además la regla dura 9 le prohíbe mirar
  // fuera de este Mac, que vigila su propio gate (`tests/unit/radar-solo-este-mac.test.ts`).
  // Su catálogo vive en `data/radar/` y entra con `include_str!` al compilar: no hay archivo que
  // leer en tiempo de ejecución. Lo cazó la comprobación de abajo en su primera corrida.
  "src-tauri/src/radar",
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
  // La PANTALLA (sprint 002, fase 3): las maneras que tienen Apple de convertir un cuadro de la
  // reunión en algo que sobreviva a la memoria. `CGImageDestination` y las representaciones de
  // `NSBitmapImageRep` lo hacen imagen (PNG, JPEG, TIFF); `SCRecordingOutput` —macOS 15— graba la
  // ventana capturada directamente a un vídeo en disco, y `AVAssetWriter` escribe cualquier vídeo.
  // Ninguna se usa, y desde aquí ninguna puede empezar a usarse sin que se vea.
  /\bCGImageDestination/, /\bNSBitmapImageRep\b/, /\bpngData\b/, /\bjpegData\b/,
  /\btiffRepresentation\b/, /\bSCRecordingOutput\b/, /\bAVAssetWriter\b/,
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
// ---------------------------------------------------------------------------------------------
// Y el gate del propio gate: **quien se declara protegido tiene que estar vigilado.**
//
// La lista de arriba se escribe a mano y el sprint 001 demostró lo que eso significa: `escucha`
// llevaba dos fases afirmando en su cabecera que este script lo comprobaba, sin estar en la lista.
// Un módulo que se cree vigilado es peor que uno que se sabe descubierto — nadie va a mirarlo.
// Así que la marca «MÓDULO PROTEGIDO» del código es la que manda: si un archivo la lleva, su
// carpeta está en `PROTEGIDOS` o esto falla.
const MARCA = /MÓDULO PROTEGIDO/;
const CANDIDATOS = ["src-tauri/src", "src-tauri/nativo", "src"];
let mentirosos = 0;
for (const raiz of CANDIDATOS) {
  for (const f of archivos(raiz)) {
    if (!MARCA.test(readFileSync(f, "utf8"))) continue;
    const ruta = relative(".", f);
    if (PROTEGIDOS.some((d) => ruta.startsWith(d + "/") || ruta === d)) continue;
    mentirosos++;
    console.error(
      `✕ ${ruta} se declara «MÓDULO PROTEGIDO» y NO está en la lista de este script: o entra en ` +
        `PROTEGIDOS, o su cabecera deja de afirmarlo.`,
    );
  }
}
if (mentirosos) process.exit(1);

const existentes = PROTEGIDOS.filter((d) => existsSync(d));
console.log(`verify:ephemeral — módulos protegidos presentes: ${existentes.length ? existentes.join(", ") : "ninguno aún"} · archivos inspeccionados: ${inspeccionados}`);
if (hallazgos) { console.error(`✕ ${hallazgos} uso(s) de disco/red en módulos efímeros. Regla dura 1 (estándar 4-T).`); process.exit(1); }
console.log("✓ cero API de disco o red en los módulos efímeros (verificación estática)");
console.log(
  "· la mitad EN MARCHA (inventario del disco tras una sesión completa) no está en este script:\n" +
    "  `pnpm verify:ephemeral:runtime` · en CI la arrastra `cargo test`",
);
