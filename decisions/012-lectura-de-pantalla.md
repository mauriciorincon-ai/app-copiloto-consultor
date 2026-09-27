# ADR 012 — Leer la pantalla: una ventana, solo lo nuevo, y un cuadro que muere con el corte

- **Fecha:** 2026-09-26
- **Sprint:** 002 «Cuándo, qué y quién mira», fase 3 (C8)
- **Estado:** aceptada
- **Fuentes:** el código (rutas de Rust relativas a `src-tauri/src/`; las de Swift, a `src-tauri/`)
  y la bitácora del sprint, `sprints/SPRINT_002-implementation-log.md`, citada como `bitácora:línea`.

## Contexto

La orden del sprint pide que la app lea la pantalla que el cliente comparte —la diapositiva con una
cifra, el cronograma— y que eso ayude a traer la ficha justa. Tres fuerzas tiran en contra:

- **La regla dura 1 pone lo leído de la pantalla del lado que muere siempre**, sin conmutador que lo
  encienda. Un cuadro de la reunión en un archivo sería una grabación de la pantalla del cliente
  (`nativo/Pantalla.swift:3-9`).
- **El Mac ya está transcribiendo dos pistas.** Leer cada cuadro con Vision sería quemar el portátil
  para leer cien veces la misma diapositiva (`pantalla/huella.rs:3-6`); la orden fija el techo en
  **una lectura por segundo** (`pantalla/mod.rs:71-72`).
- **Una videollamada nunca entrega dos cuadros iguales**: el grano del vídeo, el cursor, el reloj de
  la llamada y las cámaras de los participantes se mueven todo el rato (`pantalla/mod.rs:18-19`,
  `pantalla/mod.rs:65-68`).

Y una cuarta que no se sabía hasta medir: **la pantalla ayuda a la pregunta que habla de lo que se
ve, y estorba a todas las demás** (`bitácora:1649-1668`).

## Decisión

### 1 · Solo la ventana de la reunión, con ScreenCaptureKit

A ScreenCaptureKit se le pide **una ventana**: la más grande de la app de videollamada que Sesión
detectó y, si es un navegador, la que lleva la señal en el título («google meet»); visible, de al
menos 320 × 200, sin cursor y sin audio (`nativo/Pantalla.swift:90-122`). El correo, los documentos
del consultor y la propia banda —que además está protegida de la captura— quedan fuera **por
construcción, no por filtro** (`pantalla/mod.rs:37-40`).

El cuadro se pide ya escalado a 1600 px de lado como máximo y se pinta en grises **directamente en
el búfer que Rust reservó una vez** (`nativo/Pantalla.swift:109-113`, `nativo/Pantalla.swift:134-140`,
`pantalla/mod.rs:74-95`): Swift no se queda con la imagen y el kill-switch tiene un único sitio que
pisar. ScreenCaptureKit no promete contestar, así que la espera tiene un techo de 3 s
(`nativo/Pantalla.swift:52-64`).

### 2 · Se lee cuando hay algo nuevo: la huella por zonas

Cada 500 ms se captura (`pantalla/mod.rs:50-51`) y una **huella** decide si hay algo que leer: una
miniatura de 32 × 32 grises por cada zona de una cuadrícula de 4 × 4, y cuántas celdas cambiaron más
de 12 niveles de gris (`pantalla/huella.rs:19-56`, `pantalla/huella.rs:93-104`). Cambio = más de 12
celdas en alguna zona; quieta = 6 o menos (`pantalla/mod.rs:53-63`). Cuesta ~2 ms
(`bitácora:1646-1647`).

Sobre ella, **el vigía** (`pantalla/mod.rs:195-248`):

- espera a que la pantalla **se quede quieta** antes de leer: leer una transición es leer un cuadro
  que ya no existe cuando Vision termina (`pantalla/mod.rs:231-236`);
- deja de mirar la zona que lleva cuatro cuadros seguidos cambiando, **que es vídeo**, hasta que se
  quede quieta (`pantalla/mod.rs:65-69`, `pantalla/mod.rs:221-223`);
- no deja pasar **más de una lectura por segundo** (`pantalla/mod.rs:237-242`).

**No es un pHash, aunque el plan lo pedía.** Desviación declarada con su medida (`bitácora:1633-1647`);
las dos versiones de pHash que se probaron están en Alternativas.

### 3 · Vision lee solo lo que la huella deja pasar, y de eso se queda con el tema

`VNRecognizeTextRequest`, nivel `accurate`, en es-ES y en-US (`nativo/Pantalla.swift:172-175`). De todo
lo leído, **el refuerzo** se queda solo con lo que habla del tema: los títulos (las líneas claramente
más altas), las cifras en líneas cortas —las horas no cuentan: son el reloj de la llamada— y los
términos distintivos del corpus del consultor. Lo que Vision lee con confianza menor de 0,5 se tira, y
hay techos de 3 títulos, 5 cifras y 8 términos (`pantalla/refuerzo.rs:1-49`). Las líneas leídas se
pisan con ceros antes de soltarse (`pantalla/mod.rs:334-338`, `pantalla/apple.rs:132-134`).

### 4 · La pantalla solo desempata la búsqueda — salvo cuando es ella la que pide la ficha

Cuando el cliente pregunta, **la pantalla no abre una búsqueda propia**. Se busca la pregunta sola; si
su primera sección le saca al menos un 10 % a la segunda, esa es la respuesta y la pantalla no toca
nada. Si no, se repite la misma consulta con **la pregunta como cláusula obligatoria** y lo leído como
opcional, con peso 0,5 (`corpus/indice.rs:42-69`, `corpus/indice.rs:250-283`,
`corpus/indice.rs:297-316`). Así la pantalla solo reordena lo que la pregunta ya encontró. Sin lectura
—apagada, sin permiso, sin reunión— la búsqueda es exactamente la de siempre (`lib.rs:394-403`).

**La excepción es la ficha que pide la pantalla sola.** Si una lectura nueva trae una cifra o un
término del consultor (`pantalla/refuerzo.rs:75-80`), pide ficha por el **mismo disparador** que un
turno —seis segundos entre fichas y ninguna repetida (`disparo/mod.rs:29`, `disparo/mod.rs:140-170`)—,
se busca con lo leído como única consulta (`corpus/indice.rs:294-296`) y **solo se enseña si hay
ficha**: una pantalla que no encuentra nada no interrumpe a nadie para decir «no tengo nada», porque
nadie preguntó (`escucha/mod.rs:382-398`, `lib.rs:1518-1560`).

### 5 · En memoria vive el último cuadro, y es una pieza del kill-switch

Vive **el último cuadro** y el texto que se sacó de él, nada más. Honestidad enseña cuánto ocupan; el
búfer reservado y vacío no cuenta, porque no guarda nada de nadie (`pantalla/mod.rs:382-395`,
`pantalla/mod.rs:640-656`). El cuadro se pisa con ceros al soltarse, y también al encogerse, para que
una captura pequeña no deje viva la cola de una grande (`pantalla/mod.rs:102-128`).

El kill-switch tiene su pieza, `Pieza::UltimoFrame` (`corte.rs:46-48`, `corte.rs:65-77`): para el vigía
y pisa el cuadro y el texto (`lib.rs:788-793`). **No espera al hilo**, a propósito: si el vigía está a
mitad de una captura, el cuadro lo pisa él al terminar esa vuelta, y `⌥⎋` no se congela hasta 3 s
justo cuando el usuario pulsa la tecla de emergencia (`pantalla/mod.rs:613-631`). Con esta pieza el
corte pasa a **8 de 8** (`corte.rs:135-151`, `bitácora:1627`).

El módulo es **protegido**: `src-tauri/src/pantalla` está en la lista del efímero
(`scripts/verify-ephemeral.mjs:63-68`), que además prohíbe por su nombre la API de Swift que grabaría
la ventana a vídeo (su rojo: `bitácora:1706-1709`).

### 6 · El permiso se pregunta sin provocar el diálogo, y es uno aparte

`CGPreflightScreenCaptureAccess` se consulta **antes** de tocar ScreenCaptureKit, que provocaría el
diálogo de macOS si el permiso no está decidido: esta app no pide nada por sorpresa
(`nativo/Pantalla.swift:18-21`, `nativo/Pantalla.swift:85`). Sin permiso, Sesión dice «Sin permiso» y
la app funciona sin leer (`pantalla/mod.rs:352-366`, `docs/MANUAL-DE-USO.md:176-177`).

Dos cosas se comprobaron en este Mac y cambiaron el plan (`bitácora:1583-1604`):

- **`NSScreenCaptureUsageDescription` no existe.** El diálogo de pantalla de macOS es siempre el
  genérico, así que la descripción es/en que pedía el plan no se puede poner. Lo que se lee y lo que
  muere lo explica la pantalla de Permisos, **antes** de que macOS pregunte.
- **Pantalla y audio del sistema son dos permisos de TCC**, no uno. La frase del sprint 001 que decía
  lo contrario era falsa y salió del producto (`bitácora:1591-1598`, `bitácora:1905`).

### 7 · Bajo demanda, sin región: `⌃⌥L` lee la ventana entera una vez

El plan pedía lectura **bajo demanda por región**. Una región exigía una cuarta ventana, y la app
tiene tres (`ventana/mod.rs:1-7`). **El usuario eligió el atajo sin región** (`bitácora:1571`,
`bitácora:1605-1606`): `⌃⌥L` lee **una vez** la ventana de la reunión, ahora, aunque «Leerla sola» esté
apagada (`pantalla/mod.rs:599-602`, `lib.rs:1607-1633`). Si no hay texto, la banda lo dice —«Leí la
pantalla: no hay texto que buscar.»— en vez de callar (`lib.rs:1554-1558`).

Es la salida para una NDA estricta: apagar la lectura automática y leer solo cuando el consultor lo
pide (`lib.rs:1613-1615`, `docs/MANUAL-DE-USO.md:164-168`). Apagada, el hilo **no captura ni un
cuadro** y olvida lo que había leído (`pantalla/mod.rs:454-495`). Nace encendida (`lib.rs:1421-1427`).

La tecla se dibujó como `⌘⇧L` y pasó a `⌃⌥` con todas las demás, porque las de `⌘⇧` chocaban con
Zoom (`bitácora:1837-1856`).

## Alternativas consideradas

- **Un pHash de la ventana entera** (lo que decía el plan): dos diapositivas distintas dentro de la
  misma ventana de Meet solo diferían en 8 bits de 64, por debajo del umbral. La composición de una
  videollamada es casi toda interfaz, y el pHash tira el detalle, que es justo el texto que cambia
  (`bitácora:1635-1638`, `pantalla/huella.rs:37-41`).
- **Un pHash por zona:** veía el texto nuevo, pero un cursor en una zona casi lisa movía 34 bits de 64
  (`bitácora:1639-1641`, `pantalla/huella.rs:42-45`).
- **La pantalla siempre dentro de la consulta:** con peso 0,5, el kit v0 con la peor pantalla delante
  caía a 0,764; con peso 1,5, a 0,721. Los dos bajo el mínimo de 0,80 (`corpus/indice.rs:51-63`,
  `bitácora:1655-1668`).
- **Pregunta y pantalla como cláusulas opcionales las dos:** «¿la tarifa es cerrada?» con una
  diapositiva de adopción de datos delante devolvía el marco de adopción. Lo tumbó su propio test
  (`corpus/indice.rs:297-302`).
- **Sin máscara de vídeo:** con las cámaras encendidas la zona del vídeo nunca se queda quieta y la
  pantalla no se leería nunca (rojo 4, `bitácora:1716-1717`, `bitácora:1732-1734`). La máscara no era
  una optimización: sin ella la lectura no funciona en una reunión real.
- **Leer por región, con una ventana para elegirla:** rompía el invariante de tres ventanas; descartada
  por el usuario (`bitácora:1571`).
- **Capturar la pantalla entera y filtrar después:** habría puesto en memoria el correo y los
  documentos del consultor para luego tirarlos. Pedir la ventana sola los deja fuera sin tocarlos
  (`nativo/Pantalla.swift:23-25`).

## Consecuencias

- La lectura **solo existe con la ventana de la reunión visible**: detrás de otra ventana, minimizada
  o en otro escritorio, no hay nada que leer, y Sesión lo dice (`pantalla/mod.rs:146-147`,
  `pantalla/mod.rs:368-379`, `docs/MANUAL-DE-USO.md:178-179`).
- **El radar ámbar (ADR 013) no captura por su cuenta**: mira las mismas líneas que Vision ya leyó,
  antes de que se pisen (`pantalla/mod.rs:325-332`). Apagar «Leerla sola» apaga también el ámbar.
- **Qué mide la CI y qué no.** Vision viene con macOS, así que el kit de pantalla corre en la CI; la
  captura con ScreenCaptureKit necesita un permiso que el runner no tiene, y es del arranque en vivo y
  de la parada ⭐ (`bitácora:1697-1700`). En la máquina virtual de la CI Vision tarda ~1 s por lectura,
  así que el techo de duración se exige solo en un Mac de verdad (`bitácora:2041`).
- **Todavía no se ha visto capturar una ventana viva.** En la corrida en vivo de la fase 3 no hubo
  ventana visible que capturar; quedan el test bajo demanda
  `la_ventana_de_meet_se_captura_y_se_lee` y la parada con el usuario delante (`bitácora:2010-2031`).
- **Lo que esta decisión no cubre, con su sitio.** La auditoría del sprint registra que las copias del
  `Refuerzo` no se pisan al soltarse, contra lo que dice la cabecera del módulo (B6,
  `sprints/SPRINT_002-auditoria.md:465`); que `⌃⌥L` pide además una sugerencia con un turno del cliente
  que puede ser viejo (B5, `sprints/SPRINT_002-auditoria.md:464`); que el gate del efímero en marcha no
  ejerce Vision (M8, `sprints/SPRINT_002-auditoria.md:314-331`); y que el coste de CPU de la lectura no
  está medido (B21, `sprints/SPRINT_002-auditoria.md:480`). Y lo que Swift hace con su propia memoria
  al leer no se pisa desde Rust: el texto de Vision se arma en un `String` de Swift que se suelta sin
  ceros (`nativo/Pantalla.swift:182-198`).
- **Dos frases del código que este ADR deja con sitio:** la cabecera de `pantalla/huella.rs:8-13`
  todavía justifica el pHash, y `huella::phash` (`pantalla/huella.rs:119-126`) ya solo lo usan los
  tests del módulo. La decisión vigente es la de `pantalla/huella.rs:32-54`.

## Medición

Kit de pantalla v1: cinco ventanas sintéticas, medido en este Mac (`bitácora:1676-1702`).

| Qué                                                  | Medido                                                          | Mínimo o techo |
| ---------------------------------------------------- | --------------------------------------------------------------- | -------------- |
| Fichas que trae la pantalla sola, sin pregunta       | **4 de 4** (la agenda, sin cifras ni términos, no pide ninguna) | 4              |
| nDCG@5 de la frase vaga, sin pantalla → con pantalla | **0,626 → 0,700**                                               | 0,68           |
| Kit v0 con la peor pantalla delante                  | 0,819                                                           | 0,80           |
| Vision, por lectura                                  | mediana **~87 ms**, peor 93 ms                                  | 1000 ms        |
| La menor distancia entre dos diapositivas            | 144 celdas                                                      | umbral 12      |

- La tabla que eligió los dos números del desempate —peso 0,5, desempate a 1,1— está en
  `corpus/indice.rs:54-60` y en `bitácora:1655-1661`.
- **Cero falsos positivos:** ninguna de las cuatro preguntas sin respuesta del kit v0 consigue ficha
  con ninguna de las cinco pantallas delante (`bitácora:1695-1696`).
- **≤ 1 lectura/s es sostenible:** Vision tarda menos de 100 ms por pantalla en este Mac, diez veces
  por debajo del ritmo (`bitácora:1694`).
- Los rojos, uno por gate nuevo, en `bitácora:1704-1740`; la corrida en vivo de `⌃⌥L`, en
  `bitácora:1996-1997`.
