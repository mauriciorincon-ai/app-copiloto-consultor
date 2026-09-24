# Angel Ghost (app-copiloto-consultor) — constitución de la app (Claude Code)

> Auto-cargado en cada sesión de este repo. Esta app pertenece al pipeline **AI-APPs**; su plan
> vive en la casa planeadora. Estampada desde kit-app **v1.27.0 con `--escritorio`** el
> 2026-09-18. **Primera app de ESCRITORIO del portafolio** (Tauri) y primera bajo el estándar
> 4-T «captura de terceros». Nace con el pipeline completo desde el día 0 (Etapa de Diseño ·
> dos filtros ⭐/⭐⭐ · cierre en dos actos · cero enlaces).

## Las dos casas (regla dura)

| Casa | Path | Escritor único | Qué vive ahí |
|---|---|---|---|
| **Planeadora** | `~/Code/hr01-develop-ai-apps/` | su propia sesión | brief, VISION, sprints (plan+retro), órdenes, método, estándares |
| **Esta app** | este repo | **tú** | código, tests, ADRs de implementación, bitácora y summary del sprint |

- ✅ Puedes **leer** la planeadora (conexión fija vía `.claude/settings.local.json`, o por path absoluto).
- ❌ **Nunca escribes** en la planeadora. Si el plan necesita cambio, lo anotas en tu
  `sprints/SPRINT_NNN-implementation-log.md` bajo `## Desviación del plan` y avisas al usuario.
- El avance de implementación vive **solo aquí** — la planeadora te lee, tú no le reportas a mano.

## Qué es esta app

**Angel Ghost** — *«Tu propia experiencia, en la reunión, en el momento justo — y solo tú la
ves.»* Una ventana pequeña en el Mac del consultor que el cliente NO ve aunque comparta pantalla.
Escucha ambos lados de la videollamada (dos pistas: micrófono = consultor, audio del sistema =
cliente), lee la pantalla solo cuando cambia, y muestra **fichas de evidencia del propio corpus
del consultor** (propuesta · marco · caso · ficha de cliente · perfil del consultor): un titular,
una línea y la fuente. Si el usuario lo enciende, un modelo local redacta una sugerencia breve.
**Silenciosa y efímera:** nada se graba ni persiste; al cerrar quedan solo las notas escritas.
Bilingüe español/inglés en todo. Contrato de alcance: `portafolio/copiloto-consultor/VISION.md`
(planeadora, aprobada 2026-09-18, v1.2.0 el 2026-09-19 — 25 funcionalidades: 17 MVP personal · 2 MVP terceros · 6 roadmap).

**La tesis del producto:** el valor está en recuperar la evidencia PROPIA en el momento justo sin
romper el flujo (Salesforce: 2,8 s vs. 25–65 s manuales; CHI/CSCW 2025: la ayuda pasiva e
intermedia preserva el juicio del experto). La categoría «invisible» vende ocultarse; esta app
vende **no persistir, verificable**.

## ⚠️ Reglas duras de esta app (producto, no estilo)

1. **EFÍMERO VERIFICABLE (estándar 4-T).** Audio, transcript y frames de pantalla viven SOLO en
   memoria (ring buffers) y mueren al cerrar la sesión: ni disco, ni temporales, ni swap de
   buffers, ni logs, ni Sentry. Se blinda por capas y DEMOSTRABLE: **(a)** lint que prohíbe API
   de disco y de red en `src-tauri/src/capture/`, `stt/`, `screen/`; **(b)** test de **fuga
   inyectada** (un `fs::write` plantado hace fallar la suite — se demuestra en rojo, regla 15);
   **(c)** `pnpm verify:ephemeral`: tras una sesión completa, cero archivos nuevos fuera de la
   carpeta de notas; **(d)** término plantado en logs; **(e)** kill-switch de una tecla que corta
   captura y vacía buffers. **Qué persiste y qué no (mirada 3 de la Etapa de Diseño, 2026-09-20):**
   el diseño distingue **lo del usuario** de **lo de terceros**, no «texto» de «audio». Persiste,
   cifrado, con retención y borrado, en la carpeta del usuario: notas y acuerdos escritos · fichas
   mostradas y fijadas · **turnos del propio consultor (pista de micrófono) en TEXTO, opt-in, por
   defecto apagado** · la **bandeja de propuestas** durante la ventana elegida (defecto 3 h, techo
   24 h, mínimo cero; borrado automático al vencer aunque la app no se abra; visible en Honestidad)
   · el índice del corpus (documentos propios, en claro) · preferencias y metadatos de costo. Muere
   SIEMPRE, sin conmutador que lo encienda: audio de cualquier pista (la propia incluida),
   transcript del cliente, lo leído de la pantalla. El consultor responde por su propia carpeta.
2. **NADA CRUDO SALE DEL EQUIPO.** Ningún audio ni imagen viaja jamás a un proveedor. Por
   defecto la app es **100 % local** (STT, OCR, retrieval y síntesis on-device); el API externo
   está APAGADO hasta que el usuario lo encienda con su propia clave, y aun así solo recibe texto
   minimizado y anonimizado localmente (patrón Velo) bajo proveedor con no-retención. **Contador
   de salida a red** visible por reunión (0 en modo local) con test.
3. **CÓDIGO PRIMERO.** Captura, VAD, fin de turno, STT, OCR, disparo y recuperación son
   deterministas. La única feature LLM (síntesis de sugerencia) lleva **ADR «código primero»**
   (plantilla en `decisions/PLANTILLA-ADR-codigo-primero.md`) y su fallback permanente son las
   fichas del corpus sin LLM. Orden de proveedores: Apple Foundation Models → MLX → API opt-in;
   adapter con `mock` como proveedor de primera clase.
4. **CERO HUELLAS DE VOZ, CERO EMOCIONES.** La atribución de hablante se resuelve por pista
   (mic/sistema), nunca por biometría; no se infiere estado emocional de nadie.
5. **CERO DATOS REALES DE CLIENTES EN EL REPO.** Público por defecto: kits de prueba y maquetas
   con datos 100 % sintéticos; el corpus real del usuario y sus notas viven fuera del repo y
   gitignored. Doble cinturón gitleaks.
6. **VOCABULARIO.** Ningún copy, README, brochure ni ficha dice «indetectable», «trampa»,
   «engañar» ni equivalentes (barrido en lint/test). La app protege una ventana con un flag del
   sistema; no evade software de detección. Promesa de invisibilidad **graduada**: solo lo que el
   spike del usuario verificó por cliente (Zoom · Meet · Teams) y versión de macOS.
7. **BILINGÜE es/en** en interfaz, transcripción, corpus, fichas, sugerencias y
   `NS*UsageDescription`.
8. **Daltonismo leve del usuario:** todo estado = símbolo + texto + color; tema oscuro primario,
   claro obligatorio.
9. **CERO ACCESO A MÁQUINAS AJENAS.** La app jamás sondea, escanea ni actúa sobre el computador
   de la contraparte (sería acceso abusivo, art. 269 CP, y el espejo de lo que rechazamos). El
   **Radar de captura (C14)** observa SOLO el Mac y la pantalla del consultor: banner de
   grabación, bots de notas en la lista de participantes, agentes de monitoreo/proctoring locales;
   catálogos versionados con fuente; alerta al usuario, no «expone» a nadie. Lint que prohíbe
   sockets salientes fuera del adapter LLM (que además nace apagado).

## Stack

- **Perfil ESCRITORIO (kit v1.27.0):** **Tauri 2** (Rust) + webview **React + TypeScript strict +
  Tailwind** (Vite). Distribución como binario firmado/notarizado; **sin Vercel, sin Supabase, sin
  backend** (local-first; backend solo por ADR).
  captura_terceros: true
- **Nativo (Rust, `src-tauri/`):** audio en dos pistas (Core Audio process taps 14.4+ /
  ScreenCaptureKit audio 13+ + micrófono), ventana con `content_protected`, permisos TCC,
  captura de pantalla + pHash, kill-switch. `tracing` metadata-only.
- **STT / VAD / EOT:** Apple SpeechAnalyzer (macOS 26+) con respaldo Parakeet-TDT v3 / Whisper
  turbo (WhisperKit); Silero VAD; fin de turno determinista (prosodia + acústica).
- **Pantalla:** Apple Vision OCR (`es-ES`/`en-US`) solo ante cambio (dHash/pHash).
- **Corpus:** índice local BM25 + embedding multilingüe ≤600M + RRF; chunking por sección.
- **IA embebida:** adapter multi-proveedor conmutable por env (on-device → MLX → Claude API /
  Gemini / Groq / Azure) + `mock`; esquema Zod; prompt caching; costo por reunión (skill `ia-embebida`).
- **Tests:** Vitest (unit) + Playwright (e2e de la webview vía `pnpm preview`) + axe + `cargo test`.
- **CI:** `quality` · `e2e` · `build-escritorio` (macOS: cargo check/test + gate `verify:ephemeral`).
  Checklist pre-merge: **`/release-check`** (no `/deploy-check`).
- **Idioma de la UI:** español e inglés desde el S1.

## Adaptaciones del perfil escritorio (declaradas en el estampado — no son desviaciones)

- No hay Vercel ni previews: **G-Diseño se aprueba sobre la maqueta abierta en LOCAL** (doble
  clic en `docs/diseno/index.html`); los gates de fidelidad visual se corren con `pnpm tauri dev`.
- No hay Lighthouse: el presupuesto es de **latencia** (fin de turno → ficha ≤4 s, medida en el
  kit de prueba) y de **peso del binario** (anotado por PR).
- `src/lib/observability.ts` del kit no viaja (importa Sentry para Next): la observabilidad
  entra por ADR (tracing en Rust + logger metadata-only en la UI).
- Ruleset `main-protegida` con checks requeridos `quality` · `e2e` · `build-escritorio`.

## Estructura

```
src/
├─ app/            (App Router)
├─ components/     (UI sin lógica de negocio)
├─ engine/         (motores puros, sin side-effects, cobertura >80%)
├─ lib/            (utils, dominio)
│  └─ ia/          (patrón IA-embebida: schemas.ts · client.ts · guardrails.ts · persist.ts)
└─ types/
tests/{unit,integration,e2e}/
design-system.md          (fuente de verdad visual — se crea en el sprint 1, skill diseno-ui)
design-sync/              (bundle publicable a Claude Design — VERSIONADO; project.json + styles.css
                           + components/<grupo>/<tarjeta>.html. Nace en el primer sprint que publique)
docs/MANUAL-DE-USO.md     (manual de uso general — OBLIGATORIO, vivo desde el sprint 1)
sprints/SPRINT_NNN-implementation-log.md · SPRINT_NNN-summary.md
decisions/NNN-titulo.md   (ADRs de implementación)
```

## Reglas de desarrollo

1. **TypeScript strict.** Sin `any` ni `@ts-ignore` sin justificación en comentario.
2. **Tests con cada feature.** Motores puros >80%, UI >50%, ≥1 e2e por feature core.
   **Al escribir los PRIMEROS tests (S1): añade `--coverage` al script `test`** — sin el flag los
   umbrales del `vitest.config.ts` no se aplican en CI (el estampado lo omite para que la CI del
   commit inicial quede verde sin tests). Directorios **generados** (`coverage/`, assets copiados
   a `public/` tipo `public/pyodide/`) van a los `globalIgnores` de `eslint.config.mjs`.
3. **Motor separado de UI.** Lógica pura en `engine/`/`lib/`; componentes sin lógica de negocio.
4. **Toda salida de LLM que se persista pasa por esquema Zod** (skill `ia-embebida`) — nunca texto
   libre directo a la BD.
5. **A11y desde el inicio:** tabindex, aria-labels, contraste AA, `prefers-reduced-motion`.
   **Y dos reglas que nacen de reincidencias (kit v1.26.0):** (a) **la FORMA del árbol jamás
   depende de `useReducedMotion()`** — el hook vale `null` en el servidor y `true` en el
   navegador con «reducir movimiento»; si decide QUÉ elementos se pintan, el HTML del servidor
   y el primer render del cliente no coinciden (React #418) y la página se regenera entera
   justo para quien el cinturón quiere cuidar. Reduced motion cambia PROPIEDADES (initial,
   variants, transition) o lo hace el CSS; dos gates: test unitario «mismo HTML con `null` /
   `true` / `false`» sobre cada componente de motion + axe bajo emulación de reduced-motion
   (patrón `wiki/patterns/reduced-motion-sin-ramificar-el-arbol.md` de la planeadora).
   (b) **Los tokens de tinta VETADOS como texto se declaran en `design-system.md` y FALLAN en
   lint/test, no en axe al final** (p. ej. `ink-3`, que no alcanza AA sobre superficie): un
   barrido de clases prohibidas sobre `src/` que corre con `pnpm lint`/`pnpm test` — la
   segunda reincidencia en una misma app (hoja-de-vida S6 y post-S7) fue la señal.
6. **Commits convencionales**; branch `sprint-NNN/<tema>`; **jamás push directo a `main`** (hook lo
   bloquea); PR con CI verde + preview probado. La ruleset `main-protegida` exige los checks
   `quality`/`e2e`/`lighthouse` **desde el estampado** (regla 2026-07-10 — protección GitHub no
   negociable, repo público); **si un sprint añade un job de CI (p. ej. `integration`), se añade
   a la ruleset en el mismo sprint** (`gh api` o Settings → Rules).
7. **Secrets solo en `.env.local` (gitignored) y Vercel env vars.** Doble protección gitleaks:
   hook `pre-commit` de git (`githooks/`, cubre commits manuales) + hook PreToolUse de Claude
   Code (cubre escrituras del agente). El hook nace ejecutable (100755) y `core.hooksPath` se
   re-aplica en cada `pnpm install` (script `prepare` — K12); si un commit con secreto de prueba
   NO es bloqueado, el gate está muerto — repáralo antes de seguir. **Carnada canónica verificada
   (kit v1.6.3; desde v1.7.3 viaja PARTIDA aquí para no disparar el hook al comitear este
   archivo): ármala concatenando `AWS_ACCESS_KEY_ID=` + `AKIAQ7RTZ4PX` + `KM2WNB3S` SOLO en el
   archivo de prueba del hook** — no improvises el secreto de prueba:
   las reglas modernas de gitleaks exigen alfabeto real (base32 tras `AKIA`) y entropía, y una
   carnada floja pasa en silencio dando falsa tranquilidad (lección 2026-07-15: dos falsos "todo
   bien" seguidos). Si gitleaks sube de versión mayor, re-verificar la carnada en sandbox antes
   de confiar en ella.
8. **Presupuesto de esfuerzo:** ~12 pasos por pantalla; si lo excedes, detente y simplifica o consulta.
9. **Manual de uso vivo (`docs/MANUAL-DE-USO.md`, obligatorio).** Toda feature que llegue a `main`
   queda documentada ahí **en el mismo sprint**: qué hace, cómo se usa (pasos para el usuario final,
   no para el dev), capturas o rutas de pantalla, y limitaciones conocidas. En español llano. Es el
   documento que permite a cualquier persona conocer las features principales de la app sin leer
   código — al lanzar (F5) se convierte en la base de la guía de usuario pública.
10. **El diseño va ANTES del código — Etapa de Diseño con gate G-Diseño (kit v1.14.0, método
   v1.14.0 F2a).** Si este repo acaba de estamparse: **tu primer trabajo NO es construir — es
   diseñar**. La planeadora emite una **orden de diseño** (`ordenes/DISENO-orden.md`); en branch
   `diseno/fundacion` produces (1) el **`design-system.md` completo** (tokens ambos temas,
   personalidad, componentes canon, motion + reduced-motion, anti-patrones) y (2) la **maqueta
   navegable del H1 COMPLETO** en `docs/diseno/` — HTML autocontenido, una página por pantalla
   core con sus estados, mobile + desktop, ambos temas, **cero React y cero motores** — que se
   despliega en Vercel para que el usuario la recorra en sus dispositivos. Se trabaja en **sala
   de diseño** (rondas de propuesta → mirada del usuario → ajuste, sin presupuesto de pasos ni
   prisa; pasada de capturas desde la ronda 1). **CERO código de producto hasta que el usuario
   apruebe G-Diseño** (veredicto registrado en `docs/diseno/README.md`). Desde entonces, toda
   pantalla de producto **obedece la maqueta**: en el primer sprint con UI, DETENTE tras la
   primera pantalla construida y presenta capturas comparadas contra la maqueta (gate de
   FIDELIDAD, indiferible — no viaja con el ⭐). Claude Design sigue bajo demanda (convergencia
   trabada ≥2 rondas o exploración del usuario); `/design-sync` publica el sistema consolidado
   al cerrar cada ciclo. *(Apps estampadas ANTES de v1.14.0 y sin etapa: el gate del primer
   sprint con UI es de DIRECCIÓN — design-system + primera pantalla en capturas — igual de
   indiferible.)* Origen: Velo llegó a su S2 sin que el usuario viera un solo artefacto visual. Cada sprint con UI cierra con el **checklist de revisión de
   diseño** del skill `diseno-ui` + aprobación visual del usuario sobre la preview.
   **Claude Design — LA regla única (método v1.19.0, resuelve la contradicción que Velo S4
   destapó):** **durante el ciclo es BAJO DEMANDA** con sus dos disparadores escritos (el gate
   visual no converge · el usuario pide explorar pantallas) — jamás se crea proyecto por defecto
   al arrancar. **Al CERRAR el ciclo, la publicación del consolidado es OBLIGATORIA** con su
   razón declarada: costo en minutos (bundle `design-sync/` versionado, publicación incremental,
   destino en `project.json`), activo estable entre ciclos y base de toda exploración futura —
   **y SIEMPRE después del gate ⭐⭐ corto** (que incluye el juicio de diseño): jamás se publica
   un sistema que el usuario no ha juzgado. La invocación es del usuario (§ Cierre de CICLO); si
   decide no invocarla, el summary lo registra como decisión suya. Las órdenes CITAN esta regla,
   no la re-redactan.
   **El gate de MIRADA (kit v1.20.0, método v1.21.0 — para TODO artefacto visual, en la etapa
   de diseño Y en los sprints):** el gate de FASE es de proceso y se pasa con «continúa»; el
   gate de MIRADA es otro gate y **solo se pasa con evidencia de que el usuario VIO**: un
   comentario que delate el archivo abierto, o su **«lo abrí y apruebo»** textual.
   **«Continúa» JAMÁS aprueba diseño.** Mecánica obligatoria del mensaje — claridad ante todo:
   la **PRIMERA línea** es una pregunta simple en español llano + el lugar
   («**¿Apruebas el design system? Ábrelo aquí: docs/diseno/kit.html (doble clic)**») — sin
   jerga del método, sin códigos; el resumen técnico va DESPUÉS. Si el usuario responde solo
   la palabra de fase sin comentar el artefacto: **DETENTE y repregunta «¿qué viste al
   abrirlo?»** antes de construir encima — esa negativa es la demo en rojo de este gate.
   AskUserQuestion/previews ASCII **no sustituyen la mirada**: si preguntas por chat sobre un
   artefacto visual, pide la respuesta con el archivo abierto y dilo. Cada mirada queda
   **registrada** (README de diseño o bitácora) ANTES de la construcción siguiente — la
   planeadora lo audita en G-Diseño y al cierre; sin registro, el cierre queda condicionado.
   **Y el PLAN de miradas —número, agrupación y ORDEN— es parte del gate (kit v1.21.0):**
   cualquier cambio (agrupar, reordenar, posponer) se propone y el usuario lo aprueba ANTES de
   construir el segundo artefacto — jamás sobre la marcha. Cada desvío reduce las oportunidades
   de ver (dash S1: B6 sobrevivió a la mirada agrupada; dash S2: las 3 pantallas se
   construyeron antes de la primera mirada — «ninguna pidió ajustes» fue suerte, no proceso).
   *(Origen: Dash Agent AI, Etapa de Diseño 2026-08-16 — 4/9 pantallas construidas sin una
   mirada real con todos los gates de palabra cumplidos; 3ª ocurrencia de la clase. Es la
   regla 15-hermana del lado humano: una mirada satisfecha sin mirada es un gate que nunca
   ejecutó.)*
11. **Guía de prueba viva y ACUMULATIVA (`docs/GUIA-DE-PRUEBA.html`, OBLIGATORIA en todo sprint
   con UI — reglas duras del pipeline, G-Metodo 2026-07-12 ×2).** HTML visual y **AUTOCONTENIDO**
   (cero CDNs; casillas con `localStorage` bajo **prefijo versionado por sprint** — cambia en
   cada versión de la guía para que una regresión sin correr jamás aparezca marcada por el sprint
   anterior): **qué probar, cómo y qué resultado esperar**, por bloques. **Es bola de nieve:** la
   última versión contiene **TODAS las pruebas vigentes** de la app; el sprint N hereda ENTERAS
   las del N−1 — no las resume ni las comprime en un "verificar que sigue funcionando" (comprimir
   borra la regresión). Cada prueba lleva su **origen visible en su línea**: `Nuevo · SN` ·
   `Mejorado en SN` (hay que volver a mirarla) · `SN` a secas (heredada sin cambios ⇒ regresión),
   con **filtros por origen**. Una prueba solo se elimina cuando su feature dejó de existir, y se
   declara en el historial del pie. Y trae **DOS filtros de gate desde el sprint 1 (kit
   v1.18.0)**:
   - **⭐ Gate mínimo** — SOLO lo que ninguna automatización puede verificar (hardware/micrófono/
     voz reales, juicio humano sobre contenido, aprobación visual) MÁS la re-verificación de
     confianza que el usuario quiera hacer con sus manos aunque el CI ya la cubra. **Se OFRECE.**
   - **⭐⭐ Gate corto** — el subconjunto cuyo veredicto **solo puede ser humano**, con techo
     declarado (**~20 min**). Regla de selección: **si el CI lo verifica por otro camino, NO
     entra** — aunque viva en el ⭐ por buenas razones de confianza. Es un **recorrido caminable**
     (paradas «N de M» en el orden del documento, comprobando que la secuencia se camina),
     **declara cuántas ⭐ deja fuera y por qué** (ninguna se borra — un gate que se encoge sin
     decirlo se lee como «esto es todo lo que había»), y no mueve los conteos de los otros
     filtros: añade una lente, no reparte. **Es el que el cierre de ciclo EXIGE.** *(Origen: Velo
     llegó al cierre con 26 ⭐ / 90 min y el gate se aplazó indefinidamente — no falló el gate,
     falló su tamaño; con varias apps en curso, todo-o-nada pierde contra nada.)*
   La guía dice cuántas pruebas son y cuánto toman, en ambos gates. **Kit de prueba:** si un paso —o
   la app misma— requiere documento, código o dataset de prueba, se entrega en el repo
   (`docs/kit-de-prueba/`) enlazado desde su bloque, siempre que se pueda (precedente: los
   datasets de ds). Doble propósito: gate de prueba del usuario + entregable a usuarios finales.
   Plantilla base: la que estampa el kit (v1.6.0). **Implementación de referencia:
   `app-habla/docs/GUIA-DE-PRUEBA.html` (S2)** — 81 chips de origen, gate mínimo ⭐ de 14
   pruebas/~25 min con filtro, namespace versionado, historial de eliminaciones.
12. **PROHIBIDO entregar por artifacts de Claude o cualquier plataforma externa** (regla dura del
   pipeline, G-Metodo 2026-07-12). **Todo entregable** —guías, reportes, documentos visuales,
   resúmenes— es un **ARCHIVO DEL REPO** (HTML autocontenido o Markdown) que el usuario pueda
   **abrir, versionar y llevarse**. Sin excepciones, ni "para verlo rápido". Si algo merece
   mostrarse visualmente, se escribe como archivo y se entrega su ruta.
13. **Brochure vivo (`docs/BROCHURE.html` + ruta pública `/conoce` — molde v2, kit v1.10.0,
   informe del piloto habla 2026-08-08).** El entregable de PRESENTACIÓN de la app para
   usuarios finales y clientes — el anti-manual. **Tiene DOS estados (kit v1.19.0, método
   v1.20.0):** nace como **BROCHURE INICIAL** en el cierre DE CONSTRUCCIÓN del ciclo (declara
   visiblemente que es inicial y que se sella con las pruebas) y pasa a **BROCHURE SELLADO
   (MVP)** en el cierre DE PRUEBAS (tras el gate ⭐⭐ + correcciones). El sello NO lo congela:
   **todo sprint posterior que cambie features lo ajusta EN EL MISMO SPRINT** (DoD). Y junto al
   HTML se produce el **export estructurado `docs/brochure-export.json`** (schema versionado:
   tagline, intro, funcionalidades con el conteo del MANUAL, métricas reales, stack) — es lo que
   la vitrina de hoja-de-vida consume, anclado a versión, para re-expresarlo en SU design system.
   **REGLA CERO: el brochure NO se estampa, se
   PRODUCE** — antes de una línea de HTML, un **storyboard aprobado por el usuario** («guion
   aprobado»): escenas con mensaje/gramática/técnica justificada, **clímax explícito** (la
   promesa mayor de la app tiene escena propia, JAMÁS un acordeón del pie), ritmo, variante
   reduced-motion POR escena, dial `MOTION_INTENSITY` fijado con el usuario, identidad en una
   frase y un riesgo registrado. Las gramáticas se ejecutan con el **banco de técnicas en
   vanilla** (`docs/BROCHURE-banco-de-tecnicas.md` — sin recetas, "G3" degenera en fundido:
   el piloto lo demostró con un rechazo). Estructura de 4 capas + regla de conteo con **tabla
   de mapeo en el summary** (feature → sección del manual → tarjeta; el brochure no se
   documenta a sí mismo). Solo `transform`/`opacity` (excepciones DECLARADAS en storyboard y
   código) + presupuesto de recorrido móvil (~1 pantalla extra). A11y estructural:
   `<h3><button>` canónico · lo cerrado FUERA del árbol de accesibilidad (`visibility` en la
   transición; axe no lo ve — e2e por CDP) · LCP jamás nace de `opacity: 0`. **Gates que la
   CI no ve** (en el piloto la CI estuvo VERDE con un entregable RECHAZADO): pasada de
   capturas por bloque leídas como imagen (cuadro a cuadro en animaciones) antes de presentar
   · sala de proyección como proceso (fidelidad al producto real es criterio) · **e2e
   obligatorio de reduced-motion** (visibilidad real de elementos clave) · lo medible se
   corrige MIDIENDO contra la versión anterior · **última milla: el link de producción se
   prueba SIN sesión** y dominio + protección de deployment van al BLUEPRINT. Doble vida
   (ruta + archivo), vivo por sprint, iconos y señas del design system de la app (jamás
   emojis si el DS los prohíbe), cero datos personales. Fuentes: MANUAL-DE-USO + VISION (RO)
   + guías de prueba. Referencia: `app-habla/docs/BROCHURE.html` + su storyboard.
14. **Código primero, IA generativa después (regla dura del pipeline, G-Metodo 2026-07-12).**
   Esta app es una integración sólida entre IA y código, pero **toda funcionalidad nativa
   interna se resuelve PRIMERO con programación** — código, librerías, algoritmos deterministas —
   **antes de cualquier intención de acudir a IA generativa** (APIs o cualquier tipo de
   conexión). Activar una feature LLM exige un **ADR "código primero"** que justifique por qué
   el código y las librerías no alcanzan. La IA es acento con fallback determinista, jamás
   columna vertebral.
15. **Un gate se demuestra FALLANDO (regla dura del pipeline, G-Metodo 2026-08-10).** Todo gate
   nuevo que este repo agregue —job de CI, hook, aserción, umbral, script de verificación— nace
   con su **demo**: un cambio deliberado que lo pone en **rojo**, con el resultado (rojo → verde
   al revertir, y a quién nombró el fallo) registrado en
   `sprints/SPRINT_NNN-implementation-log.md`. Cuesta cinco minutos y es **la única evidencia
   real de que el gate funciona**: un gate que nunca se vio fallar no es un gate, es decorado —
   y decorado que da falsa tranquilidad (precedente: dos "todo bien" seguidos de una carnada
   floja de gitleaks, 2026-07-15; contraprecedente que sí lo hizo: PR desechable con `openai`
   → anti-IA en rojo en 7 s, Velo S1). La demo puede ir en un **PR desechable** que se cierra
   sin mergear; se registra igual. Aplica también al **verificar un gate heredado** cuando un
   sprint depende de él por primera vez.
   **Y su hermana (kit v1.16.0): un gate que nunca EJECUTÓ tampoco es un gate.** `skipped` no es
   verde: un job con `needs:` sobre otro que falló queda saltado y GitHub lo lista entre los
   checks requeridos **sin alarma**, así que una columna sin rojo se lee como aprobación. Antes de
   cerrar, **cada check requerido debe tener conclusión propia `success`** (`gh pr checks`), y si
   uno corrió por primera vez en este PR se dice en el summary — sin histórico **no puede
   afirmarse ni regresión ni no-regresión**. Las dos reglas cubren la misma ilusión por lados
   opuestos: *¿lo viste **fallar** cuando debía?* y *¿lo viste **correr**, alguna vez?*.
   **Y el tercer filo (kit v1.21.0): ¿lo viste correr EN EL MODO en que el usuario lo va a
   usar?** Todo modo/perfil de arranque (p. ej. `start:seguro`) corre EN VIVO al menos una vez
   antes de cerrar el sprint que lo introduce o lo toca — pasar sus tests no es haber corrido
   *(dash S2: el modo endurecido existió dos sprints sin arrancar jamás de verdad; y el índice
   world-readable solo apareció inspeccionando el proceso vivo)*. Detalle
   operativo en `/deploy-check` §11. *(Origen: ds S4 — el job `lighthouse` estuvo `skipped` las 12
   corridas de la rama; el gate de performance no corrió ni una vez en un ciclo de 4 sprints
   mientras el DoD lo daba por verde con corridas locales.)*
   **Y el rojo nace en el MISMO commit que introduce el gate (kit v1.25.0), no al final de la
   fase:** una demo diferida deja al gate viviendo «verde» sin haber medido nada, y todo lo
   construido mientras tanto se apoyó en él *(hoja-de-vida S5: el test de deriva cero pasaba en
   verde con el bug puesto — afirmaba «se llega al fondo», que el defecto también cumplía; solo
   la demo en rojo, exigida al cierre de la fase, reveló la prueba decorativa)*. La demo es parte
   de la definición del gate, no un trámite posterior.
   **Y la tercera pregunta (kit v1.26.0): ¿puede este gate FALLAR siquiera?** Antes de
   escribirlo, comprueba que existe un estado del repo que lo pondría en rojo y que ninguna
   regla anterior lo hace inalcanzable (un schema que ya rechaza el caso, un test que ya lo
   cubre, un build que ya rompe antes). Si no puede fallar, no es un gate: se retira y se
   anota cuál regla lo cubría *(hoja-de-vida S7: un gate nuevo resultó inalcanzable por una
   regla previa y solo se descubrió al exigirle el rojo)*. Las tres preguntas, juntas: ¿lo
   viste **fallar**? · ¿lo viste **correr**? · ¿**puede** fallar?
16. **El bundle publicable del design system es un ARTEFACTO DEL REPO (kit v1.17.0).** `design-sync/`
   se versiona aquí como **espejo 1:1** de lo publicado en Claude Design, y la jerarquía es fija:
   `design-system.md` (fuente de verdad) → `design-sync/` (bundle, deriva) → el proyecto remoto
   (vitrina, **jamás se edita allá**). **Todo sprint que toque UI actualiza el bundle en su MISMO
   PR** — son archivos del repo, entran a la revisión y los escanea gitleaks como a todo lo demás;
   publicar puede esperar al cierre de ciclo, y así el cierre es un delta pequeño y nunca una
   reconstrucción. El destino se **lee** de `design-sync/project.json`, no se busca. Procedimiento
   completo: `/design-sync`. *(Origen: en el cierre H1 de ds el bundle se construyó en el
   scratchpad efímero; al retomar días después quedaban **4 de 13 archivos** y hubo que
   reconstruirlo bajando del proyecto remoto uno por uno. Un entregable de ciclo que vive fuera del
   repo no tiene versión, ni diff, ni revisión, ni supervivencia.)*

17-bis. **LO QUE LA APP ESCRIBE TAMBIÉN ES SUPERFICIE (kit v1.21.0 — derivados y arneses).**
   Las capas de solo-lectura protegen a las fuentes DE LA APP; esta regla protege a los
   DERIVADOS de todo lo demás. **(a) Un derivado JAMÁS nace menos privado que su fuente:**
   índice, cache, config, reporte o log que descienda de datos privados nace con permisos
   restrictivos (700/600 o equivalente), con test que se demostró en rojo y reparación al
   abrir si ya existía mal *(origen: dash S2 Correctivo 001 — el índice nacía 644 con prompts
   reales; 531 tests verdes no lo vieron)*. **(b) Todo arnés que pueda tocar fuentes arranca
   demostrando contra qué árbol corre** (capturas, fixtures, seeds): declara su árbol al
   arrancar y ABORTA si alcanza datos reales sin confirmación explícita — comprobar, no
   recordar *(origen: un server viejo en el puerto hizo que la pasada de capturas fotografiara
   transcripts reales)*. `/deploy-check` §12 verifica ambas.

17. **CERO ENLACES: la producción se MUESTRA, jamás se ENTREGA (regla dura del pipeline, F0 #8
   2026-08-15 — kit v1.19.0).** Ningún archivo de este repo público ni campo de GitHub publica la
   URL de producción o de previews: ni el `README.md` (apunta al brochure/`/conoce` como
   CONTENIDO, jamás como link de acceso publicado), ni el campo About/website del repo, ni el
   `BLUEPRINT.html` (documenta dominio y protección como "qué ve quién sin sesión" **sin escribir
   la URL** — la URL exacta vive en la planeadora, que es privada), ni el manual, ni la guía
   (su campo de URL se llena EN USO, desde la orden), ni `package.json`. El CTA público de la app
   es la **«lista de espera»** — sin promesa de otorgamiento. **La limpieza del campo homepage
   es RECURRENTE, no de una vez (kit v1.22.0):** la GitHub App de Vercel lo reescribe tras cada
   deploy de producción (confirmado en vivo) — se re-verifica tras CADA merge a `main`, y JAMÁS
   se automatiza con un PAT de administración como secret en un repo público. Y **los documentos
   que NARRAN el barrido escriben los patrones sin el literal** (clase de carácter, p. ej.
   `vercel[.]app`): un summary que cita el patrón tal cual rompe el grep y el gate deja de ser
   binario. **El comando del barrido corre sobre TODOS los archivos versionados** (kit
   v1.23.0): `git grep -nE "vercel[.]app|workers[.]dev|pages[.]dev" -- ':!pnpm-lock.yaml'` —
   jamás con include-list de extensiones (la URL de producción de Innmobiliaria vivía en
   `wrangler.jsonc` y pasó limpiamente un gate con `--include`); el patrón de dominios sale del
   STACK REAL de la app (súmale su host si difiere). **Y todo comando que sea un gate viaja
   ENTRE BACKTICKS y se prueba copiándolo del RENDER (kit v1.24.0):** sin backticks el
   markdown come las barras invertidas y entrega un grep que no encuentra nada nunca — un
   gate muerto que pasa en verde para siempre. `/deploy-check` lo verifica
   (casilla de enlaces). *Origen: la vitrina de hoja-de-vida muestra QUÉ construyó el usuario
   (brochure + ficha del repo), nunca POR DÓNDE entrar.*
   **El barrido corre sobre el árbol que se va a subir — DESPUÉS del último `git add` (kit
   v1.26.0) — y vale también para código y comentarios de tests:** un barrido temprano deja
   ciega la ventana entre él y el push (los artefactos de `.lighthouseci/` entraron así al PR
   del S5 de hoja-de-vida con seis falsos positivos), y el comentario de un spec que cita el
   dominio de preview es una fuga igual que una URL en el README.

18. **PRs de dependencias: máximo DOS abiertos y el lockfile NO se pelea (kit v1.24.0 — regla
   del usuario 2026-08-22).** dependabot con techo real de 2 (limit 1 por ecosistema, todo
   agrupado — el yml del kit lo trae). Se mergean **DE A UNO, dejando a dependabot REGENERAR**
   entre merges (`@dependabot rebase` puede no obedecer, y el hand-merge del lockfile le rompe
   el parser: cerró un PR solo y abrió otro). Si un conflicto de lockfile TOCA resolverse a
   mano: la resolución **parte del lado que trae los bumps** y se verifica dependencia por
   dependencia que quedó la versión MÁS NUEVA de ambos lados — pnpm degrada en silencio y la
   CI pasa VERDE porque **ninguna puerta compara el resultado contra la INTENCIÓN del PR**:
   leer la salida del install ES el gate. `pnpm peers check` corre en quality (es lo único que
   ve un peer insatisfecho). Overrides: en `pnpm-workspace.yaml`, jamás en `package.json`.

## Estándares (los 6+1, gates en CI)

Testing · CI/CD · Observabilidad · Seguridad · Performance (contra `perf-budget.json`) · UX+A11y ·
**IA embebida responsable**. Detalle canónico: `estandares/estandares.md` de la planeadora
(read-only). Ítem rojo ⇒ deuda técnica explícita en el summary o el sprint no cierra.

## Workflow de un sprint

**Apertura** — el usuario trae la **orden de construcción** (`portafolio/<slug>/ordenes/SPRINT_NNN-orden.md`
de la planeadora). Léela entera + sus referencias (SPRINT_NNN.md, brief, prototipo READ-ONLY).
**Plan mode primero, siempre.** **La aprobación del plan NO arranca la construcción** (gate de
arranque, kit v1.6.2): tras aprobarse el plan, emite el bloque de arranque — tu recomendación de
**modelo y esfuerzo** para el sprint (el usuario los fija con `/model`) + espacio para sus
ajustes — y espera su **«construye»** explícito antes de tocar cualquier archivo.
Branch `sprint-NNN/<tema>`.

**Durante** — construye por fases (setup → motor → UI → integración → e2e). Mantén viva la bitácora
`sprints/SPRINT_NNN-implementation-log.md` (progreso, decisiones, bugs). ADRs en `decisions/` para
decisiones no anticipadas. `/self-review` tras cada bloque; `/run-tests` frecuente.
**Gate de FASE (kit v1.8.0): al terminar CADA fase DETENTE** — entrega el resumen completo de
la fase (qué se construyó, archivos, tests y resultados, criterio de fase completa,
desviaciones), recuerda al usuario que puede cambiar modelo/esfuerzo con `/model`, y espera su
**«continúa»** explícito antes de arrancar la siguiente fase. **Gate de FASE ≠ gate de
MIRADA (kit v1.20.0):** si la fase produjo un artefacto visual, «continúa» NO lo aprueba —
aplica la mecánica de mirada de la regla 10 (pregunta simple + lugar en la primera línea;
evidencia de archivo abierto o «lo abrí y apruebo»; sin eso, repregunta antes de construir
encima).

**Prototipo READ-ONLY** (si la orden referencia `referencias-ui/<slug>/` de la planeadora): extrae
paleta/tipografía/spacing/microcopy/patrones. ❌ No importes archivos, no copies código tal cual, no
uses su estructura de carpetas, no heredes sus gaps (testing/a11y/perf inexistentes).

**Cierre — auditoría OBLIGATORIA + summary OBLIGATORIO.** Al concluir la construcción (todas
las fases aprobadas): **corre `/audita-sprint`** (auditoría final de dos fases, método
v1.10.0 — Fase 1 solo-lectura con severidades y veredicto "listo para cierre"/"requiere
ajustes"; **el modelo poderoso audita y PLANEA los ajustes para que CUALQUIER modelo de menor
capacidad los ejecute**; Fase 2 solo tras aprobación del usuario) — ANTES de la guía/gate ⭐.
Luego, con la DoD completa: `/deploy-check` → genera `sprints/SPRINT_NNN-summary.md`
(plantilla abajo; **registra la auditoría: hallazgos y pagos**) → PR → gate ⭐ del usuario →
merge con CI verde. **El summary es CONDICIÓN DE MERGE (método v1.24.0): viaja DENTRO del PR
del sprint — un PR de sprint sin `SPRINT_NNN-summary.md` no se mergea.** Sin summary el sprint
es INVISIBLE para la planeadora (el S3 de Innmobiliaria lo estuvo UN MES) — y sin auditoría
registrada, el cierre queda condicionado. **Y si el sprint se mergea SIN completar sus fases,
el CORTE SE DECLARA EN EL MISMO ACTO (método v1.24.0):** en el PR y en el summary — qué fases
quedaron fuera y qué entregables arrastran; el corte silencioso arrastró 5 consecuencias
medibles.

**Cierre de CICLO — ocurre en DOS ACTOS (método v1.20.0; la orden declara cuando este sprint es
el ÚLTIMO del ciclo):** **Acto 1, DE CONSTRUCCIÓN** — el último sprint mergea con CI verde
(conclusión propia por check) + auditoría + contrapesos + BLUEPRINT; el gate del usuario aquí es
SOLO el storyboard + visual del **BROCHURE INICIAL** (regla 13 — llega por su orden de entrega).
**Acto 2, DE PRUEBAS (el sello MVP)** — cuando el usuario decida: gate **⭐⭐** → correcciones por
PR normal → **BROCHURE SELLADO** → `/design-sync`. **El ⭐⭐ condiciona el acto 2, no el 1**: no
retengas el merge ni el brochure inicial esperando el gate; y registra en el summary cuál acto
ocurrió — dos eventos, dos registros. Además de la DoD, el ciclo entrega (1) **`docs/BLUEPRINT.html`** — as-built
de TODA la infraestructura que soporta la app (plantilla `docs/BLUEPRINT.plantilla.html`: **HTML
autocontenido con diagrama SVG embebido** — jamás mermaid ni CDNs — + tabla por pieza + costo
real + punto único de falla), vivo y acumulativo entre ciclos;
y (2) el **design system publicado en Claude Design** con **`/design-sync`** — el comando está
estampado en este repo y su regla de reparto es (kit v1.18.0): **EL USUARIO INVOCA, TÚ EJECUTAS.**
La skill lleva `disable-model-invocation` — el disparador es del usuario, jamás lo lanzas tú —
pero la herramienta `DesignSync` sí es tuya: creas/verificas el proyecto, armas el bundle,
publicas. Lo reservado es el disparador, no el trabajo. Y **SIEMPRE después del gate ⭐⭐ corto**:
jamás se publica como activo estable un sistema que el usuario no ha juzgado. El punto de control
adicional es mecánico: `finalize_plan` le muestra al usuario la lista exacta de rutas y el
directorio de origen, independiente de lo que tú narres. Requisito previo: **el bundle
`design-sync/` del repo al día** (ver regla 16). Si el usuario decide NO invocarlo, el summary lo
registra como decisión suya explícita, no como olvido. Todo ciclo tiene MÍNIMO
3 sprints (regla dura 2026-07-17).

> **La regla llegó a su forma final equivocándose por mitades (v1.16.0 → v1.17.0 → v1.18.0):**
> v1.16.0 dijo *"lo corre EL USUARIO, no tú"* (verdad a medias: el disparador); v1.17.0 dijo *"lo
> corre TU sesión, la de la app"* (verdad a medias: el trabajo). Velo lo demostró ejecutándolo:
> el usuario escribió `/design-sync` y el constructor hizo todo — **el usuario invoca, el
> constructor ejecuta**. Se deja visible el camino a propósito.

**El gate ⭐ del cierre se corre POR BLOQUES, con arreglo en caliente (kit v1.16.0).** No entregues
la guía como una lista de 25+ pruebas para una sentada: el usuario recorre un bloque, tú corriges
lo que encontró **antes de que pase al siguiente**, y **el re-test de esa corrección es parte del
gate**. No es comodidad — es donde aparecen defectos que la primera pasada no puede ver. *(Origen:
ds S4, gate de 27 pruebas en 5 bloques: al re-verificar el arreglo del bloque C contra el proveedor
real apareció un segundo bug —`direction: null` hacía abortar la generación entera y la app caía a
plantilla **en silencio**— que un pase único habría enterrado.)* Registra en la bitácora, bloque a
bloque: resultado, ajustes aplicados en caliente y lo que va a backlog.

### Plantilla del summary

```markdown
---
sprint: NNN
app: <slug>
status: closed
opened: YYYY-MM-DD
closed: YYYY-MM-DD
branch: sprint-NNN/<tema>
pr: <link>
---
# Sprint NNN Summary — Angel Ghost
## Outcome            [¿Se logró el outcome del SPRINT_NNN.md? Sí/No/Parcial + 1 frase]
## Qué se construyó   [features/pantallas/componentes]
## DoD — checklist    [los 6+1 estándares, uno a uno, con evidencia breve]
## Métricas técnicas  [cumplidas vs. no, del SPRINT_NNN.md]
## Decisiones no anticipadas  [ADR-NNN: resumen]
## Bugs + resoluciones
## Qué salió bien / qué generó fricción
## Sugerencias de mejora al método  [¿algo de metodo/metodo.md debería cambiar?]
## Deuda técnica aceptada  [qué, por qué, sprint de pago]
## Archivos clave (máx. 10) · ## Cómo probar
```

## Patrones de dominio de esta app

[DOMAIN — llenar al estampar con los patrones del brief. Ej.: motor de fronteras de decisión en
`src/engine/decision-boundary.ts`, inferencia en Web Workers.]

## Idioma

Español en conversación y bitácoras. Inglés en código, commits, nombres y ADRs.
