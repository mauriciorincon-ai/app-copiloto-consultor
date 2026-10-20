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

## La corrida en vivo en la app (F3, 2026-10-19)

La app real en la Surface (`pnpm tauri dev`, manejada por CDP), con «Draft suggestions» encendido, **Start session**
(micrófono y audio del sistema) y `pregunta-es.wav` del kit sonando por los altavoces (6,5 s). No había auriculares:
el micrófono pudo oír los altavoces (eco). Tres corridas:

| Corrida | Contexto | Generación | Avisos de buffer de la captura |
| --- | --- | --- | --- |
| 1 | app recién compilada | pasó del techo: 6 023 ms → `Tarde`, sin sugerencia | 77 |
| 2 | la misma app, otra sesión | 6 025 ms → `Tarde`, sin sugerencia | 48 |
| 3 | tras recompilar con la línea de log nueva | **2 311 ms** (49 piezas sobre 288 de entrada); en la banda 5,3 s después de acabar la pregunta | 2 |

La corrida 3, fase por fase (proceso de la app; `netstat -ano`): RAM 70 → 70 → **1 178** (modelo cargado al empezar) →
1 940 (con la sugerencia; incluye whisper pequeño, que se carga con el primer turno) → **76** (End session lo suelta);
TCP del proceso **0** y UDP **0** en las cinco fases; contador de la reunión **0 B**; el corte (`cortar_todo`) suelta el
modelo (1 182 → 76 MB); credencial, tarea y preferencias idénticas al terminar. Latencia leída de la app: 2 312 ms.
La banda mostró la ficha «Alcance» y debajo «suggestion · on your PC · high confidence» (leída como imagen).

Lo que enseñó, y lo que se pagó:

- **El «6 023 ms» del log era el corte, no la medida.** `sugerir` espera con `recv_timeout`; la generación seguía detrás,
  sin que nadie la mirara. `Local::redactar` ahora anota `[modelo-local] generó N piezas sobre M de entrada en X ms`
  (solo cifras, ADR 003), también cuando el techo ya la descartó.
- **Con el PC muy cargado la sugerencia puede no salir, y la ficha ya está.** En el arnés, aislado, la mediana es 2,1 s;
  en la app, con la captura desbordándose, la generación pasó de los 6 s en dos de tres corridas. Es la degradación
  prevista (ADR 010: sin sugerencia, la ficha), no un fallo, pero hay que decirlo: los manuales, la guía y el BLUEPRINT
  lo dicen. **La causa de las dos lentas no se pudo atribuir con certeza**: coinciden con muchos avisos de buffer (77 y 48
  contra 2 en la buena), señal de un PC cargado; en la 1 se transcribía a la vez un turno de eco del micrófono (4,0 s,
  «marcado como eco»); en la 2 el registro no muestra ningún turno concurrente. No se subió el techo (es de la orden).
  Queda para w26, con auriculares, y para el gate del hito.
- **La generación descartada seguía quemando CPU** que whisper necesita para la ficha del turno siguiente. Ahora el bucle
  de `escribir` mira el plazo antes de pedir cada pieza (`paso_el_plazo`, el mismo techo) y se para; lo prueban una
  función pura, una comprobación de fuente y, en rojo, la mutación que quita el bloque.
- **IA y la banda pintaban el logotipo de Apple en Windows** junto a «en tu PC» y «en uso» (el icono `i-mac`, que el Mac
  usa para el modelo del sistema): ningún test miraba iconos. Con el modelo local pintan un candado; un test del IA de
  Windows exige que no salga `#i-mac` (rojo antes, verde después).
- **El árbol de WebView2 sí tiene conexiones, y no son del modelo** (auditoría M-5). Medido en dos arranques en frío
  (sin sesión ni modelo, 60 s en reposo, 18 muestras cada uno): el proceso de Rust (`app-copiloto-consultor.exe`) tiene
  **0** sockets; los procesos `msedgewebview2.exe` que cuelgan de la app (`--webview-exe-name=app-copiloto-consultor.exe`,
  su carpeta `EBWebView`) mantienen **3** conexiones TCP establecidas a Microsoft (puerto 443; 150.171.x.x y 52.96.x.x), las
  mismas en los dos arranques: el proceso del navegador y su servicio de red, el motor de Microsoft Edge que Tauri usa para
  pintar la ventana. Es anterior al S14 y el modelo no lo cambia (está igual con el modelo sin cargar). La corrida de la
  sesión había dicho 0 porque miró cinco instantes y esas conexiones son intermitentes: queda corregida. Una prueba con
  flags de WebView2 (`--disable-background-networking --disable-component-update --disable-sync …`) bajó de 2 a 1; no se
  aplicó, porque cambiar el motor de la ventana es una decisión del usuario sobre el producto. Honestidad cuenta lo que
  sale por la app (0 B), no lo que hace Edge, y hoy ningún documento lo dice; qué llevan esas conexiones no se ha mirado.

No cubre: una videollamada real con auriculares (w26 de la guía) ni la voz de una persona; viaja al gate del hito.
