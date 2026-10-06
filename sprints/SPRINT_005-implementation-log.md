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

### El primer push salió en rojo: un aviso nuevo de `pnpm audit`

La CI del primer push (`434e9c7`) dejó `quality` en rojo y `e2e` y `build-escritorio` saltados por él (regla 15: un
saltado no es un verde). Lo leyó `gh pr checks` en el mismo momento. Causa: `pnpm audit --audit-level high` encontró un
aviso **nuevo, alto y con parche**, GHSA-68fv-2mgg-jv7q (`source-map-js` < 1.2.2, transitivo de `@tailwindcss/node` y del
`css-tree` de `jsdom`). Regla 18: un aviso con parche no se excepciona, se sube la dependencia. `pnpm update
source-map-js` la llevó a 1.2.2 dentro de sus rangos (`c689199`). Después, `pnpm audit` dijo «No known vulnerabilities
found» y `verificar-dependencias`, «379 paquetes, ninguno por debajo de origin/main». La CI del segundo push (`1bdc2c4`)
dio `quality`, `e2e` y `build-escritorio` en `success`, cada uno con su conclusión propia. Es una dependencia de JS en
tiempo de construcción, no de Tauri: no cuenta como el «lote de dependencias» que obliga a `pnpm tauri build` (regla 27),
y el `/release-check` del cierre construye el binario igual.

### La constitución, regenerada (`1bdc2c4`)

Modo regenerar (método v1.39.0) **desde el `CLAUDE.md` de la app**, no desde la copia de la planeadora
(`ordenes/CLAUDE-md-para-app.md`), que se quedó en el estampado v1.27 sin las reglas 19–24 de esta casa y con `[DOMAIN]`
en los patrones (K-S5-1). Lo que entra:

- **cabecera con la frase centinela**: «Constitución regenerada con el kit v1.40.0 (modo regenerar, método v1.39.0) el
  2026-10-05»;
- **reglas nuevas 25, 26 y 27** (las 26, 27 y 28 del kit, citadas por nombre): worktrees prohibidos · la evidencia se
  escribe después del hecho (con su pareja de la casa, `git diff` línea a línea tras toda edición programática) · parejas
  de versiones entre dos ecosistemas;
- regla 7 (el hook falla cerrado) · regla 10 (tres clases de mirada, kit v1.40.0; la página guardada, n/a) · regla 15
  (`demo-rojo.sh` endurecido; se retira la salvedad local de `--puerto`) · regla 18 (degradaciones declaradas y bajadas
  forzadas) · regla 20 (estado por hallazgo) · regla 22 (el literal `--skip en_vivo_`);
- workflow: el PR nace en borrador con la línea del merge, un solo PR de cierre no vacío, la auditoría por superficies;
  la plantilla del summary gana «## Para mergear»;
- **n/a declarado** en «Adaptaciones del perfil escritorio»: Lighthouse y su margen, `build-como-proveedor`, el bundle
  contra `merge-base`, la página guardada, la maqueta servida (aquí `file://` es el modo real) y las reglas de perfiles
  de `diseno-ui` § 5.

**Lo que NO se tocó todavía, a propósito:** la regla dura 4 («la atribución de hablante se resuelve por pista») y «Qué es
esta app» (dos pistas) se ajustan en la fase 1, cuando `Pista::Sala` exista: escribirlo antes afirmaría algo que el
código aún no hace.

### ADRs, antes de su fase

- **ADR 020 «el modo presencial»** (`decisions/020-el-modo-presencial.md`): una pista y lo que no se abre, `Pista::Sala`
  y `quien()`, quién dijo qué (nadie, y se dice), las cinco candidatas de tolerancia y la fila «sin filtro», el protocolo
  de medición del kit v4 con el criterio de elección escrito **antes** de medir, «lo tuyo queda» apagado, el idioma de la
  sala, la voz al oído que espera, Windows al lado y la banda que sigue protegida (§9, desviación 1). La sección
  «Medición» queda vacía hasta el STOP de la fase 1. El porqué: [S12b] SocialMind, [S12c] ChatAR y [S12d] ChatMuse; el 4-T.
  Las referencias de código del ADR se comprobaron contra el árbol (`disparo/mod.rs:109`, `lib.rs:1727`, `lib.rs:2149`,
  `Sesion.tsx:171`, `habla/mod.rs:177`).
- **ADR 021 «Tauri iOS, el spike»**: «no corrido: decisión del usuario 2026-10-05», con lo que el spike medirá cuando se
  autorice y sus riesgos conocidos. D7' sigue en el roadmap con esa razón.
- **ADR 002, enmienda 9**: `idiomasDePista.sala` (de fábrica, el del cliente) y lo que presencial **no** guarda (tus
  turnos, propuestas); el modo y la forma de la ficha no se recuerdan. Inventario del efímero: sin cambios.
- **ADR 009, enmienda 1**: la pista única de la sala lleva un idioma, y la consecuencia medida se dice donde se elige.

### Las maquetas

- **`sesion.html` — mirada de DECISIÓN**: dos estados nuevos, **sprint 5 · presencial** y **sprint 5 · presencial · en
  marcha**, con su nota de la sala de diseño, el carril («Sin sesión · 0 B» / «Presencial · 0 B») y la tarjeta del índice
  (`<i>presencial</i>`, C19). Componentes canon, ninguno nuevo: la tarjeta, la fila, `.ventanas` como selector de dos
  opciones (como «La banda: arriba · abajo»), `.selector`, `.franja mute` y `.estado`.
- **FORMA y TEXTO, «maquetada, no vista»** (registradas en `docs/diseno/README.md`, 2026-10-05): `banda.html` (sprint 5 ·
  presencial · presencial · al oído, la sala habla · presencial · transcript) · `honestidad.html` (sprint 5 · en
  presencial) · `idioma.html` (sprint 5 · el idioma de la sala).
- **Lo que se corrigió leyendo las capturas** (cada una leída como imagen después de generarla):
  1. el icono de la franja del PC con Windows era el de `#i-mac`, que se ve como la manzana de Apple: pasó a `#i-video`;
  2. en inglés, la tecla `⌃⌥V` caía a una línea propia bajo «The card», y al impedirlo el selector se partía en dos: la
     tecla pasó a la línea de abajo («⌃⌥V switches») y el selector quedó en una línea;
  3. el selector del idioma de la sala decía «Español / English»; Idioma usa códigos (`es-ES`, `en-US`) y ahora también.
- **Encuadres leídos como imagen** (después de cada corrida del arnés): Sesión presencial en oscuro/es, claro/en (dos
  veces, antes y después del arreglo) y en marcha en claro/en y oscuro/es; la banda presencial en oscuro/en, la píldora en
  claro/es y el transcript en oscuro/es; Honestidad en presencial en claro/en; Idioma con la sala en oscuro/en. El inglés
  es el extremo de longitud de esta app y se leyó en cada página (regla 26).
- **`design-sync/` regenerado** (`node scripts/design-sync-bundle.mjs`: 21 archivos, 3 escritos — las tarjetas de la
  banda de s1, s3 y s4, que copian `banda.html`); `design-sync-espejo.test.ts` pasó de rojo a verde.

### `maqueta-cabe` cazó dos desbordes que el arnés de capturas no veía

El arnés (`scripts/capturar-maqueta.mjs`) dijo «desbordes: ninguno» con los estados nuevos. El gate de verdad,
`tests/e2e/maqueta-cabe.spec.ts` (corrido en local, macOS, `PUERTO_E2E=3417`), los cazó al nacer:
`idioma.html · s5-sala · es/en: el área que desplaza +113 px` y `sesion.html · s5-presencial · es: +11 px`. Se pagaron:
en Idioma, el estado de la sala no enseña la franja «Transcribe tu Mac»; en Sesión, la frase de la tarjeta del modo cabe
en una línea («Solo el micrófono de tu Mac: ni el audio del sistema ni la pantalla.»). Después, `maqueta-cabe` y
`maqueta-interaccion` pasaron (2 de 2).

**Por qué el arnés no lo vio, y dos defectos previos pagados aquí:** su comprobación de desbordes solo miraba la banda,
el panel y la píldora, **nunca el área que desplaza de las ventanas**; y su recorte del artefacto solo buscaba esas tres
cosas, así que **en toda página de ventana salía con código 2** («sin recorte») aunque todo estuviera bien: un rojo
permanente que nadie podía leer. Ahora mira `.ventana .contenido` (tolerancia de 1 px, la de `maqueta-cabe`) y recorta
`.ventana`. Sesión, Idioma, Honestidad y la banda salen con 0. **Su rojo, con `demo-rojo.sh`:** devolviendo a la tarjeta
del modo la frase larga, el arnés salió en rojo y nombró «s5-presencial · dark · es  contenido → alto +11px»; restaurado,
verde.

### Pruebas de la fase (corridas en local, 2026-10-05)

- `pnpm exec vitest run`: 56 archivos, 472 de 472 (con los tres tests nuevos del kit y el estado por hallazgo).
- `pnpm typecheck` y `pnpm lint`: limpios.
- `maqueta-cabe` y `maqueta-interaccion` (Playwright, proyecto `ventana-principal`): 2 de 2.
- El arnés de capturas sobre `sesion.html`, `banda.html`, `honestidad.html` e `idioma.html`: los cuatro con código 0, la
  pasada de interacción («cada control de la barra cambió algo al pulsarlo») y sin errores de la página.
- `cargo test` no se corrió en esta fase: no se tocó Rust.

### Regla 22 de la app (24 del kit) en la fase 0

Nada de lo que se corrió toca una protección del Mac: el arnés de capturas abre Chromium sin permisos y solo lee
`docs/diseno/`; `pnpm update` y `pnpm audit` hablan con el registro de npm, como cualquier instalación. No hubo aviso,
contraseña ni diálogo. La única fila del sprint, el micrófono en el modo nuevo, sigue para la corrida en vivo de la fase 2.

### Fricciones del kit (K#), para la planeadora

- **K-S5-1 — la copia de la constitución de la planeadora se quedó atrás.** `ordenes/CLAUDE-md-para-app.md` de esta app
  es el estampado v1.27 con dos secciones de delta al final; le faltan las reglas 19–24 propias y los patrones de dominio
  (`[DOMAIN]`). Regenerar «desde la copia de la planeadora», como dice el plan del sprint, habría borrado seis reglas de la
  casa. Se regeneró desde el `CLAUDE.md` de la app. Sugerencia: que el modo regenerar de `/nueva-app` parta del `CLAUDE.md`
  real de la app (lectura) y no de su copia.
- **K-S5-2 — `release-check` del kit cita las reglas por su número del kit** («Regla 24», «Regla 28»); en una app con otra
  numeración el lector busca la regla equivocada. La copia de esta casa añade el número local y el nombre. Sugerencia:
  que el comando las cite por nombre, como pide la regla 24.
- **K-S5-3 — un arnés de capturas que solo mira la banda.** No es del kit (es de esta app), pero la lección viaja: un
  arnés que dice «desbordes: ninguno» sin medir el área que desplaza da una tranquilidad falsa, y uno que sale siempre con
  código 2 es un gate que nadie lee. Lo pagó esta fase.

## Desviación del plan

1. **La banda y su relleno siguen en presencial** (la orden pedía no crearlos): con la banda protegida, el relleno es lo
   único que tapa lo que hay detrás si en la sala se comparte la pantalla en una tele. Nunca se acopla ni se escribe
   `AXPosition` (ADR 020 §9).
2. **El radar de procesos sigue en presencial** (como en el ensayo): solo mira tu Mac. El de pantalla no corre.
3. **La maniobra §10 pasa a la fase 2** (la orden la ponía en la fase 0): la fase 0 se queda con lo que necesita la mirada.
4. **La constitución se regenera desde el `CLAUDE.md` de la app**, no desde la copia de la planeadora (K-S5-1).
5. **Las reglas 26, 27 y 28 del kit entran como 25, 26 y 27 de esta casa**, citadas por nombre.
6. **`source-map-js` 1.2.2** entra en el sprint por un aviso de seguridad nuevo (no estaba en el plan).
