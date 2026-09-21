# ADR 002 — Qué persiste, dónde y con qué llave

- **Fecha:** 2026-09-20
- **Sprint:** 001 «La banda y la ficha»
- **Estado:** aceptada

## Contexto

La promesa central de la app es que **nada de la reunión queda**. Pero «nada» resultó ser falso
en cuanto el usuario miró el diseño: lo que él escribe es suyo, lo que él dice es suyo, y quiso
conservarlo. La mirada 3 de la Etapa de Diseño cambió la regla, y la mirada 4-bis añadió la
bandeja. El `design-system.md` §9-bis y §9-quater lo dejaron escrito, y el `CLAUDE.md` de la app
lo tiene en su forma final.

Este ADR fija cómo se implementa esa tabla, para que la distinción no dependa de que cada
programador recuerde la conversación.

## Decisión

**La frontera no es «texto vs. audio»: es «del usuario vs. de terceros».** Se implementa en la
estructura del crate, no en revisiones de código:

| Módulo | Puede tocar disco | Por qué |
|---|---|---|
| `capture/`, `stt/` | **No** | manejan audio y transcript, incluidos los de terceros |
| `corpus/` | Sí | documentos **propios** del usuario y su índice |
| `notas/`, `prefs/` (S3) | Sí | lo que el usuario escribe y elige |

`pnpm verify:ephemeral` barre los protegidos buscando API de archivo y de socket. `corpus/` queda
**fuera** del barrido a propósito: si estuviera dentro, el gate sería imposible de cumplir y la
salida fácil sería aflojarlo — y un gate aflojado deja de proteger lo que sí importa.

Lo que persiste va **cifrado en la carpeta del usuario**, con llave en el Llavero ligada a la
máquina, y con retención que **se cumple sola** aunque la app no vuelva a abrirse.

## Alternativas consideradas

- **Una base de datos única con banderas de «efímero»:** una bandera mal puesta convierte la
  promesa en mentira y nada lo detecta. La separación por módulo es verificable por un script.
- **Cifrar todo, incluido el índice del corpus:** el índice sale de documentos que el usuario ya
  tiene en claro en su disco; cifrarlo daría una sensación de seguridad sin añadir ninguna, y
  complicaría que el usuario inspeccione lo que la app sabe de él.

## Consecuencias

- Mover código entre `capture/` y `corpus/` deja de ser un refactor inocente: cambia lo que la
  app promete. El gate lo hace visible en la CI.
- La fase 5 añade la verificación en **runtime** (sesión completa ⇒ cero archivos nuevos), porque
  el barrido estático ve el código, no el comportamiento. Las dos capas cubren fallos distintos.
