# Angel Ghost en Windows · manual de uso

> **Tu propia experiencia, en la reunión, en el momento justo — y solo tú la ves.**
>
> Este manual es para quien usa Angel Ghost en un PC con Windows y nada más: no hace falta haber visto la app en otro sistema. Tu Windows está en inglés, así que cada pantalla y cada botón va con su rótulo en inglés entre paréntesis. Lo que este manual llama **sin verificar** lo construyó y lo probó la integración continua, pero todavía no lo ha corrido nadie en tu PC con una reunión real: va al gate de prueba (`docs/GUIA-DE-PRUEBA-WINDOWS.html`).

## Qué es

Una ventana pequeña en tu PC que el cliente no ve aunque compartas la pantalla. Escucha las dos partes de la videollamada (tu micrófono y lo que suena en el PC), lee la pantalla de la reunión solo cuando cambia, y te pone delante **fichas de evidencia de tu propio corpus**: un titular, una línea y la fuente exacta. Si lo enciendes, un modelo redacta una sugerencia breve. Antes de la reunión puedes ensayar con las preguntas que ese cliente probablemente hará.

Lo que la distingue: **no persiste lo que no debe, y se puede comprobar.** El audio, el transcript del cliente y lo leído de la pantalla viven solo en memoria y mueren al cerrar. Lo que queda es lo tuyo, cifrado. La app no esconde nada de ti ni evade nada: le pide a Windows que su banda no salga en las capturas, y dice con todas sus letras dónde eso no está verificado.

## Lo que necesitas

- **Windows 10 versión 2004 (compilación 19041) o posterior**, en un PC x64 o ARM64 (por ejemplo, una Surface con Snapdragon). En un Windows anterior, la banda no se puede ocultar de la captura, y la app no la abre sin esa protección: lo dice en un cuadro al arrancar y se cierra.
- Un **micrófono** y **auriculares** (para la ficha al oído y el ensayo, la voz no sale por altavoces).
- **Teams, Zoom o Meet** (en Chrome, Edge, Firefox o Brave) para las videollamadas.
- Unos **1,6 GB libres** para el modelo de voz (0,5 GB en un PC ARM64), más unos 6 a 9 MB del instalador.
- Opcional: **Windows Hello** configurado (PIN, huella o cara) para abrir lo cifrado, y una **clave de Groq o de Claude** solo si quieres la sugerencia redactada.

## Instalar

1. **Qué archivo.** La integración continua del repositorio publica dos paquetes. En un PC x64, `angel-ghost-windows`; en una Surface o cualquier Windows ARM, `angel-ghost-windows-arm64` (es nativo: el x64 correría emulado). Cada uno trae un `.exe` (para tu usuario, sin administrador) y un `.msi` (por equipo, pide administrador). **Instala solo uno de los dos**: comparten identificador.
2. **El aviso de Windows.** Como el instalador no está firmado, SmartScreen avisa: en tu Windows en inglés dice «Windows protected your PC». Pulsa **More info** y luego **Run anyway**. Si tu cuenta no es de administrador y eliges el `.msi`, pide a quien administre el PC que lo instale.
3. **Dónde queda.** El `.exe` va a `%LOCALAPPDATA%\Angel Ghost`. El `.msi` va por equipo, a `C:\Program Files\Angel Ghost` en un PC limpio; **si antes estuvo instalado el `.exe`, reutiliza esa carpeta** (visto en la Surface de referencia). Tus datos viven aparte, en `%LOCALAPPDATA%\com.aiapps.copiloto-consultor\`: los modelos, tus preferencias, tu diccionario y el índice del corpus.
4. **Cambiar de `.exe` a `.msi` (o al revés).** Desinstala primero: **Settings → Apps → Installed apps → Angel Ghost → Uninstall**. Si el desinstalador ofrece **Delete the application data**, déjalo sin marcar para conservar tus modelos y tu diccionario.
5. **Lo verificado.** El `.exe` y el `.msi` ARM64 se instalaron y abrieron en una Surface (Snapdragon X Plus) en los sprints 009 y 010. El instalador x64 lo construye y lo prueba la integración continua en cada cambio; todavía no se ha instalado a mano en un PC x64.

## La primera vez: el modelo de voz

La app transcribe con **whisper**, dentro de tu PC; ningún audio sale. El modelo no viene dentro del instalador: se baja una vez.

- En **Idioma (Language)** pulsa **Instalar (Install)**. En un PC x64 baja el modelo grande (`large-v3-turbo`, 1,6 GB); **en un PC ARM64 baja el pequeño** (`small`, 0,5 GB), porque el grande no cabe en el techo de 4 segundos de esos equipos. Antes de usarlo se comprueba su huella SHA-256: si no coincide, se borra.
- **No baja nada con una reunión, una sesión (tampoco «solo notas» o presencial) ni un ensayo abiertos**, y si uno empieza mientras baja, la descarga se para. Si la conexión se cuelga, a las tres horas se suelta sola.
- Al terminar, la app mide lo que tarda tu PC. En **Honestidad (Honesty)**, el bloque de la transcripción enseña **una línea por modelo, pista por pista**: tu turno de 10 s (la cifra que decide el modelo) y el turno del cliente, de 6 s. Cada línea dice si cabe en el techo de 4 s (✓), si pasa (⚠) o si falta medirla o bajarla (○).
- **Medir otra vez (Measure again)** repite la medida: con el modelo grande tarda cerca de medio minuto, y mientras tanto la pantalla lo dice («Midiendo…»).
- La regla: se transcribe con **el mayor modelo cuya pista tuya cabe en 4 s en tu PC**; uno que no se ha medido cuenta como que cabe. En la Surface de referencia: `small`, 2,0 s tu pista y 1,6 s la del cliente; `large-v3-turbo`, 9,8 s y 9,7 s. Son las de la corrida en vivo de esa Surface; las cifras cambian de una medición a otra, y el manual de uso general cita el intervalo de varias (el pequeño, cerca de 1,6 s; el grande, de 8,5 a 18 s).

## Una reunión, de punta a punta

1. **Antes.** En **Corpus** señalas la carpeta con tus propuestas, marcos y casos (PDF, Word o Markdown); se indexa en tu PC. En **Sesión (Session)** eliges «Este cliente»: su jurisdicción, la cláusula del encargo y el chequeo de la NDA; si la NDA prohíbe transcribir, pasas a «solo notas». Eliges si la banda va arriba (por defecto, junto a la cámara) o abajo. Si ese cliente debe saber que usas esta herramienta, enciendes su **modo transparente** (más abajo).
2. **Entras en la llamada** (Teams o Zoom cuando estás en una llamada; Meet por el título de la pestaña). Pulsa **Iniciar sesión (Start session)**: se abren tu micrófono y el audio del sistema —dos pistas, sin bot en la reunión—, y la ventana de la reunión baja y se encoge lo justo para que la banda no la tape.
3. **Durante.** Cuando el cliente termina una pregunta, la banda trae su ficha en menos de 4 s. La ficha es pasiva: no suena ni te roba el foco. Si el corpus no tiene nada, la banda te da una **maniobra** (aplazar con fecha, pedir el dato, reformular…) en vez de un «sin resultado» mudo. Con auriculares, **«Al oído»** (`Alt+Mayús+W`) te lee la ficha. `Alt+Mayús+H` es «ayúdame con esto».
4. **Al cerrar.** Guardas tus notas y acuerdos cifrados, con la retención que elijas. Lo del cliente muere. Las propuestas que no decidiste te esperan en una **bandeja con cuenta atrás** (defecto 3 h, techo 24 h).

## La banda

Es la ventana que solo tú ves. Le pide a Windows que no salga en las capturas, y **en Teams, Zoom y Meet eso está sin verificar**: Honestidad y Sesión lo dicen con esas palabras hasta que lo pruebes en tu PC con una videollamada real. Mientras tanto, **comparte una ventana, no la pantalla completa**.

- **Acople.** La ventana de la reunión baja y se encoge hasta debajo de la banda, a la altura que le hayas dado, al iniciar la sesión. Vuelve a su sitio cuando terminas la sesión con **Terminar sesión (End session)**, cierras la banda con su ×, cortas con `Alt+Mayús+Q` o sales de la app (y, si la app se cae, la próxima vez que la abras). Windows no pide ningún permiso para esto. Si la mueves a mano mientras tanto, la app no la toca. Sin verificar: una ventana maximizada, otro monitor, la escala al 125 % o 150 %, y que Teams o Zoom se nieguen a encogerse (entonces se deshace y la banda flota encima).
- **Tamaños.** 88 px; se amplía a 200 con el asa; 44 px en modo solo audio.
- **Altura.** Con la banda arriba, la rayita de su borde de arriba se arrastra hacia abajo para dejar a la vista lo que la banda tapaba —las pestañas y la barra de direcciones de tu navegador, por ejemplo—. La altura se guarda y la banda sale ahí al abrir la app; la app la acota a tu pantalla para que la banda no salga de ella; si debajo no queda sitio para una reunión usable, la reunión no se toca y la banda flota sin acople. En **Sesión (Session)**, la fila **Altura de la banda (Band height)** dice cuánto la bajaste y la cambia sin arrastrar: **Bajar (Lower)** y **Subir (Raise)**, de 40 en 40 puntos, y **Volver arriba (Back to the top)**, que la pega al borde. Con la banda abajo no hay altura que elegir.
- **Cerrar.** El **×** de la esquina de arriba a la derecha cierra la banda **sin cerrar la app**. Con una sesión en marcha la termina (como **Terminar sesión**, que también devuelve la reunión acoplada) y cierra la banda (su nombre lo dice: «Cerrar la banda y terminar la sesión»). `Alt+Mayús+Q`, que corta todo, también cierra la banda. Con la banda cerrada, **Sesión (Session)** dice «La banda está cerrada» y **Mostrar la banda (Show the band)** la abre en su borde y a su altura; **Iniciar sesión (Start session)** también la trae de vuelta.
- **Salir.** **Cerrar la ventana principal cierra la app entera**, la banda incluida. Si había una sesión abierta, se corta, y lo que escribiste sin guardar se guarda cifrado. La banda sola se cierra con su ×.

## Las dos pistas y la ficha

- **Quién habla** se resuelve por pista, jamás por la voz: tu micrófono eres tú; lo que suena en el PC es el cliente. La app **no separa voces**: una notificación o una canción que suene cuenta como del cliente.
- **Si cambias de salida de audio** con la sesión en marcha (unos auriculares Bluetooth, por ejemplo), la app vuelve a abrir el audio del sistema sola, en un par de segundos.
- **La voz de la propia app no te dispara fichas:** lo que dice por tus auriculares vuelve por el audio del sistema y se marca como eco.
- **Si tu PC no da abasto.** La transcripción atiende las dos pistas una tras otra y el cliente va primero. Si se acumulan más de 4 turnos, se descartan primero los tuyos más viejos; Honestidad cuenta cuántos, sin decir de quién eran.
- **La pantalla.** Lee la ventana de la reunión **solo cuando cambia**, con el reconocimiento de texto de Windows (necesita el paquete de idioma de texto: **Settings → Time & language → Language & region**). `Alt+Mayús+D` la lee una vez, ahora. La imagen vive en memoria y se descarta al leerla.

## Atajos

Son `Alt+Mayús` y una tecla; ninguno usa `Ctrl+Alt`, que en muchos teclados es AltGr. Si otra app ya tiene una combinación, Windows no deja registrarla: Honestidad lo dice, la banda avisa («N teclas no responden») y la acción se hace igual desde la app.

| Tecla         | Qué hace                                                             |
| ------------- | -------------------------------------------------------------------- |
| `Alt+Mayús+Q` | corta todo: audio, transcript, banda y relleno                       |
| `Alt+Mayús+H` | «ayúdame con esto»                                                   |
| `Alt+Mayús+C` | muestra u oculta el transcript junto a la ficha                      |
| `Alt+Mayús+W` | enciende o apaga el modo solo audio                                  |
| `Alt+Mayús+D` | lee la pantalla una vez, ahora                                       |
| `Alt+Mayús+U` | qué ve el radar en tu PC                                             |
| `Alt+Mayús+K` | anotar                                                               |
| `Alt+Mayús+F` | fijar la ficha                                                       |
| `Alt+Mayús+Z` | guardar la propuesta                                                 |
| `Alt+Mayús+X` | la banda, arriba o abajo                                             |
| `Esc`         | calla la voz. Solo existe mientras el modo solo audio está encendido |

## Tu corpus y las fichas

Arrastras o señalas propuestas, marcos, casos, fichas de cliente y tu perfil. La app entiende cinco unidades (propuesta, marco, caso, ficha de cliente y tu perfil) y la pregunta del cliente cae en la correcta. El índice vive en tu PC, en claro, dentro de la carpeta privada de la app. Una **ficha** es un titular de ocho palabras, una línea y la fuente exacta (documento y sección). Todo esto es determinista: no usa ningún modelo para buscar.

## Tus notas, la bandeja y lo que vence

- **Se guardan cifradas.** La llave está en el **Administrador de credenciales (Credential Manager)** de tu usuario de Windows, nunca en un archivo.
- **Abrir lo cifrado pide Windows Hello** (tu PIN, tu huella o tu cara), una vez por sesión de la app: exportar una reunión, abrir tus ensayos y abrir la bandeja de una sesión anterior. Si tu PC no lo tiene configurado, la app dice dónde hacerlo (**Settings → Accounts → Sign-in options**) y tus notas siguen cifradas.
- **La app propone qué guardar**, con reglas publicadas y sin modelo: cifras, plazos, tus compromisos («te lo mando», «quedamos en»), lo que choca con una ficha que fijaste. Proponer no es guardar: nada entra sin tu sí, y del cliente nunca se guarda su turno, solo un hecho en una línea.
- **Lo que vence se borra con la app cerrada.** La app deja una tarea en el **Programador de tareas (Task Scheduler)** llamada `com.aiapps.copiloto-consultor.vencimiento`, en la biblioteca raíz, que corre al minuto de cada vencimiento aunque no abras la app (si el PC estaba apagado, corre al volver). Lo que corre es PowerShell sobre una lista de la app: solo borra archivos `.ghost` vencidos, no escribe nada y no lanza la app. Cuando ya no vence nada, la tarea se quita sola. Si una directiva de tu empresa bloquea PowerShell, Honestidad dice que no corrió.
- **Las copias de seguridad.** Si haces copias de seguridad de tu PC, esas copias conservan lo borrado hasta que caduquen: la app no controla dónde van.

## El modo presencial

En la mesa con el cliente, o al lado del PC donde corre una videollamada, la app escucha **la sala por el micrófono del PC**: una sola pista, sin audio del sistema ni acople. En la sala nadie tiene dueño: el transcript dice «sala», nunca «cliente» ni «tú». Si la videollamada está en este mismo PC, usa **«reunión en este PC»**, no presencial: así oye al cliente por el audio del sistema. Si tu entrada de audio por defecto son unos auriculares Bluetooth, la app oiría sobre todo tu voz: Sesión lo avisa y dice dónde cambiarlo (**Settings → System → Sound → Input**).

## El ensayo

Antes de la reunión, la app te hace las preguntas que ese cliente probablemente hará, derivadas de tu propuesta por reglas publicadas y un catálogo de objeciones. **La voz de Windows las lee** (con auriculares; necesita una voz del idioma: **Settings → Time & language → Speech**); tú contestas en voz alta y, al terminar, ves qué evidencia de tu corpus citaste y cuál te faltó, cuánto tardaste y qué muletillas repetiste. Lo guardado es tuyo (tu texto y tus cifras, jamás tu audio) y va cifrado.

## La sugerencia (opcional)

En Windows no hay modelo del sistema: la sugerencia la redacta el **API** que eliges en **IA (AI)**, Groq o Claude, **con tu propia clave**, guardada en el Administrador de credenciales. Nace apagada; encendida, solo sale texto minimizado y anonimizado, bajo la retención que cada proveedor publica y que la app te enseña donde eliges. Cada envío se cuenta en Honestidad. Sin API, la ficha sigue siendo la respuesta.

## El radar

Mira los programas de **este PC** contra un catálogo con fuente y fecha por fila: supervisión de exámenes, monitoreo de empleados, acceso remoto (TeamViewer, AnyDesk, Asistencia rápida) y si el PC está gestionado por tu empresa. Avisa, no bloquea ni cierra nada, y no mira nada fuera de tu equipo. `Alt+Mayús+U` enseña lo que ve.

## La puerta local para Claude Code

Deja que **Claude Code**, en tu sesión y en este mismo PC, opere la app por un comando, `ghost`. No es un servidor ni abre ningún puerto de red: es un canal de este equipo que solo alcanza tu usuario de Windows.

- **Nace cerrada.** En **IA (AI)** pulsa **Puerta local (Local door)** y ábrela con su conmutador. Al abrirla, Windows crea un _named pipe_ que **solo tu usuario puede abrir** (otro usuario del PC, o cualquier equipo de la red, recibe «acceso denegado»), y la app deja una llave nueva en el **Administrador de credenciales (Credential Manager)**: «Angel Ghost · puerta / token». `ghost` la lee para identificarse. Al cerrar la puerta, o al salir de la app, el pipe y la llave desaparecen.
- **Se cierra sola** en cuanto empieza una reunión, una sesión (también «solo notas») o un ensayo, y no se abre mientras haya uno. **IA** lo dice: «Se cerró sola: hay una reunión o un ensayo».
- **Lo que puede pedir tu agente** (`ghost --help`): buscar en tu corpus, reindexarlo, correr el kit de evaluación, leer y cambiar algunas preferencias, listar tus notas guardadas y abrir una (eso pide **Windows Hello**, una vez cada vez que abres la puerta). Es una lista cerrada: nada más.
- **Lo que no puede nunca:** nada en reunión ni en un ensayo; encender el API externo; cambiar lo que respondiste de una NDA; abrirse sola; tocar otra máquina. Todo eso se deniega y queda en el registro. **Qué hizo tu agente (What your agent did)** lista cada orden, también las denegadas, sin su contenido.
- **`ghost.exe` no viaja en el instalador:** se construye desde el código con `cargo build --bin ghost -j 4`, y la vista de la puerta te da su ruta.
- **Para deshacerla,** cierra la puerta. Si una caída dejara la llave, la app la borra sola al arrancar; a mano: **Control Panel → Credential Manager → Windows Credentials →** «Angel Ghost · puerta / token» **→ Remove**.
- **Sin verificar:** usarla con tu agente de verdad, y que Windows Hello aparezca al abrir una nota. La política y el pipe los prueba la integración continua.

## El modo transparente

Hay clientes y jurisdicciones que piden saber que usas una herramienta que escucha la reunión, y otros a los que prefieres decírselo tú. El **modo transparente** es el aviso y la constancia, por cliente, sin que nada salga de la app.

- **Se enciende por cliente.** En **Sesión (Session)**, en «Este cliente», elige al cliente: debajo de su NDA aparece el interruptor **Modo transparente (Transparent mode)**. Se recuerda por cliente, como la NDA, y se apaga con el mismo interruptor.
- **El aviso es fijo.** Encendido, **Lo que le dices (What you tell them)** enseña el texto en español y en inglés, el de tu idioma primero: usas una herramienta en tu propio equipo que escucha la reunión para traerte tu propia documentación, no graba el audio ni guarda lo que dice, y todo vive en memoria y se borra al terminar. No lo redacta ningún modelo.
- **Al oído.** Con el modo solo audio encendido y auriculares, **Decírselo en voz alta (Say it out loud)** te lo lee la voz de Windows, en el idioma del cliente, para que se lo digas. Por altavoces no habla, y la pantalla lo dice.
- **«El cliente consintió» (The client consented).** Cuando el cliente dice que sí, pulsa el botón: la app anota en las **notas cifradas de esa reunión** una constancia con el cliente, su jurisdicción, la hora, el idioma y el texto que se leyó. Solo se puede con la sesión en marcha; sin ella el botón se ve apagado y lo dice, y si la reunión se cerró justo antes, la pantalla lo dice en vez de fingir que anotó.
- **Una marca en la banda.** Mientras el modo está encendido, la banda lleva **Modo transparente (Transparent mode)** con un ojo. La banda solo sabe eso: no ve el nombre del cliente ni el aviso.
- **Qué se guarda.** La constancia va al archivo cifrado de la reunión: **Se va a guardar (Will be saved)** la cuenta al cerrar y **Honestidad (Honesty)** la nombra en «Tuyo». El audio y el transcript del cliente siguen muriendo al cerrar, y el contador de red sigue en 0 B.
- **Sin verificar, y sin asesoría:** usarlo con un cliente de verdad. La app no sabe si tu cliente o tu jurisdicción exigen el aviso: lo enciendes tú.

## Honestidad, Permisos e Idioma

- **Honestidad (Honesty)** es la pantalla que no miente: cuántos bytes salieron de tu equipo en esta reunión (cero en modo local), qué vive en memoria ahora mismo, qué persiste y por cuánto, el bloque **«Hoy en Windows»** (qué hace la app aquí y qué no, cada cosa con su motivo) y la transcripción de tu PC.
- **Permisos (Permissions).** Solo el **micrófono** tiene permiso en Windows, y lo concedes tú en **Settings → Privacy & security → Microphone**; el botón **Abrir Ajustes de Windows** te lleva. El audio del sistema, la pantalla de la reunión y el acople no piden ninguno.
- **Idioma (Language).** Dos pistas, un idioma para cada una (español e inglés), el modelo de voz y tu **diccionario técnico**: términos que corrigen la transcripción (Power BI, DAX, nombres de clientes) en `diccionario.yaml`, que la app nunca completa ni adivina.

## Lo que Windows no hace por ahora

- **`ghost.exe` no viaja en el instalador.** La puerta para Claude Code ya está en Windows (IA → Puerta local), pero el comando `ghost` se construye desde el código: `cargo build --bin ghost`.
- **La invisibilidad en Teams, Zoom y Meet está sin verificar**: la promesa es graduada por plataforma, y solo vale lo que tu corrida en vivo verificó.
- **Windows anterior a la 2004** no puede ocultar la banda: la app no abre.
- **Cinco funcionalidades de la visión se descartaron para Windows, con nombre** (decisión del 2026-10-18): la app en el celular, en modo presencial (4), varias voces en el mismo canal (10), sin auriculares con cancelación de eco (11), transcripción en streaming nativo (12) y leer el dashboard del cliente como consultor (13). No se construyen para el producto completo de Windows.

## Las 30 funcionalidades de la visión, en Windows

Cada fila dice qué estado tiene en Windows y qué clave de «Hoy en Windows» la cubre. **hecho** = construido y comprobado de punta a punta en un PC con Windows; **hecho sin verificar** = construido y probado en la integración continua, pero lo que pide una videollamada real, tu micrófono o tu usuario de Windows delante todavía no lo ha corrido nadie en tu PC; **descartada para Windows** = decidido el 2026-10-18 que no se construye para el producto completo de Windows, con su nombre y su motivo.

| N   | Funcionalidad                                                  | Clave                                                          | Estado                  | Qué falta o por qué                                                                                                          |
| --- | -------------------------------------------------------------- | -------------------------------------------------------------- | ----------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| 1   | Banda privada protegida de la captura                          | `banda-protegida` · `acople`                                   | hecho sin verificar     | La protección se pide a Windows; Teams, Zoom y Meet, en el gate; la altura y el × de la banda, en w20 y w21                  |
| 2   | Permisos, reunión detectada, estado y botón de pánico          | `videollamada` · `atajos`                                      | hecho sin verificar     | El micrófono se concede en Ajustes; `Alt+Mayús+Q` corta todo                                                                 |
| 3   | Contador de salida a red y panel de honestidad                 | sin clave                                                      | hecho                   | Honestidad cuenta los bytes (0 B en modo local) y dice qué vive en memoria                                                   |
| 4   | La app en el celular, en modo presencial                       | sin clave                                                      | descartada para Windows | Decisión del 2026-10-18: es de otra plataforma y no se construye para Windows                                                |
| 5   | Windows                                                        | sin clave                                                      | hecho sin verificar     | x64 y ARM64, con `.exe` y `.msi` sin firmar; el ARM64 se instaló y abrió en una Surface                                      |
| 6   | Dos pistas de audio, sin bot                                   | `videollamada`                                                 | hecho sin verificar     | Tu micrófono y el audio del sistema; falta una videollamada real                                                             |
| 7   | Transcripción local con fin de turno                           | `videollamada`                                                 | hecho sin verificar     | whisper en tu PC, medido con el audio del kit; falta la voz real                                                             |
| 8   | Lectura de pantalla solo cuando cambia                         | `pantalla`                                                     | hecho sin verificar     | Reconocimiento de texto de Windows; falta una presentación real                                                              |
| 9   | Español e inglés, con vocabulario técnico                      | sin clave                                                      | hecho sin verificar     | Interfaz y diccionario, comprobados en los dos idiomas; la transcripción en inglés falta con voz real                        |
| 10  | Varias voces en el mismo canal                                 | sin clave                                                      | descartada para Windows | Decisión del 2026-10-18: no se construye; además, la app no separa voces por diseño                                          |
| 11  | Sin auriculares (cancelación de eco)                           | sin clave                                                      | descartada para Windows | Decisión del 2026-10-18: no se construye; con altavoces la voz al oído no habla                                              |
| 12  | Transcripción en streaming nativo                              | sin clave                                                      | descartada para Windows | Decisión del 2026-10-18: no se construye; el motor de Windows transcribe por turnos                                          |
| 13  | Leer el dashboard del cliente como consultor                   | sin clave                                                      | descartada para Windows | Decisión del 2026-10-18: no se construye para el producto completo de Windows                                                |
| 14  | Tu corpus indexado en tu PC                                    | `corpus-y-ficha`                                               | hecho sin verificar     | PDF, Word y Markdown; falta tu corpus real                                                                                   |
| 15  | Modelo de consultoría                                          | `corpus-y-ficha`                                               | hecho sin verificar     | Las cinco unidades; falta tu corpus real                                                                                     |
| 16  | Sabe cuándo ayudarte                                           | `corpus-y-ficha` · `atajos`                                    | hecho sin verificar     | Pregunta, término, cifra o silencio, y `Alt+Mayús+H`                                                                         |
| 17  | Fichas de evidencia                                            | `corpus-y-ficha`                                               | hecho sin verificar     | Titular, línea y fuente exacta                                                                                               |
| 18  | Maniobra cuando el corpus no tiene nada                        | `corpus-y-ficha`                                               | hecho sin verificar     | Catálogo de seis maniobras, sin modelo                                                                                       |
| 19  | Modo solo audio (voz al oído)                                  | `voz-al-oido`                                                  | hecho sin verificar     | La voz de Windows, con auriculares; por altavoces no habla                                                                   |
| 20  | Sugerencia sintetizada                                         | `sugerencia`                                                   | hecho sin verificar     | Hoy solo con el API que eliges y tu clave; el modelo local (Foundry Local) está decidido para el S13, sin ningún API de pago |
| 21  | Lo tuyo queda, lo del cliente muere: notas cifradas y bandeja  | `notas-cifradas` · `desbloqueo` · `borrado-con-la-app-cerrada` | hecho sin verificar     | Credential Manager, Windows Hello y el Programador de tareas; falta tu PC                                                    |
| 22  | Acuerdos detectados                                            | `notas-cifradas`                                               | hecho sin verificar     | La app propone y tú decides; nada entra sin tu sí                                                                            |
| 23  | Práctica guiada                                                | `ensayo`                                                       | hecho sin verificar     | La voz de Windows lee las preguntas; falta oírla en tu PC                                                                    |
| 24  | Modo presencial                                                | `presencial`                                                   | hecho sin verificar     | La sala por el micrófono del PC; falta una sala real                                                                         |
| 25  | Efímero verificable                                            | sin clave                                                      | hecho sin verificar     | El barrido y las pruebas corren en la integración continua; la corrida de una sesión completa en tu PC es del gate           |
| 26  | Bandera de jurisdicción, cláusula del encargo y chequeo de NDA | sin clave                                                      | hecho sin verificar     | Pantalla Sesión; la recorre un test, falta tu uso real                                                                       |
| 27  | Medida desde el primer día                                     | sin clave                                                      | hecho sin verificar     | Kit sintético en la integración continua; la cifra de tu PC la mide Honestidad                                               |
| 28  | Puerta local para Claude Code                                  | `puerta-local`                                                 | hecho sin verificar     | Named pipe solo de tu usuario y llave en el Administrador de credenciales; falta usarla con tu agente                        |
| 29  | Modo transparente                                              | sin clave                                                      | hecho sin verificar     | Interruptor por cliente, aviso en los dos idiomas y constancia cifrada; falta usarlo con un cliente real                     |
| 30  | Radar de captura                                               | `radar`                                                        | hecho sin verificar     | Catálogo de Windows con fuente por fila; falta qué tiene instalado tu PC                                                     |

## Preguntas frecuentes

- **La banda se quedó en pantalla y no sé cómo cerrarla.** Pulsa su **×**: se cierra y la app sigue; **Mostrar la banda (Show the band)**, en Sesión, la devuelve. Para salir del todo, cierra la ventana principal: la app sale entera. Si no la ves, ábrela desde la barra de tareas, o termina **Angel Ghost** en el Administrador de tareas (**Task Manager → Processes → End task**).
- **La banda me tapa las pestañas de mi navegador.** Arrastra hacia abajo la rayita de su borde de arriba hasta que se vean; queda guardada. En Sesión, **Bajar (Lower)** hace lo mismo sin arrastrar y **Volver arriba (Back to the top)** la pega al borde otra vez.
- **«Instalar» no baja nada.** Hay una reunión, una sesión o un ensayo abiertos, o falla la conexión: la pantalla lo dice («No se pudo bajar el modelo»).
- **Honestidad dice «sin medir».** El modelo está bajado pero no se ha medido en esta sesión: pulsa **Medir otra vez**.
- **Un atajo no responde.** Otra app de tu PC ya tiene esa combinación; Honestidad la nombra. La acción se hace igual desde la app.
- **La ficha al oído no habla.** Con altavoces, o con una salida por HDMI o S/PDIF, no habla: el cliente la oiría. Con auriculares, sí; y necesita una voz del idioma instalada.
- **¿Se puede mover la carpeta de datos?** No: es la carpeta privada de la app (`%LOCALAPPDATA%\com.aiapps.copiloto-consultor\`).

## Construirla tú

El `README.md` tiene «Desarrollar en Windows ARM». El script `scripts/entorno-windows.ps1` dice qué falta sin instalar nada; las pruebas se corren con `AG_SIN_HARDWARE=1 cargo test --locked -j 4` (el centinela evita que toquen micrófono ni permisos; `-j 4` evita que el enlazador se quede sin memoria) y el instalador sale de `pnpm tauri build --bundles nsis`.
