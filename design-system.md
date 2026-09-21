---
app: copiloto-consultor
nombre: Angel Ghost
version: 1.10.0  # 1.10.0: «TODAVÍA NO» — el estado de lo que aún no está construido (sprint 001, fase 2). 1.9.0: la MANIOBRA cuando el corpus no tiene nada (determinista, sin LLM). 1.8.0 los seis estados de CONTENIDO de la banda (sprint 001). 1.7.0 bandeja con cuenta atrás. 1.6.0 propuesta, idiomas, puerta local. 1.5.0 pantallas del cuaderno. 1.4.0 relleno de captura. 1.3.0 banda ACOPLADA. 1.2.0 radar 2 niveles. 1.1.0 voz. 1.0.0 completo.
fecha: 2026-09-21
estado: aprobado   # G-Diseño aprobado el 2026-09-20 («sí apruebo la pantalla completa»)
fuente_en_codigo: docs/diseno/assets/ghost.css   # el sistema en CSS; el kit en docs/diseno/kit.html
---

# Angel Ghost — design system

> Fuente de verdad visual de la app. Toda pantalla de producto la obedece; se extiende por ADR,
> nunca se contradice en silencio. El sistema vive en código en `docs/diseno/assets/ghost.css`
> y se exhibe entero en `docs/diseno/kit.html` (ambos temas, ambos idiomas). Lo que no está en
> el kit, no existe.

## 1 · Personalidad (D7)

**discreto · propio · verificable** — nunca *ruidoso*, nunca *«stealth»* (vocabulario:cita),
nunca *corporativo*.

Un **cuaderno privado** al lado de la videollamada. Se lee en ≤ 2 s y no compite con la
reunión; lo que promete («nada queda», «nada sale») se **ve**: el contador de red y el panel
de honestidad son componentes canon, no páginas de ajustes. La categoría vende ocultarse;
esta app vende **no persistir, verificable** — y su estética lo dice: nada brilla, nada se
mueve, todo tiene fuente y fecha.

## 2 · La tesis del sistema

**Tres voces tipográficas con roles** (D4), nativas de macOS — cero bytes de fuentes en el
binario (la app es macOS-only):

| Voz | Familia → fallbacks | Qué dice |
|---|---|---|
| **Evidencia** | Charter → Iowan Old Style → Georgia | lo que dice **tu** corpus: titular y línea de la ficha, la frase del vacío, lo que se oyó |
| **Interfaz** | Avenir Next → Helvetica Neue → system-ui | estados, botones, avisos, sugerencia, títulos de pantalla |
| **Medible** | Menlo → SF Mono → ui-monospace (`tabular-nums`) | contador de red, fuente de la ficha, tiempos, atajos, cifras, fuentes legales |

**Un solo acento** (`--halo`, azul hielo) gastado con avaricia: foco, ficha fijada, sugerencia,
interactivo. Los colores de estado no son decoración: son **semántica con símbolo y texto**.

**Dos superficies** y nada más: la ventana (`--bg`) y lo que flota sobre ella (`--surface`);
los chips y teclas usan `--surface-2`. Sin gradientes, sin blur, una sombra para el panel.

## 3 · Tokens (CSS variables en `ghost.css`; jamás valores mágicos en componentes)

### 3.1 Color — tema OSCURO (primario)

| Token | Hex | Rol |
|---|---|---|
| `--bg` | `#1a1917` | ventana (grafito cálido, no negro) |
| `--surface` | `#242320` | ficha, tarjetas, rail |
| `--surface-2` | `#2e2d29` | chips, teclas, hover, campos |
| `--line` / `--line-2` | `#3b3a35` / `#4a4841` | bordes / bordes de campo y en foco |
| `--ink` | `#ece7dc` | texto principal (marfil) |
| `--ink-2` | `#b9b3a6` | texto secundario |
| `--ink-3` | `#7c776c` | **SOLO ORNAMENTO** (separadores). **Vetado como texto** (§7.2) |
| `--halo` / `--halo-tint` | `#9ecbff` / 14 % | acento único |
| `--ok` / `--ok-tint` | `#62d3b4` / 15 % | activo · verificado · concedido |
| `--warn` / `--warn-tint` | `#f0b64a` / 15 % | atención · sin verificar · radar · solo notas |
| `--err` / `--err-tint` | `#ff8577` / 15 % | error · revocado · cortado (kill-switch) |
| `--mute` | `#9a958a` | inactivo (texto, AA) |

### 3.2 Color — tema CLARO (papel)

| Token | Hex |
|---|---|
| `--bg` / `--surface` / `--surface-2` | `#f5f2ea` / `#fffdf8` / `#ece8de` |
| `--line` / `--line-2` | `#d6d1c4` / `#bdb7a8` |
| `--ink` / `--ink-2` / `--ink-3` (ornamento) | `#1f1e1a` / `#5a5650` / `#a19b8e` |
| `--halo` | `#1d5c9c` |
| `--ok` / `--warn` / `--err` / `--mute` | `#0f6350` / `#7a4f00` / `#a32a22` / `#6d685f` |

Los tintes de estado son `rgba(color, .12–.16)` — el «15 %» de la regla *positivo = tinte +
borde sólido + ✓ en círculo relleno*. En claro, `--ok`, `--warn` y `--err` se oscurecieron
tras medir (4.45–4.49 → ≥ 5.40 sobre su tinte).

### 3.3 Tipografía

| Token | Tamaño / peso | Uso |
|---|---|---|
| `--t-titular` | 17 px · Charter 700 · 1.25 | titular de ficha, ≤ 8 palabras, **máx. 2 renglones** (`line-clamp`) |
| `--t-linea` | 14 px · Charter · 1.35 | línea de ficha, **máx. 2 renglones**; cita de lo que se oyó |
| `--t-ui` | 13 px · Avenir Next | interfaz, sugerencia, párrafos |
| `--t-meta` | 12 px · Avenir Next | estados, etiquetas, franjas |
| `--t-mono` | 11.5 px · Menlo | contador, fuente de ficha, atajos, tiempos (pie del panel: 11 px) |
| `--t-h1` / `--t-h2` | 22 px 600 / 15 px 600 · Avenir Next | título de pantalla / de tarjeta |
| `.seccion-t` | 11 px Menlo, mayúsculas, tracking .1em | rótulos de sección (kit, pantallas) |
| chip de unidad | 10.5 px 600, mayúsculas, tracking .04em | `PROPUESTA · MARCO · CASO · CLIENTE · PERFIL` |

### 3.4 Espacio, radios, trazos, sombras

| Grupo | Valores |
|---|---|
| Espacio | múltiplos de 4: `--s-1` 4 · `--s-2` 8 · `--s-3` 12 · `--s-4` 16 · `--s-5` 20 · `--s-6` 24 · `--s-8` 32 · `--s-10` 40 |
| Radios | `--r-chip` 5 · `--r-ficha` 8 · `--r-panel` 10 · `--r-tecla` 4 |
| Trazo | 1 px `--line`; los estados llevan **borde sólido del color del estado** (tinte + borde, no solo tinte) |
| Sombras | panel: `0 10px 30px rgba(0,0,0,.45)` (claro: `rgba(40,30,10,.18)`); ficha: 1 px de asiento. Nada más |
| Foco | `outline: 2px solid var(--halo); outline-offset: 2px` en todo elemento interactivo |

### 3.5 Motion (D10)

`--dur: 150ms` · `--ease: cubic-bezier(.2,.6,.2,1)`. **Solo tres cosas se mueven:** la ficha
nueva aparece con fade (sin desplazamiento), el conmutador desliza su knob, y el panel cambia
de altura al mostrar el transcript. Nada pulsa, nada gira: «buscando…» es texto estático; el
progreso es una barra quieta cuyo número es el texto. `@media (prefers-reduced-motion:
reduce)` ⇒ `transition: none; animation: none` global. **La forma del árbol jamás depende del
motion** (regla 5a del CLAUDE.md): reduced-motion cambia propiedades, no elementos.

### 3.6 Ventanas (D1, D2, D3, D6)

| Ventana | Tamaño | Notas |
|---|---|---|
| **Panel flotante** | **380 × 220**; con transcript **380 × 420**; con 3 fichas hasta 380 × 360; **nunca más ancho** | opaco, sin blur; **esquina superior derecha**, 16 px del borde, 8 px bajo la barra de menús; todos los Spaces; posición recordada (preferencia) |
| **Píldora de voz** (C15, modo solo audio) | **260 × 56** | **reemplaza al panel**, no convive con él: la pantalla queda libre. Misma familia visual (opaca, radio 10, borde 1 px). No muestra la ficha —se oye—: solo su origen y cómo callarla |
| **Banda inferior — forma PRINCIPAL en reunión** | ancho completo × **88** (compacta) · **200** (ampliada) · **44** (modo solo audio) | pegada al borde inferior, con **asa** que ajusta su alto. **Acoplada por defecto**: la ventana de la reunión se recorta y las dos conviven como aplicaciones pegadas; el asa mueve las dos a la vez. Sin el permiso de acople, flota encima |
| **Gota** (modo solo audio, mínima) | **44 × 44** | solo el estado, sin texto. Esquina inferior izquierda: la única zona libre en Meet, Zoom y Teams |
| **Relleno de captura** | **idéntico a la banda**, siempre | ventana SIN contenido que se dibuja justo **debajo** de la banda y viaja pegada a ella. La banda es opaca: el usuario no la ve nunca. Es lo único que una captura de pantalla completa encuentra donde vive la banda. **Jamás dibuja contenido de ninguna app** — solo un relleno plano |

**Posición y forma (D2, decidida en la mirada 3-ter sobre `docs/diseno/posicion.html`):**

| Situación | Forma por defecto |
|---|---|
| En reunión, con ficha | **banda inferior acoplada, 88 px** — ampliable a 200 con el asa |
| En reunión, modo solo audio | **banda inferior acoplada, 44 px** (una línea) |
| Sin el permiso de acople | la misma banda, **flotando** sobre la reunión |

**Relleno de captura (decidido en la mirada 3-quater, 2026-09-20).** La banda lleva el flag de protección de captura, así que una grabación o un «compartir pantalla completa» renderiza la escena **sin** la banda — y muestra lo que quede detrás. Eso no es aceptable por dos razones, y la segunda pesa más que la primera: (1) delata que algo ocupa ese rectángulo; (2) **filtra ventanas ajenas a la reunión** que estén debajo. Por eso la banda no viaja sola: viaja con su relleno.

| Relleno | Qué ve el cliente | Cuándo |
|---|---|---|
| **Fondo de escritorio** (por defecto) | una reunión que no llega al borde inferior: lo más ordinario que existe en un Mac | **elegido por el usuario en la mirada 3-quinquies (2026-09-20)**; no hay nada que explicar |
| **Negro** | una franja muerta, tipo letterbox | preferencia de una tecla; para fondos de escritorio con foto o nombre personal, y para quien prefiera no dar ninguna pista de estética |
| **Sin relleno** | el escritorio y las ventanas de detrás | **descartado como defecto**; solo el fallback honesto si el relleno no se pudo dibujar, y entonces la app lo DICE antes de compartir |

Reglas del relleno: nace sin contenido y no puede tener contenido (no es un contenedor, es un color o una imagen); no recibe foco ni clics; no aparece en el conmutador de apps; muere con la banda. **Y su verificación es un gate:** que la captura componga el relleno y no la banda depende del sistema, no de nosotros — parada ⭐ en llamada real, junto a Zoom y Teams (`docs/diseno/posicion.html`).
| Fuera de reunión o por preferencia | tarjeta 380 × 220 en cualquiera de las cuatro esquinas |

Alternativa disponible pero no por defecto: **gota de 44 × 44** abajo a la izquierda en modo
solo audio. Medido y **descartado**: la banda separada del borde tapa los controles de la
videollamada (el botón de colgar).

**Qué ve el cliente en modo acoplado** (verificación ⭐ en llamada real):
- **Compartiendo VENTANA** — ve esa ventana, más pequeña, y nada más. La banda no existe para
  él. Es el modo **más seguro de todos**.
- **Compartiendo PANTALLA completa** — la banda está protegida de la captura, así que en esa
  franja vería **el escritorio de detrás**. No revela contenido, pero sí que algo ocupa el
  espacio. Recomendación de la app en ese caso: compartir ventana.

**Permiso que exige el acople:** Accesibilidad de macOS, **opcional**. Solo se usa para cambiar
tamaño y posición de la ventana de la reunión y devolverla al cerrar. Declarado en
`permisos.html` con lo que la app **no** hace aunque el permiso lo permitiría.
| **Ventana principal** | **960 × 640** (mín. 800 × 560) | rail izquierdo 200 px fijo; contenido con padding 24/32 |

## 4 · Estados — SIEMPRE símbolo + texto + color (daltonismo leve)

| Estado | Símbolo (sprite propio) | Color | Ejemplos |
|---|---|---|---|
| ok / verificado / activo / concedido | `i-check-circle` — ✓ en círculo **relleno** | `--ok` + tinte + borde | «Escuchando», «Meet · protegido», «Concedido», protección propia |
| atención / sin verificar / aviso | `i-alert` — triángulo **relleno** con `!` | `--warn` | «Zoom · sin verificar», radar, «Solo notas», riesgo medio |
| error / revocado / cortado | `i-x-circle` — × en círculo relleno | `--err` | «Revocado», «Cortado · memoria vacía», «no se pudo leer» |
| inactivo / apagado | glifo **tachado** (`i-mic-off`, `i-sistema-off`, `i-pantalla-off`) o `i-ring` | `--mute` | pista apagada, «Sin reunión», «sin bandera», confianza baja |
| interactivo / fijado / detectado / solicitando | contorno o `i-pin-lleno` | `--halo` | ficha fijada, «Meet detectado», «Solicitando…» |
| confianza (sugerencia) | `i-dot` alta · `i-half` media · `i-ring` baja | ok / mute / mute | siempre con la palabra |

**Pares seguros para deutan/protan:** verde-azulado vs. ámbar vs. coral, que además difieren
en luminosidad. Jamás rojo vs. verde como única distinción; jamás el color solo.

## 5 · Componentes canon (todos en `kit.html`, con todos sus estados)

| # | Componente | Clases | Estados | Reglas de uso |
|---|---|---|---|---|
| 1 | **Ficha de evidencia** (C6 · B1) | `.ficha` `.titular` `.linea` `.fuente` `.unidad` `.pin` | `nueva` · `fijada` · `compacta` (bajo sugerencia) · colapsadas (`.mas`) · con sugerencia · confianza baja | titular ≤ 8 palabras (2 renglones máx.) · línea 2 renglones máx. · fuente = `UNIDAD` + documento · sección · pin ⌘⇧P. Máximo 3 fichas; 2ª y 3ª colapsadas. Se reemplazan por relevancia, nunca se reordenan mientras el consultor habla |
| 2 | **Contador de red** (B2) | `.red` (barra) · `.contador` (grande) | `cero` (local, `--ok`) · `api` (bytes en `--halo`) | 0 B en verde; con API los bytes van en el acento (decisión del usuario), **nunca en ámbar ni coral**. El nº de peticiones y «solo texto anonimizado» viven en la versión grande (Honestidad, Ajustes de IA) |
| 3 | **Estado de permiso** (C12) | `.permiso` | sin conceder · `concedido` · `revocado` (a mitad) · `solicitando` | siempre explica **para qué** en una línea; sin conceder la app sigue usable; revocado dice qué se detuvo y qué sigue |
| 4 | **Bandera de jurisdicción** (C11) | `.bandera` | `riesgo-bajo` · `riesgo-medio` · `riesgo-alto` · `desconocida` | dónde · regla · implicación · **fuente + fecha** en Menlo. «Nunca bloquea», «no es asesoría legal» — literal en la pantalla |
| 5 | **Estado de sesión** (C12 · C2 · C1) | `.barra` `.estado` `.pistas` `.pista.off` `.pie .cliente` | fuera de reunión · detectada · activa · solo notas · tras kill-switch | la barra del panel y el chip del rail son el mismo componente; la pista apagada usa glifo tachado; el pie dice cliente + protección (`Meet · protegido` ✓ / `Zoom · sin verificar` ⚠, literal del spike) |
| 6 | **Alerta del radar** (C14) | `.franja.warn` · `.franja.ok` | grabación · bot de notas · agente de monitoreo local · protección propia (verde) | aviso, no bloqueo; solo lo que se lee en **tu** pantalla o corre en **tu** Mac; catálogo con versión y fuente; jamás nombra a personas |
| 6-bis | **Alerta del radar · nivel invasivo** (C14) | `.franja.err` | supervisión de exámenes · anti-trampa con acceso al núcleo · monitoreo de empleados · acceso remoto activo · MDM (este en ámbar) | **dos niveles con símbolo y palabra**: ámbar «sábelo» (legítimo y visible: grabación, bot de notas) · coral «invasivo» (un programa del propio equipo que mira pantalla, cámara, teclas o procesos). Cada alerta declara **qué alcanza a ver**, no solo su nombre. Catálogo versionado con fuente, sin consultar la red |
| 7 | **Píldora de voz** (C15) | `.pildora` | en silencio · `hablando` (halo) · `sin-auriculares` (ámbar, **no habla**) | lee **la misma ficha** que el panel mostraría, nunca un guion. Tres salvaguardas de diseño, no advertencias: (a) sin auriculares no habla —el cliente la oiría—; (b) mientras habla, el disparador se silencia para que su voz no entre por el micrófono; (c) por defecto solo habla si se la pide (⌘⇧A), jamás mientras alguien habla |

**Además** (exhibidos en el panel y el kit): `.sugerencia` (C7: borde izquierdo halo + tinte;
cabecera origen + confianza; fallback = la ficha) · `.franja` (`warn`/`err`/`ok`; lista `→`
vertical para caminos alternos) · `.vacio` (frase Charter cursiva + meta Menlo; nunca un ícono
gris) · `.oido` (pista + hora + cita «») · `.transcript` (oculto por defecto; pistas
`cliente`/`tú`, nunca nombres) · `.kill` (⌥⎋, `--err`) · `.buffer` (`muerto` = tachado, 0 B).

### Primitivas

`.btn` (`.primario` halo, `.mini`) — discretos, el panel no tiene CTAs grandes · `.conm`
(`on`/`off`; **el track lleva palabra al lado**: Activado/Desactivado) · `.campo` (label
arriba, ayuda abajo; `.mono` para claves) · `.tecla kbd` · `.tabla` (`th` Menlo mayúsculas,
`.num` a la derecha, `tr.apagada`) · `.tarjeta` · `.titulo h1 + .sub` · `.rail` (200 px:
marca en Charter + ítems con `aria-current` + `.abajo` con estado de sesión) · `.progreso`
(quieta; el número es el texto) · `.unidad-chip` · `.diccionario mark/del` (término corregido).

## 6 · Iconografía (D9)

Sprite propio en `docs/diseno/assets/iconos.js`: 36 glifos de trazo 1.75, 16 px (`.ic`; 13 px
`.ic.s`), `currentColor`; los rellenos (`.relleno`) son los símbolos de estado. **Cero emojis,
cero librerías.** En producto el sprite pasa a `src/components/iconos.tsx` (o SVG inline) sin
cambiar nombres: `i-check-circle` · `i-alert` · `i-x-circle` · `i-dot` · `i-ring` · `i-half` ·
`i-mic(-off)` · `i-sistema(-off)` · `i-pantalla(-off)` · `i-ojo(-off)` · `i-candado` · `i-rayo` ·
`i-globo` · `i-doc` · `i-pin(-lleno)` · `i-subir` · `i-buscar` · `i-chispa` · `i-video` · `i-rec`
· `i-bot` · `i-radar` · `i-chevron` · `i-mac` · `i-nube(-off)` · `i-auriculares` · `i-nota` ·
`i-ram` · `i-reloj` · `i-basura` · `i-llave` · `i-flecha`.

**Regla del glifo tachado (fase 4, 2026-09-20):** los glifos con barra (`-off`) **nunca llevan
`.relleno`**. El relleno cierra la silueta y se traga la barra: el icono pasa a decir lo
contrario de lo que significa (una nube tachada rellena se lee «subiendo a la nube»). Para
«esto no se hace» el símbolo canon es `i-x-circle` relleno, no un `-off` relleno. Lo encontró la
pasada de capturas leída como imagen, no un test.

## 7 · Accesibilidad — medida, no declarada

### 7.1 Contraste por token (WCAG, calculado sobre los hex de `ghost.css`)

**Oscuro**

| tinta | sobre bg | sobre surface | sobre surface-2 | sobre su tinte |
|---|---|---|---|---|
| `--ink` #ece7dc | 14.25 | 12.74 | 11.18 | — |
| `--ink-2` #b9b3a6 | 8.42 | 7.53 | 6.60 | — |
| `--ink-3` #7c776c | **3.94** | **3.53** | **3.09** | — ← ornamento, vetado como texto |
| `--mute` #9a958a | 5.89 | 5.27 | 4.62 | — |
| `--halo` #9ecbff | 10.41 | 9.31 | 8.17 | 7.69 |
| `--ok` #62d3b4 | 9.62 | 8.60 | 7.55 | 7.06 |
| `--warn` #f0b64a | 9.62 | 8.60 | 7.54 | 7.01 |
| `--err` #ff8577 | 7.42 | 6.64 | 5.82 | 5.78 |

**Claro**

| tinta | sobre bg | sobre surface | sobre surface-2 | sobre su tinte |
|---|---|---|---|---|
| `--ink` #1f1e1a | 14.91 | 16.41 | 13.63 | — |
| `--ink-2` #5a5650 | 6.51 | 7.17 | 5.96 | — |
| `--ink-3` #a19b8e | **2.47** | **2.72** | **2.26** | — ← ornamento, vetado como texto |
| `--mute` #6d685f | 4.95 | 5.44 | 4.52 | — |
| `--halo` #1d5c9c | 6.13 | 6.74 | 5.60 | 5.15 |
| `--ok` #0f6350 | 6.43 | 7.07 | 5.88 | 5.41 |
| `--warn` #7a4f00 | 6.37 | 7.01 | 5.83 | 5.45 |
| `--err` #a32a22 | 6.45 | 7.09 | 5.90 | 5.40 |

**Medido en el DOM real (arnés de la etapa, Chromium):** panel 36 combinaciones (9 estados ×
2 temas × 2 idiomas) → 0 textos bajo AA (peor 5.25 claro / 6.14 oscuro); kit 4 combinaciones
× 313 textos → 0 bajo AA (peor 4.95 claro / 5.08 oscuro).

### 7.2 Tokens VETADOS como texto (fallarán en lint cuando exista `src/`)

`--ink-3` (y su futura clase `text-ink-3`) — solo separadores y ornamento. El barrido de
clases prohibidas sobre `src/` (regla 5b del CLAUDE.md) nace en el S1 con esta lista:
`text-ink-3`, `text-line`, `text-line-2`, `text-surface*`. Demo en rojo en el mismo commit.

### 7.3 Reglas

Todo estado = símbolo + texto + color (§4) · foco visible en halo · `role="status"` en franjas
de aviso y `role="alert"` solo en error · el transcript y las fichas no roban el foco ni anuncian
en vivo (`aria-live` **off**: pasivo) · textos alternativos en ambos idiomas (`lang` en cada
span) · reduced-motion global · teclado: todo atajo tiene su acción visible con `kbd`.

## 8 · Anti-patrones prohibidos

Un guion para leer en voz alta (la voz lee la ficha, no libretos) · hablar por los parlantes
(sin auriculares la voz calla) · modales, sonidos y robo de foco en el panel · spinners, pulsos, animaciones que desplazan ·
color como único portador · emojis como iconografía · blur/«glass»/transparencias · gradientes
decorativos, sombras pesadas · texto en `--ink-3` · vocabulario de ocultamiento
(vocabulario:cita; barrido en `pnpm test`) · nombres de personas en el transcript (solo pistas;
cero biometría) · rojo vs. verde como única distinción · Inter/system-ui como única voz · grid
de cards idénticas · placeholder «Lorem» o inglés residual en la UI en español (y viceversa) ·
CTAs grandes en el panel · tamaños de ventana distintos de los declarados (§3.6) · **glifos `-off` con `.relleno`** (§6) · **una `.fila` con texto largo y botones**: envuelve y deja el botón primario suelto en otra línea — el texto va arriba y los botones en su propia fila (visto dos veces en la fase 4).

## 9 · Contrato con el código futuro (S1 en adelante)

- Los tokens pasan **tal cual** a Tailwind v4 (`@theme` en `src/index.css`): `--color-bg`,
  `--color-surface`, … `--font-evidencia`, `--font-ui`, `--font-mono`, `--radius-*`,
  `--spacing-*`. Los componentes React reproducen las clases canon (`ficha`, `estado`,
  `franja`, `barra`, `permiso`, `bandera`, `contador`, `buffer`, `conm`, …) — **cero valores
  mágicos** en componentes.
- `html[data-theme]` sigue siendo el conmutador de tema (oscuro por defecto; claro obligatorio).
- Bilingüe: en producto los pares `<span lang>` se sustituyen por el diccionario i18n (`es` /
  `en`) con las MISMAS cadenas que la maqueta — la maqueta es el primer diccionario.
- Gate de FIDELIDAD (S1, indiferible): la primera pantalla construida se compara en capturas
  contra `docs/diseno/` a píxel real, ambos temas.
- `NSMicrophoneUsageDescription` / `NSScreenCaptureUsageDescription` usan los textos «para
  qué» de `.permiso` (bilingües).

## 9-quater · La bandeja (cambio de la mirada 4-bis, 2026-09-20)

**El problema:** decidir qué guardar **durante** la reunión es pedirle al usuario que trabaje
mientras habla — lo mismo que el producto entero existe para evitar.

**`.cuenta` + `.ventanas` — la bandeja.** Al cerrar la reunión, las **frases candidatas**
sobreviven en una bandeja cifrada con **cuenta atrás visible**. Reglas del componente, todas
obligatorias en cualquier pantalla que lo use:

1. **La bandeja guarda frases, no la reunión.** Audio, transcript y lecturas de pantalla mueren
   en el instante de cerrar: sin ventana, sin casilla, sin excepción. Eso no se toca.
2. **La ventana la elige el usuario** y puede ser **cero** (`al cerrar · 1 h · 3 h · fin del día ·
   24 h`; defecto 3 h, techo 24 h).
3. **La cuenta atrás se ve**, con la misma lógica que el contador de red: la promesa es visible o
   no es promesa.
4. **Se borra sola al vencer aunque la app no vuelva a abrirse** — mismo mecanismo que la
   retención de 90 días. Una ventana que depende de que el usuario vuelva no tiene fondo.
5. **Aparece en las cuentas de Honestidad.** Es lo único que no muere al instante; omitirlo en la
   pantalla que promete honestidad sería mentir por omisión.

## 9-ter · Tres componentes de la mirada 4 (2026-09-20)

**`.propuesta` — candidata a nota (extiende C9).** Anatomía fija: símbolo · qué propone (voz de
evidencia) · **de dónde salió** · acción. Tres estados: propuesta · aceptada (ok) · descartada
(tachada). **Regla del componente: proponer no es guardar.** Nada entra al archivo sin el sí del
usuario, y lo no aceptado muere con la reunión. La pantalla que lo usa **publica la lista entera
de lo que sabe reconocer** — son reglas, no un modelo adivinando (regla dura 14); el modelo local
solo redacta mejor la propuesta, jamás decide cuál merece guardarse. Una propuesta nacida de lo
que dijo un tercero se guarda como **un hecho en una línea**, nunca su transcripción literal.

**`.idiomas` — varios por pista, no uno.** Cada pista (micrófono · sistema) lleva un conjunto de
idiomas marcables, no un valor. Modos: automático · fijo · varios. El coste se declara en la
misma pantalla (décimas al fin de turno, menos acierto en frases mezcladas) y se separa de la
decisión vecina: **escuchar en N idiomas no obliga a leer en N**.

**`.puerta` — lo que un agente local puede y no puede.** Dos columnas obligatorias: **puede** y
**no puede nunca**, cada fila con su razón. La columna de la derecha no es un descargo legal: es
la parte que se lee primero. **Condición del componente: en reunión la puerta se cierra sola** —
ningún agente alcanza lo que vive en memoria. El registro muestra también lo **denegado**.

## 9-bis · Qué persiste (cambio de la mirada 3, 2026-09-20)

El diseño distingue **lo del usuario** de **lo de terceros**, no «texto» de «audio»:

| | Persiste | Cómo |
|---|---|---|
| Notas y acuerdos escritos | **sí** | cifrado, en la carpeta del usuario, con retención y borrado |
| Turnos del propio consultor (pista de micrófono), **en texto** | **sí, opt-in** | conmutador en Honestidad; por defecto apagado; retención 90 días |
| Fichas mostradas y fijadas | **sí** | son de su propio corpus, no datos del cliente |
| **Audio de cualquier pista** (la suya incluida) | **nunca** | el sonido no se guarda; lo que queda de él es texto |
| Transcript del cliente, su voz, lo leído de la pantalla | **nunca** | no hay conmutador que lo encienda; ni exportación ni cita textual |

La pantalla lo muestra como **dos columnas enfrentadas** («lo tuyo queda» / «lo del cliente
muere») y declara con letra que el usuario responde por su propia carpeta. Detalle y base legal:
`sprints/ETAPA-DISENO-implementation-log.md` § Desviación del plan.

## 9-quinquies · Los seis estados de CONTENIDO de la banda (sprint 001, 2026-09-20)

La Etapa de Diseño decidió la **forma** de la banda (`posicion.html`: 88 · 200 · 44, acoplada,
con asa) y el **contenido** de los estados (`panel.html`, dentro de 380 × 220). Lo que nunca se
escribió fue el cruce: **qué dice la banda en cada estado**. Cinco de los seis estados que el
sprint 001 construye no tenían referencia contra la cual comparar, y un gate de fidelidad sin
referencia es un juicio a ojo. La referencia es `docs/diseno/banda.html`.

**Gramática de la banda** (vale para todo estado, y el código la obedece):

| Zona | Qué lleva, siempre |
|---|---|
| `cab-b` | el estado (símbolo + texto + color) · el cliente y su verificación · el contador de red |
| `ficha-b` (izquierda) | **lo que la banda dice**: una línea fuerte y una de apoyo |
| `lado-b` (derecha) | **de dónde sale** (unidad · documento · sección) y **qué teclas hay** |

| Estado | Alto | Izquierda | Derecha |
|---|---|---|---|
| esperando | 88 | frase en Charter + conteo del corpus | reunión y minutos · atajos |
| buscando | 88 | lo oído entre comillas con su pista + «buscando» estático | atajos |
| ficha | 88 | titular + línea | fuente · atajos |
| ficha ampliada | 200 | titular + línea + **las dos acumuladas abiertas** | fuente · atajos |
| sin resultado | 88 | qué buscó y no encontró + **la maniobra** | lo más cercano que sí tienes · las salidas como teclas |
| sin resultado ampliada | 200 | la pregunta entera + la maniobra + **las tres más cercanas** | los dos botones del panel |
| sin verificar | 88 | aviso ámbar + **una** salida | cliente, versión y fecha verificadas |
| sin verificar ampliada | 200 | el aviso con las **tres** salidas | los dos botones del panel |
| transcript | 200 | la ficha y sus acumuladas, intactas | el transcript, 3 turnos **por pista** |
| sin acople (fallback) | 88 | igual que ficha | igual que ficha; la cabecera añade «sin acople» |

**Tres decisiones que la maqueta no había escrito:**

1. **En la banda, las acciones son TECLAS.** El panel de 380 × 220 tenía botones; en 88 px de
   alto dos botones y una frase larga se pelean por el renglón y el primario cae abajo — el
   anti-patrón de §8 que ya mordió dos veces. Los botones vuelven **al ampliar**, donde hay alto.
2. **El asa tiene un trabajo.** «Sin verificar» tiene tres salidas y en 88 px cabe una: la banda
   muestra la primera y dice que el asa muestra el resto. Ampliar deja de ser decorativo — es
   donde vive lo que no cabía. Y el alto es **continuo**: 88 px es el reposo, 200 px el máximo
   que dibujó el diseño.
3. **El transcript va a la derecha, no abajo.** En el panel vertical crecía hacia abajo; una
   banda ya ocupa el ancho entero y no puede crecer más. Ocupa la columna donde irá la
   sugerencia (sprint 2), y así **la banda no cambia de alto al encenderlo**.

### La MANIOBRA — qué sugiere la app cuando el corpus no tiene nada (mirada 11, 2026-09-20)

Decir «no tengo nada» y callarse deja al consultor solo justo cuando más lo necesita. Pero la app
**no puede inventar** una respuesta sobre su negocio: sería exactamente lo que promete no hacer, y
este sprint es **cero LLM** (regla del código primero). Así que sugiere **dos cosas, las dos
deterministas**:

1. **Lo más cercano que SÍ tiene.** La búsqueda no encontró nada por encima del umbral, pero sabe
   qué quedó justo debajo. Sale del corpus del usuario con su fuente exacta, y la app dice sin
   adornos que **ninguno responde la pregunta**. Es recuperación, no redacción.
2. **Una manera de responder**, de un **catálogo versionado** de seis maniobras escritas por
   personas, elegida por **reglas léxicas** sobre lo que preguntó el cliente — el mismo mecanismo
   determinista del disparo.

| Si la pregunta trae… | Maniobra | Por qué esa |
|---|---|---|
| `certificación · ISO · acreditado · licencia` | Dilo sin adornos y ofrece confirmarlo hoy mismo. | una credencial se tiene o no; dudar cuesta más que el «no» |
| `cuánto · precio · costo · descuento · tarifa` | No improvises cifras: ofrece el rango del caso comparable. | un número dicho al aire se vuelve compromiso |
| `cuándo · plazo · semanas · entrega` | Da el plazo del caso más parecido y confírmalo por escrito. | anclar en un caso real es defendible; una fecha inventada, no |
| `quién más · referencia · han trabajado con` | Ofrece una referencia del sector sin nombrar al cliente aún. | nombrar clientes sin permiso es un problema, no una venta |
| `contrato · cláusula · penalidad · NDA` | No opines de contrato en vivo: anótalo y respóndelo por escrito. | lo contractual no se improvisa en una llamada |
| *cualquier otra* | Devuelve la pregunta: ¿para qué lo necesitan? | la pregunta real suele ser otra — y da tiempo |

**Las maniobras hablan de cómo conducirse, jamás del negocio del usuario** — por eso pueden ser
fijas. Y no pueden parecer salida de un modelo: **sin acento `halo` y sin `i-chispa`** (que en
este sistema marcan la síntesis de la IA), en **Avenir** —la voz de la app— nunca en **Charter**,
que es la voz de la evidencia. Clase canon: `maniobra-b`; el vecino de la derecha, `cercano-b`.

Cuando exista la síntesis con modelo (sprint 2), la maniobra **no desaparece**: es su fallback
permanente, como manda la regla del código primero.

**Lo que esta extensión NO redecide:** el relleno de la franja (ya elegido: fondo de escritorio,
negro a una tecla), el modo solo audio de 44 px (C15, fuera del sprint 001) y la sugerencia
(sprint 2). La `unidad` sigue **sin chip** en `fuente-b`, a diferencia del panel: la columna
derecha de la banda es toda Menlo de bajo contraste —«lo medible»— y un tercer peso visual junto
a los `kbd` la volvería ruido. El chip sí aparece en las **acumuladas** de la banda ampliada,
donde la unidad es lo que distingue una ficha de otra.

**Altura y código:** las tres alturas son tokens (`--banda-h`, `--banda-h-ampliada`,
`--banda-h-voz`) y son la fuente única — el CSS las usa y la ventana nativa las lee. El gate de
tokens las excluye a propósito (el webview ocupa la ventana entera, no la dibuja); las vigila un
test de `src-tauri/` que falla si la ventana y la maqueta se separan.

## 9-sexies · «TODAVÍA NO» — el estado de lo que aún no está construido (sprint 001, fase 2, 2026-09-21)

**El problema.** La maqueta dibuja el producto TERMINADO. Cada sprint entrega un trozo. Al
construir las pantallas del cuaderno en el sprint 001 aparecieron tres cartas con contenido que
el producto de hoy no puede sostener: las dos pistas de audio (fase 3), la ficha del cliente
(sprints posteriores), los búferes de memoria (fase 3), las notas (S3).

Sin una respuesta escrita solo quedan dos salidas, y las dos son malas:

| Salida | Por qué no |
|---|---|
| Pintarlo como la maqueta, en verde | «Micrófono — Listo» con el audio sin construir es exactamente la afirmación falsa que esta app existe para no hacer |
| Quitarlo de la pantalla | el usuario no sabe que va a llegar, y la pantalla del sprint 1 se lee como el producto completo |

**La decisión: un quinto significado en el vocabulario de estados.** `.estado.pendiente` —
**«todavía no» / «not yet»**.

Y es distinto de `.mute`, que ya existía: **`.mute` significa «existe y está apagado»** (se puede
encender), **`.pendiente` significa «todavía no está construido»** (no hay interruptor). Confundir
los dos manda al usuario a buscar un botón que no existe.

**Tres señales, porque el color solo no basta** (daltonismo leve del usuario) — y aquí la señal
principal **no es el color**, que es el mismo `--mute`:

1. **trazo discontinuo** en el borde del chip — es lo que lo separa de `.mute` de un golpe de
   vista, y funciona igual en los dos temas;
2. **aro punteado** (`#i-pendiente`), no el aro continuo de `.mute`;
3. **la palabra literal**: «todavía no».

`.tarjeta.pendiente` aplica lo mismo a una tarjeta entera; `.fila.pendiente` baja el énfasis de la
fila sin ocultarla.

### La otra mitad: el estado «así se ve hoy» en la maqueta

Un componente para decir «esto no existe» no basta: hay que poder **comparar** la pantalla
entregada contra una referencia, o el gate de FIDELIDAD del sprint no tiene contra qué medir.

Por eso cada pantalla que se construye a medias gana un estado **`s1`** en la maqueta — la misma
pantalla, tal y como se entrega. La maqueta conserva la visión completa **y** registra qué se
entregó en cada versión. Cuando la fase 3 traiga el audio, el estado `s1` de sesión se convierte
en `s3` con menos «todavía no», y el que quiera ver qué cambió tiene las dos.

### Una regla de producto que sale de aquí

**Lo que no existe se dice; nunca se rellena.** La tarjeta «Este cliente» del sprint 1 no muestra
una bandera gris ni un riesgo en blanco: muestra una frase — *«No se inventa nada mientras no
exista: ni bandera, ni riesgo, ni catálogo»*. Y el kill-switch informa **«3 de 7 piezas: las otras
cuatro todavía no existen»**, no «7 de 7».

### Dos cosas que la maqueta no había escrito y el sistema obliga

1. **«Audio del sistema» y «Pantalla» son UN SOLO permiso en macOS** («Grabación de pantalla y
   audio del sistema»): se conceden y se caen juntos. Se siguen dibujando como dos filas —son dos
   usos distintos y el usuario los entiende así— con una línea que lo dice.
2. **La Accesibilidad sube a la lista principal de permisos.** En la maqueta vivía en su propio
   estado porque era opcional y futura; el acople se entrega en el sprint 001 y es **el único
   permiso que hoy cambia algo**.

## 10 · Deuda de diseño declarada

| Qué | Por qué | Cuándo se paga |
|---|---|---|
| «Solo texto anonimizado» no cabe en la cabecera de la sugerencia con API dentro de 380 px | robaba la ficha | vive en tooltip + Honestidad + Ajustes de IA; se revisa si el usuario lo pide en G-Diseño |
| El pie del panel abrevia «corta» (kill-switch) | 380 px | el `kbd` ⌥⎋ y el tooltip completan; en Sesión y Honestidad va el texto entero |
| Simulación deutan del arnés (capturas `--cvd`) | herramienta de la etapa, no gate | corregida en la Fase 2; se vuelve gate visual del S1 |
| La píldora de voz no muestra el texto de la ficha | ocuparía la pantalla que el modo existe para liberar | si el usuario lo pide, un estado «píldora expandida» en el S2 |
| **La maniobra genérica deja solo al consultor** — «devuelve la pregunta: ¿para qué lo necesitan?» | es la única de las seis que no se apoya en nada: cuando ni las reglas léxicas ni el corpus dan material, la app se queda sin qué decir | **sprint 2.** Requisito del usuario (mirada 11): *no inventar una respuesta, pero sugerir cómo abordar la situación* **a medida de la situación**. Camino determinista disponible sin LLM: construir la maniobra con lo que la app **sí sabe** —la unidad que falta, la sección más cercana del corpus, la ficha del cliente, la jurisdicción, lo ya comprometido en esta reunión— en vez de elegir una frase de catálogo. El catálogo queda como último recurso, no como respuesta normal |

## Registro de cambios

| Versión | Fecha | Qué |
|---|---|---|
| 0.1.0 | 2026-09-20 | v0 borrador para la mirada 1 (el panel) — aprobado |
| 1.0.0 | 2026-09-20 | completo: 6 componentes canon con todos sus estados, primitivas, tabla de contraste, vetados, contrato con el código |
| 1.1.0 | 2026-09-20 | mirada 3: píldora de voz (C15, 7.º componente canon) · §9-bis qué persiste · dos prohibidos nuevos |
| 1.2.0 | 2026-09-20 | mirada 3 (2.ª vuelta): banda inferior y gota · posición elegible (D2 revisada) · radar con dos niveles de severidad y cinco categorías |
| 1.3.0 | 2026-09-20 | mirada 3-ter: **banda acoplada** como forma principal (88/200/44, con asa) · permiso de acople · qué ve el cliente por modo de compartir |
| 1.4.0 | 2026-09-20 | mirada 3-quinquies: **relleno de captura** (fondo de escritorio elegido; negro a una tecla) |
| 1.5.0 | 2026-09-20 | mirada 4: las cuatro pantallas del cuaderno (corpus · notas · idioma · IA) |
| 1.6.0 | 2026-09-20 | mirada 4-bis: `propuesta` (proponer ≠ guardar) · varios idiomas · puerta local para Claude Code |
| 1.7.0 | 2026-09-20 | mirada 4-ter: **bandeja de propuestas con cuenta atrás** (§9-quater) · G-Diseño aprobado |
| 1.8.0 | 2026-09-20 | sprint 001, fase 1a: **§9-quinquies — los seis estados de CONTENIDO de la banda** · tokens de alto (`--banda-h*`) · acciones como teclas · el asa con un trabajo · transcript a la derecha |
| 1.9.0 | 2026-09-20 | mirada 11: **la maniobra** — catálogo versionado de seis maneras de responder + «lo más cercano que sí tienes», los dos deterministas; `maniobra-b` y `cercano-b`; estado «sin resultado · ampliada» |
