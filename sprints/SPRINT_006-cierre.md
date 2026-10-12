---
sprint: 006 · app: copiloto-consultor · hito: H-2 (Windows, 1 de 3)
opened: 2026-10-11 · closed: 2026-10-11 · branch: sprint-006/windows-arranca · pr: #20
---

# Sprint 006 · Windows arranca · cierre ejecutivo

| Campo                          | Valor                                                                                                                                                                                                                                                                                         |
| ------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Objetivo y resultado           | Angel Ghost compila y pasa sus pruebas en Windows, sale un instalador desde la CI y Honestidad dice qué hace allí. Construido y verde; las pruebas de PC van al gate del hito.                                                                                                                |
| Tiempo                         | estimado ~9 h · real 3,5 h activas (3,0 h de span)                                                                                                                                                                                                                                            |
| Fases                          | F1 Sonnet 5.5 · medium · 0,5 h · F2 Opus 5.5 · high · 1,3 h · F3 Haiku 5.5 · medium · 1,8 h                                                                                                                                                                                                   |
| Entregables                    | ADR 022 · micrófono WASAPI · whisper con calibración · descarga con huella · Administrador de credenciales · privacidad del micrófono · «Hoy en Windows» · manual · guía v7 · job `build-windows` · `.msi` 8,35 MiB, `.exe` 5,89 MiB, sin firmar                                              |
| No completado                  | radar de procesos · exportar notas y abrir la bandeja con Windows Hello · audio del sistema, pantalla, acople, ensayo con voz, sugerencia, atajos, puerta (S7 o no por ahora)                                                                                                                 |
| Desviaciones                   | radar a S7 por coherencia con su pantalla (no por horas) · manual solo en español (EN de Windows en `src/i18n/windows.ts`; el manual EN va al gate) · notas cifradas; exportar espera Windows Hello · pruebas de PC (regla 24, 4 filas) al gate del hito por decisión del usuario, no ejecutadas aquí                                                                                           |
| Causas                         | F2: tres vueltas rojas de CI (nombre solo-Unix, CRLF del checkout, texto de desbloqueo). F3: texto de macOS en pantallas de Windows y un gate que pasaba con la mutación; los cazó la auditoría. Última vuelta de CI: un tipo privado usado desde otro módulo (E0446), corregido en `1cbc304` |
| Decisiones                     | diccionario de Windows aparte y cortes declarados (usuario) · puerta de red única en Rust (ADR 022 §4)                                                                                                                                                                                        |
| Deuda                          | `sprints/SPRINT_006-auditoria.md`: 12 medios y 12 bajos con `archivo:línea`, al lote del hito · M11 declarada                                                                                                                                                                                 |
| Aprendizajes                   | un gate de «no aparece X» necesita su control en la otra plataforma; una guarda se prueba retornando · un modelo que no puede resolver para tras dos intentos y pide relevo; nunca reintenta en bucle                                                                                         |
| Siguiente                      | S7: audio del sistema por loopback, Windows Hello, atajos, pantalla; antes, la corrida en vivo de este sprint                                                                                                                                                                                 |
| CI del último commit de código | `1cbc304`: quality, e2e, build-escritorio, build-windows y un-pr-vivo en `success` ([run 38190556008](https://github.com/mauriciorincon-ai/app-copiloto-consultor/actions/runs/38190556008)); artefacto `angel-ghost-windows` 14,0 MiB comprimido                                             |

**Auditoría** (una pasada, Sonnet 5.5 high, detalle en `sprints/SPRINT_006-auditoria.md`): alcance → 5 altos pagados (A1–A5); calidad y seguridad → altos pagados, M2 y M11 al lote; frases caducadas → A1, A4 y A5 pagadas.

**Modelo y desvío**: Haiku falló dos veces en la misma clase (texto por plataforma; gate que no demuestra). La regla pedía parar y pedir relevo a Sonnet 5.5, y **no se hizo**: siguió en vez de cederle la tarea. Es el fallo que la orden de relevo sin bucles corrige; detalle en `sprints/SPRINT_006-informe-para-planeadora.md`.

**Horas** (`node scripts/horas-sprint.mjs`):

| Commits | Sesiones | Horas activas | Span (primer → último commit)               |
| ------: | -------: | ------------: | ------------------------------------------- |
|      11 |        1 |         3,5 h | 2026-10-11 16:46 → 2026-10-11 19:47 (3,0 h) |

| Fase | Commits | Horas activas |
| ---- | ------: | ------------: |
| F1   |       1 |         0,5 h |
| F2   |       5 |         1,3 h |
| F3   |       5 |         1,8 h |

**Para mergear**: squash y borrar la rama; después `/cierre-sprint copiloto-consultor`. Las pruebas de PC van al gate del hito.
