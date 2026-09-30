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
  «Dijeron «cuatro fuentes»; tu ficha fijada dice «tres»». De una pregunta, **ni un fragmento**: sus
  palabras clave, como mucho cinco («limpieza · datos · dentro · alcance»). Test en rojo: en un turno
  de más de 8 palabras, el turno nunca aparece, y ningún fragmento pasa de 8 palabras. (Un turno de
  ocho palabras o menos puede coincidir con su propio hecho: «Doce semanas.»)
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
- La propuesta llega **≤ 1 s** después del turno (kit): son comparaciones de palabras contra el
  catálogo y una consulta al índice del corpus, sin red y sin modelo (test con reloj).

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
  entre medias y hay que leerla del disco, pide Touch ID como exportar una reunión (ADR 015 §5 y su enmienda 3, una vez
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

## Hallazgo en vivo (2026-09-27): launchd no puede entrar en Documentos

La prueba en vivo del punto 5 (`en_vivo_launchd_borra_a_su_hora_sin_la_app`) registró la tarea con
dos archivos que vencían a la vez:

- el de una carpeta temporal lo borró **38 s después de vencer**, en el minuto siguiente, sin nada de
  la app corriendo;
- el de `~/Documents/Angel Ghost/` **no lo borró**.

Una tarea de diagnóstico lo confirma: desde launchd, `/bin/sh` recibe `ls: ~/Documents: Operation not
permitted`, y `~/Library/Application Support` lo lee sin problema. Es la protección de Documentos de
macOS (TCC): el permiso es de la app, no de `sh`.

**Y cómo la ve el usuario:** la tarea de prueba apareció en Ajustes del Sistema → General → Ítems de
inicio como **«sh · Item from unidentified developer»**, aunque su plist traía
`AssociatedBundleIdentifiers` de la app. Esa asociación solo la respeta macOS con una app firmada, y en
desarrollo no lo está. Con la app firmada y notarizada debería verse como Angel Ghost, pero **no está
verificado** hasta tener una compilación firmada. El usuario la vio y preguntó si era nuestra: una app
cuya promesa es la privacidad no puede aparecer como un `sh` anónimo.

**Consecuencia:** tal como está, launchd cumple la promesa para **la bandeja** (Application Support)
y **no** para **las notas** (Documentos). Con las notas en Documentos, lo vencido se borra al abrir la
app y cada hora mientras corre, que es lo que el manual ya dice. **La decisión de cómo se cierra la
brecha es del usuario**, porque la carpeta de Documentos fue suya (plan del sprint), y queda
registrada aquí cuando la tome.

## Decisión del usuario (2026-09-27): la A

Las tres salidas que se le dieron:

| | Dónde viven tus notas | Qué pasa al vencer con la app cerrada |
|---|---|---|
| **A** | En la carpeta privada de la app, con «Mostrar en Finder» | Las borra la misma tarea que borra la bandeja |
| **B** | En Documentos | La tarea abre la app sin ventana para que ella las borre; si borras la app, dejan de borrarse |
| **C** | En Documentos | Se borran al abrir la app; cambia lo que prometen el manual y Honestidad |

**Eligió la A** («Sí la A»). Queda en la enmienda 1 del ADR 015. Aquí cambia:
- `lo_que_vence` (`reunion.rs`) pasa a ser **tus notas y la bandeja**; las de «siempre» no entran.
  Guardar y «Borrar ahora» ponen la tarea al día.
- La prueba en vivo `en_vivo_launchd_borra_a_su_hora_sin_la_app` ya **no entra en Documentos**: su
  segundo archivo va en la carpeta de notas de la app. Sigue siendo `#[ignore]` y solo se corre con el
  «sí» del usuario (regla 22).
- **Lo que no cambia y se vigila en vivo:** la tarea sigue apareciendo en Ítems de inicio, y ahora
  vive mientras tengas notas con fecha, no solo mientras haya bandeja. Hay que ver si volver a
  registrarla tras cada reunión hace que macOS vuelva a avisar; lo dirá la corrida en vivo con la app.

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
- **La bandeja en Documentos, junto a las notas de entonces:** la papelera de iCloud guarda 30 días.
  Con la decisión A, las notas se vinieron con ella.

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

## Enmienda 2 (2026-09-28) — la bandeja, fuera de Time Machine; y su nombre, libre en las dos carpetas

Dos hallazgos de la auditoría independiente del S3 tocan el §4.

**M2 · Time Machine.** El §4 sacó la bandeja de Documentos con este argumento: «una lista que promete
morir a las 3 h no puede tener una copia que viva un mes». Pero `~/Library/Application Support` entra en
Time Machine, y nada marcaba `bandeja/` como excluida: la copia horaria la hacía vivir semanas en el disco
de copias, y con la llave en el mismo llavero (ADR 015, enmienda 2).
- **Decisión:** `Bandeja::dejar` marca su carpeta con el atributo estándar «fuera de las copias»
  (`URLResourceValues.isExcludedFromBackup`, `nativo/Copias.swift` → `almacen::fuera_de_las_copias`). No
  pide permiso ni contraseña, y es idempotente. Si macOS no deja marcarla, se dice en el log y la bandeja
  se queda.
- **Tus notas siguen entrando en las copias** (decisión del usuario, 2026-09-28: «Notas en Time Machine:
  dentro»): son tuyas, están hechas para durar, y si el Mac se estropea se recuperan de la copia. El
  manual lo dice.
- **Lo que no cubre, dicho:** las instantáneas locales de Time Machine (≤ 24 h) son del volumen entero y
  sí ven la bandeja hasta que se reciclan.
- `verify:ephemeral` prohíbe desde ahora `setResourceValues` en los protegidos: la única línea que lo usa
  lleva su marca y este ADR.
- **Test:** `la_bandeja_queda_fuera_de_las_copias` (`bandeja.rs`, solo con el puente de Swift): tras
  `dejar`, la carpeta lleva `com.apple.metadata:com_apple_backup_excludeItem`. Nació en rojo sin la llamada.
  En la prueba en vivo, `tmutil isexcluded` sobre `bandeja/` tiene que decir `[Excluded]`.

**A1 · Dos reuniones del mismo cliente el mismo día.** El §4 dice que la bandeja «se llama como su
reunión». El nombre lo elegía la carpeta de notas mirando solo `notas/`: una reunión sin nota deja bandeja
sin archivo de notas, y la siguiente del mismo cliente y día tomaba su nombre y la pisaba.
- **Decisión:** el nombre se busca libre en `notas/` **y** en `bandeja/` (`carpeta::nombre_libre`), y
  `Bandeja::dejar` no pisa jamás: si el archivo existe, es un error.
- **Tests:** `dos_reuniones_del_mismo_cliente_el_mismo_dia_no_se_pisan` (`reunion.rs`) y
  `dejar_no_pisa_una_bandeja_que_ya_existe` (`bandeja.rs`), los dos nacidos en rojo.

## Enmienda 3 (2026-09-28) — la lista que lees sale del catálogo, y un test lo comprueba (auditoría del S3, M11)

El §1 decía que lo que enseña la pantalla «sale del catálogo… (test)». No era así: la pantalla pinta
`sonReglas`, una frase escrita a mano en el diccionario (que tiene que ser fiel a la maqueta);
`catalogo::reglas()` no tenía llamador, y el único test comparaba los ids del catálogo con el `enum`.
- Cada regla de `data/propuestas/reglas.json` lleva ahora `corto` (es/en): su trozo exacto de la frase.
- `tests/unit/reglas-publicadas.test.ts` exige que `sonReglas`, en cada idioma, sea exactamente el prefijo
  más los `corto` en el orden del catálogo, unidos con « · » y con punto final. Nació en rojo (sin
  `corto`); si una regla se añade, se quita o se renombra sin la frase, es rojo.
- `catalogo::reglas()` se borró: nadie la llamaba.

