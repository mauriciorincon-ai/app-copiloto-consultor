# ADR 004 — El acople por Accessibility: qué se toca de las ventanas ajenas y qué no

- **Fecha:** 2026-09-20
- **Sprint:** 001 «La banda y la ficha»
- **Estado:** aceptada

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

1. **Al cerrar la banda** — `cerrar_banda`.
2. **Al salir de la app** — `RunEvent::Exit`, que cubre ⌘Q, el menú y el cierre de la última
   ventana, por donde `cerrar_banda` no pasa.
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
  problema.
- **Un espacio reservado del sistema (`NSApplication` presentation options).** Reserva franja para
  la app que la pide, no para una ventana flotante de otro proceso; no aplica.
