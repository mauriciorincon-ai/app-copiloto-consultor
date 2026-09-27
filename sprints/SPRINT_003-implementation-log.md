# Bitácora — Sprint 003 «El cuaderno y el cierre» (cierre del ciclo H1)

Branch `sprint-003/el-cuaderno-y-el-cierre`, desde `main` en `2eaae5d`. Orden
`SPRINT_003-orden.md` (aprobada 2026-09-27) · plan autoritativo `SPRINT_003.md` · kit v1.31.0 ·
método v1.33.0 · ciclo H1, sprint **3 de 3**.

Plan de fases aprobado el 2026-09-27, con su bloque de arranque y el «construye» del usuario.

## Las tres decisiones del usuario antes del plan (2026-09-27)

| Pregunta | Respuesta | Qué se construye |
|---|---|---|
| ¿Dónde viven las notas cifradas? | «Documentos/Angel Ghost» (la maqueta) | `~/Documents/Angel Ghost/`; macOS pide el permiso de Documentos una vez; si Documentos está en iCloud, viaja una copia cifrada ilegible fuera de este Mac |
| ¿Cómo vence la bandeja sin abrir la app? | «Me interesa la tarea… pero cada 5 min me parece excesivo y si consume recursos menos aun me convence, revísala bien» | launchd **al minuto exacto de cada vencimiento** (`StartCalendarInterval`) + una pasada al iniciar sesión (`RunAtLoad`): entre vencimientos no corre nada |
| El cuaderno no está protegido al compartir pantalla | «Protegerlo mientras hay sesión» | la ventana principal toma el flag al empezar a escuchar y lo suelta al terminar |

---

## Fase 0 · Delta del kit v1.31.0 y deuda del S2

### El delta del kit
- **`CLAUDE.md`**, citado por nombre:
  - regla 10: el párrafo de las **dos clases de mirada** (FORMA abre parada, TEXTO no bloquea; matriz de una fila);
  - regla 15: el párrafo de **`gh pr checks` tras cada push** y de la métrica `manual`;
  - las reglas del kit que faltaban: **«IA de construcción por suscripción»** entra como 21 (hoy no aplica: la app no invoca a Claude Code; la puerta local es lo contrario) y **«bilingüe en TODO desde el primer sprint»** entra en § Idioma.
- **`.claude/commands/`**, copiados del kit, porque las diferencias eran solo los trozos nuevos:
  - `audita-sprint`: las dos casillas 6, la Fase 2 paga todos, la segunda casilla 4 incluye el summary;
  - `release-check` §9: `scrollHeight`;
  - `plan-sprint` §9-bis (f);
  - `deploy-check.md` y `README.md`, que el repo no tenía aunque `release-check` remite a `/deploy-check` §3 y §7.
- **Skill `ia-embebida` §9** (grounding en dos mitades) y su casilla. La síntesis ya lo cumple: `sintesis/fiel.rs` es el origen del patrón.
- **`scrollHeight` en la pasada de capturas:** `pnpm fidelidad` ya lo medía del lado del producto; faltaba su rojo (regla 15, gate heredado).
  - **Rojo:** un `<div style={{ height: 900 }}>` plantado en Honestidad → «contenido alto +844px» en los cuatro encuadres, y exit 2.
  - Restaurado; el verde va con la corrida de cierre de fase.
  - Las maquetas las cubre `maqueta-cabe.spec.ts` (verde con los estados nuevos de la mirada 19).
- **`SPRINT = "s3"`** en `capturar-fidelidad.mjs`, en el primer commit que captura: la evidencia del S2 queda intacta.
- **WER `manual`**, corrida local registrada (2026-09-27, este Mac):

  | Audio | Sin diccionario | Con diccionario | Resultado |
  |---|---|---|---|
  | `pregunta-es` | 0,133 | 0,133 | igual |
  | `pregunta-en` | 0,000 | 0,000 | igual |
  | `mezcla-es` | 0,458 | 0,417 | **mejora** |
  | `mezcla-en` | 0,348 | 0,261 | **mejora** |

  Son las mismas cifras del S2. En la CI no se mide, y lo dicen la guía, el LEEME, el ADR 009 y el summary.

### Fricciones del kit (K#), para la planeadora
- **K1:** `audita-sprint.md` del kit tiene **dos «casilla 6»**: la de números cableados (v1.29) y la de la guía heredada (v1.31). Aquí se citan por nombre.
- **K2:** la regla 20 del kit (bilingüe) choca con la regla 20 de esta casa (artefacto de auditoría). Se citan por nombre; la 21 del kit entra como 21.
- **K3:** `BLUEPRINT.plantilla.html` es de perfil web y pide «[URL pública]» en el SVG y en la tabla, que choca con la regla 17. Se instanciará de escritorio y sin URL (fase 5).
- **K4:** el estampado de escritorio no traía `deploy-check.md`, aunque `release-check.md` remite a él.
- **K5:** el cuerpo de `metodo.md` (:260-261) sigue diciendo «Fase 2: plan de ajustes Crítico/Alto»; la v1.33.0 solo lo cambió en el changelog y en el kit.

### Un solo escritor y un Llavero compartido
- **`src-tauri/src/almacen.rs`**: todo lo que persiste pasa por aquí.
  - Nace en 600 con `create_new`.
  - Usa temporal, `sync_all` y renombrado atómico.
  - La carpeta queda en 700, y se repara solo la carpeta del archivo, nunca sus padres.
  - El diccionario y el gasto del mes ya lo usan.
  - **Rojo:** con `std::fs::write` en `nacer_cerrado`, «hubo que cerrarlo: nació abierto».
- **`src-tauri/src/llavero.rs`** y `nativo/Llavero.swift`, con servicio por dueño: «Angel Ghost · API» (conserva el nombre del S2, test), «· notas» y «· puerta».
  - El FFI salió de `sintesis/api.rs`.
  - **Corrido a mano contra el Llavero real** (`el_llavero_guarda_lee_y_borra_la_clave`, `--ignored`): guarda, lee y borra con la firma nueva.

### Las preferencias que se recuerdan (deuda S2)
- **`src-tauri/src/prefs.rs`** → `preferencias.json` (600): idiomas por pista, «redactar sugerencias», API encendido y proveedor, lectura automática.
  - Se aplican en `setup`, antes de que ninguna pantalla pregunte.
  - Se guardan en cada cambio.
  - El API **no** se enciende solo si su clave ya no está en el Llavero.
  - La puerta local **nunca** persiste abierta.
- **Comandos** `idiomas_de_pista` y `fijar_idioma_de_pista`, solo en la ventana principal. El contrato suma `IDIOMAS_DE_PISTA`, que deja de estar en `NO_CRUZAN`.
- **Rojos:**
  - Rust, sin `#[serde(default)]`: fallan los tres tests de archivo viejo o torcido.
  - Rust, sin escribir: fallan el del reinicio y el de 600.
  - TS, sin `pedirLosGuardados()` en el suscriptor y sin el `llamar` de `fijarIdiomaDePista`: fallan los dos de `tests/unit/preferencias.test.tsx`.
- **Manual** al día («la elección se recuerda», «los interruptores de IA se recuerdan», con la excepción de la clave borrada). Enmienda 2 al ADR 002.

### B37 — lo que salió al API
- **`sintesis::api::Registro` y `LoQueSalio`**: cada petición anota, **en el instante de salir**, el texto exacto por trozos, con cada marcador y su original al lado, más caracteres, cuántos se taparon y, al volver, su costo.
- **Solo en memoria:**
  - lo vacían ⌥⎋ (`cortar_la_sugerencia`), el final de la sesión y la sesión siguiente;
  - cada copia se pisa al soltarse;
  - lo pide solo la ventana principal (`lo_que_salio_al_api`), nunca un evento.
- **Rojos:**
  - sin `vaciar()` en el corte → «lo que salió sobrevivió al corte»;
  - con `tapadas: 0` → el test del texto exacto cae.
- **Desviación declarada:** la **vista** espera a la mirada 19. La pantalla IA del S2 ocupa sus 640 px, y dónde cabe «lo que salió» es una decisión de FORMA. El contrato cruza ya; `contrato-con-lectores` lleva sus 7 campos en `DEUDA` con «fase 1».
  - `hora`, `externo` y `Trozo.texto` no caben en esa lista: el gate compara por nombre y otros tipos tienen campos iguales (su limitación declarada).
- **ADR 011, punto 4, enmendado** con lo que existe hoy. El comentario de `tapadas()` también, porque la primera redacción ya decía «IA lo enseña».

### ADR 008, `lru`, maniobra §10
- **ADR 008 — embeddings decididos: no en el H1.**
  - BM25 cumple el umbral del propio kit (0,823 frente a 0,80).
  - Medirlos exigiría traer un modelo en un sprint de cero modelos.
  - El kit v0 no tiene la clase de pregunta que los arreglaría.
  - Condición escrita que los reabre: diez preguntas de paráfrasis con nDCG@5 < 0,80. **Roadmap H2.**
- **`lru`:** `cargo audit` da 0 vulnerabilidades y 9 avisos, con `lru` 0.16.4 vía tantivy. **tantivy 0.26.2 es la última publicada**: sin subida posible, sigue vigilado.
- **Maniobra §10:** es lo primero que se corta si el esfuerzo se pasa. Se intenta al final de la fase 2 solo si cabe; si no, H2, declarado.

### La mirada 19 (FORMA), maquetada
En sus archivos, como «decisión de diseño no escrita», con su «Qué mirar». Capturas leídas como imagen y `maqueta-cabe` en verde:
- `notas.html` · **sprint 3 · durante**: tu nota es un campo, ⌃⌥N te trae aquí, acuerdos sin tecla (campo + ↵) y «el cliente no la ve».
- `notas.html` · **sprint 3 · el archivo**: la retención como elección para todas las reuniones (7 d · 30 d · **90 d** · 1 año · siempre).
- `notas.html` · **sprint 3 · exportar**: el aviso convertido en pregunta con dos botones.
- `ia.html` · **sprint 3 · quién redacta**: un botón «Ver lo que salió · N» en la tarjeta del proveedor externo.
- `ia.html` · **sprint 3 · lo que salió**: el estado «API encendido» aprobado, con botón para volver.

### Casilla 6 sobre la guía v4 (kit v1.31.0): las 72 pruebas contra la arquitectura de hoy

Lo hizo un subagente de solo lectura. **Resultado: 72 revisadas.**
- **10 falsas o imposibles hoy:** k1b, d11, i2, d6, h5, e4, a6, g1, g3 y e2. De e2 falta confirmarlo en vivo.
- **10 caducan en la fase 1:** a2, a3b, a3c, b1, b5, g6, j1, j2, j4 y j6.
- **52 ciertas.** De ellas, 7 con un punto de partida que cambió la fase 0 (d3, e1, e3, g2, g5, k2, k4: las preferencias ya se recuerdan) y 6 con desajustes menores de redacción (b2, d1, b8, h4, j2b, j7).

**Las reescrituras van a la guía v5 (fase 5)**, con origen «Mejorado en S3», jamás borradas. El texto listo para pegar de cada una está en el informe del subagente; el resumen:
- **k1b:** se recuerda al reiniciar. La consola dice `[prefs] idiomas … / en-US`; sin pasar por Idioma, «Iniciar sesión» abre `sistema · en-US`.
- **d11:** la frase del S2 tenía palabras del corpus («calidad», «datos», «canal»). Nueva: «Nos preocupa el margen de las tiendas de vereda».
- **i2 y d6:** el transcript solo se pinta con una ficha en la banda, y hay que decirlo. «⌃⌥T no agranda» era cierto y **ya no lo es**: se pagó como bug 10 (abajo); la v5 dice que agranda.
- **h5:** con el modo solo audio la banda de una línea sigue «Callado». La «sin resultado» se ve al salir del modo.
- **e4:** necesita una página de kit en negro, sin texto ni pie (`meet-en-negro.html`). Tapar la ventana no sirve: ScreenCaptureKit la lee igual.
- **a6:** «Cierra la banda» pasa a ⌥⎋, que es la única forma: ⌘W no la cierra (bug 8, medido abajo).
- **g1:** IA se pone al día al volver (pagado abajo). La prueba entra y sale de IA.
- **g3:** las tres preguntas las dice el CLIENTE (`say -v Paulina …`), no tu micrófono.
- **e2:** el kit dice «Páramo Azul» en el título de la pestaña y «N de 6» en el pie. La app lee la ventana entera, y eso le da a la agenda un término y una cifra. Hay que cambiar la página del kit y esperar 6 s entre diapositivas.

**Bugs de producto que encontró, y su estado.** Cada uno con su test en rojo antes del verde:

| # | Bug | Origen | Estado |
|---|---|---|---|
| 1 | Tras reiniciar, Sesión mandaba los idiomas de fábrica si no se había abierto Idioma: el webview los mandaba desde su caché | **fase 0 (mío)** | **pagado**: `empezar_a_escuchar` ya no recibe idiomas y Rust los lee de `preferencias.json`. **Rojo:** con los idiomas en la llamada, `la-ficha-llega-a-la-banda` «no manda idiomas» cae |
| 2 | «Borrar la clave» no guardaba «API apagado»: con otra clave y un reinicio, se encendía solo | **fase 0 (mío)** | **pagado**: `apagar_si_usaba` + `recordar`. **Rojo:** con `false` siempre, cae `borrar_la_clave_del_encendido_lo_apaga` |
| 3 | Honestidad («lo único que la app escribe…») no nombraba tus preferencias. El comentario de `LaPantalla` decía «no se guarda en disco» | fase 0 | **pagado**: es/en y maqueta (dos estados), más el comentario |
| 4 | Sesión decía «Zoom · sin verificar» con cualquier cliente sin verificar | S2 | **pagado**: nombre corto del cliente + sufijo; sale la clave `proteccionSinVerificar`. **Rojo:** con «Zoom» fijo, cae «nombra a SU cliente» |
| 5 | La banda no se enteraba de la reunión si la app se abrió antes de la llamada: solo preguntaba al montarse y con el foco, y la banda no recibe foco | S2 | **pagado**: `useReunion` vuelve a preguntar al abrir una pista (`escucha` · `empieza`). No vigila nada en segundo plano (ADR 005). **Rojo:** sin `alEmpezarLaSesion`, cae `reunion-al-empezar.test.tsx` |
| 6 | Dos tooltips del asa solo en español en la interfaz inglesa | S1/S2 | **pagado**: `banda.asaAjustar` y `banda.asaVolver` es/en. La maqueta los lleva como texto `sr` para que el gate del diccionario los vea |
| 7 | IA no se ponía al día al volver a la ventana (Apple Intelligence apagado en Ajustes no emite nada) | S2 | **pagado**: `useIa` pregunta también con `focus`. **Rojo:** sin el `focus`, cae `ia-al-volver.test.tsx` |
| 8 | **⌘W sobre la banda** la cerraría sola: el relleno quedaría encima de la reunión y Chrome encogido | S1 | **no se reproduce, medido en vivo.** Con una sonda en `CloseRequested` y **sin** freno: ⌘W y «Close All» sobre la banda no llegan (tres ventanas antes y después); sobre la principal sí («CloseRequested «principal»», se cerró). La banda y el relleno no tienen bordes y macOS no les manda el cierre. Se probó primero un freno (`on_window_event` + `destroy()` en el corte, con su rojo), y **se retiró**: ningún camino lo alcanzaba, era código que nunca corre (tercera pregunta de la regla 15). Queda atada la razón real: `la_banda_y_su_relleno_no_tienen_bordes` (`ventana/mod.rs`). **Rojo:** con `decorations: true` en la banda, cae nombrándola |
| 9 | El manual promete cosas que la relectura desmintió: e4 («tapa la ventana», `MANUAL-DE-USO.md:195-196`) y el transcript incondicional (`:65-66`, `:411`) | S1/S2 | **pagado.** La lectura se hace aunque otra ventana tape la reunión (`SCContentFilter(desktopIndependentWindow:)`); lo que la deja sin leer es minimizarla u ocultarla (`isOnScreen`, `Pantalla.swift:96`). El transcript se pinta junto a la ficha y agranda la banda compacta |
| 10 | **⌃⌥T no agrandaba la banda**: cambiaba el dibujo a «ampliada» (200 px) dentro de una ventana de 88 y la banda salía recortada por abajo. El comentario de `Banda.tsx` prometía lo contrario | S1 | **pagado.** `useAltoDelTranscript` (`src/asa.ts`), llamado desde el enrutador: abre → `ajustar_banda` + `asentar_banda` a 200, como el asa; cierra → vuelve a 88, solo si la agrandó el transcript. **Rojo:** sin la llamada en `App.tsx`, cae `transcript-agranda.test.tsx` («expected [] to deeply equal [ajustar_banda:200, …]»). **En vivo:** una corrida con la banda a la vista, 88 → 200 → 88 medido por System Events, con su reacople en el log (compilación con una sonda Rust; el JS, idéntico al final). Dos intentos más no cambiaron el alto, y **no está medido por qué**: en uno se pulsó nada más abrir, con Vite en frío; en el otro, CoreGraphics daba todas las ventanas de la app fuera de pantalla (otro escritorio). Dejé de mandar teclas porque el usuario estaba trabajando en ese Mac. Lo repite la prueba i2 de la guía v5, con la banda delante |

- **Gate del contrato:** `IdiomasDePista.consultor` se lee por clave calculada (`elegidos[cual]` en Idioma) y queda declarado en `LEIDOS_POR_CLAVE`.
- **`design-sync/`** regenerado: la banda cambió por los tooltips.

---

## ⏸ PUNTO SEGURO (2026-09-27) — pedido por el usuario para compactar

**Estado:**
- Rama `sprint-003/el-cuaderno-y-el-cierre`, PR **#8** en borrador.
- Commit de la fase 0: `ac93bba`, CI con los tres checks en success.
- Este punto seguro va en el commit siguiente.
- Gates locales verdes: vitest 262 · cargo --lib 356 · clippy · lint · typecheck · `verify:ephemeral` estático.

**Lo que falta para cerrar la fase 0, en orden:**
1. **⌘W (bug 8).**
   - `ventana::cerrar_banda` (`ventana/mod.rs:208`) usa `v.close()` sobre banda y relleno, así que un `prevent_close` global bloquearía también al corte.
   - Plan: en `tauri::Builder` añadir `.on_window_event`, y para `banda` y `relleno` en `CloseRequested` hacer `api.prevent_close()`, **salvo** que el cierre venga del corte.
   - Opción A: una bandera `AtomicBool` que `cerrar_banda` levanta antes de cerrar.
   - Opción B: cambiar `cerrar_banda` a `destroy()`, que no emite `CloseRequested`; comprobar primero en la doc de Tauri 2.
   - Test del camino puro + comprobación en vivo en `pnpm tauri dev`. La prueba a6 de la guía v5 pasa a «⌥⎋ es la única forma de quitar la banda».
2. **El manual (bug 9).**
   - `MANUAL-DE-USO.md:195-196`: tapar la ventana no deja a la app sin texto; ScreenCaptureKit la lee igual.
   - `:65-66` y `:411`: el transcript se abre con ⌃⌥T **cuando hay una ficha en la banda**, y no agranda la banda.
3. **Fidelidad:** volver a correr `PUERTO_E2E=4300 pnpm fidelidad` (SPRINT ya es `s3`) y comitear `docs/fidelidad/s3-*` y `S3-*.html`. Están sin versionar a propósito: la primera corrida fue antes del cambio de texto de Honestidad.
4. Commit y push; `gh pr checks 8` en success.
5. **Mensaje de cierre de la fase 0** al usuario, que lleva la **mirada 19 en matriz de una fila por decisión**:
   - `docs/diseno/notas.html` · «sprint 3 · durante» · teclas reales, acuerdos sin tecla, «el cliente no la ve»;
   - `notas.html` · «sprint 3 · el archivo» · retención 7 d / 30 d / 90 d / 1 año / siempre, global;
   - `notas.html` · «sprint 3 · exportar» · la pregunta antes de exportar;
   - `ia.html` · «sprint 3 · quién redacta» · el botón «Ver lo que salió · N»;
   - `ia.html` · «sprint 3 · lo que salió» · la vista con volver.
   Y recordar `/model` y esperar «continúa».
6. **Después, fase 1**, que empieza por el ADR 015 «las notas y su cifrado». Ver el plan: `~/.claude/plans/idempotent-marinating-pebble.md`.

### Retomado tras el compact (2026-09-27)

- **CI de `752dca9`:** quality, e2e y build-escritorio en success, cada uno con su conclusión propia.
- **Pasos 1–3 del punto seguro, hechos:** los bugs 8, 9 y 10 de la tabla de arriba. El 10 no estaba en la lista: apareció al comprobar el 9 contra el código.
- **Gates locales:** vitest 264 · cargo --lib 357 · clippy limpio · lint · typecheck · `verify:ephemeral` estático.

---

## Fase 1 — Lo tuyo queda (en curso, 2026-09-27)

**La mirada 19 viajó en el cierre de la fase 0.** El usuario respondió «continúa» sin comentar las maquetas. Eso pasa el gate de FASE, no el de MIRADA (regla 10). Así que se repreguntó «¿qué viste al abrirlo?» y se avanzó **solo con lo que no depende de esa mirada**. La pantalla de Notas, la elección de retención, el aviso de exportar, la vista de «lo que salió» (B37), ⌃⌥N y ⌃⌥P esperan su veredicto.

### Primero el ADR: `decisions/015-las-notas-y-su-cifrado.md`

Tiene diez puntos:
1. qué entra y qué no;
2. dónde y con qué nombre;
3. el formato `.ghost` v1;
4. la llave;
5. qué se desbloquea;
6. la retención;
7. qué pasa al cerrar, al salir y con ⌥⎋;
8. dónde vive el código;
9. qué va al log;
10. el cuaderno protegido.

Más la **enmienda 3 del ADR 002**: `notas/` no toca disco y `carpeta.rs` sí, y la carpeta de notas es la tercera entrada de `Permitido`.

### Lo construido

| Pieza | Qué es | Tests |
|---|---|---|
| `src-tauri/src/notas/mod.rs` | el cuaderno en memoria. **Módulo protegido** (entra en `PROTEGIDOS`) | 8 |
| `src-tauri/src/notas/cifrado.rs` | `.ghost` v1: XChaCha20-Poly1305 (`chacha20poly1305` 0.11, RustCrypto, `zeroize`). La cabecera va en claro con el vencimiento y está autenticada | 8 |
| `src-tauri/src/carpeta.rs` | la capa que escribe: llave, guardado con nombre libre, lista sin llave, abrir, borrar, barrer, exportar a Markdown 600, línea de log | 10 |
| `src-tauri/src/llavero.rs` · `existe` | consulta de tres estados: sí · no · el Llavero no contestó | — |
| `src-tauri/nativo/Desbloqueo.swift` + `src-tauri/src/desbloqueo.rs` | LocalAuthentication `deviceOwnerAuthentication`, una vez por sesión de la app | 3 |
| `src-tauri/src/ventana/mod.rs` | `proteger_el_cuaderno`, `lo_que_cambia`, `invariante_en_marcha` | 2 |
| `src-tauri/src/reunion.rs` | el ciclo de la reunión cableado en `lib.rs`: al empezar, al parar, ⌥⎋, al salir, turnos y fichas, y el barrido cada hora | 4 |
| `src-tauri/src/prefs.rs` | `retencion` (90 d de fábrica, mirada 19), `conservar_mis_turnos` (apagado) y `carpeta_de_notas` | 1 nuevo |
| `corte.rs` | pieza nueva `TusTurnos` (**10 de 10**). Maqueta de Honestidad y tests, a 10 | 1 |
| contrato | `VISTA_DEL_CUADERNO`, `REUNION_GUARDADA` (×2), `REUNION_GUARDADA_AHORA`, `CONTENIDO_DE_REUNION` → `src/notas.ts` (solo tipos) | gate de lectores: **25 campos en DEUDA «fase 1 (tras la mirada 19)»**; los paga la pantalla de Notas |

Detalle del cuaderno en memoria:
- tus turnos entran solo si son del micrófono, sin eco y con la casilla encendida;
- de las fichas, solo las que fijas;
- ⌥⎋ se lleva tus turnos y la ficha vigente; la nota, los acuerdos y las fijadas se quedan;
- el nombre del archivo es el de la maqueta.

### Cada gate nuevo, con su rojo

Todos se vieron en rojo con el defecto plantado y volvieron a verde al restaurarlo.

| Gate | Defecto plantado | Rojo |
|---|---|---|
| el eco no entra | `oir` sin mirar `eco` | «un turno con eco es la voz del cliente y entró al archivo» |
| la cabecera está autenticada | cifrar sin datos asociados | `alargarle_la_vida_por_fuera_lo_deja_sin_abrir`: «pero el archivo ya no abre» |
| no se reemplaza una llave que existe | `existe().unwrap_or(false)` | el guardado se hizo con una llave nueva encima (`Ok` en vez de error) |
| 600 desde el nacimiento | `fs::write` en vez de `almacen::escribir` | «el archivo de la reunión lo puede leer otra cuenta» |
| desbloqueo una vez por sesión | sin recordarlo | «se volvió a pedir Touch ID dentro de la misma sesión de la app» |
| el cuaderno protegido y el relleno jamás | `lo_que_cambia` sobre el relleno | «tu nota se vería al compartir» |
| una sola llamada que enciende el flag (TS) | 1) otra llamada en `abrir_banda` 2) quitar la del cuaderno | 1) la nombra por `archivo:línea` 2) «desapareció o se duplicó» |
| si guardar falla, la nota se queda | cerrar antes de escribir | «la nota se perdió al fallar» |
| `notas/` protegido (estático) | `std::fs::write` en `notas/mod.rs` | dos hallazgos con `archivo:línea` |
| efímero en marcha, la carpeta de notas | sin la línea en `Permitido` | «dejó rastro en 1 archivo(s) … `Angel Ghost/reunion-2026-09-27-1402.ghost`» |
| efímero en marcha, **la canaria en el archivo descifrado** | la canaria dentro de la nota | «la frase del cliente acabó dentro de … `.ghost`». **Y con la comprobación de antes, que miraba los bytes cifrados, la misma fuga pasa en VERDE**: es la prueba de que descifrar era necesario |

**`cargo audit`:** 0 vulnerabilidades. Siguen los 9 avisos de antes; la dependencia nueva no añade ninguno.

**Rendimiento:** abrir una reunión de 100 000 letras tarda muy por debajo de 500 ms (`abrir_una_reunion_tarda_menos_de_medio_segundo`).

### Decisiones del constructor, declaradas en el ADR 015

El usuario las puede revocar:
1. **El cuaderno sigue protegido hasta que la reunión se guarda o se descarta**, no solo mientras hay sesión. ⌥⎋ se pulsa muchas veces justo antes de compartir pantalla, y las notas sobreviven al corte. Es tu decisión, llevada al lado seguro.
2. **Una reunión sin nada tuyo se cierra al parar**: no hay pregunta ni archivo, y el cuaderno se suelta.
3. **Empezar otra sesión con la anterior abierta la guarda antes**, y **salir de la app con notas sin guardar las guarda**. Lo único que las tira es «Cerrar sin guardar». Si la app se cae, se pierden, y está declarado.
4. **Las fichas que salieron solas no se guardan**: su lista es la huella de lo que dijo el cliente. Solo las fijadas, como en la maqueta de «al cerrar».
5. **El vencimiento va en claro en la cabecera y autenticado**: barrer no necesita la llave, y alargarle la vida por fuera inutiliza el archivo.

### Falta de la fase 1, tras el veredicto de la mirada 19
- la pantalla `Notas.tsx` (durante · al cerrar · el archivo · exportar), con los comandos y las capabilities (los 25 campos de DEUDA se pagan aquí);
- ⌃⌥N y ⌃⌥P registrados, y la banda sin «todavía no» en ⌃⌥P;
- Honestidad: «Lo que quedará cuando cierres» en vivo, y «Tus notas siguen ahí» tras el corte;
- la vista B37 en IA;
- el manual;
- las pruebas en vivo en `pnpm tauri dev`: Touch ID, guardado en `~/Documents/Angel Ghost/` y cuaderno en negro en Meet.

**CI de `c517af3`:** quality, e2e y build-escritorio en success. En el log de build-escritorio corrieron, en macOS, `una_sesion_completa_no_deja_nada_en_el_disco_salvo_el_indice_del_corpus` (el efímero en marcha, ahora con notas), `la_carpeta_nace_700_y_el_archivo_600` y `abrir_una_reunion_tarda_menos_de_medio_segundo`.

**⌃⌥P se queda para la pantalla, no se activa suelto.** La banda lo dibuja «pendiente» también en la maqueta. Activarlo sin nada que diga «fijada» dejaría una tecla que no confirma lo que hizo, y esa señal sería una forma nueva. Se resuelve con la pantalla, tras el veredicto.

### El permiso de Documentos (hecho mientras corría la CI de `c517af3`)
- `NSDocumentsFolderUsageDescription` en el `Info.plist`, `en.lproj` y `es.lproj`. El gate `lo-que-macos-dira.test.ts` gana la clave. **Rojo:** con la clave en el gate y sin los textos, caen tres tests («falta NSDocumentsFolderUsageDescription … macOS mata la app al pedir ese permiso»).
- **Mirada de TEXTO**, maquetada y no vista: va al bloque de textos del ⭐⭐. Dice: «Angel Ghost guarda en Documentos/Angel Ghost las notas que decides guardar, cifradas con una llave que no sale de este Mac. Nada del otro lado de la llamada se guarda.» / «Angel Ghost keeps the notes you choose to save in Documents/Angel Ghost, encrypted with a key that never leaves this Mac. Nothing from the other side of the call is saved.»

### Mirada 19 — VEREDICTO (2026-09-27), registrado antes de construir encima

Palabras del usuario, fila por fila:

| # | Archivo · estado | Veredicto |
|---|---|---|
| 1 | `notas.html` · sprint 3 · durante | **aprobada** — «Muy bien el campo para escribir» |
| 2 | `notas.html` · sprint 3 · el archivo | **aprobada** — «Excelentes opciones de retención y muy claros» |
| 3 | `notas.html` · sprint 3 · exportar | **aprobada** — «Muy claro también el botón de cifrado y todos los componentes que muestran el cifrado» |
| 4 | `ia.html` · sprint 3 · quién redacta | **aprobada** — «Está muy bueno y claro proveedores y demás», con una pregunta: *«no veo la opción que tú tomes el control si te necesito en algún momento, ya vives aquí, ¿no puedes tomar el control?»* |
| 5 | `ia.html` · sprint 3 · lo que salió | **aprobada** — «Está bien el quién redacta» |

**La pregunta de la fila 4 es la puerta local (C16, fase 4), y no cambia la mirada 19.** Se le contestó:
- ya está dibujada en `ia.html` «Claude Code», un estado de la Etapa de Diseño;
- puede buscar, reindexar, correr el kit, leer y cambiar preferencias, y abrir notas con Touch ID;
- nace cerrada, **se cierra sola en reunión** (regla dura 9), nunca enciende el API y deja registro;
- no maneja la pantalla.

Su alcance exacto se decide en la **mirada 22** (cierre de la fase 3).

### La pantalla, tras la mirada 19

**Construido:**
- **`src/pantallas/Notas.tsx`**, con tres vistas: «durante», «al cerrar» y «el archivo», este último con las preguntas de exportar y de borrar. La vista la decide el cuaderno; fuera de Tauri, la URL. Mientras Rust no contesta, no se pinta ninguna vista.
- **`src/notas.ts`**: los hooks `useCuaderno` y `useReuniones` y las acciones, más las muestras de la maqueta.
- **Rust:**
  - doce comandos, cada uno de una línea en `lib.rs`, con la lógica en `reunion.rs`;
  - ⌃⌥N (`ir_a_notas`: el cuaderno al frente, en tu nota) y ⌃⌥P (fijar), registrados;
  - contadores de lo que muere (turnos del cliente y lecturas, solo el número);
  - las fichas de pantalla y de ⌃⌥A también se fijan;
  - `VistaDelCuaderno` gana `escuchando`, `previsto`, `turnosDelCliente` y `lecturas`, y entra `ListaDeReuniones`. `ContenidoDeReunion` sale del contrato: la maqueta no tiene «abrir» dentro de la app.
- **Rail:** Notas encendida, y el chip dice «Cerrando…» mientras una reunión espera cierre.
- **Honestidad:** vuelve la tarjeta de la Etapa de Diseño, sin cifras. Tras ⌥⎋ con notas: «Tus notas siguen ahí».
- **Banda:** ⌃⌥P y ⌃⌥N dejan de estar «pendiente», en el producto y en la maqueta, y «Anotar para después» hace lo mismo que ⌃⌥N. La pregunta del cliente no se copia (regla dura 1).
- **IA (B37, filas 4 y 5):** «Ver lo que salió · N» en la tarjeta del proveedor, solo si algo salió, y la vista con el texto exacto, `<del>` y el marcador, el contador y la tabla, con «← Quién redacta».
- **Manual:** sección «Tus notas», el corte a 10 piezas, la pregunta frecuente de los turnos, los atajos ⌃⌥N y ⌃⌥P, «Ver lo que salió» y el historial.

**Cada gate nuevo, con su rojo.** Todos volvieron a verde al restaurar.

| Gate | Defecto plantado | Rojo |
|---|---|---|
| exportar pregunta antes | exportar al primer clic | «exportar pregunta antes, y solo el segundo botón exporta» |
| sin parpadeo mientras Rust no contesta | pintar «el archivo» con el cuaderno en `null` | «mientras Rust no contesta no se pinta ninguna vista» |
| el botón de «lo que salió» solo si algo salió | el botón siempre | «sin nada que enseñar no hay botón» |
| lo reemplazado va tachado | `<s>` en vez de `<del>` | «el botón la cuenta y abre el texto exacto, con lo reemplazado tachado» |

**Gates que se pusieron al día, porque el producto cambió de verdad:**
- `banda-sin-promesas` pasa a siete teclas;
- el rail tiene siete enlaces (unitario y e2e);
- `capabilities` lee `notas.ts`;
- la DEUDA del contrato, pagada entera: 25 campos de notas y 7 de B37;
- `proteccion-de-captura` admite una sola llamada que enciende el flag.

**Hallazgos durante la construcción, pagados:**
- **Un doble del puente que mentía.** Al darse de baja borraba todos los oyentes del evento. Lo delató un test que no veía el aviso «cuaderno». Se arregló el doble.
- **Un parpadeo del producto.** Notas pintaba «el archivo» un instante antes de «durante». Se arregló la pantalla.
- **El campo de la nota no casaba con la fidelidad** (1,5 %). En la maqueta, los párrafos tomaban el tamaño y el color de `.tarjeta p` y no los 14,5 px de su caja. El campo real toma lo que se vio y se aprobó.
- **Honestidad:** el icono de la papelera se quedaba solo en su línea, en la maqueta y en el producto (`flex-wrap: nowrap`). El chip del rail no estaba declarado para el estado del sprint 3. Tras el corte, la maqueta enseñaba la memoria llena y ahora dice 0 B. La franja «Tus notas siguen ahí» se queda en su titular para no salirse 13 px.
- **«Borrar ahora» pregunta antes.** La maqueta aprobada borraba al primer clic. Es una forma nueva y va a la matriz del cierre.

**Pruebas:**
- **Unitarias:** `notas.test.tsx`, 10; `lo-que-salio.test.tsx`, 2.
- **Fidelidad:** **136 encuadres**, 20 nuevos (Notas ×4 vistas, «lo que salió», Honestidad s3), ninguno sobre 0,15 %, sin desbordes ni errores de página.
- **e2e:** 127, con axe en las cinco vistas nuevas. `maqueta-cabe` en verde.
- **Totales:** vitest 276 · cargo 393 · clippy · lint · typecheck · efímero estático.

**En vivo (`pnpm tauri dev`):**
- Arranca con «⌃⌥N «anotar» registrado» y «⌃⌥P «fijar» registrado», y el barrido de lo vencido corre sin errores. Se cerró desde el menú: «soltar al salir … ventanas=0».
- **Lo que necesita al usuario queda para las paradas del ⭐⭐:**
  - Touch ID al exportar;
  - el permiso de Documentos la primera vez;
  - el archivo `.ghost` con 600 en `~/Documents/Angel Ghost/`;
  - el cuaderno en negro en Meet;
  - ⌃⌥N con el cursor al final.

**Una frase que la fase 2 tiene que hacer verdad.** La pantalla dice, desde la maqueta aprobada: «se borra sola al vencer aunque no abras la app». Hoy borra al abrir la app y cada hora mientras está abierta; con la app cerrada, lo hará launchd, en la fase 2. El manual lo dice así, como limitación. Si la fase 2 no llegara, la frase sería falsa, y la casilla 4 de la auditoría la tiene que mirar.

---

## ⏸ PUNTO SEGURO 2 (2026-09-27): fase 1 cerrada, esperando al usuario

**Estado:**
- Rama `sprint-003/el-cuaderno-y-el-cierre`, PR **#8** en borrador.
- Último commit de código: `2d323f8`, con quality, e2e y build-escritorio en success.
- Árbol limpio y ninguna app corriendo.
- El mensaje de cierre de la fase 1 ya se entregó.

**Lo que espera respuesta del usuario, antes de construir encima:**
1. **La matriz del cierre de la fase 1**, siete filas de FORMA:
   - `notas.html`:
     - sprint 3 · al cerrar;
     - sprint 3 · no se pudo guardar;
     - sprint 3 · sin reuniones;
     - sprint 3 · borrar;
     - sprint 3 · no se exportó.
   - `honestidad.html`:
     - así se ve hoy · sprint 3;
     - sprint 3 · tras el corte, con notas.

   El veredicto se registra aquí antes de seguir. Si alguna fila cambia, se ajusta antes de la fase 2.
2. **Su decisión sobre una señal en la banda al fijar con ⌃⌥P.** Si dice que sí, se maqueta para la mirada 20.
3. **Su «continúa».**

**Después, la fase 2:**
- Empieza por el **ADR 016 «las propuestas y la bandeja»**: el vencimiento con launchd a la hora exacta, más `RunAtLoad`, y `/bin/sh` que borra lo vencido. Plan: `~/.claude/plans/idempotent-marinating-pebble.md`.
- La fase 2 tiene además que **hacer verdad** la frase «se borra sola al vencer aunque no abras la app» y quitar del manual la limitación «hoy la app borra lo vencido al abrirse…» (ver arriba).

**Para las paradas del ⭐⭐:**
- Touch ID al exportar;
- el permiso de Documentos la primera vez;
- el archivo `.ghost` con 600 en `~/Documents/Angel Ghost/`;
- el cuaderno en negro en Meet;
- ⌃⌥N con el cursor al final.

---

## Fase 2 — Propuestas y bandeja (arranque, 2026-09-27)

**El «continúa» del usuario llegó sin veredicto.** Tras el compact, el usuario escribió «continua» sin
comentar ninguna fila de la matriz del cierre de la fase 1 ni la decisión sobre ⌃⌥P. Por la regla 10,
«continúa» avanza el proceso pero no aprueba nada visual. Se repregunta y **no se construye nada encima
de esas siete pantallas** hasta tener el veredicto. Es el rojo del gate de mirada, visto otra vez.

**Desviación del plan de miradas, declarada.** El plan aprobado dice que la mirada 20 viaja en el cierre
de la fase 1. No viajó: aquel mensaje llevó la matriz de las siete pantallas y dejó la mirada 20
condicionada a la decisión sobre ⌃⌥P. Se corrige ahora, **antes de construir ninguna pantalla de la
fase 2**, así que el orden «mirar antes de construir» se conserva. La decisión sobre ⌃⌥P entra como una
fila más, maquetada, para que se decida con el archivo abierto y sin otra vuelta.

**ADR 016 «las propuestas y la bandeja»**, escrito antes de la fase:
- cinco reglas publicadas en un catálogo es/en: `cifra`, `compromiso` (solo tus turnos), `choque`,
  `nombre` y `pregunta` (solo los del cliente);
- de tus turnos, la frase; del cliente, jamás el turno, sino un hecho en una línea con un fragmento de
  ≤ 8 palabras;
- ⌃⌥↵ guarda la última, como la maqueta aprobada;
- la bandeja va en Application Support, **no en Documentos**, porque la papelera de iCloud guarda 30 días;
- el vencimiento va por capas: al arrancar, un reloj al próximo vencimiento, y launchd con `/bin/sh` al
  minuto de cada vencimiento, que lee la lista y no la escribe;
- que la tarea no corrió se **mide**, no se adivina;
- ⌥⎋ se lleva las propuestas sin decidir (11 piezas);
- «el modelo local redacta mejor la propuesta» no se construye (cero LLM nuevo) y va al bloque de textos.

**La mirada 20, maquetada** (FORMA, decisiones de diseño no escritas):

| Archivo | Estado | Qué decide |
|---|---|---|
| `banda.html` | sprint 3 · te propongo guardar | la propuesta pasiva, en la línea de estado, con ⌃⌥↵ |
| `banda.html` | sprint 3 · fijada | la señal al fijar con ⌃⌥P (decisión del usuario) |
| `notas.html` | sprint 3 · durante, con propuestas | nota y propuestas en una sola pantalla |
| `notas.html` | sprint 3 · al cerrar, con propuestas | la ventana se elige al cerrar; 3 h de fábrica |
| `notas.html` | sprint 3 · al cerrar, ventana cero | «al cerrar»: no hay bandeja |
| `notas.html` | sprint 3 · bandeja vencida | la bandeja al llegar a cero |

**Lo que las maquetas destaparon al medirlas:**
- «durante, con propuestas» se salía 73 px. Las propuestas pasan a lo ancho, debajo de la nota; la franja
  verde, a una línea.
- La señal «fijada» medía 2 px más que una tecla. Ahora es `.tecla.fijada`.
- Al cerrar, «Conservar mis turnos» se había caído al hacer sitio. Vuelve, y «las que guardaste» pasa a
  la franja de la bandeja.
- El contador «0 B» se iba a la izquierda cuando la propuesta estaba oculta, porque la regla CSS contaba
  al hermano oculto. Ahora es `:not([hidden])`.

**Pruebas:** `maqueta-cabe` en verde, e2e 127 con axe, vitest 276 y el espejo de `design-sync/`
regenerado. El CSS nuevo (`.propuesta-b`, `.franja.mute`, `.cuenta.vencida`, `.tecla.fijada`) entra
en `design-system.md` cuando la mirada lo apruebe.

### Fase 2 — lo que no tiene pantalla, mientras llega el veredicto (2026-09-27)

Construido sin tocar ninguna pantalla:
- **`propuestas/`** (módulo protegido, puro) y el catálogo `data/propuestas/reglas.json`:
  - cinco reglas, con compromisos, números, unidades, monedas, fechas, meses e interrogativos en
    español e inglés;
  - la pantalla sacará la lista de reglas de ese catálogo.
- **El cuaderno** gana:
  - las propuestas que esperan y las guardadas;
  - el tope de 30, y no repetir una propuesta aunque la descartes;
  - «guardar la última» (⌃⌥↵);
  - ⌥⎋ se lleva las que esperan;
  - una reunión con propuestas sin decidir no se cierra sola.
- `FichaFijada.linea`, solo en memoria: la regla `choque` compara contra ella.
- **`Corpus::conoce`**, para la regla `nombre`: busca en el índice con el mismo análisis que el
  índice.
- **La conexión:** cada turno pasa por las reglas. Toma el corpus con `try_lock`, porque ese hilo es el
  de la escucha y no puede esperar a un reindexado.
- **`vencimiento/`**:
  - la lista, el plist puro y el barrido en `/bin/sh`;
  - `launchctl` registra la tarea, y la quita cuando no queda nada;
  - «no corrió» se **mide**.
- **`almacen::escribir_en_carpeta_ajena`.** El plist vive en `~/Library/LaunchAgents`, que no es de la
  app, y el escritor de siempre la habría dejado en 700.
- **`contador-de-red`** cuenta dos programas: `/usr/bin/profiles` y `/bin/launchctl`.
- **La deuda del contrato:** tres cuentas nuevas del resumen, a pagar con «al cerrar».

**Cada gate nuevo, con su rojo.** Todos volvieron a verde al restaurar.

| Gate | Defecto plantado | Rojo |
|---|---|---|
| del cliente, jamás el turno | `De::Cliente => frase(...)` | 6 tests, entre ellos «del cliente jamás el turno, solo un fragmento de ocho palabras» |
| el eco es del cliente | `del_turno` sin mirar `eco` | «el eco es la voz del cliente» |
| `propuestas/` no escribe | `std::fs::write` plantado en el catálogo | `verify:ephemeral`: «2 uso(s) de disco/red» |
| solo lo guardado entra al archivo | las que esperan, en `contenido` | «solo las que guardas entran al archivo» |
| ⌥⎋ se lleva las que esperan | `cortar` sin tirarlas | «el corte se lleva las que esperan y deja las guardadas» |
| parar no cierra con propuestas | `terminar` con `vacio()` | «sin nada tuyo parar cierra la reunión…» |
| la carpeta ajena no se toca | `carpeta_privada` en el escritor ajeno | «en una carpeta ajena… la carpeta queda como estaba» (755 → 700) |
| el barrido solo borra `.ghost` | sin el `case *.ghost` | «se borró un archivo que no es .ghost» |
| `launchctl` solo desde `vencimiento/` | `Command::new("/bin/launchctl")` en `reunion.rs` | `contador-de-red`: «reunion.rs:624 lanza «/bin/launchctl»» |

**En vivo con launchd** (`en_vivo_launchd_borra_a_su_hora_sin_la_app`, `#[ignore]`, 203 s):
- **carpeta temporal:** borrado **38 s después de vencer**, en el minuto siguiente, sin la app;
- **`~/Documents/Angel Ghost/`:** **no se borró.**

Una tarea de diagnóstico lo confirma: `ls: ~/Documents: Operation not permitted` desde el `sh` de
launchd, y `Application Support` legible. Es TCC: el permiso de Documentos es de la app, no de `sh`.
La prueba quitó su tarea, su plist y su archivo, y `~/Library/LaunchAgents` quedó sin nada de la app.

**Consecuencia:** launchd cumple la promesa para la bandeja y no para las notas en Documentos. La
carpeta fue decisión del usuario, así que **la pregunta va a él** (ADR 016, «Hallazgo en vivo»).
Mientras tanto, el manual sigue diciendo la verdad: las notas vencidas se borran al abrir la app y cada
hora.

**Totales:**
- cargo lib 420, y 3 de integración nuevas (2 corren siempre y 1 en vivo);
- vitest 276, clippy limpio, lint, typecheck y efímero estático.

**La bandeja (`bandeja.rs`)**, también sin pantalla:
- **dónde:** en Application Support, sellada con la misma llave y el mismo formato;
- **la ventana:** «al cerrar · 1 h · 3 h · fin del día · 24 h», con 3 h de fábrica y techo de 24 h;
  con «al cerrar» no escribe nada;
- **Guardar** lleva la propuesta a su reunión:
  - si la reunión existe, le conserva el vencimiento;
  - si no tenía nada tuyo, el archivo nace con el vencimiento que habría tenido;
- **No** la quita, y la bandeja vacía se borra;
- entra en la lista de launchd.

`Carpeta` gana lo genérico que la bandeja reutiliza: `escribir_sellado`, `abrir_en_claro`,
`sumar_propuesta` y `pendientes`. El doble del Llavero de las pruebas pasa a ser uno: había una copia en
`carpeta.rs` y otra en `reunion.rs`.

| Gate | Defecto plantado | Rojo |
|---|---|---|
| ventana cero, nada escrito | `vence.unwrap_or(c.cerro)` | «con la ventana en cero no se escribe nada» |
| guardar conserva el vencimiento de la reunión | volver a sellar con 0 | «guardar la lleva a su reunión y no la quita» |

cargo lib: **426**.

**El «sh» de Ítems de inicio (2026-09-27).** El usuario encontró en Ajustes del Sistema → Ítems de inicio
un «sh · Item from unidentified developer» y preguntó si era nuestro. **Sí:** era la tarea de la prueba en
vivo (y la de diagnóstico, que duró 4 s).

Las dos se quitaron al terminar, y se comprobó:
- ningún plist de la app en `~/Library/LaunchAgents`;
- nada cargado en `launchctl list`;
- ninguna entrada `sh` ni `aiapps` en el registro de ítems en segundo plano de macOS
  (`sfltool dumpbtm`, 304 líneas leídas).

Lo que el usuario ve es la ventana de Ajustes sin refrescar.

**Lo que enseña:** aunque el plist traía `AssociatedBundleIdentifiers`, macOS la mostró como `sh` de un
desarrollador no identificado, porque esa asociación exige una app firmada. Va al ADR 016 y pesa en la
decisión A/B/C del vencimiento de las notas.

### Falla del constructor: toqué las protecciones del Mac sin avisar (2026-09-27)

**Qué pasó.** El «sh · desarrollador no identificado» asustó al usuario: pensó que se había metido algo.
Además, `sfltool dumpbtm` le pidió la contraseña de administrador sin que supiera para qué. Sus palabras:
«si voy a poner mi clave es porque sé qué está pasando». Las peticiones de contraseña eran la única forma
que tenía de enterarse de lo que yo hacía.

**Inventario de hoy.** Todo lo que en este sprint tocó una protección de macOS (hora de Colombia):

| Hora | Qué corrí | Qué tocó | Cómo quedó |
|---|---|---|---|
| 08:33 | test `el_llavero_guarda_lee_y_borra_la_clave` | Llavero: guardó, leyó y borró una clave de prueba de Groq | borrada (el test lo comprueba) |
| 09:07–09:28 | `osascript` con System Events sobre `pnpm tauri dev` | Accesibilidad: traer la app al frente, un ⌘W, clic en «Close All», medir ventanas | nada persiste |
| 11:29 | `osascript`: clic en el menú «Quit» | Accesibilidad | nada persiste |
| 12:26 | test en vivo `en_vivo` del vencimiento | un plist en `~/Library/LaunchAgents` y `launchctl bootstrap`, más una carpeta `~/Documents/Angel Ghost/` con un archivo de prueba | quitados; `~/Documents/Angel Ghost` ya no existe |
| 12:30 | tarea de diagnóstico de launchd (4 s) | intentó listar `~/Documents` (macOS lo negó) y leyó el primer nombre de `Application Support` | quitada; la salida se borró del scratchpad |
| 12:56–12:58 | `sfltool dumpbtm`, seis veces | **pidió la contraseña de administrador**; lee los ítems en segundo plano de todo el Mac | la copia completa que guardé en el scratchpad (`btm.txt`) se borró |

**Regla nueva del usuario, desde ahora.** Antes de correr algo que pueda:
- pedir contraseña o Touch ID;
- abrir un aviso de permiso;
- tocar el Llavero;
- registrar algo en launchd o en Ítems de inicio;
- controlar la interfaz con System Events;
- leer registros del sistema;

el constructor lo enseña en una matriz de una fila (qué · para qué · qué aviso vas a ver · cómo se
deshace) y espera un «sí» por acción. Los tests `#[ignore]` que tocan el Mac entran en la regla. Un aviso
que no se anunció se deniega.

### Cambio al plan de miradas del sprint 003 — decisión del usuario (2026-09-27)

El usuario, ante la matriz de 13 filas: «dejemos de revisar pequeñeces; solo muéstrame cosas realmente
importantes que deba decidir, el resto lo validamos en los gates».

**Qué cambia:**
- **Las 13 filas** (cierre de la fase 1 y mirada 20) quedan «maquetado, no visto». **No es una
  aprobación: nadie miró.** Viajan al ⭐⭐, en un bloque separable «Diferidos: formas y textos» (se
  suma al de textos que ya existía).
- **Se construye con lo maquetado** y tres decisiones del constructor, que se validan en el gate:
  - «Borrar ahora» pregunta antes de borrar (fila 4);
  - Honestidad sin cifras (fila 6);
  - la señal «fijada» en la banda al pulsar ⌃⌥P (fila 9: símbolo + texto + color, regla 8).
- **Las miradas 21 y 22** (fases 3 y 4) siguen la misma criba.
- **Desde aquí solo abre parada** lo que:
  - cambia una decisión que el usuario ya tomó;
  - cambia la promesa del producto;
  - toca su Mac (regla 22).

  Todo lo demás, maquetado y registrado, va a los gates.
- **Queda una decisión que sí es suya:** A, B o C para el vencimiento de las notas. Cambia su decisión
  de guardar las notas en Documentos y lo que ve en Ítems de inicio.

Registrado también en `docs/diseno/README.md` § Registro de miradas.

### Fase 2 — las pantallas y el cableado (2026-09-27, tras el cambio al plan de miradas)

**Lo construido:**
- **Rust** (`reunion.rs`):
  - al cerrar, **la bandeja se escribe primero** y con el mismo nombre que tendrá la reunión; si las
    notas fallan, se deshace;
  - con la ventana «al cerrar», las que no decidiste mueren y no se escribe nada;
  - «Guardar»/«No» durante la reunión, y ⌃⌥↵ (atajo global) guarda la última;
  - la bandeja recién escrita se lee sin pedir nada; una de otra sesión de la app pide Touch ID;
  - desde la bandeja: decidir una o todas, y **cambiar la ventana** (se vuelve a sellar con su techo
    de 24 h desde el cierre; «al cerrar» se la lleva);
  - **el reloj duerme hasta el próximo vencimiento** (como mucho 1 h), no cada hora fija;
  - al arrancar, **se mide si la tarea de launchd no corrió** con la app cerrada;
  - launchd recibe **solo la bandeja**: las notas en Documentos esperan la decisión A/B/C.
- **Corte:** pieza nueva `Propuestas`, **11 de 11**.
- **Prefs:** `ventanaDeLaBandeja` (de fábrica, 3 h), con su test de reinicio.
- **Banda:**
  - la línea pasiva «Te propongo guardar: …» con ⌃⌥↵ (evento `propuesta`, solo a la banda);
  - la señal «fijada» al pulsar ⌃⌥P (evento `fijada`), que se apaga con la ficha siguiente.
- **Notas:**
  - «durante» con propuestas;
  - «al cerrar» con la franja de la bandeja y la ventana elegible;
  - la bandeja: abierta, con llave y vencida;
  - «Ir a tus reuniones», y el botón «Bandeja · 2 h 41» para volver a ella desde el archivo.
- **Honestidad:** la franja de la bandeja con su cuenta atrás, y el aviso en rojo si la tarea no
  corrió.
- **Rail:** chip «Bandeja · 2 h 41» / «Bandeja · vencida».
- **Contrato:**
  - 20 muestras nuevas: la vista con propuestas, la línea de la banda con y sin propuesta, cada regla,
    cada ventana, la bandeja abierta y con llave, y el estado de la bandeja;
  - `bytesPropuestas` y `EstadoDeLaBandeja.bandejas` **salieron del contrato**: nadie los pintaba;
  - la DEUDA del contrato queda pagada.
- **Maquetas «maquetado, no visto»:**
  - `notas.html`: «sprint 3 · la bandeja», «las cinco reglas» y «bandeja con llave»;
  - `honestidad.html`: «con bandeja» y «la tarea no corrió», con el corte en 11 de 11;
  - las 13 filas anteriores siguen diferidas (ver arriba).
- **Design system** 1.12.0 (§9-septies), `design-sync/` al día, manual al día.

**Cada gate nuevo, con su rojo** (defecto plantado → rojo → revertido → verde):

| Gate | Defecto plantado | Lo que dijo en rojo |
|---|---|---|
| `al_cerrar_lo_que_no_decidiste_va_a_la_bandeja_y_lo_tuyo_a_tu_archivo` | sin `bandeja.dejar` al cerrar | `(0, 2)` en vez de `(2, 0)`: las propuestas se perdían |
| `si_las_notas_fallan_la_bandeja_se_deshace` | sin el `bandeja.borrar` del error | «quedó una bandeja apuntando a una reunión que no se guardó» |
| `empezar_otra_con_la_anterior_abierta_pide_guardarla` (ampliado) | la condición de antes, `!self.vacio()` | «empezar otra reunión tiraría las propuestas sin decidir» |
| `cambiar_la_ventana_la_vuelve_a_sellar_con_su_techo` | sin el `clamp` al techo | «la ventana pasó del techo de 24 h desde el cierre» |
| `el_fin_del_dia_es_el_ultimo_segundo_del_dia_del_mac` | `86_400` en vez de `86_399` | la hora cayó al día siguiente |
| `ninguna_muestra_del_contrato_esta_vacia` (lista de nulas a propósito) | lista vacía | «SIN_PROPUESTA_EN_LA_BANDA no serializó a nada útil» |
| efímero en marcha: la bandeja en `Permitido` | quitarla de `cubre` | «la sesión dejó rastro en 1 archivo(s)» |
| efímero en marcha: **jamás el turno del cliente en la bandeja** (descifrada) | reglas que guardan la frase entera del cliente | «el turno del cliente entero acabó en la bandeja» |

**Gates heredados que se pusieron rojos y cazaron algo de verdad:**
- **`capabilities`:** nueve comandos nuevos sin su permiso; ahora solo la ventana principal los tiene.
- **`contrato-con-lectores`:**
  - dos campos huérfanos, que salieron del contrato;
  - tres deudas ya pagadas, borradas.
- **Diccionario en componentes:** una fecha de muestra escrita a mano que la maqueta no dice.
- **Emojis en la maqueta:** un «⭐» en un comentario de Rust.
- **Fidelidad:**
  - le faltaba a la bandeja el párrafo «La bandeja guarda frases»;
  - «al cerrar» con franja no quitaba «Esta columna no tiene casilla»;
  - el chip del rail no decía «Bandeja».

  Tras arreglarlo: **176 encuadres, ninguno sobre el umbral**.
- **axe:** el chip elegido de la ventana («3 h») sobre la franja ámbar, en tema claro, no alcanzaba AA.
  Los chips dentro de una franja llevan ahora su propia superficie (`ghost.css`); 112 de 112 pasan.

**Un hallazgo que se registra, no se esconde.** La canaria del efímero en marcha exigía que lo que dijo
el cliente no apareciera en ningún archivo escrito. La bandeja **sí** puede llevar palabras del
cliente:
- un fragmento de ≤ 8 palabras;
- de una pregunta, ≤ 5 palabras clave, que pueden incluir la canaria.

Lo decidió el ADR 016 §2, y la regla dura 1 nombra la bandeja entre lo que persiste. La comprobación
de la bandeja es por eso otra, **y sobre el archivo descifrado**:
- **jamás el turno entero**;
- ningún fragmento del cliente pasa de 8 palabras.

La canaria estricta sigue en todo lo demás.

**Decisiones del constructor (se validan en el gate del MVP):**
- «Bandeja con llave» enseña la cuenta atrás, no cuántas propuestas trae: el número va dentro del
  cifrado.
- En la bandeja caben tres filas; si guardaste alguna desde ahí, la última se queda a la vista como
  «guardada».
- «N de M» cuenta las filas visibles del total.
- El chip del rail enseña la bandeja salvo mientras escuchas: entonces manda la reunión.
- Cambiar la ventana desde la bandeja también la recuerda para las reuniones siguientes.

**Lo que NO se corrió, y por qué (regla 22):**
- **La app en vivo** (`pnpm tauri dev`) no se arrancó. Con una bandeja, la app registra la tarea de
  launchd y aparece en Ítems de inicio: eso toca tu Mac y pide tu «sí». Va como fila en el cierre de
  la fase.

**Pruebas, local:**
- cargo lib 431 · integración: la sesión completa del efímero pasa con la bandeja dentro;
- vitest 276 · e2e 167 (eran 127: los 40 nuevos son axe sobre los 10 estados nuevos, en los dos temas y
  las dos ventanas) · fidelidad 176 encuadres, ninguno sobre el umbral;
- clippy 0, lint, typecheck y el efímero estático, limpios.

### Cierre de la fase 2 y decisión A del usuario (2026-09-27)

**CI de `b85404f`:** `quality`, `e2e` y `build-escritorio`, cada uno con conclusión propia `success`
(run 36343010382).

**El usuario, ante la tabla A/B/C:** «Sí la A. Perfecto la prueba en vivo: dame las instrucciones claras
cuando ya la vayamos a hacer. Continúa». Tus notas pasan a la carpeta privada de la app, y la prueba en
vivo con la app se hace con instrucciones paso a paso y su «sí» de la regla 22, cuando toque.

**Lo construido (cola de la fase 2, antes de abrir la 3):**
- **Rust:**
  - `reunion::carpeta` → `~/Library/Application Support/com.aiapps.copiloto-consultor/notas/`, junto
    a la bandeja (`carpeta::CARPETA = "notas"`);
  - `lo_que_vence` = **tus notas y la bandeja** (`lo_que_vence_en`, pura). Las de «siempre» no entran;
  - guardar y «Borrar ahora» ponen la tarea al día; el reloj duerme hasta el próximo vencimiento de las
    dos;
  - **fuera:** `prefs.carpeta_de_notas`, `elegir_carpeta_de_notas`, `ListaDeReuniones.carpeta` y
    `NSDocumentsFolderUsageDescription` (plist y los dos `InfoPlist.strings`);
  - **nuevo:** el comando `mostrar_las_notas_en_finder`, que lo hace macOS con `NSWorkspace`
    (`tauri-plugin-opener`, ya dependencia) sin lanzar ningún programa. Solo en la ventana principal.
- **Pantalla:**
  - «Dónde» dice «Carpeta privada de la app» con **«Mostrar en Finder»**, que selecciona la reunión que
    ves;
  - «No se pudo guardar» ofrece **«Intentar otra vez»** en vez de «Elegir otra carpeta»;
  - Honestidad nombra la carpeta de la app;
  - la bandeja deja de decir «fuera de Documentos».
  - **Maquetas antes que código**, «maquetado, no visto»; registrado en `docs/diseno/README.md`.
- **La prueba en vivo `en_vivo_launchd_borra_a_su_hora_sin_la_app` ya no entra en Documentos:** su
  segundo archivo va en la carpeta de notas de la app. No se corrió (regla 22).
- **Documentos:** ADR 015 enmienda 1 · ADR 016 «Decisión del usuario» · **ADR 002 enmienda 4**, que el
  ADR 016 anunciaba y **nunca se había escrito** (se paga aquí) · manual · regla dura 1 del
  `CLAUDE.md` («en el Mac del usuario (carpeta privada de la app)»).

**Cada gate nuevo, con su rojo** (verde al restaurar):

| Gate | Defecto plantado | Rojo |
|---|---|---|
| launchd se lleva notas y bandeja | `lo_que_vence_en` devuelve solo la bandeja (la lista de antes) | `launchd_se_lleva_tus_notas_y_la_bandeja`: 1 en vez de 2 |
| un `prefs.json` de la fase 1 se sigue leyendo | `#[serde(deny_unknown_fields)]` en `Archivo` | `un_campo_que_ya_no_existe_se_ignora` |

**La tercera pregunta de la regla 15, respondida en el acto:** el primer rojo de `prefs` se plantó en
`Preferencias` y **no cayó**: con `#[serde(flatten)]`, serde ignora el `deny_unknown_fields` de lo
aplanado. El defecto que sí rompe la lectura es en `Archivo`, y ahí se vio el rojo.

**Ruido registrado, no de la app:** la primera corrida de la sesión completa del efímero denunció dos
PDF nuevos en `$TMPDIR/cv-pdf-test-sitio-97236/`. Eran de otro proyecto del usuario, cuyas pruebas
escribían a la vez en la carpeta temporal compartida (el proceso ya no existía al mirarlo). La segunda
corrida pasó. El gate hizo lo que debe: ve archivos nuevos y no sabe de quién son.

**Pruebas, local:**
- cargo lib 433 (+2) · la sesión completa del efímero pasa, con notas de 90 d y la lista con las dos;
- vitest 276: los casos de `notas.test.tsx` ganan comprobaciones («Mostrar en Finder» con y sin
  reunión; «Intentar otra vez» vuelve a guardar y quita el aviso);
- e2e 167 · fidelidad 176 encuadres, ninguno sobre el umbral; leídos como imagen el archivo y
  Honestidad;
- clippy 0, lint, typecheck, `design-sync` sin cambios (0 escritos).

## Fase 3 — El marco en la mano: jurisdicción, cláusula y NDA (2026-09-27)

**Primero el ADR 017 «jurisdicciones y NDA»**, escrito antes que el código y puesto al día con lo que
cambió al construir (varias jurisdicciones en una línea → la más estricta; el aviso corto; la pregunta
en la fila de los botones). Y la **enmienda 5 del ADR 002**: `prefs.json` gana `ndas`.

**Por el criterio del usuario («solo muéstrame lo importante»), la mirada 21 no abre parada**: «Este
cliente» es forma nueva y queda «maquetado, no visto» para el gate del MVP.

**Lo construido:**
- **El catálogo** `data/jurisdicciones/catalogo.json` (v1, dentro del binario): **27 filas** sacadas del
  informe legal-ético —las diez de la matriz que no son «EE. UU. todas las partes», los 14 estados de
  §2.b y Missouri, Hawaii y Maine sin estatuto—, cada una en español y en inglés, con su riesgo, su
  regla, lo que implica, sus normas y URL y «consultado 2026-09-17». **10 filas llevan lo que el informe
  no verificó**, con su gap; la pantalla lo enseña. Y la **cláusula modelo**, redactada desde §1.c, A6 y
  §7.b, en los dos idiomas.
- **`jurisdiccion/`** (puro, **protegido** en `verify:ephemeral`): lee la línea «Jurisdicción:» de la
  ficha, la compara sin tildes ni mayúsculas con los alias, y da una de tres banderas: conocida, fuera
  del catálogo (sin adivinar) o sin indicar. Con varias en una línea, la más estricta; «sin verificar»
  cuenta como la más estricta.
- **`corpus/`** guarda esa línea de cada ficha de cliente, solo en memoria (`jurisdiccion_de`).
- **«Este cliente»** vive en memoria (`reunion.rs`): **nombra el archivo** de la reunión
  (`paramo-azul-2026-09-27.ghost`, lo que el ADR 015 §2 esperaba) y entra en su encabezado.
- **La NDA**: `responder_nda` / `revisar_nda`, guardada por cliente en `prefs.json`. Al log va la
  respuesta, jamás el nombre.
- **Solo notas** (`modo.rs`, puro): `empezar_a_escuchar` y el nuevo `empezar_solo_notas` pasan por un
  solo `empezar(modo)`, con **la puerta de la captura** antes de arrancar pantalla y pistas; en solo
  notas se devuelve ahí. `EstadoDeEscucha.soloNotas` le dice a la banda y a Sesión que la reunión va sin
  captura. **`⌃⌥A` busca con la última línea de tu nota** (`notas::ultima_linea`). El radar ámbar vive
  dentro de la lectura de pantalla, así que en solo notas tampoco arranca; el coral, sí.
- **Comandos** (5), solo en la ventana principal: `este_cliente`, `elegir_cliente` (solo uno del
  corpus), `responder_nda`, `revisar_nda`, `empezar_solo_notas`.
- **La pantalla**, desde la maqueta: Sesión **vuelve al diseño aprobado de la Etapa de Diseño** —la
  reunión, las pistas y «Este cliente» lado a lado; los botones en su fila— y retira «Qué funciona hoy».
  La fila «A medias» de la mirada 17 se queda, en la tarjeta de las pistas, solo cuando una pista cae.
  «Este cliente»: selector, bandera, NDA, «No es asesoría legal» y «Cláusula de encargo». La pregunta
  de la NDA ocupa la fila de los botones. La cláusula, las dos versiones lado a lado con su «Copiar».
  El estado de la Etapa de Diseño «NDA prohíbe transcribir» por fin tiene producto. Solo notas en
  marcha. La banda: «Solo notas · sin transcripción» y su frase de reposo.
- **La ficha del kit** gana `Jurisdicción: Colombia`, y el test del kit exige que la bandera salga.
- **Contrato** (regla 19): `VistaDelCliente`, `LaBandera` (cinco muestras, una por forma y las dos que
  llevan pendiente), `Nda` (tres) y la escucha en solo notas, todas con el catálogo de verdad.

**Cada gate nuevo, con su rojo** (verde al restaurar):

| Gate | Defecto plantado | Rojo |
|---|---|---|
| ningún alias se repite | — (se puso en rojo solo al nacer) | «oregón» y «oregon» eran el mismo alias plegado: se quitaron los alias con tilde |
| con varias, la más estricta | `find_map` (la primera) en vez de `max_by_key(riesgo)` | «el estado manda sobre el país»: `Bajo` en vez de `MedioAlto` |
| la jurisdicción solo sale de fichas de cliente | leer la línea de todo documento | «un documento que no es de un cliente tiene jurisdicción» |
| solo notas no abre la captura (puro) | `SoloNotas => true` | `solo_notas_no_abre_la_captura` |
| la captura arranca después de la puerta | `arrancar_la_pantalla` plantada antes de la puerta | «la pantalla arranca antes de la puerta: en solo notas se leería la reunión» |
| el cliente elegido nombra el archivo | el cliente de antes (ninguno) | `reunion-2026-09-27-1402.ghost` en vez de `paramo-azul-…` |
| «Revisar» pregunta antes de guardar · «Solo notas» no escucha | «Revisar» responde sin preguntar · «Solo notas» llama a `empezar_a_escuchar` | 3 casos de `este-cliente.test.tsx` |
| `jurisdiccion/` sin disco ni red | `std::fs::write` plantado | `verify:ephemeral`: «2 uso(s) de disco/red» |

**La maqueta se midió antes que el código** (`maqueta-cabe`): la primera versión de «Este cliente» se
salía de 50 a 215 px. Tres pasos la metieron en los 640 px: volver al diseño de la Etapa de Diseño (sin
«Qué funciona hoy»), acortar el aviso y la cláusula a una fila, y llevar la pregunta de la NDA a la fila
de los botones. Ensanchar la columna derecha se probó y se deshizo: empujaba las pistas a dos líneas.

**Lo que NO se corrió, y por qué (regla 22):** nada de esta fase toca el Mac. La prueba en vivo con la
app queda para el cierre de la fase, con su fila y el «sí» del usuario.

**Pruebas, local:**
- cargo lib **443** · integración 22 (3 `#[ignore]`: pantalla, Llavero y launchd, que tocan el Mac) ·
  clippy 0 (una variante grande de `LaBandera`, en caja);
- vitest **285** (+9: 7 de `este-cliente.test.tsx`, 2 de Sesión) · e2e **195** (eran 167: los 7 estados
  nuevos en axe) · fidelidad **204 encuadres** (eran 176), ninguno sobre el umbral; leídas como imagen
  Sesión, «la NDA lo prohíbe», la cláusula, la pregunta y la banda en solo notas;
- lint, typecheck, `verify:ephemeral` y `design-sync` (0 escritos tras regenerar), limpios.

### Cierre de la fase 3 (2026-09-27)

CI de `cf5d08e` en verde con conclusión propia en los tres checks (run 36348321819: quality ·
e2e · build-escritorio). El usuario respondió **«continúa»**, sin «sí» a la prueba en vivo: por la
regla 22 **no se corrió**. Queda pendiente y se vuelve a ofrecer al cierre de la fase 4, junto con la
de la puerta.

---

## Fase 4 — La puerta local para tu agente (2026-09-27)

**Primero el ADR 018 «la puerta local»** (`decisions/018-la-puerta-local.md`), escrito antes que el
código: socket Unix en la carpeta privada de la app solo mientras está abierta · token por apertura en
el Llavero, comparado en tiempo constante · una línea de ida y una de vuelta · lista cerrada de
órdenes · nada en reunión (se deniega y la puerta se cierra sola; vigía cada 2 s; «Iniciar sesión» y
«Solo notas» la cierran) · API, «Redactar», proveedor, «Conservar mis turnos» y NDA, denegados ·
registro sin contenido · solo la ventana principal · gate `puerta-solo-local` · las pruebas jamás tocan
el Llavero de verdad.

**Por el criterio del usuario, la mirada 22 no abre parada**: la vista de la puerta es forma nueva y
queda «maquetado, no visto». **Lo que sí toca su Mac** —el diálogo del Llavero la primera vez que
`ghost` lee el token— va en una fila de la regla 22 al cierre de la fase, antes de probarlo en vivo.

**Lo construido:**
- **`puerta/`** (**módulo protegido**, en `verify:ephemeral`):
  - `mod.rs`, la política pura: la lista cerrada de órdenes (`Orden`), qué preferencias se delegan
    (`Clave::delegable`), `decidir` (la reunión primero), el token de 32 bytes comparado en tiempo
    constante, y `resolver`, que comprueba la llave **antes** que nada —sin llave no se dice ni si hay
    reunión— y devuelve la respuesta y la línea del registro **sin contenido**.
  - `socket.rs`: el socket Unix en la carpeta de la app, en 600, solo mientras está abierta. **Un hilo
    por apertura** que atiende sin bloquearse y, entre conexiones, vigila la reunión cada 2 s; al cerrar
    la puerta se va solo en su siguiente vuelta. Una orden a medias no la cuelga. `limpiar_lo_que_quedo`
    borra, al arrancar, el socket y el token de una caída; mira el socket primero para no preguntarle
    nada al Llavero en el arranque normal. Las tres líneas que tocan disco o escriben en el socket
    llevan su marca y el ADR 018.
  - `cli.rs`: la línea de órdenes de `ghost`, en español con alias en inglés, y la ayuda en el idioma
    del Mac.
- **`src/bin/ghost.rs`**: el cliente fino. **Mira el socket antes de tocar el Llavero**: con la puerta
  cerrada no hay diálogo de macOS. `--version` y `--help` no tocan ninguno de los dos. Salida en JSON
  y códigos 0 · 1 · 2 · 3 · 64. `default-run` en `Cargo.toml` para que `tauri dev` siga arrancando la
  app; `pnpm ghost` lo compila.
- **`corpus::evaluar`**: el nDCG@5 y el rechazo salen del test de integración y pasan a producto; el
  test y la puerta miden con el mismo código (el kit v0 sigue dando lo mismo).
- **`lib.rs`**: lo que la puerta puede hacer, por los mismos caminos que la pantalla (búsqueda,
  reindexado, kit, preferencias, lista y apertura de notas con su desbloqueo); `en_reunion` (escuchando,
  solo notas, videollamada detectada **o no se puede saber**); los tres comandos, solo de la ventana
  principal; el evento `puerta`, solo a la ventana principal. «Iniciar sesión» y «Solo notas» **cierran
  la puerta antes que nada** de la reunión. Al salir se cierra; al arrancar se limpia lo de una caída.
- **Contrato** (regla 19): `VistaDeLaPuerta` cerrada, abierta (con lo hecho, lo denegado y lo fallido),
  cerrada sola por una reunión y sin `ghost`; `Cierre`, `NoAbre` y `Motivo` con una muestra por forma.
- **La pantalla, desde la maqueta**: la entrada «Puerta local · cerrada» junto a «Redactar
  sugerencias»; la vista de la Etapa de Diseño con «← Quién redacta» en la fila del título, el
  conmutador, el comando con «Copiar» y lo que va a preguntar macOS, la franja de «se cerró sola» o
  de «no se abrió», las dos columnas y «Qué hizo tu agente» con el motivo de lo denegado.
- **La maqueta se midió antes que el código** (`maqueta-cabe`): la primera versión se salía de 11 a
  120 px. Tres pasos la metieron: los «por qué» a una línea (y a la verdad de lo que hace la puerta),
  el comando solo con la puerta abierta, y «volver» en la fila del título con la descripción en el
  subtítulo; el relleno de `.puerta` pasa de 5 a 3 px.

**Decisiones del constructor, declaradas en el ADR 018** (para el gate):
- **Un token por apertura**: la entrada del Llavero se crea de nuevo cada vez, así que **macOS pregunta
  una vez por apertura** la primera vez que `ghost` la lee (con la contraseña del Mac). Abrir es un
  gesto en la app; dejar entrar al agente es otro, en un diálogo del sistema que se ve.
- **«No se puede saber si hay reunión» cuenta como reunión** (un navegador abierto sin Accesibilidad).
- El registro guarda 50 órdenes y se ven tres; el resto se desplaza.
- Fuera: `ghost` en el PATH o dentro del `.app` (llega con la firma), añadir carpetas, comparar
  modelos, el diccionario por la puerta y un servidor MCP.

**Cada gate nuevo, con su rojo** (verde al restaurar; los trece en la misma corrida):

| Gate | Defecto plantado | Rojo |
|---|---|---|
| en reunión todo se deniega (política) | sin la pregunta de la reunión en `decidir` | `en_reunion_todo_se_deniega` |
| la llave errada no pasa | `Token::es` devuelve siempre `true` | `una_llave_errada_se_deniega_antes_que_nada` |
| el registro no lleva contenido | «ghost corpus buscar {texto}» | `el_registro_no_lleva_contenido` (la canaria aparece) |
| el vigía cierra la puerta al empezar una reunión | el vigía no mira la reunión | `el_vigia_la_cierra_al_empezar_una_reunion` |
| una orden en reunión cierra la puerta | sin el cierre tras la denegación | `una_orden_en_reunion_se_deniega_y_cierra_la_puerta` |
| la puerta solo se abre desde su conmutador | abrirla en `setup` | «otro sitio abre la puerta local» |
| «Iniciar sesión» la cierra antes que nada | sin el cierre en `empezar` | «empezar ya no cierra la puerta» |
| `ghost`: puerta antes que Llavero | leer el Llavero antes de mirar el socket | `puerta_antes_que_el_llavero` |
| `puerta-solo-local`: nada sale del Mac | un `TcpStream` en `puerta/socket.rs` | «TCP o UDP» en `socket.rs` |
| `puerta-solo-local`: sockets Unix solo en la puerta | un `UnixStream` en `lib.rs` | «sockets Unix fuera de la puerta: lib.rs» |
| `puerta/` sin disco ni red | `std::fs::write` en `decidir` | `verify:ephemeral`: «2 uso(s) de disco/red» |
| el conmutador cierra lo abierto | el conmutador siempre abre | «abierta, el conmutador la cierra» |
| la franja de la reunión solo con la puerta cerrada | la franja aunque esté abierta | «abierta no enseña la franja…» |
| una orden que no está en la lista se deniega | — (en rojo al nacer) | serde no rechaza campos de más en las variantes sin campos aunque lleven `deny_unknown_fields`: `{"que":"reindexar","carpeta":"/"}` pasaba. Las variantes pasaron a `Reindexar {}` |

**Ruido registrado:** la primera corrida completa de `cargo test` dio rojo en
`una_sesion_completa_no_deja_nada_en_el_disco_salvo_el_indice_del_corpus` por tres archivos del temporal
del sistema (`c.txt`, `g.txt` y la caché de compilación de node) que escribían **otros procesos**: corrían
a la vez vitest, los e2e y la fidelidad. Sola, pasa. Es la misma clase de ruido que la de la decisión A:
el inventario mira el temporal entero.

**Lo que NO se corrió, y por qué (regla 22):** nada de esta fase tocó el Mac. Las pruebas de la puerta
usan un doble del Llavero y un socket en una carpeta temporal; las de `ghost` corren con una casa vacía,
donde no hay socket y `ghost` dice «cerrada» sin tocar el Llavero. **Que `ghost` lea el Llavero de verdad
y que la puerta atienda a la app viva** va a la prueba en vivo, con su fila y el «sí» del usuario.

**Pruebas, local:**
- cargo lib **462** · integración: `contra-el-mac-de-verdad` 22 (3 `#[ignore]` que tocan el Mac, sin
  correr) · `puerta` **12** · `ghost` **5** · clippy 0;
- vitest **299** (+14: 10 de `puerta.test.tsx`, 4 del gate `puerta-solo-local`) · e2e **207** (eran 195:
  los 3 estados nuevos en axe) · fidelidad **216 encuadres** (eran 204), ninguno sobre el umbral; leídas
  como imagen la puerta cerrada, abierta y cerrada sola, y la entrada en «Quién redacta»;
- lint, typecheck, `verify:ephemeral` (ahora con `puerta/`) y `design-sync` (regenerado por `.puerta`),
  limpios.

---

## Para la planeadora al cierre del sprint (va al summary, «Sugerencias de mejora al método»)

1. **Regla dura nueva para TODAS las apps (pedida por el usuario el 2026-09-27): las protecciones del
   Mac se enseñan ANTES de tocarlas.** Nada que pueda:
   - pedir contraseña o Touch ID;
   - abrir un aviso de permiso;
   - tocar el Llavero;
   - registrar algo en launchd o en Ítems de inicio;
   - controlar la interfaz;
   - leer registros del sistema;

   se corre sin una matriz de una fila (qué · para qué · qué aviso vas a ver · cómo se deshace) y un
   «sí» por acción. En esta app ya es la **regla 22 del `CLAUDE.md`**. Se propone al kit como regla
   dura, para que el estampado la lleve a cada app, y a `/audita-sprint` como casilla: «¿qué protección
   del Mac tocó el sprint y dónde está el “sí” del usuario?». Origen e inventario: «Falla del
   constructor: toqué las protecciones del Mac sin avisar», más arriba en esta bitácora.
2. **La regla 10 (la mirada de FORMA abre parada) chocó con el usuario por tercera vez.** Las dos
   anteriores fueron en el S2 («así no vamos a avanzar nada», los textos al gate del MVP). En el S3:
   «solo muéstrame cosas realmente importantes que deba decidir; el resto lo validamos en los gates».
   Sugerencia al método: que la parada la abra solo lo que cambia una decisión del usuario, la
   promesa del producto o toca su Mac; que la FORMA nueva se maquete, se registre «no vista» y vaya al
   gate del MVP. Lo pidió el usuario para esta app; la planeadora decide si vale para todas.
