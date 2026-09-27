# ADR 017 — Jurisdicciones y NDA

- **Fecha:** 2026-09-27
- **Sprint:** 003 «El cuaderno y el cierre», fase 3
- **Estado:** aceptada. La pantalla «Este cliente» es FORMA nueva: se construye con lo maquetado y
  queda «maquetado, no visto» para el gate del MVP (decisión del usuario del 2026-09-27: solo abre
  parada lo que cambia una decisión suya, la promesa del producto o toca su Mac).

## Contexto

C11 es **el marco en la mano**: antes de la reunión, la regla que aplica a este cliente, con su fuente
y su fecha, una cláusula modelo para la carta de encargo y un chequeo de NDA que, si el contrato
prohíbe transcribir, pasa a **modo solo notas**. Nunca bloquea y nunca es asesoría legal.

La maqueta aprobada ya lo dibuja en tres sitios:

- `kit.html` §5, **la bandera**: cuatro variantes (riesgo bajo, medio, alto y «jurisdicción no
  indicada»), cada una con la regla, lo que implica y la fuente con su fecha;
- `sesion.html` «reunión detectada», **«Este cliente»**: la bandera, «NDA revisada: no lo prohíbe ·
  Revisar» y el radar;
- `sesion.html` «NDA prohíbe transcribir», **el modo solo notas**: qué queda apagado, qué sigue
  funcionando, «Iniciar en modo solo notas» y «Volver a revisar la NDA». Y `kit.html` §6, la barra
  «Solo notas · sin transcripción».

La fuente del catálogo es el informe legal-ético de la planeadora
(`investigacion/2026-09-17-legal-etica.md`, solo lectura): § Matriz por jurisdicción, §2.b (los
estados de todas las partes), §1.c (NDA), §7 (recomendaciones), § Gaps (17) y § Aviso. La
verificación de supuestos del plan encontró tres cosas que esta ADR resuelve:

1. **Ninguna fila de la matriz trae fecha ni URL**, y la cláusula modelo **no está redactada**.
2. En §2.b, Missouri, Hawaii y Maine se mencionan sin estatuto; y hay **17 gaps**.
3. **Ninguna parte de la app sabe quién es el cliente de negocio**: el `cliente` de la sesión es Meet
   o Zoom, y ninguna ficha del kit dice su jurisdicción.

**Cero LLM nuevo**: todo lo que sigue es código y datos.

## Decisión

### 1 · El catálogo: `data/jurisdicciones/catalogo.json`

Versionado (`version: 1`), **dentro del binario** (`include_str!`, como el del radar y el de las
propuestas): se actualiza con una versión nueva de la app, sin consultar a nadie por la red.

- **Fecha:** «consultado 2026-09-17», la del informe, en todas las filas. El informe no fecha fila a
  fila; se usa la suya y se dice.
- **Fuente:** por fila, `{norma, url}`, las normas y URLs que el informe cita en §1–§4 y en Fuentes.
  La pantalla enseña las normas; las URLs quedan en el catálogo, para quien quiera comprobarlas.
- **Cada fila en español y en inglés**, redactada y no traducida: nombre, regla, qué implica y, si lo
  hay, lo que no está verificado.
- **Lo no verificado no se afirma.** Una fila con algo sin verificar lleva su `pendiente`, con el gap
  del informe («la excepción para conversaciones profesionales solo consta en fuentes secundarias
  (G-9)»). La pantalla lo enseña debajo de las normas: «Sin verificar: …». Lo llevan 10 filas.

**Las filas (27):**

- **Colombia** · **EE. UU. (federal y estados de una parte)** · **Unión Europea (GDPR)** · Alemania ·
  Francia · España · México · Chile · Perú · Argentina: las diez filas de la matriz que no son «EE. UU.
  todas las partes», con el riesgo que la matriz les da.
- **Los 14 estados de la tabla de §2.b**, uno por fila, con su estatuto y su matiz: California,
  Delaware, Florida, Illinois, Maryland, Massachusetts, Michigan, Montana, New Hampshire, Pennsylvania
  y Washington (todas las partes), y Connecticut, Nevada y Oregon (mixtos). Riesgo medio-alto, el de
  la fila «EE. UU. all-party» de la matriz.
- **Missouri, Hawaii y Maine**: el informe los nombra (mixto; por lugar) **sin estatuto**. Entran con
  riesgo «sin verificar» y sin regla afirmada: «el informe los nombra sin verificar su estatuto».

**El riesgo** es el de la matriz, con su palabra: bajo · bajo-medio · medio · medio-alto · sin
verificar. Símbolo, texto y color (regla 8): bajo y bajo-medio en verde con ✓; medio y medio-alto en
ámbar con ⚠; sin verificar en gris con ◯.

**Test:** el catálogo se lee entero; cada fila trae los dos idiomas, al menos una fuente con norma y
URL `https://`, y la fecha; ninguna fila «sin verificar» sale sin su `pendiente`; no hay dos filas que
compartan un alias (el gate se puso en rojo solo al nacer: «oregón» y «oregon» eran el mismo alias una
vez plegados).

### 2 · La jurisdicción de un cliente sale de SU ficha

Una línea en la ficha de cliente del corpus:

```
Jurisdicción: Colombia
Jurisdiction: Florida
```

- Al indexar, `corpus/` lee esa línea de cada documento de la unidad «cliente» y la guarda con el
  documento, **solo en memoria** (no viaja a la pantalla del corpus ni al índice).
- `jurisdiccion/` (puro, sin disco) la compara con los alias del catálogo, **sin mayúsculas ni
  tildes** («colombia», «co», «EE. UU.», «florida»…). Los estados de EE. UU. no llevan su código de dos
  letras: «CA» sería California y Canadá; «CO», Colorado y Colombia.
- **Tres resultados**, los tres con forma en `kit.html` §5:
  - **conocida** → la bandera de su fila;
  - **no está en el catálogo** («Jurisdicción: Bolivia») → sin bandera, y lo dice: «“Bolivia” no está
    en el catálogo v1». No se adivina la más parecida;
  - **no indicada** → «Jurisdicción no indicada · Indica dónde está la contraparte en la ficha del
    cliente · Mientras tanto, la app funciona igual».
- **Varias en una línea, la más estricta.** «EE. UU. · Florida» o «Colombia y California»: la línea se
  parte por sus separadores y gana la de más riesgo, porque el informe dice que, con las partes en
  sitios distintos, suele aplicar la ley más estricta (§2.b). «Sin verificar» cuenta como la más
  estricta: lo que no se sabe no se trata como si fuera leve.
- La ficha del kit de prueba (`Ficha de cliente · Páramo Azul.md`) gana `Jurisdicción: Colombia`.

### 3 · «Este cliente» en Sesión

- Un **selector** con los clientes del corpus (`corpus::clientes`, los mismos que tapan la bóveda del
  API y escribe bien el diccionario), más «Sin elegir».
- **La elección vive en memoria**, mientras la app esté abierta: no se guarda en disco. Nombra el
  archivo de la reunión (`paramo-azul-2026-09-27.ghost`, ADR 015 §2, que esperaba esta fase) y elige
  la bandera y la NDA.
- Debajo del selector: la bandera (con sus normas y la fecha del informe); la fila de la NDA; y una
  fila con **«No es asesoría legal»**, siempre visible, y el botón **«Cláusula de encargo»**.
- **La pantalla vuelve al diseño aprobado de la Etapa de Diseño** («reunión detectada»): la tarjeta
  «Qué funciona hoy» de los sprints 1 y 2 era el andamio de un producto a medias, y con el H1 entero no
  le queda nada en «todavía no». Los botones van en su fila, como en aquella maqueta. La línea «Escucha
  las dos pistas · A medias» (mirada 17) se queda, en la tarjeta de las pistas, y solo cuando una cae.

### 4 · El chequeo de NDA

- **Por cliente, tres estados:** sin revisar · no lo prohíbe · lo prohíbe.
- «Revisar» abre la pregunta, que es la del informe (§1.c, A7): **«¿La NDA de este cliente prohíbe
  grabar o transcribir por cualquier medio?»** — «Sí, lo prohíbe» · «No lo prohíbe». Ocupa **la fila
  de los botones**, a lo ancho: se contesta justo antes de «Iniciar sesión». (Dentro de la tarjeta no
  cabía en los 640 px de la ventana; lo midió `maqueta-cabe`.)
- **La respuesta se guarda en tus preferencias**, por cliente (`ndas`, en `prefs.json`, 600): la NDA
  no cambia de una reunión a otra. «Volver a revisar la NDA» la borra y vuelve a preguntar.
- **«Lo prohíbe»** convierte Sesión en el estado aprobado «NDA prohíbe transcribir»: qué queda
  apagado, qué sigue funcionando, «Iniciar en modo solo notas» y «Volver a revisar la NDA». **Nunca
  bloquea**: la decisión es tuya y se cambia con un clic.
- Al log va el hecho, jamás el nombre del cliente: `[nda] respuesta guardada: lo prohíbe`.

### 5 · El modo solo notas

Se entra con «Iniciar en modo solo notas» (NDA) o con **«Solo notas»** en Sesión, por decisión tuya
(`kit.html` §6: «NDA prohíbe transcribir · o por decisión tuya»).

- **Qué se abre:** la reunión (el cuaderno, protegido de la captura como en una sesión normal) y la
  banda.
- **Qué NO se abre, ni una vez:** micrófono, audio del sistema, transcripción, lectura de pantalla
  (ni la automática ni `⌃⌥L`) y el radar ámbar, que lee la ventana de la reunión. Lo decide **una
  función pura** (`que_se_abre(modo)`) que `empezar` obedece; su test se demuestra en rojo abriendo el
  micrófono en solo notas.
- **Qué sigue:** tus notas y acuerdos (`⌃⌥N`), fijar (`⌃⌥P`), el radar coral (mira los procesos de tu
  Mac, no la reunión), el contador de red en 0 y `⌥⎋`.
- **`⌃⌥A` busca en tu corpus con la última línea de tu nota**, porque no hay turno del cliente con
  que buscar. Escribes «ETL con tres fuentes», pulsas `⌃⌥A` y la banda trae la ficha.
- **La banda lo dice:** «Solo notas · sin transcripción», con el símbolo de la nota (kit §6).
- «Terminar sesión» la cierra como cualquier otra: «al cerrar» en Notas.

### 6 · La cláusula modelo

En el catálogo, en español y en inglés, **redactada desde el informe** (§1.c, A6 y §7.b): autoriza
«asistencia de IA local, sin grabación ni retención», dice qué se procesa y dónde, qué persiste (tus
notas, cifradas) y qué sale del equipo (nada por defecto; con el API encendido, solo texto minimizado
y sin datos que identifiquen, a un proveedor sin retención), y que el cliente puede pedir que no se
use.

- Sesión la enseña con **«Ver la cláusula»**: las dos versiones, lado a lado, cada una con su
  «Copiar». Se copia la que va con la carta de encargo, no la del idioma de la interfaz.
- Lleva siempre: **«Plantilla, no asesoría legal: revísala con tu abogado.»**
- Es texto nuevo: va al bloque de textos del ⭐⭐.

## Lo que queda fuera, dicho

- **La plantilla del aviso de una línea al cliente** (A5) y **el registro de que informaste al
  cliente** (A6): la bandera sugiere el aviso, pero su texto y su registro quedan para H2.
- **Actualizar el catálogo sin actualizar la app:** nunca. Consultar la red para esto rompería la
  regla dura 2.

## Alternativas consideradas

- **Preguntar la jurisdicción en la pantalla, en vez de leerla de la ficha:** otra cosa que rellenar
  antes de cada reunión, y se olvida. La ficha es donde el consultor ya describe a su cliente.
- **Adivinar la jurisdicción por el idioma del cliente o por el nombre de la empresa:** es afirmar
  lo que no se sabe, que es lo que el informe prohíbe.
- **Bloquear la sesión si la NDA lo prohíbe:** la maqueta aprobada dice «nunca se bloquea sola», y
  la NDA la interpreta el consultor, no la app.
- **Guardar la elección de cliente en disco:** ahorraría un clic, pero pondría en disco, sin que lo
  pidas, con quién te reuniste la última vez. La NDA sí se guarda, porque la pediste tú.
- **Traer el catálogo de un servidor:** regla dura 2.

## Consecuencias

- Módulo nuevo **`jurisdiccion/`**, puro: entra en los protegidos de `verify:ephemeral` (no necesita
  disco ni red; el catálogo va dentro del binario).
- `corpus::Documento` gana la jurisdicción de la ficha, sin serializarla.
- `prefs.json` gana `ndas` (ADR 002, enmienda 5): nombres de tus clientes, en tu Mac, 600.
- `EstadoDeEscucha` gana `soloNotas`, y el contrato Rust→TS la vista de «Este cliente» (regla 19).
- Comandos nuevos, solo en la ventana principal: el cliente, la NDA y empezar solo notas.
- El manual gana «El marco en la mano» con sus limitaciones.
- **Parada del ⭐⭐:** elegir Páramo Azul ve la bandera de Colombia; «Sí, lo prohíbe» lleva al modo solo
  notas y la banda lo dice; `⌃⌥A` trae una ficha desde tu nota.
