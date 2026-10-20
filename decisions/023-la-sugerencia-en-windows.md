# ADR 023 — La sugerencia en Windows: la segunda puerta Rust a la red (retirada en el sprint 013)

- **Fecha:** 2026-10-12
- **Sprint:** 008 «Listo para el gate» (fase 1)
- **Estado:** aceptada. La pide el ADR 022 («`sintesis/api.rs` necesita su propia puerta Rust y su ADR») y enmienda su
  §4, que contaba una sola. **Enmendada por la enmienda 1 (sprint 013): en Windows el API se retiró y esta puerta con
  él.** **Y por la enmienda 2 (sprint 014): en Windows redacta un modelo local, dentro del proceso y sin red.**

> **Cómo leer este ADR (sprint 013).** Todo lo que sigue hasta la **enmienda 1** describe lo que rigió del sprint 008 al 012, en
> pasado: el API y su puerta `ureq` en Windows. Desde el sprint 013 **no existen**; la enmienda 1 dice qué los reemplaza (nada,
> por ahora) y con qué cifras; la **enmienda 2**, cómo volvió la sugerencia sin abrir ninguna conexión.

## Contexto (hasta el sprint 012)

En el Mac, la sugerencia redactada por un proveedor externo (ADR 011) sale por el puente de Swift: `nativo/Red.swift`,
una sesión `URLSession` efímera, la tercera puerta de `contador-de-red`. En Windows no hay puente de Swift, y el modelo
del sistema tampoco existe allí (`sintesis/sistema.rs` dice «sin puente»). Hasta el sprint 007, con el API encendido,
la sugerencia en Windows fallaba con «esta compilación no trae el puente de red». La única puerta Rust era la del
modelo de whisper (ADR 022 §4), que no lleva texto: solo entra el modelo.

La regla dura 2 no cambia: el API nace apagado, lleva la clave del usuario, sale solo texto minimizado y anonimizado
en el equipo, con la retención publicada por el proveedor enseñada donde se elige, y cada byte se cuenta.

## Decisión (vigente del sprint 008 al 012; ver la enmienda 1)

1. **Una segunda puerta Rust, y solo una más**: `ureq` (con rustls y sus raíces webpki), el mismo cliente que la del
   modelo, **solo** bajo `cfg(windows)` y **solo** en `src-tauri/src/sintesis/api.rs`, en **una línea** marcada con
   `verify-ephemeral:allow` y este ADR. `sintesis` sigue siendo un módulo protegido: `verify-ephemeral` prohíbe
   `ureq::` en todos los protegidos y esa línea es la excepción declarada.
2. **El envío es el mismo en los dos sistemas** (`Api::redactar_con`): el corte se mira antes y después de armar; se
   tapa en el equipo (`Boveda`); la clave va en las cabeceras y se pisa; los bytes se cuentan con
   `red::registrar_salida` **antes** de salir; lo que salió se anota para IA aunque el proveedor no conteste; la
   respuesta se destapa en el equipo y pasa por `fundar()`. Solo cambia la puerta, que es un parámetro: el puente de
   Swift en el Mac, `ureq` en Windows, y una puerta falsa en las pruebas.
3. **La puerta de Windows** (`por_la_puerta`): un `POST` https a la URL del proveedor elegido, con el mismo techo que el
   puente (el de la síntesis más un segundo), **sin redirecciones** (no la llevan a otro sitio), sin caché ni cookies,
   y lee como mucho 64 KB de respuesta. Los errores HTTP se leen como respuesta, para decir cuál fue.
4. **El gate se estrecha**: `contador-de-red` pasa de una puerta Rust a una lista de dos, cada una con su archivo y su
   número exacto de líneas. Una línea de red más —en esos archivos o en otro— es rojo (demostrado con un `ureq::get`
   plantado en `red.rs` y otro duplicado en `api.rs`).

## Consecuencias (del sprint 008; ver la enmienda 1)

- «Hoy en Windows»: la sugerencia es «sí, con el API y tu clave»; sin API, la ficha sigue siendo la respuesta, como en
  el Mac sin Apple Intelligence.
- El contador de Honestidad sigue en 0 B mientras el API esté apagado; encendido, cuenta lo que sale, igual que en el
  Mac.
- **Sin verificar, para el PC:** la salida real a Groq o a Claude con tu clave (matriz de la regla 24: la clave en el
  Administrador de credenciales y la salida a la red), y que un proxy de empresa no la corte.

## Alternativas descartadas (en el sprint 008)

- **WinHTTP o WinINet por el crate `windows`.** Sin dependencia nueva de red, pero con proxy, caché y credenciales del
  sistema que habría que apagar una a una; `ureq` ya está en el árbol, auditado por `cargo audit`, y es la misma pieza
  que la otra puerta.
- **Pasar la sugerencia por la webview (`fetch`).** Sacaría la clave y el texto al proceso de la webview y rompería
  `contador-de-red` en el webview, que hoy no tiene ninguna salida.
- **Dejar la sugerencia en «no en Windows».** Posible, pero el H-1 promete Windows completo y la pieza es pequeña y
  probable sin red.

## Enmienda 1 · El API sale de Windows; el modelo local, medido y aplazado (sprint 013, 2026-10-18)

**El API se retira de Windows** (decisión 1a del usuario en el G-Plan del S13: ningún API de pago por uso en Windows).

- La puerta `ureq` de `sintesis/api.rs` (§1 y §3) ya no existe; `contador-de-red` vuelve a **una** puerta Rust, la del
  modelo de whisper. Demostrado en rojo: un `ureq::get` plantado en `api.rs`, con su marca y este ADR, es rojo (la salida de la corrida, en el
  [anexo](anexos/023-medida-foundry-local.md)).
- `sintesis::quien_redacta` recibe la plataforma como parámetro y en Windows **nunca** elige `Api` (probado con las dos
  plataformas en cualquier equipo). En el Mac, igual que antes.
- En Windows, `api_externa` (encender) y `guardar_clave_del_api` contestan `sin-api-en-windows`; las preferencias no
  encienden el API, y la app **ni pregunta** al Administrador de credenciales por la clave. La que hubiera de antes no
  se toca: borrarla sigue en tu mano (el manual dice cómo).

**El reemplazo que se midió y no entra: Foundry Local** (`foundry-local-sdk` 2.1.0, MIT), en proceso, en la CPU de la
Surface (Snapdragon X, ARM64). Modelo `qwen2.5-1.5b-instruct-generic-cpu:4` (1 842 MB): carga en 3,2 s, ocupa 1,1 GB de
RAM y redacta con mediana de 2,1 s y p95 de 2,3 s (techo 6 s); 6 de 17 sugerencias del kit pasan `fundar`. La canaria
del turno no apareció en ningún archivo. Cabe en el tiempo, **pero no en la regla dura 2**:

1. Su runtime (Foundry Local Core y ONNX Runtime GenAI) abre una conexión HTTPS al recolector de telemetría de
   Microsoft (1DS, `mobile.events.data.microsoft.com`) al crearse, y la conexión sigue abierta mientras el modelo carga,
   redacta, espera y se cierra (vigía `netstat` del proceso, fase por fase).
2. Con sus interruptores (`DisableNonessentialTelemetry`, `ORT_TELEMETRY_DISABLED`, `ORT_DISABLE_TELEMETRY`) dejan de
   encolarse los eventos de uso, pero la conexión sigue y el evento «ProcessInfo» sube igual (modelo del equipo,
   identificador del dispositivo con hash, versión del sistema, nombre del proceso): lo dice el propio binario
   («ProcessInfo still uploads»).
3. Consulta el catálogo por la red cada vez que se crea (`ai.azure.com`, `api.catalog.azureml.ms`).
4. Escribe fuera de la carpeta que se le da: colas SQLite en `%LOCALAPPDATA%\Microsoft\DeveloperTools\.onnxruntime\`.

Nada de eso pasa por `red.rs` —es código nativo—, así que Honestidad diría «0 B» sin serlo. **En Windows la sugerencia
redactada queda «no por ahora»**, con estas cifras; la ficha sigue siendo la respuesta (regla dura 3). La NPU no se
probó: pedía instalar execution providers para todo el sistema desde la Microsoft Store, y la CPU ya cabía en el tiempo.
El arnés de la medida —`src-tauri/tests/modelo-local-en-windows.rs` y las dos líneas de configuración que pedía— se guarda
como [anexo](anexos/023-medida-foundry-local.md) con los pasos para repetirla: el commit `e1e0de2` vive en la rama del sprint, que
el merge con squash borra, y una medida que nadie puede repetir no es una medida.

**Restos en la Surface de pruebas** (el runtime escribe fuera de la carpeta de la app, y eso se dice): (a) las colas de
telemetría de Microsoft, `%LOCALAPPDATA%\Microsoft\DeveloperTools\.onnxruntime\`, **se borraron** al cerrar la medida, con el «sí»
del usuario; (b) el modelo bajado, ≈ 1,8 GB, en `%LOCALAPPDATA%\com.aiapps.copiloto-consultor\modelos\foundry\`, **se quedó a propósito** para medir en el S14 (decisión del usuario) y **se retiró al cierre del S14**, con su «sí» (enmienda 2).

**Lo que queda por medir:** ONNX Runtime GenAI directo, sin Foundry Local Core —el mismo motor y el mismo formato de
modelo—, con su telemetría apagada por API (`OgaSetTelemetryEnabled`), el modelo bajado por una puerta Rust como la de
whisper, con su huella SHA-256, y medido con el mismo vigía y la misma canaria. Solo entra con **cero conexiones del proceso** en toda la
sesión.

## Enmienda 2 · La sugerencia vuelve a Windows: llama.cpp dentro del proceso, sin red (sprint 014, 2026-10-19)

**Lo que redacta en Windows: Qwen2.5-1.5B-Instruct (GGUF, Q4_K_M, apache-2.0) por llama.cpp, dentro del proceso de la
app, en la CPU.** Ni API, ni runtime de terceros con telemetría, ni una conexión: el proceso no abre ninguna en ninguna
fase (vigía `netstat` del proceso, como en la enmienda 1). Honestidad dice «0 B» y es verdad: nada pasa por `red.rs`
porque nada sale.

| Medido en la Surface (Snapdragon X Plus, 10 núcleos, 15,6 GB; CPU) | Resultado | Criterio de entrada |
|---|---|---|
| Archivo | `qwen2.5-1.5b-instruct-q4_k_m.gguf`, 1 117 320 736 B, SHA-256 `6a1a2eb6…407e`, revisión `91cad51…` | — |
| Compila en ARM64 (clang-cl, Ninja, `-j 4`) | llama.cpp en ≈ 2 min; enlaza junto a whisper con **un solo ggml 0.9.5** | — |
| Carga · RAM | ≈ 1,5 s · +1,1 GB (pico 1,23 GB); al soltarlo, vuelve | — |
| Latencia, 17 turnos del kit (la app usa 5 hilos) | mediana **2,1 s**, p95 2,3 s, máx 2,4 s (con 10 hilos: 1,8 · 2,1 · 5,3 s) | < 6 s ✅ |
| `fundar` | **12 de 17** (Foundry Local, mismo modelo en ONNX: 6 de 17) | ≥ 9 ✅ |
| Conexiones del proceso (arrancar · cargar · redactar · reposar 60 s · soltar) | **TCP 0 · UDP 0 en las cinco** | 0 ✅ |
| Canaria del turno | **0 archivos** de los escritos durante la prueba | 0 ✅ |
| **En vivo, en la app real** (3 corridas, con la pregunta del kit por los altavoces; anexo) | **1 de 3** dentro del techo (2,3 s, en la banda); **2 de 3 pasaron de 6 s con el PC muy cargado** y se descartaron (la ficha ya estaba); causa **sin atribuir**. Con la corrida buena: TCP 0 · UDP 0 en las cinco fases, 0 B | — (viaja a w26, con auriculares, y al gate del hito) |

La tabla completa, el arnés y cómo repetirla: [anexo](anexos/023-medida-llama-cpp.md). El arnés vive en el repo esta vez
(`src-tauri/tests/modelo-local-en-windows.rs`) y desde la fase 2 mide **el camino de la app**, no una copia.

**Decisión.**

1. **`llama-cpp-2` y `llama-cpp-sys-2` fijados con `=0.1.136`, sin features** (ni OpenMP, ni GPU; `LLAMA_CURL=OFF` lo
   pone el crate). Es la última versión cuyo ggml vendorizado es el de `whisper-rs-sys` 0.15: cabeceras públicas
   idénticas y 85 de 86 fuentes de CPU iguales (difiere solo el escritor de GGUF, que la app no usa). Los dos crates
   enlazan estáticas `ggml`, `ggml-base` y `ggml-cpu` con el mismo nombre y el binario se queda con una sola copia;
   subir uno exige subir el otro a la par (lo vigila `llama_y_whisper_enlazan_con_un_solo_ggml`, en la CI).
2. **El perfil `dev` apaga `debug-assertions` para `llama-cpp-sys-2`**: su `build.rs` añade `msvcrtd` cuando él mismo
   se compiló con ellas, y mezclar la CRT de depuración con la de release colgaba antes de `main` todo binario de
   prueba (visto en la Surface: `ucrtbased.dll` en las importaciones, LNK4098). En `--release` ya están apagadas.
3. **Sin una puerta nueva** (desviación del archivo del sprint, que pedía una segunda): el GGUF baja por la línea de
   whisper, `modelo_whisper::bajar`, con su origen y revisión fijos, tamaño, SHA-256, espacio y guarda de reunión.
   `contador-de-red` sigue contando **una** puerta Rust. Desde este sprint **retoma un corte** con `Range` desde el
   último byte (hasta 5 tramos por petición): el servidor cortó dos de dos descargas a los 15 y 19 minutos, y sin retomar
   cada corte empezaba de cero. La huella se calcula igual, sobre el archivo entero.
4. **El proveedor** es `sintesis/local.rs` (módulo protegido): `Quien::Local`, una generación por sugerencia con el
   prompt de la app en ChatML (el formato del modelo, escrito en Rust para no dejar copias del turno sin pisar; idéntico
   a la plantilla del GGUF, comprobado en vivo), temperatura 0, como mucho 160 piezas, con la mitad de los núcleos (la
   otra mitad le queda a whisper). Un contexto por sugerencia, con su caché vaciada antes de soltarlo; el prompt, las
   piezas y la salida se pisan con ceros; los registros de llama.cpp, apagados. `verify-ephemeral` prohíbe en los
   protegidos las funciones de llama.cpp que guardan su estado en un archivo. `fundar` y el techo de 6 s, como siempre: pasado el techo la generación **se para** en vez de seguir quitándole CPU a whisper, y lo escrito se pisa también si sale con un error.
5. **El modelo vive lo que la sesión**: se carga al empezar con «Redactar sugerencias» encendido (o la primera vez que
   hace falta) y se suelta al terminar, al apagar «Redactar» y con el corte —dentro de la pieza `Sugerencia`, que el
   `match` sin comodín obliga a resolver; desviación del plan, que pedía una pieza nueva—. Un corte que llega durante la carga **no espera a la carga** y el modelo no queda residente; y un hilo de sugerencia lanzado antes del corte no lo vuelve a cargar.
6. **Lo que se promete en Windows**: «Redactar sugerencias», solo con el modelo instalado; **«Enriquecer el banco» sigue
   «no por ahora»** (no se midió para el banco); el API, jamás (`quien_redacta` no lo elige en Windows, con prueba).
   «Hoy en Windows» vuelve a las 15 en «sí».

**Lo que la app hace sola al cambiar de estado** (matriz de la regla 24): al arrancar borra los `.parcial` de
`modelos\` y apaga lo recordado que Windows no tiene (el API siempre; «redactar», si el modelo no está); al empezar una
sesión con «Redactar» encendido carga el modelo en RAM, sin red; al terminar, al apagar «Redactar» o con el corte, lo
suelta. Solo «Install the model», en IA y nunca con una reunión, una sesión o un ensayo abiertos, abre la conexión de la
descarga.

**Descartados.** Foundry Local (enmienda 1: telemetría a Microsoft con sus interruptores apagados). Qwen2.5-3B (su
licencia, `qwen-research`, no es comercial). ONNX Runtime GenAI directo y Phi-3-mini (MIT) eran los planes B: no
hicieron falta. La NPU no se probó: la CPU cabe en el techo. **Retirado de la Surface al cierre del S14**, con el «sí» del usuario, ruta por ruta: el modelo de Foundry Local de la enmienda 1 (≈ 1,8 GB), las compilaciones de su crate y su entrada del registro de cargo, las compilaciones viejas de `llama-cpp-sys-2` hechas antes de la corrección del CRT, el `ggml-large-v3-turbo.bin` (que en esta Surface no cabe en el techo de 4 s) y mis copias de trabajo; la búsqueda de rastros termina en 0 (`*.gguf`/`*.onnx` solo el modelo en uso; ninguna carpeta `foundry` ni `.onnxruntime`; `Cargo.toml` y `Cargo.lock` sin foundry ni ort).
