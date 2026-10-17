# Angel Ghost

*Tu propia experiencia, en la reunión, en el momento justo — y solo tú la ves.*

Angel Ghost es una app de escritorio para macOS y Windows pensada para consultores. Mientras estás en una
videollamada, escucha las dos partes (tu micrófono y el audio del cliente, cada uno por su lado),
lee la pantalla compartida cuando cambia y pone delante de ti la **evidencia de tu propio corpus**:
un titular, una línea y el documento y la sección de donde sale. La banda donde aparece **no se ve
cuando compartes pantalla en Google Meet** (verificado sobre macOS 26.6.2 con la banda abajo; arriba,
su sitio de fábrica desde el sprint 004, usa la misma protección y está sin verificar; en Zoom y Teams
no está comprobado, y la app lo dice en vez de prometerlo). Y antes de la reunión puedes **ensayarla**:
la app te hace las preguntas que ese cliente probablemente hará, sacadas de tu propia propuesta, y te
dice qué evidencia tuya usaste.

**No persistir, y que se pueda comprobar.** El audio, lo que dijo el cliente y lo que se leyó de la
pantalla viven solo en memoria y mueren al cerrar. Lo único que queda es lo tuyo: tus notas, tus
acuerdos, las fichas que fijaste, las propuestas que aceptes —de lo que dijo el cliente, como mucho un
hecho de una línea— y, si lo enciendes, lo que dijiste tú en texto, en un archivo cifrado por reunión
con la llave en tu Llavero; y tus ensayos, si los guardas: tus respuestas en texto y sus cifras, con la
misma llave. Lo que no decides espera cifrado en la bandeja y se borra solo al vencer (3 h de fábrica,
24 h como mucho; con «al cerrar», ni eso). Por defecto la app funciona entera en tu Mac: el contador de
red marca 0 bytes salvo que enciendas, con tu propia clave, un proveedor externo para las sugerencias o
para «Enriquecer el banco» del ensayo.

## Estado

**Hito H-1 «Vitrina funcional» construido.** Lo sella el gate humano del hito (≈ 20 min en el Mac y ≈ 20 min en
el PC con Windows). En el camino: el sprint 004 sumó el ensayo y la banda arriba, junto a la cámara; el 005, el modo
presencial (la sala por el micrófono del Mac, sin audio del sistema ni pantalla); y los sprints 006, 007 y 008, la app
en Windows. Hoy se compila desde este repo; la versión firmada para instalar llega más adelante.

La app también se compila para **Windows** (ADR 022): desde el sprint 006 la banda protegida, el micrófono, la
transcripción con whisper y tus notas cifradas; desde el 007, la **videollamada** en ese mismo PC (el audio del cliente
por el audio del sistema, la ficha al oído, la pantalla de la reunión y los atajos `Alt+Mayús`); y desde el 008,
el acople, la sugerencia con el API, el radar, Windows Hello y el borrado con la app cerrada. Honestidad dice qué hace
hoy y qué no (de las quince funciones, solo la puerta local queda fuera), y la protección sigue «sin verificar» en
Teams, Zoom y Meet hasta que la pruebes en tu PC. El instalador sale sin firmar de la integración continua hasta
G-Release. Desde el sprint 009 la app es además **nativa en ARM64** (una Surface con Snapdragon, por ejemplo): se
compila y se instala sin emulación, y «Instalar» baja el modelo de voz que cupo en 4 s en un equipo así (ver
«Desarrollar en Windows ARM»).

## Arrancarla

**En macOS** necesitas **macOS 26 o posterior**, [pnpm](https://pnpm.io/) y Rust.

```
pnpm install
pnpm tauri dev
```

**En Windows** necesitas **Windows 10 versión 2004 o posterior** (la primera que oculta una ventana de la captura),
pnpm, Rust con la cadena MSVC (las «Build Tools» de Visual Studio con C++) y CMake, que whisper.cpp usa al
compilarse. Los mismos dos comandos arrancan la app; el modelo de voz (1,6 GB) se baja desde Idioma, cuando lo pidas.
Para el instalador sin firmar: `pnpm tauri build --bundles msi,nsis`.

`pnpm ghost` compila además la puerta local, para que Claude Code opere la app desde el mismo Mac.

## Desarrollar en Windows ARM

Desde el sprint 009 la app se compila, se prueba y se instala **nativa en ARM64**, sin emulación (ADR 022, enmienda 3).
Se probó en una Surface con Snapdragon X Plus y Windows 11 en inglés. Cada instalación toca tu equipo, así que se hace
**una por una**, viendo antes qué hace (regla 24).

1. **Mira qué falta**, sin instalar nada: `powershell -NoProfile -File scripts/entorno-windows.ps1`. Imprime «tengo ·
   necesito» por pieza y sale con 1 si falta alguna. Git Bash sale como «aviso»: es x64 emulado, y es lo normal.
2. **Instala lo que falte**, una pieza por vez, con `winget`:

| Pieza | Para qué | Comando |
|---|---|---|
| jq | el hook de secretos de Claude Code; sin él no puede escribir | `winget install --id jqlang.jq --exact` |
| Build Tools 2022 con C++ ARM64 | el compilador y el enlazador nativos | `winget install --id Microsoft.VisualStudio.2022.BuildTools --exact --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --add Microsoft.VisualStudio.Component.VC.Tools.ARM64 --add Microsoft.VisualStudio.Component.VC.Tools.x86.x64 --add Microsoft.VisualStudio.Component.Windows11SDK.26100"` |
| Rustup (host `aarch64-pc-windows-msvc`) | Rust | `winget install --id Rustlang.Rustup --exact` |
| CMake | whisper.cpp | `winget install --id Kitware.CMake --exact` |
| LLVM (ARM64) | `libclang` para `bindgen` y `clang` para `ring` | `winget install --id LLVM.LLVM --exact` |

   Después, para tu usuario: la variable `LIBCLANG_PATH` con el valor `C:\Program Files\LLVM\bin`, y esa misma carpeta
   al final del `PATH`. Abre una terminal nueva y repite el paso 1: tiene que salir todo en `[ok]`.
3. **Compila y prueba**:

```
pnpm install
cd src-tauri
AG_SIN_HARDWARE=1 cargo test --locked -j 4
cd ..
pnpm tauri build --bundles nsis
```

El `-j 4` no es manía: enlazar con todos los núcleos se queda sin memoria en un equipo de 16 GB. `AG_SIN_HARDWARE=1`
impide que `cargo test` abra el micrófono o las credenciales (regla 25). `--bundles msi` saca el `.msi`. whisper.cpp se
compila con clang-cl, porque ggml no admite MSVC en ARM (ADR 022, enmienda 3).

## Dónde está cada cosa

| Qué | Dónde |
|---|---|
| Cómo se usa, función por función | `docs/MANUAL-DE-USO.md` |
| Qué probar y qué esperar | `docs/GUIA-DE-PRUEBA.html` (doble clic) |
| Toda la infraestructura, tal como está construida | `docs/BLUEPRINT.html` |
| El diseño aprobado y el design system | `docs/diseno/` · `design-system.md` |
| Las decisiones de arquitectura | `decisions/` |
| El kit de prueba (datos 100 % sintéticos) | `docs/kit-de-prueba/` |
| Cómo se construyó, sprint a sprint | `sprints/` |

**Ningún dato real de clientes vive en este repo:** el corpus, las notas y las preferencias del
usuario están en su Mac, fuera del repo.

---

## In English

Angel Ghost is a macOS desktop app for consultants. During a video call it listens to both sides
(your microphone and the client's audio, each on its own track), reads the shared screen when it
changes, and puts **evidence from your own documents** in front of you: a headline, a line and the
exact document and section it comes from. The band where it appears **is not visible when you share
your screen in Google Meet** (verified on macOS 26.6.2 with the band at the bottom; at the top, its
default place since sprint 004, it uses the same protection and is unverified; Zoom and Teams are not
verified, and the app says so instead of promising it). And before the meeting you can **rehearse
it**: the app asks you the questions that client will probably ask, drawn from your own proposal, and
tells you which of your evidence you used.

Audio, the client's words and whatever was read from the screen live only in memory and die when
you close. What stays is yours: your notes, agreements, pinned cards, the suggestions you accept —of
what the client said, at most a one-line fact— and, if you turn it on, what you said yourself, as
text, in one encrypted file per meeting, keyed from your Keychain; and your rehearsals, if you save
them: your answers as text and their figures, with the same key. What you leave undecided waits,
encrypted, in the tray and deletes itself on expiry (3 h by default, 24 h at most; with "on close",
not even that). By default everything runs on your Mac: the network counter reads 0 bytes unless you
turn on, with your own key, an external provider for suggestions or for the rehearsal's "Enrich the
bank".

**Status:** milestone H-1 "Functional showcase" built; it is sealed by the milestone's human gate (about 20 min on
the Mac and 20 min on the Windows PC). Along the way: sprint 004 added the rehearsal and the band at the top, next to
the camera; sprint 005, the in-person mode (the room through the Mac's microphone, with no system audio and no screen);
and sprints 006 to 008, the app on Windows (ADR 022: protected band, microphone, whisper transcription, encrypted notes,
the video call on that same PC, docking, the suggestion through the API, the radar, Windows Hello and deletion with the
app closed; the unsigned installer comes from CI). Today it is built from this repo (macOS 26+ or Windows 10 2004+,
`pnpm install && pnpm tauri dev`); a signed build comes later. The app's interface is in Spanish and English; the
documents in this repo (manual, test guide, decisions) are in Spanish, and the manual also has an English edition.

**Windows on ARM (sprint 009).** The app also builds, tests and installs natively on ARM64 (a Surface with a Snapdragon,
for example), with no emulation (ADR 022, amendment 3). `powershell -NoProfile -File scripts/entorno-windows.ps1` only
reads: it lists what is missing, and each installation —`jq`, the Visual Studio 2022 Build Tools with C++ ARM64, Rustup,
CMake and LLVM, all through `winget`— is done one at a time, after seeing what it does. Then set `LIBCLANG_PATH` to the
LLVM `bin` folder, add that folder to your `PATH`, and run `AG_SIN_HARDWARE=1 cargo test --locked -j 4` in `src-tauri`
and `pnpm tauri build --bundles nsis`. On an ARM64 PC, the model that Install downloads is the largest one that fits in
4 s: the small one, on the reference Surface.
