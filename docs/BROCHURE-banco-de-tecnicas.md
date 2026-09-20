# Brochure vivo — banco de técnicas por gramática (vanilla) + esqueleto del storyboard

> **Compañero obligatorio del molde `BROCHURE.plantilla.html` v2** (kit v1.10.0). Nació del
> piloto de app-habla: el molde v1 nombraba las gramáticas G1/G2/G3 y entregaba un `fade-up`
> — **sin recetas, "G3" degenera en fundido** y el resultado se ve a plantilla. Aquí están
> las recetas del Estudio CINE (`hr03-estudio-cine`, RO) reescritas en **vanilla** y probadas
> en el piloto. Implementación de referencia con todo comentado en el sitio:
> `app-habla/docs/BROCHURE.html` (~120 KB autocontenido, Lighthouse 100/100/100/100).
>
> **La regla que gobierna cada elección** (del estudio): *"¿qué del MENSAJE hace necesaria
> esta técnica?"*. Si la respuesta es "se ve increíble", la técnica se corta.

## 0 · El proceso: producir, no estampar

1. **Leer** MANUAL-DE-USO + VISION (RO) + las guías de prueba de la app + `design-system.md`.
2. **Storyboard** (esqueleto abajo) → gate **«guion aprobado»** del usuario. Sin guion
   aprobado no se escribe HTML.
3. Construir sobre el molde v2 con las técnicas de este banco.
4. **Gates que la CI no ve** (en orden): pasada de capturas por bloque (cuadro a cuadro en
   las animaciones) → sala de proyección del usuario (rondas, no sí/no) → última milla (link
   de producción SIN sesión).

Si el usuario aporta referencias de piezas que le gustan (motionsites, onepagelove…), se
analizan con el protocolo del estudio: **se extraen TÉCNICAS, no stacks** — las referencias
usan React/GSAP/Framer/video remoto; aquí todo se reescribe en vanilla o se descarta.

## 1 · Esqueleto del storyboard (el contrato G-Guion)

Cabecera de la pieza:

- **Dial `MOTION_INTENSITY`** fijado CON el usuario antes de construir (el piloto: "cine
  sereno" — coreografía constante, curvas suaves, fiel a la personalidad de la app).
- **Identidad en una frase** (dirección de arte). Piloto: *"La página respira como la app
  escucha: todo lo que se mueve, se mueve como movido por una voz."*
- **El riesgo registrado**: UNA decisión valiente que el usuario juzga en la sala de
  proyección. Sin riesgo registrado, el builder elige lo seguro y lo seguro se ve a plantilla.
- **Excepciones declaradas** a solo-`transform`/`opacity` (cada una con su porqué).
- **Motion-system**: curvas y duraciones derivadas del design-system + escala de escena;
  vocabulario propio de la pieza (piloto: *enfocar · alzar · dibujar · bailar · volar · latir*).

Por **escena** (tabla de 8 campos):

| Campo | Qué exige |
|---|---|
| **Mensaje** | qué dice esta escena — una frase |
| **Gramática** | G1 serena / G2 / G3, y por qué esa |
| **Técnica** | la receta concreta (de este banco) |
| **Cómo el motion cuenta el mensaje** | la justificación NARRATIVA — si no la hay, la escena se corta |
| **Assets + origen** | qué SVG/recursos usa y de dónde salen (ideal: los de la app) |
| **Peso estimado** | KB — el storyboard declara el peso, el cierre publica el real |
| **Frame budget / propiedades** | qué propiedades anima (transform/opacity + excepciones declaradas) |
| **Reduced-motion** | el corte editorial de ESTA escena — composición estática con el mensaje completo |

Cierre del storyboard: **clímax explícito** (¿cuál es la promesa mayor de la app? — escena
propia, texto visible, JAMÁS un acordeón del pie) · ritmo (*toda escena clímax = ninguna lo
es*: después del clímax la página baja la voz) · **cuadre de mensajes n/n** contra el brochure.

El gate visual posterior AMPLÍA el contrato con una adenda — no lo reescribe.

## 2 · G3 — coreografía temporizada (la gramática del sosiego; dominante por defecto)

Entradas dirigidas por IntersectionObserver + tiempo. Sin secuestrar el scroll; perfecta en móvil.

- **Cascada blur-to-focus** (apertura; patrón yevtam): el titular se enfoca palabra por
  palabra — spans `aria-hidden` con el texto íntegro en `aria-label`; `filter: blur(3px→0)`
  SOLO en apertura (jamás en scroll) y con **opacidad SIEMPRE 1 si el elemento es candidato
  LCP** (regla `lcp-nace-estatico`). Curva `cubic-bezier(.16,1,.3,1)`.
- **Alzar y asentar con stagger jerárquico**: sube ~18–22px y decelera largo sin rebote
  (`cubic-bezier(0.22,1,0.36,1)` — el "aliento" de la pieza). El stagger JAMÁS es uniforme:
  título → apoyo → señal; la tarjeta estrella entra sola un beat antes, las demás a ~70 ms.
- **Dibujar el trazo** (técnica insignia del estudio): iconos SVG de trazo que se terminan
  de dibujar al entrar — `pathLength` + `stroke-dashoffset`, UN disparo, sobre SVG decorativo
  `aria-hidden` (excepción declarada). Dice "hecho a mano", el argumento anti-genérico.
- **`steps()` como identidad / boiling line** (patrón tbh): vida en DOS poses discretas
  (`steps(2)`, ~600 ms y se queda quieta) — una llama que tiembla, un personaje que respira.
  Vida sin loop estridente.
- **El contador que se cuenta solo**: el N del pie se cuenta 0→N en rAF (~700 ms, una vez al
  entrar en viewport) — `tabular-nums` + ancho fijo en `ch` (cero layout thrash); el texto
  accesible dice el número completo desde el primer byte (los e2e ni se enteran).
- **Confeti determinista**: ~14 partículas `transform` con variación **precalculada** (seed
  en comentario — determinismo del estudio, cero aleatoriedad en runtime); corre UNA vez y
  se va. Reutilizable en mini (6 partículas) para el remate del cierre.
- **Loop ambiental** (solo para el clímax — la única promesa que nunca descansa): pausado
  fuera de viewport (IO) y bajo `document.hidden`.

## 3 · G2 — máquina de estados + transición (las tarjetas y las aperturas)

- **Apertura de tarjeta**: `grid-template-rows: 0fr→1fr` (excepción declarada, un disparo por
  clic) + **`visibility` en la transición** — con un extremo en `visible`, CSS mantiene el
  contenido visible durante todo el colapso (la animación no se corta) y, cerrado, lo saca
  del árbol de accesibilidad. **Obligatorio**: sin esto el lector de pantalla recita todas
  las features "cerradas"; axe no lo ve; el e2e lo verifica contra el árbol real por CDP.
- **Coreografía interior**: al abrir, las features **alzan** en fila (stagger ~40 ms) — el
  detalle no "aparece": te lo van sirviendo en orden de lectura.
- **Apertura anunciada** (nació en el gate del piloto): destello sutil de marco + cabecera +
  telón al abrir — la acción se confirma a la vista.
- **Dirección del recorrido** (ídem): las tarjetas pueden abrirse solas al BAJAR (pasar
  páginas, con anclaje de scroll cuidado) pero el toque del usuario SIEMPRE manda, y al subir
  quedan cerradas — jamás re-abrir contra la dirección de lectura. Al quedarse sin tarjeta
  abierta, la referencia de anclaje pasa al siguiente elemento visible (el salto del piloto:
  1291px → 119px, medido).
- **Presión táctil**: `scale(0.99)` en el press del botón.

## 4 · G1 serena — scroll→progreso (SOLO con justificación narrativa en el storyboard)

El estudio no veta G1 en documentos; veta el G1 PESADO. La versión serena: **mapeo puro
scroll→progreso, sin pin, sin scrub-jack, sin tocar la rueda** — y el rig cuesta ~1 pantalla
de scroll en móvil, no 3–4.

- **El motor entero en ~60 líneas** (patrón Mostar): UN solo `rAF` que escribe **variables
  CSS** con inercia — `clamp`/`smoothstep`/`lerp`, suavizado `lerp(actual, meta, 0.14)`,
  paralaje de puntero ~0.12 — y el patrón "**seguir pidiendo cuadros solo mientras la
  inercia no llegó**" (el rAF se apaga al asentarse). Cero lectura de layout en el tick:
  medidas cacheadas en `resize`.
- **El personaje persistente** (el riesgo del piloto, aprobado): un elemento mínimo en el
  margen cuyo avance ES el scroll del lector — la tesis del producto vuelta navegación. Solo
  `transform: translateY/scaleY`. En reduced-motion no existe como animación: aparece
  completo y quieto.
- **Medir contra la cancha**: `container-type` + unidades `cqw` para que las maquetas escalen
  contra su marco, no contra el viewport.
- **Escalado por posición** (referencia moda): tarjetas que escalan sutilmente según su
  posición en pantalla, con `transform-origin` variable.

## 5 · Lo vetado (y por qué — precedentes del piloto)

- Pantalla de carga (hacer esperar a quien odia los manuales), auto-ciclado de secciones
  (quita el control), secuestro de scroll y rigs de 3.700px.
- Video/HLS remoto y webfonts de CDN (rompen el autocontenido); filtros SVG por frame
  (`feTurbulence`/`feDisplacementMap` re-renderizan en móvil — el "grano" se logra con
  transforms); cursor personalizado y `mix-blend-mode` ilegible.
- Emojis como iconos si el design system los prohíbe; gradientes que el DS no tiene.
- Aleatoriedad en runtime (variación precalculada con seed comentada).

- **Desvanecer TEXTO con opacidad como dramatización (kit v1.23.0):** durante el fundido el
  contraste real cae bajo AA (axe midió 1.32:1 en Innmobiliaria y un lector veía texto
  lavado). El motion narrativo MUEVE (`transform`); si el relato pide «aparecer», aparece
  movimiento, no legibilidad.
- **Revelado de bloques con umbral porcentual (kit v1.23.0):** `threshold: 0.1/0.15` deja
  invisible PARA SIEMPRE un bloque más alto que la pantalla (nunca alcanza el %). Revelado
  con `threshold: 0` + red determinista: lo que ya quedó por encima del borde inferior se
  muestra siempre. Verificado a 400/900/1800 px de paso de scroll.

## 6 · Verificación mínima de la pieza

- e2e: ruta 200 + tarjetas llegan cerradas + toque manda sobre el auto-abrir + enlaces del
  mapa existen + **lo cerrado fuera del árbol de accesibilidad (CDP)** + conteo del pie
  presente + axe en ambos temas con el detalle abierto.
- **e2e OBLIGATORIO de reduced-motion**: cargar con `reducedMotion: "reduce"` y afirmar
  visibilidad real (opacidad/tamaño) de los elementos clave — el hero invisible del piloto
  pasó 12 e2e y Lighthouse 100/100 sin este test.
- Capturas por bloque leídas como imagen (cuadro a cuadro en animaciones) ANTES del gate.
- Última milla: link de producción SIN sesión (curl/incógnito); dominio + protección de
  deployment al BLUEPRINT.

## 7 · Apertura por lectura — tarjetas desplegables (regla completa y trampas) *(kit v1.22.0)*

> **La regla, en una frase: una tarjeta está abierta exactamente mientras está a la vista.**
> Si las tarjetas desplegables concentran la mayor parte de la información, se abren al llegar
> a ellas, no al toque. **El toque es el control, no el peaje.** Nacida en Velo (5 rondas de
> gate visual); re-derivada a ciegas por nutri-kids en 5 rondas más antes de adoptarla entera.
> **Sustituye cualquier versión anterior del comportamiento de tarjetas. Se adopta ENTERA
> (método v1.23.0): no se re-deriva por rondas.**

- **Abre solo bajando**, cuando su cabecera cruza la línea de los dos tercios de la pantalla
  (`0 <= top <= vh·⅔`): queda un tercio de pantalla por debajo — el hueco donde se la ve
  crecer. El umbral es de PANTALLA, jamás de la tarjeta (⅓ de una tarjeta cerrada son ~30 px:
  abriría asomando por el borde inferior, fuera de la vista). El disparo es **continuo**
  (rectángulos por cuadro de scroll) — ni al reposo ni por tick: el reposo hace la apertura
  errática y el tick la vuelve aleatoria.
- **Ancla superior:** crece hacia abajo desde su cabecera; nada de lo ya leído se mueve.
- **Cierra al salir ENTERA de pantalla**, por el borde que sea. Por abajo: con su transición.
  Por arriba: de golpe, sin transición, reponiendo el scroll con el alto exacto perdido vía
  `scrollBy({top: -perdido, behavior: "instant"})` — un `scrollBy(x, y)` a secas SE ANIMA bajo
  `scroll-behavior: smooth` y la página pega saltos. `overflow-anchor: none` en `<html>` para
  que el navegador no compense también. **Subiendo no se abre nada, nunca.**
- **Rectángulos, no IntersectionObserver:** gobierna dónde está la cabecera respecto a la
  pantalla, no cuánto de la tarjeta se ve. Con coreografía de entrada (`translateY`), revelar
  con umbral 0: una caja desplazada 18 px miente sobre su posición.
- **El toque saca la tarjeta del automático** para el resto de la visita.
- **Corre también con `prefers-reduced-motion`:** desplegar es contenido, no decoración; el
  cinturón CSS ya lo abre sin transición.
- **La apertura dura ≥0.6 s** con un cue que no mueve nada (pulso de color en el borde): más
  corta se confunde con el propio scroll.
- **Muestras de interfaz (kit v1.23.0 — veredicto del usuario en el gate de Innmobiliaria):
  si la sección enseña LA APP, son capturas de la app CORRIENDO o no va** — producidas por la
  UI real (Playwright, datos sintéticos seeded, WebP como data: URI, script re-ejecutable;
  el archivo sigue autocontenido), jamás mockups SVG dibujados a mano («interfaces que nada
  que ver con la app real»). El SVG con tokens queda para ilustraciones ABSTRACTAS que no
  pretenden ser la app. Toda muestra es ilustración (`aria-hidden`); el `figcaption` carga el
  sentido.

**Verificación:** el test de la línea (95 %/80 % no abre, 60 % sí — posicionando en dos tiempos)
· «abre al cruzar la línea AÚN SIN DETENERTE» (bajada continua — el test que distingue los
modelos) · «la que quedó atrás bajando ya está recogida» (`bottom <= 0` ⇒ cerrada) · deriva
cero midiendo una cabecera visible, jamás `scrollY`, conduciendo con `behavior: "instant"`
(ningún e2e reproduce la inercia de un dedo: **se prueba la CAUSA, no el resultado**) · lo
cerrado fuera del árbol de accesibilidad por CDP · **cada test probado EN ROJO contra el
archivo del commit anterior** (`git show <sha>:ruta`) · y el gate visual humano, en rondas:
nada de esto lo caza la CI.

**Trampa T7 — la compensación del cierre se MIDE, jamás se resta a ciegas (kit v1.25.0,
hoja-de-vida S5):** el anclaje de scroll del navegador y el recorte del fondo del documento ya
compensan parte del alto perdido al cerrar una tarjeta por arriba; restar `perdido` encima
DUPLICA la corrección → brinco arriba → reapertura → bucle (260 pasos de rueda sin alcanzar el
fondo, cazado por el usuario en el gate M1). Arreglo: medir la **deriva real de un ancla
visible** (posición antes/después del cierre) y compensar solo esa diferencia. **Y el test de
deriva cero exige dos condiciones o produce rojos falsos sobre código correcto** — la forma más
cara de fallo, porque manda a arreglar lo que no está roto: (1) **el ancla tiene `top >= 0`**,
jamás un contenedor (su techo está por encima de lo que se recoge y «deriva» aunque lo visible
no se mueva — 207 px fantasma medidos); se rastrea la franja central hasta un bloque visible;
(2) **se mide con la página asentada**: esperar a que ninguna animación de duración FINITA siga
corriendo (las infinitas declaradas quedan fuera o no termina nunca) — medir a dos cuadros leyó
la crecida de un despliegue de 620 ms como 96 px de deriva. Qué NO afirmar: «ninguna tarjeta se
reabre» contradice el patrón (la empujada bajo la pantalla DEBE reabrirse al llegar el lector);
«se llega al fondo» no vigila nada (con el bug puesto, igual llegaba — verde con defecto).

**Referencias comentadas:** `app-anonimizador/docs/BROCHURE.html` (la original) ·
`app-nutri-kids/docs/BROCHURE.html` (bloque «M1») + sus `tests/e2e/{conoce,brochure}.spec.ts` ·
historia completa en los `ENTREGA-brochure-summary.md` de ambas apps ·
`app-hoja-de-vida/src/components/vitrina/apertura-por-lectura.tsx` (T7 + test de deriva con
ancla visible y espera de asentamiento — la adopción sobre documentos de 12 fichas).

## 8 · La capa infografía — la FICHA TÉCNICA sobre el detalle *(kit v1.26.0)*

El brochure (§0–§7) es la pieza de **persuasión** de una app; la **ficha técnica** es su capa de
**entrada**: menos detallada, tipo infografía, con un botón al final que baja al detalle. Nació
en hoja-de-vida (ADR-016) para que la vitrina reciba piezas de **cualquier frente** —apps,
agentes, investigaciones, tableros— con **un solo contrato y un solo renderizador**.

- **El contrato NO vive en este kit:** es `docs/contrato-ficha-tecnica/` de `app-hoja-de-vida`
  (v1.3.1 al escribir esto), **generado del Zod** por el mismo test que exige que coincidan
  (`pnpm contrato:ficha`): JSON Schema + ejemplo real + plantilla. Una sola fuente de verdad;
  siempre **aditivo** (cuando una ficha no cabe, crece el contrato — jamás se edita la ficha).
- **Anatomía (siete bloques, cualquiera sea el frente):** cabecera con titular de valor · tira
  de 3–5 cifras con `fuente` obligatoria · para quién y qué resuelve · cómo funciona (proceso
  BPMN: carriles, tareas de ≤60 caracteres, un inicio, ≥1 fin) · qué tiene (una tarjeta por
  bloque) · límites y lo que **nunca** hace · dónde está (versión anclada) · cierre.
- **Para las APPS de este portafolio la ficha se ARMA, no se escribe:** el `brochure-export.json`
  de la app (§ regla 13) + un **complemento curado** (titular · cifras destacadas · límites ·
  nunca · proceso) → ficha, en la planeadora (`vitrina/apps/`, `armar.py`, réplica fiel del
  `armar.ts` de la app). Por eso el export debe transportar lo que la ficha necesita: propuesta
  v1.1.0 del contrato brochure-export (`icono`, `proceso`, `titular`, `limites`, `nunca`).
- **Para los demás frentes** la ficha llega **completa como JSON** desde la casa que produce la
  pieza, con el contrato de la app, y entra por copia a `content/<frente>/` (PR de contenido
  sin sprint; la aduana valida en build). **Contenido, no HTML:** un HTML ajeno no pasa por axe
  ni por el barrido de cero enlaces.
- **Misma regla que el brochure:** cero enlaces, cero PII, ninguna cifra sin procedencia — y la
  ficha de una app se re-arma en cada `/cierre-sprint`, porque el sello no la congela.
