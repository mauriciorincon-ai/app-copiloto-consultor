---
sprint: 003
app: copiloto-consultor
status: closed
opened: 2026-09-27
closed: 2026-10-03
branch: sprint-003/el-cuaderno-y-el-cierre
pr: https://github.com/mauriciorincon-ai/app-copiloto-consultor/pull/8
acto: "1 de 2 (de construcción). El Acto 2 —⭐⭐, la prueba en vivo y el sello MVP— está pendiente"
---

# Sprint 003 Summary — Angel Ghost

## Outcome

**Sí en construcción: con este merge, el MVP personal (H1) está construido. No está visto funcionar en el
Mac del usuario: esa prueba y el sello van al Acto 2, con el corte declarado abajo.**

- **Lo tuyo queda; del cliente, solo lo que aceptes, en una línea (C9).** Notas, acuerdos y fichas fijadas, y (opt-in) tus turnos en
  texto, en **un archivo por reunión, cifrado**, con la retención que eliges. La app **propone qué guardar**
  con cinco reglas publicadas, y lo no decidido espera en una **bandeja con cuenta atrás** que se borra sola
  al vencer, aunque la app esté cerrada (launchd, al minuto exacto). Audio, transcript del cliente y lecturas
  de pantalla siguen muriendo: `verify:ephemeral` en marcha descifra notas y bandeja y no encuentra la
  canaria del cliente.
- **El marco en la mano (C11).** Jurisdicción por cliente con fuente y fecha, cláusula modelo es/en
  copiable y chequeo de NDA que lleva a **modo solo notas** (sin transcripción, sin pantalla).
- **La puerta local para Claude Code (C16).** `ghost`, cerrado por defecto; un socket Unix en una carpeta
  700, con una llave nueva y su propio desbloqueo por cada apertura; se cierra sola en reunión; sin red.
- **El cierre del ciclo H1.** `docs/BLUEPRINT.html`, la auditoría del `CLAUDE.md` contra el código, el
  manual sin «todavía no» del H1, la guía v5 con el ⭐⭐ en ~20 min, el kit v2 en la CI, `design-sync/`
  regenerado y las preferencias que sobreviven al reinicio (deuda del S2).

**Lo que no se afirma.** Nada de lo que toca las protecciones del Mac se ha visto funcionar con la app
viva, en el Mac del usuario: la llave de las notas en el Llavero, Touch ID al exportar y en `ghost`, la
tarea de launchd registrada desde la app, la bandeja fuera de Time Machine, el cuaderno protegido en Meet y
los indicadores apagados en solo notas. Los tests lo cubren por partes; verlo entero es el Acto 2.

## El corte, declarado (método v1.24.0)

| Qué queda fuera de este merge | Por qué | Cuándo |
|---|---|---|
| **La prueba en vivo** (sus siete filas de la regla 22, una por acción, con su «sí») | La regla 15 (tercer filo) pide ver cada modo funcionando en vivo antes de cerrar el sprint. **Decisión del usuario (2026-10-03), «sigue · B»:** mergear con el corte declarado y correrla en la misma sentada que el ⭐⭐. Se le presentaron las dos opciones con su costo | Acto 2, con el ⭐⭐ |
| **B5** (¿macOS pide el permiso de Reconocimiento de voz?) | Solo lo responde la app viva | Acto 2, con la prueba en vivo |
| **El resto de la maniobra del `design-system.md` §10** | Primero en la lista de lo que se corta; el corte no se declaró a tiempo y lo encontró la auditoría (B28) | H2 (ADR 008, enmienda del 2026-09-28) |

**Lo que cuesta:** `main` recibe código que el usuario no ha visto funcionar en su Mac, y el cierre del Acto 1
queda **condicionado** hasta el Acto 2. **El riesgo real es bajo:** la app no se distribuye (binario sin
firmar, solo en este Mac), y lo que falle se arregla en el PR de correcciones del Acto 2.

## Qué se construyó

| Fase | Qué | Dónde |
|---|---|---|
| 0 | Delta del kit v1.31.0 (reglas 10, 15, 21 y 22; comandos; skill `ia-embebida` §9; rojo de `scrollHeight`). **Un solo escritor** para todo lo que persiste: nace 600 dentro de 700, con `fsync` y renombrado. Llavero compartido. **Preferencias que sobreviven al reinicio** (deuda S2). **B37 del S2:** IA enseña el texto exacto que salió al API y cuántos datos tapó la bóveda. Embeddings → H2 con su condición (ADR 008). WER `manual`, corrido en local | `almacen.rs` · `llavero.rs` · `prefs.rs` · `sintesis/api.rs` · `Ia.tsx` |
| 1 | **C9, tus notas** (ADR 015): `notas/` en RAM y protegido; XChaCha20-Poly1305 con la llave en el Llavero; un archivo por reunión en la carpeta privada de la app (decisión A); desbloqueo con LocalAuthentication; ⌃⌥N y ⌃⌥P; **el cuaderno, protegido mientras hay sesión**; «Conservar mis turnos», apagado por defecto y sin eco; pantalla Notas | `notas/` · `carpeta.rs` · `reunion.rs` · `desbloqueo.rs` · `Notas.tsx` |
| 2 | **Propuestas por reglas publicadas y bandeja** (ADR 016): cinco reglas en `data/propuestas/reglas.json`; del cliente, un hecho en una línea, jamás el turno; la propuesta pasiva en la banda; bandeja cifrada con ventana elegible; **launchd al minuto exacto de cada vencimiento** (`/bin/sh` borra lo vencido de la lista; sin proceso entre vencimientos) | `propuestas/` · `bandeja.rs` · `vencimiento/` |
| 3 | **C11** (ADR 017): catálogo de jurisdicciones con fuente, fecha y «sin verificar»; «Este cliente» en Sesión; cláusula es/en; NDA → **modo solo notas** | `jurisdiccion/` · `data/jurisdicciones/` · `modo.rs` · `Sesion.tsx` |
| 4 | **C16** (ADR 018): `puerta/` (socket Unix 600 en carpeta 700, llave por apertura, comparada en tiempo constante, lista cerrada de órdenes, cierre en reunión, registro sin contenido) y `bin/ghost.rs` | `puerta/` · `src/bin/ghost.rs` · `Ia.tsx` |
| 5 | BLUEPRINT · auditoría del `CLAUDE.md` · manual y README · guía v5 · kit v2 (propuestas, jurisdicciones, el kit por la puerta) · `design-sync/` (tres tarjetas) · el barrido de tokens vetados que debía nacer en el S1 | `docs/` · `design-sync/` · `tests/unit/tokens-vetados.test.ts` |
| Auditoría | **68 hallazgos** (45 de la Fase 1 y 23 de la segunda pasada de la casilla 4): 67 pagados y B5 como deuda del Acto 2 (§ Auditoría). Seis decisiones del usuario, entre ellas la de los proveedores: **Gemini sale**, Claude y Groq se quedan con su aviso | 8 commits, `689f81f` … el del cierre |

## DoD — checklist

- **Testing** ✅
  - vitest **354** (48 archivos), con cobertura: 91,6 % sentencias y 81,9 % ramas. Suelos propios para
    `propuesta.ts` (80 %) y `Notas.tsx` (60 / 50).
  - `cargo test --lib` **479** (+1 `#[ignore]`) · `puerta` **14** · `ghost` **5** · `contra-el-mac-de-verdad`
    **24** en la CI (+3 `#[ignore]`, regla 22) · e2e **206** en la CI.
  - **Regla 19:** cada evento nuevo Rust→TS entra por `contrato.rs`, con fixture del serializador real y
    tipo generado: notas, cuaderno, bandeja, jurisdicción, puerta y lo que salió.
  - Casilla 6 sobre la guía v4: 20 pruebas que la arquitectura contradecía, reescritas. Kit v2 en la
    CI; el WER, `manual`.
  - `gh pr checks` tras cada push. **Cada gate nuevo con su rojo en el mismo commit** (bitácora).
- **CI/CD** ✅ — `quality` · `e2e` · `build-escritorio` en `success` propio en cada push de la auditoría y
  del cierre. **Corrió por primera vez en este PR:** el paso `cargo audit` dentro de `build-escritorio`
  (M12). Su rojo se vio en un PR desechable (#9, RUSTSEC-2021-0003), cerrado sin mergear. **Este summary
  viaja en el PR.**
- **Observabilidad** ✅
  - Logs con solo metadatos. La puerta registra orden, resultado y motivo, jamás contenido.
  - Los errores que se loguean no nombran al cliente: las rutas salen como `[ruta]`
    (`los_errores_que_se_loguean_no_nombran_al_cliente`).
  - La canaria del cliente no aparece en el log ni en las notas o la bandeja descifradas.
- **Seguridad** ✅ con avisos aceptados
  - `pnpm audit` limpio · `cargo audit` sin vulnerabilidades, con avisos *unsound* (`lru` vía tantivy,
    `glib`): deuda.
  - gitleaks en cada commit.
  - Archivo de notas cifrado con la llave en el Llavero, sin contraseña en archivo. **Corrección de la
    auditoría (A2):** la llave vive en el llavero *de inicio de sesión*, y viaja con él; «ligada a este
    Mac» exige el llavero de protección de datos y la app firmada (H2).
  - `verify:ephemeral` estático y en marcha con notas y bandeja en juego; fuga inyectada vigente.
  - Puerta: token por apertura en tiempo constante; denegada en reunión (test); contador de red en 0 con
    la puerta abierta (`el_contador_de_red_no_se_mueve`); sockets Unix solo en `puerta/` y `bin/ghost.rs`
    (`puerta-solo-local`, rojo con un `TcpStream` plantado).
  - **Proveedores con su retención leída y decidida** (ADR 011).
- **Performance** ✅
  - Propuesta: el turno más lento del kit tarda **719 µs** (tope 50 ms; presupuesto 1 s).
  - Abrir una reunión de 100 000 letras: **≤ 500 ms** (test).
  - Binario: § Métricas.
- **UX/A11y** ✅
  - axe en los estados nuevos, dos temas.
  - Teclado de punta a punta; símbolo + texto + color (la cuenta atrás lleva su símbolo).
  - Fidelidad: **216 encuadres**, ninguno sobre el umbral, **ningún desborde**.
  - **La mirada 19, vista y con veredicto** antes de construir encima. Las **20, 21 y 22**, desde el «menos
    paradas» del usuario (2026-09-27), quedaron maquetadas y registradas «no vistas» y van al gate del
    MVP: **no es una aprobación, y se dice** (`docs/diseno/README.md`).
- **IA embebida** ✅ — sin LLM nuevo. B37 del S2, pagada. **Gemini sale** por decisión del usuario, con la tabla de
  retención delante; IA dice qué guarda Claude y qué guarda Groq.
- **Manual** ✅ — cada feature del sprint, bilingüe en la app, con sus gates; barrido por promesa aplazada;
  lo que queda se dice H2.
- **Guía v5** ✅ — 101 pruebas, las 72 de la v4 heredadas enteras (33 «Mejorado en S3»); ⭐ **84** (S1 33 ·
  S2 28 · S3 23); ⭐⭐ **9 paradas, ~20 min**, que deja fuera 75 ⭐ declaradas; bloque de **textos diferidos**,
  20 filas, ~10 min; prefijo `ag-s3-`; kit de prueba v2.
- **ADRs** ✅ — 015 · 016 · 017 · 018, cada uno antes de su fase. Enmiendas: 002 (2–7), 008 (embeddings y
  maniobra), 011 (B37 del S2, retención y decisión), 015 (1–3), 016 (2–3), 018 (1).
- **Cierres de ciclo** ✅ — BLUEPRINT · auditoría del `CLAUDE.md` (§ abajo) · `design-sync/` regenerado (la
  publicación, en el Acto 2, invocada por el usuario) · el brochure inicial llega por su orden.
- **`/audita-sprint`** ✅ con un pendiente declarado — independiente; 67 de 68 pagados, B5 en el Acto 2;
  artefacto con `archivo:línea`; casilla 4 dos veces, la segunda con otro auditor y sobre este summary.

## Métricas técnicas

| Métrica del plan | Meta | Medido |
|---|---|---|
| La propuesta llega tras el turno | ≤ 1 s | turno más lento **719 µs** (kit de propuestas, CI de `1deefbb`) |
| Propuestas por reglas | las esperadas | **15 de 15**, precisión y recall 1,00; del cliente, jamás el turno |
| Abrir el archivo cifrado | ≤ 500 ms | **≤ 500 ms** con 100 000 letras (`abrir_una_reunion_tarda_menos_de_medio_segundo`) |
| Jurisdicciones contra el catálogo | todas | **17 de 17** (catálogo v1, consultado 2026-09-17) |
| El kit por la puerta | lo mismo que directo | nDCG@5 **0,823** por la puerta y directo |
| Red con la puerta abierta | 0 | **0** (`el_contador_de_red_no_se_mueve`) |
| ⭐⭐ | ≤ ~20 min | **9 paradas, ~20 min** (v4: ~25) |
| WER | `manual` | corrida local de la fase 0: mezcla-es 0,458 → 0,417 · mezcla-en 0,348 → 0,261; el runner no tiene modelos de voz |
| Peso del binario | anotado | ejecutable de release **12,79 MB** (S2: 12,06 → **+0,73 MB**) · `ghost` 0,48 MB, dentro del `.app` · `.app` 13,4 MB · imagen comprimida 6,27 MB (S2: 5,66, medida igual) — `pnpm tauri build --bundles app --no-sign`, 2026-10-03 |

## Gate ⭐ — diferimiento y contrapesos

**Opción B: ⭐⭐ OBLIGATORIO.** No corre en el Acto 1: es el Acto 2 del cierre de ciclo, y el usuario lo
correrá **junto con la prueba en vivo** (decisión del 2026-10-03). El ⭐ largo (84 pruebas, ~145 min) se
ofrece sin fecha.

| Contrapeso | Evidencia (archivo, cuenta medida, corrida) |
|---|---|
| Pasada de capturas del builder | **216 encuadres** producto contra maqueta, dos temas y dos idiomas · `docs/fidelidad/S3-banda.html` · `docs/fidelidad/S3-cuaderno.html` · `pnpm fidelidad` 2026-10-03 tras el último cambio de interfaz (la segunda pasada de la casilla 4): ninguno sobre el 0,15 %, **ningún desborde**. Leídos como imagen en cada cierre de fase y, al final, IA, la cláusula, Notas y Permisos. **El gate de desbordes cazó en M3 una línea que dejaba IA 51 px más alta que su ventana** |
| e2e de `reduced-motion` | **42 pruebas** (10 encuadres × 2 modos + la forma del árbol, en 2 proyectos) · `tests/e2e/reduced-motion.spec.ts` · job `e2e`. Sin cambios en el S3: el sprint no añadió movimiento (ni `animation` ni `transition` en el diff) |

**Qué corre en el Acto 2, en una sentada (~60 min):**

1. **La prueba en vivo** (~30 min), con sus siete filas de la regla 22: arrancar y recordar el corpus · una
   reunión con la página de Meet de prueba (y B5) · las primeras notas guardadas (Llavero, ítem en segundo
   plano, bandeja fuera de Time Machine) · exportar con Touch ID · la puerta y `ghost` · la bandeja que vence
   con la app cerrada · solo notas sin indicadores.
2. **El ⭐⭐** (9 paradas, ~20 min).
3. **Los textos diferidos** (20 filas, ~10 min).

Por bloques, con arreglo en caliente; cada resultado, a la bitácora.

## Decisiones no anticipadas

- **Decisión A (2026-09-27):** las notas, en la carpeta privada de la app y no en Documentos (ADR 015,
  enmienda 1): las notas no piden permiso de Documentos ni pasan por iCloud.
- **El vencimiento lo cumple launchd al minuto exacto**, no cada 5 minutos. Vino de la objeción del usuario
  al consumo (ADR 016).
- **El cuaderno, protegido mientras hay sesión.** El invariante del S1 pasa a ser: «la banda siempre, el
  cuaderno en sesión, el relleno jamás».
- **Regla 22 (2026-09-27):** las protecciones del Mac se enseñan antes de tocarlas. Nació de una falla del
  constructor (§ Bugs).
- **Plan de miradas:** «menos paradas». Solo para lo que cambia una decisión, la promesa o el Mac; la FORMA y
  el TEXTO nuevos se maquetan y van al gate del MVP «no vistos».
- **Las seis decisiones de la auditoría (2026-09-28/29):**
  - A2: textos corregidos, el llavero de protección de datos va al H2.
  - M4: la puerta pide su desbloqueo **en cada apertura**.
  - M2: la bandeja sale de Time Machine; **las notas, dentro**.
  - M12: **`cargo audit`** en la CI, sin dependabot de Cargo.
  - B29: **la carpeta del corpus se recuerda** y se reindexa al arrancar.
  - M3: **Gemini sale; Claude y Groq se quedan con su aviso.**
- **La regla dura 2 queda más estrecha de lo que dice.** «Bajo proveedor con no-retención» solo lo cumple
  Groq, con su retención cero encendida; Claude guarda hasta 30 días. El usuario lo decidió con la tabla
  delante, y la cláusula modelo dejó de prometer «no retención». Va a la planeadora (§ Sugerencias).
- **El Acto 1 sin la prueba en vivo** (§ El corte).

## Bugs + resoluciones

Los que importan. El detalle, con sus rojos, está en la bitácora.

- **Los 68 hallazgos de la auditoría** (§ Auditoría). Los dos altos de la Fase 1:
  - **A1:** dos reuniones del mismo cliente el mismo día **se pisaban la bandeja**.
  - **A2:** la llave de las notas **no estaba «ligada a este Mac»** como decían cuatro documentos.
- **El constructor tocó las protecciones del Mac sin avisar, dos veces.**
  - Una prueba de launchd dejó «sh · desarrollador no identificado» en Ítems de inicio, y `sfltool` pidió la
    contraseña de administrador seis veces.
  - Después, un `cargo test` completo abrió el micrófono y el audio del sistema.
  - Origen de la regla 22; en local solo corre `cargo test --lib --test puerta --test ghost`.
- **Un rojo de CI por tiempo, no por deriva** (`dfeee23`): el generador de `design-sync/` pasaba los 5 s en
  el runner. El generador ahora parsea una vez por página, y el test tiene su límite declarado.
- **Una preferencia guardada con Gemini devolvía todas las preferencias a fábrica** (NDAs y retención
  incluidas). Lo vio el test antes de que existiera el caso: ahora solo vuelve el proveedor, con el API
  apagado.
- **Pruebas decorativas cazadas al exigirles el rojo:**
  - el residuo de `propuesta.test` (plantillas en el orden equivocado);
  - la exención de `abrir-no-existe`, que casaba con la extensión `.ghost`;
  - el test del cifrado, que encontraba su propia aguja en un comentario;
  - el bloque del corpus en `sin-todavia-no`, que devolvía vacío.

## Qué salió bien / qué generó fricción

- **Bien:**
  - **El auditor independiente encontró los dos altos** que la suite en verde no veía: los dos vivían en
    lo que el código prometía, no en lo que hacía. Y el segundo auditor encontró dos más en el texto
    que se le entrega al cliente y en una promesa de invisibilidad que nadie había comprobado.
  - **Las decisiones con su tabla delante** (la retención de cada proveedor, con fuente y fecha) dejaron
    decidir en un mensaje.
  - **El gate de desbordes** cazó que el aviso de retención del proveedor dejaba IA 51 px más alta que su
    ventana, antes de que nadie lo viera.
- **Fricción:**
  - **Las protecciones del Mac.** Dos fallas del constructor y la desconfianza que dejaron.
  - **La superficie que toca el Mac creció** (Llavero, launchd, Touch ID, Time Machine, socket) mientras
    verla en vivo dependía del tiempo del usuario. El sprint cierra sin haberla visto.
  - **Seis decisiones de la auditoría** llegaron en jerga, y el usuario pidió que se las explicaran.

## Sugerencias de mejora al método

1. **Regla dura nueva para TODAS las apps: las protecciones del Mac se enseñan ANTES de tocarlas.** Va con
   matriz de una fila (qué · para qué · qué aviso vas a ver · cómo se deshace) y un «sí» por acción. En esta
   app es la regla 22 del `CLAUDE.md`. Se propone al kit y a `/audita-sprint` como casilla: «¿qué protección
   del Mac tocó el sprint y dónde está el “sí”?».
2. **`cargo test` a secas no debería tocar el Mac.** Los tests contra el hardware van detrás de una marca
   (`#[ignore]` o una *feature*), para que el comando por defecto sea seguro. En esta app, la segunda falla
   fue un `cargo test` corrido para verificar.
3. **La parada la abre solo lo que cambia una decisión del usuario, la promesa o su Mac** (tercera vez que
   la regla 10 choca con el usuario). La FORMA nueva se maqueta, se registra «no vista» y va al gate del MVP.
4. **La regla dura 2, reescrita:** «proveedor que publica su retención, leída con fecha y fuente, y la app
   la enseña donde se elige», en vez de «con no-retención», que hoy solo un proveedor cumple, y con un
   interruptor que la app no puede ver. La tabla se relee antes de cada release (estándar 7).
5. **En apps de escritorio, la prueba en vivo del cierre de ciclo y el ⭐⭐ comparten sentada.** Las dos
   necesitan el Mac y al usuario delante. Planearlas juntas evita el corte de este sprint o una segunda
   sentada.
6. **Las decisiones de la auditoría, en llano desde la primera vez:** qué pasa con cada opción, con su costo.
7. **La segunda pasada de la casilla 4 la hace otro auditor independiente**, y sigue cada ajuste hasta sus
   frases hermanas, incluido este summary (como en el S2).

## Deuda técnica aceptada

| Qué | Por qué | Pago |
|---|---|---|
| La prueba en vivo y **B5** | decisión del usuario (§ El corte) | Acto 2, con el ⭐⭐ |
| El resto de la maniobra §10 | corte declarado tarde (B28) | H2 |
| Llavero de protección de datos para la llave de las notas (`kSecUseDataProtectionKeychain`) | exige la app firmada; hoy la llave viaja con el llavero de inicio de sesión, y la documentación lo dice | H2 (G-Release) |
| Firma y notarización: la tarea de launchd se ve como «sh» en Ítems de inicio; `NSAudioCaptureUsageDescription` sin verificar | sin firma no se puede | H2 (G-Release) |
| `lru` 0.16.4 *unsound* vía tantivy; `glib` *unsound* | sin subida posible hoy; `cargo audit` los vigila en la CI | cuando suban sus dependientes |
| Embeddings + RRF | condición escrita en el ADR 008 | H2, si se cumple |
| MLX | Apple Intelligence cumple (ADR 011) | roadmap H2 |
| `ghost` fuera del PATH; servidor MCP | ADR 018 | H2 |
| La regla dura 2, más estrecha que su texto | decisión del usuario sobre Claude | la planeadora decide (§ Sugerencias 4) |
| Ver un `invoke` prohibido rechazado desde la consola de la banda (S2, M5) | la consola del webview no es accesible desde la sesión del builder | Acto 2 |
| WER en la CI | el runner no tiene modelos de voz | `manual`, una corrida local por sprint |
| El aviso de Documentos sin la frase de la app, si la carpeta del corpus vive en Documentos (la app la relee al arrancar, B29) | `NSDocumentsFolderUsageDescription` se quitó con la decisión A; el aviso, si sale, sale sin la frase que pide la regla dura 7 | Acto 2, junto con B5 |

## Auditoría

`/audita-sprint` con **auditor independiente**: un subagente que no construyó el sprint, sobre el diff
`main...HEAD` en `3325b19`, en solo lectura. **Por la regla 22, no corrió nada que toque el Mac.** El
artefacto es `sprints/SPRINT_003-auditoria.md`: cada hallazgo con su `archivo:línea`, su ajuste y su estado
(lo vigila `auditoria-con-sitio`).

- **Fase 1:** **45 hallazgos — 0 críticos · 2 altos · 13 medios · 30 bajos.** Veredicto «requiere
  ajustes». El constructor verificó los dos altos en el código antes de enseñarlos.
- **Aprobación del usuario** (2026-09-27, «apruebo auditoria») y sus seis decisiones (§ Decisiones).
- **Fase 2:** **44 pagados** en siete commits (`689f81f` · `77f79cc` · `fdf7ba3` · `eb9044d` · `a8aad3c` ·
  `bbebbce` · `1deefbb`): **27 con su rojo antes del verde** y 17 de texto o declaración, sin gate que
  demostrar (B2, B3, B6, B8, B10, B13, B14, B18–B24, B28, M9, M10). **B5 queda como deuda del Acto 2**,
  porque solo la responde la app viva.
- **Los altos:** A1 (la bandeja se pisaba) y A2 (la llave no estaba ligada al Mac). **Los medios que más
  pesaban:**
  - M3: la retención de los proveedores, nunca leída;
  - M4: la puerta heredaba el desbloqueo de la pantalla;
  - M7 y M8: la app decía «todavía no» de lo que existe y «abrir» de lo que no existe;
  - M12: nada vigilaba las dependencias de Rust;
  - M13: la pantalla de Notas, sin tests.
- **Casilla 4, dos veces:** la primera en la Fase 1; la segunda, después del último ajuste, con otro auditor
  independiente, sobre los textos de alcance **y este summary** (resultado abajo).
- **Casilla 5:** campos del contrato sin lector, contados por el gate `contrato-con-lectores`; los que el
  auditor encontró, pagados (B13 y B14).
- **Las dos casillas 6:**
  - la guía heredada, releída contra la arquitectura en la fase 0: 20 que la contradecían (10 imposibles
    y 10 caducadas); con otras 13 de punto de partida nuevo, 33 «Mejorado en S3»;
  - los números cableados: B11 (`topes-en-el-texto`) y B25 (las cifras de la cabecera de la guía).

**La segunda pasada de la casilla 4 (2026-10-03)**, con otro auditor independiente en solo lectura, que
siguió cada pago hasta sus frases hermanas e incluyó este summary: **21 frases falsas hoy — 2 altas · 10
medias · 9 bajas** (A3–A4 · M14–M23 · B31–B39). A ellas se suman B40, cuatro comentarios de código que el
auditor vio de paso, y B41, que encontró el constructor en el `/release-check` (`ghost` sí viaja dentro del
`.app`). **Las 23, pagadas.** Un control final del mismo auditor sobre los propios pagos encontró seis
restos —el peso del binario que el BLUEPRINT daba por no medido, hermanas de B41, la cuenta de
capabilities de este summary, filas de textos diferidos que no nombraban el texto nuevo, la apertura del
manual y el titular del Outcome—, pagados en el mismo commit.

- **A3, la cláusula modelo que se entrega al cliente:** prometía anonimización completa y que solo se
  conservaban las notas. Ahora dice qué tapa la app y que también se conservan tus turnos, si lo eliges,
  y las propuestas que aceptes.
- **A4, la invisibilidad del cuaderno:** el manual decía «en Meet está comprobado» y nadie lo había
  comprobado (regla dura 6). Ahora dice que se protege con el mismo flag que la banda, sin verificar, y la
  interfaz dice «protegida al compartir» en vez de «el cliente no la ve».
- **Los medios:**
  - «del cliente, nada» (M15–M17), que las propuestas aceptadas desmienten;
  - «no hay copia en otro sitio» (M18), que Time Machine desmiente;
  - «Buscar a mano — Todavía no» (M19), que Solo notas ya hace;
  - «abrir» en el BLUEPRINT (M20);
  - «Documentos ya no se pide» (M21) y «señala antes tu carpeta» (M22), que el corpus recordado desmiente;
  - dónde está la línea de retención (M23);
  - y este mismo summary, que decía «44 pagados, cada uno con su rojo» (M14).
- **Tres gates crecieron** para que esas frases sean rojos la próxima vez, cada uno con su rojo antes de
  arreglar el texto: `abrir-no-existe` lee el BLUEPRINT y `desbloqueo.rs`; `sin-todavia-no` cubre «Buscar a
  mano»; `llavero-sin-promesas` caza «la llave no viaja».

**Cuenta final: 68 hallazgos — 0 críticos · 4 altos · 23 medios · 41 bajos. 67 pagados (30 con su rojo y 37
de texto o declaración) y B5 en el Acto 2.**

## Auditoría del `CLAUDE.md` contra el código (método v1.24.0)

La hizo un subagente de solo lectura; el constructor verificó cada dato en el código. **27 correcciones.**
Entre ellas:

- qué persiste y qué no, pieza por pieza;
- los módulos protegidos reales;
- el Stack de verdad: Core Audio, huella por zonas, VAD por energía, BM25, `println!` sin contenido;
- los checks reales de la ruleset;
- el árbol real en vez de la plantilla web;
- los doce patrones de dominio, que estaban sin llenar;
- y el barrido de tokens vetados de la regla 5b, que **no existía** y se construyó.

**Sin deriva:** las dos casas, las reglas duras 2 y 4 a 8, `captura_terceros` y las reglas 9, 11, 12, 15, 16 y
18 a 22. **Después de la auditoría** cambiaron la regla dura 1 (la ruta del corpus en las preferencias), la regla
22 (`cargo test` a secas) y las reglas 2 y 3 y el Stack (los dos proveedores). La tabla completa está en la
bitácora.

## `/release-check` (perfil escritorio)

**✅ Pasa 10/12 · ⚠️ 2 con aviso · ❌ 0.** Los avisos no bloquean el merge: son el corte declarado y la deuda
de `cargo audit`.

| # | Casilla | Resultado |
|---|---|---|
| 1 | Tests | ✅ vitest 354 con cobertura (91,6 % sentencias) · e2e 206 en la CI, cero flaky · `cargo test --locked` en `build-escritorio` (lib 479 · `contra-el-mac-de-verdad` 24 · `puerta` 14 · `ghost` 5) |
| 2 | Tipos y lint | ✅ `pnpm typecheck` · `pnpm lint` · `cargo clippy --locked --all-targets -- -D warnings` limpio, en local y en la CI · tokens vetados (`tokens-vetados`) · reduced-motion (`tests/e2e/reduced-motion.spec.ts`) |
| 3 | Build del binario | ✅ `pnpm build` · `cargo check --locked` en la CI · **`manual`:** `pnpm tauri build --bundles app --no-sign` en local, 2026-10-03: ejecutable **12,79 MB** (S2: 12,06 → **+0,73 MB** por notas, cifrado, propuestas, jurisdicciones, puerta y vencimiento) · `ghost` 0,48 MB, dentro del `.app` · `.app` 13,4 MB · imagen comprimida 6,27 MB (S2: 5,66 MB, medida igual con `hdiutil create -format UDZO`). **El `.dmg` de Tauri no se construyó:** su paso usa AppleScript sobre Finder y puede abrir un aviso de Automatización (regla 22); por eso la imagen se midió con `hdiutil`, que no lo pide. Firma omitida a propósito (`--no-sign`: sin Llavero). **`--release`:** `cargo test --release --locked --lib --test puerta --test ghost` en verde (479 · 14 · 5). `tauri.conf.json` e `Info.plist` sin cambios; las capabilities ganan 32 comandos en la ventana principal y uno en la banda (`ir-a-notas`), vigilados por `capabilities.test.ts` |
| 4 | Permisos TCC | ⚠️ `NS*UsageDescription` es/en para micrófono, audio del sistema y reconocimiento de voz; la app arranca y se usa sin permisos (S1–S2). **Lo nuevo del S3** —Llavero de las notas y de la puerta, Touch ID, ítem en segundo plano— **no se ha visto con la app viva:** corte declarado, Acto 2. B5 decide si la clave de reconocimiento de voz se queda |
| 5 | Ventana protegida | ✅ `content_protected` en la banda siempre y en el cuaderno durante la sesión, con su test (el invariante reescrito con rojo); la parada ⭐⭐ k1 mira el cuaderno al compartir. **Que funcione en Meet se ve en el Acto 2** |
| 6 | No persistencia | ✅ `pnpm verify:ephemeral` estático; la sesión completa en marcha en la CI (`una_sesion_completa`), con notas y bandeja **descifradas** y sin la canaria del cliente; fuga inyectada vigente; canaria en el log; contador en 0 B en modo local y con la puerta abierta |
| 7 | Seguridad | ⚠️ `pnpm audit --audit-level high` limpio · `cargo audit` **sin vulnerabilidades**, con avisos *unsound* (`lru` vía tantivy, `glib`): deuda, sin subida posible hoy; corre ya en la CI (M12) · gitleaks · CSP · capabilities por ventana · sockets Unix solo en la puerta (`puerta-solo-local`) |
| 8 | Observabilidad | ✅ metadata-only; los errores logueados no nombran al cliente (`[ruta]`); la puerta registra orden y resultado, jamás contenido |
| 9 | A11y y diseño | ✅ axe en los estados nuevos, dos temas · teclado · `design-sync/` regenerado y su espejo verde · **`scrollHeight`:** `pnpm fidelidad` sin desbordes en 216 encuadres (cazó uno en M3, arreglado) |
| 10 | Documentación y cero enlaces | ✅ manual y guía v5 al día; summary en el PR · barrido `git grep -nE "vercel[.]app\|workers[.]dev\|pages[.]dev" -- ':!pnpm-lock.yaml'` vacío tras el último `git add`; ningún enlace de descarga del binario |
| 11 | Checks del PR | ✅ `quality` · `e2e` · `build-escritorio` en `success` propio en cada push (el del último commit se lee antes de entregar) |
| 12 | El disco en runtime | ✅ inventario (`Permitido` de `contra-el-mac-de-verdad.rs` y el BLUEPRINT): índice del corpus (700) · `diccionario.yaml` · `costo-del-mes.json` · `preferencias.json` · `acople.json` (600, mientras la banda está acoplada) · notas `*.ghost` cifradas (600 en 700) · bandeja cifrada (600, fuera de Time Machine) · la lista de vencimientos (600) · el plist de launchd · el socket de la puerta (solo abierta, en 700). Nada fuera de la lista, y el gate en marcha lo demuestra |

**Decisión: MERGE OK con el corte declarado**, en cuanto la CI del último push cierre en verde. La prueba en
vivo y el ⭐⭐ son el Acto 2.

## Archivos clave

`src-tauri/src/notas/` · `src-tauri/src/carpeta.rs` · `src-tauri/src/bandeja.rs` ·
`src-tauri/src/vencimiento/mod.rs` · `src-tauri/src/puerta/` · `src-tauri/src/jurisdiccion/` ·
`src-tauri/src/almacen.rs` · `src/pantallas/Notas.tsx` · `docs/BLUEPRINT.html` ·
`sprints/SPRINT_003-auditoria.md`

## Cómo probar

1. Arranca con `pnpm ghost && AG_BANDEJA_DE_PRUEBA=1 pnpm tauri dev`.
2. Abre la guía `docs/GUIA-DE-PRUEBA.html` (doble clic) y elige el filtro **Gate corto ⭐⭐**: 9 paradas,
   después el bloque **Textos diferidos**. La caja **«Los avisos de macOS que vas a ver»** dice cada aviso
   que puede salir, incluido el de B5; cualquier otro, deniégalo.
3. Hace falta:
   - auriculares;
   - Terminal (el cliente de prueba habla con `say`);
   - `docs/kit-de-prueba/pantalla/meet-de-prueba.html` abierta en Chrome;
   - Apple Intelligence activado;
   - Claude Code para la puerta.

Automático: `pnpm test` · `pnpm test:e2e` · `cd src-tauri && cargo test --lib --test puerta --test ghost`
(el resto, `contra-el-mac-de-verdad`, lo corre la CI: abre el micrófono y hace sonar los altavoces, regla 22).
