# kit-app — CHANGELOG

> Las apps ya estampadas NO se actualizan solas: el delta relevante se anota en la orden de
> construcción de su siguiente sprint.

## v1.28.0 — aplicado en el sprint 002 de esta app (2026-09-24)

Batch nacido del cierre del S1 de esta misma app, así que la mayor parte ya vivía aquí. Lo que el
delta trajo de verdad:

1. **Regla 19 — gate de contrato entre lenguajes**, en el `CLAUDE.md` de la app. No existía: la
   lista terminaba en la 18. Su origen es el hallazgo C1 de este repo.
2. **Regla 15, cuarto filo: el MODO incluye el PERFIL DE COMPILACIÓN.** Un test que solo corre en
   `debug` no prueba `release` — el `catch_unwind` del PDF de este repo era letra muerta en
   release, donde `panic = "abort"` lo anula.
3. **`/audita-sprint`**: el reporte se persiste con TODOS los hallazgos y su `archivo:línea`, y la
   casilla 4 (frases caducadas) se **repite** tras el último ajuste de la Fase 2.
4. **`/release-check`**: todo comando de casilla vive en un job de CI o la casilla dice `manual` y
   por qué; `cargo clippy --locked`; casilla nueva de `--release`.
5. **El job `e2e` falla con cero pruebas**: fuera `--pass-with-no-tests` de `test:e2e`.
6. **Plantilla del summary** con la sección fija «Gate ⭐ — diferimiento y contrapesos».
7. **`testing-patterns` regla 10**: carpeta temporal única por test, creada por el propio test.

Ya estaba aplicado por haber nacido aquí: los perfiles de Playwright y Vitest de escritorio,
`afterEach(cleanup)`, `cargo clippy` dentro de `build-escritorio`, `verify-ephemeral` y el artefacto
`SPRINT_001-auditoria.md`.

**Añadido de esta casa, no del kit (regla 20):** el artefacto de auditoría se cuadra solo —
`tests/unit/auditoria-con-sitio.test.ts`.

## v1.27.1 — aplicado en el sprint 002 de esta app (2026-09-24)

El estampado real cazó tres defectos del kit. Para esta app, ya resueltos o no aplicables:
el falso verde del estampador (ahora espera `ci.yml` **por nombre**), `eslint@9` fijado por el peer
de `eslint-plugin-jsx-a11y`, `src-tauri/Cargo.lock` versionado —lo está— y `verify-ephemeral.mjs`
entrando al kit desde aquí; la mitad **en runtime** la añadió el S1 de esta app.

## v1.27.0 — 2026-09-18 (batch G-Metodo de la F1 de copiloto-consultor / Angel Ghost — primera app de ESCRITORIO)

1. **Perfil `--escritorio` en `estampar-app.sh`** (Tauri, `create-tauri-app --template react-ts`,
   Vite): sin Vercel ni Lighthouse; excluye `next.config.ts`, `instrumentation-client.ts`,
   `lighthouse-*.json`, `perf-budget.json`; instala **`ci-escritorio.yml` COMO `ci.yml`** (jobs
   `quality` · `e2e` sobre `pnpm preview` · **`build-escritorio`** en macOS con `cargo check/test`);
   `/release-check` en vez de `/deploy-check`; ESLint flat config mínima; Tailwind vía
   `@tailwindcss/vite`; ruleset con checks `quality/e2e/build-escritorio`; `.gitignore` gana
   `dist/`, `src-tauri/target/`, `src-tauri/gen/schemas/`; exige Rust (en `--simulacro` solo avisa).
   El `.ps1` NO lo porta (divergencia anotada). **Validado con `--simulacro --escritorio` ×3 el
   2026-09-18: cazó y corrigió dos defectos antes de gastar una app** — `create-tauri-app` no escribe
   `packageManager` (se inyecta tras el scaffold) y `src/lib/observability.ts` importa
   `@sentry/nextjs` (excluido del perfil; observabilidad por ADR). Corrección 2026-09-20: la instrucción
   de Rust era errónea (`rustup-init` ya no existe en el rustup de Homebrew, keg-only): PATH +
   `rustup default stable`; y la verificación local AVISA si `pnpm peers check` tiene conflictos.
2. **`/release-check`** (12 secciones): binario, permisos TCC, ventana protegida (gate humano), no
   persistencia (`verify:ephemeral`, fuga inyectada, término plantado, contador de red), `cargo
   audit`/`clippy`, cero enlaces de DESCARGA.
3. **`decisions/PLANTILLA-ADR-codigo-primero.md`**: cinco secciones obligatorias (feature · lo
   intentado con código CON evidencia · dónde entra y no entra el LLM · fallback · proveedor/costo/
   privacidad). La regla exigía el ADR desde 2026-07-12 y no existía molde.
4. **Gate de captura de terceros en la CI de escritorio** (estándar 4-T, estándares v2.13.0):
   `verify:ephemeral` corre si existe; si `CLAUDE.md` declara `captura_terceros: true` y el script no
   existe, el job FALLA (un gate saltado se ve igual que uno verde).
5. **CLAUDE.md del kit:** bloque «Perfil escritorio» + declaración `captura_terceros`.

## v1.26.0 — 2026-09-12 (batch del cierre hoja-de-vida S6 + S7 + revisiones post-S7)

1. **Regla 15 gana su tercera pregunta: «¿puede este gate FALLAR siquiera?»** — un gate
   inalcanzable por una regla anterior no es un gate; se retira y se anota quién lo cubría.
2. **CI: todo job que pueda ponerse rojo sube su evidencia al fallar** (`ci.yml` e2e:
   `upload-artifact` con `if: failure()` de `test-results/`). Un rojo sin trazas costó dos ciclos.
3. **Regla 5: la FORMA del árbol jamás depende de `useReducedMotion()`** (`null` en servidor →
   React #418); dos gates: unitario de HTML idéntico + axe bajo reduced-motion. Y **los tokens
   de tinta vetados como texto fallan en lint/test**, no en axe al final (skills `diseno-ui`,
   `accessibility-wcag`; `/deploy-check` §3).
4. **Lighthouse: ≥3 corridas por URL y aserción sobre la MEDIANA** (`--numberOfRuns=3`,
   `--aggregationMethod=median-run` en `ci.yml`; `/deploy-check` §8). Una corrida no mide.
5. **Regla 17 / `/deploy-check` §9: el barrido de cero enlaces corre sobre el árbol que se va a
   subir, DESPUÉS del último `git add`, y cubre código y comentarios de tests.**
6. **`gitignore.plantilla` gana `.lighthouseci/` y `.lh-*.json`.**
7. **`dependabot.yml`: el lote de npm lleva `update-types: [minor, patch]`** (los mayores llegan
   sueltos bajo el mismo techo) **+ `tests/unit/dependabot-config.test.ts`** como pieza del kit
   (el `ignore` por paquete no es gate: nadie lo ve operar). El estampado añade `yaml` a las
   devDependencies.
8. **`/audita-sprint`: la Fase 1 la corre un auditor INDEPENDIENTE con el diff delante** — los
   dos hallazgos más caros del S7 salieron de revisar lo que el constructor daba por bueno.
9. **Banco §8 «La capa infografía — la ficha técnica»:** referencia al contrato
   `ficha-tecnica` v1.3.1 (vive en hoja-de-vida, generado del Zod), anatomía de siete bloques,
   ficha armada (apps: export + complemento) vs. ficha completa (otros frentes).

Método → v1.27.0 el mismo día (variante corta de la bifurcación · cuentas con procedencia ·
inventario de lo que queda sin sujeto). Apps ya estampadas: los deltas viajan en su próxima
orden (hoja-de-vida S8 los lleva en fase 0).

## v1.25.0 — 2026-08-23 (batch del cierre hoja-de-vida S5 — la vitrina de los seis)

**Banco §7 «Apertura por lectura» gana la trampa T7 + el test de deriva cero completo:** la
compensación del cierre por arriba se MIDE (deriva real de un ancla visible), jamás se resta el
alto perdido a ciegas — el anclaje del navegador ya compensa parte y la resta doble produce el
bucle de reapertura. El test exige ancla con `top >= 0` (nunca un contenedor) y medición con las
animaciones finitas asentadas — sin las dos condiciones da rojos falsos sobre código correcto.
Referencia nueva: la adopción de hoja-de-vida S5 (12 fichas SSG).

**Regla 15: el rojo nace en el MISMO commit que introduce el gate**, no al final de la fase —
una demo diferida deja un gate «verde» que no mide nada (S5: deriva cero pasaba con el bug
puesto; la demo tardía reveló la prueba decorativa).

## v1.24.1 — 2026-08-22 (axe en la misma fase — cierre hoja-de-vida S4)

**Regla «pantalla tocada ⇒ su suite entera en la misma fase» ahora nombra axe explícito**
(skill testing-patterns): diferir el scan a la fase de integración dejó pasar un contraste
roto una fase entera. Ratificado con el checkpoint F0 #9.

## v1.24.0 — 2026-08-22 (batch #3 — cierre de la saga Innmobiliaria)

1. **El summary es CONDICIÓN DE MERGE** (viaja dentro del PR del sprint) y **el corte de
   fases se DECLARA en el mismo acto** (§ Cierre del CLAUDE.md; método v1.24.0).
2. **Regla 18 (nueva) — PRs de dependencias:** techo real de 2 · mergear de a uno dejando a
   dependabot regenerar · lockfile resuelto desde el lado que trae los bumps y verificado
   contra la INTENCIÓN del PR · `pnpm peers check` en quality (ci.yml) · overrides en
   `pnpm-workspace.yaml`.
3. **Regla 17: comandos-gate ENTRE BACKTICKS** probados desde el render (sin ellos el markdown
   come las barras — gate muerto en verde).
4. Patrón `rpc-publico-sin-cliente` promovido a `wiki/patterns/` (la planeadora); las órdenes
   lo citan donde aplique (apps con Supabase + páginas LCP-críticas).
5. Barrido cross-app «motion en el layout raíz» EJECUTADO por la planeadora: LIMPIO — solo
   Innmobiliaria usaba GSAP/Lenis y ya está corregida (PR #15).

## v1.23.1 — 2026-08-22 (fix del dependabot estampado — la cola de 6 era herencia del kit)

La config v1.23.0 llevaba `open-pull-requests-limit: 5` POR ecosistema (×2 = hasta 10
abiertos) y el `interval` semanal solo controla cada cuánto REVISA, no cuántos abre: la
primera activación vuelca el atraso entero de golpe (Innmobiliaria: 6 PRs el día 1). **Regla
del usuario (2026-08-22): máximo DOS PRs de dependencias abiertos, jamás una lista de merges
pendientes** ⇒ `limit: 1` por ecosistema + TODO agrupado (`patterns: ["*"]`). Apps que ya
estamparon la v1.23.0 (Innmobiliaria): corrigen su copia — el builder ya lo tiene agendado.

## v1.23.0 — 2026-08-22 (batch #2 del barrido — los 6 deltas de Innmobiliaria)

Fuente: ENTREGA-brochure de Innmobiliaria (reporte por mensaje directo entre sesiones).

1. **Grep del barrido TOTAL (regla 17 + `/deploy-check`):** todos los archivos versionados
   (excluye solo lockfiles), jamás include-list — la URL de producción vivía en
   `wrangler.jsonc` y pasó un gate con `--include`; el patrón suma `pages[.]dev` y el host real
   del stack de cada app.
2. **`.github/dependabot.yml` estampado** (semanal, agrupado minors+patches, mayores aparte):
   4 HIGH de Next envejecieron un mes sin aviso. Recordatorio pnpm 11: overrides en
   `pnpm-workspace.yaml`, NO en `package.json`.
3. **Molde: revelado con `threshold: 0` + red determinista** — un bloque más alto que la
   pantalla nunca alcanza un umbral porcentual y queda invisible (cazado a −5976 px).
4. **Banco (vetado): el motion narrativo no desvanece TEXTO** — el fundido rompe AA en vivo
   (1.32:1 medido); mover sí, deslavar legibilidad no.
5. **Banco §7 (muestras): si la sección enseña la app, son capturas de la app CORRIENDO o no
   va** (veredicto del usuario en el gate) — SVG dibujado solo para ilustración abstracta.
6. **Conteo default 1:1 contra las secciones `###` del MANUAL** (auditable con un comando).

Delta anotable en la orden del próximo sprint de las 6 apps (dependabot + grep total; el
resto es de molde/banco y aplica al próximo brochure/sellado que se toque).

## v1.22.0 — 2026-08-22 (apertura por lectura + regla 17 recurrente — cierre doble Velo/nutri-kids)

Fuente: ENTREGA-brochure de Velo (5 rondas de gate visual) y de nutri-kids (5 rondas
re-derivándola + la 6ª adoptándola entera).

1. **Banco de técnicas §7 (nueva) — «Apertura por lectura»:** tarjetas desplegables en
   brochures y piezas largas: una tarjeta está abierta exactamente mientras está a la vista;
   el toque es el control, no el peaje. Regla completa + trampas documentadas + verificación
   (cada test EN ROJO contra el commit anterior) + referencias (Velo · nutri-kids). Sustituye
   toda versión anterior del comportamiento de tarjetas y SE ADOPTA ENTERA (método v1.23.0).
2. **Regla 17 (ampliada):** (a) la limpieza del campo homepage es **RECURRENTE** — la GitHub
   App de Vercel lo reescribe tras cada deploy de producción; re-verificar tras cada merge a
   main; jamás automatizar con PAT admin como secret en repo público. (b) La narración del
   barrido escribe los patrones **sin el literal** (clase de carácter) — el grep debe seguir
   binario (evidencia: ds · nutri-kids).

## v1.21.0 — 2026-08-17 (el proceso vivo es superficie — batch del cierre dash S2)

Fuente: summary del S2 + Correctivo 001 de dash-agent-ai (el índice nacía world-readable con
prompts reales; capturas fotografiaron datos reales por un server viejo; start:seguro existió
dos sprints sin arrancar en vivo; las 3 pantallas se construyeron antes de la primera mirada).

1. **Regla 17-bis (nueva) — lo que la app ESCRIBE también es superficie:** (a) un derivado
   JAMÁS nace menos privado que su fuente (test en rojo + reparación al abrir); (b) todo arnés
   que pueda tocar fuentes arranca demostrando contra qué árbol corre y aborta ante datos
   reales sin confirmación.
2. **Regla 15 gana su tercer filo:** ¿lo viste correr EN EL MODO en que el usuario lo va a
   usar? Todo modo de arranque corre en vivo antes de cerrar el sprint que lo toca.
3. **El plan de miradas —número, agrupación y ORDEN— es parte del gate** (regla 10 +
   `plan-sprint` 9-bis e): reordenar también se declara antes, no solo agrupar.
4. **`/deploy-check` §12 (nueva): el disco en runtime** — inventario de derivados + permisos +
   modo real corrido + candados de arnés. El output pasa de N/11 a N/12.

## v1.20.1 — 2026-08-16 (refinamientos del gate de mirada — cierre de dash S1)

Fuente: desviaciones D1/D2 del summary de dash-agent-ai S1 — la primera ejecución real del
gate de mirada en construcción produjo su primer refinamiento.

1. **Agrupar N miradas se declara ANTES, jamás sobre la marcha** (regla 10 + `plan-sprint`
   §9-bis (e)): correr una mirada sobre varios artefactos que el plan separó es un cambio de
   plan que se propone y aprueba ANTES de construir el segundo. Origen: B6 (tema claro a
   360px) sobrevivió a la mirada agrupada de Resumen+Confianza; lo cazó la pasada de capturas.
2. **Del lado de la planeadora** (método v1.21.1): las órdenes citan reglas del CLAUDE.md de
   la app **por NOMBRE, jamás por número** (D1: la orden dijo «regla 10» y el diseño vivía en
   la 9 — los números se corren entre apps).

## v1.20.0 — 2026-08-16 (el gate de MIRADA — la mirada se demuestra mirando)

Fuente: informe `informe-gates-de-mirada` de Dash Agent AI (Etapa de Diseño): 4/9 pantallas
construidas sin una sola mirada real del usuario, con TODOS los gates de palabra formalmente
cumplidos. **3ª ocurrencia de la clase** (Velo S1→S2 → v1.13.0; Etapa de Diseño → v1.14.0):
las correcciones anteriores movieron DÓNDE va el gate; esta define CÓMO se verifica que
ocurrió. Directiva de claridad del usuario: *"necesito que me pregunte ¿apruebas el design
system? está en X lugar, y listo — sin códigos ni lenguaje encriptado"*.

1. **Regla 10 + § Workflow — el gate de MIRADA** (distinto del gate de FASE): solo se pasa con
   un comentario que delate el archivo abierto o el textual **«lo abrí y apruebo»**;
   **«continúa» JAMÁS aprueba diseño**; la PRIMERA línea del mensaje de gate es una pregunta
   simple en español llano + el lugar del archivo (el resumen técnico va después); silencio ⇒
   el agente se niega y repregunta «¿qué viste al abrirlo?»; AskUserQuestion/previews ASCII no
   sustituyen la mirada; cada mirada se REGISTRA antes de construir encima.
2. **`plan-sprint` §9-bis** — la mecánica completa del gate de mirada en los gates de fase.
3. **Skill `diseno-ui`** — nota de mirada por artefacto + ítem de checklist «registro de
   miradas completo» (un «continúa» en la bitácora no cuenta).
4. **Del lado de la planeadora** (método v1.21.0 · estándares v2.11.0): plantilla ORDEN-DISENO
   con la mecánica de «miro» por ronda; `/sprint` exige el registro de miradas en la
   precondición G-Diseño; `/cierre-sprint` lo audita (sin registro ⇒ cierre condicionado).

**Delta para apps ya estampadas:** viaja en la siguiente orden con UI de cada una; Dash Agent
AI lo adoptó en vivo (memoria persistente propia + etapa pausada en 4/9 hasta la primera
mirada real).

## v1.19.0 — 2026-08-15 (los dos actos del cierre + cero enlaces + el export del brochure)

Fuente: plan "vitrina + dos cierres" aprobado por el usuario (F0 #8) — hoja-de-vida como destino
de las 6 apps A NIVEL DE BROCHURE, jamás la app.

1. **El cierre de ciclo ocurre en DOS ACTOS** (§ Workflow/Cierre): **Acto 1 DE CONSTRUCCIÓN**
   (último sprint mergeado + lo mecánico + BLUEPRINT; gate del usuario: solo storyboard + visual
   del **BROCHURE INICIAL**) y **Acto 2 DE PRUEBAS = el sello MVP** (gate ⭐⭐ cuando el usuario
   decida → correcciones por PR → **BROCHURE SELLADO** → `/design-sync`). **El ⭐⭐ condiciona el
   acto 2, no el 1** — no se retiene el merge ni el brochure esperando el gate. Origen: el
   usuario quiere probar CON CALMA sin que el brochure espere ("ya no quiero correr a probar
   porque sí"); 3 apps llevaban semanas con el cierre en pausa por gates que ahora son acto 2.
2. **Regla 13 — el brochure gana sus DOS estados** (inicial → sellado; el sello NO congela: todo
   sprint posterior que cambie features lo ajusta EN EL MISMO SPRINT) **y su EXPORT**:
   `docs/brochure-export.json` (schema versionado: tagline, intro, funcionalidades con el conteo
   del MANUAL, métricas reales, stack) se produce junto al HTML — es lo que la vitrina de
   hoja-de-vida consume, anclado a versión, para re-expresarlo en SU design system (los colores,
   tipografía y estilo de la vitrina MANDAN).
3. **Regla 17 (NUEVA) — CERO ENLACES: la producción se MUESTRA, jamás se ENTREGA.** Ningún
   archivo del repo público ni campo de GitHub publica la URL de producción/previews (README ·
   About/website · BLUEPRINT, que documenta "qué ve quién" SIN la URL · manual · guía, cuyo campo
   de URL se llena EN USO · package.json). Las URLs son dato privado de la planeadora. CTA
   público = **«lista de espera»** (reemplaza a "solicitar acceso"; la gestión de acceso queda
   DIFERIDA a propósito). `/deploy-check` §9 gana la casilla con los comandos del barrido.
   ⚠️ Nota honesta: hasta hoy varios repos SÍ publican el link (Velo lo puso en su README como
   mejora; era razonable antes de esta regla) — el barrido de la fase 0 de cada orden de cierre
   los limpia.

Acompañan: método v1.20.0 (los dos actos + cero enlaces, F0 #8) · estándares v2.10.0 (2 ítems de
DoD) · plantilla ENTREGA-BROCHURE (modo inicial/sellado + export + barrido) · `/cierre-sprint` de
la planeadora (dos eventos, dos registros).

## v1.18.0 — 2026-08-15 (el gate corto ⭐⭐ + la forma final de «quién corre /design-sync»)

Fuente: cierre de Velo (`app-anonimizador`) S4 / ciclo H1 — las tres propuestas de su
implementation-log § "Propuesta para la casa planeadora", aprobadas en bloque (G-Metodo
2026-08-15).

**1 · El gate corto ⭐⭐ entra al molde de la guía.** El diferimiento del ⭐ (v1.12.0) acumuló en
Velo **26 pruebas / 90 minutos**, y el usuario lo aplazó **indefinidamente**. No falló el gate:
**falló su tamaño** — el ⭐ nació con el criterio correcto («solo lo no automatizable») y degeneró
en una lista de **transferencia de confianza**: una clase entera de sus pruebas (recalcular
huellas, mirar la pestaña Red, abrir el archivo con `strings`) el CI ya la verificaba; estaba ahí
para que la confianza fuera del usuario, no del builder. Razón legítima — pero con 20 minutos en
vez de 90, la distinción decide. La plantilla `GUIA-DE-PRUEBA.html` trae ahora **dos filtros de
gate desde el sprint 1**:

- **⭐ Gate mínimo** — como hoy, incluida la re-verificación de confianza. **Se ofrece.**
- **⭐⭐ Gate corto** — solo lo que ÚNICAMENTE un humano puede juzgar; techo **~20 min**; si el CI
  lo verifica por otro camino, **no entra**. Recorrido caminable («Parada N de M», secuencia
  comprobada), declara cuántas ⭐ deja fuera (ninguna se borra), no mueve los conteos de los
  demás filtros. **Es el que el cierre de ciclo EXIGE.**

Mecánica: atributo `data-corto` (subconjunto de `data-minimo`), botón de filtro nuevo, rama en
`visible()`, párrafo propio en la cabecera. Regla 11 del CLAUDE.md actualizada. Referencia viva:
la guía v4 de Velo (primera con los cuatro filtros: 125 · 11 · 26 · 6).

**2 · LA regla única de Claude Design (regla 10).** La orden del S4 de Velo exigía `/design-sync`
como entregable de cierre mientras la regla de feedback v1.14.0 decía «bajo demanda» — dos
fuentes contradiciéndose sin saberlo. La reconciliación queda escrita UNA vez: **bajo demanda
DURANTE el ciclo** (disparadores: gate visual que no converge · exploración pedida) ·
**OBLIGATORIA al cerrar el ciclo** (razón declarada: costo en minutos con el bundle v1.17.0 +
activo estable entre ciclos) · **SIEMPRE después del gate ⭐⭐** — jamás se publica como «activo
estable» un sistema que el usuario no ha juzgado, que era exactamente la ceremonia que la regla
v1.14.0 retiró. Las órdenes CITAN la regla; no la re-redactan.

**3 · «El usuario INVOCA, el constructor EJECUTA» — tercera iteración y la definitiva.** La regla
llegó a su forma final equivocándose por mitades opuestas: v1.16.0 dijo *«lo corre EL USUARIO»*
(verdad a medias: el disparador); v1.17.0 dijo *«lo corre TU sesión, la de la app»* (verdad a
medias: el trabajo). Velo lo demostró: el usuario escribió `/design-sync` y el constructor
ejecutó todo — creó el proyecto, armó el bundle, verificó y publicó. **Lo reservado es el
disparador, no el trabajo.** El comando del kit gana `disable-model-invocation: true` y su
sección «Quién» lo dice con esas palabras. El camino de las tres versiones queda visible a
propósito.

Acompañan: método v1.19.0 · estándares v2.9.0 · plantilla ORDEN (§ Gate ⭐ con ambos números y
las tres disciplinas del corto) · `/sprint` (el barrido declara los dos números) ·
`/cierre-sprint` (exige el corto; el largo se anota como ofrecido).

## v1.17.0 — 2026-08-15 (`/design-sync` deja de improvisarse: el bundle es un artefacto del repo)

Fuente: propuesta de la sesión constructora de `app-ds` tras su cierre de ciclo H1
(`app-ds/sprints/PROPUESTA-metodo-design-sync.md`), con el patrón **ya pilotado y funcionando** en
ese repo. G-Metodo 2026-08-15 (segundo del día).

> ### ⚠️ Corrección de v1.16.0, punto 4 — visible, no reescrita
>
> v1.16.0 estampó en el CLAUDE.md: *«`/design-sync` lo corre EL USUARIO, no tú»*. **Era falso como
> norma.** Salió de la sugerencia del summary de ds S4, que **describía el síntoma** de que el
> comando no existía en ninguna app — no una regla. Estampada en el kit, habría vuelto permanente
> en las seis apps justo el comportamiento que causó la fricción: sesiones que se declaran
> incapaces de publicar. **El eje correcto es *sesión de la app vs. planeadora*** (la regla de las
> dos casas), no *builder vs. usuario*; y el punto de control humano ya es mecánico —
> `finalize_plan` muestra al usuario la lista exacta de rutas y el `localDir` **independiente de lo
> que la sesión narre**, que vigila más que obligarlo a teclear el comando.

**El problema.** El método exigía publicar el design system al cerrar ciclo (v1.8.0) y **no definía
CÓMO**. Consecuencias medidas en ds: el bundle se construyó en el scratchpad de sesión y al
retomarlo días después quedaban **4 de 13 archivos** (hubo que reconstruirlo bajando del proyecto
remoto uno por uno); el `projectId` se re-descubría con `list_projects` en una cuenta con 6
proyectos, **dos de ellos llamados genéricamente "Design System"**; y las peculiaridades del tool
(`deletes` obligatorio aunque vaya vacío, tarjetas indexadas por el marcador `@dsCard` de la
primera línea, schema diferido) se aprendían por prueba y error, cada sesión desde cero.

**El principio:** *el bundle publicable es un artefacto del repo, no un efecto secundario de la
sesión.* Tres piezas con jerarquía fija — `design-system.md` (fuente de verdad) → `design-sync/`
(bundle versionado, deriva) → proyecto en Claude Design (vitrina, jamás se edita allá).

**Los cambios:**

1. **`.claude/commands/design-sync.md` (NUEVO)** — el procedimiento completo estampado: reglas de
   tarjeta, el diff (`git status design-sync/` **es** el plan), la secuencia del tool, y el
   registro final en `project.json` + bitácora + commit.
2. **`design-sync/` entra a la estructura estándar** del CLAUDE.md (nace en el primer sprint que
   publique; contiene `project.json`, `styles.css` y `components/<grupo>/<tarjeta>.html`).
3. **Regla 16 (NUEVA)** — el bundle es artefacto del repo, y **todo sprint que toca UI lo actualiza
   en su MISMO PR**; publicar puede esperar al cierre de ciclo. Separa lo barato (archivos, que
   entran a la revisión y a gitleaks) de lo que necesita la sesión autenticada del usuario, y
   garantiza que el cierre sea **un delta pequeño y nunca una reconstrucción**.
4. **Guarda nueva contra la divergencia silenciosa** (añadida por la planeadora al adoptar): antes
   de publicar, el comando compara el conteo de `list_files` contra `publishedFiles` de
   `project.json` y **se detiene si no cuadra**. Sin ella, *«nunca se edita en Claude Design»* era
   una buena intención sin mecanismo: una edición remota hacía divergir el repo en silencio y la
   siguiente publicación la pisaba sin dejar rastro.
5. **§ Cierre de CICLO corregido** con la nota de arriba: publica **la sesión de la app**, y la
   planeadora **solo verifica** que `lastPublished` no esté rancio.

Acompañan: método v1.17.0 (§ Ciclos) · estándares v2.8.0 (dos ítems de DoD) · `/cierre-sprint` de
la planeadora (verifica, jamás publica).

**Retrofit pendiente en las apps ya estampadas:** habla e inmobiliaria tienen design system
publicado **sin bundle en el repo** (mismo estado en que estaba ds); hoja-de-vida y nutri-kids no
tienen ninguno de los dos. La recuperación —bajar del proyecto remoto y versionarlo— va a la fase 0
de su próximo sprint y **es la última vez que hace falta en cada app**. `ds` no paga nada: su
piloto ES el estado final.

## v1.16.0 — 2026-08-15 (un gate que nunca EJECUTÓ tampoco es un gate)

Fuente: cierre de `app-ds` S4 y del ciclo H1 de Probeta DS (G-Metodo 2026-08-15, batch de 5
aprobado en bloque).

**El hallazgo.** El job `lighthouse` lleva `needs: quality`. `quality` llevaba en rojo desde antes
de esa sesión (por un `pnpm audit` que se puso rojo solo), así que **las 12 corridas de la rama
tuvieron `lighthouse: skipped`**. `skipped` no es rojo: GitHub lo lista entre los checks
requeridos **sin ninguna alarma**, y una columna sin rojo se lee como aprobación. El gate de
performance **no corrió ni una vez en un ciclo de cuatro sprints**, mientras el DoD lo daba por
cumplido apoyándose en corridas LOCALES. Al arreglar `quality`, corrió por primera vez — y salió
rojo.

**Los cambios:**

1. **`/deploy-check` §11 (nuevo) — los checks del PR.** Cada check requerido se verifica por su
   **conclusión propia `success`** (`gh pr checks`), no por la ausencia de rojo;
   `skipped`/`cancelled`/`neutral` no cuentan, y se reporta de qué `needs:` colgaba. Si un job
   corrió por primera vez, se dice en el summary: **sin histórico no puede afirmarse ni regresión
   ni no-regresión.** El output pasa de N/10 a N/11.
2. **CLAUDE.md regla 15 — gana su hermana.** *¿lo viste **fallar** cuando debía?* (v1.12.0) y
   *¿lo viste **correr**, alguna vez?* (esta). Un gate necesita las dos pruebas de vida: verde sin
   haber fallado nunca es una promesa; verde sin haber corrido nunca ni siquiera es eso.
3. **CLAUDE.md § Cierre de CICLO — el gate ⭐ se corre POR BLOQUES, con arreglo en caliente**, y
   el re-test de cada corrección **es parte del gate**. En ds, re-verificar el arreglo del bloque C
   contra el proveedor REAL destapó un segundo bug latente (`direction: null` ⇒ el LLM rechazaba
   la generación entera ⇒ **la app caía a su fallback en silencio**) que un pase único habría
   enterrado.
4. **CLAUDE.md § Cierre de CICLO — `/design-sync` lo corre EL USUARIO.** Es outward-facing, con
   prompts de permiso, y puede no estar estampado como comando en el repo. El builder deja
   `design-system.md` completo con la sección del sprint y lo lista bajo «pasos interactivos del
   usuario» en el summary; **jamás lo marca como hecho ni como automatizable.**
5. **`/deploy-check` §5 — tres cosas nuevas sobre dependencias.** (a) **La CI se puede poner roja
   por el CALENDARIO, no por el diff** (avisos nuevos sobre un árbol que no cambió), con la receta
   de menor a mayor intrusión: el parche que el aviso pide → `pnpm update` dentro de los rangos →
   overrides **acotados al rango vulnerable** y con `^` que **no cruza de major**. (b) ⚠️ **pnpm 11
   ya NO lee `pnpm.overrides` de `package.json`** — lo ignora con un WARN y sigue: en el sitio
   viejo la protección es **ficticia y nada lo delata**; viven en `pnpm-workspace.yaml`. (c) **Pin
   exacto para toda versión que viaje dentro de un artefacto exportado** (en ds, Pyodide viaja en
   cada `.probeta.json` y gobierna los avisos de compatibilidad al importar: moverlo habría
   invalidado los archivos que el usuario exportó durante su propio gate).

Acompañan: método v1.16.0 (§ F4, § Ciclos) · estándares v2.7.0 (dos ítems de DoD + fila del
estándar 2) · plantilla `ORDEN.md` de la planeadora (inputs comprobados antes de enumerarse ·
gate ⭐ por bloques · overrides en la verificación de supuestos · `/design-sync` como paso del
usuario).

## v1.15.2 — 2026-08-13 (la §4 se busca por promesa aplazada, no por la palabra de la feature)

Fuente: cierre de `app-anonimizador` S3 (G-Metodo 2026-08-13). **La casilla «¿qué frases
caducaron?» de v1.15.0 se corrió… y aun así se le escaparon dos frases.** No falló por no
correrse: **falló por cómo se buscó.**

El inventario del S3 se armó buscando **la palabra de la feature** (`irreversible`), y las dos
frases vivas nunca la usaban — decían *"no se puede **revertir**"* y *"el **camino de vuelta**"*.
Al rehacer la búsqueda por **promesa aplazada** salieron las dos, más una tercera de propina: la
guía de prueba decía que la app "todavía no transforma", **caducada desde el S2** — llevaba un
sprint entero repitiéndose.

**El cambio:** `/audita-sprint` §4 y `/deploy-check` §9 fijan el vocabulario a barrer —
`todavía no` · `aún no` · `por ahora` · `de momento` · `mientras tanto` · `próximamente` ·
`llega después` · `en esta versión` · `más adelante` · `no (se) puede` + los futuros (`podrás`,
`permitirá`) — y exigen revisar **cada coincidencia contra lo que la app hace HOY**, no contra lo
que el sprint construyó.

**Por qué funciona:** *el vocabulario de la promesa aplazada es corto y estable; el nombre de la
feature cambia con cada sprint.* Buscar por el segundo es buscar por lo único que se mueve.

Acompañan: método v1.15.1 (§ auditoría de dos fases) · estándares v2.6.2 (ítem de DoD).

## v1.15.0 — 2026-08-11 (las cuatro fricciones de instrumentación del S2 de Velo)

Fuente: `## Sugerencias de mejora al método` del summary de `app-anonimizador` S2, aprobadas en
bloque (G-Metodo 2026-08-11). Misma familia que el batch v1.12.0 — **gates y verificaciones que
dicen mirar algo y miran otra cosa** — pero del lado de la instrumentación:

1. **`/audita-sprint` §4 — «¿qué frases caducaron?»** Los pasos 1–3 auditan lo que el sprint
   **construyó**; el nuevo audita lo que dejó **falso**. Recorre portada, manual, README, guía,
   copy de estados vacíos y brochure buscando «todavía no…», «por ahora solo…», «próximamente…».
   *Origen: la portada, el manual y el README de Velo anunciaban «todavía no transforma el
   archivo» EN el sprint que transformó — el texto más leído de la app describía el sprint
   pasado.* También como casilla del `/deploy-check` §9.
2. **`/audita-sprint` §5 — campos del contrato sin consumidor.** Comprobación mecánica: por cada
   tipo de salida creado o ampliado, contar cuántos campos tienen lector fuera de su
   construcción y sus tests. *Origen: el reporte del tratamiento existía entero y probado **sin
   un solo llamador** — 6 de 9 campos huérfanos, invisible para 542 unitarias verdes.* Un motor
   probado no es un producto probado: **el defecto vive en el cable, no en la pieza.**
3. **Los reintentos dejan de taparse.** `playwright.config.ts` en CI pasa a
   `[["github"], ["list"]]`. `/run-tests` y `/deploy-check` exigen **cero flaky** y reportar por
   nombre los specs que pasaron en su 2º intento. **Un gate que necesita sus propios reintentos
   está avisando de algo.**
   > **⚠️ Corrección de este changelog (v1.15.1, 2026-08-13) — la razón que escribí aquí era
   > falsa, y la demo en rojo del S3 la desmintió.** Decía que el reporter `github` *"no imprime
   > la línea `N flaky`"*. **Sí la imprime**, dos veces: como texto al final del log y como
   > anotación `::notice` del PR (verificado aislando `--reporter=github`). Lo que `github`
   > **no** imprime es el **avance por prueba**: sin `list`, el log salta de «Running 70 tests»
   > al resumen, sin una línea intermedia — no se ve qué prueba corre, cuánto tarda ni cuál se
   > quedó colgada. **El cambio se queda porque esa visibilidad es real** (es la que vuelve
   > legible un timeout), pero con la razón corregida. *Y el origen que cité —"el e2e pasaba en
   > CI porque `retries: 2` tapaba un timeout mal puesto"— tampoco lo sostiene el registro: las
   > 12 corridas verdes del repo dan **cero flaky**, así que el timeout de 30 s nunca se disparó
   > en el runner, solo en local. Era una inferencia razonable del S2 escrita como hecho.* La
   > lección es de método, no de config: **"un gate se demuestra fallando" también demuestra
   > falsas las premisas de quien lo pide** — y la línea de partida del gate «cero flaky» es
   > cero flaky en 12 de 12, no un flaky invisible que ahora se revela.
4. **Se verifica con EL comando del CI, no con uno parecido.** Regla dura al principio de
   `/run-tests` y `/deploy-check`: `pnpm test` ≠ `vitest run` (el script lleva `--coverage`, y
   ahí viven los umbrales); `pnpm typecheck` ≠ `tsc --noEmit` (falta `next typegen`). Y **leer
   lo que viene después del resumen** — el verde de las pruebas no es el verde del job.
   *Origen: 502 pruebas verdes en local, CI rojo por el umbral de `src/lib/**`.*

**Delta para las apps ya estampadas:** los 4 puntos son independientes; (3) toca un archivo de
config, (1)(2)(4) son de comandos y no cambian código. Se anotan en la orden del siguiente
sprint de cada app.

## v1.14.0 — 2026-08-11 (el diseño va ANTES del código — Etapa de Diseño + gate G-Diseño)

Fuente: directiva del usuario, mismo día que v1.13.0 y superándola: *"la parte de diseño es muy
importante y debe estar claramente antes de cualquier código; no quiero que sea un paso más
sino un buen tiempo que debemos garantizar… justo apenas ejecutamos el script y lo subimos a
GitHub y Vercel, ahí debemos empezar una muy detallada etapa de diseño."* Es la restauración
del **paso 07 legado** (diseño-primero por el usuario, 2026-04-24) como etapa formal del
método (F2a).

1. **Regla 10 del `CLAUDE.md` reescrita:** en un repo recién estampado, **el primer trabajo del
   builder NO es construir — es diseñar**. Branch `diseno/fundacion`, orden de diseño de la
   planeadora, y dos entregables: `design-system.md` completo + **maqueta navegable del H1
   COMPLETO** en `docs/diseno/` (HTML autocontenido, una página por pantalla core con estados,
   mobile + desktop, ambos temas, **cero React y cero motores**), desplegada en Vercel.
   **CERO código de producto hasta que el usuario apruebe G-Diseño** sobre la maqueta
   desplegada. Sala de diseño: rondas sin presupuesto de pasos ni prisa.
2. **Nuevo `docs/diseno/README.md`** (se estampa en cada app): plantilla del registro de
   G-Diseño — tabla de cobertura (página → feature de la VISION) + veredicto, fecha, rondas,
   URL de aprobación, decisiones selladas. Sin registro lleno, no hay orden de construcción.
3. **El gate del primer sprint con UI (v1.13.0) se re-perfila:** con G-Diseño aprobado es
   verificación de **FIDELIDAD** (primera pantalla construida comparada contra la maqueta en el
   gate de FASE); en apps sin etapa (estampadas antes de v1.14.0) sigue siendo aprobación de
   **DIRECCIÓN**. Indiferible en ambos casos.

Acompañan: método → v1.14.0 (F2 partida en F2a fundación + F2b sprint; **G-Diseño** con fila
propia en el resumen de gates) · plantilla nueva `ORDEN-DISENO.md` en la planeadora · skills
`/nueva-app` (el estampado no termina en "repo listo", termina en "Etapa de Diseño abierta") y
`/sprint` (precondición: sin G-Diseño no se planea construcción en apps nuevas).

**Aplicación:** apps estampadas desde 2026-08-11 (primera: `dashboard-ia`). Las 5 veteranas ya
tenían el diseño del usuario (paso 07 de facto); Velo queda como excepción declarada
(identidad al ⭐ del S3, decisión del usuario 2026-08-11).

## v1.13.0 — 2026-08-11 (gate de dirección visual — temprano, una vez por app, indiferible)

Fuente: hallazgo del usuario — **Velo llegó a su S2 sin que él viera un solo artefacto visual.**
Tres capas fallaron juntas, ninguna suficiente sola: (1) el proceso legado era **diseño-primero
POR EL USUARIO** (paso 07: diseñar todas las pantallas antes del código; los prototipos de
`referencias-ui/` son obra suya, y `/nueva-app` pedía crear el proyecto de Claude Design al
arrancar) — el G-Metodo 2026-07-07 retiró esa ceremonia (creada y sin usar ×2) y movió el gate
visual AL FINAL del sprint, llevándose sin notarlo el contacto temprano del usuario con la
identidad; Velo fue la **primera app sin prototipo propio** bajo ese default; (2) el builder del
S1 declaró el gate visual pendiente en el summary y el PR se mergeó; (3) el diferimiento del ⭐
(v1.12.0) barrió la aprobación visual dentro del acumulado — **tratando una decisión de
fundación como un test de regresión**.

**Regla 10 del `CLAUDE.md` ganó el gate:** en el PRIMER sprint con UI de una app sin prototipo,
la fase de UI se parte en dos — `design-system.md` + **LA PRIMERA pantalla core** con tokens
reales → DETENERSE en el gate de FASE → **capturas** → la aprobación de **dirección** del
usuario (~5 min: tipografía, color, personalidad, tono) es requisito para el resto de la UI.
**No viaja con el ⭐:** dirección (¿es esta la identidad?) ≠ experiencia (¿funciona y se siente
bien?). Iterar con UNA pantalla es barato; con el ciclo entero encima es un rediseño.

Acompañan: método → v1.13.0 (F3 §3 + fila en el resumen de gates) · estándares → v2.5.0 (ítem
de DoD) · `/sprint` y `/cierre-sprint` de la planeadora (el cierre del primer sprint con UI
queda CONDICIONADO si el summary no registra la dirección aprobada, sin importar
`gate_estrella:`) · plantilla ORDEN (casilla en la sección Gate ⭐).

**Decisión del usuario sobre Velo:** SIN corrección retroactiva — su dirección visual va en el
⭐ acumulado del S3. La regla aplica de aquí en adelante (dashboard-ia será la primera app que
nazca con ella).

## v1.12.0 — 2026-08-10 (los tres gates que el cierre de Velo S1 dejó al descubierto)

Fuente: `## Sugerencias de mejora al método` del summary de `app-anonimizador` S1, aprobadas en
bloque (G-Metodo 2026-08-10). Las tres comparten síntoma: **un gate que dice medir algo y no lo
mide** — o mide otra cosa, o mide un artefacto que no es el que se despliega.

1. **`playwright.config.ts`: el e2e corre SIEMPRE contra el build, nunca contra el dev server.**
   `command: "pnpm build && pnpm start"` + `reuseExistingServer: false` de fábrica (antes: `pnpm
   dev` fuera de CI, con reuso). El dev server inyecta en la página cosas que **no existen en
   producción** —websocket de HMR, `eval()` de React dev— y en una app con CSP estricta o gate de
   red eso produce rojos sobre un árbol limpio: **5 en Velo S1**, justo en el suite que sostiene
   la promesa central del producto. Un suite que grita cuando no pasa nada acaba ignorado; se
   paga el tiempo del build a cambio de que el e2e local afirme lo mismo que el de CI. El
   `reuseExistingServer: false` evita además que un `pnpm dev` olvidado en :3000 secuestre la
   corrida en silencio. `timeout` 120s → 180s (ahora incluye el build).
2. **La casilla «Lighthouse ≥90» gana gate mecánico.** Nuevo `lighthouse-categorias.json`
   (aserciones `categories:*` con `minScore` 0.9 en las 4) y el job `lighthouse` pasa de un
   `lhci autorun` a **un `lhci collect` + DOS `lhci assert`** (budgets, categorías). Motivo del
   desdoble: LHCI declara `assert.budgetsFile` *mutuamente excluyente* con `assert.assertions`
   («this option cannot be used in conjunction with any other assert option»), así que no cabían
   en la misma invocación. Hasta v1.11.0 el CI medía **solo** `perf-budget.json` —tiempos y
   pesos— mientras el `/deploy-check` pedía marcar «≥90 en Performance, Best Practices,
   Accessibility, SEO»: una casilla que se podía marcar sin haberla verificado jamás. El archivo
   se llama a propósito **`lighthouse-categorias.json` y no `lighthouserc.json`**: LHCI
   auto-descubre el segundo nombre, y entonces el assert de budgets lo cargaría solo y estallaría
   por la exclusividad mutua. Ojo al flag: `lhci collect` usa `--url=`, `lhci autorun` usaba
   `--collect.url=`.
   **Demo pendiente, y es obligatoria (ver punto 3):** la primera app que adopte este delta
   —Velo S2— corre una vez con `minScore: 1.0`, comprueba que el job sale **rojo nombrando la
   categoría**, revierte y lo registra en su bitácora. Sin esa corrida, el gate es una promesa.
3. **Regla 15 del `CLAUDE.md`: un gate se demuestra FALLANDO.** Todo gate nuevo que una app
   agregue —job, hook, aserción, umbral, script— nace con un cambio deliberado que lo pone en
   rojo, registrado en el implementation-log (vale un PR desechable que se cierra sin mergear).
   Nació sola en Velo S1 (PR desechable con `openai` → anti-IA rojo en 7 s, con el culpable
   nombrado) y **ya existía en los estándares del Estudio CINE** («el job necesita su
   cambio-carnada»): cuando una regla se inventa dos veces en dos harnesses independientes,
   pertenece al método común. Sube también a `metodo/metodo.md` (F4) y a la DoD de
   `estandares/estandares.md`. Contraejemplo que la justifica: la carnada floja de gitleaks dio
   dos «todo bien» seguidos sobre un gate muerto (2026-07-15).

**Delta para las apps ya estampadas** (habla, ds, inmobiliaria, nutri-kids, hoja-de-vida, Velo):
los 3 puntos son adoptables por separado y sin dependencias entre sí. El punto 2 puede sacar en
rojo una app que hoy pasa: eso **es el hallazgo**, no una regresión del kit — se paga o se declara
deuda con el número real a la vista. Se anota en la orden del siguiente sprint de cada app.

## v1.11.0 — 2026-08-09 (el estampador se vuelve reanudable y auto-verificado — fix DE ORIGEN)

Fuente: exigencia del usuario tras el estampado de Velo ("¿por qué estaba incompleto? necesito
que lo soluciones de origen"). Las dos fricciones del día (v1.10.1 carnada, v1.10.2 pnpm) eran
**síntomas**; la causa era el diseño del estampador:

1. **Era todo-o-nada y se auto-bloqueaba la reanudación.** `set -euo pipefail` mata al primer
   error (correcto), pero la guardia *"si el directorio ya existe, aborta"* volvía IMPOSIBLE
   reintentar — y el paso 1 crea ese directorio. Cualquier fallo posterior dejaba el estampado
   a medias sin camino de vuelta: rescate manual. El README lo llamaba "10 pasos idempotentes";
   **no lo eran** — ningún paso comprobaba si ya estaba hecho.
2. **El kit no se auto-verificaba antes de viajar:** un secreto DENTRO del kit solo se
   descubría cuando el hook lo bloqueaba dentro de la app.
3. **Declaraba éxito sin ver la CI:** hacía typecheck+build locales y anunciaba "ESTAMPADO
   COMPLETO", pero el criterio declarado del estampado es *"CI verde en el primer PR"* — que
   jamás comprobaba. *(Un gate se define por su MECANISMO, no por su intención.)*

Cambios en `estampar-app.sh`:

- **A · REANUDABLE (el fix estructural):** desaparece la guardia "el dir ya existe → abortar";
  cada paso pregunta *¿ya está hecho?* y lo salta anunciándolo (`·· ya estaba`). Marcas de
  estado: `package.json` · `node_modules/next` · `.claude/skills/repo-app.md` · CLAUDE.md sin
  `[APP_NAME]` · `scripts.typecheck` · commit con `kit-app` en el log · `settings.local.json` ·
  remoto `origin` · ruleset presente. **Reintentar el mismo comando termina lo que falte, muera
  donde muera.** Nueva bandera `--reiniciar` (borra y empieza de cero, con confirmación
  tecleando `BORRAR`); sin ella el script JAMÁS borra nada.
- **B · PRE-FLIGHT del payload (paso 0-bis):** `gitleaks detect` sobre los archivos exactos que
  se van a copiar, ANTES de tocar disco; si el kit trae un secreto se detiene con archivo:línea.
  *El kit no puede enviar lo que su propio hook rechaza.*
- **C · CI DE VERDAD (paso 9-bis):** tras crear el repo espera el primer run (`gh run watch
  --exit-status`) y **reporta el resultado real**; en rojo imprime los errores del job y NO
  declara el estampado completo. Más el **guard de coherencia pnpm (paso 5-bis)** en las DOS
  direcciones: `packageManager` + `version` en el ci.yml = choque (falla); ninguno de los dos =
  la acción no sabe qué instalar (falla).
- **D · REPORTE HONESTO:** "COMPLETO Y VERIFICADO" solo con repo + ruleset + CI verde;
  si no, "INCOMPLETO — falta CI verde". Lo manual (Vercel + **Deployment Protection + prueba en
  incógnito**, regla de última milla del método v1.11.0) sale como PENDIENTE, nunca como listo.
- **E · `--simulacro`:** estampa en un directorio temporal, corre todas las verificaciones,
  borra y reporta — **detecta la deriva del toolchain sin gastar una app real**. Recomendado
  antes de cada app nueva y siempre que hayan pasado >30 días desde el último estampado (entre
  inmobiliaria y Velo pasaron 24 y el mundo cambió debajo).
- Args parseados en bucle (antes solo se miraba `$2`); `rsync` añadido a precondiciones.
- **TERCERA deriva del toolchain, cazada por el `--simulacro` recién nacido:** el scaffold de
  **Next 16** genera un `layout.tsx` que usa el tipo global `LayoutProps`, **que Next genera** y
  no existe en un árbol recién creado ⇒ `tsc --noEmit` falla con *"Cannot find name
  'LayoutProps'"* y la CI muere en `typecheck`, que corre ANTES de `build`. Fix: el script
  inyecta `"typecheck": "next typegen && tsc --noEmit"` (comando oficial de Next ≥15.5,
  verificado en 16.3.0). **Las apps ya estampadas con Next 15 no necesitan el cambio**, pero
  al subir a Next 16 sí — va como nota en sus próximas órdenes.
- **El `--simulacro` se pagó solo el día que nació:** cazó DOS bugs antes de tocar una app real
  — un falso positivo del propio guard de pnpm (matcheaba `node-version:` de setup-node) y esta
  deriva de Next 16. Ninguno de los dos era visible sin estampar de verdad.

> **Divergencia declarada (paridad):** `estampar-app.ps1` (estación Windows de respaldo) NO
> recibe A–E en este commit — no se toca PowerShell que no se puede probar aquí. Queda en el
> flujo v1.10.2. Si se vuelve a usar esa estación, portar primero.

## v1.10.2 — 2026-08-09 (fix de CI del estampado: choque de versiones de pnpm)

Fuente: estampado de `anonimizador` (Velo) — la CI del commit inicial murió en 8 s:
`pnpm/action-setup@v4` **aborta cuando la versión de pnpm viene declarada dos veces**
(`version: 11` en el workflow **y** `"packageManager": "pnpm@11.10.0"` en el `package.json`).
El `create-next-app` de hoy escribe `packageManager` solo; las apps estampadas antes
(habla, ds…) NO lo tienen — por eso la fricción aparece ahora, con la primera app estampada
con el toolchain nuevo.

- **`.github/workflows/ci.yml`: se quita `with: { version: 11 }` de los TRES jobs**
  (quality · e2e · lighthouse). `pnpm/action-setup@v4` lee la versión de `packageManager` —
  **fuente única de verdad**, y fijada al parche exacto en vez de al major.
- **⚠️ Condición para apps YA estampadas:** este delta **solo se adopta si el `package.json`
  de la app tiene `packageManager`**. Las cinco apps anteriores (habla · ds · hoja-de-vida ·
  nutri-kids · inmobiliaria) **no lo tienen: NO deben tocar su `ci.yml`** — sin `version` ni
  `packageManager`, la acción falla por lo contrario (ninguna versión declarada). Si alguna
  regenera su scaffold o añade `packageManager`, ahí sí adopta el delta.

## v1.10.1 — 2026-08-09 (fix de estampado: la carnada del CHANGELOG bloqueaba el commit del kit)

Fuente: estampado de `anonimizador` (Velo) — el script murió en su propio gate: `gitleaks`
bloqueó el commit del kit por un secreto REAL en `CHANGELOG.md` línea 272, que llevaba la
carnada canónica **entera**. La regla v1.7.3 ("la carnada viaja PARTIDA") se aplicó al
`CLAUDE.md` del kit, pero **la entrada de changelog que documentaba esa misma decisión la
escribió completa** — y el CHANGELOG SÍ viaja a cada app estampada. Velo fue la primera app
estampada desde entonces y la primera en chocar. El gate funcionó exactamente como debe:
detectó un secreto real antes de que entrara a un repo público.

- **`CHANGELOG.md` (entrada v1.6.3): la carnada ahora viaja PARTIDA** (concatenación en tres
  piezas), igual que en `CLAUDE.md`.
- **Regla explícita (nueva):** **todo archivo del kit que se COPIE a un repo de app lleva la
  carnada PARTIDA.** Los que NO viajan —`README.md` y ambos estampadores (`.sh`/`.ps1`)— la
  llevan **entera a propósito**: son la instrucción que el usuario copia para probar el hook.
  Al escribir cualquier archivo nuevo del kit, preguntarse primero: *¿este archivo aterriza en
  el repo de una app?*
- Sin cambios de comportamiento. Desbloquea el estampado de Velo y de toda app futura.

## v1.10.0 — 2026-08-08 (molde BROCHURE v2: se produce, no se estampa — informe del piloto habla)

Fuente: `app-habla/sprints/ENTREGA-brochure-informe-planeadora.md` (+ summary + storyboard).
El piloto estampó el molde v1 fielmente — CI verde, Lighthouse 100 — y el usuario lo RECHAZÓ
(«hacia abajo todo mal»); el final fue sobresaliente («está espectacular») y la diferencia fue
MÉTODO. Diagnóstico: el v1 heredó la LEY del estudio CINE y ninguna de sus TÉCNICAS (13
patrones catalogados, el molde usaba 2) y saltaba la dramaturgia (la privacidad — la promesa
mayor — enterrada en un acordeón porque nadie preguntó cuál era el clímax). Dato duro: de 16
defectos, la CI vio 4; las capturas leídas como imagen, 12; el gate del usuario sumó 11
correcciones más; la verificación externa halló el mayor (la app entera tras el login de
Vercel). Las 7 adopciones de la § 5 del informe:

- **(1 · estructural) `BROCHURE.plantilla.html` → v2 — REGLA CERO: no se estampa, se
  PRODUCE.** Storyboard obligatorio con clímax explícito, ritmo, reduced-motion por escena,
  dial MOTION_INTENSITY fijado con el usuario, identidad en una frase y riesgo registrado —
  aprobado («guion aprobado») ANTES del HTML.
- **(2) `BROCHURE-banco-de-tecnicas.md` (NUEVO):** las recetas del estudio en vanilla listas
  para copiar — G3 (blur-to-focus, alzar/asentar con stagger jerárquico, dibujar el trazo,
  steps() boiling line, contador rAF, confeti determinista), G2 (visibility en el colapso,
  apertura anunciada, dirección del recorrido, anclaje medido), G1 serena (el motor Mostar en
  ~60 líneas: un rAF → variables CSS con lerp/smoothstep e inercia) + lo vetado con porqués +
  esqueleto del storyboard.
- **(3) Gate de capturas por bloque** leídas como imagen (cuadro a cuadro en animaciones)
  antes de presentar al usuario — cazó 12 de 16.
- **(4) e2e obligatorio de reduced-motion** (visibilidad real de elementos clave): el hero
  invisible pasó 12 e2e y Lighthouse 100/100 sin él.
- **(5) Checklist de sala de proyección en el molde**, con la frase «la CI estuvo verde con
  un entregable rechazado» — verde no es bueno; fidelidad al producto real como criterio.
- **(6) Checklist de última milla:** el link de producción se prueba SIN sesión
  (curl/incógnito); dominio + protección de deployment documentados en el BLUEPRINT.
- **(7) Correcciones puntuales al molde:** `visibility` en la transición del acordeón (el
  lector de pantalla recitaba las 24 features "cerradas"; axe no lo ve) · LCP jamás desde
  `opacity: 0` (el h1 sube pintado; el fundido va a la línea de apoyo) · patrón canónico
  `<h3><button>` (el v1 anidaba al revés: HTML inválido) · icono del design system, no
  `[EMOJI]` · tabla de mapeo como evidencia del conteo N · trade-off de webfonts declarado
  (Georgia + system-ui) · nota `data-tema` (el brochure sigue al sistema, no al selector de
  la app) · presupuesto de scroll móvil (~1 pantalla).
- Transversal (al método v1.11.0): **lo medible se corrige midiendo contra la versión
  anterior** (el salto del piloto: 1291px → 119px — «creo que mejoró» es una opinión).

## v1.9.0 — 2026-08-07 (Brochure vivo: el anti-manual como entregable estándar)

Fuente: directiva del usuario — enviar Hablemos San a la mamá de su hijo (que odia los
manuales) exige una presentación de la app "de lo general a lo particular" que se consulte
como página web; y ese formato se vuelve el brochure de TODAS las apps y el contenido de la
vitrina de hoja-de-vida (H2). Decisión de arquitectura sellada: **el brochure se construye
en la casa de desarrollo de cada app** (contenido = sus features reales; entrega = ruta
pública de la app desplegada); la planeadora fabrica el molde; el Estudio CINE
(`hr03-estudio-cine`, RO) presta el ADN visual, no la fábrica. Aprobado 2026-08-07.

- **`docs/BROCHURE.plantilla.html` (NUEVO):** molde autocontenido con 10 reglas en cabecera.
  4 capas de progressive disclosure (promesa → tarjetas → detalle por tarjeta → lo fino);
  regla de conteo contra el MANUAL-DE-USO (agrupar sí, omitir jamás); ADN CINE adaptado a
  documento: G3 entrada coreografiada (IntersectionObserver + stagger CSS) y G2 transición
  de estado al abrir capas — G1 scroll-scrubbed excluida a propósito (pesa demasiado para
  un brochure); solo `transform`/`opacity` (excepción declarada: grid-rows en expansión);
  `prefers-reduced-motion` = experiencia alterna completa; mobile-first; a11y (botones con
  aria-expanded, contenido real en DOM); tokens neutros que se REEMPLAZAN por el
  design-system de la app.
- **`CLAUDE.md` regla 13 (nueva; la anterior 13 pasa a 14):** brochure vivo obligatorio con
  doble vida (ruta `/conoce` + `docs/BROCHURE.html`), actualización en el mismo sprint que
  cambia features, cero datos personales.
- Primera implementación: app-habla (orden puntual de la planeadora, decisión F0 puntual
  2026-08-07). La vitrina de hoja-de-vida consumirá los brochures de las 5 apps (H2, se
  agenda en el checkpoint F0).

## v1.8.2 — 2026-08-07 (cosecha del primer ciclo completo: gate ⭐ de habla S4)

Fuente: retrospectiva del cierre de habla S4 — primer ciclo H1 completo del portafolio. Las dos
primeras reglas nacieron de fricciones REALES del gate del usuario (una semana de sesiones por
bloques); la tercera consolida el peaje de subir a React 19. Aprobado 2026-08-07 (propuesta
chat-only, método v1.10.0).

- **`docs/GUIA-DE-PRUEBA.html` (plantilla), regla 8 — "Empieza en:" por bloque:** la meta-línea
  de CADA bloque abre con la ruta exacta donde arranca su primera prueba (`Empieza en: /jugar`),
  con la convención explicada una vez en la caja de la URL. El probador jamás deduce desde dónde
  parte un bloque. Aplicada en la anatomía de ejemplo (bloques A y B).
- **`docs/GUIA-DE-PRUEBA.html` (plantilla), regla 9 — los acumulados declaran su punto de
  partida:** toda prueba que observa un acumulado (mínimo/máximo/contador/racha) dice en su
  "Esperado" desde qué estado se parte ("reinicia la sesión y…"). Origen: la prueba A6 de habla
  pedía ver subir un mínimo corrido dentro de la misma sesión — un imposible del instrumento.
- **Skill `testing-patterns` § React 19 + linter estricto (NUEVA):** (1) cargar storage al
  montar con `useSyncExternalStore`, no `setState` dentro de `useEffect` (lint + flash del valor
  por defecto); (2) un ref que se lee en render pasa a estado (mutar el ref no re-renderiza —
  bug latente que el linter de React 19 ahora atrapa).

## v1.8.1 — 2026-07-20 (keep-alive de Supabase free estampado de fábrica)

Fuente: correo de Supabase avisando pausa por inactividad de `app-inmobiliaria` (2026-07-20).
Aprobado el mismo día. Diagnóstico: el keep-alive existía y corría verde, pero su cadencia
**semanal** no deja margen contra el umbral de 7 días y los crons de GitHub se retrasan horas
(la corrida del día salió 7 h tarde). Hermano no cubierto: hoja-de-vida S4 creó su proyecto
Supabase **sin** keep-alive.

- **`docs/supabase-keep-alive.plantilla.yml` (NUEVO):** workflow listo para copiar a
  `.github/workflows/supabase-ping.yml` + la migración compañera `ping()` (función `stable`
  que no toca tablas, `execute` solo a `anon`). **Cadencia lunes y jueves**, `curl -f`
  obligatorio, y la guarda de secrets faltantes ahora **FALLA** (`exit 1`) en vez de dar el
  falso verde que teníamos. Es plantilla y no workflow activo a propósito: una app sin
  Supabase tendría un cron corriendo en vacío y viéndose verde.
- **`docs/APROVISIONAMIENTO.plantilla.md`:** bloque obligatorio "si el servicio es Supabase
  free" — migración + workflow + secrets [TÚ] + **verificación en vivo** (corrida disparada a
  mano con 2xx real), y las dos trampas de segundo orden (GitHub deshabilita crons de repos
  quietos 60 días; el Pro se evalúa en G-Release, no antes).
- **Skill `testing-patterns` § e2e-BD-real, regla 0:** la CI corre contra Postgres LOCAL — el
  proyecto cloud no recibe actividad de ella; el keep-alive es parte del sprint que
  aprovisiona.
- **Plantilla ORDEN (planeadora):** bloque obligatorio en § Aprovisionamiento cuando el
  sprint crea un proyecto Supabase.
- Patrón completo: `wiki/patterns/supabase-en-ci-y-cloud.md` § 5ª fricción.

## v1.8.0 — 2026-07-20 (G-Metodo "Proceso v2": gates de FASE + /audita-sprint obligatoria)

Fuente: 4 directivas directas del usuario al concluir la construcción de la cola F0 #6.
Aprobado 2026-07-20 (propuesta chat/archivo temporal, borrado al aplicar — método v1.10.0).

- **Gates de FASE (`plan-sprint` paso 9 + CLAUDE.md + plantilla ORDEN):** al terminar CADA
  fase del plan, el builder SE DETIENE, entrega el resumen completo de la fase (qué se
  construyó, archivos, tests y resultados, criterio de fase completa, desviaciones) y espera
  el «continúa» explícito del usuario — cada pausa es un punto de decisión de
  modelo/esfuerzo (`/model`).
- **Comando NUEVO `/audita-sprint` (auditoría final de dos fases — OBLIGATORIA, método
  v1.10.0):** al concluir la construcción, antes de la guía/gate ⭐ y del cierre. Fase 1
  solo-lectura (cobertura de alcance ítem por ítem con archivos y líneas · calidad ·
  dependencias · severidades Crítico/Alto/Medio/Bajo · veredicto "listo para cierre" /
  "requiere ajustes"). **Regla de modelos: el poderoso audita y PLANEA — deja cada ajuste
  tan bien definido (archivo, línea, cambio exacto, criterio de verificación) que CUALQUIER
  modelo de menor capacidad ejecuta la Fase 2 de la mejor manera.** Fase 2 solo tras
  aprobación del usuario. El summary registra hallazgos y pagos; sin auditoría, el cierre
  queda condicionado (lo verifica `/cierre-sprint`).
- `arquitectura/matriz-modelos.html` (planeadora) actualizada: fila `/audita-sprint` + "los
  dos relojes de cambio de modelo".

## v1.7.5 — 2026-07-20 (G-Metodo del cierre nutri-kids S2: salida efímera de primera clase + puntas AI SDK v7)

Fuente: `sprints/SPRINT_002-summary.md` de app-nutri-kids (§ Estándar 7 — adaptación, § K9).
Aprobado en bloque 2026-07-20 — detalle en
`entrega/2026-07-20-propuesta-gmetodo-cierre-nutri-kids-s2.md`.

- **Skill `ia-embebida` § 2 — variante de salida efímera de PRIMERA CLASE:** cuando la
  privacidad manda no persistir la salida LLM, `schemas.ts`/`persist.ts` no aplican por
  diseño y el checklist se sustituye (no-persistencia por e2e · logs solo-metadatos con
  término plantado · minimización en ADR). Dos apps la declaraban como desviación
  (hoja-de-vida S3, nutri-kids S2) — se estandariza.
- **Nota "AI SDK v7 — puntas afiladas verificadas":** 5 quirks consolidados que cobraron una
  iteración cada uno en dos apps (`convertToModelMessages` async · `stream-start` primer
  chunk · tipos no exportados · `onError` en `toUIMessageStream` · `finishReason` objeto).
- **Checklist del sprint** con la alternativa efímera. K8 (lighthouse-urls.json) verificado
  YA RESUELTO — se estampa desde v1.2.0.

## v1.7.4 — 2026-07-20 (G-Metodo del cierre hoja-de-vida S3: humo de credenciales + mock de primera clase)

Fuente: `sprints/SPRINT_003-summary.md` de app-hoja-de-vida (§ Bugs, § Sugerencias).
Aprobado en bloque 2026-07-20 — detalle en
`entrega/2026-07-20-propuesta-gmetodo-cierre-hoja-de-vida-s3.md`.

- **Plantilla ORDEN § Aprovisionamiento — columna "Humo (≤1 min)":** toda credencial
  aprovisionada viaja con su comando de validación; el builder lo corre en la fase 0 y el
  checklist re-emitido al abrir el PR lo repite si la credencial cambió (la GROQ_API_KEY
  inválida de hoja-de-vida S3 se descubrió en la integración, no el día 0).
- **`plan-sprint` fase 0:** exige el humo de credenciales ANTES de la fase 1 (cinturón lado
  builder aunque la orden sea vieja).
- **Skill `ia-embebida` § 7:** el humo del proveedor real es día 0 (un 401 en integración es
  fallo de proceso). **§ 8 nueva — "el mock es un proveedor de primera clase":** dentro del
  adapter, jamás intercept de red; excepción solo para forzar caminos de error. Dos apps
  (hoja-de-vida, nutri-kids) convergieron por separado → se estandariza. Patrón wiki:
  `mock-como-proveedor-de-primera-clase`.

## v1.7.3 — 2026-07-19 (G-Metodo del cierre habla S3: regla 9 + riesgos de integración + carnada partida)

Fuente: `sprints/SPRINT_003-summary.md` de app-habla (§ Fricción, § Aprendizajes, § Remate).
Aprobado en bloque 2026-07-19 — detalle en `entrega/2026-07-19-propuesta-gmetodo-cierre-habla-s3.md`.

- **Skill `testing-patterns` regla 9:** pantalla ya cubierta por e2e ⇒ su suite ENTERA corre en
  la misma fase que la toca (F2a de habla S3 dejó un e2e aseverando "exactamente 3 juegos").
- **`plan-sprint` paso 4 ampliado:** el plan incluye SIEMPRE la sección "Riesgos de integración
  con lo existente" — leer el código de las features que la nueva toca (el riesgo nº 1 de habla
  S3, el bucle parlante→mic, no estaba en la orden).
- **CLAUDE.md regla 7 — carnada PARTIDA:** la carnada canónica viaja partida en la plantilla
  (armarla solo en el archivo de prueba) — estandariza las 2 soluciones ad-hoc (allowlist de
  inmobiliaria S2 · partida manual de habla S3). El hook debe seguir detectando la carnada
  ARMADA.
- **Fuera del kit (mismo batch):** método → v1.9.1 — cuando el gate ⭐ se difiere, se
  RECOMIENDA el remate de auditoría de dos fases antes del merge (práctica inventada por el
  usuario en habla S3; cazó un defecto S2 latente). Plantilla ORDEN: línea del remate.

## v1.7.2 — 2026-07-18 (G-Metodo del cierre inmobiliaria S2: testing-patterns ×3 + Lighthouse solo páginas públicas)

Fuente: `sprints/SPRINT_002-summary.md` de app-inmobiliaria (K7–K10). Aprobado en bloque
2026-07-18 — detalle en `entrega/2026-07-18-propuesta-gmetodo-cierre-inmobiliaria-s2.md`.

- **Skill `testing-patterns` — 3 reglas anti-flakiness nuevas (6–8):** specs de Playwright se
  transpilan a CommonJS (nada de `import.meta.url`; paths desde `process.cwd()`) ·
  `getByRole("alert")` desnudo choca con el `__next-route-announcer__` de App Router · checkbox
  controlado async → `.click()` + aseverar el resultado observable, no `.check()`.
- **README regla 5 — compromiso devtools CERRADO CON VEREDICTO (4º caso Lantern):**
  `throttlingMethod: devtools` DESCARTADO en runners compartidos (K8-bis: infló más). Regla
  vigente: **la auditoría Lighthouse de CI cubre SOLO páginas públicas**; privadas/noindex/
  hidratadas se excluyen documentadamente y su LCP real va al gate ⭐. `budgetsFile` no permite
  umbral por-path. Patrón: `wiki/patterns/lighthouse-solo-paginas-publicas.md`.
- **Fuera del kit (mismo batch):** método → v1.9.0 (figura "gate ⭐ diferido al cierre de
  ciclo") + plantilla ORDEN (línea del gate ⭐ acumulado).

## v1.7.1 — 2026-07-18 (ajuste del usuario: blueprint POR APP en HTML autocontenido + SVG)

Fuente: directiva directa del usuario (2026-07-18) corrigiendo el formato del entregable v1.7.0.
Detalle en `entrega/2026-07-18-propuesta-f0-6-mvps-en-orden-inverso.md` § A.

- **`docs/BLUEPRINT.plantilla.html` (reemplaza a `BLUEPRINT.plantilla.md`):** el blueprint
  as-built pasa a **HTML autocontenido con diagrama SVG embebido** (cero CDNs, cero mermaid) —
  el usuario lo quiere abrible en navegador y **bien definido por proyecto**, no en vistas
  agregadas de portafolio (la vista conjunta del 2026-07-17 queda como foto puntual, no se
  mantiene). El entregable de cierre de ciclo es ahora `docs/BLUEPRINT.html` en el repo de cada
  app (CLAUDE.md del kit, plantillas ORDEN/SPRINT y skill `/sprint` actualizados).
- **Fuera del kit (mismo ajuste):** método → v1.8.1 — redacción del escalado suavizada ("el
  escalado prioriza el software determinista; la IA es importante pero no la prioridad" — la
  frase "jamás metiendo IA" sobre-endurecía la intención del usuario).

## v1.7.0 — 2026-07-17 (G-Metodo "ciclos y cierres": BLUEPRINT + Claude Design al cierre de ciclo)

Fuente: 4 directivas directas del usuario (2026-07-17) tras el cierre del S1 de Innmobiliaria.
Aprobado en bloque — detalle en `entrega/2026-07-17-propuesta-gmetodo-ciclos-y-cierres.md`.

- **`docs/BLUEPRINT.plantilla.md` (nuevo):** as-built de infraestructura — entregable OBLIGATORIO
  del cierre de ciclo (diagrama mermaid + tabla por pieza + costo real/mes + punto único de
  falla); vivo y acumulativo entre ciclos. Vista de portafolio en la planeadora
  (`entrega/2026-07-17-blueprint-infraestructura-portafolio.md`, v1).
- **CLAUDE.md — bloque "Cierre de CICLO"** en el workflow: blueprint + design system publicado
  en Claude Design (`/design-sync`) cuando el sprint es el último del ciclo; todo ciclo tiene
  MÍNIMO 3 sprints (regla dura).
- **CLAUDE.md regla 10:** Claude Design sigue BAJO DEMANDA durante el ciclo, pero la PUBLICACIÓN
  del design system consolidado al cierre del ciclo es obligatoria.
- **Fuera del kit (mismo batch):** método → v1.8.0 (escalado por defecto condicional · ciclo ≥3
  sprints · cierres de ciclo) · skill `/sprint` · plantillas ORDEN/SPRINT · brief de inmobiliaria
  reestructurado (fase 1 = ciclo de 3). Retro-aplicación: las 4 apps con historia generan su
  blueprint retroactivo + design-sync en su próxima orden.

## v1.6.4 — 2026-07-17 (G-Metodo del cierre inmobiliaria S1: Supabase-en-CI + runbook + 3er caso Lantern)

Fuente: `sprints/SPRINT_001-{summary,retrospectiva}.md` de app-inmobiliaria (primer sprint del
pipeline con Postgres real en CI y primer S1 que cierra desplegado). Aprobado en bloque
2026-07-17 — detalle en `entrega/2026-07-17-propuesta-gmetodo-cierre-inmobiliaria-s1.md`.

- **Skill `testing-patterns` — sección "e2e con BD real (Supabase) en CI":** los 4 quirks
  pagados (K3–K6: comillas de `status -o env` · `stdout: "pipe"` de Playwright · GRANTs/REVOKEs
  explícitos en la migración, doblemente invisibles con RPC `SECURITY DEFINER` · rate limit
  apagado solo en CI con test a nivel RPC) + strict-mode con datos por-proyecto + nube temprana
  sin Docker. Patrón: `wiki/patterns/supabase-en-ci-y-cloud.md`.
- **`docs/APROVISIONAMIENTO.plantilla.md` (nuevo):** runbook [TÚ]/[CLAUDE] — división explícita
  dueño-de-cuentas/agente, verificación en vivo por bloque, regla de identidad (SSO satélite vs.
  credencial de primera mano + 2FA para la casa de la infraestructura).
- **README regla 5:** 3er caso Lantern REGISTRADO (inmobiliaria S1) ⇒ evaluación
  `throttlingMethod: devtools` COMPROMETIDA como spike en el próximo gate Lighthouse rojo.
- **Fuera del kit (mismo batch):** método v1.7.0 (figura "base declarada" en F1) · skill
  `/sprint` de la planeadora (condiciones para escalar el alcance de un S1) · plantilla ORDEN
  (nube temprana sin Docker).

## v1.6.3 — 2026-07-15 (G-Metodo: carnada canónica VERIFICADA para la prueba del hook gitleaks)

Fuente: estampado de inmobiliaria — la instrucción "prueba el hook con un secreto falso" sin
carnada especificada produjo DOS falsos "todo bien" seguidos (`AKIAZZZZ…` no pasa la entropía;
`AKIA…9…` viola el alfabeto base32 de la regla `aws-access-token`; hasta la llave de ejemplo de
AWS docs está exenta). La carnada correcta se halló probando 4 candidatas contra el gitleaks
local (8.30.1) en sandbox. Aprobado en bloque 2026-07-15 — detalle en
`entrega/2026-07-15-propuesta-gmetodo-carnada-canonica-gitleaks.md`.

- **Carnada canónica del pipeline** (inventada; dispara `aws-access-token`) — desde v1.10.1
  viaja **PARTIDA en este archivo**: ármala concatenando `AWS_ACCESS_KEY_ID=` + `AKIAQ7RTZ4PX`
  + `KM2WNB3S`. Escrita en: CLAUDE.md regla 7 · README § Estampado · mensaje
  final de AMBOS estampadores (`.sh`/`.ps1`, paridad) · plantilla ORDEN § Verificación de
  supuestos (las apps ya estampadas la reciben vía sus próximas órdenes, patrón v1.6.2).
- **Regla nueva:** toda carnada se verifica contra el gitleaks VIGENTE en sandbox antes de
  entrar al kit — nunca se receta de memoria; re-verificar al subir gitleaks de versión mayor.

## v1.6.2 — 2026-07-13 (G-Metodo: gate de arranque — aprobar el plan ≠ arrancar la construcción)

Fuente: fricción reportada por el usuario al preparar el arranque del S1 de Innmobiliaria — la
aprobación del plan en plan mode disparaba la construcción de inmediato, sin espacio para fijar
modelo/esfuerzo (`/model`) ni ajustes finales. Aprobado en bloque 2026-07-13 — detalle en
`entrega/2026-07-13-propuesta-gmetodo-gate-de-arranque.md`.

- **Command `plan-sprint` — paso 7 dividido en 7+8:** la aprobación del plan significa SOLO "el
  plan es correcto"; el paso 8 es el **gate de arranque**: bloque con recomendación de modelo y
  esfuerzo para el sprint (por fase si difiere) + recordatorio `/model` + espera de la palabra
  explícita **«construye»**. Prohibido crear/editar archivos antes.
- **CLAUDE.md § Workflow (Apertura):** regla espejo (aplica aunque el builder no entre por la
  skill).
- **Fuera del kit (mismo batch):** `portafolio/_template/ordenes/ORDEN.md` § Prompt de arranque
  declara el contrato en el prompt pegable — así el gate opera TAMBIÉN en las apps ya estampadas
  (el contrato viaja en cada orden nueva); enmienda en vuelo a la orden S1 de inmobiliaria.

## v1.6.1 — 2026-07-12 (G-Metodo del cierre habla S2: la CI verifica el comportamiento, no la experiencia)

Fuente: `sprints/SPRINT_002-summary.md` de app-habla (§ EL GATE DEL USUARIO: 4 defectos que 187
tests no vieron, todos de la misma familia). Aprobado en bloque por el usuario 2026-07-12 —
detalle en `entrega/2026-07-12-propuesta-gmetodo-cierre-habla-s2.md`; patrón completo en
`wiki/patterns/la-ci-verifica-comportamiento-no-experiencia.md`.

- **Skill `testing-patterns` — 3 reglas anti-"comportamiento sin experiencia":** (1) por cada
  pantalla, ≥1 e2e llega POR LA UI, no solo por `goto(url)`; (2) todo copy que AFIRMA una
  métrica tiene test que confronta la frase con la definición de la métrica; (3) los fixtures
  sintéticos incluyen casos FUERA del rango que el código asume (el test no puede validar el
  supuesto que comparte con el código).
- **Command `deploy-check`:** Lighthouse a mano es `npx @lhci/cli` — `npx lhci` a secas es un
  paquete impostor del registry.
- **CLAUDE.md regla 11:** referencia de implementación de la guía acumulativa →
  `app-habla/docs/GUIA-DE-PRUEBA.html` (S2).
- **Fuera del kit (mismo batch):** header de `estandares/estandares.md` alineado a v2.2.0
  (K-habla-5: la cita fija envejece).

## v1.6.0 — 2026-07-12 (G-Metodo: guía ACUMULATIVA + código primero + kit de prueba — directiva del usuario)

Fuente: directiva del usuario 2026-07-12, detectada construyendo habla S2 (una guía comprimida
despachaba con "todo como antes" un juego cuyo motor cambió por dentro — el gate habría pasado
sin probarlo). Detalle: `entrega/2026-07-12-propuesta-gmetodo-guia-acumulativa-y-codigo-primero.md`.

- **`docs/GUIA-DE-PRUEBA.html` REESCRITA — de "viva" a "viva y ACUMULATIVA" (bola de nieve):**
  la última versión contiene TODAS las pruebas vigentes; el sprint N hereda ENTERAS las del N−1
  (jamás "verificar que sigue funcionando"). Cada prueba lleva su **origen en su línea** —
  `Nuevo · SN` · `Mejorado en SN` · `SN` (heredada ⇒ regresión) — con **filtros** (Todo · Lo que
  cambió · **Gate mínimo ⭐**). El gate mínimo tiene criterio FIJO: solo lo que ninguna
  automatización puede verificar (hardware/mic/voz reales, juicio sobre contenido, aprobación
  visual); lo que la CI respalda queda fuera. **Namespace de `localStorage` versionado por
  sprint** (una regresión sin correr no aparece marcada por el sprint anterior). Historial de la
  guía en el pie (pruebas eliminadas solo con feature muerta, declaradas). Callout **kit de
  prueba** (`docs/kit-de-prueba/`).
- **CLAUDE.md del kit — regla 11 reescrita** (todo lo anterior) **+ regla 13 nueva: código
  primero, IA generativa después** — toda funcionalidad nativa interna se resuelve con
  programación (código/librerías/algoritmos) antes de acudir a IA generativa; feature LLM exige
  ADR "código primero"; la IA es acento con fallback, jamás columna vertebral.
- **Fuera del kit (mismo batch):** `estandares/estandares.md` v2.2 (DoD + precondición código
  primero en el estándar 7) · `metodo/metodo.md` v1.6.0 · `CLAUDE.md` de la planeadora
  (principios 6 y 7) · plantillas ORDEN y SPRINT · **enmienda en vuelo a la orden del S2 de
  habla** (sprint abierto — el caso que parió la regla).
- **Delta para ds/nutri-kids/hoja-de-vida:** su primera `GUIA-DE-PRUEBA.html` nace acumulativa
  con esta plantilla (va en la orden de su próximo sprint con UI).

## v1.5.1 — 2026-07-12 (G-Metodo del cierre habla S1: el kit contra su primer estampado del stack nuevo)

Fuente: `sprints/SPRINT_001-{summary,implementation-log}.md` de app-habla (K-habla-1..4, todo con
números). Aprobado en bloque por el usuario 2026-07-12 — detalle en
`entrega/2026-07-12-propuesta-gmetodo-cierre-habla-s1.md`.

- **perf-budget.json:** script **300→350 KB** — el framework solo (Next 16 + React 19) pesa
  ~246 KB gz; 300 era una multa a la primera feature (habla se rompió con 1,1 KB propios).
  TBT/LCP/CLS intactos como guardias reales.
- **README regla 5:** sesgo Lantern MEDIDO (LCP real 24 ms vs Lantern ~3380 ms) + criterio de
  renegociación por ADR (precedente ×2) + `throttlingMethod: devtools` como opción si hay 3er caso.
- **vitest.config.ts:** el umbral 80% de motores cubre `src/{engine,lib}/**` + advertencia de
  ajustar el glob al layout de CADA app en la verificación del kit (K-habla-1).
- **Skills `accessibility-wcag` + `diseno-ui`:** regla "el gate recorre TODAS las paletas"
  (el axe mono-tema de habla dejó pasar un 3.1:1 en modo oscuro).
- **Plantilla ORDEN (planeadora):** la versión del kit se cita como la vigente al estampar
  (K-habla-2: la orden decía v1.3.1, el estampado real fue v1.4.0).

## v1.5.0 — 2026-07-12 (G-Metodo: dos reglas duras de ENTREGABLES — directiva del usuario)

Fuente: directiva del usuario 2026-07-12, **ya aplicada y validada en app-habla S1** (su CLAUDE.md
reglas 10–11 + `docs/GUIA-DE-PRUEBA.html`). Detalle:
`entrega/2026-07-12-propuesta-gmetodo-entregables.md`.

- **`docs/GUIA-DE-PRUEBA.html` (NUEVO en el kit):** plantilla de la **guía de prueba viva** —
  entregable **estándar de todo sprint con UI**. HTML **autocontenido** (cero CDNs, cero
  dependencias; casillas persistidas en `localStorage`; modo claro/oscuro), organizada por
  **bloques** con *qué probar · cómo · qué resultado esperar*, tabla de valores correctos y
  callout "repórtame sí o sí". **Viva, no acumulativa:** cada sprint agrega lo nuevo, complementa
  y **elimina lo que ya no aplica**. Doble propósito: gate de prueba del usuario + entregable a
  usuarios finales. Referencia de implementación: `app-habla/docs/GUIA-DE-PRUEBA.html` (S1).
- **CLAUDE.md del kit — regla 11:** la guía de prueba viva es obligatoria en todo sprint con UI.
- **CLAUDE.md del kit — regla 12:** **PROHIBIDO entregar por artifacts de Claude o cualquier
  plataforma externa.** Todo entregable (guías, reportes, documentos visuales) es un **archivo del
  repo** —HTML autocontenido o Markdown— que el usuario pueda abrir, versionar y llevarse. Sin
  excepciones, ni "para verlo rápido".
- **Fuera del kit (mismo batch):** `estandares/estandares.md` → DoD + regla de entregables (v2.1);
  plantillas de orden y de sprint de la planeadora; `CLAUDE.md` de la planeadora.
- **Delta para las 3 apps ya estampadas** (ds, nutri-kids, hoja-de-vida): se anota en la orden de
  su próximo sprint (política del kit) — la primera vez que toquen UI deben crear su
  `GUIA-DE-PRUEBA.html`. app-habla ya la tiene.

## v1.4.0 — 2026-07-11 (G-Metodo del `/nueva-app habla`: estampado en macOS)

Fuente: primer `/nueva-app` corrido en el Mac (habla). El script canónico era Windows-only
(`estampar-app.ps1`, rutas `C:\Code\`) — las 3 apps previas se estamparon en Windows antes de la
migración (2026-07-10). Aprobado en G-Metodo por el usuario 2026-07-11 — detalle en
`entrega/2026-07-11-propuesta-gmetodo-estampado-macos.md`.

- **`estampar-app.sh` (nuevo):** puerto fiel del `.ps1` a bash (macOS/Linux) — misma secuencia y
  mismas lecciones (allowBuilds de pnpm 11, exit-code por paso vía `set -euo pipefail`, hook
  100755 en el índice K12, `prepare` self-healing, ruleset con checks día 0). Usa `jq` para editar
  `package.json` y `rsync` para copiar el kit. **Vía vigente del pipeline** (el `.ps1` queda para
  la estación Windows de respaldo hr02).
- **Mejora incorporada al flujo (ambos scripts la tendrán; el `.sh` ya):** el estampado escribe
  `.claude/settings.local.json` con `additionalDirectories=[<planeadora>]` — **conexión fija a la
  planeadora automatizada** (antes era paso manual del reporte final; memoria `arranque-app-en-vscode`).
- **`gitignore.plantilla`:** añade `.claude/settings.local.json` — cierra el fleco de la migración
  (app-ds lo tenía a mano; el kit no) para que toda app futura nazca protegida en cualquier máquina.
- **README:** sección de estampado con las dos vías (mac/win) + nota de paridad de scripts.
- **Deuda declarada:** el `.ps1` no recibió aún el paso de `settings.local.json` (divergencia
  anotada; se salda cuando toque estampar en Windows).
- **✅ VALIDADO en el estampado de habla (2026-07-11):** los 10 pasos pasaron sin una sola
  fricción a la primera (Next 16.2.10; commit inicial escaneado por gitleaks — hook vivo K12;
  repo público + ruleset día 0; 46 archivos, cero datos; `settings.local.json` gitignored y no
  pusheado). El puerto macOS queda confirmado ×1.

## v1.3.1 — 2026-07-11 (G-Metodo del cierre S3 ds: K12 — el hook nace ejecutable)

Fuente: `sprints/SPRINT_003-{summary,implementation-log}.md` de app-ds. El gate local de
gitleaks nunca corrió en S1–S3 (hook estampado 100644 — git ignora hooks no ejecutables EN
SILENCIO — y `core.hooksPath` por-clon perdido en los re-clones de la migración al Mac; las 3
apps afectadas, ds pagada en su S3). Aprobado en bloque por el usuario 2026-07-11 — detalle en
`entrega/2026-07-11-propuesta-gmetodo-cierre-ds-s3.md`.

- **githooks/pre-commit:** commiteado **100755** en el kit (`update-index --chmod=+x`) — todo
  estampado futuro copia un hook ya ejecutable.
- **estampar-app.ps1:** §5 inyecta script **`prepare`** = `git config core.hooksPath githooks`
  (self-healing en cada `pnpm install`, sobrevive re-clones); §6 `update-index --chmod=+x` tras
  el add (blinda el modo aunque el host sea Windows).
- **README/CLAUDE.md (regla 7):** hook 100755 + prepare + verificación "si un secreto de prueba
  no es bloqueado, el gate está muerto".
- **Plantilla de orden (planeadora):** el kit-check exige hooks ejecutables y **ACTIVOS**, no
  solo presentes.
- Apps existentes: fix manual por repo (4 comandos, sección 6 de la propuesta) — ds ✅ (S3);
  hoja-de-vida y nutri-kids pendientes del usuario.

## v1.3.0 — 2026-07-10 (G-Metodo: protección GitHub no negociable — checks requeridos día 0)

Fuente: directiva del usuario 2026-07-10 (aprobada en bloque) — detalle en
`entrega/2026-07-10-propuesta-gmetodo-mvp-dos-horizontes.md`.

- **estampar-app.ps1:** la ruleset `main-protegida` nace con la regla `required_status_checks`
  (`quality`/`e2e`/`lighthouse`, `strict: false`) — antes los checks quedaban "para después del
  primer CI verde" y ese después era un pendiente manual eterno. Echo final actualizado.
- **CLAUDE.md regla 6 + repo-app.md:** repo público (GitHub Free solo aplica rulesets en
  públicos) + checks requeridos desde el estampado; **si un sprint añade un job de CI (p. ej.
  `integration`), se añade a la ruleset en el mismo sprint**. Repo privado exigiría upgrade de
  plan por decisión F0 — la protección no se sacrifica.
- Las rulesets de las 3 apps existentes (hoja-de-vida, nutri-kids, ds) se corrigieron por API el
  mismo día (no esperan a su siguiente orden: es configuración de GitHub, no código del repo).

## v1.2.2 — 2026-07-10 (G-Metodo del cierre S2 ds: el estándar 7 tocó la realidad)

Fuente: `sprints/SPRINT_002-summary.md` de app-ds (estreno completo del estándar 7 + validación
contra Groq real). Aprobado en bloque por el usuario 2026-07-10 — detalle en
`entrega/2026-07-10-propuesta-gmetodo-cierre-ds-s2.md`. Solo reglas de skill; cero código nuevo.

- **`ia-embebida.md` §7 (nueva) — Contacto con la realidad:** validar el circuito con **key real
  ANTES del merge** del sprint que estrena/cambia proveedor (CI sigue en mock). Documenta las 5
  clases de fallo que solo el proveedor real muestra (structured outputs varía POR MODELO ·
  presupuestos de modelos de razonamiento · diacríticos vs matching literal · el LLM frasea lo
  que no conoce · varianza del Grader).
- **`ia-embebida.md` §6 (nueva) — El fallback SE ANUNCIA:** degradar nunca en silencio — motivo
  en cubetas honestas + reintento iniciado por el usuario, jamás retries automáticos.
- **`ia-embebida.md` §1:** schemas de ENTRADA de routes con **`.strict()`** (el default de Zod
  acepta-y-descarta en silencio) · en cliente, `schemas.ts` solo con **`import type`** (un import
  runtime mete zod al bundle y revienta el budget de la landing).
- **Checklist del sprint:** +3 ítems verificables por `/deploy-check` (proveedor real pre-merge ·
  fallback anunciado · strict/import-type).
- **`repo-app.md`:** bullet transversal `.strict()` en toda entrada de route (aplica más allá
  de IA).

## v1.2.1 — 2026-07-09 (G-Metodo del cierre S1 ds: Sentry promovido + fricciones K6–K12)

Fuente: `sprints/SPRINT_001-summary.md` de app-ds + estampado de ds. Aprobado por el usuario
2026-07-09 — detalle en `entrega/2026-07-09-propuesta-gmetodo-cierre-ds-s1.md`.

- **Sentry PROMOVIDO al kit (validado ×2: nutri-kids S1 + ds S1):** `instrumentation-client.ts`
  (client-only, metadata-only, **inerte sin `NEXT_PUBLIC_SENTRY_DSN`** — cero ruido en CI) +
  `src/lib/observability.ts` (`reportError`: tipos+metadatos, jamás mensajes crudos) +
  `.env.example`. El script instala `@sentry/nextjs` y deja el build script de `@sentry/cli`
  ignorado en `pnpm-workspace.yaml` (K12 — no subimos source maps). `observability.md`
  actualizado (antes decía "NO viene cableado"). Server-side por ADR cuando haya backend.
- **`vitest.config.ts` (K6/K8):** coverage `include` con `**/*.ts` (v8 tronaba parseando
  `.gitkeep`) + umbral **80% para `src/engine/**`** (regla 2 del CLAUDE.md, antes solo el piso 70).
- **K7 documentado como paso de setup:** el script `test` estampado sigue sin `--coverage` (CI
  verde día 0 sin tests); CLAUDE.md regla 2 y el comentario del config instruyen añadirlo al
  escribir los primeros tests. K9/K10 (eslint lintea `coverage/` y assets generados en `public/`)
  documentados en CLAUDE.md regla 2 — el `eslint.config.mjs` es del scaffold y no se parchea a ciegas.
- **`repo-app.md`:** patrón nuevo — workers ESM/WASM (Pyodide) se sirven desde `public/` como
  module workers, no se bundlean (Turbopack los degrada a clásicos; K11 de ds S1). Enlace al
  patrón de la planeadora.
- **CLAUDE.md § Stack:** default "Next.js 15" → **"Next.js 16+ (lo que estampe create-next-app)"**
  (drift confirmado ×2).
- **estampar-app.ps1:** el echo final ya no pide crear proyecto de Claude Design (contradecía el
  G-Metodo 2026-07-07); ahora recuerda la conexión fija vía `.claude\settings.local.json`.
  SYNOPSIS sin versión hardcodeada.

## v1.2.0 — 2026-07-07 (G-Metodo del cierre S1 nutri-kids: paga las fricciones K1–K6)

Fuente: `sprints/SPRINT_001-summary.md` de app-nutri-kids (el kit requirió cirugía DENTRO del
sprint aunque el estampado ya era limpio). Aprobado por el usuario 2026-07-07 —
detalle en `entrega/2026-07-07-propuesta-gmetodo-batch-s1-nutri-kids.md`.

- **`vitest.config.ts` + `playwright.config.ts` + `tests/setup.ts` (nuevos, K1):** los configs
  que `ci.yml` siempre asumió, con el patrón validado en nutri-kids (jsdom + coverage v8 piso 70;
  e2e móvil Pixel + desktop; webServer `pnpm build && pnpm start` bajo CI).
- **`lighthouse-urls.json` (nuevo) + ci.yml (K3):** el job Lighthouse audita la lista de URLs del
  archivo (default `["/"]`); cada sprint añade sus rutas. Nota Lantern documentada en el workflow
  (LCP simulado castiga SPAs sanas: 3.8s simulado vs 242ms observado).
- **`next.config.ts` (nuevo, K6):** `devIndicators: false` — el indicador de dev tapa la nav
  inferior móvil e intercepta taps en e2e.
- **CLAUDE.md § Stack:** IA embebida = **adapter multi-proveedor** (proveedor por ADR de cada
  app) — alineado con el principio LLM-agnóstico del pipeline.
- **CLAUDE.md regla 10 + skills:** **Claude Design pasa a BAJO DEMANDA** (2 apps seguidas
  validaron design-system.md del builder + gate visual sobre preview). `repo-app.md`: patrones
  confirmados (useSyncExternalStore para localStorage↔React; overlays de primer uso estáticos;
  devIndicators). `observability.md`: aclaración honesta — **el kit NO trae Sentry cableado**
  (K2/K5); se cablea en el S1 de cada app con el patrón de nutri-kids; se promoverá al kit a la
  3ª validación.
- **estampar-app.ps1 (K4):** el commit inicial lee la versión real del CHANGELOG.

## v1.1.5 — 2026-07-06 (limpieza final del estampado #2)

- **`--skip-install` en create-next-app:** su install interno corría ANTES de que el script
  escribiera `allowBuilds` y abortaba en rojo (`ERR_PNPM_IGNORED_BUILDS` + "Aborting
  installation") — inofensivo pero alarmante. Ahora el único install es el del paso 2, ya
  configurado. El próximo estampado debe correr de punta a punta **sin ningún rojo**.

## v1.1.4 — 2026-07-06 (hotfix #4: el kit se mordía la cola con gitleaks)

- **`security-owasp.md` línea ~116:** el ejemplo didáctico de "nunca hardcodear" traía una clave
  con pinta real (`sk-proj-abc123...`) y el **hook pre-commit de gitleaks del propio kit la
  bloqueó** en el commit inicial del estampado #2 (regla `generic-api-key`, entropía 3.69).
  Ejemplo neutralizado a placeholder sin entropía + comentario del porqué. Validación positiva
  doble: el hook `githooks/pre-commit` (nuevo en v1.1.0) **funciona en commits reales**, y el
  `Check` de v1.1.2 volvió a detener el script en el paso exacto.

## v1.1.3 — 2026-07-06 (hotfix #3: pnpm 11 cambió el mecanismo de builds nativos)

- **`allowBuilds` (pnpm 11) además de `onlyBuiltDependencies` (pnpm 10):** pnpm 11 ya no lee la
  lista `onlyBuiltDependencies`; usa el mapa `allowBuilds: {pkg: true}` y **aborta el install**
  (`ERR_PNPM_IGNORED_BUILDS` fatal) si un build nativo no está aprobado, dejando un stub
  "set this to true or false" en el yaml. El script escribe ahora AMBOS formatos (cada versión
  ignora el ajeno) y añade `@tailwindcss/oxide` (Tailwind 4 compila nativo).
- Validación en vivo del `Check` de v1.1.2: el script se detuvo honesto en
  `FALLO: pnpm install (exit 1)` — el patrón de exit codes funcionó a la primera.

## v1.1.2 — 2026-07-06 (hotfix #2 del estampado de nutri-kids — 3 bugs más)

Detectados al correr el script v1.1.1 en la máquina del usuario (el estampado llegó al final
imprimiendo "OK" con el repo git y el remoto ROTOS):

- **Escrituras sin BOM (`EscribirSinBom`):** `Set-Content -Encoding utf8` en PS 5.1 escribe BOM;
  el BOM rompía `pnpm-workspace.yaml` (pnpm ignoró los builds nativos → `ERR_PNPM_IGNORED_BUILDS`),
  `package.json` ("Invalid package.json" de pnpm) y el JSON de la ruleset. Las 3 escrituras
  ahora usan `[IO.File]::WriteAllText` con `UTF8Encoding($false)`.
- **`git init` explícito e idempotente:** el script asumía que create-next-app inicializa git;
  en la máquina real no lo hizo → 4 `fatal: not a git repository`, sin commit inicial, y el
  `gh repo create --source --push` falló en cadena.
- **`Check` de exit codes tras cada comando nativo crítico** (scaffold, installs, git, gh):
  los comandos nativos no disparan `$ErrorActionPreference=Stop`, así que el script imprimía
  "OK repo creado y push hecho" y "OK ruleset activa" sobre pasos fallidos. Ahora un fallo
  detiene el script con el paso exacto.

## v1.1.1 — 2026-07-06 (hotfix del estampado #2, nutri-kids)

- **estampar-app.ps1 — fix de encoding (bug bloqueante):** el script estaba guardado como
  UTF-8 **sin BOM** y con finales **LF**; Windows PowerShell 5.1 lee los `.ps1` sin BOM como
  ANSI, el "—" (em-dash, multibyte) se degradaba a mojibake que incluye una **comilla
  tipográfica** (0x94), PowerShell la trata como comilla real → el parseo del archivo completo
  reventaba (11 errores) y el estampado nunca arrancaba. Fix mecánico, cero cambios de texto:
  re-guardado **UTF-8 CON BOM + CRLF** (validado: 0 errores de parseo).
- **`.gitattributes` nuevo en la raíz de la planeadora:** `*.ps1 text eol=crlf` (evita que git
  o un editor degrade los scripts de vuelta a LF). Regla derivada para todo el pipeline:
  **scripts `.ps1` con caracteres no-ASCII se guardan siempre UTF-8 con BOM** — o se escriben
  100% ASCII.

## v1.1.0 — 2026-07-05 (G-Metodo: batch post-SPRINT_001 de hoja-de-vida)

Fuente: 9 hallazgos del estampado #1 + la CI real del primer sprint
(`memoria/patrones-acumulados.md` de la planeadora).

- **ci.yml:** pnpm 9 → **11** y Node 20 → **22** en los 3 jobs (el scaffold estampa pnpm 11;
  pnpm 11 exige Node ≥22.13).
- **perf-budget.json:** eliminada la propiedad `_comment` (Lighthouse CI rechaza el archivo);
  `interactive` (TTI, deprecada) → `total-blocking-time ≤300ms`; LCP 2500 → 3000ms (efecto
  font-swap documentado en README §Reglas).
- **githooks/pre-commit** (nuevo): gitleaks bloquea commits manuales con secretos — antes solo
  las escrituras de Claude estaban protegidas (hook PreToolUse).
- **estampar-app.ps1** (nuevo, aprobado G-Metodo 2026-07-04): estampado semi-automático
  completo; criterio de aceptación: app recién estampada pasa CI verde en su primer PR.
  Incorpora: scaffold ANTES del kit, `--src-dir`, builds nativos pre-aprobados
  (`pnpm-workspace.yaml`), scripts `typecheck`/`test`/`test:e2e` con `--passWithNoTests`,
  CLAUDE.md desde `ordenes/CLAUDE-md-para-app.md`, ruleset `main-protegida` por API.
- **Sin `ai`/`@ai-sdk/<proveedor>` en el estampado:** el proveedor LLM se decide por ADR en el
  sprint que active IA.
- **README:** bloque de estampado reescrito alrededor del script; reglas nuevas (Lighthouse
  local orientativo; budget y font-swap).
- **CLAUDE.md:** regla 7 actualizada (doble cinturón gitleaks).

## v1.0.0 — 2026-07-02

Versión inicial (migración Cowork→Code): CLAUDE.md, skills, commands, settings con hooks,
ci.yml, perf-budget.json, MANUAL-DE-USO, gitignore.plantilla.
