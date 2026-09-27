---
sprint: 002
app: copiloto-consultor
status: closed
opened: 2026-09-23
closed: 2026-09-26
branch: sprint-002/cuando-que-y-quien-mira
pr: https://github.com/mauriciorincon-ai/app-copiloto-consultor/pull/6
---

# Sprint 002 Summary — Angel Ghost

## Outcome

**Sí en construcción y en el kit; la reunión real queda para el gate del MVP, por decisión del usuario.**
Las tres promesas del sprint funcionan y están medidas:

- **La banda reacciona a lo que se ve.** Una cifra o un término en la pantalla compartida trae su ficha
  sin pregunta: 4 de 4 en el kit, y cero en la agenda, que no trae ninguno.
- **El radar avisa** con símbolo, texto y color: ámbar si la reunión se graba o hay un bot de notas,
  coral si un programa del propio Mac vigila. 0 falsos positivos en el kit.
- **El consultor oye la ficha al oído**: modo solo audio, banda de 44 px, callado sin auriculares.
- **Sugerencia sintetizada con ADR «código primero»**, medida con Apple Intelligence: mediana ~0,8 s y
  p95 ~1 s. Solo se enseña si dice lo que dice la ficha que cita.
- **Diccionario técnico** con WER medido antes y después, y la deuda del S1 pagada.

Lo que **no** se afirma: nada de esto se ha visto en una videollamada de verdad con un cliente. Esa
prueba es el ⭐⭐ del cierre de ciclo (§ Gate ⭐).

## Qué se construyó

| Fase | Qué                                                                                                                                                                                                                                                                                                                                                                          | Dónde                                                                                                                                                                     |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 0    | Delta del kit v1.27.1 + v1.28.0. Deuda del S1: **M1** CSP real · **M2** Sesión ya no miente · **M4** la banda vuelve · **M9** el efímero ve `~/Library` y archivos que crecen · **M10** el audio no se reinterpreta a ciegas · `por_silencio` cableado · `corte::Informe` y `corpus::Documento` al contrato · campos huérfanos contados por un gate · `cargo test --release` | `tauri.conf.json` · `capture/nativo.rs` · `disparo/` · `escucha/` · `contrato.rs` · `tests/unit/contrato-con-lectores.test.ts` · `tests/unit/auditoria-con-sitio.test.ts` |
| 1    | **B3 — el diccionario técnico**: corrección determinista después de transcribir; semilla con la jerga de datos más los clientes del corpus; persiste solo lo del usuario (ADR 002 enmendado, ADR 009 «un idioma por pista»)                                                                                                                                                  | `src-tauri/src/diccionario/`                                                                                                                                              |
| 2    | **C15 — el modo solo audio**: `AVSpeechSynthesizer` por el puente de Swift, el candado de los auriculares, silencio mientras alguien habla, banda de 44 px                                                                                                                                                                                                                   | `nativo/Habla.swift` · `src-tauri/src/habla/`                                                                                                                             |
| 3    | **C8 — leer la pantalla solo cuando cambia**: ScreenCaptureKit y Vision, huella por cambio, OCR ≤ 1/s, refuerzo al retriever; las pantallas de los «porqués» y las teclas `⌃⌥`                                                                                                                                                                                               | `nativo/Pantalla.swift` · `src-tauri/src/pantalla/`                                                                                                                       |
| 4    | **C14 — el radar**: ámbar (aviso de grabación y bots por OCR) y coral (procesos del propio Mac contra un catálogo versionado y con fuente), MDM local, gate de «solo este Mac»                                                                                                                                                                                               | `src-tauri/src/radar/` · `data/radar/` · `tests/unit/radar-solo-este-mac.test.ts`                                                                                         |
| 5    | **C7 — la sugerencia**: ADR 010 y 011; proveedores del sistema → API opt-in con la clave en el Llavero (Claude · Gemini · Groq) → `mock`; anonimización local; techo de 6 s; USD 10/mes; pantalla IA                                                                                                                                                                         | `src-tauri/src/sintesis/` · `nativo/{Sintesis,Red,Llavero}.swift` · `src/pantallas/Ia.tsx`                                                                                |
| 6    | El modelo del sistema **medido**, y la regla que eso exigió (`sintesis/fiel.rs`) · guía v4 · manual · kit en CI · reduced-motion ampliado · auditoría · este summary                                                                                                                                                                                                         | `docs/GUIA-DE-PRUEBA.html` · `tests/unit/guia-cuadra.test.ts`                                                                                                             |

## DoD — checklist

- **Testing** ✅ — vitest **257**, `cargo test --lib` **340** + 22 de integración, e2e **107** (87 → 107:
  los 20 nuevos de reduced-motion). Gates nuevos de la fase 6 y la auditoría, cada uno con su rojo:
  `guia-cuadra`, `banda-sin-promesas-vacias`, `capabilities`, `version-minima`, el programa lanzado
  (en `contador-de-red`) y el efímero en marcha que ejerce pantalla, voz y síntesis. **Regla 19:** cada evento nuevo Rust→TS entra por
  `contrato.rs`, con fixture del serializador real, tipo generado y test de suscripción: pantalla,
  radar, voz, sugerencia, estado de la IA, `Informe` y, desde la auditoría, `QueSabeTranscribir` y cada
  motivo de «nadie redacta». Lo que nadie lee ya no cruza: el evento de un turno no lleva el texto del
  cliente, y el gate de campos sin lector lee los tipos del sprint con `DEUDA` vacía. `catch_unwind` del PDF corrido con
  `--release` (fase 0). **Kit v1 en CI:** WER con y sin diccionario (sale «NO SE MIDIÓ» en el runner,
  que no tiene modelos de voz, y lo dice), nDCG con pantalla, radar coral y ámbar con Vision, grounding
  y latencia de la sugerencia con `mock`.
- **CI/CD** ✅ — `quality` · `e2e` · `build-escritorio` en `success` propio en cada push de la fase 6.
  `test:e2e` sin `--pass-with-no-tests`, `cargo clippy --locked -- -D warnings` dentro de
  `build-escritorio`. **Este summary viaja en el PR.**
- **Observabilidad** ✅ — logs metadata-only. `tests/unit/logs-de-la-sintesis.test.ts` exige que
  ningún `println!` de la síntesis nombre una variable de contenido (rojo con `{turno}` plantado), y
  la canaria del cliente recorre una sesión completa sin aparecer en el log (S1). Pantalla, radar y voz
  imprimen solo cuentas y estados, revisado a mano (`[pantalla] leída en N ms · N líneas`,
  `[radar] ámbar: grabación sí · N bots`).
- **Seguridad** ✅ con un aviso aceptado:
  - `pnpm audit` limpio; `cargo audit` sin vulnerabilidades, con **9 avisos**, entre ellos `lru` 0.16.4
    _unsound_ (RUSTSEC-2026-0253). Llega por tantivy 0.26.2, la última versión, que lo fija: no hay
    subida posible (deuda, abajo).
  - gitleaks en cada commit; CSP real (M1) comprobada con la app viva.
  - La clave del API vive en el Llavero (`kSecAttrAccessibleWhenUnlockedThisDeviceOnly`), y su ida y
    vuelta se verificó en vivo.
  - La única puerta de red nueva es efímera y está declarada, contada por el gate del contador (3
    puertas); y el único programa que la app lanza es `/usr/bin/profiles` (gate nuevo).
  - **Manifiesto de comandos por ventana** (auditoría, M5): la banda puede 13 comandos y ninguno de la
    clave, el API, el corpus ni la sesión; el relleno, uno.
  - **La bóveda del API** tapa nombres, teléfonos y correos en cualquier posición del texto real, y
    conoce a los clientes por su nombre (auditoría, A1 y A2). El corte alcanza la petición en camino
    (M2) y el tope del mes cobra lo tardío (M10).
  - `verify:ephemeral` protege `radar/`, `sintesis/`, `habla/` y `pantalla/`.
- **Performance** ✅ — medido en el kit:
  - OCR mediana 87 ms (techo 1000), ≤ 1 lectura/s.
  - Sugerencia mediana 0,79–0,83 s, p95 0,99–1,04 s (presupuesto 4 / 6).
  - Binario de release: 12,06 MB, +1,01 MB sobre el S1 (§ Métricas).
  - CPU de una lectura de pantalla: **mediana 172 ms de CPU** en un Mac con Apple silicon (≤ 17 % de un
    núcleo a una lectura/s); en la máquina virtual de la CI, sin aceleración, 886 ms —la misma VM en que
    Vision tarda 617 ms de reloj—.
  - «La voz empieza ≤ 1 s tras la ficha»: **se mide en vivo** —el puente guarda el `didStart` y el log
    dice «empezó a sonar a los N ms»—; `cargo test` no puede medirlo (no hay bucle principal que atienda
    al sintetizador). El número sale en la parada h1 del gate del MVP.
- **UX/A11y** ✅:
  - axe sobre todos los estados nuevos, en los dos temas.
  - Símbolo + texto + color en ámbar y coral, cada uno con su símbolo propio.
  - Teclado de punta a punta; reduced-motion ampliado a los estados del S2.
  - Fidelidad: **116 encuadres**, ninguno sobre el umbral.
  - Miradas 16, 16-bis, 17, 17-bis, 17-ter registradas con veredicto antes de construir encima. Desde
    la fase 3, las de texto viajan al gate del MVP por decisión del usuario (`docs/diseno/README.md`):
    **no es una aprobación, y se dice**.
- **IA embebida** ✅ — ADR 010 desde la plantilla «código primero» y ADR 011 con el orden de
  proveedores. Mock de primera clase. Esquema cerrado validado en Rust. **Grounding en dos mitades:**
  que cite una ficha dada (la regla que no compila) y que diga lo que esa ficha dice (`fiel.rs`).
  Anonimización antes del API, costo por reunión y por mes, techo visible, y la ficha siempre primero.

## Métricas técnicas

| Métrica del plan                       | Meta                      | Medido                                                                           |
| -------------------------------------- | ------------------------- | -------------------------------------------------------------------------------- |
| Cifra en pantalla → ficha sin pregunta | la correcta               | **4 de 4** (kit de pantalla, en CI con Vision)                                   |
| nDCG@5 con refuerzo de pantalla        | mejora                    | 0,626 → **0,700**; con la peor pantalla delante, el kit v0 **0,819** (mín. 0,80) |
| OCR solo ante cambio                   | ≤ 1/s                     | mediana **87 ms**, huella mínima entre diapositivas 144 celdas (umbral 12)       |
| Radar coral                            | 0 FP · 100 %              | **0** en 82 procesos · **34/34** filas                                          |
| Radar ámbar                            | aviso y bot               | los dos, con Vision; **0** avisos en 5 reuniones sin grabar                      |
| WER con diccionario                    | no empeora                | es-mezcla 0,458 → **0,417** · en-mezcla 0,348 → **0,261** · controles sin cambio |
| Sugerencia local                       | ≤ 4 s mediana · ≤ 6 s p95 | **0,79–0,83 s · 0,99–1,04 s** (Apple Intelligence, este Mac)                     |
| Sugerencia sin fuente descartada       | test                      | sí, más la regla de fidelidad: **11–13 de 17** pasan con el modelo real          |
| Voz empieza tras la ficha              | ≤ 1 s                     | medido **en vivo** (log `[habla] empezó a sonar a los N ms`); el número, en la parada h1 del MVP |
| CPU de la lectura de pantalla          | anotada                   | mediana **172 ms de CPU** por lectura · a ≤ 1 lectura/s, menos del 17 % de un núcleo |
| Peso del binario | anotado | ejecutable de release **12,06 MB** (S1: 11,05 → **+1,01 MB** por pantalla, radar, voz, síntesis y el puente de Swift) · `.app` 12 MB · `.dmg` **5,34 MB** (S1: 4,88) — `pnpm tauri build` local, 2026-09-26 |

## Gate ⭐ — diferimiento y contrapesos

| Contrapeso                     | Evidencia (archivo, cuenta medida, corrida)                                                                                                                                                                                                                                  |
| ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Pasada de capturas del builder | **116 encuadres** producto contra maqueta, en los dos temas y los dos idiomas, leídos como imagen · `docs/fidelidad/S2-banda.html` · `docs/fidelidad/S2-cuaderno.html` · `pnpm fidelidad` 2026-09-26 tras el último cambio de copy: ninguno sobre el 0,15 %, ningún desborde |
| e2e de `reduced-motion`        | **42 pruebas** (10 encuadres × 2 modos + la forma del árbol, en 2 proyectos) · `tests/e2e/reduced-motion.spec.ts` · rojo demostrado escondiendo la sugerencia solo con «reduce» · corre en el job `e2e`                                                                      |

**⭐ diferido: 61 pruebas ⭐ al acumulado del ciclo (S1: 33 · S2: +28), 9 paradas ⭐⭐; la guía v4 tiene 72.** Lo cuenta la
guía v4, no el plan. El plan esperaba 7 nuevas: las 7 están, y el resto son las que un humano tiene que
ver de cada función (sin permisos, sin auriculares, el coral de su Mac, el proveedor externo con su
clave). **El ⭐⭐ pasa del techo de ~20 min en unos cinco (~25 min), declarado en la guía.**

Qué viaja al gate del MVP, además del ⭐⭐:

- los textos de todas las miradas desde la fase 3, incluidas las siete filas de la mirada de cierre de
  la fase 3 y la 18;
- el radar con una reunión grabada de verdad (f3: el texto real del aviso de Meet y Teams);
- el coral del Mac del usuario (Chrome Remote Desktop);
- la sugerencia con un turno de cliente real.

## Decisiones no anticipadas

- **ADR 009 — un idioma por pista.** SpeechAnalyzer no sostiene la mezcla es/en en una pista: la
  pantalla de Idioma lo declara en vez de prometerlo.
- **ADR 010 — la sugerencia, código primero**, con su enmienda de la fase 6: citar bien no basta.
- **ADR 011 — proveedores y minimización.** **MLX no se construye**: Apple Intelligence cumple el
  presupuesto con holgura, y el ADR lo condicionaba a que no lo hiciera.
- **ADR 002 enmendado:** el diccionario del usuario persiste, sin nombres de clientes.
- **Las teclas pasan de `⌘⇧` a `⌃⌥`**: en Zoom, `⌘⇧A/V/T` silencian el micrófono, apagan la cámara y
  pausan la pantalla compartida.
- **Plan de miradas:** los textos van a un solo gate humano con el MVP (decisión del usuario,
  2026-09-26; memoria `textos-al-gate-del-mvp`).
- **El prompt de la sugerencia** pide escribir en el idioma de la ficha, no en el de la pregunta: una
  línea traducida no se puede cotejar (limitación declarada en el manual).
- **ADR 012 (lectura de pantalla), 013 (el radar y lo que no hace) y 014 (la voz al oído)**, escritos en
  la auditoría (M9): las decisiones de producto que solo vivían en la bitácora.
- **La pista del cliente se elige y nace en español** (auditoría A4, decisión del usuario sobre el
  valor por defecto). **La maniobra genérica se construyó a medida** (M15, decisión del usuario): el
  puente a lo más cercano que sí tienes.
- **`minimumSystemVersion` 26.0** (M1): el puente enlaza FoundationModels de forma fuerte.
- **Los embeddings no se decidieron en el S2**: pasa al S3 con nDCG@5 0,82 sin ellos (ADR 008).

## Bugs + resoluciones

Los que importan; el detalle, con sus rojos, está en la bitácora.

- **Los 68 hallazgos de la auditoría** (§ Auditoría), entre ellos los cuatro altos: la bóveda del API
  dejaba salir nombres y teléfonos, sus «conocidos» eran nombres de archivo, el diccionario estropeaba el
  castellano corriente y la pista del cliente estaba fijada en inglés.

- **La sugerencia «fundada» decía cosas falsas** («el taller de cierre va aparte»): el contador medía
  citas, no contenido. Se paga con `sintesis/fiel.rs` (fase 6).
- **Dos pruebas de la guía heredada no podían pasar nunca:** la pregunta del cliente dicha por el
  micrófono, y «nada en tu corpus» para una pregunta con ficha. Reescritas.
- **Tres corridas de CI en rojo sin mirar** (fase 3) → desde entonces, `gh pr checks` tras cada push.
- **Pruebas decorativas cazadas al exigirles el rojo:** `puente-baja` (vi.fn), «la ficha siguiente la
  quita», la mutación con `sed` que no se aplicó, el corte de comentarios del gate del radar que se
  comía las URLs, y la primera versión del reduced-motion S2, que fallaba por selectores y no por
  movimiento.

## Qué salió bien / qué generó fricción

- **Bien:** medir el modelo real antes de afirmar nada destapó el defecto más caro del sprint en diez
  minutos. Los kits con Vision de verdad corren en CI. La regla que no compila funcionó: el primer
  intento de fabricar una `Sugerencia` a mano dio E0451.
- **Fricción:** las miradas de copy consumieron más tiempo que su valor, y el usuario las movió al gate
  del MVP. Los tests que solo cuentan («17 de 17 fundadas») dieron una tranquilidad falsa hasta que
  alguien leyó las salidas.

## Sugerencias de mejora al método

1. **La mirada en una matriz** (archivo · botón · qué mirar exactamente · qué respuesta se espera), en
   la misma fila, nunca como preguntas sueltas debajo. Lo pidió el usuario y se aplicó desde la
   mirada 17.
2. **Menos paradas:** las segundas vueltas de una mirada se juntan en el cierre de fase.
3. **Los textos al gate del MVP:** el juicio de copy no bloquea la construcción; lo vigilan los gates
   automáticos (diccionario fiel a la maqueta, fidelidad, maquetas que caben).
4. **`gh pr checks` después de CADA push**, no al cierre.
5. **Medir el área de desplazamiento** (scrollHeight) en la pasada de capturas: el desborde de 15 px no
   lo vio ningún número.
6. **Leer las salidas del modelo, no solo contarlas:** un kit de IA que cuenta citas válidas debe
   imprimir también unas cuantas salidas para leerlas al menos una vez por sprint.
7. **El rojo de cada gate cazó 3–4 pruebas decorativas por fase.** La regla 15 es el gate más rentable
   del método.
8. **Una guía de prueba se relee contra la arquitectura al heredarla:** la d3 del S1 pedía algo que el
   diseño prohíbe, y nadie la había corrido.

## Deuda técnica aceptada

| Qué | Por qué | Pago |
|---|---|---|
| `lru` 0.16.4 *unsound* vía tantivy 0.26.2 | la última versión de tantivy lo fija; no hay subida posible | cuando tantivy suba de `lru` (vigilado por `cargo audit`) |
| MLX | Apple Intelligence cumple; MLX solo hacía falta sin él (ADR 011) | roadmap (Macs sin Apple Intelligence) |
| El texto exacto que salió al API, legible en IA | IA enseña cuántos datos se taparon, no el texto (ADR 011, enmendado) | sprint 003 |
| Los interruptores de IA y el idioma elegido por pista no persisten entre arranques | declarado en el manual; hoy viven en memoria | sprint 003 (preferencias) |
| La decisión sobre los embeddings | el S2 no la tomó; nDCG@5 0,82 sin ellos | sprint 003 (ADR 008) |
| La maniobra a medida, siguientes pasos (la unidad que falta, la ficha del cliente, lo comprometido) | el puente es el primero; el §10 nombra los demás | sprint 003 |
| Ver un `invoke` prohibido rechazado desde la consola de la banda (M5) | la consola del webview no es accesible desde la sesión del builder; lo cubren `tauri-build` y el gate | gate del MVP (prueba manual con las herramientas de desarrollo del webview) |

## Auditoría

`/audita-sprint` con **auditor independiente** (un subagente que no construyó el sprint, sobre el diff
`main...HEAD` en `b55f1fe`, solo lectura). Artefacto: `sprints/SPRINT_002-auditoria.md`, con cada
hallazgo, su `archivo:línea`, su ajuste y **el commit que lo pagó** (lo vigila `auditoria-con-sitio`).

- **Fase 1:** 40 hallazgos — **0 críticos · 4 altos · 15 medios · 21 bajos**. Veredicto «requiere
  ajustes». El constructor verificó en el código los dos altos más serios antes de presentarlos.
- **Aprobación del usuario:** pagar **los 40**; construir M15; la pista del cliente nace en es-ES.
- **Fase 2:** los 40 pagados, **cada uno con su test en rojo antes del verde**, en cinco commits
  (`c474f96` · `1eea670` · `7a50222` · `3090483` · `823c9f6`). En la Fase 2 aparecieron **tres bajos
  más** (B22–B24, del subagente de los ADR), pagados también. Y la segunda pasada de la casilla 4
  trajo **25 bajos más** (B25–B49), pagados en `eefc7c3`: **68 hallazgos, 68 pagados**.
- **Los altos:** A1 la bóveda dejaba salir nombres y teléfonos · A2 los «conocidos» eran nombres de
  archivo · A3 el diccionario estropeaba el castellano corriente (y se comía la «y» delante de un
  cliente, que destapó el test nuevo) · A4 la pista del cliente fijada en inglés.
- **Casilla 4 dos veces:** la primera en la Fase 1; la segunda, después del último ajuste, con otro
  auditor independiente que además siguió cada ajuste hasta sus frases hermanas. Encontró 25 frases
  falsas hoy (B25–B49): **once las creó un ajuste de la Fase 2 y dos la guía v4**, que es justo lo que
  la segunda pasada existe para cazar. Una no era solo texto: **B25**, el log del corte leía el contador
  de red después de ponerlo a cero y escribía siempre «red 0 B»; se pagó en el código, con un test de
  orden visto en rojo. Al cerrar, la casilla 12 de este release-check tenía la hermana de B41 (omitía
  `acople.json`), corregida.
- **Casilla 5:** 10 campos sin lector encontrados por el auditor; pagados (B9–B12) y el gate que no los
  veía, arreglado (M6).

## `/release-check` (perfil escritorio)

**✅ Pasa 11/12 · ⚠️ 1 aviso · ❌ 0.**

| # | Casilla | Resultado |
|---|---|---|
| 1 | Tests | ✅ vitest 257 con cobertura (90 % statements) · e2e 107, cero flaky · `cargo test --locked` en `build-escritorio` |
| 2 | Tipos y lint | ✅ `pnpm typecheck` · `pnpm lint` · `cargo clippy --locked --all-targets -- -D warnings` limpio (y en CI) |
| 3 | Build del binario | ✅ `pnpm build` · `cargo check --locked` en CI · `pnpm tauri build` local: 12,06 MB, `.dmg` 5,34 MB. **Cambios en `tauri.conf.json` declarados:** CSP real (M1) y `minimumSystemVersion` 26.0 (auditoría M1). El `--release` de los tests que dependen del binario corrió en la fase 0 |
| 4 | Permisos TCC | ✅ `NS*UsageDescription` es/en para micrófono, audio del sistema y reconocimiento de voz (la captura de pantalla no tiene clave de uso: la pide macOS); la app arranca y se usa sin permisos |
| 5 | Ventana protegida | ✅ `content_protected` con su test; la parada ⭐⭐ 1 de la guía es la invisibilidad en Meet |
| 6 | No persistencia | ✅ `pnpm verify:ephemeral` (estático) y la sesión completa en marcha, que ahora ejerce pantalla, voz y síntesis: cero intrusos; fuga inyectada (S1) y canaria en el log vigentes; contador en 0 B en modo local |
| 7 | Seguridad | ⚠️ `pnpm audit` limpio · `cargo audit` **sin vulnerabilidades, 9 avisos** (`lru` *unsound* vía tantivy, sin subida posible: deuda) · gitleaks · CSP · **capabilities mínimas por ventana (M5)** |
| 8 | Observabilidad | ✅ metadata-only; canaria del cliente en el log y `logs-de-la-sintesis` |
| 9 | A11y y diseño | ✅ axe en los estados nuevos · teclado · dos temas · `design-sync/` regenerado y espejo verde |
| 10 | Documentación y cero enlaces | ✅ manual y guía v4 al día · barrido `git grep -nE "vercel[.]app\|workers[.]dev\|pages[.]dev" -- ':!pnpm-lock.yaml'` vacío tras cada `git add` |
| 11 | Checks del PR | ✅ `quality` · `e2e` · `build-escritorio` en `success` propio en cada push (el último se comprueba antes de entregar) |
| 12 | El disco en runtime | ✅ inventario: índice del corpus (700), `diccionario.yaml` (600), `costo-del-mes.json` (600, escrito por renombrado atómico) y `acople.json` (600, solo mientras la banda está acoplada: se borra al devolver la ventana, S1); nada más, y el gate en marcha lo demuestra |

**Decisión: MERGE OK** en cuanto la CI del último push cierre en verde. El ⭐ está diferido al gate del
MVP, como declara el plan (`gate_estrella: diferido`).

## Archivos clave

`src-tauri/src/sintesis/mod.rs` · `src-tauri/src/sintesis/fiel.rs` · `src-tauri/src/pantalla/` ·
`src-tauri/src/radar/` · `src-tauri/src/habla/` · `src-tauri/src/diccionario/` ·
`src-tauri/src/contrato.rs` · `src/componentes/Banda.tsx` · `docs/GUIA-DE-PRUEBA.html` ·
`decisions/010-sintesis-codigo-primero.md`

## Cómo probar

`pnpm tauri dev`, y la guía `docs/GUIA-DE-PRUEBA.html` (doble clic): filtro **Gate corto ⭐⭐**, 9
paradas. Hace falta:

- auriculares;
- Terminal (el cliente de prueba habla con `say`);
- `docs/kit-de-prueba/pantalla/meet-de-prueba.html` abierta en Chrome;
- Apple Intelligence activado.

Automático: `pnpm test` · `pnpm test:e2e` · `cd src-tauri && cargo test`.
