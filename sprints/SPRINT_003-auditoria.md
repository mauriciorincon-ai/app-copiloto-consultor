# Auditoría del sprint 003 «El cuaderno y el cierre» — Fase 1

Esta auditoría la hizo un auditor independiente que no construyó el sprint, el **2026-09-27**, sobre el
**HEAD `3325b19`** de `sprint-003/el-cuaderno-y-el-cierre` (PR #8). Fue solo lectura: en el repo no se
editó, creó ni comiteó nada salvo este archivo.

**Método.** La evidencia principal es `git diff main...HEAD` (607 archivos), revisado archivo por archivo
en los módulos nuevos (`almacen`, `llavero`, `prefs`, `notas/`, `carpeta`, `bandeja`, `propuestas/`,
`vencimiento/`, `reunion`, `desbloqueo`, `modo`, `jurisdiccion/`, `puerta/`, `bin/ghost.rs`), en las
superficies que tocan (`lib.rs`, `contrato.rs`, `corte.rs`, `ventana/`, `sintesis/api.rs`, las pantallas y
`i18n`) y en los entregables de cierre (guía v5, manual, README, BLUEPRINT, kit, ADR 015–018 y enmiendas).
La bitácora solo se usó para saber qué creyó hacer el constructor. El plan, la orden y el `SPRINT_003.md`
se leyeron en la planeadora, sin escribir.

Lo que se corrió, y nada más (regla 22: nada que pida contraseña, abra un aviso de permiso, toque el
Llavero o launchd, ni la app en vivo):

- `cargo test --lib`: **462 ok, 1 ignorado**. `cargo test --test puerta`: **13 ok**. `cargo test --test ghost`:
  **5 ok** (se leyó antes el test: `HOME` vacío, no llega al Llavero). `cargo clippy --all-targets`: limpio.
- `pnpm test`: **41 archivos, 315 tests en verde**; cobertura global 86 % de líneas. `pnpm lint`,
  `pnpm typecheck`, `pnpm verify:ephemeral` y `node scripts/design-sync-bundle.mjs --verificar`: verdes.
- `gh pr checks 8` sobre `3325b19`: `quality`, `e2e` y `build-escritorio` en **success** propio.
- **Tres sondas medidas** en una copia del crate en el scratchpad (`scratchpad/copia/`, con su propio
  `target`): un test por sospecha, los tres en **rojo** con el código de hoy. Sus salidas se citan en A1, M1,
  M5 y M6: **son medidas, no razonadas**.
- Barridos a mano de las casillas 4, 5 y las dos 6.

No se corrió: `contra-el-mac-de-verdad` (mueve altavoces y el _tap_ del sistema), nada `--ignored`, `pnpm
tauri dev`, `ghost` contra la app viva, Playwright ni `pnpm fidelidad` (se toman de la CI). **A2 y M2 son
razonados** sobre la documentación de Apple; A2 trae una comprobación que el usuario hace mirando, sin
código y sin avisos.

## Veredicto

**«requiere ajustes».**

La ingeniería del sprint es seria: cifrado autenticado con la fecha de vencimiento dentro de lo que se
autentica, un escritor que nace en 600, la bandeja que no escribe nada con «al cerrar», un `sh` de
launchd sin interpolar rutas, una puerta que se cierra ante la duda y deja el registro sin contenido, y
casi todos los gates con su rojo. La CI está verde en los tres checks.

Pero hay **dos altos** y varios medios que tocan la promesa central del sprint, «lo tuyo queda, lo del
cliente muere»:

- **A1 (medido).** Dos reuniones con el mismo cliente el mismo día comparten nombre de archivo: la bandeja
  de la primera se pisa en silencio (de 3 propuestas quedó 1) y «Guardar» desde ella escribe en la segunda.
- **A2 (razonado).** `WhenUnlockedThisDeviceOnly` no opera en el Llavero de archivo que usa la app sin
  `kSecUseDataProtectionKeychain`: la llave de tus notas viaja con Time Machine y con el Asistente de
  migración, y seis textos afirman lo contrario («ligada a este Mac», «ni por copia de seguridad»).
- **M1 (medido).** El corte (`⌥⎋`) y «No» dejan en memoria una copia de cada propuesta, también de lo
  que dijo el cliente: `["nombre:andrea villalba"]` sobrevive al corte.
- **M9 y M10.** Dos paradas del ⭐⭐ no se pueden pasar tal como están escritas, y el ⭐⭐ es el gate del Acto 2.

Nada es crítico: nada de esto saca audio ni transcript del Mac, el API sigue apagado de fábrica y la CI
está verde. **A1, M1 y M5 deberían pagarse antes de que el usuario use la app con clientes reales; A2,
antes de que el brochure repita la promesa.**

## Cobertura de alcance

| Ítem planeado (plan · orden · `SPRINT_003.md`)                                                     | Estado                                                                                                                         | Evidencia                                                                                                                |
| -------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| F0 · Delta kit v1.31.0: reglas 10 y 15, 21 y 22 por nombre, comandos, skill `ia-embebida` §9       | Completo                                                                                                                       | `CLAUDE.md` · `.claude/commands/audita-sprint.md` · `.claude/commands/deploy-check.md` · `.claude/skills/ia-embebida.md` |
| F0 · Rojo de `scrollHeight` en `pnpm fidelidad` · WER `manual` con corrida local                   | Completo (según bitácora; no re-corrido aquí)                                                                                  | `scripts/capturar-fidelidad.mjs` · `sprints/SPRINT_003-implementation-log.md:34-47`                                      |
| F0 · Casilla 6 sobre la guía v4                                                                    | Completo, pero la v5 deja dos paradas ⭐⭐ imposibles (M9, M10)                                                                | `sprints/SPRINT_003-implementation-log.md:109-146`                                                                       |
| F0 · `almacen.rs` escritor único · `llavero.rs` compartido                                         | Con desviación: `acople.json` y la carpeta del diccionario siguen fuera (B1); el export aprieta carpetas ajenas (M5)           | `src-tauri/src/almacen.rs` · `src-tauri/src/acople/mod.rs:181-188`                                                       |
| F0 · `prefs` que sobreviven al reinicio                                                            | Completo (el archivo es `preferencias.json`; el ADR dice `prefs.json`, B3); la carpeta del corpus no se recuerda (B29)         | `src-tauri/src/prefs.rs` · `src-tauri/src/lib.rs:1755-1835`                                                              |
| F0 · B37 «lo que salió al API»                                                                     | Completo; el registro no tiene tope (B15)                                                                                      | `src-tauri/src/sintesis/api.rs:192-312` · `src/pantallas/Ia.tsx`                                                         |
| F0 · ADR 008 embeddings → H2 · `lru` releído                                                       | Completo · Parcial (`cargo audit` a mano una vez; nada en la CI, M12)                                                          | `decisions/008-el-corpus-el-disparo-y-la-ficha.md:118-140`                                                               |
| F0 · Maniobra §10 «si cabe, si no H2 declarado»                                                    | No implementada y **el corte no se declaró** (B28)                                                                             | `sprints/SPRINT_003-implementation-log.md:99`                                                                            |
| F1 · ADR 015 antes del código                                                                      | Completo, con una afirmación falsa sobre el Llavero (A2)                                                                       | `decisions/015-las-notas-y-su-cifrado.md`                                                                                |
| F1 · `notas/` protegido, sin eco, opt-in apagado · cifrado XChaCha20-Poly1305                      | Completo                                                                                                                       | `src-tauri/src/notas/mod.rs:245-255` · `src-tauri/src/notas/cifrado.rs`                                                  |
| F1 · Desbloqueo una vez por sesión · ⌃⌥N / ⌃⌥P · cuaderno protegido en sesión                      | Completo; si macOS rechaza proteger, solo lo sabe la consola (B4); la puerta hereda el desbloqueo (M4)                         | `src-tauri/src/desbloqueo.rs` · `src-tauri/src/ventana/mod.rs:66-116`                                                    |
| F1 · Notas (durante · al cerrar · archivo) · exportar · retención · borrar                         | Con defecto: exportar aprieta la carpeta de destino (M5) y no escribe las propuestas (M6); «abrir» no existe y se promete (M7) | `src/pantallas/Notas.tsx` · `src-tauri/src/carpeta.rs:268-350`                                                           |
| F1 · Efímero en marcha con notas descifradas                                                       | Completo                                                                                                                       | `src-tauri/tests/contra-el-mac-de-verdad.rs:853-980`                                                                     |
| F2 · ADR 016 · catálogo `reglas.json` · qué se guarda de cada lado                                 | Completo; la lista que enseña la pantalla no sale del catálogo (M11)                                                           | `src-tauri/src/propuestas/mod.rs:498-523` · `data/propuestas/reglas.json`                                                |
| F2 · Línea pasiva en la banda · ⌃⌥↵ · bandeja con cuenta atrás                                     | Completo, con **colisión de nombres** (A1)                                                                                     | `src-tauri/src/reunion.rs:269-331` · `src-tauri/src/bandeja.rs`                                                          |
| F2 · Vencimiento por launchd (`sh`, plist, lista 600)                                              | Completo en código y `sh` probado; **prueba en vivo con la app pendiente** (regla 22)                                          | `src-tauri/src/vencimiento/mod.rs` · `src-tauri/tests/contra-el-mac-de-verdad.rs:2323-2460`                              |
| F3 · ADR 017 · catálogo con fuente y fecha · bandera · NDA · cláusula · solo notas                 | Completo (filas contrastadas con el informe: G-1…G-12); prueba en vivo pendiente                                               | `data/jurisdicciones/catalogo.json` · `src-tauri/src/jurisdiccion/mod.rs` · `src-tauri/src/lib.rs:3270-3294`             |
| F4 · ADR 018 · socket Unix 600 · token por apertura · lista cerrada · cierre en reunión · registro | Completo; hereda el desbloqueo de la pantalla (M4); prueba en vivo pendiente                                                   | `src-tauri/src/puerta/` · `src-tauri/src/bin/ghost.rs` · `tests/unit/puerta-solo-local.test.ts`                          |
| F5 · BLUEPRINT de escritorio sin URL                                                               | Completo (barrido de enlaces limpio); repite la promesa del Llavero (A2)                                                       | `docs/BLUEPRINT.html`                                                                                                    |
| F5 · Auditoría del `CLAUDE.md`                                                                     | Completo, con dos afirmaciones que el código no sostiene: «un solo escritor» (B1) y «el corte vacía todas las piezas» (M1)     | `CLAUDE.md`                                                                                                              |
| F5 · Guía v5: 101 pruebas, ⭐⭐ 9 paradas ≤ 20 min, textos diferidos                               | Con desviación: paradas 3 y 7 no caminables (M9, M10), más B21–B25                                                             | `docs/GUIA-DE-PRUEBA.html` · `tests/unit/guia-cuadra.test.ts`                                                            |
| F5 · Kit v2 medido en la CI · `design-sync/` con bandeja, propuesta y puerta                       | Completo                                                                                                                       | `docs/kit-de-prueba/LEEME.md` · `design-sync/components/s3/la-bandeja.html`                                              |
| F5 · Manual sin «todavía no» del H1                                                                | Parcial: el manual está casi limpio; la **interfaz** no (M7, B9)                                                               | `docs/MANUAL-DE-USO.md` · `src/pantallas/Idioma.tsx:276-291`                                                             |
| F5 · `/release-check` (peso del binario) · summary · segunda casilla 4 · prueba en vivo            | Pendientes: van después de esta Fase 1, no cuentan como hallazgo                                                               | `.claude/commands/release-check.md`                                                                                      |

## CRÍTICOS (0)

Ninguno.

## ALTOS (2)

### A1 · Dos reuniones del mismo cliente el mismo día: la bandeja de la primera se pisa y «Guardar» la lleva a la segunda

**Dónde:** `src-tauri/src/reunion.rs:292-293` · `src-tauri/src/reunion.rs:248-261` · `src-tauri/src/carpeta.rs:146-150` · `src-tauri/src/carpeta.rs:187-194` · `src-tauri/src/bandeja.rs:101-110` · `src-tauri/src/carpeta.rs:210-241`

El nombre del archivo lo elige `carpeta.nombre_para(&base)`, que solo mira la carpeta de **notas**. La
bandeja se escribe con ese mismo nombre en `bandeja/`, y `escribir_sellado` escribe **encima** si ya existe.

Escenario: con «Este cliente» = Páramo Azul, una llamada corta sin escribir nada deja 2 propuestas en la
bandeja (`bandeja/paramo-azul-2026-09-27.ghost`, sin archivo de notas). Una segunda llamada ese mismo día
con una nota recibe el **mismo nombre** (`notas/` no lo tiene), escribe sus notas con él y **pisa la
bandeja de la primera**. Y si la segunda no dejó bandeja, «Guardar» desde la de la primera suma la
propuesta al archivo de la segunda.

**Medido** (sonda en `scratchpad/copia/src-tauri/src/reunion.rs`, al final): «r1: notas=None
a_la_bandeja=2 · r2: notas=Some("paramo-azul-2026-09-27.ghost") a_la_bandeja=1 · bandejas:
["paramo-azul-2026-09-27.ghost"] · propuestas vivas en la bandeja: **1 (se dejaron 3)**».

**Ajuste ejecutable**

1. En `src-tauri/src/carpeta.rs`, crear `pub fn nombre_libre(base: &str, carpetas: &[&Path]) -> Option<String>`
   con la numeración de `nombre_para` (`base.ghost`, `base-2.ghost`…): un nombre está libre solo si no
   existe en **ninguna** de las carpetas. `Carpeta::nombre_para` pasa a llamarla con `&[&self.raiz]`.
2. En `src-tauri/src/reunion.rs:293`, usar `carpeta::nombre_libre(&base, &[carpeta.raiz(), bandeja.raiz()])`.
   Hacer lo mismo en `previsto` (`reunion.rs:258`): que reciba también `&Bandeja`, y que `vista`
   (`reunion.rs:380`) le pase `&la_bandeja_de(app)`.
3. En `src-tauri/src/bandeja.rs:101-110` (`dejar`), no pisar jamás: si `self.carpeta.ruta_de(&c.reunion)?.exists()`,
   devolver `Err("ya hay una bandeja con ese nombre: no se pisa".into())`.
4. Añadir a las pruebas de `reunion.rs` el test `dos_reuniones_del_mismo_cliente_el_mismo_dia_no_se_pisan`
   (la sonda del auditor): reunión 1 sin notas y 2 propuestas; reunión 2, mismo cliente y día, con nota y
   1 propuesta. Afirma: 2 bandejas, 3 propuestas vivas, el `reunion` de la bandeja 1 distinto del archivo
   de notas de la reunión 2, y que «Guardar» desde la bandeja 1 crea **su** archivo de notas.

**Verificado cuando:** el test es rojo con el código de hoy (1 de 3) y verde después; `cargo test --lib` verde.

**Estado:** **pagado** (Fase 2) — `carpeta::nombre_libre` busca el nombre libre en notas y bandeja; `Bandeja::dejar` jamás pisa. Tests `dos_reuniones_del_mismo_cliente_el_mismo_dia_no_se_pisan` (rojo: lo previsto pisaba la bandeja; al guardar quedaba 1 bandeja) y `dejar_no_pisa_una_bandeja_que_ya_existe` (rojo sin la comprobación).

### A2 · `WhenUnlockedThisDeviceOnly` no opera en el Llavero que usa la app, y seis textos prometen que la llave no sale del Mac

**Dónde:** `src-tauri/nativo/Llavero.swift:1-7` · `src-tauri/nativo/Llavero.swift:28-31` · `src-tauri/src/llavero.rs:12-15` · `decisions/015-las-notas-y-su-cifrado.md:79-89` · `decisions/015-las-notas-y-su-cifrado.md:165-167` · `docs/MANUAL-DE-USO.md:203-208` · `docs/BLUEPRINT.html:446` · `src/i18n/es.ts:655` · `src/i18n/en.ts:549`

`SecItemAdd` se llama sin `kSecUseDataProtectionKeychain`. En macOS eso guarda el ítem en el **Llavero de
archivo** (`login.keychain-db`), y según la documentación de Apple (TN3137 «On Mac keychain APIs and
implementations» y la de `kSecAttrAccessible`) ese llavero **ignora** `kSecAttrAccessible`. Además no se
puede activar sin más: el llavero de protección de datos exige una app firmada con su grupo de llaveros, y
la app aún no se firma (G-Release, H2). La puerta local también depende de este comportamiento: el diálogo
que pide permiso a `ghost` es la lista de acceso del llavero de archivo (ADR 018 §2).

Escenario: el consultor migra a un Mac nuevo con el Asistente de migración, o restaura de Time Machine.
El `login.keychain-db` viaja con la llave, y las notas «que no se pueden abrir si cambias de Mac» se abren.
Lo que es falso es la afirmación, no la seguridad: el llavero de archivo sigue protegido por la contraseña
de la sesión. Pero ADR 015 §4 justifica el diseño con esa propiedad («`ThisDeviceOnly` es lo que hace
imposible abrir la copia»), y el producto se vende como «no persistir, verificable».

**Razonado, no medido** (regla 22: no se tocó el Llavero). **Comprobación sin código y sin avisos, la hace
el usuario:** abrir «Acceso a Llaveros», buscar «Angel Ghost · notas» y mirar la columna «Llavero». Si dice
**«inicio de sesión»** (llavero de archivo), el hallazgo queda confirmado. Si dice «Ítems locales»,
se descarta con esa evidencia.

**Ajuste ejecutable**

1. Registrar en la bitácora el resultado de la comprobación del usuario.
2. Reescribir las afirmaciones para que digan lo que hace el llavero de archivo. Frase base: «en el
   llavero de inicio de sesión de este Mac: se abre con tu sesión, no se sincroniza con iCloud, y **viaja
   con tus copias de Time Machine y con el Asistente de migración**, protegido por tu contraseña».
   - `src-tauri/nativo/Llavero.swift:1-7` y `src-tauri/src/llavero.rs:12-15`: la cabecera.
   - `decisions/015-las-notas-y-su-cifrado.md`: una **enmienda 2** al final que corrige el §4 y la
     alternativa de la línea 167, sin reescribir la historia.
   - `docs/MANUAL-DE-USO.md:203-208`: «ligada a este Mac» y «si cambias de Mac… no se pueden abrir». Pasa a:
     «si borras el Llavero, no se pueden abrir; si migras tu Mac, tu llavero y tus notas viajan juntos».
   - `docs/BLUEPRINT.html:446`.
   - `src/i18n/es.ts:655` y `src/i18n/en.ts:549`, más su maqueta `docs/diseno/notas.html`, para que el
     gate i18n↔maqueta siga verde. Es una mirada de TEXTO: queda «maquetado, no visto».
3. Dejar `kSecAttrAccessibleWhenUnlockedThisDeviceOnly` en el código, con un comentario: vale el día que
   se use el llavero de protección de datos.
4. Declarar la deuda en el summary: `kSecUseDataProtectionKeychain: true` con la firma (G-Release), y
   rediseñar entonces el diálogo de la puerta (ADR 018 §2).
5. Gate nuevo `tests/unit/llavero-sin-promesas.test.ts`. Si `src-tauri/nativo/Llavero.swift` no contiene
   `kSecUseDataProtectionKeychain`, los siguientes archivos no pueden contener
   `/ligada a este Mac|tied to this Mac|ni por copia de seguridad|no viajan en copias de seguridad|no salen de\s+este Mac/`:
   `docs/MANUAL-DE-USO.md`, `docs/BLUEPRINT.html`, `src/i18n/es.ts`, `src/i18n/en.ts`, las dos cabeceras de
   código y el cuerpo del ADR 015 anterior a la enmienda. Rojo hoy.

**Verificado cuando:** el gate pasa de rojo a verde, y la comprobación del usuario queda en la bitácora.

**Estado:** pendiente (Fase 2)

## MEDIOS (13)

### M1 · El corte (`⌥⎋`) y «No» dejan en memoria una copia de cada propuesta, también de lo que dijo el cliente

**Dónde:** `src-tauri/src/notas/mod.rs:155-157` · `src-tauri/src/notas/mod.rs:278-282` · `src-tauri/src/notas/mod.rs:311-327` · `src-tauri/src/notas/mod.rs:358-364` · `src-tauri/src/notas/mod.rs:411-436`

`propuestas_vistas` guarda `"regla:texto plegado"` de todo lo que se propuso, para no repetirlo. Ni
`cortar_las_propuestas` ni `descartar_propuesta` lo tocan, `olvidar` lo suelta sin pisar y el `Drop` no lo
pisa. La regla dura 1(e) dice que el kill-switch «vacía buffers», y el ADR 016 §3 dice que «No la descarta:
muere en ese momento».

Escenario: el cliente nombra a «Andrea Villalba», el consultor pulsa `⌥⎋`, y el nombre sigue en RAM
mientras la reunión siga abierta (con tu nota, hasta que la guardes).

**Medido** (sonda en `scratchpad/copia/src-tauri/src/notas/mod.rs`): «en_espera tras el corte: 0 ·
propuestas_vistas tras el corte: `["nombre:andrea villalba"]`».

**Ajuste ejecutable**

1. `notas/mod.rs:155-157`: cambiar el campo a `propuestas_vistas: std::collections::HashSet<u64>`, más un
   campo `vistas_con: std::hash::RandomState`, que da llaves al azar por cuaderno. `proponer`
   (`:314-322`) guarda `self.vistas_con.hash_one(format!("{}:{}", p.regla.id(), propuestas::plegar(&p.texto)))`
   en vez del texto.
2. `cortar_las_propuestas` (`:280-282`): añadir `self.propuestas_vistas.clear()`. Tras el corte, lo que se
   diga es captura nueva.
3. Test `el_corte_no_deja_copia_de_lo_que_se_propuso` (la sonda del auditor): proponer una `Nombre` del
   cliente, `cortar()`, `cortar_las_propuestas()` → `propuestas_vistas.is_empty()`. El test existente
   `la_misma_propuesta_no_se_repite_ni_despues_de_descartarla_y_hay_tope` tiene que seguir verde.

**Verificado cuando:** la sonda es roja hoy y el test nuevo es verde después; `cargo test --lib notas` verde.

**Estado:** **pagado** (Fase 2) — `propuestas_vistas` guarda una huella `u64` con `RandomState` por cuaderno, y `cortar_las_propuestas` la vacía. Test `el_corte_no_deja_copia_de_lo_que_se_propuso` (rojo: `["nombre:andrea villalba"]`).

### M2 · La bandeja entra en Time Machine: la copia que «no puede vivir un mes» vive en el disco de copias

**Dónde:** `src-tauri/src/bandeja.rs:5-8` · `src-tauri/src/carpeta.rs:22-24` · `decisions/016-las-propuestas-y-la-bandeja.md:86-89` · `docs/MANUAL-DE-USO.md:199-202` · `docs/MANUAL-DE-USO.md:242-243`

La bandeja salió de Documentos con este argumento: «una lista que promete morir a las 3 h no puede tener
una copia que viva un mes, aunque esté cifrada». Pero `~/Library/Application Support` entra en Time
Machine, y nada marca `bandeja/` como excluida. Con A2, la llave va en la misma copia. El manual dice
«ninguna copia sale de tu Mac».

Escenario: Time Machine hace su copia horaria con una bandeja viva, y ese archivo vive en el disco de
copias semanas después de haber vencido.

**Razonado** (comportamiento conocido de Time Machine; no se leyó la configuración del Mac del usuario).

**Ajuste ejecutable**

1. Nuevo `@_cdecl("ag_fuera_de_las_copias")` en un archivo `src-tauri/nativo/Copias.swift`, añadido a
   `EL_PUENTE` en `src-tauri/build.rs`. Hace
   `var v = URLResourceValues(); v.isExcludedFromBackup = true; try url.setResourceValues(v)` sobre la ruta
   recibida y devuelve 0 o -1. Si `verify:ephemeral` marca la línea, lleva `verify-ephemeral:allow — ADR 016`.
2. En `Bandeja::dejar` (`src-tauri/src/bandeja.rs:101-110`), llamarla sobre `self.carpeta.raiz()` después de
   escribir. Es idempotente. Para `notas/` decide el usuario: tus notas en tus copias son tuyas, pero entonces
   el manual tiene que decirlo.
3. Corregir la frase del manual (`:199-202`, `:242-243`), el ADR 016 §4 (una enmienda) y la cabecera de
   `bandeja.rs:5-8`: la bandeja no entra en las copias de Time Machine, y las instantáneas locales de macOS
   (≤ 24 h) sí la ven.
4. Test en `src-tauri/src/bandeja.rs` (solo con el puente): tras `dejar`, `libc::getxattr` sobre la
   carpeta encuentra `com.apple.metadata:com_apple_backup_excludeItem`.

**Verificado cuando:** el test es rojo hoy y verde después; en la prueba en vivo, `tmutil isexcluded` sobre
`bandeja/` dice `[Excluded]`, anotado en la bitácora.

**Estado:** pendiente (Fase 2)

### M3 · La regla dura 2 exige un proveedor con no-retención, y nadie registra la política de Claude, Gemini y Groq

**Dónde:** `CLAUDE.md:66` · `decisions/011-proveedores-del-modelo-y-minimizacion.md:25` · `src/ia.ts:139`

La app deja encender tres proveedores externos con la clave del usuario. Ningún ADR, ni el manual, ni la
pantalla IA dicen cuánto retiene cada uno el texto, si entrena con él, o qué hay que configurar para cumplir
«no-retención». Viene del S2 y sigue abierto: el S3 amplió lo que sale (B37) sin cerrarlo.

Escenario: el consultor enciende un proveedor con una clave estándar, y el texto minimizado queda en ese
proveedor el tiempo que digan sus términos, que nadie leyó.

**Ajuste ejecutable**

1. Una enmienda al ADR 011 con una tabla por proveedor de `EXTERNOS`: retención por defecto, uso para
   entrenar, cómo se consigue no-retención (plan de pago o acuerdo), la URL de los términos y la fecha
   de lectura.
2. Decisión del usuario, con la tabla delante: un proveedor que no cumple con una clave estándar se quita
   de `EXTERNOS`, o IA lo dice en su fila (TEXTO, «maquetado, no visto»), o la regla dura 2 cambia en la
   planeadora.
3. Gate `tests/unit/proveedores-con-su-retencion.test.ts`: cada `nombre` de `EXTERNOS` (`src/ia.ts:139`)
   tiene fila en la tabla del ADR 011, con una fecha `AAAA-MM-DD` y una URL. Rojo hoy.

**Verificado cuando:** el gate pasa de rojo a verde y la decisión del usuario queda en el summary.

**Estado:** pendiente (Fase 2)

### M4 · La puerta hereda el desbloqueo de la pantalla: tu agente abre cualquier reunión sin preguntarte

**Dónde:** `src-tauri/src/reunion.rs:869-876` · `src-tauri/src/lib.rs:2651-2658` · `src-tauri/src/desbloqueo.rs:52-79` · `src-tauri/src/puerta/cli.rs:82` · `src-tauri/src/puerta/cli.rs:101` · `decisions/018-la-puerta-local.md:95` · `docs/MANUAL-DE-USO.md:494`

`ghost notas abrir` pasa por `reunion::abrir`, que usa el mismo `Desbloqueo` de la sesión de la app que
«Exportar» en Notas. Si esta mañana exportaste una reunión, tu agente abre **todas** tus reuniones esta
tarde sin un solo Touch ID. La ayuda de `ghost` dice «pide Touch ID en el Mac», a secas.

Escenario: el usuario exporta una reunión, abre la puerta para que Claude Code busque en el corpus, y el
agente lee sus notas cifradas en silencio. Lo único que queda es una línea en el registro.

**Ajuste ejecutable** (decisión del usuario: se recomienda **una vez por apertura de la puerta**)

1. `src-tauri/src/desbloqueo.rs`: añadir `pub fn olvidar(&self)`, que pone `hecho` a `false`.
2. Un `Desbloqueo` propio de la puerta (campo nuevo en `LaPuerta`, `src-tauri/src/lib.rs`) que se olvida al
   abrirla y al cerrarla (`abrir_la_puerta` y `cerrar_la_puerta`).
3. `reunion::abrir` pasa a `abrir_con(app, &Desbloqueo, archivo, idioma)`. La rama `Orden::AbrirNota`
   (`lib.rs:2651-2658`) le pasa el de la puerta.
4. Textos: `decisions/018-la-puerta-local.md:95`, `docs/MANUAL-DE-USO.md:494`,
   `src-tauri/src/puerta/cli.rs:82` y `:101`, y la prueba n5 de la guía: «una vez cada vez que abres la puerta».
5. Test en `desbloqueo.rs`: con dos `Desbloqueo`, `asegurar(Hecho)` en el de la pantalla no evita que el de
   la puerta llame a `pedir`; y tras `olvidar()`, se vuelve a pedir.

**Verificado cuando:** el test es verde y la prueba en vivo (fila de la regla 22) muestra Touch ID al primer
`ghost notas abrir` de cada apertura.

**Estado:** pendiente (Fase 2)

### M5 · «Exportar a texto» aprieta a 700 la carpeta que elijas, y falla en una carpeta que no es tuya

**Dónde:** `src-tauri/src/carpeta.rs:270-277` · `src-tauri/src/almacen.rs:27-32` · `src-tauri/src/almacen.rs:87-91` · `src-tauri/src/reunion.rs:879-891`

`exportar` escribe con `almacen::escribir`, y ese llama a `carpeta_privada(padre)`, que pone en 700 la
carpeta de destino. Pero esa carpeta es del usuario: el propio almacén trae `escribir_en_carpeta_ajena`
para este caso, y no se usa.

Escenario: exportas a la carpeta compartida de tu equipo (755) y deja de ser legible para las otras
cuentas. Si exportas a `/Users/Shared`, que es de root, el `chmod` falla y la exportación entera también.

**Medido** (sonda en `scratchpad/copia/src-tauri/src/carpeta.rs`): una carpeta en 755 queda en **700** tras exportar.

**Ajuste ejecutable**

1. `carpeta.rs:273`: `crate::almacen::escribir_en_carpeta_ajena(destino, texto.as_bytes())`.
2. `almacen.rs:34-36`: la documentación nombra ese segundo uso (el destino que eliges al exportar).
3. Test `exportar_no_toca_la_carpeta_de_destino` en `carpeta.rs` (la sonda): destino en 755 → sigue en 755,
   y el archivo exportado nace en 600.

**Verificado cuando:** la sonda es roja hoy (700) y el test verde después.

**Estado:** **pagado** (Fase 2) — `exportar` escribe con `almacen::escribir_en_carpeta_ajena`. Test `exportar_no_toca_la_carpeta_de_destino` (rojo: 755 → 700).

### M6 · Exportar deja fuera las propuestas que guardaste

**Dónde:** `src-tauri/src/carpeta.rs:318-350` · `src/propuesta.ts:28-48` · `decisions/016-las-propuestas-y-la-bandeja.md:69-71`

`a_texto` escribe la nota, los acuerdos, las fijadas y tus turnos. Las «Propuestas guardadas», que el ADR
016 §3 mete en el archivo, **no salen** al exportar.

Escenario: guardas «Fecha real: 12 semanas desde la firma» con ⌃⌥↵ y exportas para tu CRM. No está.

**Medido** (misma sonda): «el .md exportado lleva la propuesta guardada: **false**».

**Ajuste ejecutable**

1. En `a_texto`, tras los acuerdos, una sección «Propuestas que guardaste» / «Suggestions you saved»,
   redactada en los dos idiomas. Una línea por propuesta, `- {hora} · …`, con las plantillas de
   `src/propuesta.ts:28-42`:
   - tuya: la frase;
   - del cliente: «Dijeron «{texto}»» / «They said “{texto}”»;
   - choque: el mismo texto más «; tu ficha fijada dice «{ficha}»» y la sección;
   - nombre: «Mencionaron a «{texto}», que no está en tu corpus»;
   - pregunta: «Te preguntaron: {texto}».

   El cliente sale siempre como hecho, jamás como turno.

2. Ampliar `exportar_quita_el_cifrado_y_nace_600` (`carpeta.rs:514-533`) con una propuesta guardada (tuya y
   del cliente) y afirmar sus líneas en `es` y en `en`.
3. Manual, punto 6 de Notas (`docs/MANUAL-DE-USO.md:188-190`): exportar incluye las propuestas guardadas.

**Verificado cuando:** el test ampliado es rojo hoy y verde después.

**Estado:** **pagado** (Fase 2) — `a_texto` escribe «Propuestas que guardaste» / «Suggestions you saved» con las plantillas de la pantalla, en los dos idiomas; manual, punto 6. Test ampliado `exportar_quita_el_cifrado_y_nace_600` (rojo: sin la sección).

### M7 · Tres pantallas dicen «Todavía no» de lo que el S3 construyó

**Dónde:** `src/pantallas/Idioma.tsx:276-291` · `src/i18n/es.ts:428-432` · `src/i18n/en.ts:333-337` · `src/pantallas/Permisos.tsx:125-143` · `src/componentes/Banda.tsx:135-139` · `src/componentes/Banda.tsx:826-839` · `docs/diseno/idioma.html:245-247`

Casilla 4, las tres frases vivas del producto:

- **Idioma**, «Lo que todavía no existe», dice **«Conservar lo que dijiste tú — Todavía no»**, y el detalle
  «los turnos mueren los dos —el tuyo y el del cliente— al cerrar». Hoy «Conservar mis turnos» existe
  (Notas, «al cerrar», y `prefs`) y guarda tus turnos.
- **Permisos**, «Qué puedes hacer ya, sin conceder nada», dice **«Escribir notas y acuerdos — Todavía no»**.
  Hoy «Solo notas» abre una reunión sin pedir ningún permiso de captura. El comentario dice «las notas llegan
  en el sprint 3».
- **La banda**, estado «sin verificar», lleva «Solo notas» apagado con `title="Todavía no"`. El comentario
  (`:830-831`) dice que el modo no existe, y el de `:137-138` dice que ese estado lo decide la protección de
  la ventana, pero **solo se llega por la URL**.

Escenario: el usuario lee en Idioma que no puede conservar sus turnos, mientras la casilla está encendida
en Notas.

**Ajuste ejecutable**

1. Idioma: quitar la fila `conservarTusTurnos`. Reescribir `loQueFaltaDetalle` en es y en, redactado: «Hoy
   cada pista escucha un idioma. Tus turnos solo quedan, en texto, si enciendes «Conservar mis turnos» en
   Notas; los del cliente mueren siempre». El título pasa a «Lo que llega en el H2» (el manual ya lo dice
   en `:590`). Lo mismo en la maqueta `docs/diseno/idioma.html:245-247` y `:314-316`.
2. Permisos: «Escribir notas y acuerdos» pasa a `<Funciona />`, con el comentario al día, en el código y en
   `docs/diseno/permisos.html`.
3. Banda: el botón «Solo notas» lleva `title` «Se elige en Sesión, antes de empezar» (clave nueva es/en; la
   banda no tiene el comando). Los comentarios de `:137-138` y `:830-831` dicen la verdad: estado solo de
   maqueta.
4. Las tres son miradas de TEXTO: «maquetado, no visto», al bloque de textos del ⭐⭐.
5. Gate `tests/unit/sin-todavia-no-de-lo-que-existe.test.ts`, que lee las fuentes: `Idioma.tsx` no usa
   `conservarTusTurnos`, y en `Permisos.tsx` la fila de `escribirNotas` no lleva `<TodaviaNo`. Rojo hoy.

**Verificado cuando:** el gate pasa de rojo a verde; el gate i18n↔maqueta y `pnpm fidelidad` siguen verdes.

**Estado:** pendiente (Fase 2)

### M8 · «Abrir una reunión guardada» no existe en la app, y lo prometen el manual, la interfaz, la guía y dos ADR

**Dónde:** `docs/MANUAL-DE-USO.md:204` · `docs/MANUAL-DE-USO.md:240` · `docs/MANUAL-DE-USO.md:494` · `src/i18n/es.ts:747` · `docs/GUIA-DE-PRUEBA.html:254` · `docs/GUIA-DE-PRUEBA.html:649` · `docs/GUIA-DE-PRUEBA.html:654` · `docs/GUIA-DE-PRUEBA.html:682` · `decisions/015-las-notas-y-su-cifrado.md:94` · `decisions/016-las-propuestas-y-la-bandeja.md:97`

En Notas, el archivo solo ofrece «Exportar a texto» y «Borrar ahora» (`src/pantallas/Notas.tsx:1086-1095`).
El contenido descifrado no cruza al webview, y el propio gate lo dice: «la maqueta no tiene abrir»
(`tests/unit/contrato-con-lectores.test.ts:89-93`). Aun así, estos sitios hablan de «abrir una reunión»:

- el manual: «abrir o exportar una reunión guardada te pide Touch ID», «como abrir una reunión» y «como abrir
  en _Notas_»;
- la bandeja con llave: «Para leerla, Touch ID, como para abrir una reunión»;
- la guía: «ábrela», «si no la abriste ya»;
- los ADR 015 §5 y 016 §4.

Escenario: el usuario busca un botón «Abrir» que no existe. La parada 7 del ⭐⭐ se lo pide (M10).

**Ajuste ejecutable**

1. Reescribir, sin cambiar la pantalla:
   - manual `:204`: «exportar una reunión guardada te pide…»; `:240`: «como exportar una reunión»; `:494`:
     «(una vez por sesión de la app, como al exportar en _Notas_)», que con M4 pasa a «por apertura»;
   - `src/i18n/es.ts:747` y `en.ts` `paraLeerla`, más la maqueta `docs/diseno/notas.html` (TEXTO):
     «…como para exportar una reunión…»;
   - guía `:254` «al exportar una reunión o al abrirla con `ghost`», `:654` y `:682`.
2. ADR 015: una enmienda al §5 («abrir» solo por la puerta, ADR 018; en pantalla, exportar). ADR 016 §4 `:97`, igual.
3. Gate `tests/unit/abrir-no-existe.test.ts`: `docs/MANUAL-DE-USO.md`, `src/i18n/es.ts`, `src/i18n/en.ts` y
   `docs/GUIA-DE-PRUEBA.html` no contienen `/abrir (una|la) reuni[oó]n|ábrela|como abrir en/i` salvo en
   una línea que nombre `ghost`. Rojo hoy.

**Verificado cuando:** el gate pasa de rojo a verde.

**Estado:** pendiente (Fase 2)

### M9 · Parada 3 del ⭐⭐: la misma pregunta dos veces seguidas no trae ficha, y la parada la pide

**Dónde:** `docs/GUIA-DE-PRUEBA.html:509` · `docs/GUIA-DE-PRUEBA.html:408` · `src-tauri/src/disparo/mod.rs:165-167` · `src-tauri/src/disparo/mod.rs:387-393`

La parada 2 (e3) pregunta «¿Y la limpieza de datos, eso está dentro del alcance?» y deja la sesión en
marcha. La 3 (h2) pide «haz que el cliente vuelva a preguntar» **la misma frase**, y espera «Primero llega la
ficha; en un segundo… la sugerencia». Pero el disparador no repite una consulta idéntica: lo prueba su propio
test, `la_misma_pregunta_repetida_no_vuelve_a_disparar`. No llega ficha nueva ni sugerencia. Y como la
preparación ya encendió «Redactar sugerencias», la sugerencia salió en la parada 2 sin que nadie la juzgara.

Escenario: el usuario, en el Acto 2, marca la parada 3 como fallida, o cree que la vio porque la ficha de
antes sigue en pantalla.

**Ajuste ejecutable**

1. `docs/GUIA-DE-PRUEBA.html:509` (h2): usar otra pregunta del kit con ficha que el recorrido no haya hecho,
   `say -v Paulina "¿Cuántas rondas de revisión incluye?"`. En h3 (`:514`), cambiar esa pregunta por otra
   de `docs/kit-de-prueba/preguntas.json` que tenga sección.
2. Añadir a h2 una línea: «La misma pregunta dos veces seguidas no trae ficha: la app no repite».
3. `guia-cuadra` sigue verde: los minutos no cambian.

**Verificado cuando:** en la prueba en vivo, las paradas 2 → 3 dan dos fichas distintas y una sugerencia
juzgable, anotado en la bitácora.

**Estado:** pendiente (Fase 2)

### M10 · Parada 7 del ⭐⭐: pide abrir la reunión (no existe) y ver «la propuesta que guardaste» (nunca se guardó)

**Dónde:** `docs/GUIA-DE-PRUEBA.html:649` · `docs/GUIA-DE-PRUEBA.html:584` · `docs/GUIA-DE-PRUEBA.html:612`

m1 dice «Vuelve a Notas → el archivo → la reunión y **ábrela**… Dentro está tu nota, tu acuerdo, la ficha
fijada y **la propuesta que guardaste**». Tres cosas lo impiden:

- en Notas no hay «abrir» (M8);
- ninguna parada del ⭐⭐ guarda una propuesta (la 5 fija y escribe, pero no pulsa ⌃⌥↵), y el corte de la
  parada 6 se lleva las que esperaban;
- aunque hubiera una, exportar no la escribe (M6).

Escenario: el usuario no puede cumplir la parada que certifica «lo tuyo queda».

**Ajuste ejecutable** (después de M5 y M6)

1. m1 (`:649`): «Notas → el archivo → la reunión → «Exportar a texto» → «Exportar sin cifrado» → Touch ID →
   guárdalo en una carpeta de prueba. Abre el `.md`: tu nota, tu acuerdo, la ficha fijada y la propuesta
   que guardaste; del cliente, nada. Bórralo».
2. k1 (`:584`, parada 5): añadir «Di tú: «Te lo mando el viernes con el detalle» y guárdala con ⌃⌥↵».
   Ajustar `data-min` sin pasar de 20 en total (la suma la vigila `guia-cuadra`).
3. Ninguna parada remite a «abrir».

**Verificado cuando:** `pnpm vitest run tests/unit/guia-cuadra.test.ts` verde, y la parada 7 se camina en
la prueba en vivo.

**Estado:** pendiente (Fase 2)

### M11 · «Qué sabe reconocer» no sale del catálogo: la lista es una frase fija y el ADR afirma un test que no existe

**Dónde:** `decisions/016-las-propuestas-y-la-bandeja.md:30-32` · `src/i18n/es.ts:697` · `src/i18n/en.ts:591` · `src-tauri/src/propuestas/catalogo.rs:106-109` · `data/propuestas/reglas.json`

El ADR dice que lo que enseña la pantalla «sale del catálogo, así que la lista que lees es la lista entera
(test)». Pero la pantalla pinta `sonReglas`, una frase de i18n escrita a mano. `catalogo::reglas()`, que
dice existir «para la pantalla», **no tiene llamador**, y los `nombre` del catálogo no se enseñan en ningún
sitio. El test del catálogo solo compara sus ids con el enum (casilla 6 a).

Escenario: alguien cambia el nombre de una regla en `reglas.json` y la pantalla sigue diciendo lo de antes,
con el ADR afirmando lo contrario.

**Ajuste ejecutable**

1. `data/propuestas/reglas.json`: a cada regla, `corto: {es, en}`, el trozo exacto de `sonReglas` («cifras,
   plazos y fechas»…). Añadir el campo a `FilaDeRegla` (`catalogo.rs:19-24`).
2. Gate `tests/unit/reglas-publicadas.test.ts`: por idioma, `sonReglas` es exactamente el prefijo fijo más
   `corto` unidos con « · » en el orden del catálogo, más «.». Rojo si una regla se añade, se quita o se
   renombra sin la frase.
3. Borrar `catalogo::reglas()`, o darle un llamador en un test de Rust.
4. Enmendar ADR 016 §1 (`:30-32`) para que nombre este test.

**Verificado cuando:** el gate es verde, y rojo al quitar una regla de `reglas.json` (demo en la bitácora).

**Estado:** pendiente (Fase 2)

### M12 · Las dependencias de Rust no tienen vigilancia continua: ni `cargo audit` en la CI ni dependabot de Cargo

**Dónde:** `.github/workflows/ci.yml:70-77` · `.github/dependabot.yml:12-29` · `src-tauri/Cargo.toml:48`

La DoD de Seguridad pide `cargo audit`, y el sprint añadió un crate de cifrado (`chacha20poly1305` 0.11,
sobre `aead` 0.6). `cargo audit` se corrió **a mano una vez** en la fase 0. Ningún job lo corre, y dependabot
solo vigila npm y las acciones.

Escenario: sale un aviso de RustSec para una dependencia del cifrado o de tantivy, y nadie se entera hasta
el próximo `cargo audit` a mano.

**Ajuste ejecutable**

1. `.github/workflows/ci.yml`, job `build-escritorio`, tras clippy: `cargo install cargo-audit --locked` (o
   `taiki-e/install-action` con `cargo-audit`) y `cargo audit`. Falla con vulnerabilidades; los avisos de
   «sin mantener» no bloquean, como hoy.
2. Rojo en un PR desechable que fije una versión con aviso de RustSec; se registra en la bitácora (regla 15).
3. Dependabot de Cargo: la regla 18 limita a dos PR abiertos. Decide el usuario: añadir `cargo` y ajustar
   el techo, o dejar escrito en `docs/BLUEPRINT.html` §CI que los bumps de Rust son a mano y los guarda
   `cargo audit`.

**Verificado cuando:** el paso nuevo corre en `build-escritorio` con conclusión propia, se vio en rojo en el
PR desechable, y `gh pr checks` sale en success.

**Estado:** pendiente (Fase 2)

### M13 · Notas.tsx y `propuesta.ts`, la pantalla central del sprint y el vestido de lo que dijo el cliente, casi sin tests

**Dónde:** `src/pantallas/Notas.tsx` · `src/propuesta.ts:28-64` · `vitest.config.ts:26-40`

Cobertura medida con `pnpm test`: `Notas.tsx` **56,9 % de líneas y 44,6 % de ramas**, por debajo del 50 %
que la regla 2 pide a la UI. `propuesta.ts`, **5,6 %**. Ese archivo arma, en la interfaz, la plantilla que
convierte el fragmento del cliente en «un hecho en una línea»: la mitad TS de la regla dura 1 en las
propuestas no tiene un solo test. El umbral global (86 %) lo esconde.

**Ajuste ejecutable**

1. `tests/unit/propuesta.test.ts`: `textoDe`, `origenDe`, `etiquetaDe` e `iconoDe` para las 5 reglas ×
   tuyo/cliente, en es y en. Afirma también que, del cliente, el texto solo añade la plantilla al fragmento.
2. `tests/unit/notas.test.tsx`: la bandeja abierta, cerrada con llave y vencida; la pregunta de exportar
   (sí y no); «Borrar ahora» con su pregunta; cambiar la retención.
3. `vitest.config.ts`: umbrales por archivo, `src/propuesta.ts` 80 y `src/pantallas/Notas.tsx` 60 de líneas y
   50 de ramas. Con demo en rojo, bajando uno a propósito.

**Verificado cuando:** el informe de cobertura da esas cifras y los umbrales nuevos pasan.

**Estado:** pendiente (Fase 2)

## BAJOS (30)

| #       | Hallazgo                                                                                                                                                                                                                                                                                        | Dónde                                                                                                                                                                    | Ajuste ejecutable                                                                                                                                                                                                                                                                                                                                                                  | Estado             |
| ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------ |
| **B1**  | El «escritor único» no es único. `acople.json` nace con `fs::write` (umask) y se aprieta después; la carpeta del diccionario se crea sin 700. Y el `CLAUDE.md` afirma «todo lo que persiste nace en 600 dentro de una carpeta 700 por `almacen.rs`».                                            | `src-tauri/src/acople/mod.rs:181-188` · `src-tauri/src/lib.rs:98-104` · `CLAUDE.md` (§ Patrones, «Un solo escritor»)                                                     | `acople::guardar` escribe con `crate::almacen::escribir(ruta, &bytes)`, convirtiendo el error en `io::Error::other`, y deja `restringir` solo para reparar. `asegurar_el_diccionario` cambia `create_dir_all(padre)` por `almacen::carpeta_privada(padre)`. Test en `acople`: tras `guardar`, `almacen::cerrar_permisos(ruta, 0o600)` devuelve `false`, y es rojo con `fs::write`. | pendiente (Fase 2) |
| **B2**  | La descripción de la capability principal afirma que «esta app no escribe archivos por diálogo». Exportar lo hace con el diálogo de guardar, abierto desde Rust.                                                                                                                                | `src-tauri/capabilities/default.json:4`                                                                                                                                  | Reescribir: «`dialog:allow-save` no está: la ventana no guarda por diálogo; «Exportar a texto» abre el de guardar desde Rust (`reunion::exportar`), sin esta capability». `tests/unit/capabilities.test.ts` verde.                                                                                                                                                                 | pendiente (Fase 2) |
| **B3**  | Las enmiendas del ADR 002 nombran `prefs.json`, y el archivo es `preferencias.json`.                                                                                                                                                                                                            | `decisions/002-persistencia.md:186` · `decisions/002-persistencia.md:192` · `decisions/015-las-notas-y-su-cifrado.md:227`                                                | Cambiar a `preferencias.json` en las tres. Verificado: `grep -rn "prefs\.json" decisions docs` vacío.                                                                                                                                                                                                                                                                              | pendiente (Fase 2) |
| **B4**  | Si macOS rechaza proteger el cuaderno, solo lo dice la consola: la pantalla sigue prometiendo «tu nota no la ve nadie».                                                                                                                                                                         | `src-tauri/src/ventana/mod.rs:104-116` · `src-tauri/src/reunion.rs:420`                                                                                                  | `proteger_el_cuaderno` devuelve `Result`. `reunion` guarda `sin_proteger: AtomicBool` y lo expone como `VistaDelCuaderno.sinProteger` (contrato + muestra). Notas pinta una franja `warn` «Tu cuaderno no se pudo proteger: no compartas la pantalla entera» (TEXTO). Test con la muestra `sinProteger: true`.                                                                     | pendiente (Fase 2) |
| **B5**  | `NSSpeechRecognitionUsageDescription` está declarada (desde el S1), pero ningún código pide ese permiso (`requestAuthorization`), y ni Permisos ni la lista de avisos de la guía lo nombran.                                                                                                    | `tests/unit/lo-que-macos-dira.test.ts:28` · `src/pantallas/Permisos.tsx`                                                                                                 | En la prueba en vivo (su fila de la regla 22), mirar si macOS muestra el aviso de Reconocimiento de voz. Si no aparece, quitar la clave de `src-tauri/Info.plist`, de los dos `InfoPlist.strings` y de la lista del test. Si aparece, añadir su fila a Permisos (TEXTO) y a la lista de avisos de la guía.                                                                         | pendiente (Fase 2) |
| **B6**  | `zod` está en `dependencies` y nada lo importa. El `CLAUDE.md` ya no lo nombra en el stack.                                                                                                                                                                                                     | `package.json:27`                                                                                                                                                        | `pnpm remove zod`. Leer la salida del install (regla 18) y verificar `pnpm install --frozen-lockfile` y `pnpm peers check` en verde.                                                                                                                                                                                                                                               | pendiente (Fase 2) |
| **B7**  | `verify:ephemeral` vigila una carpeta que no existe (`src/capture`), y su `ALLOW` no exige el ADR que su comentario promete: basta `verify-ephemeral:allow` para saltarse el gate.                                                                                                              | `scripts/verify-ephemeral.mjs:101` · `scripts/verify-ephemeral.mjs:126`                                                                                                  | Quitar `"src/capture"`, y fallar (exit 1) si una entrada de `PROTEGIDOS` no existe. `ALLOW = /verify-ephemeral:allow\b.*\bADR\s*\d{3}/`. Demo en rojo de las dos (entrada falsa; línea marcada sin ADR) en la bitácora.                                                                                                                                                            | pendiente (Fase 2) |
| **B8**  | El aviso del hook de gitleaks manda a «.env.local / Vercel env vars»: esta app no tiene ni lo uno ni lo otro.                                                                                                                                                                                   | `.claude/settings.json:12`                                                                                                                                               | El mensaje: «SECRET DETECTADO. Escritura bloqueada: los secretos van al Llavero de macOS, nunca a un archivo del repo».                                                                                                                                                                                                                                                            | pendiente (Fase 2) |
| **B9**  | Lo que es H2 dice «Todavía no»: la fila MLX de IA y las tres filas de Corpus (arrastrar, releer lo que cambió, leer escaneado). El manual ya lo llama H2.                                                                                                                                       | `src/pantallas/Ia.tsx:163-166` · `src/pantallas/Corpus.tsx:158-165` · `docs/MANUAL-DE-USO.md:459-460`                                                                    | Un estado «En el H2» (es/en) en lugar de `<TodaviaNo />`, y el título de Corpus «Lo que llega en el H2». Lo mismo en las maquetas `docs/diseno/ia.html:235`, `:292` y `docs/diseno/corpus.html:232-235`, `:273-276` (TEXTO). El manual `:460` cita el nuevo texto.                                                                                                                 | pendiente (Fase 2) |
| **B10** | Honestidad dice «hoy son diez» piezas (son 11). Y `piezasCola` dice «la otra todavía no existe», en singular, en una rama que hoy no se alcanza.                                                                                                                                                | `src/pantallas/Honestidad.tsx:37` · `src/i18n/es.ts:382` · `src/i18n/en.ts:296`                                                                                          | El comentario sin cifra («la cuenta sale de `corte::TODAS`»). `piezasCola` en plural: «piezas: las demás todavía no existen» / «pieces: the rest do not exist yet».                                                                                                                                                                                                                | pendiente (Fase 2) |
| **B11** | «Llegaste a 30 propuestas» fija en el texto el tope de Rust (casilla 6 a).                                                                                                                                                                                                                      | `src/i18n/es.ts:699` · `src/i18n/en.ts:593` · `src-tauri/src/notas/mod.rs:36`                                                                                            | Test `tests/unit/topes-en-el-texto.test.ts`: lee `TOPE_DE_PROPUESTAS` de `notas/mod.rs` y exige ese número en `lleno` es/en. Rojo al cambiar uno de los dos.                                                                                                                                                                                                                       | pendiente (Fase 2) |
| **B12** | La ayuda de `ghost` y la vista de la puerta dicen «La primera vez, macOS te pregunta»; el ADR 018 §2 y el manual dicen, bien, que pregunta **en cada apertura**.                                                                                                                                | `src-tauri/src/puerta/cli.rs:86` · `src-tauri/src/puerta/cli.rs:105` · `src/i18n/es.ts:839` · `src/i18n/en.ts:728`                                                       | «Cada vez que abres la puerta, macOS te pregunta…» en los cuatro y en la maqueta `docs/diseno/ia.html` (TEXTO). Test en `cli.rs`: la ayuda no contiene «La primera vez» ni «The first time».                                                                                                                                                                                       | pendiente (Fase 2) |
| **B13** | Casilla 5: `Guardada` (`archivo`, `bytes`, `vence`), lo que devuelve `guardar_la_reunion`, cruza y nadie lo lee. El gate lo da por leído porque otros tipos tienen campos con esos nombres.                                                                                                     | `src/notas.ts:334-336` · `src/pantallas/Notas.tsx:429-435` · `src-tauri/src/lib.rs:2365`                                                                                 | `guardar_la_reunion` devuelve `Result<(), String>`. Quitar el tipo `Guardada` de `src/notas.ts` y la muestra `REUNION_GUARDADA_AHORA` de `contrato.rs`; el struct de Rust se queda para el log. El gate de contrato, verde.                                                                                                                                                        | pendiente (Fase 2) |
| **B14** | Casilla 5: el evento `modo` manda `Modo` (o `null`), y su único oyente ignora el dato y vuelve a preguntar.                                                                                                                                                                                     | `src-tauri/src/lib.rs:570` · `src-tauri/src/lib.rs:652` · `src-tauri/src/lib.rs:671` · `src/cuaderno.ts:375`                                                             | Emitir `()` en los tres sitios (es una señal). Quitar `Serialize` de `Modo` si deja de usarse.                                                                                                                                                                                                                                                                                     | pendiente (Fase 2) |
| **B15** | El registro de lo que salió al API no tiene tope: con el API encendido, guarda cada turno anonimizado del cliente, y los datos tapados en claro, durante toda la reunión, mucho más allá del anillo de 30 s. Honestidad no lo cuenta.                                                           | `src-tauri/src/sintesis/api.rs:283-296` · `src/pantallas/Honestidad.tsx`                                                                                                 | `const TOPE: usize = 20` en `Registro::anotar`, que suelta (y pisa) la más vieja al pasarlo. Honestidad suma una fila «lo que salió al API · N peticiones» (TEXTO). Test: 25 `anotar` → quedan 20 y la primera ya no está.                                                                                                                                                         | **pagado** (Fase 2) — `TOPE_DE_LO_QUE_SALIO` = 20 en `Registro::anotar`; Honestidad cuenta «Lo que salió al API · N peticiones» (estado de maqueta `s3-api`, TEXTO: maquetado, no visto). Test `el_registro_recuerda_solo_las_ultimas` (rojo: 25). |
| **B16** | La bandeja abierta en memoria sobrevive a su archivo vencido cuando hay otra viva, y el barrido no la suelta: lo del cliente sigue en RAM pasada su hora.                                                                                                                                       | `src-tauri/src/reunion.rs:641-654` · `src-tauri/src/reunion.rs:942-946`                                                                                                  | En `la_bandeja` y tras `barrer` con bandejas > 0: si la abierta no está en `vivas`, `*m = None`. Test con `ElCuaderno`: bandeja abierta cuyo archivo se borró → `con_la_abierta` devuelve `None` tras refrescar.                                                                                                                                                                   | **pagado** (Fase 2) — `ElCuaderno::soltar_si_no_vive` en `la_bandeja` y en el barrido. Test `la_bandeja_abierta_muere_con_su_archivo_aunque_haya_otra` (rojo con el cuerpo vacío). |
| **B17** | La puerta se abre aunque la carpeta de la app no quede en 700 (solo se loguea), y el socket nace con el umask antes del `chmod` a 600.                                                                                                                                                          | `src-tauri/src/lib.rs:2726-2731` · `src-tauri/src/puerta/socket.rs:123-137`                                                                                              | En `intentar_abrir`, antes de `bind`, exigir que la carpeta esté en 700 (`metadata().permissions().mode() & 0o777 == 0o700`) o devolver `NoAbre::Socket` sin crear nada. Test en `src-tauri/tests/puerta.rs`: carpeta en 755 → no se abre y no queda socket.                                                                                                                       | pendiente (Fase 2) |
| **B18** | Un comentario de test está pegado al test equivocado: el de «no tienen bordes» encabeza el del cuaderno protegido.                                                                                                                                                                              | `src-tauri/src/ventana/mod.rs:327-336` · `src-tauri/src/ventana/mod.rs:367`                                                                                              | Mover el primer párrafo (`:327-332`) encima de `la_banda_y_su_relleno_no_tienen_bordes`.                                                                                                                                                                                                                                                                                           | pendiente (Fase 2) |
| **B19** | El README dice «lo que dijo el cliente… muere al cerrar; lo único que queda es lo tuyo», y calla que un hecho de una línea del cliente puede esperar hasta 24 h en la bandeja.                                                                                                                  | `README.md:12-16` · `README.md:57-60`                                                                                                                                    | Añadir en es y en: «salvo, si lo eliges, un hecho de una línea en la bandeja, cifrado, que se borra solo en tu ventana (máximo 24 h)».                                                                                                                                                                                                                                             | pendiente (Fase 2) |
| **B20** | El ADR 011 dice «La vista en IA **llega** en la fase 1 del sprint 003»: ya existe («Ver lo que salió»).                                                                                                                                                                                         | `decisions/011-proveedores-del-modelo-y-minimizacion.md:54`                                                                                                              | «La vista en IA existe desde la fase 1 del sprint 003 (mirada 19): «Ver lo que salió».»                                                                                                                                                                                                                                                                                            | pendiente (Fase 2) |
| **B21** | La guía dice «El `.txt` se lee con cualquier editor»; lo que se exporta es `.md`.                                                                                                                                                                                                               | `docs/GUIA-DE-PRUEBA.html:654` · `src-tauri/src/reunion.rs:883`                                                                                                          | «El `.md` se lee con cualquier editor».                                                                                                                                                                                                                                                                                                                                            | pendiente (Fase 2) |
| **B22** | La prueba l6 lista `~/Documents`, `~/Desktop` y `~/Downloads` en Terminal, que puede pedir su permiso TCC. La lista de avisos de la guía no lo anuncia, y dice «si aparece uno que no está aquí, deniégalo» (regla 22).                                                                         | `docs/GUIA-DE-PRUEBA.html:635` · `docs/GUIA-DE-PRUEBA.html:247-256`                                                                                                      | Añadir a la lista: «Terminal puede pedirte acceso a Documentos, Escritorio o Descargas al listar esas carpetas (prueba l6): permítelo solo si quieres correrla».                                                                                                                                                                                                                   | pendiente (Fase 2) |
| **B23** | En el ⭐⭐, el aviso «ítem en segundo plano añadido» sale en la **parada 7** (las notas con fecha registran la tarea), no en la 9, que es la única que lo anuncia. Y la parada 8 empieza «Con Meet cerrado» sin decir que hay que dejar de compartir y cerrar la pestaña que abrió la parada 1. | `docs/GUIA-DE-PRUEBA.html:649` · `docs/GUIA-DE-PRUEBA.html:688` · `docs/GUIA-DE-PRUEBA.html:701`                                                                         | m1: «Al guardar, macOS avisa la primera vez de un ítem en segundo plano (la tarea de borrado)». o1: «si no lo viste en la parada 7…». n7: «Deja de compartir y cierra la pestaña de Meet». Minutos igual; `guia-cuadra` verde.                                                                                                                                                     | pendiente (Fase 2) |
| **B24** | La prueba o5 dice «espera a que venza», pero el aviso rojo solo sale si al abrir la app lo vencido lleva más de 2 min.                                                                                                                                                                          | `docs/GUIA-DE-PRUEBA.html:714` · `src-tauri/src/vencimiento/mod.rs:157-159`                                                                                              | «Espera cinco minutos después de que venza (la app solo acusa lo que venció hace más de 2 min)».                                                                                                                                                                                                                                                                                   | pendiente (Fase 2) |
| **B25** | Tres cifras de la cabecera de la guía no tienen gate (casilla 6 a): «Deja fuera 75 pruebas ⭐», «29 pruebas nuevas» y «33 de las 72 heredadas reescritas». Hoy son ciertas: 84 − 9, 29 y 33 de 72.                                                                                              | `docs/GUIA-DE-PRUEBA.html:185-186` · `docs/GUIA-DE-PRUEBA.html:213` · `tests/unit/guia-cuadra.test.ts`                                                                   | Tres aserciones en `guia-cuadra`: fuera = ⭐ − ⭐⭐; nuevas = `data-origen="nuevo"`; reescritas = `mejorado` de `heredada + mejorado`. Demo en rojo cambiando una cifra.                                                                                                                                                                                                           | pendiente (Fase 2) |
| **B26** | Copias de la llave sin pisar: el cifrador arma una `Key` temporal, y `la_llave` compara la llave nueva con la leída convirtiendo las dos en `String` hexadecimales.                                                                                                                             | `src-tauri/src/notas/cifrado.rs:61-63` · `src-tauri/src/carpeta.rs:77`                                                                                                   | `XChaCha20Poly1305::new_from_slice(&self.0)` sin `Key` intermedia. Un `Llave::igual(&self, &Llave) -> bool` en tiempo constante sobre los bytes, en vez de `a_hex() !=`. Verificado: `grep -n "a_hex() !=" src-tauri/src` vacío y `cargo test --lib` verde.                                                                                                                        | pendiente (Fase 2) |
| **B27** | `bootout` y luego `bootstrap` inmediato, sin reintento. Si launchd tarda en soltar la tarea, el registro falla y solo lo dice la consola; hasta el próximo inicio de sesión no salta ninguna hora.                                                                                              | `src-tauri/src/vencimiento/mod.rs:193-196` · `src-tauri/src/reunion.rs:808`                                                                                              | Reintentar `bootstrap` hasta 3 veces con 300 ms entre intentos, con un ayudante puro que recibe el ejecutor, testeable. Si falla, un indicador que Honestidad enseñe como `noCorrio` (TEXTO). Test del ayudante con un ejecutor que falla dos veces.                                                                                                                               | pendiente (Fase 2) |
| **B28** | La maniobra §10 era lo primero que se cortaba «si no cabe, H2 declarado», y no se construyó ni se declaró: el corte silencioso que el método prohíbe.                                                                                                                                           | `decisions/008-el-corpus-el-disparo-y-la-ficha.md:115` · `sprints/SPRINT_003-implementation-log.md:99`                                                                   | Una enmienda al ADR 008 que diga que la §10 pasa a H2 y por qué; `## Desviación del plan` en la bitácora; y el corte declarado en el summary y en el PR.                                                                                                                                                                                                                           | pendiente (Fase 2) |
| **B29** | La carpeta del corpus no se recuerda. Tras reiniciar, «Este cliente» queda vacío, la NDA guardada no se alcanza y la puerta no tiene corpus hasta volver a señalarla. El manual lo dice, pero sin horizonte.                                                                                    | `src-tauri/src/prefs.rs:89-112` · `docs/MANUAL-DE-USO.md:109-110`                                                                                                        | **Decide el usuario.** O bien `carpeta_del_corpus: Option<String>` en `Preferencias`, que se reindexa en segundo plano al arrancar si existe (con el test de reinicio de `prefs` ampliado). O bien el manual la declara «H2».                                                                                                                                                      | pendiente (Fase 2) |
| **B30** | Los logs de error llevan el nombre del archivo de la reunión, que lleva el del cliente: «no se pudo guardar: … `paramo-azul-…ghost.tmp`», «[bandeja] no se pudo abrir: paramo-azul-…». El ADR 015 §9 lo prohíbe; el test solo mira la línea de éxito.                                           | `src-tauri/src/carpeta.rs:200-203` · `src-tauri/src/reunion.rs:416` · `src-tauri/src/reunion.rs:551` · `src-tauri/src/reunion.rs:675` · `src-tauri/src/almacen.rs:77-79` | Los errores de `carpeta` y `bandeja` dicen «el archivo de la reunión» sin nombre. Las tres líneas de `reunion.rs` loguean un motivo sin ruta: un `fn sin_ruta(e: &str) -> String` que corta en la primera ruta absoluta. Test: el error de `abrir_en_claro` sobre `paramo-azul-2026-09-27.ghost` inexistente no contiene «paramo».                                                 | **pagado** (Fase 2) — `carpeta::abrir_en_claro` dice «el archivo de la reunión»; `sin_ruta` en los tres `println!` de `reunion.rs`. Test `los_errores_que_se_loguean_no_nombran_al_cliente` (rojo: «no se pudo leer paramo-azul-…»). |

## Casilla 4 — frases caducadas (primera pasada)

El barrido se hizo por vocabulario de promesa aplazada: «todavía no», «aún no», «por ahora», «de momento»,
«mientras tanto», «próximamente», «llega», «en esta versión», «más adelante», «no (se) puede», «sin
embargo», «podrás», «permitirá»; en inglés, «not yet», «yet», «for now», «later», «will», «cannot».

Se barrieron `docs/MANUAL-DE-USO.md`, `README.md`, `docs/GUIA-DE-PRUEBA.html`, `src/i18n/es.ts` y `en.ts`, el
uso de `<TodaviaNo />` en las pantallas, las maquetas que esas pantallas copian, `docs/BLUEPRINT.html` (en
texto plano) y `docs/kit-de-prueba/LEEME.md`. Cada acierto se cotejó con lo que la app hace **hoy**.

| Frase                                                                       | Dónde                                                                               | Lo que la app hace hoy                                     | Hallazgo |
| --------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- | ---------------------------------------------------------- | -------- |
| «Conservar lo que dijiste tú — Todavía no»                                  | `src/pantallas/Idioma.tsx:286-289` · `docs/diseno/idioma.html:247`                  | existe: Notas, «al cerrar», y se recuerda                  | M7       |
| «los turnos mueren los dos —el tuyo y el del cliente— al cerrar»            | `src/i18n/es.ts:430-431`                                                            | los tuyos quedan si enciendes «Conservar mis turnos»       | M7       |
| «Escribir notas y acuerdos — Todavía no»                                    | `src/pantallas/Permisos.tsx:139-141`                                                | «Solo notas» funciona sin conceder permisos de captura     | M7       |
| «Solo notas» apagado con `title` «Todavía no»                               | `src/componentes/Banda.tsx:836`                                                     | el modo existe (se elige en Sesión)                        | M7       |
| «abrir o exportar una reunión guardada», «como abrir una reunión», «ábrela» | `docs/MANUAL-DE-USO.md:204` · `src/i18n/es.ts:747` · `docs/GUIA-DE-PRUEBA.html:649` | en la pantalla no hay «abrir», solo exportar               | M8       |
| «ligada a este Mac», «ni por copia de seguridad»                            | `docs/MANUAL-DE-USO.md:203` · `docs/BLUEPRINT.html:446`                             | el llavero de archivo viaja con Time Machine y migraciones | A2       |
| «ninguna copia sale de tu Mac», «no puede tener una copia que viva un mes»  | `docs/MANUAL-DE-USO.md:201` · `docs/MANUAL-DE-USO.md:242-243`                       | Time Machine la copia                                      | M2       |
| «la lista que lees es la lista entera (test)»                               | `decisions/016-las-propuestas-y-la-bandeja.md:30-32`                                | la lista es una frase fija; el test no existe              | M11      |
| MLX «Todavía no» · Corpus «Lo que todavía no existe»                        | `src/pantallas/Ia.tsx:166` · `src/pantallas/Corpus.tsx:159`                         | son H2                                                     | B9       |
| «la otra todavía no existe» · «hoy son diez»                                | `src/i18n/es.ts:382` · `src/pantallas/Honestidad.tsx:37`                            | 11 de 11 cortadas                                          | B10      |
| «La primera vez, macOS te pregunta…»                                        | `src-tauri/src/puerta/cli.rs:105` · `src/i18n/es.ts:839`                            | pregunta en cada apertura                                  | B12      |
| «lo único que queda es lo tuyo»                                             | `README.md:12-16`                                                                   | también la bandeja, hasta 24 h                             | B19      |
| «La vista en IA llega en la fase 1»                                         | `decisions/011-proveedores-del-modelo-y-minimizacion.md:54`                         | ya existe                                                  | B20      |
| «esta app no escribe archivos por diálogo»                                  | `src-tauri/capabilities/default.json:4`                                             | exportar sí                                                | B2       |

**Revisadas y ciertas hoy (sin hallazgo):**

- «Todavía no has guardado ninguna reunión» (estado vacío).
- «Mientras tanto, la app funciona igual» (bandera sin jurisdicción).
- «Mientras tanto, prepara la reunión» (Sesión sin reunión).
- «varios idiomas por pista queda para el H2» (`docs/MANUAL-DE-USO.md:590`).
- «la versión firmada… llega en el H2» (`README.md:21`).
- Los dos «no puede» de `docs/kit-de-prueba/LEEME.md:38` y `:57`.
- «Todavía no te pregunta nada» de la guía n1: `--help` no toca el Llavero.
- Los «no puede» del BLUEPRINT (CSP, radar, puerta, Accesibilidad), contrastados con el código.
- «Zoom and Teams are not verified yet» (README): cierto, y es la promesa graduada.

La segunda pasada va después del último ajuste de la Fase 2, e incluye el summary (kit v1.31.0).

## Casilla 5 — campos sin lector

**Resultado: 94 campos de tipos nuevos o ampliados en el sprint; 3 sin lector, más el dato de un evento que
nadie lee.** El gate `tests/unit/contrato-con-lectores.test.ts` da `DEUDA` vacía. No los ve porque compara por
nombre de campo, y otros tipos tienen `archivo`, `bytes` y `vence`: es la limitación que él mismo declara.

Tienen lector de su mismo tipo, comprobado a mano con grep fuera de su construcción y de los tests:

- `VistaDelCuaderno` (14), `ResumenDelCuaderno` (10), `FichaFijada` (4), `Propuesta` (6) y `EnEspera.id`;
- `Previsto` (4), `VistaDeLaBandeja` (7), `EstadoDeLaBandeja` (2), `ReunionGuardada` (4) y `ListaDeReuniones`;
- `VistaDelCliente` (5), `Bandera` (7) y las tres formas de `LaBandera`;
- `VistaDeLaPuerta` (5), `Entrada` (3) y `Resultado` (`cuenta`, `motivo`);
- `LoQueSalio` (7) con `Trozo` (3);
- `IdiomasDePista` (2), por clave calculada, y `EstadoDeEscucha.soloNotas`.

| Tipo o evento                              | Campo                       | Lector                                                      | Hallazgo |
| ------------------------------------------ | --------------------------- | ----------------------------------------------------------- | -------- |
| `Guardada` (`src-tauri/src/carpeta.rs:98`) | `archivo`, `bytes`, `vence` | ninguno: `guardarLaReunion().then(() => …)` ignora el valor | B13      |
| evento `modo` (`src-tauri/src/lib.rs:570`) | el `Modo` que viaja         | ninguno: `src/cuaderno.ts:375` solo vuelve a preguntar      | B14      |

**Fuera de la cuenta, sin hallazgo:** los eventos `cuaderno` (una señal sin dato), `fijada` (un `bool` con lector
en `src/notas.ts:483`) y `propuesta` (`LineaDePropuesta`, en el contrato con su muestra `null`). El contenido
descifrado de una reunión no cruza al webview, y eso es correcto (ver M8).

## Casilla 6 (a) — números cableados

| Número                                          | Dónde                                       | Quién debería darlo                                                                                                                                                   | Estado                                                                                                                               |
| ----------------------------------------------- | ------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| «30 propuestas»                                 | `src/i18n/es.ts:699` · `src/i18n/en.ts:593` | `TOPE_DE_PROPUESTAS` (`src-tauri/src/notas/mod.rs:36`)                                                                                                                | B11                                                                                                                                  |
| las cinco reglas, en una frase                  | `src/i18n/es.ts:697` · `src/i18n/en.ts:591` | `data/propuestas/reglas.json`                                                                                                                                         | M11                                                                                                                                  |
| «hoy son diez»                                  | `src/pantallas/Honestidad.tsx:37`           | `corte::TODAS` (la pantalla ya la pregunta)                                                                                                                           | B10                                                                                                                                  |
| «Deja fuera 75», «29 nuevas», «33 de las 72»    | `docs/GUIA-DE-PRUEBA.html:185-186` · `:213` | el propio documento, contado                                                                                                                                          | B25                                                                                                                                  |
| «0 B» en «Muere al cerrar»                      | `src/pantallas/Notas.tsx:504-505`           | **aceptable**: es cero por construcción, porque `notas::Contenido` no tiene campo donde quepa un turno del cliente ni una lectura. Conviene un comentario que lo diga | sin hallazgo                                                                                                                         |
| 50 órdenes, 8 palabras, 160 letras, 24 h, 2 min | `docs/MANUAL-DE-USO.md`                     | constantes de Rust                                                                                                                                                    | **aceptable**: el manual describe; hoy cuadran (`TOPE_DEL_REGISTRO`, `TOPE_DEL_FRAGMENTO`, `TOPE_DE_LA_FRASE`, `TECHO`, `no_corrio`) | sin hallazgo |

Entidades que la VISION declara por catálogo (unidades, maniobras, jurisdicciones, avisos del radar): la
interfaz de jurisdicciones itera `vista.clientes` y el catálogo sin cardinalidad fija. No hay un literal de N
donde el dato dice N, salvo el de las reglas (M11), que se clasifica **medio** y no alto: el conjunto de
reglas es código (`Regla`) y no datos.

## Casilla 6 (b) — la guía heredada

La guía v5 tiene **101 pruebas**: 39 heredadas, **33 «Mejorado en S3»** y **29 «Nuevo · S3»**. Tiene 84 ⭐,
9 ⭐⭐ y 19 filas de textos. Las cuentas y los minutos que vigila `guia-cuadra` están en verde.

Se leyeron las 62 reescritas y nuevas contra el código de hoy. Las diez «imposibles» que el constructor
reescribió están bien reescritas: e11 (silencio), e6/j2 (transcript con ficha), f2/f4 (página de prueba),
a6 (⌥⎋), h3 (voz del cliente), l2 (11 piezas), l4 (lo tuyo queda) y p1b (idiomas desde `prefs`).

**El recorrido ⭐⭐, parada a parada** (¿deja la app lista para la siguiente?):

| Parada      | Prueba | ¿Caminable?                          | Nota                                                                                                                                             |
| ----------- | ------ | ------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------ |
| Preparación | —      | sí                                   | `pnpm ghost`, bandeja de prueba, corpus, idiomas, «Este cliente», redactar                                                                       |
| 1 de 9      | a2     | sí                                   | deja la pantalla compartida                                                                                                                      |
| 2 de 9      | e3     | sí                                   | con «Redactar» encendido desde la preparación, la sugerencia ya sale aquí                                                                        |
| 3 de 9      | h2     | **no**                               | la misma pregunta no vuelve a disparar → M9                                                                                                      |
| 4 de 9      | i1     | sí                                   | necesita una ficha a la vista: la de la parada 2 sigue                                                                                           |
| 5 de 9      | k1     | sí, pero no guarda ninguna propuesta | lo necesita la 7 → M10                                                                                                                           |
| 6 de 9      | l2     | sí                                   | el corte se lleva las propuestas sin decidir                                                                                                     |
| 7 de 9      | m1     | **no**                               | «ábrela» no existe; «la propuesta que guardaste» no hay; exportar no la escribe → M10, M8, M6. El aviso de ítem en segundo plano sale aquí → B23 |
| 8 de 9      | n7     | sí, con transición implícita         | dejar de compartir y cerrar Meet → B23                                                                                                           |
| 9 de 9      | o1     | sí                                   | «La primera vez, macOS avisa» ya pasó en la 7 → B23                                                                                              |

Suma: 2 + 18 = 20 min, dentro del techo. Declara que deja fuera 75 ⭐, y cuadra. Fuera del ⭐⭐: B21 (m2
`.txt`), B22 (l6, aviso TCC de Terminal) y B24 (o5, tiempo de espera). La n5 («si ya la abriste») cae en M8.

## Regla 22 — las protecciones del Mac

**Qué toca el código de este sprint:**

- **Llavero:** dos servicios nuevos. «Angel Ghost · notas» crea la llave en el primer guardado y la lee al
  guardar, exportar y abrir por la puerta. «Angel Ghost · puerta» crea y borra un token en cada apertura, y
  `ghost` lo lee con el diálogo de la lista de acceso. Además, «Angel Ghost · API» pasó a `llavero.rs`.
  Ver A2 para lo que ese llavero hace de verdad.
- **Touch ID o contraseña** (LocalAuthentication): al exportar, al abrir una bandeja de otra sesión y en
  `ghost notas abrir`.
- **launchd e Ítems de inicio:** un plist en `~/Library/LaunchAgents`, `launchctl bootout/bootstrap` sobre
  `gui/<uid>`, el aviso «Ítem en segundo plano añadido» y la entrada «sh · desarrollador no identificado».
- **TCC:** `NSDocumentsFolderUsageDescription` entró y salió (decisión A). La app ya no pide Documentos.

**Qué tocó el constructor sin «sí».** El inventario honesto de la bitácora
(`sprints/SPRINT_003-implementation-log.md:533-561`):

- el test del Llavero `--ignored`;
- `osascript` con System Events;
- la prueba en vivo de launchd, con `~/Documents/Angel Ghost`;
- una tarea de diagnóstico que leyó `~/Documents`;
- seis `sfltool dumpbtm` con contraseña de administrador.

De ahí nace la regla 22 (`CLAUDE.md`). **Después de la regla no hay ningún «sí» registrado:** en los cierres
de las fases 3 y 4 el usuario contestó «continúa» sin «sí» (bitácora `:820-821` y `:945-952`), y el
constructor no corrió nada.

**Lo que sigue pendiente, y la regla 15 (tercer filo) lo exige antes de cerrar.** La prueba en vivo con la app:

- launchd registrándose desde la app;
- el cuaderno negro en Meet;
- Touch ID al exportar;
- los indicadores del sistema apagados en solo notas;
- `ghost` leyendo el Llavero y la puerta cerrándose al empezar la sesión.

Cada acción, con su matriz de una fila y su «sí».

**A mirar en esa prueba, sin hallazgo todavía.** Cada guardado con fecha cambia el plist y hace
`bootout + bootstrap`. Hay que ver si macOS repite el aviso de ítem en segundo plano en cada reunión: el ADR
016 lo deja abierto (`decisions/016-las-propuestas-y-la-bandeja.md:237-239`). Si lo repite, es un hallazgo
de la segunda pasada.

**En esta auditoría** no se corrió nada de la regla 22. `cargo test --test ghost` se leyó antes de correrlo:
usa un `HOME` vacío y no llega al Llavero.

## Lo que está bien

- **El cifrado** es el adecuado y está bien cosido. XChaCha20-Poly1305 con nonce al azar y la cabecera
  (magia, versión, vencimiento) como datos asociados: alargarle la vida a un archivo por fuera lo deja sin
  abrir, y hay test. `zeroize` activado. La llave solo se crea si el Llavero dice «no», nunca ante un error,
  y se vuelve a leer antes de cifrar.
- **`almacen`:** `create_new` con su modo, `sync_all`, renombrado atómico y solo la carpeta del archivo en 700. Cada propiedad con su rojo.
- **Propuestas:** del cliente, jamás el turno (≤ 8 palabras; de una pregunta, solo palabras clave), el eco
  cuenta como cliente, y dos tests nacieron en rojo. Se resuelven en microsegundos.
- **La bandeja:** con «al cerrar» no se escribe ni la carpeta. `revencer` tiene su techo de 24 h. El barrido
  borra leyendo solo la cabecera, sin llave.
- **El `sh` de launchd:** comillas en todas las variables, `--` antes de la ruta, solo `.ghost`, la lista
  entra como argumento y no dentro del script. Probado con `/bin/sh` de verdad, sin depender del binario.
- **La puerta:**
  - socket Unix, carpeta 700 y socket 600;
  - token por apertura, comparado en tiempo constante;
  - `enum` cerrado con `deny_unknown_fields`;
  - la reunión se mira antes que la orden, y ante la duda (candado envenenado, sin Accesibilidad) cuenta
    como reunión;
  - el estado cambia antes de responder;
  - el registro sin contenido, probado con canaria;
  - `ghost` mira el socket antes que el Llavero;
  - ningún test toca el Llavero;
  - el gate `puerta-solo-local` nace con su rojo.
- **Solo notas** tiene un gate que cierra la captura por construcción (el orden en la fuente de `empezar`),
  y `⌃⌥L` no lee nada sin lectura viva.
- **El contrato** creció en 48 muestras con los datos de la maqueta, y `DEUDA` sigue vacía.
- **El catálogo legal** marca como «sin verificar» cada gap del informe (G-1…G-12; Missouri, Hawái y
  Maine), trata «sin verificar» como el riesgo mayor y no adivina la jurisdicción más parecida. Las filas
  contrastadas con §2.b y la matriz dicen lo mismo que el informe.
- **La guía:** gate de minutos, desglose por sprint, chips con su origen y namespace `ag-s3-`. Las diez
  pruebas imposibles de la v4 están bien reescritas.
- **El incidente de la regla 22** está contado entero, con hora, qué tocó y cómo quedó. De ahí salió una
  regla para todas las apps.

## Orden propuesto para la Fase 2

1. **Antes de usarla con clientes reales (datos y privacidad):**
   - A1, colisión de nombres;
   - M1, copia en memoria tras el corte;
   - M5 y M6, exportar;
   - B16, bandeja en memoria vencida;
   - B15, tope del registro;
   - B30, nombres de cliente en los logs de error.
2. **Afirmaciones de seguridad** (empieza con la comprobación del usuario en Acceso a Llaveros):
   - A2, el Llavero;
   - M2, Time Machine (el usuario decide sobre `notas/`);
   - M3, retención de proveedores (el usuario decide la lista);
   - M4, la puerta y el desbloqueo (el usuario decide: por apertura o por orden);
   - B17 y B12.
3. **Lo que la app dice y no es cierto:** M7, M8, B9, B10, B19, B20, B2, B3.
4. **La guía, antes del Acto 2:** M9, M10, B21, B22, B23, B24, B25.
5. **Gates, contrato y tests:** M11, M13, B11, B13, B14, B7, M12 (el usuario decide sobre dependabot de Cargo).
6. **Higiene y declaraciones:** B1, B4, B5 (tras la prueba en vivo), B6, B8, B18, B26, B27, B28, B29 (decide el usuario).
7. **Después del último ajuste:**
   - la casilla 4 otra vez, con el summary incluido;
   - la prueba en vivo con sus filas de la regla 22;
   - `/release-check` con el peso del binario.

_Aprueba la Fase 1 y fija el modelo de la Fase 2 con `/model` (un modelo menor basta si sigue este plan)._
