---
sprint: 006 · app: copiloto-consultor · hito: H-2 (Windows, 1 de 3)
opened: 2026-10-11 · closed: 2026-10-11 · branch: sprint-006/windows-arranca · pr: #20
---

# Sprint 006 · Windows arranca · cierre ejecutivo

| Campo | Valor |
|---|---|
| Objetivo y resultado | Angel Ghost compila y pasa sus pruebas en Windows, sale un instalador desde la CI y Honestidad dice qué hace allí. Construido y verde; falta el STOP (PC) y el gate del hito. |
| Tiempo | estimado ~9 h · real 3,5 h activas (3,0 h de span) |
| Fases | F1 Sonnet 5.5 · medium · 0,5 h · F2 Opus 5.5 · high · 1,3 h · F3 Haiku 5.5 · medium · 1,8 h |
| Entregables | ADR 022 · micrófono WASAPI · whisper con calibración · descarga con huella · Administrador de credenciales · privacidad del micrófono · «Hoy en Windows» · manual · guía v7 · job `build-windows` · `.msi` 8,35 MiB, `.exe` 5,89 MiB, sin firmar |
| No completado | radar de procesos · exportar notas y abrir la bandeja con Windows Hello · audio del sistema, pantalla, acople, ensayo con voz, sugerencia, atajos, puerta (S7 o no por ahora) |
| Desviaciones | radar a S7 por coherencia con su pantalla (no por horas) · manual solo en español (EN de Windows en `src/i18n/windows.ts`; el manual EN va al gate) · notas cifradas; exportar espera Windows Hello |
| Causas | F2: tres vueltas rojas de CI (nombre solo-Unix, CRLF del checkout, texto de desbloqueo). F3: texto de macOS en pantallas de Windows y un gate que pasaba con la mutación; los cazó la auditoría |
| Decisiones | diccionario de Windows aparte y cortes declarados (usuario) · puerta de red única en Rust (ADR 022 §4) |
| Deuda | `sprints/SPRINT_006-auditoria.md`: 12 medios y 12 bajos con `archivo:línea`, al lote del hito · M11 declarada |
| Aprendizajes | un gate de «no aparece X» necesita su control en la otra plataforma; una guarda se prueba retornando |
| Siguiente | S7: audio del sistema por loopback, Windows Hello, atajos, pantalla; antes, la corrida en vivo de este sprint |
| CI del último commit | _pendiente de `d39aedd`_ |

**Auditoría** (una pasada, Sonnet 5.5 high, detalle en `sprints/SPRINT_006-auditoria.md`): alcance → 5 altos pagados (A1–A5); calidad y seguridad → altos pagados, M2 y M11 al lote; frases caducadas → A1, A4 y A6 pagadas.

**Modelo y desvío**: Haiku falló dos veces en la misma clase (texto por plataforma; gate que no demuestra). La regla pedía subir a Sonnet 5.5 y **no se subió**: queda dicho aquí.

**Horas** (`node scripts/horas-sprint.mjs`):

| Commits | Sesiones | Horas activas | Span (primer → último commit) |
|---:|---:|---:|---|
| 11 | 1 | 3,5 h | 2026-10-11 16:46 → 2026-10-11 19:47 (3,0 h) |

| Fase | Commits | Horas activas |
|---|---:|---:|
| F1 | 1 | 0,5 h |
| F2 | 5 | 1,3 h |
| F3 | 5 | 1,8 h |

**Para mergear**: squash y borrar la rama; después `/cierre-sprint copiloto-consultor`.
