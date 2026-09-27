# ADR 016 — Las propuestas y la bandeja

- **Fecha:** 2026-09-27
- **Sprint:** 003 «El cuaderno y el cierre», fase 2
- **Estado:** aceptada. Los puntos de FORMA esperan el veredicto de la mirada 20 y se marcan
  «(mirada 20)»; si la mirada los cambia, esta ADR se enmienda antes de construir su pantalla.

## Contexto

La maqueta aprobada responde a «soy malo tomando notas» con dos piezas (`notas.html`, «te propongo
guardar» y «la bandeja, tras la reunión»; `honestidad.html`, mirada 4-ter):

1. **Propuestas:** la app te propone qué guardar (una cifra, un compromiso, un nombre) con **reglas
   publicadas**, y tú aceptas con una tecla. Proponer no es guardar: nada entra sin tu sí.
2. **La bandeja:** lo que no decidiste durante la reunión te espera después, en una ventana que
   eliges tú (defecto 3 h, techo 24 h, puede ser cero), con una cuenta atrás visible. Al llegar a
   cero **se borra sola aunque no abras la app**.

La regla dura 1 ya admite la bandeja «durante la ventana elegida, con borrado automático al vencer
aunque la app no se abra». Esta ADR decide cómo se cumple. Y la misma frase de la maqueta, dicha de
las notas guardadas («se borra sola al vencer aunque no abras la app»), sigue siendo falsa hasta
esta fase: hoy la app borra lo vencido al arrancar y cada hora mientras está abierta (ADR 015 §6).

No hay modelo en esta fase: **cero LLM nuevo**. Todo lo que sigue es código.

## Decisión

### 1 · Qué sabe reconocer: las reglas publicadas

El catálogo vive en `data/propuestas/reglas.json`, versionado y en español y en inglés, como el del
radar. Lo que enseña la pantalla («Qué sabe reconocer») **sale del catálogo**, así que la lista que
lees es la lista entera (test).

| Regla        | Qué reconoce                                                                                                                         | En qué turnos        |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------ | -------------------- |
| `cifra`      | cifras con unidad, plazos y fechas dichos en voz alta («12 semanas», «el 15 de octubre», «el viernes», «USD 40.000», «30 %»)         | los dos              |
| `compromiso` | compromisos tuyos («te lo mando», «quedamos en», «me comprometo», «I'll send», «we agreed»)                                          | solo los tuyos       |
| `choque`     | una cifra que contradice una ficha que fijaste: la misma cosa con otro número («cuatro fuentes» cuando tu ficha dice «tres fuentes») | los dos              |
| `nombre`     | nombres propios que no están en tu corpus                                                                                            | los dos              |
| `pregunta`   | una pregunta que te hicieron                                                                                                         | solo los del cliente |

`pregunta` es la «pregunta abierta» del plan del sprint. La maqueta aprobada enseña las cuatro
primeras; la quinta entra en su tabla como texto y va al bloque de textos del ⭐⭐.

**Límites, para que proponer no se convierta en ruido:** la misma propuesta no se repite en una
reunión, y como mucho hay 30 esperando; al llegar a 30, las nuevas no entran y la pantalla lo dice.

### 2 · Qué se escribe de cada lado

- **Lo tuyo** (micrófono, sin eco): la frase del turno en la que saltó la regla, como máximo 160
  caracteres. Es texto tuyo, con el mismo estatuto que tu nota.
- **Lo del cliente** (sistema, **o micrófono marcado como eco**, que es su voz): **jamás el turno**.
  Un hecho en una línea, hecho con una plantilla y un fragmento de **como mucho 8 palabras**:
  «Dijeron «12 semanas»», «Mencionaron a «Andrea Villalba», que no está en tu corpus»,
  «Dijeron «cuatro fuentes»; tu ficha fijada dice «tres»», «Te preguntaron: «…»». Test en rojo: el
  turno entero nunca aparece, y el fragmento no pasa de 8 palabras.
- **De dónde salió** va siempre con la propuesta: «lo dijiste tú · 14:16», «lo dijo el cliente ·
  14:21», «choca con una ficha que fijaste · §3.2», como en la maqueta.

### 3 · Durante la reunión

- Las propuestas aparecen en Notas, en «Te propongo guardar esto», y **la última, en la banda, como
  una línea pasiva** (mirada 20): no suena, no se lee en voz alta en el modo solo audio, no tapa la
  ficha y no toma el foco.
- **⌃⌥↵ guarda la última propuesta**, como dibuja la maqueta aprobada. En Notas, cada una tiene
  «Guardar» y «No».
- **Guardar** la mueve a **«Propuestas guardadas»**, que entra al archivo de la reunión como una
  lista aparte de tus acuerdos (tus acuerdos los escribes tú; estas las aceptaste). El archivo `.ghost`
  gana ese campo sin cambiar de versión: un archivo sin él se lee como una lista vacía (test).
- **No** la descarta: muere en ese momento.
- La propuesta llega **≤ 1 s** después del turno (kit): son expresiones regulares y una consulta al
  índice del corpus, sin red y sin modelo (test con reloj).

### 4 · Al cerrar: la bandeja

- **La ventana** es una preferencia tuya, como la retención: **al cerrar · 1 h · 3 h (de fábrica) ·
  fin del día · 24 h**. «Fin del día» es las 23:59 del día en que cierras. Se elige en «al cerrar»
  (mirada 20) y se puede cambiar desde la bandeja, siempre con el techo de 24 h desde el cierre.
- **Guardar cifrado y cerrar** con propuestas sin decidir: van a la bandeja con su vencimiento
  (cierre + ventana). Con la ventana **al cerrar**, no se escribe nada y mueren ahí (mirada 20).
- **Cerrar sin guardar** se lleva también las propuestas sin decidir: dijiste que no querías nada.
- **Una reunión sin nada tuyo pero con propuestas** ya no se cierra sola al parar (enmienda del ADR
  015 §7): te queda la decisión de qué hacer con ellas.
- **Dónde:** `~/Library/Application Support/com.aiapps.copiloto-consultor/bandeja/`, carpeta 700 y
  archivo 600, **no en Documentos**. Documentos puede estar en iCloud, y la papelera de iCloud
  («Eliminado recientemente») guarda 30 días lo que borras: una lista que promete morir a las 3 h
  no puede tener una copia que viva un mes, aunque esté cifrada.
- **El formato** es el `.ghost` v1 del ADR 015 §3, con la misma llave: el vencimiento va en la
  cabecera, en claro y autenticado. Un archivo por reunión, con el nombre del archivo de notas al
  que pertenece.
- **Guardar desde la bandeja** vuelve a sellar el archivo de la reunión con la propuesta añadida
  (mismo vencimiento) y la quita de la bandeja. **No** la quita y ya está. Cuando la bandeja se
  queda vacía, su archivo se borra.
- **Leer la bandeja:** justo después de cerrar está en memoria y no pide nada. Si la app se cerró
  entre medias y hay que leerla del disco, pide Touch ID como abrir una reunión (ADR 015 §5, una vez
  por sesión de la app).
- **⌥⎋** se lleva las propuestas sin decidir de la reunión en marcha (nueva pieza del corte:
  `Propuestas`), porque salen de la captura. Las guardadas se quedan, como tus notas. Las bandejas de
  reuniones ya cerradas no son captura y siguen su ventana; Honestidad las enseña.

### 5 · El vencimiento, aunque no abras la app

Código primero y por capas, cada una con su test:

1. **Al arrancar**, la app borra todo lo vencido: notas y bandeja (ya existe para notas).
2. **Mientras corre**, un reloj que despierta **en el próximo vencimiento** (como mucho, cada hora),
   no cada cinco minutos.
3. **Con la app cerrada, launchd** (decisión del usuario en el plan del sprint): una tarea
   programada **al minuto de cada vencimiento**. Entre vencimientos no corre nada; solo hay una hora
   anotada en launchd.

**La tarea** es `~/Library/LaunchAgents/com.aiapps.copiloto-consultor.vencimiento.plist`:

- `ProgramArguments`: `/bin/sh -c <el barrido>` sobre una lista, **no el binario de la app**, así
  que funciona aunque borres la app.
- `StartCalendarInterval`: una entrada `{Month, Day, Hour, Minute}` por cada vencimiento pendiente,
  **redondeado al minuto siguiente** («se borra en el minuto siguiente a su vencimiento»).
- `RunAtLoad`: una pasada al iniciar sesión, por si el Mac estaba apagado a esa hora.
- `AssociatedBundleIdentifiers`: `com.aiapps.copiloto-consultor`, para que Ajustes del Sistema la
  enseñe como Angel Ghost.
- Sin `StandardOutPath` ni `StandardErrorPath`: no escribe nada.

**La lista** es `~/Library/Application Support/com.aiapps.copiloto-consultor/vencimientos` (600): una
línea por archivo, «segundos · ruta». La app la **regenera** a partir de las cabeceras de los
archivos, que son la fuente de verdad, cada vez que guarda, borra o barre. El barrido la **lee y no
la escribe**, así que la app y launchd no se pisan.

**El barrido**, entero:

```sh
ahora=$(/bin/date +%s)
[ -f "$1" ] || exit 0
while IFS='	' read -r vence ruta; do
  case "$ruta" in *.ghost) ;; *) continue ;; esac
  [ "$vence" -le "$ahora" ] 2>/dev/null && /bin/rm -f -- "$ruta"
done < "$1"
```

Solo borra archivos `.ghost`, aunque alguien escriba otra cosa en la lista. Una línea de un archivo
que ya no existe no hace nada.

**Lo que launchd hace y lo que no:**

- `StartCalendarInterval` no tiene año: una entrada vuelve a saltar el mismo día del año siguiente.
  No pasa nada, porque el barrido mira la hora de verdad; la app quita las entradas pasadas en cuanto
  corre.
- Si el Mac duerme a esa hora, la tarea corre al despertar. Si está apagado, al iniciar sesión
  (`RunAtLoad`).
- **Registrar:** `/bin/launchctl bootout` y `bootstrap` sobre `gui/<uid>`, desde `vencimiento/` y
  desde ningún otro sitio: es el **segundo programa** que lanza la app, y los gates que cuentan
  programas lo declaran. Si no queda nada con vencimiento, la app quita la tarea y la lista.
- La primera vez, macOS avisa una vez de que se añadió un ítem en segundo plano. Lo dice el manual.
- **Si desactivas la tarea** en Ajustes del Sistema → Ítems de inicio, launchd no la corre. La app
  no adivina si está desactivada: **lo mide**. Si al arrancar encuentra algo que venció hace más de
  2 minutos con la tarea instalada, es que no corrió, y Honestidad lo dice («la tarea de borrado no
  corrió: revisa Ítems de inicio»). Lo borra igual en ese momento.
- **Si borras la app**, la tarea sigue borrando a su hora lo que quede. Cuando ya no queda nada, se
  queda como una entrada inerte en Ítems de inicio que puedes quitar. Lo dice el manual.

**La prueba en vivo** usa una ventana de 2 minutos que existe solo en la compilación de desarrollo
(`debug_assertions`). No es un gate: es cómo el constructor y la parada del ⭐⭐ ven a launchd borrar
un archivo con la app cerrada.

### 6 · Honestidad

- **Mientras haya bandeja:** la franja «Y una tercera cosa, que no muere al instante: la bandeja» de
  la maqueta aprobada, con la cuenta atrás de verdad.
- **Si la tarea no corrió:** el aviso del punto 5.

### 7 · Dónde vive el código

| Módulo         | Qué hace                                                        | Toca disco                                        |
| -------------- | --------------------------------------------------------------- | ------------------------------------------------- |
| `propuestas/`  | el catálogo y las reglas: de un turno, sus propuestas. Puro     | **No**, entra en `PROTEGIDOS`                     |
| `notas/`       | las propuestas esperando y las guardadas, en el cuaderno        | **No**, ya es protegido                           |
| `bandeja.rs`   | sella, lee y borra la bandeja                                   | Sí, y solo ve propuestas ya reducidas a una línea |
| `vencimiento/` | la lista, la tarea (el plist, generado de forma pura) y launchd | Sí, y lanza `/bin/launchctl`                      |

`propuestas/` recibe turnos del cliente, así que **no puede** escribir (el patrón del diccionario,
ADR 002 enmienda 1).

### 8 · Lo que se registra en el log

Cuántas propuestas, cuántas guardadas, cuántas a la bandeja, cuántos archivos vencidos. **Ni una
propuesta, ni un nombre de archivo, ni una ruta.**

### 9 · Lo que la maqueta aprobada dice y esta fase no construye

«Si enciendes el modelo local, solo **redacta mejor** la propuesta — nunca decide cuál merece
guardarse.» En H1 el modelo **no toca** las propuestas (cero LLM nuevo). La pantalla dice solo «Son
reglas, no un modelo adivinando…», y la frase del modelo no se construye. Va al bloque de textos.

## Alternativas consideradas

- **Que un modelo decida qué proponer:** regla 14 de la app, y la maqueta dice «son reglas». Una
  regla se puede leer entera; un modelo, no.
- **Bandeja solo en memoria**, que muere al cerrar la app: más privada, pero la maqueta aprobada
  promete que la bandeja te espera su ventana, y cerrar la app no es decidir.
- **Revisar cada 5 minutos** con una tarea residente (`StartInterval`): el usuario la rechazó en el
  plan. Entre vencimientos no hay nada que hacer.
- **Borrar solo al abrir la app:** rompe la promesa «aunque no abras la app».
- **Que la tarea lance el binario de la app:** si borras la app, deja de borrar. `/bin/sh` siempre
  está.
- **El vencimiento solo en la lista:** la lista es una copia. La fuente es la cabecera autenticada de
  cada archivo; la lista se regenera de ella.
- **La bandeja en Documentos, junto a las notas:** la papelera de iCloud guarda 30 días.

## Consecuencias

- `Permitido` del efímero en marcha gana la carpeta de la bandeja, la lista y el plist. La canaria se
  busca en la bandeja **descifrada**, y lo que se comprueba es que **el turno del cliente nunca
  está**: un fragmento suyo de 8 palabras sí puede estar, porque es la bandeja que la regla dura 1
  admite.
- `verify:ephemeral` estático gana `src-tauri/src/propuestas`.
- `contador-de-red` y el gate del radar pasan a contar dos programas: `/usr/bin/profiles` desde el
  radar y `/bin/launchctl` desde `vencimiento/`. Rojo con un tercero.
- El corte pasa a 11 piezas (`Propuestas`).
- ADR 002, enmienda 4: la bandeja, la lista y la tarea de launchd.
- ADR 015 §7, enmendado: una reunión con propuestas sin decidir no se cierra sola.
- Contrato (regla 19): la vista del cuaderno gana las propuestas; la bandeja entra como tipo nuevo.
- El manual quita la limitación «hoy lo vencido se borra al abrir la app» y dice lo que hace launchd,
  cómo se ve en Ítems de inicio y qué pasa si la desactivas.
- Parada del ⭐⭐: una bandeja de 2 minutos que desaparece con la app cerrada.
