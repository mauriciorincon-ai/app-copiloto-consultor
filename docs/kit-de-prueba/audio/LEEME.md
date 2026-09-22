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

**Qué falta aquí, y en qué fase llega.** El kit de evaluación v0 de la fase 5 añade los turnos
marcados (para medir precisión y recall del disparador con margen de ±1 turno), un audio con
mezcla de idiomas, y el corpus sintético «Páramo Azul» con sus 30 preguntas.
