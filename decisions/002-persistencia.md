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

---

## Enmienda 2 — las preferencias y un solo escritor (sprint 003, fase 0, 2026-09-27)

**Lo que cambia.** La tabla de arriba reservaba `prefs/` para el sprint 3: «lo que el usuario
escribe y elige». Ahora existe, como `src-tauri/src/prefs.rs` y un archivo:
`preferencias.json`, en la carpeta de configuración de la app, al lado del diccionario y del gasto
del mes.

| Qué se recuerda | Qué NO, a propósito |
|---|---|
| el idioma de cada pista · «redactar sugerencias» · el proveedor externo y si está encendido · la lectura automática de la pantalla | la puerta local (nace cerrada en cada arranque, ADR 018) · el modo solo audio (lo enciende una tecla en la reunión) · nada de ninguna reunión |

**El API no se enciende solo si su clave ya no está:** el archivo dice «encendido», pero al arrancar
se comprueba el Llavero, y borrar la clave en «Acceso a Llaveros» manda sobre lo que el archivo
recuerde.

**Un solo escritor para todo lo que persiste: `src-tauri/src/almacen.rs`.** Hasta el sprint 002 cada
archivo tenía su ayudante privado (el diccionario, el gasto, la huella del acople). El sprint 003
añade preferencias, notas cifradas, la bandeja y la lista de vencimientos, y cinco copias de la misma
regla eran cinco oportunidades de olvidarla en una. Las tres reglas viven ahora en un sitio, con su
test en rojo:

1. el archivo **nace** en 600 (`create_new` con su modo, sin ventana en 644);
2. **nunca desaparece a medias**: temporal, `sync_all` y renombrado atómico;
3. su carpeta es solo del dueño (700), y se repara si estaba floja — **solo la carpeta del archivo**:
   sus padres (`~/Documents`, por ejemplo) no son de esta app.

**Qué entra en el inventario del efímero:** nada nuevo en esta enmienda. `preferencias.json` se
escribe cuando el usuario cambia algo, no durante una sesión; la sesión completa del gate en marcha
no lo toca, y si algún día un camino de la sesión lo escribiera, el gate lo denunciaría como intruso.

---

## Enmienda 3 — las notas: el módulo que decide no escribe (sprint 003, fase 1, 2026-09-27)

La tabla de la decisión decía `notas/ — Sí`. **Deja de ser así**, por la misma razón que el
diccionario en la enmienda 1: el cuaderno recibe los turnos del micrófono para «Conservar mis turnos»,
y entre ellos llegan los marcados como **eco** —el micrófono oyendo al cliente por los altavoces—,
que son la voz del cliente. El módulo que tiene eso en las manos no puede tener manera de escribirlo.

| Quién | Qué hace | Puede tocar disco |
|---|---|---|
| `notas/` | el cuaderno en memoria, el filtro de tus turnos (micrófono, sin eco), el contenido del archivo y el cifrado | **No** — está en `PROTEGIDOS` |
| `carpeta.rs` | pide la llave al Llavero, escribe con `almacen`, lista, abre, exporta, borra y barre lo vencido | Sí — recibe el contenido ya armado; no ve un turno |
| `reunion.rs` | cuándo empieza y se cierra una reunión, ⌃⌥P, la protección del cuaderno, guardar al salir | Sí, a través de `carpeta.rs` |

**Lo que entra en el inventario del efímero: la carpeta de tus notas**, la tercera entrada de
`Permitido` en `contra-el-mac-de-verdad.rs`. La sesión completa del gate en marcha escribe notas de
verdad —con tu turno, el del cliente y su eco llevando la canaria— y el test **descifra** el archivo
con su llave antes de buscar la canaria. Las dos demos en rojo están en la bitácora: sin la línea en
`Permitido`, el archivo se denuncia como intruso; con la canaria plantada en la nota, solo el test que
descifra la ve (la comprobación vieja, sobre los bytes cifrados, pasa en verde sin haber mirado nada).

El detalle —formato, llave, desbloqueo, retención— es el ADR 015.
