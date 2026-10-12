# Calibración de la transcripción en Windows — 100 % sintético

`sala-es-10s.wav` son los **primeros 10 s** de `docs/kit-de-prueba/audio/sala-es.wav` (kit v4, voces de macOS, cero
datos reales): mono, 16 bits, 16 kHz, 160 000 muestras. Tres turnos de la sala, el cliente y tú.

Para qué: al instalar el modelo de whisper (Idioma, «Instalar»), la app transcribe este recorte y **mide cuánto tarda
en tu PC** (ADR 022 §5). Honestidad enseña esa latencia; si pasa de 4 s, el modelo grande es lento para ese equipo y se
ofrece el pequeño. Entra en el binario con `include_bytes!`: no se lee de disco ni se baja.

Cómo se hizo (y cómo se comprueba: `stt::calibracion`, prueba `el_recorte_es_el_principio_del_kit`): las primeras
160 000 muestras del `data` de `sala-es.wav`, con la misma cabecera PCM.
