# ADR 011 — Proveedores del modelo y minimización: qué redacta, qué sale y cómo se anonimiza

**Estado:** aceptado · **Fecha:** 2026-09-26 · **Sprint:** S2, fase 5 · Hermano de **«síntesis,
código primero»** (ADR 010), que decide **si** entra un LLM; este decide **cuál** y **qué ve**.

## Contexto

La orden del sprint fija el orden (a) modelo del sistema → (b) MLX → (c) API opt-in, «medidos en el
kit; nada se afirma antes», con `mock` como proveedor de CI. La regla dura 2 de la casa: **nada crudo
sale del equipo**; el API nace apagado, con clave del usuario, solo texto minimizado y anonimizado
localmente.

## Decisión

### Un adapter, cuatro proveedores, una interfaz

`src-tauri/src/sintesis/` define `Proveedor` (`nombre`, `disponible`, `redactar`). Los cuatro
implementan lo mismo y **el mismo contrato los envuelve**: esquema cerrado, `fundar()` (la fuente
tiene que ser una de las fichas dadas) y el techo de 6 s. Cuál se usa lo decide la app por orden
de disponibilidad, o `AG_SINTESIS=mock|sistema|api` para forzarlo (la CI usa `mock`).

| Proveedor | Dónde corre | Qué necesita | Cómo se apaga |
|---|---|---|---|
| **mock** | en el proceso | nada | es el de la CI; en la app solo con `AG_SINTESIS=mock` |
| **(a) modelo del sistema** | en el Mac (Foundation Models, macOS 26, por el puente de Swift) | Apple Intelligence activado en Ajustes | el interruptor «Redactar sugerencias» |
| **(b) MLX** | en el Mac, con un modelo que el usuario descarga a su carpeta | **solo si (a) no está o no rinde** (la orden) | igual |
| **(c) API** | en el proveedor que el usuario elija | su clave, en el **Llavero** | su propio interruptor, apagado por defecto |

### Qué sale por el API, y cómo se anonimiza (patrón Velo)

1. **Minimizar:** solo el último turno del cliente y el titular, la línea y la fuente de las tres
   fichas del top. Nunca el transcript entero, ni documentos, ni audio, ni pantalla.
2. **Anonimizar en el Mac, antes de salir:** una **bóveda** en memoria cambia cada nombre conocido por
   un marcador (`[CLIENTE_1]`, `[PERSONA_1]`…) —los nombres del corpus (fichas de cliente) y del
   diccionario— y además los patrones que delatan a alguien sin estar en ninguna lista: correos,
   teléfonos y números de identificación. La respuesta vuelve con marcadores y la bóveda los
   restaura **en el Mac**. La bóveda muere con la petición.
3. **Contar:** cada byte que sale pasa por `red::registrar_salida` —el único camino de entrada al
   contador— y Honestidad lo enseña. El gate del contador de red deja de ser «ninguna puerta» y se
   estrecha a **una puerta declarada**: el cliente HTTP vive solo en `sintesis/api.rs`.
4. **Registrar sin contenido:** proveedor, bytes, ms y costo al log; el texto que salió, solo en la
   pantalla IA y en memoria, para que el usuario lo pueda leer.

### Costo

Tokens de entrada y salida (los que el proveedor devuelve) × su precio declarado en código con su
fecha. Suma por reunión (memoria) y por mes (un número en la carpeta de la app: metadato de costo,
que la regla 1 permite persistir). **Techo US$10/mes**; al llegarlo, la app vuelve sola a lo local.

## Consecuencias

- El contador de red **puede dejar de ser cero**, y solo por decisión del usuario. La frase de
  Honestidad y el manual se reescriben para decirlo, con su gate al lado.
- (b) MLX se construye **solo si** (a) no está disponible o no cumple el presupuesto en el kit.
- La validación contra un proveedor real es manual (estándar 7): la CI no llama a nadie.
