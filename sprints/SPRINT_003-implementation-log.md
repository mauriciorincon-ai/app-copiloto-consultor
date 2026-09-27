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
- **i2 y d6:** el transcript solo se pinta con una ficha en la banda. Hay que decirlo, y ⌃⌥T no agranda.
- **h5:** con el modo solo audio la banda de una línea sigue «Callado». La «sin resultado» se ve al salir del modo.
- **e4:** necesita una página de kit en negro, sin texto ni pie (`meet-en-negro.html`). Tapar la ventana no sirve: ScreenCaptureKit la lee igual.
- **a6:** «Cierra la banda» pasa a ⌥⎋. Queda ligado al arreglo de ⌘W, pendiente abajo.
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
| 8 | **⌘W sobre la banda** la cerraría sola: el relleno quedaría encima de la reunión y Chrome encogido | S1 | **PENDIENTE**, ver abajo |
| 9 | El manual promete cosas que la relectura desmintió: e4 («tapa la ventana», `MANUAL-DE-USO.md:195-196`) y el transcript incondicional (`:65-66`, `:411`) | S1/S2 | **PENDIENTE**, ver abajo |

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
