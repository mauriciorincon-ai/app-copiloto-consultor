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
4. **Registrar sin contenido:** proveedor, bytes, ms y costo al log. **Pendiente, sprint 003:** el
   texto exacto que salió, visible en la pantalla IA y solo en memoria, para que el usuario lo pueda
   leer. Hoy IA no enseña ni el texto ni cuántos datos se taparon: la bóveda los cuenta
   (`Boveda::tapadas`) y solo lo leen los tests. *(Enmienda B4.)*

### Costo

Tokens de entrada y salida (los que el proveedor devuelve) × su precio declarado en código con su
fecha. Suma por reunión (memoria) y por mes (un número en la carpeta de la app: metadato de costo,
que la regla 1 permite persistir). **Techo US$10/mes**; al llegarlo, la app vuelve sola a lo local.

## Consecuencias

- El contador de red **puede dejar de ser cero**, y solo por decisión del usuario. La frase de
  Honestidad y el manual se reescriben para decirlo, con su gate al lado.
- (b) MLX se construye **solo si** (a) no está disponible o no cumple el presupuesto en el kit.
- La validación contra un proveedor real es manual (estándar 7): la CI no llama a nadie.
