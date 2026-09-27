# ADR 013 — El radar, y lo que no hace: tu Mac y tu pantalla, y nada más

- **Fecha:** 2026-09-26
- **Sprint:** 002 «Cuándo, qué y quién mira», fase 4 (C14)
- **Estado:** aceptada
- **Fuentes:** el código (rutas de Rust relativas a `src-tauri/src/`) y la bitácora del sprint,
  `sprints/SPRINT_002-implementation-log.md`, citada como `bitácora:línea`.

## Contexto

El radar tiene que avisar al consultor de dos cosas que no se parecen: que **la reunión se está
grabando** —o que hay un bot de notas dentro— y que **un programa de su propio Mac lo está mirando**:
supervisión de exámenes, monitoreo de empleados, acceso remoto (`radar/mod.rs:1-13`).

Es el único módulo de la app cuyo trabajo es mirar «qué hay», y por eso es el que queda más cerca de
cruzar la **regla dura 9**: la app jamás sondea, escanea ni actúa sobre el computador de la
contraparte. Un `ping` para saber quién está en la red, un nombre que resolver, una conexión «solo
para comprobar» serían exactamente lo que el radar existe para señalar
(`tests/unit/radar-solo-este-mac.test.ts:8-12`).

Y la **regla 8** pesa en el diseño: el consultor tiene daltonismo leve, así que los dos avisos no
pueden distinguirse solo por el color.

## Decisión

### 1 · Dos mitades, dos niveles, y ninguno se distingue solo por el color

- **Ámbar, «sábelo»:** el aviso de grabación o de transcripción de Meet, Zoom o Teams, y los bots de
  notas. Se buscan en las líneas que la lectura de pantalla **ya sacó** de la ventana de la reunión:
  el radar no captura nada por su cuenta (`radar/avisos.rs:1-12`, `pantalla/mod.rs:325-332`; ADR 012).
- **Coral, «invasivo»:** programas de este Mac que miran la pantalla, la cámara, las teclas o los
  procesos, cotejados contra un catálogo versionado (`radar/procesos.rs:1-5`).
- **El MDM es «sábelo», no «invasivo»:** es lo normal en un equipo de empresa
  (`radar/mod.rs:44-55`).

Cada nivel lleva **símbolo, palabra y color**: el coral, la equis rellena; el ámbar, el punto de
grabación (`bitácora:1374-1376`). Lo validó quien lo necesita: «yo que tengo leve daltonismo los puedo
identificar rápidamente» (`bitácora:1480-1482`).

**El radar avisa, no bloquea**: no cierra ni frena ningún programa y no saca a nadie de la reunión; el
usuario decide (`radar/mod.rs:22-23`, `docs/MANUAL-DE-USO.md:213-214`). El cliente tiene derecho a
grabar su reunión y a traer su bot (`radar/avisos.rs:6-8`), y la banda añade lo único que es siempre
cierto: **«Ese bot no es Angel Ghost, que nunca entra a la llamada.»** No dice quién lo trajo, porque el
radar no puede saberlo (`bitácora:1500-1507`).

### 2 · El coral: la lista de procesos de este Mac, contra un catálogo con fuente

La lista se le pide al núcleo de **este** Mac con `proc_listallpids` y `proc_pidpath`, sin privilegios
—la misma que ve el Monitor de Actividad—, se coteja en memoria y se tira (`radar/procesos.rs:10-43`).

Se compara el **nombre entero del ejecutable**, sin mayúsculas, **jamás por trozos**: «Teams» no es
«TeamViewer», «AnyConnect» no es «AnyDesk», y «Screen Sharing» —la app con la que el consultor mira la
pantalla de otro— no es `screensharingd`, que es por donde otro mira la suya (`radar/procesos.rs:55-69`,
`radar/catalogo.rs:27-31`).

El catálogo trae **35 filas**: 5 de supervisión de exámenes, 7 de monitoreo de empleados, 19 de acceso
remoto, 3 agentes de MDM y la inscripción de macOS en un MDM (`bitácora:2069`). Cada fila lleva sus
ejecutables, **qué alcanza a ver** en es/en, **su fuente**, y una `nota` cuando algo se infirió en vez
de verificarse (`radar/catalogo.rs:21-38`). **Una fila sin fuente no entra**: lo exige un test
(`radar/catalogo.rs:112-134`), y otro impide que un mismo ejecutable delate a dos filas
(`radar/catalogo.rs:136-147`).

El coral corre **desde que arranca la app, no desde que empieza la sesión**: Sesión tiene que poder
decir que algo vigila el Mac **antes** de empezar, cuando el consultor todavía puede elegir «Iniciar de
todos modos» o «No iniciar» (`lib.rs:2054-2059`, `bitácora:2077`). Mira cada 10 s
(`lib.rs:2049-2052`) y **avisa solo cuando cambia lo que hay**; la primera vuelta avisa siempre, para
que nadie confunda «nada» con «todavía no» (`lib.rs:2058-2059`, `lib.rs:2075-2091`).

### 3 · El MDM se le pregunta a `/usr/bin/profiles`, una vez

Una inscripción en un MDM no siempre deja un proceso a la vista, así que se le pregunta a
`/usr/bin/profiles status -type enrollment`, que lee la configuración de este equipo y no habla con
nadie (`radar/mdm.rs:1-33`). Es **la única llamada del radar que lanza un programa**
(`radar/mod.rs:19-22`), y se hace una vez al arrancar (`lib.rs:2063`): cuesta 0,13 s
(`bitácora:2072`).

### 4 · El ámbar: frases y nombres del catálogo, sobre lo que Vision ya leyó

- Frases de aviso de Zoom, Meet y Teams, y **12 bots** por su nombre por defecto (`bitácora:2070`).
- Se comparan normalizadas, sin tildes ni mayúsculas y **por palabras enteras**: «otter ai» no puede
  aparecer dentro de «spotter aid» (`radar/avisos.rs:33-56`).
- Solo cuentan las líneas que Vision leyó con confianza suficiente, la misma vara que el refuerzo
  (`pantalla/mod.rs:325-332`).
- **Lo que sale son datos del catálogo** —si hay aviso, y el nombre del bot como lo escribe el
  catálogo—, **nunca el texto leído**; las copias normalizadas se pisan antes de soltarse
  (`radar/avisos.rs:10-12`, `radar/avisos.rs:72-73`).
- **Avisa una vez de lo mismo.** El aviso de grabación sigue en la esquina de Meet en cada
  diapositiva, y la banda no puede repetirlo en cada una: se avisa cuando cambia (un bot nuevo), una
  lectura que no lo ve no borra lo ya contado, y se olvida cuando la reunión deja de verse
  (`pantalla/mod.rs:525-536`, `pantalla/mod.rs:556-560`). Los bots van en el orden del catálogo para
  que dos lecturas de la misma reunión den el mismo aviso (`radar/avisos.rs:75-78`).

### 5 · El catálogo es JSON, y viaja dentro del binario

**Desviación declarada:** el plan decía `data/radar/*.yaml`. Es JSON porque `serde_json` ya está en el
crate y un YAML con filas anidadas pedía una dependencia nueva o un parser propio. El contenido es el
mismo y se versiona igual (`bitácora:2100-2103`).

Entra al binario con `include_str!` al compilar (`radar/catalogo.rs:1-12`): **el radar no lee un
archivo en marcha ni pregunta a nadie para ponerse al día**. Un catálogo nuevo es una versión nueva de
la app, con su diff y su revisión (`radar/mod.rs:15-17`).

### 6 · La regla dura 9, con su gate: todo lo que el radar llama está en una lista

El gate `tests/unit/radar-solo-este-mac.test.ts` invierte la pregunta: no «¿hay algo prohibido?», sino
**«¿todo lo que llama está en la lista?»** (`tests/unit/radar-solo-este-mac.test.ts:14-23`):

| Qué                     | Permitido                                                                                                                   | Test                                                                                   |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- |
| llamadas a `libc`       | `proc_listallpids`, `proc_pidpath` y sus tipos                                                                              | `tests/unit/radar-solo-este-mac.test.ts:30-38`                                         |
| programas que lanza     | solo `/usr/bin/profiles`, con la ruta entera                                                                                | `tests/unit/radar-solo-este-mac.test.ts:40-41`                                         |
| formas de salir del Mac | ninguna: ni `std::net`, ni sockets, ni resolver nombres, ni `extern "C"` propio, ni `#[link]`, ni `dlopen`, ni URLs, ni IPs | `tests/unit/radar-solo-este-mac.test.ts:43-53`                                         |
| ni disco ni red         | `radar` entre los módulos protegidos del efímero                                                                            | `tests/unit/radar-solo-este-mac.test.ts:115-118`, `scripts/verify-ephemeral.mjs:69-75` |

Se vio en rojo de cinco maneras: un `libc::socket`, un `Command::new("ping")`, un `extern "C"` propio,
una URL con una IP, y el radar fuera de los protegidos (`bitácora:2142`). **Su primera versión no podía
fallar por las URLs** —quitaba todo lo que viniera detrás de `//` y se comía el esquema de toda URL—; corregida,
cazó una URL de ejemplo en `mdm.rs` (`bitácora:2142`,
`tests/unit/radar-solo-este-mac.test.ts:55-65`).

### 7 · Las frases de Meet y Teams son probables hasta verlas en vivo

Google y Microsoft **describen** el aviso de grabación pero no lo citan, y el de Teams lo puede
reescribir el administrador (`bitácora:2110-2113`). El catálogo lo dice fila por fila: Meet, «probables
todas» (`data/radar/avisos.json:26`); Teams, «los textos por defecto son probables»
(`data/radar/avisos.json:37`); Zoom, el inglés verificado en una fuente secundaria y el español
probable (`data/radar/avisos.json:13`). Las confirma la prueba ⭐ «radar ámbar con el aviso real» del
gate del MVP (`data/radar/avisos.json:2`).

### 8 · Lo que el radar NO ve, declarado

- **Un programa renombrado**, o uno que no está en el catálogo: se reconoce por el nombre de su
  ejecutable (`docs/MANUAL-DE-USO.md:226-228`).
- **Las extensiones de navegador** (Proctorio, Honorlock): no tienen proceso propio
  (`bitácora:2107-2109`, `data/radar/programas.json:2`).
- **Los nombres demasiado genéricos para cotejarlos solos**: el `service` de RustDesk, el `go-agent`
  de Addigy, el `hubd` de Workspace ONE (`bitácora:2107-2109`).
- **Los controles de examen que viven en el núcleo:** la clase existe y no tiene filas. En macOS Apple
  desaconseja las extensiones de núcleo y en Apple silicon exigen bajar la seguridad al arrancar
  (`bitácora:2104-2106`, `data/radar/programas.json:2`).
- **Si alguien está conectado ahora.** Un acceso remoto abierto significa que alguien **podría**, no que
  lo esté: saberlo exigiría mirar fuera de este Mac (`docs/MANUAL-DE-USO.md:229-230`).
- **El ámbar sin lectura de pantalla:** si el consultor apaga «Leerla sola», el ámbar tampoco mira; el
  coral no depende de eso (`docs/MANUAL-DE-USO.md:224-225`).

## Alternativas consideradas

- **Cotejar por trozos del nombre:** «Teams» acabaría pareciéndose a «TeamViewer». Se vio en rojo
  cotejando con `contains` y metiendo «MSTeams» en el kit (`bitácora:2136`, `bitácora:2140`).
- **YAML, como decía el plan:** una dependencia nueva o un parser propio para el mismo contenido
  (`bitácora:2100-2103`).
- **Actualizar el catálogo por la red:** sería la única puerta de red del radar, y la regla 9 la cierra.
  Versionado dentro del binario, se revisa como el código (`radar/mod.rs:15-17`).
- **Cerrar o bloquear el programa invasivo:** la app no actúa sobre nada ni sobre nadie; avisa y el
  usuario decide (`radar/mod.rs:22-23`).
- **Una captura propia para el ámbar:** duplicaría la lectura de pantalla y su superficie. Mira las
  mismas líneas, antes de que se pisen (`pantalla/mod.rs:325-332`).
- **Mirar la red para saber si alguien está conectado por acceso remoto:** sería sondear otras
  máquinas. Se renuncia a saberlo y se dice (`docs/MANUAL-DE-USO.md:229-230`).

## Consecuencias

- **El consultor sabe qué vio el radar y de dónde lo sacó**: «leído de tu pantalla · 14:03»
  (`docs/MANUAL-DE-USO.md:191-194`). Con la sesión en marcha, el coral salta en la banda y `⌃⌥R` («qué
  ve») abre Sesión con la tabla de lo que alcanza a ver cada programa (`lib.rs:2109-2114`,
  `docs/MANUAL-DE-USO.md:205-207`).
- **`⌃⌥R` se registra al arrancar** (`lib.rs:2207`). La propuesta de cogerla solo con el coral en
  pantalla (`bitácora:1530-1532`) se escribió cuando la tecla era `⌘⇧R`, la recarga forzada de los
  navegadores; con el paso de todas las teclas a `⌃⌥` (`bitácora:1837-1856`) ese choque desaparece. Los
  de `⌃⌥` que quedan —VoiceOver y las apps que ordenan ventanas— están en el manual
  (`bitácora:1852-1854`).
- **Lo que la auditoría registró sobre esta pieza, ya pagado:** el copy del bot dice «aparece en la
  ventana de la reunión», porque el ámbar coteja cualquier línea (B7); un Mac con MDM y sin invasivo
  lo ve en Sesión como fila «Sábelo» (B15); el gate del contador ve `Command::new` fuera de `radar/`
  (B13). **La cifra, corregida:** el Mac limpio del kit tiene **82** rutas de proceso
  (`radar/procesos.rs:108-115`); el manual y el summary ya lo dicen, y la bitácora lleva su nota de
  corrección (`bitácora:2483`, B22). El resultado —cero falsos positivos— no cambia.

## Medición

- **Kit coral** (`radar/procesos.rs:117-139`): **0 falsos positivos** en el Mac limpio —82 procesos
  en el archivo del kit; la bitácora dice 100, ver Consecuencias— y **34 de 34** filas por proceso del
  catálogo encontradas en el Mac vigilado (`bitácora:2083-2084`).
- **Kit ámbar, con el OCR de verdad:** la reunión grabada da el aviso y el bot «Otter.ai»; las **cinco**
  reuniones sin grabar del kit de pantalla dan **0** avisos (`bitácora:2085-2086`).
- **En vivo, en este Mac** (`pnpm tauri dev`): «catálogo v1 · MDM: no» y «1 invasivos» antes de lanzar
  nada. **No es un falso positivo:** corría el anfitrión de **Chrome Remote Desktop**, es decir, el
  acceso remoto a este equipo estaba activado. Con un programa propio llamado `TeamViewer` corriendo,
  el radar pasó a «2 invasivos» en menos de diez segundos, `⌃⌥R` respondió, y al cerrarlo volvió a «1»
  (`bitácora:2087-2093`). Las ventanas de la app están protegidas de la captura, así que lo visto fue el
  log; la banda y Sesión con esos datos las cubren los tests que cruzan la costura.
- **Fidelidad:** 100 encuadres sin ninguno sobre el umbral; **axe** sin hallazgos en ámbar, coral y
  «vigilancia», en los dos temas (`bitácora:2094-2096`).
- Los rojos de cada gate y test nuevo, en `bitácora:2131-2148`.
