# ADR 007 — Las dos pistas: Core Audio a mano, y el eco que apareció al correrlo

- **Fecha:** 2026-09-21
- **Sprint:** 001 «La banda y la ficha», fase 3
- **Estado:** aceptada

## Contexto

La app promete oír los dos lados de la videollamada **sin meter un bot en la reunión**: el
micrófono es el consultor y el audio del sistema es el cliente. La atribución se resuelve **por
pista, jamás por biometría** (regla de cero huellas de voz), así que la calidad de esa promesa
depende por entero de que las dos pistas estén de verdad separadas.

## Decisión

### 1 · Core Audio a mano para las dos pistas, sin `cpal`

El plan nombraba `cpal` para el micrófono. Para el micrófono solo habría estado bien, pero **el
audio del sistema no está en ninguna biblioteca**: el *process tap* de Core Audio (macOS 14.2+)
hay que escribirlo contra la API de C pase lo que pase. Escrito el camino difícil, usar otro
distinto para el fácil añade una dependencia, un segundo modelo mental y un segundo sitio donde
puede fallar la conversión a mono — sin ahorrar nada.

Los dos grifos son el mismo código con un dispositivo distinto. Todo el `unsafe` vive en
`capture/nativo.rs`, igual que `acople/ax.rs` concentra el del acople.

El tap **se excluye a sí mismo**. Hoy la app no hace ruido; en cuanto exista el modo solo audio
(C15, sprint 2) hablará por los altavoces, y un tap que se oyera a sí mismo transcribiría su
propia voz como si fuera el cliente.

### 2 · Todo a 16 kHz mono en la puerta de entrada, con filtro

El tap entrega 48 kHz; el micrófono, lo que diga el dispositivo. Dentro de la app todo vive a
16 kHz —lo que quieren el detector de voz y el transcriptor— y la conversión **filtra antes de
decimar**. Quedarse con una muestra de cada tres cuesta tres líneas y funciona hasta que entra un
agudo: todo lo que pase de 8 kHz se dobla hacia abajo y **reaparece como un tono grave que nunca
existió**, justo en la banda de la voz humana. Medido: un tono de 18 kHz reaparece a 2 kHz con
RMS 0,707 — a todo volumen.

### 3 · «Cero muestras» no es una avería

Descubierto corriéndolo: **cuando no suena nada, el sistema no llama al callback ni una vez**. No
llegan ceros, no llega nada. Un cliente callado y un tap roto se ven idénticos desde dentro del
programa.

Lo que sí se puede afirmar es otra cosa —si el grifo se abrió—, y es lo que la app enseña. La
pantalla de Sesión dice «Funciona» del grifo, no de las muestras; la de Honestidad enseña los
bytes que hay, que pueden ser cero sin que nada esté roto.

### 4 · El eco: se marca, no se borra

La primera prueba de punta a punta sonó por los altavoces del portátil y el turno del cliente
apareció **en las dos pistas**. Con altavoces, la promesa «micrófono = tú» es falsa: la app le
atribuiría al consultor palabras que no dijo.

La respuesta canónica es un cancelador de eco acústico. La de esta casa es la determinista y
legible (`voz::eco`): un turno del micrófono es un reflejo si **se solapa en el tiempo** con uno
del sistema (≥50 %) **y dice casi lo mismo** (≥60 % de trozos de tres letras). Las dos
condiciones, porque cada una sola falla por su lado: solapar es también interrumpir, y repetir lo
que acaba de decir el cliente es una cosa normal que pasa desplazada en el tiempo.

Se compara por trozos de tres letras y no por palabras porque en los datos reales el transcriptor
escribió «certificaciones» en una pista y «certificación» en la otra.

**Y el turno se marca, no se borra.** Un turno que desaparece sin explicación es la clase de
silencio que esta app no se permite. La banda no lo atribuye al consultor; la pantalla de Sesión
dice lo único que resuelve el problema de verdad: ponte los auriculares. Y para poder decirlo,
la app **mide por dónde sale el sonido** (`bltn` + `ispk` = altavoces internos).

### 5 · El audio vive en un solo sitio

Anillos de treinta segundos por pista (1,8 MB cada uno). El turno no se acumula aparte: se pide
por su sitio en el tiempo cuando ya terminó (`Anillo::rango`). Dos copias serían dos vaciados, y
el segundo es el que alguien olvidaría el día que añada una pieza.

`Anillo::vaciar` **sobrescribe con ceros**; mover el cursor deja las muestras íntegras en la
memoria del proceso, y el kill-switch de esta app se pulsa delante del cliente.

## Consecuencias

- Tres piezas más del kill-switch pasaron de «todavía no existe» a cortadas: **6 de 7**. El
  `match` sin comodín de `corte.rs` no dejó compilar hasta resolverlas.
- La app pide `NSMicrophoneUsageDescription` y `NSAudioCaptureUsageDescription`. La segunda clave
  **no aparece en las cabeceras públicas del SDK** y se declara porque una de más es inofensiva y
  una de menos mata la app al pedir el permiso. **Parada ⭐:** solo se comprueba de verdad con la
  app empaquetada y firmada.
- Se declara `libc` para leer el desplazamiento horario y escribir «14:02» en un turno.

## Lo que este ADR NO decide

- Cancelación de eco de verdad (AEC) ni diarización: son de H2, y el estándar 4-T las mira aparte.
- Qué se dispara con el fin de turno. Eso es la fase 4.
