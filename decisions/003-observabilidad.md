# ADR 003 — Observabilidad sin contenido

- **Fecha:** 2026-09-20
- **Sprint:** 001 «La banda y la ficha»
- **Estado:** aceptada

## Contexto

La app necesita diagnosticarse: latencias, errores del bridge, permisos denegados. Pero una app
que promete no persistir nada de la reunión **no puede escribir logs con contenido** — un log es
disco, y un log con una frase del cliente rompe la promesa igual que un archivo de audio.

El `src/lib/observability.ts` del kit no viaja: importa Sentry, que es para Next y que además
mandaría datos fuera del equipo.

## Decisión

**`tracing` en Rust + un logger de metadatos en la UI. Jamás contenido.**

Se registran: qué ocurrió, cuándo, cuánto tardó, de qué pista vino. **Nunca**: lo que se dijo, lo
que se transcribió, lo que se leyó de la pantalla, el texto de una ficha.

Y para que «jamás» sea comprobable y no una intención, el gate es un **término plantado**: una
sesión sintética incluye una frase marcada y un test falla si esa frase aparece en cualquier log.
Es la misma forma de la carnada de gitleaks — una promesa sobre lo que NO pasa solo se sostiene
si algo intenta activamente hacerla fallar.

**Sin Sentry ni telemetría remota.** El contador de salida a red marca 0 en modo local, y un
reportero de errores lo pondría en otra cosa.

## Consecuencias

- Depurar un problema de transcripción es más incómodo: hay identificadores y tiempos, no frases.
  Es el precio de la promesa, y se paga a sabiendas.
- Cuando el usuario reporte un fallo, lo que se le pedirá son metadatos, nunca «pégame el
  transcript».
