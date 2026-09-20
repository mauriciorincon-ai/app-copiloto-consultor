---
etapa: Etapa de Diseño (F2a)
app: copiloto-consultor
branch: diseno/fundacion
opened: 2026-09-20
status: open
orden: ~/Code/hr01-develop-ai-apps/portafolio/copiloto-consultor/ordenes/DISENO-orden.md
plan aprobado: 2026-09-20 (plan mode → «construye»)
---
# Etapa de Diseño — bitácora de implementación (Angel Ghost)

> Cero código de producto hasta G-Diseño. Entregables: `design-system.md` + `docs/diseno/`
> (maqueta navegable del H1, 8 pantallas) + `docs/diseno/README.md` con registro de miradas.

## Plan de miradas (declarado en el plan, aprobado 2026-09-20)

| Mirada | Artefacto(s) | Estado |
|---|---|---|
| 1 | `docs/diseno/panel.html` ⭐ (+ `ghost.css` y `design-system.md` borrador) | pendiente |
| 2 | `docs/diseno/kit.html` + `design-system.md` completo | pendiente |
| 3 | `sesion.html` · `permisos.html` · `honestidad.html` | pendiente |
| 4 | `corpus.html` · `notas.html` · `idioma.html` · `ia.html` | pendiente |
| 5 = G-Diseño | `index.html` + todo | pendiente |

Regla: «continúa» no aprueba diseño; cada mirada se registra en `docs/diseno/README.md`
ANTES del siguiente artefacto. Cambios al plan de miradas se aprueban antes de construir.

## Decisiones de diseño que la orden no escribió (D1–D13, del plan aprobado)

| # | Decisión |
|---|---|
| D1 | Panel flotante 380×220 (compacto) · 380×420 con transcript · hasta 380×360 con 2–3 fichas; nunca más ancho |
| D2 | Posición por defecto: esquina superior derecha, 16 px del borde, 8 px bajo la barra de menús; todos los Spaces; posición recordada |
| D3 | Opaco, sin blur ni «glass» — cuaderno, no overlay |
| D4 | Tipografía nativa de macOS sin embeber bytes: Charter (evidencia) · Avenir Next (interfaz) · Menlo (cifras, fuentes, contador; `tabular-nums`) |
| D5 | Panel: titular 17 px · línea 14 px · fuente/estado 12 px (Menlo 11.5 px) |
| D6 | Ventana principal 960×640 (mín. 800×560), rail izquierdo fijo |
| D7 | Personalidad: discreto · propio · verificable / nunca ruidoso, nunca «stealth» (vocabulario:cita), nunca corporativo |
| D8 | Oscuro primario grafito cálido + tinta marfil; claro = papel; un solo acento frío «halo»; estados ok=verde-azulado · atención=ámbar · error=coral · inactivo=gris; jamás rojo vs. verde como única distinción |
| D9 | Iconografía SVG de trazo 16 px monocromo, inline; cero emojis |
| D10 | Motion casi nulo: fade 150 ms sin desplazamiento; «buscando…» estático; reduced-motion ⇒ cero transiciones; la forma del árbol nunca depende del motion |
| D11 | Bilingüe en la maqueta: `<span lang>` pareados, conmutador es/en, `lang` del documento cambia |
| D12 | Datos sintéticos: «Comercializadora Páramo Azul S.A.S.» · «Tablero de rentabilidad por canal en Power BI» · «Adopción de datos en 4 etapas» · «Cooperativa Sur del Valle» · perfil genérico; jurisdicciones Colombia · Florida · UE |
| D13 | `docs/diseno/{index,panel,sesion,permisos,corpus,notas,honestidad,idioma,ia,kit}.html` + `assets/ghost.css` + `assets/maqueta.css` + `assets/maqueta.js` |

## Fase 0 — Setup (2026-09-20)

- Branch `diseno/fundacion` desde `main` (13cc7e5).
- Carpetas `docs/diseno/assets/` y `sprints/`.
- **Arnés de capturas** en el scratchpad de la sesión (`capturas.mjs`, NO versionado): abre cada
  `docs/diseno/*.html` por `file://`, recorre estado × tema × idioma según los botones de la
  barra de maqueta, captura a tamaño real y mide contraste WCAG por nodo de texto. Declara al
  arrancar el árbol que lee y aborta si se le pasa una ruta fuera de `docs/diseno/` (regla 17-bis b).
- **Gates nuevos, demo en rojo en el MISMO commit (regla 15):**

| Gate | Puede fallar (3ª pregunta) | Demo en rojo | A quién nombró | Verde al revertir |
|---|---|---|---|---|
| `tests/unit/vocabulario-vetado.test.ts` | Sí: eslint solo mira `src/`; nada más lee `docs/` | plantado «indetectable» (vocabulario:cita) en `docs/diseno/panel.html` | `docs/diseno/panel.html:3  «indetectable»` | ✓ 7/7 |
| `tests/unit/maqueta-autocontenida.test.ts` | Sí: ninguna regla previa inspecciona recursos externos en HTML | plantado `<script src="https://cdn…">` (autocontenida:cita) en `panel.html` | `docs/diseno/panel.html:2  URL absoluta` + `script externo` | ✓ 7/7 |

  Ambos corren en el job `quality` vía `pnpm test` (vitest incluye `tests/unit/**`).
- **Arnés de capturas — su propia demo en rojo:** un HTML temporal con texto `#333` sobre `#111`
  lo puso en rojo (`p «texto apagado» 1.49 < 4.5`) en los 4 combos tema × idioma; el HTML se
  borró. El abort fuera del árbol también se vio: `ABORTO: …/MANUAL-DE-USO.md no está dentro
  de …/docs/diseno`. Requirió `pnpm exec playwright install chromium` (1.63 pide el
  headless-shell 1243; la máquina tenía 1228/1234).
- Verificación local: `pnpm typecheck` ✓ · `pnpm lint` ✓ · `pnpm test` 3 archivos / 7 tests ✓.
- `docs/diseno/panel.html` existe como placeholder vacío para que los gates tengan qué leer
  (un gate que no lee nada no es un gate — el test lo exige).

## Fase 1 — El panel ⭐
(pendiente)
