---
sprint: 011 · app: copiloto-consultor · hito: H-1 · opened: 2026-10-18 · closed: 2026-10-18 · branch: sprint-011/la-banda-a-tu-altura · pr: abierto con este cierre
---

# Sprint 011 · La banda a tu altura

| Campo                               | Valor                                                                                                                                                                    |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Objetivo                            | En Windows, la banda arriba se baja a tu altura para ver las pestañas del navegador; se cierra con su × sin cerrar la app; la reunión acoplada baja hasta debajo de ella |
| Resultado                           | Hecho en código. Auditoría de una pasada: 0 críticos, 1 alto, 4 medios y 7 bajos, todos pagados (082c944)                                                                |
| Estado                              | PR abierto con este cierre. Pendiente el STOP en vivo en la Surface (matriz del PR)                                                                                      |
| Tiempo estimado / real / diferencia | 4 h / 2,0 h / −2,0 h (heurística de horas-sprint: no ve la lectura, las esperas de cargo ni el e2e)                                                                      |
| Horas por fase (planeadas / reales) | F1 2 / 0,5 · F2 1,5 / 0,8 · F3 0,5 / 0,7 (incluye los pagos de la auditoría)                                                                                             |

Horas (`scripts/horas-sprint.mjs` sobre `origin/main..49e04b4`, sin el commit de este cierre):

```
| Commits 5 | Sesiones 1 | Horas activas 2,0 h | 2026-10-18 12:49 → 2026-10-18 14:21 (1,5 h) |
| F1 0,5 h | F2 0,8 h | F3 0,7 h |
```

- **Fases.** F1 geometría, comandos y contrato (Opus 5.5) · F2 asa, × y filas de Sesión, solo Windows (Sonnet 5.5) · F3 guía, manual, e2e, BLUEPRINT, auditoría y pagos (Sonnet 5.5) · cierre (Haiku 5.5).
- **Entregables.** `geometria::franja_desplazada` (`src-tauri/src/ventana/geometria.rs`) · comandos `desplazar_banda`, `asentar_desplazamiento`, `cerrar_la_banda`, `mostrar_la_banda` (`src-tauri/src/lib.rs`) · `desplazamientoDeLaBanda` en `src-tauri/src/prefs.rs` · asa de altura y × (`src/componentes/Banda.tsx`, `src/asa.ts`) · «Altura de la banda», «Bajar», «Subir», «Mostrar la banda» (`src/pantallas/Sesion.tsx`) · textos en `src/i18n/windows.ts` · guía con `ag-w11-` (w16 mejorada; w20 y w21 nuevas) · manuales ES y EN · `sprints/SPRINT_011-auditoria.md`.
- **Verificado.** vitest 755 pasan y 2 se saltan (79 archivos) · `cargo test --lib` con `AG_SIN_HARDWARE=1` y `-j 4`: 698 de 698 (9 ignoradas) · clippy `--all-targets -D warnings` limpio · e2e completo en Edge: 467 pasan y 3 se saltan · demos en rojo de cada gate nuevo, uno por uno, y restauradas · cero diff en `*.swift`, `apple.rs`, `ax.rs`, `atajos.rs`, `es.ts`, `en.ts`, la guía general y `docs/diseno/` · `verify:ephemeral` estático limpio · `cero-enlaces` limpio, también con la homepage.
- **Release-check.** Pasan 2 (capturas leídas: 8 de 8, sin desborde; la altura y los botones de Sesión saltan a otra línea al ser largos, anotado a la matriz), 5, 7 y 8 · 6 estático limpio y la corrida real va al STOP · 1 y 6 (corrida real) pendientes del STOP en vivo · 3 y 4 n/a (sin dependencias nuevas, sin permisos TCC) · 9 se lee al final con `gh pr checks` · 10 n/a: ninguna protección nueva (w20 y w21 lo dicen).
- **Desviaciones.** (a) Textos de Windows en `src/i18n/windows.ts`, no en maqueta ni en `es.ts`/`en.ts`. (b) Rust en Sonnet 5.5 en F2 (el aviso de `empezar`) y en el pago de la auditoría (A1, B1, B2); la tabla asigna Rust a Opus high: desvío declarado. (c) `w20` y `w21` quedan fuera de las 9 paradas del gate, para no tocar la guía general; el filtro «Lo que cambió» cuenta desde S10. (d) Tope de 60 s en `windows-sin-mac` (7,1 s medidos en la Surface con vitest en paralelo). (e) Ninguna maqueta tocada: el sprint no cambia el diseño aprobado.
- **Causas.** La diferencia por debajo del estimado se explica sobre todo porque la heurística no cuenta lectura ni esperas (cargo, e2e). La auditoría encontró un fallo de hilo (A1) que ningún test había visto: en Windows, «Iniciar sesión» tras el × creaba la ventana desde un comando síncrono.
- **Decisiones.** El × con una sesión en marcha la termina, como decidió el G-Plan (1a). «Terminar sesión» NO devuelve la ventana acoplada: así estaba desde S4/S8; la guía, el manual y la enmienda 3 del ADR 004 ahora lo dicen. No se cambia: es decisión de producto.
- **Deuda generada o aplazada (al lote del hito H-1).** Maquetado, no visto: asa, × y filas de Sesión en la Surface, y los textos nuevos de `windows.ts` en un PC real · los dos arreglos de S10 pendientes de ver en vivo (partir la línea de Honestidad y «Measuring…») · decisión de producto: que «Terminar sesión» devuelva la reunión. Ningún medio ni bajo del auditor queda abierto.
- **Aprendizajes.** Mover código a una función nueva rompe las pruebas de fuente que buscan en el cuerpo de la vieja: se vio por el rojo, no por lectura, y cada prueba se reapuntó y se volvió a ver en rojo. En Windows, un comando síncrono no crea ventanas, y ningún test de la CI lo detectaba: lo cubre ahora un gate de fuente.
- **Recomendación.** Gate humano del hito H-1 (unos 20 min) con la matriz del STOP; después, merge squash. Siguiente sprint: decidir si «Terminar sesión» devuelve la ventana acoplada.
- **Esfuerzo-beneficio.** Alto: la banda deja ver las pestañas de Chrome sin cortar nada, y se cierra sin matar la app; el coste fue 2 h activas.
- **Auditoría.** `sprints/SPRINT_011-auditoria.md`, Sonnet 5.5 en solo lectura, cuatro casillas. Alcance completo con las desviaciones ciertas. Calidad: A1 (alto), B1, B2, B3 y B4 pagados. Seguridad limpia. Frases caducadas: M3, M4, B6 y B7 pagados. El auditor no corrió cargo ni Playwright; el constructor sí, y están en «Verificado». 12 hallazgos, todos pagados en 082c944. Veredicto: listo para cierre.
- **Modelos y relevos.** F1 Opus 5.5 · F2 Sonnet 5.5 · F3 Sonnet 5.5 · auditor Sonnet 5.5 · cierre Haiku 5.5, que bastó para el cierre (Haiku no se usó en construcción simple, así que no hay dato de construcción). Desvío: Rust en Sonnet (ver Desviaciones). Relevos: ninguno.

**Para mergear:** squash y borrar la rama; después `/cierre-sprint copiloto-consultor`.
