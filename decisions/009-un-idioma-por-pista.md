# ADR 009 — Un idioma por pista, y lo que el diccionario alcanza a arreglar

- **Fecha:** 2026-09-24
- **Sprint:** 002 «Cuándo, qué y quién mira», fase 1
- **Estado:** aceptada

## Contexto

La app promete ser bilingüe en todo, y el consultor al que sirve habla así de verdad: castellano con
jerga inglesa, y frases enteras que cambian de idioma a mitad. El sprint 001 dejó dos deudas escritas
en `docs/kit-de-prueba/audio/LEEME.md`: **un audio con mezcla de idiomas** y **un WER de la
transcripción**. Sin ellas no había forma de saber si el diccionario técnico —el trabajo de esta fase—
sirve o estorba, ni qué aguanta el motor de voz de macOS.

Las dos están pagadas. Este ADR recoge lo que dijeron los números.

## Lo que se midió

Cuatro audios del kit, 100 % sintéticos (`say` + `afconvert`, 16 kHz mono). WER contra la
transcripción de referencia escrita con la jerga **como el consultor quiere verla escrita**, porque eso
es lo que la app tiene que acabar produciendo:

| Audio | Idioma del motor | WER sin diccionario | WER con diccionario | |
|---|---|---|---|---|
| `pregunta-es.wav` | es-ES | 0,133 | 0,133 | control, sin jerga |
| `pregunta-en.wav` | en-US | 0,000 | 0,000 | control, sin jerga |
| `mezcla-es.wav` | es-ES | 0,458 | **0,417** | mejora |
| `mezcla-en.wav` | en-US | 0,348 | **0,261** | mejora |

**Los dos controles no se mueven ni un punto.** Es la mitad que más importa: un corrector que mejora la
jerga y estropea el resto de la reunión sale perdiendo, y eso no se ve en los ejemplos elegidos.

## Decisión

### 1 · Un idioma por pista. Se declara, no se promete

Con un solo idioma puesto, **una frase en el otro idioma no se transcribe: se destroza**. Medido, no
supuesto:

```
dicho:  «… Y el DAX lo escribió otro proveedor.»          (dentro de un audio en-US)
oído:   «… YL Daxlo is Gribbio Otro Provider.»

dicho:  «And we need the data cleaning inside the scope.»  (dentro de un audio es-ES)
oído:   «and Whened t Data Klean Inside the Scult.»
```

No es un WER alto: es texto que no significa nada. Así que la app hace lo que ya hacía y **lo dice**:
cada pista escucha **un** idioma —el consultor por el micrófono, el cliente por el audio del sistema— y
la pantalla de Idioma lo declara con esas palabras. No hay invento y no hay promesa.

### 2 · Qué alcanza el diccionario y qué no

El diccionario arregla **casi-aciertos**, no pérdidas totales:

- `Power B` → **Power BI** · `power BI` → **Power BI** · `lake house` → **Lakehouse** ·
  `semantic model` → **Semantic Model**. Tres correcciones en un solo audio.
- `Lakehouse` oído como «en la que usé» **no se arregla**, y no hay diccionario que lo arregle: no
  queda nada del término a lo que parecerse.
- `semanticque Model` **tampoco**: está a tres ediciones de «semantic model» y el techo de un término
  de catorce letras son dos. Subirlo arreglaría este caso y empezaría a corregir palabras que no son.

Es la frontera honesta de esta pieza, y por eso el gate mide **las dos direcciones**: que baje donde
hay jerga y que no suba en ningún sitio.

### 3 · Lo que NO se midió, dicho para que no se lea como más de lo que es

**No se midió qué hace SpeechAnalyzer con varios idiomas configurados a la vez**, porque **la app no
tiene manera de configurarlo**: `stt::Motor::transcribir` recibe **un** `idioma: &str`, y los dos que
usa la app son dos constantes (`cuaderno.ts` `DEL_CONSULTOR` / `DEL_CLIENTE`). El techo de cinco
idiomas que la pantalla menciona es cuántos modelos puede tener **instalados** el Mac, no cuántos
reconoce en una frase.

Esto deja una promesa **sin verificar** en la maqueta (`docs/diseno/idioma.html`), dentro de la
descripción de una feature marcada «todavía no»: *«Con varios marcados el motor no adivina el idioma de
la reunión — reconoce cualquiera de ellos, incluso mezclados en la misma frase»*. Lo medido aquí **no la
desmiente** —es otra configuración— pero tampoco la sostiene, y la frase afirma un comportamiento del
motor, no un deseo. **No se reescribe en este sprint**: es copy de la maqueta y su cambio es una
decisión del usuario. Queda anotado como lo que es: una afirmación que hay que comprobar con un spike
o suavizar antes de construir esa feature.

## Alternativas consideradas

- **Subir el techo de la distancia de edición** para cazar `semanticque Model`: arregla un caso y abre
  la puerta a corregir palabras que no son. El WER de los controles es hoy idéntico con y sin
  diccionario; ese es el activo que se estaría gastando.
- **Un segundo pase del motor con el otro idioma** sobre el mismo audio, quedándose con el de más
  confianza: dobla el coste de CPU de cada turno —el presupuesto es de 4 s de fin de turno a ficha— y
  el motor de Apple no devuelve confianza por el puente. Cuando la devuelva, se vuelve a mirar.
- **Prometer la mezcla y arreglarlo después:** es exactamente lo que este ADR existe para no hacer.

## Consecuencias

- El kit mide el WER en cada `cargo test` y **el umbral es doble**: no empeora en ningún audio, y baja
  en al menos uno con jerga. Un diccionario que no corrigiera nada pasaría el primero y falla el
  segundo.
- Los números de arriba son la **línea base**. Cuando la fase 3 traiga el OCR y la 5 el modelo, esta
  tabla dice si algo se rompió por el camino.
- La frase de la maqueta sobre varios idiomas mezclados queda **marcada como no verificada**. Si esa
  feature se construye, empieza por un spike que la mida.
