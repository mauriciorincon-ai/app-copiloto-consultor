---
sprint: 001
app: copiloto-consultor
fase: 1 (solo lectura)
fecha: 2026-09-22
auditor: sesión independiente, con el diff delante
veredicto: requiere ajustes
---

# Auditoría del sprint 001 — Fase 1

**Método (kit v1.26.0):** la Fase 1 la corrió un auditor **independiente del constructor**, con
`git diff main...HEAD` delante y archivo por archivo — no con la bitácora, que dice qué se creyó
hacer mientras el diff dice qué se hizo. 27 commits, 256 archivos, +21 753 líneas.

**Veredicto: REQUIERE AJUSTES.** 1 hallazgo crítico, 10 altos, 14 medios, 7 bajos. El crítico deja
**el outcome principal del sprint sin funcionar en el producto**, y cuatro de los altos tienen la
bitácora o un comentario del código afirmando lo contrario de lo que el código sostiene.

> La ingeniería de la parte nativa está bien: el anillo que se pisa, el invariante que aborta el
> arranque, el acople con huella y devolución idempotente, el `catch_unwind` del PDF, el turno de
> los tests, la demo en rojo por gate. **El problema no está ahí: está en la costura entre Rust y
> el webview, que es lo único que ningún test de este sprint atraviesa.**

---

## CRÍTICO

### C1 · La ficha automática nunca llega a la banda

`escucha::Novedad` se serializa **internamente etiquetada** (`#[serde(tag = "que")]`,
`src-tauri/src/escucha/mod.rs:48`) y el webview la lee **externamente etiquetada**
(`src/ficha.ts:70`: `{ Aparece?: Aparicion; Turno?: … }`).

**Comprobado imprimiendo el JSON real**, no razonándolo:

```
{"que":"aparece","clase":"sinResultado","buscado":"iso 27001","cercanas":[],
 "maniobra":"credencial","motivo":"pregunta","ms":1200,"hora":"14:02"}
```

`n.Aparece` y `n.Turno` son **siempre `undefined`**. En el binario:

- el estado «buscando» nunca se enciende;
- **la ficha automática nunca se pinta** — la banda se queda en «esperando» toda la reunión;
- solo funciona `⌘⇧A`, que va por otro evento y por `pedir_ficha`.

El outcome del sprint es *«en ≤4 s tras el fin de turno del cliente la banda muestra una ficha»*.
Hoy no ocurre.

**Por qué ningún test lo vio:** los 87 unitarios y los 66 e2e corren con `hayTauri() === false`,
donde `useFicha` devuelve la muestra sin suscribirse. La cobertura lo delata: `src/ficha.ts`
líneas 98-121 sin cubrir — exactamente ese bloque.

**Y el mismo repo demuestra que la convención se conocía:** `src/cuaderno.ts:22-24` modela
`Reunion` correctamente con `que:`.

---

## ALTOS

| # | Hallazgo | Dónde |
|---|---|---|
| **A1** | La banda sigue pintando contenido inventado dentro del producto — incluida **una frase puesta en boca del cliente** con hora falsa («cliente 14:02 · "Y la limpieza de datos, ¿eso está dentro del alcance?"»). Y `verificado` es `true` por defecto: la banda dice «protegido» en una llamada de Zoom, contra la promesa de invisibilidad **graduada** | `src/componentes/Banda.tsx:149,181,187,199-201,253` · `src/App.tsx:70` |
| **A2** | **El kill-switch no corta el hilo que transcribe.** El bucle `for encargo in recibe` no consulta `viva`: el audio del cliente en vuelo se transcribe DESPUÉS del corte, repuebla la ventana que se acaba de vaciar y vuelve a guardar la última pregunta que `reiniciar()` había pisado | `src-tauri/src/escucha/mod.rs:264-291` vs `:345-372` |
| **A3** | **`panic = "abort"` en release anula el `catch_unwind` del PDF.** El módulo promete que «un documento dañado no detiene a los otros»; en el binario que se distribuye, **cierra la app**. El test que lo cubre corre en debug: un gate que no puede fallar en el modo en que el usuario lo usa | `src-tauri/Cargo.toml:86` vs `src-tauri/src/corpus/leer.rs:235` |
| **A4** | **`escucha/` no está en `verify:ephemeral`, y su propia cabecera afirma que sí.** Es el módulo que copia el audio del turno y mantiene la ventana de transcript: el de mayor superficie. Hoy un `fs::write` ahí pasa el gate en verde | `escucha/mod.rs:9-10` vs `scripts/verify-ephemeral.mjs:34-46` |
| **A5** | Tras un reenganche de pista, **el disparador queda bloqueado el resto de la sesión**: el reloj de la pista vuelve a 0 y `Disparador.ultimo_ms` sigue en el reloj viejo, así que `ahora - antes < ESPERA_MS` es siempre cierto. El mismo desfase corrompe el detector de eco, que compara las dos pistas como si compartieran reloj | `escucha/mod.rs:396-406` · `disparo/mod.rs:128-134` · `voz/eco.rs:57-66` |
| **A6** | Copy en español cableado en un componente llega a la interfaz inglesa («sin modelo», «no lo reconoce», y `d.motivo` que viene de Rust). **Y el gate del diccionario no puede verlo**: compara `i18n/` contra la maqueta, no barre los componentes. Es un agujero de clase | `src/pantallas/Idioma.tsx:141-152` |
| **A7** | **No hay forma de instalar el modelo de voz desde la app.** `instalar_idioma` no tiene un llamador. En un Mac sin el modelo, la transcripción es inalcanzable y la pantalla solo dice «sin modelo» sin ninguna acción — mientras tres comentarios del código afirman que hay un botón | `lib.rs:420` · `cuaderno.ts:318` · `Idioma.tsx` |
| **A8** | La pantalla de Honestidad afirma *«no existe código capaz de abrir una conexión»* — y existe desde la fase 3 (`downloadAndInstall` en el puente). **El gate del contador no barre `src-tauri/nativo/`**: el único archivo que puede abrir una conexión es el único que ese gate no lee | `src/i18n/es.ts:242` · `tests/unit/contador-de-red.test.ts:29-31` |
| **A9** | El kit de evaluación no mide **tres de las cuatro** cosas del plan: P/R del disparo ±1 turno, WER informativo, audio de mezcla, y la **mediana** de latencia sobre el kit. El `LEEME` las sigue prometiendo «para la fase 5», que ya terminó | `contra-el-mac-de-verdad.rs:696-761` · `docs/kit-de-prueba/audio/LEEME.md:17` |
| **A10** | **ADR 003 decide `tracing` y el código no lo usa**: 45 `println!` y cero `tracing::`. `pino` declarado en producción sin un solo uso. Y el «término plantado en logs» que la DoD exige **no existe como test**: la canaria solo se busca en archivos del disco, nunca en la salida de log | `decisions/003-observabilidad.md` · `package.json` |

---

## MEDIOS (14) y BAJOS (7)

Resumen de los que tocan una regla dura o una promesa de pantalla:

- **M1** `"csp": null` en `tauri.conf.json` — en una app cuya promesa mayor es «nada crudo sale del
  equipo», el webview puede cargar de cualquier origen.
- **M2** Sesión afirma `Funciona` en las dos pistas **sin leer `abierta`/`motivo`**: con el tap
  caído, la app dice que funciona.
- **M4** Tras `⌥⎋` **la banda no vuelve** hasta reiniciar la app, y el manual no lo advierte.
- **M9** El gate de efímero en runtime **no mira `~/Library`** —donde escribe una app de macOS— y
  solo compara archivos NUEVOS: una fuga que *añada* a un archivo existente es invisible.
- **M10** `nativo.rs` reinterpreta los bytes del buffer como `f32` **sin validar el formato**.
- **M11** «Qué puedes hacer ya, sin conceder nada» marca «Todavía no» en dos cosas que **ya se
  pueden hacer**.
- **M3, M5, M6, M7, M8, M12, M13, M14** y **B1–B7**: ver el detalle en la bitácora de ajustes.

---

## Frases que caducaron (9)

Barrido **por promesa aplazada**, no por el nombre de la feature.

| Dónde | Qué afirma y por qué es falso hoy |
|---|---|
| `MANUAL-DE-USO.md:89-92` | «o se queda callado, la app busca… **sola**» — el disparo por silencio no está cableado, y por C1 **la ficha no aparece sola en absoluto** |
| `MANUAL-DE-USO.md:39-42` | «te lo dice en la banda con "sin verificar"» — la banda dice «protegido» siempre (A1) |
| `MANUAL-DE-USO.md:146-149` | «la única vez que toca la red es **cuando tú le pides** instalar el modelo» — no hay forma de pedirlo (A7) |
| `src/i18n/es.ts:242` **en pantalla** | «no existe código capaz de abrir una conexión» — dejó de ser cierto en la fase 3 (A8). En la pantalla de Honestidad, el peor sitio posible |
| `src/i18n/es.ts:90-96` | «**Muere en la fase 4**, cuando el corpus real alimente la banda» — no murió: cinco de sus cadenas se pintan (A1) |
| `Permisos.tsx:122-133` | «Todavía no» en indexar el corpus y buscar a mano, que ya funcionan (M11) |
| `kit-de-prueba/audio/LEEME.md:17` | promete para la fase 5 lo que la fase 5 no añadió (A9) |
| `corpus/leer.rs:10-13` | «va dentro de `catch_unwind`… la indexación sigue» — falso en release (A3) |
| `escucha/mod.rs:9-10` | «`pnpm verify:ephemeral` lo comprueba» — no lo comprueba (A4) |

---

## Campos del contrato sin consumidor: 17 campos y 2 structs completas

Comprobación mecánica sobre cada struct serializada a la interfaz.

**Huérfanos graves:** `EstadoDePista.abierta` y `.motivo` (0 lectores — son los campos que
distinguen una avería de un silencio, y Sesión pinta `Funciona` sin ellos) · **toda la struct
`Documento`** (`documentos_del_corpus` no tiene llamador: la promesa de la maqueta «cada fila con
su motivo en español llano» no llega a existir) · **toda la struct `corte::Informe`**, que además
viaja **sin `rename_all`** (`bytes_en_red` llegaría en snake_case) · `Aparicion.ms` — **la latencia
medida no se enseña en ninguna parte**, y es el presupuesto del sprint.

**El patrón detrás es el mismo que C1:** el contrato Rust→TS se escribió dos veces a mano y nadie
los compara. Un gate que derive los nombres del `.rs` y los compare con `cuaderno.ts`/`ficha.ts`
habría cazado C1, los 17 huérfanos y el snake_case — y **puede fallar**, porque el estado de hoy
lo pone en rojo.

---

## Lo que está bien

Invariante de protección que **aborta el arranque** y se comprueba sobre el archivo que se envía ·
el anillo que sobrescribe en vez de mover el cursor · `corte.rs` con dos `match` sin comodín en
vez de un test contra un número propio · el fin de turno medido contra el rango de la **orden** y
no contra su propia constante · la huella del acople en 600/700 con reparación · el remuestreador
que filtra antes de decimar · el eco descubierto corriendo y **marcado, no borrado** · la reserva
de idiomas descubierta midiendo y movida fuera de la consulta al ver que gastaba los cupos del
usuario · ADR 006 con la medición delante · el arnés de fidelidad que declara su árbol · el kit de
evaluación que **falló al nacer** y encontró seis interrogativos que faltaban · las cuatro miradas
con veredicto citado del usuario y archivo abierto.

---

## Plan de la Fase 2 (escrito para que lo ejecute cualquier modelo)

Cada hallazgo Crítico/Alto trae en el reporte del auditor su ajuste con archivo, línea, cambio
exacto y criterio de «ajuste verificado». Orden propuesto:

1. **El producto no funciona sin esto:** C1 → A2 → A4 → A5 → A3.
2. **La app no puede afirmar lo que afirma:** A1 → A8 → A6 → A7 → las 9 frases caducadas.
3. **Alcance y método:** A9 (P/R + mediana; WER y mezcla a deuda) → A10 (implementar el ADR o
   enmendarlo, y en los dos casos el gate de la canaria sobre el log) → **el gate de contrato
   Rust→TS**, con su demo en rojo.
4. **Deuda declarada en el summary con sprint de pago:** M1–M14, B1–B7, y la desviación del VAD
   (hoy solo en el encabezado del módulo; debe subir a `## Desviación del plan`).
