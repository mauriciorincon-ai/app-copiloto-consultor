# Auditoría del sprint 002 — Fase 1 y Fase 2

> **Fase 2 cerrada (2026-09-26):** el usuario aprobó pagar los 40 y el constructor los pagó todos, cada
> uno con su test en rojo antes del verde (bitácora, sección «`/audita-sprint` — Fase 2»). En la Fase 2
> aparecieron tres bajos más (B22–B24), también pagados. **La casilla 4 corrió dos veces:** la segunda
> pasada, después del último ajuste, encontró 25 bajos más (B25–B49), pagados en `eefc7c3`: **68
> hallazgos, 68 pagados**. El estado de cada uno lleva el commit que lo pagó.

Esta auditoría la hizo un auditor independiente que no construyó el sprint, el **2026-09-26**, sobre el **HEAD `b55f1fe`** de `sprint-002/cuando-que-y-quien-mira` (PR #6). Fue solo lectura: no se editó, creó ni comiteó nada en el repo.

**Método.** La evidencia principal es `git diff main...HEAD`, revisado archivo por archivo en los módulos nuevos y en las superficies del S1. La bitácora solo se usó para saber qué creyó hacer el constructor.

Además se hicieron cuatro comprobaciones:
- Se corrieron las suites sobre ese árbol: `cargo test --lib` dio **327 ok y 1 ignorado**; `pnpm test` dio **27 archivos y 216 tests en verde**; `pnpm typecheck` pasó limpio.
- La bóveda (`sintesis/anonimo.rs`) y el diccionario (`diccionario/mod.rs`) se copiaron a un crate del scratchpad y se ejecutaron con frases reales. Las salidas citadas en A1–A3 son **medidas, no razonadas**.
- `otool -l` sobre el binario de `target/debug` para M1.
- Dos barridos delegados (casillas 4 y 5), verificados a mano en sus puntos clave.

**Mientras se auditaba, la rama avanzó** a `ea45751` (guía v4) y `7ce1d9f` (e2e de reduced-motion). Además hay un `SPRINT_002-summary.md` sin versionar. Nada de eso se audita aquí salvo donde se dice. La fase 6 está en curso y sus entregables (guía, summary, manual final, design-sync, release-check) **no cuentan como hallazgo**.

## Veredicto

**«requiere ajustes».**

La ingeniería del sprint es seria y trae sus rojos:
- la regla que no compila (`Sugerencia` solo nace de `fundar`);
- el techo de 6 s;
- el vigía de pantalla por zonas;
- el radar cotejado por ejecutable entero y con su gate de la regla 9;
- M10 validado al abrir;
- el silencio cableado;
- `fiel.rs`, que es real y está en `fundar`.

Pero hay **cuatro altos**, y tres de ellos no los ve ningún test porque los tests no ejercen la configuración de producción:

- **A1 y A2.** La bóveda que protege el API deja salir nombres y teléfonos del cliente, y su capa de «nombres conocidos» recibe nombres de archivo.
- **A3.** El diccionario técnico corrompe el transcript del cliente en todas las sesiones: «Vale» sale «valle», «fábrica» sale «Microsoft Fabric», «Páramo» sale «paramo». Además el manual afirma lo contrario.
- **A4.** La pista del cliente está fijada en inglés, sin forma de cambiarla. Viene del S1.

Nada de esto es crítico hoy: el API nace apagado y el producto funciona en su camino principal de pantalla y radar. Aun así, **A1 y A2 deben pagarse antes de que nadie encienda el API**.

## Cobertura de alcance

| Ítem | Estado | Evidencia |
|---|---|---|
| Delta del kit: regla 19, cuarto filo de la regla 15, sección fija del ⭐, audita-sprint y release-check, testing-patterns 10, CHANGELOG | Completo | `CLAUDE.md` (reglas 15 y 19) · `.claude/commands/audita-sprint.md` · `.claude/skills/testing-patterns.md` · `CHANGELOG.md` |
| Quitar `--pass-with-no-tests` · clippy dentro de `build-escritorio` | Completo | `package.json:14` · `.github/workflows/ci.yml:70-72` |
| Gate del artefacto de auditoría (petición del usuario) | Completo | `tests/unit/auditoria-con-sitio.test.ts` |
| M1 CSP | Completo | `src-tauri/tauri.conf.json:56-57` · `tests/unit/csp-que-no-deja-salir.test.ts` |
| M2 Sesión con la pista caída | Completo | `src/pantallas/Sesion.tsx:49` · `src/pantallas/Sesion.tsx:270-290` |
| M4 la banda vuelve tras ⌥⎋ | Completo (vuelve al iniciar sesión) | `src-tauri/src/lib.rs:546-556` |
| M9 efímero en runtime (`~/Library`, archivos que crecen) | Con desviación: tamaño y fecha en vez de hash. **No ejerce nada del S2** (M8) | `src-tauri/tests/contra-el-mac-de-verdad.rs:505-580` |
| M10 validación f32 | Completo | `src-tauri/src/capture/nativo.rs:337-346` · `src-tauri/src/capture/nativo.rs:424-435` |
| M3, M5–M8, M12–M14 y B1–B7 del S1 (irrecuperables) | Re-barridos en esta auditoría. Lo que era cierto sale aquí con sitio: A4, M12, B8, B10, B11 | `sprints/SPRINT_001-auditoria.md:88-111` |
| `por_silencio` cableado | Completo | `src-tauri/src/escucha/mod.rs:505-548` · `src-tauri/src/escucha/mod.rs:761-783` |
| 17 campos sin lector | Parcial: el gate está verde pero ciego a los tipos nuevos, y quedan 10 huérfanos (M6, B9–B12) | `tests/unit/contrato-con-lectores.test.ts:31` |
| `corte::Informe` y `corpus::Documento` al contrato | `Informe` completo. `Documento` salió de la costura porque se retiró su comando (con desviación) | `src-tauri/src/contrato.rs:293` · `src-tauri/src/corte.rs:117` |
| `--release` una vez | Completo | `sprints/SPRINT_002-implementation-log.md:471-495` |
| B3 diccionario determinista y persistido a 600 (+ ADR 002) | Implementado **con defecto** (A3) | `src-tauri/src/diccionario/mod.rs` · `src-tauri/src/lib.rs:53-156` · `decisions/002-persistencia.md` |
| Kit: audio de mezcla + WER con y sin diccionario | Completo, pero mide la semilla y no la configuración real (A3); no mide en la CI (declarado) | `src-tauri/tests/contra-el-mac-de-verdad.rs:1160-1240` |
| «Un idioma por pista» declarado | Completo; el idioma **no se puede elegir** (A4) | `decisions/009-un-idioma-por-pista.md` |
| C15 voz (Habla.swift, `habla/`, ⌃⌥V en vez de ⌘⇧A) | Con desviación declarada | `src-tauri/nativo/Habla.swift` · `src-tauri/src/lib.rs:1253-1352` |
| C15 auriculares como candado | Con desviación: habla con cualquier dispositivo externo (M4) | `src-tauri/src/habla/mod.rs:183-205` |
| C15 banda a 44 px | Completo; salir arrastrando el asa no funciona (M14) | `src/componentes/Banda.tsx:324-340` |
| C8 Pantalla.swift + `pantalla/` protegido · pHash · OCR solo ante cambio · refuerzo BM25 | Completo | `src-tauri/nativo/Pantalla.swift` · `src-tauri/src/pantalla/mod.rs:49-257` · `scripts/verify-ephemeral.mjs:63-68` |
| C8 bajo demanda por región | Con desviación (decisión del usuario): ⌃⌥L lee la ventana entera | `src-tauri/src/lib.rs:1608-1636` |
| C8 kit de imágenes · nDCG · OCR/s | Completo; CPU no anotada (B21) | `src-tauri/tests/contra-el-mac-de-verdad.rs:1403-1622` |
| C8 frames en el inventario de lo que muere | Parcial: pieza `UltimoFrame` en el corte, pero el gate del disco no ejerce el OCR (M8) | `src-tauri/src/corte.rs:47` |
| C14 ámbar por OCR · catálogo con fuente | Completo; copy del bot inexacto (B7) | `src-tauri/src/radar/avisos.rs` · `data/radar/avisos.json` |
| C14 MDM local | Parcial: se detecta pero no se ve sin un invasivo (B15) | `src-tauri/src/radar/mdm.rs` · `src/pantallas/Sesion.tsx:62-67` |
| C14 coral por procesos · regla 9 con gate · kit 0 FP | Completo (JSON en vez de YAML, desviación declarada) | `src-tauri/src/radar/procesos.rs` · `tests/unit/radar-solo-este-mac.test.ts` |
| C7 ADR 010 antes del código | Completo | `decisions/010-sintesis-codigo-primero.md` |
| C7 proveedores: sistema · mock · API | Sistema y mock completos. API **con defectos** (A1, A2, M10) | `src-tauri/src/sintesis/` · `src-tauri/nativo/Red.swift` · `src-tauri/nativo/Llavero.swift` |
| C7 proveedor MLX | No implementado: desviación declarada, el ADR 011 lo condiciona | `decisions/010-sintesis-codigo-primero.md:90-92` |
| C7 esquema + regla 19 · grounding · `fiel.rs` · techo | Completo en Rust. En TS no hay Zod: el tipo es generado (desviación aceptable; `zod` sigue en `package.json:26` sin un solo import) | `src-tauri/src/sintesis/mod.rs:196-239` · `src-tauri/src/sintesis/fiel.rs` |
| C7 anonimización local | Parcial (A1, A2) | `src-tauri/src/sintesis/anonimo.rs` |
| C7 bytes y costo en Honestidad e IA · techo US$10 | Parcial (M10, M11, B3) | `src/pantallas/Ia.tsx:145-248` · `src/pantallas/Honestidad.tsx:92-101` |
| ADRs de la DoD: pantalla · radar · voz | No implementado (M9) | `decisions/` |
| DoD: capabilities mínimas | No implementado (M5) | `src-tauri/build.rs:2` |
| DoD: presupuestos (voz ≤1 s, CPU de la pantalla) | No medido (B21) | `sprints/SPRINT_002-implementation-log.md:1171-1185` |
| Deuda de diseño «maniobra genérica · sprint 2» | Ni construida ni re-declarada (M15) | `design-system.md:524` |
| Fase 6: guía v4, e2e reduced-motion, summary, manual, design-sync, release-check | **En curso** (commits posteriores a `b55f1fe`). No cuenta | `docs/GUIA-DE-PRUEBA.html` |

## CRÍTICOS (0)

Ninguno.

## ALTOS (4)

### A1 · La bóveda deja salir al API nombres, teléfonos y la etiqueta F1

**Dónde:** `src-tauri/src/sintesis/anonimo.rs:64-105` · `src-tauri/src/sintesis/anonimo.rs:146-161` · `src-tauri/src/sintesis/mod.rs:83-89`

`tapar` parte el texto solo por `' '`, y `separar_puntuacion` quita puntuación solo al final. El texto que sale es `"Client: {turno}\nF1 · …"`, así que la última palabra del turno queda pegada a `\nF1`.

Medido con una copia del módulo sobre el texto real de `Peticion::texto`. Los siete casos siguientes salen **tal cual**, salvo el correo, que se tapa pero se come la etiqueta `F1` y deja al modelo sin poder citar la primera ficha:
- «¿Andrea Villalba ya lo aprobó?»
- «Lo aprobó Andrea Villalba»
- «Mi celular es 300 555 1234»
- «llámame al 3005551234»
- «(Juan Pérez) dijo que sí»
- ««Ana Torres» lo firmó»
- «escríbele a andrea@paramo.co»

La regla dura 2 pide texto anonimizado localmente. El comentario de `anonimo.rs:156-157` promete cubrir los números «separados por espacios dentro de la palabra», y eso es imposible después del `split(' ')`.

El test `src-tauri/src/sintesis/api.rs:309-324` pasa con el defecto puesto: su frase no empieza por «¿» y acaba en un correo.

**Ajuste ejecutable**
1. En `anonimo.rs`, cambiar `separar_puntuacion` por `partir(p) -> (cabeza, nucleo, cola)`. La cabeza quita `¿¡(«"“'[` y la cola quita `,.;:!?)»"”'`.
2. En `tapar`, tokenizar con `t.split_inclusive(char::is_whitespace)`. Separar el espacio final de cada token y reconstruir con `concat()`, no con `join(" ")`. Una racha de nombres propios solo continúa si el separador es un espacio (no `\n`) y el token no trae cola.
3. Antes de mirar token a token, juntar las rachas de tokens cuyo núcleo sea solo `[0-9+().-]`. Si suman 7 dígitos o más, taparlas como un único `NUMERO`.
4. Añadir tests en `anonimo.rs` que armen el texto con `Peticion::nueva(turno, &pruebas::respaldo()).texto()` para los siete turnos de arriba. Cada uno afirma tres cosas: ningún dato plantado aparece; el resultado contiene `"F1 · "`, `"F2 · "` y `"F3 · "`; y `b.destapar(&fuera) == texto`.

**Verificado cuando:** los siete tests salen rojos con el código actual y verdes después, y `cargo test --lib sintesis` sigue verde.

**Estado:** **pagado** · `c474f96`

### A2 · La capa de «nombres conocidos» recibe nombres de archivo: los clientes de una palabra salen enteros al API

**Dónde:** `src-tauri/src/lib.rs:1746-1762` · `src-tauri/src/corpus/mod.rs:142` · `decisions/011-proveedores-del-modelo-y-minimizacion.md:33-35`

`clientes_del_corpus` pasa `d.nombre`, que es el *file stem*: «Ficha de cliente · Páramo Azul». Ese nombre de archivo nunca aparece en una frase del cliente, así que la capa 1 de la bóveda no tapa nada en producción.

Medido con el turno «Bancolombia pide lo mismo que Páramo Azul, dice Nutresa»:
- «Bancolombia» y «Nutresa» salen tal cual.
- «Páramo Azul» solo se tapa por la heurística de parejas de mayúsculas, como `[PERSONA_1]`.

El ADR 011 promete además los nombres «del diccionario», y no se usan. El test del API (`src-tauri/src/sintesis/api.rs:316`) pasa una lista escrita a mano, así que el cableado real no tiene test.

**Ajuste ejecutable**
1. Crear en `sintesis/anonimo.rs` una función pura `pub fn nombres_de_clientes(stems: &[String]) -> Vec<String>`. Por cada stem devuelve el stem entero y el trozo tras el último `·`, recortado, si existe.
2. Usarla en `clientes_del_corpus` (`lib.rs:1756`).
3. Cambiar `api.rs:309-324` para que obtenga los conocidos con esa función sobre los nombres de `docs/kit-de-prueba/corpus/`, y añadir un documento «Ficha de cliente · Bancolombia» con el caso «Bancolombia pide…».
4. En `decisions/011-…md:33-35`, quitar «y del diccionario», o implementarlo.

**Verificado cuando:** el test nuevo es rojo con el cableado actual y verde después, y «Bancolombia» sale como `[CLIENTE_1]`.

**Estado:** **pagado** · `c474f96`

### A3 · El diccionario técnico corrompe el transcript del cliente en cada sesión

**Dónde:** `src-tauri/src/lib.rs:573-576` · `src-tauri/src/lib.rs:719` · `src-tauri/src/disparo/mod.rs:236-252` · `src-tauri/src/diccionario/mod.rs:78` · `src-tauri/src/diccionario/mod.rs:116-133` · `src-tauri/src/diccionario/mod.rs:163-195` · `docs/MANUAL-DE-USO.md:345-346` · `docs/MANUAL-DE-USO.md:363-365`

La sesión le pasa al diccionario, como «nombres propios del corpus», el **vocabulario del disparador**. Ese vocabulario son palabras sueltas en minúscula y sin tildes, sacadas de nombres de documento y títulos de sección («paramo», «valle», «cierre», «manejo», «etapas», «datos»). `buscar` devuelve ese canónico en minúscula y sin tildes, y además con tolerancia 1–2. La semilla trae además la variante «fabrica».

Medido con los seis documentos reales del kit:

| Frase del cliente | Transcript tras el diccionario |
|---|---|
| «Vale, la fábrica ya está lista» | «valle, la Microsoft Fabric ya está lista» |
| «¿Quién manejó la adopción?» | «¿Quién manejo la adopcion?» |
| «Páramo Azul cierra la etapa» | «paramo Azul cierre la etapas» |
| «Queremos fabricar más» | «Queremos Microsoft Fabric más» |

Este texto es el que ven la banda, el disparador y el buscador.

El manual afirma que «no toca lo que ya estaba bien» y que con una ficha de «Páramo Azul» la app «ya sabe escribirlo bien». Hace justo lo contrario.

El gate del WER (`src-tauri/tests/contra-el-mac-de-verdad.rs:1173`) usa solo `Diccionario::semilla()`: nunca corre en la configuración que usa el usuario.

**Ajuste ejecutable**
1. En `lib.rs:575` y `lib.rs:719`, no pasar `Buscador::vocabulario`. Pasar los nombres reales de cliente con la función de A2, con sus mayúsculas y tildes.
2. En `diccionario/mod.rs:78`, borrar las variantes `"fabrica"` y `"fabrics"`.
3. En `buscar` (`:185-195`), no aplicar la corrección por parecido a los términos de `del_corpus` de menos de 8 letras.
4. Añadir en `diccionario/mod.rs` el test `con_el_corpus_del_kit_no_toca_el_castellano_corriente`. Construye `semilla()` más `con_nombres_del_corpus(nombres_de_clientes(stems del kit))` y afirma que `corregir(x) == x` para las cuatro frases de la tabla.
5. En `contra-el-mac-de-verdad.rs:1173`, construir la jerga igual que la sesión.
6. Registrar la demo en rojo del test con el cableado actual.

**Verificado cuando:** el test nuevo pasa de rojo a verde, y el WER sigue sin empeorar en el kit.

**Estado:** **pagado** · `c474f96`

### A4 · La pista del cliente está fijada en inglés y no hay forma de cambiarla (origen S1)

**Dónde:** `src/cuaderno.ts:549-558` · `src/pantallas/Sesion.tsx:87` · `src/pantallas/Sesion.tsx:166` · `src/pantallas/Idioma.tsx:220-221` · `decisions/009-un-idioma-por-pista.md:74-78`

`DEL_CLIENTE = "en-US"` es una constante. Idioma la enseña pero no deja elegir. El propio ADR 009 midió que la frase en el otro idioma «se destroza». Además:
- la bitácora registra que este Mac **no tiene el modelo de en-US** (`sprints/SPRINT_002-implementation-log.md:774`);
- la guía v4 (`ea45751`) le da voz castellana (`say -v Paulina`) a esa pista.

Con un cliente hispanohablante —el caso del corpus del kit— no llegan turnos útiles. Sin turnos no hay ficha automática, ni sugerencia, ni ⌃⌥A. El comentario «Elegirlos es de la fase siguiente» (S1) caducó sin pagarse.

**Ajuste ejecutable**
1. En `src/cuaderno.ts`, cambiar las dos constantes por un almacén en memoria (contexto React) `useIdiomasDePista()`, que devuelva `{consultor, cliente, fijar(pista, codigo)}`. Por defecto `consultor = "es-ES"` y `cliente = "es-ES"`.
2. En `Idioma.tsx:220-221`, un `<select>` accesible por pista con los códigos de `transcribe.idiomas`, con etiqueta es/en en `src/i18n/es.ts` y `src/i18n/en.ts`.
3. `Sesion.tsx:87` y `Sesion.tsx:166` leen del almacén.
4. Añadir la frase al manual.
5. Test en `tests/unit/cuaderno.test.tsx`: elegir «en-US» para el cliente hace que `empezar_a_escuchar` se invoque con `idiomaDelCliente: "en-US"` (puente simulado). El test es rojo antes del cambio.

**Verificado cuando:** ese test es verde, y en `pnpm tauri dev` el log dice `[escucha] pista «sistema» abierta` con el idioma elegido.

**Estado:** **pagado** · `7a50222` — por defecto es-ES (decisión del usuario)

## MEDIOS (15)

### M1 · El binario no arranca por debajo de macOS 26 y declara 14.2

**Dónde:** `src-tauri/build.rs:52-73` · `src-tauri/tauri.conf.json:65` · `decisions/006-stt-local-y-su-modelo.md:85`

`swiftc` compila sin `-target`, es decir, para el sistema anfitrión. Desde este sprint enlaza `FoundationModels`, que solo existe en macOS 26. `otool -l target/debug/app-copiloto-consultor` lo enseña como `LC_LOAD_DYLIB` fuerte (no *weak*).

En macOS 14 o 15, dyld aborta al arrancar, antes de que la app pueda decir «sin modelo». ADR 006:85 afirma «exige macOS 26 para transcribir y 14.2 para lo demás», y ya no es verdad.

**Ajuste ejecutable**
1. Poner `"minimumSystemVersion": "26.0"` en `tauri.conf.json:65`.
2. Enmendar ADR 006:85 y los requisitos del manual.
3. Añadir `tests/unit/version-minima.test.ts`, que lee `src-tauri/build.rs` y `tauri.conf.json`: si `build.rs` contiene `framework=FoundationModels`, el mayor de `minimumSystemVersion` tiene que ser 26 o más. Es rojo hoy.

(Alternativa para H2: `-target arm64-apple-macos14.2`, `cargo:rustc-link-arg=-Wl,-weak_framework,FoundationModels` y guardas `@available`.)

**Verificado cuando:** el test es verde y el ADR dice lo mismo que la configuración.

**Estado:** **pagado** · `7a50222`

### M2 · El kill-switch no alcanza a la síntesis en vuelo

**Dónde:** `src-tauri/src/lib.rs:750-758` · `src-tauri/src/corte.rs:35-77` · `src-tauri/src/sintesis/api.rs:244-276` · `src-tauri/src/sintesis/mod.rs:295-312`

Tras ⌥⎋ solo sube la época, que tira la respuesta al volver. Dos consecuencias:
- El turno del cliente sigue vivo en el hilo del proveedor hasta TECHO+1 = 7 s.
- Con el API encendido, una petición armada antes del corte **sale igual**: nada comprueba la época antes de `ag_red_post`.

Honestidad dice «8 de 8 piezas» y hay una novena que no se corta. Es la misma clase de defecto que el A2 del S1.

**Ajuste ejecutable**
1. Añadir `Pieza::Sugerencia` a `corte.rs` (enum, `TODAS` justo después de `Voz`, `orden`, y `Cortada` en `suerte_en_este_sprint`). El compilador obligará a tratarla en `ejecutar_el_corte`.
2. Mover el `fetch_add` de la época a ese brazo.
3. Hacer `LaSintesis.epoca` un `Arc<AtomicU64>`. Añadir a `sintesis::api::Api` los campos `epoca: Arc<AtomicU64>` y `desde: u64`, y una función pura `fn sigue_vigente(&self) -> bool`.
4. En `api.rs`, justo antes de `registrar_salida` (`:254`), `if !self.sigue_vigente() { return Err("cortado".into()) }`.
5. Test en `api.rs`: con la época adelantada, `sigue_vigente()` es falso.

**Verificado cuando:** el test es verde, rojo al quitar el `if`, y Honestidad enseña «9 de 9».

**Estado:** **pagado** · `1eea670`

### M3 · Elegir Gemini o Groq con el API apagado no llega a Rust

**Dónde:** `src/pantallas/Ia.tsx:57-60` · `src-tauri/src/lib.rs:1939-1951`

Con el API apagado, `elegir` solo cambia el estado local. `guardar_clave_del_api` devuelve el `EstadoDeLaIa` de Rust, que sigue diciendo `externo: "claude"`, y la pantalla vuelve sola a Claude.

El flujo natural falla así: elegir Gemini, pegar la clave, guardar y encender responde «sin clave», porque se intenta encender Claude.

**Ajuste ejecutable**
1. Cambiar `Ia.tsx:57-60` por `const elegir = (externo: Externo) => aplicar(apiExterna(ia.api.encendida, externo));`. `api_externa` con `encendida: false` guarda la elección sin exigir clave (`lib.rs:1928-1933`).
2. Test en `tests/unit/`, con el puente simulado: pulsar «Gemini» con el API apagado invoca `api_externa` con `{encendida:false, externo:"gemini"}`.

**Verificado cuando:** el test pasa de rojo a verde.

**Estado:** **pagado** · `1eea670`

### M4 · El modo solo audio habla por HDMI, DisplayPort y AirPlay

**Dónde:** `src-tauri/src/capture/nativo.rs:209-226` · `src-tauri/src/habla/mod.rs:183-205`

Todo transporte que no sea `bltn` cae en `Salida::Otra`, con `puede_haber_eco() == None`, y entonces la app habla. La orden pedía «auriculares obligatorios». La desviación declarada (hablar con USB y Bluetooth porque no se distinguen) no alcanza a transportes que **sí** se distinguen y son casi siempre altavoces: el monitor por HDMI o DisplayPort y AirPlay. Por ahí el micrófono recoge la ficha y el cliente la oye.

**Ajuste ejecutable**
1. Añadir en `nativo.rs`, junto a `:62`, las constantes `cuatro(b"hdmi")`, `cuatro(b"dprt")` y `cuatro(b"airp")`.
2. Crear la variante `Salida::AltavozExterno { nombre }`, con `puede_haber_eco() = Some(true)`.
3. Sacar la clasificación a una función pura `fn clasificar(transporte: u32, fuente: Option<u32>, nombre: String) -> Salida`, con test por transporte.
4. Actualizar la copia no-mac (`src-tauri/src/capture/mod.rs:27-33`), la unión TS (`src/cuaderno.ts:195`), la muestra del contrato y `hayEco` en `Sesion.tsx:48`.

**Verificado cuando:** `cabe_decirla` devuelve `TeOiriaElCliente` para `AltavozExterno` (test) y `pnpm typecheck` pasa.

**Estado:** **pagado** · `7a50222`

### M5 · Sin manifiesto de comandos: la banda y el relleno pueden llamar a todo, incluida la clave del API

**Dónde:** `src-tauri/build.rs:2` · `src-tauri/capabilities/banda.json` · `src-tauri/capabilities/relleno.json`

Tauri 2 permite todos los comandos de `generate_handler!` a todas las ventanas salvo que haya `AppManifest::commands`. La DoD pedía «capabilities mínimas (los comandos nuevos, y solo esos)». Hoy la banda —la ventana que pinta texto de terceros encima de la reunión— puede invocar `guardar_clave_del_api`, `api_externa` y `redactar_sugerencias`. El relleno, que según su capability «no puede tener contenido», también.

**Ajuste ejecutable**
1. En `build.rs:2`, `tauri_build::try_build(tauri_build::Attributes::new().app_manifest(tauri_build::AppManifest::new().commands(&[/* todos los de lib.rs:838-885 */]))).expect("manifiesto")`.
2. En `default.json`, `allow-<comando>` de todo lo que usa el cuaderno.
3. En `banda.json`, solo los que llaman `src/ficha.ts`, `src/turnos.ts`, `src/asa.ts`, `src/acople.ts`, `src/radar.ts` y los hooks de `src/cuaderno.ts` que usa `Banda.tsx`: `pedir_ficha`, `radar_de_tu_mac`, `turnos_recientes`, `ajustar_banda`, `asentar_banda`, `estado_del_acople`, `abrir_lo_que_ve`, `bytes_a_la_red`, `estado_del_corpus`, `estado_de_la_escucha`, `reunion_abierta`, `estado_de_la_voz`, `cortar_todo`.
4. En `relleno.json`, solo `allow-fondo-del-relleno`.
5. Test `tests/unit/capabilities.test.ts`: `banda.json` no contiene ningún `allow-*clave*`, `allow-api-externa` ni `allow-redactar-sugerencias`.

**Verificado cuando:** el test es verde, y en `pnpm tauri dev` un `invoke("guardar_clave_del_api")` desde la consola de la banda es rechazado.

**Estado:** **pagado** · `823c9f6` — verificado por `tauri-build` y `tests/unit/capabilities.test.ts`; el «invoke rechazado desde la consola de la banda» no se pudo ver (consola del webview inaccesible)

### M6 · El gate de campos sin lector no lee los tipos nuevos del sprint

**Dónde:** `tests/unit/contrato-con-lectores.test.ts:31` · `tests/unit/contrato-con-lectores.test.ts:86-89` · `src/ficha.ts:90`

`DECLARACIONES` solo incluye `cuaderno.ts` y `ficha.ts`. `radar.ts`, `ia.ts` y `acople.ts` —donde viven `Sugerencia`, `EstadoDeLaIa`, `EstadoDelApi`, `EnTuMac`, `Programa`, `CatalogoDelRadar` y `EstadoDelAcople`, 32 campos— no se leen: **para ellos el gate no puede fallar**.

Además, el parser de uniones se corta en `({ que: "turno" } & Turno)` (el `& Turno)` no empieza por `|`), así que tampoco ve las variantes de `Novedad` que siguen. Una réplica del gate con esos archivos encuentra 10 huérfanos (casilla 5) donde hoy informa 1.

**Ajuste ejecutable**
1. Añadir `"src/radar.ts"`, `"src/ia.ts"` y `"src/acople.ts"` a `DECLARACIONES`.
2. En `:87-88`, aceptar tras la llave cerrada un `\s*\)?\s*(&\s*\w+\s*\)?)?` y los comentarios antes de buscar `|`.
3. Añadir `RadarEnLaBanda` a `NO_CRUZAN`.
4. Aceptar los accesos por clave calculada (`p.ve[idioma]`) para `Bilingue.es`/`.en`, con una lista corta que diga por qué.
5. Registrar la demo: con el parser arreglado el árbol actual sale rojo, nombrando los huérfanos.

**Verificado cuando:** está en rojo antes de pagar B9–B12 y en verde después.

**Estado:** **pagado** · `3090483`

### M7 · `QueSabeTranscribir` cambió de forma en este sprint y no está en el contrato

**Dónde:** `src-tauri/src/lib.rs:655-697` · `src/cuaderno.ts:275-280` · `src-tauri/src/contrato.rs`

`motivo` pasó de `Option<String>` a `Option<stt::PorQueNoHayMotor>`, y es el único camino por el que ese enum cruza. No tiene muestra en `contrato.rs`: sus tres grafías no las comprueba nadie. La regla 19 dice «todo evento nuevo entra por ahí».

**Ajuste ejecutable**
1. Hacer `QueSabeTranscribir` e `IdiomaDelMotor` `pub(crate)`.
2. Añadir dos muestras en `contrato.rs`, una con `motivo: None` y otra con `Some(PorQueNoHayMotor::…)`, por cada variante.
3. Regenerar `src/contrato.generado.ts` y hacer que el tipo de `cuaderno.ts:275-280` se compruebe contra la muestra, igual que los demás.

**Verificado cuando:** renombrar un campo en `cuaderno.ts` hace fallar `pnpm typecheck`.

**Estado:** **pagado** · `823c9f6`

### M8 · El gate runtime del efímero no ejerce ninguna pieza del sprint 002

**Dónde:** `src-tauri/tests/contra-el-mac-de-verdad.rs:590-667` · `src-tauri/tests/contra-el-mac-de-verdad.rs:557-580`

`una_sesion_completa` corre STT, disparador, ficha y diccionario. No corre Vision (OCR), ni AVSpeechSynthesizer, ni Foundation Models, ni la sesión de red. La DoD pedía «los frames de pantalla están dentro del inventario de lo que muere». Hoy lo sostiene solo el barrido estático, y lo que Apple escriba por debajo al leer, hablar o redactar no se inventaría. La canaria del log tampoco atraviesa OCR ni sugerencia.

**Ajuste ejecutable**
1. Añadir tres pasos dentro de la ventana del inventario:
   - leer `docs/kit-de-prueba/pantalla/tablero-margen.png` con el lector de `pantalla::apple`;
   - `habla::voz().decir("es-ES", …)` si hay voz;
   - `sintesis::sugerir` con `DelSistema` si está disponible, o con `Mock` si no.
2. Que cada paso escriba si se ejerció («OCR ejercido: N líneas»).
3. Añadir `~/Library/Caches/<nombre del ejecutable del test>` a `donde_se_mira`.
4. Demo en rojo: con el paso de OCR quitado, la aserción nueva `assert!(ocr_ejercido)` falla.

**Verificado cuando:** la salida `--nocapture` enseña los tres pasos ejercidos y el gate sigue verde.

**Estado:** **pagado** · `823c9f6`

### M9 · Faltan tres ADRs que la DoD exige: lectura de pantalla, radar y voz

**Dónde:** `decisions/` · `sprints/SPRINT_002-implementation-log.md:1018-1063` · `sprints/SPRINT_002-implementation-log.md:2098-2120`

Solo existen 009, 010 y 011. Decisiones de producto quedan solo en la bitácora: hablar con dispositivos externos, ⎋ global mientras dura el modo, lectura sin región, catálogo en JSON, frases de Meet y Teams «probables».

**Ajuste ejecutable**
Escribir, con esas decisiones y sus mediciones (citando las líneas de la bitácora):
- `decisions/012-lectura-de-pantalla.md` (pHash, Vision, permiso, ⌃⌥L en vez de región, qué muere);
- `decisions/013-el-radar-y-lo-que-no-hace.md` (catálogos con fuente, regla 9, qué no ve, frases probables);
- `decisions/014-la-voz-al-oido.md` (candado, M4, ⎋, fichas sin resultado no se leen).

**Verificado cuando:** los tres archivos existen y el summary los cita.

**Estado:** **pagado** · `7a50222`

### M10 · El tope de US$10/mes cuenta de menos

**Dónde:** `src-tauri/src/sintesis/mod.rs:312-316` · `src-tauri/src/sintesis/api.rs:268` · `src-tauri/src/lib.rs:1733-1743`

Hay dos fugas de cuenta:
- La red de Swift espera TECHO+1 (7 s) y Rust abandona a los 6 s. Una respuesta que llega en ese segundo la factura el proveedor, pero `Respuesta::default()` suma 0 tokens al gasto.
- `guardar_el_gasto` borra el archivo y luego lo crea: una caída entre las dos operaciones deja el mes en 0 y el tope sin efecto.

**Ajuste ejecutable**
1. Añadir a `sugerir` un parámetro `al_llegar_tarde: Box<dyn FnOnce(Respuesta) + Send>`. En `Err(Timeout)`, lanzar un hilo que haga `if let Ok(Ok(r)) = rx.recv() { al_llegar_tarde(r) }`.
2. En `lib.rs`, pasar un cierre que sume el costo igual que `:1868-1885`.
3. En `guardar_el_gasto`, escribir con `nacer_cerrado` en `costo-del-mes.json.tmp` y `std::fs::rename` sobre el definitivo.
4. Test del cierre tardío con el proveedor `Lento` (400 ms, techo 100).

**Verificado cuando:** el test ve la respuesta tardía y el archivo nunca desaparece.

**Estado:** **pagado** · `1eea670`

### M11 · «0 B», «nada sale» y «nada se escribe» siguen escritos como constantes

**Dónde:** `src/componentes/Ventana.tsx:95-104` · `src/i18n/es.ts:293-294` · `src/componentes/Banda.tsx:84` · `src/componentes/Banda.tsx:829-833` · `src/componentes/Banda.tsx:567` · `src/pantallas/Sesion.tsx:177` · `src/i18n/es.ts:320` · `src/i18n/es.ts:537` · `src/pantallas/Corpus.tsx:70-73` · `src/pantallas/Honestidad.tsx:92-101` · `src/i18n/es.ts:370-371`

Con el API encendido, estas superficies siguen diciendo «0 B» o «nada sale de tu equipo»:
- el rail;
- la banda de 44 px;
- el coral ampliado;
- Sesión;
- Corpus.

Además, Honestidad pinta su contador con clase `cero` y un check verde sea cual sea la cifra, cuando Ia.tsx sí cambia de estilo. Eso rompe la regla 8.

Y «nada se escribe en disco» / «hoy no se escribe nada» es falso siempre: la app escribe `diccionario.yaml`, `costo-del-mes.json` y el índice.

**Ajuste ejecutable**
1. Usar `useBytesALaRed()` en esos cinco sitios, en lugar de `RED` y de los literales.
2. Separar la cifra del texto en `sinSesion` y `meetDetectado` de es y en.
3. `nadaSale` y `panelProtegido` solo cuando `bytes === "0 B"`. Si no, una variante nueva es/en que diga los bytes.
4. Cambiar «nada se escribe en disco» por «nada de la reunión se escribe en disco», y lo mismo en `loQueQuedaraDetalle`.
5. En Honestidad, `className={bytes === "0 B" ? "contador cero" : "contador api"}` y el icono verde solo en cero.
6. Actualizar las maquetas afectadas para que el gate i18n↔maqueta siga verde.
7. Test: renderizar Honestidad, la banda de 44 px y Ventana con el puente devolviendo `"1,2 KB"` enseña esa cifra y no el check verde.

**Verificado cuando:** ese test es verde.

**Estado:** **pagado** · `3090483`

### M12 · La banda anuncia teclas y botones que no hacen nada (origen S1)

**Dónde:** `src/componentes/Banda.tsx:270` · `src/componentes/Banda.tsx:483` · `src/componentes/Banda.tsx:639` · `src/componentes/Banda.tsx:689-696` · `src/componentes/Banda.tsx:720` · `src/componentes/Banda.tsx:763-769` · `src/i18n/es.ts:92-94`

- ⌃⌥P «fijar» y ⌃⌥N «anotar» no están registradas: los únicos atajos son ⌥⎋ ⌃⌥T ⌃⌥A ⌃⌥V ⌃⌥R ⌃⌥L.
- Cuatro botones no tienen `onClick`: «Buscar con otras palabras», «Anotar para después», «Ya lo verifiqué en Zoom» y «Modo solo notas».
- El consejo «O pasa a modo solo notas» apunta a un modo que no existe.

Es la superficie que el consultor usa delante del cliente.

**Ajuste ejecutable**
1. Quitar los `kbd` y botones de funciones inexistentes, o pintarlos `disabled` con `<TodaviaNo />`, igual en `docs/diseno/banda.html` para el gate de fidelidad.
2. Test que recorra la banda renderizada y afirme dos cosas: cada combinación `⌃⌥X` de un `kbd` está en la lista de `las_teclas_de_la_app_son_control_opcion` (`src-tauri/src/lib.rs:2312`); y todo `<button>` tiene manejador o `disabled`.

**Verificado cuando:** ese test pasa de rojo a verde.

**Estado:** **pagado** · `3090483`

### M13 · El manual dice que al modelo y al API sale «tu pregunta»; sale la frase del cliente

**Dónde:** `docs/MANUAL-DE-USO.md:252` · `docs/MANUAL-DE-USO.md:264-268` · `src-tauri/src/lib.rs:1847-1854`

Lo que sale es el último turno de la pista del sistema, es decir, palabras de un tercero. Es justo lo que el usuario necesita saber antes de encender el API.

**Ajuste ejecutable**
1. Cambiar «tu pregunta» por «la última frase de tu cliente» en esas líneas y en la sección de red del manual (`:413`).
2. Revisar con `grep -n "tu pregunta" docs/MANUAL-DE-USO.md`.

**Verificado cuando:** ese grep no devuelve líneas que hablen de lo que sale.

**Estado:** **pagado** · `3090483`

### M14 · Arrastrar el asa no saca del modo solo audio

**Dónde:** `docs/MANUAL-DE-USO.md:311-312` · `src/componentes/Banda.tsx:863-866` · `src/asa.ts:43-60` · `src-tauri/src/lib.rs:223-243`

El manual y el propio título del asa dicen que arrastrar «apaga el modo». Pero el asa llama a `ajustar_banda` y `asentar_banda`, que no tocan `LaVozQueSale.encendida`. La ventana crece, la banda sigue pintando la línea de 44 px, la app **sigue hablando y sigue quedándose con ⎋ de Meet**.

**Ajuste ejecutable**
1. Extraer de `conmutar_el_modo` (`lib.rs:1284-1288`) una función `apagar_el_modo(app)`: `con_el_callar(false)`, `voz.callar()`, `encendida.store(false)` y emitir `EVENTO_VOZ`.
2. Al principio de `ajustar_banda` (`lib.rs:223-225`), que solo usa el asa: `if alto > ventana::ALTO_VOZ && app.state::<LaVozQueSale>().encendida.load(..) { apagar_el_modo(&app) }`.

**Verificado cuando:** en `pnpm tauri dev`, con el modo encendido, arrastrar el asa escribe `[habla] ⎋ devuelta al sistema` y la banda pinta 88 px.

**Estado:** **pagado** · `7a50222`

### M15 · La deuda «maniobra genérica · sprint 2» no se pagó ni se re-declaró

**Dónde:** `design-system.md:524` · `decisions/008-el-corpus-el-disparo-y-la-ficha.md:108-113` · `src-tauri/src/ficha/maniobra.rs:14-17`

Es un requisito del usuario (mirada 11: «a medida de la situación») con sprint de pago explícito, el 2. No aparece en la orden, ni en el plan, ni en la bitácora, ni en el diff. Es la misma clase que los quince hallazgos del S1 que se perdieron.

**Ajuste ejecutable**
Hay dos caminos:
- Construirla con el camino determinista de `design-system.md:524`.
- O re-declararla **con sprint de pago** en esos tres sitios y en la tabla de deuda de `sprints/SPRINT_002-summary.md`, citando la mirada 11.

**Verificado cuando:** los tres sitios nombran el mismo sprint de pago, o el test de la maniobra nueva está verde.

**Estado:** **pagado** · `823c9f6` — construida (decisión del usuario): el puente

## BAJOS (49)

| # | Hallazgo | Dónde | Ajuste ejecutable | Estado |
|---|---|---|---|---|
| **B1** | `hay_clave` lee la clave entera del Llavero cada vez que se pinta IA para saber si existe. `Llavero.swift` afirma que solo se lee «en el instante de enviar». | `src-tauri/src/sintesis/api.rs:190-195` · `src-tauri/nativo/Llavero.swift:1-3` · `src-tauri/src/lib.rs:1776` · `src-tauri/src/lib.rs:1787` · `src-tauri/src/lib.rs:1928` | Añadir `@_cdecl("ag_llavero_hay")` que consulte con `kSecReturnAttributes: true` y sin `kSecReturnData`, y devuelva 1, 0 o -1. Declararlo en `api.rs` (`mod puente`) y usarlo en `hay_clave`. Verificado: `grep -n "leer_clave(" src-tauri/src` solo devuelve `api.rs:246`. | **pagado** · `1eea670` |
| **B2** | Cualquier otra razón de indisponibilidad de Foundation Models (código -4) se muestra como «Este Mac no puede usar el modelo del sistema». Es copy falso. | `src-tauri/src/sintesis/sistema.rs:27-34` · `src-tauri/nativo/Sintesis.swift:64-66` | Añadir `PorQueNoRedacta::NoDisponible` (kebab `no-disponible`) con frase es/en en `porQueNoRedacta`, la unión de `src/ia.ts:14-20` y una muestra del contrato. Mapear `-4` y `_` a esa variante. Test: `motivo(-4) == NoDisponible`. | **pagado** · `1eea670` |
| **B3** | Con la app abierta al cambiar de mes, el gasto del mes anterior sigue bloqueando el API: solo se reinicia dentro de una llamada al API que ya no ocurre. | `src-tauri/src/lib.rs:1775` · `src-tauri/src/lib.rs:1789` · `src-tauri/src/lib.rs:1877-1879` | Crear `fn gasto_vigente(g: &mut GastoDelMes, hoy: &str)` que ponga a cero si `g.mes != hoy`, y llamarla antes de leer `usd` en `proveedor_de_ahora` y en `estado_de_la_ia_de`. Test con dos meses inyectados. | **pagado** · `1eea670` |
| **B4** | Los ADRs afirman lo que el código no hace: «cuatro proveedores», que `AG_SINTESIS` acepte `sistema` y `api` (solo lee `mock`), que la bóveda use el diccionario, que el cliente HTTP viva en `sintesis/api.rs` (vive en `Red.swift`) y que el texto que salió se pueda leer en IA (no existe). | `decisions/011-proveedores-del-modelo-y-minimizacion.md:15-20` · `decisions/011-proveedores-del-modelo-y-minimizacion.md:33-42` · `decisions/010-sintesis-codigo-primero.md:71-72` | Enmendar con lo real: tres construidos y MLX condicionado; solo `AG_SINTESIS=mock`; bóveda con los clientes del corpus (tras A2); red en `nativo/Red.swift`; «el texto que salió» declarado como pendiente con sprint. Verificado: releer contra `lib.rs:1766-1781`. | **pagado** · `1eea670` |
| **B5** | ⌃⌥L pide sugerencia con el último turno del cliente, que puede ser de hace minutos y de otra cosa. El comentario dice que lleva «la pregunta del usuario», y no la hay. | `src-tauri/src/lib.rs:1549-1551` · `src-tauri/src/lib.rs:1833-1836` | Quitar `sintetizar(app, &a)` de `atender_la_pantalla` y reescribir el comentario (misma razón que la ficha de pantalla). Verificado: tras ⌃⌥L el log no escribe `[sintesis]`. | **pagado** · `1eea670` |
| **B6** | `Refuerzo` no tiene `Drop`. Cada lectura suelta sin pisar la copia `Resultado::Leido.refuerzo` y sus clones, contra la cabecera del módulo («el texto leído se pisa antes de soltarse»). | `src-tauri/src/pantalla/refuerzo.rs:53-57` · `src-tauri/src/pantalla/mod.rs:5-6` · `src-tauri/src/pantalla/mod.rs:519-547` | `impl Drop for Refuerzo { fn drop(&mut self) { self.olvidar(); } }`. Si algún sitio mueve campos (E0509), usar `std::mem::take`. Verificado: `cargo test --lib pantalla` verde y el `impl Drop` presente. | **pagado** · `823c9f6` |
| **B7** | El ámbar afirma que el bot «está en la lista de participantes», pero coteja cualquier línea de la ventana: una diapositiva o el chat que mencione «Otter.ai» o «tl;dv» lo dispara. | `src-tauri/src/radar/avisos.rs:59-70` · `src/i18n/es.ts:155` · `src/i18n/en.ts:110` | Cambiar el copy a «aparece en la ventana de la reunión» / «appears in the meeting window» en i18n, `docs/diseno/banda.html` y el manual. Verificado: el gate i18n↔maqueta sigue verde. | **pagado** · `3090483` |
| **B8** | Seis comandos registrados sin llamador en el webview, todos invocables: `modo_solo_audio` (nuevo) y `cerrar_banda`, `acoplar`, `soltar_acople`, `pedir_permiso_de_acople`, `version_del_catalogo` (S1). | `src-tauri/src/lib.rs:842-849` · `src-tauri/src/lib.rs:867` · `src-tauri/src/lib.rs:1252-1255` | Sacarlos de `generate_handler!` y quitarles `#[tauri::command]` si se usan por dentro. Añadir un test que compare los nombres de `generate_handler!` con los que invoca `src/`. Verificado: el test es verde. | **pagado** · `823c9f6` |
| **B9** | `InformeDelCorte.bytesEnRed` cruza sin lector: lo lee Rust antes de serializar. El gate lo exime con «No se paga», un tercer estado que la regla 20 no tiene, y el doc dice «Es 0 siempre». | `src-tauri/src/corte.rs:116-124` · `tests/unit/contrato-con-lectores.test.ts:59-60` · `tests/unit/contrato-con-lectores.test.ts:184` | Poner `#[serde(skip)]` a `bytes_en_red`, quitarlo del tipo de `src/cuaderno.ts:257-260` y regenerar la muestra. Borrar la entrada de `DEUDA` y la salida «No se paga» del filtro. Corregir el doc. Verificado: el gate queda verde con `DEUDA` vacía. | **pagado** · `3090483` |
| **B10** | `EstadoDelAcople.permiso` no tiene lector, y Rust hace una llamada a la Accessibility API en cada consulta para rellenarlo (origen S1). | `src/acople.ts:17` · `src-tauri/src/lib.rs:252-263` | Quitar el campo y la llamada `acople::hay_permiso()` de `estado_ahora`, o darle lector con copy aprobado. Verificado: el gate de M6 no lo lista. | **pagado** · `3090483` |
| **B11** | Siete campos de `Novedad` no tienen lector: `Empieza.pista`, los cuatro de `SinTexto` y los dos de `Ruido`. Además `Novedad::Turno` manda el **texto del cliente** a todas las ventanas en una copia que nadie lee (la banda lo vuelve a pedir con `turnos_recientes`), y `SinTexto.motivo` sigue siendo castellano libre. | `src-tauri/src/escucha/mod.rs:66-72` · `src-tauri/src/escucha/mod.rs:831` · `src/ficha.ts:89-98` · `src-tauri/src/lib.rs:616` | Poner `#[serde(skip)]` a los campos no leídos de `Empieza`, `SinTexto` y `Ruido`, y a `texto`, `hora`, `desde_ms` y `hasta_ms` del turno en el evento (con `Turno` aparte para `turnos_recientes`), o crear un `NovedadParaLaBanda`. Cerrar `motivo` en un enum si se queda. Test: el JSON serializado de `Novedad::Turno` no contiene `texto`. | **pagado** · `3090483` |
| **B12** | `Aparicion.hora` solo se lee en la rama de la maqueta. En el producto la banda enseña la hora del último turno, no la de la ficha. | `src/componentes/Banda.tsx:666` · `src-tauri/src/ficha/mod.rs:99-106` | Usar `aparicion.hora` en la rama del producto de la línea «oído», o `#[serde(skip)]`. Verificado: el gate de M6 no lo lista. | **pagado** · `3090483` |
| **B13** | El gate del contador de red no ve `std::process::Command` fuera de `radar/`: un `Command::new("curl")` en `lib.rs` pasa todos los gates. La regla 9 prohíbe salidas fuera del adapter. | `tests/unit/contador-de-red.test.ts:52-65` | Añadir `Command::new` a `SOCKETS_RUST`, con una lista permitida `{src-tauri/src/radar/mdm.rs → /usr/bin/profiles}`. Demo en rojo con `Command::new("curl")` plantado en `lib.rs`. | **pagado** · `823c9f6` |
| **B14** | Sin voz para el idioma, ⌃⌥V enciende igual el modo: baja a 44 px, se queda con ⎋ y la banda dice «Conecta auriculares», que es el motivo equivocado. El manual dice que «el modo no se enciende». | `src-tauri/src/lib.rs:1262-1290` · `src-tauri/src/habla/mod.rs:207-209` · `docs/MANUAL-DE-USO.md:331` | En `conmutar_el_modo`, si se va a encender y `!estado.voz.hay_para(&idioma)`, no encender y escribir `[habla] sin voz para {idioma}: el modo no se enciende`. Test sobre una función pura `puede_encender(hay_voz)`. | **pagado** · `3090483` |
| **B15** | Un Mac inscrito en un MDM sin ningún invasivo no lo ve en ningún sitio. El manual dice que Sesión lo lista. | `src/pantallas/Sesion.tsx:62-67` · `docs/MANUAL-DE-USO.md:210-211` | Pintar en Sesión la fila «Sábelo» del MDM cuando `radar.programas` tenga `categoria === "mdm"`, sin tomar la pantalla entera, o corregir el manual. Test: un `EnTuMac` con solo el MDM enseña la fila. | **pagado** · `3090483` |
| **B16** | El botón de Sesión dice «Iniciar sesión» también cuando lo que hace es pararla (regla 8). | `src/pantallas/Sesion.tsx:161-171` | Etiqueta e icono según `escucha.escuchando`, con la clave nueva `terminarSesion` es/en. Test: con `escuchando: true` se lee «Terminar sesión». | **pagado** · `3090483` |
| **B17** | El rail dice «Meet detectado» con cualquier reunión, también en Zoom o Teams. | `src/componentes/Ventana.tsx:95-99` · `src/i18n/es.ts:294` · `src/i18n/en.ts:223` | Usar `useReunion()` y pintar «{cliente} detectado». Test con cliente Zoom. | **pagado** · `3090483` |
| **B18** | El contador dice «salieron de tu equipo en esta reunión», pero solo lo pone a cero ⌥⎋: una reunión hereda los bytes de la anterior. | `src-tauri/src/lib.rs:536-545` · `src-tauri/src/red.rs:33` · `src/i18n/es.ts:355` | Llamar `red::reiniciar()` en `empezar_a_escuchar`, junto al reinicio del costo de la reunión. Verificado: el log de inicio escribe la red en 0. | **pagado** · `3090483` |
| **B19** | Tres frases más del producto son falsas hoy:<br>1. Idioma: «La banda sigue funcionando con ⌃⌥A…; lo que no llega es la ficha automática». ⌃⌥A necesita un turno transcrito, y la ficha de pantalla sí llega.<br>2. Idioma: «los idiomas que marques». No se marca ninguno (A4).<br>3. Permisos: «Buscar tu evidencia a mano — Funciona» sin conceder nada. ⌃⌥A necesita el audio del sistema. | `src/i18n/es.ts:475` · `src/i18n/en.ts:377` · `src/i18n/es.ts:384` · `src/i18n/en.ts:301` · `src/pantallas/Permisos.tsx:136-141` | Reescribir `laBandaSigue`, quitar «que marques» hasta A4 y marcar «Buscar a mano» como `<TodaviaNo />`, también en las maquetas. Verificado: el gate i18n↔maqueta sigue verde y las frases describen lo que el código hace. | **pagado** · `3090483` |
| **B20** | Comentarios y documentos con promesas caducadas o falsas. La lista completa está en la casilla 4, primera pasada, filas marcadas B20. | `src/pantallas/Honestidad.tsx:22-23` · `src-tauri/src/red.rs:8-11` · `src-tauri/src/corte.rs:106-107` · `src-tauri/src/sintesis/mod.rs:67-69` · `src-tauri/src/ficha/maniobra.rs:10-12` | Reescribir cada línea de la tabla de la casilla 4 marcada B20 para que diga lo que el código hace hoy. Verificado: un segundo barrido de la casilla 4 no las devuelve. | **pagado** · `3090483` |
| **B21** | Dos presupuestos de la DoD sin medir: «la voz empieza ≤1 s tras la ficha» y «la CPU de la lectura de pantalla». El peso del binario va con el release-check. | `src-tauri/tests/contra-el-mac-de-verdad.rs:1257-1310` · `src-tauri/tests/contra-el-mac-de-verdad.rs:1403-1622` | Voz: en `la_voz_de_este_mac…`, medir desde `decir` hasta `hablando()==true` (sondeo de 10 ms, techo 2 s) e imprimirlo. CPU: en el kit de pantalla, `libc::getrusage(RUSAGE_SELF)` antes y después de 10 `una_vuelta` y el tiempo de CPU por lectura. Anotar los números en la bitácora y el summary. | **pagado** · `823c9f6` — CPU medida en el kit; la voz se mide en vivo (log `[habla] empezó a sonar…`) y el número sale en la parada h1 del gate del MVP |
| **B22** | *(hallazgo nuevo de la Fase 2, del subagente de los ADR)* El manual, el summary y la bitácora decían «100 procesos» en el Mac limpio del kit del radar: son 82, contados con el filtro del test | `docs/kit-de-prueba/radar/mac-limpio.txt` · `src-tauri/src/radar/procesos.rs:108-124` · `docs/MANUAL-DE-USO.md` | Corregir la cifra a 82 en el manual y el summary; nota de corrección en la bitácora | **pagado** · `3090483` |
| **B23** | *(hallazgo nuevo de la Fase 2)* La cabecera de `huella.rs` defendía el pHash, que el vigía ya no usa; las funciones de pHash solo las usaban los tests | `src-tauri/src/pantalla/huella.rs:8-13` · `src-tauri/src/pantalla/huella.rs:119-122` | Reescribir la cabecera (miniaturas por zonas) y dejar el pHash `#[cfg(test)]` | **pagado** · `3090483` |
| **B24** | *(hallazgo nuevo de la Fase 2)* El texto que Vision lee se juntaba en `String` de Swift que se soltaban sin pisar | `src-tauri/nativo/Pantalla.swift:182-198` | Escribir byte a byte en el búfer de Rust, sin copias propias; si no cabe, pisar lo escrito y devolver `cabeMal` | **pagado** · `3090483` |
| **B25** | *(segunda pasada; creado por B9)* El log del corte escribía siempre `red 0 B`: la pieza `ContadorDeRed` pone el contador a cero dentro del bucle y el informe lo leía después. El doc de `Informe` afirmaba lo contrario. | `src-tauri/src/corte.rs:131-134` · `src-tauri/src/lib.rs:752` · `src-tauri/src/lib.rs:778` · `src-tauri/src/lib.rs:800` | Leer `red::bytes()` antes del bucle y pasarlo al informe. Test de orden `el_contador_se_lee_antes_de_ponerlo_a_cero`, rojo con el orden viejo. | **pagado** · `eefc7c3` |
| **B26** | *(segunda pasada)* «Sin Accesibilidad todo lo demás funciona igual» / «el único permiso opcional»: con Meet en el navegador, sin ese permiso la app no encuentra la pestaña, no lee su pantalla y el ámbar no mira. | `docs/MANUAL-DE-USO.md:55-56` · `docs/MANUAL-DE-USO.md:187-188` · `src-tauri/src/permisos.rs:89-90` · `src-tauri/src/permisos.rs:319-320` · `src/componentes/Banda.tsx:75-76` · `design-system.md:515-516` | Manual, comentarios y `design-system.md` dicen para qué más hace falta. | **pagado** · `eefc7c3` |
| **B27** | *(segunda pasada; creado por B19)* «DOS de las tres funcionan sin permisos»: tras B19 solo funciona indexar. | `src/pantallas/Permisos.tsx:125-129` · `docs/diseno/permisos.html:251-255` · `docs/diseno/permisos.html:298-302` | El comentario y la maqueta dicen «una de las tres» y por qué. | **pagado** · `eefc7c3` |
| **B28** | *(segunda pasada; creado por M4)* Frases que decían que solo calla con los altavoces internos o que todo externo es desconocido; HDMI, DisplayPort y AirPlay ya son altavoz, y Sesión ya pinta `Salida.nombre`. | `src-tauri/src/habla/mod.rs:108-109` · `src-tauri/src/habla/mod.rs:158-162` · `src-tauri/src/habla/mod.rs:170-173` · `src-tauri/src/habla/mod.rs:179-182` · `src/pantallas/Sesion.tsx:404-408` · `docs/MANUAL-DE-USO.md:84-86` · `decisions/014-la-voz-al-oido.md:58-60` · `decisions/014-la-voz-al-oido.md:64-66` · `decisions/014-la-voz-al-oido.md:170` | Reescritas con las tres respuestas: altavoz conocido (internos, HDMI, DisplayPort, AirPlay), auriculares por conector, USB/Bluetooth sin saber. | **pagado** · `eefc7c3` |
| **B29** | *(segunda pasada)* Los ADR 012, 013 y 014 listaban como abiertos hallazgos ya pagados (B6, M8, B21, B24, B23, B7, B15, B13, B22, B14, M4, M14). | `decisions/012-lectura-de-pantalla.md:175-184` · `decisions/013-el-radar-y-lo-que-no-hace.md:172-183` · `decisions/014-la-voz-al-oido.md:175-179` · `decisions/014-la-voz-al-oido.md:194-198` | Cada ADR dice «ya pagado» y cómo; la cifra 82 con su nota en la bitácora. | **pagado** · `eefc7c3` |
| **B30** | *(segunda pasada; creado por M15)* «Seis maniobras» en código, i18n, `design-system.md`, maqueta y ADR 010: son siete con el puente. Y «cuando exista la síntesis, la maniobra es su fallback»: el fallback es la ficha. | `src-tauri/src/ficha/maniobra.rs:3` · `src-tauri/src/ficha/maniobra.rs:84` · `src/i18n/es.ts:56-58` · `design-system.md:395` · `design-system.md:414-415` · `docs/diseno/banda.html:593` · `decisions/010-sintesis-codigo-primero.md:22` | «Siete» en los seis sitios; la genérica dice cuándo sale; el fallback, bien dicho. | **pagado** · `eefc7c3` |
| **B31** | *(segunda pasada; creado por A3)* «Cuatro letras o menos» / «por debajo de cinco»: A3 subió el tramo exacto a seis letras y los nombres de cliente cortos a ocho. | `docs/MANUAL-DE-USO.md:385-387` · `src-tauri/src/diccionario/mod.rs:36` · `src-tauri/src/diccionario/mod.rs:120-121` | Seis letras en el manual y el código; el manual añade la regla de los nombres de menos de ocho. | **pagado** · `eefc7c3` |
| **B32** | *(segunda pasada; creado por B7)* Cinco comentarios seguían con «lista de participantes»; el ámbar coteja cualquier línea de la ventana. | `src/componentes/Banda.tsx:237-238` · `src/ficha.ts:99-100` · `src-tauri/src/escucha/mod.rs:112-113` · `src-tauri/src/radar/mod.rs:6-7` · `src-tauri/src/radar/avisos.rs:4-5` | «aparece en la ventana de la reunión» en los cinco. | **pagado** · `eefc7c3` |
| **B33** | *(segunda pasada; creado por la guía v4)* «Cinco bloques nuevos con 31 pruebas nuevas» (son 26 + 5 en bloques viejos), «8 piezas» (son 9) y un historial que decía «tres» y enumeraba cuatro. | `docs/GUIA-DE-PRUEBA.html:170-172` · `docs/GUIA-DE-PRUEBA.html:580-584` | Cabecera e historial con 26 + 5, 9 piezas y las cinco enumeradas. | **pagado** · `eefc7c3` |
| **B34** | *(segunda pasada; verificación de A4)* k1b pedía ver en la consola que la pista del sistema abre en inglés, y ningún log escribía el idioma. | `docs/GUIA-DE-PRUEBA.html:552` · `src-tauri/src/escucha/mod.rs:361` | El log dice `[escucha] pista «sistema» abierta · en-US`, y k1b cita esa línea. | **pagado** · `eefc7c3` |
| **B35** | *(segunda pasada)* «La CI mide el WER»: la CI no tiene modelos para reconocer y lo declara sin medir. | `docs/GUIA-DE-PRUEBA.html:500` · `docs/kit-de-prueba/audio/LEEME.md:29` · `docs/kit-de-prueba/audio/LEEME.md:65` · `decisions/009-un-idioma-por-pista.md:94` | «En un Mac con modelos de voz; en la CI no mide», en los cuatro. | **pagado** · `eefc7c3` |
| **B36** | *(segunda pasada; creado por A4)* ADR 009 citaba las constantes `DEL_CONSULTOR` / `DEL_CLIENTE`, que ya no existen. | `decisions/009-un-idioma-por-pista.md:69-70` | El idioma se elige por pista (A4). | **pagado** · `eefc7c3` |
| **B37** | *(segunda pasada; creado por B4)* «IA enseña cuántos datos se taparon»: ninguna pantalla lo enseña; solo lo leen los tests. | `decisions/011-proveedores-del-modelo-y-minimizacion.md:50` · `src-tauri/src/sintesis/anonimo.rs:39` | El ADR y el comentario lo dicen tal cual. | **pagado** · `eefc7c3` |
| **B38** | *(segunda pasada; creado por M5)* Las descripciones de `banda.json` y `relleno.json` no nombraban los comandos que M5 les dio. | `src-tauri/capabilities/banda.json:4` · `src-tauri/capabilities/relleno.json:4` | Cada descripción dice qué comandos permite y quién lo vigila. | **pagado** · `eefc7c3` |
| **B39** | *(segunda pasada)* La mirada 16-bis seguía «pendiente» en código y contrato, la tabla del estado callado describía lo de antes, y «cinco cadenas» eran siete. | `src/componentes/Banda.tsx:816-817` · `src/componentes/Banda.tsx:826` · `src-tauri/src/contrato.rs:302-305` · `src/i18n/es.ts:115` | Aprobada el 2026-09-26; «Callado · esperando el siguiente turno»; siete cadenas. | **pagado** · `eefc7c3` |
| **B40** | *(segunda pasada; tras B15)* «Sesión lo lista como Sábelo dentro de la tabla»: con solo un MDM no hay tabla. | `docs/MANUAL-DE-USO.md:221-222` | «en su lista de lo que funciona, o dentro de la tabla si además hay un invasivo». | **pagado** · `eefc7c3` |
| **B41** | *(segunda pasada)* «lo único que la app escribe es tuyo —índice, diccionario y gasto—»: también escribe `acople.json`, dónde estaba la ventana de la reunión. | `src/i18n/es.ts:377` · `src/i18n/en.ts:293-294` · `docs/diseno/honestidad.html:262` · `docs/diseno/honestidad.html:292` | La frase suma «dónde estaba la ventana de la reunión, para devolvérsela», es/en y maqueta; fidelidad 116 verde. Su hermana en la casilla 12 del release-check del summary (`sprints/SPRINT_002-summary.md`), también. | **pagado** · `eefc7c3` |
| **B42** | *(segunda pasada; creado por B23)* El diagrama del módulo decía «huella (pHash)»: es por zonas. | `src-tauri/src/pantalla/mod.rs:13` | «huella (zonas)». | **pagado** · `eefc7c3` |
| **B43** | *(segunda pasada; resto de B20)* «la ÚNICA puerta a la red de un módulo protegido»: `Red.swift` también lo es. | `src-tauri/nativo/Transcriptor.swift:160` | «la puerta a la red de la transcripción —la otra es `Red.swift`—». | **pagado** · `eefc7c3` |
| **B44** | *(segunda pasada; tras M6)* «vive en la deuda del sprint»: los lectores del contrato ya tienen su gate. | `src-tauri/src/contrato.rs:27` | Apunta a `tests/unit/contrato-con-lectores.test.ts`. | **pagado** · `eefc7c3` |
| **B45** | *(segunda pasada; clase B20)* Futuros de cosas que existen desde el S1: «el disparador de la fase 4 leerá», «la banda sabrá». | `src-tauri/src/stt/ventana.rs:3` · `src-tauri/src/stt/mod.rs:28` · `src-tauri/src/voz/eco.rs:27-28` | En presente, con dónde vive. | **pagado** · `eefc7c3` |
| **B46** | *(segunda pasada)* «Los otros tres atajos emiten a la banda»: solo `⌃⌥T` y `⌃⌥A`. | `src-tauri/src/lib.rs:1251` | Nombra los dos. | **pagado** · `eefc7c3` |
| **B47** | *(segunda pasada)* «El Swift vive entero en un archivo»: son seis. | `decisions/006-stt-local-y-su-modelo.md:92` | Los seis, nombrados. | **pagado** · `eefc7c3` |
| **B48** | *(segunda pasada)* `design-system.md` decía que audio del sistema y pantalla son UN permiso; son dos (mirada 17-quater, `tccd`). | `design-system.md:511-513` | DOS permisos, con sus nombres TCC y la historia. | **pagado** · `eefc7c3` |
| **B49** | *(segunda pasada; creado por B8)* ADR 004 daba `cerrar_banda` como primer camino de devolución; el comando salió en B8. | `decisions/004-acople-por-accessibility.md:63-65` | El primer camino es el corte (`corte::Pieza::Acople`). | **pagado** · `eefc7c3` |

## Casilla 4 — frases caducadas (primera pasada)

El barrido se hizo por vocabulario de promesa aplazada («todavía no», «más adelante», «sprint 2», «0 B», «nada sale», «ya no», «not yet», «will»…) sobre el manual, el README, la guía, las maquetas, i18n, comentarios y ADRs. Cada acierto se cotejó con el código.

- **Teclas:** no queda ninguna combinación `⌘⇧` que no sea historia explícita.
- **README.md:** sigue siendo la plantilla de Tauri. No afirma nada, así que no caduca nada.
- **Guía:** en `ea45751` ya es la v4 (fase 6, no cuenta). Su única frase falsa detectada es la del asa (→ M14), y usa voz castellana para la pista en-US (→ A4).
- **Manual:** se leyó con un cambio sin comitear en `:148-149`, que es correcto.

**Promesas aplazadas que ya no son ciertas**

| Dónde | Frase | Por qué es falsa hoy | Hallazgo |
|---|---|---|---|
| `src/i18n/es.ts:293-294` · `src/i18n/en.ts:222-223` | «Sin sesión · 0 B» / «Meet detectado · 0 B» | es un literal, y el API mueve el contador | M11, B17 |
| `src/componentes/Banda.tsx:83-84` | «En el sprint 001 no hay red» (`RED = "0 B"`) | se pinta en la banda de 44 px y en el coral | M11 |
| `src/i18n/es.ts:164-166` | «0 B en red · nada persiste» | constante en el coral ampliado | M11 |
| `src/i18n/es.ts:537` · `src/i18n/en.ts:436` | «0 B a la red · nada se escribe en disco» | escribe el diccionario y el costo | M11 |
| `src/i18n/es.ts:320` · `src/i18n/en.ts:247` | «nada sale de tu equipo» | con el API sí sale | M11 |
| `src/i18n/es.ts:370-371` · `src/i18n/en.ts:287-288` | «hoy no se escribe nada» | se escriben el diccionario y el costo | M11 |
| `src/pantallas/Corpus.tsx:72` | «0 B» literal | extractos de las fichas salen con el API | M11 |
| `src/i18n/es.ts:475` · `src/i18n/en.ts:377` | «La banda sigue funcionando con ⌃⌥A…» | falsa en sus dos mitades | B19 |
| `src/i18n/es.ts:384` · `src/i18n/en.ts:301` | «los idiomas que marques» | son constantes | B19, A4 |
| `src/cuaderno.ts:549-555` · `decisions/006-stt-local-y-su-modelo.md:95` | «Elegirlos es de la fase siguiente» | no se eligen | A4 |
| `docs/MANUAL-DE-USO.md:118-119` · `src-tauri/src/ficha/mod.rs:7-9` | «La app no redacta» | desde C7 redacta (la ficha no) | B20 |
| `src/pantallas/Honestidad.tsx:22-23` | «no existe código capaz de abrir una conexión» | existe `nativo/Red.swift` | B20 |
| `src-tauri/src/red.rs:8-11` · `src-tauri/src/red.rs:97-105` | «existe ya, vacío… (sprint 2)» | tiene llamador (`api.rs:254`) | B20 |
| `src-tauri/src/corte.rs:106-107` · `src-tauri/src/corte.rs:122-123` | «lo que hay es poco» / «Es 0 siempre» | 8 de 8; los bytes del API | B20, B9 |
| `src-tauri/src/lib.rs:514` · `src-tauri/src/ventana/mod.rs:86` | «una de las siete piezas» | son ocho | B20 |
| `src/pantallas/Idioma.tsx:22-23` | «todavía no: … el diccionario técnico» | está construido | B20 |
| `src/pantallas/Idioma.tsx:26-28` · `src-tauri/nativo/Transcriptor.swift:149-150` | «la única vez que un módulo protegido toca la red» | `sintesis` también | B20 |
| `src/asa.ts:7` | «cuando exista el acople» | existe desde el S1 | B20 |
| `src-tauri/src/ficha/maniobra.rs:10-17` · `decisions/008-el-corpus-el-disparo-y-la-ficha.md:108-113` | «cuando llegue la síntesis… fallback» · «trabajo del sprint 2» | el fallback es la ficha; la deuda no se pagó | B20, M15 |
| `docs/kit-de-prueba/LEEME.md:39` · `decisions/008-el-corpus-el-disparo-y-la-ficha.md:89` | «el sprint 2 decidirá si los embeddings hacen falta» | no se decidió | B20 |
| `docs/kit-de-prueba/LEEME.md:46-50` | «audio/ — dos preguntas» | son cuatro audios más pantalla/ y radar/ | B20 |
| `decisions/011-proveedores-del-modelo-y-minimizacion.md:15-18` | «cuatro proveedores» | tres | B4 |
| `decisions/006-stt-local-y-su-modelo.md:96-97` · `decisions/007-el-audio-en-dos-pistas.md:26-28` | «son del sprint 2» · C15 «hablará por los altavoces» | entregado · los rechaza | B20 |
| `docs/diseno/banda.html:638-645` | «El modo solo audio… no entra en este sprint» | el mismo archivo lo dibuja | B20 |
| `src-tauri/src/disparo/mod.rs:21-22` · `src-tauri/src/voz/mod.rs:6-7` | «ni una línea de LLM» · «será el disparador de la fase 4» | `sintesis/` y `disparo/` existen | B20 |
| `src-tauri/src/sintesis/mod.rs:67-69` | «la respuesta va en el idioma del cliente» | el prompt pide el de la ficha | B20 |

**Falsas por otra razón (no son aplazamientos)**

| Dónde | Frase | Por qué es falsa hoy | Hallazgo |
|---|---|---|---|
| `docs/MANUAL-DE-USO.md:311-312` · `src/componentes/Banda.tsx:863-866` | «Arrastrar el asa… apaga el modo» | no lo apaga | M14 |
| `docs/MANUAL-DE-USO.md:252` · `docs/MANUAL-DE-USO.md:264-268` | «tu pregunta» sale al modelo o al API | sale la frase del cliente | M13 |
| `docs/MANUAL-DE-USO.md:345-346` · `docs/MANUAL-DE-USO.md:365` | «ya sabe escribirlo bien» · «no toca lo que ya estaba bien» | lo corrompe | A3 |
| `docs/MANUAL-DE-USO.md:331` | «sin voz… el modo no se enciende» | se enciende | B14 |
| `docs/MANUAL-DE-USO.md:210-211` | «Sesión lo lista como Sábelo» (MDM) | solo con un invasivo | B15 |
| `src/pantallas/Permisos.tsx:139-141` | «Buscar tu evidencia a mano — Funciona» | necesita el audio del sistema | B19 |
| `src/i18n/es.ts:155` | «está en la lista de participantes» | coteja toda la ventana | B7 |
| `src/i18n/es.ts:92-94` | «O pasa a modo solo notas» | el modo no existe | M12 |
| `src-tauri/nativo/Llavero.swift:1-3` | «la lee solo en el instante de enviar» | se lee en cada estado de IA | B1 |
| `decisions/006-stt-local-y-su-modelo.md:85` | «14.2 para lo demás» | no arranca por debajo de 26 | M1 |
| `src-tauri/src/pantalla/mod.rs:5-6` | «el texto leído se pisa antes de soltarse» | las copias del refuerzo no | B6 |
| `src-tauri/src/lib.rs:1549-1550` | «hay una pregunta —la del usuario—» | no la hay | B5 |

## Casilla 4 — segunda pasada (después del último ajuste)

La hizo un auditor independiente el **2026-09-26**, en solo lectura, sobre `823c9f6` más la guía v4 sin
comitear (lo que luego fue `0b90589`). Buscó por vocabulario de promesa aplazada, y además siguió cada
ajuste de la Fase 2 hasta sus **frases hermanas**: el arreglo cambia el código y deja atrás, en otros
archivos, frases que decían lo contrario. Cotejó contra el código las cifras que citan los documentos.

- **441 líneas** con vocabulario de promesa (257 en documentos, maquetas, i18n y ADRs; 184 en
  comentarios) y **80** afirmaciones de exclusividad («solo», «único», «only»). 435 + 77 son ciertas
  hoy o historia fechada; las 9 restantes caen en B26, B30, B41 y B45.
- **25 hallazgos (B25–B49), todos bajos**, en la tabla de BAJOS. **Once los creó un ajuste de la
  Fase 2** (B25, B27, B28, B30, B31, B32, B36, B37, B38, B42, B49) y **dos la guía v4** (B33 y parte
  de B34): es exactamente el riesgo que la segunda pasada existe para cubrir.
- **B25 no era solo texto:** el log del corte leía el contador después de ponerlo a cero. Se pagó en
  el código, con un test de orden que se vio en rojo con el orden viejo.
- **Cifras cotejadas y correctas:** 9 piezas del corte (`src-tauri/src/corte.rs:72-83`), 7 atajos,
  30 s y 12 turnos, 4 s de silencio, techo de 6 s, USD 10, radar cada 10 s, 88/44 px, 82 procesos,
  macOS 26, las dos pistas en es-ES, 7 maniobras, las cuentas de la guía (72 · 31 · 13 · ⭐ 61 · 9
  paradas) y el WER sin cambios tras A3.
- **Comprobadas y ciertas, para no reabrirlas:** la MLX «Todavía no»; «Anotar llega con las notas
  (sprint 003)»; el «Pendiente, sprint 003» del ADR 011; «Lo que todavía no existe» de Idioma y de
  Corpus; «cuando exista la banda por monitor»; «Todavía no se ha visto capturar una ventana viva»
  (ADR 012); `Llavero.swift` y `Red.swift` como la única puerta del proveedor externo; los
  interruptores de IA que vuelven a apagado; el «9 de 9»; la tabla de atajos; B14 y M14 ya pagados;
  «Páramo Azul ya sabe escribirlo bien»; la tabla del WER.

## Casilla 5 — campos sin lector

**Resultado: 10 de 114 campos que cruzan Rust→TS no tienen lector** en `src/`, sin contar su declaración, el fixture ni los tests. El gate del constructor informa 1, porque no lee `radar.ts`, `ia.ts` ni `acople.ts` y su parser se corta en `& Turno)` (→ M6).

Todos los campos de los tipos nuevos o ampliados en el sprint tienen lector de su mismo tipo:
- `Sugerencia` (6), `FichaCitada`, `EstadoDeLaIa` (8), `EstadoDelApi` (3);
- `EnTuMac`, `Programa`, `CatalogoDelRadar`, `Bilingue` (este vía `[idioma]`);
- `EstadoDeLaPantalla`, `LaVoz`, `EstadoDelDiccionario`;
- `Novedad::Radar` y `Novedad::NadaEnPantalla`;
- `EstadoDePista` (`abierta`, `motivo`, `bytes`), `EstadoDelCorpus`, `Salida`.

**Los 10 campos sin lector**

| Tipo (Rust) | Campo | Lector | Hallazgo |
|---|---|---|---|
| `Novedad::Empieza` (`src-tauri/src/escucha/mod.rs:66`) | `pista` | ninguno | B11 |
| `Novedad::SinTexto` (`src-tauri/src/escucha/mod.rs:70`) | `pista`, `desdeMs`, `hastaMs`, `motivo` | ninguno | B11 |
| `Novedad::Ruido` (`src-tauri/src/escucha/mod.rs:72`) | `pista`, `duracionMs` | ninguno | B11 |
| `ficha::Aparicion` (`src-tauri/src/ficha/mod.rs:105`) | `hora` | solo en la maqueta (`src/componentes/Banda.tsx:666`) | B12 |
| `EstadoDelAcople` (`src-tauri/src/lib.rs:254`) | `permiso` | ninguno | B10 |
| `corte::Informe` (`src-tauri/src/corte.rs:124`) | `bytesEnRed` | ninguno (exento con «No se paga») | B9 |

**Relacionado, fuera de la cuenta:** `Novedad::Turno` manda `texto`, `hora`, `desdeMs` y `hastaMs` que el evento nadie lee: la banda los vuelve a pedir. Es texto del cliente cruzando dos veces (→ B11).

**Tipos que cruzan y no están en `contrato.rs`:** `QueSabeTranscribir` e `IdiomaDelMotor` (`src-tauri/src/lib.rs:655-697`). Su forma cambió este sprint (→ M7).

**Comandos registrados sin llamador:** los seis de B8.

**Riesgo latente, sin hallazgo porque hoy no rompe:** las variantes no muestreadas de varios enums (`Motivo`, `Categoria`, `PorQueNoAbrio`, `PorQueNoRedacta`, `PorQueNoHayMotor`…) no tienen fixture. Un cambio de `rename_all` en variantes de varias palabras no lo vería ningún gate. Conviene muestrearlas en `contrato.rs` al pagar M7.

## Lo que está bien

- `Sugerencia` con campos privados: la regla que no compila (E0451 demostrado).
- El techo de la síntesis en un hilo aparte, con test.
- `fiel.rs` es real y está cableado en `fundar` (`src-tauri/src/sintesis/mod.rs:222`).
- La sesión de red es `.ephemeral`, solo https, sin crate de red en Rust.
- El Llavero usa `WhenUnlockedThisDeviceOnly`.
- El vigía de pantalla es una máquina de estados sin reloj, probada entera, con máscara de vídeo y el cuadro en memoria de Rust.
- El radar coteja por ejecutable entero, tiene la regla 9 con lista blanca de libc y programas, y un kit con parecidos puestos a propósito.
- M10 se resuelve al abrir el grifo y no en el hilo de tiempo real.
- El silencio está cableado con su test de cable.
- La CSP es real y tiene su gate.
- El kit escribe si midió o no.
- Casi todos los gates nuevos traen su rojo en la bitácora, incluidos cuatro que nacieron sin poder fallar y se corrigieron.

## Orden propuesto para la Fase 2

1. **Antes de que nadie encienda el API:** A1, A2, M2, M10, B1, B3, M3.
2. **Lo que corrompe o impide el uso real:** A3, A4, M14, M4, M1.
3. **Lo que la app afirma y no es cierto:** M11, M13, M12, B19, B7, B14, B15, B16, B17, B18, B20, B4.
4. **Gates y contrato:** M6 (con su demo en rojo) → B9, B10, B11, B12 → M7 → M5, B8 → B13 → M8.
5. **Documentación y medición:** M9, M15, B21, B2, B5, B6.
6. **Casilla 4 otra vez** al terminar, como exige el kit v1.28.0.
