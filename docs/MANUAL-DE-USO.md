# Angel Ghost — Manual de uso

> **Documento obligatorio y vivo.** Toda feature que llega a `main` se documenta aquí en el mismo
> sprint. Escrito para quien va a usar la app, en español llano — sin jerga técnica ni referencias
> al código.

## Qué es esta app

Angel Ghost es una ventana pequeña en tu Mac que **el cliente no ve, aunque compartas pantalla**.
Escucha los dos lados de la videollamada, busca en tus propios documentos, y cuando alguien
pregunta algo que tú ya respondiste en una propuesta, un caso o un marco tuyo, te lo pone delante:
un titular, una línea y de dónde sale.

No graba nada. Al cerrar, del audio no queda rastro, y de lo que se dijo solo lo que tú decidas
guardar —tus notas y, del cliente, como mucho un hecho de una línea—, y lo que no decidas espera unas
horas en la bandeja y se borra solo; la propia app tiene una
pantalla para que puedas comprobarlo en vez de creértelo.

**Y antes de la reunión, puedes ensayarla:** la app te hace las preguntas que ese cliente probablemente
hará, sacadas de tu propia propuesta, te escucha por el micrófono y te dice qué evidencia tuya usaste y
cuál tenías sin usar (*El ensayo*, abajo).

**Para quién:** consultores y asesores que ya tienen su material escrito y lo necesitan en el
momento exacto de la conversación, sin ponerse a buscar delante del cliente.

## Primeros pasos

**Necesitas macOS 26 o posterior.** La app usa el reconocimiento de voz y el modelo del sistema que
solo trae macOS 26; en un Mac con una versión anterior no arranca.

1. **Arranca la app.** Aparecen tres cosas: el cuaderno (la ventana grande), una banda apaisada
   bajo la barra de menús, junto a la cámara —es su sitio de fábrica; la puedes poner abajo—, y el
   relleno que la tapa cuando compartes pantalla.
2. **Concede los permisos.** Ve a *Permisos*. La app te lleva al sitio exacto de Ajustes del
   Sistema para cada uno. **Puedes usarla sin conceder ninguno**: lo que no funcione te lo dirá.
   **Audio del sistema y Pantalla son dos permisos distintos**, aunque Ajustes los enseñe en el mismo
   panel («Grabación de pantalla y audio del sistema»): cada fila de *Permisos* dice el suyo.
3. **Señala tu carpeta de documentos.** Ve a *Corpus* → «Señalar una carpeta». La app lee tus PDF,
   Word y Markdown **donde están** — no los copia ni los sube a ninguna parte.
4. **Ponte los auriculares** y entra a tu reunión. En *Sesión*, pulsa «Iniciar sesión».

## Features

### La banda protegida · desde Sprint 001

- **Qué hace:** una franja bajo la barra de menús (o en el borde inferior, si la prefieres abajo)
  donde aparece todo lo que la app tiene que decirte. **Cuando compartes pantalla, el cliente ve tu fondo de escritorio en ese
  trozo**, no la banda.
- **Cómo se usa:** aparece sola al arrancar. Arrastra el **asa** del centro, alejándola de su borde,
  para ampliarla y ver más; de vuelta, la deja compacta. Con la banda arriba, el asa va abajo y se
  arrastra hacia abajo; con la banda abajo, hacia arriba.
- **Limitaciones conocidas:** la invisibilidad está **verificada en Google Meet sobre macOS
  26.6.2, con la banda abajo**. Con la banda arriba, y en Zoom y Teams, la app no lo ha comprobado,
  y te lo dice («sin verificar») en vez de prometértelo. Si eso te preocupa: comparte una ventana en
  vez de la pantalla completa, o usa un segundo monitor.

### El acople: la reunión se hace sitio · desde Sprint 001

- **Qué hace:** le hace sitio a la banda en la ventana de la videollamada para que no le tape nada,
  como si fueran dos aplicaciones pegadas. **Con la banda abajo**, la recorta por abajo. **Con la banda
  arriba** (desde el sprint 4), **la baja y la encoge** lo que mide la banda, sin mover su borde de
  abajo. Al cerrar la banda, la ventana vuelve a su sitio y a su tamaño.
- **Cómo se usa:** necesita el permiso de **Accesibilidad**. Si no lo concediste, la banda flota
  encima y te avisa con «sin acople».
- **Arriba, solo se toca la ventana de la reunión** que la app detectó —Meet en tu navegador, Zoom o
  Teams—, nunca la que tengas delante. Sin sesión, la banda flota bajo la barra de menús y dice «sin
  acople»; se acopla al iniciar la sesión.
- **Abajo, el acople es el de siempre: la ventana que tengas delante.** La app la acopla sola en los
  30 segundos que siguen a abrirla, y cada vez que pones la banda abajo (`⌃⌥B` o *Sesión*). «Iniciar
  sesión» no la acopla: si abriste la reunión más tarde, ponla delante y pulsa `⌃⌥B` dos veces (la
  banda sube y vuelve abajo, y al volver la acopla).
- **Tras `⌥⎋` no se acopla nada:** con la banda cerrada, `⌃⌥B` y la fila de *Sesión* solo cambian tu
  elección; no tocan ninguna ventana.
- **Limitaciones conocidas:** sin él la banda flota encima en vez de acoplarse. Y con **Meet en el
  navegador** hace falta para algo más: sin él la app no puede leer el título de la pestaña, así que
  no sabe que hay reunión —*Sesión* dice «No se puede saber si hay reunión»—, no lee su pantalla y
  el radar ámbar no mira. Con Zoom y Teams no hace falta para eso.
  - **Arriba:** si la ventana de la reunión quedaría con menos de 240 px de alto, o está en otra
    pantalla, no se toca y la banda flota. Si macOS no la deja mover (pantalla completa, otro
    escritorio), la app deshace lo que hizo y la banda flota.
  - Arriba está **probado en vivo con Meet en Chrome**: se acopla en unos 230 ms y se devuelve en unos
    120. Con Zoom, Teams, una pantalla externa y la pantalla completa, todavía no.
  - Si mueves tú la ventana mientras está acoplada, al cerrar no se devuelve: la app solo devuelve lo
    que sigue como ella lo dejó.

### Arriba o abajo: la banda junto a la cámara · Nuevo · Sprint 004

- **Qué hace:** de fábrica, la banda va **arriba**, bajo la barra de menús y el notch, junto a la
  cámara. Leer la ficha ahí se ve como mirar a quien habla; abajo, se ve como bajar la vista. Su
  relleno va con ella, así que al compartir pantalla el cliente ve tu fondo de escritorio en esa franja.
- **Cómo se usa:** en *Sesión*, la fila **«La banda: arriba · abajo»**, o **`⌃⌥B`** desde cualquier
  sitio. Al cambiar, la banda suelta la reunión, se recoloca y vuelve a acoplarla. La elección se
  recuerda.
- **La primera vez**, *Sesión* te avisa: «La banda ahora va arriba, junto a la cámara», con
  **«Entendido»**. Mientras no lo pulses, el aviso ocupa el sitio de la tarjeta de la reunión; la
  reunión sigue en el rail.
- **Limitaciones conocidas:** arriba la banda usa la misma protección que abajo, pero está **sin
  verificar**: nadie la ha mirado compartiendo pantalla con la banda arriba, y va en las pruebas del
  ciclo. Lo verificado es lo de la banda abajo, y *Sesión* lo dice así: «Tu panel no aparece en la
  pantalla que compartes. Verificado en tu Mac (macOS 26.6.2) el 2026-09-20 con la banda abajo; con la
  banda arriba, y en Zoom y Teams, está sin verificar.»

### Escucha las dos pistas · desde Sprint 001

- **Qué hace:** oye tu micrófono (eres tú) y el audio de tu Mac (es el cliente) por separado, y
  convierte los dos a texto **dentro de tu equipo**. Ningún audio sale de tu Mac para convertirse
  en texto.
- **Cómo se usa:** *Sesión* → «Iniciar sesión». Nada se enciende hasta que tú lo digas. `⌃⌥T`
  muestra u oculta el transcript. Aparece **junto a la ficha** que haya en la banda (es su columna
  derecha: sin ficha no hay dónde pintarlo) y, si la banda está compacta, la agranda como
  el asa; al ocultarlo vuelve a su alto. Nace oculto a propósito, porque leer lo que acaban de
  decir es la forma más rápida de dejar de escuchar.
- **Limitaciones conocidas:**
  - **Usa auriculares.** Con los altavoces del Mac, tu micrófono también oye al cliente y el mismo
    turno llega por las dos pistas. La app lo detecta y lo marca como eco, pero la conversación se
    lee peor. Te avisa antes de empezar, cuando todavía puedes ponértelos.
  - Cada pista escucha **un** idioma, y **se elige en *Idioma***: el código de cada pista (es-ES,
    en-US…) es un selector. Las dos nacen en **español**; si tu cliente habla inglés, cámbiala antes
    de «Iniciar sesión». **La elección se recuerda** · desde Sprint 003: la próxima vez que abras
    la app, cada pista sigue en el idioma que dejaste (lo prueba `lo_que_se_elige_sobrevive_al_reinicio`,
    en `prefs.rs`). Marcar varios idiomas en la misma pista no está en la app.
  - macOS solo deja tener **cinco idiomas de voz listos a la vez**. Es un límite del sistema.
  - Si no hablas, los contadores de audio se quedan quietos. **Eso es correcto**: cuando nadie
    habla, macOS no entrega una sola muestra.
- **Si una pista no abre, *Sesión* te dice por qué · nuevo en Sprint 002.** Con la sesión en marcha,
  la fila de la pista que no abrió cambia de símbolo, de palabra («No abrió») y de color, y debajo
  dice el motivo y qué hacer: falta el permiso del micrófono o del audio del sistema, otra app tiene
  el dispositivo, el dispositivo entrega un formato que la app no sabe leer, o macOS no dejó abrirla.
  Y debajo aparece la fila **«A medias»**, que dice cuál queda (desde el sprint 3 solo aparece cuando
  una pista cae). Hasta el sprint 002
  esta pantalla decía «Funciona» en las dos pistas pasara lo que pasara, y te habrías enterado al
  terminar la reunión, al ver que faltaba medio transcript.
- **Los auriculares por su nombre · nuevo en Sprint 002.** Si el sonido sale por un aparato por USB
  o Bluetooth (unos AirPods, un altavoz USB), *Sesión* dice su nombre. La app **no puede saber** si
  es un casco o un altavoz: si es un altavoz, no uses el modo solo audio. Por HDMI, DisplayPort o
  AirPlay lo trata como un altavoz: la fila sale en ámbar y lo dice.

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
  - **La app recuerda la carpeta** (solo su ruta, en tus preferencias) y **la vuelve a leer al
    arrancar**, en segundo plano: *Corpus*, «Este cliente» y la puerta local la tienen sin que la
    señales otra vez. Si la carpeta vive en Documentos, Escritorio o Descargas, macOS puede preguntarte
    la primera vez si Angel Ghost puede leerla. Si ya no está (un disco desconectado), la app lo dice y
    la puedes señalar de nuevo.
  - Arrastrar documentos sobre la ventana y releer solo lo que cambió no están en la app: hoy se
    vuelve a recorrer la carpeta entera.
  - El índice vive en la carpeta de datos de la app, **y solo tu cuenta del Mac puede leerlo**.

### La ficha en el momento justo · desde Sprint 001

- **Qué hace:** cuando el cliente pregunta algo, dice una cifra, nombra algo que está en tus
  documentos **o se queda callado después de hablar**, la app busca en tu corpus y pone en la banda
  un **titular de ocho palabras, una línea y la fuente exacta** — documento y sección.
- **Cómo se usa:** sola. Y `⌃⌥A` («ayúdame con esto») la pide a mano cuando no acierte.
- **Dice por qué llegó y cuánto tardó · nuevo en Sprint 002.** Debajo de la fuente, una línea corta:
  «pregunta · 1,2 s», «cifra · 0,9 s», «término tuyo», «silencio», «lo pediste» o **«en pantalla»**
  —esta con el símbolo de la pantalla, porque es la única ficha que llega sin que nadie diga nada—.
  Si la sección salió de un PDF, donde la app **conjetura** los títulos por la forma del texto, lo
  dice ahí mismo: «sección conjeturada».
- **El silencio también pide ficha · nuevo en Sprint 002.** Si el cliente dice algo que no lleva
  pregunta ni cifra y se queda callado **cuatro segundos**, la app busca por su cuenta lo último que
  dijo — es el hueco en el que te toca hablar a ti. Dos cosas que hace bien y conviene saber:
  **mientras el cliente siga hablando no cuenta como silencio** (aunque su frase anterior cerrara
  hace rato), y **no repite la ficha que ya está en la banda**, así que una pregunta que ya trajo su
  ficha no trae una segunda al callarse. En el sprint 001 esto estaba escrito en el código y sin
  conectar, y este manual lo declaraba como limitación; ahora funciona.
- **Limitaciones conocidas:**
  - **Todo lo que la ficha dice está recortado de tus documentos.** La ficha no se redacta: si no
    encuentra nada, lo dice. (Lo único redactado es la *sugerencia*, si la enciendes, y va aparte,
    marcada como tal y comprobada contra su ficha.)
  - Cuando no encuentra nada, enseña **qué buscó** —para que veas en el acto si te entendió mal—,
    lo más parecido que sí tienes, y una sugerencia de cómo conducirte. Esa sugerencia habla de
    **cómo responder**, jamás de tu negocio. Si la pregunta no encaja en ninguna situación conocida
    (precio, plazo, credencial, referencia, contrato) pero tu corpus tiene algo cercano, **te lo
    nombra**: «Lo más cercano que sí tienes es «Alcance»: ofrécelo y pregunta para qué lo
    necesitan.» · nuevo en Sprint 002.
  - Busca por las **palabras** de la pregunta. Si el cliente pregunta algo con palabras
    completamente distintas a las de tu documento, no lo encontrará.
  - Tu propia voz no dispara fichas: la app te contestaría a ti en mitad de tu frase.

### Nada se graba, y se puede comprobar · desde Sprint 001

- **Qué hace:** el audio, lo que se dijo y **el último cuadro leído de la pantalla** viven **solo en
  memoria** y mueren al cerrar. La pantalla de *Honestidad* enseña cuánto ocupa cada uno ahora mismo y
  cuántos bytes han salido de tu equipo.
- **Cómo se usa:** `⌥⎋` corta todo en el acto — audio, transcript, la banda y su relleno — y
  devuelve la ventana de la reunión a su sitio y a su tamaño. También está el botón en *Honestidad*,
  por si el atajo está cogido por otra app.
- **El corte alcanza las 12 piezas · desde Sprint 004.** *Honestidad* lo dice debajo del botón rojo:
  «El botón corta 12 de 12 piezas: ninguna queda fuera.» En el sprint 4 llegó **el ensayo**: si cortas a
  mitad de uno, el micrófono se cierra y tus respuestas sin guardar se van. En el sprint 2 llegaron la lectura de
  pantalla y **la sugerencia en camino**: si el modelo está redactando cuando cortas, lo que vuelva se
  tira y, con el proveedor externo, una petición que no haya salido ya no sale. En el sprint 3, **tus
  turnos en texto** y **las propuestas que no has decidido**: el corte se las lleva, igual que al
  resto de la captura. **Tus notas, tus acuerdos, tus fichas fijadas y las propuestas que guardaste se
  quedan** —*Honestidad* dice «Tus notas siguen ahí»—, y la bandeja de una reunión ya cerrada sigue su
  ventana. La voz del
  modo solo audio es la primera que se corta, porque es la única que tu cliente podría oír.
- **Limitaciones conocidas:** el botón corta lo que vive en memoria **ahora**; lo que ya salió por
  los altavoces o lo que ya viste en la banda, obviamente, no se puede deshacer.

### Tus notas: lo único de la reunión que queda · Nuevo · Sprint 003

- **Qué hace:** mientras escuchas, *Notas* es tu cuaderno de la reunión: **tu nota** (un campo para
  escribir lo que quieras), **tus acuerdos** (los escribes tú; la app no decide qué fue un acuerdo) y
  **las fichas que fijaste**. Al terminar, se guardan en **un archivo por reunión, cifrado**, en la
  **carpeta privada de la app**. Es lo único de la reunión que llega al día siguiente, junto con las
  propuestas que guardes: del cliente no se guarda ni su voz, ni sus turnos, ni lo que se leyó de su
  pantalla; como mucho, un hecho de una línea que tú aceptaste.
- **Cómo se usa:**
  1. Durante la sesión, `⌃⌥N` te trae el cuaderno a *Notas* con el cursor al final de tu nota. Escribe
     y sigue: se va guardando en memoria a cada letra.
  2. Un acuerdo: escríbelo en su campo y pulsa ↵.
  3. `⌃⌥P` fija la ficha que estás viendo en la banda (su titular y su fuente, no el documento). En
     la banda, «Anotar para después» hace lo mismo que `⌃⌥N`: te lleva a tu nota para que lo escribas
     con tus palabras —la pregunta del cliente no se copia—.
  4. Al parar la sesión, *Notas* te enseña **qué se va a guardar y qué muere**, contado, y el nombre
     del archivo que va a nacer. «Guardar cifrado y cerrar» o «Cerrar sin guardar».
  5. Sin reunión abierta, *Notas* es **el archivo**: tus reuniones guardadas, dónde viven, con qué
     llave y cuándo se borran. **«Mostrar en Finder»** abre su carpeta con la reunión que estás viendo
     seleccionada. **«Cuánto viven tus notas»** es una sola elección para todas: 7 días,
     30 días, **90 días** (de fábrica), 1 año o siempre. Cada reunión se guarda con la que haya al
     guardarla.
  6. **«Exportar a texto»** te pregunta antes —porque el archivo exportado ya no está cifrado—, después
     te pide Touch ID o la contraseña del Mac, y después dónde. El `.md` lleva tu nota, tus acuerdos,
     **las propuestas que guardaste** (del cliente, el hecho en una línea, como en la pantalla), tus
     fichas fijadas y, si los conservas, tus turnos. La carpeta que eliges no cambia de permisos: el
     archivo exportado nace legible solo para tu cuenta. **«Borrar ahora»** también pregunta antes: se
     borra de la app y no vuelve; si usas Time Machine, tus copias la conservan hasta que caduquen.
- **«Conservar mis turnos»**: si lo enciendes (nace apagado), lo que **tú** dices, en texto, entra
  también en el archivo de las reuniones siguientes. Nunca el audio, ni el tuyo. Y nunca un turno de
  tu micrófono marcado como eco: con altavoces, ese turno es la voz del cliente.
- **El cuaderno, protegido:** desde que empiezas a escuchar hasta que guardas o descartas la reunión,
  la ventana del cuaderno se **protege de la captura** como la banda, con el mismo flag del sistema.
  Así tu nota no se ve si compartes la pantalla entera por descuido. **Sin verificar todavía con el
  cuaderno:** la banda está comprobada en Meet sobre macOS 26.6.2, con la banda abajo; el cuaderno se
  mira en la parada 5 del ⭐⭐ (Acto 2), y en Zoom y Teams, ni la una ni el otro.
- **Dónde viven:** en `~/Library/Application Support/com.aiapps.copiloto-consultor/notas/`, junto a
  la bandeja. **No en Documentos**: desde ahí la tarea que borra lo vencido con la app cerrada no puede
  entrar, y la papelera de iCloud guardaría 30 días lo que se borra. Esa carpeta no se sincroniza con
  iCloud. **Tus notas sí entran en tus copias de Time Machine**: son tuyas y están hechas para durar, y
  si se te estropea el Mac las recuperas de la copia. Si guardar falla —el Llavero no contesta, el
  disco está lleno—, la reunión sigue abierta, tu nota entera, y «Intentar otra vez» vuelve a guardar.
- **La llave:** vive en el **llavero de inicio de sesión** de tu Mac («Angel Ghost · notas»): se abre
  con tu sesión y no se sincroniza con iCloud, pero **viaja con tus copias de Time Machine y con el
  Asistente de migración**, protegida por tu contraseña. Guardar no te pide nada; exportar una reunión
  guardada te pide Touch ID o tu contraseña, una vez cada vez que abres la app. **Tus ensayos guardados
  usan la misma llave y la misma retención** (*El ensayo*, abajo).
- **Limitaciones conocidas:**
  - **Si borras el Llavero, tus reuniones guardadas no se pueden abrir.** La app no tiene recuperación:
    sería una segunda llave. Solo una copia de Time Machine (o el Asistente de migración) devuelve tu
    llavero, y con él tus notas, que se siguen abriendo.
  - **Que la llave quede solo en este Mac llega con la firma de Apple (H2):** hasta entonces, el
    llavero que la app puede usar no sabe atarla a un solo equipo.
  - **Tus notas no aparecen en Documentos:** se llega a ellas desde *Notas*, o con «Mostrar en Finder».
  - **El nombre del archivo va en claro** (`reunion-2026-09-27-1402.ghost`, o
    `paramo-azul-2026-09-27.ghost` si elegiste cliente): dice qué día y, si elegiste cliente, con
    quién; no qué se habló.
  - **Si la app se cae, lo no guardado se pierde**: vive en memoria a propósito. Si sales de la app con
    notas sin guardar, se guardan solas; y si empiezas otra sesión con la anterior abierta, también.
  - **Tus notas vencidas se borran solas, aunque no abras la app:** las borra la misma tarea de macOS
    que la bandeja (abajo), en el minuto siguiente a su vencimiento. Las que guardaste para «siempre»
    no vencen. Mientras tengas notas con fecha —o ensayos—, esa tarea aparece en Ítems de inicio.
  - Si eliges el cliente en *Sesión* («Este cliente», abajo), el archivo lleva su nombre
    (`paramo-azul-2026-09-27.ghost`); si no, se llama `reunion-<fecha>-<hora>`.

### La app te propone qué guardar, y la bandeja · Nuevo · Sprint 003

- **Qué hace:** mientras hablas, la app **propone** qué guardar de la reunión, con **reglas publicadas y
  sin modelo**: cifras, plazos y fechas · tus compromisos («te lo mando», «quedamos en») · lo que choca
  con una ficha que fijaste · nombres que no están en tu corpus · preguntas que te hicieron. Proponer
  no es guardar: nada entra en tu archivo sin tu sí.
- **Qué se guarda de cada lado:** de lo que dices tú, tu frase. Del cliente, **nunca su turno**: un
  hecho en una línea («Dijeron «cuatro fuentes»; tu ficha fijada dice «tres»»), con un fragmento de
  ocho palabras como mucho; de una pregunta, solo sus palabras clave.
- **Cómo se usa:**
  1. La última propuesta aparece en la banda, arriba y en una línea: «Te propongo guardar: …». No
     tapa la ficha, no suena y no se lee en voz alta. **`⌃⌥↵` la guarda.**
  2. En *Notas*, «Te propongo guardar esto» las enseña todas, cada una con «Guardar» y «No». Las que
     guardas entran en tu archivo, aparte de tus acuerdos.
  3. Al parar, «al cerrar» dice cuántas quedan sin decidir y te deja elegir **cuánto esperan en la
     bandeja**: al cerrar · 1 h · **3 h** (de fábrica) · fin del día · 24 h. La elección se recuerda.
  4. Tras «Guardar cifrado y cerrar», *Notas* enseña **la bandeja** con su cuenta atrás: «Guardar» lleva
     la propuesta a su reunión (con la retención de la reunión) y «No» la borra. También «Guardar
     todas», «Descartar todas», cambiar la ventana ahí mismo e «Ir a tus reuniones».
  5. Si la bandeja es de una vez anterior que abriste la app, se ve su cuenta atrás, pero **lo que dice
     pide Touch ID**, como exportar una reunión.
- **Dónde vive:** cifrada con la misma llave que tus notas y junto a ellas, en la carpeta privada de la
  app, **no en Documentos**: la papelera de iCloud guarda 30 días lo que se borra, y una lista que
  promete morir a las 3 h no puede tener una copia que viva un mes. Por lo mismo, **la bandeja no entra
  en tus copias de Time Machine**: la app la marca fuera de las copias al escribirla. Las instantáneas
  locales que macOS guarda unas horas (menos de 24 h) sí la ven hasta que se reciclan.
- **Se borra sola al vencer, aunque no abras la app.** Lo cumple macOS con una tarea programada al
  minuto de cada vencimiento; entre vencimientos no corre nada. *Honestidad* enseña la bandeja con su
  cuenta atrás, y si la tarea no corrió mientras la app estaba cerrada, lo dice en rojo.
- **Limitaciones conocidas:**
  - **La primera vez, macOS avisa de que se añadió un ítem en segundo plano.** Hasta que la app vaya
    firmada, en Ajustes del Sistema → General → Ítems de inicio aparece como **«sh · desarrollador no
    identificado»**: es la tarea de borrado de Angel Ghost, y se queda mientras haya bandeja, notas o
    ensayos con fecha. Si la desactivas, la bandeja, tus notas y tus ensayos vencidos solo se borran con
    la app abierta, y *Honestidad* lo dice.
  - Si borras la app, la tarea sigue borrando a su hora lo que quede; después se queda como una
    entrada inerte en Ítems de inicio, que puedes quitar.
  - Como mucho hay 30 propuestas esperando; al llegar a 30, las nuevas no entran y *Notas* lo dice.
  - Con el modelo local encendido, la app **no** redacta mejor las propuestas: en este ciclo son solo
    reglas.

### El marco en la mano: tu cliente, su jurisdicción y su NDA · Nuevo · Sprint 003

- **Qué hace:** antes de empezar, *Sesión* te enseña **qué regla aplica a este cliente**, con sus
  normas y su fecha; te pregunta **si su NDA prohíbe grabar o transcribir**, y si lo prohíbe te
  propone el **modo solo notas**; y te da una **cláusula modelo** para tu carta de encargo, en español y
  en inglés. **Nunca te bloquea** y **no es asesoría legal**: lo dice siempre.
- **Cómo se usa:**
  1. En la ficha de tu cliente (en tu corpus), escribe una línea con dónde está la contraparte:
     `Jurisdicción: Colombia` (o `Jurisdiction: Florida`). Vuelve a indexar la carpeta.
  2. En *Sesión*, **«Este cliente»** → elige el cliente. Aparece su **bandera**: el país o el estado,
     su riesgo (con símbolo, texto y color), la regla, lo que implica, las normas y la fecha en que se
     consultaron (2026-09-17). Si algo de esa fila no se pudo verificar, lo dice debajo: «Sin
     verificar: …».
  3. **«Revisar»** junto a la NDA: la app pregunta «¿La NDA de este cliente prohíbe grabar o transcribir
     por cualquier medio?». Tu respuesta se guarda para ese cliente.
  4. Si respondes **«Sí, lo prohíbe»**, *Sesión* te propone **«Iniciar en modo solo notas»**, y te deja
     volver a revisar la NDA si te equivocaste.
  5. **«Cláusula de encargo»** enseña la cláusula en los dos idiomas, lado a lado, con un «Copiar» cada
     una: copia la del idioma de tu carta, no la de la app.
- **El modo solo notas:** también lo puedes elegir tú, con **«Solo notas»** junto a «Iniciar sesión».
  La reunión se abre —tu cuaderno, protegido, y la banda— y **nada la escucha, la transcribe ni la
  lee**: ni micrófono, ni audio del sistema, ni pantalla, ni el radar ámbar. Siguen tus notas y acuerdos
  (`⌃⌥N`), fijar (`⌃⌥P`), el radar coral y `⌥⎋`. **`⌃⌥A` busca en tu corpus con la última línea de tu
  nota.** La banda lo dice en ámbar: «Solo notas · sin transcripción». Se termina como cualquier
  sesión.
- **El archivo de la reunión** lleva el nombre del cliente que elegiste. La elección vive mientras la
  app está abierta: no se guarda en disco.
- **Limitaciones conocidas:**
  - **No es asesoría legal.** El catálogo sale de una investigación con fuentes, fechada el 2026-09-17,
    con 17 puntos que no se pudieron verificar y que la app no afirma. Antes de usar la app con clientes
    en estados de EE. UU. de consentimiento de todas las partes, en Alemania, Francia, Chile o
    Argentina, o bajo una NDA que prohíba transcribir, consulta a un abogado.
  - El catálogo trae **27 jurisdicciones**: Colombia, EE. UU. (federal y estados de una parte), 14
    estados de todas las partes o mixtos, 3 sin estatuto verificado, la Unión Europea, Alemania,
    Francia, España, México, Chile, Perú y Argentina. Si la ficha dice otra, la app lo dice («no está en
    el catálogo v1») y no adivina.
  - Si la línea nombra varias («Colombia y California»), la bandera es la de **la más estricta**.
  - El catálogo se actualiza con una versión nueva de la app, nunca por la red.
  - La plantilla del aviso de una línea al cliente y el registro de que le informaste quedan para H2.

### El ensayo: practica antes de la reunión · Nuevo · Sprint 004

- **Qué hace:** antes de la reunión eliges cliente y propuesta, y la app te hace **las preguntas que ese
  cliente probablemente hará**. Salen con reglas publicadas de tu propia propuesta (sus secciones, sus
  cifras, sus plazos y compromisos, sus riesgos y supuestos), de la ficha de ese cliente (quién decide,
  acuerdos previos) y de un catálogo de objeciones típicas de datos, BI e IA, cada una con su fuente.
  Te lee la pregunta, **te escucha solo por el micrófono** y, al terminar cada respuesta, te dice:
  - qué evidencia de tu corpus **usaste** y cuál **tenías y no usaste**;
  - cuánto tardaste;
  - tu ritmo, en palabras por minuto;
  - qué muletillas repetiste.

  **Sin puntaje**: son cifras de lo que se midió. Ningún modelo decide nada de esto.
- **Cómo se usa:**
  1. En el rail, *Ensayo* (bajo *Sesión*). Elige el **cliente** y, si tiene varias, la **propuesta**;
     elige **5, 8 o 12 preguntas**. «Leer la pregunta en voz alta» viene encendido: apágalo si
     prefieres leerla tú.
  2. **«Empezar el ensayo».** La voz lee la pregunta; mientras habla, el micrófono no escucha. Después,
     responde en voz alta.
  3. Tu respuesta se cierra sola con **2,5 s de silencio**, o con **Enter** si terminas antes. Pensar
     antes de la primera palabra no la cierra.
  4. Mira la evaluación. Si dijiste algo con otras palabras y la app no lo reconoció, pulsa **«Sí lo
     dije»**. **Enter** trae la siguiente pregunta.
  5. Las teclas son de la ventana: **Enter** listo · **R** repetir · **S** saltar · **Esc** terminar.
     `⌥⎋` corta todo, el ensayo incluido.
  6. Al final, **el informe**: cuántas respondiste y cuántas saltaste, la evidencia usada, el ritmo
     medio, el tiempo medio y las muletillas. Tres botones: **«Guardar con tus notas»**, **«Exportar
     como texto»** y **«Cerrar sin guardar»**.
- **Qué se guarda y dónde:** solo si pulsas «Guardar con tus notas»: **tus respuestas en texto y las
  cifras de cada pregunta**. El audio, nunca, ni un momento en disco. Se cifra con **la llave de tus
  notas** y se borra con **su retención** (90 días de fábrica; «siempre» no vence), aunque no abras la
  app: lo borra la misma tarea de macOS que tus notas. Vive en la carpeta privada de la app, en
  `ensayos/`, con el nombre del cliente y la fecha (`paramo-azul-2026-10-04.ghost`): dice con quién
  ensayaste y qué día, no qué dijiste. Guardar no te pide nada. La primera vez que guardas algo con
  fecha, macOS puede avisar de un ítem en segundo plano añadido (Background Items Added): es la tarea que
  borra lo vencido. Y si la llave de tus notas la creó otra compilación de la app, macOS puede preguntar
  si puede usarla. Un informe que no guardas no se guarda solo al salir de la app.
- **Tu progreso con un cliente:** con un ensayo guardado, *Ensayo* dice «Ensayos guardados con este
  cliente» con su cuenta y **«Ver tu progreso»**. Abrirlo te pide **Touch ID o tu contraseña** («abrir
  tus ensayos guardados»), una vez cada vez que abres la app. Es el mismo desbloqueo que tus notas: si
  en esta sesión de la app ya exportaste una reunión, no lo vuelve a pedir. Es una tabla con un ensayo
  por fila —evidencia, ritmo, muletillas y tiempo— y, desde el segundo, **«Desde el primero»**: cómo cambió cada
  cifra, con su flecha. Solo cifras: tus respuestas no se enseñan ahí. Se ven los últimos seis, y los
  anteriores se cuentan.
- **Exportar y borrar:** «Exportar como texto» guarda el ensayo que acabas de terminar como un `.md`
  legible, donde elijas, con el mismo desbloqueo; ese archivo ya no lo protege nadie. Un ensayo ya
  guardado no se vuelve a exportar. **«Borrar los ensayos de este cliente»**, en tu progreso, pregunta
  antes, y no abre los archivos ni pide otro desbloqueo.
- **«Enriquecer el banco»** (en *IA*, apagado de fábrica): si lo enciendes, al empezar un modelo propone
  **hasta 5 preguntas más**, apoyadas en tus secciones; van al final y marcadas «sugerida por el
  modelo». Va por el mismo camino que la sugerencia —el modelo del sistema, o el proveedor externo si lo
  encendiste, con lo del cliente tapado antes de salir y contado en la red—; en «Ver lo que salió», su
  fila dice «enriquecer el banco». Una pregunta que no se apoya en una sección tuya se tira. Sin
  modelo, el ensayo funciona igual, con las reglas.
- **Mientras ensayas:** *Honestidad* enseña **«Tus respuestas del ensayo»** en memoria, en texto, hasta
  guardar o cerrar, y cuánto salió de tu equipo **en este ensayo** (0 B, salvo «Enriquecer el banco» con
  el proveedor externo); el rail dice «Ensayando». El ensayo **no abre** el audio del sistema ni la
  pantalla. La puerta local se cierra sola, y *IA* dice «Se cerró sola: hay una reunión o un ensayo».
- **El ensayo y la sesión no conviven.** No se puede ensayar con una sesión abierta, y empezar una sesión
  descarta el ensayo. Si tienes uno terminado sin guardar, *Sesión* te avisa antes, encima de «Iniciar
  sesión»: «Tienes un ensayo terminado sin guardar» · «Empezar la sesión lo descarta. Guárdalo antes en
  Ensayo.»
- **Con una videollamada abierta, solo con auriculares.** Si Zoom o Teams están abiertos, o hay una
  pestaña de Meet, el ensayo no empieza por los altavoces: tu micrófono oiría la llamada —y la tomaría
  por tu respuesta— y la llamada oiría la voz que lee la pregunta. *Ensayo* dice «Hay una videollamada
  abierta» («Con los altavoces, tu micrófono oiría la llamada y la llamada oiría la voz del ensayo. Ponte
  auriculares o ciérrala para ensayar.»). Sin el permiso de Accesibilidad, la app no ve si tu navegador
  está en una llamada, y con el navegador abierto también te los pide: «No se puede saber si hay una
  videollamada» («Sin el permiso de Accesibilidad, la app no ve si tu navegador está en una llamada.
  Ponte auriculares o concédelo en Permisos.»).
- **Limitaciones conocidas:**
  - **La transcripción de macOS no conserva «eh» ni «um»**, así que no se cuentan: solo las muletillas
    que sí escribe («o sea», «básicamente», «like», «you know»…). La pantalla lo dice.
  - **«Usaste» compara palabras, no ideas:** una ficha cuenta como citada si tu respuesta comparte con
    ella al menos dos términos que no estaban ya en la pregunta. Si la dijiste con otras palabras, «Sí
    lo dije».
  - Si la pregunta es muy general, puede no haber fichas con qué comparar y la cifra sale «—»: la app
    no te acusa de nada que no tenías.
  - **Si haces pausas largas a mitad de respuesta**, los 2,5 s pueden cortarte: termina con Enter.
  - Sin reunión, **la voz sale por donde suene tu Mac**; con una videollamada abierta, solo con
    auriculares. Mientras habla, el micrófono no escucha.
  - **Con una videollamada abierta, el ensayo no empieza si el sonido sale por altavoces que la app
    reconoce**: los del Mac, un monitor por HDMI o DisplayPort, o AirPlay. Con auriculares de cable,
    Bluetooth o USB —unos AirPods, por ejemplo— sí empieza: macOS no distingue unos auriculares Bluetooth
    o USB de un altavoz de mesa, y la app se fía, como con el modo solo audio. **Si el sonido te sale por
    un altavoz Bluetooth o USB, no ensayes con una llamada abierta.** Sin videollamada, ensaya con lo que
    quieras.
  - **El ensayo todavía no se ha probado con una voz de verdad.** Lo prueba la CI, con el audio del kit
    y con una sesión completa en la que se guarda un ensayo y se descifra; con tu voz, tu micrófono,
    el Llavero y Touch ID va en las pruebas del ciclo.
  - Sin propuesta ni ficha de ese cliente en tu corpus, *Ensayo* dice «Tu corpus no tiene nada de este
    cliente»: la app no inventa preguntas.
  - El idioma del ensayo es el de tu propuesta; si no se puede saber, el de la ficha del cliente, y si
    tampoco, el de tu pista. *Ensayo* lo dice arriba («Idioma del ensayo: español.»). Si tu Mac no
    transcribe ese idioma, *Ensayo* te lo dice y te manda a *Idioma*.

### El modelo de voz de un idioma · desde Sprint 001

- **Qué hace:** cada idioma necesita su modelo de voz, y macOS no los trae todos. *Idioma* dice de
  cada pista si su modelo está instalado y, **si falta, ofrece instalarlo**.
- **Cómo se usa:** *Idioma* → «Instalar», el botón junto al aviso ámbar de la fila. Lo
  descarga macOS y tarda: mientras dura, el botón dice «instalando…» y no se deja pulsar otra
  vez. Es la única vez que la transcripción toca la red, y solo porque tú lo pediste (la otra
  puerta a la red de la app es el proveedor externo de IA, apagado salvo que lo enciendas).
- **Limitaciones conocidas:**
  - Si tu Mac **no reconoce** ese idioma, o no trae motor de voz, no hay nada que instalar y la app
    lo dice con esas palabras en vez de ofrecerte un botón que no puede funcionar.
  - macOS permite **cinco idiomas listos a la vez**. Es un límite del sistema.

### Lee la pantalla de la reunión · Nuevo · Sprint 002

- **Qué hace:** mira la ventana de la reunión —Zoom, Teams o la pestaña de Meet— y **solo lee lo
  nuevo**: cuando aparece otra diapositiva, otra hoja, otro documento, espera a que se quede quieto y
  lo lee **una vez**. Si trae una cifra o uno de tus términos, **te trae la ficha sin que nadie
  pregunte**. Y cuando el cliente pregunta algo que dos fichas responderían igual de bien, lo que se
  ve en pantalla **desempata**.
- **Cómo se usa:** sola, desde que pulsas «Iniciar sesión». En *Sesión*, debajo de la fila de la
  pantalla:
  - el interruptor **«Leerla sola»** la apaga: entonces no mira nada y olvida lo que había leído;
  - **`⌃⌥L`** la lee **una vez, ahora**, aunque esté apagada. Si no hay texto —una cámara, un vídeo, una
    pantalla en negro— la banda te contesta «Leí la pantalla: no hay texto que buscar.»

  **Con una NDA estricta:** apaga «Leerla sola» y usa solo `⌃⌥L` cuando tú quieras.
- **Lo que queda y lo que no:** la imagen **no se guarda nunca**. En memoria vive solo **el último
  cuadro** y las palabras que se sacaron de él; *Honestidad* dice cuánto ocupa, y el corte (`⌥⎋`) lo
  borra con lo demás. Nada de esto sale de tu Mac: la lectura la hace macOS dentro de tu equipo.
- **Medido:** en el kit de prueba (cinco pantallas sintéticas), **cuatro de cuatro** traen su ficha sin
  pregunta y la agenda —que no trae ni cifra ni término— no trae ninguna. Cada lectura tarda **menos de
  una décima de segundo**, y como mucho hay una por segundo.
- **Limitaciones conocidas:**
  - Necesita el permiso de **Pantalla** y, con Meet en el navegador, también el de **Accesibilidad**
    (para encontrar la pestaña). Sin el de Pantalla, *Sesión* lo dice («Sin permiso») y la app
    funciona igual, sin leerla.
  - Lee **la ventana de la reunión**, no tu pantalla entera. La lee **aunque otra ventana la
    tape**: macOS le entrega su contenido tal cual. Lo que la deja sin leer es **minimizarla u
    ocultarla** (`⌘H`).
  - **El vídeo de los participantes no cuenta como «algo nuevo»**: se mueve todo el rato y no trae
    texto. Si alguien comparte un vídeo, la app espera a que se quede quieto.
  - Lee español e inglés. Una diapositiva en otro idioma se lee peor.
  - Mientras el cliente mueve el ratón sobre la diapositiva, la app espera: solo lee lo que se queda
    quieto.

### El radar: quién graba y quién te vigila · Nuevo · Sprint 002

- **Qué hace:** te avisa de dos cosas, con dos colores y dos símbolos distintos, **mirando solo tu
  Mac y tu pantalla**:
  - **Ámbar, «sábelo»** (el punto de grabación): la pantalla de la reunión muestra el **aviso de
    grabación o de transcripción** de Meet, Zoom o Teams, o el nombre de un **bot de notas** aparece
    en la ventana de la reunión. La banda dice, por ejemplo, «Reunión grabada · bot de notas presente» y de dónde lo
    sacó: «leído de tu pantalla · 14:03». Ese bot es de otra persona: **Angel Ghost nunca entra a la
    llamada**. El cliente tiene derecho a grabar y a traer su bot; la app **avisa, no bloquea**.
  - **Coral, «invasivo»** (la equis rellena): un programa **de tu propio Mac** que mira tu pantalla,
    tu cámara, tus teclas o tus procesos —supervisión de exámenes, monitoreo de empleados, acceso
    remoto—. La banda dice cuál es y **qué alcanza a ver**: «Te está mirando un programa de tu Mac ·
    «X» (acceso remoto): …».
- **Cómo se usa:** no hay que hacer nada.
  - El **coral** mira los programas de tu Mac desde que abres la app, cada diez segundos. Si encuentra
    uno **antes de empezar**, *Sesión* entera es el aviso —«Software invasivo corriendo en tu Mac»—, con
    una tabla de lo que encontró, **qué alcanza a ver** cada programa y la versión del catálogo; decides
    tú: **«Iniciar de todos modos»** o **«No iniciar»**.
  - Con la sesión en marcha, el coral salta en la banda. **`⌃⌥R`** («qué ve») abre *Sesión* con la
    tabla; con la banda ampliada tienes además **«Ver qué alcanza a ver»** y **«Corta todo»** (el mismo
    corte de `⌥⎋`).
  - El **ámbar** llega con la lectura de pantalla: lo que ella ya lee de la ventana de la reunión es
    lo que el radar revisa. El mismo aviso no se repite en cada diapositiva: sale una vez, y otra vez
    solo si cambia (un bot nuevo, por ejemplo).
  - Si tu Mac está **inscrito en un MDM** (la gestión de equipos de una empresa), *Sesión* lo lista
    como «Sábelo» —en su lista de lo que funciona, o dentro de la tabla si además hay un programa
    invasivo—: es normal en un equipo de empresa y no salta a la banda.
- **Lo que NO hace, y cómo se comprueba:** no toca la máquina de nadie, no abre conexiones, no
  pregunta nada a internet y no cierra ni bloquea ningún programa. La lista de programas se le pide
  al núcleo de tu Mac —la misma que ves en el Monitor de Actividad—, se compara en memoria y se
  tira. El catálogo viaja **dentro de la app**, versionado y con la fuente de cada fila; se actualiza
  con una versión nueva de la app, no por la red. Lo vigila un test que falla si el radar llama a
  algo fuera de su lista (`tests/unit/radar-solo-este-mac.test.ts`).
- **Medido:** en el kit de prueba, un Mac sintético con 82 programas normales —y nombres
  parecidos a propósito, como Microsoft Teams o la app «Compartir pantalla» con la que tú miras la de
  otro— da **cero** avisos; el mismo Mac con un programa de cada fila del catálogo los encuentra
  **todos**.
- **Limitaciones conocidas:**
  - El **ámbar necesita la lectura de pantalla encendida** y la ventana de la reunión visible: si
    apagas «Leerla sola», el radar ámbar tampoco mira. El coral no depende de eso.
  - Reconoce los programas **por el nombre de su ejecutable**. Un programa renombrado, o uno que no
    está en el catálogo, no se ve. Las **extensiones del navegador** (algunas de supervisión de
    exámenes lo son) no tienen proceso propio y no se ven.
  - Un programa de acceso remoto **abierto** no significa que alguien esté conectado **ahora**: la
    app avisa de que podría, no de que lo esté.
  - Si hay dos bots de notas, la banda nombra el primero.
  - El bot se reconoce por su nombre **en cualquier sitio de la ventana de la reunión** —la lista de
    participantes, pero también una diapositiva o el chat que lo mencione—: la banda dice que
    «aparece en la ventana», no que esté conectado.
  - El aviso de grabación se reconoce por sus frases en español e inglés, tal y como Meet, Zoom y
    Teams las escriben hoy. Si cambian, el catálogo tiene que ponerse al día.

### La sugerencia: qué decir, con su fuente · Nuevo · Sprint 002

- **Qué hace:** cuando llega una ficha que responde a lo que dijo el cliente, la app **te propone qué
  decir en una frase**, debajo de la ficha: «Confirma que la limpieza incluye hasta tres fuentes; una
  cuarta es adicional y se cotiza aparte». Dice dónde se redactó («sugerencia · en tu Mac», o el nombre
  del proveedor externo) y con qué **confianza** (alta, media o baja). **La ficha no se va**: su
  titular y su fuente siguen a la vista, y con la banda ampliada la sugerencia ocupa el hueco de la
  derecha.
- **Cómo se usa:** en **IA** (la última pantalla del cuaderno), enciende **«Redactar sugerencias»**.
  Nace apagado: la app funciona entera sin ello. Arriba, **quién redacta**: el modelo del sistema de
  macOS, dentro de tu Mac, si lo tienes; y si nadie puede, **por qué** («Apple Intelligence está
  apagado en Ajustes»).
  - **El modelo del sistema** necesita **Apple Intelligence** activado en *Ajustes del Sistema →
    Apple Intelligence y Siri*. Es gratis y no sale nada de tu Mac.
  - **El proveedor externo** (Claude o Groq) es opcional y es **tuyo**: pegas tu clave, la app
    la guarda en tu **Llavero** —nunca en un archivo— y enciendes su interruptor. Sin clave no se
    enciende.
  - **Lo que cada proveedor hace con lo que le mandas** está escrito en IA, bajo el costo, para el
    proveedor que elijas.
    Ninguno de los dos entrena con ello. **Claude lo guarda hasta 30 días**; para que no guarde nada
    hace falta un acuerdo de retención cero con Anthropic. **Groq puede guardarlo hasta 30 días**
    para vigilar abusos, salvo que enciendas la retención cero en *Data Controls*, en su consola: si
    usas Groq, enciéndela. Lo que sale va siempre anonimizado en tu Mac, pero sale. Gemini estuvo en
    la app y salió: su API no ofrece retención cero en ningún nivel, y sin facturación entrena con lo
    que recibe (la tabla, con sus fuentes y la fecha en que se leyó, está en el ADR 011).
- **Lo que no hace, y cómo se comprueba:**
  - **Nunca inventa la fuente.** El modelo recibe **la última frase de tu cliente** y las tres fichas del momento, cada una
    con un número; tiene que decir de cuál sacó la sugerencia, y si cita una que no se le dio, la
    sugerencia **se tira** y no la ves. Lo vigila `sintesis::pruebas::una_sugerencia_sin_fuente_dada_se_descarta`,
    y el tipo que llega a la banda solo se puede fabricar pasando por esa comprobación (no compila de
    otra forma).
  - **Tampoco le deja decir lo que la ficha no dice.** Citar bien no basta: cada palabra con
    contenido de la sugerencia tiene que estar en la ficha que cita, las cifras tienen que ser las
    mismas («cuatro» es 4) y no puede poner ni quitar un «no». Si falla, la sugerencia se tira y te
    queda la ficha. Nació de medir el modelo de verdad: citó bien y dijo «el taller de cierre va
    aparte» cuando la propuesta lo incluye. Lo vigilan `sintesis::fiel::pruebas` y
    `sintesis::pruebas::una_linea_que_la_ficha_citada_no_dice_se_descarta`. Si el titular que propone
    el modelo no sale ni de la ficha ni de la frase de tu cliente, ves el titular de la ficha
    (`…un_titular_inventado_se_cambia_por_el_de_la_ficha`).
  - **No te hace esperar.** La ficha sale primero y sin esperar al modelo; la sugerencia llega
    después. Si tarda más de **6 segundos**, no se enseña (`…pasado_el_techo_no_hay_sugerencia…`).
  - **Al proveedor externo solo sale texto corto y anonimizado en tu Mac:** la última frase de tu cliente —palabras de un tercero— y las tres
    fichas, con los nombres de tus clientes, los correos, los teléfonos, los números largos y los
    nombres de personas cambiados por marcadores antes de salir; la respuesta vuelve con los nombres
    puestos otra vez, en tu Mac. Nunca audio, pantalla ni documentos enteros: ese camino no existe
    (`api::pruebas::lo_que_sale_al_api_no_lleva_los_nombres_plantados`). Cada byte que sale se
    cuenta en *Honestidad* y en la banda.
  - **Puedes ver exactamente lo que salió · desde Sprint 003.** En *IA*, cuando algo salió —en una
    reunión, o en un ensayo con «Enriquecer el banco»—, aparece **«Ver lo que salió · N»** en la
    tarjeta del proveedor externo. Abre el texto exacto de la última petición —con lo que se tapó en
    tu Mac **tachado** y su marcador al lado—, cuántos caracteres fueron, y la tabla de las
    **peticiones recientes**, las últimas 20 (hora, para qué salió —«redactar sugerencia» o «enriquecer
    el banco»—, caracteres, datos tapados y USD). Ese registro vive en memoria como todo lo demás: se
    borra al terminar la sesión, con `⌥⎋` o al salir de la app. «← Quién redacta» te devuelve.
  - **Nada de la sugerencia va al log**: solo quién, cuánto tardó y cuánto salió
    (`tests/unit/logs-de-la-sintesis.test.ts`).
- **Costo:** el modelo del sistema cuesta cero. El externo se cobra en tu cuenta del proveedor; IA
  enseña **esta reunión**, **este mes** y el **tope del mes (USD 10)**. Al llegar al tope, vuelve sola
  a lo local; la reunión no se detiene. La cifra del mes es lo único que se guarda (un número, no el
  texto de nada).
- **Medido** (kit de prueba, `el_kit_de_sugerencias_mide_grounding_y_latencia`): de las 30
  preguntas, 17 llegan a ficha. Con **el modelo del sistema** (Apple Intelligence, en un Mac con
  macOS 26, 2026-09-26): entre **11 y 13 de 17** sugerencias pasan la comprobación —varía de una
  corrida a otra—; el resto se tira y queda la ficha. Tarda **~0,8 s de mediana y ~1 s en el peor
  5 %** (el presupuesto es 4 y 6). Con el proveedor de prueba de la CI, 17 de 17. Antes de la
  comprobación de fidelidad, 17 de 17 «citaban bien» y 3 de ellas decían algo falso o ajeno a su
  ficha: por eso la cifra baja es la honesta.
- **Limitaciones conocidas:**
  - Las fichas que trae **la pantalla sola** no llevan sugerencia: no responden a ninguna pregunta.
  - Un nombre de persona **suelto** («Andrea») que no esté en tu corpus no se reconoce al anonimizar;
    dos palabras con mayúscula seguidas («Andrea Villalba») sí.
  - El camino **MLX** (un modelo que descargas tú) no está en este MVP: queda en el roadmap del H2, y
    en IA su fila lo dice con «En el H2». Sin Apple Intelligence, la sugerencia local no está.
  - La sugerencia se escribe **en el idioma de la ficha que cita**, no en el de la pregunta: si tu
    cliente pregunta en inglés y tu propuesta está en español, la sugerencia sale en español.
  - A veces la sugerencia es fiel a su ficha pero **no contesta la pregunta** (cita otra de las tres
    fichas del momento). No es falsa; es poco útil. La confianza suele decirlo.
  - Los interruptores de IA **se recuerdan** · desde Sprint 003: «Redactar sugerencias» y el
    proveedor externo quedan como los dejaste. Con una excepción, a propósito: si borraste la clave
    del proveedor en «Acceso a Llaveros», el API **no** se enciende solo al abrir la app.
  - El precio de cada proveedor está escrito en la app con su fecha (2026-09-26); si el proveedor lo
    cambia, el costo que ves se desvía hasta la versión siguiente.

### La puerta local: que Claude Code opere la app · Nuevo · Sprint 003

- **Qué hace:** deja que **Claude Code**, en tu propio Mac y en tu sesión, opere la app por un
  comando, `ghost`: buscar en tu corpus, reindexarlo, correr el kit de evaluación con tus preguntas,
  leer y cambiar algunas preferencias y abrir tus notas guardadas. **Nace cerrada**, la abres tú, y
  **en reunión o durante un ensayo se cierra sola** y lo deniega todo: un agente no toca jamás lo que
  está vivo en memoria. No sale a la red —es un canal local del Mac, que solo alcanza tu usuario— y el contador sigue en 0 B.
- **Cómo se usa:**
  1. Compila `ghost` una vez, junto a la app: `pnpm ghost` en la carpeta del proyecto.
  2. En *IA*, **«Puerta local · cerrada»** (en la fila de «Redactar sugerencias») → se abre la vista de
     la puerta → el conmutador **«cerrada»** la abre.
  3. La vista enseña el comando, con **«Copiar»**: la ruta de `ghost` y `--help`. Pégaselo a Claude
     Code; `ghost --help` le explica lo que puede pedir.
  4. **La primera vez que Claude Code use la puerta, macOS te pregunta** si `ghost` puede usar la
     información guardada en «Angel Ghost · puerta» de tu llavero, con la contraseña de tu Mac. Esa es
     la llave: si no la das, no entra. Se vuelve a preguntar **una vez cada vez que abres la puerta**.
  5. **«Qué hizo tu agente»** lista cada orden, **también las denegadas**, con su hora y su motivo, sin
     el contenido: dice «ghost corpus buscar», no qué buscó.
  6. Para cerrarla, el mismo conmutador. Se cierra también al salir de la app.
- **Lo que puede pedir** (`ghost --help`): `corpus buscar <texto>` · `corpus reindexar` ·
  `kit <preguntas.json>` (por ejemplo, `docs/kit-de-prueba/preguntas.json`) · `prefs leer` ·
  `prefs cambiar <clave> <valor>`, solo con `idioma-consultor`, `idioma-cliente`, `retencion`,
  `ventana-de-la-bandeja` y `lectura-automatica` · `notas listar` · `notas abrir <archivo>`, que te
  pide **Touch ID o tu contraseña en el Mac** una vez cada vez que abres la puerta: aunque hayas
  exportado en *Notas*, la puerta pide el suyo.
- **Lo que no puede nunca:** nada en reunión (si la app escucha, si hay una reunión en solo notas, si ve
  una videollamada abierta **o si no puede saberlo**) ni durante un ensayo; encender el API externo;
  cambiar «Redactar sugerencias», el proveedor externo, «Conservar mis turnos» ni lo que respondiste de una NDA; abrirse
  sola; tocar otra máquina. Todo eso se deniega y queda en el registro.
- **Si se cerró sola:** *IA* lo dice en ámbar, «Se cerró sola: hay una reunión o un ensayo». Vuelve a
  abrirla tú cuando termine; no se abre sola.
- **Limitaciones conocidas:**
  - `ghost` **no se instala en tu PATH**: vive junto a la app que compilaste (y dentro del `.app` si la
    empaquetas), y la vista de la puerta te da su ruta. Instalarlo en el PATH llega con la app firmada.
  - Busca, reindexa y mide sobre **el corpus que la app tiene indexado**: al arrancar, la app vuelve a
    leer en segundo plano la carpeta que recuerda; si acabas de abrirla, espera a que *Corpus* diga
    cuántos documentos hay. Añadir carpetas se hace a mano, en *Corpus*.
  - El kit mide **la búsqueda** (nDCG@5 y lo que no encuentra); no compara modelos.
  - El registro guarda las últimas 50 órdenes de esta sesión de la app; se ven tres y el resto se
    desplaza. Al salir, se borra.

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
  - **Por HDMI, DisplayPort o AirPlay** —el altavoz de un monitor, un televisor— **no habla**: esas
    conexiones casi siempre son un altavoz, y la app las trata como los altavoces del Mac.
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
- **Dónde verlo · nuevo en la fase 3 del Sprint 002:** *Idioma* enseña cuántos términos tiene, cuántos
  salieron de tu corpus y cuántos de tu archivo, y la ruta entera del archivo.
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
  - **Los términos de seis letras o menos solo se corrigen si tú escribes cómo se oyen.** «DAX»
    está a una letra de «das», «dos», «tax» y «max», y «fabric» a una de «fábrica»: corregir por
    parecido ahí estropearía más de lo que arregla. Y un nombre de cliente de menos de ocho letras
    solo se corrige si se oyó exacto.
  - **Los nombres de tus clientes no se guardan en el archivo.** Salen de tu corpus cada vez que
    empiezas sesión, así que ese archivo no contiene el nombre de nadie.
  - **Cada pista escucha un idioma.** Si en mitad de una frase castellana el cliente dice tres
    palabras en inglés, el diccionario arregla la jerga que reconozca, pero **una frase entera en el
    otro idioma no se transcribe bien** — está medido y está dicho en la pantalla de *Idioma*. Marcar
    varios idiomas por pista no está en la app.

## Atajos de teclado

| Tecla | Qué hace |
|---|---|
| `⌥⎋` | corta todo: audio, transcript, banda y relleno. Devuelve la ventana de la reunión |
| `⌃⌥A` | «ayúdame con esto»: busca una ficha sobre lo último que dijo el cliente; **en solo notas, con la última línea de tu nota** |
| `⌃⌥T` | muestra u oculta el transcript junto a la ficha; con la banda compacta, la agranda |
| `⌃⌥V` | enciende o apaga el **modo solo audio**: te lee la ficha y la banda baja a una línea |
| `⌃⌥L` | **lee la pantalla una vez, ahora** — también con la lectura automática apagada |
| `⌃⌥R` | **qué ve**: abre *Sesión* con la tabla del radar — qué programa de tu Mac te mira y qué alcanza a ver |
| `⌃⌥N` | **anotar**: el cuaderno al frente, en *Notas*, con el cursor al final de tu nota |
| `⌃⌥P` | **fijar** la ficha que ves en la banda: queda en tus notas, con su titular y su fuente; la banda dice «fijada» |
| `⌃⌥↵` | **guardar la propuesta** que ves en la banda: entra en tu archivo |
| `⎋` | calla la voz — **solo mientras el modo solo audio está encendido** |
| `⌃⌥B` | **la banda, arriba o abajo**: la recoloca y vuelve a acoplar la reunión (con la banda cerrada tras `⌥⎋`, solo cambia tu elección). Se recuerda |

En la pantalla *Ensayo*, y solo ahí, las teclas son de la ventana: **Enter** listo · **R** repetir ·
**S** saltar · **Esc** terminar. No se disparan con el cursor en un selector.

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
En Google Meet sobre macOS 26.6.2, con la banda abajo, no: verá tu fondo de escritorio en ese trozo.
Con la banda arriba —su sitio de fábrica— usa la misma protección, pero está sin verificar. En Zoom y
Teams tampoco está comprobado, y la app te lo dice en vez de prometértelo.

**¿Sube mis documentos a algún sitio?**
No. Los lee donde están y el índice se queda en tu Mac. El contador de *Honestidad* marca los bytes
que salieron de tu equipo: **0** salvo que enciendas el proveedor externo en *IA*, y aun entonces solo
sale texto corto y anonimizado: con «Redactar sugerencias», la última frase de tu cliente y tres fichas
cortas; con «Enriquecer el banco», los títulos y la primera frase de las secciones de tu propuesta y de
la ficha del cliente, una vez por ensayo —nunca un documento entero—.

**¿Guarda lo que se habla en la reunión?**
No. El audio vive treinta segundos en memoria y se va pisando; el texto, los últimos doce turnos.
Al cerrar, de lo que se habló no queda nada salvo lo que tú decidas: tu nota, tus acuerdos, tus
fichas fijadas, las propuestas que guardes (del cliente, un hecho de una línea, nunca su frase) y
—solo si enciendes «Conservar mis turnos»— lo que dijiste **tú**, en texto. Cifrado, en un archivo
por reunión. Lo que no decidas espera en la bandeja y se borra solo (3 h de fábrica, 24 h como mucho).

**¿El ensayo graba mi voz?**
No. Tu voz pasa por la memoria para transcribirse y se pisa; el audio no toca el disco. Si guardas el
ensayo, quedan tus respuestas en texto y sus cifras, cifradas con la llave de tus notas y con su
retención. Si no lo guardas, al cerrar no queda nada. Y con una videollamada abierta solo empieza con
auriculares: así tu micrófono no oye a nadie más.

**¿Necesito internet?**
Solo para la videollamada. La app transcribe, busca y —con el modelo del sistema— redacta dentro de
tu Mac. Toca la red en dos casos, y los dos los enciendes tú: cuando le pides instalar el modelo de
voz de un idioma —el botón **«Instalar»** de *Idioma*; entonces macOS lo descarga y no sale nada de
aquí—, y si enciendes el **proveedor externo** en *IA*, con tu clave.

**¿Por qué me pide auriculares?**
Con los altavoces, tu micrófono oye también al cliente y las dos pistas se mezclan. La app lo
detecta y lo marca, pero funciona mejor con auriculares. En el ensayo, con una videollamada abierta, son
obligatorios —de cable, Bluetooth o USB—: con los altavoces, tu micrófono oiría la llamada y la llamada oiría
la voz del ensayo.

## Historial

| Sprint | Features añadidas a este manual |
|---|---|
| 001 | la banda protegida · el acople · las dos pistas y la transcripción local · el corpus indexado · la ficha de evidencia y la sugerencia de cómo conducirse · el modelo de voz de un idioma · el corte y la pantalla de Honestidad · español e inglés |
| 002 | el disparo por silencio · **tu diccionario técnico** · **el modo solo audio** · **la lectura de pantalla** y `⌃⌥L` · **los porqués** (la pista que no abrió, la salida de audio por su nombre, el motor que falta) · **por qué llegó la ficha y cuánto tardó** · las teclas pasan a `⌃⌥` · **el radar** (ámbar y coral) y `⌃⌥R` · **la sugerencia** y la pantalla **IA** |
| 003 | **tus notas** (el cuaderno de la reunión, el archivo cifrado por reunión, la retención, exportar y borrar) · `⌃⌥N` y `⌃⌥P` · **el cuaderno protegido** mientras la reunión está abierta · **lo que salió al API**, en *IA* · **las propuestas por reglas y la bandeja** con su cuenta atrás y su borrado con la app cerrada · `⌃⌥↵` · la señal «fijada» · el corte pasa a 11 piezas · las preferencias se recuerdan · tus notas pasan a la **carpeta privada de la app**, con «Mostrar en Finder», y se borran solas al vencer aunque no abras la app · **el marco en la mano**: «Este cliente», su bandera de jurisdicción, el chequeo de NDA, la cláusula de encargo y el **modo solo notas** · **la puerta local para Claude Code** (`ghost`), en *IA* |
| 004 | **el ensayo** (las preguntas por reglas de tu propuesta, tu ficha y las objeciones típicas; la evaluación sin puntaje: usaste, tenías y no usaste, tiempo, ritmo y muletillas; el informe; guardarlo con la llave y la retención de tus notas; tu progreso por cliente; exportar y borrar) · **«Enriquecer el banco»**, en *IA* · **la banda arriba**, junto a la cámara, con `⌃⌥B` y el aviso de la primera vez · el acople arriba **baja y encoge solo la ventana de la reunión**, al iniciar la sesión · el corte pasa a 12 piezas · tras su auditoría: con una videollamada abierta el ensayo pide auriculares, *Sesión* avisa antes de descartar un ensayo sin guardar, la puerta se cierra también con un ensayo y lo dice, y la protección arriba se declara sin verificar |

> **Corregido tras la auditoría del sprint 001** (2026-09-22): tres frases de este manual habían
> dejado de ser ciertas y se arreglaron con lo que el código hacía de verdad — el disparo por
> silencio, que entonces no existía y se declaró como limitación (**se cableó en el sprint 002**, y
> lo que dice este manual arriba es lo que hace hoy); la instalación del modelo de voz, que no tenía
> botón y ahora lo tiene; y la ficha automática, que no llegaba a la banda por un defecto del
> puente. El detalle está en `sprints/SPRINT_001-auditoria.md`.
