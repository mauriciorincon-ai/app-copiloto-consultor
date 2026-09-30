# ADR 018 — La puerta local para tu agente

- **Fecha:** 2026-09-27
- **Sprint:** 003 «El cuaderno y el cierre», fase 4
- **Estado:** aceptada. La vista de la puerta en IA es FORMA nueva: se construye con lo maquetado
  (`ia.html`, estados «sprint 3 · la puerta») y queda «maquetado, no visto» para el gate del MVP
  (decisión del usuario del 2026-09-27: solo abre parada lo que cambia una decisión suya, la promesa
  del producto o toca su Mac). **Lo que toca su Mac —el diálogo del Llavero la primera vez que el
  agente usa la puerta— se enseña en una fila de la regla 22 antes de probarlo en vivo.**

## Contexto

C16: que **Claude Code opere la app desde este mismo Mac**. Lo pidió el usuario en la Etapa de
Diseño, y `ia.html` «Claude Code» lo dibuja con su promesa entera:

- **Lo que puede hacer:** buscar en tu corpus y devolver fichas · reindexar · correr el kit de
  evaluación · leer y cambiar preferencias · abrir tus notas guardadas («te pide desbloquear una vez
  por sesión»; desde la enmienda 1, **una vez por apertura**).
- **Lo que no puede hacer nunca:** tocar una reunión en curso («en reunión la puerta se cierra
  sola») · encender el API externo · sacar nada a la red · abrirse sola · tocar una máquina que no sea
  la tuya.
- **«Qué hizo tu agente»**, con lo denegado también registrado.

La orden la resume en una línea: `ghost` con llave en el Llavero, cerrada por defecto y cerrada sola
en reunión, red 0 con la puerta abierta, registro con lo denegado.

La verificación de supuestos del plan encontró dos cosas que esta ADR resuelve:

1. **Ningún gate mira los sockets Unix.** El contador de red busca `std::net`, `TcpStream`,
   `UdpSocket` y los clientes HTTP; un `std::os::unix::net::UnixListener` pasa todos los barridos sin
   que nadie lo vea. La puerta es el primer socket del producto, así que nace con su gate.
2. **El estado de la reunión se consulta, no se avisa.** No hay un evento «empezó la reunión» al que
   colgarse: la puerta tiene que preguntar ella.

**Regla dura 9** (cero acceso a máquinas ajenas) y **regla dura 2** (nada crudo sale del equipo)
aplican a la puerta igual que al resto. **La regla 21 de la casa no aplica**: la app no invoca a
Claude Code; es Claude Code, en la sesión del usuario, quien llama a la app.

**Cero LLM nuevo.** La puerta es código: un socket, una lista cerrada de órdenes y una política.

## Decisión

### 1 · Un socket Unix en la carpeta privada de la app, y solo mientras está abierta

`~/Library/Application Support/com.aiapps.copiloto-consultor/puerta.sock`.

- **Solo existe con la puerta abierta.** Al cerrarla se borra; si la app se cayó con la puerta
  abierta, al arrancar se borra el que quedó, y el token del Llavero con él.
- **La carpeta es 700** (`almacen`) **y el socket, 600.** Un socket Unix en una carpeta 700 solo lo
  alcanza el propio usuario del Mac: ni otra cuenta del Mac ni, por construcción, otra máquina. Esa
  es la regla dura 9 hecha estructura, no disciplina.
- **Por qué no un puerto TCP en `localhost`:** lo alcanza cualquier proceso de cualquier cuenta del
  Mac, se confunde con un servidor, y abriría la primera excepción a «ningún archivo de Rust abre un
  socket» del contador de red. El socket Unix no sale de la máquina porque no sabe hacerlo.
- **La ruta de un socket en macOS no pasa de 104 bytes.** Con un nombre de usuario muy largo no
  cabría: la puerta no se abre y lo dice, en vez de abrirse en otra carpeta menos privada.

### 2 · La llave: un token por apertura, en el Llavero

Cada vez que abres la puerta, la app saca **32 bytes del generador del sistema**, los guarda en
memoria y en el Llavero («Angel Ghost · puerta», cuenta `token`). `ghost` lo lee del Llavero y lo
manda con cada orden; la app lo compara **en tiempo constante**. Al cerrar la puerta, el token se
borra del Llavero y de la memoria.

- **El diálogo de macOS es la llave.** La entrada del Llavero la crea la app, así que la app la usa
  sin preguntar; `ghost` es otro programa, y la primera vez que la lee macOS pregunta: **«ghost quiere
  usar tu información confidencial guardada en "Angel Ghost · puerta" en tu llavero»**, con la
  contraseña del Mac. Como el token se crea de nuevo en cada apertura, **la pregunta vuelve una vez
  por apertura**: abrir la puerta es un gesto tuyo en la app, y dejar entrar al agente es otro, en un
  diálogo del sistema que ves.
- **Por qué no un archivo 600:** cualquier proceso del usuario lo lee en silencio. El Llavero
  pregunta, y esa pregunta es exactamente lo que el usuario pidió en la regla 22: enterarse.
- **`ghost` mira el socket ANTES de tocar el Llavero.** Con la puerta cerrada contesta «la puerta
  está cerrada» sin pedirle nada a macOS: ningún diálogo por una orden que no iba a entrar.

### 3 · El protocolo: una línea de ida y una de vuelta

Una conexión, una orden: una línea JSON (`{"token": …, "orden": {…}}`) y una línea JSON de
respuesta (`hecho` con sus datos, `denegado` con su motivo, o `fallo` con el error). Tope de 1 MiB
por línea y 5 s para recibirla: una orden a medias no deja la puerta colgada. Se atienden de una en
una.

### 4 · Lo que puede hacer: una lista cerrada

Es un `enum`: lo que no está en él no se interpreta, se deniega como «orden desconocida».

| Orden                                 | Qué hace                                                               | Nota                                                                                               |
| ------------------------------------- | ---------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `ghost corpus buscar <texto>`         | las fichas de tu corpus, como la banda                                 | el corpus que la app tiene indexado ahora                                                          |
| `ghost corpus reindexar`              | vuelve a leer tu carpeta y reintenta los ilegibles                     | la carpeta la señalas tú en Corpus                                                                 |
| `ghost kit <preguntas.json>`          | nDCG@5 y lo que no encuentra, contra tu corpus                         | `ghost` lee el archivo y manda las preguntas: la app no abre rutas que le dicte el agente          |
| `ghost prefs leer`                    | tus preferencias                                                       |                                                                                                    |
| `ghost prefs cambiar <clave> <valor>` | idiomas de pista, retención, ventana de la bandeja, lectura automática | lista blanca (§5)                                                                                  |
| `ghost notas listar`                  | tus reuniones guardadas                                                | nombre, fecha y vencimiento                                                                        |
| `ghost notas abrir <archivo>`         | una reunión guardada                                                   | **pide el desbloqueo en el Mac** (Touch ID o contraseña) **una vez por apertura de la puerta**, con su propio desbloqueo: el de la pantalla no le sirve (enmienda 1, M4) |

`corpus::evaluar` saca el nDCG@5 del test de integración y lo hace producto: el test y la puerta
miden con el mismo código.

### 5 · Lo que no puede hacer nunca

- **Nada en reunión.** Hay reunión si la app escucha, si hay una reunión en solo notas, si el
  detector ve una videollamada **o si no puede saberlo** (un navegador abierto sin permiso de
  Accesibilidad): lo que no se sabe se trata como reunión. Toda orden en reunión se deniega **y la
  puerta se cierra sola**. Además, mientras está abierta, un vigía mira cada 2 s y la cierra en cuanto
  aparece una reunión, aunque nadie llame; y «Iniciar sesión» y «Solo notas» la cierran al empezar.
  **No se vuelve a abrir sola:** la abres tú, en IA, cuando termina.
- **Encender el API externo.** La orden existe (`ghost ia --encender-api`) para que se deniegue y
  quede en el registro: gastar tu dinero y sacar texto del Mac son decisiones tuyas.
- **Cambiar lo que es decisión tuya:** «Redactar sugerencias», el proveedor externo, «Conservar mis
  turnos» y la respuesta de la NDA de cada cliente. Se deniegan con su motivo.
- **Abrirse sola.** La puerta **no se recuerda** en `preferencias.json` (ADR 002, enmienda 2 ya lo
  decía): nace cerrada en cada arranque y se cierra al salir.
- **Sacar nada a la red.** Es un socket Unix y el gate `puerta-solo-local` lo sostiene (§8).

### 6 · El registro: qué pidió, qué pasó, jamás el contenido

En memoria, las últimas 50 órdenes de esta sesión de la app: la hora, la orden **sin su contenido**
(«ghost corpus buscar», no lo que buscó; «ghost notas abrir», no qué reunión), si se hizo o se
denegó, el motivo y, si la hay, una cuenta («28» documentos, «3» fichas). Lo denegado también. Se
enseña en IA y muere al cerrar la app.

### 7 · Solo la ventana principal

`la_puerta`, `abrir_la_puerta` y `cerrar_la_puerta` están solo en `capabilities/default.json`: la
banda, que pinta texto de terceros sobre la reunión, no puede abrir la puerta.

### 8 · `ghost`, y cómo se vigila

- **`src-tauri/src/bin/ghost.rs`**, un cliente fino: interpreta los argumentos (en español, con los
  alias en inglés), mira el socket, lee el token del Llavero, manda la orden e imprime la respuesta en
  JSON. `ghost --version` y `ghost --help` no tocan ni el socket ni el Llavero. Salida: 0 hecho · 2
  denegado · 3 puerta cerrada · 1 fallo · 64 uso.
- **En H1 no se instala en el PATH.** Se compila junto a la app (`pnpm ghost`) y la puerta abierta
  enseña su ruta y un comando para copiar y dárselo a Claude Code.
- **Gate `puerta-solo-local`** (`tests/unit/puerta-solo-local.test.ts`): los sockets Unix solo
  pueden aparecer en `puerta/` y en `bin/ghost.rs`, y ahí dentro nada de TCP, UDP, DNS ni `libc` de
  red. Nace con su rojo: un `TcpStream` plantado en la puerta.
- **`puerta/` es módulo protegido** (`verify:ephemeral`): ni disco ni red, salvo las líneas del
  socket, que llevan su marca y este ADR en la misma línea.
- **Las pruebas jamás tocan el Llavero de verdad.** El token entra inyectado: la puerta se prueba de
  punta a punta —socket real, en una carpeta temporal— sin un solo diálogo. Lo único que queda para la
  prueba en vivo es que `ghost` lea el Llavero, y esa prueba lleva su fila de la regla 22.

## Qué se queda fuera (H2)

- **Instalar `ghost` en el PATH o dentro del `.app`**: llega con la firma (G-Release).
- **Añadir carpetas al corpus desde la puerta**: la carpeta la señalas tú con el diálogo.
- **Comparar proveedores del modelo**: el kit de la puerta mide la búsqueda, que es código; comparar
  modelos pide el modelo, y eso no es de esta fase.
- **Editar el diccionario por la puerta**: sigue siendo tu archivo.
- **Un servidor MCP** (`ghost mcp`): sería más cómodo para Claude Code; la línea de órdenes cubre H1.

## Consecuencias

- **ADR 002, enmienda 6:** `puerta.sock` es un punto de encuentro que existe mientras la puerta está
  abierta y no lleva contenido. **No entra en `Permitido`**: la puerta se cierra al empezar una sesión,
  así que si apareciera en el inventario de una, el gate en marcha tiene que delatarlo.
- La pantalla IA suma la puerta (FORMA «maquetado, no visto»), con su entrada en la fila de
  «Redactar sugerencias» y la vista de la maqueta aprobada.
- Los textos de la maqueta de la Etapa de Diseño que prometían más de lo que hace (añadir carpetas,
  comparar proveedores, el diccionario) se ajustan a lo que la puerta hace y van al bloque de textos
  del ⭐⭐.
- El manual explica cómo abrirla y cómo dársela a Claude Code.
- **Para el constructor, regla 22:** correr `ghost` contra la app de verdad lee el Llavero y abre el
  diálogo de macOS. Solo con un «sí» del usuario y su fila.

## Enmienda 1 (2026-09-28) — la puerta pide su propio desbloqueo, una vez por apertura (auditoría del S3, M4 y B12)

**Lo que encontró la auditoría.** `ghost notas abrir` pasaba por `reunion::abrir`, que usaba el mismo
`Desbloqueo` que «Exportar» en Notas. Si por la mañana exportabas una reunión, tu agente abría **todas** tus
reuniones por la tarde sin un solo Touch ID; solo quedaba una línea en el registro. Y la ayuda de `ghost` y
la vista de la puerta decían «la primera vez, macOS te pregunta», cuando el diálogo de la lista de acceso
sale en cada apertura (el token es nuevo cada vez).

**Decisión del usuario (2026-09-28): «por apertura».**
- `LaPuerta` lleva su propio `Desbloqueo` (`lib.rs`), que se olvida al abrirla, al cerrarla a mano y al
  cerrarse por la reunión (`Desbloqueo::olvidar`). `Orden::AbrirNota` abre con `reunion::abrir_con` y ese
  desbloqueo: la primera nota de cada apertura pide Touch ID; las siguientes de esa apertura, no.
- La ayuda de `ghost`, la vista de la puerta (`daselo`, `notasPor`), la maqueta `ia.html`, el manual y la
  guía (n5) dicen «cada vez que abres la puerta» y «una vez por apertura».

**Tests:** `la_puerta_pide_el_suyo_y_al_olvidarlo_vuelve_a_pedir` (`desbloqueo.rs`, rojo con `olvidar`
vacía), `la_puerta_abre_con_su_desbloqueo_y_lo_olvida_al_abrirse` (`lib.rs`, rojo sin el `olvidar` al
abrir) y `la_ayuda_dice_que_pregunta_en_cada_apertura` (`puerta/cli.rs`, rojo con la ayuda de antes). En la
prueba en vivo (su fila de la regla 22): Touch ID al primer `ghost notas abrir` de cada apertura.

**Y la carpeta, antes del socket (auditoría del S3, B17).** El socket nace con el umask y se aprieta a 600
un instante después; en ese instante lo protege la carpeta de la app. `intentar_abrir` exige ahora que la
carpeta esté en 700 **antes** de crear nada: si no, no se abre, no queda socket y no se guarda token. Antes
solo se logueaba y se abría igual. Test `sin_la_carpeta_en_700_no_se_abre` (`tests/puerta.rs`), en rojo con
el código de antes.

