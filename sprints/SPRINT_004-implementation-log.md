# Bitácora — Sprint 004 «El ensayo» (primer sprint del ciclo H2)

Branch `sprint-004/el-ensayo`, desde `main` en `adb2493` (el S3 ya mergeado). Orden `SPRINT_004-orden.md`
(aprobada 2026-10-04) · plan autoritativo `SPRINT_004.md` · kit v1.36.0 · método v1.38.0 · estándares
v2.18.0 · ciclo H2, sprint **1 de 3**. El Acto 2 del H1 (prueba en vivo y ⭐⭐ del S3) va aparte: este
sprint no lo espera ni lo toca.

Plan aprobado el 2026-10-04, con su bloque de arranque (Opus 5.5 alto en las fases 0, 1, 3 y 4; Sonnet 5
alto en la 2; Opus medio en la 5) y el «construye» del usuario.

## Las decisiones que el plan tomó (en llano; aprobadas con el plan)

1. Con la banda arriba, **solo se mueve la ventana de la reunión que la app detectó**; sin reunión, la banda
   flota. Abajo, todo como en el H1.
2. En el ensayo la voz puede salir por los altavoces; **el micrófono está sordo mientras la app habla**.
3. La respuesta termina con 2,5 s de silencio o con Enter.
4. Los ensayos vencen como tus notas (retención y la tarea de launchd: **cuarta protección**, fila 4).
5. El progreso pide el desbloqueo una vez por sesión de la app.
6. La puerta local se cierra durante el ensayo.
7. La maniobra §10 (deuda del S3) pasa al S5, declarada ya (abajo).

---

## Fase 0 · Delta del kit v1.32→v1.36, maquetas y ADRs

### El delta del kit (citado por nombre)

- **`CLAUDE.md`:**
  - regla dura 2: «bajo **retención PUBLICADA** por el proveedor» (estándares v2.18.0 §4-T), que resuelve la
    desviación del S3; también `PLANTILLA-ADR-codigo-primero.md` y una nota en el ADR 017;
  - regla 10: **tres clases de mirada** (DECISIÓN abre parada; FORMA y TEXTO, «maquetada, no vista»);
  - regla 15: la demo en rojo se corre con `scripts/demo-rojo.sh`;
  - regla 17: el homepage del repo **apunta al propio repo** (`gh repo edit --homepage`, hecho el
    2026-10-04: estaba vacío);
  - regla 18: comprobación mecánica (`verificar-dependencias.mjs`) y excepciones de `pnpm audit` con ADR;
  - regla 22: las reglas 24 y 25 del kit, por nombre, y el mecanismo de la 25 (abajo);
  - **regla 23 nueva** (la 22 del kit), «los controles de la maqueta tienen su script». La parte del
    generador no aplica: la maqueta se escribe a mano;
  - **regla 24 nueva** (la 23 del kit), «la matriz de envejecimiento». Aplica a la bandeja y a la retención
    de las notas; la fecha «consultado» de las jurisdicciones no cambia el estado de nada y queda fuera.
- **`.claude/commands/`:** `audita-sprint` v1.36 (casilla 7 renumerada, casilla 8, decisiones en llano, la
  segunda casilla 4 con otro auditor, el orden de pago) · `deploy-check` v1.36 (§3 pasada de interacción,
  §9 homepage con reparación, 7-S y envejecimiento) · `plan-sprint` con tres clases **y su paso 10** ·
  `release-check` §1 con la regla 25 · `README`.
- **Scripts y configuración:** `scripts/demo-rojo.sh` y `.demo-rojo/` en `.gitignore` ·
  `scripts/verificar-dependencias.mjs` como paso de `quality` en los PR · `auditConfig.ignoreGhsas: []`
  (pnpm 11.10) · hook PreToolUse de gitleaks que escanea el **contenido** y avisa cuando no puede (con el
  mensaje del Llavero de esta casa) · `githooks/pre-commit` que **falla cerrado** sin gitleaks
  (`KIT_SIN_GITLEAKS=1` a sabiendas).

### Regla 25: `cargo test` a secas ya no toca el Mac

Hasta hoy, 24 de los 27 tests de `contra-el-mac-de-verdad.rs` corrían con `cargo test` a secas: abrían el
micrófono y el audio del sistema, hacían sonar los altavoces e inventariaban `~/Documents`. La regla 22 lo
resolvía con una costumbre (correr en local solo `--lib --test puerta --test ghost`). Ahora es mecánico:

- **Marcas.** 11 tests pasan a `#[ignore = "hardware: …"]` (reconocimiento de voz, micrófono, audio del
  sistema, altavoces, la sesión completa y su canaria del log, el WER, la voz). Los que necesitan al usuario
  delante se llaman `en_vivo_*` (`en_vivo_la_ventana_de_meet_se_captura_y_se_lee`,
  `en_vivo_el_llavero_guarda_lee_y_borra_la_clave`, `en_vivo_launchd_borra_a_su_hora_sin_la_app`).
- **El centinela** (`src-tauri/src/hardware.rs`): con `AG_SIN_HARDWARE=1`, cada entrada nativa **aborta el
  binario antes de cruzar a C** con el nombre de lo que iba a abrir. Está en 15 sitios: los dos grifos, la
  voz, el reconocimiento y la descarga de modelos, la captura de pantalla, el aviso y la escritura de
  Accessibility, los cinco del Llavero, el desbloqueo y `launchctl`. Aborta en vez de `panic!` porque la
  escucha abre sus grifos en un hilo de fondo, y un pánico ajeno deja el test en verde.
- **CI:** `cargo test --locked` con `AG_SIN_HARDWARE=1`, y otro paso con
  `cargo test --locked -- --include-ignored --skip en_vivo_`. `verify:ephemeral:runtime` y «el kit, con su
  salida» ganan `--include-ignored`. La canaria del log lanza a su hijo con `--include-ignored`.
- **Gate `tests/unit/cargo-test-sin-hardware.test.ts`:** cada `#[ignore]` dice por qué (hardware, en vivo o
  medición, y «en vivo» va con `en_vivo_*`); en `contra-el-mac-de-verdad`, todo test que llega a una entrada
  nativa —aunque sea por un ayudante— lleva su marca; el centinela está en cada entrada; la CI tiene los dos
  pasos.

**El primer rojo del centinela fue real.** Su primera corrida (`AG_SIN_HARDWARE=1 cargo test --lib`) abortó
en `habla::apple::pruebas::una_ficha_con_un_cero_dentro_no_rompe_nada`: un test unitario que **desde el S2
encolaba una frase en la voz del sistema** y la callaba en el acto. Nadie lo había visto porque se callaba
antes de sonar. Ahora va marcado `hardware:`, y el centinela de la voz y del reconocimiento se movió justo
antes de la llamada nativa (después de validar), para que las pruebas de validación pura no toquen nada.

### Los rojos de los gates nuevos (regla 15, con `scripts/demo-rojo.sh`)

| Gate | Mutación | Rojo (a quién nombró) | Verde al restaurar |
|---|---|---|---|
| `verificar-dependencias.mjs` (regla 18) | `jsdom@30.1.1` → `30.1.0` en `pnpm-lock.yaml` | «jsdom: 30.1.1 (origin/main) → 30.1.0 (este árbol)» | ✓ 381 paquetes |
| hook de gitleaks sobre el contenido | carnada del `CLAUDE.md` (armada por partes, sin escribirla en un archivo) en `tool_input.content` | exit 2 · «SECRET DETECTADO en el contenido a escribir» | exit 0 con «hola» |
| `pre-commit` falla cerrado | `PATH` sin gitleaks | exit 1 · «commit BLOQUEADO» | exit 0 con `KIT_SIN_GITLEAKS=1` y con gitleaks |
| centinela de hardware | se quita el `#[ignore]` de `el_microfono_se_abre_o_dice_por_que_no` | aborto: «[hardware] el micrófono: se iba a abrir con AG_SIN_HARDWARE puesta» (antes de abrirlo) | 1 ignorado, 0 fallos |
| `cargo-test-sin-hardware` | se quita el `#[ignore]` de `transcribe_la_pregunta_en_espanol`, que llega por el ayudante `probar` | «transcribe_la_pregunta_en_espanol (línea 123) llega a probar(» | 4/4 |
| `controladores-maqueta` (regla 23) | se quita `<script src="assets/maqueta.js">` de `ia.html` | «ia.html: tiene barra y no carga assets/maqueta.js» | 25/25 |
| `maqueta-interaccion` (e2e, regla 23) | un botón de `posicion.html` repite el estado de otro | «posicion.html · «a» se ve igual que «a»» | ✓ |
| `envejecimiento` (regla 24) | se quita el `Math.max(0, …)` de los días que le quedan a una reunión guardada | «Notas · 7 d · ahora 8640000 s: una cuenta negativa → «se borra solo en -100 días»» (×4 plazos) | 2/2 |

La pasada de interacción del arnés de capturas (`scripts/capturar-maqueta.mjs`) se vio nombrar un botón
muerto («el control [(sin data-*)] no cambió nada»). Su primera versión daba un falso positivo con el botón
de tema (reseteaba al mismo tema que el botón pedía); se corrigió reseteando al valor contrario del botón.

### ADRs, antes de su fase

- **ADR 019 «el ensayo, código primero»** (nuevo): banco por reglas publicadas, catálogo de objeciones con
  fuente, evaluación determinista sin puntaje, acento LLM opt-in con bóveda y fallback, la sesión solo con el
  micrófono, sordo mientras la voz habla, respuesta por silencio o Enter, teclas de ventana, excluyente con la
  reunión. El porqué: [S8] y [S13] de la investigación científica.
- **ADR 004, enmienda 1:** con la banda arriba, la reunión baja y se encoge y se devuelve entera; `AXPosition`
  en escritura solo para la ventana acoplada, `AXMainWindow` en lectura, destino = la reunión detectada.
- **ADR 002, enmienda 8:** `posicionDeLaBanda`, `avisoDeArribaVisto`, `enriquecerElBanco` y `ensayos/`.
- La enmienda 4 del ADR 015 (`ensayos/` con la misma llave) se escribe antes de la fase 4.

### Las maquetas

- **`docs/diseno/ensayo.html`** (nueva, mirada de DECISIÓN 1): siete estados —preparar, preguntando,
  respondiendo, evaluada, el informe, tu progreso, sin corpus—, en es/en y en los dos temas. Componentes
  nuevos en `ghost.css` (`.pregunta-e`, `.reloj-e`, `.respuesta-e`, `.evidencia-e`, `.cifras-e`) e icono
  `i-ensayo`. Enlazada desde el recorrido (`index.html`, 09).
- **`docs/diseno/posicion.html`, variante «arriba»** (mirada de DECISIÓN 2): 88 acoplada, ampliada, solo
  audio, qué ve el cliente y sin reunión (flota), con la cámara dibujada y **el criterio de la cámara
  escrito** (distancia, hacia dónde se mira, la zona libre de Meet, Zoom y Teams, lo que se mira en una
  llamada real —la barra flotante de Zoom y Teams al compartir—, qué ventana se mueve, y abajo).
  `ghost.css` gana `.banda[data-borde="arriba"]`.
- **FORMA y TEXTO, maquetada, no vista** (registro en `docs/diseno/README.md`): `sesion.html` (la fila «La
  banda» con ⌃⌥B y el aviso de la primera vez) · `honestidad.html` («Tuyo» suma tus ensayos; 12 de 12) ·
  `ia.html` («Enriquecer el banco», apagado).
- **El rail con «Ensayo»** solo está en `ensayo.html`: el resto de las maquetas lo ganan en la fase 3, junto
  con el producto, para que la fidelidad compare lo mismo con lo mismo.
- **Leídas como imagen:** las 28 capturas de `ensayo.html` (siete estados × dos temas × dos idiomas) y las de
  `posicion.html` en sus estados nuevos. Dos ajustes salieron de mirarlas: el Dock tapaba los controles de la
  reunión en la variante arriba (se esconde, como en los acoplados de abajo) y la etiqueta de «sin reunión»
  pisaba la línea de la ficha (bajó). `maqueta-cabe` midió +16 px en `ia.html · s4 · es`: el interruptor
  nuevo se acortó a «Enriquecer el banco».

### Las miradas de DECISIÓN

| # | Archivo | Fecha | Veredicto, con la frase del usuario |
|---|---|---|---|
| 1 | `docs/diseno/ensayo.html` (siete estados) | 2026-10-04 | **Aprobada** — «La abri y la apruebo la pantalla ensayo, continua». El «continua» no abre la fase 1: la mirada 2 va en el mensaje siguiente |
| 2 | `docs/diseno/posicion.html`, variante «arriba» (cinco estados y el criterio de la cámara) | 2026-10-04 | **Aprobada** — «Si me gusta mucho ka banda arriba buen diseño, lo abri y lo apruebo». Con las dos en «sí» y el «continua» de la mirada 1, arranca la fase 1 |

### La deuda del S3

- **Maniobra §10 → S5**, declarada aquí, en la fase 0, sin intentarla (decisión 7 del plan): el resto de
  `design-system.md` §10 —la unidad que falta, la ficha del cliente y lo comprometido en la reunión— no mueve
  el outcome de este sprint, y en el S3 el corte se declaró tarde (B28).
- **B5 y el aviso de Documentos** siguen en el Acto 2 del H1 y no se tocan aquí.
- **WER `manual`:** desde hoy sus tests llevan `#[ignore = "hardware: …"]`. Su corrida local de este sprint
  va con la corrida en vivo de la fase 4.

### Fricciones del kit (K#), para la planeadora

- **K-S4-1:** la plantilla `ci-escritorio.yml` no trae el paso de `verificar-dependencias` (solo la web); se
  copió de ahí.
- **K-S4-2:** la edición v1.36.0 de `plan-sprint.md` borró el paso 10 (auditar al concluir), la matriz de una
  fila, «segundas vueltas sin parada» y «`gh pr checks` tras cada push». Aquí se conservan.
- **K-S4-3:** la regla 10 del `CLAUDE.md` del kit sigue diciendo «dos clases»; las tres solo están en
  `plan-sprint` y en el método.
- **K-S4-4:** `release-check.md` del kit no se actualizó para las reglas 24 y 25.
- **K-S4-5:** `.demo-rojo/` no está en la plantilla de `.gitignore`.
- **K-S4-6:** `--include-ignored` a secas, como lo trae el kit, arrastraría a la CI las pruebas que solo
  pueden correr en el Mac del usuario (ventana de Meet, su Llavero, launchd). Hace falta una segunda marca:
  aquí, el prefijo `en_vivo_` y `--skip en_vivo_`.
- **K-S4-7:** `demo-rojo.sh --puerto` mata lo que ocupe el puerto, sea de quien sea. En esta casa solo se
  mata por PID propio; el `CLAUDE.md` lo dice.
- **K-S4-8:** la orden nombra **⌃⌥K** para terminar el ensayo; esa tecla no existe en la app (desviación 2).
- **K-S4-9:** la orden dice que «el README ya pide `--lib --test puerta --test ghost`»: estaba en el
  `CLAUDE.md` (regla 22), no en el README.

## Fase 1 · La banda arriba (C1')

Arranca el 2026-10-04 tras las dos miradas de DECISIÓN en «sí» y el «continua» de la mirada 1.

### Lo que se construyó

- **La geometría, pura** (`src-tauri/src/ventana/geometria.rs`, nuevo): `Borde` (arriba de fábrica · abajo),
  `Monitor` (marco + área útil) y `franja(borde, monitor, alto)`. Arriba nace en el borde superior del **área
  útil** que da Tauri (`work_area`, el `visibleFrame` de macOS ya volteado): bajo la barra de menús y el notch,
  nunca con una constante. Abajo es la fórmula del H1, intacta. `barra(monitor)` es lo que el relleno sube el
  fondo. `ventana/mod.rs` pide la franja aquí: `abrir_banda`, `ajustar_banda` y `franja` reciben el borde.
- **La preferencia** (`prefs.rs`): `posicionDeLaBanda` (arriba) y `avisoDeArribaVisto` (no), como dice la
  enmienda 8 del ADR 002. Un archivo del H1 abre la banda arriba y enseña el aviso una vez (test).
- **El acople «baja y se encoge»** (`acople/mod.rs`, ADR 004 enmienda 1):
  - `decidir_arriba` (puro): baja el borde superior hasta la banda y deja el inferior donde estaba; por debajo
    de 240 px no se toca; otro monitor, no se toca.
  - `Destino` + `plan_arriba`: de una lista de ventanas sale **una sola** decisión, la del destino.
  - `Paso` y el orden como dato: al acoplar, **alto y luego posición**; al devolver, **posición y luego alto**,
    y solo lo que cambió (una huella del H1 se devuelve con un paso de alto, como entonces; devolver dos veces
    no mueve nada).
  - `acoplar_arriba` (nativo): suelta primero, comprueba el cerrojo del PID, ejecuta los pasos, relee y solo
    cuenta lo que pasó (`quedo_bajo_la_franja`); si macOS no la deja bajar, deshace lo hecho y la banda flota.
  - `soltar` pasa a la devolución en dos pasos. `hubo_cambio` mira posición **o** tamaño.
  - `Informe.ms`: el acople y la devolución dicen cuánto tardaron, para medir el ≤ 300 ms en vivo.
- **La única escritura de `AXPosition`** (`acople/ax.rs`): `poner_posicion`, que solo llama `ax::mover`, que
  solo llama el acople, con su centinela de la regla 25. Y `indice_de_la_principal` (`AXMainWindow` comparado
  con `CFEqual`), `titulos_con_indice` y `nombre_de`.
- **El detector nombra la ventana** (`sesion/mod.rs`): `ventana_de_la_reunion()` sigue el orden de
  `objetivo_de`; Zoom y Teams dan su ventana principal y un navegador da la de Meet por su posición en la
  lista. El acople recibe un número, no un título.
- **El cableado** (`lib.rs`):
  - `borde_de`, `LaFranja` (borde, barra, aviso visto) con su comando `la_franja` y su evento `franja`;
  - `fijar_posicion_de_la_banda` y `entendido_el_aviso_de_arriba`, solo en la principal;
  - `poner_la_banda`: **suelta → recoloca → reacopla**, y lo guarda;
  - **⌃⌥B**, la octava letra con ⌃⌥, que alterna el borde desde cualquier sitio;
  - arriba, el latido del arranque espera una **reunión** (no a quien esté delante) y al empezar una sesión se
    acopla su ventana si no lo estaba; soltar el asa solo reacopla si había algo acoplado. Abajo, todo como en
    el H1.
- **La interfaz:**
  - `src/franja.ts` (nuevo): `useFranja`, `fijarPosicionDeLaBanda`, `entendidoElAvisoDeArriba`.
  - `Banda.tsx`: `data-borde` en la banda y en la de solo audio.
  - `asa.ts`: `altoAlArrastrar`, arriba se arrastra hacia abajo para ampliar.
  - `Relleno.tsx`: `desplazamientoDelFondo`, arriba sube el fondo lo que mide la barra.
  - `Sesion.tsx`: la fila **«La banda: arriba · abajo · ⌃⌥B»** y **el aviso de la primera vez**, que ocupa el
    sitio de la tarjeta de la reunión como en `sesion.html` · la primera vez.
  - `index.css`: la línea de la banda arriba va abajo.
  - Textos en es/en, idénticos a la maqueta.
- **Contrato (regla 19):** `LA_FRANJA_ARRIBA` y `LA_FRANJA_ABAJO` salen del serde de Rust; `src/franja.ts`
  entra en las declaraciones del gate de lectores. Permisos: la banda y el relleno solo leen la franja; los dos
  comandos que la cambian son de la principal y entran en `SENSIBLES`.
- **Un arreglo del puente** (`puente.ts`): los módulos de Tauri se importan una vez y se comparten. Con dos
  preguntas a la vez (el relleno pide su fondo y la franja), vitest resolvía el segundo `import()` al módulo
  real: dos «Unhandled Rejection» sueltos en `acople.test.tsx`. Medido con una traza: `fondo_del_relleno` iba al
  fingido y `la_franja` al real. En la app no cambia nada.

### Pruebas

- **Rust** (`AG_SIN_HARDWARE=1`): lib **497** (+16: geometría 5, acople 9, prefs 1, detector 1) · contra el
  Mac 13 (+14 apartados) · ghost 5 · puerta 14 · clippy `-D warnings` limpio.
- **Interfaz:** vitest **395** (+8: `la-banda-arriba.test.tsx` 6, asa 2) · lint · typecheck.
- **e2e:** 208 + **12 de axe nuevos** (banda arriba, arriba en solo audio, la primera vez en Sesión; dos temas).
- `verify:ephemeral` estático en verde; `maqueta-cabe` y `maqueta-interaccion` en verde.
- **Fidelidad** (`pnpm fidelidad`, evidencia `docs/fidelidad/s4-*`): **232 encuadres, ninguno sobre el umbral
  (0,15 %), sin desbordes ni errores de página.** Nuevos: `banda-arriba` (88 · 200 · 44) y `sesion-aviso`;
  `sesion` pasa a compararse con «sprint 4».

### Los rojos (regla 15, con `scripts/demo-rojo.sh`)

| Gate o prueba | Mutación | Quién lo nombró |
|---|---|---|
| barrido de `AXPosition` (gate nuevo) | un segundo `"AXPosition"` en `ventana/mod.rs` | `ax_mover_es_el_unico…`: «ventana/mod.rs nombra AXPosition: solo el acople lo toca» |
| orden del acople | `[Mover, Alto]` en `pasos_del_acople_arriba` | `al_acoplar_arriba_primero_se_encoge_y_despues_baja` |
| una sola decisión | `.find(\|_\| true)` en `plan_arriba` | `arriba_de_una_lista…`: «decidió sobre otra ventana» |
| geometría arriba | `m.y` en vez de `monitor.util.y` | `arriba_nace_bajo_la_barra…`, con el monitor en el mensaje |
| `hubo_cambio` con posición | la fórmula de solo alto | `hubo_cambio_mira_tambien_la_posicion` |
| relleno arriba | `return alto - pantalla` | `la-banda-arriba.test.tsx`: «-barra con cualquier alto del asa» |
| asa arriba | `delta = desde - ahora` en los dos bordes | `asa.test.ts`, dos casos |
| «Entendido» llega a Rust | sin `entendidoElAvisoDeArriba()` | `la-banda-arriba.test.tsx`: el aviso |

Las ocho volvieron a verde tras restaurar (grep + cmp). **Y la fidelidad se vio en rojo de verdad** antes de
pasar, con tres causas, las tres arregladas sin subir el umbral:

1. la banda arriba se comparaba con las copias de `posicion.html`, que son una ficha resumida dentro de un
   escritorio (5,7 % · 4,1 % · 17,3 %). Ahora la referencia es `banda.html` con `data-borde="arriba"`, que es
   exactamente la variante (el mismo artefacto con su modificador);
2. la regla de la sala de diseño que quita la línea de abajo a las bandas pegadas abajo pisaba el modificador
   (1,9 % en la de 44): `maqueta.css` respeta ahora `[data-borde="arriba"]`;
3. el aviso de la primera vez convivía con la tarjeta de la reunión (25 %) y la maqueta lo pone en su sitio:
   el producto obedece.

### Desviaciones de la fase

- La fila «La banda» va en **todos** los estados de Sesión con «Las dos pistas», no solo antes de empezar: es
  una preferencia, como «Leerla sola». `sesion.html` la suma en «en marcha», «la pregunta de la NDA» y «sin
  jurisdicción» (forma, **maquetada, no vista**; `maqueta-cabe` en verde).
- El aviso de la primera vez **ocupa el sitio de la tarjeta de la reunión** hasta «Entendido», como en la
  maqueta: las dos no caben en 640 px y la reunión sigue en el rail («Meet detectado»).
- El relleno puede llamar a **dos** comandos, los dos de lectura (su fondo y la franja). El gate de permisos
  lo dice así.
- El acople arriba gana un disparador nuevo: **empezar la reunión** (Iniciar sesión o Solo notas). Abajo no
  lo tiene, porque al pulsar el botón la aplicación de delante somos nosotros.

### La corrida en vivo (fila 1 de la regla 22)

Pendiente del «sí» del usuario. Antes de pedirlo se miró, sin ningún permiso, qué tocaría el arranque de la
app en este Mac: `ls ~/Library/LaunchAgents` y `launchctl list` sin nada de la app, y en su carpeta solo
`diccionario.yaml` (sin preferencias, notas, bandeja ni huella). Así que el arranque no re-registra launchd,
no lee el Llavero (el API está apagado) y no reindexa ningún corpus.

## Desviación del plan

1. Arriba, el acople actúa sobre la ventana de la reunión detectada, no sobre la de delante (ADR 004, enmienda 1).
2. No hay ⌃⌥K: las teclas del ensayo son de la ventana (Enter, R, S, Esc) y ⌥⎋ corta todo.
3. El radar coral de procesos no se apaga durante el ensayo; el ensayo no arranca nada de reunión.
4. Una cuarta protección que la orden no listaba: launchd, al vencer los ensayos (fila 4 de la matriz).
5. Las reglas 22 y 23 del kit entran como 23 y 24 de la app; la 24 y la 25 del kit, dentro de la 22 de la app.
6. La maniobra §10 pasa al S5, declarada en la fase 0.
7. La enmienda 002 se escribió en la fase 0 porque la fase 1 ya persiste la preferencia.
8. La preferencia arriba/abajo lleva tecla global: **⌃⌥B** (libre). El test de teclas pasará de siete a ocho
   letras en la fase 1.
9. La fila «La banda» va en todos los estados de Sesión con «Las dos pistas» (fase 1), y el aviso de la primera
   vez ocupa el sitio de la tarjeta de la reunión hasta «Entendido», como en la maqueta.
10. El acople arriba también se dispara al empezar la reunión (fase 1); abajo, como en el H1.
