# ADR 015 — Las notas y su cifrado

- **Fecha:** 2026-09-27
- **Sprint:** 003 «El cuaderno y el cierre», fase 1
- **Estado:** aceptada. Tres puntos de FORMA esperan el veredicto de la mirada 19 y se marcan
  «(mirada 19)»; si la mirada los cambia, esta ADR se enmienda antes de construir su pantalla.

## Contexto

C9 es la mitad de la promesa que hasta ahora no existía: **lo tuyo queda**. La regla dura 1 dice qué
puede persistir (notas y acuerdos escritos, fichas fijadas, tus propios turnos en texto si lo
enciendes) y cómo (cifrado, con retención y borrado, en tu carpeta). La maqueta aprobada lo dibuja en
`notas.html` (durante · al cerrar · el archivo), y el usuario decidió en el plan del sprint tres
cosas que esta ADR convierte en código:

1. la carpeta es **`~/Documentos/Angel Ghost/`**, como en la maqueta, y no la carpeta oculta de la
   app que decía el plan de la planeadora;
2. el vencimiento lo cumple macOS aunque la app no se abra (ADR 016, fase 2);
3. el cuaderno se **protege mientras hay sesión** (punto 10).

## Decisión

### 1 · Qué entra en el archivo, y qué no

| Entra | No entra, nunca |
|---|---|
| **tu nota** (texto libre) | la voz de nadie, **tampoco la tuya** |
| **los acuerdos** que escribes tú (la app no decide qué fue un acuerdo) | los turnos del cliente |
| **las fichas que fijaste** (titular, fuente y unidad; no el texto del documento) | lo leído de la pantalla |
| **tus turnos en texto**, solo si enciendes «Conservar mis turnos» (nace apagado) | las fichas que salieron solas |
| cuándo empezó, cuánto duró y, desde la fase 3, el cliente que elijas en «Este cliente» | las sugerencias, lo que salió al API, el radar |

**Las fichas que salieron solas no se guardan, y es a propósito.** El disparador las eligió por las
palabras del cliente: la lista de lo que apareció es la huella de lo que dijo. Fijar una es un acto
tuyo y entra; que aparezca, no. La maqueta aprobada («al cerrar») ya solo lista las fijadas.

**Tus turnos sin eco.** Con altavoces, el micrófono oye al cliente y el mismo turno llega por las dos
pistas; la app lo marca como eco desde el sprint 001. Un turno del micrófono marcado como eco **es la
voz del cliente** y no entra jamás, aunque la casilla esté encendida (test en rojo).

### 2 · Dónde y con qué nombre

> **Enmendado el mismo día (enmienda 1, al final):** el usuario eligió la **A** y las notas pasaron a
> la carpeta privada de la app. Lo que sigue sobre Documentos, iCloud y «elegir otra carpeta» queda
> como historia de la decisión, no como lo que hace la app.

- Carpeta: `~/Documents/Angel Ghost/` (Finder la enseña como «Documentos»). **700**; el archivo,
  **600**, y los dos nacen así: los escribe `almacen.rs` (ADR 002, enmienda 2).
- Nombre: `<cliente>-<AAAA-MM-DD>.ghost`, con el cliente en minúsculas, sin tildes y con guiones
  (`paramo-azul-2026-09-20.ghost`). Sin cliente elegido —hasta la fase 3, siempre—:
  `reunion-<AAAA-MM-DD>-<HHMM>.ghost`. Si el nombre ya existe: `-2`, `-3`…
- **El nombre va en claro y lo dice todo menos el contenido:** con quién y qué día. Es lo que dibuja
  la maqueta y lo que hace falta para encontrar una reunión sin abrirla; queda declarado en el manual.
- **Si macOS niega el acceso a Documentos**, guardar falla con ese motivo: la app lo dice, **no pierde
  la nota** (sigue en memoria) y ofrece elegir otra carpeta con el diálogo del sistema. La carpeta
  elegida se recuerda en tus preferencias. Nunca se pierde en silencio.
- `Info.plist` gana `NSDocumentsFolderUsageDescription`, en español y en inglés (regla dura 7).

### 3 · El formato `.ghost` (versión 1)

```
 6 B  «AGHOST»             ─┐
 1 B  versión = 1           ├─ cabecera EN CLARO, autenticada (datos asociados del AEAD)
 8 B  vence (i64, BE, s)   ─┘  0 = «siempre»
24 B  nonce aleatorio
 N B  XChaCha20-Poly1305( JSON del contenido ) + 16 B de etiqueta
```

- **XChaCha20-Poly1305** (`chacha20poly1305` de RustCrypto): cifrado autenticado, y con un nonce de
  24 bytes que se puede sacar al azar sin llevar la cuenta de cuántos se usaron.
- **El vencimiento va en claro para que el borrado no necesite la llave**: la app al arrancar y la
  tarea de launchd de la fase 2 saben qué borrar sin abrir nada. Y va **autenticado**: cambiarle la
  fecha a un archivo hace que no se pueda abrir, así que alargarle la vida por fuera lo inutiliza.
- Un archivo manipulado, truncado o de otra versión **no se abre y lo dice**; nunca se entrega medio
  contenido.

### 4 · La llave

- 32 bytes aleatorios del sistema, creados al **primer guardado**. Una sola llave para todas tus
  reuniones.
- Viven en el Llavero de macOS, servicio **«Angel Ghost · notas»**, con
  `kSecAttrAccessibleWhenUnlockedThisDeviceOnly`: se leen solo con tu sesión abierta y **no salen de
  este Mac**, ni por copia de seguridad ni por iCloud. Sin contraseña que recordar y sin archivo de
  llave.
- **Consecuencia que se dice, no se arregla:** si borras el Llavero o cambias de Mac, tus notas
  guardadas no se pueden abrir. No hay recuperación, porque cualquier recuperación sería una segunda
  llave. Lo dice el manual.
- **Si Documentos está en iCloud**, viaja una copia del archivo cifrado. Nadie puede abrirla fuera de
  este Mac, porque la llave no viaja. Lo dicen el manual y Honestidad.

### 5 · Guardar no pide nada; abrir y exportar, sí

- **Guardar** usa la llave sin preguntarte: al cerrar una reunión no hay que pararse a desbloquear.
- **Abrir o exportar** una reunión guardada pide **Touch ID o la contraseña del Mac**
  (LocalAuthentication, `deviceOwnerAuthentication`) **una vez por sesión de la app**; al salir de la
  app se olvida.
- **Qué protege:** a alguien frente a tu Mac desbloqueado que abre Angel Ghost para leer tus
  reuniones. **Qué no:** un programa que corre como tú puede copiar el archivo, pero sin la llave no
  lo lee; y el Llavero le pide permiso al sistema a cualquier app que no sea la que creó la llave.

### 6 · Retención (mirada 19)

- Es **una elección tuya para todas las reuniones**, guardada en tus preferencias: 7 d · 30 d ·
  **90 d** (de fábrica, como la maqueta) · 1 año · siempre. «Nunca» no es una retención: es «Cerrar sin
  guardar».
- Al guardar, el archivo se estampa con **su** vencimiento (guardado + retención). Cambiar la
  retención vale para las reuniones que guardes desde entonces; las que ya estaban conservan la suya.
- **Se cumple sola:** la app borra lo vencido al arrancar y cada hora mientras corre (esta fase); con
  la app cerrada, la tarea de launchd del ADR 016 (fase 2). «Borrar ahora» borra un archivo al
  momento.

### 7 · Al cerrar la sesión, al salir y con ⌥⎋

- **Al terminar la sesión:** «Guardar cifrado y cerrar» o «Cerrar sin guardar», como la maqueta. Si
  no hay nada tuyo (ni nota, ni acuerdos, ni fijadas, ni turnos), no hay archivo **ni pregunta**: la
  reunión se cierra sola al parar, y el cuaderno deja de estar protegido.
- **Si sales de la app con notas sin guardar**, se guardan cifradas: perder lo que escribiste por
  cerrar la ventana equivocada es peor que guardarlo. La única forma de tirarlas es decirlo: «Cerrar
  sin guardar».
- **Si la app se cae**, lo no guardado se pierde. Está en memoria a propósito; lo dice el manual.
- **⌥⎋ corta la captura, no tus notas:** la nota, los acuerdos y las fijadas **sobreviven** (la
  maqueta dice «Tus notas siguen ahí»). **Tus turnos mueren**: salen de la captura, y el corte vacía
  todo lo que la captura produjo. Es una pieza nueva del corte, con su test.

### 8 · Dónde vive el código

| Módulo | Qué hace | Toca disco |
|---|---|---|
| `notas/` | el cuaderno en memoria, el filtro de tus turnos (sin eco), el contenido del archivo y **el cifrado**, todo puro | **No** — entra en `PROTEGIDOS` |
| `carpeta.rs` | guarda, lista, abre, exporta, borra y barre lo vencido; pide la llave al Llavero | Sí — no ve un solo turno del cliente |

Es el patrón del diccionario (ADR 002, enmienda 1) y se elige por la misma razón: `notas/` recibe los
turnos del micrófono, y con ellos los marcados como eco, que son la voz del cliente. El módulo que
tiene eso en las manos **no tiene manera** de escribirlo.

### 9 · Lo que se registra en el log

Metadatos y nada más: cuántos acuerdos, cuántas fijadas, cuántos turnos y cuántos bytes. **Ni el
texto, ni el nombre del archivo** (lleva el del cliente), ni la ruta. La canaria del log lo vigila
para la voz del cliente; para tus notas, el test de `carpeta.rs` comprueba que el mensaje de guardado
no lleva ni el nombre del archivo ni una palabra de la nota.

### 10 · El cuaderno, protegido mientras hay sesión

- Al empezar a escuchar, la ventana principal toma el mismo flag que la banda (`content_protected`,
  que en macOS es `NSWindowSharingNone`), y **lo suelta cuando la reunión se guarda o se descarta**,
  no al terminar la sesión. Mientras escuchas, tu nota está ahí: si compartes la pantalla entera por
  descuido, el cliente ve un rectángulo negro en vez de tu cuaderno.
- **Por qué no se suelta con ⌥⎋ ni al parar de escuchar** (lo decidió el constructor al escribir el
  código, en la dirección de proteger más, y se declara): ⌥⎋ se pulsa, muchas veces, justo antes de
  compartir pantalla, y tus notas **sobreviven** al corte. Soltar la protección en ese instante
  destaparía el cuaderno cuando más importa. La decisión del usuario («protegerlo mientras hay
  sesión») queda cubierta entera y se alarga hasta que la reunión se cierra de verdad.
- **El invariante del sprint 001 cambia de forma, no de fondo:** era «exactamente una ventana
  protegida, y es la banda»; pasa a ser **«la banda siempre, el cuaderno mientras hay sesión, el
  relleno jamás»** (con «sesión» leído como «reunión abierta», arriba). La parte fija sigue viviendo solo en `tauri.conf.json` (banda sí, las otras dos
  no). La parte que cambia pasa por **una sola función**, `ventana::proteger_el_cuaderno`, que solo
  sabe tocar la principal: el riesgo nº 1 del S1 (un relleno protegido que vuelve a enseñar lo que hay
  detrás) sigue sin camino. Test en rojo.
- La promesa es la misma que la de la banda y está igual de graduada: verificada en Meet, sin
  verificar en Zoom y Teams. Es una parada del ⭐⭐.

## Alternativas consideradas

- **Cifrar con una contraseña tuya (derivada con Argon2):** una contraseña más que recordar, y que si
  se olvida pierde lo mismo que perder el Llavero. El Llavero ya está desbloqueado cuando tu sesión lo
  está, y `ThisDeviceOnly` es lo que hace imposible abrir la copia de iCloud.
- **Pedir Touch ID también al guardar:** cierra la reunión con un diálogo en el peor momento, y
  protege lo contrario de lo que importa: escribir tus propias notas no es el riesgo, leerlas otro sí.
- **Un archivo por nota, o una base de datos:** la maqueta dibuja un archivo por reunión, y un archivo
  por reunión es lo que se puede borrar, exportar y vencer de una vez.
- **Guardar las fichas que salieron solas:** ver el punto 1.
- **Autoguardado cifrado mientras dura la reunión** (para sobrevivir a una caída): escribiría a disco
  durante la sesión, que es justo lo que el efímero en marcha vigila, y abriría una ventana en la que
  un archivo a medias existe. Se deja fuera y la caída queda declarada.

## Consecuencias

- `Permitido` del efímero en marcha gana **la carpeta de notas**, y la canaria se busca **en el
  contenido descifrado**: comprobarla sobre bytes cifrados no probaría nada.
- `verify:ephemeral` estático gana `src-tauri/src/notas`.
- ADR 002 se enmienda (tercera): `notas/` no toca disco; lo hace `carpeta.rs`.
- Se añade una dependencia: `chacha20poly1305` 0.11 (RustCrypto), con `zeroize`. `cargo audit` la
  cubre desde este commit.
- El rendimiento prometido: abrir un archivo cifrado en **≤ 500 ms** (plan del sprint), con test.

## Enmienda 1 (2026-09-27) — tus notas, en la carpeta privada de la app (decisión A)

**Qué pasó.** La prueba en vivo de launchd (ADR 016, «Hallazgo en vivo») mostró que el `sh` que borra
lo vencido con la app cerrada **no puede entrar en Documentos**: el permiso es de la app, no suyo. Con
las notas en `~/Documents/Angel Ghost/`, la retención solo se cumplía al abrir la app. La carpeta había
sido decisión del usuario, así que se le preguntó con tres salidas:

- **A:** mover las notas a la carpeta privada de la app;
- **B:** dejarlas en Documentos y que la tarea abra la app para borrarlas;
- **C:** dejarlas en Documentos y cambiar la promesa.

**Eligió la A** («Sí la A», 2026-09-27).

**Qué cambia:**
- **Dónde:** `~/Library/Application Support/com.aiapps.copiloto-consultor/notas/`, junto a la bandeja.
  700 la carpeta y 600 cada archivo, como antes. El nombre y el formato `.ghost` no cambian.
- **La pantalla la nombra, no la escribe:** «Carpeta privada de la app», con **«Mostrar en Finder»**,
  que la abre con la reunión que estás viendo seleccionada. Lo hace macOS (`NSWorkspace`), sin lanzar
  ningún programa.
- **La retención se cumple aunque no abras la app:** tus notas entran en la lista de launchd, como la
  bandeja. Las guardadas para «siempre» no entran.
- **Fuera:** `NSDocumentsFolderUsageDescription` (la app ya no pide Documentos), la preferencia
  `carpetaDeNotas`, el comando `elegir_carpeta_de_notas` y el camino «elegir otra carpeta». Si guardar
  falla —el Llavero no contesta, el disco está lleno—, la reunión sigue abierta, la nota entera, y la
  salida es «Intentar otra vez».
- **iCloud deja de aplicar:** la carpeta de la app no se sincroniza, así que no viaja ninguna copia ni
  queda nada 30 días en «Eliminado recientemente». Exportar a texto sigue yendo donde tú elijas.
- **No hay migración:** la app no se ha publicado y nadie tiene notas en Documentos.

**Lo que cuesta, dicho:**
- Tus notas no se ven al abrir Documentos: se llega a ellas desde la app, o con «Mostrar en Finder».
- **La tarea de borrado vive mientras tengas notas con fecha**, no solo mientras haya bandeja: con la
  retención de fábrica (90 d), queda en Ítems de inicio casi siempre. En desarrollo se ve como «sh ·
  desarrollador no identificado», porque la app no está firmada; firmada, debería verse como Angel
  Ghost, y eso no está verificado todavía.

**Tests:**
- `launchd_se_lleva_tus_notas_y_la_bandeja` (`reunion.rs`), en rojo con la lista de antes (solo la
  bandeja);
- la sesión completa del efímero guarda las notas con 90 d y exige que la lista traiga notas y bandeja;
- `un_campo_que_ya_no_existe_se_ignora` (`prefs.rs`): un `prefs.json` de la fase 1 con `carpetaDeNotas`
  se sigue leyendo. En rojo con `deny_unknown_fields` en el archivo;
- `lo-que-macos-dira`: la clave de Documentos ya no está, y el gate exige que tampoco esté en los
  `InfoPlist.strings`.
