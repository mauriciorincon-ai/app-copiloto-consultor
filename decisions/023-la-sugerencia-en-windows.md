# ADR 023 — La sugerencia en Windows: la segunda puerta Rust a la red (retirada en el sprint 013)

- **Fecha:** 2026-10-12
- **Sprint:** 008 «Listo para el gate» (fase 1)
- **Estado:** aceptada. La pide el ADR 022 («`sintesis/api.rs` necesita su propia puerta Rust y su ADR») y enmienda su
  §4, que contaba una sola. **Enmendada por la enmienda 1 (sprint 013): en Windows el API se retiró y esta puerta con
  él.**

> **Cómo leer este ADR (sprint 013).** Todo lo que sigue hasta la **enmienda 1** describe lo que rigió del sprint 008 al 012, en
> pasado: el API y su puerta `ureq` en Windows. Desde el sprint 013 **no existen**; la enmienda 1 dice qué los reemplaza (nada,
> por ahora) y con qué cifras.

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
del usuario; (b) el modelo bajado, ≈ 1,8 GB, en `%LOCALAPPDATA%\com.aiapps.copiloto-consultor\modelos\foundry\`, **se queda a
propósito** para medir en el S14 (decisión del usuario); borrar esa carpeta lo quita y no deja nada más.

**Lo que queda por medir:** ONNX Runtime GenAI directo, sin Foundry Local Core —el mismo motor y el mismo formato de
modelo—, con su telemetría apagada por API (`OgaSetTelemetryEnabled`), el modelo bajado por una puerta Rust como la de
whisper, con su huella SHA-256, y medido con el mismo vigía y la misma canaria. Solo entra con **cero conexiones del proceso** en toda la
sesión.
