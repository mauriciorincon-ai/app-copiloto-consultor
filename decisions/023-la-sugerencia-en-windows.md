# ADR 023 — La sugerencia en Windows: la segunda puerta Rust a la red

- **Fecha:** 2026-10-12
- **Sprint:** 008 «Listo para el gate» (fase 1)
- **Estado:** aceptada. La pide el ADR 022 («`sintesis/api.rs` necesita su propia puerta Rust y su ADR») y enmienda su
  §4, que contaba una sola.

## Contexto

En el Mac, la sugerencia redactada por un proveedor externo (ADR 011) sale por el puente de Swift: `nativo/Red.swift`,
una sesión `URLSession` efímera, la tercera puerta de `contador-de-red`. En Windows no hay puente de Swift, y el modelo
del sistema tampoco existe allí (`sintesis/sistema.rs` dice «sin puente»). Hasta el sprint 007, con el API encendido,
la sugerencia en Windows fallaba con «esta compilación no trae el puente de red». La única puerta Rust era la del
modelo de whisper (ADR 022 §4), que no lleva texto: solo entra el modelo.

La regla dura 2 no cambia: el API nace apagado, lleva la clave del usuario, sale solo texto minimizado y anonimizado
en el equipo, con la retención publicada por el proveedor enseñada donde se elige, y cada byte se cuenta.

## Decisión

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

## Consecuencias

- «Hoy en Windows»: la sugerencia es «sí, con el API y tu clave»; sin API, la ficha sigue siendo la respuesta, como en
  el Mac sin Apple Intelligence.
- El contador de Honestidad sigue en 0 B mientras el API esté apagado; encendido, cuenta lo que sale, igual que en el
  Mac.
- **Sin verificar, para el PC:** la salida real a Groq o a Claude con tu clave (matriz de la regla 24: la clave en el
  Administrador de credenciales y la salida a la red), y que un proxy de empresa no la corte.

## Alternativas descartadas

- **WinHTTP o WinINet por el crate `windows`.** Sin dependencia nueva de red, pero con proxy, caché y credenciales del
  sistema que habría que apagar una a una; `ureq` ya está en el árbol, auditado por `cargo audit`, y es la misma pieza
  que la otra puerta.
- **Pasar la sugerencia por la webview (`fetch`).** Sacaría la clave y el texto al proceso de la webview y rompería
  `contador-de-red` en el webview, que hoy no tiene ninguna salida.
- **Dejar la sugerencia en «no en Windows».** Posible, pero el H-1 promete Windows completo y la pieza es pequeña y
  probable sin red.
