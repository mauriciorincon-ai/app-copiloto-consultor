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

## Fase 1 · El núcleo presencial y su medición (en curso)

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

**Lo que queda de la fase 1, en orden:**

- **el kit v4**: `scripts/kit-v4-sala.sh` (dos voces de `say`), `sala-{es,en}.wav`, `presencial.json`, niveles A y B
  en `contra-el-mac-de-verdad.rs` (puros, en la CI), latencia y nivel C (`hardware`);
- el efímero presencial (término plantado, `verify:ephemeral`), el contrato con su muestra de la sala, el vocabulario
  vetado (`voiceprint`, `speaker embedding`) y el gate **`cero-huellas-de-voz`**;
- la regla dura 4 y «Qué es esta app» en el `CLAUDE.md`, ahora que `Pista::Sala` existe;
- **STOP de medición.**
