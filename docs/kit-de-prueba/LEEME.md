# Kit de prueba — Angel Ghost · v3 (sprint 004)

Nació en el sprint 001 con el corpus y las preguntas; el sprint 002 le sumó el audio, la pantalla y el
radar; el sprint 003 (v2) le sumó las propuestas, las jurisdicciones, la puerta local y dos documentos
de ejemplo para la NDA y la carta de encargo; **el sprint 004 (v3) le suma el banco de preguntas del
ensayo** (`ensayo.json`).

**Qué mide la integración continua y qué no.** Todo lo de este kit lo mide la CI en cada push
(`cargo test --test contra-el-mac-de-verdad el_kit` y `cargo test --test puerta el_kit`), **salvo el
WER**, que necesita los modelos de voz de macOS y el runner no los tiene: es una medida **manual**, con
su corrida local registrada en la bitácora de cada sprint (ver `audio/LEEME.md`).

| Archivo | Qué mide | Hoy | Mínimo |
|---|---|---|---|
| `preguntas.json` | el buscador: nDCG@5 y cuántas sin respuesta rechaza | 0,823 · 1,000 | 0,80 · 1,00 |
| `disparo.json` | cuándo la app se pone a buscar, turno a turno | 1,000 · 1,000 | 1,00 · 1,00 |
| `pantalla.json` y `pantalla/` | la lectura de pantalla: ficha sin preguntar y el kit con la peor pantalla delante | 4 de 4 · 0,819 | 4 de 4 · 0,80 |
| `reunion-con-acuerdos.json` | **v2** · las propuestas: regla y dueño, turno a turno; del cliente jamás el turno | 15 de 15 · el turno más lento, menos de 1 ms | 100 % · 50 ms |
| `jurisdicciones.json` | **v2** · la bandera que sale de la línea «Jurisdicción:» de la ficha | 17 de 17 | 100 % |
| `preguntas.json` por la puerta | **v2** · el kit que corre `ghost` mide lo mismo que la CI | 0,823 = 0,823 | igual |
| `ensayo.json` | **v3** · el banco del ensayo, regla por regla: precisión y recall de las seis reglas, en español (Páramo Azul) y en inglés (Northwind); y el acento del modelo con el `mock` | lo imprime la CI | 0,75 · 0,75 por regla |
| `ensayo.json`, bloque `respuestas` | **v3** · la evaluación de una respuesta: ficha por ficha, «citada» contra lo que la respuesta usó; muletillas y ritmo; y el camino entero contra 500 ms | 1,000 · 0,714 · 4 ms | 0,75 · 0,70 · 500 ms |
| `audio/` | el WER con y sin diccionario | ver `audio/LEEME.md` | **manual** |

Todo lo de esta carpeta es **100 % sintético**. Ni un cliente real, ni un dato real, ni una cifra
real: la regla de esta app es que los datos del usuario viven fuera del repo, y un kit que la
rompiera sería el peor sitio posible para romperla.

## `corpus/` — seis documentos, las cinco unidades

Un corpus de consultoría en miniatura, escrito para que las cinco unidades del modelo tengan al
menos un documento: propuesta, marco, caso, ficha de cliente y perfil (más uno de seguridad, que
cae en «marco»). El cliente inventado se llama **Páramo Azul**.

Sirve para dos cosas: **probar la app a mano** —señálale esta carpeta desde la pantalla de
Corpus— y **medirla**, que es lo que hace `preguntas.json`.

## `preguntas.json` — el kit de evaluación v0

Treinta preguntas que un cliente haría en una reunión, cada una con la sección que **debería**
responderla, más cuatro que el corpus **no puede** responder.

Las cuatro últimas son la mitad que más importa. Un buscador que acierta 30 de 30 y además
contesta con seguridad a lo que no sabe es peor que uno que acierta 25: el fallo caro de esta app
no es no encontrar, es **encontrar cualquier cosa y ponerle una fuente debajo**.

Se mide solo, en la integración continua:

```
pnpm --dir . exec true && cd src-tauri && cargo test --test contra-el-mac-de-verdad el_kit -- --nocapture
```

| Medida | Qué es | Hoy | Mínimo |
|---|---|---|---|
| **nDCG@5** | si la sección correcta sale entre las cinco primeras, y en qué puesto | **0,823** | 0,80 |
| **rechazo** | cuántas de las cuatro imposibles se declaran sin respuesta | **1,000** | 1,00 |

**Tres preguntas fallan hoy, a propósito.** Ninguna comparte una sola palabra con su sección
(«¿por qué nos contrataron para esto?» contra una sección que habla de márgenes y canales).
BM25 no puede resolverlas, y reescribir las preguntas para que las acierte convertiría el kit en
un espejo. Eran la evidencia para decidir si los embeddings hacen falta, y el sprint 003 lo decidió
(ADR 008): **no en el H1**. BM25 cumple el umbral (0,823) y los embeddings pasan al H2 con una
condición escrita: diez preguntas de paráfrasis con nDCG@5 por debajo de 0,80.

**Lo que este kit ya encontró:** al correr por primera vez, la app citó una sección sobre gobierno
de datos para responder «¿cuánto cuesta el software de Salesforce?». La sección traía «cuánto» y
«cuesta», y con eso le bastaba para pasar por respuesta. Faltaba media docena de interrogativos en
la lista de palabras vacías.

## `audio/` — cuatro frases dichas en voz alta

Dos preguntas del sprint 001 (`pregunta-es.wav`, `pregunta-en.wav`) y dos frases con jerga y cambio
de idioma del sprint 002 (`mezcla-es.wav`, `mezcla-en.wav`), generadas con `say` y `afconvert` del
propio macOS: 16 kHz, mono, 16 bits. Alimentan el WER con y sin diccionario, los tests que
comprueban que el motor de voz transcribe y que una sesión completa no deja nada en el disco. Ver
`audio/LEEME.md`.

## `pantalla/` — cinco diapositivas y una reunión grabada

Imágenes sintéticas de una reunión compartiendo pantalla (`scripts/kit-de-pantalla.mjs`): cuatro con
cifras o términos del corpus y una agenda sin ninguno, para medir que la pantalla trae su ficha sin
pregunta; y `reunion-grabada.png`, con el aviso de grabación y un bot de notas, para el radar ámbar.
`meet-de-prueba.html` las enseña como si fuera una reunión de Meet (lo usa la guía de prueba).

## `radar/` — un Mac limpio y un Mac vigilado

`mac-limpio.txt` (82 procesos de un Mac corriente, con nombres parecidos a los del catálogo puestos
a propósito) y `mac-vigilado.txt` (un proceso de cada fila del catálogo): el radar coral tiene que
dar cero en el primero y todas las filas en el segundo.

## `reunion-con-acuerdos.json` — las propuestas (v2)

Una reunión inventada con Páramo Azul, veinte turnos en español y en inglés, cada uno con lo que las
reglas publicadas (`data/propuestas/reglas.json`) tienen que proponerte: la regla —cifra, compromiso,
choque con tu ficha fijada, nombre que tu corpus no tiene, pregunta del cliente— y de quién es. Trae
también los casos en que **no** debe proponer nada: una cortesía, un nombre que tu corpus sí conoce, un
compromiso que hace el cliente (la regla de compromisos solo mira los tuyos).

Además de acertar, la prueba exige lo que la app promete y ninguna pantalla deja ver: **del cliente
jamás se guarda el turno**, solo un fragmento de como mucho ocho palabras o, de una pregunta, sus
palabras clave.

## `jurisdicciones.json` — la bandera de cada cliente (v2)

Diecisiete líneas «Jurisdicción:» como las escribirías en la ficha de un cliente, y la bandera que la
app tiene que sacar del catálogo (`data/jurisdicciones/catalogo.json`): varias en una línea, la más
estricta; lo que el informe legal no verificó, dicho; lo que no está en el catálogo, «fuera del
catálogo», sin adivinar.

## `nda-de-ejemplo.md` y `carta-de-encargo-de-ejemplo.md` — para la guía (v2)

Dos documentos sintéticos, en español y en inglés, **que no son asesoría legal**. La NDA trae dos
versiones de la cláusula de registro de las reuniones —una que lo permite y otra que lo prohíbe— para
contestar las dos respuestas del chequeo de NDA de Sesión. La carta de encargo tiene una sección vacía
donde pegar la cláusula que copias en Sesión → «Cláusula de encargo».

## `pantalla/meet-en-negro.html` — una reunión sin nada que leer (v2)

Una página negra cuyo título dice «Google Meet»: con ella, ⌃⌥L tiene que contestar «Leí la pantalla: no
hay texto que buscar». Y `meet-de-prueba.html` se retocó en el sprint 003: la app lee la ventana entera,
pestaña y pie incluidos, y su título («Páramo Azul») y su contador («5 de 6») le daban a la agenda un
término y una cifra que la diapositiva no tiene. Ahora el título es neutro y el pie cuenta con puntos.

## `ensayo.json` — el banco de preguntas del ensayo (v3, sprint 004)

Dos casos: **Páramo Azul**, que lee la propuesta y la ficha de `corpus/`, y **Northwind Feed Co.**, escrito
dentro del archivo y en inglés, porque el banco es bilingüe. Cada caso dice qué preguntas esperaría un
consultor, regla por regla: una por sección (alcance, supuestos, entregables, precio, plazo, contexto),
por cada cifra que escribiste, por cada compromiso, por cada «si…» o supuesto, por lo que la ficha dice que
le importa al cliente y por las objeciones del catálogo que tocan la propuesta.

El test (`cargo test --test contra-el-mac-de-verdad el_kit_del_ensayo -- --nocapture`) arma el banco
**entero**, sin tope, e imprime por regla cuántas dio, cuántas esperaba el kit, la precisión y el recall,
más un ensayo de 8 tal como lo vería el usuario. Después pide al `mock` el acento del modelo e imprime lo
que propone, lo que se funda y lo que se tira, y lo que costaría una llamada con cada proveedor externo.

**La evaluación (fase 3 del sprint 004).** El bloque `respuestas` trae siete preguntas con una respuesta
sintética cada una, dicha en tantos segundos, y lo que un consultor diría que esa respuesta **usó**: los
títulos de las secciones, sus muletillas y su ritmo. El test
(`cargo test --test contra-el-mac-de-verdad el_kit_del_ensayo_mide_la_evaluacion -- --nocapture`) busca cada
pregunta como lo haría la banda, arma sus tres fichas, evalúa la respuesta y compara, ficha por ficha,
«citada» con «usada». Imprime la tabla, la precisión y el recall, y mide el camino entero.

- **Hoy: 15 fichas, precisión 1,000 y recall 0,714**, la más lenta en 4 ms (presupuesto: 500 ms).
- **Dos fallan a propósito:** una respuesta que dice lo de la ficha con otras palabras y otra en inglés a una
  pregunta en español. Ninguna comparte dos términos con su ficha, y eso es lo que corrige «Sí lo dije». El kit
  lo enseña en vez de esconderlo.
- **Sirven para probar a mano:** en el bloque R de la guía, si no sabes qué contestar, lee una en voz alta.

**Una advertencia, escrita aquí para que nadie la lea como más de lo que es:** el kit y las reglas los
escribió el mismo constructor, en el mismo sprint. Que el banco acierte su propio kit es un **piso** —que
ninguna regla se rompa sin que se note—, no una prueba de calidad. La prueba de verdad es tu propuesta
real, y tu voz, en el ⭐ del ensayo (bloque R de la guía).
