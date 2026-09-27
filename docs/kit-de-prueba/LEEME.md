# Kit de prueba — Angel Ghost, sprint 001

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
un espejo. Son la evidencia para decidir si los embeddings hacen falta; el sprint 002 no lo
decidió (nDCG@5 0,82 sin ellos) y la decisión pasa al sprint 003 (ADR 008).

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
