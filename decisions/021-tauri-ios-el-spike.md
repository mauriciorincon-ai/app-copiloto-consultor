# ADR 021 — Tauri iOS, el spike que decide si el iPhone entra (D7')

- **Fecha:** 2026-10-05
- **Sprint:** 005 «En la mesa» (fase 0)
- **Estado:** **no corrido: decisión del usuario 2026-10-05.** En el G-Plan, la pregunta 1 («¿instalamos Xcode completo
  para el spike de iOS?») quedó en su default, **(b) no por ahora**. La fase 3 del sprint no corre y D7' sigue en el
  roadmap con esta razón. Este ADR deja escrito qué habría que medir, para que el spike arranque sin replantearlo
  cuando lo autorices.

## Contexto

La VISION v1.5.0 redefine D7' como **«tu iPhone en modo presencial»**: la misma app en el celular, escuchando la sala por
su micrófono y mostrándote la ficha. Nace del modo presencial (ADR 020), y entra o no según un spike de compilación para
iOS. `[roadmap]`

La precondición es **Xcode completo** (unos 12 GB, gratis, con la licencia de Apple). El ADR 001 lo evitó a propósito:
el puente Swift se compila desde `build.rs` con las Command Line Tools, sin proyecto `.xcodeproj` (ADR 001 §Decisión).
Para iOS no hay atajo: el SDK de iOS y el simulador solo vienen con Xcode. Instalarlo toca el Mac del usuario (licencia,
«Xcode quiere instalar componentes», quizá la contraseña de administrador), así que es una fila de la regla 22, con su
matriz y su «sí».

## Decisión

**No se corre en este sprint.** Sin Xcode no hay nada que compilar para iOS, y el modo presencial del Mac no depende del
spike.

## Lo que el spike medirá cuando se autorice

En una rama de spike, sin código de producto:

1. **Regla 22 primero:** la matriz de Xcode (qué · para qué · qué aviso vas a ver · cómo se deshace: borrar Xcode y
   `xcode-select -r`) y tu «sí».
2. `pnpm tauri ios init`: ¿genera el proyecto con la versión de Tauri del repo (2.12)?
3. **¿Compila el núcleo de Rust para `aarch64-apple-ios-sim`?** Lo que puede romper: los crates que hoy dan por hecho
   macOS (Core Audio con el _process tap_ del sistema, ScreenCaptureKit, Accessibility, launchd). En iOS no existen, y el
   modo presencial no los necesita. El spike dirá cuánto hay que poner detrás de `#[cfg(target_os = "macos")]`.
4. **¿Compila `Transcriptor.swift` contra el SDK de iOS 26?** SpeechAnalyzer existe en iOS 26. El puente se compila hoy
   desde `build.rs` con `swiftc` para macOS: el spike dirá si basta con otro `-target` o hace falta el proyecto de Xcode.
5. **¿Arranca la interfaz en el simulador?** Y qué pesa el binario.
6. **Recomendación**, con esas cifras: entra al S6 como trabajo paralelo acotado · entra a un S7 · se queda en el
   roadmap, con la razón escrita.

## Riesgos conocidos de antemano

- El puente Swift compilado en `build.rs` para macOS puede ser el primer muro. Es un resultado válido para este ADR,
  no un fracaso del spike.
- Lo que en el Mac protege la ventana al compartir pantalla no existe igual en iOS. En presencial no hace falta (ADR 020
  §9 explica por qué en el Mac sí se queda).
- Firmar para un iPhone real exige Apple Developer (US$99/año), diferido a G-Release. El simulador no lo necesita.

## Consecuencias

- D7' sigue `[roadmap]` con esta razón: «spike no corrido: decisión del usuario 2026-10-05».
- El summary del S5 lo declara, y la fase 3 aparece como «no corrida», no como corte silencioso.
- El modo presencial del Mac (ADR 020) se construye sin suponer nada de iOS.
