# ADR 010 — La sugerencia sintetizada (C7) con IA generativa: por qué el código no alcanza

> Desde la plantilla del kit (`decisions/PLANTILLA-ADR-codigo-primero.md`), escrita **antes de una
> línea de código** de la síntesis (sprint 002, fase 5). Se cita por su tema: **«síntesis, código
> primero»**. Su hermano es el ADR 011, **«proveedores del modelo y minimización»**.

**Estado:** aceptado · **Fecha:** 2026-09-26 · **Sprint:** S2 «Cuándo, qué y quién mira», fase 5

## 1. La funcionalidad, en una frase de usuario

Cuando el cliente pregunta algo y la app ya encontró las fichas de tu corpus, **te propone qué
decir en una frase** —«Confirma que las tres fuentes están dentro; una cuarta va como adicional
con costo aparte»—, **debajo de la ficha y citando de cuál salió**, para que no tengas que componer
la respuesta con tres fichas a la vista mientras el cliente espera.

## 2. Lo que se intentó con CÓDIGO primero (obligatorio, con evidencia)

| Intento determinista | Qué resolvió | Dónde se quedó corto | Evidencia |
|---|---|---|---|
| **Recuperación** (BM25 por sección + disparo por pregunta, cifra, término o silencio) | encontrar la evidencia propia en el momento justo | nada: sigue siendo quien decide **qué** se enseña | nDCG@5 **0,823** en el kit v0 (30 preguntas; mínimo 0,80) — `SPRINT_001-summary.md`; con la pantalla, 0,700 → sin ella 0,626 en el kit de pantalla v1 |
| **La ficha** (titular de la sección o primera frase, línea = la frase que más responde; recorta y cita, no redacta) | **qué dice tu documento**, con su fuente, sin inventar | responde «qué dice el corpus», no **«qué le digo al cliente ahora»**: con dos o tres fichas a la vista, componer la respuesta sigue siendo trabajo del consultor en mitad de la frase del cliente | `src-tauri/src/ficha/mod.rs`; la negativa del kit: **0** fichas inventadas en las preguntas sin respuesta |
| **La maniobra** (catálogo versionado de seis maneras de responder cuando no hay nada) | qué hacer cuando el corpus **no** tiene la respuesta | es genérica por diseño: no puede usar el contenido de las fichas, porque su caso es que no hay | `src-tauri/src/ficha/` (C17, mirada 11) |
| **Plantillas** («Confirma {titular}; {línea}») sobre la ficha | una frase con forma de sugerencia | repiten la ficha con otro verbo; no **eligen** entre dos fichas, no **responden a la pregunta concreta** («¿y si sumamos el Excel de la fuerza comercial?» → la respuesta está en «una cuarta fuente es adicional», que la plantilla no sabe relacionar con «sumar un Excel») | **no se construyó, y se dice**: una plantilla solo puede reordenar el texto de UNA ficha, así que por construcción no relaciona la pregunta con otra ficha ni elige entre dos. Es un argumento, no una medición |

**Lo que el código no hace, en una frase:** relacionar la pregunta **tal como la dijo el cliente**
con **varias** fichas y redactar la frase que el consultor diría. Eso es lenguaje, no recuperación.

## 3. Dónde entra el LLM y dónde NO

- **Entra en:** redactar **una** frase de sugerencia (titular corto + línea) a partir del último
  turno del cliente y de las fichas **ya recuperadas** (top-3). Nada más.
- **NO entra en:** el disparo, la búsqueda, qué fichas se enseñan, la ficha misma, la maniobra, la
  lectura de pantalla, el radar, la voz, el diccionario, lo que se guarda ni lo que se borra.
  **La ficha se enseña primero y sin esperar al modelo**; la sugerencia llega después, debajo.
- **Entrada al modelo:** el último turno del cliente y el titular, la línea y la fuente de cada
  ficha del top-3 — texto corto. **Jamás** audio, imagen de la pantalla ni documentos enteros: el
  camino no existe en el código (lo vigilan `verify:ephemeral` y el gate del contador de red). Al
  API opt-in, además, **anonimizado en el Mac** antes de salir (ADR 011).
- **Salida del modelo:** un esquema cerrado `{titular, linea, fuente, confianza}`, validado en
  Rust (serde) y cruzando la costura con fixture real (regla 19). **La regla dura que no compila:**
  una `Sugerencia` solo se construye con `fundar()`, que exige que `fuente` sea **exactamente** una
  de las fichas que se le dieron; si no lo es, la sugerencia se **descarta** (test). No hay otra
  forma de crear el tipo que la banda recibe.
- **Enmienda (fase 6, al medir el modelo real): citar bien no basta.** `fundar()` exige además que la
  línea **diga lo que dice la ficha citada** (`sintesis/fiel.rs`, determinista y léxica): cada palabra
  con contenido sale de esa ficha, las cifras son exactas y no se pone ni se quita una negación. La
  primera medida con Apple Intelligence dio 17 de 17 citas válidas y **3 líneas falsas o ajenas** a su
  ficha («el taller de cierre va aparte», cuando la propuesta lo incluye). Con la regla, 11–13 de 17
  pasan y las falsas se tiran. El prompt pide ahora escribir **con las palabras y en el idioma de la
  ficha citada** (antes: en el de la pregunta), porque una línea traducida no se puede cotejar. El
  titular que no sale de la ficha ni de la pregunta se cambia por el de la ficha.

## 4. Fallback determinista (obligatorio)

La ficha y la maniobra, **como hoy**. La sugerencia es un acento: si el modelo no está, no hay
clave, tarda más de **6 s** (el techo; la mediana presupuestada es ≤4 s), devuelve algo que no
encaja en el esquema o cita una fuente que no se le dio, **no hay sugerencia y la ficha sigue ahí**.
El motivo se anuncia en la pantalla IA (qué proveedor, por qué no está) y va al log como metadata
(nunca el texto). **Redactar sugerencias nace apagado**: la app es entera sin ello.

## 5. Proveedor, costo y privacidad

- **Orden de proveedores (ADR 011):** (a) el modelo del sistema (Apple Foundation Models, en el Mac)
  → (b) MLX con un modelo que el usuario descarga a su carpeta → (c) API opt-in con la clave del
  usuario en el Llavero. **`mock` es un proveedor más, de primera clase, dentro del adapter** (por
  env, `AG_SINTESIS=mock`), y es el de la CI: jamás se intercepta la red.
- **Costo medido:** (a) y (b) cuestan US$0. (c) se mide por petición (tokens de entrada y salida ×
  precio del proveedor) y se suma por reunión y por mes; **techo US$10/mes** por defecto, visible
  en IA. Al llegar al techo vuelve sola a lo local; la reunión no se detiene.
- **Privacidad:** al log solo metadata (proveedor, bytes, ms, si se descartó y por qué) — un término
  plantado en una sugerencia no puede aparecer en el log (test). El registro de lo que salió vive en
  memoria y muere con la reunión; lo que persiste es la cifra del mes, no el texto.
- **HITL:** el consultor es quien decide qué decir. La sugerencia se enseña como tal, con su
  fuente y su confianza, **debajo** de la ficha que la respalda.

## Consecuencias

- **Gana:** la frase accionable que la ficha sola no da, sin perder la garantía de la ficha: toda
  sugerencia cita una ficha que el consultor está viendo.
- **Acepta:** latencia de un modelo en un Mac que además transcribe y lee la pantalla (presupuesto
  medido en el kit); calidad en español del modelo local **medida, no supuesta**; y, si se enciende
  el API, texto anonimizado fuera del Mac con un contador que deja de ser cero.
- **Verifica:** la prueba ⭐ «sugerencia local a tiempo y sin tapar la ficha» (gate del MVP) y el kit
  de sugerencias en CI con `mock` (grounding y presupuesto).

## Medición (se llena al construir; nada se afirma antes)

Ver la sección «Medido» de la fase 5 en `sprints/SPRINT_002-implementation-log.md`, y la entrada
«Fase 6 — el modelo del sistema, medido» para Apple Intelligence: mediana ~0,8 s, p95 ~1 s,
11–13 de 17 sugerencias pasan el contrato completo. **MLX no se construye**: el proveedor (a) cumple el
presupuesto con holgura y el (b) solo tenía sentido si (a) no estaba (ADR 011); queda en el roadmap
para Macs sin Apple Intelligence.
