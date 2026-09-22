# Angel Ghost — Manual de uso

> **Documento obligatorio y vivo.** Toda feature que llega a `main` se documenta aquí en el mismo
> sprint. Escrito para quien va a usar la app, en español llano — sin jerga técnica ni referencias
> al código.

## Qué es esta app

Angel Ghost es una ventana pequeña en tu Mac que **el cliente no ve, aunque compartas pantalla**.
Escucha los dos lados de la videollamada, busca en tus propios documentos, y cuando alguien
pregunta algo que tú ya respondiste en una propuesta, un caso o un marco tuyo, te lo pone delante:
un titular, una línea y de dónde sale.

No graba nada. Al cerrar, del audio y de lo que se dijo no queda rastro — y la propia app tiene una
pantalla para que puedas comprobarlo en vez de creértelo.

**Para quién:** consultores y asesores que ya tienen su material escrito y lo necesitan en el
momento exacto de la conversación, sin ponerse a buscar delante del cliente.

## Primeros pasos

1. **Arranca la app.** Aparecen tres cosas: el cuaderno (la ventana grande), una banda apaisada
   pegada al borde inferior de la pantalla, y el relleno que la tapa cuando compartes pantalla.
2. **Concede los permisos.** Ve a *Permisos*. La app te lleva al sitio exacto de Ajustes del
   Sistema para cada uno. **Puedes usarla sin conceder ninguno**: lo que no funcione te lo dirá.
3. **Señala tu carpeta de documentos.** Ve a *Corpus* → «Señalar una carpeta». La app lee tus PDF,
   Word y Markdown **donde están** — no los copia ni los sube a ninguna parte.
4. **Ponte los auriculares** y entra a tu reunión. En *Sesión*, pulsa «Iniciar sesión».

## Features

### La banda protegida · desde Sprint 001

- **Qué hace:** una franja en el borde inferior de tu pantalla donde aparece todo lo que la app
  tiene que decirte. **Cuando compartes pantalla, el cliente ve tu fondo de escritorio en ese
  trozo**, no la banda.
- **Cómo se usa:** aparece sola al arrancar. Arrastra el **asa** del centro hacia arriba para
  ampliarla y ver más; arrástrala hacia abajo para dejarla compacta.
- **Limitaciones conocidas:** la invisibilidad está **verificada en Google Meet sobre macOS
  26.6.2**. En Zoom y Teams la app no lo ha comprobado, y te lo dice en la propia banda con «sin
  verificar» en vez de prometértelo. Si eso te preocupa: comparte una ventana en vez de la
  pantalla completa, o usa un segundo monitor.

### El acople: la reunión se hace sitio · desde Sprint 001

- **Qué hace:** recorta por abajo la ventana de la videollamada para que la banda no le tape nada,
  como si fueran dos aplicaciones pegadas. Al cerrar la banda, la ventana vuelve a su tamaño.
- **Cómo se usa:** solo, si concediste el permiso de **Accesibilidad**. Si no, la banda flota
  encima y te avisa con «sin acople».
- **Limitaciones conocidas:** es el único permiso opcional de la app. Sin él todo lo demás
  funciona igual.

### Escucha las dos pistas · desde Sprint 001

- **Qué hace:** oye tu micrófono (eres tú) y el audio de tu Mac (es el cliente) por separado, y
  convierte los dos a texto **dentro de tu equipo**. Ningún audio sale de tu Mac para convertirse
  en texto.
- **Cómo se usa:** *Sesión* → «Iniciar sesión». Nada se enciende hasta que tú lo digas. `⌘⇧T`
  muestra u oculta el transcript en la banda; nace oculto a propósito, porque leer lo que acaban
  de decir es la forma más rápida de dejar de escuchar.
- **Limitaciones conocidas:**
  - **Usa auriculares.** Con los altavoces del Mac, tu micrófono también oye al cliente y el mismo
    turno llega por las dos pistas. La app lo detecta y lo marca como eco, pero la conversación se
    lee peor. Te avisa antes de empezar, cuando todavía puedes ponértelos.
  - Cada pista escucha **un** idioma. Marcar varios a la vez llega más adelante.
  - macOS solo deja tener **cinco idiomas de voz listos a la vez**. Es un límite del sistema.
  - Si no hablas, los contadores de audio se quedan quietos. **Eso es correcto**: cuando nadie
    habla, macOS no entrega una sola muestra.

### Tu corpus, indexado · desde Sprint 001

- **Qué hace:** lee tus documentos y los reparte en las cinco unidades del trabajo de consultoría:
  propuesta, marco, caso, ficha de cliente y tu perfil. De ahí sale cada ficha que verás en una
  reunión.
- **Cómo se usa:** *Corpus* → «Señalar una carpeta». Lee PDF, Word (.docx) y Markdown, bajando por
  las subcarpetas.
- **Limitaciones conocidas:**
  - **No lee lo escaneado.** Un PDF que es una foto de un papel se queda fuera, contado y con su
    motivo escrito. Nunca se manda a un servicio externo para que lo lea.
  - **Un PDF no trae títulos**, trae líneas: la app conjetura dónde empieza cada sección por la
    forma del texto, y te dice de cuántos documentos hizo esa conjetura para que puedas juzgarlo.
  - Hasta **2 000 documentos** por carpeta.
  - Todavía no se pueden arrastrar documentos sobre la ventana, ni releer solo lo que cambie: hoy
    se vuelve a recorrer la carpeta entera.
  - El índice vive en la carpeta de datos de la app, **y solo tu cuenta del Mac puede leerlo**.

### La ficha en el momento justo · desde Sprint 001

- **Qué hace:** cuando el cliente pregunta algo, dice una cifra, nombra algo tuyo o se queda
  callado, la app busca en tu corpus y pone en la banda un **titular de ocho palabras, una línea y
  la fuente exacta** — documento y sección.
- **Cómo se usa:** sola. Y `⌘⇧A` («ayúdame con esto») la pide a mano cuando no acierte.
- **Limitaciones conocidas:**
  - **Todo lo que la ficha dice está recortado de tus documentos.** La app no redacta: si no
    encuentra nada, lo dice.
  - Cuando no encuentra nada, enseña **qué buscó** —para que veas en el acto si te entendió mal—,
    lo más parecido que sí tienes, y una sugerencia de cómo conducirte. Esa sugerencia habla de
    **cómo responder**, jamás de tu negocio.
  - Busca por las **palabras** de la pregunta. Si el cliente pregunta algo con palabras
    completamente distintas a las de tu documento, no lo encontrará.
  - Tu propia voz no dispara fichas: la app te contestaría a ti en mitad de tu frase.

### Nada se graba, y se puede comprobar · desde Sprint 001

- **Qué hace:** el audio y lo que se dijo viven **solo en memoria** y mueren al cerrar. La pantalla
  de *Honestidad* enseña cuánto ocupan ahora mismo y cuántos bytes han salido de tu equipo.
- **Cómo se usa:** `⌥⎋` corta todo en el acto — audio, transcript, la banda y su relleno — y
  devuelve la ventana de la reunión a su tamaño. También está el botón en *Honestidad*, por si el
  atajo está cogido por otra app.
- **Limitaciones conocidas:** el corte alcanza **6 de las 7 piezas** previstas. La séptima —lo que
  la app lea de tu pantalla— todavía no existe, y por eso no dice «7 de 7».

### Español e inglés, en todo · desde Sprint 001

- **Qué hace:** la app entera, incluidos los textos que macOS te enseña al pedir permisos, está en
  los dos idiomas. Sigue el del sistema.
- **Limitaciones conocidas:** el diccionario técnico —decirle a la app cómo quieres leer un término
  que se oye de otra manera— llega más adelante.

## Atajos de teclado

| Tecla | Qué hace |
|---|---|
| `⌥⎋` | corta todo: audio, transcript, banda y relleno. Devuelve la ventana de la reunión |
| `⌘⇧A` | «ayúdame con esto»: busca una ficha sobre lo último que dijo el cliente |
| `⌘⇧T` | muestra u oculta el transcript en la banda |

> **Ojo con `⌘⇧T`:** mientras Angel Ghost esté abierto, el navegador deja de reabrir con esa tecla
> la última pestaña que cerraste. Si la usas mucho, dilo y se cambia.

## Preguntas frecuentes

**¿El cliente puede ver la banda si comparto pantalla?**
En Google Meet sobre macOS 26.6.2, no: verá tu fondo de escritorio en ese trozo. En Zoom y Teams no
está comprobado y la app te lo dice en vez de prometértelo.

**¿Sube mis documentos a algún sitio?**
No. Los lee donde están y el índice se queda en tu Mac. El contador de *Honestidad* marca los bytes
que salieron de tu equipo, y en esta versión son **0**.

**¿Guarda lo que se habla en la reunión?**
No. El audio vive treinta segundos en memoria y se va pisando; el texto, los últimos doce turnos.
Al cerrar no queda nada. Guardar **tus propios** turnos llega más adelante, y será algo que tú
enciendas.

**¿Necesito internet?**
Solo para la videollamada. La app transcribe y busca dentro de tu Mac. La única vez que toca la red
es cuando tú le pides instalar el modelo de voz de un idioma: entonces macOS lo descarga, y no sale
nada de aquí.

**¿Por qué me pide auriculares?**
Con los altavoces, tu micrófono oye también al cliente y las dos pistas se mezclan. La app lo
detecta y lo marca, pero funciona mejor con auriculares.

## Historial

| Sprint | Features añadidas a este manual |
|---|---|
| 001 | la banda protegida · el acople · las dos pistas y la transcripción local · el corpus indexado · la ficha de evidencia y la sugerencia de cómo conducirse · el corte y la pantalla de Honestidad · español e inglés |
