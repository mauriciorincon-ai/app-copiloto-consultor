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

---

## Enmienda 1 — el diccionario técnico (sprint 002, fase 1, 2026-09-24)

El diccionario técnico (B3) obligó a mirar esta tabla otra vez, y de paso a afinar su propio
principio.

**El caso es nuevo:** el módulo `diccionario/` **recibe el transcript del cliente** —cada turno entra
y sale corregido— y a la vez **produce un archivo que persiste**, porque la lista de términos es del
consultor, como sus notas. Por la tabla de arriba, lo primero lo pone del lado de los protegidos y lo
segundo del lado de los que pueden escribir. No puede ser las dos cosas.

**Cómo se resuelve:** la frontera de este ADR no es «texto vs. audio» sino «del usuario vs. de
terceros», y aquí hay que aplicarla **al dato, no al módulo**. Lo que se hace es partir la
responsabilidad:

| Quién | Qué hace | Puede tocar disco |
|---|---|---|
| `diccionario/` | el motor: corrige el turno, y **serializa a un `String`** | **No** — está en `PROTEGIDOS` |
| `lib.rs` | lee y escribe ese `String` como archivo, y cierra sus permisos | Sí — no ve un solo turno |

Queda así en la tabla de la decisión:

| Módulo | Puede tocar disco | Por qué |
|---|---|---|
| `diccionario/` | **No** | recibe el transcript del cliente. Su archivo lo escribe la capa de arriba |

**El plan del sprint 002 decía «`diccionario/` puede tocar disco».** Esto es más fuerte y cuesta lo
mismo: el módulo que toca la voz del cliente **no tiene manera** de escribirla, y no hace falta
confiar en que nadie se equivoque al añadir la función siguiente. `pnpm verify:ephemeral` lo vigila
desde la primera línea — de hecho el gate sobre el gate lo cazó solo, en cuanto la cabecera del
módulo nuevo se declaró protegida sin estar en la lista.

**La regla que hace inocuo que persista: el diccionario NO APRENDE DE LA REUNIÓN.** Sus entradas
salen de dos sitios y de ninguno más: el archivo que el usuario escribe, y los **nombres propios de
su corpus** —sus propios documentos—, que se recalculan en cada arranque y **no se guardan en el
archivo**. Un diccionario que se corrigiera con lo que oye sería un transcript persistido con otro
nombre. `corregir` toma `&self`, y ese `&` es la regla escrita en el tipo.

**Y lo que entra en el inventario del efímero.** El archivo es una entrada nueva del `Permitido` de
`contra-el-mac-de-verdad.rs`, la segunda de la app tras el índice del corpus. Cada entrada de esa
lista es una promesa que se afloja, así que se añaden de a una, nombradas, y el summary las lista. Su
rojo está registrado: sin la línea, la sesión lo denuncia como fuga.

**Permisos: 600, y nace así.** Es la regla 17-bis — un derivado no nace menos privado que su fuente, y
este desciende de los documentos del usuario. La primera versión lo creaba con `fs::write` y lo
apretaba después; lo delató la traza del propio gate del efímero («el archivo estaba en 644; se dejó
en 600»). Funcionaba y estaba mal: entre las dos llamadas hay una ventana, corta pero real, en la que
la jerga del consultor es legible por cualquier cuenta del Mac. Ahora se crea con `create_new` y su
modo, y **hay un test que comprueba que no hubo reparación** — no solo que el modo final sea el bueno.
