# Sprint 001 «La banda y la ficha» — bitácora

> Registro vivo del sprint. Decisiones, fricciones (K#), gates con su demo en rojo y
> desviaciones del plan. La planeadora lee esto; no se le reporta a mano.

## Fase 0 — Setup y verificación de supuestos (2026-09-20)

### Verificación de supuestos del kit escritorio (primera app que lo usa)

| Supuesto del kit | Resultado |
|---|---|
| Hooks activos (`githooks`, `pre-commit` ejecutable) | ✓ |
| Scripts `typecheck · lint · test · test:e2e · verify:ephemeral · prepare` | ✓ |
| `ci.yml` con `build-escritorio` | ✓ |
| `playwright.config.ts` → `pnpm preview --port 3000` | ✓ (pero ver **K1**) |
| `cargo test` corre en `src-tauri/` | ✓ |
| `pnpm peers check` limpio | ✓ (eslint 9.39.5 por lockfile) |
| Tailwind: `@tailwindcss/vite` instalado pero sin cablear | deuda declarada del estampado, **pagada** |

### Fricciones encontradas (K#)

- **K1 — Playwright venía mobile-first en una app de escritorio.** El kit trae un proyecto
  `devices["Pixel 7"]` porque las apps del pipeline son mobile-first; Angel Ghost no lo es y la
  orden de diseño lo dice («sin viewport móvil»). Dejarlo no era neutro: duplicaba cada prueba
  contra un viewport que el producto no tiene, y un rojo ahí habría costado depuración sobre algo
  inexistente. Sustituido por los **dos tamaños reales** del design system §3.6: `ventana-principal`
  960 × 640 y `banda` 1180 × 200.

- **K2 — Los umbrales de cobertura apuntaban a motores que en esta app no son de TypeScript.**
  El kit fija 80 % sobre `src/lib/**` y `src/engine/**`. En Angel Ghost los motores puros (VAD,
  fin de turno, BM25, disparo) son **Rust**, y los cubre `cargo test`. Dejar los globs del kit
  habría dado un umbral que se cumple porque no mide nada — el peor tipo de verde. Ahora:
  `include: src/**/*.{ts,tsx}` con umbral 50 (regla de UI) y 80 reservado para `src/lib/**` si
  algún día aparece lógica pura en TS. `--coverage` añadido al script `test`, como pedía la orden.

- **K3 — Testing Library no limpiaba entre tests.** Su limpieza automática solo se registra con
  `globals: true` en vitest, y aquí no lo está. Sin ella los renders se **acumulan**: el segundo
  `getByTestId` encuentra dos nodos y falla con un mensaje que parece del componente cuando el
  defecto es del arnés. Lo descubrió el primer test de UI del sprint; arreglado en `tests/setup.ts`
  para todos los que vienen. *(Misma clase que el defecto de la barra sticky en la Etapa de
  Diseño: un fallo del arnés se ve idéntico a un fallo del producto.)*

### Hallazgo que cambia un riesgo del plan

**El bridge nativo no necesita Xcode completo.** Se verificó ANTES de comprometer la fase 3, no
después: `swiftc` con Command Line Tools compila contra `ScreenCaptureKit`, `AVFoundation`,
`CoreAudio` y `Speech`, y `SpeechAnalyzer` está en el SDK (macOS 27 SDK; la máquina corre 26.6.2).
Desaparece el riesgo nº 1 del sprint y con él una descarga de ~10 GB. Queda en el **ADR 001**.

### Qué se construyó

- **`CLAUDE.md`**: regla del efímero verificable en su **forma final** (tabla de qué persiste y
  qué muere), trasplantada de `ordenes/CLAUDE-md-para-app.md`.
- **`src/index.css`**: los tokens del design system v1.7.0 **copiados literalmente** de
  `ghost.css`, más el puente a Tailwind v4 con `@theme inline` — `inline` a propósito: con
  `@theme` a secas los valores se congelan en el build y el tema claro dejaría de existir.
- **`src/i18n/{es,en,index}.ts`**: el diccionario con las cadenas de la banda, verbatim de la
  maqueta. El tipo `Diccionario` deriva la **forma** de `es` (no sus valores) para que ningún
  idioma se quede atrás sin que el compilador lo note.
- **`src/App.tsx`**: la cáscara real (tema e idioma en `<html>`, como la maqueta). **Se borró el
  scaffold `greet()`** de Tauri: era código muerto que además arrastraba la cobertura al 15 %.
- **`src-tauri/src/{capture,stt,corpus}/`**: la estructura con **la frontera del efímero escrita
  en el árbol**, no en una convención. `capture/` y `stt/` protegidos; `corpus/` fuera del barrido
  a propósito (ver ADR 002).
- **ADRs 001 (plataforma y bridge), 002 (persistencia), 003 (observabilidad).**

### Gates nuevos, cada uno visto en rojo antes que en verde (regla 15)

| Gate | Qué impide | Demo en rojo |
|---|---|---|
| `tokens-fieles` | que los tokens del producto se separen de la maqueta | valor cambiado (`--halo`) ⇒ rojo · token borrado (`--ok-tint`) ⇒ rojo · verde al revertir |
| `i18n-fiel-a-la-maqueta` | que el producto invente copy que la maqueta no dice | «Buscando en tu corpus, un momento…» ⇒ rojo nombrando la clave |
| `cascara` | que tema/idioma dejen de vivir en `<html>` | (cubierto por sus 4 aserciones) |

**Gates heredados extendidos al terreno nuevo, y demostrados ahí:** `vocabulario-vetado` y
`maqueta-sin-emojis` ahora barren `src/` y `src-tauri/src/`. El primero se puso rojo con
«indetectable» plantado en `src/i18n/es.ts`; el segundo, con un emoji en `src/App.tsx`.
`verify:ephemeral` pasó de inspeccionar **0 archivos a 2** —antes era un gate que no leía nada— y
se demostró rojo con un `fs::write` plantado en `capture/`.

### Dependabot PR #1 (4 bumps de Actions)

No se mergeó ni se cerró: **está obsoleto, no roto.** Su CI falla en `pnpm peers check` por una
causa ajena a los bumps — la rama nace de un `main` anterior al arreglo `eslint@9` (13cc7e5), así
que resuelve eslint 10 y rompe el peer de `jsx-a11y`. Se pidió `@dependabot recreate` en vez de
resolver el lockfile a mano (regla de dependencias: se deja regenerar, no se pelea). Si no
regenera antes del cierre, se declara en el summary.

### Criterio de fase completa

`pnpm typecheck` ✓ · `pnpm lint` ✓ · `pnpm test` **19/19 con `--coverage` activo** (95 % líneas) ·
`pnpm build` ✓ · `cargo test` ✓ (1 test) · `pnpm verify:ephemeral` ✓ · CI con los tres checks.

## Fase 1 — La banda

### Fase 1a — decisión de diseño no escrita: los seis estados de CONTENIDO de la banda (2026-09-20)

La orden lo exige literalmente: *«cualquier estado que la maqueta no cubra se propone en la
bitácora bajo "decisión de diseño no escrita" ANTES de construirlo»*. Aquí está.

**El hueco.** La Etapa de Diseño decidió la **forma** de la banda (`posicion.html`: 88 · 200 · 44,
acoplada, con asa) y el **contenido** de los estados (`panel.html`, dentro de 380 × 220). Nunca se
escribió el cruce. La maqueta dibujó la banda **tres veces y las tres con una ficha dentro**; el
sprint construye **seis** estados de contenido. Cinco no tenían referencia contra la cual comparar
— y el gate de FIDELIDAD de esta fase se habría resuelto a ojo.

**Qué se hizo antes de construir:** extender la maqueta con `docs/diseno/banda.html`, nueve
estados (los seis del sprint + dos variantes ampliadas + el fallback sin acople), ambos temas,
ambos idiomas, con el CSS ya aprobado y el copy tomado literalmente de `panel.html` y
`posicion.html`. Es la **mirada 11**, propuesta en el plan del sprint y aprobada por el usuario
antes de construir (el plan de miradas es parte del gate).

**Las tres decisiones que la maqueta no había escrito** (en `design-system.md` §9-quinquies):

| # | Decisión | Por qué |
|---|---|---|
| 1 | En la banda, **las acciones son teclas**; los botones vuelven al ampliar | en 88 px de alto dos botones y una frase larga se pelean por el renglón y el primario cae abajo — el anti-patrón §8 que ya mordió dos veces en la Etapa de Diseño |
| 2 | **El asa tiene un trabajo**: «sin verificar» muestra una salida en 88 px y las tres al ampliar | la alternativa era recortar el aviso o inventar un menú. El alto es continuo: 88 es el reposo, 200 el máximo dibujado |
| 3 | **El transcript va a la derecha**, no abajo | una banda ya ocupa el ancho entero y no puede crecer; ocupa la columna de la sugerencia (sprint 2) y así no cambia de alto al encenderse |

Y una **no-decisión declarada**: la `unidad` sigue sin chip en `fuente-b` (a diferencia del
panel), porque la columna derecha de la banda es toda Menlo de bajo contraste y un tercer peso
visual junto a los `kbd` la volvería ruido. El chip sí aparece en las acumuladas de la ampliada,
donde la unidad es lo que distingue una ficha de otra.

**Cambio menor de contenido, declarado:** en la banda ampliada el «+2» se **abre** (las dos fichas
acumuladas, con su unidad). `posicion.html` D3 lo dibujó colapsado, pero su propia nota prometía
que el alto extra era para «la ficha entera, **las acumuladas** y la sugerencia». Con el «+2»
colapsado y sin sugerencia (sprint 2), la mitad inferior de la banda quedaba vacía sin razón.

### El arnés de capturas vuelve al repo — y se le exigió el rojo

En la Etapa de Diseño el arnés de capturas vivió en el scratchpad y **se perdió al terminar**; sus
avisos de desborde se imprimían antes de cada estado y un `tail -n 3` los escondió durante cuatro
fases (queda registrado en la bitácora de diseño: *leer la salida ES el gate, y leerla entera*).
Ahora vive en `scripts/capturar-maqueta.mjs`, declara su árbol al arrancar (regla 17-bis) e
imprime **al final y juntos** tres bloques: desbordes · estados sin recorte · errores de página.

- **Demo en rojo (regla 15):** `--banda-h: 88px` → `58px` ⇒ **12 desbordes** nombrados por estado,
  tema e idioma (`banda → alto +11px`, `cuerpo-b → alto +13px`, …). Revertido ⇒ verde. El gate
  puede fallar, y falla nombrando el estado.
- **Fallo encontrado por el propio arnés, en su primera corrida útil:** escribió **24 de 36**
  recortes del artefacto y no dijo nada. Tomaba `$(".banda")` —el primero del DOM— y los tres
  estados ampliados no tenían recorte. Corregido a recorrer todos los candidatos, **y el hueco es
  ahora un hallazgo impreso**, no un silencio: «estados sin recorte del artefacto». Mismo defecto
  de clase que un `skipped` leído como verde.

### Archivos de la fase 1a

| Archivo | Qué |
|---|---|
| `docs/diseno/banda.html` | **nuevo** — la referencia del gate de FIDELIDAD: 9 estados × 2 temas × 2 idiomas |
| `docs/diseno/assets/ghost.css` | tokens `--banda-h*` (las tres alturas dejan de ser literales) + bloque «estados de contenido de la banda» |
| `docs/diseno/index.html` | tarjeta `01-c` en el recorrido; la portada declara que la décima pantalla la añadió el sprint |
| `docs/diseno/README.md` | mirada 11 en el plan y en la tabla pantalla → funcionalidad |
| `design-system.md` | **v1.8.0** — §9-quinquies; y el registro de cambios, que se había quedado en 1.3.0 mientras el frontmatter iba en 1.7.0 |
| `scripts/capturar-maqueta.mjs` | **nuevo** — el arnés, ya no efímero |
| `tests/unit/tokens-fieles.test.ts` | los tres `--banda-h*` a `NO_APLICAN`, con la razón y quién sí los vigila |

`pnpm test` **19/19** verdes tras cada cambio. Capturas: 36 del escritorio + 36 del artefacto
solo, **cero desbordes, cero errores de página, cero estados sin recorte**.

### Mirada 11 — veredicto del usuario (2026-09-20)

**Aprobada con un cambio** — *«Si me gusta mucho muy bien docs/diseno/banda.html, pero en Sin
resultado esta bien que digas que no hay nada pero sugierele como abordar la situacion. El resto
esta muy muy bien»*. (Abrió el archivo: nombra la ruta y un estado concreto con su crítica.)

**El cambio, resuelto sin una línea de IA.** «Sugerir cómo abordar la situación» suena a LLM y es
exactamente donde la regla del código primero tiene que morder. La app no puede inventar una
respuesta sobre el negocio del usuario —sería lo que promete no hacer— así que el estado sugiere
**dos cosas, las dos deterministas**:

1. **Lo más cercano que SÍ tiene.** La búsqueda no encontró nada sobre el umbral pero sabe qué
   quedó debajo: sale del corpus del usuario, con su fuente, y la app dice sin adornos que
   **ninguno responde la pregunta**. Recuperación, no redacción.
2. **Una maniobra** de un **catálogo versionado de seis** maneras de responder, elegida por reglas
   léxicas sobre lo que preguntó el cliente — el mismo mecanismo del disparo. Hablan de **cómo
   conducirse**, jamás del negocio: por eso pueden ser fijas.

Y no puede **parecer** salida de un modelo: sin acento `halo` ni `i-chispa` (que en este sistema
marcan la síntesis de la IA), en **Avenir** —la voz de la app— nunca en **Charter**, que es la voz
de la evidencia. Cuando exista la síntesis (sprint 2), la maniobra es su **fallback permanente**.

**Cabía en 88 px reordenando, no recortando.** El estado tenía dos renglones ocupados por el
veredicto y la pregunta oída; la maniobra necesitaba un tercero y tres renglones no entran (50 px
de cuerpo contra ~57 px de texto — medido, no estimado). Se fundieron veredicto y pregunta en uno:
**«Nada en tu corpus sobre "certificación ISO 27001"»**, con los términos que realmente se
buscaron. Sale ganando: si la app entendió mal, se ve en el acto. El asa abre el estado ampliado
con la pregunta entera y las tres más cercanas — el mismo trato que «sin verificar».

| Archivo | Qué cambió |
|---|---|
| `docs/diseno/banda.html` | `sin resultado` reescrito · estado nuevo `sin resultado · ampliada` · bloque del catálogo de maniobras con su tabla y la nota de lo que la maniobra NUNCA hace |
| `docs/diseno/assets/ghost.css` | `maniobra-b` y `cercano-b` |
| `design-system.md` | **v1.9.0** — la maniobra dentro de §9-quinquies, con el catálogo entero |

`pnpm test` 19/19 · 40 capturas × 2 encuadres · cero desbordes, cero errores, cero huecos.

### Mirada 11, segunda vuelta — aprobada, con una deuda abierta a propósito (2026-09-20)

*«Así está perfecta la sugerencia, pero la sugerencia estándar… me preocupa, deja solo al
consultor/asesor, pero bueno después lo resolvemos. No quiero de todas maneras que invente una
respuesta, quiero es que le sugiera cómo abordar la situación muy a medida de la situación.»*

Tiene razón y el reparo es exacto: cinco de las seis maniobras se apoyan en algo (una credencial,
una cifra, un plazo, una referencia, un contrato); **la sexta no se apoya en nada** y es
precisamente la que más se va a disparar. El usuario decide aplazarlo, y el requisito queda
escrito para que no se pierda: **sin inventar respuesta, pero a medida de la situación**.

**Lo que abre el camino sin LLM** (para el ADR del sprint 2): «a medida» no exige un modelo, exige
**material**. La app ya sabrá, de forma determinista, qué unidad falta, cuál es la sección más
cercana del corpus, qué dice la ficha del cliente, la jurisdicción, y qué se comprometió ya en
esta reunión. Una maniobra armada con eso —«no tienes nada de certificación; lo más parecido es tu
§5.1 de seguridad de datos: apóyate ahí y ofrece confirmarlo hoy»— es específica **y** sigue sin
afirmar nada que la app no haya leído. El catálogo fijo pasa entonces a ser el último recurso, no
la respuesta normal. Registrado en `design-system.md` §10.

### Fase 1b — la banda construida (2026-09-20)

#### Las tres ventanas, y dónde vive el flag que lo sostiene todo

| Ventana | Qué es | Protegida de la captura |
|---|---|---|
| `principal` | 960 × 640, las pantallas del cuaderno (fase 2) | no |
| `banda` | ancho de pantalla × 88 (asa → 200), pegada al borde inferior | **sí, y es la única** |
| `relleno` | la misma geometría, sin contenido, justo debajo | no, **a propósito** |

El **riesgo nº 1 del plan** era que el relleno heredara el flag de protección al copiar el
constructor de la banda: la franja volvería a mostrar lo que hay detrás y **nadie se enteraría**,
porque desde el Mac del consultor las dos versiones se ven idénticas. Se resolvió quitándole el
sitio donde podía pasar: **el flag vive solo en `tauri.conf.json`** y las ventanas se construyen
con `from_config` (verificado en el código de `tauri-runtime-wry`: `with_config` aplica
`content_protected`). No hay nada que heredar en un constructor porque el constructor no lo lleva.

Encima, `invariante_de_proteccion()` corre en `setup()` **antes de abrir nada** y aborta el
arranque si el invariante no se cumple. Una banda que se abre sin su promesa es peor que una
banda que no se abre.

Capabilities por ventana: el relleno solo tiene `core:event` y `core:window` — lo justo para que
Rust lo mueva. Nada que pueda cargar ni mostrar algo.

#### La banda en React — sin copiar el design system

Dos decisiones que quitan deriva en vez de vigilarla:

- **El producto importa `ghost.css`**, no lo copia. `design-system.md` ya declaraba ese archivo
  como `fuente_en_codigo`; ahora es literal. La banda construida y la maqueta comparten el mismo
  CSS, así que el gate de fidelidad compara dos cosas que **no pueden** separarse.
- **El sprite de iconos se extrae de `iconos.js` en compilación** (`?raw`). Una sola fuente para
  los cuarenta símbolos. `<use href="#i-lo-que-sea">` con un id inexistente no lanza error: dibuja
  **nada**, y el estado se queda sin símbolo — justo lo que la regla del daltonismo prohíbe.

#### Gates nuevos, cada uno visto en rojo en el mismo commit (regla 15)

| Gate | Demo en rojo | Qué nombró |
|---|---|---|
| `proteccion-de-captura` (config) | `contentProtected: true` en el relleno | «protegidas: [relleno, banda] — tiene que ser exactamente ["banda"]» |
| `proteccion-de-captura` (código) | `.content_protected(true)` plantado en el bucle que crea las dos ventanas | `ventana/mod.rs:103`. **La config no lo habría visto**: seguiría siendo correcta. Por eso son dos caminos |
| `invariante_de_proteccion` (Rust) | el mismo cambio de config | «hay 2 ventanas protegidas (relleno, banda), y solo la banda puede estarlo» |
| `iconos` | renombrar `i-flecha` en la maqueta | «Banda.tsx pide «i-flecha», que el sprite no declara» |
| `sistema-sin-sala-de-diseno` | plantar `.mq-bar .banda {…}` en `ghost.css` | `ghost.css:498`. Ahora que el producto importa ese archivo, lo que entre ahí viaja al binario |
| **fidelidad** (píxel a píxel) | `padding-left: 17px` en la cabecera de la banda del producto | los **40** encuadres en rojo, 1,2–1,9 % |

Y uno que no es de código: **el relleno no dibuja nada** — ni texto, ni banda, ni iconos, ni
imágenes. Es lo único que una captura encuentra donde vive la banda; si alguien le mete contenido
«solo para depurar», eso es exactamente lo que vería el cliente, y no hay aviso posible porque
desde este lado el relleno está tapado por la banda y no se ve nunca.

#### El gate de FIDELIDAD se mide, no se ojea

`pnpm fidelidad` captura el mismo encuadre dos veces —la maqueta aprobada y el producto servido
desde el build— en **diez encuadres × dos temas × dos idiomas**, y los compara **píxel a píxel**
dentro del navegador (canvas; sin dependencias nuevas). Una hoja de contacto en
`docs/fidelidad/S1-banda.html` los pone en pareja para mirarlos.

Comparar de verdad no es ceremonia: **un ojo cansado aprueba una banda desplazada 3 px**, y esa
banda ya no obedece a la maqueta. El umbral es **0,15 %**, con su suelo medido: la banda de la
maqueta vive dentro del escritorio de referencia, cuyo marco redondeado le muerde la última fila
—73 px sobre 103 000, todos en `y = 85..87`— y eso es del encuadre, no del producto. Un
desplazamiento real de texto pasa del 2 %, así que 0,15 % separa artefacto de defecto sin holgura
de sobra. Un umbral generoso «por si acaso» es un gate que no puede fallar.

**Resultado: 40/40 por debajo del umbral, máximo 0,070 %.**

#### Lo que el gate encontró — tres defectos que ninguna captura mirada a ojo habría dado

1. **El *preflight* de Tailwind reescribe el design system.** Pone `display: block` en todo `svg`;
   el contador de red partía en dos renglones («↑» arriba, «0 B» abajo) solo en el producto. Y al
   ir a arreglarlo apareció la causa de fondo: **el contador estaba escrito como `.barra .red`**,
   es decir como hijo del panel, así que dentro de la banda perdía su anatomía entera —ni Menlo,
   ni cifras de ancho fijo, ni verde, ni una sola línea— **también en la maqueta**. Se promovió a
   componente (`.red`) y `.ic` declara su propio `display`, para que ningún reset de al lado
   vuelva a moverlo. Arreglado, la maqueta y el producto mejoraron a la vez.
2. **`index.html` seguía siendo el andamio de Vite**, con `<html lang="en">`. Eso pisaba la
   detección de idioma: un Mac en español veía la app en inglés, y la primera pasada del gate
   capturó los cuatro cruces de tema e idioma… en inglés los cuatro. El archivo ya no pinta el
   idioma: lo fija la cáscara desde el sistema. De paso murió el resto del andamio (`vite.svg`,
   `tauri.svg`, `react.svg`, `App.css`, el título «Tauri + React + Typescript»).
3. **El transcript se desviaba 2,5 %** en los cuatro cruces, y a ojo parecía idéntico. La causa:
   en la maqueta «cliente» y «14:01» son dos elementos flex (el espacio cae fuera del par
   `<span lang>`), y en el producto uno solo — 3 px de `gap` de diferencia que empujaban toda la
   cita. La hora pasa a ser **un elemento con nombre** en los dos lados, con cifras de ancho fijo,
   que es lo que debió ser desde el principio: un dato, no «lo que quedó del renglón».

> **Regla que sale de aquí, para el design system:** dentro de un contenedor flex, **cada dato
> lleva su propio elemento**. Dejar que el hueco lo decida dónde cae un espacio del HTML funciona
> hasta que dos plantillas parten el texto por sitios distintos — y entonces la diferencia es de
> 3 px y no la ve nadie.

#### El arnés de capturas, tercera corrección — y el patrón

Ya volvió al repo en la fase 1a; en esta fase falló **dos veces más**, las dos por lo mismo:
- escribía **24 de 36** recortes y callaba (tomaba el primer `.banda` del DOM; los tres estados
  ampliados no tenían recorte). Ahora los huecos se **imprimen**;
- la barra de estados de la sala de diseño es `position: sticky`: al desplazarse para fotografiar
  la banda ampliada **se le montaba encima**, y la referencia del gate salía con media banda
  tapada por botones. Ahora se apaga la sala de diseño entera antes de cada recorte.

El patrón es el mismo las tres veces: **un arnés de imágenes falla por lo que NO está en el
cuadro**, y eso no se ve mirando el cuadro. De ahí que la comparación numérica valga más que la
hoja de contacto: el 2,5 % del transcript lo encontró la resta, no el ojo.

#### Archivos de la fase 1b

| Archivo | Qué |
|---|---|
| `src-tauri/tauri.conf.json` | las tres ventanas; `productName: Angel Ghost` |
| `src-tauri/src/ventana/mod.rs` | **nuevo** — geometría, ciclo de vida e invariante de protección |
| `src-tauri/src/lib.rs` | comandos `abrir_banda`/`cerrar_banda`; el invariante aborta el arranque |
| `src-tauri/capabilities/{default,banda,relleno}.json` | permisos por ventana |
| `src/componentes/{Banda,Iconos,Relleno,Principal}.tsx` | **nuevos** |
| `src/ventanas.ts`, `src/App.tsx` | **nuevo** / enrutado por etiqueta de ventana |
| `src/i18n/{es,en}.ts` | el diccionario de la banda entera + la muestra sintética (muere en la fase 4) |
| `src/index.css`, `index.html` | CSS de ventana; el andamio de Vite fuera |
| `docs/diseno/assets/ghost.css` | `.red` promovido a componente · `.ic` con `display` propio · `.hora` |
| `scripts/capturar-fidelidad.mjs` | **nuevo** — el gate de fidelidad, con su comparación numérica |
| `docs/fidelidad/S1-banda.html` + `s1/` | la evidencia: 80 imágenes en pareja |
| `tests/unit/{banda,enrutador,ventanas,iconos,proteccion-de-captura,sistema-sin-sala-de-diseno}.*` | **nuevos** |

**Estado:** `typecheck` · `lint` · `test` **50/50 (94,5 % líneas)** · `build` · `cargo test` 8/8 ·
`verify:ephemeral` · `fidelidad` 40/40. 

#### La verificación en vivo de la promesa — lo que se probó y lo que NO

La app corrió de verdad (regla 15, tercer filo). El registro de geometría que se añadió para esto
dice, con la ventana ya asentada:

```
[ventanas] «principal»: 960x641 en (255,175)
[ventanas] «banda»:    1470x88  en (0,868)
[ventanas] «relleno»:  1470x88  en (0,868)
```

**Verificado, con evidencia directa:**

| Qué | Cómo |
|---|---|
| La banda lleva la protección del sistema | `sharingState = 0` y macOS **se niega** a capturarla: `screencapture -l <id>` responde *«could not create image from window»*. La misma orden sobre el relleno sí produce imagen |
| El relleno es opaco y negro, exactamente en el rectángulo de la banda | capturado a solas: 2940×176, **100 % opaco, rgb(0,0,0)** |
| Exactamente una ventana protegida, y es la banda | en ejecución, no solo en la configuración |

**NO verificado — y es lo que más importa:** que el relleno **cubra la franja en una pantalla
compartida**. No se pudo decidir aquí, y las dos herramientas fallan por motivos distintos:

- **`screencapture` mueve lo que mide.** Justo después de una captura, las ventanas de la app se
  reportan en `x=-1530, y=839, 1470x117` cuando en reposo están en `x=0, y=868, 1470x88`. Con la
  herramienta desplazando aquello mismo que se quiere fotografiar, la franja sale «vacía» — y eso
  se lee, con toda la buena fe, como «el relleno no funciona».
- **ScreenCaptureKit** (la API que usan Meet, Zoom y Teams; `scripts/verificar-proteccion.swift`)
  dio **100 % negro una vez** —la franja cubierta, nada filtrado— y **0 % las cinco siguientes**,
  sin cambiar nada. Intermitente.

**Un error propio, registrado porque casi se publica.** Entre medias monté un A/B: con
`visibleOnAllWorkspaces: false` la franja salió negra y con `true` no, y la conclusión «la bandera
de todos los Espacios rompe el relleno» era redonda, grave y **falsa**. Al repetirla con la
bandera en `false`, cinco de cinco dieron 0 %. La bandera queda como estaba, aprobada en el
diseño. Lo que fallaba era el método: **una sola pasada de una medición intermitente no es una
medición**, y una conclusión limpia sobre un fallo grave es justo la que más ganas dan de no
repetir.

**Consecuencia:** la comprobación del relleno queda como **parada ⭐ obligatoria** en llamada real
con pantalla compartida — que era el riesgo nº 3 del plan, ahora con medidas detrás en vez de
cautela genérica. `scripts/verificar-proteccion.swift` viaja en el repo como su instrumento:
inspecciona **solo** la franja inferior y reporta color medio y porcentaje de negro.

#### Lo que la fase 1 todavía debe

Dos comportamientos nativos que **necesitan al usuario delante** para verlos correr de verdad
(regla 15, tercer filo: ¿lo viste correr en el modo en que se va a usar?):

- **El acople** (Accessibility): recortar la ventana de la reunión y devolverla al cerrar y al
  crashear. Exige que el usuario conceda el permiso de Accesibilidad en su Mac.
- **El relleno con el fondo de escritorio** (hoy negro, que es la opción declarada de una tecla y
  no filtra nada). Exige mirar una captura de pantalla compartida real.

Ninguno de los dos cambia la banda que el gate de fidelidad compara: son comportamiento, no UI.

### Fase 1c — el acople y el fondo del relleno (2026-09-20)

Los dos comportamientos nativos que la fase 1 debía. No cambian ni un píxel de la banda que el
gate de fidelidad compara: son comportamiento, no UI — el gate se volvió a correr entero y siguió
en 40/40, con el mismo máximo de 0,070 %.

#### El acople — la banda no tapa la reunión, la reunión se hace sitio

Sin acople, la banda se queda **encima** de la ventana de la videollamada y el consultor pierde
los 88 px de abajo de su reunión. Con acople, esa ventana se **encoge** hasta que su borde
inferior queda justo sobre la franja, y vuelve a su tamaño al terminar.

Tocar la ventana de otra aplicación es la operación más invasiva de toda la app. Se escribió con
tres reglas explícitas, y las tres están en código, no en la documentación:

| Regla | Cómo se sostiene |
|---|---|
| **Se encoge, nunca se mueve** | solo se escribe `AXSize`. La posición no se toca: una ventana que se mueve sola es un susto, una que se acorta por abajo es una que cabe |
| **Se devuelve SIEMPRE** | al cerrar la banda · al salir de la app (`RunEvent::Exit`, que cubre ⌘Q y el menú, por donde `cerrar_banda` no pasa) · **y al arrancar**, si la vez anterior terminó en una caída |
| **Solo se devuelve lo que sigue como lo dejamos** | la huella guarda el marco *leído del sistema después de escribir*, no el pedido; si el usuario redimensionó esa ventana a mano, no encaja y no se toca |

Esa tercera regla es la que hace que **la devolución normal y la devolución tras una caída sean
el mismo camino**: la huella no distingue, y por eso no hay dos funciones que puedan divergir.

**Lo que NO se pide, pudiendo.** La Accessibility API es una llave maestra: da el árbol entero de
cualquier aplicación, sus títulos y su contenido. Aquí se leen **tres atributos** (`AXWindows`,
`AXPosition`, `AXSize`) y se escribe uno. La huella **no tiene dónde guardar un título de
ventana** —el nombre de una reunión es información del cliente, y el estándar 4-T divide por de
quién es la información, no por su formato— y hay un test que lo afirma, para que añadir el campo
«para depurar» tenga que pasar por encima de un motivo escrito.

**Frontera de memoria.** Todo lo `unsafe` vive en `src-tauri/src/acople/ax.rs`: seis funciones de
`ApplicationServices` y dos clases de AppKit. Hacia arriba solo salen `Marco`, `String` y `bool`;
la lógica que decide qué hacer con esos valores es Rust seguro con tests. Las cuatro dependencias
nativas nuevas (`core-foundation`, `objc2`, `objc2-app-kit`, `objc2-foundation`) **ya viajaban en
el árbol de Tauri**: declararlas no añade una descarga, hace explícito que este crate las usa de
primera mano.

#### El relleno pinta el fondo de escritorio, y declara sus límites

`NSWorkspace.desktopImageURL` da la ruta; Rust la lee y la entrega en `data:`; el relleno la
encuadra al tamaño de la **pantalla** (`cover` centrado, que es «Rellenar pantalla», el modo por
defecto de macOS) y la sube para que por la franja asome exactamente el trozo que habría debajo.
Reproducir el **encuadre** y no solo la imagen es lo que hace que la costura no se vea.

Tres límites, escritos en el código en vez de descubiertos por el usuario: **solo la pantalla
principal** · **solo el modo «Rellenar pantalla»** · **hasta 12 MB** (los fondos dinámicos de
macOS son HEIC de varias decenas de MB con todas las horas del día dentro; meter eso por el IPC
para pintar 88 px no compensa). Y en **cualquier** fallo, negro — que no es un modo degradado
sino la otra opción que el diseño aprobó, y la única que no filtra nada.

La imagen se **pide**, no se empuja: un evento emitido antes de que el webview del relleno
registre su oyente se pierde en silencio y la franja se quedaría negra sin que nada lo dijera.
Preguntando, el orden lo pone quien necesita la respuesta.

#### «acoplada» deja de ser una etiqueta

La cabecera de la banda dibuja «acoplada» o «sin acople». Hasta ahora salía de un parámetro de
URL, es decir, era **fija**: exactamente la clase de promesa que esta app existe para no hacer.
Ahora se pregunta a la parte nativa al montar y se escucha el cambio, y la verdad está en **el
mismo archivo de huella que usa la devolución** — así la banda no puede decir «acoplada» mientras
no hay nada que devolver, ni al revés. El parámetro de URL sigue existiendo pero solo manda
cuando está escrito: sin él, el gate de fidelidad no podría recorrer el encuadre «sin acople» en
un navegador.

#### Los dos gates nuevos, y lo que el rojo encontró en ELLOS

Los dos nacen en este mismo commit (regla 15, kit v1.25.0). Y la primera pasada en rojo **no
encontró defectos en el código: encontró que los dos gates no podían fallar**.

| Gate | Primera versión | Qué pasó al exigirle el rojo |
|---|---|---|
| **La huella nace privada** (regla 17-bis a) | afirmaba `modo(ruta) == MODO_ARCHIVO` | **VERDE con la huella en 0o644.** El test leía la misma constante que el código: cambiarla cambiaba también lo que el test esperaba. Un gate medido contra sí mismo |
| **El recorte no invade la franja** | afirmaba `fondo <= franja.y + HOLGURA` | **VERDE con un error de 1 px plantado.** La tolerancia del test era la del propio código, y se tragaba cualquier error menor que ella |

Arreglados —permisos con el **número literal**, invasión **sin holgura** porque el alto se calcula
exacto— los dos fallan como deben:

```
DEMO 1 · MODO_ARCHIVO = 0o644
  la_huella_nace_privada_y_repara_lo_que_encuentre_flojo ... FAILED
  assertion `left == right` failed: la huella quedó legible por otros
DEMO 2 · alto = franja.y - ventana.y + 1.0
  ninguna_decision_de_encoger_deja_la_ventana_dentro_de_la_franja ... FAILED
  franja invadida por 1 px: y=0 alto=900 banda=44
  una_ventana_que_llega_al_fondo_se_encoge_hasta_justo_encima_de_la_franja ... FAILED
  una_ventana_pequena_abajo_no_se_mutila ... FAILED
```

y vuelven a verde al revertir (24/24 en `cargo test`).

> **La lección, que vale para todo el pipeline:** la tercera pregunta de la regla 15 —*¿puede este
> gate fallar siquiera?*— **no se contesta leyendo el test**. Los dos se leían perfectamente. Se
> contesta plantando el fallo. Y las dos formas de gate inalcanzable que aparecieron aquí son
> genéricas y fáciles de repetir: **medirse contra la misma constante que el código**, y **usar
> la misma tolerancia que el código**.

#### Lo que el arranque en vivo encontró, que ningún test podía

La app corrió de verdad, varias veces (regla 15, tercer filo). **Cuatro hallazgos, y ninguno
estaba al alcance de un test**: los tests prueban la decisión y la huella; lo que falló fue quién
dispara, qué devuelve el sistema y qué pasa cuando el proceso muere mal.

**1 · El disparador estaba mal pensado — dos veces.**

```
[acople] al arrancar: permiso=true app=«—» ventanas=0
[acople]   · no hay ninguna otra aplicación al frente
```

«Acopla la aplicación que esté al frente», llamado al arrancar, **se salta a sí mismo**: la
aplicación de delante somos nosotros, que acabamos de abrir la ventana principal. El mecanismo
entero era correcto y no se ejecutó ni una vez.

Segundo intento: disparar cuando la ventana principal **pierde el foco** —el usuario ha vuelto a
su trabajo y delante hay otra aplicación—. Funcionó a la primera:

```
[acople] al volver el usuario a su trabajo: permiso=true app=«Claude» ventanas=2
[acople] reacople: permiso=true app=«Claude» ventanas=2      ← el asa, al soltarla
```

…y **no volvió a dispararse** tras el reinicio del observador de `tauri dev`: si la principal
nunca llegó a tener el foco, `Focused(false)` no llega nunca. Un disparador que depende de un
evento que puede no ocurrir no es un disparador. Tercera versión, la que queda: **un latido
acotado** (cada 1,5 s durante 30 s) que acopla en cuanto hay alguien delante que no seamos
nosotros, y para.

> Es **andamio de la fase 1 y se declara como tal**: en el producto el disparador es la detección
> de la reunión (fase 2), que llama a `acoplar` sabiendo a quién. Lo de debajo —medir, encoger,
> anotar la huella, devolver— es lo mismo y no cambia.

**2 · La devolución tras una caída, probada de verdad y sin querer.** Al cerrar la app con
`pkill` —que es SIGTERM, y **SIGTERM no pasa por `RunEvent::Exit`**— quedaron dos ventanas
encogidas y la huella en disco. Es decir: una caída real. Al relanzar:

```
[acople] la sesión anterior dejó 2 ventana(s) encogida(s): se devuelven
[acople] devolver tras una caída: permiso=true app=«Claude» ventanas=1
[acople]   · … no se toca
```

Medido después con la Accessibility API: la ventana que seguía existiendo volvió a **1249×815**,
su tamaño original exacto. La huella se borró sola. La otra no se tocó porque ya no existía.

**3 · Y ahí el mensaje mentía.** Decía *«cambió de tamaño desde el acople: manda el usuario»*
cuando en realidad la ventana **se había cerrado**. Dos causas distintas por el mismo camino, y el
mensaje solo nombraba una — mandando a buscar un fallo donde no lo había. Ahora dice lo que de
verdad se sabe: *ninguna ventana coincide con la huella (la redimensionaron o la cerraron)*.

**4 · macOS no entrega el fondo de escritorio de este Mac.** `NSWorkspace.desktopImageURL`
devuelve `/System/Library/CoreServices/DefaultDesktop.heic`, que pesa **54 bytes**: un marcador de
posición. Es lo que da el sistema cuando el usuario tiene un fondo dinámico o un aéreo — la imagen
real no se expone. Sin suelo de tamaño, esos 54 bytes viajaban como `data:` perfectamente válido,
el webview no los sabía decodificar y **la franja acababa negra igual… por un camino que nadie
registraba**. Un fallo silencioso indistinguible de un acierto. Se añadió el **suelo de 4 KB** con
su test, y ahora el sistema lo dice en una línea.

> **Consecuencia honesta:** el relleno con fondo de escritorio está construido y es correcto, pero
> **en este Mac no se puede ver funcionando** porque macOS no entrega la imagen. Queda como
> **parada ⭐** en una máquina con fondo estático. Mientras tanto la franja es negra, que es la
> otra opción que el diseño aprobó y la única que no filtra nada.

**Y un aviso sobre `permiso=true`.** En `pnpm tauri dev` el binario lo lanza la terminal, y la
Accesibilidad de macOS se concede al **proceso responsable**: un `true` en desarrollo puede ser el
permiso de la terminal, no el de Angel Ghost. El binario firmado pedirá el suyo. No es un matiz —
es «funcionaba en mi máquina» con nombre y apellidos, y va a la guía de prueba como parada.

**5 · El acople se apuntaba ventanas que no encogió.** La huella lo delató:

```
Code: original 923 -> dejada 923
[acople] al volver el usuario a su trabajo: permiso=true app=«Code» ventanas=1
```

La Accessibility API **aceptó la escritura sin error** y la aplicación mantuvo su tamaño: pasa con
ventanas en pantalla completa, en Split View o con tamaño fijo. Sin comprobarlo, el acople se
apunta una ventana que no cambió, la huella guarda una devolución que no hay que hacer, y la
banda dice **«acoplada» mientras sigue tapando la reunión** — exactamente la etiqueta falsa que
esta app existe para no poner.

> **La regla que sale de aquí:** *«lo pedí» no es «pasó»*. Se cuenta releyendo del sistema, no
> asumiendo que la escritura hizo algo. Vale por los dos lados: acoplar **y** devolver, porque la
> devolución es la mitad que el usuario nota. Ahora, cuando una ventana no se deja, lo dice con
> su causa probable en vez de sumar uno.

#### Lo que la devolución cubre, y lo que no

| Camino | Quién lo devuelve | Probado |
|---|---|---|
| cerrar la banda | `cerrar_banda` | por código |
| salir de la app (⌘Q, menú, última ventana) | `RunEvent::Exit` | **no en vivo** — parada ⭐ |
| **matar el proceso / caída** | la huella, al arrancar la vez siguiente | **sí, en vivo** |
| force quit (SIGKILL) | la huella, al arrancar la vez siguiente | mismo camino que el anterior |

El hueco declarado: si el proceso muere mal y **Angel Ghost no se vuelve a abrir**, la ventana se
queda corta hasta que se abra. Se puede cerrar con un hilo en `sigwait` para SIGTERM/SIGINT; no se
hizo porque el plan resuelve la caída por la huella y esto queda anotado, no olvidado.

#### El asa, el acople y por qué no van en la misma llamada

Arrastrar el asa dispara decenas de ajustes por segundo. Cada acople son varias idas y vueltas a
**otro proceso** por la Accessibility API: hacerlo en cada cuadro convertiría el arrastre en un
tirón y dejaría la ventana de la reunión parpadeando. Se separó en dos comandos: `ajustar_banda`
mueve lo nuestro durante el arrastre, `asentar_banda` rehace el acople **al soltar**. Y el
reacople usa el PID de la huella, no «quien esté al frente»: mientras arrastras, el que está al
frente eres tú arrastrando la banda, y preguntar soltaría la reunión justo al agrandarla.

#### Archivos de la fase 1c

| Archivo | Qué |
|---|---|
| `src-tauri/src/acople/mod.rs` | **nuevo** — geometría pura, huella en disco, la maniobra completa |
| `src-tauri/src/acople/ax.rs` | **nuevo** — Accessibility + NSWorkspace; todo el `unsafe` del crate |
| `src-tauri/src/relleno.rs` | **nuevo** — el fondo de escritorio, con sus tres límites y el negro |
| `src-tauri/src/lib.rs` | comandos del acople · devolución en `RunEvent::Exit` · disparador por foco |
| `src-tauri/src/ventana/mod.rs` | `franja()` y `alto_actual()` — la franja en coordenadas de la API de accesibilidad |
| `src-tauri/Cargo.toml` | cuatro dependencias nativas, ya presentes en el árbol de Tauri |
| `src/acople.ts` | **nuevo** — «acoplada» preguntado y escuchado, no supuesto |
| `src/componentes/Relleno.tsx` | el fondo de escritorio encuadrado; negro ante cualquier fallo |
| `src/puente.ts` | `preguntar` y `escuchar` |
| `src/asa.ts` | `asentar_banda` al soltar |
| `tests/unit/acople.test.tsx` | **nuevo** — 9 tests de las dos cosas que solo fallan en el webview |

**Estado:** `typecheck` · `lint` · `test` **63/63 (85,1 % líneas)** · `cargo test` **24/24** ·
`fidelidad` **40/40**, máximo 0,070 %.

## Fase 2 — Sesión, permisos, kill-switch, contador y honestidad

### Fase 2a — el motor, y el problema que apareció antes de escribir una línea de UI (2026-09-21)

#### Lo que se construyó

| Módulo | Qué hace | Tests |
|---|---|---|
| `sesion/` | detecta la videollamada abierta por **catálogo versionado** de identificadores | 12 |
| `permisos.rs` | lee los tres permisos de macOS; **no los pide** | 8 |
| `red.rs` | el contador de bytes que salieron (B2) | 4 |
| `corte.rs` | el kill-switch `⌥⎋`, pieza por pieza | 3 |

**La detección es un catálogo, no una heurística.** Adivinar por nombre —«algo que contenga
*meeting*»— confundiría un calendario con una llamada y lo haría distinto en cada Mac. El
catálogo viaja versionado en el repo, **se consulta sin red**, y su versión se muestra en pantalla
al lado de lo que afirma: un catálogo sin versión visible no se puede contrastar con nada.

**Meet obligó a una decisión incómoda y se escribe entera.** Zoom y Teams se detectan por su
identificador. Meet es una **pestaña**, y la única forma de distinguir «tiene Meet abierto» de
«tiene Chrome abierto» —que es siempre cierto— es mirar el **título de la ventana**. De ahí tres
consecuencias, y las tres están en código:

- el título de una reunión **es información del cliente** (el estándar 4-T divide por de quién es,
  no por su formato), así que `sesion` entra en los módulos protegidos de `verify:ephemeral`:
  memoria mientras la pantalla lo muestra, y nada más;
- **solo se preguntan títulos a los navegadores del catálogo**, nunca a todo el Mac. Es una función
  pura con su test — sin ella, «leer títulos» significaría leer el nombre de cada documento
  abierto, cada conversación y cada expediente;
- sin permiso no se dice «no hay reunión», que sería mentira por omisión: se dice **«no puedo
  saberlo»**, con su motivo. La diferencia entre las dos es lo único que separa una app honesta de
  una que contesta «no» cuando no sabe.

**Los permisos se leen, no se piden.** Es decisión de la maqueta (*«Tú los concedes en el sistema,
no aquí»*) y además es lo correcto: un permiso pedido en el primer arranque, antes de que la app
haya demostrado nada, se deniega — y un «no» de macOS es muchísimo más caro de deshacer que un
«todavía no». El botón abre el panel de Ajustes del Sistema que corresponde.

**El contador de red existe ya, vacío, a propósito.** Una promesa que se instrumenta cuando llega
la primera conexión llega tarde: el día que el adaptador de LLM se encienda (sprint 2, con su ADR),
el contador tiene que estar puesto desde antes, con su cero comprobado, o no hay contra qué
comparar.

#### Los gates nuevos, y lo que el rojo encontró en ellos — **otra vez**

| Gate | Demo en rojo | Resultado |
|---|---|---|
| ningún archivo de Rust abre un socket | `TcpStream::connect` plantado | **rojo**, nombra archivo y línea |
| ninguna dependencia de Rust es un cliente de red | `reqwest` en `Cargo.toml` | **rojo** |
| ningún archivo del webview sale a la red | `fetch()` plantado en `puente.ts` | **rojo** |
| `sesion` no escribe en disco (efímero) | `fs::write` con un título de reunión | **rojo** |
| el kill-switch no deja piezas sin cortar | una pieza nueva sin resolver | **VERDE — el gate no podía fallar** |

La quinta volvió a pasar lo mismo que en la fase 1: el test comprobaba `TODAS.len() == 7`, y **el
7 lo había escrito yo**. Se medía contra mi propio número, no contra el código.

Rehecho: el gate ya no es un test, es **el compilador**. Dos `match` sin comodín (`Pieza::orden` y
`suerte_en_este_sprint`) hacen que añadir una pieza y no resolverla **no compile**:

```
error[E0004]: non-exhaustive patterns: `corte::Pieza::NotasSinGuardar` not covered
error[E0004]: non-exhaustive patterns: `corte::Pieza::NotasSinGuardar` not covered
error: could not compile `app-copiloto-consultor` (lib test) due to 2 previous errors
```

y el test se queda con lo que al compilador se le escapa — que la lista y los puestos digan lo
mismo —, que también se vio en rojo (`«Banda» está en el puesto 5 de TODAS pero dice ser el 4`).

> **Tercera vez en el sprint, y ya es un patrón con nombre: un gate que se mide contra un número o
> una tolerancia que escribí yo no mide el código, me mide a mí.** Las tres formas encontradas
> hasta ahora: comparar contra la **misma constante** que usa el código · usar la **misma
> tolerancia** que usa el código · comparar contra un **conteo escrito a mano**. Cuando el
> lenguaje puede obligar —un `match` exhaustivo, un tipo—, el gate es el compilador y el test
> solo cubre el resto.

#### Un test intermitente, cazado antes de entrar

Los dos tests del contador compartían un estático y `cargo test` corre en paralelo: se pisaban una
vez de cada muchas. Un gate intermitente es peor que no tenerlo —se aprende a reintentar hasta que
pasa—, así que se turnan con un mutex. Comprobado **cinco de cinco**, que es la lección de la
fase 1 aplicada sin que hiciera falta equivocarse otra vez.

### Decisión de diseño no escrita — «TODAVÍA NO» (mirada 12 propuesta)

**El problema, encontrado al ir a construir las tres pantallas.** La maqueta dibuja el producto
TERMINADO. El sprint 001 entrega un trozo. Las tres pantallas de esta fase tienen cartas enteras
que el producto de hoy no puede sostener: las dos pistas de audio (fase 3), la ficha del cliente,
los búferes de memoria, las notas.

Solo hay dos salidas sin una decisión escrita, y las dos son malas: **pintarlo en verde** —
«Micrófono · Listo» con el audio sin construir es la afirmación falsa que esta app existe para no
hacer— o **quitarlo de la pantalla**, y entonces el usuario no sabe que va a llegar y la pantalla
del sprint 1 se lee como el producto completo.

La orden ya lo había anticipado para dos casos concretos («NDA y radar: *próximamente*, sin
inventar»). Lo que faltaba era la forma, y la maqueta **no tenía ninguna palabra para esto**
(se comprobó: cero apariciones de «próximamente» o equivalente en las nueve pantallas).

**Lo construido para la mirada:** el componente `.estado.pendiente` en `ghost.css`, el icono
`#i-pendiente`, y un estado nuevo **«así se ve hoy · sprint 1»** en las tres pantallas — la misma
pantalla tal y como se entrega, que es además la referencia del gate de fidelidad. Registrado en
`design-system.md` §9-sexies (v1.10.0) y en el plan de miradas del README de diseño.

**Y dos cosas que la maqueta no había escrito y el sistema obliga**, encontradas construyendo:

1. **«Audio del sistema» y «Pantalla» son UN SOLO permiso en macOS.** Se conceden y se caen
   juntos. Se siguen dibujando como dos filas —son dos usos distintos— con una línea que lo dice.
2. **La Accesibilidad sube a la lista principal de permisos.** En la maqueta vivía en su propio
   estado porque era opcional y futura; el acople se entrega en este sprint y es **el único
   permiso que hoy cambia algo**.

**La construcción de las tres pantallas de producto espera a la mirada 12.** El motor no: no
depende de la respuesta.

### Mirada 12 — aprobada (2026-09-21)

> **«Esta muy bien como indica que no todavia no existe en sesion permisos y honestidad»**

Registrada en `docs/diseno/README.md`. Con ella quedan aprobados `.estado.pendiente`, el estado
«así se ve hoy · sprint 1» y las dos decisiones que la maqueta no había escrito (el permiso único
de macOS, la Accesibilidad en la lista principal).

### Fase 2b — las tres pantallas construidas (2026-09-21)

#### El gate de fidelidad deja de estar cableado a un artefacto

Nació para la banda. Cablearlo otra vez para tres pantallas más habría duplicado justo la parte
delicada —medir el ancho de la referencia, apagar la sala de diseño, restar los mapas de bits— y
**una copia de un gate es un gate que se arregla en un sitio y sigue roto en el otro**. Ahora
recorre una lista de artefactos: añadir uno es añadir una entrada.

**52 encuadres** (10 de la banda + 3 del cuaderno, × 2 temas × 2 idiomas), todos bajo el umbral.

#### Seis defectos que encontró la comparación, y ninguno se veía a ojo

Esta es la parte que importa de la fase: **la pantalla «parecía bien» en las seis versiones**.

| # | Lo que medía | Causa |
|---|---|---|
| 1 | `924x640 vs 960x640` | la maqueta centra su artefacto con relleno lateral: si el navegador mide justo lo que mide el artefacto, ese relleno lo **encoge**. Se medía en una ventana y se fotografiaba en otra |
| 2 | **10 %** | el rail del producto no llevaba la clase `item`: sin ella la fila pierde `display:flex`, el hueco y el relleno, y el icono se pega al texto |
| 3 | **9 %** | quité el borde de 1 px del cuaderno «porque la ventana nativa ya es el marco». Eso **sube el contenido entero 1 px**, y un desplazamiento de 1 px ensucia el borde de cada letra de la pantalla |
| 4 | 765 px en la primera fila | quitar la sombra se llevaba por delante la **línea interior** de 1 px que `--sombra-panel` dibuja arriba. La sombra exterior no se ve dentro de una ventana; la interior sí |
| 5 | **0,75 %** | en permisos y honestidad olvidé darle chip al rail en el estado nuevo: la maqueta no dibujaba ninguno y el producto sí |
| 6 | **0,34 %, solo en inglés** | el título de la reunión de muestra estaba escrito en español **fijo**, así que el cruce inglés mostraba texto español dentro de una pantalla inglesa |

> **El patrón, y es el mismo de la fase 1:** los defectos de fidelidad no son de forma, son de
> **un píxel que se arrastra**. El nº 3 es el ejemplar: una decisión razonable —«quita el marco,
> la ventana ya es el marco»— movió la pantalla entera 1 px y valió 9 % de píxeles distintos.
> Ningún ojo lo ve; la resta sí.

#### Y un agujero en el gate del diccionario, que este encontró

El defecto nº 6 llevó a mirar por qué el gate del diccionario **no había cazado** una cadena
inglesa mal. Y apareció otro, peor: el gate normalizaba `’` a `'` antes de comparar, así que
`the client's voice` con apóstrofo recto pasaba en verde mientras la maqueta escribía
`the client’s voice`. El producto renderiza un glifo distinto, la pantalla deja de ser idéntica, y
el gate cuyo trabajo es exactamente eso decía que sí.

Lo cazó el gate de FIDELIDAD, comparando píxeles, **tres pantallas después**. Ahora las entidades
HTML sí se traducen (`&#8217;` ES el mismo carácter, escrito de otra forma) pero los signos
tipográficos **no se normalizan**: se comparan carácter a carácter. Demostrado en rojo.

> **La lección se suma a la lista del sprint:** un gate que *normaliza* antes de comparar está
> decidiendo qué diferencias no le importan. Aquí decidió que el apóstrofo no importaba, y el
> apóstrofo era la diferencia.

#### Lo que corrió en vivo, y la limitación que apareció al probarlo

La app corrió de verdad (regla 15, tercer filo). Se añadió un registro de arranque —**solo
metadatos**: el cliente de videollamada y los cuatro estados de permiso, **nunca el título de la
reunión**, que es información del cliente y un log es un archivo.

```
[corte]    kill-switch ⌥⎋ registrado
[permisos] micrófono=Concedido pantalla=Concedido accesibilidad=Concedido · cara=Concedido
[sesion]   reunión detectada: «Google Meet» · protección Verificada · catálogo v1 · 2026-09-21
[red]      salida acumulada: 0 B
```

Con eso quedan verificadas en vivo las tres lecturas del sistema —y la de permisos es la que más
importaba, porque el micrófono se pregunta por **mensaje de Objective-C a una clase buscada por
nombre**, que es la FFI más frágil de esta fase y la que ningún test podía afirmar.

**La limitación, encontrada probándolo mal.** La primera prueba abrió una página titulada como una
pestaña de Meet… y el detector dijo que no había reunión. No era un fallo: `open -a` la dejó en
una **pestaña de fondo**, y el `AXTitle` de una ventana de Chrome es el título de su **pestaña
activa**. Comprobado con una sonda aparte:

```
pid 59137 · com.google.Chrome
  1 ventana(s)
    [0] AXTitle = «What's new - Google Chrome»
```

Abierta en ventana propia, el detector la reconoció a la primera. **Consecuencia real y
declarada (ADR 005): si el consultor tiene Meet en una pestaña de fondo, la app no lo ve** —
aunque el audio siga sonando. Llegar a las pestañas exige automatizar el navegador, que es
bastante más invasivo que leer un título; queda como decisión futura con su propio ADR.

**Lo que NO se vio correr:** el atajo `⌥⎋`. Se registra —el log lo dice— pero pulsarlo es cosa de
una tecla que esta sesión no puede pulsar. **Parada ⭐** del gate de prueba, junto con el botón de
la pantalla de Honestidad.

#### Archivos de la fase 2

| Archivo | Qué |
|---|---|
| `src-tauri/src/sesion/mod.rs` · `permisos.rs` · `red.rs` · `corte.rs` | **nuevos** — el motor |
| `src-tauri/src/lib.rs` | seis comandos nuevos · el atajo global `⌥⎋` |
| `src/cuaderno.ts` | **nuevo** — lo que las pantallas preguntan a lo nativo |
| `src/componentes/Ventana.tsx` | **nuevo** — el marco del cuaderno, el rail, «todavía no» |
| `src/pantallas/{Sesion,Permisos,Honestidad}.tsx` | **nuevas** |
| `src/componentes/Principal.tsx` | de hueco a cuaderno |
| `docs/diseno/{sesion,permisos,honestidad}.html` | el estado «así se ve hoy · sprint 1» |
| `docs/diseno/assets/{ghost.css,iconos.js}` | `.estado.pendiente` · `#i-pendiente` |
| `design-system.md` | §9-sexies (v1.10.0) |
| `scripts/capturar-fidelidad.mjs` | de un artefacto a una lista |
| `tests/unit/{cuaderno,contador-de-red}.test.tsx` | **nuevos** |

## Fase 3 — Audio en dos pistas + transcripción local

### Fase 3a — el spike, antes de comprometer ningún motor (2026-09-21)

El plan del sprint lo pedía con estas palabras: *«Spike en la fase 3 antes de comprometer el
motor»*. Tres preguntas, y ninguna se podía contestar leyendo documentación: si el audio del
sistema se puede capturar sin bot y sin pedirle nada al cliente, si la transcripción local de
macOS 26 alcanza para una reunión en vivo, y si un puente Swift se deja enlazar dentro del
binario de Rust con solo las Command Line Tools instaladas. Las tres se probaron con programas
de usar y tirar en el scratchpad; ninguno viaja al repo.

#### Pregunta 1 — ¿se puede oír al cliente sin meter un bot en la reunión?

**Sí, con Core Audio process taps** (macOS 14.2+ según la cabecera; el plan decía 14.4).
`AudioHardwareCreateProcessTap` sobre una `CATapDescription` de *mezcla mono global excluyendo
nuestro propio proceso* → dispositivo agregado privado → `AudioDeviceCreateIOProcIDWithBlock`.

```
translate pid 66144 -> 108 (st=0)
AudioHardwareCreateProcessTap -> st=0 tapID=109
formato st=0 rate=48000.0 ch=1 bits=32 flags=9
salida por defecto: BuiltInSpeakerDevice
AudioHardwareCreateAggregateDevice -> st=0 agg=110
AudioDeviceStart -> st=0
RESULTADO: llamadas=374 buffers=1 191488 muestras en 4.0s · pico=0.55
```

48 kHz, **mono**, float32 — el formato lo decide el tap, no nosotros. Excluir nuestro propio
proceso importa: sin eso, el modo solo audio (C15, sprint 2) se oiría a sí mismo.

**El hallazgo que cambia el diseño, y que solo apareció corriéndolo.** La primera pasada devolvió
`0 muestras` y pareció un fallo. No lo era. El control lo dejó claro:

```
=== control: SIN audio sonando ===
RESULTADO: llamadas=0 buffers=0 0 muestras en 2.0s
=== con audio ===
RESULTADO: llamadas=268 buffers=1 137216 muestras en 3.0s · pico=0.75
```

**Sin nada sonando, el callback no se llama ni una vez.** No llegan ceros: no llega nada. La pista
del sistema no tiene «nivel cero», tiene *silencio del que no se entera nadie*. Consecuencia
directa para la pantalla de Honestidad y para la de Sesión: **«0 muestras» NO se puede pintar como
avería**. Un cliente callado y un tap roto se ven exactamente igual desde dentro del programa, y
la única diferencia honesta que podemos mostrar es *«conectada, todavía sin sonido»* frente a
*«no se pudo abrir»* — que sí son distinguibles, porque el fallo aparece en la creación del tap,
no en la ausencia de muestras.

#### Pregunta 2 — ¿alcanza la transcripción local de macOS 26?

**Sí, y con mucho margen.** `SpeechAnalyzer` + `SpeechTranscriber` (macOS 26), sobre los dos audios
sintéticos que genera `say` con las frases de la maqueta:

| Pasada | Audio | Primer parcial | Total | Velocidad |
|---|---|---|---|---|
| es-ES, modelo recién instalado | 5,88 s | 247 ms | 370 ms | ×15,9 |
| en-US, modelo recién instalado | 5,39 s | 87 ms | 227 ms | ×23,7 |
| **es-ES, modelo ya instalado** | 5,88 s | **51 ms** | **84 ms** | **×69,9** |

Treinta locales soportados; `es-ES`, `es-MX`, `es-US`, `es-CL`, `en-US`, `en-GB` y el resto de la
familia inglesa quedaron instalados tras pedirlo. El presupuesto de la orden es **≤4 s de fin de
turno a ficha**: la transcripción de un turno de seis segundos cuesta 84 ms. El cuello de botella
de este sprint no va a ser el STT.

**Y el defecto que hay que anotar ahora, porque muerde en la fase 4.** Las dos transcripciones
escribieron mal el número:

```
es: «certificación ISO27.001»        en: «ISO 27,001 certification»
```

El motor formatea cifras según el idioma. La frase de la maqueta —la que dispara el estado «sin
resultado»— es literalmente *«certificación ISO 27001»*. Si el disparador de la fase 4 busca
`27001` en el corpus, no lo va a encontrar. **Se normalizan los separadores de miles antes de
buscar**; queda escrito aquí para que no se descubra como un bug misterioso dentro de dos fases.

#### Pregunta 3 — ¿se deja enlazar Swift dentro del binario de Rust?

`SpeechAnalyzer` es un `actor` de Swift con secuencias asíncronas: no hay forma de llamarlo por
mensajes de Objective-C como hicimos con la Accessibility API. O hay puente, o no hay motor.

```
swiftc -emit-library -static -O -module-name agstt -o libagstt.a
cargo run → SpeechTranscriber.isAvailable = 1
```

**Enlaza a la primera**, con las Command Line Tools y sin Xcode completo, añadiendo
`/usr/lib/swift` a las rutas de búsqueda. El riesgo nº 1 de la fase queda cerrado igual que quedó
el de la fase 0: probándolo, no razonándolo.

#### Lo que el spike decide, y lo que deja abierto

- **El audio del sistema se hace con Core Audio taps en Rust puro** (FFI declarada a mano, como
  `acople/ax.rs`), sin puente Swift: `AudioDeviceCreateIOProcID` acepta un puntero a función de C.
- **El STT se hace con SpeechAnalyzer a través de un puente Swift** compilado por `build.rs`.
- **Queda abierto** —y se decide en el ADR con la medición delante— si el VAD necesita Silero o si
  el detector determinista alcanza. Medirlo es de la fase 5 (el kit); construirlo, de esta.


### Fase 3b — las dos pistas, los turnos y la transcripción (2026-09-21)

La fase más grande del sprint, y la primera en la que la app **oye**. Cuatro piezas que no se
conocen entre sí —`capture` abre los grifos, `voz` corta los turnos, `stt` los convierte en texto,
`escucha` los junta— y una lección que se repitió tres veces: **lo que se descubre corriendo no se
descubre leyendo**.

#### Lo que se construyó

| Módulo | Qué hace | Por qué está separado |
|---|---|---|
| `capture/anillo.rs` | búfer circular de 30 s por pista, con índice global | es la promesa del efímero hecha forma: un tamaño que no crece |
| `capture/remuestreo.rs` | 48 kHz → 16 kHz **con filtro** | decimar sin filtrar convierte los agudos en voz que nadie dijo |
| `capture/nativo.rs` | los dos grifos de Core Audio | todo el `unsafe` de la captura, en un solo archivo |
| `voz/vad.rs` | detector de voz por energía con suelo adaptativo | código primero: Silero tendrá que ganarse el puesto con una medición |
| `voz/turno.rs` | fin de turno determinista (320 ms) | es de donde arrancan los 4 s de presupuesto del sprint |
| `voz/eco.rs` | el micrófono repitiendo al cliente | nació de la primera prueba de punta a punta |
| `stt/mod.rs` · `apple.rs` · `ventana.rs` | motor, puente y ventana de 12 turnos | el motor es sustituible **con una medición delante** |
| `nativo/Transcriptor.swift` | el puente a `SpeechAnalyzer` | `actor` de Swift: no hay selectores que mandar desde Rust |
| `escucha/mod.rs` | dos hilos: uno mira marcos, otro transcribe | transcribir no puede dejar ciega a la otra pista |

Los dos ADRs: **006** (el motor y su modelo) y **007** (las dos pistas y el eco).

#### Los cinco defectos que encontró un test antes que una persona

1. **El suelo de ruido se comía al hablante.** El detector actualizaba su estimación del silencio
   en todos los marcos, subiendo despacio. «Despacio» sigue siendo subir: a los **3,7 segundos**
   de habla continua el hablante quedaba por debajo de su propio umbral y la app se habría quedado
   muda justo con el cliente que más habla. La regla correcta es que **el suelo solo se mueve
   cuando NO hay voz**; y contra el atasco que eso abre —un ruido nuevo que empieza a mitad de una
   frase—, [`PACIENCIA_MS`]: a los treinta segundos afirmando «voz» sin una sola pausa, el
   detector desconfía de sí mismo y vuelve a medir la sala.
2. **Decimar sin filtrar inventaba voz.** Un tono de 18 kHz reaparecía a 2 kHz con RMS **0,707**,
   a todo volumen y en mitad de la banda de la voz humana. El detector lo habría oído como alguien
   hablando y el fin de turno se habría disparado sobre silencio.
3. **`vaciar()` disimulaba.** Mover el cursor deja las muestras íntegras en la memoria del
   proceso. El kill-switch de esta app se pulsa **delante del cliente**; si después su voz sigue
   ahí, la tecla es un adorno. Ahora se sobrescribe con ceros — y lo mismo con las letras del
   transcript antes de soltarlas.
4. **Quedarse con el primer canal dejaba sorda a la app** con unos auriculares desbalanceados.
5. **La pantalla no cabía en la pantalla.** El gate de fidelidad midió **+108 px** de desborde en
   Sesión: el botón «Iniciar sesión» se salía de la ventana. A ojo se veía perfecta.

#### Los tres hallazgos que solo aparecieron corriéndolo

**Uno · el silencio y la avería se ven igual.** Con nada sonando, el callback del tap **no se
llama ni una vez**. No llegan ceros: no llega nada.

```
=== control: SIN audio sonando ===   RESULTADO: llamadas=0 buffers=0 0 muestras en 2.0s
=== con audio ===                    RESULTADO: llamadas=268 buffers=1 137216 muestras en 3.0s · pico=0.75
```

Así que un contador en cero no distingue «el cliente está callado» de «el tap se rompió». Lo que
sí se puede afirmar es si el grifo **se abrió**, y es lo que la app enseña: la pantalla de Sesión
dice «Funciona» del grifo, no de las muestras.

**Dos · el micrófono oye a los altavoces.** La primera prueba de punta a punta —sonó
`pregunta-es.wav` por los altavoces del MacBook— devolvió el mismo turno por las dos pistas:

```
turno · Microfono · 880–2920 ms · «Tienen certificaciones o 27»
turno · Microfono · 3420–6240 ms · «Y la limpieza de datos, eso está dentro del alcance?»
turno · Sistema   · 740–6100 ms · «¿Tienen certificación ISO27.001 y la limpieza de datos eso está dentro del alcance.»
```

La app promete «micrófono = tú, sistema = el cliente». Con altavoces esa promesa **es falsa**: le
atribuye al consultor palabras que no dijo. La maqueta ya lo había previsto —la fila «Auriculares
conectados» existía desde la mirada 3— pero como un «todavía no» sin nada detrás. Ahora mide de
verdad (`bltn` + `ispk` = altavoces internos) y la app hace dos cosas: **avisa antes de la
reunión** y **marca el eco** con dos condiciones que tienen que cumplirse las dos —solapar en el
tiempo y decir casi lo mismo—, porque cada una sola confunde una interrupción o un resumen con un
reflejo. Los textos de arriba son, literalmente, los casos del test.

**Tres · preguntar estaba cambiando el sistema.** Al arrancar la app en vivo, el log dijo:

```
[stt] motor «apple-speechanalyzer» · 30 idiomas soportados · techo 5
[stt] modelos instalados: en-AU, es-ES
```

Dos de treinta, y elegidos por el orden en que la pantalla preguntó. macOS reparte los modelos de
reconocimiento **por reserva**, con techo de cinco por app — y enumerar los idiomas para pintar
una lista estaba gastando los cinco cupos del usuario, en silencio. Ahora el estado se lee de
`installedLocales` (qué hay en el Mac) y solo se reserva al instalar y al transcribir. Después del
arreglo, el mismo arranque:

```
[stt] modelos instalados: en-AU, en-CA, en-GB, en-IE, en-IN, en-NZ, en-SG, en-US, en-ZA, es-CL, es-ES, es-MX, es-US
```

#### La medición que decidió el motor (ADR 006)

| Pasada | Audio | Primer parcial | Total | Velocidad |
|---|---|---|---|---|
| es-ES, modelo recién instalado | 5,88 s | 247 ms | 370 ms | ×15,9 |
| en-US, modelo recién instalado | 5,39 s | 87 ms | 227 ms | ×23,7 |
| **es-ES, modelo ya instalado** | 5,88 s | **51 ms** | **84 ms** | **×69,9** |
| es-ES, desde Rust a través del puente | 5,88 s | — | 243 ms | ×24,2 |

El presupuesto del sprint son **cuatro segundos**. `whisper-rs` no llegó a medirse: la comparación
se detiene cuando una opción cabe treinta veces dentro del presupuesto y la otra pide 1,5 GB de
descarga para entrar en la carrera. Queda declarado como respaldo para macOS < 26, **sin
implementar**: deuda dicha, no olvido.

#### Gates nuevos, cada uno visto en rojo antes que en verde (regla 15)

| Gate | Qué vigila | Cómo se vio en rojo | Qué dijo al caer |
|---|---|---|---|
| `vaciar_sobrescribe_la_memoria_no_solo_el_cursor` | el kill-switch vacía de verdad | `vaciar()` solo mueve el cursor | «la muestra 0 sigue en memoria después de vaciar: el kill-switch no vacía, disimula» |
| `un_agudo_no_se_convierte_en_voz` | el remuestreo no inventa voz | decimación de una de cada tres | «un siseo de 18 kHz salió a 0.707 de RMS» |
| `una_frase_larga_no_se_convierte_en_silencio` | el suelo no se come al hablante | suelo actualizado en todos los marcos | «dejó de oír la voz en el segundo 3.6» |
| `el_fin_de_turno_cae_dentro_del_presupuesto_de_la_orden` | 160–400 ms | `FIN_MS = 800` | «el fin de turno tardó 800 ms; la orden pide entre 160 y 400» |
| `una_voz_que_solo_esta_en_un_canal_no_se_pierde` | la mezcla a mono | quedarse con el primer canal | «la voz del canal derecho se perdió al mezclar» |
| `un_trozo_que_ya_se_piso_se_declara_perdido` | un turno perdido se dice | sin la comprobación del borde | devolvía audio recortado como si fuera entero |
| `interrumpir_no_es_hacer_eco` | el eco pide las dos condiciones | solo la del solape | «una interrupción del consultor se tomó por eco (parecido 0.17)» |
| `repetir_despues_lo_que_dijo_el_cliente_no_es_eco` | ídem, por el otro lado | solo la del parecido | el resumen del consultor se borraba |
| `verify:ephemeral` extendido a Swift y a `voz` | el barrido sabe leer `.swift` | `Data(...).write(to:)` plantado en el puente | `✕ Transcriptor.swift:161 /\bwrite\(to:/` |
| `lo-que-macos-dira.test.ts` | el plist promete lo que la pantalla enseña | una palabra cambiada en el plist | «expected … to contain 'solo en memoria'» |

**Y dos gates que cobraron solos, sin que nadie los provocara:**

- el **barrido de vocabulario vetado** encontró «trampa» y «engañar» en mis propios comentarios de
  `remuestreo.rs`, `turno.rs` y `vad.rs`, usados como metáfora. La regla es absoluta y el barrido
  cubre `src-tauri/src`: se reescribieron las tres frases;
- el **gate de fidelidad** midió +108 px de desborde en Sesión y +50 en Idioma, y no dejó cerrar
  la fase hasta que las dos pantallas cupieron.

#### La pantalla que no cabía, y lo que se reordenó para que cupiera

Sesión estaba **al límite exacto** de los 640 px antes de esta fase (638 de 638). Todo lo que la
fase 3 tenía que añadir —dos pistas que ya funcionan, el aviso del eco, el botón de iniciar— la
sacaba de la ventana. Se reordenó midiendo, no a ojo:

- el **kill-switch salió de «Qué funciona hoy»** y bajó a la fila de la acción, al lado de la
  promesa que cumple: «corta todo · el sonido nunca se guarda · nada sale de tu equipo». La lista
  se quedó con lo que sí lleva la palabra «Funciona»;
- el **aviso del eco** cabe en una línea;
- en Idioma, los **tres «todavía no»** —varios idiomas, diccionario, conservar tus turnos— pasaron
  de tres tarjetas a una sola con tres filas. Ocupaban media pantalla y además se leían como tres
  ausencias distintas cuando son la misma: lo que llega después de este sprint.

> **Y una pregunta estructural que este sprint deja abierta, porque no es mía:** el cuaderno está
> **al borde de su techo**. Sesión cabe hoy con 0 px de margen y las pantallas crecen cada sprint
> —en el S2 llegan la lectura de pantalla y el radar; en el S3, las notas y la bandeja—. O la
> ventana crece (960 × 720), o se acepta que estas pantallas se desplacen. Las dos son decisiones
> de diseño y ninguna es urgente hoy.

#### Qué se vio correr en vivo, y qué no

`pnpm tauri dev`, con la app abierta de verdad:

```
[transcript] ⌘⇧T registrado · OJO: mientras Angel Ghost esté abierto, el navegador deja de reabrir la última pestaña cerrada con esa tecla
[stt] motor «apple-speechanalyzer» · 30 idiomas soportados · techo 5
[stt] modelos instalados: en-AU, en-CA, en-GB, … es-ES, es-MX, es-US
[audio] Altavoces · el micrófono va a oír al cliente: se marcará el eco
```

Y la cadena entera, de los altavoces al texto, en el test de integración `de-la-voz-a-la-frase`
(13 s, con `afplay` sonando de verdad). Las dos pistas abiertas, medidas: **19 265 muestras en
1,20 s** por el micrófono y **19 094 en 1,19 s** por el tap del sistema.

**Lo que NO se vio correr, y por qué:**

- **el botón «Iniciar sesión»** y el de Honestidad: hay que pulsarlos. Parada ⭐.
- **`⌘⇧T`**: registrada —el log lo dice— pero pulsarla es cosa de una tecla. Parada ⭐, y con una
  pregunta encima: **esa combinación es «reabrir la última pestaña» en Chrome, Safari y Firefox**,
  y el navegador es donde vive la reunión de Meet. Se registra porque es lo que el diseño aprobó,
  se avisa en el log, y cambiarla es decisión del usuario (riesgo nº 7 del plan).
- **el transcript de la banda con turnos reales**: necesita una sesión encendida a mano.
- **`NSAudioCaptureUsageDescription`**: la clave no aparece en las cabeceras públicas del SDK. Se
  declara porque una de más es inofensiva y una de menos mata la app al pedir el permiso, pero
  solo se comprueba de verdad con la app **empaquetada y firmada**. Parada ⭐.

#### El kill-switch pasó de 3 piezas a 6, y lo obligó el compilador

`Pieza::orden` es un `match` sin comodín. Al llegar el audio, `AudioDelMicrofono`,
`AudioDelSistema` y `Transcript` no se pudieron dejar como «todavía no existe»: el crate no
compilaba. La única que sigue declarada es `UltimoFrame` —la lectura de pantalla, C8, sprint 2— y
la pantalla de Honestidad lo dice: **«6 de 7 piezas: la otra todavía no existe»**.

#### Archivos de la fase 3

| Archivo | Qué |
|---|---|
| `src-tauri/src/capture/{anillo,remuestreo,nativo}.rs` | **nuevos** — los dos grifos y el audio en memoria |
| `src-tauri/src/voz/{mod,vad,turno,eco}.rs` | **nuevos** — cuándo alguien habla y cuándo terminó |
| `src-tauri/src/stt/{mod,apple,ventana}.rs` | **nuevos** — el motor, el puente y los 12 turnos |
| `src-tauri/src/escucha/mod.rs` | **nuevo** — los dos hilos que lo juntan todo |
| `src-tauri/nativo/Transcriptor.swift` · `build.rs` | **nuevos** — el puente de Swift y su compilación |
| `src-tauri/Info.plist` · `lproj/{es,en}.lproj/InfoPlist.strings` | **nuevos** — lo que macOS dirá al pedir un permiso |
| `src-tauri/src/{lib,corte,capture/mod}.rs` | siete comandos nuevos · `⌘⇧T` · el corte de seis piezas |
| `src-tauri/tests/contra-el-mac-de-verdad.rs` | **nuevo** — lo que ningún test unitario puede afirmar |
| `src/pantallas/Idioma.tsx` · `src/turnos.ts` | **nuevos** — la cuarta pantalla y el transcript |
| `src/{cuaderno.ts,App.tsx,componentes/{Banda,Principal,Ventana}.tsx,pantallas/{Sesion,Honestidad}.tsx}` | las pistas, el eco y los turnos reales |
| `docs/diseno/{idioma,sesion,honestidad}.html` | el estado `s1` nuevo y los dos puestos al día (mirada 13) |
| `docs/kit-de-prueba/audio/` | **nuevo** — dos frases sintéticas, 16 kHz mono |
| `scripts/{verify-ephemeral,capturar-fidelidad}.mjs` | Swift y `voz` bajo el barrido · el cuarto encuadre |
| `tests/unit/{lo-que-macos-dira,cuaderno,vocabulario-vetado}.test.*` | **uno nuevo** y dos puestos al día |
| `decisions/{006-stt-local-y-su-modelo,007-el-audio-en-dos-pistas}.md` | **nuevos** |

#### Criterio de fase completa

- `pnpm typecheck` ✓ · `pnpm lint` ✓ · **86/86** vitest · **130/130** `cargo test` (124 unitarios
  + 6 de integración contra el Mac de verdad) · `pnpm verify:ephemeral` ✓
- **fidelidad 56/56** bajo el umbral de 0,15 % · **cero desbordes** · cero errores de página
- las dos pistas capturan en vivo, el fin de turno cae en 320 ms y un turno de 5,9 s se transcribe
  en 243 ms — todo medido, nada supuesto
- **CI verde con conclusión propia por check** (`quality` · `e2e` · `build-escritorio`)

#### Y un defecto que no encontró ningún test, sino releer el diff

En el hilo que mira los marcos, la rama que detecta «el anillo dio la vuelta entera» estaba
escrita así:

```rust
if totales > p.origen + p.procesadas + Anillo::de_la_app().capacidad() as u64 {
```

`Anillo::de_la_app()` **construye un anillo nuevo** —480 000 flotantes, 1,9 MB— solo para
preguntarle su tamaño. Y esa rama se evalúa cada vez que no ha entrado audio, que es lo normal
cuando nadie habla: **1,9 MB reservados y tirados veinticinco veces por segundo**, en una app que
presume de caber en la memoria de un Mac en mitad de una videollamada.

Se arregló mirando mejor lo que ya había: `Anillo::rango` distingue `Some(vacío)` —«no ha entrado
nada», lo normal— de `None` —«ese audio ya se pisó»—, y esa diferencia es exactamente la pregunta
que la rama quería hacer. Sin constante nueva y sin reservar nada. Y ahora, cuando pasa, **se
dice**: `la pista «sistema» se quedó atrás 31.2s: ese audio ya se pisó y no se va a transcribir`.
Que la app se salte medio minuto de reunión sin que nadie se entere es el mismo silencio que el
resto de este sprint se ha dedicado a no permitir.

#### Y detrás de ese defecto había otro, y detrás del otro un test que no podía fallar

Arreglar lo de arriba dejó a la vista un segundo problema en la misma rama. Cuando la pista se
reengancha al presente, `origen` salta a donde va el anillo — pero **el reloj de los turnos seguía
contando desde el principio**. Y `indice()`, que traduce un instante del reloj a una muestra
concreta, suma los dos. Con uno movido y el otro no, cada turno posterior habría pedido un trozo
desplazado por todo lo que la pista llevaba vista: **se transcribiría un momento de la reunión
creyendo que es otro**. Perder audio es malo; inventar de quién es una frase es peor.

El arreglo es una línea (`p.turnos.reiniciar()` junto al salto del origen). Lo que costó fue el
test — porque el primero que escribí **no podía fallar**:

```rust
p.origen = 9_999;  p.procesadas = 0;  p.turnos.reiniciar();
assert_eq!(p.indice(p.turnos.reloj_ms()), p.origen + p.procesadas);
```

Movía las dos cosas **a mano** y luego comprobaba que cuadraban: comprobaba su propia aritmética,
no el código. Con el defecto puesto pasó en verde. Es la tercera pregunta de la regla 15 —*¿puede
este gate fallar siquiera?*— y la respuesta era no.

El bueno hace lo único que sirve: llama a `mirar()` con el anillo ya dado la vuelta y deja que
decida el código. Con el defecto puesto cae solo, y además dice por cuánto:

```
left: 552000   right: 520000
```

Treinta y dos mil muestras: exactamente los dos segundos que la pista llevaba vistos. De paso el
test corrigió el invariante que yo había escrito mal — no es `origen + procesadas`, porque el
reloj solo avanza con **marcos completos** y lo que sobra esperando al siguiente todavía no ha
llegado a él.

#### El gate de fidelidad dio dos respuestas distintas al mismo código

Terminando la fase, una corrida marcó **2,574 %** de divergencia en `sin-verificar-2 · light · es`
y la siguiente, sin tocar una línea, **0,069 %**. Eso no es un defecto de la pantalla: es un
defecto **del gate**. Un gate que contesta distinto al mismo código deja de creerse, y el día que
pare de verdad nadie va a mirarlo.

El arnés fotografiaba en cuanto aparecía el selector. Ahora, **en los dos lados por igual**,
espera tres cosas antes de disparar: que las tipografías estén cargadas (`document.fonts.ready`),
que no haya transiciones en marcha (se anulan por CSS) y que haya pasado un cuadro de pintado
entero. Tres corridas seguidas después del arreglo: `0.084 %`, `0.084 %`, `0.084 %`. Y de paso
desapareció una diferencia real que llevaba escondida entre el ruido —`sesion · light · en` bajó
de 0,105 % a cero—, que era una transición congelada a media ejecución.

#### Un test que fallaba por el reloj y no por el código

Dentro de un `cargo test` completo, el de punta a punta cayó una vez y pasó solo al repetirlo. La
causa no era el audio: eran los **seis segundos** que esperaba a que llegara el turno. En este Mac
en reposo sobran; recién compilando —o en una máquina de integración continua— no. Un test que se
rinde antes de tiempo falla por el reloj, y un fallo que no se puede reproducir enseña a ignorar
los rojos.

Ahora espera doce segundos y **se corta en cuanto llega el turno del cliente**, que es lo que
viene a ver. En el caso normal tarda menos que antes (9,7 s en vez de 15,8) y bajo carga aguanta
el doble. Tres `cargo test` completos seguidos, verdes.

#### Lo que costó la integración continua, y lo que se hizo con eso

La primera corrida verde de esta fase (`35673597848`) dejó a `build-escritorio` en **7 min 32 s**,
de 1 min 11 s que venía marcando. No fue el crate: fueron los **tres archivos de `tests/`**. Cada
archivo de ahí es un binario aparte y cada binario vuelve a enlazar el crate entero más la
librería de Swift — tres veces lo mismo, más de cinco minutos.

Los tres se juntaron en `contra-el-mac-de-verdad.rs`. Y juntarlos trajo su propio problema, que es
por qué el archivo tiene un turno: **en un solo binario los tests corren en paralelo y comparten
los altavoces del Mac**, así que el `afplay` de uno entraba en las mediciones de otro. Se
descubrió en el momento más justo: mientras escribía esto puse a sonar el audio a mano para
mirar la salida, y el test cayó con la mezcla transcrita. El mensaje del fallo lo dice ahora, para
que a nadie le cueste media hora — *«el tap oye TODO lo que suena en este Mac»*.

### Decisión de diseño no escrita — lo que la fase 3 descubrió y la maqueta no dice (mirada 13 propuesta)

La mirada 12 dejó una regla: *«a partir de aquí, toda pantalla que se entregue a medias usa este
estado: no se pinta en verde lo que no existe, y no se esconde»*. La fase 3 es la primera que la
cobra por el otro lado — **mover filas de «todavía no» a «funciona» también es diseño**, y mover
cuatro de golpe cambia lo que el usuario entiende al abrir la pantalla.

Y hay tres hechos que no salieron de ningún plan, sino de construir:

| Hecho | Cómo se supo | Por qué necesita sitio en la maqueta |
|---|---|---|
| **Con altavoces, el micrófono oye al cliente** | la primera prueba de punta a punta: la misma frase salió por las dos pistas | la app promete «micrófono = tú, sistema = el cliente». Con altavoces esa promesa es falsa y hay que decirlo **antes** de la reunión, no después |
| **macOS solo deja cinco idiomas listos a la vez** | `AssetInventory.maximumReservedLocales` | la pantalla ofrecía una lista; una lista sin techo deja que el sexto falle sin explicación |
| **El modelo de cada idioma lo descarga macOS** | `AssetInventory.status` decía «sin instalar» con el modelo puesto: faltaba reservarlo | es la **única** vez que un módulo protegido de esta app toca la red. Esconderlo sería exactamente lo que la pantalla de Honestidad existe para no hacer |

**Qué se propone, y por qué es una mirada y no un ajuste sobre la marcha.** Tres artefactos:

1. **`idioma.html` — estado `s1` nuevo.** La pantalla no tenía versión de sprint 1. Lleva lo que
   funciona (transcripción oculta por defecto, idioma por pista con el estado real de su modelo, el
   motor dentro del Mac) y lo que no (varios idiomas a la vez, diccionario técnico, conservar tus
   turnos), con el mismo trazo discontinuo de la mirada 12.
2. **`sesion.html` — estado `s1` al día.** Micrófono y audio del sistema pasan a «Funciona»; se
   añade «Escucha las dos pistas y las transcribe en tu Mac»; el botón «Iniciar sesión» deja de ser
   una promesa. Y la fila **«Auriculares conectados»**, que era «todavía no», ahora mide de verdad:
   con altavoces internos avisa, y explica qué hace la app mientras tanto.
3. **`honestidad.html` — estado `s1` al día.** Los anillos y el transcript dejan de estar
   pendientes y muestran cifras contadas; el kill-switch pasa de **3 de 7 a 6 de 7** piezas.

No se redecide nada aprobado: se aplica la forma de la mirada 12 al trozo que la fase 3 entrega.
Pero **el usuario no ha visto ninguna de las tres**, y «continúa» no aprueba diseño.

### Mirada 13 — aprobada (2026-09-21)

> **«Me gustó mucho el diseño y cómo se van evidenciando los elementos construidos y lo que falta,
> muy bien lograda»**

Lo que el usuario nombra —*«cómo se van evidenciando los elementos construidos y lo que falta»*— es
exactamente el mecanismo que la mirada 12 aprobó en abstracto y esta ve ya aplicado a un sprint
concreto: el par «funciona / todavía no» de `design-system.md` §9-sexies, con cuatro filas que
cambiaron de lado en esta fase. Queda aprobado, entonces, no solo el aspecto de las tres pantallas
sino **la manera en que envejece «todavía no»**: el estado `s1` de `idioma.html`, y los de
`sesion.html` y `honestidad.html` puestos al día.

**Cómo llegó el veredicto, que importa para la auditoría.** El usuario respondió primero
«Apruebo las tres pantallas idioma sesion y honestidad. continúa». Un «apruebo» no es un «lo vi»:
se repreguntó por la regla 10 —*«¿qué viste al abrirlas?»*— y la frase de arriba es la respuesta.
Es la tercera vez en esta app que la repregunta hace falta (miradas 1, 3-quinquies y 13) y la
segunda que, al hacerla, aparece contenido que la palabra de aprobación no traía.

Registrada también en `docs/diseno/README.md`. Con ella se desbloquea la fase 4.

## Fase 4 — Corpus, disparo y ficha

### Fase 4a — el motor, antes de tocar una pantalla (2026-09-21)

#### Lo que se construyó

| Módulo | Qué hace |
|---|---|
| `corpus/unidad.rs` | las cinco unidades del modelo de consultoría, por reglas léxicas bilingües. **El nombre del archivo pesa tres veces más que el cuerpo**: quien guarda «Propuesta Páramo Azul.pdf» ya clasificó el documento |
| `corpus/seccion.rs` | el troceado por sección; parte las largas conservando la fuente y pega las migajas sin cruzar un título |
| `corpus/leer.rs` | Markdown, `.docx` (zip + XML) y PDF, cada uno con su forma de reconocer títulos |
| `corpus/consulta.rs` | del turno hablado a la consulta: separadores de millares, palabras vacías de los dos idiomas, signos que rompen el analizador |
| `corpus/indice.rs` | BM25 sobre `tantivy`, dos campos por idioma, título con peso 3 |
| `corpus/mod.rs` | recorre la carpeta, orquesta, y reparte por unidad |
| `disparo/mod.rs` | los cinco motivos de la VISION: pregunta, cifra, término tuyo, silencio, atajo |
| `ficha/mod.rs` | titular ≤8 palabras · línea · fuente, **recortados del documento, jamás redactados** |
| `ficha/maniobra.rs` | el catálogo de seis de la mirada 11, ejecutado y no reescrito |

#### Las cuatro cosas que solo se supieron midiendo

1. **Mi propio spike se dio la razón a sí mismo.** Afirmaba que «rentable» encuentra
   «rentabilidad» y pasaba en verde — pero pasaba por la palabra «canal», que iba en la misma
   consulta. `rentabl` y `rentabil` son raíces distintas. Una aserción que no distingue entre dos
   causas no ha medido nada; el test que quedó afirma lo que el stemmer hace de verdad.
2. **`metodología` y `metodologia` no se encuentran entre sí.** El stemmer español usa la tilde
   para reconocer sufijos. Se midieron las dos cadenas de análisis sobre **23 parejas** de
   lenguaje de consultoría: **17/23 con plegado de acentos contra 15/23 sin él**. Va con plegado,
   y las dos pérdidas (`implementación~implementar`) quedan declaradas en el ADR 008 con el
   instrumento para revisarlas: el nDCG@5 del kit.
3. **Un `.docx` del propio macOS no escribe un solo `pStyle`.** Marca los títulos con negrita y
   cuerpo mayor. Con la regla semántica sola, un documento así se indexaba entero como una
   sección. El lector reconoce los dos caminos.
4. **Un PDF no trae títulos, trae líneas.** Sus secciones son **conjetura** por la forma del
   texto, y el documento lo declara hasta la ficha para que no prometa lo que nadie escribió.

#### Tres defectos que encontró un test antes que una reunión

- **«¿Tienen certificación?» no disparaba.** El mínimo de tres palabras se comía las preguntas
  cortas. Con signo de interrogación bastan dos, y el signo es la señal más fiable que hay.
- **Cuatro tests del corpus compartían carpeta** por PID y se pisaban al correr en paralelo — el
  mismo defecto que en la fase 3 obligó a serializar los tests de audio. Aquí se resolvió con un
  nombre por test en vez de con un candado.
- **Un test mío preguntaba «cuánto cuesta» a un documento que dice «tarifa cerrada»**: ni una
  palabra en común. Estaba mal el test, no el código.

#### Gates nuevos, cada uno visto en rojo en su propio commit (regla 15)

| Gate | Qué protege | Su rojo |
|---|---|---|
| `el_indice_nace_en_700_y_se_repara_si_lo_encuentra_abierto` | el índice guarda el corpus del usuario **en claro** y no puede nacer legible para las demás cuentas del Mac | desactivado: **493 (`0o755`) contra 448 (`0o700`)** |
| `el_catalogo_dice_lo_mismo_que_el_design_system` | el catálogo de maniobras vive en dos sitios y **tienen que decir lo mismo** | cambiada una palabra de la sexta maniobra: *«no está en design-system.md — el catálogo se separó de lo que el usuario aprobó»* |
| `disparo/` y `ficha/` en `verify:ephemeral` | el disparador guarda la última pregunta del **cliente** | el gate mordió al primer intento: la autorización de la lectura de test estaba una línea más arriba de donde tiene que ir |

#### Lo que el kill-switch no alcanzaba

El disparador guarda la última consulta para no repetir ficha, y eso son palabras del cliente en
memoria. Vivía dentro del hilo de transcripción, donde `cortar()` no llega. Ahora vive en la
`Escucha` tras un candado, y el corte **pisa las letras con ceros** antes de soltarlas — la misma
disciplina que la ventana de turnos, en su otro escondite. No añade una pieza al kill-switch: es
la pieza «Transcript», que sigue siendo **6 de 7**.

### Decisión de diseño no escrita — la pantalla de Corpus a medio construir (mirada 14 propuesta)

La fase 4 entrega el corpus indexado y la ficha. Eso toca dos superficies visuales, y **solo una
de ellas ya tiene veredicto**:

1. **La banda con ficha real.** Sus seis estados de contenido —incluido «sin resultado» con su
   maniobra— se aprobaron en la **mirada 11** sobre `docs/diseno/banda.html`. Aquí no se decide
   nada nuevo: se llena con datos de verdad la forma que ya está aprobada. **No pide mirada.**
2. **La pantalla de Corpus.** `docs/diseno/corpus.html` existe con sus cuatro estados, pero
   **no tiene estado «así se ve hoy · sprint 1»** — y este sprint entrega solo una parte: se
   indexa, se reparte por unidad, se dice qué quedó sin leer; **no** hay arrastrar y soltar, ni
   reindexado automático al cambiar un archivo, ni OCR de lo escaneado. Por la regla que la
   mirada 12 dejó escrita —*«toda pantalla que se entregue a medias usa este estado»*— hay que
   construir su `s1`, y eso **sí es diseño**.

Y hay tres hechos que solo se supieron construyendo y que la maqueta no dice:

- **la sección de un PDF es una conjetura**, y la ficha tiene que poder declararlo sin que parezca
  una avería;
- **el índice vive en la carpeta de datos de la app, en 700**, y la pantalla lo enseña: quien
  confía su carpeta tiene derecho a saber dónde acabó el derivado;
- **hay un techo de documentos por carpeta** (2 000) que es un aviso y no un límite técnico.

**Lo que se propone, entonces:** una sola mirada, la **14**, sobre `corpus.html` en su estado
`s1`, antes de escribir la primera línea de la pantalla de Corpus. La banda no entra porque su
forma ya tiene veredicto. Si el usuario prefiere que la banda con ficha real también se mire
—tiene todo el derecho: una cosa es la forma aprobada y otra verla con contenido de verdad— se
agrupa en la misma mirada y se dice aquí antes de construir.

### Fase 4b — la banda con fichas de verdad y la pantalla de Corpus (2026-09-21)

#### Lo que la banda dejó de inventarse

La banda pintaba «Páramo Azul» desde el diccionario. Ahora pinta lo que devolvió el corpus, y
fuera de Tauri sigue pintando la muestra — que es lo que sostiene el gate de FIDELIDAD.

**Dos defectos aparecieron al conectarla:**

1. **La banda tenía DOS fuentes de verdad para lo mismo.** El `estado` llegaba por URL (para el
   arnés de capturas) y la clase de la ficha decía otra cosa, así que pedirle «sin resultado» con
   una ficha cargada **no pintaba nada**. Dentro del producto manda la ficha; fuera sigue mandando
   la URL, que es donde vive el arnés.
2. **En `deMuestra` casteaba la etiqueta traducida a clave de unidad.** En español coincidían por
   casualidad —«propuesta» es la clave y la etiqueta— y en inglés dejaba la unidad vacía. Lo cazó
   el gate de fidelidad: **ocho encuadres en inglés y ninguno en español**, y esa asimetría era
   toda la pista que hacía falta.

#### La regla del diccionario mordió antes de escribir el código

La maqueta dibujó **una** de las seis maniobras. Las otras cinco no tenían texto en inglés, y la
app es bilingüe por regla dura. Se escribió primero en `banda.html` —la maqueta es el primer
diccionario— y Rust pasó a devolver **cuál** maniobra (`credencial`, `cifra`, …) en vez de su
texto: la maniobra es voz de la app, y la voz de la app vive en `src/i18n/`.

#### La pantalla de Corpus, y lo que costó que cupiera

El bloque `s1` desbordaba **74 px**. Recortar párrafos no sirvió de nada —ya cabían— y una
reestructura a ojo lo dejó **peor (82/101 px)** por romper el anidamiento. Lo que funcionó fue
medir: la rejilla de tres que la maqueta ya había aprobado para las unidades convierte seis filas
en dos, y las tarjetas de abajo en dos columnas quitan el resto. **0 px de desborde** en los
cuatro cruces de tema e idioma.

Y dos cosas que solo se vieron **leyendo la captura**, no en un test:

- el chip del pie decía «143 documentos» porque copié el del estado «con documentos»; el producto
  pinta ahí el estado de la sesión, como en todas sus pantallas;
- **«18,4 MB» con coma decimal también en inglés.** La maqueta ya distinguía («4,2 MB» / «4.2 MB»)
  en otro estado, y mi bloque traía el defecto — por eso los dos encuadres pasaban el umbral: el
  producto copiaba fielmente una maqueta equivocada. El tamaño del índice se formatea ahora con
  el separador del idioma.

#### El corpus contra archivos de verdad

Tres tests nuevos recorren el camino entero contra el disco: Markdown y **un PDF hecho con las
herramientas del propio macOS**. Comprueban lo que ninguna pieza ve sola — que el lector devuelva
texto, que el troceado encuentre secciones en lo que devolvió, que la unidad salga del nombre del
archivo y que la ficha cite una fuente que existe.

**Van en el MISMO binario** que los tests de audio. Un archivo más en `tests/` es un binario más,
y cada binario vuelve a enlazar el crate entero más la librería de Swift: fue lo que llevó la CI
de macOS de 1 min 11 s a 7 min 32 s en la fase 3. Los del corpus **no toman el turno** de los de
audio: no tocan hardware y cada uno estrena carpeta.

#### Qué se vio correr en vivo, y qué no

`pnpm tauri dev`: las tres ventanas, los **tres** atajos registrados —`⌥⎋`, `⌘⇧T` y el nuevo
`⌘⇧A`—, el motor de voz con sus 13 modelos y el aviso del eco. El plugin de diálogo carga sin
romper nada y su capability es la mínima: `dialog:allow-open`, sin `allow-save`, porque esta app
no escribe archivos por diálogo.

**Lo que NO se pudo comprobar aquí y es parada ⭐:** señalar una carpeta de verdad con el panel
de macOS e indexarla desde la ventana. Es un panel nativo; solo una persona puede pulsarlo.

#### Archivos de la fase 4

| Archivo | Qué es |
|---|---|
| `src-tauri/src/corpus/{unidad,seccion,leer,consulta,indice,mod}.rs` | **nuevos** — las cinco unidades, el troceado, los tres lectores, la consulta y BM25 |
| `src-tauri/src/disparo/mod.rs` | **nuevo** — los cinco motivos de la VISION |
| `src-tauri/src/ficha/{mod,maniobra}.rs` | **nuevos** — la ficha y el catálogo de seis |
| `src-tauri/src/escucha/mod.rs` | el `Buscador`, el «buscando», la latencia medida por aparición |
| `src-tauri/src/lib.rs` | `ElCorpus`, `⌘⇧A`, y los comandos de corpus y ficha |
| `src/ficha.ts` · `src/pantallas/Corpus.tsx` | **nuevos** |
| `docs/diseno/corpus.html` | estado `s1` **nuevo** (mirada 14) |
| `docs/diseno/banda.html` | el catálogo de maniobras, bilingüe |
| `decisions/008-el-corpus-el-disparo-y-la-ficha.md` | **nuevo** |

#### Criterio de fase completa

- `pnpm test` **87/87** · `tsc` · `eslint` · `verify:ephemeral` ✓
- `cargo test` **211** (202 de librería + 9 contra el Mac y el disco) · `clippy` 0 avisos
- gate de fidelidad **60/60** bajo el umbral del 0,15 %, cero desbordes, cero errores de página
- arrancada en vivo con los tres atajos registrados

## Desviación del plan (2026-09-20) — la MANIOBRA es producto nuevo

**Qué.** El estado «sin resultado» deja de limitarse a admitir el vacío: sugiere **cómo abordar la
situación** con (a) lo más cercano del propio corpus, declarado como insuficiente, y (b) una
maniobra de un catálogo versionado de seis, elegida por reglas léxicas.

**Por qué.** Petición del usuario en la mirada 11, con su razón: *«está bien que digas que no hay
nada pero sugiérele cómo abordar la situación»*. Un vacío honesto que no ofrece salida deja al
consultor solo en el peor momento.

**Qué NO es.** No es una funcionalidad de IA y no debe contarse como tal: cero tokens, cero red,
catálogo escrito por personas y versionado en el repo. Es **código primero** en su forma literal —
y cuando llegue la síntesis con modelo (sprint 2), este catálogo es su fallback permanente, como
exige la regla. No necesita ADR «código primero» porque no enciende ninguna feature LLM; lo que
necesitaría ADR es lo contrario.

**Qué debe absorber la planeadora.** El catálogo de maniobras es superficie de producto nueva,
hermana de **C6** (fichas de evidencia): la VISION debería recogerla al lado de C6, o como C17 si
prefiere numerarla aparte. Va sumada a **C15** (modo solo audio) y **C16** (puerta local para
Claude Code), que siguen pendientes de absorción desde la Etapa de Diseño.
