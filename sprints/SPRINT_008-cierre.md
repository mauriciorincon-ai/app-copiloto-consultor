---
sprint: 008 · app: copiloto-consultor · hito: H-1 · opened: 2026-10-12 · closed: 2026-10-12 · branch: sprint-008/listo-para-el-gate · pr: #24
---

# Sprint 008 · Listo para el gate

| Campo                  | Valor                                                                                                                                                                      |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Objetivo               | Cerrar el H-1 salvo la sentada del usuario: Windows completo, deuda medios/bajos de S5–S7 a cero, documentos del hito al día y gate del H-1 en la guía                     |
| Resultado              | Construido entero; auditoría pagada en todas sus severidades. Pendiente del hito: solo el gate humano (Mac y PC)                                                           |
| Estado                 | Listo para mergear                                                                                                                                                         |
| Tiempo estimado / real | ~21 h (presupuesto «hasta terminar», decidido por el usuario) / 3,4 h activas (≈ 4 h de reloj, 09:20 → 13:20)                                                              |
| Diferencia             | ≈ −17 h · causa: cada uno de los 68 pendientes contado como horas, la mayoría de una línea con su test · medida: estimar el lote por tamaño (5 · 15 · 45 min), no por fila |

Horas (`scripts/horas-sprint.mjs`):

```
| Commits 15 | Sesiones 1 | Horas activas 3,4 h | 2026-10-12 09:47 → 2026-10-12 12:39 (2,9 h) |
| F1 0,8 h | F2 1,3 h | F3 1,2 h |
```

- **Fases.** F1 Windows completo (Opus 5.5) · 6,5 h / 0,8 h, 4 commits: reunión por sesión de captura, radar con catálogo propio, Windows Hello, borrado programado, acople, sugerencia por API (ADR 023) · F2 deuda a cero (Sonnet 5.5) · 6,5 h / 1,3 h, 3 commits: 83 filas de S5–S7 en 0 · F3 documentos y gate (Sonnet 5.5) · 8 h / 1,2 h, 8 commits: manual EN, design-system 1.18.0, auditoría de la constitución, BLUEPRINT, brochure, guía v9.
- **Entregables.** `docs/MANUAL-EN.md` · `docs/BROCHURE.html` (427 KB, 20 funcionalidades, ES/EN) · `docs/brochure-export.json` · `docs/BLUEPRINT.html` · `design-system.md` 1.18.0 · `decisions/023` + enmiendas de ADR 005, 016 (4) y 022 (2) · `docs/GUIA-DE-PRUEBA.html` v9 · `sprints/SPRINT_008-constitucion.md` · `sprints/SPRINT_008-auditoria.md`.
- **No completado.** Nada del plan. Fuera por diseño: el gate humano (lo corre el usuario), `/design-sync`, copiar el export en CV Viva y publicar la ficha (otros repos).
- **Desviaciones.** MDM de Windows por `IsDeviceRegisteredWithManagement` · el acople no guarda el HWND en `Destino` · `windows-future` como dependencia directa · radar de Windows sin Inspera, Hubstaff ni DeskTime (sin fuente) · «Conoce» solo dentro de la app, en Honestidad · brochure 427 KB frente a ≈ 150 KB estimados · textos sin maqueta («maquetado, no visto», al gate): `presencial-nuevo.ts`, `conoce.ts`, los de Windows y las filas 21–26 de la tabla U · la mirada del brochure en la fila 25 de la tabla U (~3 min).
- **Decisiones.** Reunión en Windows = cliente del catálogo con sesión de captura activa (ADR 005 enmienda 1) · sugerencia en Windows por API (ADR 023) · «En el roadmap» sustituye al «H2» · tope de 48 disparadores por tarea (ADR 016 enmienda 4).
- **Deuda.** Ninguna: S5, S6 y S7 sin filas en «deuda» (lo exige `lote-del-hito-a-cero`); los 17 hallazgos del S8 pagados.
- **Auditoría.** `sprints/SPRINT_008-auditoria.md`, auditor Sonnet 5.5 en solo lectura, 4 casillas: 0 críticos · 2 altos (tope de 48 disparadores; nombre real de la tarea en la guía) · 5 medios · 10 bajos, todos `pagado · PR #24` (`ba58ca7`, y el commit de este cierre para la cifra de pruebas del export y el cierre real). Casilla 8: ninguna protección del sistema tocada en el sprint; la matriz de 13 filas viaja al gate.
- **Demos en rojo.** F1: 7 · F2: 26 · F3: manual 2, guía 3, brochure 3, auditoría 7. Todas nombraron lo esperado y volvieron a verde; dos mutaciones no se pusieron rojas y se cambiaron por una que sí.
- **Pruebas.** 1806 en la corrida 38285677757 sobre `ba58ca7`, cinco checks en success: Vitest 629 de 631 · cargo test macOS 748 (librería 699, contra el Mac 30, puerta 14, ghost 5) · Playwright 429 (3 saltadas a propósito) · Windows: librería 672 y 4 de integración.
- **Binario.** macOS, `pnpm tauri build --bundles app --no-sign`: `.app` 13,88 MiB · ejecutable 12,91 MiB · `ghost` 0,45 MiB · brochure dentro, idéntico byte a byte. No se ejecutó la app (nada tocó TCC, Llavero ni launchd).
- **Aprendizajes.** Un estado de pago se escribe con el número del PR, no con el hash de una rama que el squash borra · una demo en rojo cuya mutación no se pone roja delata un cinturón redundante: se cambia la mutación, no el gate · lo que la app escribe en Windows (nombre de la tarea, credencial) se ata a la guía con un test.
- **Recomendación.** Siguiente: el gate humano del H-1, Mac y PC, con la matriz de 13 filas. Después, H-3 o H-2 según el uso real.
- **Esfuerzo-beneficio.** Alto: un sprint deja el hito a falta solo de la sentada del usuario.
- **Modelos y relevos.** F1 Opus 5.5 · F2 Sonnet 5.5 · F3 Sonnet 5.5 · auditor, traductores del manual y redactor del BLUEPRINT: subagentes Sonnet 5.5 · tras la compactación final la sesión corre en Opus 5.5 (fila «cerrar sprint») · el borrador lo escribió un subagente Haiku 5.5 (alias `haiku` = `claude-haiku-5-5`, lo dijo él mismo) y bastó: tres retoques del revisor. Relevos: ninguno.

**Para mergear:** squash y borrar la rama; después `/cierre-sprint copiloto-consultor`.
