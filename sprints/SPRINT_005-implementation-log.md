# Bitácora — Sprint 005 «En la mesa» (ciclo H2, sprint 2 de 3)

Branch `sprint-005/en-la-mesa`, desde `main` en `3f578d9` (el S4 mergeado; la CI de ese merge, con sus tres checks en
`success`). Orden `SPRINT_005-orden.md` (aprobada en el G-Plan del 2026-10-05) · plan autoritativo `SPRINT_005.md` · kit
v1.40.0 · método v1.42.0 · estándares v2.19.0. El Acto 2 del H1 y la corrida en vivo del ensayo van aparte: este sprint no
los espera ni los toca, y no toca el ensayo.

Plan aprobado el 2026-10-05, con su bloque de arranque (Opus 5.5: medio en las fases 0 y 4, alto en la 1 y la 2) y el
«construye» del usuario.

**Las dos preguntas cerradas de la orden** se respondieron por default en el G-Plan (2026-10-05) y no se vuelven a hacer:
1 = **(b) sin Xcode** (la fase 3 no corre; ADR 021 «no corrido») · 2 = **(a) la voz al oído por defecto, la banda a una
tecla** (la confirma la mirada de DECISIÓN).

## Las decisiones que el plan tomó (en llano; aprobadas con el plan)

1. **La banda sigue protegida y con su relleno en presencial.** Si en la sala compartes tu pantalla en una tele, el
   cliente no ve la banda. Nunca se acopla ni se escribe `AXPosition`, y al empezar se suelta lo acoplado (desviación 1).
2. **Sin propuestas automáticas en presencial**: la app no sabe de quién es cada frase. Tus turnos tampoco se guardan.
3. **El radar de pantalla no corre; el de procesos sigue**, como en el ensayo (desviación 2).
4. **El idioma de la sala es una preferencia nueva**; de fábrica, el del cliente. Toda la sala va en una lengua.
5. **⌃⌥A funciona en presencial** con lo último que oyó la sala, sin filtro.
6. **«Al oído» viene marcado al empezar**, salvo con altavoces que la app reconoce; la voz espera a que la sala calle,
   con tope, y si no calla la ficha se queda en la banda.
7. **La sugerencia usa la frase que disparó la ficha**, no la última que se dijo.
8. **El disparo por silencio arranca apagado en la sala**; lo decide la medición.
9. **La NDA que prohíbe transcribir también bloquea presencial**, y lo comprueba Rust.
10. **La medición** de la regla va en la CI sobre texto y cortes de turno; la transcripción de las dos voces, en local.
11. **La maniobra §10 pasa a la fase 2** (desviación 3).
12. **Un solo PR para todo el sprint**, en borrador desde el primer push.

---

## Fase 0 · Constitución, delta del kit, maqueta y ADRs

### El delta del kit v1.37→v1.40 (citado por nombre)

- **`scripts/demo-rojo.sh`** ← kit v1.40.0 (el endurecido de ds S6: `--debe-nombrar`, `--minimo-tests`, exit 126/127 y
  una señal no son rojo, restauración ante interrupción, presencia y ausencia con Python también en varias líneas; y
  `--puerto` que **solo mata procesos de este repo**, K-S4-7). La salvedad local de la regla 15 («solo con un puerto
  propio») deja de hacer falta.
- **Hook PreToolUse de secretos que FALLA CERRADO** (kit v1.37.0) en `.claude/settings.json`, con el mensaje del
  Llavero de esta casa, `KIT_SIN_GITLEAKS=1` a sabiendas y su prueba `tests/unit/hook-secretos.test.ts` (la del kit,
  que corre el comando real con un PATH sin herramientas).
- **`scripts/verificar-dependencias.mjs`** ← kit v1.39.0: degradaciones DECLARADAS en
  `scripts/degradaciones-permitidas.json` (hoy `[]`; una entrada sin uso falla), cada línea mayor comparada, y las
  **bajadas forzadas** que el registro confirma exactas. Prueba `tests/unit/verificar-dependencias.test.ts` (la del kit).
  Corrida contra `origin/main`: «✓ 379 paquetes, ninguno por debajo de origin/main».
- **`tests/unit/tauri-a-la-par.test.ts`** = la plantilla del kit v1.40.0 (que nació de este test), con su origen de esta
  casa conservado en el comentario.
- **`tests/unit/auditoria-con-sitio.test.ts`** exige además el **ESTADO** de cada hallazgo (kit v1.40.0: `pagado` ·
  `deuda` · `descartado`, más el `irrecuperable` de esta casa). Al nacer la regla, el S1 tenía once sin estado (C1 y
  A1–A10; los contó un script de diagnóstico antes de escribir el test): se completaron con su pago, que vive en la tabla
  de pagos de `SPRINT_001-summary.md`. El S2, el S3 y el S4 ya lo tenían en todas sus filas.
- **`.claude/COMANDOS.md`** sustituye a `.claude/commands/README.md` (kit v1.38.0, K-S6-1: aparecía como `/README`).
- **`.claude/commands/`:** `release-check` §1 (regla 25), §3 (regla 28) y §4 (regla 24), conservando la casilla propia
  del centinela; `audita-sprint` ← kit (Fase 1 por superficies, frases de evidencia en la segunda casilla 4, estado por
  hallazgo); `deploy-check` ← kit (n/a en esta casa: es del perfil web); `plan-sprint` con la nota de que el kit
  recuperó en v1.39.1 lo que esta casa conservó.
- **Ya estaban:** `verificar-dependencias` en `quality` y `--skip en_vivo_` en `build-escritorio` (`ci.yml`), y
  `.demo-rojo/` en `.gitignore`.

### Los rojos del delta (regla 15, con el `scripts/demo-rojo.sh` nuevo)

| Gate                                                  | Mutación                                                                                                                       | Rojo (lo que nombró)                                                                                              | Verde tras restaurar |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------- | -------------------- |
| `hook-secretos.test.ts`                               | `.claude/settings.json`: la rama «faltan gitleaks o jq» sale con `exit 0` en vez de `exit 2` (el hook vuelve a fallar abierto) | «sin gitleaks ni jq bloquea, y lo dice»: esperaba 2, recibió 0 · 1 falló, 2 pasaron                               | 3 de 3               |
| `verificar-dependencias.test.ts`                      | `exacta()` acepta cualquier rango (`rango !== null`)                                                                           | «rojo: una bajada que el rango declarado admite es pnpm degradando» · 1 falló, 5 pasaron                          | 6 de 6               |
| `tauri-a-la-par.test.ts`                              | `src-tauri/Cargo.lock`: `tauri` 2.12.1 → 2.11.0 (la mutación de dos líneas que el `demo-rojo.sh` viejo no sabía verificar)     | «@tauri-apps/api ↔ tauri»: «está en 2.12.1 y tauri en 2.11.0» · 1 falló, 2 pasaron                                | 3 de 3               |
| `auditoria-con-sitio.test.ts` (estado)                | `SPRINT_004-auditoria.md`: el estado de B2 («pagado · desviación 26») cambiado por un texto sin estado                         | «cada hallazgo dice su ESTADO»: nombró `B2` · 1 falló, 12 pasaron                                                 | 13 de 13             |
| `demo-rojo.sh` mismo (¿puede rechazar un rojo falso?) | `degradaciones-permitidas.json` `[]` → `[ ]` con un gate que no existe                                                         | «✗ el gate no corrió (exit 127: comando inexistente o no ejecutable) — eso no es un rojo»; salió con 1 y restauró | —                    |
