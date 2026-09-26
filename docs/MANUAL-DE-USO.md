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
- **Cómo se usa:** *Sesión* → «Iniciar sesión». Nada se enciende hasta que tú lo digas. `⌃⌥T`
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

- **Qué hace:** cuando el cliente pregunta algo, dice una cifra, nombra algo que está en tus
  documentos **o se queda callado después de hablar**, la app busca en tu corpus y pone en la banda
  un **titular de ocho palabras, una línea y la fuente exacta** — documento y sección.
- **Cómo se usa:** sola. Y `⌃⌥A` («ayúdame con esto») la pide a mano cuando no acierte.
- **El silencio también pide ficha · nuevo en Sprint 002.** Si el cliente dice algo que no lleva
  pregunta ni cifra y se queda callado **cuatro segundos**, la app busca por su cuenta lo último que
  dijo — es el hueco en el que te toca hablar a ti. Dos cosas que hace bien y conviene saber:
  **mientras el cliente siga hablando no cuenta como silencio** (aunque su frase anterior cerrara
  hace rato), y **no repite la ficha que ya está en la banda**, así que una pregunta que ya trajo su
  ficha no trae una segunda al callarse. En el sprint 001 esto estaba escrito en el código y sin
  conectar, y este manual lo declaraba como limitación; ahora funciona.
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
- **Limitaciones conocidas:** el corte alcanza **7 de las 8 piezas** previstas. La octava —lo que
  la app lea de tu pantalla— todavía no existe, y por eso no dice «8 de 8». *(Eran 7 hasta el
  sprint 002: la voz del modo solo audio es una pieza más, y es la primera que se corta, porque es
  la única que tu cliente podría oír.)*

### El modelo de voz de un idioma · desde Sprint 001

- **Qué hace:** cada idioma necesita su modelo de voz, y macOS no los trae todos. *Idioma* dice de
  cada pista si su modelo está instalado y, **si falta, ofrece instalarlo**.
- **Cómo se usa:** *Idioma* → «Instalar», el botón junto al aviso ámbar de la fila. Lo
  descarga macOS y tarda: mientras dura, el botón dice «instalando…» y no se deja pulsar otra
  vez. Es la única vez que la app toca la red, y solo porque tú lo pediste.
- **Limitaciones conocidas:**
  - Si tu Mac **no reconoce** ese idioma, o no trae motor de voz, no hay nada que instalar y la app
    lo dice con esas palabras en vez de ofrecerte un botón que no puede funcionar.
  - macOS permite **cinco idiomas listos a la vez**. Es un límite del sistema.

### El modo solo audio: la ficha, al oído · Nuevo · Sprint 002

- **Qué hace:** te **lee la ficha en voz alta** mientras la banda se encoge a una sola línea, para
  que recuperes la pantalla completa de la reunión. Es lo que pediste en el diseño: *«que me hable
  de forma paralela por si quiero ver completamente la pantalla y no me interrumpa»*.
- **Cómo se usa:**
  1. `⌃⌥V` **enciende el modo**. La banda baja de 88 a 44 px, la ventana de la reunión se hace más
     grande, y la app te lee la ficha que tengas delante: el titular, la línea y **de dónde sale**.
  2. A partir de ahí, cada vez que el cliente termine de hablar y aparezca una ficha nueva, te la
     lee sola. No hay que pulsar nada.
  3. `⎋` la **calla** en el acto, sin esperar a que termine la frase.
  4. `⌃⌥V` otra vez **apaga el modo** y la banda vuelve a sus 88 px. Arrastrar el asa hacia arriba
     hace lo mismo: el alto *es* el modo.
- **Cuándo se calla sola, y por qué:**
  - **Si el sonido sale por los altavoces de tu Mac**, no habla y te lo dice en ámbar: «Conecta
    auriculares · el cliente te oiría». Es el candado del modo.
  - **Mientras alguien esté hablando** en la reunión —el cliente o tú—. Nunca habla encima de nadie.
  - **Mientras ya esté diciendo otra ficha.** No encola una detrás de otra.
  - Y `⌥⎋` la calla y apaga el modo, como todo lo demás.
- **Limitaciones conocidas:**
  - **Con auriculares por Bluetooth o USB —unos AirPods, por ejemplo— la app habla, y no puede
    estar segura de que sean auriculares.** macOS no distingue un casco de un altavoz de mesa
    conectado por el mismo cable: solo sabe con certeza cuándo el sonido sale por el altavoz interno
    del Mac. La app dibuja la línea ahí —igual que el aviso de eco de *Sesión*— y te lo dice aquí en
    vez de prometerte lo que no puede saber. Si tu salida es un altavoz externo, **no enciendas el
    modo**.
  - **`⎋` deja de llegarle a la reunión mientras el modo está encendido.** En Google Meet es la
    tecla que sale de pantalla completa. Se coge solo mientras dura el modo y se devuelve al
    apagarlo.
  - **Las fichas «no tengo nada sobre…» no se leen.** Su titular son las palabras del cliente, y
    sacarlas por el altavoz sería leerte el transcript. La banda sí las pinta.
  - **Si tu Mac no tiene voz para tu idioma**, el modo no se enciende y lo dice en el registro.
  - Lee en **tu** idioma —el de tu pista—, porque la ficha sale de tus documentos.

### Español e inglés, en todo · desde Sprint 001

- **Qué hace:** la app entera, incluidos los textos que macOS te enseña al pedir permisos, está en
  los dos idiomas. Sigue el del sistema.

### Tu diccionario técnico · Nuevo · Sprint 002

- **Qué hace:** el motor de voz de macOS no conoce tu jerga. «Power BI» le sale «power by», «DAX» le
  sale «the ax» y «Lakehouse» le sale «lake house» — y cada una de esas es una ficha que no llega,
  porque la app busca en tus documentos la palabra equivocada. El diccionario **corrige el texto
  después de transcribir**, en tu Mac, sin modelo y sin que nada salga de tu equipo.
- **Cómo se usa:** sin hacer nada. Viene con la jerga de datos ya puesta (Power BI · DAX · Microsoft
  Fabric · Semantic Model · Lakehouse) y **los nombres de tus clientes salen solos de tu corpus** —
  si tienes una ficha de «Páramo Azul», la app ya sabe escribirlo bien.
- **Y si quieres añadir lo tuyo:** el archivo es
  `~/Library/Application Support/com.aiapps.copiloto-consultor/diccionario.yaml`, y puedes editarlo
  con cualquier editor de texto. Una línea por término:

  ```
  Power BI: [power bi, powerbi, power by]
  Lakehouse: [lake house]
  Contabilidad Regulatoria:
  ```

  A la izquierda, **como quieres verlo escrito**; entre corchetes, las formas en que se oye mal. Un
  término sin corchetes también vale: se corrige por parecido. Los cambios se aplican **al empezar la
  sesión siguiente** — no hace falta cerrar la app.
- **Cuánto mejora, medido:** en el audio de prueba con jerga, la transcripción pasa de un 46 % de
  palabras erradas a un 42 % en castellano, y de un 35 % a un 26 % en inglés. En los audios **sin**
  jerga no cambia nada, que es igual de importante: no toca lo que ya estaba bien.
- **Limitaciones conocidas:**
  - **Arregla lo que se oyó parecido, no lo que se perdió.** «Power B» se convierte en «Power BI»,
    pero si el motor oyó «Lakehouse» como «en la que usé», no queda nada a lo que parecerse.
  - **Los términos de cuatro letras o menos solo se corrigen si tú escribes cómo se oyen.** «DAX»
    está a una letra de «das», «dos», «tax» y «max»: corregir por parecido ahí estropearía más de lo
    que arregla.
  - **Los nombres de tus clientes no se guardan en el archivo.** Salen de tu corpus cada vez que
    empiezas sesión, así que ese archivo no contiene el nombre de nadie.
  - **Cada pista escucha un idioma.** Si en mitad de una frase castellana el cliente dice tres
    palabras en inglés, el diccionario arregla la jerga que reconozca, pero **una frase entera en el
    otro idioma no se transcribe bien** — está medido y está dicho en la pantalla de *Idioma*. Marcar
    varios idiomas por pista llega más adelante.

## Atajos de teclado

| Tecla | Qué hace |
|---|---|
| `⌥⎋` | corta todo: audio, transcript, banda y relleno. Devuelve la ventana de la reunión |
| `⌃⌥A` | «ayúdame con esto»: busca una ficha sobre lo último que dijo el cliente |
| `⌃⌥T` | muestra u oculta el transcript en la banda |
| `⌃⌥V` | enciende o apaga el **modo solo audio**: te lee la ficha y la banda baja a una línea |
| `⎋` | calla la voz — **solo mientras el modo solo audio está encendido** |

> **Por qué `⌃⌥` (Control + Opción) y no `⌘⇧`.** Hasta el sprint 2 las teclas eran `⌘⇧`, y según
> la documentación de Zoom para Mac, en Zoom esas teclas silencian tu micrófono (`⌘⇧A`), apagan tu
> cámara (`⌘⇧V`) y pausan la pantalla compartida (`⌘⇧T`). Mientras Angel Ghost estuviera abierto,
> Zoom dejaba de recibirlas. Y `⌘⇧T` reabre la última pestaña cerrada en el navegador. Zoom, Meet y
> Teams no documentan ninguna tecla `⌃⌥`. Lo de Zoom no se ha probado en vivo: no está instalado en
> el Mac donde se construye la app.
>
> **Si usas VoiceOver**, sus órdenes también empiezan por `⌃⌥`. Con VoiceOver encendido, estas
> teclas pueden chocar con las suyas. Lo mismo con apps que ordenan ventanas con `⌃⌥`, como Rectangle.
>
> **Y ojo con `⎋`:** mientras el modo solo audio está encendido, esa tecla es de la app y no le
> llega a la reunión. Se devuelve en cuanto apagas el modo con `⌃⌥V`.

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
es cuando tú le pides instalar el modelo de voz de un idioma —el botón **«Instalar»** de
*Idioma*—: entonces macOS lo descarga, y no sale nada de aquí.

**¿Por qué me pide auriculares?**
Con los altavoces, tu micrófono oye también al cliente y las dos pistas se mezclan. La app lo
detecta y lo marca, pero funciona mejor con auriculares.

## Historial

| Sprint | Features añadidas a este manual |
|---|---|
| 001 | la banda protegida · el acople · las dos pistas y la transcripción local · el corpus indexado · la ficha de evidencia y la sugerencia de cómo conducirse · el modelo de voz de un idioma · el corte y la pantalla de Honestidad · español e inglés |
| 002 | el disparo por silencio · **tu diccionario técnico** · **el modo solo audio** |

> **Corregido tras la auditoría del sprint 001** (2026-09-22): tres frases de este manual habían
> dejado de ser ciertas y se arreglaron con lo que el código hacía de verdad — el disparo por
> silencio, que entonces no existía y se declaró como limitación (**se cableó en el sprint 002**, y
> lo que dice este manual arriba es lo que hace hoy); la instalación del modelo de voz, que no tenía
> botón y ahora lo tiene; y la ficha automática, que no llegaba a la banda por un defecto del
> puente. El detalle está en `sprints/SPRINT_001-auditoria.md`.
