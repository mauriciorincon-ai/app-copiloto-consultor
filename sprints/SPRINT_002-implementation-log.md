# Bitácora — Sprint 002 «Cuándo, qué y quién mira»

Branch `sprint-002/cuando-que-y-quien-mira`, desde `main` en `7625683`. Orden
`SPRINT_002-orden.md` (aprobada 2026-09-23) · plan autoritativo `SPRINT_002.md` · kit v1.28.0 ·
método v1.30.0 · ciclo H1, sprint **2 de 3**.

Plan de fases aprobado el 2026-09-24, con su bloque de arranque y el «construye» del usuario.

---

## Desviación del plan — lo primero que pidió el usuario al arrancar

El plan que se aprobó traía una desviación numerada 4: **cerrar como «no reproducibles» los quince
hallazgos medios y bajos del sprint 001** cuyo detalle no existe en ningún archivo del repo. El
usuario la rechazó en el mismo mensaje del «construye», y con una razón que cambia el sprint:

> _«esto no puede pasar nunca más, los hallazgos se deben resolver al finalizar el sprint; siempre
> al finalizar el sprint se buscan hallazgos y siempre me caracterizo por resolver todos, hasta los
> bajos»_.

**Qué cambia, entonces:**

1. Los quince **no se cierran**. Sus superficies entran en el alcance de la `/audita-sprint` del S2,
   que por el kit v1.28.0 ya exige `archivo:línea` en todas las severidades: lo que siga siendo
   cierto volverá a salir con su ubicación, y se paga.
2. La fase 6 planea tiempo para **pagar también los medios y los bajos**, no para declararlos.
3. Y lo que impide que se repita **no es una buena intención, es un gate** — abajo.

---

## Fase 0 · El gate del artefacto de auditoría

### Por qué existe

El auditor del S1 entregó 21 hallazgos medios y bajos. Al escribirlos en el artefacto del repo se
resumieron en una sola frase —«**M3, M5, M6, M7, M8, M12, M13, M14** y **B1–B7**: ver el detalle en
la bitácora de ajustes»— con un puntero que además estaba **roto**: la bitácora tampoco los tenía.
Quince hallazgos desaparecieron con lo único que los hace pagables, que es dónde están.

`tests/unit/auditoria-con-sitio.test.ts` le exige dos cosas a todo `SPRINT_NNN-auditoria.md`:

1. **cada severidad declara su cuenta y la cuenta cuadra** — `## MEDIOS (14)` obliga a catorce
   filas, de `M1` a `M14`;
2. **cada hallazgo dice dónde** — un `archivo:línea` en su fila o dentro de su sección, y la única
   salida es marcarlo `irrecuperable` con su razón, que es una confesión escrita y no un silencio.

### Su rojo no hubo que fabricarlo: el artefacto del S1 estaba roto hoy

```
× sprints/SPRINT_001-auditoria.md — cada hallazgo dice DÓNDE, o se declara irrecuperable
  M11 · M2 · M4 · M9
× sprints/SPRINT_001-auditoria.md — la numeración no tiene huecos
  M3 (medio) · M5 (medio) · M6 (medio) · M7 (medio) · M8 (medio)
```

### Y el agujero que el propio gate tenía, encontrado al leer ESA salida

La primera versión buscaba los identificadores uno a uno (`**M12**`) y comprobaba que no hubiera
huecos entre los que encontrara. Mirando el rojo se ve lo que le faltó: denunció `M3` y `M5–M8`
—huecos por debajo del máximo que veía— **y no dijo ni una palabra de `M12`, `M13`, `M14` ni de los
siete `B`**. Estaban escritos dentro de una sola negrita agrupada, así que para ese gate no
existían; y como no existían, la numeración que él veía era continua y perfecta.

**Un gate que mide lo que el documento le enseña es un gate que el documento puede engañar.** La
versión que quedó cuenta contra **la cuenta que el propio documento declara de sí mismo**: si el
encabezado dice catorce, hay que enseñar catorce filas. Y por eso un encabezado sin cuenta —`##
ALTOS`, tal como estaba— también es rojo: un documento que no dice cuántos tiene no se puede cuadrar
con nada. El segundo rojo, ya con la regla nueva, sacó los diecisiete que faltaban y los dos
encabezados mudos.

Una tercera vuelta lo hizo menos rígido en lo que debía: `C1` vive como **sección** (`### C1 · …`)
porque un crítico merece prosa, y obligarlo a caber en una celda sería empujarlo a resumirse, que es
justo el defecto que este gate persigue. Ahora un hallazgo puede ser fila, viñeta o sección, y el
sitio se busca dentro de su trozo.

### Qué se arregló en el artefacto del S1

`## CRÍTICO` y `## ALTOS` ganan su cuenta —**(1)** y **(10)**—, y la sección de medios y bajos pasa
de una frase a **veintiuna filas**: las seis que tenían detalle con su `archivo:línea` y su estado
(M11 **pagado**; M1, M2, M4, M9 y M10 deuda con su fase de pago), y las quince restantes **marcadas
una a una como irrecuperables**, con una nota de cabecera que dice qué pasó y remite a la auditoría
del S2. Verde: 3 de 3, y la suite entera en 153.

**Lo que este episodio deja dicho:** el S1 escribió el gate de contrato porque un defecto se coló
entre dos lenguajes. Este es el mismo movimiento aplicado al método: el artefacto de auditoría
también es un puente —entre el auditor y el sprint que viene—, y también necesitaba su gate.

---

## Fase 0 · El delta del kit (v1.27.1 + v1.28.0)

El estampado de esta app fue con **v1.27.0**, así que el salto real son **dos** versiones, no una.
Y la mayor parte del batch **nació aquí**, en el cierre del S1: los perfiles de Playwright y Vitest
de escritorio, el `afterEach(cleanup)`, `cargo clippy` dentro de `build-escritorio`,
`verify-ephemeral` y el propio artefacto de auditoría ya estaban. Lo que sí faltaba:

| Qué | Dónde |
|---|---|
| **Regla 19 — gate de contrato entre lenguajes** | `CLAUDE.md:418`. **No existía**: la lista terminaba en la 18. Su origen es el C1 de este repo, así que se cita con el precedente de casa y con la advertencia de lo que NO cubre (la forma, no si alguien lee) |
| **Regla 15, cuarto filo: el modo incluye el perfil de compilación** | `CLAUDE.md:348` |
| **Regla 20 — el artefacto de auditoría se cuadra solo** | `CLAUDE.md:435`. No es del kit: es la petición del usuario de este sprint, escrita como regla de la casa |
| Sección fija «Gate ⭐ — diferimiento y contrapesos» en la plantilla del summary | `CLAUDE.md:546-551` |
| `/audita-sprint` — artefacto con todas las severidades · casilla 4 repetida tras la Fase 2 | `.claude/commands/audita-sprint.md` |
| `/release-check` — la inversa de la regla v1.15.0, `clippy --locked`, `manual` en `tauri build`, casilla de `--release` | `.claude/commands/release-check.md` |
| `testing-patterns` regla 10 — carpeta temporal única por test | `.claude/skills/testing-patterns.md:167` |
| Fuera `--pass-with-no-tests` | `package.json:14` |
| El CHANGELOG al día | `CHANGELOG.md` |

### El rojo del `e2e` con cero pruebas

La regla del kit es que **un job que no corre nada no puede dar verde**. Quitado el flag, se
comprueba pidiéndole a Playwright un filtro que no existe:

```
$ pnpm test:e2e --grep "una-prueba-que-no-existe"
Error: No tests found
[ELIFECYCLE] Command failed with exit code 1.
```

Con `--pass-with-no-tests` eso era un verde. Los 66 e2e de verdad siguen pasando.

**La regla 10 de `testing-patterns` llega con deuda pagada por adelantado:** este repo aprendió esa
lección tres veces en el S1 —audio, corpus y la canaria de la CI— y la cuarta la encontró el
`/release-check`, cuando el inventario del efímero acusó al compilador. El arreglo de entonces
(excluir las carpetas de la herramienta) es exactamente lo que la regla ahora exige por defecto.

---

## Fase 0 · M10 — el audio no se reinterpreta a ciegas

El callback del tap leía los bytes del sistema como `f32` **sin comprobar que lo fueran**
(`capture/nativo.rs:402-403`). Si Core Audio negociaba otra cosa —entero de 16 bits, por ejemplo,
que depende del dispositivo— cada muestra habría salido de los bytes de dos muestras distintas. Y
esto es lo que lo hace feo: **no es un fallo ruidoso, es ruido**, y el detector de voz lo habría
tomado por sonido.

La comprobación va **al abrir el grifo**, no en el callback: el callback corre en un hilo de tiempo
real, donde ya es tarde para negociar nada y lo único que se puede hacer es devolver sin tocar nada.
`es_float32_empaquetado()` exige `lpcm`, 32 bits y las banderas de flotante y empaquetado, y si no
se cumple el grifo **no abre**, con el error nombrado y las cuatro letras del formato — para lo cual
se extrajo `cuatro_letras()`, que `formato_de_error` ya hacía a mano.

Y un cinturón dentro del callback para lo que el formato no dice: que **este** búfer traiga un
número entero de muestras y empiece donde un `f32` puede empezar. Un `from_raw_parts` desalineado no
es un número raro: es comportamiento indefinido.

**Demo en rojo** (el defecto de M10 replantado, `es_float32_empaquetado` devolviendo siempre `true`):

```
thread '…::solo_se_abre_el_grifo_si_el_audio_llega_como_flotante_de_32_bits' panicked at
src/capture/nativo.rs:798:13: entero de 16 bits no se puede leer como f32
test result: FAILED. 0 passed; 1 failed
```

**Y el primer dividendo de haber metido clippy en la CI (S1, `/release-check`):** el cinturón del
callback lo escribí con `% != 0` y clippy lo paró en seco —`manual implementation of
is_multiple_of`— antes de que llegara a ningún sitio. Un gate que se ganó el sueldo en su primer
cambio ajeno.

---

## Fase 0 · El payload que el gate del contrato no miraba

`corte::Informe` —lo que el kill-switch devuelve— **no tenía `rename_all`**. Llegaba al webview como
`bytes_en_red` mientras TypeScript habría esperado `bytesEnRed`: **el mismo defecto que el C1 del
sprint 001**, vivo, en el único payload que el gate nacido para cazarlo no miraba, porque esa struct
no estaba en `contrato.rs`.

Sobrevivió por una razón que vale la pena escribir: los **cuatro** suscriptores del evento `corte`
lo usan como **señal** y ninguno lee el payload. Un contrato roto que nadie usa no se nota — hasta
que alguien lo usa.

Así que se arregló usándolo:

1. `Informe` gana `#[serde(rename_all = "camelCase")]` y **entra al contrato** (`INFORME_DEL_CORTE`).
2. Nace el comando `piezas_del_corte`, que devuelve lo que el corte **haría**, leído de
   `corte::TODAS` y de su `match` sin comodín.
3. **Honestidad deja de afirmar y pasa a leer.** Tenía dos constantes, `PIEZAS_CORTADAS = 6` y
   `PIEZAS_TOTALES = 7`, con un comentario que confesaba el atajo: *«si algún día se separan, lo que
   hay que arreglar es que este lado lo pregunte»*. El día llegó con la deuda del S1.

**Demo en rojo** (quitarle el `rename_all` y regenerar el contrato):

```
src/contrato.generado.ts(200,5): error TS2353: Object literal may only specify known properties,
and '"bytes_en_red"' does not exist in type 'InformeDelCorte'.
```

Eso es exactamente el C1, cazado por `pnpm typecheck` en la orilla que lo lee. En el sprint 001 ese
mismo defecto necesitó un auditor independiente.

**Y el otro huérfano: `documentos_del_corpus` se retira.** El comando existía, estaba registrado y
**no tenía ni un llamador**; la maqueta de Corpus tampoco dibuja ninguna lista de documentos, así
que no es una pantalla pendiente de cablear sino API muerta. La struct `Documento` se queda: el
corpus la usa por dentro. Cuando el S3 diseñe la lista, el comando vuelve **con su entrada en el
contrato**.

---

## Fase 0 · El quinto motivo — `por_silencio`, cableado

La VISION nombra cinco disparos: *«una pregunta, un término tuyo, una cifra o un silencio disparan la
ficha; y un atajo global "ayúdame con esto"»*. En el sprint 001 el quinto se escribió, se probó y
**no se conectó**: `Disparador::por_silencio` (`disparo/mod.rs:121`) tenía sus dos llamadores en sus
propios tests y ninguno más, así que la app no disparó por silencio en toda la vida del sprint. El
manual lo declaró como limitación, que fue lo honesto que se podía hacer entonces.

### Dónde tenía que ir, y por qué no fue donde el plan decía

El plan lo mandaba al **latido de 40 ms** (`escucha/mod.rs:272-287`), por una razón correcta: es el
único bucle que tictaquea cuando nadie habla. Al abrirlo se ve que ese hilo **no conoce al buscador
ni a la ventana del transcript** —solo mira marcos de audio y manda encargos—, y llevárselos habría
duplicado el cableado que el otro hilo ya tiene entero.

El sitio bueno es el **hilo que transcribe**, que ya tiene los dos: bastaba con que dejara de esperar
el encargo siguiente para siempre. `recv_timeout` cada **400 ms** (diez por ciento del umbral de
cuatro segundos, imperceptible al lado de los 320 ms que cuesta decidir un fin de turno), y el hilo
sigue durmiendo en el canal el resto del tiempo: no es un bucle nuevo, es el mismo con un
despertador.

### Tres cosas que aparecieron al cablearlo

**Una · `por_silencio` recibía una cadena vacía, y eso estaba mal.** `aceptar("")` se salta la
guardia contra repetidos —una consulta vacía no se compara con nada— **y además pisa la última
consulta con «»**. Dos consecuencias, las dos invisibles sin llamador: el silencio que sigue a una
pregunta ya contestada habría puesto una **segunda ficha idéntica** en la banda cada seis segundos, y
la pregunta siguiente del cliente, aunque fuera la misma de antes, habría vuelto a disparar. Ahora
recibe **lo último que dijo el cliente**, que es lo que hace funcionar la guardia — y con eso el
silencio dispara justo por lo que no disparó solo, que es para lo que existe.

**Dos · la voz del cliente no se copia.** Lo último que dijo ya vive en la ventana del transcript,
que es el sitio que el kill-switch alcanza; `Ventana::ultimo_de` existía desde el S1 con un comentario
que decía *«es lo que la fase 4 preguntará»* y tampoco tenía llamador. La búsqueda se hace sobre una
**referencia**, con el candado de la ventana puesto, para no dejar una segunda copia en un hilo al
que `cortar()` no llega. El precio es que quien consulte el estado en ese instante espera lo que dure
la búsqueda; en esta app ese es el lado correcto del trato. Los dos candados se anidan
—ventana → disparador— y es el único sitio de la app donde eso pasa: se comprobó que ningún otro
camino los toma en el orden inverso.

**Tres · mientras el cliente habla no hay silencio.** Su turno anterior cerró hace rato y el que está
en curso todavía no tiene fin, así que el reloj diría que lleva callado justo cuando no lo está. Se
pregunta a la pista del sistema (`turnos.hablando()`) antes de mirar el reloj.

### El gate, y el agujero que tenía mi primer gate

El primer test que escribí prueba `el_silencio_pide_ficha` —el ayudante— de punta a punta: corpus
indexado, frase que no dispara sola, silencio, ficha correcta, y no insiste. Verde.

**Y con el cable cortado ese test sigue verde.** Probar el ayudante y no el cable habría sido dejar
exactamente la misma deuda del sprint 001 —código probado sin llamador— esta vez con un test verde
encima tapándola. Así que el latido salió del hilo a `ElQueTranscribe::latir`, por la misma razón por
la que `atender` ya vivía fuera, y hay un segundo test que cruza el cable de verdad: canal real,
`recv_timeout` que vence sin encargos, y la aparición saliendo por `avisar`, que es el camino por el
que la banda se entera.

**Demo en rojo** (borrada la llamada a `el_silencio_pide_ficha` del brazo del `Timeout` — el estado
exacto en que el sprint 001 lo dejó):

```
test escucha::tests::el_cliente_se_queda_callado_y_la_ficha_llega_sin_que_nadie_hable ... ok
test escucha::tests::el_latido_sin_encargos_saca_la_ficha_del_silencio_por_donde_la_banda_la_oye ... FAILED
  panicked at src/escucha/mod.rs:1422:13: el latido no sacó la ficha del silencio: []
test result: FAILED. 16 passed; 1 failed
```

Dieciséis en verde y uno en rojo: el del ayudante pasó con el cable cortado, y eso es la medida de
para qué sirve cada uno.

### De propina

- `armar_y_anunciar` — los dos caminos que disparan comparten el armado, la medida y la línea de log.
  Dos copias habrían acabado midiendo distinto, y la medida es la que el presupuesto de 4 s acota.
- **`Ventana::esta_vacia` se retira**: misma clase que `por_silencio` —`pub fn` sin llamador fuera de
  sus tests— y aquí no hay nada que cablear, porque `cuantos()` ya dice lo mismo.
- **El manual deja de declarar la limitación** y describe lo que hace, incluidas las dos cosas que el
  usuario nota: que no repite la ficha y que no cuenta como silencio mientras el cliente habla. La
  nota histórica de la auditoría del S1 queda, fechada, con el cable declarado.

---

## Fase 0 · M4 — la banda vuelve

`⌥⎋` cierra la banda: es una de las siete piezas del corte, y está bien que lo sea. Lo que estaba mal
es que **no había manera de recuperarla sin reiniciar la app**. `ventana::abrir_banda` existía, tenía
su comando registrado en el `invoke_handler`, y **ni un llamador en `src/`**.

**Dónde va el llamador, y por qué no es un botón.** Es `empezar_a_escuchar`. La banda es donde la
ficha aparece, así que empezar una sesión sin banda es empezar una sesión sin ningún sitio donde
enseñar nada; poniéndolo ahí el invariante —hay escucha ⇒ hay banda— queda en **un solo lado**, y no
en cada rincón de la interfaz que se acuerde de pedirla. Un botón nuevo, además, habría sido un
estado visual nuevo y esto no es una fase con mirada.

**Y `abrir_banda` tenía que volverse idempotente**, porque `build()` no admite una etiqueta repetida:
sin eso, «que la banda vuelva» habría sido un error cada vez que la banda no se hubiera ido. Se mira
**ventana por ventana** y no «si falta alguna, las dos»: si alguna vez quedara el relleno sin su
banda, lo que hay que reponer es la banda, y abrir un segundo relleno encima del que ya está serían
dos rectángulos opacos sobre la reunión.

**Lo que NO se repone, y es una decisión: el acople.** El corte lo suelta a propósito. Volver a
encoger la ventana de la reunión sin que nadie lo pida sería deshacer una pieza del kill-switch por
la puerta de atrás. La banda vuelve **flotando y diciéndolo** —«sin acople» sale de la huella, no de
una variable nuestra— y el asa la vuelve a acoplar cuando el usuario quiera.

**Y el comando huérfano se retira.** `#[tauri::command] abrir_banda` sale del `invoke_handler`: su
doc decía que «la fase 2 la llamará al detectar una reunión», y esa detección vive en Rust, así que
no necesita pasar por el puente. Es el mismo movimiento que con `documentos_del_corpus`: cuando haga
falta, vuelve **con su llamador**.

**Verificación:** esto no tiene test. `abrir_banda` necesita un `AppHandle` y un monitor de verdad
—`geometria()` pregunta por el monitor principal—, y `tauri::test::mock_app` no trae ninguno de los
dos; montarlo habría sido una maqueta del sistema de ventanas probándose a sí misma. Va por el
**tercer filo de la regla 15**: se ve correr en el modo, con `pnpm tauri dev`, junto con la CSP.
Queda anotado abajo, con lo que se vio.

---

## Fase 0 · M9 — el inventario del efímero veía aparecer, no crecer

Dos formas de dejar rastro en un disco: **crear** un archivo y **escribir en uno que ya estaba**. El
gate del sprint 001 inventariaba un **conjunto de rutas** (`contra-el-mac-de-verdad.rs:482`), así que
solo veía la primera: una fuga que le añade una línea a un archivo existente no le cambia la ruta, el
inventario de antes y el de después salían idénticos, y el gate daba verde.

**Ahora cada archivo lleva su huella: tamaño y fecha de escritura.** Un archivo que engorda cambia de
tamaño; uno reescrito del mismo largo cambia de fecha. Las dos escrituras se ven.

### Una desviación del plan, y es la parte que importa

El plan pedía **ruta → (tamaño, hash)**. El hash no se puso, y no por ahorrar: hashear lo que hay en
`~/Documents`, `~/Desktop` y `~/Downloads` significa **leer los documentos del usuario en cada corrida
del gate**, y un gate que abre los archivos privados para demostrar que la app no los toca es un trato
que esta casa no hace. Tamaño y fecha salen de la **misma llamada a `metadata()`** que el inventario
ya necesitaba para saber si algo es un archivo —cero lecturas de más— y cazan exactamente las dos
escrituras que un hash cazaría.

### Y las carpetas de la app, que no se miraban

`donde_se_mira` gana `~/Library/{Application Support, Caches, Logs}/com.aiapps.copiloto-consultor`.
El test le pasa a la sesión una `casa` en el temporal, así que nada de lo que corre aquí escribe en
esas tres — **y justo por eso hacían falta**: si un día la app escribe con su ruta de producción en
vez de con la que se le pasa, el rastro cae ahí y en ningún otro sitio del inventario.

**El nombre de la carpeta lo trae el plan mal.** El plan dice `~/Library/Application Support/Angel
Ghost`, que es el nombre del producto; macOS usa el **identificador**,
`com.aiapps.copiloto-consultor`, que es la carpeta que existe de verdad (y ya nace en 700). Mirar
donde no hay nada es la forma más fácil de que un gate dé verde para siempre.

### Demo en rojo

Una fuga que **añade** a un archivo que ya existía, plantada en `Corpus::buscar` — a propósito en un
módulo **no protegido**, porque el barrido estático prohíbe el disco en `capture/`, `stt/`, `voz/` y
`escucha/` y habría cazado la fuga antes que este gate. Es la tercera pregunta de la regla 15 puesta
en práctica: ¿puede este gate fallar siquiera, o hay una regla anterior que lo hace inalcanzable? Sí
puede, y este es el hueco que le toca cubrir — lo que escriben los módulos que sí pueden escribir.

```
[efímero] 11 archivos tocados (creados o escritos)
la sesión dejó rastro en 1 archivo(s) fuera del índice del corpus:
  …/docs/kit-de-prueba/audio/pregunta-es.wav — ya existía y la sesión escribió encima:
  192306 → 192312 bytes
```

Seis bytes. Con el inventario del sprint 001, verde. Revertido: verde con 10 archivos tocados, que
son los del índice, y el WAV del kit intacto (`git diff` en cero).
