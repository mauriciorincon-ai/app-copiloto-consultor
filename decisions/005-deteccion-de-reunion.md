# ADR 005 — Detectar la reunión: catálogo versionado y títulos de ventana

- **Fecha:** 2026-09-21
- **Sprint:** 001 «La banda y la ficha»
- **Estado:** aceptada

## Contexto

La app tiene que saber qué videollamada hay abierta para **ofrecer** empezar —nunca para empezar
sola— y para decir si en ese cliente la protección de la banda está verificada o solo declarada.

Zoom y Teams son aplicaciones: basta con que estén corriendo. **Meet no es una aplicación, es una
pestaña**, y «Chrome está abierto» es verdad prácticamente siempre. Ahí está el problema entero.

Y choca con el ADR 004, que acaba de declarar que el acople **no lee títulos de ventana**. Si este
módulo sí los lee, la restricción de aquel se queda en nada a menos que se escriba por qué.

## Decisión

### 1 · Un catálogo versionado, no una heurística

Los clientes de videollamada se reconocen por el **identificador de su aplicación**, en una lista
escrita en `src-tauri/src/sesion/mod.rs`, versionada en el repo y **consultada sin red**. Su
versión (`v1 · 2026-09-21`) se muestra en pantalla al lado de lo que afirma: un catálogo sin
versión visible no se puede contrastar con nada.

Adivinar por nombre —«algo que contenga *meeting*»— confundiría un calendario con una llamada, y
lo haría **distinto en cada Mac**.

### 2 · Los títulos, solo de los navegadores del catálogo

Para Meet se lee el `AXTitle` de las ventanas. Pero **solo** de las aplicaciones que el catálogo
marca como navegador. Es una función pura con su test
(`solo_se_preguntan_titulos_a_los_navegadores_del_catalogo`), y sin ella «leer títulos»
significaría leer los de **todo el Mac**: el nombre de cada documento abierto, cada conversación,
cada expediente. La Accessibility API lo permitiría; el catálogo es lo que lo impide.

**Relación con el ADR 004:** el acople no lee títulos porque no los necesita —le basta la
geometría—. El detector sí, porque sin el título no hay forma de distinguir una reunión de una
pestaña de correo. Dos usos de la misma llave, y el que pide más lo justifica.

### 3 · El título es información del cliente

El nombre de una reunión —«Páramo Azul — Propuesta tablero de rentabilidad»— identifica al cliente
y el asunto. El estándar 4-T divide por **de quién** es la información, no por su formato: cae del
mismo lado que el transcript.

Por eso `sesion` entra en los **módulos protegidos** de `verify:ephemeral`, que se pone rojo si
aparece una API de disco o de red ahí dentro (demostrado en rojo con un `fs::write` plantado). El
título vive en memoria mientras la pantalla lo muestra y **no se escribe en ningún sitio**, ni
siquiera en el log de diagnóstico: ahí solo va el nombre del cliente de videollamada.

### 4 · «No lo sé» es una respuesta, y distinta de «no»

Leer títulos exige el permiso de Accesibilidad. Sin él, la respuesta **no** es «no hay reunión»
—que sería mentira por omisión— sino `NoSePuedeSaber`, con su motivo, y la pantalla lo dice.

La diferencia entre las dos es lo único que separa una app honesta de una que contesta «no»
cuando no sabe.

### 5 · La protección es graduada por cliente

Cada entrada del catálogo declara si la protección de la banda está **verificada** en ese cliente.
Hoy solo lo está en Meet (Chrome), comprobado por el usuario en su Mac con fecha. Los demás dicen
**«sin verificar»**, y un test impide que alguien declare verificado un cliente que nadie ha
mirado.

## Consecuencias

- **Una aplicación nativa gana a una pestaña:** con Zoom y Chrome abiertos a la vez, la reunión es
  la de Zoom. Es el orden del producto, no el del código, y tiene su test.
- **Limitación medida en vivo, y es real:** el `AXTitle` de una ventana de Chrome es el título de
  su **pestaña activa**. Si el consultor tiene Meet en una pestaña de fondo, el detector no la ve
  — aunque el audio siga sonando. Se descubrió probándolo: la primera prueba abrió la página en
  una pestaña de fondo y el detector, correctamente, dijo que no había reunión.
  Mitigación posible en un sprint futuro (no ahora, y no sin ADR): mirar todas las ventanas del
  navegador en vez de solo la activa ya se hace; lo que falta son las **pestañas**, y llegar a
  ellas exige automatización del navegador — bastante más invasivo que leer un título.
- **El detector se consulta, no vigila.** No hay nada en segundo plano mirando las ventanas del
  usuario: la pantalla pregunta cuando se muestra y cuando el usuario vuelve a ella.

## Alternativas descartadas

- **Detectar por uso del micrófono o de la cámara.** Diría «hay una llamada» sin decir cuál, y no
  distinguiría una reunión de una nota de voz. Además exige permisos que la app no necesita para
  esto.
- **Una extensión de navegador.** Sabría exactamente qué pestaña es, pero mete una pieza en el
  navegador del usuario, con su propio ciclo de actualizaciones y su propia superficie. Para
  decidir si mostrar un botón, es desproporcionado.
- **Preguntar al usuario siempre.** Es el fallback y sigue estando: el botón de empezar existe con
  o sin detección. Lo que la detección ahorra es el paso, no la decisión.
