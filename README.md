# Angel Ghost

*Tu propia experiencia, en la reunión, en el momento justo — y solo tú la ves.*

Angel Ghost es una app de escritorio para macOS pensada para consultores. Mientras estás en una
videollamada, escucha las dos partes (tu micrófono y el audio del cliente, cada uno por su lado),
lee la pantalla compartida cuando cambia y pone delante de ti la **evidencia de tu propio corpus**:
un titular, una línea y el documento y la sección de donde sale. La banda donde aparece **no se ve
cuando compartes pantalla en Google Meet** (verificado sobre macOS 26.6.2; en Zoom y Teams no está
comprobado, y la app lo dice en vez de prometerlo).

**No persistir, y que se pueda comprobar.** El audio, lo que dijo el cliente y lo que se leyó de la
pantalla viven solo en memoria y mueren al cerrar. Lo único que queda es lo tuyo: tus notas, tus
acuerdos y las fichas que fijaste, en un archivo cifrado por reunión con la llave en tu Llavero; y, si lo
eliges, un hecho de una línea que dijo el cliente, en la bandeja, cifrado, que se borra solo en la ventana
que elijas (24 h como mucho). Por
defecto la app funciona entera en tu Mac: el contador de red marca 0 bytes salvo que enciendas, con tu
propia clave, un proveedor externo para las sugerencias.

## Estado

**MVP personal (ciclo H1) construido.** Lo sella el gate humano del cierre de ciclo. Hoy la app se
compila desde este repo; la versión firmada para instalar llega en el H2, con la firma de Apple.

## Arrancarla

Necesitas **macOS 26 o posterior**, [pnpm](https://pnpm.io/) y Rust.

```
pnpm install
pnpm tauri dev
```

`pnpm ghost` compila además la puerta local, para que Claude Code opere la app desde el mismo Mac.

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
your screen in Google Meet** (verified on macOS 26.6.2; Zoom and Teams are not verified yet, and the
app says so instead of promising it).

Audio, the client's words and whatever was read from the screen live only in memory and die when
you close. What stays is yours: your notes, agreements and pinned cards, in one encrypted file per
meeting, keyed from your Keychain; and, if you choose, a one-line fact the client said, in the tray,
encrypted, which deletes itself within the window you pick (24 h at most). By default everything runs on your Mac: the network counter reads
0 bytes unless you turn on, with your own key, an external provider for suggestions.

**Status:** personal MVP (cycle H1) built; it is sealed by the cycle-closing human gate. Today it is
built from this repo (macOS 26+, `pnpm install && pnpm tauri dev`); a signed build comes in H2. The
app's interface is in Spanish and English; the documents in this repo (manual, test guide, decisions)
are in Spanish.
