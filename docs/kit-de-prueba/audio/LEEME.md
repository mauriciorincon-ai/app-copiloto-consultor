# Audios del kit — 100 % sintéticos

Dos frases habladas por la voz del sistema de macOS (`say`), con el texto de la maqueta. **No hay
ninguna grabación de ninguna persona**, aquí ni en ningún otro sitio del repo: la regla de cero
datos reales de clientes se aplica también a las voces.

| Archivo | Idioma | Qué dice | Cómo se hizo |
|---|---|---|---|
| `pregunta-es.wav` | es-ES | «¿Ustedes tienen certificación ISO 27001? Y la limpieza de datos, ¿eso está dentro del alcance?» | `say -v Mónica` → `afconvert -f WAVE -d LEI16@16000 -c 1` |
| `pregunta-en.wav` | en-US | «Do you have ISO 27001 certification? And is data cleaning within the scope?» | `say -v Samantha` → mismo `afconvert` |

Formato: WAV PCM 16 bits, **16 kHz, mono** — la misma frecuencia a la que vive el audio dentro de
la app (`capture::anillo::HZ`), para que la prueba del puente mida el camino real y no uno con una
conversión de más.

## Qué hay y qué falta — al día del cierre del sprint

Esta nota prometía tres cosas «para la fase 5» y la fase 5 entregó una. Se corrige aquí con lo que
hay de verdad (hallazgo A9 de la auditoría del sprint 001):

| Qué | Dónde está | Estado |
|---|---|---|
| El corpus sintético «Páramo Azul» y sus 30 preguntas | `../corpus/` · `../preguntas.json` | **hecho** — nDCG@5 y rechazo, medidos en cada `cargo test` |
| Los turnos marcados del disparador | `../disparo.json` | **hecho** en la auditoría — 26 turnos con precisión y recall |
| La mediana de latencia del kit | dentro del test del kit | **hecho** en la auditoría |
| Un audio con **mezcla de idiomas** | — | **no está.** Deuda declarada, se paga en el sprint 2 |
| **WER** informativo de la transcripción | — | **no está.** Deuda declarada, se paga en el sprint 2 |

**Sobre el margen de ±1 turno** que el plan pedía para el disparador: `disparo.json` mide turno a
turno **sin margen**, porque sus turnos son texto y su reloj es exacto. Es más estricto, no más
laxo, y el propio archivo lo explica en su campo `tolerancia`. El margen tendrá sentido cuando la
medida se haga sobre audio, que es cuando el fin de turno puede caer un turno antes o después.
