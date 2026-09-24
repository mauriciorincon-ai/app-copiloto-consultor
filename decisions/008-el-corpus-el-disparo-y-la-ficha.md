# ADR 008 — El corpus, el disparo y la ficha: tres decisiones que se tomaron midiendo

- **Fecha:** 2026-09-21
- **Sprint:** 001 «La banda y la ficha», fase 4
- **Estado:** aceptada

## Contexto

La fase 4 cierra el camino que da nombre al sprint: del corpus propio del consultor a la ficha
que aparece en la banda. Son tres piezas encadenadas —índice, disparador, ficha— y **ninguna
lleva una línea de LLM**, como manda la regla del código primero y como exige este sprint.

Tres de las decisiones no se podían tomar leyendo documentación. Se tomaron midiendo, y esto
registra qué se midió y qué salió.

## Decisión 1 · El stemmer, el acento, y un spike que se dio la razón a sí mismo

`tantivy` trae stemmers Snowball por idioma. La app es bilingüe por regla dura, así que **cada
sección se indexa dos veces, una con cada stemmer**, y la consulta pregunta a los dos. Adivinar
el idioma del documento habría fallado justo en los corpus mezclados, que son los de este
usuario.

Al escribirlo aparecieron dos cosas que no estaban en ningún manual:

**(a) El spike no probaba lo que yo creía.** Afirmaba que «rentable» encuentra «rentabilidad» y
pasaba en verde. Es falso —`rentabl` contra `rentabil`— y pasaba porque en la misma consulta iba
la palabra «canal», que sí estaba en el documento. Un spike cuya aserción no distingue entre dos
causas no ha medido nada. El test que quedó afirma lo que el stemmer hace de verdad: número y
conjugación.

**(b) `metodología` y `metodologia` producen raíces DISTINTAS.** El stemmer español usa la tilde
para reconocer algunos sufijos, así que la misma palabra con y sin acento no se encuentra. Como
el transcriptor escribe sin tilde y el teclado del usuario también, eso es una pérdida constante.

La cadena de análisis se decidió **midiendo las dos opciones** sobre 23 parejas de lenguaje de
consultoría (lo que dice el documento ~ lo que diría el cliente):

| Cadena | Parejas que une |
|---|---|
| minúsculas → stemmer | 15 / 23 |
| minúsculas → **plegado de acentos** → stemmer | **17 / 23** |

Va con plegado. **No es gratis y se declara:** al plegar la tilde, el stemmer deja de reconocer
el sufijo «-ción» y se pierde la familia `implementación~implementar` / `implementan`. Se ganan
cuatro parejas de acento (`metodología`, `implementación`, `García`, `auditoría`) y se pierden
dos de morfología. El instrumento para revisarlo es el kit de evaluación (nDCG@5) de la fase 5;
si mostrara que la pérdida pesa, el camino es indexar las dos variantes en campos aparte, no
volver atrás.

## Decisión 2 · El umbral de la ficha no es un puntaje

BM25 **siempre devuelve algo**. Sobre un corpus de propuestas, cualquier pregunta encuentra la
sección «menos mala», y enseñarla con su fuente concreta debajo sería el fallo más caro que esta
app puede cometer: una fuente exacta hace creíble una respuesta equivocada.

Un umbral sobre el puntaje no sirve —BM25 no es comparable entre consultas distintas— así que el
criterio es otro, y tiene la ventaja de que se le puede explicar al usuario: **la sección tiene
que contener de verdad al menos dos palabras distintas de las que se buscaron**. Si no, la app
declara que no tiene nada, **dice qué buscó** (para que se vea en el acto si entendió mal) y
ofrece lo más cercano más una maniobra del catálogo.

## Decisión 3 · El troceado por sección, y lo que un PDF no puede dar

La ficha promete «documento y sección», así que el troceado es por sección y no por número de
palabras: una fuente que dijera «trozo 7» no le sirve a nadie en mitad de una reunión.

Cada formato entrega sus títulos como puede, y no todos pueden igual:

| Formato | De dónde salen los títulos | ¿Fiable? |
|---|---|---|
| Markdown | `#` y los subrayados `===` / `---` | sí, los escribió el autor |
| `.docx` | `w:pStyle`/`w:outlineLvl` **y además** negrita con cuerpo mayor que el del texto | sí |
| PDF | la **forma** de la línea: corta, sin puntuación de cierre, con cuerpo debajo | **no: es conjetura** |

Lo del `.docx` no es exceso de celo: un conversor del propio macOS **no escribe un solo
`pStyle`** y marca los títulos solo con negrita y cuerpo grande. Con la regla semántica sola, un
documento así se indexaría entero como una sección.

Y el PDF, medido: la extracción devuelve líneas planas, sin cuerpo de letra ni negrita. Por eso
`Leido::conjeturado` viaja hasta la ficha, que lo declara. **Un documento del que no se pudo
sacar ninguna sección se indexa entero y su ficha cita el documento sin sección** — menos preciso
que la promesa, pero cierto.

## Alternativas descartadas

| Alternativa | Por qué no |
|---|---|
| **BM25 a mano** en vez de `tantivy` | habría que escribir también tokenización, stemming de dos idiomas y persistencia. Medido antes de comprometerlo: `tantivy` cuesta 193 nodos y 16,7 s de compilación en frío, asumible |
| **Embeddings + RRF** ya en el S1 | la regla del código primero: BM25 es determinista y explicable. Los embeddings entran en el S2 **si el kit de evaluación demuestra** que BM25 no basta — y ahora habrá con qué demostrarlo |
| **Una librería de Word** para el `.docx` | un `.docx` es un zip con XML: se abre con `zip` + `quick-xml`. Menos superficie, y ningún escritor de `.docx` enlazado en una app que jamás escribe `.docx` |
| **Detectar el idioma** del documento y usar un solo stemmer | falla en los corpus mezclados, que son exactamente los de este usuario |
| **Un LLM** para clasificar la unidad o redactar el titular | cero LLM en este sprint, y no hace falta: el titular **se recorta del documento del usuario**, no se redacta |

## Consecuencias

- El índice guarda trozos del corpus **en claro**. Por la regla de los derivados nace en `700` y
  **se repara al abrir** si una versión anterior lo dejó flojo. Gate con su rojo: sin él, `0o755`.
- `disparo/` y `ficha/` entran en los **módulos protegidos** de `verify:ephemeral`: el disparador
  guarda la última pregunta del cliente y la ficha se arma con sus palabras. El kill-switch
  alcanza esa memoria y **pisa las letras** antes de soltarlas, igual que la ventana de turnos.
- El catálogo de maniobras vive en dos sitios —`design-system.md`, que el usuario aprobó, y el
  código— y **un test los compara palabra por palabra**. Sin él se separarían en silencio y el
  design system dejaría de describir la app.
- La latencia se mide **por aparición**, desde que el turno cierra: es el trayecto que el usuario
  percibe. Cuando pasa de 4 s se dice en el momento, no en una media que esconde los picos.

## Deuda declarada

La **maniobra genérica** —la sexta, la que sale cuando ninguna marca aparece— sigue siendo la del
catálogo fijo, y es la que más se va a disparar. El usuario ya lo señaló en la mirada 11: *«no
quiero que invente una respuesta, quiero que le sugiera cómo abordar la situación muy a medida de
la situación»*. El camino determinista está escrito en `design-system.md` §10 y es trabajo del
sprint 2.
