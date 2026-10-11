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

### La mirada de DECISIÓN (2026-10-05)

`docs/diseno/sesion.html`, estados «sprint 5 · presencial» y «sprint 5 · presencial · en marcha», presentada en matriz de
una fila al cerrar la fase. Primero llegó un **«sí»** a secas; por la regla 10 se le repreguntó qué vio al abrirla, y
respondió: **«Si la abri y la apruebo, sigue»**. Queda **aprobada** y registrada en `docs/diseno/README.md`. Confirma la
respuesta (a) de la pregunta 2 del G-Plan: la ficha al oído de fábrica, la banda a una tecla. El «sigue» es también el
paso de fase: arranca la fase 1.

---

## Fase 1 · El núcleo presencial y su medición

### Punto de control (2026-10-05, pedido por el usuario para compactar)

**Hecho y en verde** (pasos 1 y 2 del plan, y la mitad del 3):

1. **`Pista::Sala` y `quien()`** (`44bbde4`). `capture::Pista` gana `Sala`; nacen `Quien { Tuyo, Cliente, SinAtribuir }`
   y `Fuente { Microfono, Sistema }` con `quien()` y `fuente()` sin comodín. Todos los sitios que comparaban pistas
   preguntan `quien()`: el disparador, el eco, el silencio (`Ventana::ultimo_que`), `⌃⌥A`/`⌃⌥V` y la sugerencia
   (`Turno::pudo_decirlo_el_cliente`), «muere al cerrar» (la sala cuenta), «conservar mis turnos» (la sala nunca) y
   las propuestas (`De::del_turno` es `Option`: la sala no propone). La reunión no cambió: 577 pruebas de la librería.
2. **`Modo::Presencial`** (`src-tauri/src/modo.rs`): `que_abre(modo) → Apertura { pistas, pantalla, acople, tus_turnos,
   propuestas }`, con la pantalla tras un **`PermisoDePantalla`** que solo crea `que_abre` (en presencial no hay con
   qué llamar a `arrancar_la_pantalla`). `puede_empezar(modo, nda)`: **la NDA que lo prohíbe, comprobada en Rust**,
   también en reunión (solo deja solo notas).
3. **`empezar` reescrito** (`lib.rs`): la NDA primero; **`ElModo` marcado lo primero**; en presencial
   `acople::soltar`; `se_puede_acoplar()` en `acoplar_segun_el_borde` y `asentar_banda` (cubre el hilo de arriba, el
   latido, `⌃⌥B`, el asa y `⌃⌥V`); el hilo del acople al empezar solo si el modo acopla; `reunion::al_empezar(&app,
   tus_turnos)` con `conservar_en_este_modo` (la preferencia no se toca); las propuestas solo si el modo las abre;
   `ElModo` vuelve a `None` al terminar y con el corte. Comando nuevo **`empezar_presencial`** (manifiesto,
   `generate_handler!`, capability de la ventana principal, `SENSIBLES`) y `empezarPresencial()` en
   `src/jurisdiccion.ts`.
4. **`Escucha::arrancar(Pistas, …)`** con `Pistas::{Reunion { consultor, cliente }, Sala { idioma }}`; la costura
   **`Escucha::sobre_anillos`** (`#[doc(hidden)]`, sin grifos); `estado()` con `presencial` (la fila del micrófono es
   la sala y la del sistema va cerrada sin motivo); `alguien_hablando()`.
5. **Preferencias:** `idiomasDePista.sala` (`Option`, «la del cliente» de fábrica, una torcida vuelve a eso) y
   `fijar_idioma_de_pista("sala")`.
6. **Contrato (regla 19):** regenerado (`EstadoDeEscucha.presencial`, `IdiomasDePista.sala`); los tipos de TS al día
   (`Pista` con `"sala"`). El gate de la forma se vio en rojo solo al regenerar (los tipos de TS no conocían los
   campos). `contrato-con-lectores` declara los dos huérfanos como deuda **que se paga en la fase 2** (Sesión,
   Honestidad e Idioma), con su sitio.

**Pruebas en este punto (corridas, 2026-10-05):** `AG_SIN_HARDWARE=1 cargo test --locked`: 581 de la librería más los
de integración que no tocan el Mac, todos en verde · `cargo clippy --locked --all-targets -- -D warnings`: limpio ·
`pnpm exec vitest run`: 57 archivos, 474 de 474 · `pnpm typecheck` y `pnpm lint`: limpios. **La CI de `00c1eba`**
(corrida 37405541393, leída con `gh pr checks 15` después de compactar): `quality`, `e2e` y `build-escritorio` en
`success` (12 min 58 s el de macOS).

**Rojos de esta parte (con `scripts/demo-rojo.sh`):**

| Gate o test | Mutación | Rojo (lo que nombró) | Verde |
|---|---|---|---|
| `tests/unit/pista-por-quien.test.ts` (nuevo) | `disparo/mod.rs`: `Quien::Tuyo if turno.pista == Pista::Sistema => None` | `src-tauri/src/disparo/mod.rs:110` · comparación con una pista | 2 de 2 |
| `modo::pruebas::presencial_abre_la_sala_y_nada_mas` | presencial con `pantalla: Some(PermisoDePantalla(()))` | el test, en `modo.rs:134` · 1 falló, 3 pasaron | 4 de 4 |
| `modo::pruebas::la_nda_que_lo_prohibe_solo_deja_solo_notas` | la NDA solo bloquea `Modo::Normal` | el test, en `modo.rs:157` | 4 de 4 |
| `capture::tests::la_sala_no_tiene_dueno_y_entra_por_el_microfono` | `Pista::Sala => Quien::Tuyo` | el test, en `capture/mod.rs:212` | 2 de 2 |

### Lo que cambia en la sala, y el orden de `empezar`, con sus pruebas (2026-10-05)

- **El interruptor «Conservar mis turnos» pregunta al modo** (`lib.rs`, `reunion::conservar_mis_turnos(app, si,
  el_modo_lo_deja)`): en presencial recuerda tu preferencia y el cuaderno la usa apagada. Sesión lo desactivará en la
  fase 2; esto es el cinturón por debajo.
- **`escucha::Pistas::cuales()`**: qué pista abre cada modo, pura. `arrancar` abre exactamente esas, y
  `es_presencial` sale de ellas (antes eran dos `match` paralelos, uno en `arrancar` y otro en `sobre_anillos`).
- **Pruebas nuevas** (593 de la librería en verde, `cargo clippy -D warnings` limpio):
  - `notas::la_sala_nunca_entra_como_tuya` y `propuestas::la_sala_no_propone_nada` (con una frase de control del
    cliente que sí propone, para que el test no sea vacuo);
  - `reunion::conservar_mis_turnos_solo_si_el_modo_lo_deja`;
  - `escucha::la_sala_abre_una_pista_y_de_su_turno_sale_la_ficha`: **la sala de punta a punta sobre un anillo**, sin
    grifos: el estado (presencial, la sala abierta, el sistema cerrado sin avería), el turno con la pista «sala», en
    el idioma de la sala, sin eco, y su ficha; `la_sala_abre_solo_la_sala_y_la_reunion_sus_dos_pistas`;
    `una_reunion_sobre_anillos_no_es_presencial`; `la_sala_no_se_marca_como_eco`;
  - `lib.rs`, módulo nuevo `pruebas_del_modo_presencial`: el orden de `empezar` (NDA → modo → `soltar` solo si no
    acopla → hilo del acople solo si acopla); **las dos puertas del acople** (cada llamada a `acople::acoplar`,
    `reacoplar` y `acoplar_arriba` vive dentro de `acoplar_segun_el_borde`, `asentar_banda` o `acoplar_arriba`, y las
    dos puertas preguntan por el modo antes); el modo se marca solo en `empezar` y se borra en «Terminar» y en `⌥⎋`;
    **`sobre_anillos` no aparece en `lib.rs`**; el interruptor pregunta al modo.
- **Dos agujas de mis tests de fuente estaban mal medidas y lo dijo la primera corrida, no el código:** una ventana de
  60 caracteres que no llegaba al `if !acopla {`, y `concat!("acople::", "acoplar_arriba(")`, cuya segunda mitad
  contiene la aguja entera y se contaba a sí misma. Se partió por otro sitio.
- **`git diff` línea a línea (regla 26) cazó una inserción mía en mal sitio:** el test de la sala de `propuestas` quedó
  entre el comentario de `el_eco_es_la_voz_del_cliente` y su `#[test]`, y le robó la documentación. Se movió encima.

**Rojos (con `scripts/demo-rojo.sh`, cada uno `--debe-nombrar` el mensaje de su aserción, `--esperar-verde` con el
mismo filtro y `--minimo-tests 1`; las trece salieron con código 0: rojo nombrando lo esperado, restaurado y verificado
con Python y `cmp`, verde con 1 prueba):**

| # | Test | Mutación | Nombró |
|---|---|---|---|
| D1 | `notas::la_sala_nunca_entra_como_tuya` | `Quien::SinAtribuir => !turno.eco` | «un turno de la sala entró como tuyo» |
| D2 | `propuestas::la_sala_no_propone_nada` | `Quien::SinAtribuir => Some(De::Cliente)` | «la sala propuso una nota» |
| D3 | `reunion::conservar_mis_turnos_solo_si_el_modo_lo_deja` | `preferencia \|\| el_modo_lo_deja` | «en presencial no se conservan tus turnos» |
| D4 | `escucha::la_sala_abre_solo_la_sala_…` | `Pistas::Sala` abre `Pista::Microfono` | «presencial abre otra cosa que la sala» |
| D5 | `escucha::la_sala_no_se_marca_como_eco` | `Quien::Tuyo \| Quien::SinAtribuir =>` en `atender` | «un turno de la sala salió marcado como eco» |
| D6 | `escucha::la_sala_abre_una_pista_y_de_su_turno_sale_la_ficha` | `Quien::SinAtribuir => None` en `disparo::mirar` | «de la pregunta oída en la sala no salió ficha» |
| D7 | `escucha::una_reunion_sobre_anillos_no_es_presencial` | `es_presencial` = `!pistas.is_empty()` | «una reunión se declaró presencial» |
| D8 | `lib::…::empezar_mira_la_nda_marca_el_modo_y_suelta_antes_de_acoplar` | `if acopla {` delante de `soltar` | «se suelta lo acoplado también en reunión» |
| D9 | el mismo | sin `acopla &&` en el hilo del acople | «el hilo del acople corre sin mirar el modo» |
| D10 | `lib::…::las_dos_puertas_del_acople_preguntan_por_el_modo` | sin la guarda de `asentar_banda` | «falta «se_puede_acoplar(&app)»» |
| D11 | `lib::…::el_modo_se_marca_al_empezar_y_se_borra_…` | sin `marcar_el_modo(&app, None)` en «Terminar» | ««Terminar» deja el modo puesto» |
| D12 | `lib::…::la_app_no_arranca_la_escucha_por_la_costura_…` | `sobre_anillos(` en un comentario de `empezar` | «lib.rs arranca la escucha sobre anillos de prueba» |
| D13 | `lib::…::el_interruptor_de_tus_turnos_pregunta_al_modo` | `conservar_mis_turnos(&app, si, true \|\| el_modo_lo_deja)` | «el interruptor no le pasa al cuaderno lo que dice el modo» |

### La tolerancia a tu voz, sin valores elegidos (2026-10-05)

- **`disparo::Tolerancia`** (`src-tauri/src/disparo/tolerancia.rs`, pura, en un módulo protegido): cinco perillas
  —C1 espera tras una ficha, C2 eco de la ficha, C3 un término tuyo no basta, el silencio y el tope de turno—, cada una
  apagada en `Tolerancia::SIN_FILTRO`, la línea base. `deja(motivo, texto, ahora, ficha_vista)` decide; `perillas()`
  cuenta las movidas (el desempate del kit). C2 compara **palabras con peso** (normalizadas, de cuatro letras o con
  cifra, sin las vacías de los dos idiomas) del turno contra el titular y la línea de la última ficha con resultado.
- **`data/presencial/reglas.json`**, versionado y dentro del binario con `include_str!`: hoy es **la línea base, sin
  ningún valor elegido** (`"estado": "línea base, pendiente de la medición del kit v4"`). La regla la elige la medición
  al STOP; ningún número está cableado en el código.
- **El `Disparador`** gana la tolerancia (de fábrica, la de la casa; `con_tolerancia` para el kit) y la última ficha
  vista, que se olvida con el corte. `mirar` la aplica **solo** a `Quien::SinAtribuir`. **`por_silencio` recibe el turno
  entero**: el silencio tras la sala pasa por la tolerancia (que puede apagarlo) y el que sigue a tu voz no dispara.
- **La escucha le dice al disparador qué ficha salió**: `escucha::del_turno_a_la_ficha` (pública, el camino que el kit
  v4 usa en vez de una copia) y el silencio llaman a `vio_ficha` con cada `Respuesta::Ficha`. El candado del
  disparador se suelta mientras se busca, para que el corte no espere.
- **El tope de turno**: `voz::turno::Turnos::con_tope`; al alcanzarlo el turno se cierra y el siguiente empieza en el
  mismo marco. Solo la sala lo lleva (con el de la regla de la casa: hoy ninguno).
- **Pruebas** (601 de la librería en verde y 2 ignoradas de hardware, clippy limpio, `verify:ephemeral` limpio): `la_regla_de_la_casa_se_lee`,
  `sin_filtro_lo_deja_todo`, `cada_perilla_hace_lo_suyo`, `repetir_la_ficha_se_mide_en_palabras_con_peso`,
  `la_tolerancia_solo_toca_la_sala` (también el silencio: el de la sala, apagado; el tuyo, nunca),
  `el_corte_olvida_la_ficha_vista`, `el_tope_corta_un_turno_que_no_se_calla`,
  `la_ficha_que_sale_la_ve_la_tolerancia_de_la_sala`.

**Rojos (con `scripts/demo-rojo.sh`; los siete con código 0: rojo nombrando lo esperado, restaurado, verde con 1 prueba):**

| # | Test | Mutación | Nombró |
|---|---|---|---|
| T1 | `la_regla_de_la_casa_se_lee` | `reglas.json`: `"esperaTrasFicha"` en vez de `"esperaTrasFichaMs"` | «data/presencial/reglas.json no se lee» |
| T2 | `cada_perilla_hace_lo_suyo` | C1 con la comparación al revés (`>=`) | «C1 dejó disparar durante la espera» |
| T3 | el mismo | C2 con un umbral inalcanzable (`> 1.0 + umbral`) | «C2 dejó disparar tu lectura de la ficha» |
| T4 | `la_tolerancia_solo_toca_la_sala` | la tolerancia aplicada también a `Quien::Cliente` | «la tolerancia de la sala calló al cliente» |
| T5 | `el_corte_olvida_la_ficha_vista` | sin `self.ficha_vista = None` en `reiniciar` | «tras el corte la sala sigue callada por una ficha olvidada» |
| T6 | `el_tope_corta_un_turno_que_no_se_calla` | la rama del tope con `&& false` | «el tope no cortó el turno en trozos de 1 s» |
| T7 | `la_ficha_que_sale_la_ve_la_tolerancia_de_la_sala` | sin `vio_ficha` en `del_turno_a_la_ficha` | «la sala que lee la ficha disparó otra» |

### La sugerencia, con la frase que disparó (decisión 7, 2026-10-05)

`ficha::Aparicion` gana `de_ms` (`#[serde(skip)]`: no cruza a la pantalla ni cambia el contrato), el `hasta_ms` del
turno que la disparó, o `None` si no la disparó un turno. `sintetizar` redacta sobre `stt::el_turno_de_la_ficha(ts,
de_ms)`: ese turno, y **si ya no está entre los recientes, ninguno**, mejor sin sugerencia que una sobre tu respuesta;
sin `de_ms`, el último que pudo decir el cliente, como antes. En una reunión no cambia nada visible: el turno que
dispara suele ser el último del cliente.

Rojos: **S1** `stt::la_sugerencia_usa_el_turno_que_disparo` con `Some(_) => candidatos.next()` nombró «la sugerencia
tomó la última frase de la sala y no la que disparó»; **S2** `lib::…::la_sugerencia_pide_el_turno_que_disparo` con
`de_ms` anulado en `sintetizar` nombró «la sugerencia no usa el turno que disparó». Los dos con código 0 y verde con
1 prueba. Clippy limpio; el contrato, sin cambios (4 de 4).

### El kit v4: las salas, el guion y los tres niveles (2026-10-05)

- **El guion** (`docs/kit-de-prueba/presencial.json`, escrito antes de medir): dos salas con Páramo Azul, **32 turnos en
  español y 30 en inglés**, cada uno con quién lo dice (la verdad, solo para el kit), la pausa antes, si debería traer
  ficha y por qué; **7 fichas debidas por sala**, tus preguntas («¿Les parece si…?», «¿Qué plazo manejan…?»), tus
  lecturas de la ficha en voz alta, una repregunta rápida del cliente, un silencio del cliente que pide ficha y uno
  tuyo que no, y un intercambio con pausas de 200 ms (por debajo de los 320 ms que cierran un turno). Lleva también
  las candidatas que se recorren y el criterio.
- **Las salas** (`scripts/kit-v4-sala.sh`): `say -o` con dos voces por idioma (Reed y Mónica; Reed y Samantha), sin
  sonar; `afconvert` a 16 kHz mono; una mezcla en Python que recorta el silencio de cada clip, pone la pausa del guion,
  baja a 0,45 la voz del cliente (más lejos del micrófono) y un ruido de fondo fijo bajo el suelo del VAD. Corrida
  local: `sala-es.wav` 127,3 s, `sala-en.wav` 123,8 s (unos 4 MB cada uno), con su línea de tiempo en
  `*.tiempos.json`. **El intercambio rápido dura 29,1 s en español y 29,3 s en inglés**: lo escribí para rondar los
  30 s del anillo y salió por debajo; no se retocó el guion para forzar el resultado.
- **Nivel A** (`el_kit_presencial_mide_la_tolerancia`, puro, en la CI): cada turno entra como `Pista::Sala` con el reloj
  de su wav, **por el mismo camino que la escucha** (`escucha::del_turno_a_la_ficha` y la nueva
  `escucha::del_silencio_a_la_ficha`, que `el_silencio_pide_ficha` usa por dentro), con el silencio mirado cada 400 ms
  como el latido de la escucha; las 36 combinaciones de C1 × C2 × C3 × silencio, ordenadas por el criterio, con el
  detalle por motivo y por turno de la línea base y de la primera. Gates: el kit mide el problema (≥ 5 fichas debidas y
  ≥ 3 preguntas tuyas por sala; la línea base con pertinentes y falsas), la latencia < 4 s, y **la regla de la casa no
  pierde más fichas pertinentes que la línea base**.
- **Nivel B** (`el_kit_presencial_corta_los_turnos_de_la_sala`, puro, en la CI): el VAD y el fin de turno de siempre
  sobre los dos wav, con cada tope candidato: turnos cortados, turnos con dos voces, el más largo, los que pasan del
  anillo y los cortes a media frase. Gate: **con la regla de la casa, ningún turno de la sala pasa del anillo**.
- **Nivel C** (`el_kit_presencial_transcribe_la_sala`, `#[ignore = "hardware: …"]`): corta con la regla de la casa,
  transcribe cada turno en el idioma de la sala y da el WER de la sala entera. `manual`: va a la corrida en vivo.

**Rojos de los gates del kit:** **K1** (nivel A) con `reglas.json` en `esperaTrasFichaMs: 10000` y `silencio: false`
nombró «la regla de la casa pierde 13 fichas pertinentes y la línea base 10»; **K2** (nivel B) con el anillo a 25 s
(`capture/anillo.rs`) nombró «un turno de la sala dura 29240 ms, más que el anillo de 25000». Los dos con código 0 y
verde con 1 prueba. **El primer intento de K2 no contó, y está bien que no contara:** con `FIN_MS = 1_000` el VAD
juntaba tantos turnos que saltaba antes la aserción de «el nivel B no mide nada», y `demo-rojo.sh` (v1.40,
`--debe-nombrar`) lo rechazó: «el gate falló (exit 101) pero no nombró 'más que el anillo'».

### Lo primero que encontró el kit: un defecto del disparador en inglés, de siempre (2026-10-05)

La primera corrida del nivel A enseñó en la sala inglesa **tres frases dichas sin pregunta que disparaban como
preguntas**: «We have the same problem with the rural stores.» (del cliente), «That is exactly what…» y «Access is
requested…» (tuyas). La causa no es la sala: `por_que` contaba los auxiliares del inglés (`is`, `have`, `do`, `can`…)
en cualquiera de las tres primeras palabras, como los interrogativos («¿Y cuánto…?»). En inglés un auxiliar pregunta
solo si **abre** la frase. **Es un defecto del disparador desde el sprint 001 y afecta también a la reunión**: el
cliente que dice «We have four sources» por el audio del sistema disparaba igual.

- **Arreglo** (`disparo/mod.rs`): los auxiliares salen de `INTERROGATIVOS` a `AUXILIARES`, que solo cuentan si son la
  primera palabra, o la primera tras una conjunción («And is that included?»). Los interrogativos y el español no
  cambian. Test `los_auxiliares_del_ingles_preguntan_solo_al_abrir_la_frase` (tres afirmaciones que no disparan,
  cuatro preguntas que sí); **rojo** devolviendo los auxiliares a las tres primeras palabras: nombró ««We have the
  same problem with the rural stores.» no es una pregunta»; código 0, verde con 1 prueba.
- **Y destapó un turno mal marcado del kit del sprint 001** (`disparo.json`): «We have four data sources, not three.»
  debía disparar como «cifra en inglés», pero la regla de la cifra mira dígitos —el mismo archivo lo declara como
  limitación con «Somos catorce en la mesa», que no dispara— y **solo pasaba por el «have» de la segunda palabra**.
  Con el arreglo el recall del kit bajó a 0,909, y se corrigió el turno, no el umbral: «We have 4 data sources, not
  3.», en dígitos como escribe las cifras el transcriptor, con el porqué escrito en el propio turno. El kit vuelve a
  1,000 de precisión y 1,000 de recall.
- **En la sala**, con el arreglo: la línea base pasa de 19 falsas y 10 perdidas (la primera corrida local del nivel A,
  antes del arreglo) a **16 falsas y 9 perdidas** (la tabla de abajo).

### La medición del kit v4 (corrida local, 2026-10-05, después del arreglo del inglés)

`AG_SIN_HARDWARE=1 cargo test --locked --test contra-el-mac-de-verdad el_kit -- --test-threads=1 --nocapture`: los
diez kits en verde (el nivel C, ignorado: es de hardware). Las dos salas juntas, **14 fichas debidas** (7 por sala):

| C1 espera tras ficha | C2 eco | C3 término no basta | silencio | pertinentes | falsas (tuyas + cliente) | perdidas | perillas |
|---|---|---|---|---|---|---|---|
| — | — | — | sí | 5 | 16 (16 + 0) | 9 | 0 · **línea base** |
| — | — | sí | sí | 6 | 9 (9 + 0) | **8** | 1 |
| 15 s | — | sí | sí | 6 | 7 (7 + 0) | **8** | 2 |
| 10 s | — | sí | sí | 6 | 8 (8 + 0) | 8 | 2 |
| — | — | sí | no | 5 | 7 (7 + 0) | 9 | 2 |
| 10 s | — | sí | no | 5 | 6 (6 + 0) | 9 | 3 |
| **15 s** | — | **sí** | **no** | 5 | **5 (5 + 0)** | 9 | 3 · **primera por el criterio** |
| 15 s | — | — | sí | 4 | 12 (12 + 0) | 10 | 1 |
| 15 s | — | — | no | 3 | 11 (11 + 0) | 11 | 2 |
| — | — | — | no | 3 | 14 (14 + 0) | 11 | 1 |

(Las 36 filas salen en la salida del test; aquí, las que deciden. **C2 no cambia ninguna fila**: con 0,34 o con 0,50
da lo mismo que apagado, en las 18 parejas.)

**Por motivo, la línea base** (pertinentes / falsas / perdidas): pregunta 3/5/6 · término 0/7/3 · cifra 0/2/0 ·
silencio 2/2/0. **La primera por el criterio:** pregunta 5/3/5 · término 0/0/3 · cifra 0/2/0 · silencio 0/0/1. Le
sobran tus preguntas («¿Les parece si empezamos…?», «Shall we start…?», «What timeline…?») y tus dos cifras en
inglés; le faltan las del cliente que caen justo detrás de una tuya, la repregunta rápida (C1), los términos que
nombra el cliente sin preguntar (C3) y el silencio de la sala inglesa.

**Nivel B** (el VAD y el fin de turno de siempre sobre los wav):

| sala | tope | turnos del guion | cortados | con dos voces | el más largo | pasan del anillo | cortes a media frase |
|---|---|---|---|---|---|---|---|
| sala-es | ninguno | 32 | 26 | 1 | 29,0 s | 0 | 0 |
| sala-es | 15 s | 32 | 27 | 2 | 15,0 s | 0 | 0 |
| sala-es | 25 s | 32 | 27 | 2 | 25,1 s | 0 | 1 |
| sala-en | ninguno | 30 | 25 | 1 | 29,2 s | 0 | 0 |
| sala-en | 15 s | 30 | 26 | 2 | 15,0 s | 0 | 1 |
| sala-en | 25 s | 30 | 26 | 2 | 25,0 s | 0 | 1 |

El intercambio rápido se junta en **un turno de 29 s con las dos voces**, en las dos salas: cabe en el anillo por un
segundo, y la pregunta del cliente que va dentro solo llega al disparador cuando acaba el bloque entero.

**Latencia:** de fin de turno a ficha, 4 ms la peor (sin audio ni modelo; el presupuesto es 4 s).

**Por qué se pierden las 9 de la primera por el criterio** (el kit lo dice desde el commit «kit v4 says what blocked each lost card»: cada perdida, con cuánto
antes salió la ficha anterior y de quién; la tabla no cambió con ese añadido, comparada línea a línea):

- **4 caen justo detrás de una ficha que disparaste tú**, dentro de la espera de 6 s entre fichas que ya existía:
  «¿Y la limpieza de datos…?» (5,1 s después de tu «¿Les parece si empezamos…?»), «Is data cleaning…?» (3,5 s),
  «How many weeks…?» (5,9 s) y «And do those 4 weeks…?» (4,5 s después de tu cifra). Ninguna de las candidatas toca
  esto: la sala no sabe que la primera pregunta era tuya.
- **3 son términos que el cliente nombra sin preguntar** («Lo de Sur del Valle…», «Y el jefe de sistemas…», «Tell me
  more about Sur del Valle»): el precio de C3.
- **1 es la repregunta rápida** («¿Y esas 4 semanas…?», 8,8 s después de la ficha que pidió el cliente): el precio de C1.
- **1 es el silencio del cliente en la sala inglesa**: el precio de apagar el silencio.

**Lo que el kit no puede medir, dicho:** el nivel A trata cada turno por separado, como texto; la sala de verdad
junta turnos (nivel B) y transcribe con errores (nivel C, manual). El corpus del kit es en español: en la sala
inglesa casi ninguna pregunta trae ficha con resultado, así que **C1 y C2 casi no actúan en inglés**.

### Cero huellas de voz: el vocabulario y el gate nuevo (2026-10-05)

- **El vocabulario vetado** (`tests/unit/vocabulario-vetado.test.ts`) suma `voiceprint` y `speaker embedding`: ningún
  copy puede prometer que la app separa voces. **Rojo:** «Reconoce tu voiceprint.» plantado en el `README.md` nombró
  «voiceprint»; verde con 2 pruebas.
- **Gate nuevo `tests/unit/cero-huellas-de-voz.test.ts`** (regla dura 4, ADR 020 §2): barre Rust, el puente de Swift, la
  webview, `Cargo.toml` y `package.json` buscando **nombres de API y de librería** —SoundAnalysis, MFCC, diarización,
  identificación de hablante, voiceprint, x-vector/ECAPA, pyannote, resemblyzer, speechbrain, wespeaker, titanet,
  emociones por la voz, `sentimentScore`—, con límite de palabra (la primera búsqueda a mano cazó «frameCapacity» del
  puente por «ecapa»); y los paquetes de los dos lockfiles **por nombre**, no por texto (los hashes pueden contener
  cualquier cosa). La prosa que cita la regla («no identifica a nadie por su voz», en la cláusula de encargo) no salta.
  **Rojos:** `import SoundAnalysis` en `Transcriptor.swift` nombró «SoundAnalysis (Apple)» en `:22`; un paquete
  `pyannote-rs` en el `Cargo.lock` nombró «pyannote-rs · librería de hablantes». Verde con 3 pruebas los dos.

### El contrato, con la sala (regla 19, 2026-10-05)

Dos muestras nuevas en `contrato.rs`, regeneradas en `src/contrato.generado.ts`: **`TURNO_DE_LA_SALA`** (un `Turno` con
`pista: "sala"`) y **`ESTADO_DE_LA_ESCUCHA_PRESENCIAL`** (`presencial: true`, la sala abierta, el sistema cerrado sin
motivo). Sus lectores en la webview llegan en la fase 2, como ya declara la deuda de `contrato-con-lectores`. **La
muestra del turno también vigila el tipo de TS:** con `Pista = "microfono" | "sistema"` (sin «sala») en
`src/cuaderno.ts`, `pnpm typecheck` fue rojo en `src/contrato.generado.ts(117,5)`: «Type '"sala"' is not assignable to
type 'Pista'»; verde al restaurar.

### El efímero con la sala, la constitución y los LEEME (2026-10-05)

- **`una_sesion_completa` gana el paso 7-quater, la sala**: `Escucha::sobre_anillos` con una sola pista, la sala, y
  **el motor de verdad** sobre un anillo que llena el test (el micrófono no se abre), con el audio del kit; espera un
  turno de la sala (transcrito, o sin texto en la CI) y lo cuenta sin enseñarlo. Después **un turno de la sala con la
  canaria** recorre `del_turno_a_la_ficha` con la tolerancia de la casa, el cuaderno (no entra) y las propuestas (no
  propone). El gate del disco exige `ejercido` con «sala», y el del log, la línea «la sala disparó por» del hijo (que
  el paso corrió) además de que la canaria no salga. **Son tests de hardware** (motor de voz): en local no se corren
  (regla 22); su primera corrida es la de la CI de este push, y lo que la sostiene sin hardware ya está en la
  librería con sus rojos (D5, D6, D1, D2 y T7).
- **`CLAUDE.md`**: la regla dura 4 dice la sala sin dueño, la tolerancia medida y los dos gates que la vigilan; «Qué es
  esta app» nombra el modo presencial (sin leer la pantalla); el patrón de las pistas suma la sala, la tolerancia y
  `modo::que_abre`.
- **Los LEEME del kit** (`docs/kit-de-prueba/LEEME.md`, `audio/LEEME.md`): v4, la fila de `presencial.json` con las
  cifras de la corrida de arriba, la fila del nivel C (`manual`) y las dos salas con cómo se hicieron. La nota del
  audio decía «Cuatro frases» y habría quedado falsa.
- **Pruebas** (local): `pnpm exec vitest run` **477 de 477** en 58 archivos (+3 del gate nuevo); `pnpm typecheck`
  limpio; `cargo clippy --locked --all-targets -- -D warnings` limpio.

### Fase 1 · cierre y STOP de medición (2026-10-05)

**Pruebas al cierre (corridas en local, después del último cambio de código):** `AG_SIN_HARDWARE=1 cargo test --locked`:
librería **604** en verde (2 ignoradas de hardware), contra el Mac **18** (15 de hardware, que corre la CI), puerta
**14**, ghost **5** · `cargo clippy --locked --all-targets -- -D warnings` limpio · `pnpm exec vitest run` **477 de 477**
en 58 archivos · `pnpm typecheck`, `pnpm lint` y `pnpm verify:ephemeral` limpios. Rojos de la fase con
`scripts/demo-rojo.sh`: los 4 del punto de control, **D1–D13**, **T1–T7**, **S1–S2**, **K1–K2**, el de los auxiliares
del inglés, los 3 de las huellas y el vocabulario, y el del tipo `Pista` de TS: **33**, cada uno con código 0.

**Desviaciones de la fase 1 respecto del plan:**

1. **Un arreglo del disparador que no estaba en el plan**: los auxiliares del inglés (arriba). Lo encontró el kit v4 y
   afecta también a la reunión.
2. **Un turno del kit del sprint 001 corregido** («four… three» → «4… 3»), con su porqué en el propio turno.
3. **La voz que espera al silencio de la sala** (ADR 020 §7) queda para la fase 2, con la interfaz, como dice el plan.
   `Escucha::alguien_hablando()` ya existe para ella.
4. **El paso de la sala en la sesión completa** no se vio en rojo en local: es de hardware (regla 22). Su primera
   corrida es la de la CI, y lo que lo sostiene sin hardware tiene sus rojos en la librería.

**STOP de medición:** la tabla está arriba («La medición del kit v4») y la regla la eliges tú. Lo que se elija entra en
la fase 2 a `data/presencial/reglas.json`, y el ADR 020 gana su sección «Medición» con estas cifras.

**La CI del cierre de la fase 1** (`379f8ef`, corrida 37409586560, leída con `gh pr checks 15`): `quality`, `e2e` y
`build-escritorio` en `success` (14 min 34 s el de macOS). En el paso de hardware pasaron
`la_canaria_del_cliente_no_aparece_en_el_log` (que exige la línea «la sala disparó por» del hijo: el paso de la sala
corrió) y `una_sesion_completa_no_deja_nada_en_el_disco_salvo_el_indice_del_corpus`. La tabla del kit que imprimió la CI
coincide fila por fila con la local; la latencia peor, 2 ms.

---

## La directiva del 2026-10-06 y la respuesta al STOP (2026-10-10)

**La directiva del usuario del 2026-10-06** (en la cabecera de la orden, que prevalece sobre el resto): el sprint se
reduce a las fases 1 y 2 y cierra en **modo mínimo** (CI verde con conclusión propia por check · una auditoría de una
pasada, un auditor, solo lectura: críticos y altos se pagan, medios y bajos van al summary como deuda con sitio · un
summary breve dentro del PR · PR normal y el merge del usuario). Fuera: la fase 4 (guía, `design-sync/`, brochure,
BLUEPRINT, `/release-check`, gate ⭐), la maniobra §10 y el resto de la fase 0. **La fase 0 ya se había ejecutado y
aprobado el 2026-10-05**, antes de la directiva: lo hecho se queda (constitución, delta del kit, ADR 020 y 021,
enmiendas, maqueta con su mirada) y lo declara el summary. Paradas que quedan: el STOP de fin de fase 2, y la fila de
la regla 22 (el micrófono en el modo nuevo), que se enseña igual.

**La respuesta al STOP de medición** llegó con el hueco de la elección sin llenar («La regla del disparador elegida es:
<respuesta a Q2>»). El STOP había dicho que, sin respuesta, valía lo marcado por defecto, y eso se aplicó: **1 = A**
(espera de 15 s tras una ficha, un término suelto no basta, sin ficha por silencio) · **2 = tope de 15 s** · **3 = no**
(no se mide otra regla para las preguntas que van detrás de las tuyas; queda como limitación). Cambiarla es una línea de
`data/presencial/reglas.json`.

## Fase 2 · La regla elegida, la interfaz y Windows al lado

### La regla elegida (2026-10-10)

- **`data/presencial/reglas.json`**: `esperaTrasFichaMs: 15000`, `ecoDeLaFicha: null`, `soloPreguntaOCifra: true`,
  `silencio: false`, `topeDeTurnoMs: 15000`, con su fecha y su estado.
- **ADR 020 § Medición**, escrita después de las corridas: la tabla que decide, el nivel B, la elección y por qué, lo
  que la vigila y lo que el kit no mide. El estado del ADR deja de decir «salvo la regla».
- **Un gate más en el nivel A**: la regla de la casa **deja menos falsas que la línea base** (antes solo se exigía que no
  perdiera más; devolver `reglas.json` a la línea base pasaba en verde). **Rojo K3** con `scripts/demo-rojo.sh`: los tres
  valores de vuelta a la línea base nombraron «la regla de la casa deja 16 fichas falsas y la línea base 16: la sala no
  tolera tu voz»; restaurado y verificado, verde con 1 prueba, código 0. **El primer intento no contó**: con solo C3 y el
  silencio de vuelta, la regla perdía 10 y saltó antes la aserción de las perdidas; `--debe-nombrar` lo rechazó.
- `AG_SIN_HARDWARE=1 cargo test --locked` con la regla nueva: librería 604, contra el Mac 18, puerta 14, ghost 5, todo en
  verde. La canaria de la sesión completa es una pregunta («¿Y el alcance del…?»), así que la regla nueva la deja pasar.
- La fila del kit v4 en `docs/kit-de-prueba/LEEME.md` suma la regla de la casa y su gate nuevo.
