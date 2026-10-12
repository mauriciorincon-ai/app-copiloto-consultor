# Auditoría del sprint 006 «Windows arranca» (una pasada)

Auditor independiente, solo lectura (subagente Sonnet 5.5, high), sobre el diff de PR #20. Primer veredicto:
**requiere ajustes** (0 críticos, 5 altos). El informe del auditor numeró el quinto alto como A6 (no hay A5 en su informe); aquí va como A5. Los altos se pagan en el sprint; los medios y bajos van al lote del hito,
cada uno con su `archivo:línea`. **El lote del hito se pagó en el sprint 008 (PR #24): cada fila lleva su estado.**

## CRÍTICOS (0)

Ninguno.

## ALTOS (5)

| Id | Sitio | Hallazgo | Estado |
|---|---|---|---|
| **A1** | `src-tauri/src/lib.rs:3828` | Se registraban atajos globales (y Escape) en Windows, pese a «atajos: todavía no». | pagado · `fd24008` (PR #20; el gate exige el `return`) |
| **A2** | `src-tauri/src/stt/whisper.rs:161` | El texto de la última transcripción sobrevivía al corte y a «Terminar». | pagado · `fd24008` (PR #20; `soltar()` en el corte y al terminar; copias del audio a cero; lo verifica la CI de Windows) |
| **A3** | `src-tauri/src/modelo_whisper.rs:92` | La descarga del modelo no tenía ningún test; la guía y la CI afirmaban lo contrario. | pagado · `fd24008` (PR #20; `descargar_a`, 4 pruebas en el Mac) |
| **A4** | `src/pantallas/Idioma.tsx:254` | Texto de macOS en Idioma y Honestidad en Windows (motor, descarga, conexión, icono). | pagado · `fd24008` (PR #20; textos de Windows; e2e sin «macOS» ni «Mac», con control en macOS) |
| **A5** | `src/pantallas/Notas.tsx:573` | «Se borra aunque no abras la app» es falso en Windows. | pagado · `fd24008` (PR #20; variantes de Windows; e2e con control en macOS) |

## MEDIOS (12)

| Id | Sitio | Hallazgo | Estado |
|---|---|---|---|
| **M1** | `src-tauri/src/modelo_whisper.rs:109` | Sin timeout en la descarga: una conexión colgada deja `BAJANDO` activo hasta reiniciar. | pagado · PR #24 (S8 F2; `modelo_whisper.rs`: 20 s para conectar, 60 s la respuesta, 3 h el cuerpo, y la prueba de la línea mala) |
| **M2** | `src-tauri/src/lib.rs:1056` | La guarda «nunca en reunión» solo se mira al empezar la descarga. | pagado · PR #24 (S8 F2; `modelo_whisper::poner_la_guarda` corta la descarga si empieza una reunión) |
| **M3** | `src-tauri/src/stt/whisper.rs:38` | La latencia y el escalón al modelo pequeño no sobreviven a reabrir la app. | pagado · PR #24 (S8 F2; `prefs::LatenciasDeWhisper` y `stt::whisper::restaurar_latencias`) |
| **M4** | `src-tauri/src/lib.rs:1073` | `hay_una_reunion` mira solo la escucha; no mira el ensayo ni «solo notas». | pagado · PR #24 (S8 F2; `lib.rs::algo_en_marcha` cuenta la escucha, la sesión en cualquier modo y el ensayo) |
| **M5** | `src/pantallas/Sesion.tsx:446` | «Al oído» queda elegido por defecto en Windows, con la voz muda. | pagado · `0d05bf0` (PR #23; `Sesion.tsx:451-453` parte elegido solo si `voz.puede`) |
| **M6** | `src-tauri/src/ventana/mod.rs:107` | Falta el mínimo de Windows (10 2004) y su comprobación de versión. | pagado · PR #24 (S8 F2; `ventana::exigir_la_version_de_windows`: 19041, con cuadro al arrancar) |
| **M7** | `src/pantallas/Permisos.tsx:104` | El título y los «para» de Permisos siguen diciendo macOS en Windows. | pagado · PR #23 y PR #24 (S7 A1/A2 y S8 F1; `windows-sin-mac.test.tsx` recorre Permisos sin una sola coincidencia) |
| **M8** | `src/i18n/es.ts:331` | Más texto de macOS visible en Windows (Sesión, Notas, IA). | pagado · PR #23 y PR #24 (S7 A1/A2 y S8 F1; `windows-sin-mac.test.tsx` recorre Sesión, Notas e IA sin coincidencias) |
| **M9** | `src-tauri/src/desbloqueo.rs:85` | Jerga «sprint 007» en un error que llega a la interfaz. | pagado · PR #24 (S8 F1; `desbloqueo.rs`: el motivo de Windows dice dónde configurar Windows Hello y un test exige que no diga «sprint») |
| **M10** | `src/pantallas/Idioma.tsx:113` | «Instalar» que falla vuelve al botón sin un motivo escrito. | pagado · PR #24 (S8 F2; `Idioma.tsx` dice «No se pudo bajar el modelo» y `idioma-sin-modelo.test.tsx`) |
| **M11** | `src-tauri/tests/contra-el-mac-de-verdad.rs:28` | La mitad en marcha de la regla 1 solo corre en macOS. Declarada en el cierre. | pagado · PR #24 (S8 F2; `tests/efimero-en-windows.rs`, en el paso de `build-windows`) |
| **M12** | `src-tauri/src/llavero/windows.rs:131` | El test «de hardware» salta el centinela `vigilar` de `llavero.rs`. | pagado · PR #24 (S8 F2; el test de hardware llama a `hardware::vigilar`) |

## BAJOS (12)

| Id | Sitio | Hallazgo | Estado |
|---|---|---|---|
| **B1** | `src-tauri/src/ventana/mod.rs:394` | El comentario doc se partió al insertar el test. | pagado · PR #24 (S8 F2; los dos tests de `ventana/mod.rs` recuperaron cada uno su doc) |
| **B2** | `docs/GUIA-DE-PRUEBA.html:21` | El reemplazo global de «nuevo» dañó una instrucción de la guía. | pagado · PR #24 (S8 F2; la instrucción de la guía vuelve a decir `nuevo` y `mejorado`) |
| **B3** | `README.md:5` | Textos que dicen solo macOS; falta cómo compilar en Windows. | pagado · PR #24 (S8 F2; el README nombra Windows y dice cómo compilarlo) |
| **B4** | `decisions/006-stt-local-y-su-modelo.md:79` | Sigue diciendo que whisper es «el camino escrito». | pagado · PR #24 (S8 F2; enmienda en el ADR 006 §5) |
| **B5** | `src-tauri/src/modelo_whisper.rs:116` | Un corte dejaba un `.parcial` en disco. | pagado · `fd24008` (PR #20; se borra en cada error); pagado · PR #24 (S8 F2; limpieza al arrancar y comprobación de espacio, además del borrado en cada error de `fd24008`) |
| **B6** | `src/componentes/HoyEnWindows.tsx:37` | «Bajando…» no se refresca si la descarga empieza desde Idioma. | pagado · PR #24 (S8 F2; `useRefrescoMientras` en `HoyEnWindows`) |
| **B7** | `src/componentes/HoyEnWindows.tsx:156` | El botón del modelo pequeño sale aunque ya esté bajado. | pagado · PR #24 (S8 F2; el botón no sale con el modelo pequeño puesto) |
| **B8** | `tests/unit/windows-honestidad-y-manual.test.ts:43` | `estadoDelDetalle` devuelve «todavía no» por defecto. | pagado · PR #24 (S8 F1; `estadoDelTexto` lanza error ante un estado que no conoce) |
| **B9** | `src-tauri/src/stt/calibracion.rs:9` | El WAV de 320 KB entra también en el binario de macOS. | pagado · PR #24 (S8 F2; el recorte solo entra con `cfg(any(windows, test))`) |
| **B10** | `decisions/022-windows-detras-de-la-misma-frontera.md:9` | Cita «la invisibilidad está garantizada por el sistema», contra la regla 6. | pagado · PR #24 (S8 F2; `decisions/022` ya no cita la garantía del sistema) |
| **B11** | `src-tauri/Cargo.toml:96` | El gate no mira las features de `windows-sys`. | pagado · PR #24 (S8 F2; `contador-de-red` lee las features de `windows` y `windows-sys`) |
| **B12** | `docs/GUIA-DE-PRUEBA.html:873` | «Honestidad enseña la medición» solo es cierto tras medir; el botón de ajustes solo sale sin micrófono concedido. | pagado · PR #24 (S8 F2; la prueba w3 dice cuándo sale el botón y cuándo hay medición) |

## Casillas

1. **Alcance:** requiere ajustes → corregido en el sprint (A1); el resto, lote.
2. **Calidad:** la lógica nativa es sólida; los altos de calidad están pagados.
3. **Seguridad y privacidad:** sin secretos, sin enlaces de producción, sin datos reales. A2 y A5 pagados; M2 y M11 al lote.
4. **Frases caducadas:** A1, A4 y A5 pagados en el sprint; M6 a M9 y B3, en el sprint 008 (PR #24).
