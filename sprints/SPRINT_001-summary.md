---
sprint: 001
app: copiloto-consultor
status: closed
opened: 2026-09-20
closed: 2026-09-22
branch: sprint-001/la-banda-y-la-ficha
pr: "#4"
---
# Sprint 001 Summary — Angel Ghost

## Outcome

**Sí en el producto — y por muy poco.** Al concluir la construcción, el outcome principal **no
funcionaba**: la ficha nunca llegaba a la banda. `escucha::Novedad` se serializa etiquetada por
dentro (`{"que":"aparece", …}`) y el webview la leía etiquetada por fuera, así que el campo era
`undefined` en cada evento; la banda se quedaba en «esperando» toda la reunión y solo respondía a
`⌘⇧A`. Ni 87 unitarios ni 66 e2e podían verlo —todos corren fuera de Tauri, donde esa suscripción no
se monta— y **la bitácora afirmaba lo contrario**. Lo encontró la auditoría independiente (C1), y
está arreglado con dos gates nuevos que lo vigilan desde dos jobs distintos.

Los tres outcomes, uno a uno:

| Outcome | Estado |
|---|---|
| **Principal** — banda acoplada y protegida, ficha del corpus propio en ≤4 s tras el fin de turno, nada en disco, red en 0 | **en el producto sí; la reunión real de verdad es del gate ⭐** del usuario. Lo verificado por máquina: la ficha llega por el evento (test dentro de Tauri con el payload que Rust emite), la latencia determinista es de 376 µs de mediana, la sesión completa no deja un archivo fuera del índice, y el contador no tiene con qué moverse |
| **Secundario** — las pantallas fieles a la maqueta, dos temas, dos idiomas | **sí.** 60 encuadres comparados, ninguno pasa del umbral de 0,15 %. Un desborde de 15 px declarado en Idioma cuando falta un modelo |
| **Terciario** — kit de evaluación v0 con P/R del disparo y nDCG@5 | **sí, tras la auditoría.** La fase 5 entregó el nDCG; el P/R del disparador y la mediana de latencia se añadieron pagando el hallazgo A9. WER y audio de mezcla quedan como deuda |

## Qué se construyó

- **La banda** (88/200 px, `content_protected`, todos los Spaces) con sus seis estados de contenido,
  su **relleno** que tapa la franja con el fondo de escritorio, y el **acople** por Accessibility
  API con devolución idempotente y huella en disco a 600/700.
- **Cinco pantallas del cuaderno**: Sesión, Permisos, Corpus, Honestidad, Idioma.
- **Audio en dos pistas** en anillos de 30 s que se sobreescriben, **VAD por energía** con suelo
  adaptativo, fin de turno determinista, **STT local** (SpeechAnalyzer de macOS) y detector de eco.
- **El corpus propio**: tres lectores (Markdown, .docx, PDF), troceado por sección, índice BM25 con
  tantivy y las cinco unidades de la consultoría, a 0700 con reparación al abrir.
- **El disparo determinista** (pregunta · cifra · término del corpus · atajo) y la **ficha**:
  titular de ocho palabras, línea, fuente — y el catálogo de **maniobras** para cuando no hay nada.
- **El kill-switch** `⌥⎋`, el contador de red, y `verify:ephemeral` estático y en runtime.

## DoD — checklist

| Estándar | Evidencia |
|---|---|
| **Testing** | 108 unitarios (TS) · 208 de la librería (Rust) · 14 contra el Mac de verdad · 66 e2e con axe. Cobertura del webview 89 % de líneas |
| **CI/CD** | `quality` · `e2e` · `build-escritorio`, con **conclusión propia** por check. El `e2e` corrió por primera vez en la fase 5: sin histórico **no puede afirmarse no-regresión**, y su primera corrida encontró un defecto real de accesibilidad. **`cargo clippy` entra a `build-escritorio` en el `/release-check`**: el checklist lo exigía desde el estampado y ningún job lo corría — al ejecutarlo por primera vez estaba en rojo |
| **Observabilidad** | ADR 003 **enmendado** (A10): `println!` con prefijo por subsistema y solo metadatos; `tracing` cuando exista un sumidero, con su razón escrita. `pino` fuera del manifiesto. **Término plantado en el log**, que no existía, ahora corre la sesión en un proceso hijo y lee su salida |
| **Seguridad** | `pnpm audit` limpio · `cargo audit` **0 vulnerabilidades** (9 warnings; `lru` *unsound* llega por tantivy y no tiene arreglo compatible) · gitleaks bloqueó la carnada canónica · capabilities por ventana · `verify:ephemeral` estático **y en runtime**, con fuga inyectada en rojo |
| **Performance** | latencia determinista de la ficha: **mediana 376 µs · p90 567 · peor 2 492**, contra 4 s de presupuesto. **Binario de release: 11,05 MB** (subió 2,07 MB al quitar `panic = "abort"`, ver A3); `.app` 11 MB, `.dmg` 4,88 MB |
| **UX/A11y** | teclado de punta a punta · axe en 66 e2e · símbolo + texto + color · dos temas · dos idiomas · **gate de FIDELIDAD** con 60 encuadres, aprobado en las miradas 11 a 14 |
| **IA embebida** | **no aplica: cero LLM, cero tokens, cero red.** El catálogo de maniobras es código y datos versionados, y es el fallback permanente de la síntesis del S2 |
| **Manual** | `docs/MANUAL-DE-USO.md`, ocho features con sus limitaciones. **En español** — desviación declarada de mi propio plan, con su razón |
| **Guía de prueba** | `docs/GUIA-DE-PRUEBA.html` **v2**: 39 pruebas, ⭐ de 31 (~45 min), ⭐⭐ de **9 paradas** caminables (~22 min), kit de prueba en el repo |

## La auditoría — hallazgos, pagos y deuda

Corrida por un **auditor independiente** con el diff delante (kit v1.26.0). Veredicto: **requiere
ajustes** — 1 crítico, 10 altos, 14 medios, 7 bajos. Reporte: `sprints/SPRINT_001-auditoria.md`.
Pagos y demos en rojo: `sprints/SPRINT_001-implementation-log.md`.

**Pagado: el crítico y los diez altos.**

| # | Hallazgo | Pago |
|---|---|---|
| **C1** | la ficha automática nunca llegaba a la banda | la unión discriminada real + **gate de contrato Rust→TS**: Rust escribe `src/contrato.generado.ts` con el serde de producción y `pnpm typecheck` falla si un campo deja de encajar. Y el primer test que atraviesa la suscripción |
| **A1** | la banda pintaba dentro del producto los datos de la consultora inventada, **incluida una frase puesta en boca del cliente** | cada dato de su comando; fuera de Tauri sigue la maqueta. Las pantallas del cuaderno arrancan con el vacío honesto |
| **A2** | el kill-switch no alcanzaba al hilo que transcribe | dos comprobaciones —antes del motor y al volver— y el audio del encargo se pisa |
| **A3** | `panic = "abort"` anulaba el `catch_unwind` del PDF en release | fuera, con su gate. **+2,07 MB de binario, medido** |
| **A4** | `escucha/` afirmaba estar vigilado por `verify:ephemeral` y no lo estaba | en la lista, y un gate sobre el gate: quien lleva la marca está en la lista o falla |
| **A5** | tras un reenganche el disparador quedaba muerto el resto de la sesión | nace el reloj de la escucha, uno solo y monótono |
| **A6** | copy en español cableado llegaba a la interfaz inglesa | al diccionario, y el gate del diccionario deja de mirar solo `i18n/` |
| **A7** | no había forma de instalar el modelo de voz | el estado se dibujó en la maqueta y se construyó el botón |
| **A8** | la pantalla de Honestidad afirmaba que no existe código capaz de abrir una conexión | la frase dice la verdad y el gate **cuenta las puertas** del puente nativo |
| **A9** | el kit medía una de las cuatro cosas del plan | 26 turnos marcados (P/R 1.000/1.000) y la mediana de latencia |
| **A10** | el ADR decía `tracing` y el código tenía 45 `println!` | ADR enmendado con su razón, `pino` fuera, y el término plantado en el log |

**Nueve frases caducadas:** cuatro se volvieron verdad al arreglar el código; cinco se reescribieron
(manual, `LEEME` del kit, Permisos y dos cabeceras de módulo).

**Cada arreglo nació con su demo en rojo en el mismo commit.** Trece rojos preparados — y **el que
más enseñó fue el catorceavo, que no pedí**: el gate nuevo de la canaria tumbó `build-escritorio` a
la primera. Su hijo inventariaba el temporal entero mientras el padre creaba los fixtures de otro
test, y los denunciaba como fuga. Tercera vez en el sprint que dos tests comparten una carpeta
temporal; la primera con el vecino en otro proceso, donde el mutex del turno no llega. El arreglo fue
quitarle al hijo el inventario, que no necesitaba.

Dos de los rojos preparados cambiaron el arreglo: la comprobación de entrada del kill-switch no tenía rojo propio hasta que el
motor de prueba aprendió a decir «he trabajado», y el umbral de recall del disparador tuvo que subir
de 0,90 a 1.0 porque con 0,90 **romper una regla dejaba el test verde**.

## Métricas técnicas

| Métrica | Objetivo | Medido |
|---|---|---|
| fin de turno → ficha | ≤ 4 s | tramo determinista: **mediana 376 µs**, peor 2 492 µs. El tramo con audio y STT se mide en el Mac de verdad |
| fin de turno (decisión) | 160–400 ms | **320 ms**, contra el rango de la orden y no contra una constante propia |
| nDCG@5 del retriever | umbral declarado | **0,823** (mínimo 0,80) |
| rechazo de lo que no está | sin margen | **1,000** (mínimo 1,00) |
| P/R del disparador | nuevo en la auditoría | **1,000 / 1,000** sobre 26 turnos marcados |
| bytes a la red | 0 | **0**, y el gate cuenta las dos puertas declaradas del puente |
| peso del binario | anotado por PR | **11,05 MB** (9,01 antes de A3). Bundle del `/release-check`: `.app` 11 MB · `.dmg` **4,88 MB** |

## Decisiones no anticipadas

- **ADR 006** — el STT local y su modelo: SpeechAnalyzer con medición delante.
- **ADR 008** — el corpus, el disparo y la ficha.
- **ADR 003 enmendado** — observabilidad: `println!` hoy, `tracing` cuando haya sumidero (A10).
- **La maniobra** — producto nuevo, pedido por el usuario en la mirada 11. Cero LLM, catálogo
  versionado. **La planeadora tiene que absorberla** al lado de C6, o como C17.

## Bugs + resoluciones

Los once del cierre de construcción están en la bitácora. Los que más enseñaron:

- **Dos fuentes de verdad en la banda** (URL vs. ficha) dejaban la pantalla en blanco.
- **Una unidad casteada desde su etiqueta traducida**: funcionaba en español por casualidad.
- **«18,4 MB» con coma española en la interfaz inglesa**, y el chip del pie con un conteo copiado de
  otro estado: los dos solo se vieron **leyendo la captura**, no corriendo el producto.
- **Cuatro tests del corpus compartían una carpeta temporal** y se pisaban en paralelo — la misma
  clase de defecto que los de audio en la fase 3.
- **El kit de evaluación falló al nacer** y encontró seis interrogativos que faltaban en las
  palabras vacías: rechazo 0,750 → 1,000.
- **El `e2e` llevaba cuatro fases verde con cero pruebas.** `--pass-with-no-tests` y nadie mirando.

## Qué salió bien / qué generó fricción

**Bien.** Medir en vez de suponer, cada vez: el costo de tantivy antes de comprometerla, el plegado
de acentos contra 23 pares de consultoría, el desborde de la maqueta, los 2,07 MB de `panic`. Las
cuatro miradas con veredicto citado del usuario. Y la auditoría **independiente**, que encontró en un
día lo que el constructor no vio en cinco fases.

**Fricción.** La costura entre Rust y el webview: **tres de los defectos más caros del sprint
vivieron ahí** —la unidad casteada, el separador decimal y C1— y era lo único que ningún test
atravesaba. El contrato se escribía dos veces a mano y nadie comparaba las copias.

## Sugerencias de mejora al método

1. **Todo puente entre dos lenguajes necesita su gate de contrato, y el sprint que lo cruza lo
   construye.** No es una idea de este sprint: es la tercera reincidencia de la misma clase en un
   mismo ciclo. La forma que funcionó —el lado que emite escribe el fixture, el lado que lee lo
   declara con su tipo— cuesta una tarde y caza los tres defectos de golpe.
2. **Un test que corre solo en el modo que no se distribuye no prueba ese modo.** La regla 15 ya
   tiene su tercer filo («¿lo viste correr EN EL MODO…?»); A3 dice que hay que aplicarlo también al
   **perfil de compilación**, no solo a los modos de arranque.
3. **El artefacto de auditoría del repo tiene que llevar TODAS las severidades con archivo y línea.**
   El de este sprint resumió los medios y los bajos, y el resumen se comió lo único que hace pagable
   un hallazgo: dónde está. Ocho medios y siete bajos quedan declarados por conteo.
4. **Una afirmación de pantalla caduca en silencio.** El barrido por promesa aplazada (casilla 4 de
   `/audita-sprint`) funcionó, y lo que encontró sugiere algo más fuerte: **las frases que afirman
   algo sobre el código deberían llevar el gate que las sostiene escrito al lado** — como quedaron
   las de `leer.rs`, `escucha/mod.rs` y la de Honestidad.
5. **El barrido de frases caducadas se corre DESPUÉS del último arreglo, no en la Fase 1.** La casilla
   4 de `/audita-sprint` barrió el repo en la Fase 1; la Fase 2 arregló el desborde de Idioma
   acortando un botón de «Instalar el modelo» a «Instalar», y **fabricó una frase caducada nueva** que
   el `/release-check` encontró en el manual doce horas después. Es palabra por palabra la lección que
   la regla 17 ya aprendió con los enlaces —*el barrido corre sobre el árbol que se va a subir,
   después del último `git add`*—: un barrido de caducidad hecho antes de los arreglos audita un repo
   que ya no existe. Pequeño y mecánico: repetir la casilla 4 al cerrar la Fase 2.
6. **Un gate que el checklist exige y ningún job ejecuta no existe.** `cargo clippy -- -D warnings`
   estaba en el `/release-check` desde el estampado y **ningún job del `ci.yml` lo corría**: al
   ejecutarlo por primera vez en este cierre estaba en rojo. La sugerencia no es «acuérdate de correr
   clippy», es **mecánica**: el `/release-check` debería exigir, casilla por casilla, que el comando
   viva en un job de CI o quede declarado como manual con su razón — hoy la plantilla dice «verifica
   con EL comando del `ci-escritorio.yml`» y no comprueba que ese comando esté en algún `yml`.

## Deuda técnica aceptada

| Qué | Por qué | Pago |
|---|---|---|
| **M1** `"csp": null` en `tauri.conf.json` | poner una CSP toca el IPC y los estilos que Tailwind inyecta en caliente; hacerlo sin arrancar la app es cambiar un gate por una avería silenciosa | S2 |
| **M2** Sesión dice `Funciona` sin leer `abierta`/`motivo` | misma clase que A1; su estado «pista caída» no está dibujado en la maqueta | S2, con mirada |
| **M4** tras `⌥⎋` la banda no vuelve | o la vuelta, o la advertencia en el manual | S2 |
| **M9** el efímero en runtime no mira `~/Library` y solo ve archivos NUEVOS | hacerlo bien pide comparar hashes de un árbol grande | S2 |
| **M10** `nativo.rs` reinterpreta bytes como `f32` sin validar | hoy el formato lo fija la misma app en los dos lados | S2 |
| **M3, M5–M8, M12–M14 · B1–B7** | **declarados por conteo**: su detalle no llegó al artefacto del repo (ver sugerencia 3) | S2 |
| **WER informativo y audio de mezcla** del kit | el plan los pedía; la fase 5 no los hizo y la auditoría los declara | S2 |
| **Disparo por silencio** | `por_silencio` existe y no tiene llamadores. **El manual ya lo declara** en vez de prometerlo | S2 |
| **El VAD no es Silero** | regla 14: el modelo se gana el puesto con una medición, y esa medición es el WER que falta | S2, en el ADR del STT |
| **Desborde de 15 px en Idioma** sin modelo | la alternativa era recortar una frase de honestidad | a la mirada del usuario |
| **`lru` *unsound*** (RUSTSEC-2026-0253) vía tantivy | sin arreglo compatible: tantivy fija `^0.16` y el arreglo está en 0.18. **Y quitar `panic = "abort"` (A3) ensancha su exposición**, porque ahora un pánico se desenreda en vez de abortar | S2, vigilando tantivy |
| **`design-sync/` no existe** | la Etapa de Diseño lo anotó como «no hay ciclo cerrado que publicar», y ese no es el motivo que manda: la **regla 16** pide que todo sprint que toque UI actualice el bundle **en su mismo PR**, precisamente para que el cierre de ciclo sea un delta y no una reconstrucción. Este es el primer sprint con UI. **Decisión del usuario**: construirlo antes del merge, o pagarlo en el S2 con el cierre más caro | S2, salvo que el usuario lo pida ahora |
| **17 campos del contrato sin consumidor** | el gate nuevo compara la FORMA, no si alguien lee. `Documento`, `corte::Informe` y `Aparicion.ms` siguen sin llegar a la pantalla | S2 |

## Archivos clave

| Archivo | Qué es |
|---|---|
| `src-tauri/src/escucha/mod.rs` | donde se juntan las dos pistas, el turno, el disparo y la ficha |
| `src-tauri/src/contrato.rs` → `src/contrato.generado.ts` | el contrato con la interfaz, escrito por Rust y vigilado por `tsc` |
| `src-tauri/src/corpus/` | los tres lectores, el troceado, el índice BM25 y las cinco unidades |
| `src-tauri/src/ficha/` | el titular, la línea, la fuente y el catálogo de maniobras |
| `src/componentes/Banda.tsx` · `src/ficha.ts` | la banda y lo que enseña, ahora con datos reales |
| `scripts/verify-ephemeral.mjs` | el barrido estático, con su gate sobre sí mismo |
| `src-tauri/tests/contra-el-mac-de-verdad.rs` | el único binario de integración: audio real, efímero en marcha, la canaria en el log y el kit |
| `docs/GUIA-DE-PRUEBA.html` | v2 tras la auditoría: 39 pruebas, ⭐ 31, ⭐⭐ 9 paradas |
| `sprints/SPRINT_001-auditoria.md` | el reporte de la auditoría independiente |

## Cómo probar

```bash
pnpm install
pnpm typecheck && pnpm lint && pnpm test        # 108 unitarios con cobertura
pnpm test:e2e                                    # 66 e2e con axe
pnpm verify:ephemeral                            # el barrido estático
cd src-tauri && cargo test                       # 208 + 14, con el Mac de verdad
pnpm fidelidad                                   # 60 encuadres contra la maqueta
pnpm tauri dev                                   # la app: tres ventanas
```

Y lo que ninguna máquina puede verificar: `docs/GUIA-DE-PRUEBA.html` — **gate ⭐⭐ de 9 paradas,
~22 min**, empezando por la parada 5, que es la que la auditoría encontró rota.
