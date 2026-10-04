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
