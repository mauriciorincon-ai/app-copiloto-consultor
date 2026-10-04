---
sprint: 004
app: copiloto-consultor
status: closed
opened: 2026-10-04
closed: 2026-10-04
branch: sprint-004/el-ensayo (fases 0 a 4) · sprint-004/cierre (registro del corte) · sprint-004/fase-5 (cierre)
pr: https://github.com/mauriciorincon-ai/app-copiloto-consultor/pull/10 · /pull/12 · /pull/13
ciclo: "H2, sprint 1 de 3. No cierra ciclo. El Acto 2 del H1 (prueba en vivo y ⭐⭐ del S3) va aparte"
gate_estrella: diferido (Opción A)
---

# Sprint 004 Summary — Angel Ghost

## Outcome

**Sí en construcción, con un corte declarado: el ensayo y la banda arriba están construidos y probados por la
suite. El ensayo con tu voz, en tu Mac, no se ha visto: esa corrida se aplazó y viaja al ⭐ del MVP.**

- **El ensayo (C18).** Eliges cliente y propuesta. La app arma, por reglas publicadas, las preguntas que ese
  cliente probablemente hará, y suma un catálogo de objeciones de datos, BI e IA con su fuente. Te las lee,
  te escucha **solo por el micrófono** y, al final de cada respuesta, te enseña:
  - la evidencia de tu corpus que usaste y la que tenías sin usar, sin puntaje;
  - cuánto tardaste;
  - tu ritmo en palabras por minuto;
  - las muletillas que la transcripción conserva.

  Se guarda cifrado con la llave y la retención de tus notas, y ves tu progreso por cliente. Un modelo
  enriquece el banco solo si lo enciendes; el banco por reglas es el camino y el respaldo.
- **La banda arriba (C1').** La banda nace bajo la barra de menús, junto a la cámara. Con la sesión iniciada,
  la ventana de la reunión **baja y se encoge, y se devuelve entera** (posición y tamaño). Arriba o abajo, con
  ⌃⌥B; arriba de fábrica. Abajo, todo como en el H1.
- **El ciclo H2, encarrilado.** Delta del kit v1.32→v1.36. `cargo test` a secas ya no toca el Mac (regla 25,
  con un centinela). Guía v6 con el ⭐⭐ del H1 intacto. Kit v3, manual y `design-sync/` al día.

**Lo que no se afirma.** Nadie ha ensayado todavía con su voz en la app de verdad. Tampoco se ha visto la voz
de la app por los altavoces sin entrar al micrófono, ni si los 2,5 s de silencio cortan pausas reales. El
aviso del Llavero, la tarea de launchd y Touch ID con los ensayos tampoco se han visto. La banda arriba se vio
acoplar y devolver la página de prueba de Meet en Chrome (228 ms y 123 ms); con Zoom, Teams, una pantalla
externa y la pantalla completa no se ha probado.

## El corte, declarado (método v1.24.0)

| Qué | Dónde quedó | Por qué |
|---|---|---|
| El PR #10 se mergeó (`0009dba`, 2026-10-04) con las **fases 0 a 4** | el corte se declaró en el mismo acto, en un comentario del #10; su cuerpo, al día con las casillas (B3) | el usuario mergeó antes de la fase 5 |
| El PR #12 se mergeó con **solo el registro del corte** (`2f48a58`) | un comentario en el #12 dice qué trajo y qué no (B3) | se abrió para el cierre y entró antes de tenerlo |
| **Fase 5, auditoría, `/release-check` y este summary** | PR #13, rama `sprint-004/fase-5` | — |
| **La corrida en vivo de la fase 4** (filas 2 a 5 de la regla 22: micrófono fuera de reunión, Llavero, launchd y desbloqueo) | aplazada, **viaja al ⭐ del MVP**; se recomienda correrla antes de que el S5 toque el ensayo (≈ 5 min, con las mismas matrices) | el usuario preguntó si se podía aplazar (2026-10-04) |

**Lo que cuesta:** `main` tiene un ensayo que el usuario no ha usado con su voz. **El riesgo real es bajo:** la
app no se distribuye (binario sin firmar, solo en este Mac). La CI corre la sesión efímera entera con un ensayo
dentro, con el motor de voz de verdad. Lo que falle se arregla por PR normal.

## Qué se construyó

| Fase | Qué | Dónde |
|---|---|---|
| 0 | Delta del kit v1.32→v1.36 (reglas 10, 15, 17, 18, 22, y las nuevas 23 y 24; comandos; `demo-rojo.sh`; `verificar-dependencias`; hooks que fallan cerrado; homepage del repo). **Regla 25:** marcas `hardware:` y `en_vivo_`, el centinela `AG_SIN_HARDWARE` en 15 entradas nativas y dos pasos de CI. ADR 019 y enmiendas 004 y 002. Maquetas `ensayo.html` y `posicion.html` · arriba | `CLAUDE.md` · `src-tauri/src/hardware.rs` · `.github/workflows/ci.yml` · `decisions/` · `docs/diseno/` |
| 1 | **La banda arriba:** geometría pura por borde con el área útil del monitor; ⌃⌥B y la preferencia; el aviso de la primera vez; el acople «baja y se encoge» solo sobre la ventana de la reunión, con devolución doble; `ax::mover`, la única escritura de `AXPosition` | `ventana/geometria.rs` · `acople/` · `prefs.rs` · `Sesion.tsx` · `asa.ts` · `Relleno.tsx` |
| 2 | **El banco de preguntas:** seis reglas y diez objeciones con fuente (`data/ensayo/`), `ensayo/banco.rs` puro y protegido, el acento del modelo con bóveda y `fundar` (`ensayo/enriquecer.rs`), «Enriquecer el banco» en IA, apagado. Kit v3 | `src-tauri/src/ensayo/` · `data/ensayo/` · `docs/kit-de-prueba/ensayo.json` |
| 3 | **La sesión de ensayo:** máquina de estados pura con el reloj por parámetro, el oído solo del micrófono y sordo mientras la voz lee, la evaluación determinista, la pantalla `Ensayo.tsx` y el corte de 12 piezas | `ensayo/{sesion,oido,evaluacion,mod}.rs` · `src/pantallas/Ensayo.tsx` |
| 4 | **Resultado, progreso y lo que queda:** enmienda 4 del ADR 015; `ensayos/` cifrado (`ensayos.rs`, molde de la bandeja); el vencimiento por launchd; el informe, exportar, el progreso y borrar; Honestidad cuenta el ensayo; la sesión efímera en marcha con un ensayo dentro | `ensayo/guardado.rs` · `ensayos.rs` · `reunion.rs` · `Honestidad.tsx` |
| 5 | Manual · guía v6 · kit v3 en la guía · `design-system.md` v1.16.0 · `design-sync/` con dos tarjetas nuevas · nota del brochure | `docs/` · `design-system.md` · `design-sync/` |
| Auditoría | **82 hallazgos** de cuatro auditores independientes, todos pagados (§ Auditoría), y la segunda casilla 4 con otro auditor | `f3dd820` · ⟨commit de la segunda pasada⟩ |
| `/release-check` | `pnpm tauri build` se negaba a construir: Tauri en npm (2.12) y en Rust (2.11) no casaban. Crates subidos y un gate nuevo, `tauri-a-la-par` (§ Bugs) | `src-tauri/Cargo.lock` · `tests/unit/tauri-a-la-par.test.ts` |

## DoD — checklist

- **Testing** ✅
  - Vitest **455** (54 archivos), con cobertura: 92,6 % de sentencias y 81,4 % de ramas.
  - `AG_SIN_HARDWARE=1 cargo test --locked`: lib **575** (+2 ignorados) · `contra-el-mac-de-verdad` **16** (+14 que
    corre la CI) · `ghost` **5** · `puerta` **14**. Ni un aborto del centinela.
  - e2e **⟨N⟩** en la CI, con axe.
  - **El banco:** kit v3, precisión y recall 1,000 por regla en los dos casos (es y en). Es un piso, no una
    prueba de calidad: las reglas y el kit los escribió el mismo constructor, y el LEEME lo dice.
  - **La evaluación:** 15 fichas, precisión 1,000 y recall 0,714. Dos respuestas fallan a propósito: es lo que
    corrige «Sí lo dije».
  - **El ensayo no abre la pista del sistema ni la pantalla** (`el_oido_solo_abre_el_microfono`, y el test de la
    puerta de la captura).
  - **Regla 19:** cada evento nuevo Rust→TS entra por `contrato.rs` (franja, ensayo en sus fases, progreso,
    preparación, el estado de la escucha en un ensayo, lo que salió para el banco).
  - **Regla 25:** `cargo test` a secas no toca el Mac. Su primera corrida cazó un test que desde el S2 encolaba
    una frase en la voz del sistema.
  - Cada gate nuevo, con su rojo en el mismo commit (`scripts/demo-rojo.sh`). `gh pr checks` tras cada push.
- **CI/CD** ✅ — `quality` · `e2e` · `build-escritorio` en `success` propio en cada push. **Corrieron por primera
  vez en este sprint:** `verificar-dependencias` (en `quality`) y los dos pasos de `cargo test` de la regla 25 (a
  secas con el centinela, y `--include-ignored --skip en_vivo_`). El rojo de la canaria del ensayo (B9) se vio en
  la CI, en el PR desechable #14. **Este summary viaja en el PR.**
- **Observabilidad** ✅ — logs con solo metadatos. En marcha, la CI busca en el log de la sesión entera la
  canaria del cliente **y un término plantado en la propuesta del ensayo**; ninguno sale. Si el proveedor falla,
  el log lo dice sin contenido (B21).
- **Seguridad** ✅
  - `pnpm audit` limpio · `cargo audit` en la CI · gitleaks en cada commit, y el hook falla cerrado.
  - Los comandos del ensayo y de la franja, solo en la ventana principal (`SENSIBLES`).
  - `ensayos/` cifrado con la llave de tus notas, 600 en 700, dentro del inventario. En marcha, cada archivo de
    ahí se abre como ensayo y sin la canaria; un intruso es rojo.
  - El acento del modelo pasa por la bóveda y solo con el API que enciendes tú.
  - **`AXPosition` se escribe en un solo sitio** (`ax::mover`, con test de barrido), solo sobre la ventana de
    la reunión, y el acople va por turnos (A3).
- **Performance** ✅
  - La siguiente pregunta, **< 1 s** tras Enter (prueba de punta a punta sin Mac).
  - Evaluar una respuesta, **4 ms** en el kit (presupuesto 500 ms).
  - Acople arriba **228 ms** y devolución **123 ms**, medidos en vivo con Meet (presupuesto 300 ms).
  - Binario: § Métricas.
- **UX/A11y** ✅
  - axe en los estados nuevos, dos temas · teclado (teclas de ventana, sin repetir al mantener, sin robar Enter a
    un enlace) · símbolo + texto + color (el reloj lleva su símbolo; «tenías y no usaste» no es un rojo de
    castigo).
  - Fidelidad: **284 encuadres**, ninguno sobre el umbral, ningún desborde (el gate cazó uno en la auditoría).
  - **Las dos miradas de DECISIÓN, vistas y con veredicto** antes del código (§ abajo). Las FORMA y TEXTO
    nuevas, maquetadas y registradas «no vistas»: **18 filas** en la tabla U de la guía, para el gate del MVP.
- **IA embebida** ✅ — **sin proveedor nuevo.** ADR 019 «el ensayo, código primero». El acento usa el proveedor
  de ahora (modelo del sistema, o el API si lo encendiste), opt-in y apagado; esquema cerrado; `fundar` tira lo
  que nombra una sección que no existe. El kit imprime las salidas del mock: 2 fundadas y 1 descartada. Una
  llamada cuesta ≈ US$0,0014 con Claude Haiku y ≈ US$0,0004 con Groq.
- **Manual** ✅ — «El ensayo» y «Arriba o abajo», cada afirmación con su gate (tabla en la bitácora). Lo no
  probado en vivo se dice.
- **Guía v6** ✅ — **118 pruebas**: 17 nuevas, 11 «Mejorado en S4» y 90 heredadas sin cambio; las 101 de la v5,
  íntegras. Prefijo `ag-s4-`. ⭐ **95**, ~179 min (S1 33 · S2 28 · S3 23 · **S4 11**). El ⭐⭐ del H1, intacto:
  9 paradas, ~20 min. Acumulado del H2: **11 ⭐**, cada una con su candidatura (6 sí · 5 no). Formas y textos del
  H2: 18 filas, ~11 min.
- **ADRs** ✅ — **019** (y su enmienda 1, de la auditoría: con una videollamada, solo con auriculares) · **004**,
  enmienda 1 · **002**, enmienda 8 · **015**, enmienda 4. Cada uno antes de su fase.
- **Regla 24 (las protecciones del Mac)** ✅ con corte — matriz de una fila y «sí» para la corrida de la fase 1
  (§ abajo). Las filas 2 a 5 se enseñaron y no se corrieron (§ El corte).
- **`/audita-sprint` v1.36.0** ✅ — cuatro auditores independientes; 82 hallazgos, los 82 pagados; casilla 8
  limpia; decisiones en llano; la segunda casilla 4, con otro auditor, sobre el diff entero y este summary.
- **Brochure** — en `main` no hay `docs/BROCHURE.html` ni `docs/brochure-export.json`: llega por su orden aparte,
  tras el Acto 1 del H1 (regla 13). Cuando se produzca, ya trae el ensayo y la banda arriba.

## Métricas técnicas

| Métrica del plan | Meta | Medido |
|---|---|---|
| Banco por reglas | precisión y recall por regla, declarados | **1,000 y 1,000** en las seis reglas, Páramo Azul (es) y Northwind (en); el banco entero en 5,3 y 4,6 ms. Es un piso (§ DoD) |
| La evaluación | ≤ 500 ms | **4 ms** la más lenta (buscar + armar + evaluar); precisión 1,000 y recall 0,714 sobre 15 fichas |
| Siguiente pregunta | ≤ 1 s | **< 1 s** tras Enter (`el_ensayo_de_punta_a_punta…`) |
| Acople arriba | ≤ 300 ms | **228 ms** al acoplar y **123 ms** al devolver, en vivo con Meet en Chrome (fase 1) |
| WER | `manual` | **no se midió en el S4**: su corrida local iba con la corrida en vivo de la fase 4, que se aplazó. Sin medida, no se afirma ni regresión ni no regresión (B6) |
| Peso del binario | anotado | ejecutable de release **13,42 MB** (S3: 12,79 → **+0,63 MB**) · `ghost` 0,48 MB · `.app` 14,0 MB · imagen comprimida 6,77 MB (S3: 6,27) — `pnpm tauri build --bundles app --no-sign`, 2026-10-04, con Tauri 2.12 |
| ⭐ que deja el sprint | ~6 | **11** (desviación 24, y r12 de la auditoría) |

## Gate ⭐ — diferimiento y contrapesos

**Opción A: ⭐ diferido.** Sprint intermedio del H2. **⭐ diferido: 11 pruebas al acumulado del ciclo H2 (S4:
11).** Candidatas al ⭐⭐ del H2: **6 sí** (q2, q3, r2, r3, r7, r12) y **5 no** (q1, q5, r6, r10, r11), cada una
con su porqué en la guía. El ⭐⭐ del H2 se arma en el S6. El ⭐⭐ del H1 no se mezcla: sigue en el Acto 2 del S3.

| Contrapeso | Evidencia (archivo, cuenta medida, corrida) |
|---|---|
| Pasada de capturas del builder | **284 encuadres** producto contra maqueta, dos temas y dos idiomas · `docs/fidelidad/S4-cuaderno.html` · `S4-banda.html` · `S4-banda-arriba.html` · `pnpm fidelidad` 2026-10-04, tras el último cambio de interfaz: ninguno sobre el 0,15 %, **ningún desborde** (cazó uno en la auditoría: el aviso de Sesión salía 37 px). **Leídos como imagen: 18 de producto del S4**, 6 en las fases 3 y 4 y 12 en el `/release-check` (B10), uno por bloque: preparar, preguntando, respondiendo, evaluada, progreso, borrar, sin corpus, no empezó, el aviso de Sesión, «lo que salió» en IA, y la banda arriba a 88 y 200. Más las 28 de `ensayo.html` en la fase 0 |
| e2e de `reduced-motion` | **62 pruebas** (15 encuadres × 2 modos + la forma del árbol, en 2 proyectos) · `tests/e2e/reduced-motion.spec.ts` · job `e2e`. El S4 sumó el reloj, la cuenta de preguntas (M17), las cifras, la pregunta y la tabla del progreso. **M17:** el gate pasaba con la cuenta invisible; ahora multiplica la opacidad de la cadena entera |

## Las miradas de DECISIÓN

| # | Archivo | Veredicto, con la frase del usuario |
|---|---|---|
| 1 | `docs/diseno/ensayo.html`, siete estados | **Aprobada**: «La abri y la apruebo la pantalla ensayo, continua» |
| 2 | `docs/diseno/posicion.html`, variante «arriba» con el criterio de la cámara | **Aprobada**: «Si me gusta mucho ka banda arriba buen diseño, lo abri y lo apruebo» |

**FORMA y TEXTO, «maquetada, no vista»** (`docs/diseno/README.md`, y la tabla U de la guía, 18 filas):

- Sesión: la fila «La banda» con ⌃⌥B, el aviso de la primera vez y el aviso del ensayo sin guardar.
- Ensayo: no empezó, del modelo, guardado, borrar, la marca de sección conjeturada y los textos.
- Honestidad mientras ensayas, y «Tuyo» con tus ensayos.
- IA: «Enriquecer el banco» y «para el banco» en «lo que salió».

**No es una aprobación, y se dice:** van al gate del MVP del H2.

## La regla 24: lo que tocó el Mac, con su matriz y su «sí»

| Fila | Qué | Matriz | «Sí» | Corrida |
|---|---|---|---|---|
| 1 | El acople escribe `AXPosition` y el alto en la ventana de la página de prueba de Meet | enseñada, copiada en la bitácora | «Sí, haz la corrida en vivo de la banda arriba. Abierto en Google Chrome» | dos corridas, ningún aviso de macOS |
| 2 | El micrófono fuera de reunión | enseñada | — | **aplazada** (§ El corte) |
| 3 | El Llavero: la llave de tus notas cifra `ensayos/` | enseñada | — | **aplazada** |
| 4 | launchd: la tarea de vencimiento suma tus ensayos | enseñada | — | **aplazada** |
| 5 | El desbloqueo al abrir tu progreso o exportar | enseñada (apareció en la fase 4) | — | **aplazada** |

En local, el constructor solo corrió `AG_SIN_HARDWARE=1 cargo test`; lo marcado lo corre la CI. El
`/release-check` construyó el binario con `--no-sign` (sin Llavero) y sin `.dmg` (su paso usa AppleScript sobre
Finder).

## Decisiones no anticipadas

- **Las siete decisiones del plan** (aprobadas con él): solo se mueve la ventana de la reunión; la voz puede
  salir por los altavoces en el ensayo, con el micrófono sordo; 2,5 s o Enter; los ensayos vencen como tus
  notas; el progreso pide el desbloqueo; la puerta se cierra en el ensayo; la maniobra §10 pasa al S5.
- **Las seis decisiones de la auditoría** (el usuario, 2026-10-04, la opción recomendada en todas):
  - con una videollamada abierta, el ensayo empieza solo con auriculares, de cable, Bluetooth o USB (ADR 019,
    enmienda 1). Se aplicó con la regla que la voz ya usa en reunión, no con el ajuste literal, que dejaba
    fuera los AirPods;
  - la guía pide ⌃⌥B dos veces con Chrome delante; la app acopla abajo como en el H1;
  - arriba solo se acopla con la sesión iniciada;
  - Sesión avisa de un ensayo terminado sin guardar;
  - la protección se dice «verificada con la banda abajo; arriba, y en Zoom y Teams, sin verificar»;
  - «Lo que llega en el H2» pasa a «Lo que no hace hoy», sin chip.
- **⌃⌥B**, la octava tecla con ⌃⌥ (no había ⌃⌥K para el ensayo: sus teclas son de la ventana).
- **Los crates de Tauri suben a 2.12** para casar con npm (desviación 32).
- El resto, en la lista de desviaciones de la bitácora (32).

## Bugs + resoluciones

- **Chrome aplica lo que pide la Accessibility API un instante después.** En la primera corrida en vivo la app
  anotó una lectura intermedia y, al salir, no reconoció la ventana: no la devolvió (nunca la devolvió mal).
  `acople::asentar` espera a que la ventana se quede quieta. La auditoría lo endureció (B18: ≥ 6 lecturas
  iguales o el objetivo).
- **El primer rojo del centinela fue real:** un test unitario encolaba desde el S2 una frase en la voz del
  sistema y la callaba antes de sonar.
- **`pnpm tauri build` no construía la app.** Dependabot (#11) subió `@tauri-apps/api` a 2.12.1 y
  `@tauri-apps/plugin-opener` a 2.7.0, y los crates se quedaron en 2.11.6 y 2.5.5: dependabot solo vigila npm, y
  la CI hace `cargo check`, no el binario. Todo salió verde y el binario no se podía hacer. Se vio en el
  `/release-check`. Los crates subieron (`tauri` 2.12.1, `tauri-build` 2.7.1, `tauri-plugin-opener` 2.7.0, y
  sus dependencias, entre ellas `wry` 0.57), y nació `tests/unit/tauri-a-la-par.test.ts`, que compara las parejas
  en los dos lockfiles. Su primer rojo fue el estado real; el segundo, con `demo-rojo.sh` sobre `Cargo.lock`, nombró «@tauri-apps/api está en 2.12.1 y tauri en 2.11.6».
- **Pruebas decorativas cazadas al exigirles el rojo:** M17 (reduced-motion miraba la opacidad del elemento y
  no la de su caja), B20 (la ruta fuera de la carpeta no se resolvía sin `ensayos/`) y M7 (en `cargo test` la
  voz no avisa de que terminó; se vigila en la fuente de Swift). Y un intento de demo sobre `pnpm-lock.yaml`
  que `pnpm exec` reparó solo antes de correr el test.
- **Un desborde:** el aviso nuevo de Sesión (M10) sacaba la pantalla 37 px de su ventana; pasó a una línea.

## Qué salió bien / qué generó fricción

- **Bien:**
  - **Cuatro auditores independientes** encontraron 82 hallazgos, cuatro altos entre ellos, que la suite en
    verde no veía. Ocho de los de interfaz los confirmaron con sondas medidas, no razonadas.
  - **La corrida en vivo de la fase 1** cazó lo que ningún test sin Mac podía ver: Chrome tarda en aplicar.
  - **El centinela de la regla 25** convirtió una costumbre en mecánica, y su primera corrida encontró un test
    que llevaba dos sprints tocando la voz del sistema.
- **Fricción:**
  - **El cierre se partió en tres PR.** El #10 entró sin la fase 5 y el #12 entró vacío. El corte se declaró
    cada vez, pero el lector tiene que juntar tres sitios.
  - **La corrida en vivo de la fase 4 se aplazó**, y con ella el WER del sprint.
  - **Una dependencia rota invisible para la CI:** el lote de npm de dependabot movió a Tauri por un lado.
  - **El artefacto de auditoría no llevaba el estado de cada hallazgo** después de pagarlos; se completó en el
    `/release-check` (regla 20).

## Sugerencias de mejora al método

1. **En apps Tauri, las parejas npm↔Rust de Tauri se vigilan** (un test como `tauri-a-la-par`, o construir el
   binario en la CI): dependabot de npm las separa y `cargo check` no lo ve. Va al kit de escritorio.
2. **El `/release-check` construye el binario también cuando entra un lote de dependencias**, no solo al cierre.
3. **Un PR de cierre no se mergea vacío** (#12): si el sprint ya se mergeó con corte, el cierre va en un PR que
   se mergea con su contenido.
4. **`auditoria-con-sitio` podría exigir también el estado de cada hallazgo** (regla 20), no solo su sitio.
5. **Las fricciones del kit K-S4-1 a K-S4-9** (bitácora, fase 0), para la planeadora.

## Deuda técnica aceptada

| Qué | Por qué | Pago |
|---|---|---|
| La corrida en vivo del ensayo (filas 2 a 5) | decisión del usuario (§ El corte) | ⭐ del MVP; se recomienda antes del S5 |
| El WER del S4 | iba con esa corrida | `manual`, en la próxima corrida local |
| La banda arriba con Zoom, Teams, una pantalla externa y la pantalla completa; «si macOS no la deja, se deshace y flota» sin test unitario (es nativo) | solo se ve en vivo | ⭐ (q5) |
| Las FORMA y TEXTO del S4, no vistas (18 filas) | regla 10, tres clases de mirada | gate del MVP del H2 |
| El resto de la maniobra §10 | declarada en la fase 0 | S5 |
| Firma y notarización | sin firma no se puede | H2 (G-Release) |
| Dependabot no vigila cargo | el techo de dos PR de dependencias (regla 18); `cargo audit` y ahora `tauri-a-la-par` cubren lo que más pesa | se revisa si otra pareja se separa |
| Las del S3 que siguen abiertas (`lru` y `glib` *unsound*, el llavero de protección de datos, `ghost` fuera del PATH, embeddings y MLX por condición) | ver el summary del S3 | las mismas fechas |

## Auditoría

`/audita-sprint` v1.36.0 sobre `adb2493..HEAD` en `898166e`, con **cuatro auditores independientes** (alcance con
las casillas 7 y 8 · Rust · interfaz con la casilla 5 · casillas 4 y 6), en solo lectura y sin tocar el Mac. El
artefacto es `sprints/SPRINT_004-auditoria.md`: cada hallazgo con su `archivo:línea`, su ajuste y su estado (lo
vigila `auditoria-con-sitio`).

- **Fase 1:** **82 hallazgos — 0 críticos · 4 altos · 28 medios · 50 bajos.** Veredicto «requiere ajustes».
- **Los cuatro altos:**
  - A1: un ensayo con una videollamada abierta podía oír la llamada y guardarla como tu respuesta;
  - A2: el ensayo cableaba `es-ES` y `en-US`, aunque Idioma deja elegir;
  - A3: el acople no iba por turnos, y dos maniobras a la vez podían dejar una ventana movida sin huella;
  - A4: dos paradas del ⭐⭐ del H1 no podían pasar con la banda abajo.
- **Aprobación del usuario** (2026-10-04): «apribada Fase 1… construyamos, si publica en Github (No se publica
  ningun dato personal)». Las seis decisiones, con la opción recomendada (§ Decisiones).
- **Fase 2 (`f3dd820`): los 82, pagados.** **45 con su rojo antes del verde** (3 altos, 19 medios y 23 bajos: 50
  demos con `demo-rojo.sh`, más la de B9 en la CI; dos más pasaron primero en verde y arreglaron su gate) y **37 de texto o declaración**. Ninguno como deuda. Dos
  pagos se apartaron del ajuste propuesto, declarados: A1 (la regla de los auriculares, desviación 27) y M7 (la
  fuente de Swift, desviación 28). B19 lo pagó M3 (desviación 29).
- **Casilla 5:** cinco valores de retorno sin lector (B34–B38) y la marca de sección conjeturada (B33), pagados.
- **Casilla 6:** la guía heredada contra la arquitectura (A4, M25, B40–B44), pagada.
- **Casilla 7:** los idiomas (A2) y el tope de fábrica (M18), que pasan a ser datos.
- **Casilla 8:** limpia. Todo lo que se corrió contra el Mac tuvo su matriz y su «sí».
- **La segunda casilla 4** ⟨resultado⟩.

## `/release-check` (perfil escritorio)

**✅ Pasa 11/12 · ⚠️ 1 con aviso · ❌ 0.** El aviso es el corte declarado. **El `/release-check` encontró un
bloqueo de verdad:** `pnpm tauri build` no construía la app (casilla 3), y quedó arreglado y vigilado antes de esta
tabla.

| # | Casilla | Resultado |
|---|---|---|
| 1 | Tests | ✅ vitest 455 con cobertura (92,6 % de sentencias) · e2e ⟨N⟩ en la CI, cero flaky · `cargo test --locked` en `build-escritorio` (lib 575 · `contra-el-mac-de-verdad` 16 + 14 con `--include-ignored --skip en_vivo_` · `puerta` 14 · `ghost` 5) · **regla 25:** `cargo test` a secas no abre hardware; el centinela `AG_SIN_HARDWARE` y `cargo-test-sin-hardware` lo vigilan |
| 2 | Tipos y lint | ✅ `pnpm typecheck` · `pnpm lint` · `cargo clippy --locked --all-targets -- -D warnings` limpio en local (con Tauri 2.12) y en la CI · tokens vetados · reduced-motion (`tests/e2e/reduced-motion.spec.ts`, 62) |
| 3 | Build del binario | ✅ tras arreglarlo · `pnpm build` · `cargo check --locked` en la CI · **`manual`:** `pnpm tauri build --bundles app --no-sign` en local, 2026-10-04. **La primera vez se negó**: «Found version mismatched Tauri packages» (`tauri` 2.11.6 contra `@tauri-apps/api` 2.12.1; `tauri-plugin-opener` 2.5.5 contra 2.7.0, de dependabot #11). Crates subidos (desviación 32) y gate `tauri-a-la-par`, con su rojo. Después: ejecutable **13,42 MB** (S3: 12,79 → **+0,63 MB**, por el ensayo, la banda arriba y Tauri 2.12, sin separar) · `ghost` 0,48 MB, dentro del `.app` · `.app` 14,0 MB · imagen comprimida **6,77 MB** (S3: 6,27, medida igual con `hdiutil create -format UDZO`). El `.dmg` de Tauri no se construyó: su paso usa AppleScript sobre Finder y puede abrir un aviso de Automatización (regla 22). Firma omitida a propósito (`--no-sign`: sin Llavero). **`--release`:** `cargo test --release --locked --lib --test puerta --test ghost` en verde (575 · 14 · 5). `tauri.conf.json` e `Info.plist` sin cambios. Las capabilities ganan 17 comandos en la ventana principal (13 del ensayo, la franja, arriba o abajo, el aviso y «Enriquecer el banco») y uno en la banda y en el relleno (`la_franja`, solo de lectura), vigilados por `capabilities.test.ts` |
| 4 | Permisos TCC | ⚠️ Ningún permiso nuevo: el ensayo usa el micrófono ya concedido, y el acople, Accessibility. **Lo nuevo con la app viva —el micrófono fuera de reunión, el Llavero con `ensayos/`, launchd con tus ensayos y el desbloqueo— no se ha visto:** corte declarado, al ⭐ del MVP. La app arranca y se usa sin permisos (S1–S3); sin micrófono, el ensayo dice «El micrófono no se abrió» |
| 5 | Ventana protegida | ✅ `content_protected` sin cambios: la banda siempre, el cuaderno en sesión. **Con la banda arriba no se ha mirado al compartir pantalla**, y la app lo dice («verificado… con la banda abajo; con la banda arriba… sin verificar», M20). La ⭐ q2 lo mira |
| 6 | No persistencia | ✅ `pnpm verify:ephemeral` estático con `ensayo/` protegido · en marcha en la CI con un ensayo dentro: `ensayos/` se descifra, sin la canaria, y un intruso es rojo · fuga inyectada vigente · canaria y **término del ensayo** en el log (rojo de B9 en la CI, PR #14) · contador en 0 B en modo local, y a cero al empezar un ensayo (B27) |
| 7 | Seguridad | ✅ `pnpm audit --audit-level high` limpio · `cargo audit` en la CI sobre el `Cargo.lock` nuevo ⟨resultado⟩ · gitleaks · CSP sin cambios · capabilities por ventana (casilla 3) · `AXPosition` se escribe en un solo sitio |
| 8 | Observabilidad | ✅ metadata-only: ni la pregunta, ni la respuesta, ni el cliente en el log (término plantado); el fallo del proveedor se loguea sin contenido |
| 9 | A11y y diseño | ✅ axe en los estados nuevos, dos temas · teclado · `design-system.md` v1.16.0 y `design-sync/` regenerado, con su espejo en verde · **`scrollHeight`:** `pnpm fidelidad`, 284 encuadres, sin desbordes (cazó uno en la auditoría, arreglado) |
| 10 | Documentación y cero enlaces | ✅ manual y guía v6 al día; summary en el PR · barrido `git grep -nE "vercel[.]app\|workers[.]dev\|pages[.]dev" -- ':!pnpm-lock.yaml'` vacío tras el último `git add`; ningún enlace de descarga del binario |
| 11 | Checks del PR | ✅ `quality` · `e2e` · `build-escritorio` en `success` propio en cada push (el del último commit se lee antes de entregar) |
| 12 | El disco en runtime | ✅ inventario (`Permitido` de `contra-el-mac-de-verdad.rs` y el BLUEPRINT): lo del S3, más **`ensayos/`** (cifrados, 600 en 700, con su vencimiento en la lista de launchd), las preferencias nuevas (`posicionDeLaBanda`, `avisoDeArribaVisto`, `enriquecerElBanco`) y la huella del acople, que ahora guarda también la posición. Nada fuera de la lista, y el gate en marcha lo demuestra |

**Decisión: MERGE OK con el corte declarado**, en cuanto la CI del último push cierre en verde.

## Archivos clave

`src-tauri/src/ensayo/` · `src-tauri/src/ensayos.rs` · `src-tauri/src/acople/mod.rs` ·
`src-tauri/src/ventana/geometria.rs` · `src-tauri/src/hardware.rs` · `src/pantallas/Ensayo.tsx` ·
`data/ensayo/` · `decisions/019-el-ensayo-codigo-primero.md` · `docs/GUIA-DE-PRUEBA.html` ·
`sprints/SPRINT_004-auditoria.md`

## Cómo probar

1. Arranca con `pnpm tauri dev`.
2. Abre `docs/GUIA-DE-PRUEBA.html` (doble clic) y elige el filtro **«⭐ del ciclo H2»**: los bloques Q (la banda
   arriba) y R (el ensayo). La caja **«Los avisos de macOS que vas a ver»** dice cada aviso que puede salir;
   cualquier otro, deniégalo.
3. Hace falta:
   - auriculares;
   - `docs/kit-de-prueba/pantalla/meet-de-prueba.html` abierta en Chrome;
   - una propuesta y una ficha de cliente en tu corpus (o las del kit).

Automático: `pnpm test` · `pnpm test:e2e` · `cd src-tauri && AG_SIN_HARDWARE=1 cargo test` (lo marcado
`hardware:` lo corre la CI; lo `en_vivo_*`, a mano, con su matriz y su «sí»).
