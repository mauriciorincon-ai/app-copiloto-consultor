---
sprint: 005
app: copiloto-consultor
status: closed
opened: 2026-10-05
closed: 2026-10-11
branch: sprint-005/en-la-mesa
pr: https://github.com/mauriciorincon-ai/app-copiloto-consultor/pull/15
cierre: "mínimo (directiva del 2026-10-06): fases 1 y 2 · H2, sprint 2 de 3 · no cierra ciclo"
---
# Sprint 005 Summary — Angel Ghost «En la mesa»

**Outcome: parcial, como lo redujo la directiva.** El modo presencial (C19) está construido, medido por el kit v4 y en la interfaz, pero no ha corrido en vivo con un micrófono real (corte declarado).

## Qué se construyó
- **Fase 0**, corrida y aprobada el 2026-10-05, antes de la directiva (se declara, no se deshace): constitución regenerada con el kit v1.40.0, delta del kit v1.37→v1.40 con sus rojos, ADR 020 y 021, enmiendas 002 y 009, maqueta presencial con su mirada de DECISIÓN.
- **Fase 1 · el núcleo:** `Modo::Presencial` abre solo el micrófono como `Pista::Sala`, sin dueño (`Quien::SinAtribuir`). La pantalla no se abre (por tipo), no hay acople, propuestas ni tus turnos, y la NDA se comprueba en Rust. La tolerancia a tu voz es código (`disparo/tolerancia.rs`), sin biometría (gate `cero-huellas-de-voz`). Kit v4 con dos voces sintéticas: niveles A (la regla) y B (los turnos) en la CI.
- **Fase 2 · la regla y la interfaz:** la regla que eligió el kit (15 s tras una ficha, solo pregunta o cifra, sin ficha por silencio, tope de turno de 15 s) deja 5 fichas falsas en vez de 16 y 5 pertinentes, tantas como sin regla. La voz al oído espera a que la sala calle (4 s). Sesión, banda, Honestidad y el rail conocen la sala; el manual explica el Mac al lado del PC con Windows.

## Qué no se construyó
- La maniobra §10 (directiva); la fase 3, el spike de iOS (sin Xcode, ADR 021 «no corrido»); la fase 4, que sustituye el cierre mínimo (sin guía, `design-sync/`, brochure, BLUEPRINT ni `/release-check`).
- **La corrida en vivo del modo presencial** (regla 15, tercer filo): el usuario no pudo hacerla el 2026-10-10. El modo no ha corrido con micrófono ni reconocimiento de voz reales, y el nivel C del kit v4 no ha medido nunca: la CI no tiene modelos de voz. Va al gate del MVP del H2.

## Desviaciones frente al plan
1. La banda sigue protegida y con su relleno en presencial (ADR 020 §9), y el radar de procesos sigue.
2. La regla entró con los valores por defecto del STOP: la respuesta llegó como «<respuesta a Q2>», sin rellenar. Cambiarla es una línea de `data/presencial/reglas.json`.
3. El idioma de la sala se elige en Sesión, no en Idioma (no cabe sin quitar una franja aprobada). Va al gate del MVP, con la segunda vuelta de la maqueta presencial, «maquetada, no vista».
4. La espera de 4 s de la voz es decisión de producto; el kit no la mide.

## Auditoría y deuda
Una pasada, un auditor independiente, solo lectura: `sprints/SPRINT_005-auditoria.md`, con 0 críticos, 1 alto, 8 medios y 14 bajos. **A1 pagado:** «Terminar sesión» no apagaba la voz que encendió la sala; ahora la apaga, con test y su rojo. **Deuda del S6**, sin pagar por la directiva (el detalle y el ajuste de cada uno, en el artefacto):
- **Medios:** M1 la banda dice «la sala» sin micrófono abierto `src/componentes/Banda.tsx:443` · M2 chip «sin acople» en presencial `src/componentes/Banda.tsx:450` · M3 Notas no conoce la sala `src/pantallas/Notas.tsx:488` · M4 el radar coral empieza una reunión desde presencial `src/pantallas/Sesion.tsx:152` · M5 idiomas de la sala cableados `src/pantallas/Sesion.tsx:444` · M6 ⌃⌥V rebusca sin tolerancia `src-tauri/src/lib.rs:2301` · M7 auriculares con micrófono como entrada (se confirma en vivo) `src-tauri/src/capture/nativo.rs:321` · M8 cifras del kit mal contadas en el manual `docs/MANUAL-DE-USO.md:170`.
- **Bajos:** B1 `src-tauri/src/lib.rs:2724` · B2 `src/pantallas/Idioma.tsx:149` · B3 `decisions/009-un-idioma-por-pista.md:107` · B4 `src-tauri/src/lib.rs:2657` · B5 `src-tauri/src/lib.rs:773` · B6 `tests/unit/pista-por-quien.test.ts:28` · B7 `src-tauri/src/disparo/mod.rs:94` · B8 `.claude/settings.json:12` · B9 `src-tauri/src/contrato.rs:353` · B10 `README.md:28` · B11 `docs/MANUAL-DE-USO.md:156` · B12 `docs/MANUAL-DE-USO.md:165` · B13 `src-tauri/src/lib.rs:718` · B14 `src-tauri/src/lib.rs:2247`.

## Commits y CI
Primer commit de la rama: `434e9c7` (2026-10-05 20:15, el delta del kit). Último: el de este cierre, sobre `0cc590f`. La CI de `0cc590f` dio `quality`, `e2e` y `build-escritorio` en `success`; la del cierre se lee con `gh pr checks 15` antes de marcar el PR listo.

## Para mergear
Marca el PR listo, mergea con **squash** y borra la rama; después corre `/cierre-sprint copiloto-consultor`. Lo hace el usuario.
