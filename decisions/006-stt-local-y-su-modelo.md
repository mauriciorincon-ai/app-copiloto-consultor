# ADR 006 — El motor de transcripción: SpeechAnalyzer, medido contra su alternativa

- **Fecha:** 2026-09-21
- **Sprint:** 001 «La banda y la ficha», fase 3
- **Estado:** aceptada

## Contexto

La orden del sprint dejó la elección abierta con una condición escrita: *«Apple SpeechAnalyzer
(macOS 26+) vía bridge, con respaldo `whisper-rs` (large-v3-turbo) — **el motor se decide por ADR
con medición**, no por preferencia»*. Y el presupuesto del sprint cuelga de ella: **≤4 s desde el
fin de turno hasta la ficha**, contados desde un instante que decide `voz::turno`.

Las dos opciones no se parecen en nada salvo en el resultado:

| | Apple SpeechAnalyzer | whisper-rs (large-v3-turbo) |
|---|---|---|
| Dónde vive | en macOS 26, ya instalado | 1,5 GB que descarga el usuario a su carpeta |
| Cómo se llama | Swift (`actor` + secuencias asíncronas) | crate de Rust que compila whisper.cpp |
| Coste de arranque | el modelo del idioma, una vez, por macOS | compilar C++ en cada `cargo build` |
| macOS < 26 | no existe | funciona |

## Medición

Hecha antes de comprometer nada (bitácora, fase 3a), con los dos audios sintéticos del kit —las
frases de la maqueta dichas por la voz del sistema— y el reloj dentro del programa:

| Pasada | Audio | Primer parcial | Total | Velocidad |
|---|---|---|---|---|
| es-ES, modelo recién instalado | 5,88 s | 247 ms | 370 ms | ×15,9 |
| en-US, modelo recién instalado | 5,39 s | 87 ms | 227 ms | ×23,7 |
| **es-ES, modelo ya instalado** | 5,88 s | **51 ms** | **84 ms** | **×69,9** |
| es-ES, a través del puente desde Rust | 5,88 s | — | 243 ms | ×24,2 |

Un turno de seis segundos cuesta entre 84 y 243 ms de transcripción. El presupuesto del sprint son
**cuatro segundos**. No hubo empate que deshacer, y por eso **whisper-rs no llegó a medirse**: la
comparación se detiene cuando una opción cabe treinta veces dentro del presupuesto y la otra pide
1,5 GB de descarga para entrar en la carrera.

## Decisión

### 1 · SpeechAnalyzer a través de un puente de Swift compilado en el build

`nativo/Transcriptor.swift` se compila desde `build.rs` como librería estática y se enlaza dentro
del binario. Se comprobó que funciona **con solo las Command Line Tools**, sin Xcode completo.

El puente **transcribe turnos completos, no un flujo continuo**. El fin de turno ya lo decide
aritmética que se puede leer y probar (`voz::turno`); dejárselo al motor sería devolverle a una
caja cerrada la decisión más delicada del sprint, y encima la más barata de hacer bien.

### 2 · Los dos fallos del build se tratan distinto, y esa asimetría es el gate

- **`swiftc` no está** (un Mac sin herramientas de desarrollo): se avisa y se sigue. La app
  compila, arranca y funciona; pierde la transcripción y **lo dice** en la pantalla de Idioma.
- **`swiftc` está y falla**: se rompe la compilación. Un binario verde sin transcripción y sin
  nadie enterado es el único desenlace inaceptable.

### 3 · El modelo lo descarga macOS, a petición del usuario, y se declara

`AssetInventory.assetInstallationRequest` + `downloadAndInstall` es **la única puerta a la red de
un módulo protegido de esta app**. No sale audio ni texto: entra el modelo. Por eso:

- se llama **solo** desde el botón de la pantalla de Idioma, nunca por iniciativa de la app;
- las dos líneas llevan `verify-ephemeral:allow` citando este ADR, porque el barrido las prohíbe
  por defecto — se añadieron a la lista de API vetada **en el mismo cambio** que las usó;
- la pantalla de Idioma lo escribe en español llano, no en una nota al pie.

### 4 · Dos hechos de macOS que la pantalla enseña porque no puede esconderlos

- **Los modelos se reparten por reserva.** `AssetInventory.status` decía «soportado, sin instalar»
  de un español que `installedLocales` listaba como instalado: el modelo estaba en el Mac, pero no
  *para nosotros*. Hasta que la app reserva el idioma, el activo no le cuenta. Se descubrió
  midiendo, no leyendo.
- **El techo es de cinco idiomas por app** (`maximumReservedLocales`). No es una decisión nuestra
  y la pantalla lo dice: una lista sin techo dejaría que el sexto fallara sin explicación.

### 5 · El respaldo se declara y NO se implementa en este sprint

macOS < 26 se queda sin transcripción y lo dice con esa frase exacta. `whisper-rs` sigue siendo el
camino escrito para ese caso; ponerlo hoy sería compilar whisper.cpp en cada build y descargar
1,5 GB para un caso que este usuario no tiene. **Deuda declarada**, no olvido.

## Consecuencias

- La app **exige macOS 26** (`minimumSystemVersion`). Decía «26 para transcribir y 14.2 para lo
  demás», y dejó de ser verdad en el sprint 002: el puente enlaza `FoundationModels` —que solo existe
  en macOS 26— de forma fuerte, y en un Mac con 14 o 15 dyld aborta al arrancar. Lo vigila
  `tests/unit/version-minima.test.ts` (auditoría del S2, M1). Bajar el mínimo exige enlazarlo débil
  (`-weak_framework`) y guardas `@available`: queda para H2.
- El puente añade `swiftc` a las herramientas del build. La CI de macOS lo tiene; si un día no lo
  tuviera, la compilación se rompería a propósito.
- Una parte del código de esta app está en Swift. Vive entera en un archivo, bajo la misma regla
  del efímero, y `pnpm verify:ephemeral` lo barre con una lista de API prohibida traducida a
  Swift — que **también se añadió en este sprint**, porque antes el barrido leía el archivo y no
  veía nada.

## Lo que este ADR NO decide

- Qué idioma escucha cada pista. Era fijo (es-ES / en-US); desde la auditoría del sprint 002 (A4)
  se elige en la pantalla de Idioma y las dos nacen en es-ES.
- El diccionario técnico, que llegó en el sprint 002 (fase 1). Varios idiomas en una misma pista
  siguen pendientes: el ADR 009 midió que el motor no sostiene la mezcla y la pantalla lo dice.
