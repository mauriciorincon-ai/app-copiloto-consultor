# Auditoría del sprint 006 «Windows arranca» (una pasada)

Auditor independiente, solo lectura (subagente Sonnet 5.5, high), sobre el diff de PR #20. Primer veredicto:
**requiere ajustes** (0 críticos, 5 altos). El informe del auditor numeró el quinto alto como A6 (no hay A5 en su informe); aquí va como A5. Los altos se pagan en el sprint; los medios y bajos van al lote del hito,
cada uno con su `archivo:línea`.

## CRÍTICOS (0)

Ninguno.

## ALTOS (5)

| Id | Sitio | Hallazgo | Estado |
|---|---|---|---|
| **A1** | `src-tauri/src/lib.rs:3828` | Se registraban atajos globales (y Escape) en Windows, pese a «atajos: todavía no». | pagado · 553b68e · 09ec07b (el gate exige el `return`) |
| **A2** | `src-tauri/src/stt/whisper.rs:161` | El texto de la última transcripción sobrevivía al corte y a «Terminar». | pagado · 553b68e (`soltar()` en el corte y al terminar; copias del audio a cero; lo verifica la CI de Windows) |
| **A3** | `src-tauri/src/modelo_whisper.rs:92` | La descarga del modelo no tenía ningún test; la guía y la CI afirmaban lo contrario. | pagado · 553b68e (`descargar_a`, 4 pruebas en el Mac) |
| **A4** | `src/pantallas/Idioma.tsx:254` | Texto de macOS en Idioma y Honestidad en Windows (motor, descarga, conexión, icono). | pagado · 553b68e (textos de Windows; e2e sin «macOS» ni «Mac», con control en macOS) |
| **A5** | `src/pantallas/Notas.tsx:573` | «Se borra aunque no abras la app» es falso en Windows. | pagado · 553b68e (variantes de Windows; e2e con control en macOS) |

## MEDIOS (12)

| Id | Sitio | Hallazgo | Estado |
|---|---|---|---|
| **M1** | `src-tauri/src/modelo_whisper.rs:109` | Sin timeout en la descarga: una conexión colgada deja `BAJANDO` activo hasta reiniciar. | deuda · lote del hito |
| **M2** | `src-tauri/src/lib.rs:1056` | La guarda «nunca en reunión» solo se mira al empezar la descarga. | deuda · lote del hito |
| **M3** | `src-tauri/src/stt/whisper.rs:38` | La latencia y el escalón al modelo pequeño no sobreviven a reabrir la app. | deuda · lote del hito |
| **M4** | `src-tauri/src/lib.rs:1073` | `hay_una_reunion` mira solo la escucha; no mira el ensayo ni «solo notas». | deuda · lote del hito |
| **M5** | `src/pantallas/Sesion.tsx:446` | «Al oído» queda elegido por defecto en Windows, con la voz muda. | deuda · lote del hito |
| **M6** | `src-tauri/src/ventana/mod.rs:107` | Falta el mínimo de Windows (10 2004) y su comprobación de versión. | deuda · lote del hito |
| **M7** | `src/pantallas/Permisos.tsx:104` | El título y los «para» de Permisos siguen diciendo macOS en Windows. | deuda · lote del hito |
| **M8** | `src/i18n/es.ts:331` | Más texto de macOS visible en Windows (Sesión, Notas, IA). | deuda · lote del hito |
| **M9** | `src-tauri/src/desbloqueo.rs:85` | Jerga «sprint 007» en un error que llega a la interfaz. | deuda · lote del hito |
| **M10** | `src/pantallas/Idioma.tsx:113` | «Instalar» que falla vuelve al botón sin un motivo escrito. | deuda · lote del hito |
| **M11** | `src-tauri/tests/contra-el-mac-de-verdad.rs:28` | La mitad en marcha de la regla 1 solo corre en macOS. Declarada en el cierre. | deuda · lote del hito |
| **M12** | `src-tauri/src/llavero/windows.rs:131` | El test «de hardware» salta el centinela `vigilar` de `llavero.rs`. | deuda · lote del hito |

## BAJOS (12)

| Id | Sitio | Hallazgo | Estado |
|---|---|---|---|
| **B1** | `src-tauri/src/ventana/mod.rs:394` | El comentario doc se partió al insertar el test. | deuda · lote del hito |
| **B2** | `docs/GUIA-DE-PRUEBA.html:21` | El reemplazo global de «nuevo» dañó una instrucción de la guía. | deuda · lote del hito |
| **B3** | `README.md:5` | Textos que dicen solo macOS; falta cómo compilar en Windows. | deuda · lote del hito |
| **B4** | `decisions/006-stt-local-y-su-modelo.md:79` | Sigue diciendo que whisper es «el camino escrito». | deuda · lote del hito |
| **B5** | `src-tauri/src/modelo_whisper.rs:116` | Un corte dejaba un `.parcial` en disco. | pagado · 553b68e (se borra en cada error); deuda: limpieza al arrancar y comprobación de espacio |
| **B6** | `src/componentes/HoyEnWindows.tsx:37` | «Bajando…» no se refresca si la descarga empieza desde Idioma. | deuda · lote del hito |
| **B7** | `src/componentes/HoyEnWindows.tsx:156` | El botón del modelo pequeño sale aunque ya esté bajado. | deuda · lote del hito |
| **B8** | `tests/unit/windows-honestidad-y-manual.test.ts:43` | `estadoDelDetalle` devuelve «todavía no» por defecto. | deuda · lote del hito |
| **B9** | `src-tauri/src/stt/calibracion.rs:9` | El WAV de 320 KB entra también en el binario de macOS. | deuda · lote del hito (medir el peso en el PR) |
| **B10** | `decisions/022-windows-detras-de-la-misma-frontera.md:9` | Cita «la invisibilidad está garantizada por el sistema», contra la regla 6. | deuda · lote del hito |
| **B11** | `src-tauri/Cargo.toml:96` | El gate no mira las features de `windows-sys`. | deuda · lote del hito |
| **B12** | `docs/GUIA-DE-PRUEBA.html:873` | «Honestidad enseña la medición» solo es cierto tras medir; el botón de ajustes solo sale sin micrófono concedido. | deuda · lote del hito |

## Casillas

1. **Alcance:** requiere ajustes → corregido en el sprint (A1); el resto, lote.
2. **Calidad:** la lógica nativa es sólida; los altos de calidad están pagados.
3. **Seguridad y privacidad:** sin secretos, sin enlaces de producción, sin datos reales. A2 y A5 pagados; M2 y M11 al lote.
4. **Frases caducadas:** A1, A4 y A5 pagados; M6 a M9 y B3 al lote.
