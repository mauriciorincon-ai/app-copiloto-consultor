# Etapa de Diseño — registro vivo (Angel Ghost)

> Registro de la Etapa de Diseño (F2a). Se completa en la Fase 5 (tabla pantalla →
> funcionalidad, tabla del spike, auto-auditoría, veredicto G-Diseño). Desde ya lleva el
> **registro de miradas**: ningún artefacto visual se construye encima sin feedback del usuario
> fechado. «Continúa» no aprueba diseño.

## Cómo abrir la maqueta

`git checkout diseno/fundacion` → doble clic en el archivo `.html` que nombra cada mirada (o en
`index.html` cuando exista). Sin build, sin red. La barra superior conmuta **estado · tema ·
idioma**; la nota bajo la barra dice qué mirar.

## Plan de miradas (aprobado con el plan, 2026-09-20)

| Mirada | Artefacto(s) | Orden |
|---|---|---|
| 1 | `panel.html` ⭐ (+ `assets/ghost.css` y `design-system.md` v0) | el panel primero, siempre |
| 2 | `kit.html` + `design-system.md` completo | |
| 3 | `sesion.html` · `permisos.html` · `honestidad.html` | |
| 4 | `corpus.html` · `notas.html` · `idioma.html` · `ia.html` | |
| 3-bis | `honestidad.html` (qué queda) + `panel.html` (modo solo audio) — **añadida en la mirada 3** | antes de la Fase 4 |
| 3-ter | `posicion.html` (dónde vive el panel) + radar invasivo — **añadida en la mirada 3-bis** | antes de la Fase 4 |
| 3-quater | `posicion.html` (banda acoplada D2/D3/F) + `permisos.html` (permiso de acople) — **añadida en la mirada 3-ter** | antes de la Fase 4 |
| 3-quinquies | `posicion.html` (relleno de captura: qué ve el cliente en la franja) — **añadida en la mirada 3-quater** | antes de la Fase 4 |
| 5 = G-Diseño | `index.html` + recorrido completo | |

Cualquier cambio al plan (agrupar, reordenar, posponer) se propone y se aprueba ANTES de
construir el siguiente artefacto.

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
| — | `corpus.html` · `notas.html` · `idioma.html` · `ia.html` (mirada 4) | *(pendiente)* | — |

## Decisiones de diseño declaradas antes del segundo artefacto

D1–D13 en `sprints/ETAPA-DISENO-implementation-log.md` (tamaño 380 × 220, esquina superior
derecha, opaco, Charter · Avenir Next · Menlo, iconografía SVG sin emojis, motion casi nulo,
bilingüe pareado, datos sintéticos «Páramo Azul», estructura de archivos).

## Pantalla → funcionalidad de la VISION

*(se completa en la Fase 5; contrato: las 17 funcionalidades `[MVP · personal]` de VISION
v1.2.0, ninguna sin pantalla, ninguna pantalla sin funcionalidad)*

## Registro de G-Diseño

*(pendiente — se llena con el veredicto sobre la maqueta completa abierta en el Mac del usuario)*
