# ADR 014 — La voz al oído: el modo solo audio, su candado y cuándo se calla

- **Fecha:** 2026-09-26
- **Sprint:** 002 «Cuándo, qué y quién mira», fase 2 (C15)
- **Estado:** aceptada
- **Fuentes:** el código (rutas de Rust relativas a `src-tauri/src/`; las de Swift, a `src-tauri/`)
  y la bitácora del sprint, `sprints/SPRINT_002-implementation-log.md`, citada como `bitácora:línea`.

## Contexto

El modo solo audio no estaba en la VISION: nació en la mirada 3 de la Etapa de Diseño, con las
palabras del usuario —_«quisiera tener un modo solo audio que me hable de forma paralela por si
quiero ver completamente la pantalla y no me interrumpa»_—. Las dos mitades de la frase son los dos
requisitos: **hablar** y **devolver la pantalla** (`habla/mod.rs:8-13`).

Es la única pieza de la app que **se oye desde el otro lado de la llamada** (`corte.rs:55-57`). Si
suena por los altavoces del Mac, el cliente la oye y el micrófono la recoge. Y macOS no ayuda: desde
Core Audio, un dispositivo por USB o Bluetooth puede ser un casco o un altavoz de mesa, y **no se
distinguen** (`capture/nativo.rs:190-192`).

## Decisión

### 1 · `AVSpeechSynthesizer` por el puente de Swift, en un módulo que se llama `habla/`

La voz del sistema, por cuatro funciones de C que expone `nativo/Habla.swift` —¿hay voz?, di,
cállate, ¿estás hablando?— y todo el `unsafe` en una sola puerta (`habla/apple.rs:1-26`,
`bitácora:1004-1012`). Tres decisiones de forma del puente (`nativo/Habla.swift:14-29`):

- **Un solo sintetizador, vivo lo que viva la app.** Uno por frase se libera antes de que el sistema
  termine y la voz se corta a mitad (`nativo/Habla.swift:85-91`).
- **Hablar no bloquea**: encola y vuelve; quien quiera saber si terminó, pregunta
  (`nativo/Habla.swift:117-144`).
- **«¿Está hablando?» es una bandera que se pone al encolar**, no solo en el delegado: en el hueco de
  milisegundos entre encolar y que el sistema avise sonarían dos voces a la vez
  (`nativo/Habla.swift:135-139`).

La voz se busca **sin aceptar el respaldo que inventa el sistema**: la del idioma exacto o, si no, la
de la misma lengua («es» acepta es-ES y es-MX); si no hay ninguna, no hay voz, y no se lee una ficha en
español con una voz inglesa (`nativo/Habla.swift:93-107`). Se lee en **el idioma del consultor**,
porque la ficha sale de sus documentos (`lib.rs:1151-1154`). Sin puente de Swift —en la CI o en una
compilación sin herramientas— la voz es **la muda**, de primera clase, con su motivo
(`habla/mod.rs:44-75`, `habla/apple.rs:9-11`).

**El módulo se llama `habla/`, no `voz/` como decía el plan:** `voz/` ya es la voz que ENTRA —VAD,
turnos, eco—, y dos módulos con el mismo nombre para las dos direcciones del sonido es la clase de
confusión que se descubre tarde (`habla/mod.rs:3-6`, `bitácora:998-1000`). **Y es módulo protegido**
aunque lo que dice sea del usuario: `AVSpeechSynthesizer` trae `write(_:toBufferCallback:)`, que
convierte la ficha en búferes de audio, y un archivo con la evidencia del consultor leída en voz alta
sería una grabación de la reunión con otro nombre (`habla/mod.rs:15-18`,
`scripts/verify-ephemeral.mjs:55-61`, `bitácora:1057-1063`).

**La voz no decide cuándo hablar.** Lo decide `cabe_decirla`, una función pura sobre cinco booleanos
(`habla/mod.rs:20-24`, `habla/mod.rs:135-201`), y hay un solo camino por el que la app habla,
`decir_la_ficha`, que usan los dos disparadores (`lib.rs:1190-1250`).

### 2 · El candado de los auriculares, y lo que macOS no deja saber

`Salida::puede_haber_eco()` tiene **tres** respuestas: `Some(true)` con los altavoces internos y
con HDMI, DisplayPort o AirPlay (`Salida::AltavozExterno`, auditoría M4), `Some(false)` con
auriculares por el conector del Mac, y `None` con un dispositivo por USB o Bluetooth o cuando no se
sabe (`capture/nativo.rs:188-246`). Los altavoces internos
se reconocen por su transporte y su fuente de datos, comprobados en este Mac
(`capture/nativo.rs:59-65`).

- **Con los altavoces internos, o por HDMI, DisplayPort o AirPlay, calla.** Son los casos en que la
  app **sabe** —o da por hecho, por el transporte— que el cliente oiría
  (`habla/mod.rs:187-190`), y la banda lo dice en ámbar: «Conecta auriculares · el cliente te oiría»
  (`docs/MANUAL-DE-USO.md:315-316`).
- **Con un dispositivo externo por USB o Bluetooth —unos AirPods incluidos— habla.** Negarse dejaría
  C15 inservible para el caso normal; hablar acepta que algún día suene por un altavoz de mesa. Se
  habla por tres razones, en orden de peso: la app ya trazó esa línea en el aviso de eco de Sesión, que
  el usuario aprobó en la mirada 13; un dispositivo externo en una videollamada es, casi siempre, lo
  que el consultor se pone para no oírse; y el modo lo enciende el usuario con una tecla, sabiendo por
  dónde le suena el Mac (`habla/mod.rs:156-182`, `bitácora:1020-1042`).
- **Declarado, no escondido:** el manual lo dice como limitación —«si tu salida es un altavoz externo,
  no enciendas el modo»— (`docs/MANUAL-DE-USO.md:321-326`), y Sesión enseña el nombre del dispositivo
  con «Si es un altavoz, no uses el modo solo audio.» (`bitácora:1786-1789`). Cambiar la decisión es
  una línea de `cabe_decirla` (`habla/mod.rs:182`).

La banda pinta **la capacidad, no el instante**: que alguien hable no la pone en «Conecta
auriculares», o parpadearía toda la reunión (`habla/mod.rs:203-209`). La otra mitad de «no oírse a sí
misma» la puso el sprint 001: el tap del sistema excluye el propio proceso, así que la pista del
cliente no trae la voz de la app aunque suene (`capture/nativo.rs:611`, `bitácora:1163-1165`).

### 3 · Nunca habla encima de nadie

Cinco razones para callarse, **en orden de gravedad** —la primera que falla es la que se registra—:
modo apagado, el cliente la oiría, no hay voz para el idioma, **alguien está hablando en la reunión**
(cualquiera de las dos pistas) y **ya está diciendo otra ficha** (`habla/mod.rs:103-119`,
`habla/mod.rs:150-201`). No encola una segunda ficha encima de la que dice (`habla/mod.rs:116-118`).

Si el candado de la escucha está ocupado cuando llega una ficha, **se supone que sí hay alguien
hablando**, que es el lado que calla: es la salida a un abrazo mortal real entre la escucha y la voz
(`lib.rs:1209-1224`).

### 4 · Lee titular, línea y fuente — y no lee las fichas «sin resultado»

Lo mismo que pinta la banda y en el mismo orden, para que la voz y la pantalla no digan cosas
distintas; **la fuente, al final y siempre**, porque una frase sin su origen no es evidencia
(`habla/mod.rs:211-231`). La fuente se dice con palabras, sin el `·` ni el `§` de la banda
(`lib.rs:1252-1265`).

**Las fichas «sin resultado» no se leen.** Su titular son **las palabras del cliente** («nada sobre
"certificación ISO 27001"»): leérselas al usuario sería sacar el transcript del cliente por el
altavoz, por un camino que ninguna capa vigilaba. La banda sí las pinta (`lib.rs:1200-1205`,
`bitácora:1044-1048`).

### 5 · `⌃⌥V` enciende y apaga el modo, y la banda baja a 44 px

`⌃⌥V` conmuta el modo (`lib.rs:1267-1332`, `lib.rs:1372-1379`). No es la `⌃⌥A` que pedía la orden:
esa tecla es «ayúdame con esto» desde el sprint 001, y el panel aprobado en la Etapa de Diseño ya
usaba la V (`lib.rs:1273-1276`, `bitácora:838-841`). Nació `⌘⇧V` y pasó a `⌃⌥V` con todas las teclas,
por los choques con Zoom (`bitácora:1837-1840`).

**El alto de la banda es el modo:** 44 px en vez de 88 (`ventana/mod.rs:32-34`), y se asienta con el
reacople para que la ventana de la reunión **gane** esos 44 px; sin esa segunda mitad el modo bajaría
la banda y el usuario no ganaría un píxel (`lib.rs:1302-1315`). Al encender lee la ficha vigente; si
todavía no se ha oído al cliente, queda a la espera (`lib.rs:1322-1327`). El modo dura la reunión
entera: bajarlo y subirlo por cada ficha obligaría a rehacer el acople en cada transición
(`bitácora:1083-1086`).

La banda de 44 px tiene tres estados —«Diciéndote la ficha…», «Callado · esperando el siguiente
turno» y «Conecta auriculares · el cliente te oiría»— (`bitácora:1326-1334`). El callado no estaba en
la mirada 16 y se aprobó en la 16-bis con un cambio del usuario: tiene que decir que lo está
(`bitácora:1311-1324`).
Rust manda **tres booleanos y ninguna frase**: el copy vive en `src/i18n/`, contra la maqueta
(`habla/mod.rs:77-96`).

### 6 · `⎋` calla, y solo se coge mientras el modo está encendido

`⎋` corta en el acto, sin esperar al final de la palabra (`nativo/Habla.swift:146-157`). Pero un
Escape global permanente se lo quitaría a la reunión —en Meet es la tecla que sale de pantalla
completa— y a todas las apps del Mac, para una función que existe unos segundos por ficha. **Se coge
al encender y se devuelve al apagar**, y las dos cosas se escriben en el log (`lib.rs:1317-1320`,
`lib.rs:1401-1431`,
`bitácora:1050-1055`, `docs/MANUAL-DE-USO.md:327-329`).

Cogerla y soltarla ocurre **en otro hilo**. Registrar un atajo desde dentro del manejador de atajos
bloquea el plugin: la app sigue viva y **ningún atajo vuelve a funcionar, `⌥⎋` incluido**. Se
encontró corriendo la app, con todos los tests en verde (`lib.rs:1387-1431`, `bitácora:1185-1209`).

### 7 · El kill-switch la calla primero

`Pieza::Voz` va **la primera** de la lista del corte. El orden de las piezas se decide por lo que
sigue entrando; la voz es la excepción, y su puesto se decide por lo que el cliente **percibe**: una
frase a medio decir delante de él no la arregla ningún informe (`corte.rs:52-58`, `corte.rs:71-74`,
`bitácora:1065-1075`). El corte calla la voz, apaga el modo y devuelve `⎋` (`lib.rs:782-793`). No hubo
que acordarse de añadirla: `Pieza::orden` es un `match` sin comodín y el crate no compilaba sin ella
(`corte.rs:87-106`, `bitácora:1073-1075`).

## Alternativas consideradas

- **Negarse a hablar con cualquier dispositivo externo:** C15 inservible para el caso normal, y una
  funcionalidad que nunca corre es peor que una limitación declarada (`habla/mod.rs:164-166`,
  `bitácora:1028-1030`).
- **Llamar al módulo `voz/`, como el plan:** dos módulos con el mismo nombre para las dos direcciones
  del sonido (`bitácora:998-1000`).
- **Un `⎋` global y permanente:** se lo quitaría a Meet y a todas las apps del Mac
  (`bitácora:1050-1055`).
- **`⌃⌥A` para el modo, como la orden:** ya era «ayúdame con esto» (`lib.rs:1273-1276`).
- **Bajar y subir la banda por cada ficha:** un reacople por transición, varias idas y vueltas a otro
  proceso por la Accessibility API (`bitácora:1083-1086`).
- **Leer también las fichas «sin resultado»:** sacaría las palabras del cliente por el altavoz
  (`lib.rs:1200-1205`).
- **Que Rust mande las frases a la banda:** el copy se podría cambiar en Rust sin que ninguna mirada
  lo viera (`habla/mod.rs:79-81`).
- **Pedir la voz con `AVSpeechSynthesisVoice(language:)` a ciegas:** puede devolver la voz por
  defecto del Mac y leer la ficha en inglés creyendo que es español (`nativo/Habla.swift:95-100`).

## Consecuencias

- **Con un dispositivo por USB o Bluetooth la app habla y sí podría oírse a sí misma por el
  micrófono**: el tap
  la excluye, el micrófono no. Esa parada es del gate ⭐, con auriculares puestos y sin ellos
  (`bitácora:1167-1169`, `src-tauri/tests/contra-el-mac-de-verdad.rs:1312-1316`).
- **La CI sí mide la voz.** `macos-latest` no trae modelos para reconocer, pero sí voces para
  sintetizar: el puente de `habla/` y el candado se miden en cada PR (`bitácora:1287-1309`).
- **Lo que la auditoría registró sobre esta pieza, ya pagado:** sin voz para el idioma, `⌃⌥V` no
  enciende el modo y lo dice en el registro (B14); el arranque de la voz se mide en vivo —`[habla]
  empezó a sonar a los N ms (presupuesto 1000)`— y el número sale en la parada h1 del gate del MVP
  (B21).

## Medición

- **En este Mac** (`bitácora:1171-1183`): voz «apple-avspeechsynthesizer» con es-ES y en-US; salida
  «Altavoces», `¿eco? Some(true)`; el puente encoló y calló; y el candado, contra esa salida, **se
  calla**: «el sonido saldría por los altavoces y el cliente te oiría — MEDIDO».
- **En vivo, en el modo** (`bitácora:1211-1228`): el modo enciende y apaga, la banda pasa de 88 a 44
  px y vuelve, `⎋` se coge y se devuelve, y **`⌥⎋` calla la voz antes que nada y apaga el modo**. Con
  las teclas ya en `⌃⌥`, la corrida en vivo de la fase 3 repitió el encendido y el apagado
  (`bitácora:1998-1999`).
- **Los rojos** (`bitácora:1097-1155`): `habla/` escribiendo en disco, la costura de `LaVoz` por los
  dos lados, y el candado abierto —tres tests en rojo, uno contra el Mac de verdad—. Y el cuarto, el de
  fidelidad de la banda de 44 px (`bitácora:1272-1274`).

## Dos decisiones de la auditoría, ya implementadas

La auditoría del sprint encontró dos huecos en esta pieza. Este ADR registra lo que se decidió; los
dos se pagaron en la Fase 2 de la auditoría, dentro del sprint 002 (`7a50222`), por decisión del
usuario de pagar los cuarenta hallazgos (`bitácora:2361`).

1. **HDMI, DisplayPort y AirPlay cuentan como altavoz** (auditoría M4,
   `sprints/SPRINT_002-auditoria.md:247-261`). La auditoría encontró que todo transporte que no sea el
   interno cae en `Salida::Otra` y la app habla (`capture/nativo.rs:217-219`, tal como estaba al
   escribir este ADR). La desviación declarada arriba —hablar con USB y Bluetooth porque no se
   distinguen— **no alcanza** a transportes que sí se distinguen y que son casi siempre altavoces: el monitor por HDMI o DisplayPort, y AirPlay. **Decisión:** esos tres
   transportes son un altavoz externo, `puede_haber_eco()` responde `Some(true)` y `cabe_decirla` se
   calla con el mismo motivo que con los altavoces internos. USB y Bluetooth siguen como están, con su
   limitación declarada.
2. **Arrastrar el asa fuera de los 44 px apaga el modo** (auditoría M14,
   `sprints/SPRINT_002-auditoria.md:427-439`). El manual lo promete —«arrastrar el asa hacia arriba
   hace lo mismo: el alto _es_ el modo» (`docs/MANUAL-DE-USO.md:312-313`)—, y la auditoría encontró
   que no pasaba: el asa cambiaba el alto de la ventana sin tocar el modo, así que la app seguía
   hablando y quedándose con el `⎋` de la reunión. **Decisión:** apagar el modo por el asa es el mismo
   apagado que el de `⌃⌥V` —soltar `⎋`, callar la voz, avisar a la banda—, en un solo sitio que usan
   los dos caminos.
