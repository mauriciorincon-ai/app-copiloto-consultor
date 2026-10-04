# ADR 011 — Proveedores del modelo y minimización: qué redacta, qué sale y cómo se anonimiza

**Estado:** aceptado · **Fecha:** 2026-09-26 · **Sprint:** S2, fase 5 · Hermano de **«síntesis,
código primero»** (ADR 010), que decide **si** entra un LLM; este decide **cuál** y **qué ve**.

## Contexto

La orden del sprint fija el orden (a) modelo del sistema → (b) MLX → (c) API opt-in, «medidos en el
kit; nada se afirma antes», con `mock` como proveedor de CI. La regla dura 2 de la casa: **nada crudo
sale del equipo**; el API nace apagado, con clave del usuario, solo texto minimizado y anonimizado
localmente.

## Decisión

### Un adapter, tres proveedores construidos (y MLX condicionado), una interfaz

`src-tauri/src/sintesis/` define `Proveedor` (`nombre`, `disponible`, `redactar`). Los construidos
—`mock`, el modelo del sistema y el API— implementan lo mismo y **el mismo contrato los envuelve**:
esquema cerrado, `fundar()` (la fuente tiene que ser una de las fichas dadas, y la línea tiene que
decir lo que esa ficha dice) y el techo de 6 s. Cuál se usa lo decide la app por orden de
disponibilidad; **solo `AG_SINTESIS=mock` lo fuerza** (la CI y el kit). MLX se construye solo si (a)
no está o no rinde, y (a) rinde (medido en la fase 6: mediana ~0,8 s): queda en el roadmap.
*(Enmienda de la auditoría del S2, B4: decía «cuatro proveedores» y `AG_SINTESIS=mock|sistema|api`.)*

| Proveedor | Dónde corre | Qué necesita | Cómo se apaga |
|---|---|---|---|
| **mock** | en el proceso | nada | es el de la CI; en la app solo con `AG_SINTESIS=mock` |
| **(a) modelo del sistema** | en el Mac (Foundation Models, macOS 26, por el puente de Swift) | Apple Intelligence activado en Ajustes | el interruptor «Redactar sugerencias» |
| **(b) MLX** | en el Mac, con un modelo que el usuario descarga a su carpeta | **solo si (a) no está o no rinde** (la orden) | igual |
| **(c) API** | en el proveedor que el usuario elija | su clave, en el **Llavero** | su propio interruptor, apagado por defecto |

### Qué sale por el API, y cómo se anonimiza (patrón Velo)

1. **Minimizar:** solo el último turno del cliente —la frase de un tercero— y el titular y la línea
   de las tres fichas del top, cada una con su id. Nunca el transcript entero, ni documentos, ni
   audio, ni pantalla.
2. **Anonimizar en el Mac, antes de salir:** una **bóveda** en memoria cambia cada nombre conocido por
   un marcador (`[CLIENTE_1]`, `[PERSONA_1]`…) —los nombres de cliente del corpus
   (`corpus::clientes`: el nombre que va tras el «·» de cada ficha de cliente)— y además los patrones
   que delatan a alguien sin estar en ninguna lista: correos, teléfonos (también partidos por
   espacios), números de identificación y parejas de nombres propios. *(Enmienda B4: decía «y del
   diccionario», que no se usa; y los conocidos eran nombres de archivo hasta A2.)* La respuesta vuelve con marcadores y la bóveda los
   restaura **en el Mac**. La bóveda muere con la petición.
3. **Contar:** cada byte que sale pasa por `red::registrar_salida` —el único camino de entrada al
   contador— y Honestidad lo enseña. El gate del contador de red deja de ser «ninguna puerta» y se
   estrecha a **una puerta declarada**: la sesión de red vive solo en `nativo/Red.swift`
   (`URLSession` efímera), y `sintesis/api.rs` arma la petición y la cuenta.
4. **Registrar sin contenido:** proveedor, bytes, ms y costo al log. **Hecho en el sprint 003:** el
   texto exacto que salió, visible en la pantalla IA y solo en memoria, para que el usuario lo pueda
   leer. **Desde el sprint 003 (fase 0), cada petición deja en un registro de la reunión el texto
   exacto que salió**, trozo a trozo, con lo que la bóveda reemplazó en el Mac al lado de cada
   marcador y la cuenta de lo tapado (`sintesis::api::Registro` y `LoQueSalio`). Solo en memoria: lo
   vacían `⌥⎋` y el final de la sesión, y lo pide únicamente la ventana principal por comando
   (`lo_que_salio_al_api`), nunca un evento. La vista en IA existe desde la fase 1 del sprint 003
   (mirada 19): «Ver lo que salió». El registro guarda **las últimas 20 peticiones**
   (`TOPE_DE_LO_QUE_SALIO`) y Honestidad las cuenta. *(Enmienda B4; pago de B37; auditoría del S3, B20
   y B15.)*

### Costo

Tokens de entrada y salida (los que el proveedor devuelve) × su precio declarado en código con su
fecha. Suma por reunión (memoria) y por mes (un número en la carpeta de la app: metadato de costo,
que la regla 1 permite persistir). **Techo US$10/mes**; al llegarlo, la app vuelve sola a lo local.

## Consecuencias

- El contador de red **puede dejar de ser cero**, y solo por decisión del usuario. La frase de
  Honestidad y el manual se reescriben para decirlo, con su gate al lado.
- (b) MLX se construye **solo si** (a) no está disponible o no cumple el presupuesto en el kit.
- La validación contra un proveedor real es manual (estándar 7): la CI no llama a nadie.

## Enmienda (2026-09-28) — cuánto guarda cada proveedor lo que le mandas (auditoría del S3, M3)

La regla dura 2 dice que el API externo, si el usuario lo enciende, manda texto minimizado y anonimizado
**«bajo proveedor con no-retención»**. Hasta aquí nadie había leído qué hace cada proveedor con ese texto.
Se leyó en sus páginas oficiales el **2026-09-28**. Hasta la decisión de abajo, la app llamaba a la Gemini Developer
API (no a Vertex AI) y pedía `claude-haiku-4-5`, `gemini-2.5-flash` y `llama-3.3-70b-versatile` (`sintesis/api.rs`).

| Proveedor | Retención por defecto | ¿Entrena con lo que recibe? | Cómo se consigue no-retención | ¿Con una clave estándar? | Fuente | Leído |
|---|---|---|---|---|---|---|
| **Claude** (Anthropic API) | se borra en ≤ 30 días; lo que marque su sistema de seguridad, hasta 2 años | no, salvo que mandes feedback o lo autorices | acuerdo de retención cero por organización, pedido a Ventas | **no** | https://privacy.claude.com/en/articles/7996866 · https://privacy.claude.com/en/articles/7996868 · https://platform.claude.com/docs/en/manage-claude/api-and-data-retention | 2026-09-28 |
| **Gemini**, sin facturación | 55 días para vigilar abusos; lo que usa para mejorar productos, sin plazo publicado | **sí**, con revisores humanos | no existe | **no** | https://ai.google.dev/gemini-api/terms · https://ai.google.dev/gemini-api/docs/usage-policies | 2026-09-28 |
| **Gemini**, con facturación | 55 días para vigilar abusos | no | no existe en esta API; Google remite a Vertex AI, que es otro endpoint | **no** | https://ai.google.dev/gemini-api/terms · https://ai.google.dev/gemini-api/docs/zdr | 2026-09-28 |
| **Groq** | no retiene por defecto; puede registrar hasta 30 días solo para fiabilidad o abuso | no (lo prohíbe su contrato) | interruptor de retención cero en *Data Controls* de su consola | **sí**, según su documentación («All customers may enable»); su contrato dice «Eligible Customers» sin definirlo | https://console.groq.com/docs/your-data · https://console.groq.com/docs/legal/services-agreement | 2026-09-28 |

**Lo que dice la tabla:** con una clave individual estándar, **solo Groq** cumple «sin retención y sin
entrenamiento», y solo si el usuario enciende la retención cero en su consola. Claude no entrena, pero
guarda hasta 30 días sin un acuerdo. La Gemini Developer API no ofrece retención cero en ningún nivel, y
sin facturación entrena con lo que recibe.

**Decisión del usuario (2026-09-29):** con esta tabla delante, «Groq se queda y Claude también, Gemini sale».

| Proveedor | Decisión | Qué dice la app |
|---|---|---|
| Claude | se queda, con aviso | En IA, bajo el costo: «Claude no entrena con lo que le mandas, pero lo guarda hasta 30 días.» |
| Groq | se queda, con aviso | «Groq no entrena con lo que le mandas; sin retención cero en su consola, puede guardarlo hasta 30 días.» |
| Gemini | sale | Nada: sale de la lista de IA, del tipo `Externo` de Rust y de la maqueta. |

La línea de IA cabe en dos renglones: la pantalla llena su ventana de 640 px, y el gate de desbordes de
`pnpm fidelidad` la midió. El detalle —el acuerdo con Anthropic, dónde está el interruptor de Groq
(*Data Controls*)— está en el manual.

**Consecuencias:**

1. **La regla dura 2 queda más estrecha de lo que dice.** «Bajo proveedor con no-retención» solo lo cumple
   Groq, y solo con su interruptor encendido. Claude guarda hasta 30 días sin un acuerdo. El usuario lo
   decide sabiéndolo, y la app lo dice donde se elige el proveedor, en vez de callarlo. Es una desviación
   de la regla de la planeadora: va a la bitácora bajo «Desviación del plan» y al summary.
2. **La cláusula modelo** (`data/jurisdicciones/catalogo.json`) decía «bajo condiciones de no retención».
   Ahora dice lo que es verdad con los dos proveedores: «el proveedor no entrena con ellos y puede
   conservarlos hasta 30 días».
3. **Unas preferencias guardadas con Gemini** no tumban el archivo entero: el proveedor vuelve al de
   fábrica y **el API queda apagado**. Nunca se enciende solo con otro proveedor (`prefs::de_texto`, con
   test que se vio en rojo).
4. **Una clave de Gemini guardada** se quedaría en el Llavero, sin que la app la lea. La app nunca se
   distribuyó, y su único usuario no guardó ninguna: lo comprobó el 2026-09-28 en Keychain Access.

**Gate:** `tests/unit/proveedores-con-su-retencion.test.ts`. Cada `nombre` de `EXTERNOS` (`src/ia.ts`)
tiene al menos una fila en esta tabla con una fecha `AAAA-MM-DD` y una URL. Nació en rojo: antes de esta
enmienda no había tabla. Desde la decisión, además: cada proveedor de `EXTERNOS` tiene «se queda» en la
tabla de la decisión y su aviso en los dos idiomas (`cuaderno.retencion<Nombre>`), y el que «sale» no está
en `EXTERNOS`, ni en `enum Externo`, ni entre los botones de `docs/diseno/ia.html`. También nació en rojo. Los términos se vuelven a leer antes de cada release (estándar 7), y la fila se
fecha de nuevo.

