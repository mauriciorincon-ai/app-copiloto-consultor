# Anexo del ADR 023 · La medida de llama.cpp en la Surface (sprint 014, 2026-10-19)

Este anexo guarda lo que hace falta para **repetir la medida** de la enmienda 2 del
[ADR 023](../023-la-sugerencia-en-windows.md). A diferencia del de Foundry Local
([anexo de la enmienda 1](023-medida-foundry-local.md)), el arnés **vive en el repo**:
`src-tauri/tests/modelo-local-en-windows.rs`. Desde la fase 2 mide el camino de la app —`modelo_local::bajar`,
`modelo_local::cargado` y el proveedor `sintesis::local::Local` por `sugerir` y `fundar`—, no una copia.

## Cómo repetirla (en una Surface ARM64 o en cualquier PC con Windows)

1. `cd src-tauri`, con la memoria de la Surface en cuenta (`-j 4` y `CARGO_BUILD_JOBS=4`, o el enlace se queda sin
   memoria). `cargo test -j 4 --test modelo-local-en-windows` (con `AG_SIN_HARDWARE=1`) corre solo la prueba que corre
   también en la CI: llama.cpp y whisper.cpp en el mismo binario, con un solo ggml 0.9.5.
2. Con el «sí» del usuario a la fila de la descarga (1 117 320 736 B):
   `cargo test -j 4 --test modelo-local-en-windows en_vivo_baja -- --ignored --nocapture`. Baja por la puerta de la
   app, con su huella; si el servidor corta, retoma desde el último byte.
3. **En otro proceso** (la conexión de la descarga no ensucia la medida de la red):
   `cargo test -j 4 --test modelo-local-en-windows en_vivo_mide -- --ignored --nocapture`. Mide carga, RAM, la
   latencia sobre los 17 turnos del kit que dan ficha, cuántas pasan `fundar`, la canaria del turno buscada en todo lo
   escrito durante la prueba (`%LOCALAPPDATA%`, `%APPDATA%`, `%TEMP%` y el repo) y las conexiones TCP y UDP del proceso
   fase por fase (`netstat -ano`, unas tres veces por segundo). `AG_LLAMA_HILOS=<n>` cambia los hilos. Falla si el
   ChatML de la app no es la plantilla del GGUF, si la canaria aparece, si hay una conexión o si no cumple el criterio.
4. Al terminar, el modelo se queda en `%LOCALAPPDATA%\com.aiapps.copiloto-consultor\modelos\`, donde lo usa la app;
   borrar ese archivo lo quita y no deja nada más.

## Lo que salió

Qwen2.5-1.5B-Instruct Q4_K_M (`qwen2.5-1.5b-instruct-q4_k_m.gguf`, revisión `91cad51170dc346986eccefdc2dd33a9da36ead9`,
SHA-256 `6a1a2eb6d15622bf3c96857206351ba97e1af16c30d7a74ee38970e434e9407e`, apache-2.0). Surface con Snapdragon X Plus
(X1P64100, 10 núcleos, 15,6 GB), CPU.

|                                                                                   | F1 · arnés, 10 hilos               | F1 · arnés, 5 hilos              | F2 · **el proveedor de la app**, 5 hilos |
| --------------------------------------------------------------------------------- | ---------------------------------- | -------------------------------- | ---------------------------------------- |
| Carga                                                                             | 892 ms                             | 1 055 ms                         | 1 499 ms                                 |
| RAM del proceso (MB): inicio · cargado · pico · suelto                            | 8 · 1 127 · 1 230 · 17             | 8 · 1 127 · 1 228 · 17           | 7 · 1 112 · 1 227 · 18                   |
| Pedidas (preguntas del kit que dan ficha)                                         | 17 de 30                           | 17 de 30                         | 17 de 30                                 |
| `fundar`                                                                          | 12 de 17 (5 «no lo dice la ficha») | 12 de 17                         | **12 de 17** (5 «no lo dice la ficha»)   |
| En su sección                                                                     | 11 de 17                           | —                                | 11 de 17                                 |
| Latencia: primera · mediana · p95 · máx (techo 6 s)                               | 1 912 · 1 843 · 2 096 · 5 271 ms   | 2 310 · 2 171 · 2 313 · 2 381 ms | 2 145 · **2 105** · 2 321 · 2 356 ms     |
| Piezas escritas por sugerencia                                                    | 38–53                              | 38–53                            | 38–53                                    |
| Bytes fuera                                                                       | —                                  | —                                | **0**                                    |
| Red del proceso: arrancar · cargar · redactar 17+1 · reposar 60 s · soltar + 10 s | TCP 0 · UDP 0 en las cinco         | TCP 0 · UDP 0 en las cinco       | **TCP 0 · UDP 0 en las cinco**           |
| Canaria del turno                                                                 | 0 archivos (62 mirados)            | 0 archivos (33 mirados)          | **0 archivos** (42 mirados)              |
| ChatML de la app = plantilla del GGUF                                             | —                                  | —                                | **sí**                                   |

La app usa la mitad de los núcleos (`sintesis::local::hilos_para`): con 10 hilos la mediana baja a 1,8 s, pero whisper
transcribe la reunión a la vez y necesita la otra mitad; con 5, el máximo es más estable (2,4 s contra 5,3 s).

## Lo que pasó por el camino (y quedó en el código)

- **Dos copias de ggml.** `whisper-rs-sys` 0.15 y `llama-cpp-sys-2` enlazan estáticas `ggml`, `ggml-base` y `ggml-cpu`
  con el mismo nombre. Comparados archivo por archivo los árboles de CPU (sin los backends de GPU): de la 0.1.132 a la
  0.1.136, **85 de 86 iguales y las cabeceras públicas idénticas** (difiere `gguf.cpp`, su escritor); la 0.1.137 ya
  cambia 25 archivos y tres cabeceras; la 0.1.160 trae ggml 0.26. Se fijó la 0.1.136.
- **El cuelgue antes de `main`.** Con el perfil `dev`, todo binario de prueba con llama se quedaba quieto antes de
  arrancar (también `--list`; 0 % de CPU). El `build.rs` de `llama-cpp-sys-2` pide `msvcrtd` cuando él mismo se compiló
  con `debug_assertions`: el binario importaba `ucrtbased.dll` junto a la CRT de release y el enlazador avisaba con
  LNK4098. `[profile.dev.package.llama-cpp-sys-2] debug-assertions = false` lo resuelve (sin LNK4098, sin
  `ucrtbased.dll`).
- **Los cortes de la descarga.** Dos descargas seguidas se cortaron del lado del servidor («Peer disconnected») a los
  1 154 s y a los 874 s, a ≈ 1,3 MB/s. La puerta pasó a retomar desde el último byte con `Range` (hasta 5 tramos por
  petición); la tercera se cortó en 368 MB, retomó y terminó en 843 s con su tamaño y su huella.
- **Demostrado en rojo (2026-10-19), cuatro gates nuevos**, con una mutación de una línea cada uno, el fallo nombrando
  lo plantado, el archivo restaurado (sha256 idéntico) y el gate otra vez en verde. En la Surface no hay Python, así que
  `scripts/demo-rojo.sh` no corre; lo hizo un equivalente en node con los mismos candados.
  1. `ctx.state_save_file("x", &[])` en `sintesis/local.rs` → `verify-ephemeral`:
     `✕ src-tauri/src/sintesis/local.rs:165 /\b(llama_)?(save_session_file|state_save_file|state_seq_save_file)\b/`.
  2. `ureq::get(…)` en `sintesis/local.rs` → `contador-de-red`: `src-tauri/src/sintesis/local.rs:18 ureq …`.
  3. Quitar `modelo_local::soltar()` de la pieza `Sugerencia` del corte → `cargo test`: «el corte no suelta el modelo
     local».
  4. `quien_redacta` sin `admite_api()` → `cargo test`: «Windows eligió el API (api true, local false)».
- **Sin red por construcción.** El binario con solo llama no importa `ws2_32.dll`: el código de descarga que trae
  llama.cpp (`common`, `cpp-httplib`) no se enlaza.
