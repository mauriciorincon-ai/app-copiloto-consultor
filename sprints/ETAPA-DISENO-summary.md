---
sprint: ETAPA-DISENO
app: copiloto-consultor
status: closed
opened: 2026-09-20
closed: 2026-09-20
branch: diseno/fundacion
pr: https://github.com/mauriciorincon-ai/app-copiloto-consultor/pull/3
---

# Etapa de Diseño (F2a) — Summary · Angel Ghost

## Outcome

**Sí — G-Diseño APROBADO el 2026-09-20** («sí apruebo la pantalla completa»).

**Sí.** La fundación visual existe y el usuario la recorrió entera: `design-system.md` completo
(v1.7.0) + maqueta navegable del H1 completo en `docs/diseno/` — **nueve pantallas, 51 estados**,
dos temas, dos idiomas, datos 100 % sintéticos, HTML autocontenido que se abre con doble clic.
**Cero código de producto:** `src/` y `src-tauri/src/` no aparecen en el diff de la rama.

## Qué se construyó

| Entregable | Qué es |
|---|---|
| `design-system.md` v1.7.0 | personalidad · tesis de tres voces tipográficas · tokens en ambos temas con contraste **medido por token** · 7 componentes canon + los 4 nacidos en las miradas · motion y reduced-motion · a11y medida · anti-patrones · contrato con el código futuro · deuda declarada |
| `docs/diseno/` | `index.html` (recorrido) + 9 pantallas + `posicion.html` (el comparador que decidió la forma) + `kit.html` + `assets/{ghost,maqueta}.css`, `maqueta.js`, `iconos.js` (sprite de 38 glifos propios) |
| `docs/diseno/README.md` | tabla pantalla → funcionalidad de la VISION · tabla del spike copiada · auto-auditoría por tema · **registro de las 10 miradas** · gates con su demo en rojo · veredicto de G-Diseño |
| 3 gates nuevos en `quality` | `vocabulario-vetado` · `maqueta-autocontenida` · `maqueta-sin-emojis` |

**Decisiones de forma que la orden no escribía** y se decidieron midiendo, no opinando: la
**banda inferior acoplada** (88 px, ampliable a 200, 44 en solo audio) reemplaza a la tarjeta
flotante como forma principal — se eligió sobre `posicion.html`, que dibujó las colisiones reales
de ocho posiciones sobre un escritorio de referencia y **descartó la opción C** porque tapaba el
botón de colgar.

## DoD — los 6+1 estándares

| Estándar | Estado | Evidencia |
|---|---|---|
| Testing | ✓ | 9 tests en `quality`; **los 3 nuevos se vieron en rojo** antes de verlos en verde |
| CI/CD | ✓ | **conclusión propia `success`** en los tres checks requeridos: `quality` 24 s · `e2e` 37 s · `build-escritorio` 1 m 20 s. Ninguno `skipped`. `build-escritorio` y `e2e` corrieron por primera vez con tests reales en esta rama |
| Observabilidad | n/a | no hay código de producto todavía; entra por ADR en el S1 |
| Seguridad | ✓ | gitleaks en cada commit; **barrido de CERO ENLACES limpio** (se corrigió el único hit heredado, `CHANGELOG.md`) |
| Performance | ✓ (de diseño) | presupuesto declarado: fin de turno → ficha ≤ 4 s; el kit de evaluación lo maqueta medido |
| UX + A11y | ✓ | **0 textos bajo AA** sobre ~200 capturas; **0 desbordes**; símbolo + texto + color en todo estado; pasada deutan |
| IA embebida responsable | ✓ (de diseño) | local por defecto, API apagado con clave del usuario, contador de red visible, kit de evaluación con su fila incómoda a la vista |

## Métricas

| Métrica | Resultado |
|---|---|
| Pantallas / estados | 9 / 51 |
| Capturas leídas como imagen | ~200 (estado × tema × idioma) |
| Textos bajo AA | **0** |
| Desbordes de ventana | **0** |
| Funcionalidades de la VISION cubiertas | 19/19 + C14, C15, C16 |
| Miradas del usuario registradas | **10** (plan de 5, ampliado 5 veces a petición suya) |
| Código de producto escrito | **0 líneas** |

## Decisiones no anticipadas → **cinco cambios de PRODUCTO**

Detalle y razones en `## Desviación del plan` de la bitácora. **La planeadora debe absorberlos:
la VISION pasa de 25 a 27 funcionalidades.**

| # | Cambio | Origen | Qué toca |
|---|---|---|---|
| A | Lo que el consultor dijo o escribió **se conserva** (opt-in, cifrado) | mirada 3 | regla dura 1 + VISION |
| B | **C15 · modo solo audio** — la app habla en paralelo | mirada 3 | VISION: funcionalidad nueva |
| C | La app **propone** notas; decidir sigue siendo del usuario | mirada 4 | extiende C9 |
| D | **C16 · puerta local** para que Claude Code opere la app | mirada 4 | VISION: funcionalidad nueva, **con su condición escrita: en reunión la puerta se cierra sola** |
| E | **Bandeja** de propuestas con ventana elegible (defecto 3 h) | mirada 4-bis | **relaja la regla dura 1** bajo cinco condiciones |

## Bugs y resoluciones

| Qué falló | Cómo se encontró | Arreglo |
|---|---|---|
| `[hidden]` perdía contra las clases `display:` — todos los estados pintados a la vez | pasada de capturas | `[hidden] { display: none !important }` |
| Tema claro: `--ok` 3.96 · `--warn` 4.49 · `--err` 4.45 | medición por token | `#0f6350` · `#7a4f00` · `#a32a22` (5.25 · 5.45 · 5.40) |
| Glifo `-off` con `.relleno` pierde la barra y **dice lo contrario** | capturas leídas como imagen | anti-patrón nuevo; `i-x-circle` para «esto no se hace» |
| `.fila` con texto largo + botones envuelve y suelta el botón primario | capturas (dos veces) | anti-patrón nuevo: el texto arriba, los botones en su fila |
| La maqueta declaraba «cero emojis» **con cuatro dentro** | el gate nuevo, al nacer | `maqueta-sin-emojis` + limpieza |
| Cinco estados desbordaban 960 × 640 | el arnés avisaba y **no leí la salida** | compactados; 0 desbordes |
| La barra `sticky` del arnés cortaba el `h1` en cada captura | leer la imagen | defecto de la herramienta, no del diseño |

## Qué salió bien / qué generó fricción

**Bien.** Medir en vez de opinar: el arnés capturó estado × tema × idioma, midió contraste nodo a
nodo y avisó de desbordes; casi todos los defectos reales los encontró **leer la imagen**, no los
tests. Y el sistema aguantó: las cuatro pantallas de la Fase 4 se compusieron **sin una sola
clase nueva**.

**Fricción.** Tres veces cometí la misma falta en formas distintas: comitear con un test en rojo
por encadenar comandos sin leer la salida; narrar el barrido de enlaces citando el dominio
literal, rompiéndolo; y no leer los avisos de desborde del arnés. **Leer la salida ES el gate.**

## Sugerencias de mejora al método

1. **El plan de miradas debería nacer con «ramas» previstas.** El de 5 se amplió a 10 porque cada
   mirada del usuario abría un cambio de producto. Funcionó —cada ampliación se propuso y se
   aprobó antes de construir— pero el método lo trata como excepción cuando es lo normal en una
   etapa de diseño sana. Sugerencia: que el plan declare «N miradas base + ramas esperadas».
2. **La regla 10 necesita decir qué hacer cuando la aprobación es ambigua.** Dos veces llegó
   «listo, continúa» tras una repregunta. Repreguntar otra vez raya en no creerle al usuario;
   seguir sin evidencia rompe el gate. Lo resuelto aquí —registrar literalmente lo que el usuario
   dijo, con la nota de que no describió el artefacto, y **devolver ese artefacto a una mirada
   posterior**— parece la salida correcta y podría escribirse en el método.
3. **Un gate que narra a otro gate debe nacer con su marcador de cita.** Pasó dos veces en esta
   etapa (enlaces y vocabulario). El kit ya trae `vocabulario:cita`; convendría que la regla 17 lo
   nombre explícitamente para el barrido de enlaces también.

## Deuda técnica aceptada

| Qué | Por qué | Cuándo se paga |
|---|---|---|
| Chrome de la sala de diseño solo en español | es andamiaje, no producto; no viaja al código | no se paga: muere con la etapa |
| Tres promesas **sin verificar en vivo**: Zoom/Teams, el relleno de la franja, la devolución de la ventana al desacoplar | necesitan una llamada real | paradas del gate de prueba del S1 |
| `design-sync/` no existe todavía | no hay ciclo cerrado que publicar | primer cierre de ciclo |

## Archivos clave

`design-system.md` · `docs/diseno/index.html` · `docs/diseno/panel.html` ·
`docs/diseno/posicion.html` · `docs/diseno/notas.html` · `docs/diseno/ia.html` ·
`docs/diseno/assets/ghost.css` · `docs/diseno/README.md` ·
`sprints/ETAPA-DISENO-implementation-log.md` · `tests/unit/maqueta-sin-emojis.test.ts`

## Cómo probar

1. `git checkout diseno/fundacion` → **doble clic en `docs/diseno/index.html`**.
2. Entra a cada pantalla y conmuta **estado · tema · idioma** en su barra superior.
3. `pnpm test` → 9 verdes. Para ver un gate fallar: planta un emoji en cualquier `.html` de
   `docs/diseno/`, o la palabra «indetectable», y vuelve a correrlo.
4. `git grep -nE "vercel[.]app|workers[.]dev|pages[.]dev" -- ':!pnpm-lock.yaml'` → sin resultados.
