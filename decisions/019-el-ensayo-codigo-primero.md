# ADR 019 — El ensayo, código primero: el banco por reglas, la evaluación determinista y el acento opt-in

- **Fecha:** 2026-10-04
- **Sprint:** 004 «El ensayo» (fase 0, antes de construir)
- **Estado:** aceptada

> Sigue la plantilla «código primero» (`decisions/PLANTILLA-ADR-codigo-primero.md`). La feature entera
> es determinista; el LLM entra solo como un acento que el usuario enciende, y su ausencia no quita
> nada.

## 1. La funcionalidad, en una frase de usuario

Antes de la reunión eliges el cliente y tu propuesta, la app te hace las preguntas que ese cliente
probablemente hará, te escucha responder en voz alta y te enseña qué evidencia de tu corpus usaste,
cuál tenías y no usaste, cuánto tardaste, a qué ritmo hablaste y qué muletillas repetiste.

**Por qué existe** (investigación científica de la planeadora, 2026-09-17): la ayuda en vivo tiene un
precio. [S13] (Chen et al., CSCW 2025): con ayuda automática en tiempo real la comprensión posterior
cae, y aun así la gente la prefiere por comodidad; el nivel intermedio fue el mejor. [S8] (Zhu et al.,
2026): la gente prefiere al asesor que propone y deja decidir, y filtra lo que se le sugiere. El ensayo
es el lado de la app que **prepara al consultor en vez de alimentarlo en vivo**: practica con su propia
evidencia, sin nadie delante, y en la reunión la necesita menos.

## 2. Lo que se intentó con CÓDIGO primero

| Intento determinista | Qué resolvió | Dónde se queda corto | Evidencia |
|---|---|---|---|
| **Banco por reglas publicadas** (`data/ensayo/reglas.json`): una pregunta por sección de la propuesta (Alcance, Entregables, Supuestos, Precio, Plazo…), por cifra («¿de dónde sale el N %?»), por compromiso o fecha, por riesgo o supuesto declarados, y por lo que la ficha de cliente dice que le importa (quién decide, acuerdos previos) | las preguntas que se desprenden de lo que **tú escribiste**: si la propuesta promete un plazo, alguien preguntará por él | una propuesta corta da pocas preguntas; las plantillas suenan genéricas; no anticipan un ángulo que el documento no menciona | kit v3 (`docs/kit-de-prueba/ensayo.json`): precisión y recall **por regla**, calculados e impresos por el test (fase 2) |
| **Catálogo bilingüe de objeciones** de datos, BI e IA (`data/ensayo/objeciones.json`), con etiquetas para elegir las que tocan esta propuesta | lo genérico del dominio: «¿estos números son correctos?», «no cuadran con el ERP», «¿quién lo va a usar?» | lo que no es genérico | cada entrada lleva su fuente del informe de mercado de la planeadora (Kuznetsova, SeattleDataGuy, Gartner, Blind, Bumeran) o «criterio del builder»; ninguna cifra inventada |
| **Evaluación determinista** de cada respuesta | cobertura de tu evidencia, tiempo, ritmo y muletillas, sin puntaje | «te faltó» puede acusar de más cuando dijiste lo mismo con otras palabras | kit v3, `respuestas` (fase 3, 2026-10-04): 7 respuestas sintéticas y 15 fichas medidas; **precisión 1,000 y recall 0,714**. Las dos que falla son una paráfrasis y una respuesta en inglés, escritas para que falle: es lo que «Sí lo dije» corrige. Ritmo y muletillas, exactos en los 7. La evaluación más lenta, 4 ms (buscar + armar + evaluar), frente a 500 ms de presupuesto |

## 3. Dónde entra el LLM y dónde NO

- **Entra en:** «Enriquecer el banco», un interruptor en IA, **apagado de fábrica**. Con él encendido,
  y una sola vez por ensayo, el modelo propone **hasta 5 preguntas más** a partir de los títulos y las
  primeras líneas de las secciones de la propuesta y de la ficha de cliente.
- **NO entra en:** elegir el cliente, armar el banco por reglas, ordenar ni deduplicar el banco final,
  leer la pregunta en voz, escuchar, cerrar la respuesta, transcribir, evaluar, guardar ni enseñar el
  progreso. Todo eso es código.
- **Entrada al modelo:** solo texto de **tus** documentos (propuesta y ficha), recortado a títulos y
  primeras líneas, **pasado por la bóveda** (`sintesis/anonimo.rs`: el nombre del cliente, correos,
  teléfonos y números largos se tapan antes de salir). Jamás audio, jamás tu respuesta, jamás nada de
  una reunión.
- **Salida del modelo:** esquema cerrado `{ "preguntas": [{ "texto": "…", "seccion": "…" }] }`. Cada
  pregunta se **funda** (grounding en dos mitades, como `sintesis/fiel.rs`): la sección que nombra
  tiene que existir en la propuesta o en la ficha y compartir con ella al menos dos términos; el texto
  termina en «?» y cabe en 25 palabras. Lo que no se funda se tira y se cuenta. En pantalla, lo que
  sobrevive va marcado «sugerida por el modelo», nunca mezclado sin marca con las reglas.

## 4. Fallback determinista

El banco por reglas **es** el ensayo; el acento solo suma. Si el interruptor está apagado, no hay
proveedor, se pasó el tope del mes, tarda más de 6 s (`sintesis::TECHO`) o devuelve algo que no se
funda, el ensayo empieza con el banco por reglas y la pantalla dice por qué no se enriqueció, en una
línea. Sin corpus para ese cliente no hay ensayo, y el estado «sin corpus» lo dice.

## 5. Proveedor, costo y privacidad

- **Orden de proveedores:** el adaptador de la síntesis (ADR 010/011), sin proveedor nuevo: `mock`
  (`AG_SINTESIS=mock`, de primera clase, con el que el kit imprime sus salidas) → el API si el usuario
  lo encendió (Claude o Groq) → el modelo del sistema, en el Mac.
- **Costo:** una llamada por ensayo, con el texto recortado. Cuenta contra el mismo tope de US$10 al mes
  y sale en «esta reunión» y «este mes» de IA. **La cifra, medida por el kit v3 (fase 2, 2026-10-04):**
  la petición de Páramo Azul son ≈ 390 tokens de entrada (instrucciones, títulos y primeras frases); con
  una salida de cinco preguntas (≈ 200 tokens), **≈ US$0,0014 con Claude Haiku y ≈ US$0,0004 con Groq**
  por ensayo. Los tokens se estiman a cuatro caracteres por token; la cifra exacta la da el proveedor al
  contestar y es la que se cobra.
- **Privacidad:** lo que salió se ve exacto en IA («Ver lo que salió», B37), con lo tapado marcado. El
  contador de red lo cuenta (`red::registrar_salida`, antes de enviar). En los logs, solo bytes y
  proveedor, jamás el texto. La retención es la que cada proveedor publica (ADR 011, estándares v2.18.0
  §4-T), y la app la enseña donde se elige el proveedor.
- **HITL:** el usuario es quien practica; ninguna pregunta del modelo llega a nadie más.

## 6. La sesión de ensayo, código de punta a punta

1. **Solo el micrófono.** El ensayo tiene su propio oído (`ensayo/oido.rs`, módulo protegido) que abre
   `Grifo::del_microfono` y nada más: ni el audio del sistema, ni la pantalla, ni el radar ámbar, que
   vive dentro de la lectura de pantalla. El radar coral de procesos de la app sigue como siempre: mira
   procesos de este Mac y no lee nada de ninguna reunión. Lo vigila un test sobre el código.
2. **La respuesta.** Cada turno del micrófono se cierra con el fin de turno de siempre (320 ms,
   `voz/turno.rs`) y se transcribe en cuanto se cierra, así que el anillo de 30 s nunca es el techo de
   una respuesta. **La respuesta** termina con un silencio de respuesta (2,5 s de partida, se ajusta
   con el kit) o con Enter.
3. **La voz de la pregunta.** Se lee con la voz al oído (`habla`), en el idioma de la propuesta. En el
   ensayo no hay reunión, así que **puede** salir por los altavoces; para que la app no se oiga a sí
   misma, **el micrófono está sordo mientras la voz habla y 300 ms después** (se descartan los marcos).
   La regla «en reunión, nunca por los altavoces» (ADR 014) no cambia.
4. **Teclas de la ventana, no globales:** Enter (listo), R (repetir), S (saltar), Esc (terminar). ⌥⎋ lo
   corta todo, como siempre: el ensayo es una pieza más del corte (`corte::Pieza::Ensayo`).
5. **Excluyente con una reunión:** empezar a escuchar corta el ensayo, y no se puede ensayar con una
   sesión abierta. Durante el ensayo la puerta local está cerrada, como en reunión: hay un micrófono
   abierto.
6. **La evaluación** (`ensayo/evaluacion.rs`, puro, ≤ 500 ms):
   - **cobertura**: las fichas que el disparo habría enseñado para esa pregunta (la misma búsqueda BM25 y
     `ficha::armar`, las tres del respaldo) frente a los términos de tu respuesta: una ficha cuenta como
     **citada** si tu respuesta comparte con ella al menos dos términos **que no estaban ya en la
     pregunta** (prefijo, sin tildes, con el diccionario técnico B3); las demás son **«evidencia que tenías
     y no usaste»**. Sin nota ni puntaje. Puedes marcar «sí lo dije», y eso manda. *La condición de la
     pregunta se añadió en la fase 3:* las fichas salen de buscar las palabras de la pregunta, así que
     repetir la pregunta en la respuesta «citaba» las tres siempre. Con preguntas genéricas (una objeción
     del catálogo) el disparo puede no encontrar ninguna ficha: entonces no hay nada que acusar y la cifra
     sale «—»;
   - **tiempo**: desde que termina la pregunta hasta que cierras la respuesta (con Enter, el instante de
     Enter; si cierra el silencio, el fin de tu voz: los 2,5 s son la espera de la app y no se te cargan);
   - **ritmo**: palabras por minuto entre la primera palabra y el fin del último turno;
   - **muletillas**: las de `data/ensayo/muletillas.json` (es/en, palabras y frases enteras). Solo las
     que la transcripción conserva: la de Apple suele quitar «eh» y «um», y la pantalla lo dice.
7. **El idioma** es el de la propuesta (palabras vacías es/en; si empatan, el tuyo en Idioma).
8. **Lo que queda** (enmienda 4 del ADR 015, que se escribe antes de la fase 4, y enmienda 8 del ADR 002): el informe del ensayo, cifrado
   con la llave de tus notas en `ensayos/`, con la retención de tus notas. Lleva tus respuestas **en
   texto** y las cifras de cada pregunta; **el audio no se guarda nunca**, tampoco el tuyo (regla dura 1).
   El progreso por cliente lee esos informes con el mismo desbloqueo que tus notas.

## Consecuencias

- La app gana un modo **sin reunión** que usa el micrófono. Es una protección del sistema usada fuera
  de su contexto de siempre: va a la matriz de la regla 22 antes de la primera corrida en vivo.
- El acento LLM no sustituye nada: un ensayo sin él es el ensayo completo.
- «Te faltó» es una pista, no un juicio. Si el kit demuestra que acusa de más, se ajusta el umbral, no
  se esconde.
- Lo que la CI no ve (tu voz, tu ritmo, que la pregunta se oiga bien) va a las ⭐ del acumulado del H2.
