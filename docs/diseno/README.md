# Etapa de Diseño — registro vivo (Angel Ghost)

> Registro de la Etapa de Diseño (F2a): **cero código de producto**, solo la fundación visual.
> Lleva el **registro de miradas** —ningún artefacto se construyó encima sin feedback del usuario
> fechado; «continúa» no aprueba diseño—, la tabla pantalla → funcionalidad, la del spike, la
> auto-auditoría y el veredicto de G-Diseño.

## Cómo abrir la maqueta

`git checkout diseno/fundacion` → **doble clic en `docs/diseno/index.html`**. Sin build, sin
red, sin servidor. Desde ahí se entra a las nueve pantallas; dentro de cada una, la barra
superior conmuta **estado · tema · idioma** y la nota bajo la barra dice qué mirar.

## Plan de miradas (aprobado con el plan, 2026-09-20)

| Mirada | Artefacto(s) | Orden |
|---|---|---|
| 1 | `panel.html` (+ `assets/ghost.css` y `design-system.md` v0) | el panel primero, siempre |
| 2 | `kit.html` + `design-system.md` completo | |
| 3 | `sesion.html` · `permisos.html` · `honestidad.html` | |
| 4 | `corpus.html` · `notas.html` · `idioma.html` · `ia.html` | |
| 3-bis | `honestidad.html` (qué queda) + `panel.html` (modo solo audio) — **añadida en la mirada 3** | antes de la Fase 4 |
| 3-ter | `posicion.html` (dónde vive el panel) + radar invasivo — **añadida en la mirada 3-bis** | antes de la Fase 4 |
| 3-quater | `posicion.html` (banda acoplada D2/D3/F) + `permisos.html` (permiso de acople) — **añadida en la mirada 3-ter** | antes de la Fase 4 |
| 3-quinquies | `posicion.html` (relleno de captura: qué ve el cliente en la franja) — **añadida en la mirada 3-quater** | antes de la Fase 4 |
| 4-bis | `notas.html` (propuestas) · `idioma.html` (varios idiomas) · `ia.html` (Claude Code) — **añadida en la mirada 4** | antes de la Fase 5 |
| 4-ter | `notas.html` (bandeja con cuenta atrás) + `honestidad.html` — **añadida en la mirada 4-bis** | antes de la Fase 5 |
| 5 = G-Diseño | `index.html` + recorrido completo | |

Cualquier cambio al plan (agrupar, reordenar, posponer) se propone y se aprueba ANTES de
construir el siguiente artefacto.

### Extensión posterior a G-Diseño (sprint 001)

| Mirada | Artefacto | Por qué existe | Orden |
|---|---|---|---|
| 11 | `banda.html` — los seis estados de contenido de la banda | el gate de FIDELIDAD del sprint 001 compara la banda construida contra la maqueta, y la maqueta dibujó la banda **siempre con una ficha dentro**: cinco de los seis estados no tenían referencia. Se propuso en el plan del sprint, **aprobado por el usuario antes de construir** | antes de escribir la primera línea de UI del sprint 001 |

| 12 | `sesion.html` · `permisos.html` · `honestidad.html` — el estado **«así se ve hoy · sprint 1»** y el componente **«todavía no»** | la maqueta dibuja el producto terminado y cada sprint entrega un trozo; sin una forma escrita de decir «esto aún no existe», las pantallas de la fase 2 solo podían mentir en verde o esconder lo que falta. Se propuso en la bitácora ANTES de construir la UI del producto | antes de escribir la primera línea de las pantallas del cuaderno |

La maqueta no se congela con G-Diseño: se extiende por el mismo camino (propuesta → mirada →
registro). Lo que **no** cambia sin una mirada nueva es lo ya aprobado — y esta extensión no
redecide nada: cruza la forma elegida en la mirada 3-ter con el contenido aprobado en la 1.

## Registro de miradas

| Fecha | Artefacto | Veredicto del usuario (línea textual) | Qué se construyó encima, después |
|---|---|---|---|
| 2026-09-20 | `panel.html` (mirada 1) ⭐ | **Aprobado** — «El panel se ve muy bien aprobado» (con el archivo abierto en su Mac; antes había respondido «Continua» y se le repreguntó, regla 10) | Fase 2: `kit.html` + `design-system.md` completo |
| 2026-09-20 | `kit.html` + `design-system.md` v1.0.0 (mirada 2) | **Aprobado** — «Si apruebo el kit me gusto mucho muy oportuno el diseño y elementos» (con el archivo abierto en su Mac) | Fase 3: `sesion.html` · `permisos.html` · `honestidad.html` |
| 2026-09-20 | `sesion.html` · `permisos.html` · `honestidad.html` (mirada 3) | **Aprobadas con dos cambios** — «en honestidad está bien que lo del cliente se elimine no le veo problema pero lo que sí quiero es que me quede lo que es mío o lo que dije o escribí, adicional quisiera tener un modo solo audio que me hable de forma paralela por si quiero ver completamente la pantalla y no me interrumpa, todo el resto lo veo muy bien» | Fase 3-bis: los dos cambios, antes de la Fase 4 |
| 2026-09-20 | `honestidad.html` + `panel.html` (mirada 3-bis) | **Dos ajustes más** — modo audio como icono pequeño abajo izquierda o banda inferior («¿y si manejamos el panel en la parte inferior?»); y el radar debe detectar proctoring y anti-cheat, «que son invasivos» | Fase 3-ter: `posicion.html` + radar de dos niveles |
| 2026-09-20 | `posicion.html` + radar invasivo (mirada 3-ter) | **Decidido: D acoplada + F** — «me gusta mucho la D · banda pegada al borde aunque un poco más arriba al menos el doble, con la posibilidad de ampliarla y ojalá […] recorte la pantalla de la reunión como si fueran dos aplicaciones pegadas […]. En cuanto al E y F ambas me gustan mucho pero vamos con F. Muy bien lo de anti…, estamos solo protegiéndonos de software que quiera invadir nuestra independencia» | Fase 3-quater: banda acoplada 88/200/44 + permiso de acople |
| 2026-09-20 | `posicion.html` (banda acoplada) + `permisos.html` (acople) (mirada 3-quater) | **Aprobada con un cambio** — «me pareció genial D2, D3 y F. ¿Qué te parece a ti? ¿Por qué es más seguro que el flotante? Podemos hacer que no se vea el escritorio sino en negro, no quiero que vea que algo ocupa ese espacio» | Fase 3-quinquies: **relleno de captura** (la franja deja de mostrar el escritorio) |
| 2026-09-20 | `posicion.html` (relleno de captura) (mirada 3-quinquies) | **Decidido: fondo de escritorio** — «sí vamos con el relleno fondo de escritorio». *(Elección entre las tres opciones presentadas. Se repreguntó si había abierto el archivo, regla 10; respondió «Listo continúa» — respuesta afirmativa, pero sin describir lo que vio. Se registra tal cual y **el relleno vuelve a la mirada 5**, dentro del recorrido completo.)* | Fase 4 |
| 2026-09-20 | `corpus.html` · `notas.html` · `idioma.html` · `ia.html` (mirada 4) | **Corpus aprobado; tres cambios** — «Notas: me gusta pero es que soy malo tomando notas, no sé cómo voy a lograr tomar notas; no sé si fuera posible que me propusiera si x información deba guardarse como notas. Idioma está bien pero siento que también debería la opción bilingüe para seleccionar más de un idioma. IA me gusta, está bien, aunque quiero que Claude Code también me pueda ayudar a operarla ya que estamos en local» | Fase 4-bis: notas propuestas · varios idiomas · puerta local para Claude Code |
| 2026-09-20 | `notas.html` · `idioma.html` · `ia.html` (mirada 4-bis) | **Idioma e IA aprobados; una cosa en notas** — «me gusta mucho lo de las notas, me preocupa es que mientras estoy en la reunión no puedo decidir; ¿es posible decidir apenas finalice la reunión y darme una o unas horas antes de borrar las sugerencias? De resto sí que me gusta mucho notas. Idioma muy completo, incluso mejor de lo que pensaba. IA también quedó excelente, aprobado» | Fase 4-ter: bandeja de propuestas con cuenta atrás |
| 2026-09-20 | `notas.html` (bandeja) + `honestidad.html` (bandeja en las cuentas) (mirada 4-ter) | **Aprobada** — «abrí la ventana, claramente la exploré con sus botones y me pareció excelente, continuamos». *(Se le repreguntó por la regla 10 al llegar solo la palabra de fase; respondió con la exploración descrita. Observación suya, justa y registrada: no tiene que rendir cuentas de lo que ve.)* | Fase 5: recorrido, README, auditoría y G-Diseño |
| 2026-09-20 | `banda.html` — los seis estados de contenido de la banda (mirada 11, sprint 001) | **Aprobada con un cambio** — «Si me gusta mucho muy bien docs/diseno/banda.html, pero en Sin resultado esta bien que digas que no hay nada pero sugierele como abordar la situacion. El resto esta muy muy bien» | La **maniobra**: lo más cercano del corpus + un catálogo versionado de seis maneras de responder, los dos deterministas (cero LLM). Luego, la fase 1b: las tres ventanas |

## Decisiones de diseño declaradas antes del segundo artefacto

D1–D13 en `sprints/ETAPA-DISENO-implementation-log.md` (tamaño 380 × 220, esquina superior
derecha, opaco, Charter · Avenir Next · Menlo, iconografía SVG sin emojis, motion casi nulo,
bilingüe pareado, datos sintéticos «Páramo Azul», estructura de archivos).

## Pantalla → funcionalidad de la VISION

Contrato de la orden: las **17 funcionalidades `[MVP · personal]`** de la VISION v1.2.0 más las
**2 `[MVP · terceros]`**, ninguna sin pantalla y ninguna pantalla sin funcionalidad. A esas se
suman **tres nacidas en las miradas** (C14 ya estaba en la orden; **C15 y C16 son nuevas** y la
planeadora debe absorberlas: ver `## Desviación del plan` en la bitácora).

| # | Pantalla | Estados | Funcionalidades |
|---|---|---|---|
| 01 | `panel.html` | esperando · buscando · ficha · sugerencia local · sugerencia API · sin resultado · sin verificar · radar · radar invasivo · transcript · voz · voz sin auriculares | **C1** ventana protegida · **C5** disparo y atajo · **C6** fichas de evidencia · **C7** sugerencia · **C8** lectura de pantalla · **C14** radar · **C15** modo solo audio · **B2** contador de red |
| 01-b | `posicion.html` | A · B · C (descartada) · D · D2 · D3 · F · E | **C1** — decide forma y posición: banda acoplada 88/200, solo audio 44, y el relleno de la franja |
| 01-c | `banda.html` *(sprint 001)* | esperando · buscando · ficha · ficha ampliada · sin resultado · sin verificar · sin verificar ampliada · transcript · sin acople | **C1** forma × contenido · **C3** transcript por pista · **C6** ficha y acumuladas — referencia del gate de FIDELIDAD |
| 02 | `sesion.html` | detectada · sin reunión · NDA (solo notas) · vigilancia local | **C2** dos pistas · **C11** jurisdicción y NDA · **C12** detección de cliente · **C14** radar local |
| 03 | `permisos.html` | sin conceder · concedido · revocado a mitad · solicitando · acople | **C8** consentimiento de pantalla · **C12** micrófono, audio del sistema y acople |
| 04 | `corpus.html` | vacío · indexando · con documentos · ilegible | **C4** ingesta e índice · **B1** cinco unidades |
| 05 | `notas.html` | durante · bandeja · propuestas · al cerrar · archivo cifrado | **C9** anotar, fijar, acuerdos, cierre y retención |
| 06 | `honestidad.html` | sesión activa · tras kill-switch · verificación · al cerrar | **C10** efímero verificable · **C12** kill-switch · **B2** contador |
| 07 | `idioma.html` | oculto · visible · varios idiomas · diccionario | **C3** transcripción · **B3** diccionario técnico es/en |
| 08 | `ia.html` | local · API apagado · API encendido · kit de evaluación · Claude Code | **C7** proveedor y costo · **C13** kit de evaluación · **C16** puerta local · **B2** contador |
| 09 | `kit.html` | 11 secciones de componentes | la fuente de verdad visual: `design-system.md` v1.7.0 |

**Cobertura:** 19/19 (17 MVP personal + 2 MVP terceros) + C14 + **C15 y C16, nuevas**. Ninguna
funcionalidad quedó sin pantalla; ninguna pantalla existe sin funcionalidad que la justifique.

## Qué promete el panel, por cliente — spike del usuario (2026-09-20)

Copiada del spike (`investigacion/2026-09-20-spike-invisibilidad.md` de la planeadora). La
promesa de invisibilidad es **graduada**: solo se afirma lo verificado, con fecha y versión.

| Cliente | Resultado | Fuente |
|---|---|---|
| **Google Meet (Chrome)** | **INVISIBLE** — la ventana con `sharingType = .none` no aparece en la pantalla compartida; la de control sí | usuario, 2026-09-20, macOS 26.6.2: «perfecto, la roja no se visualiza» |
| Zoom (app) | **NO PROBADO** | decisión del usuario |
| Microsoft Teams (app) | **NO PROBADO** | ídem |

La maqueta lo respeta literalmente: `panel.html` tiene el estado **«sin verificar en este
cliente»** con el camino alterno (compartir ventana · segundo monitor · modo solo notas), y en
ningún sitio dice que funcione donde no se probó.

**Paradas del gate de prueba en llamada real** (nacidas en esta etapa, no verificables aquí):

1. Zoom y Teams — la promesa graduada del panel.
2. Que la captura componga el **relleno** de la franja y no la banda (mirada 3-quinquies).
3. Que el **acople** devuelva la ventana de la reunión a su sitio al cerrar (mirada 3-quater).

## Auto-auditoría (checklist §4 del skill `diseno-ui`, por tema)

| Criterio | Oscuro | Claro | Evidencia |
|---|---|---|---|
| Contraste AA en todo texto | ✓ | ✓ | medido nodo a nodo en cada estado × tema × idioma; **0 bajo AA** |
| Estado = símbolo + texto + color | ✓ | ✓ | tabla §4 del design system; pasada **deutan** sobre corpus, IA y panel |
| Jerarquía de un golpe de vista | ✓ | ✓ | el panel se lee en titular · línea · fuente; nada compite |
| Sin color como único portador | ✓ | ✓ | anti-patrón con gate: `rojo vs. verde` jamás solo |
| Motion mínimo y reduced-motion | ✓ | ✓ | §3.5; la forma del árbol nunca depende del hook |
| Tipografía con roles, no genérica | ✓ | ✓ | Charter evidencia · Avenir Next interfaz · Menlo medible |
| Iconografía propia, cero emojis | ✓ | ✓ | sprite de 38 glifos + **gate en `pnpm test`** (nació en la Fase 5) |
| Todo cabe en su ventana | ✓ | ✓ | 960 × 640 y 380 × 220 sin scroll: **0 desbordes** medidos |
| Bilingüe real, no pendiente | ✓ | ✓ | textos pareados `lang` en toda pantalla de producto |
| Datos 100 % sintéticos | ✓ | ✓ | «Páramo Azul», «Sur del Valle»; cero nombres reales |

**Deuda declarada:** el chrome de la sala de diseño (barra, notas «qué mirar», `index.html`,
`posicion.html`) está **solo en español**. Es andamiaje, no producto, y no viaja al código; se
declara para que nadie lo lea como una traducción pendiente.

## Gates que nacieron en esta etapa (con su demo en rojo)

Regla 15 del pipeline, las tres preguntas: *¿lo viste fallar? · ¿lo viste correr? · ¿puede
fallar?* Los tres corren en `pnpm test`, es decir en el job `quality`.

| Gate | Qué impide | Demo en rojo |
|---|---|---|
| `vocabulario-vetado` | los términos de ocultamiento vetados por la regla dura 6 —«indetectable», «stealth», «trampa» y equivalentes— en maqueta, README y design system <!-- vocabulario:cita --> | plantado «indetectable» en `panel.html` (Fase 0) y en `ia.html` (Fase 4) ⇒ rojo nombrando el archivo. Excepción nominal declarada para `anti-cheat`/`anti-trampa`: el término suelto sigue en rojo (demostrado) |
| `maqueta-autocontenida` | `http(s)://`, `<script src>`/`<link href>` externos, `@import url(` y dominios de despliegue en `docs/diseno/**` | plantado un CDN ⇒ rojo (Fase 0) |
| `maqueta-sin-emojis` | emojis como iconografía en el markup y los estilos de la maqueta | plantado 🔒 ⇒ rojo; y `✅` ⇒ rojo, demostrando que la excepción nominal de `✓` **no abre su bloque** (Fase 5) |

## Registro de G-Diseño

**APROBADO — 2026-09-20.** Veredicto del usuario sobre la maqueta completa abierta en su Mac:

> «sí apruebo la pantalla completa»

**Qué queda aprobado:** `design-system.md` v1.7.0 como **fuente de verdad visual** y la maqueta de
`docs/diseno/` (9 pantallas, 51 estados, dos temas, dos idiomas) como **contrato de forma**. Desde
aquí, toda pantalla de producto obedece a esta maqueta: el primer sprint con UI se detiene tras la
primera pantalla construida y presenta capturas comparadas contra ella (**gate de FIDELIDAD**,
indiferible — no viaja con el gate ⭐).

**Cómo se llegó:** **10 miradas** del usuario (plan de 5, ampliado cinco veces a petición suya,
cada ampliación propuesta y aprobada antes de construir). Ninguna pantalla se construyó encima de
un artefacto que él no hubiera abierto. Dos veces llegó solo la palabra de fase y se repreguntó;
la segunda vez él observó, con razón, que no tiene que rendir cuentas de lo que ve — la
repregunta se hizo porque esa pantalla aflojaba una regla dura, y eso debió decirse junto a la
pregunta, no después.

**Estado de cierre:** PR [#3](https://github.com/mauriciorincon-ai/app-copiloto-consultor/pull/3)
con conclusión propia `success` en `quality`, `e2e` y `build-escritorio`; barrido de CERO ENLACES
limpio; 9 gates verdes, los 3 nuevos vistos en rojo.

**Lo que esta etapa NO pudo verificar** (paradas del gate de prueba del primer sprint): Zoom y
Teams · que la captura componga el relleno de la franja · que el acople devuelva la ventana de la
reunión a su sitio al cerrar.
