# Etapa de Diseño — registro vivo (Angel Ghost)

> Registro de la Etapa de Diseño (F2a): **cero código de producto**, solo la fundación visual.
> Lleva el **registro de miradas** —ningún artefacto se construyó encima sin feedback del usuario
> fechado; «continúa» no aprueba diseño—, la tabla pantalla → funcionalidad, la del spike, la
> auto-auditoría y el veredicto de G-Diseño.

## Cómo abrir la maqueta

`git checkout diseno/fundacion` → **doble clic en `docs/diseno/index.html`**. Sin build, sin
red, sin servidor. Desde ahí se entra a las nueve pantallas; dentro de cada una, la barra
superior conmuta **estado · tema · idioma** y la nota bajo la barra dice qué mirar.

## Plan de miradas (aprobado con el plan, 2026-09-20)

| Mirada | Artefacto(s) | Orden |
|---|---|---|
| 1 | `panel.html` (+ `assets/ghost.css` y `design-system.md` v0) | el panel primero, siempre |
| 2 | `kit.html` + `design-system.md` completo | |
| 3 | `sesion.html` · `permisos.html` · `honestidad.html` | |
| 4 | `corpus.html` · `notas.html` · `idioma.html` · `ia.html` | |
| 3-bis | `honestidad.html` (qué queda) + `panel.html` (modo solo audio) — **añadida en la mirada 3** | antes de la Fase 4 |
| 3-ter | `posicion.html` (dónde vive el panel) + radar invasivo — **añadida en la mirada 3-bis** | antes de la Fase 4 |
| 3-quater | `posicion.html` (banda acoplada D2/D3/F) + `permisos.html` (permiso de acople) — **añadida en la mirada 3-ter** | antes de la Fase 4 |
| 3-quinquies | `posicion.html` (relleno de captura: qué ve el cliente en la franja) — **añadida en la mirada 3-quater** | antes de la Fase 4 |
| 4-bis | `notas.html` (propuestas) · `idioma.html` (varios idiomas) · `ia.html` (Claude Code) — **añadida en la mirada 4** | antes de la Fase 5 |
| 4-ter | `notas.html` (bandeja con cuenta atrás) + `honestidad.html` — **añadida en la mirada 4-bis** | antes de la Fase 5 |
| 5 = G-Diseño | `index.html` + recorrido completo | |

Cualquier cambio al plan (agrupar, reordenar, posponer) se propone y se aprueba ANTES de
construir el siguiente artefacto.

### Extensión posterior a G-Diseño (sprint 001)

| Mirada | Artefacto | Por qué existe | Orden |
|---|---|---|---|
| 11 | `banda.html` — los seis estados de contenido de la banda | el gate de FIDELIDAD del sprint 001 compara la banda construida contra la maqueta, y la maqueta dibujó la banda **siempre con una ficha dentro**: cinco de los seis estados no tenían referencia. Se propuso en el plan del sprint, **aprobado por el usuario antes de construir** | antes de escribir la primera línea de UI del sprint 001 |
| 12 | `sesion.html` · `permisos.html` · `honestidad.html` — el estado **«así se ve hoy · sprint 1»** y el componente **«todavía no»** | la maqueta dibuja el producto terminado y cada sprint entrega un trozo; sin una forma escrita de decir «esto aún no existe», las pantallas de la fase 2 solo podían mentir en verde o esconder lo que falta. Se propuso en la bitácora ANTES de construir la UI del producto | antes de escribir la primera línea de las pantallas del cuaderno |
| 13 | `idioma.html` (estado s1 nuevo) · `sesion.html` y `honestidad.html` (su estado s1 **puesto al día**) | consecuencia directa del «todavía no» aprobado en la 12: cada sprint mueve filas de *pendiente* a *funciona*, y ese movimiento **también es diseño**. La fase 3 mueve cuatro (las dos pistas, la escucha, el transcript) y trae una pantalla que no tenía estado de sprint 1. Además, tres hechos que solo se supieron construyendo piden sitio en la maqueta: el **eco** con altavoces, el **techo de cinco idiomas** de macOS y la **descarga del modelo**. Se propuso en la bitácora ANTES de construir la UI de la fase 3 | antes de escribir la primera línea de la pantalla de Idioma |

### Mirada 12 — veredicto del usuario (2026-09-21)

> **«Esta muy bien como indica que no todavia no existe en sesion permisos y honestidad»**

**Aprobada.** El usuario abrió las tres pantallas y nombró las tres. `.estado.pendiente` y el
estado «así se ve hoy · sprint 1» quedan aprobados como parte del design system (§9-sexies,
v1.10.0), y con ellos las dos decisiones que la maqueta no había escrito: el permiso único de
macOS para «Audio del sistema» y «Pantalla», y la Accesibilidad en la lista principal de permisos.

A partir de aquí, **toda pantalla que se entregue a medias usa este estado**: no se pinta en verde
lo que no existe, y no se esconde.

### Mirada 13 — veredicto del usuario (2026-09-21)

> **«Me gustó mucho el diseño y cómo se van evidenciando los elementos construidos y lo que falta,
> muy bien lograda»**

**Aprobada.** Lo que el usuario nombra es el mecanismo de la mirada 12 ya aplicado a un sprint
concreto, no su enunciado: las cuatro filas que la fase 3 movió de *pendiente* a *funciona*. Con
ello quedan aprobados el estado `s1` nuevo de `idioma.html`, los de `sesion.html` y
`honestidad.html` puestos al día, y los tres hechos que solo se supieron construyendo y que la
maqueta ahora dice — el **eco** con altavoces internos, el **techo de cinco idiomas** de macOS y
la **descarga del modelo**.

*(El veredicto llegó tras repreguntar por la regla 10: la primera respuesta fue «Apruebo las tres
pantallas… continúa», y un «apruebo» no es un «lo vi». Se registra el camino porque es la segunda
vez en esta app que la repregunta hace aparecer contenido que la palabra de aprobación no traía.)*

La maqueta no se congela con G-Diseño: se extiende por el mismo camino (propuesta → mirada →
registro). Lo que **no** cambia sin una mirada nueva es lo ya aprobado — y esta extensión no
redecide nada: cruza la forma elegida en la mirada 3-ter con el contenido aprobado en la 1.

### Mirada 14 — veredicto del usuario (2026-09-21)

> **«Ya vi el diseño del corpus, vamos muy bien; ya están las secciones de las temáticas
> principales y lo que falta es muy interesante, por ejemplo lo de arrastrar los documentos»**

**Aprobada.** El usuario nombra una de las tres filas de «todavía no» del bloque —*arrastrar y
soltar documentos*—, así que el veredicto llega con el archivo abierto. Quedan aprobados el
estado `s1` de `corpus.html` y las tres cosas que solo se supieron construyendo y que la pantalla
dice en vez de esconder: que **las secciones de un PDF son conjetura** y se cuentan, que el
índice vive en la carpeta de datos de la app **y solo su dueño puede leerlo**, y el techo de
2 000 documentos por carpeta.

### Mirada 16 — veredicto del usuario (2026-09-26)

> **«Apruebo el diseño muy limpio icono azul a la izquierda indicando que se habla muy intuitivo y
> amplio margen para la pantalla de reunión»**

**Aprobada.** Las dos cosas que nombra son las dos que esta mirada tenía que decidir, y ninguna se
puede ver sin abrir el archivo: el **glifo azul a la izquierda** es `i-voz` pintado con `--halo`
(`#9ecbff` en oscuro, `#1d5c9c` en claro) —el mismo acento único del resto del sistema— y es el
símbolo que la regla 8 exige al lado del texto; y el **margen para la pantalla de la reunión** es
justamente lo que compra bajar la banda de 88 px a 44 px, que era el motivo de existir del modo.

Quedan aprobados, con ella, los **tres criterios declarados** en las notas de «Qué mirar» del
artefacto:

1. **el contador de red se queda** en la línea, aunque a 44 px la cabecera desaparezca — es una
   promesa dura (regla 2), no un adorno;
2. **la tecla es `⌘⇧V`**, no el `⌘⇧A` que pedía la orden del sprint: `⌘⇧A` ya es «ayúdame con
   esto» desde el sprint 001. Desviación declarada en la bitácora;
3. **el glifo `⎋` se deja como está** — se lee como un borrón a 13 px igual que el `⌥⎋ corta` ya
   aprobado de la banda de 88 px; si molesta, molesta en los dos sitios y es un cambio del sistema.

No hubo que repreguntar: el veredicto llegó con la descripción de lo que vio, que es lo que la
regla 10 pide.

### Mirada 16-bis — veredicto del usuario (2026-09-26)

> **«Ya revisé solo audio · callado, está bien, pero creo que sí debería hacer evidente el estado;
> esto por ejemplo me parece diciente y apropiado: «callado», «esperando el siguiente turno»»**

**Aprobada con un cambio, y el cambio corrige un error de criterio mío.** La propuesta enseñaba la
línea de la ficha recién leída y presumía de «no inventar ni una palabra» — pero ese ahorro se pagaba
con lo único que el estado tenía que comunicar: **que la app está encendida y callada**. Una banda
que no dice en qué está obliga a mirarla para averiguarlo, que es exactamente lo contrario de un modo
que existe para no tener que mirar.

La línea queda **«Callado · esperando el siguiente turno»**, y detrás la fuente de lo último que se
leyó. Es el mismo patrón **«estado · por qué»** que el usuario ya había aprobado en la mirada 16 con
«Conecta auriculares · el cliente te oiría», así que el modo entero se lee igual en sus tres estados:

| Estado | Lo que dice la línea |
|---|---|
| hablando | Diciéndote la ficha… · *de dónde sale* |
| **callado** | **Callado · esperando el siguiente turno · *de dónde salió la última*** |
| sin auriculares | Conecta auriculares · el cliente te oiría |

Aplicado el mismo día en la maqueta, en el diccionario (`callado`, `esperandoElSiguienteTurno`, en los
dos idiomas) y en el producto, con sus tests y el gate de fidelidad en verde.

### Cambio al plan de miradas del sprint 002 — aprobado ANTES de construir (2026-09-26)

El plan del sprint declaró **tres** miradas (16, 17, 18). La 17 se parte en **dos sesiones seguidas**,
las dos al empezar la fase 3. Propuesto y **aprobado por el usuario antes de maquetar la primera**,
que es lo que la regla del plan de miradas exige (kit v1.21.0).

**Por qué se parte.** Al construir la fase 2 aparecieron **19 campos del contrato sin lector** —datos
que la parte nativa ya mide y manda a la pantalla, y que nadie pinta, porque pintarlos exige copy que
la maqueta no tiene—. Nueve caían dentro de la 17; los otros **diez, no**, y no tenían mirada
asignada. Eran dos familias visuales distintas metidas en una sesión:

| Mirada | Familia | Qué se maqueta |
|---|---|---|
| **17** | *algo falla o algo vigila* | radar ámbar · radar coral (banda) · «vigilancia local» (Sesión) · **«pista caída»** (Sesión, M2) · el consentimiento de pantalla (Permisos) |
| **17-bis** | *la app se explica a sí misma* | la **ficha** con su motivo, su latencia y la marca de sección conjeturada · el **transcript** con el tramo del turno · **Idioma**: motor, techo, motivo y la fila del **diccionario** · **Corpus**: carpeta y secciones |
| **17-ter** | *la frase de «MinutaBot»* — **añadida en la mirada 17** | una frase en el radar ámbar (banda) y en el catálogo (`kit.html`): **«Ese bot no es Angel Ghost, que nunca entra a la llamada»**. Va **antes de la fase 4**, que es la que construye el radar: no frena la fase 3 |
| **17-quater** | *lo que la fase 3 necesita y la 17 no dibujó* — **aprobada por el usuario el 2026-09-26, antes de maquetar** | **Permisos**: dos frases falsas corregidas (macOS no admite texto propio en el diálogo de pantalla; audio del sistema y pantalla son **dos** permisos) · **Sesión**: el interruptor de la lectura automática, el estado sin permiso de pantalla, los motivos de la pista caída en es/en, el nombre de la salida de audio, y por qué no se sabe la salida ni la reunión · **banda**: los motivos de la ficha que faltan y el atajo de lectura bajo demanda. Se maqueta **después del motor**, para que el copy diga lo que el código distingue; va antes de construir esas pantallas |
| **17-quinquies** | *la segunda vuelta de la 17-quater* — **nace de su veredicto**, como la 16-bis nació de la 16 | **Permisos** y **Sesión**: «solo cuando cambia», que no se entendió, pasa a «solo si hay algo nuevo, como otra diapositiva» (Permisos) y «Pantalla — solo lee lo nuevo» (Sesión) · **Honestidad**: «El botón corta 8 de 8 piezas…», porque «8 de 8» se leyó como «leyó todo» · **todas las teclas** pasan de `⌘⇧` a `⌃⌥` (decisión 7 = B) · y la fila de «ficha · la trajo la pantalla», que se quedó sin respuesta. Va antes de construir esas pantallas |

**Las dos van seguidas, en la misma sesión de mirada**, para no añadir una parada más. Y las dos van
antes de la fase 3, no de la 5: si esperaran a la 18, los diez campos se caerían con la fase 5, que es
la primera de la lista de cortes declarada en el plan.

**Y de paso queda decidido qué enseña Idioma del diccionario** (la otra pregunta abierta). De las
cuatro piezas que la mirada 4 aprobó en `idioma.html`, entran las **dos que se pueden pintar hoy con
datos de verdad** —la tabla «de dónde salen» y la franja «corregir no es inventar»— más **una línea
con la ruta del archivo**, que es el único copy nuevo. Quedan fuera, declaradas: el **formulario**
para añadir términos (haría escribir al módulo protegido, que la enmienda 1 del ADR 002 prohíbe a
propósito) y la **lista de correcciones** tachado → resaltado (exige guardar las correcciones del
turno). Las dos son decisiones de producto y su sitio es el sprint 003.

### Cambio al plan de miradas del sprint 002 — una sola mirada humana, en el gate del MVP (2026-09-26)

**Decidido por el usuario** al cerrar la fase 3 (su frase textual está en el registro, abajo). Desde
ese momento, **las miradas de copy** —si una frase se entiende, si un rótulo es claro— **no paran la
construcción**: se escriben en la maqueta, las vigilan los gates automáticos y se prueban y se editan
**en el gate del MVP**, que es el único gate humano del ciclo. Lo que viaja allí, declarado para que
no se pierda:

| Qué | De dónde viene |
|---|---|
| Las siete filas de la mirada de cierre de la fase 3 (Sesión, Permisos, Honestidad, la ficha que trajo la pantalla, «pantalla · nada que leer», § 8-ter, la prueba en vivo con `meet-de-prueba.html`) | fase 3 |
| El radar construido al lado de su maqueta (la 17-ter lo dejó para «el gate de fidelidad de la fase 4») y el estado «vigilancia» de Sesión ya arreglado | fase 4 |
| La 18 (sugerencia local, sugerencia API, pantalla IA): se maqueta igual antes de construir, pero su mirada viaja al gate del MVP | fase 5 |

La regla 10 sigue en pie en lo que el usuario no cambió: **ninguna de estas pantallas cuenta como
vista** hasta ese gate, y el summary del sprint lo dice así.

## Registro de miradas

| Fecha | Artefacto | Veredicto del usuario (línea textual) | Qué se construyó encima, después |
|---|---|---|---|
| 2026-09-20 | `panel.html` (mirada 1) ⭐ | **Aprobado** — «El panel se ve muy bien aprobado» (con el archivo abierto en su Mac; antes había respondido «Continua» y se le repreguntó, regla 10) | Fase 2: `kit.html` + `design-system.md` completo |
| 2026-09-20 | `kit.html` + `design-system.md` v1.0.0 (mirada 2) | **Aprobado** — «Si apruebo el kit me gusto mucho muy oportuno el diseño y elementos» (con el archivo abierto en su Mac) | Fase 3: `sesion.html` · `permisos.html` · `honestidad.html` |
| 2026-09-20 | `sesion.html` · `permisos.html` · `honestidad.html` (mirada 3) | **Aprobadas con dos cambios** — «en honestidad está bien que lo del cliente se elimine no le veo problema pero lo que sí quiero es que me quede lo que es mío o lo que dije o escribí, adicional quisiera tener un modo solo audio que me hable de forma paralela por si quiero ver completamente la pantalla y no me interrumpa, todo el resto lo veo muy bien» | Fase 3-bis: los dos cambios, antes de la Fase 4 |
| 2026-09-20 | `honestidad.html` + `panel.html` (mirada 3-bis) | **Dos ajustes más** — modo audio como icono pequeño abajo izquierda o banda inferior («¿y si manejamos el panel en la parte inferior?»); y el radar debe detectar proctoring y anti-cheat, «que son invasivos» | Fase 3-ter: `posicion.html` + radar de dos niveles |
| 2026-09-20 | `posicion.html` + radar invasivo (mirada 3-ter) | **Decidido: D acoplada + F** — «me gusta mucho la D · banda pegada al borde aunque un poco más arriba al menos el doble, con la posibilidad de ampliarla y ojalá […] recorte la pantalla de la reunión como si fueran dos aplicaciones pegadas […]. En cuanto al E y F ambas me gustan mucho pero vamos con F. Muy bien lo de anti…, estamos solo protegiéndonos de software que quiera invadir nuestra independencia» | Fase 3-quater: banda acoplada 88/200/44 + permiso de acople |
| 2026-09-20 | `posicion.html` (banda acoplada) + `permisos.html` (acople) (mirada 3-quater) | **Aprobada con un cambio** — «me pareció genial D2, D3 y F. ¿Qué te parece a ti? ¿Por qué es más seguro que el flotante? Podemos hacer que no se vea el escritorio sino en negro, no quiero que vea que algo ocupa ese espacio» | Fase 3-quinquies: **relleno de captura** (la franja deja de mostrar el escritorio) |
| 2026-09-20 | `posicion.html` (relleno de captura) (mirada 3-quinquies) | **Decidido: fondo de escritorio** — «sí vamos con el relleno fondo de escritorio». *(Elección entre las tres opciones presentadas. Se repreguntó si había abierto el archivo, regla 10; respondió «Listo continúa» — respuesta afirmativa, pero sin describir lo que vio. Se registra tal cual y **el relleno vuelve a la mirada 5**, dentro del recorrido completo.)* | Fase 4 |
| 2026-09-20 | `corpus.html` · `notas.html` · `idioma.html` · `ia.html` (mirada 4) | **Corpus aprobado; tres cambios** — «Notas: me gusta pero es que soy malo tomando notas, no sé cómo voy a lograr tomar notas; no sé si fuera posible que me propusiera si x información deba guardarse como notas. Idioma está bien pero siento que también debería la opción bilingüe para seleccionar más de un idioma. IA me gusta, está bien, aunque quiero que Claude Code también me pueda ayudar a operarla ya que estamos en local» | Fase 4-bis: notas propuestas · varios idiomas · puerta local para Claude Code |
| 2026-09-20 | `notas.html` · `idioma.html` · `ia.html` (mirada 4-bis) | **Idioma e IA aprobados; una cosa en notas** — «me gusta mucho lo de las notas, me preocupa es que mientras estoy en la reunión no puedo decidir; ¿es posible decidir apenas finalice la reunión y darme una o unas horas antes de borrar las sugerencias? De resto sí que me gusta mucho notas. Idioma muy completo, incluso mejor de lo que pensaba. IA también quedó excelente, aprobado» | Fase 4-ter: bandeja de propuestas con cuenta atrás |
| 2026-09-20 | `notas.html` (bandeja) + `honestidad.html` (bandeja en las cuentas) (mirada 4-ter) | **Aprobada** — «abrí la ventana, claramente la exploré con sus botones y me pareció excelente, continuamos». *(Se le repreguntó por la regla 10 al llegar solo la palabra de fase; respondió con la exploración descrita. Observación suya, justa y registrada: no tiene que rendir cuentas de lo que ve.)* | Fase 5: recorrido, README, auditoría y G-Diseño |
| 2026-09-20 | `banda.html` — los seis estados de contenido de la banda (mirada 11, sprint 001) | **Aprobada con un cambio** — «Si me gusta mucho muy bien docs/diseno/banda.html, pero en Sin resultado esta bien que digas que no hay nada pero sugierele como abordar la situacion. El resto esta muy muy bien» | La **maniobra**: lo más cercano del corpus + un catálogo versionado de seis maneras de responder, los dos deterministas (cero LLM). Luego, la fase 1b: las tres ventanas |
| 2026-09-21 | `sesion.html` · `permisos.html` · `honestidad.html` — estado «así se ve hoy · sprint 1» y el componente «todavía no» (mirada 12, sprint 001) | **Aprobada** — «Esta muy bien como indica que no todavia no existe en sesion permisos y honestidad» (abrió las tres y nombró las tres) | Fase 2b: las tres pantallas del cuaderno construidas |
| 2026-09-21 | `idioma.html` (estado s1 nuevo) · `sesion.html` y `honestidad.html` al día (mirada 13, sprint 001) | **Aprobada** — «Me gustó mucho el diseño y cómo se van evidenciando los elementos construidos y lo que falta, muy bien lograda». *(Se repreguntó por la regla 10: la primera respuesta fue «Apruebo las tres pantallas… continúa», y un «apruebo» no es un «lo vi».)* | Fase 4: corpus, disparo y ficha |
| 2026-09-21 | `corpus.html` (estado s1 nuevo) (mirada 14, sprint 001) | **Aprobada** — «Ya vi el diseño del corpus, vamos muy bien; ya están las secciones de las temáticas principales y lo que falta es muy interesante, por ejemplo lo de arrastrar los documentos» (nombra una de las tres filas de «todavía no»: llegó con el archivo abierto) | Fase 5: efímero en runtime, kit de evaluación, guía de prueba y manual |
| 2026-09-26 | `banda.html` — el **modo solo audio** a 44 px: «hablando» y «sin auriculares» (mirada 16, sprint 002) | **Aprobada** — «Apruebo el diseño muy limpio icono azul a la izquierda indicando que se habla muy intuitivo y amplio margen para la pantalla de reunión» (nombra el glifo `i-voz` en `--halo` y el alto de 44 px: llegó con el archivo abierto) | Fase 2: la voz que sale (C15) — `habla/`, `⌘⇧V` y la banda cableada a 44 px |
| 2026-09-26 | `banda.html` — **«solo audio · callado»**, el estado que la 16 no dibujó (mirada 16-bis, sprint 002) | **Aprobada con un cambio** — «Ya revisé solo audio · callado, está bien, pero creo que sí debería hacer evidente el estado; esto por ejemplo me parece diciente y apropiado: «callado», «esperando el siguiente turno»» | El cambio, aplicado en el acto: la línea dice **«Callado · esperando el siguiente turno»** y detrás la fuente de lo último leído. Fase 3: la pantalla (C8) |
| 2026-09-26 | **Mirada 17** — `banda.html` (radar ámbar, radar coral y su ampliada) · `sesion.html` (sprint 2 · pista caída) · `permisos.html` (sprint 2 · la pantalla) — *algo falla o algo vigila* | **Aprobada, con una pregunta que se volvió frase** — «“radar · te graban” · “radar · te vigilan” · “… · ampliada” está espectacular y muy bien, pero esto me dejó loco: «MinutaBot» está en la lista de participantes; yo no quiero que haya un participante adicional en la reunión […], no quiero que las notas dependan de esto […]. “sprint 2 · pista caída” está perfecto, aviso importante para no enterarse al terminar la reunión. “sprint 2 · la pantalla” excelente la descripción de lo que se hace con los permisos, muy adecuado para el usuario. […] Yo que tengo leve daltonismo los puedo identificar rápidamente para determinar los inconvenientes. Por ahora veo bien que hayas puesto la tecla nueva que inventaste» (nombra cada botón por su rótulo: llegó con los archivos abiertos) | La **mirada 17-ter**: la frase que dice que el bot no es Angel Ghost. `⌘⇧R qué ve` queda **aprobada por ahora**. Fase 3: la pantalla (C8) |
| 2026-09-26 | **Mirada 17-bis** — `banda.html` (la ficha con su motivo y su latencia, la ficha con sección conjeturada, el transcript con la duración) · `idioma.html` (sprint 2, y sin motor) · `corpus.html` (sprint 2) — *la app se explica a sí misma* | **Aprobada** — «“ficha” · “ficha · sección conjeturada” · “transcript” las veo muy bien, sin comentarios. “sprint 2” · “sprint 2 · sin motor de voz” muy bien, excelente aviso de ausencia de motor de voz, muy apropiado. “sprint 2” corpus muy bien, avisa cuáles documentos no se pudieron leer con sus secciones, muy adecuado» | Fase 3: los diez campos del contrato sin lector |
| 2026-09-26 | **Mirada 17-ter** — `banda.html` (radar · te graban) · `kit.html` (§ 7, alerta del radar) — la frase **«Ese bot no es Angel Ghost, que nunca entra a la llamada»** | **Aprobada** — «Ammm, eso quiere decir que MinutaBot no es nuestro, es de otro; me queda más claro, pero sobre todo tranquilo: nunca diseñamos algo así. Sigue». *(Juzgó la frase, que iba citada en el mensaje; el estado visual ya lo había visto en la 17. No hay señal de que reabriera el archivo, y se registra tal cual: el estado vuelve a sus ojos en el gate de fidelidad de la fase 4, con la banda construida al lado de la maqueta.)* | Fase 4: el radar (C14) |
| 2026-09-26 | **Mirada 17-quater** — `permisos.html` (sprint 2 · la pantalla, **corregido**) · `sesion.html` (sprint 2 · la pantalla) · `honestidad.html` (sprint 2) · `banda.html` (ficha · la trajo la pantalla · pantalla · nada que leer) · `kit.html` (§ 8-ter, «Los porqués») — *lo que la fase 3 necesita y la 17 no dibujó* | **Aprobada en 2, 3, 5 y 6 · la 1 con un cambio · la 4 sin respuesta · decisión 7 = B** — «1. Esta parte es la que no entiendo: “solo cuando cambia”; de resto está bien. 2. Sí se entienden. 3. Sí, me deja tranquilo: entiendo que leyó todo sin problema, que el último está en memoria y que pesa 1,4 MB. 5. Sí, de acuerdo y claro. 6. Todas están bien y se entienden. 7. B, pasar todas a ⌃⌥». *(Cita «solo cuando cambia» y el «1,4 MB»: llegó con los archivos abiertos. **La 4 no se contestó y no se da por vista**: vuelve en la 17-quinquies. Y en la 3, «leyó todo sin problema» no es lo que dice «8 de 8 piezas», que habla del botón rojo: la frase se reescribe y vuelve a sus ojos.)* | La **mirada 17-quinquies**. La UI de la fase 3 no se construye hasta su veredicto |
| 2026-09-26 | **Mirada 17-quinquies** — `permisos.html` y `sesion.html` (sprint 2 · la pantalla) · `honestidad.html` (sprint 2) · `banda.html` (ficha · la trajo la pantalla · esperando, con las teclas `⌃⌥`) — *la segunda vuelta de la 17-quater* | **Fundida con la mirada de cierre de la fase 3, por decisión del usuario** — «Uyyy, no paro; así no vamos a avanzar nada» y, enseguida, «sigue». Se construye con las frases nuevas y con la fila 4 tal como se maquetó; las cinco filas vuelven a sus ojos **al cerrar la fase**, con la pantalla construida al lado de la maqueta. La fila 6 (el test de desbordes) se toma con la opción recomendada | Fase 3: las pantallas de la lectura de pantalla y los 7 campos |
| 2026-09-26 | **Mirada de cierre de la fase 3** — `docs/fidelidad/S2-cuaderno.html` · `docs/fidelidad/S2-banda.html` (la pantalla construida al lado de la maqueta) · `kit.html` § 8-ter · la prueba en vivo con `meet-de-prueba.html` — *incluye las cinco filas de la 17-quinquies* | **Diferida al gate del MVP, por decisión del usuario** — «Estamos perdiendo demasiado tiempo en esas pruebas de texto; no están validando funcionalidad ni cosas serias, sino si se entienden o no los mensajes. Necesito terminar esto cuanto antes, y esto se puede probar y editar en los gates; y a propósito, solo vamos a hacer un gate cuando ya esté el MVP, entonces vamos a dejar estas pruebas de texto para el gate. No más de esto, vamos a avanzar». **No es una aprobación: nadie miró.** Lo construido queda vigilado por los gates automáticos (fidelidad 84 encuadres, diccionario fiel a la maqueta, maquetas que caben) y las siete filas viajan al gate del MVP | Fase 4: el radar (C14) |
| 2026-09-26 | **Mirada 18** — `banda.html` (sugerencia · en tu Mac · sugerencia · API · sugerencia · ampliada) · `ia.html` (así se ve hoy · sprint 2) · `honestidad.html` (sprint 2: la frase de la red) · `kit.html` § 7-bis — *la síntesis (C7)* | **Maquetada antes de construir, y su veredicto viaja al gate del MVP** por la decisión del usuario del mismo día (un solo gate humano). Traslado a la banda y a la pantalla de hoy de piezas que el panel y la Etapa de Diseño ya aprobaron; lo nuevo es el sitio y dos frases corregidas (Honestidad decía «la app no abre ninguna conexión», falso con el API opcional). **No cuenta como vista** | Fase 5: la síntesis (C7) |
| 2026-09-27 | **Cierre de la fase 1 (7 filas) + mirada 20 (6 filas), sprint 003** — `notas.html` (al cerrar · no se pudo guardar · sin reuniones · borrar · no se exportó · durante, con propuestas · al cerrar, con propuestas · al cerrar, ventana cero · bandeja vencida) · `honestidad.html` (sprint 3 · tras el corte, con notas) · `banda.html` (te propongo guardar · fijada) | **Diferidas a los gates, por decisión del usuario** — «dejemos de revisar pequeñeces; solo muéstrame cosas realmente importantes que deba decidir, el resto lo validamos en los gates». **No es una aprobación: nadie miró.** Se construye con lo maquetado y tres decisiones del constructor: «Borrar ahora» pregunta antes · Honestidad sin cifras · la señal «fijada» en la banda (símbolo + texto + color, regla 8). Las 13 filas viajan al bloque de diferidos del ⭐⭐. Desde aquí solo abre parada lo que cambia una decisión del usuario, la promesa del producto o toca su Mac | Fase 2: las pantallas de propuestas y bandeja |
| 2026-09-27 | **Decisión A del usuario: tus notas, en la carpeta privada de la app** — `notas.html` (la fila «Dónde» de los seis estados del archivo pasa de `~/Documentos/Angel Ghost/` a «Carpeta privada de la app» con **«Mostrar en Finder»** · «no se pudo guardar» cambia «Elegir otra carpeta» por **«Intentar otra vez»** · la bandeja deja de decir «fuera de Documentos») · `honestidad.html` («Lo que quedará cuando cierres» nombra la carpeta de la app en los cinco estados) | **No es una mirada: es la consecuencia de una decisión del usuario** («Sí la A», ante la tabla A/B/C del cierre de la fase 2). La forma y el copy nuevos quedan **«maquetado, no visto»** y viajan al bloque de diferidos del ⭐⭐. Fidelidad: 176 encuadres, ninguno sobre el umbral | ADR 015 enmienda 1 · ADR 016 «Decisión del usuario» |
| 2026-09-27 | **Fase 3 del sprint 003 — el marco en la mano** (ADR 017) — `sesion.html` (seis estados nuevos: este cliente · en marcha · la pregunta de la NDA · sin jurisdicción · la cláusula · en solo notas; y el estado de la Etapa de Diseño «NDA prohíbe transcribir», ya con producto, pasa su chip del rail a «Meet detectado» como los demás) · `banda.html` (solo notas) · `kit.html` §5 (fuera del catálogo · sin verificar · los cinco riesgos · el selector · «Copiada») | **Maquetado, no visto**, por el criterio del usuario (solo abre parada lo que cambia una decisión suya, la promesa o toca su Mac). Tres decisiones del constructor que se validan en el gate: **Sesión vuelve al diseño de la Etapa de Diseño** y retira «Qué funciona hoy» (la fila «A medias» de la mirada 17 se queda, solo cuando una pista cae) · **la pregunta de la NDA ocupa la fila de los botones** (en la tarjeta no cabía: `maqueta-cabe` midió +65 px) · **el aviso se acorta a «No es asesoría legal»** (las normas y la fecha ya están en la bandera). Fidelidad: 204 encuadres, ninguno sobre el umbral | ADR 017 · bitácora, fase 3 |
| 2026-09-27 | **Fase 4 del sprint 003 — la puerta local** (ADR 018) — `ia.html` (tres estados nuevos: la puerta · la puerta abierta · se cerró sola; y en «sprint 3 · quién redacta», la entrada «Puerta local · cerrada» junto a «Redactar sugerencias») · `ghost.css` (`.puerta`, relleno de 5 a 3 px) | **Maquetado, no visto** (mirada 22), por el criterio del usuario. Lo que cambia respecto a la Etapa de Diseño, para el gate: **se entra por un botón y se vuelve con otro**, como «lo que salió» · el botón de volver va en la fila del título y la descripción pasa al subtítulo (`maqueta-cabe` midió +120 px) · **los textos que prometían más de lo que la puerta hace se ajustan**: reindexar sin «añadir carpetas», el kit mide la búsqueda y no «compara proveedores», las preferencias son las de la lista blanca, y «Encender el API ni decidir por ti» (redactar, tus turnos, tu NDA) · el comando para Claude Code y **qué va a preguntar macOS** · el registro con el motivo de lo denegado. Fidelidad: 216 encuadres, ninguno sobre el umbral | ADR 018 · bitácora, fase 4 |
| 2026-10-04 | **Fase 0 del sprint 004 — forma y texto** — `sesion.html` (sprint 4 · la banda arriba o abajo · la primera vez: la fila «La banda» con ⌃⌥B y el aviso de la primera vez) · `honestidad.html` (sprint 4 · con tus ensayos: «Tuyo» suma tus ensayos y el corte pasa a 12 de 12) · `ia.html` (sprint 4 · enriquecer el banco: el segundo interruptor, apagado) | **Maquetada, no vista** (método v1.38.0: FORMA y TEXTO no abren parada). Va al gate del MVP del H2. Las dos miradas de DECISIÓN del sprint —`ensayo.html` y la variante «arriba» de `posicion.html`— se registran aparte, con la frase del usuario | ADR 019 · ADR 004 enmienda 1 · ADR 002 enmienda 8 |
| 2026-10-04 | **Mirada de DECISIÓN 1 del sprint 004** — `ensayo.html` (los siete estados: preparar · preguntando · respondiendo · evaluada · el informe · tu progreso · sin corpus) — *la pantalla del ensayo (C18)* | **Aprobada** — «La abri y la apruebo la pantalla ensayo, continua». *(«La abrí y la apruebo» es la fórmula de la regla 10: llegó con el archivo abierto. El «continua» no se toma como paso de fase: la mirada 2 sigue pendiente y sin ella no se construye encima.)* | Fase 3: la pantalla `Ensayo.tsx`, fiel a esta maqueta. Antes, la mirada de DECISIÓN 2 (`posicion.html`, variante «arriba») |
| 2026-10-04 | **Mirada de DECISIÓN 2 del sprint 004** — `posicion.html`, variante «arriba» (88 · acoplada · ampliada · solo audio · qué ve el cliente · sin reunión: flota), con el criterio de la cámara escrito y la regla «solo se mueve la ventana de la reunión» — *la banda arriba (C1')* | **Aprobada** — «Si me gusta mucho ka banda arriba buen diseño, lo abri y lo apruebo». *(«Lo abrí y lo apruebo»: llegó con el archivo abierto. Con las dos miradas de DECISIÓN en «sí», la fase 0 cierra y se construye encima.)* | Fase 1: la banda arriba, fiel a esta variante |
| 2026-10-04 | **Fase 1 del sprint 004 — forma** — `sesion.html` (la fila «La banda» con ⌃⌥B también en «sprint 3 · en marcha», «la pregunta de la NDA» y «sin jurisdicción», porque el producto la enseña en todos los estados con «Las dos pistas») · `assets/maqueta.css` (la sala de diseño ya no le quita la línea de abajo a una banda con `data-borde="arriba"`) | **Maquetada, no vista** (forma). Va al gate del MVP del H2. Fidelidad: 232 encuadres, ninguno sobre el umbral; `maqueta-cabe` en verde | ADR 004 enmienda 1 · bitácora, fase 1 |
| 2026-10-04 | **Fase 3 del sprint 004 — forma y texto** — `ensayo.html` (tres estados nuevos: **1b · no empezó** —el micrófono sin permiso, encima de «preparar»—, **2b · del modelo** —una pregunta «sugerida por el modelo» al final, con la línea «El modelo sumó 2 preguntas…»— y **8 · textos** con **9 · más textos** —el chip de cada fuente, las cinco líneas de «el modelo no sumó», el informe en singular; el interruptor encendido, los otros dos porqués de «no empezó», el aviso de un idioma que este Mac no transcribe y el idioma en inglés—; y el chip del rail pasa de «Sin reunión · 0 B» a «Sin sesión · 0 B», el de las otras siete pantallas: segunda vuelta de texto, sin parada) · el rail de `sesion.html`, `permisos.html`, `corpus.html`, `notas.html`, `honestidad.html`, `idioma.html` e `ia.html` gana «Ensayo» bajo Sesión · `honestidad.html` (las vistas «sprint 3» cuentan 12 de 12: el corte corta también el ensayo) | **Maquetada, no vista** (forma y texto, método v1.38.0). Va al gate del MVP del H2. Fidelidad: 260 encuadres —siete del ensayo—, ninguno sobre el umbral | ADR 019 · bitácora, fase 3 |
| 2026-10-04 | **Fase 4 del sprint 004 — forma y texto** — `ensayo.html` (cuatro estados nuevos: **1c · guardado** —«Ensayo guardado con tus notas.» encima de «preparar», y en «Con quién ensayas» la fila «Ensayos guardados con este cliente · 4 · Ver tu progreso», que solo aparece con uno o más—; **6 · tu progreso** gana abajo «Volver» y «Borrar los ensayos de este cliente»; **6b · borrar** —la pregunta antes de borrar, como en Notas, con lo de Time Machine—; y **10 · textos de lo guardado** —el desbloqueo cancelado, guardar o exportar que fallan, la retención «siempre», un cliente sin ensayos, una cifra que no cambió y los meses de la tabla—) · `honestidad.html` (un estado nuevo, **sprint 4 · ensayando**: el micrófono es el del ensayo, la pista del sistema, el transcript y la pantalla en 0 B, la fila nueva «Tus respuestas del ensayo» y el chip «Ensayando»; y «Tuyo» se acorta a «notas, acuerdos, fichas fijadas, propuestas guardadas, ensayos y, si lo enciendes, tus turnos. Cifrado.» —en inglés, «…rehearsals and, if on, your turns»— porque con «tus ensayos» ocupaba dos líneas y tres estados con franja se salían 17–20 px de su ventana; entra igual en los estados «de hoy» del sprint 3, y «sprint 3» queda como historia) | **Maquetada, no vista** (forma y texto, método v1.38.0). Va al gate del MVP del H2. Fidelidad: 280 encuadres —el informe, guardado, tu progreso, borrar y Honestidad ensayando, nuevos—, ninguno sobre el umbral; la primera corrida cazó «161 ppm → 138 ppm» frente a «161 → 138 ppm» de la maqueta | ADR 015 enmienda 4 · bitácora, fase 4 |
| 2026-10-04 | **Auditoría del sprint 004 (Fase 2) — forma y texto** — `ensayo.html` (**4 · evaluada**: la marca «sección conjeturada» en una ficha; **8 · textos**: dos porqués más de «el modelo no sumó»; **9 · más textos**: «Hay una videollamada abierta» y «No se puede saber si hay una videollamada», y el idioma del ensayo sin «el de la propuesta»; **10 · textos de lo guardado**: «No se borraron tus ensayos») · `sesion.html` (estado nuevo **sprint 4 · ensayo sin guardar**: la franja en una línea bajo «Iniciar sesión»; y en los nueve estados, la protección «verificado… con la banda abajo; con la banda arriba, y en Zoom y Teams, está sin verificar») · `ia.html` («Se cerró sola: hay una reunión o un ensayo», «en reunión o en un ensayo se cierra sola», «en reunión o ensayo»; en **sprint 3 · lo que salió**, la fila «enriquecer el banco · Propuesta Páramo Azul», «Las 3 peticiones recientes» y el registro que se borra al terminar la sesión) · `honestidad.html` (**sprint 4 · ensayando**: «salieron de tu equipo en este ensayo») · `corpus.html` e `idioma.html` («Lo que no hace hoy», sin el chip «En el H2») · `ghost.css` (`.ayuda-e` con `.tras`, `.pegada` y `.rotulo`, `.tecla.chica` y `.evidencia-e .conjetura`: los estilos sueltos pasan a clases, mismo tamaño) | **Maquetada, no vista** (auditoría A1, M10, M14, M19, M20, M21, B14, B22, B27, B29, B33 y B46; las decisiones A1, M10, M20 y B46 son del usuario, 2026-10-04). Va al gate del MVP del H2: filas U14 a U18 de la guía | `sprints/SPRINT_004-auditoria.md` · bitácora, Fase 2 de la auditoría |
| 2026-10-04 | **Segunda pasada de la casilla 4 del S4 — texto, segunda vuelta** — `banda.html` (con la banda arriba, el chip «Meet · sin verificar» en ámbar en vez de «Meet · protegido», con `.solo-abajo` y `.solo-arriba`) · `sesion.html` (en los cuatro estados con «La banda: arriba», la tarjeta de la reunión dice «Meet · sin verificar» en vez de «Protección verificada») · `posicion.html` (arriba a 88 y 200, «Meet · sin verificar»; la nota de qué ve el cliente y la de la banda que flota) · `ensayo.html` (**preparar**: «con una videollamada abierta, el ensayo no empieza por los altavoces del Mac» y «Si lo guardas, queda tu respuesta en texto…») · `ia.html` («Esta reunión o ensayo») | **Maquetada, no vista** (A5, A6, M37 y B53 de la segunda pasada; segunda vuelta de las decisiones A1 y M20 del usuario, sin parada). Va al gate del MVP del H2: filas U13, U14, U16 y U17 de la guía | `sprints/SPRINT_004-auditoria.md` (casilla 4, segunda pasada) |
| 2026-10-05 | **Fase 0 del sprint 005 — forma y texto** (ADR 020) — `banda.html` (tres estados nuevos: **sprint 5 · presencial** —«Escuchando · la sala» y el chip «Presencial · la sala» donde iba «Meet · protegido»; sin acople—, **presencial · al oído, la sala habla** —la píldora de 44 px: «La sala habla · te la digo cuando calle», con ⌃⌥V «verla»— y **presencial · transcript** —cada turno dice «sala»—) · `honestidad.html` (**sprint 5 · en presencial**: una pista en memoria, el sistema y la pantalla «no se abren en presencial» en 0 B, el transcript «de la sala, sin dueño», y al cerrar «Tus turnos, no» y «De la sala… sin propuestas») · `idioma.html` (**sprint 5 · el idioma de la sala**: la tercera fila de «Idioma por pista», con su consecuencia debajo) | **Maquetada, no vista** (método v1.38.0: FORMA y TEXTO no abren parada). Va al gate del MVP del H2. `maqueta-cabe` cazó al nacer dos desbordes (Idioma +113 px, Sesión +11 px) que el arnés de capturas no veía; se pagaron y el arnés aprendió a mirar el área que desplaza (bitácora, fase 0) | ADR 020 · ADR 009 enmienda 1 · ADR 002 enmienda 9 |
| 2026-10-05 | **Mirada de DECISIÓN del sprint 005** — `sesion.html`, estados **sprint 5 · presencial** y **sprint 5 · presencial · en marcha** (cómo eliges el modo · el idioma de la sala · cómo ves la ficha: al oído de fábrica, la banda a una tecla · «Lo tuyo queda: tus turnos · apagado» · la pista única con el sistema y la pantalla cerrados a propósito · la franja del PC con Windows al lado) | **Aprobada** — «Si la abri y la apruebo, sigue» (2026-10-05). *(Primero llegó un «sí» a secas; se le repreguntó qué vio, regla 10, y respondió con la fórmula «la abrí y la apruebo»: llegó con el archivo abierto. Confirma la respuesta (a) de la pregunta 2 del G-Plan: la voz al oído de fábrica, la banda a una tecla.)* | Fase 1 del sprint 005: el núcleo presencial (ADR 020) |

## Decisiones de diseño declaradas antes del segundo artefacto

D1–D13 en `sprints/ETAPA-DISENO-implementation-log.md` (tamaño 380 × 220, esquina superior
derecha, opaco, Charter · Avenir Next · Menlo, iconografía SVG sin emojis, motion casi nulo,
bilingüe pareado, datos sintéticos «Páramo Azul», estructura de archivos).

## Pantalla → funcionalidad de la VISION

Contrato de la orden: las **17 funcionalidades `[MVP · personal]`** de la VISION v1.2.0 más las
**2 `[MVP · terceros]`**, ninguna sin pantalla y ninguna pantalla sin funcionalidad. A esas se
suman **tres nacidas en las miradas** (C14 ya estaba en la orden; **C15 y C16 son nuevas** y la
planeadora debe absorberlas: ver `## Desviación del plan` en la bitácora).

| # | Pantalla | Estados | Funcionalidades |
|---|---|---|---|
| 01 | `panel.html` | esperando · buscando · ficha · sugerencia local · sugerencia API · sin resultado · sin verificar · radar · radar invasivo · transcript · voz · voz sin auriculares | **C1** ventana protegida · **C5** disparo y atajo · **C6** fichas de evidencia · **C7** sugerencia · **C8** lectura de pantalla · **C14** radar · **C15** modo solo audio · **B2** contador de red |
| 01-b | `posicion.html` | A · B · C (descartada) · D · D2 · D3 · F · E · **ARRIBA** (sprint 004: 88 acoplada, la elegida · ampliada · solo audio · qué ve el cliente · sin reunión, flota) | **C1** — decide forma y posición: banda acoplada 88/200, solo audio 44, y el relleno de la franja; **C1'** la banda arriba, junto a la cámara (mirada de DECISIÓN 2 del S4) |
| 01-c | `banda.html` *(sprint 001)* | esperando · buscando · ficha · ficha ampliada · sin resultado · sin verificar · sin verificar ampliada · transcript · sin acople | **C1** forma × contenido · **C3** transcript por pista · **C6** ficha y acumuladas — referencia del gate de FIDELIDAD |
| 02 | `sesion.html` | detectada · sin reunión · NDA (solo notas) · vigilancia local | **C2** dos pistas · **C11** jurisdicción y NDA · **C12** detección de cliente · **C14** radar local |
| 03 | `permisos.html` | sin conceder · concedido · revocado a mitad · solicitando · acople | **C8** consentimiento de pantalla · **C12** micrófono, audio del sistema y acople |
| 04 | `corpus.html` | vacío · indexando · con documentos · ilegible | **C4** ingesta e índice · **B1** cinco unidades |
| 05 | `notas.html` | durante · bandeja · propuestas · al cerrar · archivo cifrado | **C9** anotar, fijar, acuerdos, cierre y retención |
| 06 | `honestidad.html` | sesión activa · tras kill-switch · verificación · al cerrar | **C10** efímero verificable · **C12** kill-switch · **B2** contador |
| 07 | `idioma.html` | oculto · visible · varios idiomas · diccionario | **C3** transcripción · **B3** diccionario técnico es/en |
| 08 | `ia.html` | local · API apagado · API encendido · kit de evaluación · Claude Code | **C7** proveedor y costo · **C13** kit de evaluación · **C16** puerta local · **B2** contador |
| 09 | `kit.html` | 11 secciones de componentes | la fuente de verdad visual: `design-system.md` (hoy v1.16.1) |
| 10 | `ensayo.html` *(sprint 004)* | preparar · no empezó · guardado · preguntando · del modelo · respondiendo · evaluada · el informe · tu progreso · borrar · sin corpus · textos · más textos · textos de lo guardado | **C18** el ensayo — mirada de DECISIÓN 1 del S4 |

**Cobertura:** 19/19 (17 MVP personal + 2 MVP terceros) + C14 + **C15 y C16, nuevas**. Ninguna
funcionalidad quedó sin pantalla; ninguna pantalla existe sin funcionalidad que la justifique.

## Qué promete el panel, por cliente — spike del usuario (2026-09-20)

Copiada del spike (`investigacion/2026-09-20-spike-invisibilidad.md` de la planeadora). La
promesa de invisibilidad es **graduada**: solo se afirma lo verificado, con fecha y versión.

| Cliente | Resultado | Fuente |
|---|---|---|
| **Google Meet (Chrome)** | **INVISIBLE** — la ventana con `sharingType = .none` no aparece en la pantalla compartida; la de control sí | usuario, 2026-09-20, macOS 26.6.2: «perfecto, la roja no se visualiza» |
| Zoom (app) | **NO PROBADO** | decisión del usuario |
| Microsoft Teams (app) | **NO PROBADO** | ídem |

La maqueta lo respeta literalmente: `panel.html` tiene el estado **«sin verificar en este
cliente»** con el camino alterno (compartir ventana · segundo monitor · modo solo notas), y en
ningún sitio dice que funcione donde no se probó.

**Paradas del gate de prueba en llamada real** (nacidas en esta etapa, no verificables aquí):

1. Zoom y Teams — la promesa graduada del panel.
2. Que la captura componga el **relleno** de la franja y no la banda (mirada 3-quinquies).
3. Que el **acople** devuelva la ventana de la reunión a su sitio al cerrar (mirada 3-quater).

## Auto-auditoría (checklist §4 del skill `diseno-ui`, por tema)

| Criterio | Oscuro | Claro | Evidencia |
|---|---|---|---|
| Contraste AA en todo texto | ✓ | ✓ | medido nodo a nodo en cada estado × tema × idioma; **0 bajo AA** |
| Estado = símbolo + texto + color | ✓ | ✓ | tabla §4 del design system; pasada **deutan** sobre corpus, IA y panel |
| Jerarquía de un golpe de vista | ✓ | ✓ | el panel se lee en titular · línea · fuente; nada compite |
| Sin color como único portador | ✓ | ✓ | anti-patrón con gate: `rojo vs. verde` jamás solo |
| Motion mínimo y reduced-motion | ✓ | ✓ | §3.5; la forma del árbol nunca depende del hook |
| Tipografía con roles, no genérica | ✓ | ✓ | Charter evidencia · Avenir Next interfaz · Menlo medible |
| Iconografía propia, cero emojis | ✓ | ✓ | sprite de 38 glifos + **gate en `pnpm test`** (nació en la Fase 5) |
| Todo cabe en su ventana | ✓ | ✓ | 960 × 640 y 380 × 220 sin scroll: **0 desbordes** medidos |
| Bilingüe real, no pendiente | ✓ | ✓ | textos pareados `lang` en toda pantalla de producto |
| Datos 100 % sintéticos | ✓ | ✓ | «Páramo Azul», «Sur del Valle»; cero nombres reales |

**Deuda declarada:** el chrome de la sala de diseño (barra, notas «qué mirar», `index.html`,
`posicion.html`) está **solo en español**. Es andamiaje, no producto, y no viaja al código; se
declara para que nadie lo lea como una traducción pendiente.

## Gates que nacieron en esta etapa (con su demo en rojo)

Regla 15 del pipeline, las tres preguntas: *¿lo viste fallar? · ¿lo viste correr? · ¿puede
fallar?* Los tres corren en `pnpm test`, es decir en el job `quality`.

| Gate | Qué impide | Demo en rojo |
|---|---|---|
| `vocabulario-vetado` | los términos de ocultamiento vetados por la regla dura 6 —«indetectable», «stealth», «trampa» y equivalentes— en maqueta, README y design system <!-- vocabulario:cita --> | plantado «indetectable» en `panel.html` (Fase 0) y en `ia.html` (Fase 4) ⇒ rojo nombrando el archivo. Excepción nominal declarada para `anti-cheat`/`anti-trampa`: el término suelto sigue en rojo (demostrado) |
| `maqueta-autocontenida` | `http(s)://`, `<script src>`/`<link href>` externos, `@import url(` y dominios de despliegue en `docs/diseno/**` | plantado un CDN ⇒ rojo (Fase 0) |
| `maqueta-sin-emojis` | emojis como iconografía en el markup y los estilos de la maqueta | plantado 🔒 ⇒ rojo; y `✅` ⇒ rojo, demostrando que la excepción nominal de `✓` **no abre su bloque** (Fase 5) |

## Registro de G-Diseño

**APROBADO — 2026-09-20.** Veredicto del usuario sobre la maqueta completa abierta en su Mac:

> «sí apruebo la pantalla completa»

**Qué queda aprobado:** `design-system.md` v1.7.0 como **fuente de verdad visual** y la maqueta de
`docs/diseno/` (9 pantallas, 51 estados, dos temas, dos idiomas) como **contrato de forma**. Desde
aquí, toda pantalla de producto obedece a esta maqueta: el primer sprint con UI se detiene tras la
primera pantalla construida y presenta capturas comparadas contra ella (**gate de FIDELIDAD**,
indiferible — no viaja con el gate ⭐).

**Cómo se llegó:** **10 miradas** del usuario (plan de 5, ampliado cinco veces a petición suya,
cada ampliación propuesta y aprobada antes de construir). Ninguna pantalla se construyó encima de
un artefacto que él no hubiera abierto. Dos veces llegó solo la palabra de fase y se repreguntó;
la segunda vez él observó, con razón, que no tiene que rendir cuentas de lo que ve — la
repregunta se hizo porque esa pantalla aflojaba una regla dura, y eso debió decirse junto a la
pregunta, no después.

**Estado de cierre:** PR [#3](https://github.com/mauriciorincon-ai/app-copiloto-consultor/pull/3)
con conclusión propia `success` en `quality`, `e2e` y `build-escritorio`; barrido de CERO ENLACES
limpio; 9 gates verdes, los 3 nuevos vistos en rojo.

**Lo que esta etapa NO pudo verificar** (paradas del gate de prueba del primer sprint): Zoom y
Teams · que la captura componga el relleno de la franja · que el acople devuelva la ventana de la
reunión a su sitio al cerrar.
