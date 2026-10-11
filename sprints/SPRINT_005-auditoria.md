# Auditoría del sprint 005 «En la mesa»: una pasada (cierre mínimo)

- **Fecha:** 2026-10-11.
- **Auditor:** independiente, una sola pasada, solo lectura (directiva del 2026-10-06: cierre mínimo).
- **HEAD auditado:** `0cc590f`, en `sprint-005/en-la-mesa`. La línea sin comitear de la bitácora se ignoró.
- **Diff auditado:** `git diff main...HEAD -- . ':!docs/fidelidad'`: 85 archivos, +5972 / −364.
  - **Leído archivo por archivo:**
    - el Rust del modo: `lib.rs`, `escucha/`, `modo.rs`, `disparo/`, `habla/`, `voz/turno.rs`, `stt/`, `capture/mod.rs`, `reunion.rs`, `notas/`, `propuestas/`, `prefs.rs` y `contrato.rs`;
    - la webview: Sesión, Banda, Honestidad, Ventana, Principal, `cuaderno.ts`, `jurisdiccion.ts` e i18n;
    - los tests nuevos;
    - el manual, los ADR 020 y 021 y las enmiendas;
    - el kit v4.
  - **Leído en diagonal:** el delta del kit (comandos, `demo-rojo.sh`, `verificar-dependencias`).

| Qué corrí | Resultado |
|---|---|
| `git diff main...HEAD --stat` y `git diff` por archivo | 85 archivos |
| `pnpm exec vitest run tests/unit/presencial.test.tsx tests/unit/pista-por-quien.test.ts tests/unit/cero-huellas-de-voz.test.ts tests/unit/auditoria-con-sitio.test.ts tests/unit/contrato-con-lectores.test.ts` | 5 archivos, 31 de 31 en verde |
| Barridos `grep` de la casilla 4 (promesa aplazada) en el manual, el README, los LEEME del kit, los ADR 002, 009, 020 y 021, `src/i18n/*.ts` y las maquetas tocadas | ver casilla 4 |
| Barrido de lectores de lo nuevo del contrato (casilla 5) | ver casilla 5 |
| `cargo test` | **No corrido.** Cada hallazgo de Rust se confirmó leyendo las líneas citadas; ninguno depende de una corrida. |

La CI de `0cc590f` dio `quality`, `e2e` y `build-escritorio` en `success`. El dato viene del encargo; no lo volví a leer.

## Veredicto

**Requiere ajustes.**

- **Un alto, que la directiva obliga a corregir dentro del sprint (A1).** «Terminar sesión» deja encendida la voz que «Empezar en presencial» encendió «para esta sesión». En consecuencia:
  - `⎋` queda cogida en todo el Mac sin sesión;
  - la reunión siguiente arranca hablando;
  - una ficha que esperaba a la sala se dice justo después de terminar.
- **Ningún crítico.**
  - Nada de la sala persiste ni sale en el log: los `println!` nuevos son solo metadatos, y la canaria del cliente recorre la sala en el paso 7-quater sin salir.
  - Nada crudo sale del equipo.
  - No hay biometría de voz.
  - Nada toca otra máquina.
- **Medios y bajos:** van al summary con su sitio, como manda el cierre mínimo.

| Severidad | Cuenta |
|---|---|
| Críticos | 0 |
| Altos | 1 |
| Medios | 8 |
| Bajos | 14 |

**Cortes declarados que no cuento como hallazgos:**
- la corrida en vivo (tercer filo de la regla 15), y con ella el nivel C del kit v4;
- la maniobra §10;
- la fila de la sala en Idioma;
- la regla aplicada con sus valores por defecto.

A1 y M7 son justo la clase de defecto que esa corrida en vivo habría enseñado.

**Lo que hizo el constructor con este informe (2026-10-11), según el cierre mínimo de la directiva del 2026-10-06:**

- **A1, pagado en el sprint**, con su test y su rojo (abajo, en su sección).
- **Los 8 medios y los 14 bajos van al summary como deuda, con su sitio, y no se pagan ahora**: la directiva lo manda
  así, aunque la regla de la casa es pagarlos todos. Su sprint de pago es el **006**; M7 se confirma además en la
  corrida en vivo del modo presencial, que quedó como corte declarado.
- **Los sitios de la deuda en `src-tauri/src/lib.rs` posteriores a la línea 946** (M6, B1, B4 y B14) están al día
  tras el pago de A1, que añadió 7 líneas ahí, y los del manual posteriores a su línea 156 (que ganó una); los demás
  sitios de este informe son los de `0cc590f`.

## Cobertura de alcance

| Ítem (orden · plan) | Estado | Cita |
|---|---|---|
| Fase 0, ejecutada antes de la directiva: constitución regenerada, delta del kit v1.37→v1.40, ADR 020 y 021, enmiendas 002 y 009, maqueta presencial con su mirada de DECISIÓN | Completo; se declara | `CLAUDE.md:9`, `decisions/020-el-modo-presencial.md:1`, `decisions/021-tauri-ios-el-spike.md:1`, `docs/diseno/README.md:221` |
| Maniobra §10 | No implementado (fuera por la directiva) | `sprints/SPRINT_005-implementation-log.md` |
| F1 · `Modo::Presencial`: solo el micrófono, como la sala | Completo | `src-tauri/src/modo.rs:77`, `src-tauri/src/escucha/mod.rs:413` |
| F1 · un test por cosa que no se abre: sistema; pantalla, y con ella ⌃⌥L y el radar ámbar; acople y `AXPosition`; tus turnos; propuestas | Completo | `src-tauri/src/modo.rs:131`, más las dos puertas del acople en `src-tauri/src/lib.rs` (`pruebas_del_modo_presencial`) |
| F1 · relleno y protección «no se crean» | Con desviación: se quedan (ADR 020 §9) | `decisions/020-el-modo-presencial.md:139` |
| F1 · `Pista::Sala` y `quien()` | Completo (ver B6) | `src-tauri/src/capture/mod.rs:177`, `tests/unit/pista-por-quien.test.ts` |
| F1 · tolerancia a tu voz por código | Completo | `src-tauri/src/disparo/tolerancia.rs:98`, `src-tauri/src/disparo/mod.rs:150` |
| F1 · kit v4 con dos voces sintéticas y STOP de medición | Completo; el nivel C sin medir (corte declarado) | `src-tauri/tests/contra-el-mac-de-verdad.rs:3201`, `:3331`, `:3387`, `docs/kit-de-prueba/presencial.json` |
| F1 · tope de turno de la sala | Completo | `src-tauri/src/voz/turno.rs:88` |
| F1 · «lo tuyo queda» apagado | Parcial: Rust, Sesión y Honestidad sí; Notas no (M3) | `src-tauri/src/reunion.rs:455`, `src/pantallas/Notas.tsx:488` |
| F1 · idioma de la sala (enmienda 009) | Con desviación: Rust y Sesión sí; la fila de Idioma no (declarado) | `src-tauri/src/prefs.rs:46`, `src-tauri/src/lib.rs:2731`, `src/pantallas/Sesion.tsx:444` |
| F1 · NDA comprobada en Rust | Completo | `src-tauri/src/modo.rs:102`, `src-tauri/src/lib.rs:767` |
| F1 · `verify:ephemeral` con una sesión presencial | Completo; paso de hardware, corrió en la CI | `src-tauri/tests/contra-el-mac-de-verdad.rs:940` |
| F1 · contrato Rust→TS (regla 19) | Completo (ver B9) | `src-tauri/src/contrato.rs:353` |
| F1 · cero huellas de voz y vocabulario vetado | Completo | `tests/unit/cero-huellas-de-voz.test.ts`, `tests/unit/vocabulario-vetado.test.ts:54` |
| F2 · regla elegida en `reglas.json` y § Medición del ADR 020 | Con desviación: valores por defecto del STOP, porque la respuesta llegó sin rellenar (declarado) | `data/presencial/reglas.json:7`, `decisions/020-el-modo-presencial.md:171` |
| F2 · Sesión presencial, fiel a la maqueta | Completo (ver M4 y M5) | `src/pantallas/Sesion.tsx:417` |
| F2 · banda y píldora | Completo, con defectos (M1, M2) | `src/componentes/Banda.tsx:443`, `:462`, `:1026`, `:1095` |
| F2 · la voz espera a que la sala calle | Completo (ver A1 y M6) | `src-tauri/src/lib.rs:2199`, `src-tauri/src/habla/mod.rs:120` |
| F2 · Honestidad y el rail | Completo | `src/pantallas/Honestidad.tsx:173`, `src/componentes/Ventana.tsx:129` |
| F2 · Idioma | No implementado (desviación declarada) | `src/pantallas/Idioma.tsx:149` |
| F2 · manual: presencial, el PC con Windows al lado, lo que no hace | Completo (ver M8, B11 y B12) | `docs/MANUAL-DE-USO.md:143` |
| F2 · corrida en vivo del modo nuevo (regla 15, tercer filo) | No implementado: corte declarado el 2026-10-10 | `sprints/SPRINT_005-implementation-log.md` |
| Fase 3 (spike iOS) · Fase 4 | No corrida (respuesta 1b) · sustituida por el cierre mínimo | `decisions/021-tauri-ios-el-spike.md:5` |

## CRÍTICOS (0)

Ninguno.

## ALTOS (1)

### A1 · «Terminar sesión» no apaga la voz que «Empezar en presencial» encendió para esa sesión

`src-tauri/src/lib.rs:946`

**Qué pasa:**

1. Sesión → «Presencial» → «Empezar en presencial», con «al oído» marcado. Viene marcado de fábrica salvo con altavoces que la app reconoce: con AirPods o cascos de cable, sí.
2. `empezar_presencial` llama a `fijar_la_voz_en(&app, true)` (`src-tauri/src/lib.rs:718`). `conmutar_el_modo` enciende el modo, baja la banda a 44 px y registra `⎋` como atajo global (`src-tauri/src/lib.rs:2290`).
3. «Terminar sesión» → `dejar_de_escuchar` (`src-tauri/src/lib.rs:946-962`): corta la escucha y borra el modo, pero no toca `LaVozQueSale`. La voz solo la apagan el asa (`:206`), `⌃⌥V` o `fijar_la_voz` (`:2260`) y `⌥⎋` (`:1107`).

**Resultado:**

- **(a) `⎋` sigue cogida en todo el Mac**, ya sin sesión. El propio código lo prohíbe: un Escape global permanente «se lo quitaría a la reunión… y a todas las demás apps del Mac» (`src-tauri/src/lib.rs`, comentario de `el_atajo_de_callar`).
- **(b) La reunión siguiente (videollamada) empieza con la voz encendida** sin que nadie la encendiera para ella.
  - Contradice la promesa escrita «para esta sesión» (`src-tauri/src/lib.rs:708`, `decisions/020-el-modo-presencial.md:124`).
  - Contradice la regla «el modo solo audio no se enciende solo nunca».
  - Con Bluetooth la app no sabe si la salida es un altavoz, y habla igual.
- **(c) Una ficha que esperaba a la sala se dice justo después de «Terminar».**
  - Con la escucha cerrada, `alguien_hablando_en` devuelve `false` (`src-tauri/src/lib.rs:2109-2114`).
  - `esperar_a_la_sala` (`:2199`) da «calló» al primer paso de 100 ms, y `vez` no cambió.
  - `decir_la_ficha_si_cabe(…, false)` la lee.

**Ajuste ejecutable:**

1. En `src-tauri/src/lib.rs`, dentro de `dejar_de_escuchar` (`:946`): justo después del bloque `if let Ok(mut g) = estado.0.lock() { … }` y antes de `marcar_el_modo(&app, None);` (`:953`), insertar:

```rust
    // La voz que «Empezar en presencial» encendió era para esta sesión (ADR 020 §7): se apaga con ella, y con
    // ella la ficha que esperaba a que la sala callara. Antes de borrar el modo: después ya no se sabe que era la sala.
    if modo_de_la_sesion(&app) == Some(modo::Modo::Presencial) {
        fijar_la_voz_en(&app, false);
    }
```

`fijar_la_voz_en(false)` solo actúa si la voz está encendida:
- llama a `conmutar_el_modo`, que pasa la banda a 88 px con `asentar_banda`, sin reacoplar porque el modo sigue siendo presencial;
- después llama a `apagar_el_modo`, que sube `vez`, apaga `esperando`, devuelve `⎋` y avisa a la banda.

2. En el mismo archivo, módulo `pruebas_del_modo_presencial`, un test de fuente nuevo:

```rust
    /// **«Terminar» apaga la voz que encendió la sala** (ADR 020 §7; auditoría del S5, A1), antes de borrar el modo.
    #[test]
    fn terminar_apaga_la_voz_que_encendio_la_sala() {
        let c = cuerpo_de("\nfn dejar_de_escuchar(");
        let apaga = en(c, concat!("fijar_la_voz_en(&app, ", "false)"));
        let borra = en(c, concat!("marcar_el_modo(&app, ", "None)"));
        assert!(apaga < borra, "la voz se apaga después de borrar el modo: ya no se sabe que era la sala");
        assert!(c[..apaga].contains(concat!("== Some(modo::Modo::", "Presencial)")), "«Terminar» apaga la voz también en una reunión");
    }
```

3. En `docs/MANUAL-DE-USO.md:156`, el paso 4 queda así:
   - es: «**«Empezar en presencial».** Para acabar, «Terminar sesión»: la voz se apaga con la sesión. `⌥⎋` corta todo, como siempre.»
   - en: «**“Start in person”.** To finish, “End session”: the voice turns off with the session. `⌥⎋` cuts everything, as always.»

**Criterio de ajuste verificado:**

- `cd src-tauri && AG_SIN_HARDWARE=1 cargo test --locked --lib pruebas_del_modo_presencial` en verde, con el test nuevo.
- Su rojo registrado en la bitácora (regla 15):
  - comando: `scripts/demo-rojo.sh --archivo src-tauri/src/lib.rs --buscar 'fijar_la_voz_en(&app, false);' --reemplazar '' --gate 'cd src-tauri && AG_SIN_HARDWARE=1 cargo test --locked --lib terminar_apaga_la_voz_que_encendio_la_sala' --debe-nombrar 'falta «fijar_la_voz_en(&app, false)»' --esperar-verde '<mismo comando>' --minimo-tests 1`;
  - resultado esperado: código 0.

**Estado:** pagado · `src-tauri/src/lib.rs:946` (`dejar_de_escuchar`) apaga la voz en presencial, y el test de fuente
`terminar_apaga_la_voz_que_encendio_la_sala` (en `pruebas_del_modo_presencial`) lo vigila. **Una desviación del ajuste
propuesto:** la voz se apaga **antes de cortar la escucha**, no después del candado. Si se apagara después, la sala ya
estaría cortada, el hilo que espera vería silencio y podría decir la ficha en el instante entre el corte y el apagado.
El test exige ese orden: apagar antes de `e.cortar()` y antes de `marcar_el_modo(&app, None)`. **Rojo** con
`scripts/demo-rojo.sh`, quitando la llamada: el test falló nombrando «falta «fijar_la_voz_en(&app, false)»» y pasó en
verde al restaurar (1 prueba). El manual (paso 4 de «Cómo se usa») y el ADR 020 §7 dicen ya que la voz se apaga con
la sesión.

## MEDIOS (8)

| Id | Sitio | Problema | Ajuste propuesto | Estado |
|---|---|---|---|---|
| **M1** | `src/componentes/Banda.tsx:443` | En presencial la cabecera dice «Escuchando · la sala», con el check verde, siempre que `escucha.escuchando` sea verdad. Lo dice aunque la única pista no abriera (permiso del micrófono denegado, dispositivo ocupado): `Escucha::estado()` da `escuchando: true` con `microfono.abierta: false`. En reunión, la misma cabecera cuenta las pistas abiertas: «Escuchando · 0 pistas». | En `Banda.tsx:442-446`: `presencial ? (deLaMaqueta \|\| escucha.microfono.abierta ? t.escuchandoLaSala : \`${t.escuchandoPrefijo} · ${cuenta(0, t.pista, t.pistas)}\`) : …`. Usa copy que ya existe en es y en. Test en `tests/unit/presencial.test.tsx`: `ESTADO_DE_LA_ESCUCHA_PRESENCIAL` con `microfono: { abierta: false, motivo: "sin-permiso-del-microfono", bytes: 0 }`; no debe aparecer `tb.escuchandoLaSala`. Su rojo: quitar la condición. | deuda · S6 |
| **M2** | `src/componentes/Banda.tsx:450` | En presencial, `empezar` suelta el acople (`src-tauri/src/lib.rs:776`) y `registrar_acople` avisa a la banda. `acoplada` pasa a `false` y la banda de 88 px pinta el chip ámbar «⚠ sin acople»: un aviso de avería para el estado que presencial busca. La maqueta aprobada no lo tiene (`docs/diseno/banda.html:116-123`, s5-presencial). Ni la fidelidad ni `presencial.test.tsx` lo ven, porque fuera de Tauri `acoplada` vale `true`. Con los altavoces del Mac la banda viene marcada de fábrica, así que se ve en uso normal. | `Banda.tsx:450`: `{!acoplada && !presencial && (`. Test: `<Banda estado="esperando" acoplada={false} />` con la escucha presencial; `queryByText(tb.sinAcople)` debe ser `null`. Su rojo: quitar `!presencial`. | deuda · S6 |
| **M3** | `src/pantallas/Notas.tsx:488` | Notas no conoce la sala. **(1)** El interruptor «Conservar mis turnos» sigue activo en presencial. Al pulsarlo, Rust guarda la preferencia y deja el cuaderno apagado (`reunion::conservar_mis_turnos`); la pantalla se refresca y el interruptor vuelve a «apagado» sin decir por qué. El ADR promete «el interruptor se desactiva en presencial» (`decisions/020-el-modo-presencial.md:108`). **(2)** «Muere al cerrar» dice «Voz del cliente» y «Sus turnos del transcript» (`Notas.tsx:514-515`), que en presencial cuentan los turnos de la sala, los tuyos incluidos. Al cerrar dice «Audio · las dos pistas» (`Notas.tsx:596`), con una sola pista abierta. | Rust: que el cuaderno recuerde si la reunión fue presencial (`reunion::al_empezar(app, tus_turnos, presencial)` → campo nuevo en el estado que lee Notas, por `contrato.rs`, regla 19). Notas, con `presencial`: interruptor con `aria-disabled="true"` y sin `onClick`, con `cuaderno.presencial.noSabeCuales` debajo. Filas con copy nuevo: «Las voces de la sala» / «The room's voices»; «Los turnos de la sala» / «The room's turns»; «Audio · la sala» / «Audio · the room». Test de Notas con la muestra presencial; su rojo, quitando la rama. | deuda · S6 |
| **M4** | `src/pantallas/Sesion.tsx:152` | El radar coral no conoce presencial. **Antes de empezar:** si aparece un programa invasivo, `LaVigilancia` ocupa Sesión y su «Iniciar de todos modos» llama a `empezarAEscuchar()` (`:152-155`). Empieza una REUNIÓN, con audio del sistema y pantalla, aunque elegiste presencial. **Con la sesión presencial en marcha:** `EnPresencial` (`:417`) no pinta el aviso coral que la reunión pone arriba ni la fila MDM. La banda sí enseña el coral. | `iniciar={() => { setVisto(clave); if (modo !== "presencial") empezarAEscuchar(); }}`: en presencial vuelve a la vista de la sala, donde se pulsa «Empezar en presencial». Pasar a `EnPresencial` el aviso (`vigilado ? <LaVigilancia radar={radar} /> : null`) y la fila `mdm`, pintados arriba como en la reunión. Test: en `presencial.test.tsx`, fuera de Tauri con `radarDeMuestra` y `estado=presencial`, «Iniciar de todos modos» no pide `empezar_a_escuchar`. | deuda · S6 |
| **M5** | `src/pantallas/Sesion.tsx:444` | El selector «Idioma de la sala» ofrece `["es-ES", "en-US", sala]`, cableados (casilla 7). No mira lo que sabe el motor de este Mac (`Idioma.tsx` usa `useQueSabeTranscribir`) ni si el modelo está instalado. Un idioma del cliente como `es-CO` desaparece de la lista en cuanto eliges otro, y no hay forma de volver a él, porque la fila de Idioma no existe. Con un modelo sin instalar, la sala abre («Funciona»), no transcribe y nadie dice por qué. | `codigos = [...new Set(["es-ES", "en-US", ...transcribe.idiomas.map((i) => i.codigo), idiomas.cliente, sala])]`, con `transcribe = useQueSabeTranscribir()`. Al lado del selector, el estado del modelo con el mismo `estadoDelModelo` de Idioma (copy que ya existe). Test: con un motor que conoce `es-CO`, la opción sigue en la lista tras elegir `en-US`. | deuda · S6 |
| **M6** | `src-tauri/src/lib.rs:2301` | Al encender la voz a mitad de sesión (`⌃⌥V`, o «al oído» en Sesión), `conmutar_el_modo` dice `ficha_vigente`, que **vuelve a buscar** con el último turno de la sala y sin tolerancia (`src-tauri/src/lib.rs:2323-2343`). En la sala ese turno suele ser tu respuesta: es la premisa de C1. La voz dice entonces otra ficha distinta de la que enseña la banda (el comentario de `ficha_vigente` lo llama «la peor manera posible de romper un modo»), o nada si sale «sin resultado». | En presencial, al encender, decir la ficha que enseña la banda: la que el cuaderno ya guarda para `⌃⌥P` (`reunion::ver`, `src-tauri/src/reunion.rs:594`; el corte ya la vacía). Hace falta un `reunion::vista(app) -> Option<FichaFijada>` y `habla::a_voz(titular, linea, fuente)`. En reunión, como hoy. Test de fuente en `pruebas_del_modo_presencial`: la rama presencial de `conmutar_el_modo` no llama a `ficha_vigente`. Su rojo. | deuda · S6 |
| **M7** | `src-tauri/src/capture/nativo.rs:321` | **No confirmado en hardware.** Presencial abre el dispositivo de entrada **por defecto** del Mac. Con auriculares que traen micrófono (AirPods, el caso de «al oído», que viene marcado), macOS suele pasar la entrada por defecto a ellos. La «sala» sería entonces sobre todo tu voz, y el cliente llegaría lejos o no llegaría. Ni Sesión («Micrófono — tú y el cliente», `src/i18n/es.ts:363`) ni el manual nombran el aparato de entrada ni lo avisan. | Rust: `capture::nativo::entrada_de_audio()` con el nombre del dispositivo de entrada por defecto (la misma lectura de `abrir_el_microfono`), como comando por `contrato.rs`; Sesión lo nombra en la fila de la sala. Manual, en «Lo que no hace»: es «**Si tus auriculares tienen micrófono**, macOS puede escuchar por ellos y la app oiría sobre todo tu voz: elige el micrófono del Mac en System Settings → Sound → Input.»; en «**If your headphones have a microphone**, macOS may listen through them and the app would mostly hear you: pick the Mac's microphone in System Settings → Sound → Input.». Se verifica en la corrida en vivo, con su fila de la regla 22 y su «sí». | deuda · S6, se confirma en la corrida en vivo |
| **M8** | `docs/MANUAL-DE-USO.md:170` | La frase «de 14 preguntas del cliente salieron 5 fichas —las mismas que sin reglas—» falla en dos cosas. **Las 14** son turnos del cliente que debían traer ficha, no todas preguntas: 9 son preguntas, 3 términos y 2 silencios (`docs/kit-de-prueba/presencial.json`). **Las 5 no son las mismas:** la línea base acierta 3 por pregunta y 2 por silencio, y la regla elegida 5 por pregunta y 0 por silencio. Además, «casi todas por tu voz» fueron todas (16 + 0). | Sustituir por: es «En la prueba, de 14 turnos del cliente que pedían ficha salieron 5 —tantas como sin reglas, aunque no las mismas— y las que sobraban, todas por tu voz, bajaron de 16 a 5.»; en «In the test, of 14 client turns that called for a card, 5 got one —as many as with no rules, though not the same ones— and the extra cards, all from your voice, went down from 16 to 5.» | deuda · S6 |

## BAJOS (14)

| Id | Sitio | Problema | Ajuste propuesto | Estado |
|---|---|---|---|---|
| **B1** | `src-tauri/src/lib.rs:2724` | La doc de `fijar_idioma_de_pista` dice «Solo `consultor` o `cliente`», pero acepta también `sala` (`:2738`). | «Solo `consultor`, `cliente` o `sala` (presencial, ADR 020 §6), y solo un código de idioma.» | deuda · S6 |
| **B2** | `src/pantallas/Idioma.tsx:149` | El comentario dice que «la sala tiene su fila aparte». No la tiene: es la desviación de la fase 2, y la sala se elige en Sesión. | «La sala (sprint 005) se elige en Sesión, no aquí (desviación de la fase 2, bitácora): aquí, las dos de una reunión.» | deuda · S6 |
| **B3** | `decisions/009-un-idioma-por-pista.md:107` | La enmienda 1 dice que el idioma de la sala se elige «en Idioma», y que la app lo avisa «en Sesión … e Idioma» (`:112`). La fila de Idioma no se construyó. | Cambiar «elegido en Idioma («El idioma de la sala»)» por «elegido en Sesión, en la vista presencial». Quitar «e Idioma» de `:112`, con una línea fechada que remita a la desviación 1 de la fase 2. | deuda · S6 |
| **B4** | `src-tauri/src/lib.rs:2657` | En presencial `⌃⌥L` no hace nada (la lectura no arranca, por diseño), y el log dice «lectura pedida sin sesión: no hay reunión que leer». Es falso: hay sesión. Solo son metadatos. | Pasar `app` a `leer_una_vez` y, con `modo_de_la_sesion == Some(Presencial)`, escribir «[pantalla] ⌃⌥L en presencial: no se lee la pantalla (ADR 020 §1)». | deuda · S6 |
| **B5** | `src-tauri/src/lib.rs:773` | `empezar` marca el modo (y en presencial suelta el acople) antes de `estado.0.lock()…?` (`:793`). Si ese candado está envenenado, vuelve con error y `ElModo` se queda en `Presencial`: el latido y `⌃⌥B` no acoplan hasta la siguiente sesión. Es improbable: exige un pánico previo. | En el `map_err` de `:793`, `marcar_el_modo(&app, None)` antes de devolver el error. | deuda · S6 |
| **B6** | `tests/unit/pista-por-quien.test.ts:28` | El gate solo mira Rust, y solo literales. Quedan dos huecos. En Rust, `ultimo_de(pista)` en `src-tauri/src/stt/ventana.rs:48` compara por parámetro y ya no tiene llamadores fuera de sus tests. En la webview, `src/ficha.ts:223` y `src/componentes/Banda.tsx:234` comparan `pista === "sistema"`: con la sala, un turno no abre el «Buscando…» de la banda ni limpia la línea del radar coral, y nadie lo decidió. | Que el gate barra también `src/**/*.ts(x)` buscando comparaciones con `"sistema"`, `"microfono"` o `"sala"`, con una lista explícita de excepciones y su porqué. Retirar `ultimo_de`. En `ficha.ts:223`, decidir la sala a propósito: `(n.pista === "sistema" && !n.eco) \|\| n.pista === "sala"`. | deuda · S6 |
| **B7** | `src-tauri/src/disparo/mod.rs:94` | Los auxiliares del inglés solo cuentan como pregunta si abren la frase, tras `and`, `so`, `but`, `or` o `then`. Con una muletilla delante («Okay, is that included», «Well, do you…»), la frase ya no dispara sin `?`, también en reunión; antes disparaba por la ventana de tres palabras. Lo amortigua que el transcriptor suele escribir `?`. | Añadir a `CONJUNCIONES` las muletillas de apertura (`okay`, `ok`, `well`, `now`, `also`, `right`) y un caso «Okay is that included» en `los_auxiliares_del_ingles_preguntan_solo_al_abrir_la_frase`. Comprobar que `disparo.json` sigue en 1,000 de precisión y 1,000 de recall. | deuda · S6 |
| **B8** | `.claude/settings.json:12` | El hook nuevo escribe el porqué del bloqueo («BLOQUEADO: falta gitleaks o jq…», «SECRET DETECTADO…») en stdout y sale con 2. Con exit 2, Claude Code le enseña al agente el stderr, así que el agente ve un bloqueo sin razón. Además, la mitad positiva de su prueba («bloquea la carnada», `tests/unit/hook-secretos.test.ts:62`) se salta en la CI: el runner `ubuntu-latest` de `quality` no tiene gitleaks. Viene de la plantilla del kit. | Enviar los tres mensajes a stderr (`echo '…' >&2`) y que el test lea `r.stderr`. Instalar gitleaks en `quality`, o declarar en el test que esa mitad es local. Anotarlo como fricción del kit (K#). | deuda · S6 |
| **B9** | `src-tauri/src/contrato.rs:353` | **`TURNO_DE_LA_SALA` no tiene lector.** El test del transcript («cada turno dice «sala»», `tests/unit/presencial.test.tsx`) corre fuera de Tauri y pinta la muestra de la maqueta, que la banda reetiqueta a `"sala"` en TS (`src/componentes/Banda.tsx:205`); el turno que emite Rust no llega a un render. **En `IDIOMAS_DE_PISTA`** (`contrato.rs:748`), `sala` vale lo mismo que `cliente` (`en-US`): un `sala` que no llegara caería al idioma del cliente y ningún test lo distinguiría. | Un test de la banda en Tauri, con el puente fingido, donde `turnos_recientes` devuelva `[TURNO_DE_LA_SALA]` y el test pida «sala» en `.transcript-b .quien`. La muestra `IDIOMAS_DE_PISTA` con `sala: Some("es-ES")`, más un test de Sesión presencial que lea `es-ES` en el selector. | deuda · S6 |
| **B10** | `README.md:28` | La sección «Estado» se queda en el sprint 004: el README no nombra el modo presencial (sí el ensayo). | Una frase en «Estado» y en «In English»: es «el sprint 005 sumó el modo presencial: la sala por el micrófono del Mac, sin audio del sistema ni pantalla»; en «sprint 005 added the in-person mode: the room through the Mac's microphone, with no system audio and no screen». | deuda · S6 |
| **B11** | `docs/MANUAL-DE-USO.md:156` | La sección Presencial no dice que «al oído», que viene marcado, se queda con `⎋` en todo el Mac mientras dure. Lo dice la sección general del modo solo audio (`:765`), pero en presencial dura la sesión entera. | Una viñeta en «Lo que no hace»: es «**Con la ficha al oído, `⎋` es de la app** mientras dure la sesión: no le llega a ninguna otra.»; en «**With the card in your ear, `⎋` belongs to the app** while the session lasts: no other app gets it.» | deuda · S6 |
| **B12** | `docs/MANUAL-DE-USO.md:165` | Casilla 7. El manual repite a mano los valores de `data/presencial/reglas.json` (15 s y 15 s, en `:165` y `:169`), la espera de 4 s (`:173` y `:753`) y las cifras del kit (5, 16 y 5). El ADR 020 dice que cambiar la regla «es una línea de `reglas.json`», pero ningún gate ata el manual a ese archivo. | Un test en `tests/unit/` que lea `data/presencial/reglas.json` y exija que la sección Presencial del manual cite `esperaTrasFichaMs/1000` y `topeDeTurnoMs/1000`. O que el manual remita a `reglas.json` sin dar cifras. | deuda · S6 |
| **B13** | `src-tauri/src/lib.rs:718` | `empezar_presencial` ignora lo que devuelve `fijar_la_voz_en`. Sin voz para el idioma del consultor, el modo no se enciende (`:2266-2268`, que solo lo dice en el log), y Sesión pasa a enseñar «la banda» marcada sin decir por qué, aunque elegiste «al oído». | Con la sesión en marcha, si `alOido` se pidió y `voz.encendida` es falso, una línea bajo el selector: es «No hay voz para leer en {idioma}: la ficha va a la banda.»; en «There is no voice to read in {idioma}: the card goes to the band.» | deuda · S6 |
| **B14** | `src-tauri/src/lib.rs:2247` | `fijar_la_voz` devuelve `habla::LaVoz` y nadie lo lee: `src/pantallas/Sesion.tsx:450` hace `void fijarLaVoz(si)`, y el estado llega por el evento `voz`. Es el mismo caso que la auditoría del S4 cazó en B37: retornos sin lector. | Que `fijar_la_voz` no devuelva nada y `fijarLaVoz` pase a `llamar`; o que Sesión use el retorno. | deuda · S6 |

## Casilla 4: frases caducadas

### Coincidencias del barrido por promesa aplazada

| Sitio | Frase | Lo que la app hace hoy | Veredicto |
|---|---|---|---|
| `docs/MANUAL-DE-USO.md:150` | «Todavía no empieza nada: ves cómo será la sesión» | El botón «Presencial» no empieza nada (`src/pantallas/Sesion.tsx:317`) | vigente |
| `docs/MANUAL-DE-USO.md:139` | «La app **no puede saber**» si es un casco o un altavoz | Igual en presencial («Si es un altavoz, elige la banda») | vigente |
| `docs/MANUAL-DE-USO.md:86`, `:504` | «todavía no» (acople con Zoom y Teams; el ensayo sin voz de verdad) | No las tocó el sprint y no dependen de presencial | vigentes |
| `decisions/021-tauri-ios-el-spike.md:6` | «no por ahora» | Cita la respuesta del G-Plan | vigente |
| `docs/kit-de-prueba/audio/LEEME.md:13` | «todavía no está» | Es el texto que dice un audio del kit | vigente |
| `src/i18n/es.ts` y `src/i18n/en.ts`, bloques nuevos | «la app no oye al cliente» / «cannot hear the client»; «no sabe cuáles son tuyos» / «cannot tell which are yours» | Son capacidades de hoy | vigentes |

### Frases que el modo presencial vuelve falsas

| Sitio | Frase | Por qué caduca | Hallazgo |
|---|---|---|---|
| `src/i18n/es.ts:821` / `src/i18n/en.ts:696` | «Audio · las dos pistas» / «Audio · both tracks» | Notas, al cerrar, tras una sesión de una pista | ver M3 |
| `src/i18n/es.ts:721`, `:723` / `src/i18n/en.ts:596`, `:598` | «Voz del cliente», «Sus turnos del transcript» | En presencial cuentan la sala entera | ver M3 |
| `decisions/020-el-modo-presencial.md:108` | «El interruptor se desactiva en presencial» | No se desactiva | ver M3 |
| `src-tauri/src/lib.rs:708` y `decisions/020-el-modo-presencial.md:124` | «para esta sesión» | La voz sobrevive a la sesión | ver A1 |
| `src/componentes/Banda.tsx:443` | «Escuchando · la sala» | Se dice aunque la sala no abriera | ver M1 |
| `docs/MANUAL-DE-USO.md:170` | «14 preguntas… las mismas» | Las cifras no dicen eso | ver M8 |
| `decisions/009-un-idioma-por-pista.md:107`, `:112` | «elegido en Idioma», «e Idioma» | No hay fila en Idioma | ver B3 |
| `src/pantallas/Idioma.tsx:149` | «tiene su fila aparte» | No la tiene | ver B2 |
| `src-tauri/src/lib.rs:2717` | «Solo `consultor` o `cliente`» | Acepta `sala` | ver B1 |
| `README.md:28` | «el sprint 004 sumó…» | Sin presencial | ver B10 |

### Vigentes a pesar de presencial

- `README.md:5-7` («escucha las dos partes… lee la pantalla compartida») describe la videollamada, y sigue siendo verdad para la reunión.
- `src/i18n/es.ts:469`, el subtítulo de Idioma («Dos pistas, un idioma elegido para cada una»): Idioma configura la reunión, y la sala se elige en Sesión (desviación declarada).
- `docs/MANUAL-DE-USO.md:108` («Escucha las dos pistas») es la sección de la reunión.

**Revisión de mi propio copy:** el copy que propongo en A1, M3, M7, M8, B10, B11 y B13 pasó por este barrido; no contiene promesas aplazadas.

## Casilla 5: campos del contrato sin lector

| Campo o muestra nueva | Lector fuera de su construcción y de sus tests | Veredicto |
|---|---|---|
| `EstadoDeEscucha.presencial` | `src/pantallas/Sesion.tsx:193`, `src/componentes/Banda.tsx:192`, `src/pantallas/Honestidad.tsx:68`, `src/componentes/Principal.tsx:126` | tiene lector |
| `IdiomasDePista.sala` | `src/pantallas/Sesion.tsx:443` | tiene lector; la muestra no distingue sala y cliente (ver B9) |
| `LaVoz.esperando` | `src/componentes/Banda.tsx:947`, `:1026`, `:1056` | tiene lector |
| `Pista` «sala» | `src/componentes/Banda.tsx:1095` | tiene lector; la muestra `TURNO_DE_LA_SALA` no cruza a un render (ver B9) |
| `ficha::Aparicion.de_ms` (`#[serde(skip)]`, no cruza a la webview) | `src-tauri/src/lib.rs` (`sintetizar`) y el kit v4 | tiene lector |
| Retorno de `fijar_la_voz` (`LaVoz`) | ninguno | ver B14 |
| Retorno de `empezar_presencial` (`EstadoDeEscucha`) | `llamar` lo descarta, igual que `empezar_a_escuchar` y `empezar_solo_notas` (`src/cuaderno.ts:776`, `src/jurisdiccion.ts:178`) | patrón anterior al sprint; no lo cuento |

## Casilla 7: números cableados

| Sitio | Número | Lo que dice el dato | Hallazgo |
|---|---|---|---|
| `src/pantallas/Sesion.tsx:444` | dos idiomas de la sala (`es-ES`, `en-US`) más el actual | El motor dice N (`useQueSabeTranscribir`) | ver M5 |
| `docs/MANUAL-DE-USO.md:165`, `:169`, `:173`, `:753` | 15 s, 15 s, 4 s y las cifras 5, 16 y 5 del kit | `data/presencial/reglas.json`, `habla::ESPERA_DE_LA_SALA` y la salida del kit; sin gate que los ate | ver B12 |
| `src/i18n/es.ts` y `en.ts`, `enPresencial.transcript` | «12 turnos» | `stt::ventana::TURNOS` = 12 (mismo patrón que `ventana12`) | coincide |
| Honestidad en presencial | «últimos 30 s» | `capture::anillo::SEGUNDOS` = 30 | coincide |
| `src-tauri/src/habla/mod.rs:110` | `ESPERA_DE_LA_SALA` = 4 s | Decisión de producto con su comentario, fijada por `en_la_sala_la_voz_espera_al_silencio_y_no_mas_que_el_tope` | correcto |
| `src/componentes/Banda.tsx:212` | `pistasAbiertas` sobre dos campos | En presencial no se pinta: la rama de la sala lo sustituye | correcto |

## Casilla 8: las protecciones del Mac, y dónde está el «sí»

| Acción sobre lo que macOS protege | Dónde | ¿Corrió en este sprint? | Matriz y «sí» |
|---|---|---|---|
| Micrófono (TCC, ya concedido) en el modo nuevo, más el reconocimiento de voz (posible primer aviso «Speech Recognition») | `src-tauri/src/lib.rs:711` → `escucha::Escucha::arrancar` → `src-tauri/src/capture/nativo.rs:315` | **No.** La fila 1 de la regla 22 se enseñó en el plan y otra vez al STOP de la fase 2 (`sprints/SPRINT_005-implementation-log.md`, «STOP de fin de fase 2»). El usuario respondió «no puedo hacer esa prueba ahora», y no se corrió nada. | Hubo matriz; el «sí» no llegó y no se corrió. Correcto. Es corte declarado. |
| Accesibilidad: `acople::soltar` al empezar presencial, que devuelve la ventana acoplada | `src-tauri/src/lib.rs:776` | Solo dentro del producto, nunca en una corrida del constructor. Permiso ya concedido desde el H1; es la misma llamada que hace `⌥⎋`. | No es una acción nueva del constructor |
| `⎋` como atajo global (no es TCC) y la voz del sistema (`AVSpeechSynthesizer`, sin permiso) | `src-tauri/src/lib.rs:2290`, `:2118` | Solo dentro del producto | n/a (el efecto de `⎋` está en A1) |
| Tests que tocan hardware: el paso 7-quater de la sesión completa (motor de voz) y el nivel C del kit | `src-tauri/tests/contra-el-mac-de-verdad.rs:1067`, `:1240`, `:3386` | Solo en la CI (`--include-ignored --skip en_vivo_`). Los tres llevan `#[ignore = "hardware: …"]`. | `cargo test` a secas no los corre: correcto |
| Tests nuevos sin hardware: `escucha` (`sobre_anillos`, sin grifos), `modo`, `disparo`, `habla::esperar_al_silencio` (reloj inyectado), kit v4 niveles A y B (texto y wav) | `src-tauri/src/escucha/mod.rs`, `src-tauri/tests/contra-el-mac-de-verdad.rs:3201`, `:3331` | La bitácora registra `AG_SIN_HARDWARE=1 cargo test --locked` en verde (librería 606, contra el Mac 18). Con el centinela armado, cualquier cruce a C habría abortado. | Correcto |
| Generar el kit con `scripts/kit-v4-sala.sh` (`say -o`, `afconvert`, `sw_vers`) | `scripts/kit-v4-sala.sh` | Corrido en local. Escribe archivos; no suena ni pide permisos. | Declarado en el plan como «lo que no es protección»: correcto |
| Comandos del constructor: `pnpm update` y `pnpm audit` (registro de npm), Playwright con Chromium, `pnpm fidelidad`, `AG_SIN_HARDWARE=1 cargo test` | n/a | Sí | Ninguno toca TCC, Llavero, launchd ni Accesibilidad. `pnpm tauri dev` y `afplay` solo aparecieron en la matriz; no se corrieron. |
| Xcode | n/a | No aplica (respuesta 1b) | correcto |

## Lo que está bien

- **`Pista::quien()` con `match` sin comodín, y un gate de fuente que prohíbe comparar pistas en Rust.** Una tercera pista ya no cae en «tuyo» ni en «del cliente» por un `else`. Las propuestas (`De::del_turno → Option`), el cuaderno, el eco, el silencio y ⌃⌥A deciden la sala a propósito.
- **Abrir la pantalla en presencial es imposible por tipo.** `PermisoDePantalla` solo lo fabrica `modo::que_abre`, así que no depende de un `if`. La NDA, por fin, se comprueba en Rust.
- **Regla dura 1 intacta.**
  - Los `println!` nuevos son solo metadatos.
  - La canaria del cliente recorre la sala (paso 7-quater) y no aparece en el log.
  - `Permitido` no cambia.
  - `FichaVista` guarda palabras de TU corpus y se olvida con el corte.
- **El kit v4 está bien hecho.**
  - El criterio se escribió antes de medir, con línea base.
  - El nivel A usa el mismo camino que la escucha (`del_turno_a_la_ficha`, `del_silencio_a_la_ficha`).
  - Cada ficha perdida trae su causa.
  - El gate «la regla de la casa deja menos falsas» nació con su rojo (K3).
  - El kit cazó un defecto del disparador que venía del S1 (los auxiliares del inglés), y se arregló con su test.
- **`cero-huellas-de-voz` está bien hecho.** Barre código y lockfiles por nombre de paquete, con límite de palabra: evita el falso positivo de `frameCapacity`.
- **Unos cuarenta rojos registrados con `demo-rojo.sh --debe-nombrar`.** Dos demos que no eran rojos de verdad los rechazó la propia herramienta.
- **El arnés de capturas aprendió a medir `.ventana .contenido`.** Ya no sale con código 2 en toda página de ventana.
- **Las desviaciones están declaradas con su porqué**: la banda protegida, el radar de procesos, la fila de Idioma y los valores por defecto de la regla.
