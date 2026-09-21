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

### Fase 1b — la banda construida
(en curso)

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
