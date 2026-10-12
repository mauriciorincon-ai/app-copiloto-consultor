# Auditoría del sprint 006 «Windows arranca» (una pasada)

Auditor independiente, solo lectura (subagente Sonnet 5.5, high), sobre el diff `origin/main...HEAD` de PR #20.
Primer veredicto: **requiere ajustes** (0 críticos, 5 altos). Los altos se pagan en el sprint; los medios y bajos
van al lote del hito, cada uno con su `archivo:línea`.

## Altos — pagados en el sprint

| Id  | Sitio                                                                                                          | Hallazgo                                                                          | Estado                                                                                                          |
| --- | -------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------- |
| A1  | `src-tauri/src/lib.rs` (`registrar_el_kill_switch`, `con_el_callar`)                                           | En Windows se registraban atajos globales (y Escape) pese a «atajos: todavía no». | pagado · 553b68e (guarda con `return`) · 09ec07b (el gate exige el `return`)                                    |
| A2  | `src-tauri/src/stt/whisper.rs` (`CARGADO`, `transcribir`); `src-tauri/src/lib.rs` (corte, `dejar_de_escuchar`) | El texto de la última transcripción sobrevivía al corte y a «Terminar».           | pagado · 553b68e (`soltar()` en el corte y al terminar; copias del audio a cero). Lo verifica la CI de Windows. |
| A3  | `src-tauri/src/modelo_whisper.rs` (`bajar`)                                                                    | La descarga no tenía ningún test; la guía y la CI afirmaban lo contrario.         | pagado · 553b68e (`descargar_a`, 4 pruebas en el Mac; guía y comentario de CI corregidos)                       |
| A4  | `src/pantallas/Idioma.tsx`, `src/pantallas/Honestidad.tsx`                                                     | Texto de macOS en Windows (motor, descarga, conexión, icono).                     | pagado · 553b68e (textos de Windows; e2e sin «macOS» ni «Mac» en las dos pantallas, con control en macOS)       |
| A6  | `src/pantallas/Notas.tsx`, `src/i18n/windows.ts`                                                               | «Se borra aunque no abras la app» falso en Windows.                               | pagado · 553b68e (variantes de Windows; e2e con control en macOS)                                               |

## Medios — lote del hito (sin pagar en el sprint)

| Id  | Sitio                                                                 | Hallazgo                                                                      | Estado                    |
| --- | --------------------------------------------------------------------- | ----------------------------------------------------------------------------- | ------------------------- |
| M1  | `src-tauri/src/modelo_whisper.rs` (`bajar`)                           | Sin timeout: una conexión colgada deja `BAJANDO` activo hasta reiniciar.      | lote del hito             |
| M2  | `src-tauri/src/lib.rs` (`empezar`, `instalar_idioma`)                 | La guarda «nunca en reunión» solo mira al empezar la descarga.                | lote del hito             |
| M3  | `src-tauri/src/stt/whisper.rs` (latencia en memoria); `lib.rs`        | El escalón al modelo pequeño no sobrevive a reabrir la app.                   | lote del hito             |
| M4  | `src-tauri/src/lib.rs` (`hay_una_reunion`)                            | Solo mira `LaEscucha`; no mira ensayo ni «solo notas».                        | lote del hito             |
| M5  | `src/pantallas/Sesion.tsx:446-448`; `capture/windows.rs:46-48`        | «Al oído» elegido por defecto en Windows, con la voz muda.                    | lote del hito             |
| M6  | `src-tauri/src/ventana/mod.rs:107-124`; manual; README                | Falta el mínimo de Windows (10 2004) y la comprobación de versión.            | lote del hito             |
| M7  | `src/pantallas/Permisos.tsx:104`; `src/i18n/es.ts`                    | El título y los «para» de Permisos siguen diciendo macOS en Windows.          | lote del hito             |
| M8  | `src/i18n/es.ts` (varias claves de Sesión, Notas, IA)                 | Más texto de macOS visible en Windows.                                        | lote del hito             |
| M9  | `src-tauri/src/desbloqueo.rs:85-88`                                   | Jerga «sprint 007» en un error que llega a la interfaz.                       | lote del hito             |
| M10 | `src-tauri/src/lib.rs` (instalar); `src/pantallas/Idioma.tsx:113-124` | «Instalar» falla sin un motivo escrito.                                       | lote del hito             |
| M11 | `src-tauri/tests/contra-el-mac-de-verdad.rs:28`                       | La mitad en marcha de la regla 1 solo corre en macOS. Declarado en el cierre. | lote del hito (declarado) |
| M12 | `src-tauri/src/llavero/windows.rs` (test `hardware:`)                 | El test llama a `windows::*` sin `vigilar`.                                   | lote del hito             |

## Bajos — lote del hito

| Id  | Sitio                                                        | Hallazgo                                                                                                                                                                 |
| --- | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| B1  | `src-tauri/src/ventana/mod.rs:394-408`                       | El comentario doc se partió al insertar el test.                                                                                                                         |
| B2  | `docs/GUIA-DE-PRUEBA.html:21-22`                             | El reemplazo global de «nuevo» dañó una instrucción.                                                                                                                     |
| B3  | `README.md`; `docs/MANUAL-DE-USO.md`                         | Textos que dicen solo macOS; falta cómo compilar en Windows.                                                                                                             |
| B4  | `decisions/006-stt-local-y-su-modelo.md:79-80`               | Sigue diciendo que whisper es «el camino escrito».                                                                                                                       |
| B5  | `src-tauri/src/modelo_whisper.rs`                            | Un corte dejaba un `.parcial` en disco. **Pagado en el sprint** dentro de `descargar_a` (ver A3); queda sin cerrar la limpieza al arrancar y la comprobación de espacio. |
| B6  | `src/componentes/HoyEnWindows.tsx`                           | «Bajando…» no se refresca si la descarga empieza desde Idioma.                                                                                                           |
| B7  | `src/componentes/HoyEnWindows.tsx`                           | El botón del modelo pequeño sale aunque ya esté bajado.                                                                                                                  |
| B8  | `tests/unit/windows-honestidad-y-manual.test.ts`             | `estadoDelDetalle` devuelve «todavía no» por defecto.                                                                                                                    |
| B9  | `src-tauri/src/stt/calibracion.rs`                           | El WAV de 320 KB entra también en el binario de macOS.                                                                                                                   |
| B10 | `decisions/022-windows-detras-de-la-misma-frontera.md:9-10`  | Cita «la invisibilidad está garantizada», contra la regla 6.                                                                                                             |
| B11 | `src-tauri/Cargo.toml`; `tests/unit/contador-de-red.test.ts` | El gate no mira las features de `windows-sys`.                                                                                                                           |
| B12 | Guía (paso w3); manual                                       | «Honestidad enseña la medición» solo es cierto tras medir; el botón de ajustes solo sale sin micrófono concedido.                                                        |

## Casillas

1. **Alcance** — requiere ajustes → corregido en el sprint (A1); lo demás, lote.
2. **Calidad** — la lógica nativa es sólida (hilo propio del flujo, sin `unwrap` en caminos de usuario, llamadas `unsafe` comentadas). Los altos de calidad están pagados.
3. **Seguridad y privacidad** — sin secretos, sin enlaces de producción, sin datos reales. A2 y A6 pagados; M2 y M11 al lote.
4. **Frases caducadas** — A1, A4 y A6 pagados; M6 a M9 y B3 al lote.
