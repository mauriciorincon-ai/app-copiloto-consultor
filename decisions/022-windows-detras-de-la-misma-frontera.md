# ADR 022 — Windows detrás de la misma frontera

- **Fecha:** 2026-10-11
- **Sprint:** 006 «Windows arranca» (fase 2, antes del código)
- **Estado:** aceptada. Enmienda el ADR 006 (§ Enmienda) y declara la primera puerta a la red escrita en Rust (§4).

## Contexto

Angel Ghost existe solo en macOS. La VISION promete Windows en el H-2 («la misma app donde la invisibilidad está
garantizada por el sistema»), y el plan lo parte en dos: el **S6 arranca** (un instalador que sale de la CI, la banda
protegida, la sala por el micrófono con whisper local y su latencia medida) y el **S7 trae la videollamada** (el audio
del sistema, la pantalla, la voz al oído, el acople).

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
| `desbloqueo.rs`             | Touch ID o contraseña para abrir notas               | **S7**: Windows Hello                                                                                   |
| `llavero.rs`                | el Llavero                                           | **S6**: Administrador de credenciales (§6)                                                              |
| `sintesis/api.rs`           | el POST al proveedor externo por `Red.swift`         | **S7**: necesita su propia puerta Rust y su ADR                                                         |
| `sintesis/sistema.rs`       | Apple Foundation Models                              | **no en Windows** por ahora: sin modelo del sistema; la ficha sigue siendo la respuesta                 |
| `capture/mod.rs`            | micrófono y audio del sistema (CoreAudio)            | **S6** el micrófono (`capture/windows.rs`, cpal/WASAPI) · **S7** el audio del sistema (WASAPI loopback) |
| `sesion/mod.rs`             | qué reunión hay y su ventana                         | **S7**                                                                                                  |
| `acople/mod.rs`             | hacerle sitio a la banda en la reunión               | **S7**                                                                                                  |
| `pantalla/apple.rs`         | leer la pantalla (ScreenCaptureKit + Vision)         | **S7** (Windows.Media.Ocr)                                                                              |
| `habla/apple.rs`            | la voz al oído (AVSpeechSynthesizer)                 | **S7**                                                                                                  |
| `radar/procesos.rs`         | los programas que vigilan tu Mac                     | **S7**                                                                                                  |
| `radar/mdm.rs`              | el perfil de gestión del Mac                         | **S7**                                                                                                  |
| `stt/apple.rs`              | SpeechAnalyzer                                       | **S6**: whisper (`stt/whisper.rs`, §5)                                                                  |
| `nativo/Transcriptor.swift` | SpeechAnalyzer                                       | sustituido por whisper (S6)                                                                             |
| `nativo/Habla.swift`        | la voz                                               | S7                                                                                                      |
| `nativo/Pantalla.swift`     | la pantalla                                          | S7                                                                                                      |
| `nativo/Sintesis.swift`     | el modelo del sistema                                | no en Windows                                                                                           |
| `nativo/Llavero.swift`      | el Llavero                                           | sustituido por el Administrador de credenciales (S6)                                                    |
| `nativo/Red.swift`          | la puerta del API externo                            | S7                                                                                                      |
| `nativo/Desbloqueo.swift`   | Touch ID                                             | S7 (Windows Hello)                                                                                      |
| `nativo/Copias.swift`       | Time Machine                                         | no en Windows                                                                                           |

**Y lo que impedía compilar** (no estaba en el inventario porque no llevaba `cfg`): `relleno.rs` (`base64` pasa a todas
las plataformas) · `puerta/socket.rs` y `bin/ghost.rs` (la puerta local es un socket Unix: **no en Windows** en el S6,
dicho por la propia puerta) · la hora local en `lib.rs`, `reunion.rs`, `escucha/mod.rs` y `vencimiento/` (un solo
`reloj.rs`, `libc` en Unix y `chrono` en Windows) · `vencimiento/` (la tarea que borra con la app cerrada es de launchd:
**S7**, con el Programador de tareas; mientras, se barre con la app abierta) · ocho tests con permisos Unix (llevan
`cfg(unix)`) · `tests/puerta.rs` y `tests/ghost.rs` (Unix) · `build.rs` (el `check-cfg` del puente, para todas).

## Hoy en Windows (lo que enseñan Honestidad y el manual)

La misma lista vive en `src-tauri/src/plataforma.rs`, en Honestidad y en el manual; un test comprueba que dicen lo mismo.

| Clave                        | Hoy en Windows                                                    |
| ---------------------------- | ----------------------------------------------------------------- |
| `banda-protegida`            | sí, sin verificar en Teams, Zoom ni Meet hasta tu prueba en el PC |
| `presencial`                 | sí: la sala por el micrófono, con whisper                         |
| `corpus-y-ficha`             | sí                                                                |
| `notas-cifradas`             | sí, con la llave en el Administrador de credenciales              |
| `videollamada`               | todavía no (S7): el audio del sistema                             |
| `pantalla`                   | todavía no (S7)                                                   |
| `voz-al-oido`                | todavía no (S7)                                                   |
| `acople`                     | todavía no (S7)                                                   |
| `ensayo`                     | todavía no (S7): la voz que lee las preguntas                     |
| `sugerencia`                 | todavía no (S7)                                                   |
| `radar`                      | todavía no (S7)                                                   |
| `atajos`                     | todavía no (S7): en muchos teclados Ctrl+Alt es AltGr             |
| `borrado-con-la-app-cerrada` | todavía no (S7): se borra con la app abierta                      |
| `desbloqueo`                 | todavía no (S7): Windows Hello                                    |
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

## Lo que este ADR NO decide

La videollamada en Windows (S7), la firma del instalador (G-Release), una DACL propia para la carpeta (todavía no), ni
si la puerta local llega a Windows (otro ADR, si llega).
