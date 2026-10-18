# ADR 024 — El modo transparente: el aviso al cliente y la constancia de que consintió

- **Fecha:** 2026-10-18
- **Sprint:** 012 «Producto completo en Windows, 1 de 2» (fases 1 y 2)
- **Estado:** aceptada. Es la funcionalidad 29 de la tabla de la visión en el manual de Windows. Se construye con la interfaz **solo en Windows**
  («nada cambia en macOS» en este sprint) y queda «maquetada, no vista» para el gate humano del hito: no hay maqueta, y
  sus textos viven en el diccionario de Windows (ADR 022).

## Contexto

La tesis de la app es que lo del cliente no se guarda. Hay clientes, y jurisdicciones (el catálogo de `jurisdiccion/`
las describe), donde además se espera saber que quien te atiende usa una herramienta que escucha la reunión. Hasta hoy
la app solo ayudaba a **saber** (la bandera, la NDA, la cláusula del encargo); no ayudaba a **decirlo**, ni dejaba
rastro de que se dijo.

El H-2 «Producto completo» exige el inventario de la visión entero, y esta es una de las dos funciones que faltaban en
Windows. Las condiciones del sprint son duras: nada sale de la app, nada nuevo se guarda del cliente (regla dura 1), el
texto no lo redacta un modelo (regla 3) y el Mac no cambia.

## Decisión

1. **Un interruptor por cliente, que se recuerda como la NDA.** `Preferencias.transparentes` es un conjunto de nombres
   de cliente; vive en `preferencias.json` (en claro, 600/700, como la NDA por cliente: regla 1). **Vacío no se escribe**
   (`skip_serializing_if`): un archivo sin el modo es byte a byte el de antes.
2. **El aviso es fijo y bilingüe, y no lo redacta nadie.** `jurisdiccion::aviso_transparente()` devuelve `{ es, en }`:
   una herramienta en tu propio equipo escucha la reunión para traerte tu propia documentación, no graba el audio ni
   guarda la transcripción, todo vive en memoria y se borra al terminar, y lo único que podría guardar, si el consultor lo
   decide, es una frase corta de un acuerdo (la regla dura 1 permite eso, y decir «nada de lo que dices se guarda» era
   más de lo que la app cumple: auditoría del S12, M3). Un test impide que prometa de más. Es texto, no
   «sugerencia»: no pasa por ningún modelo ni por el API.
3. **«El cliente consintió» deja una constancia, y solo eso.** `notas::Consentimiento { cliente, jurisdiccion, hora,
idioma, texto }` entra en `Contenido.consentimientos`, **dentro del archivo cifrado que la reunión ya tiene** (sin
   formato de archivo nuevo ni superficie nueva: regla 17-bis). El `texto` es el aviso que se leyó, palabra por palabra,
   en el idioma de la pista del cliente (`aviso_en_el_idioma_del_cliente`); la `hora` es la local, `HH:MM`, y no hay
   audio ni transcript del cliente en ella. La lista vacía no se escribe y un archivo de antes se abre igual
   (`#[serde(default)]`). La misma constancia dos veces seguidas no se repite, pero la segunda pulsación **no es un
   fallo**: la constancia ya está dentro.
4. **La orden se niega en lugar de fingir.** `el_cliente_consintio` falla si no hay un cliente elegido, si el modo está
   apagado para él o si no hay una reunión abierta donde dejarla (el cuaderno cerrado no recibe constancias: auditoría
   del S12, M2); la pantalla lo dice. «Se va a guardar» cuenta las
   constancias (sin descifrar nada) y Honestidad las nombra en «Tuyo». **Una reunión con solo la constancia no se descarta
   al parar** (cuenta como algo tuyo: auditoría del S12, A1).
5. **La banda solo sabe si el modo está encendido.** Su capability tiene `modo_transparente` (un booleano) y nada más
   de esto: no puede pedir el cliente, ni el aviso, ni encender el modo, ni anotar. Un test lee las dos capabilities.
6. **Al oído, con el freno de siempre.** Con el modo solo audio encendido, `decir_el_aviso` pasa por `habla::cabe_decirla`,
   la misma regla que las fichas: por altavoces, sin voz del idioma o con alguien hablando, se calla y la pantalla lo
   dice. La voz lo lee con la voz del idioma del aviso —el del cliente, o el de la sala en presencial— y no con la del consultor
   (auditoría del S12, M1 y B12), para que se lo digas tú.
7. **Solo Windows en la interfaz, por ahora.** Las cuatro órdenes y los campos viven en Rust para los dos sistemas, pero
   Sesión y la banda solo pintan el modo en Windows (`esWindows`), y los campos vacíos no se escriben: en macOS no
   cambia ni un byte. Llevarlo al Mac sería otra decisión del usuario, con su maqueta.
8. **Los módulos protegidos no ganan disco ni red.** `notas/` y `jurisdiccion/` siguen en `PROTEGIDOS`; el modo no añade
   una sola llamada de disco o red y `verify:ephemeral` queda limpio. Lo que se escribe lo escribe `carpeta.rs`, como
   siempre.

## Lo que el modo NO hace

- **No es asesoría legal.** La app no sabe si tu cliente o tu jurisdicción exigen el aviso, ni si lo que dice basta: lo
  enciendes tú, y la pantalla lo repite igual que la de la jurisdicción.
- **No notifica al cliente.** No escribe en el chat de la reunión ni envía nada: el aviso lo dices tú (o lo lee la voz
  a tu oído). Actuar sobre la reunión del cliente iría contra la regla dura 9.
- **No prueba que el cliente lo oyó o lo entendió.** La constancia dice que **tú** pulsaste el botón, a esa hora, con
  ese texto.
- **No escucha el «sí» del cliente.** Detectarlo exigiría interpretar su transcript, que por regla muere y no se guarda.

## Consecuencias

- Contrato Rust → TS (regla 19): `VistaDelCliente.transparente` y `.aviso`, y `ResumenDelCuaderno.consentimientos`
  cruzan por `contrato.rs` (con la muestra `VISTA_DEL_CLIENTE_TRANSPARENTE`) y tienen lector en la interfaz.
- Cuatro órdenes nuevas (`modo_transparente`, `fijar_modo_transparente`, `el_cliente_consintio`, `decir_el_aviso`), cada
  una en la capability que le toca; el gate de capabilities impide que la banda reciba alguna de más.
- Manual de Windows (§ «El modo transparente», fila 29 «hecho sin verificar»), parada **w23** de la guía de Windows y
  BLUEPRINT.
- Verificación: la CI prueba la estructura, la negativa sin reunión, la forma del archivo y la interfaz; lo que solo ve
  una persona con un cliente delante (que el aviso se entienda, que la voz lo lea bien) viaja al gate humano del hito.

## Alternativas descartadas

- **Aviso redactado por el modelo.** Rompe «código primero» (regla 3) y haría depender de un API una cosa que debe ser
  idéntica cada vez, y comprobable.
- **Un archivo aparte de constancias.** Es un formato nuevo, otra carpeta que proteger y otra cosa que borrar; la
  constancia es parte de la reunión y viaja cifrada con ella.
- **Detectar el consentimiento por la voz del cliente.** Persistiría algo del tercero y exigiría interpretar su
  transcript (reglas 1, 3 y 4).
- **Encenderlo en todas las reuniones.** El aviso depende del cliente y de su jurisdicción; por eso es por cliente,
  junto a la NDA.
- **Llevarlo al Mac en el mismo sprint.** El usuario fijó «nada cambia en macOS» para el S12; una pantalla nueva en el
  Mac pide su maqueta y su mirada.
