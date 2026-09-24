# ADR 001 — Plataforma nativa y bridge de Swift

- **Fecha:** 2026-09-20
- **Sprint:** 001 «La banda y la ficha»
- **Estado:** aceptada

## Contexto

Angel Ghost necesita cuatro cosas que el webview no puede dar: una ventana excluida de la
captura de pantalla, audio del **sistema** (no solo del micrófono), transcripción en el
dispositivo y la capacidad de redimensionar la ventana de otra app. Todas viven en frameworks de
Apple —ScreenCaptureKit, Core Audio, Speech, Accessibility— y ninguna está expuesta como crate
de Rust con la cobertura que el sprint necesita.

La pregunta abierta al planear era si eso obligaba a instalar **Xcode completo** (~10 GB) o si
bastaban las Command Line Tools que la máquina ya tiene.

## Decisión

**Un helper en Swift, compilado desde `build.rs` con las Command Line Tools**, expuesto a Rust
por una interfaz C mínima. Nada de Xcode completo, nada de proyecto `.xcodeproj`.

Se verificó antes de decidir, no después: `swiftc` con CLT compila contra `ScreenCaptureKit`,
`AVFoundation`, `CoreAudio` y `Speech`, y `SpeechAnalyzer` está presente en el SDK (macOS 27 SDK,
la máquina corre 26.6.2). La prueba es reproducible: un archivo Swift que importa los cuatro
frameworks y compila.

La superficie del bridge se mantiene **estrecha a propósito**: tipos primitivos y búferes, sin
objetos que cruzar. Cuanto menos cruce la frontera, menos sitios donde el efímero pueda
escaparse.

## Alternativas consideradas

- **Solo crates de Rust** (`cpal`, `screencapturekit-rs`): `cpal` sirve para el micrófono, pero
  el audio del sistema por process taps y `SpeechAnalyzer` no tienen equivalente estable. Se usa
  `cpal` donde alcanza y el bridge donde no.
- **Proceso auxiliar en Swift** en vez de biblioteca enlazada: aislaría fallos, pero mover audio
  entre procesos añade una copia y un canal más — un sitio más donde algo podría persistir.
  Rechazado por la misma razón que el bridge es estrecho.
- **Xcode completo:** innecesario, y una dependencia de 10 GB para el primer `git clone` de
  cualquiera que herede el repo.

## Consecuencias

- El build exige `swiftc` en el PATH. La CI de `build-escritorio` corre en `macos-latest`, que lo
  trae; el job falla con un mensaje claro si algún día no está.
- El bridge es **específico de macOS**. Windows (D3) es horizonte H2 y exigirá su propio ADR: la
  decisión de hoy no lo hipoteca porque la frontera es una interfaz C, no un diseño.
- `capture/` y `stt/` siguen siendo módulos protegidos: el código Swift también obedece la regla
  del efímero, y el barrido de `verify:ephemeral` se extiende a los `.swift` del bridge cuando
  existan.
