---
sprint: 009 · app: copiloto-consultor · hito: H-1 · opened: 2026-10-17 · closed: 2026-10-17 · branch: sprint-009/la-surface · pr: #25
---

# Sprint 009 · La Surface

| Campo                               | Valor                                                                                                                                                                                                      |
| ----------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Objetivo                            | Angel Ghost nativo en Windows ARM64 (la Surface del usuario): build, tests e instaladores; whisper medido allí; modelo por defecto por la regla de 4 s; job de CI ARM                                      |
| Resultado                           | Construido y verificado en la Surface: `cargo test` sin hardware en verde, el instalador NSIS ARM64 instalado y ejecutado en vivo, y CI con seis checks en `success` sobre `4832612` (corrida 39057154839) |
| Estado                              | Listo para mergear · PR #25 (borrador a normal al cerrar)                                                                                                                                                  |
| Tiempo estimado / real / diferencia | 4 h / 2,0 h (piso de la heurística) / −2 h · la heurística no ve la lectura ni las esperas; la CI tardó 25 min en `build-windows`                                                                          |
| Horas por fase (planeadas / reales) | F1 1,5 / 0,5 · F2 1,5 / 1,1 · F3 1 / 0,5                                                                                                                                                                   |

Horas (`scripts/horas-sprint.mjs` sobre `origin/main..4832612`; el commit de este cierre no entra):

```
| Commits 12 | Sesiones 1 | Horas activas 2,0 h | 2026-10-17 11:40 → 2026-10-17 13:13 (1,5 h) |
| F1 0,5 h | F2 1,1 h | F3 0,5 h |
```

- **Fases.** F1 Entorno y build nativo (Opus 5.5) · F2 CI ARM y cifra real (Sonnet 5.5) · F3 Evidencia, auditoría y cierre (Sonnet 5.5): ronda de correcciones de la auditoría (cuatro commits `[F3]`).
- **Entregables.** `scripts/entorno-windows.ps1` (solo lee; suma `clang-cl` y `ninja`) · `src-tauri/.cargo/config.toml` y `cmake/windows-arm64-clang-cl.cmake` (whisper.cpp con clang-cl y Ninja; `GGML_NATIVE=OFF` en todos los targets) · `scripts/arquitectura-pe.mjs` (gate PE, `0xAA64`) · job `build-windows-arm` en la ruleset · `el_mayor_que_cabe` en `stt/whisper.rs` · ADR 022 enmienda 3 · README, manual ES/EN, guía (bloque PC), BLUEPRINT y brochure · `sprints/SPRINT_009-auditoria.md`.
- **Medidas en la Surface** (Snapdragon X Plus, dos pistas, mediana del turno de 10 s): `large-v3-turbo` 9144 ms (rango 8523 a 18428) · `small` 1632 ms (rango 1590 a 1641). El mayor modelo que cabe en 4 s es `small`: ese es el defecto en ARM64 (0,5 GB). x64 y macOS no cambian.
- **Instalado en vivo.** El `.exe` NSIS ARM64, instalado y ejecutado en la Surface (fila 9, con «sí»). El `.msi` ARM64 se construyó y no se instaló en vivo: va al gate del hito.
- **Pruebas.** En la Surface: `AG_SIN_HARDWARE=1 cargo test --locked -j 4`, 677 de 677 en la librería (9 ignoradas), con `GGML_NATIVE:BOOL=OFF` y `GGML_CPU_ARM_ARCH` en la caché de CMake. Vitest 654 de 654. CI sobre `4832612`: `quality`, `e2e`, `build-escritorio`, `build-windows` (caché nueva, compilación limpia), `build-windows-arm` y `un-pr-vivo` en `success`.
- **Desviaciones.** GGML portable también en x64 (no estaba en el plan; lo encontró la CI) · `zip` 9 y `rustls` 0.23.46 por RustSec (no estaba en el plan) · `-j 4` por LNK1102 · el orden de las instalaciones y un reinicio de VS Code tras la fila C1 (PATH) · M3: «Instalar» en ARM64 no corrido por el botón, los modelos llegaron por script (lo lleva la guía, w4) · M7: Honestidad enseña la calibración de 10 s por modelo, no por pista (las cifras por pista están en el ADR) · `scripts/demo-rojo.sh` necesita `python3`, ausente en la Surface: demos en rojo a mano.
- **Causas.** Con `GGML_NATIVE` por defecto, ggml mide el SIMD de la máquina que compila; la caché lo llevó a otro runner y dio `STATUS_ILLEGAL_INSTRUCTION` en x64. `build-escritorio` estaba rojo por avisos de RustSec sobre `rustls` 0.23.45.
- **Decisiones.** El defecto en ARM64 es el mayor modelo que cabe en 4 s medido (hoy `small`) · binarios portables con `GGML_NATIVE=OFF` · el borrador del cierre con Haiku 5.5, por decisión del usuario.
- **Deuda generada o aplazada.** M5 abierto: `CLAUDE.md:15`, `.claude/commands/release-check.md:7,31` y `docs/REGLAS-ANEXO.md:645` no nombran `build-windows-arm`; lo decide el usuario. Las ramas de respaldo del job ARM (`.github/workflows/ci.yml`) nunca se ejecutaron: la CI tuvo siempre las piezas.
- **Aprendizajes.** Una caché de compilación lleva la configuración en su clave. Un camino de respaldo que nunca corre se declara como tal, no se da por probado.
- **Recomendación.** Gate humano del hito H-1 en Mac y PC, con la matriz de 13 filas. En la Surface quedan el `.msi` ARM64 en vivo y «Instalar» por el botón. Decidir M5.
- **Esfuerzo-beneficio.** Alto: la app corre nativa en la Surface y el modelo por defecto sale de la medición de ese equipo.
- **Auditoría.** `sprints/SPRINT_009-auditoria.md`, Sonnet 5.5 en solo lectura, cuatro casillas. Alcance con desviación (M3, M7 y `.msi` en vivo). Calidad: A1 e2e, A2 caché x64 y A3 textos «1,6 GB», todos `pagado` (`345960e` y `1749c9c`). Seguridad sin hallazgos. Frases caducadas: M4 y M6 pagados (`4832612`). 16 hallazgos: 3 altos pagados; M5 abierto y M7 descartado; los demás medios y bajos pagados.
- **Modelos y relevos.** F1 Opus 5.5 · F2 Sonnet 5.5 · F3 Sonnet 5.5, con la ronda de correcciones en Rust (la tabla del núcleo asigna Rust a Opus high: desvío declarado, sin relevo, compiló y pasó la suite sin segundo intento) · auditor Sonnet 5.5 · cierre Haiku 5.5 (`/model claude-haiku-5-5`). Relevos: ninguno.

**Para mergear:** squash y borrar la rama; después `/cierre-sprint copiloto-consultor`.
