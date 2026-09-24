# ADR 003 — Observabilidad sin contenido

- **Fecha:** 2026-09-20 · **enmendada** 2026-09-22 (fase 2 de la auditoría, hallazgo A10)
- **Sprint:** 001 «La banda y la ficha»
- **Estado:** aceptada, con la enmienda de abajo

## Contexto

La app necesita diagnosticarse: latencias, errores del bridge, permisos denegados. Pero una app
que promete no persistir nada de la reunión **no puede escribir logs con contenido** — un log es
disco, y un log con una frase del cliente rompe la promesa igual que un archivo de audio.

El `src/lib/observability.ts` del kit no viaja: importa Sentry, que es para Next y que además
mandaría datos fuera del equipo.

## Decisión

**Metadatos y jamás contenido. La biblioteca con que se escriben es lo secundario, y la enmienda de
abajo cambia justo eso.**

Se registran: qué ocurrió, cuándo, cuánto tardó, de qué pista vino. **Nunca**: lo que se dijo, lo
que se transcribió, lo que se leyó de la pantalla, el texto de una ficha.

Y para que «jamás» sea comprobable y no una intención, el gate es un **término plantado**: una
sesión sintética incluye una frase marcada y un test falla si esa frase aparece en cualquier log.
Es la misma forma de la carnada de gitleaks — una promesa sobre lo que NO pasa solo se sostiene
si algo intenta activamente hacerla fallar.

**Dónde vive ese gate, desde la fase 2 de la auditoría:**

| Gate | Qué mira | Dónde |
|---|---|---|
| `una_sesion_completa_no_deja_nada_en_el_disco…` | los **archivos**: ni uno nuevo contiene la frase | `tests/contra-el-mac-de-verdad.rs` |
| `la_canaria_del_cliente_no_aparece_en_el_log` | el **log**: corre la sesión en un proceso hijo y lee su salida | idem |
| `pnpm verify:ephemeral` | que ningún módulo protegido tenga API de disco o de red | `scripts/verify-ephemeral.mjs` |

El del log corre la sesión en un **proceso aparte** a propósito: capturar `println!` desde dentro
del mismo proceso obligaría a sustituir la salida estándar, y entonces el gate mediría un logger de
mentira en vez del del producto.

**Sin Sentry ni telemetría remota.** El contador de salida a red marca 0 en modo local, y un
reportero de errores lo pondría en otra cosa.

## Enmienda (2026-09-22) — `println!` hoy, `tracing` cuando haya a quién escuchar

**Lo que la auditoría encontró:** esta decisión nombraba `tracing` y el sprint entregó **45
`println!` y cero `tracing::`**. Y `pino`, el logger de la interfaz, estaba declarado en
`package.json` **sin un solo uso**. Una decisión que el código no sigue no es una decisión: es una
frase, y una dependencia declarada y sin usar es superficie regalada en un repo público.

**La enmienda, con su razón.** Se elige lo segundo —ajustar la decisión al código— y no lo primero,
y no por comodidad:

- `tracing` vale por lo que trae **alrededor**: suscriptores, filtros por módulo, spans anidados,
  salidas estructuradas. Esta app no tiene ninguno de los cuatro y no puede tenerlos: **no hay
  sumidero al que escribir.** No hay archivo de log (sería disco), no hay servicio remoto (sería
  red), no hay telemetría. El único destino es la consola de quien está desarrollando.
- Con ese único destino, `tracing` sin suscriptor es una indirección que no registra nada, y con
  `tracing_subscriber::fmt()` es `println!` con más pasos y un árbol de dependencias detrás.
- `pino` sale de `package.json`. Vuelve el día que la interfaz tenga algo que registrar, con su uso
  en el mismo PR.

**Qué se registra, entonces:** `println!` con un prefijo por subsistema (`[escucha]`, `[corpus]`,
`[ficha]`, `[acople]`, `[sesión]`), y **solo metadatos**: qué pasó, de qué pista, cuánto duró,
cuántas letras tenía —nunca cuáles—. Es lo que el sprint ya hacía; lo que faltaba era decirlo aquí.

**Cuándo se revisa:** cuando exista un destino de verdad (un archivo de diagnóstico que el usuario
exporte a mano, un modo de depuración con niveles) la decisión vuelve a `tracing` y este ADR se
enmienda otra vez con esa razón delante.

**Y el gate que sí faltaba se construyó** (ver abajo): hasta la auditoría, el «término plantado» de
esta decisión solo se buscaba en los **archivos del disco**. En el log —la otra salida, la que más
fácil se copia y se pega— no se buscaba nunca.

## Consecuencias

- Depurar un problema de transcripción es más incómodo: hay identificadores y tiempos, no frases.
  Es el precio de la promesa, y se paga a sabiendas.
- Cuando el usuario reporte un fallo, lo que se le pedirá son metadatos, nunca «pégame el
  transcript».
