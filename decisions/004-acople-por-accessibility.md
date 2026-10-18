# ADR 004 — El acople por Accessibility: qué se toca de las ventanas ajenas y qué no

- **Fecha:** 2026-09-20
- **Sprint:** 001 «La banda y la ficha»
- **Estado:** aceptada, con la enmienda 1 (sprint 004, 2026-10-04): con la banda arriba, la reunión baja y se encoge

## Contexto

La banda vive pegada al borde inferior de la pantalla y **siempre encima**. Sin hacer nada más, se
queda tapando los 88 px de abajo de la ventana de la videollamada: justo donde Meet y Zoom ponen
los controles. La promesa del producto —«en la reunión, en el momento justo»— se rompe si para
leer la banda hay que perder el botón de silenciar.

La única forma de que la reunión se haga sitio es **cambiar el tamaño de una ventana que no es
nuestra**, y en macOS eso solo se puede hacer por la **Accessibility API**. Es la operación más
invasiva de toda la app: la misma llave que mueve una ventana lee el árbol entero de cualquier
aplicación, sus títulos y su contenido. Y choca de frente con la regla dura nº 9 de esta app
—cero acceso a máquinas ajenas— si no se acota por escrito qué se hace con ella.

Hay además un riesgo operativo que el plan del sprint ya había nombrado (riesgo nº 2): **si
Angel Ghost muere con una ventana encogida, esa ventana se queda encogida** y el usuario no sabe
quién se la rompió.

## Decisión

**Se usa la Accessibility API, acotada a tres atributos de lectura y uno de escritura**, con la
devolución garantizada por tres caminos independientes.

### Lo que se pide

| Atributo | Para qué | Lectura / escritura |
|---|---|---|
| `AXWindows` | la lista de ventanas de una aplicación | lectura |
| `AXPosition` | dónde está cada una | lectura |
| `AXSize` | cuánto mide | lectura **y** escritura |

Y nada más. Ni `AXTitle` de ventana, ni `AXChildren`, ni el árbol de elementos, ni el contenido de
ningún campo. La API los daría; no se piden.

### Lo que NO se hace, pudiendo

- **No se leen títulos de ventana.** El nombre de una reunión es información del cliente. El
  estándar 4-T divide por **de quién** es la información, no por su formato, así que un título de
  ventana cae del mismo lado que el transcript del cliente. La huella que el acople guarda en
  disco **no tiene campo donde escribirlo**, y hay un test que lo afirma para que añadirlo «para
  depurar» tenga que pasar por encima de un motivo escrito.
- **No se enumera el sistema.** Se pregunta por **una** aplicación: la que está al frente (fase 1)
  o la que la detección de reunión señale (fase 2). No hay recorrido de procesos.
- **No se mueve nada.** Solo se escribe el alto. La posición no se toca: una ventana que se
  desplaza sola es un susto; una que se acorta por abajo es una ventana que cabe.
- **No se mutila.** Si encoger dejara la ventana por debajo de 240 px, no se toca y la banda flota
  encima, diciéndolo. Preferimos una banda que estorba a una reunión que no se puede usar.
- **No se toca lo que el usuario tocó después.** La devolución solo actúa sobre una ventana que
  siga **exactamente** como la dejamos. Si el usuario la redimensionó a mano mientras tanto, su
  gesto manda sobre nuestro registro.
- **No se cuenta lo que no pasó.** El tamaño se **relee del sistema** después de escribir. Si la
  aplicación acepta el cambio y se queda igual —pantalla completa, Split View, tamaño fijo— eso no
  es un acople: no se anota huella y la banda dice «sin acople». *(Se encontró en vivo: una
  ventana quedó registrada como `923 → 923`.)*

### La devolución, por tres caminos

1. **Al cortar (`⌥⎋`)** — la pieza `Acople` del kill-switch (`lib.rs`, `corte::Pieza::Acople`),
   que es también la única forma de cerrar la banda sin salir de la app: el comando `cerrar_banda`
   no tenía quien lo llamara y salió en la auditoría del S2 (B8).
2. **Al salir de la app** — `RunEvent::Exit`, que cubre ⌘Q, el menú y el cierre de la última
   ventana, por donde el corte no pasa.
3. **Al arrancar la vez siguiente** — la **huella** en disco, que es lo único que cubre la caída y
   el force quit. Probada en vivo: matar el proceso dejó dos ventanas encogidas y el arranque
   siguiente devolvió a su tamaño exacto la que seguía existiendo.

La huella guarda PID, nombre de la aplicación y dos rectángulos. El nombre es un **cerrojo contra
el reciclado de PID**: tras una caída ese número puede pertenecer ya a otro programa, y devolverle
un tamaño a la ventana de otro sería el peor fallo que este módulo puede cometer. Nace `600` en
una carpeta `700`, con test demostrado en rojo (regla 17-bis a).

### El permiso

**No se pide al arrancar sin más.** Si no está concedido, la banda **flota** y lo dice en su
cabecera: «sin acople» es un estado dibujado en la maqueta, no un error. El permiso se pide una
vez, con el diálogo propio de macOS, y hasta que el usuario vuelva de Ajustes del Sistema la
respuesta honesta es que no lo tenemos.

## Consecuencias

- El acople **no siempre funciona, y eso es parte del diseño**: pantalla completa, Split View y
  ventanas de tamaño fijo se quedan como están. La banda lo refleja.
- «Acoplada» en la cabecera de la banda es un **hecho comprobado** —sale del mismo archivo de
  huella que usa la devolución— y no una etiqueta.
- Queda un hueco declarado: si el proceso muere mal **y Angel Ghost no se vuelve a abrir**, la
  ventana sigue corta hasta que se abra. Se puede cerrar con un hilo en `sigwait` para
  SIGTERM/SIGINT; no se hizo porque el plan resuelve la caída por la huella.
- En `pnpm tauri dev` la Accesibilidad puede venir heredada del **proceso responsable** (la
  terminal). El binario firmado pedirá la suya: es parada del gate ⭐, no un matiz.

## Alternativas descartadas

- **Dejar la banda flotando siempre.** Es el fallback, no la opción por defecto: obliga al usuario
  a mover su reunión a mano en cada llamada.
- **Reducir la ventana de la reunión moviéndola hacia arriba.** Cambia dos cosas en vez de una y
  el usuario ve su ventana «saltar». Encoger por abajo es el mínimo cambio que resuelve el
  problema. *(Con la banda abajo sigue descartada. Con la banda arriba es la única forma: ver la
  enmienda 1.)*
- **Un espacio reservado del sistema (`NSApplication` presentation options).** Reserva franja para
  la app que la pide, no para una ventana flotante de otro proceso; no aplica.

---

## Enmienda 1 — con la banda arriba, la reunión baja y se encoge, y se devuelve entera (sprint 004, fase 0, 2026-10-04)

**Por qué.** La VISION v1.5.0 pone la banda **arriba por defecto**, bajo la barra de menús, junto a la
cámara (decisión del usuario, 2026-09-27): leer la ficha abajo se ve como mirar otra cosa; arriba, como
mirar al cliente. Con la banda arriba, encoger por abajo no sirve de nada: lo que la banda tapa es el
borde **superior** de la reunión (la barra de pestañas, el título, los controles de arriba). La única
forma de que la reunión se haga sitio es **bajar su borde superior**: moverla y encogerla a la vez,
dejando el borde inferior donde estaba.

**Lo que se pide ahora:**

| Atributo | Para qué | Lectura / escritura |
|---|---|---|
| `AXWindows` | la lista de ventanas de una aplicación | lectura (igual) |
| `AXPosition` | dónde está cada una | lectura **y, solo con la banda arriba, escritura sobre la ventana acoplada** |
| `AXSize` | cuánto mide | lectura y escritura (igual) |
| `AXMainWindow` | cuál es la ventana principal de Zoom o Teams | lectura (nuevo) |

Sigue sin pedirse `AXTitle` desde el acople, ni el árbol, ni el contenido de ningún campo.

**Una sola ventana, la de la reunión.** Abajo, el acople sigue como en el H1: la aplicación que está al
frente y sus ventanas que tocan la franja (el andamio de la fase 1 que este ADR declaró). **Arriba no:**
mover la ventana de otro es más invasivo que acortarla, y mover el editor de alguien porque estaba
delante sería un susto. Arriba, el destino es **la ventana de la reunión que nombra la detección**
(ADR 005): en Zoom o Teams, su `AXMainWindow`; en un navegador con Meet, la ventana cuyo título lleva las
señales de Meet, que encuentra **el detector** —que ya lee títulos, solo en memoria— y le pasa al acople
por su posición en la lista. El acople sigue sin leer un título. **Sin reunión detectada no hay acople**:
la banda flota arriba y lo dice.

**Cómo se baja y se encoge:**

1. La ventana invade la franja si su borde superior queda por encima del borde inferior de la banda. Si
   no la invade, no se toca.
2. Nuevo borde superior = el borde inferior de la banda; nuevo alto = el que deja el borde inferior
   donde estaba. Si ese alto queda por debajo de 240 px, no se toca y la banda flota (no se mutila).
3. **Primero el tamaño, después la posición:** la ventana se acorta por abajo (no sale de la pantalla)
   y luego baja hasta la banda. En ningún instante queda fuera de la pantalla.
4. Se **relee** del sistema. Si macOS no lo dejó (pantalla completa, Spaces, una app que pelea su
   posición), se deshace lo que sí se hizo y la banda flota. No se cuenta lo que no pasó.
5. `AXPosition` se escribe en **un solo sitio** (`acople/ax.rs`, `mover`), y un test lo barre.

**La devolución, entera.** La huella **ya guardaba los dos rectángulos completos** (posición y tamaño),
así que no cambia de versión. Se devuelve lo que cambió: **primero la posición** (sube), **después el
tamaño** (crece), y solo si la ventana sigue exactamente como la dejamos —los cuatro números, con 2 pt de
tolerancia—. Si el usuario la movió o la redimensionó mientras tanto, su gesto manda. Los tres caminos
siguen: al cortar (`⌥⎋`), al salir y al arrancar. Una huella del H1, que solo cambió el alto, se
devuelve igual que antes.

**Lo que no cambia:** el acople no lee títulos, no enumera el sistema, no mutila, no toca lo que el
usuario tocó después y no cuenta lo que no pasó. Abajo, todo sigue como en el H1 (los 13 tests de hoy,
intactos).

**Cómo se prueba (fase 1):** la geometría pura arriba (tres pantallas sintéticas: 1440×900, con notch,
externa desplazada); de una lista de ventanas sale **una sola** decisión; la devolución doble y su
idempotencia; la huella del H1; la ventana que el usuario movió no se devuelve; el barrido de
`AXPosition`. **En vivo, una vez, con la matriz de la regla 22 y el «sí» del usuario** antes de mover
una ventana real (la página de Meet de prueba del kit): baja 88 px, vuelve al cerrar, ≤ 300 ms. Y
mirada de DECISIÓN sobre la variante «arriba» de `docs/diseno/posicion.html` antes del código.

## Enmienda 2 — en Windows, `SetWindowPos` y la misma maniobra (sprint 008, 2026-10-12)

En Windows no hay Accessibility API ni permiso que pedir: el mismo usuario puede mover y redimensionar las ventanas
de sus otros programas con `SetWindowPos`. `acople/windows.rs` da **las mismas funciones** que `ax.rs` (leer las
ventanas de un proceso, encoger, mover, releer, saber quién está al frente y si el proceso sigue siendo el mismo), y
la maniobra entera de `acople/mod.rs` corre igual en los dos sistemas: soltar primero, leer lo que de verdad quedó,
deshacer lo que no salió y anotar la huella para devolverla al cerrar o tras una caída.

Lo que cambia, y por qué: se mide **lo que se ve** de la ventana (`DWMWA_EXTENDED_FRAME_BOUNDS`), no su rectángulo,
que incluye los bordes invisibles de redimensionar, y se escribe el rectángulo con esos bordes por fuera; las medidas
pasan a puntos con la escala del monitor de cada ventana, que es como Tauri da la franja; y el índice de una ventana
es su posición entre las de su proceso ordenadas por identificador, que no cambia con cada clic. La conversión es
pura y tiene su test. Como aquí, **no se lee ningún título ni contenido**: la ventana de la reunión la elige la
detección (`sesion::windows`), donde sí se leen, y llega como un número. Detalle en el ADR 022, enmienda 2.
## Enmienda 3 — la banda a tu altura, y su «×» (sprint 011, 2026-10-18)

**El problema, medido en la Surface del usuario.** Arriba la banda ocupa el ancho entero bajo el borde superior del área
útil, y el acople solo mueve **la ventana de la reunión detectada, con la sesión iniciada** (enmienda 1). Antes de la
llamada —al entrar a Chrome para iniciar sesión en la videollamada— no hay reunión que mover y la banda flota encima de
las pestañas y los controles del navegador. Y sin reunión no había forma de quitarla sin cortarlo todo (`Alt+Mayús+Q`)
o cerrar la app.

**La altura (decisión 2a del G-Plan, 2026-10-18).** La banda sigue arriba y de ancho completo, pero se desplaza **solo en
vertical** con un asa de posición, y la altura se guarda (`desplazamientoDeLaBanda`, ADR 002 enmienda 10):

- **Una sola geometría.** `geometria::franja_desplazada` baja la franja ese desplazamiento desde el borde superior del área
  útil, **acotado** a que la banda quede entera dentro y con `HUECO_BAJO_LA_BANDA` (44 pt) libres debajo. Se acota al
  calcular, no solo al guardar: agrandar con el asa de alto una banda bajada no la saca de la pantalla.
- **Con 0 es la de siempre** (test `con_desplazamiento_0_es_la_franja_de_hoy`): quien no la baja, en el Mac y en Windows,
  no ve nada distinto. **Abajo el desplazamiento no se mira.**
- **El acople no cambia de regla.** `decidir_arriba` ya bajaba la reunión hasta `franja.fondo()`; con la franja
  desplazada, la reunión queda bajo la banda esté donde esté. Si debajo no cabe una reunión usable (`ALTO_MINIMO_UTIL`),
  no se mutila: la banda flota y el informe lo dice.
- **La pareja de siempre.** Mientras arrastras se mueve lo nuestro (`desplazar_banda`); al soltar se guarda acotado y la
  reunión, si estaba acoplada, se reacopla (`asentar_desplazamiento`), por la misma puerta que el asa de alto
  (`reacoplar_bajo`, que pregunta por el modo presencial antes de tocar nada).

**El «×» (decisión 1a).** `cerrar_la_banda`, en este orden y con el turno de los cambios de borde: con una sesión en
marcha la **termina** como «Terminar sesión»; **suelta** la ventana de la reunión (vuelve a su sitio); **cierra** la
banda (y su relleno, en el Mac); **avisa** a Sesión. La app sigue viva —la ventana principal y la bandeja—, y «Mostrar
la banda» (Sesión, `mostrar_la_banda`, asíncrono porque en Windows crear una ventana desde un comando síncrono se
bloquea) o «Iniciar sesión» la vuelven a abrir a su altura. El corte también avisa de que la banda se cerró.

**Un hecho que esta enmienda deja escrito, porque los manuales lo decían distinto:** «Terminar sesión» **no** suelta la
ventana de la reunión —la reunión sigue abierta hasta que la guardes o la descartes—. La sueltan cambiar de borde, el
modo presencial, el corte, el «×», salir de la app y la recuperación tras una caída.

**Plataformas.** El desplazamiento nace en 0 y no cambia nada en macOS; el asa de posición, el «×» y la fila de Sesión se
enseñan solo en Windows (sprint 011), donde se pidieron.

**Lo que pagó la auditoría del sprint** (cada una con su prueba en `lib.rs` o en `la-banda-a-tu-altura.test.tsx`):

- **«Iniciar sesión» tras el «×» no crea la banda en el hilo del comando.** En Windows crear una ventana desde un comando
  síncrono se bloquea (WebView2), y tras el «×» o el corte ese es el camino de siempre. `empezar` repone la banda en otro
  hilo y el acople, que sin banda no se calcula, corre después en el mismo. En el Mac sigue síncrono, como estaba.
- **La altura guardada es la que cabe.** Agrandar la banda con el asa de alto la deja más arriba de lo guardado; al soltar,
  `asentar_banda` guarda lo acotado al alto nuevo, y Sesión y el siguiente arrastre parten de donde la banda está.
- **Asentar la altura toma el turno de los cambios de borde, en otro hilo**, como el «×».
- **Sin arrastrar** (WCAG 2.5.7): «Bajar» y «Subir» en Sesión mueven la altura de 40 en 40 puntos; el «×» mide 24 px
  (WCAG 2.5.8); un `pointercancel` suelta el asa como un `pointerup`.
