# Audios del kit — 100 % sintéticos

**Cuatro** frases habladas por las voces del sistema de macOS (`say`): dos del sprint 001 con el texto
de la maqueta, y dos del sprint 002 con la mezcla de idiomas y la jerga técnica. **No hay ninguna
grabación de ninguna persona**, aquí ni en ningún otro sitio del repo: la regla de cero datos reales de
clientes se aplica también a las voces.

| Archivo | Idioma | Qué dice | Cómo se hizo |
|---|---|---|---|
| `pregunta-es.wav` | es-ES | «¿Ustedes tienen certificación ISO 27001? Y la limpieza de datos, ¿eso está dentro del alcance?» | `say -v Mónica` → `afconvert -f WAVE -d LEI16@16000 -c 1` |
| `pregunta-en.wav` | en-US | «Do you have ISO 27001 certification? And is data cleaning within the scope?» | `say -v Samantha` → mismo `afconvert` |
| `mezcla-es.wav` | es-ES | «Nosotros ya tenemos el Semantic Model en Power BI, pero el Lakehouse todavía no está. And we need the data cleaning inside the scope.» | `say -v Paulina` → mismo `afconvert` |
| `mezcla-en.wav` | en-US | «We already have the Semantic Model in Power BI, but the Lakehouse is not ready yet. Y el DAX lo escribió otro proveedor.» | `say -v Samantha` → mismo `afconvert` |

**Sobre la voz:** los dos audios del sprint 001 se hicieron con **Mónica**, que en este Mac **ya no está
instalada** (macOS mueve las voces entre versiones). Los del sprint 002 usan **Paulina** (es_MX). No
afecta a lo que miden —el motor transcribe las dos igual de bien— pero conviene que la tabla diga la
verdad: quien quiera regenerar `pregunta-es.wav` hoy tendrá que elegir otra voz, y su WER cambiará un
poco. Por eso los audios se **versionan** en vez de generarse en cada corrida: un kit que se
regenerara solo mediría una voz distinta cada vez que Apple cambie de catálogo.

## El WER — la referencia y los números

`transcripciones.json` guarda **lo que cada audio dice, palabra por palabra**, con la jerga escrita como
el consultor quiere verla («Power BI», no «power by»): eso es lo que la app tiene que acabar
produciendo, así que es la referencia correcta. Medir contra lo que el motor oye sería medirlo contra sí
mismo.

Se mide en cada `cargo test`, con el diccionario técnico puesto y sin él:

```
cd src-tauri && cargo test --test contra-el-mac-de-verdad el_wer -- --nocapture
```

| Audio | WER sin diccionario | WER con diccionario | |
|---|---|---|---|
| `pregunta-es.wav` | 0,133 | 0,133 | control — sin jerga, no se mueve |
| `pregunta-en.wav` | 0,000 | 0,000 | control — sin jerga, no se mueve |
| `mezcla-es.wav` | 0,458 | **0,417** | mejora |
| `mezcla-en.wav` | 0,348 | **0,261** | mejora |

**El umbral es doble:** no empeora en ningún audio, y **baja en al menos uno con jerga**. Solo el
primero sería trampa — un diccionario que no corrigiera nada lo pasaría.

Y lo que los números dicen además del diccionario: **una frase en el otro idioma no se transcribe, se
destroza** («Y el DAX lo escribió otro proveedor» → «YL Daxlo is Gribbio Otro Provider»). De ahí sale el
ADR 009: un idioma por pista, declarado en la pantalla en vez de prometido.

Formato: WAV PCM 16 bits, **16 kHz, mono** — la misma frecuencia a la que vive el audio dentro de
la app (`capture::anillo::HZ`), para que la prueba del puente mida el camino real y no uno con una
conversión de más.

## Qué hay y qué falta — al día del sprint 002, fase 1

Esta nota prometía tres cosas «para la fase 5» del sprint 001 y esa fase entregó una. Se corrigió al
cerrar aquel sprint (hallazgo A9 de su auditoría) y se vuelve a poner al día aquí, con las dos deudas
que quedaban ya pagadas:

| Qué | Dónde está | Estado |
|---|---|---|
| El corpus sintético «Páramo Azul» y sus 30 preguntas | `../corpus/` · `../preguntas.json` | **hecho** — nDCG@5 y rechazo, medidos en cada `cargo test` |
| Los turnos marcados del disparador | `../disparo.json` | **hecho** en la auditoría — 26 turnos con precisión y recall |
| La mediana de latencia del kit | dentro del test del kit | **hecho** en la auditoría |
| Un audio con **mezcla de idiomas** | `mezcla-es.wav` · `mezcla-en.wav` | **pagado en el sprint 002**, fase 1 |
| **WER** informativo de la transcripción | `transcripciones.json` + el test `el_wer…` | **pagado en el sprint 002**, fase 1 — y no es informativo: tiene umbral y falla |

**Sobre el margen de ±1 turno** que el plan pedía para el disparador: `disparo.json` mide turno a
turno **sin margen**, porque sus turnos son texto y su reloj es exacto. Es más estricto, no más
laxo, y el propio archivo lo explica en su campo `tolerancia`. El margen tendrá sentido cuando la
medida se haga sobre audio, que es cuando el fin de turno puede caer un turno antes o después.
