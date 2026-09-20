---
app: copiloto-consultor
nombre: Angel Ghost
version: 1.0.0   # completo (Fase 2, mirada 2). 0.1.0 fue el borrador de la mirada 1 (el panel).
fecha: 2026-09-20
estado: propuesto   # → aprobado con G-Diseño
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

Modales, sonidos y robo de foco en el panel · spinners, pulsos, animaciones que desplazan ·
color como único portador · emojis como iconografía · blur/«glass»/transparencias · gradientes
decorativos, sombras pesadas · texto en `--ink-3` · vocabulario de ocultamiento
(vocabulario:cita; barrido en `pnpm test`) · nombres de personas en el transcript (solo pistas;
cero biometría) · rojo vs. verde como única distinción · Inter/system-ui como única voz · grid
de cards idénticas · placeholder «Lorem» o inglés residual en la UI en español (y viceversa) ·
CTAs grandes en el panel · tamaños de ventana distintos de los declarados (§3.6).

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

## 10 · Deuda de diseño declarada

| Qué | Por qué | Cuándo se paga |
|---|---|---|
| «Solo texto anonimizado» no cabe en la cabecera de la sugerencia con API dentro de 380 px | robaba la ficha | vive en tooltip + Honestidad + Ajustes de IA; se revisa si el usuario lo pide en G-Diseño |
| El pie del panel abrevia «corta» (kill-switch) | 380 px | el `kbd` ⌥⎋ y el tooltip completan; en Sesión y Honestidad va el texto entero |
| Simulación deutan del arnés (capturas `--cvd`) | herramienta de la etapa, no gate | corregida en la Fase 2; se vuelve gate visual del S1 |

## Registro de cambios

| Versión | Fecha | Qué |
|---|---|---|
| 0.1.0 | 2026-09-20 | v0 borrador para la mirada 1 (el panel) — aprobado |
| 1.0.0 | 2026-09-20 | completo: 6 componentes canon con todos sus estados, primitivas, tabla de contraste, vetados, contrato con el código |
