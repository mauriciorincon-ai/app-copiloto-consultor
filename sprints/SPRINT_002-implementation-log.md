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

---

## Fase 0 · M1 — la CSP, que estaba en `null`

`tauri.conf.json:57` traía `"csp": null` desde el estampado. Sin política, el webview puede pedir
cualquier cosa a cualquier sitio.

**Por qué importa aquí más que en una app cualquiera.** La promesa «nada crudo sale del equipo» tenía
dos capas y las dos viven en Rust: el barrido que prohíbe API de red en los módulos protegidos y el
contador de puertas declaradas. **Ninguna de las dos mira lo que el webview pueda hacer por su
cuenta** — un `fetch` en un `.tsx`, un `<img>` a un dominio de fuera, una fuente de Google en un CSS.
La CSP es la capa de ese lado, y la única que el navegador aplica sin depender de que alguien se
acuerde.

### La política

`connect-src 'self' ipc: http://ipc.localhost` y nada más. Lo demás: `default-src 'self'`,
`script-src 'self'`, `img-src 'self' data:` (el fondo del relleno llega como data URL),
`media-src`/`object-src`/`frame-src`/`worker-src`/`child-src` en `'none'`, `base-uri 'self'`,
`form-action 'none'`, `frame-ancestors 'none'`.

Dos decisiones que conviene dejar escritas:

- **`devCsp` aparte.** En desarrollo el webview carga de `http://localhost:1420` y el recargado en
  caliente habla por `ws://`, y Vite inyecta scripts inline. Meter eso en la política de producción
  habría dejado `localhost` y `'unsafe-inline'` en el binario firmado; `devCsp` es un campo del propio
  Tauri (`tauri-utils` 2.9.3, `dev_csp`) y existe exactamente para esto.
- **`'unsafe-inline'` en `style-src`, sí; en `script-src`, no.** La interfaz usa atributos `style` de
  React por todas partes —es como se traslada la maqueta— y un atributo `style` inline no pasa por
  nonce ni por hash. Es riesgo de estilo, no de ejecución. Queda escrito en el gate para que nadie
  tenga que adivinar si fue decisión o prisa.

### Con la app corriendo, que es la única forma de hacer esto

Un error de CSP es **una pantalla en blanco callada**. Así que se hizo con `pnpm tauri dev` delante, y
lo que se vio:

```
[ventanas] «principal»: 960x641 · «banda»: 1470x88 · «relleno»: 1470x88
[permisos] micrófono=Concedido pantalla=Concedido accesibilidad=Concedido · cara=Concedido
[stt] motor «apple-speechanalyzer» · 30 idiomas soportados
```

Las tres ventanas abiertas, el motor cargado, y —lo que prueba que el puente sobrevive a la política—
**dos comandos ida y vuelta por el IPC de verdad** (`cortar_todo` y `empezar_a_escuchar`), con las dos
pistas abriéndose después.

### Demo en rojo, con su control

Un `fetch("https://example.com/…")` plantado en `main.tsx`, sin `catch`, porque el cliente de Vite
reenvía las promesas rechazadas al terminal:

```
[vite] (client) [Unhandled rejection] TypeError: Load failed
```

**Y «Load failed» también sería un fallo de red, así que la demo no vale sin control.** Dos lados más:

```
$ curl -o /dev/null -w "HTTP %{http_code} en %{time_total}s" https://example.com/
HTTP 200 en 0.070958s
```

```
[vite] (client) [Unhandled rejection] Error: CONTROL OK — el origen permitido pasa y el de fuera NO:
TypeError: Load failed
```

El dominio se alcanza desde esta máquina en 70 ms; desde el webview no. Y un `fetch` al origen que la
`devCsp` **sí** permite pasa sin problema. Lo que bloquea es la política, no la red.

### Y un gate que la mantenga cerrada

`tests/unit/csp-que-no-deja-salir.test.ts` (7 aserciones). No vigila «hay una CSP» —eso es un campo
con texto— sino que **siga siendo cerrada**: todo origen que aparezca en cualquier directiva tiene que
estar en una lista con su razón escrita. El día que el API opt-in de la fase 5 necesite un dominio,
este test falla y **obliga a nombrarlo**: un proveedor concreto, con su línea en el summary, en vez de
un `https:` suelto que abre la puerta a todos.

Sus dos rojos:

```
· con "csp": null, como estaba el S1 → 5 de 7 en rojo
· con https://api.anthropic.com en connect-src → 2 en rojo:
  «la app promete que nada sale del equipo y la CSP dejaría salir por:
   connect-src: https://api.anthropic.com»
```

### De paso, en vivo

- **M4 verificado**, que es lo que esa fase debía: `[corte] ⌥⎋: 6 de 7 piezas cortadas` →
  `[ventanas] la banda estaba cortada: vuelve` → las dos pistas abiertas. La línea del log se queda:
  es la única traza de que M4 está cableado, y sin ella «vuelve» sería una afirmación sin testigo.
- **El disparo por silencio no se dispara solo.** La sesión corrió sin que nadie hablara y no salió ni
  una línea `[ficha]`: sin turnos del cliente, `ultimo_de(Sistema)` no devuelve nada y el latido se
  calla. Es una observación débil —no prueba que dispare cuando debe, eso lo prueban sus tests— pero
  es la que descarta el fallo más caro: una banda estrenando fichas en una reunión en silencio.
- **Y un ruido de desarrollo que NO es de este sprint, anotado para la auditoría:**
  `[Unhandled rejection] TypeError: undefined is not an object (evaluating 'listeners[eventId].handlerId')`
  en `src/puente.ts:45`. Es la carrera de `StrictMode`, que monta los efectos dos veces: el
  `unlisten` llega antes de que el `listen` acabe de registrarse. Solo en desarrollo, y anterior a
  este sprint.

---

## Fase 0 · El `cargo test --release`, que es el cuarto filo de la regla 15

La regla del kit v1.28.0 dice que un test que solo corre en `debug` no prueba `release`, y su origen es
de esta casa: el `catch_unwind` que protege el parseo de PDF pasaba todos sus tests en debug y era
letra muerta en release, donde `panic = "abort"` lo anula.

El sprint 001 dejó un gate **estático** (`el_perfil_de_release_desenreda`, `corpus/leer.rs:300`) que
lee el manifiesto y falla si `panic = "abort"` vuelve, con esta razón escrita: correr la suite en
release en cada PR costaría otra compilación con LTO para vigilar una línea de configuración. Eso es
correcto **y no sustituye a la corrida**, porque la regla pide *al menos una vez* con el perfil con que
la app se distribuye. Esta es esa vez:

```
$ cargo test --release --lib corpus
test corpus::leer::pruebas::el_perfil_de_release_desenreda ... ok
test corpus::leer::pruebas::un_archivo_sin_texto_se_declara_ilegible_en_vez_de_indexarse_vacio ... ok
test corpus::pruebas::indexa_una_carpeta_entera_y_un_documento_roto_no_detiene_a_los_demas ... ok
test corpus::pruebas::el_documento_ilegible_trae_su_motivo_en_espanol_llano ... ok
test result: ok. 50 passed; 0 failed
```

Las cincuenta del corpus en el perfil de distribución, con el camino del PDF dentro. El summary lo
declara.

---

## Fase 0 · Los campos del contrato sin lector — y el gate que los cuenta

La casilla 5 de `/audita-sprint` contó **diecisiete** campos que cruzan la costura y nadie lee. El
plan decía: «cada uno gana lector o sale del contrato».

### Primero, contarlos bien — y aquí me equivoqué dos veces

Escribí un script para contarlos y **dio 13**. La segunda versión dio 17, que coincidía con la
auditoría… por casualidad: eran 17 nombres distintos por un camino y 17 pares (tipo, campo) por otro.
Los dos scripts tenían fallos de verdad:

1. **Extraer los tipos con una expresión regular se comía bloques enteros.** Un `export type X = A | B;`
   sin cuerpo hacía que la búsqueda no greedy se saltara el tipo siguiente. `EstadoDelCorpus`,
   `Documento` y `QueSabeTranscribir` no se miraron nunca. Se arregló con un **parser que cuenta
   llaves**, como el del generador de design-sync.
2. **Y lo peor: daba por «leído» un campo que en realidad se ESCRIBE.** `EstadoDelCorpus.carpeta`
   aparecía consumido por `preguntar("indexar_corpus", { carpeta })` — que es la carpeta viajando
   **hacia** Rust, no la respuesta leyéndose. Un gate que confunde escribir con leer declara pagada la
   deuda que existe.
3. Y las **uniones** solo se miraban en su primera variante, así que los cuatro «motivo» de los
   estados que no se pudieron determinar estaban invisibles.

**La cuenta buena, con el parser y la regla de lectura arreglada: 20 pares (tipo, campo) sin un solo
lector**, de 58 declarados. Tres más de los que la auditoría vio, no menos.

### Dos que se pagaron borrándolos

`EstadoDePista.legible` y `EstadoDeEscucha.ramLegible` mandaban los bytes **ya escritos** («1,8 MB»),
con la razón de no tener dos formateadores. La fase 5 del sprint 001 descubrió que hacían falta dos:
el separador decimal es interfaz, y este venía siempre con coma —«1,8 MB» dentro de «What lives in
memory now»—. Desde entonces la pantalla los formatea con el idioma puesto y **estos dos cruzaban la
costura sin que nadie los leyera, ni en TypeScript ni en Rust**. Fuera del contrato: quedan 18.

### Y dieciocho que no se pueden pagar en la fase 0, dicho con nombre y sitio

Se miraron uno a uno, y **ninguno se puede cablear sin escribir copy que la maqueta no tiene**:

- los cinco de la pista y la ventana (`motivo`, `hablando`, `segundos`, `muestrasRecibidas`,
  `turnosEnMemoria`) son el estado **«pista caída»** de Sesión, que es el M2 y espera la **mirada 17**;
- los cuatro del motor (`EstadoDeEscucha.motor`, `QueSabeTranscribir.motor`/`techo`/`motivo`): la
  maqueta de Idioma habla del motor **en prosa** («el motor de voz de macOS») y no tiene sitio para el
  dato. Se verificó abriendo `idioma.html`;
- los cuatro «por qué» (`Reunion.motivo`, `Salida.motivo`, `Salida.nombre`, `Disponibilidad.motivo`):
  cada pantalla pinta el **estado** y se calla el motivo que lo acompaña. Idioma enseña
  `t.sinMotorDeVoz`, una cadena fija del diccionario, no el motivo que Rust manda;
- los tres de la ficha (`Aparicion.motivo`, `Aparicion.ms`, `Fuente.conjeturada`): la banda **mide** la
  latencia y la registra en el log con su presupuesto de 4 s, y sabe por qué disparó —
  `Motivo::etiqueta()` existe desde el S1— y no pinta ninguno de los dos. No hay hueco en `banda.html`;
- `Turno.hastaMs`, `EstadoDelCorpus.carpeta` y `EstadoDelCorpus.secciones`, por lo mismo.

**Y uno que no es deuda:** `InformeDelCorte.bytesEnRed` **lo lee Rust** —`ejecutar_el_corte` lo escribe
en el log del corte— y cruza porque la forma tiene que cuadrar en las dos orillas. El webview ya tiene
su contador por otro camino. Se declara, no se paga.

### Desviación del plan, declarada

**El plan ponía los 17 en la fase 0 y no cabían ahí.** Dos se pagaron; los dieciocho restantes **no se
pueden cablear sin una mirada del usuario**, y en esta casa el copy nuevo no existe antes de su mirada
(regla 10). Pagarlos en la fase 0 habría significado inventar interfaz a espaldas del gate de mirada —
exactamente lo que el método prohíbe. Se pagan en la **fase 3**, con la mirada 17, que ya estaba
planeada para el estado «pista caída» (M2) y es donde casi todos caen.

**Tres de ellos piden algo que la mirada 17 no tenía en su lista** —la latencia y el motivo de la
ficha, y la marca de sección conjeturada, que son de la banda y no de Sesión—, así que el plan de
miradas necesita ese añadido. **Eso se propone, no se decide sobre la marcha** (kit v1.21.0): va en el
resumen de esta fase.

### Y lo que impide que se pierdan: `tests/unit/contrato-con-lectores.test.ts`

La casilla 5 es una comprobación a mano, una vez por sprint, al final. Los diecisiete del S1 salieron a
la luz **con el gate de la forma en verde y 153 tests pasando**. Así que la comprobación pasa a ser un
gate, con la lista de deuda dentro, **cada campo con su `archivo:línea` y su fase de pago** — que es la
regla 20 aplicada a esta clase de hallazgo.

**Falla en los dos sentidos, y es a propósito.** Si aparece un huérfano nuevo, hay que cablearlo,
sacarlo del contrato o declararlo. Y **si uno se paga y la línea sobrevive, también falla**: una lista
de deuda que sobrevive a su deuda miente sobre lo que queda por hacer, que es la otra mitad de lo que
pasó en el S1.

**Demo en rojo, por los dos lados:**

```
· borrada la línea de «Aparicion.ms» de DEUDA:
  campos que cruzan la costura y nadie lee, sin una línea en DEUDA:
    Aparicion.ms  (src/ficha.ts:66)

· cableado `escucha.turnosEnMemoria` sin borrar su línea:
  DEUDA declara campos que YA tienen lector:
    EstadoDeEscucha.turnosEnMemoria — ya tiene lector: src/pantallas/Honestidad.tsx:37
```

Y una cuarta aserción que exige que **cada línea de la deuda diga su fase**: nada por conteo.

### De propina, un descuido propio

Al revertir la segunda demo con `git checkout` me llevé por delante un arreglo sin comitear: el
comentario de `Honestidad.tsx` que nombraba `legible` y `ramLegible` como campos vivos — una frase
caducada por mi propio cambio de hace veinte minutos, que es la casilla 4 de la auditoría aplicada a
uno mismo. Se rehízo y se comprobó. **`git checkout` sobre un archivo con trabajo sin comitear no
distingue lo que plantaste de lo que arreglaste**, y en una fase que planta y revierte demos en rojo
todo el rato, eso no es mala suerte: es el guion. Lo que toca es comitear antes de plantar.

---

## Fase 0 · Criterio de fase, verificado

| Gate | Resultado |
|---|---|
| `pnpm typecheck` | ✓ |
| `pnpm lint` | ✓ |
| `pnpm test` | ✓ 24 archivos · **164** tests (eran 153 al abrir la fase) |
| `pnpm verify:ephemeral` (estático) | ✓ cero API de disco o red en los 8 módulos protegidos · 17 archivos |
| `pnpm verify:ephemeral:runtime` | ✓ 10 archivos tocados, todos del índice del corpus |
| `cargo clippy --locked --all-targets -- -D warnings` | ✓ |
| `cargo test --locked --lib` | ✓ **217** (eran 210) |
| `cargo test --release --lib corpus` | ✓ 50 — el cuarto filo de la regla 15 |
| CERO ENLACES, después del último `git add` | ✓ limpio |
| **La CSP viva con su rojo** | ✓ con control de tres lados |
| **M9 con su rojo** | ✓ 192306 → 192312 bytes |
| **CI con conclusión propia por check** | ✓ `quality` success · `e2e` success · `build-escritorio` success — leídas de la API, no de la columna |

PR #6, en borrador: el sprint tiene siete fases y va por la 0. Se abre ahora porque la CI de este repo
solo corre en `pull_request`, y un check sin conclusión propia no es verde.

### Lo que esta fase deja pedido al usuario

**El plan de miradas necesita un añadido, y se propone — no se decide sobre la marcha** (kit v1.21.0).
La mirada 17 estaba planeada para el radar, «vigilancia local», «pista caída» (M2) y el consentimiento
de pantalla. Tres de los campos huérfanos son de la **banda** y no de Sesión, así que no caben en esa
lista tal como está:

- **el motivo del disparo** en la ficha — `Motivo::etiqueta()` existe desde el S1 y la banda no lo
  pinta; ahora hay cinco motivos, porque el silencio ya dispara;
- **la latencia** de fin de turno a ficha, que se mide y se registra con su presupuesto de 4 s;
- **la marca de sección conjeturada** en la fuente de la ficha (el lector de PDF la adivinó).

Los tres son de `banda.html`. Caben en la mirada 17 sin partir la sesión en dos, o pueden ir a la 18
con la sugerencia, que también es de la banda. Es decisión del usuario.

---
---

# FASE 1 · El diccionario técnico y el kit que mide (B3)

## El diseño se aparta del plan, y a mejor

El plan decía: «`diccionario/` … exige entrar en el `Permitido` del gate del efímero» y, en la tabla del
ADR 002, que el módulo **puede tocar disco**. Al escribirlo se ve el problema: ese módulo **recibe cada
turno del cliente** y devuelve el turno corregido, así que tiene el transcript en las manos. Por la
frontera de este repo eso lo pone del lado de los protegidos; por su archivo, del otro.

**Se resuelve partiendo la responsabilidad, no relajando la regla:**

| Quién | Qué hace | Disco |
|---|---|---|
| `diccionario/` | corrige el turno y **serializa a un `String`** | **No** — está en `PROTEGIDOS` |
| `lib.rs` | lee y escribe ese `String`, y cierra sus permisos | Sí — no ve un solo turno |

Cuesta lo mismo y es más fuerte: el módulo que toca la voz del cliente **no tiene manera** de
escribirla, y no hace falta confiar en que nadie se equivoque al añadir la función siguiente. Enmienda 1
del ADR 002.

### Y el gate del sprint 001 cazó mi cambio él solo

Al escribir la cabecera «MÓDULO PROTEGIDO» y aún no estar en la lista, `pnpm verify:ephemeral` salió en
rojo sin que yo provocara nada:

```
✕ src-tauri/src/diccionario/mod.rs se declara «MÓDULO PROTEGIDO» y NO está en la lista de este
  script: o entra en PROTEGIDOS, o su cabecera deja de afirmarlo.
```

Ese gate sobre el gate nació del hallazgo A4 del S1 —`escucha` llevaba dos fases afirmando que el script
lo comprobaba sin estar en la lista— y **este es el primer cambio ajeno que examina**. Funcionó.

## La decisión que evita que el diccionario haga más daño que bien

Con **cuatro letras o menos, la tolerancia es cero.** «DAX» está a una edición de «das», «dos», «día»,
«tax» y «max»: palabras que la gente dice de verdad. Para los términos cortos la única corrección que se
acepta es una **variante escrita a mano** — «the ax» → «DAX» porque alguien lo puso ahí, no porque se
parezca. Su test mide cinco frases normales, en los dos idiomas, y exige que salgan **sin una letra
cambiada**.

La otra regla, la de privacidad: **el diccionario no aprende de la reunión.** Las entradas salen del
archivo del usuario y de los **nombres propios de su corpus**, que se recalculan en cada arranque y **no
se guardan en el archivo**. Un diccionario que se corrigiera con lo que oye sería un transcript
persistido con otro nombre. `corregir` toma `&self`, y ese `&` es la regla escrita en el tipo; el test
`lo_que_se_guarda_no_lleva_el_nombre_de_ningun_cliente` comprueba la otra mitad.

## El kit: las dos deudas del sprint 001, pagadas

`mezcla-es.wav` y `mezcla-en.wav`, generados con `say` + `afconvert` (16 kHz mono, como los del S1), con
jerga técnica **y** una frase entera en el otro idioma — que es como habla de verdad un consultor de
datos bilingüe. `transcripciones.json` guarda lo que cada audio dice palabra por palabra, con la jerga
escrita **como el consultor quiere verla**: medir contra lo que el motor oye sería medirlo contra sí
mismo.

| Audio | WER sin diccionario | WER con diccionario | |
|---|---|---|---|
| `pregunta-es.wav` | 0,133 | 0,133 | control · sin jerga, **no se mueve** |
| `pregunta-en.wav` | 0,000 | 0,000 | control · sin jerga, **no se mueve** |
| `mezcla-es.wav` | 0,458 | **0,417** | mejora |
| `mezcla-en.wav` | 0,348 | **0,261** | mejora |

**El umbral es doble, y esa es la parte pensada.** El plan pedía «no empeora»; solo con eso, un
diccionario que no corrigiera nada pasaría el gate. Así que también se exige que **baje en al menos un
audio con jerga**. Las dos aserciones juntas son la única forma de que el número signifique algo.

### Y de paso, la respuesta a la pregunta que el plan dejó abierta

El plan decía: «si SpeechAnalyzer no sostiene la mezcla, la pantalla lo declara». **No la sostiene**, y
no es un WER alto: es texto que no significa nada.

```
dicho:  «… Y el DAX lo escribió otro proveedor.»          (dentro de un audio en-US)
oído:   «… YL Daxlo is Gribbio Otro Provider.»
```

De ahí sale el **ADR 009**, y con una distinción que importa: **lo que se midió es un idioma por pista**,
que es lo único que la app puede configurar —`stt::Motor::transcribir` recibe **un** `idioma: &str` y los
dos de la app son dos constantes—. Lo que **no** se midió es qué hace el motor con varios idiomas
configurados a la vez, porque no hay manera de pedírselo. La frase de la maqueta sobre eso queda
**marcada como no verificada**, no desmentida, y no se reescribe: es copy y es decisión del usuario.

## Tres frases que esta fase volvió falsas, y su arreglo

Es la casilla 4 de la auditoría aplicada a uno mismo, y salieron las tres del mismo sitio:

1. **La pantalla de Idioma listaba «Diccionario técnico» dentro de «Lo que todavía no existe».** Una
   pantalla que dice «todavía no» de algo que existe miente igual que una que promete lo que falta. La
   fila **sale de la lista** (maqueta, diccionario i18n y `Idioma.tsx`), y su test pasa de contar tres
   pendientes a comprobar **los dos que quedan por su nombre** — un conteo que cambia en silencio no
   dice cuál se fue.
2. **«los nombres propios se transcriben como suenen»**, en el detalle de esa misma tarjeta: falso desde
   hoy. Se quita la cláusula y se deja la que sigue siendo verdad («hoy cada pista escucha un idioma»),
   que además es justo lo que el ADR 009 manda declarar.
3. **El manual decía que el diccionario técnico «llega más adelante».** Ahora tiene su sección, con la
   ruta del archivo, cómo editarlo, los números medidos y sus cuatro limitaciones.

**Y lo que NO se hizo, a propósito:** quitar una afirmación falsa es una corrección; **inventar cómo la
pantalla enseña el diccionario es diseño**, y el diseño pasa por mirada. Así que la pantalla de Idioma
deja de negarlo y no lo presume todavía. La propuesta va al gate de esta fase.

## De propina, dos hallazgos del propio trabajo

- **La voz «Mónica» ya no está instalada en este Mac.** El LEEME del kit documentaba los dos audios del
  S1 como hechos con ella; los nuevos van con **Paulina**. Se corrige la tabla y se dice por qué los
  audios se **versionan** en vez de generarse en cada corrida: un kit que se regenerara solo mediría una
  voz distinta cada vez que Apple cambie de catálogo.
- **El archivo del diccionario nacía en 644 y se apretaba a 600 después.** Lo delató la traza del propio
  gate del efímero. Funcionaba y estaba mal: entre el `write` y el `set_permissions` hay una ventana en
  la que la jerga del consultor es legible por cualquier cuenta del Mac, y la regla 17-bis dice **nace**.
  Ahora se crea con `create_new` y su modo, y el test comprueba **que no hubo reparación** — no solo que
  el modo final sea el bueno.

## Criterio de fase, verificado

| Gate | Resultado |
|---|---|
| WER medido y publicado en el kit | ✓ cuatro audios · tabla en `audio/LEEME.md` y en el ADR 009 |
| El diccionario persiste sin romper el efímero | ✓ `Permitido` gana su segunda entrada, con su rojo |
| El gate del diccionario (i18n fiel a la maqueta) | ✓ verde, con las tres frases corregidas en los dos lados |
| `pnpm typecheck` · `lint` · `test` | ✓ 24 archivos · 164 tests |
| `pnpm verify:ephemeral` estático y runtime | ✓ 18 archivos inspeccionados (era 17) |
| `cargo clippy --locked --all-targets -- -D warnings` | ✓ **tras tres hallazgos suyos en mi código** |
| `cargo test --locked` | ✓ 235 + 15 |

---

## Fase 1 · El paso de evidencia encontró que el WER nunca ha medido en la CI

El WER salía `ok` en la CI y eso no significaba nada. Con el paso de evidencia puesto, el log dice lo
que estaba pasando:

```
┌─ WER del kit · con y sin diccionario ────────────────────────────────
│ pregunta-es.wav  sin modelo de es-ES en esta máquina: no se mide
│ pregunta-en.wav  sin modelo de en-US en esta máquina: no se mide
│ mezcla-es.wav    sin modelo de es-ES en esta máquina: no se mide
│ mezcla-en.wav    sin modelo de en-US en esta máquina: no se mide
sin modelos de voz en esta máquina: el WER no se pudo medir en ninguna pista
```

**El runner de `macos-latest` no trae ni un modelo de voz.** El test hacía lo correcto —salir temprano
diciendo por qué— y aun así el resultado era un `ok` verde que se leía como «medido».

### Y no es solo el mío: es un gate del sprint 001 que nunca ha corrido en la CI

`transcribe_la_pregunta_en_espanol` y `transcribe_la_pregunta_en_ingles` existen desde el S1 y llevan
todo ese tiempo saliendo `ok` **sin transcribir nada** en la integración continua, por la misma razón.
Su contrato tiene dos mitades y en la CI solo se ejerce la segunda —«sin motor, un motivo que se puede
enseñar»—, que es la que no necesita modelo. Nadie lo había visto porque `ok` no distingue.

`nDCG@5 0,823`, en cambio, **sí mide en la CI**: es texto contra texto y no necesita audio.

### Qué se hace con esto, y qué NO

**No se hace fallar la CI por esto.** Sería rojo en cada PR por un motivo del entorno, y lo que pasaría
después es que alguien lo desactivaría — que es la muerte de todos los gates de esta familia. Tampoco
se instalan modelos de voz en el runner: sería una descarga de red en la CI de una app cuya promesa es
que no toca la red.

**Lo que se hace es dejar de afirmar lo que no se sabe.** Tres cosas:

1. El paso de evidencia se queda: sin él, esto era invisible.
2. **El WER es una medida del Mac, no de la CI.** Su sitio es `/release-check` y el gate ⭐ del
   usuario, y el summary del sprint tiene que decirlo con estas palabras: *sin histórico en la CI no
   puede afirmarse ni regresión ni no-regresión* (regla 15, segunda pregunta). Los números de la tabla
   se midieron en este Mac, con su fecha.
3. Va a la auditoría del sprint como hallazgo, con su sitio: los dos tests de transcripción del S1 y el
   del WER **no son gates de la CI**; son gates de la máquina del desarrollador con su salida escrita.

**La lección, que es la útil:** el tercer filo de la regla 15 pregunta si lo viste correr *en el modo en
que el usuario lo va a usar*. Aquí hacía falta la pregunta de al lado — **¿lo viste correr en el sitio
donde crees que corre?** Un test que sale temprano diciendo por qué está bien escrito; lo que faltaba
era que alguien leyera lo que decía.

---
---

# FASE 2 · El modo solo audio (C15) — **mirada 16 primero**

## Lo que se maquetó, y lo que NO

El plan aprobó la mirada 16 como **«banda de 44 px "voz" + "voz sin auriculares"»**, y eso es lo que
hay: ni más ni menos. En el gate de la fase 1 quedaron **dos preguntas de diseño sin contestar** —a
dónde van los tres estados de la banda que no caben en la mirada 17, y cómo enseña la pantalla de
Idioma el diccionario— y el usuario respondió «continúa» sin tocarlas. **«Continúa» no aprueba
diseño y tampoco reorganiza el plan de miradas** (kit v1.21.0), así que no se metió nada de eso por
la puerta de atrás: la mirada 16 es exactamente la que él aprobó en el plan.

El alto y el CSS estaban **reservados desde el sprint 001** (`--banda-h-voz`, `.banda.voz`, y la nota
de medida del propio `banda.html` decía «44 px reservado para el modo solo audio»). Lo que nace aquí
es el **contenido**, trasladado del panel —donde el estado ya estaba aprobado como píldora de
260 × 56— a la anatomía de la banda.

### Tres decisiones que van a la nota para que el usuario las juzgue

1. **El contador de red se queda.** A 44 px la cabecera se oculta por diseño (`.banda.voz .cab-b {
   display: none }`), y con ella se iría el «0 B» — que es una **promesa dura** de esta app (regla 2:
   contador de salida a red visible por reunión), no un adorno de la cabecera. Baja a la línea.
2. **La tecla es `⌘⇧V`, no `⌘⇧A`.** La orden del sprint pedía `⌘⇧A` para leer la ficha en voz alta, y
   **`⌘⇧A` ya es «ayúdame con esto» desde el sprint 001**: está registrada en `lib.rs`, está en el
   manual y está dibujada en la banda. El panel aprobado ya usaba `⌘⇧V` para la píldora de voz. Se
   respeta el artefacto aprobado y se declara la desviación de la orden.
3. **El glifo `⎋`** se lee como un borrón a 13 px — **igual que el `⌥⎋ corta` de la banda de 88 px**,
   que ya está aprobado. Se deja por coherencia y se señala: si molesta aquí, molesta en los dos
   sitios y es un cambio del sistema, no de esta pantalla.

## La pasada de capturas encontró dos defectos que ningún número reportaba

El arnés dijo **cero desbordes, cero errores de página, 48 capturas**. Y al leer las imágenes:

1. **El glifo del altavoz no se dibujaba.** `i-voz` son cinco líneas verticales
   (`<path d="M2 7v2M5 4.5v7…"/>`) sin área, y yo lo pedí con `relleno`, que es
   `fill: currentColor; stroke: none`. Resultado: **nada**. El estado se quedaba en texto y color,
   que es justo lo que la regla del daltonismo leve prohíbe.
2. **Y el segundo es peor:** `i-auriculares-off` con `relleno` pierde el arco y la barra tachada y
   deja dos rectángulos rellenos — unos auriculares **conectados**. El icono decía **lo contrario**
   del estado que acompañaba.

Ninguno de los dos da error en ninguna consola. Los encontró mirar la imagen.

### Y el gate que salió de ahí encontró seis más, de sprints anteriores

`tests/unit/iconos.test.ts` gana su hermano: **`relleno` solo vale sobre un símbolo que decide su
propio `fill`.** Los símbolos del sprite son de dos familias y la diferencia no está en el nombre —
`i-check-circle` o `i-alert` llevan sus propios `fill="currentColor"` y `stroke="var(--bg)"` en cada
hijo, así que `relleno` no les hace nada; `i-voz`, `i-doc`, `i-candado`, `i-reloj` y
`i-auriculares-off` no llevan atributos y heredan del CSS.

Su rojo, con los ocho usos que había:

```
docs/diseno/banda.html:196       i-voz              ← mío
docs/diseno/banda.html:213       i-auriculares-off  ← mío
docs/diseno/honestidad.html:173  i-reloj
docs/diseno/ia.html:82           i-doc
docs/diseno/notas.html:259       i-candado
docs/diseno/notas.html:265       i-reloj
docs/diseno/sesion.html:240      i-voz
src/pantallas/Sesion.tsx:158     i-voz              ← EN EL PRODUCTO
```

**El último es el que importa.** El botón «Iniciar sesión» de la pantalla de Sesión lleva desde el
sprint 001 con un icono que no dibuja nada — y la maqueta **nunca lo pidió relleno** (`sesion.html:240`
escribe `class="ic s"` a secas). El producto se desvió de la maqueta por su cuenta y el gate de
fidelidad no podía verlo: compara píxeles contra una maqueta que en ese punto tenía el **mismo**
defecto, porque su otro uso de `i-voz` también estaba relleno. Dos errores que se tapaban el uno al
otro.

Los ocho arreglados; el bundle de `design-sync/` regenerado (14 archivos).

## Un defecto de copy que también salió de mirar

En el estado «hablando», la tecla decía `⌘⇧V solo audio` — **estando ya dentro del modo**. Lo que esa
tecla hace ahí es **salir**. Dice `volver` / `back`.

## La mirada 16 — APROBADA (2026-09-26)

> «Apruebo el diseño muy limpio icono azul a la izquierda indicando que se habla muy intuitivo y
> amplio margen para la pantalla de reunión»

Llegó con el artefacto abierto y se registra por eso: nombra **el glifo azul a la izquierda** —`i-voz`
en `--halo`, que es el símbolo que la regla 8 exige junto al texto— y **el margen para la pantalla de
la reunión**, que es exactamente lo que compra bajar la banda de 88 px a 44 px. Ninguna de las dos se
puede describir sin haberlo visto, así que no hubo que repreguntar.

Con ella quedan aprobados los **tres criterios declarados** en las notas del artefacto: el contador de
red se queda en la línea, la tecla es `⌘⇧V` (desviación de la orden, declarada arriba) y el glifo `⎋`
se deja como el `⌥⎋` ya aprobado. Registrada en `docs/diseno/README.md` con su fila y su sección.

**Lo que desbloquea:** la construcción de la fase 2 — `nativo/Habla.swift`, `src/habla/`, `⌘⇧V`, el
candado de los auriculares y la banda cableada a 44 px en `Banda.tsx` y `src/asa.ts`.

## Las dos preguntas de diseño que siguen abiertas

No las contesta ningún «continúa» y no bloquean la fase 2, pero **sí** bloquean la mirada 17 (fase 3),
que es donde se pagan diecinueve campos del contrato sin lector. Se especifican aquí para que la
respuesta quede en el repo y no solo en el chat.

### Pregunta A — ¿dónde se miran los tres estados de la ficha que la mirada 17 no cubre?

La mirada 17, tal como el plan la aprobó, cubre: **radar ámbar** · **radar coral** · «vigilancia
local» en Sesión · **«pista caída»** (M2) · el consentimiento de pantalla en Permisos. Son la familia
de *«algo va mal o algo vigila»*.

Los tres de abajo son otra familia —*«la ficha se explica a sí misma»*— y viven todos en la banda, con
la ficha dentro. Tienen su dato midiéndose ya en Rust y cruzando la costura; lo único que falta es
**copy que la maqueta no tiene**, y por la regla 10 ese copy no puede nacer sin una mirada:

| Campo | Archivo:línea de la declaración | Qué enseñaría | Qué existe ya |
|---|---|---|---|
| `Aparicion.motivo` | `src/ficha.ts:64` | **por qué** disparó: pregunta · cifra · término tuyo · silencio · lo pediste | `Motivo::etiqueta()` (`disparo/mod.rs:59`) ya da las cinco etiquetas — **solo en español**: el inglés también es copy nuevo |
| `Aparicion.ms` | `src/ficha.ts:66` | la **latencia** de fin de turno a ficha, contra su presupuesto de 4 s | se mide y se escribe en el log en cada aparición |
| `Fuente.conjeturada` | `src/ficha.ts:27` | que la **sección** la conjeturó el lector de PDF, nadie la escribió | `corpus/` ya lo marca documento a documento y Corpus lo cuenta en bloque |

Las tres opciones, con lo que cuesta cada una:

- **(a) entran en la mirada 17** — una sesión de mirada en vez de dos, y los tres campos se pagan en
  la fase 3 como el resto. Coste: la mirada 17 pasa de 5 estados a 8 y mezcla dos familias visuales,
  que es exactamente lo que el plan de miradas separó a propósito.
- **(b) esperan a la mirada 18** (fase 5, la sugerencia y la pantalla IA) — donde la banda vuelve a
  abrirse para dibujar «sugerencia local» y «sugerencia API». Coste: si la fase 5 se corta —y es la
  primera de la lista de cortes—, los tres se caen con ella y la deuda pasa al sprint 003.
- **(c) mirada propia, 16-bis, corta** — solo la banda con la ficha explicándose, antes de la fase 3.
  Coste: una parada más; beneficio: la fase 3 puede pagarlos y no dependen de la fase 5.

**Y la misma respuesta resuelve siete campos más**, que están en la misma situación —dato vivo, sitio
sin copy— y que hoy la lista de deuda manda a «fase 3» sin nombrar su mirada:

| Dónde | Campos |
|---|---|
| Pantalla de **Idioma** | `EstadoDeEscucha.motor` · `QueSabeTranscribir.motor` · `.techo` · `.motivo` |
| Pantalla de **Corpus** | `EstadoDelCorpus.carpeta` · `.secciones` |
| **Transcript** de la banda | `Turno.hastaMs` |

La lista completa y su estado viven en `tests/unit/contrato-con-lectores.test.ts` (constante `DEUDA`),
que falla en los dos sentidos: si aparece un huérfano nuevo y si una deuda sobrevive a su pago.

### Pregunta B — ¿qué enseña la pantalla de Idioma del diccionario que la fase 1 construyó?

Aquí el punto de partida es mejor de lo que parecía: **`idioma.html` ya tiene un estado «diccionario
técnico» aprobado en la mirada 4** («Idioma muy completo, incluso mejor de lo que pensaba»). Dibuja
cuatro piezas (`docs/diseno/idioma.html:169-205`):

1. **«Corregido por tu diccionario»** + la cuenta + tres ejemplos tachado → resaltado;
2. **«De dónde salen los 312»** — tabla: *de tu corpus* / *añadidos por ti*;
3. **«Añadir un término»** — un formulario de dos campos dentro de la app;
4. **«Corregir no es inventar»** — la franja que promete que solo sustituye lo que el usuario escribió.

Lo que la fase 1 construyó **no encaja entero** con eso, y ahí está la pregunta:

- la pieza **2 encaja tal cual** y es fidelidad, no mirada: el motor sabe cuántos términos tiene y
  cuántos vinieron del corpus (`con_nombres_del_corpus`, ≥5 caracteres), así que la tabla se puede
  pintar con datos de verdad hoy;
- la pieza **3 no existe**: los términos se añaden **editando un archivo**
  (`~/Library/Application Support/com.aiapps.copiloto-consultor/diccionario.yaml`, permisos 600). Un
  formulario haría escribir al módulo protegido, que es lo que la enmienda 1 del ADR 002 prohíbe a
  propósito. Enseñar **la ruta del archivo** y cómo se edita es **copy nuevo** → mirada;
- la pieza **1** necesita que las correcciones del turno se guarden para poder enseñarlas; hoy
  `corregir(&self, …)` devuelve el texto y no lleva registro. Enseñarlas **en vivo** sí es posible y
  muere con la sesión, pero es una decisión de producto, no un detalle;
- la pieza **4** se puede pintar ya: es verdad literal del motor.

Las opciones:

- **(a) mínimo honesto ahora** — la tabla de origen (pieza 2) + la franja (pieza 4) + una línea con la
  ruta del archivo; la fila «diccionario técnico» sale de «lo que todavía no existe», donde la fase 1
  ya la quitó. Es lo más cerca de la maqueta aprobada que se puede estar sin inventar.
- **(b) mínimo + las correcciones en vivo** (pieza 1) durante la sesión, sin guardar nada.
- **(c) esperar** y que Idioma entre completa en la mirada 17, con el formulario o con la decisión
  explícita de que el archivo es la única puerta.

Mientras no haya respuesta, `docs/MANUAL-DE-USO.md` es el único sitio donde el usuario se entera del
archivo y de su formato — y eso ya está escrito, con sus números medidos y sus cuatro limitaciones.

---

# Fase 2 — El modo solo audio (C15)

La voz que SALE. `voz/` ya era la voz que entra —VAD, turnos, eco—, así que el módulo nuevo se
llama **`habla/`**: dos módulos con el mismo nombre para las dos direcciones del sonido es la clase
de confusión que se descubre tarde. Desviación 1 del plan, ya declarada.

## Lo que se construyó

| Pieza | Dónde | Qué hace |
|---|---|---|
| El puente | `src-tauri/nativo/Habla.swift` | `AVSpeechSynthesizer` por C ABI: `ag_habla_voz` · `decir` · `callar` · `hablando` |
| El motor | `src-tauri/src/habla/mod.rs` · `apple.rs` | el `trait Voz`, la **muda** de primera clase, y todo el `unsafe` en una sola puerta |
| El candado | `habla::cabe_decirla` | **función pura** sobre cinco booleanos: las cinco maneras de callarse |
| El contrato | `contrato.rs` → `contrato.generado.ts` | `LaVoz` con sus tres muestras, cruzando la costura |
| El cableado | `lib.rs` | `⌘⇧V`, `⎋`, el opt-in automático, y el alto de la banda |
| La banda | `src/componentes/Banda.tsx` (`BandaDeVoz`) | los tres estados de 44 px |
| El corte | `corte.rs` | **`Pieza::Voz`**, la primera que se corta |

`build.rs` pasa de compilar un archivo de Swift a compilar **una lista** (`EL_PUENTE`): los dos
exportan símbolos de C, ninguno importa al otro, y partirlos en dos librerías solo añadiría una
manera nueva de que falte la mitad.

## Las cuatro decisiones que hubo que tomar, y por qué

### 1. El candado con un dispositivo externo: **se habla**

`Salida::puede_haber_eco()` tiene **tres** respuestas, no dos: `Some(true)` con los altavoces
internos, `Some(false)` con auriculares por el conector, y **`None` con cualquier dispositivo
externo** — porque desde Core Audio un USB o un Bluetooth puede ser un casco o un altavoz de mesa y
no se distinguen. **Eso incluye los AirPods**, que es como la mayoría de la gente hace una
videollamada (`capture/nativo.rs:216` — todo lo que no sea transporte interno cae en `Otra`).

Las dos salidas eran malas: negarse con `None` deja C15 inservible para el caso normal —y una
funcionalidad que nunca corre es peor que una limitación declarada—; hablar con `None` acepta que
algún día el sonido salga por un altavoz de mesa. **Se habla**, por tres razones en orden de peso:

1. **La app ya trazó esta línea y el usuario la aprobó.** El aviso de eco de Sesión se enciende SOLO
   con los altavoces internos (`Sesion.tsx:38`), y esa pantalla pasó la mirada 13. Trazarla distinta
   aquí sería que la misma app respondiera dos cosas a la misma pregunta.
2. Un dispositivo externo en una videollamada **es un casco**, porque es lo que el consultor se pone
   para no oírse. No es una certeza y no se escribe como tal en ningún sitio.
3. Quien lo enciende es el usuario, con una tecla, sabiendo por dónde le suena el Mac.

**Está en el manual como limitación, con la frase «si tu salida es un altavoz externo, no enciendas
el modo», y cambiarla es UNA línea de `cabe_decirla`.** La frase precisa que lo cerraría —«no sé si
"AirPods Pro" es un casco»— necesita `Salida.nombre`, que es uno de los campos del contrato sin
lector y tiene su sitio pedido para la mirada 17.

### 2. Las fichas «sin resultado» NO se leen

Su titular son **las palabras del cliente** («nada sobre "certificación ISO 27001"»). Leérselas al
usuario sería sacar el transcript del cliente por el altavoz, que es exactamente lo que la regla 1
no permite que salga de la RAM — y por un camino que ninguna capa vigilaba. La banda sí las pinta.

### 3. `⎋` se registra **solo mientras el modo está encendido**

Un Escape global permanente se lo quita a la reunión —en Meet es la tecla de salir de pantalla
completa— y a todas las demás apps del Mac, para una función que existe unos segundos por ficha. Se
coge al encender y se devuelve al apagar, y las dos cosas se dicen en el log. Está en el manual con
su ojo, igual que `⌘⇧T`.

### 4. `habla/` es **módulo protegido**, aunque lo que dice sea del usuario

Por la frontera del ADR 002 no le tocaría: la ficha sale del corpus del consultor, no del cliente.
Está en `PROTEGIDOS` por la **API**: `AVSpeechSynthesizer` trae `write(_:toBufferCallback:)`, que
convierte lo que va a decir en búferes de audio — una manera de dejar en un archivo la evidencia del
consultor leída en voz alta. Eso sería una grabación de la reunión con otro nombre. El módulo no la
usa, y desde aquí no puede empezar a usarla en silencio.

## La pieza que el kill-switch no tenía

`corte::Pieza::Voz`, y va **la primera de las ocho**. El orden de `TODAS` se decide por lo que sigue
entrando —grifo antes que vaso—; esta es la única excepción, y se decide por lo que el cliente
**percibe**: es lo único de la lista que se oye desde el otro lado de la llamada. Si el usuario pulsa
`⌥⎋` delante de su cliente y la app sigue leyéndole una ficha en voz alta, no hay informe que
arregle eso.

No hubo que acordarse de venir a añadirla: `Pieza::orden` y `suerte_en_este_sprint` son dos `match`
sin comodín y **no compilaba**. La cuenta pasa de «6 de 7» a **«7 de 8»** en Honestidad, en el test
de `cuaderno.test.tsx` y en el manual. La que falta sigue siendo la lectura de pantalla.

## El estado que la mirada 16 no dibujó — **mirada 16-bis, pendiente**

Construir el modo hizo aparecer una pregunta que la maqueta no contestaba: **¿qué enseña la banda de
44 px mientras el modo está encendido y CALLADO?** La mirada 16 tenía «hablando» y «sin auriculares»,
y ninguno de los dos es eso.

No es un caso raro: es **la mayor parte del tiempo**. El modo dura toda la reunión —bajarlo y
subirlo por cada ficha obligaría a rehacer el acople en cada transición, que son varias idas y
vueltas a otro proceso por la Accessibility API— así que entre una ficha y la siguiente la banda
está a 44 px sin nada que decir.

**Lo que se propone no inventa ni una palabra:** es la MISMA `linea-b` del estado «hablando» sin el
verbo, con la línea de la ficha que acaba de leerse y su fuente — texto que la banda de 88 px ya
enseña desde la mirada 11. Sin ficha todavía, dice lo que dice hoy la banda en reposo.

Está **maquetado** (`banda.html`, estado `voz-espera`, con su nota de «Qué mirar») y **construido**,
porque el contrato lo obligaba: `LaVoz` cruza la costura con tres campos y el gate de campos sin
lector exige que los tres tengan quien los lea — dejar `diciendo` sin consumidor habría sido añadir
un huérfano nuevo en el sprint que los está pagando. Se presenta en el gate de esta fase.

## Los cuatro gates, y sus rojos

| Gate | Qué impide | Demo en rojo |
|---|---|---|
| `habla` en `PROTEGIDOS` | que el módulo que dice la ficha aprenda a escribirla en disco | ✅ abajo |
| `LaVoz` en el contrato (regla 19) | que Rust y TS lean el payload distinto, como el C1 del S1 | ✅ abajo, **por los dos lados** |
| `la_app_nunca_habla_por_los_altavoces_internos` | que la app hable por donde el cliente oye | ✅ abajo |
| `Pieza::Voz` | que el kill-switch deje la voz hablando | el compilador: `orden` es un `match` sin comodín |

### Rojo 1 — `habla/` no puede escribir en disco

Plantada una función que guarda en un archivo lo que se va a decir:

```
✕ src-tauri/src/habla/mod.rs:223  /std::fs\b/       →  pub fn a_voz_plantada(t: &str) { std::fs::write("/tmp/ficha-dicha.txt", t).ok(); }
✕ src-tauri/src/habla/mod.rs:223  /\bfs::write\b/  →  pub fn a_voz_plantada(t: &str) { std::fs::write("/tmp/ficha-dicha.txt", t).ok(); }
✕ 2 uso(s) de disco/red en módulos efímeros. Regla dura 1 (estándar 4-T).
```

Verde al revertir. **Los 21 archivos inspeccionados se nombran en la salida**, así que también se ve
que el módulo entró en la lista y no solo que el barrido corrió.

### Rojo 2 — la costura, por los dos lados

`LaVoz.diciendo` renombrado a `hablando` **solo en Rust**:

```
test contrato::tests::el_contrato_del_repo_es_el_que_rust_emite ... FAILED
  Regenéralo con `ACTUALIZA_CONTRATO=1 cargo test contrato` y mira qué cambió: si un campo se
  renombró, la interfaz que lo leía dejó de leerlo.
```

Y al regenerarlo —que es lo que haría quien intentara «arreglarlo» sin mirar—, el otro lado:

```
src/contrato.generado.ts(236,5): error TS2353: '"hablando"' does not exist in type 'LaVoz'.
src/contrato.generado.ts(242,5): error TS2353: …
src/contrato.generado.ts(248,5): error TS2353: …
```

**Esto es el C1 del sprint 001, reproducido en dos minutos y atajado en los dos sitios.** Aquel
defecto —Rust etiquetando por dentro y el webview leyendo por fuera— pasó 153 tests verdes y lo cazó
un auditor. Este no llega al commit.

### Rojo 3 — el candado

La comprobación de la salida sustituida por `if false`:

```
test habla::pruebas::con_los_altavoces_internos_se_calla_y_dice_por_que ... FAILED
test habla::pruebas::el_impedimento_que_se_cuenta_es_el_mas_grave ... FAILED
thread 'la_app_nunca_habla_por_los_altavoces_internos' panicked at contra-el-mac-de-verdad.rs:1322:
  la app habló por los altavoces internos: el cliente la habría oído
```

Tres tests en rojo, y el tercero **contra el Mac de verdad**, que es el que mide la salida real. Los
otros dos siguieron verdes a propósito: `con_un_dispositivo_externo_se_habla` y
`sin_saber_por_donde_suena_se_habla` afirman lo que la decisión 1 declara, y un candado abierto no
los contradice. Verde los veinte al revertir.

## Lo que este sprint NO puede afirmar de la voz

El plan pedía «un test que demuestre que **el disparador no se oye a sí mismo**». Hay dos mitades:

- **La de arriba, probada:** la app solo habla cuando macOS no dice que el sonido sale por los
  altavoces internos. Si no sale por ahí, el micrófono no puede captarlo.
- **La de abajo, construida en el sprint 001 y verificada aparte:** el tap del sistema nace con
  `initMonoGlobalTapButExcludeProcesses` y nuestro propio PID (`capture/nativo.rs:532`), así que la
  pista del cliente no puede traer nuestra voz aunque suene.

**Lo que queda fuera, dicho:** con un dispositivo externo la app habla y **sí podría oírse a sí
misma por el micrófono**. El tap la excluye; el micrófono no. Esa parada es del gate ⭐ —con
auriculares puestos y sin ellos— y está escrita como tal.

## Medido en este Mac

```
[habla] voz «apple-avspeechsynthesizer» · es-ES=true · en-US=true
[habla] salida de audio de este Mac: Altavoces · ¿eco? Some(true)
[habla] el puente encoló y calló · idioma es-ES — MEDIDO
[habla] el candado, contra Altavoces
[habla] con esta salida la app se callaría: el sonido saldría por los altavoces y el cliente te oiría — MEDIDO
```

El paso de evidencia de la CI (`--nocapture`) suma `la_voz_de_este_mac` y `la_app_nunca`: en el
runner no hay voces ni dispositivo de audio, así que el puente no se cruza — **y el log lo dirá con
todas las letras** en vez de salir `ok` a secas. Es la lección de la fase 1, aplicada al nacer.

## El tercer filo: verlo correr EN EL MODO — y el cuelgue que solo aparece ahí

253 tests de Rust verdes, 171 del webview verdes, clippy limpio. Se arrancó `pnpm tauri dev` y se
pulsó `⌘⇧V`. **El registro se cortó en seco:**

```
[acople] reacople: permiso=true app=«—» ventanas=0
[acople]   · no había nada acoplado que reajustar
```

Y nada más. Ni la línea de `⎋ registrada`, ni la de `modo solo audio ENCENDIDO`. La ventana seguía
respondiendo, la app seguía viva — **y `⌥⎋` ya no hacía nada**. El kill-switch, muerto.

**La causa:** `⌘⇧V` llega por el manejador de atajos globales, y **registrar un atajo desde dentro de
ese manejador bloquea el plugin**: se queda esperando un candado que tiene cogido el propio hilo que
lo llamó. Lo que se ve desde fuera es peor que un error — nada falla, nada se queja, y ningún atajo
vuelve a funcionar en toda la sesión.

**El arreglo** es una línea de diseño, no un parche: coger y soltar `⎋` ocurre **en otro hilo**
(`con_el_callar`). El manejador vuelve, suelta el candado, y el registro pasa microsegundos después.

**Lo que esto es, dicho con su nombre:** el tercer filo de la regla 15 —*¿lo viste correr EN EL MODO
en que el usuario lo va a usar?*— donde el modo es «con el dedo en la tecla». Ninguna de las tres
capas de tests podía verlo: el candado es del plugin, no del código de esta app, y solo existe
cuando hay un manejador de atajos de verdad ejecutándose.

### Y el modo, corriendo de verdad

```
[habla] voz «apple-avspeechsynthesizer» · ¿hay para es-ES? true · ¿para en-US? true
[habla] ⌘⇧V «modo solo audio» registrado
[ventanas] «banda»: 1470x88 en (0,868) escala 2
[habla] ⌘⇧V: modo solo audio ENCENDIDO · banda a 44 px
[habla] todavía no he oído nada del cliente: el modo queda a la espera
[habla] ⎋ registrada MIENTRAS dure el modo · OJO: durante estos segundos la tecla no le llega a la reunión
[habla] ⌘⇧V: modo solo audio APAGADO · banda a 88 px
[habla] ⎋ devuelta al sistema
[corte] el modo solo audio estaba encendido: callado y apagado
[corte] ⌥⎋: 7 de 8 piezas cortadas · red 0 B
[habla] ⎋ devuelta al sistema
```

Las cuatro cosas que había que ver corriendo: el modo enciende y apaga, la banda cambia de alto,
`⎋` se coge y se devuelve, y **`⌥⎋` calla la voz antes que nada y apaga el modo**.

## El gate de FIDELIDAD — y el defecto que tenía su umbral

Los tres encuadres de 44 px entran en `capturar-fidelidad.mjs`, y la primera pasada salió en rojo:

```
6.089 %  banda · voz-espera · light · en
1.555 %  banda · voz · light · es
0.154 %  banda · voz-sin · light · es
```

**Los dos primeros eran defectos míos, en la maqueta.** Al escribir la mirada 16 acorté la fuente a
«propuesta · §3.2 Alcance» y la línea a una frase más corta, en vez de usar las que el producto
compone de verdad —la banda de 88 px ya escribía `<span class="unidad">propuesta</span> Páramo Azul ·
§3.2 Alcance` desde la mirada 11—. La maqueta es el contrato de forma: corregida ella, los dos
bajaron a 0,13 %.

**El tercero no era un defecto de nadie, y ahí estaba lo interesante.** El suelo de este gate tenía
dos partes mezcladas bajo un solo número:

| Parte | Qué es | Cómo escala |
|---|---|---|
| **la mordida del marco** | 58 px en las nueve filas de abajo: el recorte de la maqueta sale de una página con marco redondeado | **constante** — los mismos 58 px a 88 px que a 44 |
| **el antialias del texto** | la misma fuente pintada dos veces varía un punto | con la **tinta**, o sea con el área |

Medir las dos en porcentaje hacía que la primera **subiera al encoger el artefacto**: 58 px son el
0,056 % de una banda de 88 px y el **0,112 % de una de 44**. Por eso los seis encuadres del modo
aparecieron pegados al umbral y dos lo pasaron sin que hubiera nada que arreglar.

Se probó lo contrario —umbral en píxeles absolutos— y tiene el defecto simétrico, y se comprobó: con
un techo de 120 px **las nueve pantallas del cuaderno se ponían en rojo a 0,03 %**, porque 960 × 640
tiene mucha más tinta.

**El arreglo separa las dos partes en vez de elegir una unidad:** la mordida **se deja fuera de la
comparación** (`MARCO = 9` filas, con su medida escrita: 2,2,2,4,4,6,8,12,18 px) y lo que queda —el
antialias— se sigue midiendo en porcentaje, que es su unidad. El umbral se queda en **0,15 %** y
ahora quiere decir lo mismo a cualquier alto. El suelo real bajó de 0,084 % a **0,072 %**: el gate
quedó **más estricto** que antes en los 72 encuadres, no más laxo.

> **Lo que casi pasa, y conviene dejarlo escrito:** el camino fácil era subir el umbral a 0,2 %
> «porque da falsos positivos en la banda pequeña». Habría funcionado, y habría aflojado el gate
> para las nueve pantallas grandes, que son la mayoría — sin que nadie lo notara jamás.

**Rojo 4, con la matemática nueva:** quitado el `·` del estado «voz» en el producto ⇒ **3,4 %** en los
ocho encuadres del modo, veinte veces el umbral y cuarenta veces el suelo. Verde al revertir: 72
encuadres, cero desbordes, cero errores de página.

## La CI, con sus tres checks en verde — y una asimetría que conviene saber

```
build-escritorio   pass   9m12s
e2e                pass   52s
quality            pass   31s
```

**Conclusión propia por check** (regla 15, segunda pregunta): ninguno `skipped`, ninguno arrastrado
por un `needs:` de otro.

Y el paso de evidencia destapó algo que el sprint anterior no podía saber. El runner reparte las dos
mitades del habla de forma **distinta**:

```
│ pregunta-es.wav  sin modelo de es-ES en esta máquina: no se mide
│ mezcla-en.wav    sin modelo de en-US en esta máquina: no se mide
sin modelos de voz en esta máquina: el WER no se pudo medir en ninguna pista
…
[habla] voz «apple-avspeechsynthesizer» · es-ES=true · en-US=true
[habla] el puente encoló y calló · idioma es-ES — MEDIDO
[habla] con esta salida la app se callaría: el sonido saldría por los altavoces y el cliente te oiría — MEDIDO
```

**`macos-latest` no trae modelos para RECONOCER, y sí trae voces para SINTETIZAR.** Así que:

| | En la CI | Dónde se mide, entonces |
|---|---|---|
| WER, y las dos transcripciones del S1 | **no mide** | el Mac del desarrollador · `/release-check` y el gate ⭐ |
| El puente de `habla/` y el candado | **MIDE, en cada PR** | aquí mismo |

La diferencia importa porque invita a generalizar mal: «la CI no tiene voz» habría hecho declarar
como no verificable un puente que **sí** se verifica en cada PR. La nota de memoria del proyecto
queda corregida con la distinción.
