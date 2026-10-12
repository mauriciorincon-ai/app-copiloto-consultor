# Auditoría de la constitución · cierre del H-1 (sprint 008)

Fecha 2026-10-12 · lo exige el cierre de hito (`metodo.md:191`: el `CLAUDE.md` se audita contra el código al cerrarlo).
Alcance: el núcleo (`CLAUDE.md`, kit v2.2.0), el § «Angel Ghost» de `docs/REGLAS-ANEXO.md` y los comandos de `.claude/commands/`.
Quién: un auditor independiente en solo lectura (subagente Sonnet 5.5) comprobó cada afirmación verificable contra el código, la CI,
la ruleset de `main` y la configuración; el constructor **reverificó con `grep`, `ls`, `wc` y `gh api` cada deriva antes de corregirla**.
Resultado: **13 derivas en el núcleo, 9 en el anexo y 4 en los comandos; las 26 corregidas o declaradas; ~90 afirmaciones sin deriva.**

## Derivas del núcleo (`CLAUDE.md`) — corregidas

| # | Decía | Era | Se corrigió |
|---|---|---|---|
| 1 | kill-switch `⌥⎋` (reglas 1e y «el corte») | en Windows es `Alt+Mayús+Q` (`atajos.rs:75`, `:93`) | las dos teclas, por sistema |
| 2 | cada módulo nativo tiene su `apple.rs` | solo `habla/`, `pantalla/` y `stt/`; el resto usa `ax.rs`, `nativo.rs` o el `mod.rs`; `stt/` usa `whisper.rs` | el patrón dice cómo se separan de verdad |
| 3 | `componentes/` «banda, relleno, cuaderno»; `i18n/` es.ts · en.ts; tests y `data/` | `componentes/` trae la ventana del cuaderno, `tecla` y `HoyEnWindows`; `i18n/` suma `windows*.ts` y `presencial-nuevo.ts`; hay tres `*-en-windows.rs`; `data/` trae calibración y Windows | la estructura al día |
| 4 | la puerta local, sin más | solo macOS: `plataforma.rs` la marca «no por ahora» en Windows | dicho |
| 5 | «la red, contada en un solo sitio, cada salida declarada»; síntesis sin Windows | tres puertas de Swift y dos de Rust solo en Windows (`contador-de-red`, ADR 022 §4 y 023); en Windows no hay modelo del sistema | dichas las cinco puertas y el ADR 023 |
| 6 | regla 2: «motores puros > 80 %» | no hay gate de cobertura para Rust (`vitest.config.ts` solo mide la UI; el anexo lo admite) | «UI > 50 %…; los motores los cubre `cargo test`, sin umbral» |
| 7 | «peso del binario anotado por PR» | ningún job de CI lo mide (el cierre del S6 y del S7 no lo anotan) | «anotado en el cierre cuando `/release-check` construye el binario» |
| 8 | «Reglas (29…)» | son 29 más la 17-bis | «29 + la 17-bis» |
| 9 | el cierre «reemplaza al summary, la bitácora y la auditoría» | `sprints/SPRINT_NNN-auditoria.md` existe y dos gates lo leen | reemplaza al summary y a la bitácora; la auditoría sigue en su archivo |
| 10 | «test de fuga inyectada en rojo» | existe la demo registrada (S1) y el rojo permanente solo para `ensayos/` | dicho tal cual |
| 11 | scripts: sin `cero-enlaces.mjs` ni `design-sync-bundle.mjs` | el primero es gate de `quality` desde el S6 | añadidos |
| 12 | la matriz de hitos pide Lighthouse, request-id y rate limiting | el perfil escritorio no los tiene | una nota bajo la tabla |
| 13 | pie «puesta al día para el S7» | falta el S8 | «auditada contra el código el 2026-10-12» |

## Derivas del anexo (`docs/REGLAS-ANEXO.md`) — declaradas en un bloque de estado

El § «Angel Ghost» es la v1.40.0 «sin una palabra tocada», y así se queda. Se le puso arriba un bloque **«Estado a 2026-10-12»**
con nueve sustituciones: los cinco checks requeridos (decía tres, y Lighthouse), el stack de Windows (WASAPI, whisper-rs, `PrintWindow`
y `Windows.Media.Ocr`, el Administrador de credenciales), las cinco puertas de red, launchd · Programador de tareas / Llavero ·
Credential Manager / Touch ID · Windows Hello, la puerta local solo en macOS, la estructura (el núcleo es la fuente), la regla 18 de
«dos PR» sustituida por «un solo PR vivo», H1 · H2 como ciclos antiguos frente a H-1 → H-4, y el cierre sin «auditoría aparte». Además:
el título del anexo (kit v2.2.0) y «116 líneas» del núcleo (hoy 135).

## Derivas de los comandos (`.claude/commands/`) — corregidas

| Archivo | Decía | Se corrigió |
|---|---|---|
| `release-check.md` | los tres checks de CI | los cinco, y la lista de PR abiertos = ese uno |
| `release-check.md` | casilla 7, `cargo audit` manual «porque la CI no lo corre» | la CI lo corre en `build-escritorio`; la casilla solo confirma el ADR de la excepción |
| `release-check.md` | casilla 9, el barrido de cero enlaces como paso manual | lo corre `quality`; lo manual es que nadie ofrezca un enlace de descarga |
| `deploy-check.md` | la plantilla web del kit, sin aviso | una línea: no aplica, usa `/release-check` |

## No verificable aquí (va al gate humano del hito)

La invisibilidad graduada por plataforma (regla dura 6: Zoom · Meet · Teams, por versión del sistema), la ficha publicada en CV Viva,
y la latencia fin de turno → ficha ≤ 4 s en el PC (el runner da una cifra manual, no la que vale).

## Lo que el texto nombra y estaba pendiente de este sprint — recomprobado al cerrar la fase 3

| Afirmación | Estado al cerrar el S8 |
|---|---|
| manual ES/EN (reglas 7 y 9) | `docs/MANUAL-EN.md` existe y `manual-bilingue` ata su estructura al manual en español |
| brochure (regla 13) | existe: `docs/BROCHURE.html` (20 funcionalidades, Mac y Windows, es/en) + `docs/brochure-export.json` al día + la entrada «Conoce Angel Ghost» en Honestidad; `brochure.test.ts` (21) y `brochure.spec.ts` (30) lo vigilan, y el guion está «maquetado, no visto» para el gate. Copiar el export a CV Viva y publicar la ficha son del usuario |
| documentos de arquitectura al día (BLUEPRINT) | `docs/BLUEPRINT.html` as-built al 2026-10-12 (commit `a485a47`, 89 comandos, cinco jobs de CI, «La misma frontera, dos sistemas»), con `design-system.md` 1.18.0 y `design-sync/` regenerado; publicar `design-sync/` con `/design-sync` es del usuario |
| guía con «Gate del H-1» | existe: Mac 8 paradas ~20 min y PC 9 paradas ~20 min, con su matriz de la regla 24; `guia-cuadra` vigila el techo por máquina; las filas 24 a 26 de la tabla U (textos y el brochure) van aparte, sin techo |

## Sin deriva

Unas noventa afirmaciones comprobadas contra el código: la estructura (las ocho pantallas, `contrato.generado.ts` con su cabecera,
`bin/ghost.rs`, `nativo/`, `data/`, los 23 ADR), el stack (Tauri 2, TypeScript strict, Tailwind v4, tantivy, XChaCha20-Poly1305,
fin de turno a 320 ms, `AG_SINTESIS=mock`, sin Sentry · Supabase · Vercel), las reglas duras (los gates `pista-por-quien`,
`cero-huellas-de-voz`, `contador-de-red`, `puerta-solo-local`, `radar-solo-este-mac`, `proveedores-con-su-retencion`,
`vocabulario-vetado`, `auditoria-con-sitio`, `cargo-test-sin-hardware` y `hook-secretos` existen y `PROTEGIDOS` incluye el puente de
Swift), las reglas de una línea mecánicas (6 con su hook y su ruleset, 17 a 19, 22, 23, 25 a 29: un solo worktree y ningún stash, y
las parejas de Tauri en la misma versión), los patrones de dominio y el workflow de cinco pasos con la tabla de modelos y la matriz de hitos.
