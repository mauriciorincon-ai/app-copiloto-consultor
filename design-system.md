---
app: copiloto-consultor
nombre: Angel Ghost
version: 0.1.0   # v0 BORRADOR (Fase 1, mirada 1: lo que el panel exhibe). Completo en la Fase 2.
fecha: 2026-09-20
estado: borrador
fuente_en_codigo: docs/diseno/assets/ghost.css
---

# Angel Ghost — design system (v0, borrador de la mirada 1)

> Fuente de verdad visual de la app. Toda pantalla de producto la obedece; se extiende por ADR,
> nunca se contradice en silencio. **Este v0 declara lo que el panel ya exhibe**; la Fase 2 lo
> completa (todos los componentes canon con todos sus estados, tabla de contraste medida,
> tokens vetados, contrato con el código).

## 1 · Personalidad (D7)

**discreto · propio · verificable** — nunca *ruidoso*, nunca *«stealth»* (vocabulario:cita),
nunca *corporativo*.

Es un **cuaderno privado** que vive al lado de una videollamada: se lee en ≤ 2 s, no compite
con la reunión, y lo que promete («nada queda», «nada sale») se **ve** — el contador de red y
el panel de honestidad son componentes, no páginas de ajustes.

## 2 · La tesis del sistema

Tres voces tipográficas con **roles**, no estilos (D4):

| Voz | Familia (nativa de macOS) | Qué dice |
|---|---|---|
| **Evidencia** | Charter → Iowan Old Style → Georgia | lo que dice **tu** corpus: titular y línea de la ficha, la frase del vacío, lo que se oyó |
| **Interfaz** | Avenir Next → Helvetica Neue → system-ui | estados, botones, avisos, sugerencia |
| **Medible** | Menlo → SF Mono → ui-monospace, `tabular-nums` | contador de red, fuente de la ficha, tiempos, atajos |

Cero bytes de fuentes en el binario: las tres vienen con macOS (la app es macOS-only).

## 3 · Tokens (implementados como CSS variables en `ghost.css`)

### 3.1 Color — tema OSCURO (primario)

| Token | Hex | Rol |
|---|---|---|
| `--bg` | `#1a1917` | ventana (grafito cálido, no negro) |
| `--surface` | `#242320` | ficha, tarjetas |
| `--surface-2` | `#2e2d29` | chips, teclas, hover |
| `--line` / `--line-2` | `#3b3a35` / `#4a4841` | bordes / bordes en foco |
| `--ink` | `#ece7dc` | texto principal (marfil) |
| `--ink-2` | `#b9b3a6` | texto secundario — AA sobre `bg` y `surface` |
| `--ink-3` | `#7c776c` | **SOLO ORNAMENTO** (separadores). **Vetado como texto** |
| `--halo` | `#9ecbff` | acento único: foco, ficha fijada, sugerencia, interactivo |
| `--ok` | `#62d3b4` | activo · verificado · verde |
| `--warn` | `#f0b64a` | atención · sin verificar · aviso |
| `--err` | `#ff8577` | error · cortado · revocado |
| `--mute` | `#9a958a` | inactivo (texto, AA) |

### 3.2 Color — tema CLARO (papel)

| Token | Hex |
|---|---|
| `--bg` / `--surface` / `--surface-2` | `#f5f2ea` / `#fffdf8` / `#ece8de` |
| `--line` / `--line-2` | `#d6d1c4` / `#bdb7a8` |
| `--ink` / `--ink-2` / `--ink-3` (ornamento) | `#1f1e1a` / `#5a5650` / `#a19b8e` |
| `--halo` | `#1d5c9c` |
| `--ok` / `--warn` / `--err` / `--mute` | `#0f6350` / `#8a5b00` / `#b8322a` / `#6d685f` |

Tintes de estado: `rgba(color, .13–.16)` — el **15 %** de la regla «positivo = tinte + borde
sólido + ✓ en círculo relleno».

### 3.3 Tipografía, espacio, radios, motion

| Grupo | Valores |
|---|---|
| Escala (panel) | titular **17 px** (Charter 700, ≤ 8 palabras, máx. 2 renglones) · línea **14 px** (máx. 2 renglones) · UI 13 px · meta 12 px · mono 11.5 px · pie 11 px |
| Espacio | múltiplos de 4: 4 · 8 · 12 · 16 · 20 · 24 · 32 · 40 |
| Radios | chip 5 · ficha 8 · panel 10 · tecla 4 |
| Trazo | 1 px (`--line`); estados con borde sólido del color del estado |
| Sombras | panel: `0 10px 30px rgba(0,0,0,.45)`; ficha: 1 px de asiento. Nada más |
| Motion (D10) | `150ms cubic-bezier(.2,.6,.2,1)`. Ficha nueva: fade, **sin desplazamiento**. «Buscando…»: texto estático, sin spinner ni pulso. `prefers-reduced-motion` ⇒ cero transiciones. **La forma del árbol nunca depende del motion** |

### 3.4 Ventanas (D1, D2, D3, D6)

| Ventana | Tamaño | Notas |
|---|---|---|
| Panel flotante | **380 × 220** · con transcript 380 × 420 · máx. 380 × 360 con 3 fichas | opaco, sin blur; esquina superior derecha, 16 px del borde, 8 px bajo la barra de menús; todos los Spaces; posición recordada |
| Ventana principal | 960 × 640 (mín. 800 × 560) | rail izquierdo fijo (Fase 3) |

## 4 · Estados — SIEMPRE símbolo + texto + color (daltonismo leve)

| Estado | Símbolo | Color | Ejemplo |
|---|---|---|---|
| ok / verificado / activo | ✓ en círculo **relleno** | `--ok` + tinte + borde sólido | «Escuchando», «Meet · protegido» |
| atención / sin verificar | triángulo **relleno** con `!` | `--warn` | «Zoom · sin verificar», radar |
| error / cortado | × en círculo relleno | `--err` | (Fase 2) |
| inactivo | glifo **tachado** (mic-off, pantalla-off) | `--mute` | pista apagada |
| interactivo / fijado / sugerencia | contorno o pin relleno | `--halo` | ficha fijada, sugerencia |

Pares seguros para deutan/protan: verde-azulado vs. ámbar vs. coral — jamás rojo vs. verde
como única distinción, y jamás el color solo (verificado con simulación deutan en la pasada de
capturas).

## 5 · Componentes canon exhibidos en el panel (v0)

- **Barra de estado** (C12 · C2 · C8 · B2): pastilla de sesión + pistas (mic · sistema · pantalla)
  + **contador de red** (Menlo; `0 B` en `--ok`; con API, bytes en `--halo` — no en rojo: fue
  decisión del usuario).
- **Ficha de evidencia** (C6 · B1): titular · línea · fuente = `UNIDAD` + documento · sección +
  pin (⌘⇧P). Variantes: `nueva` (fade), `fijada` (borde halo), `compacta` (sin línea, bajo
  sugerencia). Fichas 2ª y 3ª colapsadas en una fila «+2».
- **Sugerencia** (C7): borde izquierdo halo + tinte; cabecera con origen (`en tu Mac` · `Claude
  Haiku · API`) y **confianza** (símbolo ◐ + texto). Fallback permanente: la ficha.
- **Franja de aviso** (`warn` / `err` / `ok`): tinte 15 % + borde sólido + símbolo relleno +
  título + texto; lista `→` vertical para caminos alternos. Usada por el estado «cliente sin
  verificar» (spike) y por la **alerta del radar** (C14).
- **Vacío diseñado**: frase en Charter cursiva + metadatos en Menlo. Sin ícono gris.
- **Lo que se oyó** (`.oido`): pista + hora en Menlo + cita en Charter con «comillas».
- **Transcript en vivo** (C3): oculto por defecto; turnos con pista (sistema = cliente, mic =
  tú), nunca nombres; «solo en memoria».
- **Pie**: cliente + protección (`Meet · protegido` ✓ / `Zoom · sin verificar` ⚠) + atajos
  con `kbd` (⌘⇧A ayúdame · ⌥⎋ corta).
- **Tecla** (`kbd`), **botón discreto** (`.btn`, `.mini`, `.primario`), **estado** (`.estado`).

## 6 · Anti-patrones prohibidos (desde el v0)

Emojis como iconografía · gradientes decorativos · blur/«glass» · spinners y pulsos · modales y
sonidos en el panel · animaciones que desplazan · Inter/system-ui como única voz · sombras
pesadas en todo · vocabulario de «trampa»/«indetectable» (vocabulario:cita; barrido en
`pnpm test`) · color como único portador de un estado · texto en `--ink-3`.

## 7 · Accesibilidad — medida, no declarada

Contraste WCAG medido por nodo de texto en cada estado × tema × idioma (arnés de la etapa):
**36 combinaciones del panel, 0 textos bajo AA** (peor caso 5.25 en claro, 6.14 en oscuro).
La tabla completa por token llega en la Fase 2.

## Registro de cambios

| Versión | Fecha | Qué |
|---|---|---|
| 0.1.0 | 2026-09-20 | v0 borrador para la mirada 1 (el panel) |
