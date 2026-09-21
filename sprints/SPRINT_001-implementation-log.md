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
(pendiente)
