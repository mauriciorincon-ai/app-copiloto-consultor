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

Antes de pedir el «sí» se miró, sin ningún permiso, qué tocaría el arranque de la app en este Mac:
`ls ~/Library/LaunchAgents` y `launchctl list` sin nada de la app, y en su carpeta solo `diccionario.yaml`
(sin preferencias, notas, bandeja ni huella). Así que el arranque no re-registra launchd, no lee el Llavero
(el API está apagado) y no reindexa ningún corpus. **Matriz de una fila enseñada** (la fila 1 del plan, copiada
aquí por la auditoría del S4, B4):

| Qué | Para qué | Qué aviso vas a ver | Cómo se deshace |
|---|---|---|---|
| La app escribe `AXPosition` (y el alto) en la ventana de la página de prueba de Meet, en Chrome | bajarla y encogerla bajo la banda arriba, y medir cuánto tarda | ninguno nuevo: Accesibilidad ya está concedida | sola, al cerrar: devuelve la posición y el tamaño (devolución doble) |

**El «sí» del usuario:** «Sí, haz la corrida en vivo de la banda arriba. Abierto en Google Chrome». Dos corridas con `pnpm tauri dev` y
`docs/kit-de-prueba/pantalla/meet-de-prueba.html` en Chrome. **Ningún aviso de macOS** en ninguna:
Accesibilidad ya estaba concedida, el registro lo dice (`accesibilidad=Concedido`).

**Primera corrida: acopló, y la devolución no la reconoció.**

| Momento | Registro | Lo que pasó |
|---|---|---|
| arranque | `«banda»: 1470x88 en (0,33)` y el relleno igual | la banda bajo la barra de menús de 33 pt |
| detector | `reunión detectada: «Google Meet»` | la ventana de la página de prueba |
| acople | `ventanas=1 · 178 ms` | **el usuario la vio pegada bajo la banda**; la huella anotó `y=176, alto=692` en vez de `121 · 747` |
| clic en el asa | `reacople … ventanas=1 · 338 ms` | el clic para traer la app al frente cayó en el asa y rehízo el acople |
| ⌘Q | `soltar al salir … ventanas=0` · «ninguna ventana de «Google Chrome» coincide con la huella» | **Chrome no volvió**: el usuario lo puso en su sitio a mano |

**Lo que enseñó.** Chrome aplica lo que se le pide por la Accessibility API **un instante después**: la app leyó
la ventana justo después de moverla, anotó una lectura intermedia, y al salir la ventana real no coincidía con
lo anotado. Por la regla 3 del acople («solo se devuelve lo que sigue como lo dejamos») no la tocó: el fallo
fue no devolver, nunca devolver mal. Dos arreglos, cada uno con su test visto en rojo:

1. `acople::asentar`: tras cada escritura se relee hasta que **dos lecturas seguidas coinciden** (cada 25 ms,
   techo de 400 ms por paso), y es eso lo que se anota y lo que decide. Rojo: con el bucle anulado,
   `se_anota_la_ventana_cuando_se_queda_quieta_y_no_antes` —que repite la secuencia medida, 176 y luego 121—
   dice «se anotó la lectura de justo después».
2. `asa.ts`: soltar el asa **sin haberla arrastrado** no reacopla. Rojo: sin la condición,
   `la-banda-arriba.test.tsx` · «un clic no reacopla».

Y el acople arriba deja en el log su geometría (de dónde, qué se pidió, qué quedó; solo números, ADR 003).

**Segunda corrida (el usuario: «Volvamos a hacer la prueba»): todo cuadra.**

| Momento | Registro |
|---|---|
| acople | `arriba: estaba y=33 alto=835 · pedido y=121 alto=747 · quedó y=121 alto=747` · `ventanas=1 · 228 ms` |
| ⌘Q | `soltar al salir: … ventanas=1 · 123 ms` (cuenta como devuelta solo si, releída y quieta, coincide con la original) |
| después | la huella borrada; en la carpeta de la app, solo `diccionario.yaml` |

El presupuesto se cumple: **228 ms al acoplar y 123 ms al devolver** (≤ 300 ms). Lo que no se probó en vivo
queda para el ⭐: Zoom y Teams (su ventana principal), una pantalla externa y la pantalla completa.

## Fase 2 · El banco de preguntas (C18)

Arranca el 2026-10-04 con el «continúa» del usuario tras la fase 1.

### Lo que se construyó

- **Los catálogos, publicados y dentro del binario:**
  - `data/ensayo/reglas.json`: las seis reglas con su nombre es/en; las clases de sección por título; las
    plantillas; qué clases no dan cifras ni compromisos (contexto y supuestos son del cliente); topes por
    regla; y las palabras vacías es/en para el idioma.
  - `data/ensayo/objeciones.json`: diez objeciones bilingües con etiquetas. Cada una con su **fuente del
    informe de mercado** de la planeadora (Kuznetsova ×3, SeattleDataGuy, Gartner, Bumeran, la reseña de
    Techjockey) o «criterio del builder» (tres). Ninguna repite una cifra.
- **`ensayo/banco.rs`** (módulo nuevo, **protegido** en `verify:ephemeral`, puro): `todas` arma el banco entero
  regla por regla; `armar` reparte **por turnos** hasta el tope (5 · 8 · 12) sin repetir; `idioma_de` por
  palabras vacías (si empatan, `None`: manda el del usuario); `terminos` (prefijo de cinco letras, sin tildes)
  para fundar; `con_las_del_modelo` las pone detrás, sin repetir. Las frases largas se cortan en la primera
  coma o dos puntos.
- **`ensayo/enriquecer.rs`** (el acento opt-in, ADR 019 §3): `Peticion` (títulos y primera frase con ids
  `S1…`/`C1…` y el idioma), `fundar` (sección dada por id o título, ≥ 2 términos en común, «?», ≤ 25 palabras,
  máximo 5; lo demás se cuenta), `enriquecer` con el techo de la síntesis y la respuesta tardía cobrable.
  `PorQueNo` cerrado para la línea de la pantalla.
- **`propuestas::cifras`**: las cifras con lo que cuentan, reutilizando el detector de la regla `cifra` (y
  el plural del modificador en inglés: «two review rounds»).
- **El corpus:** `secciones_de` (el documento entero por título, solo si es de tu corpus), `propuestas_de`
  (las que nombran al cliente) y `ficha_de`.
- **El `mock`** aprende el banco: dos preguntas fundadas y una que no (sección `S99`), para que el kit recorra
  el descarte en cada corrida.
- **«Enriquecer el banco» en IA**, apagado de fábrica: la preferencia `enriquecerElBanco` (ADR 002, enmienda
  8), el comando `enriquecer_el_banco` (solo en la principal, en `SENSIBLES`), `EstadoDeLaIa.enriquecer` en el
  contrato y el segundo interruptor de `ia.html` · sprint 4. La fidelidad de IA se compara ya con «sprint 4».
- **Kit v3** (`docs/kit-de-prueba/ensayo.json` y su test `el_kit_del_ensayo_mide_el_banco_por_regla`, que
  entra en el paso «el kit, con su salida» de la CI): Páramo Azul (de `corpus/`) y Northwind Feed Co. (inline,
  en inglés). El LEEME pasa a v3.

### Lo que imprime el kit v3 (corrida local, 2026-10-04)

| Caso | seccion | cifra | compromiso | riesgo | cliente | objecion | banco entero |
|---|---|---|---|---|---|---|---|
| Páramo Azul (es) | 6 · P 1,000 · R 1,000 | 4 · 1,000 · 1,000 | 3 · 1,000 · 1,000 | 2 · 1,000 · 1,000 | 3 · 1,000 · 1,000 | 3 · 1,000 · 1,000 | 5,3 ms |
| Northwind (en) | 6 · 1,000 · 1,000 | 4 · 1,000 · 1,000 | 3 · 1,000 · 1,000 | 2 · 1,000 · 1,000 | 2 · 1,000 · 1,000 | 3 · 1,000 · 1,000 | 4,6 ms |

- **El mock:** 2 fundadas y 1 descartada en los dos casos.
- **Una llamada de enriquecer:** ≈ 390 tokens de entrada + 200 de salida ≈ **US$0,0014 con Claude Haiku y
  US$0,0004 con Groq** (queda escrito en el ADR 019 §5, que lo dejó pendiente para esta fase).
- **Dicho sin adornos:** el kit y las reglas los escribió el mismo constructor, en la misma fase; que todo dé
  1,000 es un **piso** (mínimo 0,75 por regla), no una prueba de calidad. La de verdad es una propuesta real
  del usuario, en el ⭐ del ensayo. El LEEME lo dice.

### Pruebas

- **Rust** (`AG_SIN_HARDWARE=1`): lib **515** (+17: banco 9, enriquecer 6, cifras 1, corpus 1) · contra el Mac
  14 (+1, el kit v3) · ghost 5 · puerta 14 · clippy limpio.
- **Interfaz:** vitest **397** (+1: el interruptor de IA) · lint · typecheck.
- **e2e:** 220 (+12 de la fase 1) · `verify:ephemeral` con `ensayo/` en la lista.
- **Fidelidad:** 232 encuadres, ninguno sobre el umbral.

### Los rojos (con `scripts/demo-rojo.sh`)

| Gate o prueba | Mutación | Quién lo nombró |
|---|---|---|
| piso del kit v3 | sin «incluye» en los compromisos | `el_kit_del_ensayo…`: «el banco del ensayo por debajo de su piso» |
| grounding del modelo | `TERMINOS_MINIMOS = 0` | `solo_se_funda_lo_que_nombra_una_seccion_dada…` |
| las cifras del contexto son del cliente | `sin_cifras` sin «contexto» | `las_seis_reglas_leen_lo_que_tu_escribiste` |
| las del modelo van marcadas | `De::Propuesta` en vez de `De::Modelo` | `solo_se_funda…` |
| `ensayo/` es protegido | un `std::fs::write` en `banco.rs` | `verify:ephemeral`: «2 uso(s) de disco/red en módulos efímeros» |
| el interruptor de IA | el botón llamando a `redactar_sugerencias` | `la-ficha-llega-a-la-banda.test.tsx` |

Las seis volvieron a verde tras restaurar.

### Desviaciones de la fase

- **La llamada al modelo desde la app** (elegir el proveedor con `proveedor_de_ahora`, el «sobre» de «Ver lo
  que salió», `cobrar`) se cablea en la **fase 3**, con la pantalla del ensayo: un comando sin quien lo llame
  tumba el gate de comandos muertos, y una función sin llamador, clippy. Lo que la llamada necesita —la bóveda,
  el registro B37, el contador de red y el tope— ya lo hace el proveedor externo por dentro (`sintesis/api.rs`).
- Las preguntas del modelo **se suman detrás** del tope elegido («hasta cinco más», ADR 019 §3), no dentro.

## Fase 3 · La sesión de ensayo (C18)

Arranca el 2026-10-04 con el «continúa» del usuario tras la fase 2 (CI de `8bb7d02`: los tres checks en
`success`).

### Lo que se construyó

- **`ensayo/sesion.rs`** — la máquina de estados, **pura** y con el reloj por parámetro: preguntando (la voz
  lee) → respondiendo (el micrófono oye) → evaluada → … → cerrado (el informe). Enter, R, S, Esc y «Sí lo
  dije». Devuelve acciones (`Decir`, `Callar`, `Evaluar`, `Avisar`) y quien la lleva las ejecuta. Las reglas
  de tiempo del ADR 019 §6, cada una con su test:
  - **sordo mientras la voz lee y 300 ms después** (`COLA_DE_LA_VOZ_MS`); si la voz no suena en 2 s, se
    responde sin esperarla; sin voz para el idioma, el resto del ensayo va sin voz;
  - **la respuesta se cierra con 2,5 s de silencio desde el fin de tu voz** (`SILENCIO_DE_RESPUESTA_MS`), y
    pensar antes de la primera palabra no cierra nada; o con Enter, que espera al turno que se está
    transcribiendo;
  - **tu tiempo** empieza cuando la pregunta terminó (la cola es de la app) y termina con Enter o con el
    fin de tu voz (los 2,5 s son de la app);
  - **un turno de otra ronda se tira** (repetir mientras se transcribía no mezcla respuestas);
  - Esc a media respuesta: esa pregunta no cuenta ni como respondida ni como saltada;
  - el informe suma y promedia sin inventar (sin ritmo, «—»).
- **`ensayo/oido.rs`** (protegido) — **solo `Grifo::del_microfono`**: la `Oreja` reutiliza el anillo de 30 s,
  el detector por energía y el fin de turno de 320 ms sin tocarlos; cada turno se transcribe al cerrarse, en
  su hilo, con el diccionario B3, y su audio se pisa al volver del motor. Sordo, el turno a medias se cierra
  con lo que tenía y lo que entra no llega al detector (ni se transcribe ni le enseña al suelo de ruido un
  silencio falso); el origen se mueve con lo que se salta, así el reloj y el anillo siguen juntos.
- **`ensayo/evaluacion.rs`** (pura) — evidencia citada si comparte **≥ 2 términos que no estaban ya en la
  pregunta** con la ficha (titular y línea); tiempo; ritmo (≥ 5 palabras y ≥ 2 s, si no «—»); muletillas
  de **`data/ensayo/muletillas.json`** (es/en, por frases enteras, con «no_tras» para las que muchas veces
  no lo son: «I'd like», «vamos a ver», «what kind of»; sin «eh» ni «um»).
- **`ensayo/mod.rs`** — `Ensayo`: un latido de 40 ms que pasa lo que oyó el micrófono a la sesión y ejecuta
  lo que pide; cierra el micrófono **en cuanto el ensayo termina**, no al cerrar la pantalla. Lo que necesita
  de la app entra por el trait `Mundo` (la voz, las fichas del disparo, la señal a la pantalla), y por eso
  hay una prueba de **punta a punta sin Mac**: la voz lee, la «voz» que entra mientras lee no se convierte en
  respuesta, un segundo de voz inventada se transcribe, 2,5 s de silencio cierran, «Sí lo dije», Enter, Esc
  y el informe. La siguiente pregunta llega **en menos de un segundo** tras Enter (lo mide la misma prueba).
- **`lib.rs`** — `ElEnsayo` (con un número, para que lo que vuelva del modelo no caiga en otro ensayo); los
  comandos `preparar_el_ensayo`, `empezar_el_ensayo`, `ensayo_listo`/`repetir`/`saltar`/`terminar`,
  `ensayo_si_lo_dije`, `estado_del_ensayo` y `cerrar_el_ensayo`, **solo en la ventana principal** (`build.rs`,
  `capabilities/default.json`, `SENSIBLES`); la señal `ensayo` sin dato (tus respuestas no viajan en eventos).
  - **Excluyente con la reunión:** con una sesión abierta (escuchando, en solo notas o cerrándose) no se
    ensaya —`NoEmpezo::EnReunion`—, y `empezar` suelta el ensayo antes de abrir sus pistas.
  - **La puerta local se cierra** antes de abrir el micrófono, y `en_reunion` cuenta un ensayo abierto: la
    puerta no se abre mientras ensayas.
  - **`corte::Pieza::Ensayo`**: el corte pasa a **12 de 12** (el compilador obligó a resolverla).
  - **El acento del modelo, cableado:** `proveedor_de_ahora` con el nombre de la propuesta como «sobre» de
    «Ver lo que salió», `cobrar` y el cobro tardío; el ensayo no espera: empieza con el banco por reglas y lo
    del modelo llega al final cuando llega, con su línea («El modelo sumó 2 preguntas…» o por qué no).
  - La voz se usa **sin el candado de los auriculares**: ese candado es de la reunión (ADR 014,
    `habla::cabe_decirla`, intacto); en el ensayo no hay nadie que oiga y el micrófono está sordo.
- **La pantalla `Ensayo.tsx`**, fiel a `ensayo.html`: preparar (cliente, propuesta, 5 · 8 · 12, la voz, el
  idioma, de dónde salen, el interruptor del modelo y, si este Mac no transcribe el idioma de la propuesta, el
  aviso —`Preparacion.transcribe` pregunta al sistema sin abrir nada—), no empezó, preguntando, del modelo, respondiendo (el
  reloj corre en la pantalla desde lo que dijo Rust), evaluada (usaste · tenías y no usaste · «Sí lo dije» ·
  cuatro cifras), el informe y sin corpus. **Las teclas son de la ventana** y no se disparan con el foco en un
  selector ni con ⌘/⌃/⌥. El rail gana «Ensayo» bajo Sesión, y su chip dice «Ensayando · 0 B».
- **El contrato** gana sus muestras (`Preparacion`, `VistaDelEnsayo` en cinco fases, `NoEmpezo` en tres) y
  `src/ensayo.ts` entra en `DECLARACIONES` del gate de lectores.
- **Kit v3, la evaluación** (`ensayo.json`, bloque `respuestas`, test `el_kit_del_ensayo_mide_la_evaluacion`,
  que entra en «el kit, con su salida» de la CI).

### Lo que imprime el kit de la evaluación (corrida local, 2026-10-04)

| Respuesta | Fichas (usada → citada) | Ritmo | Muletillas |
|---|---|---|---|
| el retraso del ERP | Supuestos ✓ · Qué costó ✓ · Alcance (ni una ni otra) | 107 | «o sea» ×1 |
| de dónde salen las cuatro semanas | Plazo de entrega ✓ · Precio ✓ · Las cuatro etapas (no) · Aprendizajes fuera de las tres | 102 | «básicamente» ×1 |
| quién lo usa después | ninguna ficha: el disparo no encuentra dos términos | 105 | «digamos» ×1 |
| los números son correctos | ninguna ficha (objeción genérica) | 112 | — |
| la cuarta fuente, con otras palabras | **Alcance ✗ (paráfrasis)** · Precio ✓ · Entregables (no) | 113 | «o sea» ×1 |
| en inglés, a una pregunta en español | **Supuestos ✗** · Qué costó (no) · Alcance (no) | 112 | — (las del inglés no se cuentan en un ensayo en español) |
| no responde | las tres, ni usadas ni citadas | 140 | — |

**15 fichas, precisión 1,000, recall 0,714** (pisos 0,75 y 0,70, escritos antes de la corrida). La más lenta,
**4 ms** de buscar + armar + evaluar (presupuesto 500 ms). Las dos que falla están escritas para fallar: es lo
que «Sí lo dije» corrige, y el kit lo dice en vez de esconderlo. **Hallazgo:** con preguntas genéricas el
disparo no da fichas y la cifra sale «—»: la pantalla no acusa de nada que no tenías.

### Pruebas

- **Rust** (`AG_SIN_HARDWARE=1`): lib **541** (+26: sesión 10, oído 4, evaluación 8, ensayo 3, puerta de la
  captura 1; y el corte de 11 a 12) · contra el Mac 15 (+1, el kit de la evaluación) · ghost 5 · puerta 14 ·
  clippy limpio.
- **Interfaz:** vitest **411** (+14: 12 de `el-ensayo.test.tsx` y 2 del gate del log del ensayo; el rail con
  ocho y el corte con 12) · lint · typecheck.
- **e2e:** **264** (2 saltadas): axe en los ocho estados del ensayo, en los dos temas; reduced-motion: el reloj,
  las cifras y la pregunta se ven sin movimiento; el recorrido camina las ocho pantallas; `maqueta-cabe` cazó
  que «8 · textos» no cabía en su ventana (+101 px) y se partió en «8 · textos» y «9 · más textos».
- **Fidelidad:** **260 encuadres** (+28: siete estados del ensayo × dos temas × dos idiomas), ninguno sobre el
  umbral; el rail con «Ensayo» no movió ninguna de las otras pantallas. Leí como imagen la evaluada, la del
  modelo en inglés claro y la de «no empezó».
- `verify:ephemeral` con `oido.rs` dentro de `ensayo/`: cero API de disco o red.

### Los rojos (con `scripts/demo-rojo.sh`)

| Gate o prueba | Mutación | Quién lo nombró |
|---|---|---|
| sordo mientras lee y su cola | la cola de 300 ms a 1 ms | `el_microfono_esta_sordo_mientras_la_voz_lee_y_su_cola` |
| el reloj y el anillo juntos tras la sordera | el origen no se mueve al saltar audio | `sordo_no_oye_y_despues_el_reloj_sigue_cuadrado` |
| repetir la pregunta no cita | la unión con los términos de la pregunta en vez de la resta | `repetir_la_pregunta_no_cuenta_como_citar` |
| la puerta se cierra antes del micrófono | sin `cerrar_la_puerta_al_empezar` en el ensayo | `el_ensayo_abre_solo_el_microfono_y_cierra_la_puerta_antes` |
| la reunión corta el ensayo | `if false` en vez de `soltar_el_ensayo` en `empezar` | `la_captura_arranca_despues_de_la_puerta_y_solo_desde_empezar` |
| el corte corta el ensayo | la pieza `Ensayo` como «aún no existe» | `en_este_sprint_se_cortan_las_doce` |
| solo el micrófono (sobre la fuente) | un `Grifo::del_sistema(` plantado en `oido.rs` | `el_oido_solo_abre_el_microfono` |
| el piso del kit de la evaluación | `TERMINOS_PARA_CITAR = 4` | `el_kit_del_ensayo_mide_la_evaluacion`: «por debajo del piso» |
| el log del ensayo (término plantado) | `"[ensayo] … {cliente}"` en `lib.rs` | `logs-de-la-sintesis.test.ts` · «el log del ensayo» |
| Enter llega a Rust | Enter llamando a `ensayo_repetir` | `el-ensayo.test.tsx` |
| campos sin lector, ahora con `ensayo.ts` | `useReloj(0, …)`: `transcurridoMs` sin lector | `contrato-con-lectores.test.ts` |
| el porqué de la reunión | la franja de la reunión pintada para «sin corpus» | `el-ensayo.test.tsx` |

Las doce volvieron a verde tras restaurar (`demo-rojo.sh` con `--esperar-verde`); la carpeta `.demo-rojo/` no
quedó.

**Y un rojo que no plantó nadie (CI de `6000930`):** `build-escritorio` cayó en
`un_ensayo_entero_con_audio_inventado` —«de la pregunta al fin de tu voz: 3499 ms»—. La prueba usa el reloj de
verdad (el audio entra a ritmo de micrófono) y acotaba el tiempo arriba en 2,5 s; el runner de la CI, más lento,
dio 3,5 s. El código estaba bien y la prueba afirmaba de más: ahora afirma lo que no depende de la máquina
(al menos el segundo de voz, y nunca más de lo que de verdad pasó). Y la espera al hilo que transcribe en la
prueba del oído pasa de 0,5 a 2 s.

### Desviaciones de la fase

- **«Citada» exige términos que no estaban en la pregunta** (ADR 019 §6.6, actualizado): sin eso, repetir la
  pregunta citaba las tres fichas.
- **El informe** se pinta en esta fase con «Cerrar sin guardar»; **Guardar** y **Exportar** quedan apagados
  hasta la fase 4 (enmienda 4 del ADR 015), y la línea de la retención llega con ellos. Por eso la fidelidad
  del informe y del progreso entra en la fase 4.
- **Un ensayo no se bloquea por una videollamada abierta sin sesión**: solo una sesión (escuchando, solo
  notas o cerrándose) lo impide. Para la puerta local, en cambio, un ensayo cuenta como reunión.
- **FORMA y TEXTO nuevos** en `ensayo.html`, registrados «maquetada, no vista» (README de diseño): 1b · no
  empezó, 2b · del modelo, 8 · textos y 9 · más textos (incluido el aviso de un idioma que este Mac no
  transcribe, que la fase añadió a «preparar») y el chip
  «Sin sesión · 0 B» del rail, como el de las otras siete pantallas. Las vistas «sprint 3» de Honestidad
  cuentan 12 de 12.
- **Honestidad todavía no enseña el micrófono del ensayo** en «qué vive en memoria»: entra en la fase 4, con
  `ensayos/` (el chip «Ensayando» y el punto naranja de macOS lo dicen mientras tanto).

## Fase 4 · Resultado, progreso y lo que queda (C18)

Arranca el 2026-10-04 con el «continúa» del usuario tras la fase 3 (CI de `93d11a9`: los tres checks en
`success`, cada uno con su conclusión propia). **Primero la enmienda 4 del ADR 015**, antes de una línea de
código: `ensayos/` con la llave, el formato `.ghost` y la retención de tus notas; qué entra y qué no; guardar
sin pedir nada; abrir y exportar con el desbloqueo de tus notas; borrar sin abrir; un informe sin guardar no se
guarda solo al salir (decisión del constructor, declarada en la enmienda: guardarlo a escondidas registraría
launchd).

### Lo que se construyó

- **`ensayo/guardado.rs`** (protegido y puro): `Guardado` —cliente, propuesta, idioma, cuándo empezó y cada
  pregunta **a la que llegaste**, con tu respuesta en texto y sus cifras; jamás audio, ni las preguntas a las
  que no llegaste—; su informe sale de **la misma cuenta que la pantalla** (`sesion::informe_de`, que la fase
  sacó de `Sesion::informe`); el texto que se exporta, redactado en los dos idiomas; y **el progreso**: las
  cuatro cifras ensayo a ensayo, los seis más recientes, y «desde el primero» de cada cifra (del primer ensayo
  que la tiene al último: un ensayo sin ritmo o sin fichas no es punto de partida). Al soltarse, pisa tus
  respuestas.
- **`ensayos.rs`** (el escritor, molde de `bandeja.rs`): sella con la carpeta de `carpeta.rs` (misma llave, mismo
  `almacen`, 700/600), **cuenta por cliente por el nombre del archivo** sin abrir ninguno
  (`cliente_del_nombre`, que entiende `-2`, `-12` y clientes que acaban en cifras), abre para el progreso
  saltando el que no se deja abrir, borra los de un cliente, barre lo vencido y da sus pendientes a launchd. La
  línea de log dice cuánto, nunca qué.
- **El vencimiento:** `reunion::lo_que_vence_en(notas, bandeja, ensayos)` y el barrido de cada hora barre también
  los ensayos.
- **`lib.rs`:** `guardar_el_ensayo` (retención de tus notas, `poner_al_dia_el_vencimiento` después),
  `exportar_el_ensayo` y `progreso_del_ensayo` (con `reunion::desbloquear`, el desbloqueo de la pantalla, y su
  razón propia: «abrir tus ensayos guardados»), `borrar_los_ensayos`; `Preparacion.guardados`; y
  `estado_de_la_escucha` cuenta un ensayo (`EstadoDeEscucha::del_ensayo`: el micrófono del ensayo en la fila del
  micrófono y `bytes_del_ensayo`, tus respuestas). Los cuatro comandos, solo en la ventana principal
  (`build.rs`, `capabilities/default.json`, `SENSIBLES`).
- **`Ensayo` sabe lo que guarda:** `Rotulo` (cliente, propuesta y hora), `para_guardar()` —solo terminado y con
  el micrófono ya cerrado— y `memoria()`.
- **La pantalla:** el informe con «Guardar con tus notas», «Exportar como texto» y la línea de la retención
  (la de Notas, `nombreDeLaRetencion` compartida; «siempre» tiene su frase); si guardar o exportar fallan, el
  informe sigue y lo dice. Guardado, vuelves a «preparar» con «Ensayo guardado con tus notas.»; con uno o más
  guardados, «Ensayos guardados con este cliente · N · Ver tu progreso». **Tu progreso:** la tabla, «Desde el
  primero» con flecha y texto (↑ ↓ =, sin color de juicio), «Volver» y «Borrar los ensayos de este cliente»,
  que pregunta antes. Si el desbloqueo no se da: «No se abrieron tus ensayos · Siguen cifrados y en su sitio.»
- **Honestidad:** el micrófono del ensayo en su fila, **«Tus respuestas del ensayo»** como fila nueva y en la
  RAM, y «Tuyo» con tus ensayos. El chip del rail dice «Ensayando».
- **El efímero en marcha** (`una_sesion_completa`): un paso **7-ter** ensaya con el oído de verdad sobre un anillo
  que llena el test (sin abrir el micrófono) y **el motor de voz de verdad**, cierra con Enter, termina, guarda
  en `ensayos/`, y la lista de launchd tiene que traer notas, bandeja **y** ensayos. `Permitido` suma `ensayos/`,
  y **cada archivo de ahí se abre como un ensayo** con la llave de la sesión y sin la canaria
  (`revisar_un_ensayo`): un intruso dentro de la carpeta permitida es rojo. Esa revisión tiene su prueba sin
  hardware (`un_intruso_en_ensayos_se_delata`), porque la sesión completa solo corre en la CI.
- **El contrato:** `Progreso` (tres muestras: cuatro ensayos, uno solo, más de los que caben),
  `ESTADO_DE_LA_ESCUCHA_EN_UN_ENSAYO` y `Preparacion.guardados`.
- **Las maquetas** (FORMA y TEXTO, «maquetada, no vista», README de diseño): `ensayo.html` gana 1c · guardado,
  6b · borrar, 10 · textos de lo guardado y los botones de 6; `honestidad.html` gana «sprint 4 · ensayando».
  **«Tuyo» se acortó** («notas, acuerdos, fichas fijadas, propuestas guardadas, ensayos y, si lo enciendes, tus
  turnos. Cifrado.»): con «tus ensayos» ocupaba dos líneas y `maqueta-cabe` midió 17–20 px de más en tres estados
  con franja. Se midió cada redacción candidata en el navegador antes de elegir.

### Pruebas

- **Rust** (`AG_SIN_HARDWARE=1`): lib **555** (+14: lo guardado 6, el escritor 8; el ensayo de punta a punta
  afirma ahora lo que se guarda y la memoria que cuenta; launchd con tus ensayos) · contra el Mac **16** (+1, el
  intruso en `ensayos/`) · ghost 5 · puerta 14 · clippy limpio con `--all-targets`.
- **Interfaz:** vitest **423** (+12 en `el-ensayo.test.tsx`: guardar, su fallo, exportar y su fallo, la
  retención y «siempre», sin guardados no hay camino, el progreso con sus flechas, uno solo y más de los que
  caben, el desbloqueo que no se da, borrar que pregunta antes, Honestidad mientras ensayas y terminado; los
  formatos del día y la flecha) · lint · typecheck.
- **e2e:** **284** (2 saltadas): axe en guardado, tu progreso, borrar y Honestidad ensayando, en los dos temas;
  reduced-motion: la tabla del progreso se ve sin movimiento; `maqueta-cabe` cazó el «Tuyo» de dos líneas
  (arriba) y `maqueta-interaccion` pulsa los estados nuevos.
- **Fidelidad:** **280 encuadres** (+20: el informe, guardado, tu progreso, borrar y Honestidad ensayando, en los
  dos temas y los dos idiomas), ninguno sobre el umbral. **La primera corrida cazó una diferencia de verdad**: el
  producto escribía «↓ ritmo 161 ppm → 138 ppm» y la maqueta «161 → 138 ppm» (0,35–0,41 %); la unidad va una vez,
  al final. Leí como imagen el informe en inglés claro, «guardado» en oscuro y Honestidad ensayando.
- `verify:ephemeral` estático con `ensayo/guardado.rs` dentro: cero API de disco o red (57 archivos).
- **El efímero en marcha** (con el ensayo dentro) corre en la CI: sin hardware no se puede correr en local.

### Los rojos (con `scripts/demo-rojo.sh`)

| Gate o prueba | Mutación | Quién lo nombró |
|---|---|---|
| launchd se lleva tus ensayos | sin `.chain(ensayos.pendientes())` | `launchd_se_lleva_tus_notas_la_bandeja_y_tus_ensayos` |
| contar por cliente sin abrir | `starts_with` en vez de igual | `se_cuentan_y_se_borran_por_cliente_sin_abrirlos`: «"Páramo" contó los de "Páramo Azul"» |
| el progreso en orden | sin el `sort_by` | `el_progreso_va_del_primero_al_ultimo_y_cuenta_los_que_no_caben` |
| «0 de 0» no es punto de partida | la evidencia sin la condición de fichas | `sin_dos_ensayos_con_la_cifra_no_hay_cambio`: «contó "0 de 0"…» |
| lo guardado es texto y cifras | `palabras` sin `#[serde(skip)]` | `lo_guardado_es_texto_y_solo_lo_que_llegaste` |
| no se guarda sin terminar | `para_guardar` sin mirar la fase | `un_ensayo_entero_con_audio_inventado`: «se pudo guardar un ensayo sin terminar» |
| el log de guardar no nombra nada | el nombre del archivo en la línea | `el_log_de_guardar_no_nombra_nada` |
| la matriz de envejecimiento | `barrer(ahora + 1)` | `lo_vencido_se_barre_justo_al_vencer_y_siempre_no_vence`: «a -1 s del vencimiento» |
| Honestidad cuenta el micrófono del ensayo | `memoria()` con el micrófono a cero | `un_ensayo_entero_con_audio_inventado`: «la memoria del ensayo no se cuenta» |
| un intruso en `ensayos/` | la canaria sin `Err` | `un_intruso_en_ensayos_se_delata` |
| los comandos nuevos, solo la principal | `allow-progreso-del-ensayo` en la banda | `capabilities.test.ts` |
| guardar que falla lo dice | el fallo sin franja | `el-ensayo.test.tsx` · «si guardar falla…» |
| «siempre» no dice «se borra» | la condición sobre «1a» | `el-ensayo.test.tsx` · la retención |
| el progreso pide su cliente y tu idioma | el idioma cableado a «en» | `el-ensayo.test.tsx` · el progreso |
| borrar pregunta antes | borrar al primer clic | `el-ensayo.test.tsx` · «pregunta antes» |
| la RAM cuenta tus respuestas | sin `bytesDelEnsayo` en la suma | `el-ensayo.test.tsx` · «terminado, la RAM es exactamente tus respuestas» |
| el día en inglés | siempre «02 Oct» | `el-ensayo.test.tsx` · los formatos |

Las diecisiete volvieron a verde tras restaurar (`--esperar-verde`); `.demo-rojo/` no quedó. **Un gate que no podía
fallar, encontrado al buscarle el rojo:** la primera prueba de la RAM en Honestidad usaba el anillo de 1,8 MB al
lado de 412 B de respuestas, y quitar esos 412 B de la suma seguía dando «1,8 MB». Se añadió el caso «terminado»
(sin anillo), donde la RAM es exactamente lo tuyo, y ese es el que se vio en rojo.

### Desviaciones de la fase

- **Un informe sin guardar no se guarda solo al salir** de la app (tus notas sí): guardarlo sin pedirlo
  registraría la tarea de launchd a tus espaldas. Declarado en la enmienda 4 del ADR 015.
- **El camino al progreso aparece con un ensayo guardado**, no con dos: con uno, la tabla tiene una fila y no hay
  «Desde el primero» (la comparación sí pide dos). Así un ensayo suelto también se puede borrar.
- **«Borrar los ensayos de este cliente» vive en tu progreso**, no en Honestidad (la fila 3 de la matriz del plan
  decía «desde Honestidad»): Honestidad no borra nada en ninguna de sus filas, y el sitio donde ves tus ensayos
  es donde se borran, como «Borrar ahora» en Notas.
- **Exportar es del ensayo que acabas de terminar**, no de uno guardado (enmienda 4).
- **«Tuyo» se acortó** para caber en una línea, y entró también en los estados «de hoy» del sprint 3 de
  Honestidad (TEXTO, maquetado, no visto).
- **Una quinta protección en la corrida en vivo:** el desbloqueo (Touch ID o la contraseña) al abrir tu progreso
  o exportar. El plan decía que si aparecía una quinta cosa me detendría a enseñarla: va como fila 5 de la matriz,
  antes de la corrida.

### La corrida en vivo (filas 2 a 5 de la regla 22) — aplazada

Las cuatro matrices se enseñaron antes de correr nada: micrófono fuera de reunión (2), Llavero (3), launchd (4) y
desbloqueo (5). Antes de pedir los «sí» se comprobó el Mac sin pedir permiso: ninguna tarea de la app en
`~/Library/LaunchAgents` ni en `launchctl list`, y en su carpeta solo `diccionario.yaml`. **No se corrió ninguna fila.**
El usuario preguntó si se podían aplazar y se aplazaron (2026-10-04): **corte declarado, viaja al ⭐ del MVP**.

- **Lo que ya cubre la CI:** la sesión efímera completa con un ensayo dentro (`build-escritorio`, `ae64e3b`). Se
  guarda en `ensayos/` y se descifra, la canaria no está, y hay un solo archivo.
- **Lo que solo ve la corrida:** tu voz por el micrófono real; que la voz de la app por los altavoces no entre al
  micrófono; si los 2,5 s de silencio cortan pausas reales; el aviso del Llavero con tu llave; la tarea de launchd en
  tu Mac; Touch ID; la pantalla en la app de verdad.
- **Recomendación:** correrla antes de que el S5 toque el ensayo (≈ 5 min, con las mismas cuatro matrices).

## El merge sin el cierre (2026-10-04)

El usuario mergeó el PR #10 el **2026-10-04 a las 19:50 UTC** (`0009dba`, commit de merge), con las fases 0 a 4
construidas y la CI de `ae64e3b` en verde. La fase 4 no tuvo su parada. Fuera quedaron:

- la **fase 5**: manual, guía v6, kit v3 en la guía, `design-sync/` y nota del brochure;
- la **auditoría** v1.36.0;
- el **`/release-check`**;
- el **summary**.

El corte se declaró en el mismo acto, con un comentario en el PR #10. El cierre sigue en la rama
`sprint-004/cierre`, desde `main`, con su propio PR.

Dependabot #11 se mergeó 20 s antes que el #10, así que la mezcla de los dos se prueba por primera vez en la CI de
`main` (`0009dba`).

## Fase 5 · El cierre: manual, guía v6, kit v3 y `design-sync/` (rama `sprint-004/fase-5`)

Arranca el 2026-10-04 con el «continúa» del usuario, después de que mergeara el #10 (fases 0 a 4) y el #12 (el
registro del corte). La CI de `main` sobre los dos merges salió con sus tres checks en `success`.

### Lo que se construyó

- **Manual** (`docs/MANUAL-DE-USO.md`):
  - secciones nuevas: «El ensayo: practica antes de la reunión» y «Arriba o abajo: la banda junto a la cámara»;
  - frases que la banda arriba dejó falsas: «pegada al borde inferior», el asa «hacia arriba», el acople que
    «recorta por abajo» y la pregunta frecuente de compartir pantalla;
  - el corte pasa a 12 piezas; ⌃⌥B y las teclas del ensayo en los atajos;
  - la pregunta frecuente «¿El ensayo graba mi voz?» y la fila 004 del historial.

  Lo que no se ha probado en vivo se dice así:
  - el ensayo con una voz de verdad;
  - la banda arriba al compartir pantalla;
  - Zoom, Teams, una pantalla externa y la pantalla completa.
- **Guía v6** (`docs/GUIA-DE-PRUEBA.html`, namespace `ag-s4-`): 117 pruebas.
  - **16 nuevas** en dos bloques: Q, la banda arriba, y R, el ensayo.
  - **4 heredadas reescritas** (a1, b1, b5 y l2) tras releer las 101 contra la app de hoy:
    - la banda nace arriba;
    - el rail tiene ocho secciones;
    - la app en inglés incluye Ensayo;
    - el corte es de 12 piezas.
  - **Los bloques A a P y el ⭐⭐ del H1 se corren con la banda abajo**, como eran, y la preparación lo dice.
  - El ⭐⭐ del H1 no se mueve ni se renumera. Su parada 6 dice ahora «12 de 12», en su sitio.
  - **Gate mínimo ⭐: 94** (S1 33 · S2 28 · S3 23 · **S4 10**).
  - **Acumulado del H2: 10**, cada una con su candidatura al ⭐⭐ del H2: **5 sí** (q2, q3, r2, r3, r7) y
    **5 no** (q1, q5, r6, r10, r11).
  - Filtros nuevos: «⭐ del ciclo H2» y «Formas y textos del H2». La tabla U lleva 13 filas con lo que el S4
    maquetó y nadie ha visto.
  - Las cuatro protecciones de la corrida aplazada de la fase 4 (micrófono, Llavero, launchd y Touch ID) van en
    el bloque R y en la caja de avisos.
- **Kit v3** (`docs/kit-de-prueba/LEEME.md`): la evaluación de la fase 3, con su fila en la tabla y su sección:
  15 fichas, precisión 1,000, recall 0,714, 4 ms. Dos respuestas fallan a propósito, y es lo que corrige «Sí lo
  dije».
- **`design-system.md` v1.15.0:**
  - **§9-decies, la banda arriba**: el criterio de la cámara, `.banda[data-borde="arriba"]`, el asa abajo, el
    relleno, la fila de Sesión y el aviso de la primera vez;
  - **§9-undecies, el ensayo**: cifras y no notas, `.pregunta-e`, `.reloj-e`, `.respuesta-e`, `.evidencia-e`,
    `.cifras-e` y el progreso. Las fases 0 a 4 habían escrito el CSS y no la sección.
- **`design-sync/`:**
  - el generador acepta un modificador (`atributos`), porque la banda arriba es la misma banda con
    `data-borde`;
  - dos tarjetas nuevas: «La banda arriba, junto a la cámara» (88 · 200 · 44) y «El ensayo» (pregunta, evaluada
    y progreso);
  - el bundle se regeneró: 21 archivos, v1.15.0. Las dos tarjetas se leyeron como imagen en los dos temas, sin
    errores de página ni recursos de fuera.
- **Brochure:** en `main` no hay `docs/BROCHURE.html` ni `docs/brochure-export.json`. Llega por su orden aparte
  tras el Acto 1 del H1 (regla 13). Queda la nota para el summary: cuando nazca, ya trae el ensayo y la banda
  arriba.

### Las afirmaciones del manual y lo que las sostiene

| Afirmación | Gate |
|---|---|
| arriba, bajo la barra y el notch, nunca con una constante | `geometria::pruebas::arriba_nace_bajo_la_barra_de_menus_y_el_notch_nunca_con_una_constante` |
| arriba es de fábrica, ⌃⌥B alterna y se recuerda | `geometria::…::el_borde_de_fabrica_es_arriba_y_b_lo_alterna` · `prefs::…::lo_que_se_elige_sobrevive_al_reinicio` · `la-banda-arriba.test.tsx` |
| el asa se arrastra alejándose del borde | `asa.test.ts` (los dos bordes) |
| el aviso de la primera vez, hasta «Entendido» | `la-banda-arriba.test.tsx` · fidelidad `sesion-aviso` |
| baja y encoge; el borde de abajo no se mueve | `acople::…::arriba_la_reunion_baja_hasta_la_banda_y_su_borde_inferior_no_se_mueve` |
| solo la ventana de la reunión | `acople::…::arriba_de_una_lista_de_ventanas_sale_una_sola_decision` |
| por debajo de 240 px o en otra pantalla, no se toca | `acople::…::arriba_lo_que_ya_empieza_bajo_la_banda_o_esta_en_otro_monitor_no_se_toca` (y `QuedariaInservible`) |
| si la mueves tú, no se devuelve | `acople::…::arriba_no_se_devuelve_una_ventana_que_el_usuario_movio` |
| si macOS no la deja, se deshace y flota | **sin test unitario**: es nativo (`acoplar_arriba` relee con `quedo_bajo_la_franja`); en vivo solo se vio el camino feliz → ⭐ q5 |
| 230 ms y 120 ms en Meet | la corrida en vivo de la fase 1 (228 y 123 ms) |
| el ensayo solo abre el micrófono | `ensayo::oido::…::el_oido_solo_abre_el_microfono` |
| sordo mientras la voz lee | `ensayo::sesion::…::el_microfono_esta_sordo_mientras_la_voz_lee_y_su_cola` |
| se cierra con 2,5 s o con Enter | `…::la_respuesta_se_cierra_con_dos_segundos_y_medio_de_silencio_tras_tu_voz` · `…::enter_cierra_y_espera_a_lo_que_falta_por_transcribir` |
| R, S y Esc | `…::repetir_descarta_y_lo_que_llega_tarde_de_otra_ronda_se_tira` · `…::saltar_y_terminar_cuentan_lo_que_toca` |
| «usaste» por dos términos que no estaban en la pregunta | `ensayo::evaluacion::…::repetir_la_pregunta_no_cuenta_como_citar` · kit de la evaluación |
| «eh» y «um» no se cuentan | `…::las_muletillas_se_cuentan_por_frases_enteras_y_sin_las_que_no_lo_son` |
| sin cifra, «—» | `…::el_informe_suma_y_promedia_sin_inventar` |
| guardado con la llave y la retención de tus notas; el nombre dice el cliente | `ensayos::…::un_ensayo_nace_cifrado_cerrado_y_con_su_vencimiento` · `…::el_nombre_dice_el_cliente_y_nada_mas` · la sesión efímera en marcha (CI) |
| vence aunque no abras la app; «siempre» no vence | `ensayos::…::lo_vencido_se_barre_justo_al_vencer_y_siempre_no_vence` · `reunion::…::launchd_se_lleva_tus_notas_la_bandeja_y_tus_ensayos` |
| el progreso: solo cifras, los seis últimos y los anteriores contados | `guardado::…::el_progreso_va_del_primero_al_ultimo_y_cuenta_los_que_no_caben` · `el-ensayo.test.tsx` |
| borrar no abre ni desbloquea | `ensayos::…::se_cuentan_y_se_borran_por_cliente_sin_abrirlos` |
| enriquecer: hasta 5, fundadas, al final | `enriquecer::…::mas_de_cinco_se_cuentan_como_descartadas…` · `…::solo_se_funda_lo_que_nombra_una_seccion_dada…` · `banco::…::las_del_modelo_van_detras_y_sin_repetir` |
| el corte, 12 de 12 | `corte::…::en_este_sprint_se_cortan_las_doce` |
| no se ensaya con una sesión abierta; la puerta se cierra | las pruebas de la fase 3 (`NoEmpezo::EnReunion`, `en_reunion` cuenta el ensayo) |

### Pruebas

- `guia-cuadra` (11 pruebas) · `design-sync-espejo` (57) · vocabulario vetado, Llavero sin promesas y «abrir no
  existe» (7): en verde.
- La guía, abierta en un navegador sin cabeza a 1280 y a 375 px:
  - los siete filtros cuentan bien: Todo 150 · cambió en S4 20 · ⭐ 94 · H2 10 · ⭐⭐ H1 9 · textos H1 20 ·
    H2 13;
  - cero desborde, cero errores.
- Los bloques Q, R y U se leyeron como imagen.

### Los rojos (con `scripts/demo-rojo.sh`)

| Gate | Mutación | Quién lo nombró |
|---|---|---|
| candidatura del H2 (nuevo) | `q1` con `data-candidata="si"` y su línea en «no» | `guia-cuadra` · «expected [ 'q1' ]» |
| candidatura del H2 | `r7` sin `data-candidata` | «expected [ 'r7' ]» |
| la cabecera cuenta las candidatas | «6 sí · 4 no» | «[ 10, 10, 6, 4 ] … [ 10, 10, 5, 5 ]» |
| las filas del H2 | `u13` con el id de `u12` | «expected [ 'u12' ]» |
| el desglose con el S4 | «S4 9» | «[ 33, 28, 23, 10 ] … [ 33, 28, 23, 9 ]» |

Las cinco volvieron a verde tras restaurar, y `.demo-rojo/` no quedó.

### Encontrado al escribir la guía (para la auditoría; se pagan con ella)

1. **IA dice «Se cerró sola: hay una reunión» cuando la cierra un ensayo**, y no hay reunión
   (`src/pantallas/Ia.tsx:567`, `src/i18n/es.ts:864` y su par en inglés). Es una frase que el S4 dejó falsa.
2. **`sesion.funcionaBanda` («La banda, abajo, protegida de la captura») es una clave sin lector** que dice lo
   contrario de lo de hoy (`src/i18n/es.ts:331` y `en.ts`); solo la pintan los estados de historia s1/s2 de
   `sesion.html`.
3. **`Ensayo.tsx` lleva once `fontSize: 11.5` en línea**, copiados del `style` de `ensayo.html`. Son el valor
   mágico que el design system prohíbe en componentes (§3), y en ninguna otra pantalla aparecen.

## Auditoría · Fase 2: los 82 hallazgos, pagados (2026-10-04)

**Lo que dijo el usuario.** «apribada Fase 1, ahora las decisiones son pruebas pues claro que hay que hacerlas
pero claramente te dije si eran urgente sy me dijiste que podiamos seguir construyendo pues construyamos, si
publica en Github (No se publica ningun dato personal)». Se tomó como: Fase 1 aprobada; las seis decisiones,
con la opción recomendada (la A en todas), dicho en el mensaje y corregible; y el «sí» para publicar en GitHub
(B3). Las pruebas de la guía siguen aplazadas, como se acordó.

**Las seis decisiones, aplicadas:**

| Decisión | Lo que hace la app ahora |
|---|---|
| A1 · ensayar con una videollamada abierta | con altavoces no empieza y dice por qué; con auriculares —de cable, Bluetooth o USB— sí (desviación 27) |
| A4 · la guía y el acople abajo | la guía pide ⌃⌥B dos veces con Chrome delante (a2, a5, l5); la app, como en el H1 |
| M3 · arriba, Zoom o Teams sin llamada | arriba solo se acopla con la sesión iniciada; el latido no acopla arriba |
| M10 · un ensayo sin guardar al iniciar la sesión | Sesión lo dice encima de «Iniciar sesión» |
| M20 · la protección con la banda arriba | la app dice «verificado… con la banda abajo; con la banda arriba, y en Zoom y Teams, está sin verificar» |
| B46 · «Lo que llega en el H2» | «Lo que no hace hoy», sin chip (desviación 30) |

**Cómo se pagó.** En el orden de la Fase 1: primero lo que crea o amplía gates, después el resto, y al final
todos los gates sobre el árbol entero. Los documentos (manual, guía, README, BLUEPRINT, `design-system.md`,
`CLAUDE.md`, el LEEME del audio) los escribió un subagente en paralelo con la lista de la auditoría; el
constructor revisó lo que depende del código y corrigió la regla de los auriculares (desviación 27), que el
subagente había escrito con el ajuste literal.

**Los rojos de la Fase 2** (todos con `scripts/demo-rojo.sh`, cada uno restaurado y verificado con `grep` y
`cmp`; la carpeta `.demo-rojo/` no quedó):

| Hallazgo | Mutación | Quién cayó |
|---|---|---|
| M17 | `.progreso { opacity: 0 }` bajo reduced-motion | **primero pasó en verde**: `seVe` miraba la opacidad del elemento y no la de su caja. Se arregló el gate (multiplica la opacidad de la cadena); después, 2 rojos (reduce, banda y ventana principal) y 62 verdes |
| B5 | un test sin marca que llama a `sesion::ventana_de_la_reunion()` | `cargo-test-sin-hardware`: «demo_b5_lee_la_reunion (línea 1832) llega a ventana_de_la_reunion(» |
| B20 | `ruta_de` sin `nombre_valido` | **primero pasó en verde**: sin la carpeta `ensayos/`, la ruta `ensayos/../notas/…` no se resolvía. El test crea la carpeta; después, «se abrió un archivo de fuera de la carpeta» |
| M1 · B18 · M4 · M2 · A3 | quitar la comprobación de identidad · conformarse con 2 lecturas · sin la condición del fondo · `sin_deshacer` siempre `None` · `acoplar` sin turno | sus cinco tests del acople, por nombre («pub fn acoplar( no espera su turno») |
| A1 · B27 · A2 · A3 · M3 (×2) · M5 · B17 · B21 | la guarda quitada, en `ensayo/mod.rs` y en `lib.rs` | `con_una_videollamada…` y los seis de `pruebas_de_la_auditoria_del_s4` |
| M6 · M8 · M9 · M12 · M18 · B12 · B11 · B15 · B16 · B22 · B23 · M7 | la guarda quitada; `"tope_de_fabrica": 7`; un id inventado en `reglas.json`; las señales cableadas; el orden del latido al revés; `reunion` en vez de la huella; «sumó 0»; el `Drop` sin `pisar`; el `if ultima.es` fuera de Swift | su test, por nombre: «se volvió a leer el disco», «cero muletillas en nada se contó como cifra»… |
| M11 · M16 · M13 · M14 · M15 · B14 · B30 · B39 · B26 · A1 · B24 · B31 · M12 · B33 · B32 | la guarda quitada en `Ensayo.tsx` | `el-ensayo.test.tsx`, los quince |
| M15 (Notas) · B25 · M10 · M21 · B28 | la guarda quitada | `notas`, `la-banda-arriba` (×2), `lo-que-salio`, `el-ensayo` |

Las salidas enteras están en el scratchpad de la sesión (`demos-*.txt`); aquí, a quién nombró cada fallo.

**Los gates sobre el árbol entero (2026-10-04):** `pnpm typecheck` y `pnpm lint` limpios · Vitest **452** (53
archivos), con cobertura · `AG_SIN_HARDWARE=1 cargo test`: lib **575** (+2 ignorados), contra el Mac **16** (+14
para la CI), `ghost` **5**, puerta **14**, sin un aborto del centinela · `cargo clippy --all-targets -D warnings`
limpio · `pnpm verify:ephemeral` limpio · e2e **289 de 290** y fidelidad **284 encuadres** sin ninguno sobre el
umbral, con **un desborde** que cazaron los dos: el aviso nuevo de Sesión (M10), en dos líneas encima de
«Iniciar sesión», sacaba la pantalla 37 px (es) y 20 px (en) de su ventana. Pasó a una línea, en el sitio de
la promesa «⌥⎋ corta todo…», que vuelve al guardar o cerrar el ensayo; `maqueta-cabe` en verde después. La fidelidad, repetida tras el arreglo: 284 encuadres, ninguno sobre el umbral, ningún desborde.

**Pendiente del cierre:** el rojo de B9 en la CI (desviación 31) · B3 (publicar en GitHub, con el «sí») · B6,
B7 y B10 con el `/release-check` y el summary · la segunda casilla 4, con **otro** auditor, sobre el diff entero
y el summary.

**Dónde quedó (para retomar):** hechos el commit de la Fase 2 con su CI, B9 en la CI y B3 (sección siguiente).
Lo que falta, en orden:

1. El `/release-check`: el peso del binario (B7), `--release`, y su tabla en el summary.
2. `sprints/SPRINT_004-summary.md`, ya en borrador: llenar lo marcado con ⟨…⟩.
3. La segunda casilla 4 con **otro** subagente, sobre `adb2493..HEAD` y el summary; pagar lo que encuentre.
4. Todos los gates otra vez, commit, push y `gh pr checks`; PR listo y «Sprint 004 de Angel Ghost listo para mergear».

## El cierre: B9 en la CI, B3 en GitHub y el `/release-check` (2026-10-04)

**La CI de la Fase 2 (`f3dd820`):** `quality`, `e2e` y `build-escritorio` en `success`, cada uno con su conclusión
propia. `build-escritorio` (13 min 48 s) corrió la sesión efímera con el término plantado del ensayo y sus dos
comprobaciones nuevas: el término está en las secciones y el hijo llegó a «ensayo guardado».

**B9, su rojo en la CI** (desviación 31). Rama desechable desde `f3dd820` con un `println!` del texto de cada
sección de la propuesta al principio de `ensayo::banco::armar`, en el PR en borrador #14. En `build-escritorio`
cayó **solo** `la_canaria_del_cliente_no_aparece_en_el_log` (26 pasaron, 1 falló), y nombró la línea:
«lo del ensayo —la propuesta, la pregunta o la evidencia— salió por el log, en 1 línea(s): [demo-rojo] El hito
pterodaustro-de-escritorio-4419 abre el plazo…». `quality` pasó con el `println!` puesto: el lint estático no mira
el log, y por eso existe la canaria en marcha. El #14 se cerró sin mergear y su rama se borró.

**B3, publicado con el «sí» del usuario.** El cuerpo del #10 dice que se mergeó el 2026-10-04 (`0009dba`) con las
fases 0 a 4, con las casillas 1 a 4 marcadas y la 5 «fuera de este PR, va en el #13». El #12 tiene un comentario:
trajo solo el registro del corte (`2f48a58`), y la fase 5, la auditoría, el `/release-check` y el summary van en el
#13. Ningún dato personal.

**B10, los encuadres leídos como imagen.** Doce más, uno por bloque, de `docs/fidelidad/s4-cuaderno` y
`s4-banda-arriba`: preparar (oscuro, es), preguntando (claro, en), respondiendo (oscuro, en), evaluada (claro,
es), progreso (oscuro, es), borrar (claro, en), sin corpus (oscuro, en), no empezó (claro, es), el aviso de Sesión
de un ensayo sin guardar (oscuro, es), «lo que salió» en IA con la fila «para el banco» (claro, en), y la banda
arriba a 88 (oscuro, es) y a 200 (claro, en). Nada roto: el asa va abajo, la marca «sección conjeturada» se lee, y
el aviso de Sesión cabe en su línea. Con los seis de las fases 3 y 4, **18 encuadres de producto del S4**.

**El hallazgo del `/release-check`: el binario no se podía construir.** `pnpm tauri build --bundles app --no-sign`
se negó: «Found version mismatched Tauri packages… tauri (v2.11.6) : @tauri-apps/api (v2.12.1) ·
tauri-plugin-opener (v2.5.5) : @tauri-apps/plugin-opener (v2.7.0)».
- **De dónde viene.** Dependabot #11 (el lote de npm, mergeado 20 s antes que el #10) subió los paquetes npm de
  Tauri. Los crates de Rust no se movieron: dependabot solo vigila npm y GitHub Actions, por el techo de dos PR de
  la regla 18. La CI hace `cargo check` y `cargo test`, que no comparan nada con npm, así que todo salió verde.
- **El gate nuevo, `tests/unit/tauri-a-la-par.test.ts`:** lee `pnpm-lock.yaml` y `src-tauri/Cargo.lock` y exige la
  misma versión menor en cada pareja (`@tauri-apps/api` ↔ `tauri`, `@tauri-apps/plugin-*` ↔ `tauri-plugin-*`). Corre
  en `quality`, con `pnpm test`.
  - **Rojo 1, el estado real**, antes de arreglar nada: «@tauri-apps/api está en 2.12.1 y tauri en 2.11.6…» y
    «@tauri-apps/plugin-opener está en 2.7.0 y tauri-plugin-opener en 2.5.5…» (2 de 3 fallan).
  - **Un intento de demo que no valía:** mutar `pnpm-lock.yaml` dejó el lockfile roto, y `pnpm exec` lo reparó solo
    antes de correr el test, que pasó. `demo-rojo.sh` lo dijo («el gate pasó con la mutación») y restauró el archivo.
  - **Rojo 2, con `demo-rojo.sh` sobre `Cargo.lock`** (`tauri` 2.12.1 → 2.11.6), que es como pasó de verdad:
    «@tauri-apps/api está en 2.12.1 y tauri en 2.11.6: «pnpm tauri build» no construye así. Sube el crate con
    «cargo update -p tauri».» (1 de 3 falla). Restaurado con `grep` y `cmp`, y 3 de 3 en verde.
- **El arreglo:** `cargo update -p tauri --precise 2.12.1` y `-p tauri-plugin-opener --precise 2.7.0`. Arrastra a
  `tauri-build` 2.7.1, `tauri-runtime` 2.12.1, `tauri-utils` 2.10.1, `wry` 0.57.0, `tray-icon` 0.25.1 y otras
  transitivas. `cargo update -p` a secas no los movía: la subida pide dependencias nuevas.
- **Lo que se corrió con los crates nuevos:** `AG_SIN_HARDWARE=1 cargo test --locked`: lib 575 (+2 ignorados) ·
  contra el Mac 16 (+14) · ghost 5 · puerta 14, igual que antes. `cargo clippy --locked --all-targets -- -D
  warnings` limpio. `cargo test --release --locked --lib --test puerta --test ghost` (con `AG_SIN_HARDWARE=1`): 575 ·
  14 · 5, en verde. Vitest 455 (54 archivos), 92,6 % de sentencias y 81,4 % de ramas.
- **El peso (B7).** `pnpm tauri build --bundles app --no-sign`, ya con Tauri 2.12: ejecutable **13,42 MB** (S3: 12,79;
  **+0,63 MB**, por el ensayo, la banda arriba y Tauri 2.12, sin separar) · `ghost` 0,48 MB, dentro del `.app` · `.app`
  14,0 MB · imagen comprimida **6,77 MB** (S3: 6,27), medida igual, con `hdiutil create -format UDZO`. Sin `.dmg` de
  Tauri (AppleScript sobre Finder, regla 22) y sin firma (`--no-sign`: sin Llavero).
- **El resto de la lista:** `pnpm audit --audit-level high` limpio; `cargo audit` lo corre la CI sobre el `Cargo.lock`
  nuevo. `tauri.conf.json` e `Info.plist` sin cambios en el sprint.

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
11. Lo encontrado en la corrida en vivo (fase 1): la app espera a que la ventana se quede quieta antes de
    anotarla (`acople::asentar`), y soltar el asa sin arrastrarla no reacopla.
12. La llamada al modelo desde la app se cablea en la fase 3, con la pantalla (fase 2). Hecho en la fase 3.
13. «Citada» exige términos que no estaban ya en la pregunta (fase 3; ADR 019 §6.6 actualizado).
14. El informe se pinta en la fase 3 sin «Guardar» ni «Exportar», que llegan con la fase 4.
15. Una videollamada abierta sin sesión no impide ensayar; una sesión abierta, sí (fase 3). *La auditoría lo
    cambió (A1, desviación 27): con una videollamada abierta y el sonido por altavoces, el ensayo no empieza.*
16. Un informe sin guardar no se guarda solo al salir de la app (fase 4; ADR 015, enmienda 4).
17. El camino a tu progreso aparece con un ensayo guardado, no con dos; «desde el primero» sí pide dos (fase 4).
18. «Borrar los ensayos de este cliente» vive en tu progreso, no en Honestidad (fase 4).
19. Exportar es del ensayo recién terminado; uno guardado no se vuelve a exportar (fase 4).
20. Una quinta protección en la corrida en vivo de la fase 4: el desbloqueo de macOS, enseñado antes como fila 5.
21. La corrida en vivo de la fase 4 (filas 2 a 5) se aplazó: corte declarado, viaja al ⭐ del MVP.
22. El PR #10 se mergeó sin la fase 5, la auditoría, el `/release-check` ni el summary. El corte se declaró en el
    PR, y el cierre va en `sprint-004/cierre`.
23. La fase 5 va en `sprint-004/fase-5`, con su PR, porque el #12 se mergeó con solo el registro del corte.
24. El ⭐ del S4 deja 10 pruebas al acumulado del H2, no «~6». Son las seis de la orden más cuatro que el
    sprint se había comprometido a dejar en el ⭐:
    - Zoom, Teams, una pantalla externa y la pantalla completa, de la corrida en vivo de la fase 1;
    - guardar y limpiar, de la corrida aplazada de la fase 4;
    - enriquecer, que es juicio sobre lo que propone un modelo.
25. `design-system.md` gana sus secciones del S4 (§9-decies y §9-undecies) en el cierre, no en sus fases:
    las fases 0 a 4 escribieron el CSS y no la sección.
26. La guía v6 corre los bloques A a P y el ⭐⭐ del H1 **con la banda abajo** (se fija en la preparación y en la
    a1), y la parada 6 (l2) dice «12 de 12»: el corte suma el ensayo. Ninguna parada del ⭐⭐ del H1 se movió ni
    se renumeró (auditoría del S4, B2).
27. **A1 con la regla del H1.** La auditoría proponía parar el ensayo con una videollamada abierta salvo con
    auriculares «de verdad» (`puede_haber_eco() == Some(false)`): eso dejaba fuera los AirPods. Se para solo
    cuando la app **sabe** que se oiría (altavoces del Mac, HDMI, DisplayPort, AirPlay), como la voz en reunión
    (`habla::cabe_decirla`); Bluetooth y USB pasan, y el manual dice que con un altavoz así no se ensaye con una
    llamada abierta. Y nace un segundo porqué, «No se puede saber si hay una videollamada», para el navegador
    abierto sin Accesibilidad (ADR 019, enmienda 1).
28. **M7 se vigila en la fuente de Swift, no con los altavoces.** En `cargo test` nadie atiende la cola principal:
    `didFinish` y `didCancel` no llegan nunca, y el test con la voz de verdad que proponía la auditoría pasaba con
    el fallo puesto (la tercera pregunta de la regla 15). Lo de verdad va al ⭐ (R durante la lectura).
29. **B19 lo paga M3:** arriba, el latido del arranque ya no acopla, así que no hay bucle que mueva y devuelva
    una ventana que pelea su sitio.
30. **B46 quita también el chip «En el H2»** de las filas de Corpus e Idioma: bajo «Lo que no hace hoy», el
    chip seguía prometiendo. En IA se queda (MLX, que respalda el ADR 011).
31. **El rojo de B9 se ve en la CI**, en un PR desechable: el test de la canaria del log toca el reconocimiento
    de voz y en local no se corre.
32. **Los crates de Tauri suben a 2.12 en el cierre** (`tauri` 2.12.1, `tauri-plugin-opener` 2.7.0, con sus
    transitivas), para casar con lo que dependabot subió en npm: si no, `pnpm tauri build` no construye la app.
    Lo encontró el `/release-check`, y lo vigila un gate nuevo, `tauri-a-la-par`. Bajar los de npm no se puede:
    `verificar-dependencias` (regla 18) no deja nada por debajo de `main`.
