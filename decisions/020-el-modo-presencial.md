# ADR 020 — El modo presencial: una pista para la sala, y la tolerancia a tu voz por código

- **Fecha:** 2026-10-05
- **Sprint:** 005 «En la mesa» (fase 0, antes de construir)
- **Estado:** aceptada. La regla de tolerancia (§3) se eligió con la medición del kit v4 al STOP de la fase 1, y sus
  cifras están en § Medición (2026-10-10).

## Contexto

La VISION v1.5.0 promete el modo presencial (C19) así: _«en la mesa con el cliente, o al lado del PC donde corre una
videollamada, el Mac escucha la sala por su micrófono (una sola pista, sin captura del sistema ni acople) y te muestra
la ficha igual que en la banda […]. Sigue siendo efímero y sigues siendo participante. Cubre desde el primer día las
reuniones que ocurren en Windows mientras la versión nativa llega. Con auriculares y el modo solo audio es el que menos
se nota en presencia.»_

Hasta hoy la app reconoce a cada uno por el **dispositivo** por el que entra su voz, nunca por la voz misma (ADR 007,
regla dura 4): el micrófono es el consultor y el audio del sistema es el cliente. Sobre eso se construyó el disparo:
**«dispara el cliente, no el consultor»** (ADR 008; `src-tauri/src/disparo/mod.rs:9-11`). En la sala las dos voces
llegan por el mismo micrófono. Con el código de hoy no saldría ninguna ficha automática: el disparador descarta todo
turno que no venga de la pista del sistema (`disparo/mod.rs:109`). Y ⌃⌥A tampoco encontraría nada, porque busca la
última frase «del cliente» (`lib.rs:2149`).

**Por qué la función vale la pena.** Hay asistentes de conversación cara a cara y se usan: _SocialMind_ [S12b], con 20
participantes, +38,3 % de engagement y un 95 % dispuesto a usarlo; _ChatAR_ [S12c], que reconoce palabras clave y
cuyo método de presentación reduce que el interlocutor note la lectura; _ChatMuse_ [S12d], con una guía textual privada
y un agente proactivo. Son trabajos con casco de realidad aumentada o mixta; aquí el dispositivo es el Mac en la mesa o
la voz al oído, pero la conclusión que importa se traslada: **la ayuda privada y breve en una conversación presencial
no rompe la conversación si no se nota** (investigación científica de la planeadora, 2026-09-17, §B). Por eso la voz al
oído es el modo de partida (pregunta 2 del G-Plan, respuesta (a)).

**Y por qué se resuelve sin identificar voces:** el estándar 4-T prohíbe las huellas de voz, y la VISION las pone en
«fuera de alcance (nunca)». La app no aprende a distinguir tu voz de la del cliente: ni registrándote, ni agrupando
voces (la diarización también deriva rasgos de la voz). Lo que se pueda separar con texto y con tiempo, se separa. Lo
que no, se declara.

## Decisión

### 1 · Un tercer modo con una sola pista: la sala

- **`Modo::Presencial`**, junto a Reunión y Solo notas. Abre **solo el micrófono**. La pista se llama
  **`Pista::Sala`** y es distinta de `Pista::Microfono`, porque no dice de quién es la voz.
- **De quién es cada turno lo decide un `match`, no una comparación:** `Pista::quien() → Quien { Tuyo, Cliente,
SinAtribuir }`, sin comodín, y el dispositivo va aparte (`capture::Fuente { Microfono, Sistema }`). Hoy unos 25
  sitios comparan pistas con `==`. Con una variante nueva el compilador no avisaría, y la sala acabaría tratada como el
  cliente en la banda o como tú en las propuestas. Con el `match`, cada sitio tiene que decidir. Un test de fuente
  prohíbe comparar `Pista::Sistema` o `Pista::Microfono` fuera de `capture/`.
- **Lo que NO se abre en presencial, con un test por cosa:** el audio del sistema · la lectura de pantalla (y con ella
  ⌃⌥L y el radar de pantalla, que viven dentro de la lectura) · el acople a la ventana de la reunión (`AXPosition` no se
  escribe nunca, y al empezar se suelta lo que estuviera acoplado) · las propuestas automáticas · «conservar mis
  turnos».
- **Lo que sigue igual:** la banda **con su protección y su relleno** (§9) · el radar de procesos, que solo mira tu Mac
  (como en el ensayo) · el corte ⌥⎋ · el contador de red (0 en modo local) · la sugerencia del modelo, si la tienes
  encendida · la puerta local cerrada durante la sesión, como en toda sesión.
- **La NDA que prohíbe transcribir bloquea también presencial**, y desde ahora lo comprueba Rust, no solo la pantalla
  (hoy solo lo mira `Sesion.tsx:171`).

### 2 · Quién dijo qué: la app no lo sabe, y lo dice

El transcript de la sala no tiene dueño: cada línea lleva «Sala». La consecuencia, escrita en el manual: **en presencial
la ficha puede responder a algo que dijiste tú**. Para que la prohibición no dependa de la memoria:

- el lint del vocabulario vetado suma `voiceprint` y `speaker embedding`;
- nace el gate **`cero-huellas-de-voz`**: ningún módulo usa una API de identificación de hablante ni de rasgos de voz
  (`SoundAnalysis` para hablantes, embeddings de voz, `mfcc`, diarización…). Su rojo se ve plantando uno.

### 3 · La tolerancia a tu voz: reglas sobre texto y tiempo, elegidas midiendo

Delante del disparador de siempre, y solo para los turnos `Quien::SinAtribuir`, va una capa pura, **`disparo::Tolerancia`**.
Sus parámetros viven en **`data/presencial/reglas.json`**, versionado y dentro del binario (`include_str!`), como los
otros catálogos. **Este ADR no fija ningún valor:** el kit v4 recorre las candidatas y sus valores, y la regla que entra
es la que muestra la medición.

| Candidata                      | La idea                                                                                                                                                                                                    | El riesgo                                                             |
| ------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- |
| **C1 · espera tras una ficha** | Tras una ficha, lo siguiente suele ser tu respuesta: no se dispara durante N s                                                                                                                             | pierde la repregunta rápida del cliente                               |
| **C2 · eco de la ficha**       | Un turno que repite los términos de la ficha que acabas de ver suele ser tú leyéndola o parafraseándola: no dispara. Solo cuentan las fichas con resultado (`Respuesta::Ficha`), y se olvidan al reiniciar | pierde la pregunta del cliente sobre lo mismo que se acaba de mostrar |
| **C3 · solo pregunta o cifra** | En la sala, nombrar un término de tu corpus no basta, porque tú los dices todo el tiempo: hace falta forma de pregunta o una cifra                                                                         | pierde el término que el cliente nombra sin preguntar                 |
| **Silencio**                   | El disparo por silencio largo arranca apagado en la sala, porque lo último que se dijo suele ser tuyo                                                                                                      | pierde el hueco en el que el cliente espera                           |
| **Tope de turno**              | Dos voces con pausas de menos de 320 ms se juntan en un turno. Un tope de duración lo corta antes de que pase de los 30 s del anillo y su audio se pise antes de transcribirse                             | un corte en mitad de una frase larga                                  |

El kit mide también **la fila «sin filtro»** (el disparador tal cual) como línea base. **⌃⌥A funciona siempre**, sin
filtro, con lo último que oyó la sala: es la salida cuando la regla calla.

### 4 · El protocolo de medición (kit v4), escrito antes de medir

- **Audio 100 % sintético, cero grabaciones de personas:** dos voces de `say`, una por papel y por idioma, en una sola
  pista. `scripts/kit-v4-sala.sh` las genera, las pasa a 16 kHz mono (`afconvert`), las concatena con pausas y baja la
  ganancia de la voz que está lejos → `docs/kit-de-prueba/audio/sala-es.wav` y `sala-en.wav`.
- **El guion, `docs/kit-de-prueba/presencial.json`:** cada turno con quién lo dice (la verdad, **solo para el kit**: la
  app nunca la ve), su texto, sus ms, si debería disparar y por qué. **Incluye preguntas del consultor** («¿les parece
  si…?», «¿qué plazo manejan?»), que son el falso positivo más probable, y respuestas del consultor que leen la ficha.
- **Nivel A, en la CI (puro):** los turnos de texto pasan por el disparador y la tolerancia para cada combinación de
  candidatas y valores. Sale una tabla con fichas **pertinentes** (un turno del cliente que debía disparar y disparó),
  **falsas** (disparó con un turno tuyo, o con uno del cliente que no debía) y **perdidas** (uno del cliente que debía
  disparar y no disparó), **por motivo**.
- **Nivel B, en la CI (puro):** el VAD y el fin de turno de siempre sobre los wav: ¿se juntan dos voces en un turno?,
  ¿cuánto dura el turno más largo?, ¿hace falta el tope y de cuánto?
- **Latencia:** disparo, búsqueda y ficha frente al presupuesto de ≤ 4 s.
- **Nivel C, solo en local (`hardware`):** la transcripción de las salas. La CI no tiene modelos de reconocimiento de voz:
  esta métrica es **`manual`** y se corre en la corrida en vivo de la fase 2 (regla 15).
- **Cómo se elige, dicho antes de ver los números:** primero, las combinaciones que no pierden más pertinentes que la
  línea base; entre ellas, la de **menos falsas**; si empatan, la de menos parámetros. Te enseño la tabla y una
  recomendación en llano, y la decides tú al STOP de la fase 1.

### 5 · «Lo tuyo queda», apagado en presencial, y dicho

Sin una pista tuya no hay turnos tuyos que conservar. `al_empezar` recibe el modo: `abrir(conservar && !presencial)`.
El interruptor se desactiva en presencial **sin tocar tu preferencia**, que vuelve a valer en la siguiente reunión. Un
turno `SinAtribuir` no se guarda nunca. Tus notas, tus fichas fijadas y tus acuerdos escritos sí se quedan. Sesión y
Honestidad lo dicen: «lo tuyo queda: tus turnos, apagado en presencial».

### 6 · El idioma de la sala (enmienda 009)

Toda la sala va en **un** idioma, una preferencia nueva (`idiomasDePista.sala`, enmienda 002). De fábrica es el de la
pista del cliente: en una mesa, el que manda es el idioma de quien pregunta. Si tú hablas en otro, tu parte se
transcribe mal (ADR 009 §1, medido), y la app lo dice al elegirlo. La voz al oído lee en **tu** idioma, el de tu pista,
porque la ficha sale de tus documentos.

### 7 · Cómo ves la ficha: al oído, y la banda a una tecla (pregunta 2 del G-Plan)

- **«Al oído» viene marcado al empezar** (el modo solo audio, C15), **salvo** que el sonido salga por altavoces que la
  app reconoce (los del Mac, HDMI, DisplayPort, AirPlay). Entonces va la banda, y la app dice por qué.
- **Pulsar «Empezar» con «al oído» elegido es tu gesto.** La regla «el modo solo audio no se enciende solo nunca»
  (`habla/mod.rs:177`, `prefs.rs:13`) se mantiene: lo enciende tu elección para esta sesión, no la app.
- **En la sala la voz espera a que la sala calle**, con un tope de unos segundos. Hoy `cabe_decirla` descarta la ficha
  si alguien habla. Si la sala no calla, la ficha se queda en la banda y `⌃⌥V` la enseña (apaga el modo y la banda
  recupera su alto). El candado de los altavoces (ADR 014 §2) no cambia.
- **Con Bluetooth o USB la app no distingue un auricular de un altavoz** (ADR 014 §2), y lo dice: «si es un altavoz,
  elige la banda».

### 8 · Windows al lado

La videollamada corre en tu PC con Windows y el Mac está al lado. El PC suena por su altavoz y el Mac escucha la sala.
**Si llevas auriculares en el PC, la app no oye al cliente**, y el manual y Sesión lo dicen. En el PC no se instala
nada y la app no toca esa máquina (regla dura 9). Sigues siendo participante, y lo efímero no cambia. El marco legal es
el de siempre: participante y efímero, y la bandera de jurisdicción aplica igual (investigación legal de la planeadora,
2026-09-17).

### 9 · La banda y su relleno siguen protegidos (desviación de la orden)

La orden pedía no crear el relleno ni la protección en presencial, porque «no hay nada compartido». **Se quedan.** La
banda y el relleno existen desde el arranque de la app (`setup`, `lib.rs:1727`), y la protección es estática y tiene su
invariante (`ventana/mod.rs:48-79`). En una sala, compartir tu pantalla en una tele o un proyector es normal. Con la
banda protegida, el relleno es lo único que tapa lo que hay detrás; quitar los dos dejaría la ficha a la vista de la
sala. Lo que sí se cumple de la orden: **nunca se acopla ni se escribe `AXPosition`**. La banda no afirma una
protección verificada «por cliente de videollamada» que en presencial no aplica: dice «Presencial · la sala».

## Alternativas consideradas

- **Registrar tu voz (huella) o agrupar las voces de la sala (diarización):** prohibido por el 4-T y fuera de alcance
  en la VISION. Registrar «solo la tuya» sigue siendo una huella de voz.
- **Un segundo micrófono** (el de unos auriculares para ti y el del Mac para la sala): dos dispositivos, dos pistas, y
  parece resolverlo. Pero en una sala el micrófono de los auriculares también oye al cliente, así que separar exige
  cancelación de eco, y depende de un aparato que el usuario puede no tener. Se queda en el roadmap.
- **Llevar el audio del PC con Windows al Mac por la red:** exige instalar algo en el PC. Ese camino es el S6 (Windows
  nativo).
- **Sin tolerancia, todo dispara:** se mide como línea base («sin filtro») y no se elige a ciegas.
- **Quitar la banda y el relleno en presencial, como pedía la orden:** §9.

## Consecuencias

- La fase 1 empieza por lo que no cambia el comportamiento (`Fuente`, `Pista::Sala`, `quien()`), con la suite entera en
  verde, antes de tocar el modo.
- El contrato Rust→TS gana `Pista::Sala` y `EstadoDeEscucha.presencial` (regla 19), y `empezar_presencial` vive solo
  en la ventana principal y en `SENSIBLES`.
- `verify:ephemeral` suma lo nuevo a los módulos protegidos, y la sesión completa del gate gana un paso presencial con un
  término plantado en un turno de la sala, que no debe aparecer en el log.
- El manual dice lo que no hace: no separa voces, no conserva tus turnos, no lee la pantalla y no oye al cliente si
  llevas auriculares en el PC.

## Medición

Corrida local del 2026-10-05 (`AG_SIN_HARDWARE=1 cargo test --locked --test contra-el-mac-de-verdad el_kit --
--nocapture`) y la de la CI de `379f8ef` (corrida 37409586560, paso «el kit, con su salida»): **las mismas cifras fila
por fila**; la latencia peor, 4 ms en local y 2 ms en la CI. Detalle, con las 36 filas y cada ficha perdida con su
causa: bitácora del sprint 005, «La medición del kit v4».

**Nivel A** (las dos salas, 14 fichas debidas; falsas = disparadas por tu voz o por una frase del cliente que no debía):

| Regla | Pertinentes | Falsas | Perdidas | Ajustes |
|---|---|---|---|---|
| Sin filtro (línea base) | 5 | 16 | 9 | 0 |
| **C1 15 s · C3 · silencio apagado (la elegida)** | 5 | **5** | 9 | 3 |
| C1 15 s · C3 · silencio encendido | 6 | 7 | 8 | 2 |
| C3 sola | 6 | 9 | 8 | 1 |
| C1 15 s sola | 4 | 12 | 10 | 1 |

- **C2 no cambia ninguna fila** (con 0,34 o 0,50, igual que apagada): queda apagada (`ecoDeLaFicha: null`).
- **Las 9 perdidas de la elegida:** 4 preguntas del cliente que llegan de 3,5 a 5,9 s después de una ficha que
  disparaste tú (las calla la espera de 6 s entre fichas que ya existía; ninguna candidata la toca), 3 términos que el
  cliente nombra sin preguntar (el precio de C3), 1 repregunta rápida a los 8,8 s (C1) y 1 silencio del cliente (el
  silencio apagado). Para todas está `⌃⌥A`.

**Nivel B** (el VAD y el fin de turno de siempre sobre los wav): sin tope, el intercambio rápido se junta en **un turno
de 29,0 s (español) y 29,2 s (inglés) con las dos voces**, a un segundo del anillo de 30 s. Con **15 s**, el más largo
es 15,0 s y corta a media frase una vez (en la sala inglesa); con 25 s, 25,0–25,1 s y una vez en cada sala.

**La elección (STOP de la fase 1):** la primera por el criterio escrito en §4 antes de medir —no pierde más que la
línea base y es la de menos falsas— y el tope de 15 s. El usuario no eligió otra en su respuesta, que llegó con el hueco
de la elección sin llenar; se aplicó lo que el STOP declaró por defecto (bitácora). Va en `data/presencial/reglas.json`:
`esperaTrasFichaMs: 15000`, `ecoDeLaFicha: null`, `soloPreguntaOCifra: true`, `silencio: false`, `topeDeTurnoMs: 15000`.
Con la voz al oído de fábrica, una ficha falsa es una voz que te habla mientras hablas: pesan más que una pertinente de
más.

**Lo que vigila la elección:** el nivel A exige que la regla de la casa no pierda más que la línea base y que **deje
menos falsas** que ella (su rojo: devolver `reglas.json` a la línea base nombró «la sala no tolera tu voz»); el nivel B,
que con el tope de la casa ningún turno pase del anillo.

**Lo que el kit no mide, dicho:** el nivel A trata cada turno como texto; la sala de verdad junta turnos (nivel B) y
transcribe con errores (nivel C, `manual`, en la corrida en vivo). El corpus del kit es en español, así que en la sala
inglesa C1 casi no actúa. Una regla para las preguntas que van justo detrás de las tuyas no se midió (respuesta por
defecto del STOP): queda como limitación en el manual.
