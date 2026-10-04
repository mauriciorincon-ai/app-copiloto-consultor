# Auditoría del sprint 004 «El ensayo» — Fase 1

Fecha: **2026-10-04**. Código auditado:

- **HEAD `898166e`** de `sprint-004/fase-5` (PR #13);
- el diff del sprint entero, **`adb2493..HEAD`**, que cubre las fases 0 a 5 (las fases 0 a 4 ya estaban en `main` por los PR #10 y #12).

La Fase 1 fue de solo lectura: en el repo solo se escribió este archivo.

## Método

**Cuatro auditores independientes:** subagentes que no construyeron el sprint, cada uno con el diff delante y la
bitácora solo como pista.

1. Alcance contra el plan y la orden, con la casilla 7 (números cableados) y la casilla 8 (protecciones del Mac).
2. Código Rust.
3. Interfaz y casilla 5 (campos del contrato sin lector).
4. Casilla 4 (frases caducadas) y casilla 6 (la guía heredada).

El constructor consolidó y quitó duplicados. Al fundir dos hallazgos se queda la severidad mayor. Además comprobó a
mano A4 y M10 contra `src-tauri/src/lib.rs:671`, `:712-722` y `:3595-3598`.

**Lo que se corrió.** Nada que tocara el Mac del usuario.

| Auditor | Corrida | Resultado |
|---|---|---|
| 1 | `AG_SIN_HARDWARE=1 cargo test --locked`, con su `target` en el scratchpad | lib 555 (+2 ignorados) · contra el Mac 16 (+14) · ghost 5 · puerta 14; el centinela no abortó ninguna |
| 2 | `cargo clippy --all-targets -D warnings` | limpio |
| 2 | `cargo test --lib` | 555 |
| 3 | `pnpm typecheck`, `pnpm lint`, `pnpm test` | 430 |
| 3 | ocho sondas de Vitest en el scratchpad | confirman M11, M13, M14, M15, M16, B24, B25 y B26: **son medidas, no razonadas** |
| 4 | barridos por promesa aplazada | — |

La CI del PR #13 dio `quality`, `e2e` y `build-escritorio` en `success`.

## Veredicto

**Requiere ajustes.** Hay 82 hallazgos y se pagan todos:

| Severidad | Cuántos |
|---|---|
| Crítico | 0 |
| Alto | 4 |
| Medio | 28 |
| Bajo | 50 |

Los cuatro altos:

1. Un ensayo con una videollamada abierta puede oír la llamada y guardarla como tu respuesta.
2. El ensayo cablea dos idiomas regionales.
3. El acople no se hace por turnos, así que dos maniobras a la vez pueden dejar una ventana movida sin huella.
4. Dos paradas del ⭐⭐ del H1 no pueden pasar con la banda abajo.

**Estado tras la Fase 2 (2026-10-04, `f3dd820`): los 82, pagados.**

- **45 con su rojo antes del verde:** 3 altos, 19 medios y 23 bajos. Todos se vieron con `scripts/demo-rojo.sh`, salvo B9, que se vio en la CI (PR #14, desechable).
- **37 de texto o declaración**, sin gate que demostrar.
- **Ninguno queda como deuda.** Cada uno dice su estado en su sección o en su fila.

Las fases 0 a 4 cumplen su alcance. La corrida en vivo de la fase 4 va aplazada con su corte declarado. **La
casilla 8 sale limpia:** todo lo que se corrió contra el Mac tuvo su matriz y su «sí».

## Decisiones que son del usuario

Van en llano en el mensaje de entrega. Seis hallazgos cambian lo que el producto promete o cómo se usa, y su
ajuste depende de la respuesta:

| Hallazgo | Decisión |
|---|---|
| A1 | qué pasa al ensayar con una videollamada abierta |
| A4 | si la guía se arregla con texto o la app acopla también abajo al iniciar sesión |
| M3 | si la banda arriba puede acoplar Zoom o Teams sin llamada |
| M10 | qué pasa con un ensayo sin guardar al iniciar una sesión |
| M20 | qué dice la app de la protección con la banda arriba |
| B46 | las frases «Lo que llega en el H2» |

## Cobertura de alcance

| Ítem | Clasificación | Cita |
|---|---|---|
| Fase 0 · delta del kit en `CLAUDE.md` y en los comandos, `demo-rojo.sh`, `verificar-dependencias`, `auditConfig`, hooks, homepage | Completo | `CLAUDE.md` · `.github/workflows/ci.yml:27-34` · `pnpm-workspace.yaml:16-17` · `githooks/pre-commit:16-22` |
| Fase 0 · gates de controladores de maqueta y de envejecimiento | Completo | `tests/unit/controladores-maqueta.test.ts` · `tests/unit/envejecimiento.test.tsx` |
| Fase 0 · regla 25: centinela, marcas, dos pasos de CI, gate | Completo, con huecos (B5) | `src-tauri/src/hardware.rs:29-38` · `tests/unit/cargo-test-sin-hardware.test.ts` |
| Fase 0 · ADR 019 y enmiendas 004, 015 y 002, antes de su fase | Completo | `decisions/019-*.md` (commit `3c47f9e`) |
| Fase 0 · maquetas, dos miradas de DECISIÓN y registro «no vista» | Completo | `docs/diseno/README.md` · bitácora |
| Fase 1 · geometría, preferencia, ⌃⌥B, aviso, asa y relleno | Completo | `src-tauri/src/ventana/geometria.rs:54-68` · `src-tauri/src/lib.rs:325-357` |
| Fase 1 · acople «baja y se encoge», solo la reunión | Implementado con hallazgos (A3, M1-M5) | `src-tauri/src/acople/mod.rs:144-150` · `:502-598` |
| Fase 1 · devolución doble, huella v1, `AXPosition` única | Completo | `src-tauri/src/acople/mod.rs:188-197` · `src-tauri/src/acople/ax.rs:143-154` |
| Fase 1 · corrida en vivo, fila 1 | Completo | bitácora, fase 1 (228 y 123 ms) |
| Fase 2 · catálogos, banco, kit v3, acento opt-in con bóveda | Completo, con hallazgos (M18, B12, B22) | `src-tauri/src/ensayo/banco.rs:353-480` · `src-tauri/src/ensayo/enriquecer.rs` |
| Fase 3 · sesión, oído solo del micrófono, evaluación ≤ 500 ms, pantalla | Completo, con hallazgos (A1, A2, M6-M8, M11-M16) | `src-tauri/src/ensayo/` · `src/pantallas/Ensayo.tsx` |
| Fase 3 · el test de los altavoces dice «en reunión nunca; en el ensayo sí» | **No implementado, sin declarar** (B1) | `src-tauri/tests/contra-el-mac-de-verdad.rs:1886` |
| Fase 3 · término plantado en los logs | Parcial (B9) | `tests/unit/logs-de-la-sintesis.test.ts:69-97` |
| Fase 4 · `ensayos.rs`, vencimiento, efímero en marcha con intruso, progreso, exportar, Honestidad | Completo | `src-tauri/src/ensayos.rs` · `src-tauri/tests/contra-el-mac-de-verdad.rs:518-583` |
| Fase 4 · corrida en vivo, filas 2 a 5 | No implementado: corte declarado (desviación 21) | bitácora, fase 4 |
| Fase 5 · manual, guía v6, kit v3, design system 1.15.0, `design-sync/`, nota del brochure | Completo, con hallazgos (A4, M22-M28, B40-B50) | `docs/MANUAL-DE-USO.md` · `docs/GUIA-DE-PRUEBA.html` |
| Contrapeso · pasada de capturas | Parcial (B10) | bitácora, fases 3 y 4 |
| Contrapeso · e2e de reduced-motion: temporizador **y cuenta de preguntas** | Parcial (M17) | `tests/e2e/reduced-motion.spec.ts:31-36` |
| Peso del binario anotado | Pendiente (B7) | — |
| `/release-check` y summary | Pendiente (después de esta auditoría) | — |

## CRÍTICOS (0)

Ninguno.

## ALTOS (4)

### A1 · Un ensayo con una videollamada abierta oye la llamada y la guarda como «tu respuesta»; la voz del ensayo sale a la llamada

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`). **Decisión del usuario:** con altavoces no empieza y dice por qué; con auriculares —de cable, Bluetooth o USB— sí. Se aplicó con la regla del H1 (`habla::cabe_decirla`) y no con el ajuste literal, que dejaba fuera los AirPods (desviación 27; ADR 019, enmienda 1). Nace un segundo porqué, «No se puede saber si hay una videollamada».

**Sitio:**

- `src-tauri/src/lib.rs:1236-1242`, con el comentario «Una videollamada abierta sin sesión no lo impide: no hay nada
  que oír»;
- `src-tauri/src/lib.rs:1090-1101`: la voz sale sin el candado de los auriculares;
- la misma premisa en la enmienda 8 de `decisions/002-persistencia.md`;
- el texto `src/i18n/es.ts:920` (`vozSinReunion`) y `docs/MANUAL-DE-USO.md:416-417` («En la reunión, nunca»).

**Escenario:**

1. Zoom o Meet está en llamada, sin sesión de Angel Ghost, y suena por los altavoces.
2. Pulsas «Empezar el ensayo».
3. El micrófono transcribe a la otra parte: en el ensayo no hay pista del sistema contra la que marcar eco.
4. «Guardar con tus notas» lo guarda cifrado en `ensayos/` como tu respuesta.

Eso rompe la **regla dura 1**: el transcript del cliente muere siempre. Y la voz que lee la pregunta, con frases de
tu propuesta, la oye la llamada: es el «te oiría el cliente» que el H1 prohíbe. El gate en marcha no lo ve, porque
parte de que el micrófono no puede traer al cliente.

**Ajuste: decisión del usuario.** La opción recomendada es la A: con una videollamada abierta, el ensayo solo empieza
con auriculares.

1. `src-tauri/src/ensayo/mod.rs`, en `NoEmpezo`, la variante nueva:

   ```rust
   /// Hay una videollamada abierta y sin auriculares: tu micrófono la oiría y la llamada oiría la voz del ensayo.
   Videollamada,
   ```

2. `src-tauri/src/lib.rs`, tras la línea 1242:

   ```rust
   let salida = capture::nativo::salida_de_audio();
   if !matches!(sesion::ahora(), sesion::Reunion::Ninguna) && salida.puede_haber_eco() != Some(false) {
       return Err(ensayo::NoEmpezo::Videollamada);
   }
   ```

   Con la opción B se quita la condición de la salida y se bloquea siempre.
3. Muestra `NO_EMPEZO_VIDEOLLAMADA` en `src-tauri/src/contrato.rs`, y regenerar el contrato.
4. Rama `no.que === "videollamada"` en `src/pantallas/Ensayo.tsx`, junto a la de `:385`, con `role="alert"`.
5. Las claves `hayVideollamada` y `hayVideollamadaQue`:

   | Idioma | Título | Detalle |
   |---|---|---|
   | es | «Hay una videollamada abierta» | «Con los altavoces, tu micrófono oiría la llamada y la llamada oiría la voz del ensayo. Ponte auriculares o ciérrala para ensayar.» |
   | en | «A video call is open» | «With the speakers, your microphone would hear the call and the call would hear the rehearsal's voice. Put on headphones or close it to rehearse.» |

   Las dos, literales, en `docs/diseno/ensayo.html`, estado «9 · más textos».
6. Corregir el comentario de `src-tauri/src/lib.rs:1237`, el ADR 019 §6.5 y la enmienda 8 del ADR 002.
7. `vozSinReunion` y el manual (`:416-417`) dicen lo que hace la app con la opción elegida. Con la A:

   | Idioma | Texto |
   |---|---|
   | es | «Sin reunión, la voz sale por donde suene tu Mac; con una videollamada abierta, solo con auriculares. Mientras habla, el micrófono no escucha.» |
   | en | «With no meeting, the voice plays wherever your Mac plays sound; with a video call open, only with headphones. While it speaks, the microphone does not listen.» |

8. En el test de fuente `src-tauri/src/lib.rs:4005-4023`, afirmar que `NoEmpezo::Videollamada` aparece antes de
   `Oido::del_microfono(`.

**Verificado cuando:**

- `AG_SIN_HARDWARE=1 cargo test pruebas_de_la_puerta_de_la_captura` está en verde, y se pone en rojo sin el `if`
  (con `demo-rojo.sh`);
- en `pnpm test`, `el-ensayo.test.tsx` pinta `hayVideollamada` con la muestra;
- el gate i18n está en verde.

### A2 · El ensayo cablea `es-ES` y `en-US`, aunque Idioma deja elegir N locales del sistema (casilla 7)

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`): el ensayo transcribe con el locale de Idioma (`locale_del_ensayo`, `MundoDeLaApp`).

**Sitio:**

- `src-tauri/src/lib.rs:1083-1088` (`codigo_de`);
- usado en `:1099` (la voz), `:1218` (`Preparacion.transcribe`), `:1269` (`Oido::del_microfono`) y `:1283` (el log).

**Escenario.** Tu pista es `es-MX`, y es el único modelo instalado. Preparar avisa «este Mac no sabe transcribir» y
te manda a Idioma. Si empieza, transcribe con otra variante. El dato dice N locales (`src/pantallas/Idioma.tsx:148`,
`src-tauri/src/stt/apple.rs:121`); el código asume dos.

**Ajuste.** En `src-tauri/src/lib.rs`, junto a `codigo_de`:

```rust
/// El locale del ensayo: tu pista si es de ese idioma; si no, la del cliente; si no, el primero de ese idioma que
/// el motor tenga listo; y solo si no hay ninguno, el de fábrica.
fn locale_del_ensayo(idioma: ensayo::banco::Idioma, pistas: &prefs::IdiomasDePista, listos: &[String]) -> String {
    let p = match idioma { ensayo::banco::Idioma::Es => "es-", ensayo::banco::Idioma::En => "en-" };
    [pistas.consultor.as_str(), pistas.cliente.as_str()].into_iter()
        .chain(listos.iter().map(String::as_str))
        .find(|c| c.starts_with(p))
        .unwrap_or(codigo_de(idioma)).to_string()
}
```

- `listos` sale de `motor.idiomas()`, filtrado por `Disponibilidad::Listo`.
- Usarla en `:1218`, `:1269` y `:1283`.
- Guardar el locale en `MundoDeLaApp` para la voz de `:1099`.

**Verificado cuando:** el test `locale_del_ensayo` está en verde y se pone en rojo si la función devuelve siempre
`codigo_de`. Casos:

| Pistas | Idioma | Listos | Resultado |
|---|---|---|---|
| `es-MX` | Es | — | `es-MX` |
| consultor `es-CO`, cliente `en-GB` | En | — | `en-GB` |
| sin `en-*` | En | `["en-AU"]` | `en-AU` |
| nada | En | nada | `en-US` |

### A3 · El acople no se hace por turnos: dos maniobras a la vez pierden la huella y dejan la reunión movida para siempre

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`): `UNA_A_LA_VEZ`, y cada maniobra nativa (`acoplar`, `reacoplar`, `acoplar_arriba`, `soltar`) espera su turno.

**Sitio:**

- ninguna maniobra toma un candado en `src-tauri/src/acople/mod.rs:385` (`acoplar`), `:411` (`reacoplar`), `:495`
  (`acoplar_arriba`) y `:603` (`soltar`, que acaba con `olvidar(huella)`);
- el sprint añadió entradas concurrentes, cada una en su hilo: `src-tauri/src/lib.rs:343-345` (un hilo por clic),
  `:3469-3471` (⌃⌥B, un hilo por pulsación, con `otro()` calculado fuera del hilo), `:715-721` y el latido
  (`:3595-3604`).

**Escenario:**

1. La banda está arriba y Meet acoplada.
2. Pulsas ⌃⌥B dos veces en 150 ms.
3. El segundo hilo lee la ventana a medio devolver y dice «ninguna coincide».
4. Olvida la huella y anota como original un estado que ya era nuestro.
5. El primer hilo termina y borra la huella del segundo.
6. La reunión queda movida o encogida **sin huella**: no se devuelve ni al salir ni al arrancar.

Es justo el fallo que el módulo existe para no cometer.

**Ajuste.**

1. En `src-tauri/src/acople/mod.rs`, dentro de `mod nativo`:

   ```rust
   /// Una maniobra a la vez: todas leen y escriben la misma huella y las mismas ventanas, desde varios hilos.
   static UNA_A_LA_VEZ: std::sync::Mutex<()> = std::sync::Mutex::new(());
   fn turno() -> std::sync::MutexGuard<'static, ()> { UNA_A_LA_VEZ.lock().unwrap_or_else(|e| e.into_inner()) }
   ```

   Va `let _turno = turno();` como primera línea de `acoplar`, `reacoplar`, `acoplar_arriba` y `soltar`. Dentro de
   ellas, las llamadas internas a `soltar(huella)` (`:396`, `:422`, `:512`) pasan a `soltar_sin_medir(huella)`, para
   que el candado no se bloquee consigo mismo.
2. En `src-tauri/src/lib.rs`, `static CAMBIO_DE_BORDE: std::sync::Mutex<()>`, que toman el hilo de `:345` y el de
   ⌃⌥B. En el de ⌃⌥B, `borde_de(&mango).otro()` se calcula **dentro** del hilo y después del candado.

**Verificado cuando:**

- el test de fuente `cada_maniobra_nativa_espera_su_turno` encuentra `let _turno = turno();` en las cuatro y ningún
  `soltar(huella)` interno;
- otro test de fuente comprueba que la rama de ⌃⌥B llama a `.otro()` después de `CAMBIO_DE_BORDE.lock()`;
- los dos están en rojo antes del arreglo.

### A4 · Guía: con la banda abajo nada acopla Chrome, y dos paradas del ⭐⭐ del H1 no pueden pasar (casilla 6)

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`). **Decisión del usuario:** la guía pide ⌃⌥B dos veces con Chrome delante (a2, a5, l5); la app, como en el H1.

**Sitio:** `docs/GUIA-DE-PRUEBA.html:214` (la preparación), y las pruebas a2 (`:306`, parada 1), a5 (`:325`), i1
(`:560`, parada 4), l2 (`:635`, parada 6) y l5 (`:655`).

**Escenario.** Abajo, la app solo acopla con el latido del arranque: 30 s y una vez, a quien esté delante
(`src-tauri/src/lib.rs:3595-3598`). «Iniciar sesión» acopla solo arriba (`src-tauri/src/lib.rs:712-722`, comprobado
por el constructor). La preparación del ⭐⭐ dura unos 2 min con Meet cerrado. Cuando la parada 1 abre Meet, el
latido ya terminó. Por eso:

- en la parada 4, la reunión no puede crecer;
- la parada 6 pasa sin medir que Chrome recupera su tamaño;
- a5 solo pasa si Chrome llegó delante en los primeros 30 s.

**Ajuste: decisión del usuario.** La opción recomendada es arreglar el texto de la guía; la otra cambia «abajo, como
en el H1».

- **a2** (parada 1, «Mejorado en S4»), primera frase: «Con la app arrancada, abre Google Meet en Chrome, entra a una
  sala y deja la ventana de Chrome **maximizada (no a pantalla completa) y delante**. **Pulsa ⌃⌥B dos veces**: la
  banda sube y vuelve abajo, y al volver acopla la ventana que tienes delante —Chrome se recorta por abajo—. Ahora
  **comparte la pantalla completa**.» Y la nota: «Decía solo «abre Google Meet»: abajo la app acopla sola únicamente
  en los 30 s que siguen al arranque, y la preparación dura más.»
- **a5** («Mejorado en S4»): «Mira la ventana de Chrome: **se ha recortado por abajo** para dejarle sitio a la banda
  (la acoplaste en la a2). Si no concediste Accesibilidad, la banda flota encima y lo dice con «sin acople».»
- **l5** («Mejorado en S4»), al principio: «Con la sesión iniciada y la banda en pantalla, pon Chrome delante y pulsa
  ⌃⌥B dos veces (tras el corte, el acople no vuelve solo).»
- La cabecera y el historial de la guía cuentan las reescritas.
- La alternativa en código: que «Iniciar sesión» también acople abajo la ventana de la reunión detectada.

**Verificado cuando:** `guia-cuadra` está en verde con la cuenta nueva. En el ⭐ del usuario, la consola dice
`[acople] reacople: … ventanas=1` en la parada 4, y la parada 6 deja `[acople] kill-switch: … ventanas=1`.

## MEDIOS (28)

### M1 · El acople escribe por la posición de la ventana en la lista sin comprobar que sigue siendo la misma

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:**
- `src-tauri/src/acople/ax.rs:169-170` dice «el índice solo vale dentro de esta llamada». Pero `Destino.indice` se
  reusa paso a paso en `src-tauri/src/acople/mod.rs:582-598` (`ejecutar`), y el deshacer busca por índice en `:552-556`.
- Cada paso vuelve a pedir `AXWindows` (`src-tauri/src/acople/ax.rs:195-207`), con hasta 400 ms de espera entre pasos.

**Qué pasa:** con dos ventanas de Chrome, si traes la otra al frente entre dos pasos, el `Mover` o el deshacer escriben
en otra ventana. Por ejemplo, el correo acaba con la geometría de Meet.

**Ajuste:**
1. En `ejecutar`, antes de cada paso, comprobar `let ahora = ax::marco_de_la_ventana(pid, indice)?;` y
   `if !casi_iguales(ahora, quedo) { return None; }`.
2. En la rama de fallo, quitar el `.or_else(...)` por índice.
3. Para poder probarlo, extraer `fn ejecutar_con(leer, escribir, esperar, desde, pasos)`.

**Verificado cuando:** el test `ejecutar_no_escribe_si_la_ventana_del_indice_ya_no_es_la_misma` pasa. Un `leer` falso
cambia de geometría antes del segundo paso, y el test exige una sola escritura y `None`. Antes del ajuste está en rojo.

### M2 · El deshacer del acople arriba no se verifica

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:** `src-tauri/src/acople/mod.rs:551-560`. Además, `soltar_sin_medir` (`:666`) olvida la huella aunque la
devolución fallara a medias.

**Qué pasa:** el paso del alto entra y el `Mover` falla (Spaces, pantalla completa). Se llama a deshacer **sin mirar
su resultado**, y el motivo dice «se deshizo y la banda flota». Si el deshacer también falla:
- la reunión queda encogida;
- no hay huella que la devuelva;
- el log dice lo contrario.

**Ajuste:**
1. Función pura nueva:
   ```rust
   pub fn sin_deshacer(original: Marco, deshecha: Option<Marco>) -> Option<Marco> { deshecha.filter(|q| hubo_cambio(original, *q)) }
   ```
2. Guardar el resultado del deshacer (`let deshecha = ahora.and_then(|a| ejecutar(...))`).
3. Si `deshecha.or(ahora)` sigue cambiada: el motivo pasa a «no dejó bajar su ventana y no se pudo deshacer del todo:
   quedó en (x,y) con N px», se reintenta una vez y se guarda una huella para que `soltar` la devuelva.
4. En `soltar_sin_medir`, conservar en la huella las ventanas que no volvieron enteras.

**Verificado cuando:** un test cubre los cuatro casos de `sin_deshacer` y se pone en rojo si devuelve siempre `None`.

### M3 · Con Zoom o Teams abiertos sin llamada, el acople de arriba mueve una ventana que no es una reunión

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`) (dos). **Decisión del usuario:** arriba solo se acopla con la sesión iniciada; el latido no acopla arriba.

**Sitio:** `src-tauri/src/sesion/mod.rs:275-285` (`ventana_de_la_reunion`, que sigue a `objetivo_de`; la clase
`Aplicacion` «se detecta con que esté corriendo», `:31`). El latido la acopla en `src-tauri/src/lib.rs:3595-3598`.

**Qué pasa:** Teams está abierto todo el día con su chat. Al arrancar la app, en menos de 30 s la ventana de chat de
Teams **baja y se encoge sola**, sin reunión. Con una llamada en Meet y Teams abierto, se acopla Teams y Meet queda
tapada. Choca con el ADR 004, enmienda 1: «sin reunión detectada no hay acople».

**Ajuste: decisión del usuario.** Recomendación: arriba solo se acopla al iniciar la sesión.
1. En `acoplar_arriba` (`src-tauri/src/lib.rs:262`), actuar solo con el cuaderno abierto:
   `let abierta = app.try_state::<reunion::ElCuaderno>().is_some_and(|c| c.abierta());`, y si no, el motivo
   «sin sesión abierta o sin reunión detectada: la banda flota arriba sin tocar ninguna ventana».
2. En `:3597`, la rama `Arriba => false`: arriba, el latido no acopla.
3. El manual (`docs/MANUAL-DE-USO.md:66-67`, M24) y la guía q4 dicen lo nuevo.

**Verificado cuando:** el test de fuente `arriba_no_se_acopla_sin_sesion` encuentra `abierta()` antes de
`ventana_de_la_reunion` en `acoplar_arriba`, y la rama `Arriba` del latido no la llama. Antes del ajuste está en rojo.

### M4 · Arriba se da por acoplada una ventana que bajó sin encogerse, y sus controles salen de la pantalla

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:** `src-tauri/src/acople/mod.rs:539` (`hubo_cambio && quedo_bajo_la_franja`) y `:255-256` (los dos pasos se
ejecutan seguidos).

**Qué pasa:** una app acepta `AXSize` y no cambia de alto (en el H1 se midió «Code 923 → 923»), y aun así se mueve
88 pt hacia abajo. Su borde inferior, con los controles de la llamada, sale de la pantalla, y se anota como
«acoplada».

**Ajuste:**
1. Nueva función:
   ```rust
   pub fn acople_arriba_logrado(antes: Marco, dejada: Marco, franja: Marco) -> bool {
       hubo_cambio(antes, dejada) && quedo_bajo_la_franja(dejada, franja) && (dejada.fondo() - antes.fondo()).abs() <= HOLGURA
   }
   ```
   y usarla en `:539`.
2. Ejecutar primero el paso del alto.
3. Si el alto leído no es el pedido (± `HOLGURA`): deshacer con `Paso::Alto(marco.alto)`, añadir el motivo «no se
   dejó encoger» y **no mover**.

**Verificado cuando:** el test `arriba_si_no_se_dejo_encoger_no_cuenta_como_acoplada` pasa:
`(100,38,1300,944)→(100,126,1300,944)` es falso y `…→(100,126,1300,856)` es verdadero. Antes del ajuste está en rojo.

### M5 · Después de ⌥⎋ (la banda cerrada), ⌃⌥B o el selector de Sesión acoplan ventanas sin banda

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:**
- `src-tauri/src/lib.rs:274-282` (`acoplar_segun_el_borde`), al que llaman `poner_la_banda` (`:325-338`) y el latido
  (`:3602`);
- `ajustar_banda` se salta en silencio las ventanas que no existen (`src-tauri/src/ventana/mod.rs:198-210`).

**Qué pasa:** justo después del corte, ⌃⌥B encoge o baja la reunión y deja una franja vacía. Contradice la l3b de la
guía y el ADR 004 («la única forma de cerrar la banda»).

**Ajuste:** primera línea de `acoplar_segun_el_borde`:
```rust
if app.get_webview_window(ventana::BANDA).is_none() {
    return Err("la banda no está en pantalla: no se toca ninguna ventana".into());
}
```

**Verificado cuando:** el test de fuente `sin_banda_no_se_acopla` encuentra `get_webview_window(ventana::BANDA)`
antes de `acople::acoplar(`. Antes del ajuste está en rojo.

### M6 · R o S a media frase meten lo descartado en la respuesta nueva

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:** `src-tauri/src/ensayo/oido.rs:234-247`. `latir` sella cada turno con la ronda del latido en que se
**cierra**, no en la que empezó. `src-tauri/src/ensayo/mod.rs:436` lo acepta.

**Qué pasa:**
- **Sin voz:** hablas, pulsas R, el turno sigue y se cierra con la ronda nueva y con todo su audio. «R descarta lo que
  llevabas» no se cumple.
- **Con voz:** su texto llega con `desde = hasta = abrio_ms` y, a los 2,5 s, cierra la respuesta con lo descartado.

**Ajuste:** en `Oido`, el campo `ronda: u64`. En `latir`:
```rust
let (oye, del_lote) = if ronda != self.ronda { (false, self.ronda) } else { (oye, ronda) };
self.ronda = ronda;
```
y sellar los `Encargo` con `del_lote`.

**Verificado cuando:** el test `un_turno_que_empezo_antes_de_repetir_sale_con_su_ronda` pasa con `Oido::con_anillo`
y la secuencia silencio 600 → voz 800 → (ronda 2) voz 200 → silencio 500: todos los turnos recibidos tienen ronda 1.
Antes del ajuste está en rojo.

### M7 · Al pulsar R o S mientras la voz lee, el micrófono se abre antes de que suene la pregunta nueva

**Estado:** **pagado**, con su rojo sobre la fuente de Swift (Fase 2, `f3dd820`): en `cargo test` la voz no avisa de que terminó, y el test con los altavoces que proponía la auditoría pasaba con el fallo puesto (desviación 28). Lo de verdad —R mientras lee— va al ⭐.

**Sitio:** `src-tauri/nativo/Habla.swift:105-111`. `didFinish` y `didCancel` de **cualquier** frase apagan la
bandera. `src-tauri/src/ensayo/sesion.rs:207-219` se fía de ella.

**Qué pasa:** R durante la lectura → el `didCancel` de la frase vieja llega después de `decir` y apaga la bandera.
Si el arranque de la nueva tarda más de 300 ms, el micrófono se abre y **la app se transcribe a sí misma** como tu
respuesta. Rompe el §6.3 del ADR 019.

**Ajuste:** en `Habla.swift`, recordar la última frase (`Ultima`, con `NSLock`, `poner` y `es`) y, en
`didFinish`/`didCancel`, apagar la bandera solo `if ultima.es(u)`.

**Verificado cuando:** el test `la_bandera_de_la_voz_no_cae_entre_callar_y_la_frase_siguiente`, en
`src-tauri/tests/contra-el-mac-de-verdad.rs` y marcado `#[ignore = "hardware: los altavoces; lo corre la CI"]`,
lee `hablando()` 60 veces cada 10 ms y todas valen `true`. Lo corre la CI de macOS. Antes del ajuste está en rojo.

### M8 · Un ruido sin texto arranca el reloj del silencio: «pensar antes de empezar» sí cierra la respuesta

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:** `src-tauri/src/ensayo/sesion.rs:233-239`. `self.tramos.last()` no mira si el tramo tiene texto.

**Qué pasa:** carraspeas (≥ 300 ms, cuenta como turno), el motor devuelve vacío, y a los 2,5 s se evalúa una
respuesta vacía como «respondida». Eso ensucia el informe y el progreso.

**Ajuste:**
- La constante `VOZ_SIN_TEXTO_MS: u64 = 1_500`.
- En `tick`, el reloj cuenta desde el último tramo con texto, o desde el último que dure al menos eso (sin modelo de
  voz, las respuestas de verdad llegan sin texto).

**Verificado cuando:** el test `una_tos_antes_de_hablar_no_cierra_la_respuesta` pasa:
- un tramo de 1.000 a 1.400 sin texto, y `tick(5_000)` vacío;
- un tramo de 6.000 a 9.000 con texto, y `tick(11_600)` da `Evaluar`.

Antes del ajuste está en rojo.

### M9 · «Preparar» relee y parsea cada propuesta del disco, en el hilo principal y con el candado del corpus puesto

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:** `src-tauri/src/corpus/mod.rs:304-318` (`propuestas_de` hace `leer::leer` con `pdf-extract`). La llaman
`src-tauri/src/lib.rs:1158-1220` (en cada cambio de cliente, propuesta o tope) y `empezar_el_ensayo`. Los comandos
síncronos de Tauri corren en el hilo principal.

**Qué pasa:**
- Con 30 PDF, la interfaz se congela segundos en cada cambio.
- Un PDF que cuelgue a `pdf-extract` cuelga la app.
- Mientras tanto, el corpus queda bloqueado para el disparo.

**Ajuste:**
1. En `Documento` (`src-tauri/src/corpus/mod.rs:75-87`), el campo `#[serde(skip)] pub texto_plegado: Option<String>`.
   `indexar_uno` (`:196-197`) lo rellena solo para `Unidad::Propuesta`.
2. `propuestas_de` busca en él, sin releer.
3. `preparar_el_ensayo` pasa a `#[tauri::command(async)]`.

**Verificado cuando:** el test `propuestas_de_no_relee_el_disco` indexa, borra el `.md` del disco y aun así da 1.
Antes del ajuste da 0.

### M10 · Empezar una sesión borra sin avisar el informe del ensayo que no guardaste

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`). **Decisión del usuario:** Sesión lo dice en una línea, en el sitio de la promesa del corte, hasta que guardes o cierres el ensayo.

**Sitio:** `src-tauri/src/lib.rs:671` (`soltar_el_ensayo` dentro de `empezar`, comprobado por el constructor).
Sesión no sabe que hay un ensayo: `src/pantallas/Sesion.tsx:263-271`.

**Qué pasa:** es el flujo central. Ensayas justo antes, terminas, entra el cliente y pulsas «Iniciar sesión». El
informe y tus respuestas desaparecen sin una palabra. Contradice el «el informe sigue aquí, entero» de la propia
pantalla.

**Ajuste: decisión del usuario.** Recomendación: avisar en Sesión.
1. `src/componentes/Principal.tsx`: `const ensayoSinGuardar = hayTauri() && ensayo?.fase === "cerrado";`, pasado a
   `<Sesion … ensayoSinGuardar />`.
2. `Sesion.tsx`, encima de la fila de «Iniciar sesión»: una `franja warn` con `role="status"`.
3. Los textos:
   - es: «Tienes un ensayo terminado sin guardar» / «Empezar la sesión lo descarta. Guárdalo antes en Ensayo.»
   - en: «You have a finished rehearsal you have not saved» / «Starting the session discards it. Save it first in
     Rehearsal.»
4. En `docs/diseno/sesion.html`, el estado «sprint 4 · ensayo sin guardar», maquetado y no visto.
5. La alternativa: cortar solo el ensayo que aún escucha. El informe sobrevive a la sesión, y Honestidad lo cuenta.

**Verificado cuando:** un test renderiza `<Sesion ensayoSinGuardar />` y encuentra el texto. Antes del ajuste está
en rojo.

### M11 · Mantener pulsada una tecla manda la orden una y otra vez

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:** `src/pantallas/Ensayo.tsx:632-648` (`alPulsar`, sin mirar `e.repeat`). Sonda: 3 llamadas.

**Qué pasa:** Enter sostenido encadena «siguiente» → cierra una respuesta vacía. Una S sostenida salta varias
preguntas. El informe y el progreso quedan con datos falsos.

**Ajuste:** en `:633`, `if (e.repeat || e.isComposing || e.metaKey || e.ctrlKey || e.altKey) return;`.

**Verificado cuando:** Enter y S con `repeat: true` dan una sola llamada cada uno. Antes del ajuste está en rojo.

### M12 · «Muletillas 0» sin ninguna palabra (un cero inventado), y la webview recalcula reglas de Rust

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:**
- `src/pantallas/Ensayo.tsx:848-851`, `:911`, `:1024`;
- `src-tauri/src/ensayo/sesion.rs:522` (`muletillas: u32`);
- `src-tauri/src/ensayo/guardado.rs:89-90`.

**Qué pasa:** sin modelo de voz o con todo saltado, se ve «Muletillas 0» mientras «Ritmo» dice «—». El mismo ensayo,
guardado, sale «—» en el progreso. Rompe el §9-undecies («jamás un cero inventado»). Además, `citadas` y el total de
muletillas se recalculan en la webview, cuando Rust ya los tiene (`src-tauri/src/ensayo/evaluacion.rs:81-87`).

**Ajuste:**
1. En `Informe`, `muletillas: Option<u32>`, en `None` sin respondidas.
2. En `guardado.rs:90`, `map_or("—")`.
3. `VistaDelEnsayo` gana `usadas: usize` y `muletillas: Option<u32>`, en `None` sin palabras.
4. Muestras del contrato, regenerar, y los tipos de `src/ensayo.ts`.
5. `Ensayo.tsx` lee `vista.usadas` y `vista.muletillas`, y pinta «—» con `null`.

**Verificado cuando:**
- un informe todo saltado pinta «—» en «Muletillas»;
- en Rust, `assert_eq!(todo_saltado.informe().muletillas, None)`.

Antes del ajuste, los dos están en rojo.

### M13 · Volver del progreso, o terminar un ensayo, te devuelve a otro cliente, otro tope y la voz encendida

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:** `src/pantallas/Ensayo.tsx:135-138` (el estado es local de `Preparar`, que se desmonta en `:89-108`).
Confirmado con una sonda.

**Qué pasa:** eliges Sur del Valle, 12 preguntas y la voz apagada → «Ver tu progreso» → «Volver». Vuelves a Páramo
Azul, 8 y con voz. Un clic en «Empezar» ensaya con el cliente equivocado.

**Ajuste:**
1. Subir `cliente`, `propuesta`, `tope` y `voz` a `Ensayo` como `eleccion`, y pasarlos a `Preparar` por props.
2. `TuProgreso.volver(cliente)`.

**Verificado cuando:** el test «volver del progreso conserva lo elegido» ve el último `preparar_el_ensayo` con
`{cliente:"Sur del Valle", tope:12}`. Antes del ajuste está en rojo.

### M14 · Si borrar falla, la promesa rechazada queda sin manejar y la pantalla calla

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:** `src/pantallas/Ensayo.tsx:559` (`void borrarLosEnsayos(p.cliente).then(volver)`). Confirmado con una sonda.

**Ajuste:**
1. En `TuProgreso`, el estado `noSeBorraron`, y `.then(() => volver(p.cliente), () => setNoSeBorraron(true))`.
2. Una `franja err` con `role="alert"`:
   - es: «No se borraron tus ensayos» / «Siguen cifrados y en su sitio. Vuelve a intentarlo.»
   - en: «Your rehearsals were not deleted» / «They are still encrypted and in place. Try again.»
3. Las dos frases, literales, en `docs/diseno/ensayo.html`, «10 · textos de lo guardado».

**Verificado cuando:** con un rechazo de `borrar_los_ensayos`, la pantalla enseña el texto. Antes del ajuste está en
rojo.

### M15 · La confirmación de borrar (`alertdialog`) no recibe el foco, y al cancelar el foco se pierde

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`) (Ensayo y Notas).

**Sitio:** `src/pantallas/Ensayo.tsx:549-581`. El mismo patrón está en `src/pantallas/Notas.tsx:1007`. Confirmado con
una sonda: el foco acaba en BODY.

**Ajuste:** dos `useRef` (abrir y cancelar) y un `useEffect` que, al abrir, enfoca «Cancelar» y, al cerrar, devuelve
el foco al botón que la abrió. Lo mismo en Notas.

**Verificado cuando:** tras abrir, «Cancelar» tiene el foco; tras cancelar, lo tiene «Borrar los ensayos de este
cliente». Antes del ajuste está en rojo.

### M16 · Enter con el foco en un enlace del rail cierra tu respuesta, y el enlace no navega

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:** `src/pantallas/Ensayo.tsx:641`. Solo se exceptúa BUTTON, y `e.preventDefault()` (`:648`) cancela el
`<a>`. Confirmado con una sonda.

**Ajuste:** `if (e.key === "Enter" && objetivo?.closest("a, button")) return;`.

**Verificado cuando:** Enter sobre un `<a>` con el foco no llama a `ensayo_listo`. Antes del ajuste está en rojo.

### M17 · El contrapeso de reduced-motion no comprueba la cuenta de preguntas, y la orden la exige

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`). **El primer intento pasó en verde:** `seVe` miraba la opacidad del elemento y no la de su cadena; se arregló el gate y después dio sus dos rojos.

**Sitio:** `tests/e2e/reduced-motion.spec.ts:31-36`. La cuenta se pinta en `src/pantallas/Ensayo.tsx:680-683`
(`.progreso`).

**Qué pasa:** la orden pide el temporizador **y** la cuenta visibles sin movimiento: «sin ellos no hay derecho a
diferir». Hoy solo se comprueba el temporizador.

**Ajuste:** una entrada más:
```ts
{ que: "ensayo · la cuenta de preguntas", url: "ventana=principal&pantalla=ensayo&estado=preguntando", clave: ".progreso > span:first-child" },
```

**Verificado cuando:** las dos pruebas nuevas, en los dos temas, están en verde. Plantar
`.progreso{opacity:0}` bajo `prefers-reduced-motion` las pone en rojo (con `demo-rojo.sh`).

### M18 · El tope de fábrica es un literal y una posición, no un dato (casilla 7)

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:**
- `src/pantallas/Ensayo.tsx:137` (`useState(8)`);
- `src-tauri/src/lib.rs:1138-1145` (`topes.get(1)…unwrap_or(8)`);
- `data/ensayo/reglas.json` (sin `tope_de_fabrica`);
- `src-tauri/src/ensayo/banco.rs:560` (`[5,8,12]` fijo en el test).

**Ajuste:**
1. `"tope_de_fabrica": 8` en `reglas.json`, y su campo en `ArchivoDeReglas` y en `Catalogo`.
2. `fn tope_valido(tope: Option<usize>) -> usize`, que usa el de fábrica si el pedido no está en el catálogo.
3. Los comandos reciben `tope: Option<usize>`, y en `Ensayo.tsx` es `useState<number | null>(null)`.
4. El test pasa a afirmar `c.topes.contains(&c.tope_de_fabrica)`.

**Verificado cuando:** `"tope_de_fabrica": 7` pone el test en rojo.

### M19 · La puerta local dice «hay una reunión» cuando la cierra un ensayo

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`): las claves, `ghost`, su ayuda, la maqueta, el manual, la guía, `design-system.md`, el BLUEPRINT y el `CLAUDE.md`.

**Sitio:**
- `src-tauri/src/lib.rs:1265` → `:3390-3395` (`Cierre::EnReunion`);
- `src/pantallas/Ia.tsx:567`;
- `src/i18n/es.ts:864`, `:868`, `:886`, `:903` y sus pares en `en.ts` (`:749`, `:753`, `:771`, `:788`);
- `src-tauri/src/bin/ghost.rs:119-120`;
- `src-tauri/src/puerta/cli.rs:85` y `:104` (`ghost --help`).

El constructor lo anotó en la bitácora.

**Ajuste (solo texto; la puerta se cierra igual):**

| Clave | es | en |
|---|---|---|
| `seCerroSola` | «Se cerró sola: hay una reunión o un ensayo» | «It closed itself: there is a meeting or a rehearsal» |
| `noAbre["en-reunion"]` | «Hay una reunión o un ensayo, o no se puede saber si hay reunión.» | «There is a meeting or a rehearsal, or it cannot be known whether there is a meeting.» |
| `reunionPor` | «en reunión o en un ensayo se cierra sola» | «in a meeting or a rehearsal it closes itself» |
| `motivo` | «en reunión o ensayo» | «in a meeting or rehearsal» |
| `ghost.rs` | «Denegado: hay una reunión o un ensayo. La puerta se cerró sola; se vuelve a abrir a mano, en IA.» | «Denied: there is a meeting or a rehearsal. The door closed itself; it is reopened by hand, in AI.» |
| ayuda de `ghost` | «En reunión o durante un ensayo se cierra sola y lo deniega todo.» | «In a meeting or during a rehearsal it closes itself and denies everything.» |

Los mismos cambios en:
- `docs/diseno/ia.html:44`, `:447`, `:462`;
- `docs/MANUAL-DE-USO.md:611`, `:631`, `:635`;
- la guía: la n7 (parada 8, «Mejorado en S4»), la fila T19 (con una nota) y la r9;
- `design-system.md:326`, `docs/BLUEPRINT.html:475` y `CLAUDE.md:742`.

**Verificado cuando:** el gate i18n, `cargo test --test ghost` (la ayuda) y `puerta.test.tsx` están en verde.

### M20 · La app afirma que la protección está verificada en la posición que el manual dice que nadie ha mirado

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`). **Decisión del usuario:** «verificado… con la banda abajo; con la banda arriba, y en Zoom y Teams, está sin verificar», en los nueve estados de Sesión.

**Sitio:**
- `src/i18n/es.ts:320-321` y `en.ts:244-245` (`proteccionDetalle`: «Verificado en tu Mac … el 2026-09-20»);
- el chip «Meet · protegido» (`src/i18n/es.ts:27`);
- `README.md:8-10` y `:59-61`.

El chip y la tarjeta no miran el borde (`src/componentes/Banda.tsx:240-249`, `src/pantallas/Sesion.tsx:660-676`).
Pero el manual (`docs/MANUAL-DE-USO.md:92-94`) y la q2 dicen que con la banda arriba «todavía no se ha mirado».

**Ajuste: decisión del usuario, porque es la promesa.** Recomendación: la app dice solo lo verificado.
1. `proteccionDetalle`:
   - es: «Tu panel no aparece en la pantalla que compartes. Verificado en tu Mac (macOS 26.6.2) el 2026-09-20 con la
     banda abajo; con la banda arriba, y en Zoom y Teams, está sin verificar.»
   - en: «Your panel does not show in the screen you share. Verified on your Mac (macOS 26.6.2) on 2026-09-20 with
     the band at the bottom; with the band at the top, and in Zoom and Teams, it is unverified.»
   - Va también en los nueve estados de `docs/diseno/sesion.html` (`:88`, `:234`, `:275`, `:315`, `:359`, `:409`,
     `:458`, `:503`, `:548`).
2. El README añade «con la banda abajo; arriba, su sitio de fábrica desde el sprint 004, usa la misma protección y
   está sin verificar».
3. La alternativa: un ADR que declare que la protección no depende de la posición (es un flag de la ventana), y
   retirar la salvedad del manual.

**Verificado cuando:** la app, el README y el manual dicen lo mismo, y el gate i18n está en verde.

### M21 · «Ver lo que salió» registra lo del ensayo como «redactar sugerencia» y como parte de «esta reunión»

**Estado:** **pagado**, con su rojo antes del verde (Fase 2, `f3dd820`).

**Sitio:**
- `src/pantallas/Ia.tsx:513`;
- `src/i18n/es.ts:595`, `:600`, `:601`, con sus pares en `en.ts`;
- `src-tauri/src/lib.rs:1302-1320` (mismo `Api`, con `sobre` = la propuesta);
- `:2784-2786` (`cobrar` suma a «Esta reunión»).

**Ajuste:**
1. `LoQueSalio` (`src-tauri/src/sintesis/api.rs:204`) gana `para: sugerencia | banco`, y `enriquecer_el_ensayo` lo
   marca `banco`. Regenerar el contrato.
2. En `Ia.tsx:513`, «enriquecer el banco» / «enrich the bank» cuando `para === "banco"`.
3. `registroEnMemoria`:
   - es: «Este registro vive en memoria como todo lo demás: se borra al terminar la sesión, con ⌥⎋ o al salir de la
     app. Lo que persiste del costo es la cifra del mes, no el texto.»
   - en: «This log lives in memory like everything else: it is erased when the session ends, with ⌥⎋ or when you
     quit the app. What persists of the cost is the monthly figure, not the text.»
4. `peticionesDeEstaReunion`: «peticiones recientes» / «recent requests».
5. Lo mismo en `docs/diseno/ia.html:177`, `:181-183`, `:424`, `:428-430`, y en el manual (`:575`).

**Verificado cuando:** con el mock, un ensayo enriquecido deja la fila «enriquecer el banco · …» (test de `Ia`).

### M22 · El manual y el README dicen qué sale a la red y no cuentan el ensayo

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`).

**Sitio:** `docs/MANUAL-DE-USO.md:769-771` («solo salen la última frase de tu cliente y tres fichas cortas») y
`README.md:18-19` / `:67-68`. Con «Enriquecer el banco» salen, una vez por ensayo, los títulos y la primera frase de
las secciones de tu propuesta y de la ficha (`src-tauri/src/ensayo/enriquecer.rs:60-92`).

**Ajuste:**
- Manual: «…y aun entonces solo sale texto corto y anonimizado: con «Redactar sugerencias», la última frase de tu
  cliente y tres fichas cortas; con «Enriquecer el banco», los títulos y la primera frase de las secciones de tu
  propuesta y de la ficha del cliente, una vez por ensayo —nunca un documento entero—.»
- README: «…un proveedor externo para las sugerencias o para «Enriquecer el banco» del ensayo» / «…an external
  provider for suggestions or for the rehearsal's "Enrich the bank"».

**Verificado cuando:** el barrido de la segunda casilla 4 está limpio.

### M23 · El README no cuenta los ensayos entre lo que queda

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`).

**Sitio:** `README.md:12-17` y `:63-67`.

**Ajuste (es; el equivalente en inglés en la parte inglesa):** «Lo único que queda es lo tuyo: tus notas, tus
acuerdos, las fichas que fijaste, las propuestas que aceptes —de lo que dijo el cliente, como mucho un hecho de una
línea— y, si lo enciendes, lo que dijiste tú en texto, en un archivo cifrado por reunión con la llave en tu Llavero;
y tus ensayos, si los guardas: tus respuestas en texto y sus cifras, con la misma llave.»

**Verificado cuando:** el barrido de la segunda casilla 4 está limpio.

### M24 · El manual dice que la banda arriba se acopla «cuando aparece la reunión»

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`).

**Sitio:** `docs/MANUAL-DE-USO.md:66-67`. Sola, solo lo hace en los 30 s del latido del arranque
(`src-tauri/src/lib.rs:3580-3610`).

**Ajuste:** depende de M3. Con la recomendación de M3: «Sin sesión, la banda flota bajo la barra de menús y dice
«sin acople»; se acopla al iniciar la sesión.» Sin ella: «…se acopla al iniciar la sesión; sola, solo si la reunión
ya está abierta o aparece en los primeros 30 segundos tras abrir la app.»

**Verificado cuando:** el barrido de la segunda casilla 4 está limpio.

### M25 · Guía q4: ⌃⌥B con el cuaderno delante no acopla nada abajo

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`).

**Sitio:** `docs/GUIA-DE-PRUEBA.html:791`. Abajo se acopla la app que está delante, y Angel Ghost no cuenta
(`src-tauri/src/acople/ax.rs:321-328`). Además, «Con Meet cerrado» nunca pide cerrarlo.

**Ajuste (sigue «Nuevo · S4»):** «**Cierra la pestaña de Meet**, sal con ⌘Q y vuelve a arrancar: … Abre tu sala de
Meet y pulsa «Iniciar sesión»: la ventana de la reunión se acopla. **Haz clic en la ventana de Chrome** para traerla
delante y pulsa ⌃⌥B: la banda baja al borde inferior, la reunión se suelta entera y se acopla como en el H1 —abajo
se acopla la ventana que tengas delante—; otra vez ⌃⌥B, y vuelve arriba. …» Se ajusta a M3 si cambia.

**Verificado cuando:** `guia-cuadra` está en verde.

### M26 · El comando del WER ya no corre nada

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`): el comando del LEEME lleva `--include-ignored`, y el texto dice que el WER se mide a mano.

**Sitio:** `docs/kit-de-prueba/audio/LEEME.md:29-35` y `:67`. Desde la fase 0, el test lleva `#[ignore = "hardware:
…"]` (`src-tauri/tests/contra-el-mac-de-verdad.rs:1726-1728`): el comando copiado da «1 ignored».

**Ajuste:**
- El texto: «Se mide **a mano**, en un Mac con los modelos de voz instalados: desde el sprint 004 `cargo test` a secas
  no toca el reconocimiento de voz y este test lleva `#[ignore = "hardware: …"]`; se corre con su matriz y su «sí».
  En la CI corre con `--include-ignored`, dice «el WER no se pudo medir en ninguna pista» y no mide nada.»
- El comando: `cd src-tauri && cargo test --test contra-el-mac-de-verdad el_wer -- --include-ignored --nocapture`.

**Verificado cuando:** el comando, copiado del render, ya no dice «1 ignored» (sin correrlo aquí: toca la voz).

### M27 · `docs/BLUEPRINT.html` sigue describiendo la app del H1

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`).

**Sitio:** `docs/BLUEPRINT.html:139` y siguientes. No se tocó en el sprint.

**Ajuste:** las frases de esta tabla, más la pieza nueva «El ensayo (C18)», con `ensayo/`, `ensayos.rs` y
`data/ensayo/*.json`, y la carpeta `…/ensayos/` (600) en la sección 4.

| Línea | Ahora debe decir |
|---|---|
| `:139` | «Ciclos: H1 (S1–S3) y H2 en curso (S4)», el commit y la fecha |
| `:148` | lo que persiste incluye «tus ensayos» |
| `:185` | «8 pantallas» |
| `:191` | «88 px, arriba (o abajo)» |
| `:379` | la banda bajo la barra de menús de fábrica, y la cuenta de comandos |
| `:381` | las ocho pantallas, por su nombre |
| `:393` | «17 módulos», con `ensayo` |
| `:403` | «ensayos» en el escritor común |
| `:404` | «doce piezas», con el ensayo |
| `:405` | el acople, abajo y arriba |
| `:406` | las tres preferencias nuevas |
| `:439` | «o el ensayo que acabas de terminar» |
| `:451` | «y de tus ensayos» |
| `:357` y `:524` | el Touch ID de los ensayos |
| `:519` | el micrófono del ensayo |
| `:552` | «tus notas y tus ensayos guardados» |
| `:581-585` | el S4 sale de «Lo que viene» y entra como construido, con su fila de historial |

**Verificado cuando:** el barrido de la segunda casilla 4 sobre el BLUEPRINT está limpio, igual que el de CERO
ENLACES.

### M28 · `design-system.md`, la fuente de verdad visual, se contradice

**Estado:** **pagado**, de texto o documento: sin gate que demostrar (Fase 2, `f3dd820`).

**Sitio:**
- `design-system.md:116` y `:124-125` (la banda inferior como forma principal y por defecto), frente al §9-decies
  («arriba es el sitio de fábrica»);
- el §9-bis (`:332-339`) no nombra los ensayos, ni las propuestas, ni la bandeja;
- la fila de los turnos dice «conmutador en Honestidad».

**Ajuste:**
- `:116`: «Banda — forma PRINCIPAL en reunión | ancho completo × 88 · 200 · 44 | arriba de fábrica (bajo la barra de
  menús y el notch, §9-decies) o pegada al borde inferior si el usuario la prefiere (Sesión o ⌃⌥B), con asa.
  Acoplada por defecto: abajo, la reunión se recorta; arriba, baja y se encoge. Sin el permiso de acople, flota
  encima».
- `:124`: «banda acoplada, 88 px, arriba (abajo si lo eliges)».
- `:125`: «banda acoplada, 44 px, en su borde».
- El §9-bis gana tres filas: las propuestas que aceptas, la bandeja por horas y tus ensayos.
- La fila de los turnos pasa a «conmutador en Notas; apagado de fábrica; con la retención de tus notas».
- Regenerar `design-sync/`.

**Verificado cuando:** `design-sync-espejo` está en verde y el barrido de la segunda casilla 4 está limpio.

## BAJOS (50)

| # | Sitio | Qué está mal | Ajuste · verificado cuando | Estado |
|---|---|---|---|---|
| **B1** | `src-tauri/tests/contra-el-mac-de-verdad.rs:1886` | `la_app_nunca_habla_por_los_altavoces_internos` dice algo falso: en el ensayo la app sí habla por ellos. El plan lo iba a renombrar y no se hizo ni se declaró | Renombrar a `en_reunion_la_app_nunca_habla_por_los_altavoces_internos` y añadir al doc «en el ensayo sí, con el micrófono sordo (ADR 019 §6.3)». Verificado: el filtro `la_app_nunca` del paso de la CI sigue listándolo | pagado · renombrado |
| **B2** | `sprints/SPRINT_004-implementation-log.md` (§ Desviación del plan) | Dos cambios de la guía no están en la lista de desviaciones: la parada 6 dice «12 de 12», y A–P y el ⭐⭐ del H1 se corren con la banda abajo | Añadir la desviación 26. Verificado: la lista los nombra | pagado · desviación 26 |
| **B3** | `sprints/SPRINT_004-implementation-log.md` (PR #12 y #10) | El #12 se mergeó sin traer lo que su título promete y sin comentario de corte. El cuerpo del #10 sigue con casillas sin marcar y «En borrador» | Comentar en el #12 qué trajo y qué quedó fuera, y actualizar las casillas del #10 (pide el «sí» del usuario: se publica). Verificado: `gh pr view` lo enseña | pagado · publicado con el «sí» del usuario: cuerpo del #10 y comentario en el #12 |
| **B4** | `sprints/SPRINT_004-implementation-log.md` (fase 1, corrida en vivo) | Dice «Matriz de una fila enseñada» pero no la copia | Pegar la fila del plan (qué · para qué · aviso · cómo se deshace) junto al «sí». Verificado: la bitácora la tiene | pagado · la fila, en la bitácora |
| **B5** | `tests/unit/cargo-test-sin-hardware.test.ts:32-47` · `src-tauri/src/acople/ax.rs:164` | Las agujas no incluyen `Oido::del_microfono(`, `DelLlavero`, `reunion::desbloquear(` ni `ventana_de_la_reunion(`. El centinela no cubre las **lecturas** de Accessibility (`Aplicacion::de`, `titulos_con_indice`, `indice_de_la_principal`, `marco_de_la_ventana`) | Añadir las agujas y `crate::hardware::vigilar("leer ventanas de otra aplicación (Accessibility)")` en `Aplicacion::de`; los `CENTINELAS` de `ax.rs` pasan a 4. Verificado: `AG_SIN_HARDWARE=1 cargo test` en verde y demo en rojo con `demo-rojo.sh` | pagado · con su rojo |
| **B6** | `docs/kit-de-prueba/audio/LEEME.md` | El WER no tuvo corrida local en el S4: estaba atada a la corrida en vivo, que se aplazó | El summary dice que el WER no se midió en el S4, sin afirmar regresión ni no regresión. Verificado: la frase está en el summary | pagado · el summary lo dice |
| **B7** | `sprints/SPRINT_004-implementation-log.md` (DoD de rendimiento) | El peso del binario no está anotado (S3: 12,79 MB) | En el `/release-check`, `pnpm tauri build` (release) y anotar el tamaño frente a 12,79 MB. Verificado: la cifra está en la bitácora y el summary | pagado · `/release-check` y summary |
| **B8** | `docs/MANUAL-DE-USO.md:386` | «Guardar no te pide nada» no menciona que macOS puede avisar de un ítem en segundo plano (ni la pregunta del Llavero si la llave la creó otra compilación) | Añadir: «La primera vez que guardas algo con fecha, macOS puede avisar de un ítem en segundo plano añadido (Background Items Added): es la tarea que borra lo vencido.» Verificado: segunda casilla 4 | pagado · texto |
| **B9** | `src-tauri/tests/contra-el-mac-de-verdad.rs:981-992` | En marcha no hay término plantado para el log del ensayo: el paso 7-ter no lleva canaria propia | Plantar un término propio al principio de «Plazo de entrega» en `corpus_para_el_efimero()` y buscarlo también en el log. Verificado: con un `println!` plantado del término, la CI se pone en rojo | pagado · su rojo, en la CI (PR #14, desechable) |
| **B10** | `sprints/SPRINT_004-implementation-log.md` (contrapeso de capturas) | Solo se leyeron como imagen 6 encuadres de producto nuevos | Leer como imagen uno por bloque: preparar, preguntando, respondiendo, progreso, borrar, sin corpus, banda arriba, aviso de Sesión e IA. Registrar N en el summary. Verificado: la cuenta está en el summary | pagado · 12 encuadres más leídos como imagen; la cuenta, en el summary |
| **B11** | `src-tauri/src/sesion/mod.rs:266` · `:282` | `indice_de_meet` usa señales de Meet cableadas e ignora las `senales` que da `objetivo_de` (`:252`) | Cambiar a `indice_por_senales(titulos, &senales)` y pasar `&senales` en `:282`. Verificado: test con señales de un catálogo de prueba | pagado · con su rojo |
| **B12** | `src-tauri/src/ensayo/banco.rs:560` | El catálogo no se cruza con el código: una regla nueva en `reglas.json` se ignora en silencio | `assert_eq!(c.reglas.len(), Regla::DEL_BANCO.len())` y cada `id` con su variante. Verificado: un id inventado en el json lo pone en rojo | pagado · con su rojo |
| **B13** | `src/i18n/es.ts:331` · `src/i18n/en.ts:255` | `sesion.funcionaBanda` («La banda, abajo, protegida…») no tiene lector y es falsa hoy | Borrar la clave en es y en en: la maqueta conserva los estados de historia s1/s2. Verificado: `pnpm typecheck` y `pnpm test` en verde | pagado · la clave, borrada (typecheck) |
| **B14** | `src/pantallas/Ensayo.tsx:295` · `src/pantallas/Sesion.tsx:314` · `docs/diseno/ensayo.html:92` | Once `fontSize: 11.5` en línea en Ensayo (`:295`, `:298`, `:351`, `:500`, `:532`, `:548`, `:708`, `:822`, `:934`, `:1069`, `:1097`), más `marginTop: -6` (`:708`) y `fontSize: "11px"` en `Sesion.tsx:314`. Son valores mágicos que el design system prohíbe (§3) | Clase `.ayuda-e { font-size: var(--t-mono); color: var(--ink-2); }` en `ghost.css`, y `.ayuda-e.tras { margin-top: 6px }`, usadas en Ensayo y en `ensayo.html`; Sesión, con su token. Verificado: un test que lea `Ensayo.tsx` encuentra 0 `fontSize:`, y `pnpm fidelidad` sigue bajo el umbral | pagado · con su rojo |
| **B15** | `src-tauri/src/ensayo/mod.rs:438-441` | Carrera entre recoger lo transcrito y contar lo pendiente: el último turno puede perderse al pulsar Enter | Leer `o.pendientes()` **antes** que `o.recibidos()`. Verificado: test de fuente con el orden | pagado · con su rojo |
| **B16** | `src-tauri/src/lib.rs:1431` · `src-tauri/src/ensayos.rs:75-78` | Un cliente sin letras latinas («東京商事») guarda `reunion-…`: no tiene progreso, no se borra, y «Reunión» se queda con los de todos | `ensayos::nombre_del_cliente`, con un hash cuando el slug queda vacío, y `ensayos::base(cliente, fecha)`. Verificado: test `un_cliente_sin_letras_latinas_tiene_sus_ensayos` | pagado · con su rojo |
| **B17** | `src-tauri/src/lib.rs:1656-1669` | Al salir de la app, el ensayo no se suelta: el micrófono sigue abierto hasta que muere el proceso y tus respuestas no se pisan | `soltar_el_ensayo(mango)` en `RunEvent::Exit`. Verificado: test de fuente | pagado · con su rojo |
| **B18** | `src-tauri/src/acople/mod.rs:187-198` | `asentar` se conforma con dos lecturas viejas iguales, antes de que la app empiece a aplicar el cambio | `asentar(leer, esperar, intentos, objetivo)`: vuelve al alcanzar el objetivo, o con ≥ 6 lecturas iguales. Verificado: test `asentar_no_se_conforma_con_dos_lecturas_viejas` (33, 33, 80, 121, 121 → 121) | pagado · con su rojo |
| **B19** | `src-tauri/src/lib.rs:3602-3617` | Si la app pelea su posición, el latido la mueve y la devuelve cada 1,5 s durante 30 s | `Informe.deshecho` y `seguir_esperando`, para no reintentar tras un deshecho. Verificado: test `tras_un_deshecho_el_latido_no_lo_reintenta` | pagado por M3: arriba el latido ya no acopla (desviación 29) |
| **B20** | `src-tauri/src/ensayos.rs:286-290` | El test de la ruta que se sale de la carpeta no puede fallar: los dos casos dan `Err` también sin la protección | Un ensayo de verdad en `notas/` (`../notas/x.ghost`) que no se abre desde `ensayos/`, con un control que sí abre. Verificado: puentear `nombre_valido` lo pone en rojo (con `demo-rojo.sh`) | pagado · con su rojo (el primer intento pasó en verde: sin `ensayos/` la ruta no se resolvía; el test crea la carpeta) |
| **B21** | `src-tauri/src/ensayo/enriquecer.rs:184-185` · `src-tauri/src/lib.rs:1338-1346` | `Resultado::fallo` no lo lee nadie, y el log dice «0 B fuera» cuando el API falló después de enviar | Loguear el fallo (sin contenido) y no imprimir `fuera` en ese caso. Verificado: test de fuente | pagado · con su rojo |
| **B22** | `src-tauri/src/ensayo/mod.rs:322-327` · `src-tauri/src/lib.rs:1316` · `src/pantallas/Ensayo.tsx:699` | «El modelo sumó 0 preguntas…» cuando todas repetían o el ensayo ya había cerrado, y `NadaFundado` cuando no había secciones | `cuantas == 0` pasa a `NoSeEnriquecio { Repetidas \| Tarde }`; `PorQueNo::SinSecciones` y `Repetidas` en el contrato y en i18n («las que propuso ya estaban en el banco.» / «the ones it suggested were already in the bank.», y para sin secciones «tu propuesta no tiene secciones con título.» / «your proposal has no titled sections.»), literales en `ensayo.html`. Verificado: test `lo_del_modelo_repetido_no_dice_que_sumo` | pagado · con su rojo |
| **B23** | `src-tauri/src/ensayo/sesion.rs:188` · `src-tauri/src/ensayo/evaluacion.rs:220-225` · `src-tauri/src/ensayo/oido.rs:191-193` | Copias de tu respuesta que se sueltan sin pisar: `Cerrada.tramos`, `respuesta` en `evaluar`, el texto del motor antes de corregir y `r.respuesta.json` en `src-tauri/src/lib.rs:1319-1350` | `impl Drop for Cerrada`, y `pisar` en `evaluar`, en el oído y en `enriquecer_el_ensayo`. Verificado: test `una_cerrada_se_pisa_al_soltarla` y test de fuente del `Drop` | pagado · con su rojo |
| **B24** | `src/pantallas/Ensayo.tsx:1044` · `docs/diseno/ensayo.html:216` | La tabla del informe dice «ppm» también en inglés; las cifras dicen «wpm» | `{t.ppm}`, y la maqueta con sus dos `<span lang>`. Verificado: en inglés, `columnheader` «wpm» | pagado · con su rojo |
| **B25** | `src/pantallas/Sesion.tsx:306` | Pulsar el borde que ya está elegido suelta y vuelve a acoplar la reunión | `if (!on) void fijarPosicionDeLaBanda(b);`, y en Rust, `poner_la_banda` no hace nada si el borde no cambia. Verificado: el clic en el radio activo no llama al comando | pagado · con su rojo |
| **B26** | `src/pantallas/Ensayo.tsx:156` · `:195-197` | Si `empezar` devuelve «sin-corpus», la pantalla no dice nada | `if (prep.sinCorpus \|\| noEmpezo?.que === "sin-corpus") return <SinCorpus …/>`. Verificado: con un rechazo «sin-corpus» se ve `nadaDeEsteCliente` | pagado · con su rojo |
| **B27** | `src-tauri/src/lib.rs:1265` · `src/pantallas/Honestidad.tsx:187` | «Ensayando · N» y Honestidad cuentan los bytes de la reunión anterior con el rótulo «en esta reunión» | `red::reiniciar()` al empezar el ensayo, y el rótulo «salieron de tu equipo en este ensayo» / «left your machine in this rehearsal», literal en `honestidad.html`. Verificado: test de fuente y test de Honestidad | pagado · con su rojo |
| **B28** | `src/pantallas/Honestidad.tsx:96` | Honestidad adivina que hay ensayo: es una regla de negocio en la webview, y con todo saltado la fila desaparece | `EstadoDeEscucha.ensayo: bool` en Rust (`src-tauri/src/escucha/mod.rs:161-209`), en las muestras y en el tipo; Honestidad lo lee. Verificado: test con el micrófono cerrado y 0 B que enseña la fila | pagado · con su rojo |
| **B29** | `src/i18n/es.ts:921-924` · `:945` · `src/i18n/en.ts:804-807` · `docs/diseno/ensayo.html:93` | «Idioma del ensayo: el de la propuesta» aunque no haya propuesta: Rust elige propuesta → ficha → tu idioma | «Idioma del ensayo: español.» / «Rehearsal language: Spanish.» (y su par en inglés); «Este Mac no sabe transcribir el idioma del ensayo» / «This Mac cannot transcribe the rehearsal's language»; literales en la maqueta y el manual. Verificado: `grep` de «de la propuesta» da 0 y el gate i18n sigue en verde | pagado · texto |
| **B30** | `src/pantallas/Ensayo.tsx:967` · `:1097-1101` | La línea de retención dice «90 d» antes de saber la tuya | `cuaderno?.retencion ?? null`, y la línea solo con valor. Verificado: sin respuesta del cuaderno, la línea no aparece | pagado · con su rojo |
| **B31** | `src/pantallas/Ensayo.tsx:516` | La `key` de React se repite con dos ensayos guardados en el mismo minuto | ``key={`${i}·${f.empezo}`}``. Verificado: sin `console.error` con dos filas iguales | pagado · con su rojo |
| **B32** | `src/pantallas/Ensayo.tsx:387` · `:399` | La franja de «no empezó» no lleva `role` y no se anuncia | `role="alert"`. Verificado: `getByRole("alert")` | pagado · con su rojo |
| **B33** | `src/pantallas/Ensayo.tsx:878` · `:900` | `Evidencia.fuente.conjeturada` cruza el contrato y el ensayo no la lee (casilla 5): la marca de sección conjeturada falta en la evaluación | La marca `.conjetura` con `seccionConjeturada`, la regla `.evidencia-e .conjetura` en `ghost.css`, y una muestra en `ensayo.html`. Verificado: con `conjeturada:true` se ve el texto | pagado · con su rojo |
| **B34** | `src-tauri/src/lib.rs:1235` · `:1287` · `:1295` | El `Ok(VistaDelEnsayo)` de `empezar_el_ensayo` no lo lee nadie (casilla 5), y su `SinCorpus` por una vista vacía está mal etiquetado | `-> Result<(), ensayo::NoEmpezo>`, y `preguntar<null>` en `src/ensayo.ts:475`. Verificado: `grep "Result<ensayo::VistaDelEnsayo"` da 0, y `cargo check` y el ensayo en verde | pagado · `grep` en 0 y `cargo check` |
| **B35** | `src-tauri/src/lib.rs:1445` | El `bool` de `exportar_el_ensayo` no lo lee nadie (casilla 5) | `-> Result<(), String>`, y `preguntar<null>` en `src/ensayo.ts:496-498`. Verificado: `grep` da 0 | pagado · `grep` en 0 y `cargo check` |
| **B36** | `src-tauri/src/lib.rs:1469` | El `usize` de `borrar_los_ensayos` no lo lee nadie (casilla 5) | `-> Result<(), String>`; la cuenta se queda en el `println!`. Verificado: `grep` da 0 | pagado · `grep` en 0 y `cargo check` |
| **B37** | `src-tauri/src/lib.rs:343-346` · `src/franja.ts:63-64` | El `LaFranja` de `fijar_posicion_de_la_banda` no lo lee nadie (casilla 5): el borde llega por el evento | Sin retorno, y `llamar(...)` en `franja.ts`. Verificado: `grep` da 0 | pagado · `grep` en 0 y `cargo check` |
| **B38** | `src-tauri/src/lib.rs:351-355` · `src/franja.ts:69` | El `LaFranja` de `entendido_el_aviso_de_arriba` no lo lee nadie (casilla 5) | Sin retorno; se emite el evento. Verificado: `grep` da 0 | pagado · `grep` en 0 y `cargo check` |
| **B39** | `src/pantallas/Ensayo.tsx:238-252` | El selector «Propuesta» sale vacío cuando el cliente solo tiene ficha | Pintar la fila solo con `prep.propuestas.length > 0`. Verificado: sin propuestas no hay `combobox` «Propuesta» | pagado · con su rojo |
| **B40** | `docs/GUIA-DE-PRUEBA.html:337` | El título del bloque B dice «siete pantallas» | «El cuaderno: ocho pantallas que no mienten». Verificado: `guia-cuadra` en verde | pagado · texto |
| **B41** | `docs/GUIA-DE-PRUEBA.html:734` · `docs/MANUAL-DE-USO.md:302-306` | La o4 y el manual dicen «mientras haya bandeja o notas con fecha» y dejan fuera los ensayos (`src-tauri/src/reunion.rs:845-847`) | «mientras haya bandeja, notas o ensayos con fecha», y en el manual «la bandeja, tus notas y tus ensayos vencidos»; la o4 pasa a «Mejorado en S4». Verificado: `guia-cuadra` | pagado · texto |
| **B42** | `docs/GUIA-DE-PRUEBA.html:823` · `docs/MANUAL-DE-USO.md:389-397` | El desbloqueo de los ensayos es el de tus notas (`src-tauri/src/reunion.rs:942-945`): la r7 depende de no haberlo dado antes, y «borrar no pide desbloqueo» se lee raro porque para llegar a tu progreso ya lo diste | La r7 añade «(si en esta sesión de la app ya abriste algo cifrado, no lo vuelve a pedir: es el mismo desbloqueo que tus notas)». Manual: «es el mismo desbloqueo que tus notas…» y «no abre los archivos ni pide otro desbloqueo». Verificado: segunda casilla 4 | pagado · texto |
| **B43** | `docs/GUIA-DE-PRUEBA.html:853` | La fila T3 cita «11 de 11», y hoy la app dice 12 | Añadir «(en el comparativo del S3; hoy dice «12 de 12», fila 4 de la tabla U)». Verificado: `guia-cuadra` | pagado · texto |
| **B44** | `docs/GUIA-DE-PRUEBA.html` (fila U3) | Dice que el interruptor va «bajo «Redactar sugerencias»», y está junto a él (`src/pantallas/Ia.tsx:226-246`) | «junto a». Verificado: lectura | pagado · texto |
| **B45** | `docs/MANUAL-DE-USO.md:207` · `:192-193` | El título «Tus notas: lo único que queda» (los ensayos también quedan), y «devuelve la ventana de la reunión a su tamaño» (arriba también se mueve) | «Tus notas: lo único de la reunión que queda», y «a su sitio y a su tamaño». Verificado: segunda casilla 4 | pagado · texto |
| **B46** | `src/i18n/es.ts:283` · `:436` · `src/i18n/en.ts:209` · `:340` · `docs/MANUAL-DE-USO.md:114-115` | «Lo que llega en el H2»: la ruta del H2 aprobada (S4 ensayo, S5 presencial, S6 Windows) no incluye arrastrar documentos, releer solo lo cambiado, leer lo escaneado ni varios idiomas por pista | **Decisión del usuario.** Recomendación: «Lo que no hace hoy» / «What it does not do today» (con `corpus.html:274` e `idioma.html:315`), y en el manual «no está en la app»; MLX se queda, porque lo respalda el ADR 011. Verificado: gate i18n | pagado · texto |
| **B47** | `docs/diseno/index.html:19` · `:92` · `docs/diseno/README.md:232-246` | El índice de la maqueta dice «diez pantallas»; Ensayo lleva el número 09, repetido; `posicion.html` no nombra la variante ARRIBA; dice «v1.7.0» | «once», «10 · sprint 004», los cinco estados ARRIBA con «elegida», «v1.15.0», y la fila de `ensayo.html` / C18 en el README. Verificado: `maqueta-interaccion` y `controladores-maqueta` | pagado · texto |
| **B48** | `CLAUDE.md:156` · `:739` · `:742` | La lista de pantallas sin Ensayo; los catálogos sin `data/ensayo/`; la puerta «cerrada en reunión» | Añadir Ensayo y `data/ensayo/`; «cerrada en reunión o durante un ensayo». Verificado: lectura | pagado · texto |
| **B49** | `README.md:23-24` · `:70-71` | «Estado» solo nombra el H1 | «El ciclo H2 está en curso: el sprint 004 sumó el ensayo y la banda arriba, junto a la cámara.» (y en inglés). Verificado: segunda casilla 4 | pagado · texto |
| **B50** | `design-system.md:326` · `:132` | La puerta «en reunión se cierra sola», y el relleno «una reunión que no llega al borde inferior» | «en reunión o durante un ensayo», y «a su borde (abajo, el inferior; arriba, la barra de menús)». Verificado: `design-sync-espejo` en verde | pagado · texto |

## Casilla 4 — frases caducadas (primera pasada)

**¿Qué afirmaba la app antes que ya no es cierto?**

| Tema | Frase de antes | Hallazgos |
|---|---|---|
| Dónde está la banda | «la banda, abajo» | M28, B13, B50 |
| Qué está verificado | la protección «verificada», sin decir en qué posición | M20 |
| Cuándo se cierra la puerta | «en reunión» | M19, B48 |
| Qué sale a la red | la lista no cuenta el ensayo | M22 |
| Qué queda | no cuenta los ensayos | M23, B45 |
| Qué vence | no cuenta los ensayos | B41 |
| Cuándo se acopla arriba | «cuando aparece la reunión» | M24 |
| El WER | se mide en cada `cargo test` | M26 |
| El BLUEPRINT | la app del H1 entera | M27 |
| Lo que llega «en el H2» | promesas sin sprint | B46 |
| El índice y el README de la maqueta | sin Ensayo ni ARRIBA | B47, B49 |
| El ensayo con una videollamada | «En la reunión, nunca» | A1 |
| El registro de «Ver lo que salió» | lo del ensayo como sugerencia | M21 |

El barrido se hizo por promesa aplazada: «todavía no», «aún no», «por ahora», «mientras tanto», «próximamente»,
«podrás», «no se puede» y sus pares en inglés. Cubrió el manual, el README, la guía, el BLUEPRINT, los LEEME del
kit, el design system, el `CLAUDE.md`, el README y el índice de la maqueta, i18n es/en, `Info.plist` y la ayuda de
`ghost`. Sin hallazgo en `Info.plist`, `InfoPlist.strings` ni el resto de `ghost`. La segunda pasada la hará **otro**
auditor después de la Fase 2, incluido el summary.

## Casilla 5 — campos del contrato sin lector

Se revisaron todos los campos de `LaFranja`, `Preparacion`, `Elegible`, `Cuentas`, `VistaDelEnsayo`,
`PreguntaEnPantalla`, `EstadoDelBanco`, `Evaluacion`, `Evidencia`, `Muletilla`, `Informe`, `Fila`, `Progreso`,
`FilaDelProgreso`, `DesdeElPrimero`, `Cambio` y `NoEmpezo`, más `EstadoDeEscucha.bytesDelEnsayo`,
`EstadoDeLaIa.enriquecer`, la pieza del corte y los eventos `franja` y `ensayo`. Todos tienen lector.

**Seis huérfanos:**

| Huérfano | Hallazgo |
|---|---|
| `Evidencia.fuente.conjeturada`, en el ensayo | B33 |
| el retorno de `empezar_el_ensayo` | B34 |
| el retorno de `exportar_el_ensayo` | B35 |
| el retorno de `borrar_los_ensayos` | B36 |
| el retorno de `fijar_posicion_de_la_banda` | B37 |
| el retorno de `entendido_el_aviso_de_arriba` | B38 |

El gate `contrato-con-lectores` no los ve: compara por nombre y no mira los retornos. Es lo que la regla 19 del
`CLAUDE.md` ya declara que no cubre.

## Casilla 6 — la guía heredada, contra la arquitectura

- Las 101 heredadas se releyeron contra el código.
- Hay dos pruebas que no pueden pasar tal como están: A4 (parada 1 → paradas 4 y 6, a5 y l5) y M25 (q4).
- Hay frases que dejaron de ser ciertas: B40 (título del bloque B), B41 (o4), B42 (r7), B43 (T3), B44 (U3) y M19 (n7
  y T19).
- El resto, comprobado sin hallazgo: b1–b8, c1–c4, d1–d6, e1–e11, f1–f6, g1–g5, h1–h6, i2–i6, j1–j3, k1–k5, l1,
  l3, l3b, l4, l6, l7, m1–m5, n1–n6, o1–o3, o5, o6, p1–p5, q1–q3, q5, r1–r6 y r8–r11.

## Casilla 7 — números de entidades cableados

**Hallazgos:** A2 (locales), M18 (tope de fábrica), B11 (señales de Meet) y B12 (catálogo frente a código).

**Aceptado, con su razón:**
- el diseño cerrado de dos idiomas (`Idioma {Es, En}`): «español e inglés en todo»;
- `FILAS_DEL_PROGRESO = 6`, `FILAS_A_LA_VISTA = 4` y `enriquecer::MAXIMO = 5` (ADR 019): constantes declaradas con
  su razón;
- las tres fuentes del banco, las cuatro cifras y los dos bordes: categorías cerradas por diseño.

## Casilla 8 — las protecciones del Mac, y dónde está el «sí»

| Acción sobre el Mac del usuario | ¿Se corrió? | Matriz | «Sí» |
|---|---|---|---|
| Escribir `AXPosition` en la página de prueba de Meet (fase 1, dos corridas) | Sí | plan, fila 1 (copiarla a la bitácora: B4) | «Sí, haz la corrida en vivo de la banda arriba. Abierto en Google Chrome» y «Volvamos a hacer la prueba» |
| `ls ~/Library/LaunchAgents` y `launchctl list` | Sí | no piden permiso (`CLAUDE.md`, regla 22) | — |
| Micrófono fuera de reunión, Llavero, launchd y Touch ID (filas 2 a 5) | **No**: enseñadas y aplazadas | bitácora, fase 4 | no aplica |

**Sin hallazgo alto.**

**Regla 25:**
- ningún script nuevo llama a `osascript`, `security`, `launchctl`, `tccutil`, `sfltool`, `log show` ni `say`;
- los tests nuevos usan motores falsos;
- la sesión efímera completa lleva `hardware:`;
- el centinela cubre `poner_posicion`, el grifo del micrófono y el desbloqueo de ensayos;
- con `AG_SIN_HARDWARE=1`, 0 abortos;
- huecos menores: B5.

## Lo que está bien

- **La geometría:** `work_area` llega ya volteado; se probaron notch, pantalla externa, barra cero y la regresión
  del H1.
- **`AXPosition`:** `ax::mover` es su única escritura, con barrido. La huella v1 del H1 se devuelve igual que
  entonces.
- **`ensayos.rs`:**
  - escritor único, 700/600, `-2` y `-3` sin pisar;
  - «siempre» no va a launchd;
  - el vencimiento borra en el segundo exacto;
  - `nombre_valido` bloquea las rutas que se salen de la carpeta;
  - exportar no toca la carpeta elegida.
- **`guardado.rs`:** solo texto y cifras, el `Drop` pisa, y el orden del progreso es correcto.
- **El banco y el acento:**
  - catálogos coherentes;
  - `fundar` con esquema cerrado;
  - bóveda, `registrar_salida`, el tope del mes y la vigencia;
  - con el API apagado solo actúan el mock o el modelo local.
- **La concurrencia del ensayo:** sin interbloqueos, y lo que vuelve tarde se descarta por id. El ensayo y la reunión
  se excluyen, y la puerta se cierra antes del micrófono.
- **Eventos y logs sin contenido.** `ensayo/` es protegido y su oído solo abre el micrófono.
- **La interfaz:**
  - suscripciones y temporizadores se limpian;
  - cada estado lleva símbolo, texto y color;
  - el reloj tiene `role="timer"`;
  - no hay movimiento nuevo ni tokens vetados;
  - el copy nuevo no trae promesas aplazadas;
  - la regla 19 se cumple de punta a punta, y los tests prueban el producto, no las muestras.

## Orden propuesto para la Fase 2

Va primero lo que crea o amplía gates; después el resto; y los gates nuevos se corren **al final sobre el árbol
completo**. Cada rojo, con `scripts/demo-rojo.sh`.

| Bloque | Hallazgos |
|---|---|
| 1. Gates | M17 (reduced-motion), B5 (agujas y centinela), B9 (término plantado del ensayo), B20 (el test que no podía fallar), B12 (catálogo frente a código) y el test de `fontSize` de B14 |
| 2. Decisiones del usuario, ya decididas | A1, A4, M3, M10, M20 y B46 |
| 3. Rust del acople | A3 → M1 → M2 → M4 → M5 → B18 → B19 → B25 (la parte de Rust) → B11 |
| 4. Rust del ensayo | A2 → M6 → M7 (Swift) → M8 → M9 → M12 (Rust) → M18 → B15 → B16 → B17 → B21 → B22 → B23 → B27 → B28 → B34–B38 |
| 5. Interfaz | M11 → M12 (TS) → M13 → M14 → M15 (y Notas) → M16 → B14 → B24 → B25 → B26 → B29 → B30 → B31 → B32 → B33 → B39 |
| 6. Textos y documentos | M19 → M21 → M22 → M23 → M24 → M25 → M26 → M27 → M28 → B1 → B8 → B13 → B40–B50 |
| 7. Registro | B2, B4, B6, B7 (con el `/release-check`), B10 y B3 (pide el «sí» del usuario: publica en GitHub) |
| 8. Cierre | todos los gates sobre el árbol completo (Rust con `AG_SIN_HARDWARE=1`, Vitest, e2e, fidelidad, `verify:ephemeral`) → **segunda casilla 4 con OTRO auditor independiente**, sobre el diff completo y el summary |
