# ADR 022 — Windows detrás de la misma frontera

- **Fecha:** 2026-10-11
- **Sprint:** 006 «Windows arranca» (fase 2, antes del código)
- **Estado:** aceptada. Enmienda el ADR 006 (§ Enmienda) y declara la primera puerta a la red escrita en Rust (§4).

## Contexto

Angel Ghost existe solo en macOS. La VISION promete Windows en el H-2 («la misma app donde la invisibilidad está
garantizada por el sistema»), y el plan lo parte en dos: el **S6 arranca** (un instalador que sale de la CI, la banda
protegida, la sala por el micrófono con whisper local y su latencia medida) y el **S7 trae la videollamada** (el audio
del sistema, la pantalla, la voz al oído y los atajos; el acople, el desbloqueo y el radar pasan al **S8**).

Antes de escribir una línea de Windows, la lectura del código encontró algo que los comentarios negaban: **el crate no
compilaba fuera de macOS**. `relleno.rs` usaba `base64` declarada solo para macOS; la puerta local y `ghost` usaban
sockets Unix sin `cfg`; siete funciones leían la hora local con `libc::localtime_r` sin alternativa; `vencimiento`
llamaba a `getuid`; y ocho tests usaban permisos Unix sin `cfg(unix)`. Los stubs «fuera de macOS» que ya existían
(`capture`, `acople`, `stt`, `habla`, `pantalla`, `sintesis`, `desbloqueo`, `llavero`) estaban bien; el resto no.

## Decisión

### 1. Una plataforma nueva es un `windows.rs` hermano detrás de la misma frontera, no un crate ni un fork

Cada módulo nativo tiene su lado de macOS (`apple.rs`, el puente Swift, `nativo.rs`) y, desde el S6, su lado de Windows
detrás de la **misma interfaz Rust**: `capture::nativo::Grifo`, `stt::Motor`, `llavero::{guardar, leer, …}`. El resto
del crate —disparo, corpus, ficha, ensayo, voz, notas, propuestas— es portable y **no se toca**: solo compila. Lo que
una plataforma aún no hace devuelve su motivo escrito y la app lo dice en Honestidad; jamás finge.

Es irreversible en el sentido que importa: un crate aparte o una rama de Windows duplicarían los motores, y cada regla
dura (efímero, red contada, cero huellas) tendría que demostrarse dos veces.

### 2. La carpeta privada en Windows es la LOCAL, no la que se sincroniza

En Windows, `app_data_dir` y `app_config_dir` de Tauri caen en `%APPDATA%` (Roaming), que en un equipo de empresa se
puede sincronizar con un servidor. Tus notas cifradas, tu bandeja y tus ensayos no van ahí: en Windows la carpeta de la
app es `app_local_data_dir` (`%LOCALAPPDATA%\com.aiapps.copiloto-consultor`). En macOS no cambia nada (las tres rutas
son la misma, `~/Library/Application Support/<id>`). Un solo ayudante decide la carpeta (`lib.rs`).

Los permisos 600/700 de la regla 17-bis son de Unix. En Windows la carpeta hereda la ACL del perfil del usuario (solo
él, el sistema y los administradores); **una DACL propia, más estricta, queda como «todavía no»**, dicho aquí y en el
cierre del sprint.

### 3. La banda se protege igual, y el relleno no se crea

`set_content_protected` de Tauri es el mismo en las dos plataformas; en Windows pide al sistema que **excluya la
ventana de la captura** (`WDA_EXCLUDEFROMCAPTURE`, Windows 10 2004 o posterior). La diferencia decide algo: en macOS
la ventana protegida deja un hueco y el relleno lo tapa con tu fondo de escritorio; en Windows la ventana excluida
simplemente no está en la captura y se ve lo que hay detrás. **Si se creara el relleno, sería él lo que se capturase**:
una franja negra donde estaba la banda. En Windows el relleno no se crea. Que la banda no salga de verdad en Teams,
Zoom o Meet **no se afirma**: queda «sin verificar» hasta tu corrida en el PC (regla dura 6).

### 4. La primera puerta a la red escrita en Rust: el modelo de whisper

`contador-de-red` contaba tres puertas, las tres en el puente Swift. Whisper necesita su modelo
(`ggml-large-v3-turbo.bin`, 1 624 555 275 bytes) y en Windows no hay puente. Se declara **una** puerta Rust, y solo una:

- crate `ureq` (con rustls), **solo** bajo `cfg(windows)` y **solo** en `src-tauri/src/modelo_whisper.rs`;
- se abre **solo cuando lo pides**, desde Idioma («Instalar», el mismo botón que en macOS baja el modelo de voz del
  sistema), **jamás durante una reunión**;
- baja de `huggingface.co/ggerganov/whisper.cpp`, y **la huella SHA-256 está fija en el repo**:
  `1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69` (leída del puntero LFS público el 2026-10-11).
  Si no coincide, el archivo se borra y no se usa;
- no sube nada tuyo: entra el modelo, no sale texto ni audio. El contador de Honestidad, que cuenta lo tuyo que
  sale, sigue en 0 B; que la puerta se usó lo dicen el log y Honestidad (el estado del modelo).

El gate se **estrecha**, no se afloja: cuenta las puertas Rust aparte (exactamente una) y sigue prohibiendo cualquier
cliente HTTP fuera de ese archivo y de esa plataforma.

### 5. Whisper en Windows: el motor es el mismo `stt::Motor`

`stt/whisper.rs` implementa el `Motor` de siempre con `whisper-rs` (CPU, sin GPU), para `es-ES` y `en-US`. El módulo
sigue protegido (`verify-ephemeral`): no abre archivos ni red; recibe la ruta del modelo de `modelo_whisper.rs`.
**Calibración:** al quedar el modelo listo, transcribe 10 s del audio sintético del kit v4 (`data/calibracion/`) y mide
cuánto tarda; Honestidad enseña esa cifra. Si pasa de 4 s (el presupuesto de la app, del fin de turno a la ficha), lo
dice. El escalón al modelo `small` (huella `1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b`,
487 601 967 bytes) queda escrito para cuando la cifra lo pida.

### 6. Secretos en el Administrador de credenciales

`llavero/windows.rs` guarda las tres llaves (API, notas, puerta) en el **Administrador de credenciales** de Windows
(`CredWriteW`/`CredReadW`/`CredDeleteW`), con los mismos nombres de servicio que en el Llavero.

## Inventario: los 15 archivos Rust con `cfg(target_os = "macos")` y los 8 del puente Swift

| Archivo                     | Qué hace en macOS                                    | En Windows                                                                                              |
| --------------------------- | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `almacen.rs`                | excluir la bandeja de Time Machine                   | **no en Windows**: no hay Time Machine; la carpeta es local (§2)                                        |
| `permisos.rs`               | leer TCC (micrófono, audio, pantalla, Accessibility) | **S6**: el micrófono, de la privacidad de Windows; lo demás no aplica                                   |
| `bandeja.rs`                | la bandeja fuera de las copias                       | **no en Windows** (como `almacen.rs`)                                                                   |
| `desbloqueo.rs`             | Touch ID o contraseña para abrir notas               | **S8 (hecho)**: Windows Hello (`desbloqueo/windows.rs`)                                                  |
| `llavero.rs`                | el Llavero                                           | **S6**: Administrador de credenciales (§6)                                                              |
| `sintesis/api.rs`           | el POST al proveedor externo por `Red.swift`         | **S8 (hecho)**: la segunda puerta Rust, `ureq` (ADR 023)                                                 |
| `sintesis/sistema.rs`       | Apple Foundation Models                              | **no en Windows** por ahora: sin modelo del sistema; la ficha sigue siendo la respuesta                 |
| `capture/mod.rs`            | micrófono y audio del sistema (CoreAudio)            | **S6** el micrófono (`capture/windows.rs`, cpal/WASAPI) · **S7 (hecho)** el audio del sistema (loopback) |
| `sesion/mod.rs`             | qué reunión hay y su ventana                         | **S7 (hecho)**: `sesion/windows.rs` y `data/clientes-windows.json`                                      |
| `acople/mod.rs`             | hacerle sitio a la banda en la reunión               | **S8 (hecho)**: `acople/windows.rs` (`SetWindowPos`, el marco de DWM), la misma maniobra                 |
| `pantalla/apple.rs`         | leer la pantalla (ScreenCaptureKit + Vision)         | **S7 (hecho)**: `pantalla/windows.rs` (PrintWindow + Windows.Media.Ocr)                                 |
| `habla/apple.rs`            | la voz al oído (AVSpeechSynthesizer)                 | **S7 (hecho)**: `habla/windows.rs` (SpeechSynthesizer)                                                  |
| `radar/procesos.rs`         | los programas que vigilan tu Mac                     | **S8 (hecho)**: la foto de procesos y `data/radar/programas-windows.json`                                |
| `radar/mdm.rs`              | el perfil de gestión del Mac                         | **S8 (hecho)**: `IsDeviceRegisteredWithManagement`                                                       |
| `stt/apple.rs`              | SpeechAnalyzer                                       | **S6**: whisper (`stt/whisper.rs`, §5)                                                                  |
| `nativo/Transcriptor.swift` | SpeechAnalyzer                                       | sustituido por whisper (S6)                                                                             |
| `nativo/Habla.swift`        | la voz                                               | sustituido por `habla/windows.rs` (S7)                                                                  |
| `nativo/Pantalla.swift`     | la pantalla                                          | sustituido por `pantalla/windows.rs` (S7)                                                               |
| `nativo/Sintesis.swift`     | el modelo del sistema                                | no en Windows                                                                                           |
| `nativo/Llavero.swift`      | el Llavero                                           | sustituido por el Administrador de credenciales (S6)                                                    |
| `nativo/Red.swift`          | la puerta del API externo                            | sustituido por `ureq` en `sintesis/api.rs` (S8, ADR 023)                                                |
| `nativo/Desbloqueo.swift`   | Touch ID                                             | sustituido por Windows Hello (S8)                                                                       |
| `nativo/Copias.swift`       | Time Machine                                         | no en Windows                                                                                           |

**Y lo que impedía compilar** (no estaba en el inventario porque no llevaba `cfg`): `relleno.rs` (`base64` pasa a todas
las plataformas) · `puerta/socket.rs` y `bin/ghost.rs` (la puerta local es un socket Unix: **no en Windows** en el S6,
dicho por la propia puerta) · la hora local en `lib.rs`, `reunion.rs`, `escucha/mod.rs` y `vencimiento/` (un solo
`reloj.rs`, `libc` en Unix y `chrono` en Windows) · `vencimiento/` (la tarea que borra con la app cerrada es de launchd;
en Windows, el Programador de tareas desde el S8, ADR 016 enmienda 4) · ocho tests con permisos Unix (llevan
`cfg(unix)`) · `tests/puerta.rs` y `tests/ghost.rs` (Unix) · `build.rs` (el `check-cfg` del puente, para todas).

## Hoy en Windows (lo que enseñan Honestidad y el manual)

La misma lista vive en `src-tauri/src/plataforma.rs`, en Honestidad y en el manual; un test comprueba que dicen lo mismo.

| Clave                        | Hoy en Windows                                                    |
| ---------------------------- | ----------------------------------------------------------------- |
| `banda-protegida`            | sí, sin verificar en Teams, Zoom ni Meet hasta tu prueba en el PC |
| `presencial`                 | sí: la sala por el micrófono, con whisper                         |
| `corpus-y-ficha`             | sí                                                                |
| `notas-cifradas` | sí, se guardan cifradas con la llave en el Administrador de credenciales; exportarlas y abrir la bandeja piden Windows Hello |
| `videollamada` | sí: tu micrófono y el audio del sistema, dos pistas; sin verificar en Teams, Zoom ni Meet hasta tu prueba en el PC |
| `pantalla` | sí: la ventana de la reunión, leída solo cuando cambia, con el reconocimiento de texto de Windows |
| `voz-al-oido` | sí, con auriculares: la voz del sistema; por altavoces no habla |
| `acople` | sí: la ventana de la reunión baja y se encoge, y vuelve al cerrar; sin permiso que conceder |
| `ensayo` | sí: la voz del sistema lee las preguntas |
| `sugerencia` | sí, con el API y tu clave: en Windows no hay modelo del sistema |
| `radar` | sí: los programas de este PC que te miran, con su catálogo de Windows, y si está gestionado (MDM) |
| `atajos` | sí: Alt+Mayús y una tecla (tabla de la enmienda 1); ninguno con Ctrl+Alt, que en muchos teclados es AltGr |
| `borrado-con-la-app-cerrada` | sí: el Programador de tareas, al minuto de cada vencimiento |
| `desbloqueo` | sí: Windows Hello (PIN, huella o cara), una vez por sesión de la app |
| `puerta-local`               | no en Windows por ahora                                           |

## Consecuencias

- `contador-de-red` cuenta 3 puertas Swift y 1 Rust; `verify-ephemeral` protege `capture/windows.rs` y
  `stt/whisper.rs`; `cargo-test-sin-hardware` vigila las entradas nuevas (`hardware::vigilar`).
- La CI gana `build-windows` (`windows-latest`): tests sin hardware y el instalador como artifact. Whisper solo se
  compila ahí; la inferencia real y la latencia se leen en el PC.
- El instalador sale **sin firmar** hasta G-Release: Windows SmartScreen avisará; el manual dice cómo pasar.

## Enmienda al ADR 006 (2026-10-11)

El ADR 006 dejó whisper-rs como «el camino escrito» y sin construir. Desde el S6 es **el motor de Windows**. En macOS
nada cambia: SpeechAnalyzer, sin respaldo.

## Enmienda 1 · la videollamada (sprint 007, 2026-10-11)

El S7 trae a Windows lo que la videollamada necesita, cada pieza detrás de la frontera del §1. Cinco claves pasan a
«sí»; lo que queda (acople, sugerencia, radar, borrado con la app cerrada, desbloqueo) es del **S8 «Windows completo»**.

1. **El audio del sistema es el *loopback* de WASAPI** (`capture/windows.rs`, con `cpal`, sin dependencia nueva): la
   pista del cliente, `Pista::Sistema`. Dos diferencias con el tap del Mac, cada una con su respuesta:
   - **no entrega nada mientras nada suena**: el reloj de la pista lo rellena con silencio medido con el reloj de pared
     (`capture/reloj.rs`), para que el eco y el disparo comparen tiempos de un solo reloj;
   - **oye también la voz de la app** (el tap del Mac excluye su proceso; el loopback por proceso de Windows exige
     Windows 11 o build 20348+): un turno del cliente que cae mientras la app hablaba se marca como eco y no dispara
     (`escucha::atender`, `habla::la_app_hablo`). Cuando el mínimo sea Windows 11, el loopback por proceso lo sustituye;
   - **sigue a la salida** (auditoría del S7, A4): el flujo nace atado al dispositivo por defecto de ese momento. Si el
     usuario cambia de salida, `cpal` avisa (`StreamInvalidated` o `DeviceNotAvailable`) y el hilo del sistema lo vuelve a
     abrir sobre la nueva, sin parar el reloj; si no puede, reintenta cada segundo. Sin esto el flujo viejo callaba y el
     reloj lo rellenaba con silencio: la pista parecía sana mientras la app no oía al cliente.
2. **Por dónde sale el sonido** se lee de la forma que Windows declara del dispositivo de salida
   (`PKEY_AudioEndpoint_FormFactor`): altavoces y HDMI, la voz no habla; auriculares, sí. Es el mismo freno que en el Mac.
3. **La reunión** se detecta con los procesos y los títulos de las ventanas del propio PC, contra
   `data/clientes-windows.json`: ejecutable, fuente pública y fecha por fila. Solo se leen los títulos de los navegadores
   del catálogo. Windows no pide permiso para esto, así que «no se puede saber» no aplica. Teams arranca con Windows y
   queda abierto, así que **una aplicación de videollamada cuenta solo cuando ella o un proceso suyo tiene una sesión
   de captura de audio activa** (sprint 008, auditoría del S7, M1; ADR 005, enmienda 1). Meet, por el título.
4. **La voz** es `Windows.Media.SpeechSynthesis`: la frase en un WAV en memoria (`habla/pcm.rs`) que suena por `cpal`. Con
   ella, el ensayo con voz. Preguntar si hay voz para un idioma (`hay_para`) se hace en un hilo propio y corto: pasar por
   la cola del hilo que habla dejaba al que preguntaba esperando detrás de la frase (auditoría del S7, A3).
5. **La pantalla** es la ventana de la reunión por `PrintWindow` (`PW_RENDERFULLCONTENT`) y su texto por
   `Windows.Media.Ocr`, detrás de la misma huella por zonas y el mismo `Vigia` que en el Mac. Windows no da confianza por
   línea (se toma 1,0) y cada motor lee un idioma: el del perfil del usuario, o español o inglés. Sin el reconocimiento
   de texto de ningún idioma instalado, lo dice.
6. **La cola de whisper**: con dos pistas, el cliente va primero y, si esperan más de 4 turnos, se descarta lo tuyo más
   viejo, pisado con ceros y contado en Honestidad. En macOS no cambia nada: orden de llegada, sin tope.
7. **Los atajos**: misma acción, otra combinación (`atajos.rs`). En Windows `Ctrl+Alt` es AltGr en muchos teclados y
   `Alt+Esc` es del sistema, así que se usa **`Alt+Mayús` + letra**, la única familia de letras libre en las seis tablas
   publicadas que se leyeron el 2026-10-11. Un atajo que Windows no deja registrar se dice en Honestidad. El radar
   registra la suya desde el S8 (enmienda 2).

| Acción | macOS | Windows | Choques en las tablas publicadas |
| --- | --- | --- | --- |
| Cortar (kill-switch) | ⌥⎋ | Alt+Mayús+Q | ninguno; tampoco en Word ni Excel |
| Transcript | ⌃⌥T | Alt+Mayús+C | ninguno (Word la usa) |
| Ayúdame con esto | ⌃⌥A | Alt+Mayús+H | ninguno (Excel la usa) |
| Modo solo audio | ⌃⌥V | Alt+Mayús+W | ninguno; tampoco en Word ni Excel |
| Léela ahora | ⌃⌥L | Alt+Mayús+D | ninguno (Word la usa) |
| Qué ve (radar) | ⌃⌥R | Alt+Mayús+U (desde el S8) | ninguno |
| Anotar | ⌃⌥N | Alt+Mayús+K | ninguno (Word la usa) |
| Fijar | ⌃⌥P | Alt+Mayús+F | ninguno (Word la usa) |
| Guardar la propuesta | ⌃⌥↵ | Alt+Mayús+Z | ninguno; tampoco en Word ni Excel |
| La banda, al otro borde | ⌃⌥B | Alt+Mayús+X | ninguno (Word la usa) |
| Callar la voz (solo con el modo encendido) | ⎋ | Esc | Esc no es del sistema; mientras el modo está encendido no le llega a la reunión, como en el Mac |

Las seis tablas, leídas el 2026-10-11: Windows 11
(support.microsoft.com/windows/keyboard-shortcuts-in-windows-dcc61a57-8ff0-cffe-9796-cb9706c75eec), Microsoft Teams
(support.microsoft.com/office/keyboard-shortcuts-for-microsoft-teams-2e8e2a70-e8d8-4a19-949b-4c36dd5292d2), Zoom
Workplace (support.zoom.com/hc/en/article?id=zm_kb&sysparm_article=KB0067050), Chrome
(support.google.com/chrome/answer/157179), Edge
(support.microsoft.com/microsoft-edge/keyboard-shortcuts-in-microsoft-edge-50d3edab-30d9-c7e4-21ce-37fe2713cfad) y Google
Meet (support.google.com/a/users/answer/9896256). La tecla Windows y F12 están reservadas
(learn.microsoft.com/windows/win32/api/winuser/nf-winuser-registerhotkey); `Ctrl+Alt` como AltGr lo dice
learn.microsoft.com/windows/win32/uxguide/inter-keyboard. **Sin verificar, para el PC:** que `Alt+Mayús` no cambie la
distribución del teclado al pulsar la letra en un PC con dos distribuciones, y que ninguna app del usuario fuera de esas
seis tablas use estas combinaciones; Word y Excel usan algunas, y mientras la app está abierta no les llegan.

## Enmienda 2 · Windows completo (sprint 008, 2026-10-12)

Las cinco funciones que el S7 dejó en «todavía no» están, cada una detrás de la misma frontera y con la lógica pura de
macOS intacta. «Hoy en Windows» queda en **14 «sí» y 1 «no por ahora»** (la puerta local), y `Cuando::TodaviaNo`
desaparece de `plataforma.rs`: no queda ninguna.

1. **La reunión cuenta cuando hay llamada** (ADR 005, enmienda 1): Teams o Zoom, solo con una sesión de captura activa
   en su proceso o en uno hijo (`IAudioSessionManager2`). Sesión, la pantalla, el acople y el ensayo ven la misma.
2. **El acople** (`acople/windows.rs`) da las mismas funciones que `ax.rs` y la maniobra de `acople/mod.rs` es la del
   Mac: el turno, soltar primero, leer lo que de verdad quedó, deshacer y la huella. Se escribe con `SetWindowPos` (sin
   activar ni cambiar el orden) y se mide lo que se ve (`DWMWA_EXTENDED_FRAME_BOUNDS`, sin los bordes invisibles de
   redimensionar), en puntos con la escala del monitor de cada ventana (`GetDpiForMonitor`), que es como Tauri da la
   franja. El índice de una ventana es su posición entre las de su proceso **ordenadas por identificador**, estable
   mientras vive. Windows no pide permiso para esto, y la app lo dice. Sin verificar, para el PC: una ventana
   maximizada se acopla sin restaurarse, y Teams o Zoom pueden pelear su tamaño (si lo hacen, se deshace y la banda
   flota, como en el Mac).
3. **El desbloqueo es Windows Hello** (`desbloqueo/windows.rs`): `CheckAvailabilityAsync` y
   `RequestVerificationForWindowAsync` con la ventana principal como dueña, en un hilo propio con COM. Verificado abre;
   cancelar o agotar los intentos es tu «no»; sin Windows Hello configurado se dice dónde se configura.
4. **El borrado con la app cerrada** es el Programador de tareas (ADR 016, enmienda 4).
5. **El radar** mira la foto de procesos de este PC contra `data/radar/programas-windows.json` (el ejecutable `.exe`,
   la fuente y la fecha por fila) y pregunta a `IsDeviceRegisteredWithManagement` si está gestionado, sin leer la cuenta.
   Fuentes: el fabricante cuando nombra el ejecutable; si no, la lista de Safe Exam Browser para Windows o LOLRMM,
   fijadas a un commit, con su nota cuando la fuente es débil. Fuera, sin fuente pública: Inspera, Hubstaff y DeskTime.
   `radar-solo-este-mac` deja pasar esas dos lecturas y ninguna otra de Windows.
6. **La sugerencia** sale por la segunda puerta Rust (ADR 023). El modelo del sistema sigue sin existir en Windows: la
   sugerencia es «sí, con el API y tu clave», y sin API la ficha sigue siendo la respuesta.

## Lo que este ADR NO decide

La firma del instalador (G-Release), una DACL propia para la carpeta (todavía no), ni
si la puerta local llega a Windows (otro ADR, si llega).
