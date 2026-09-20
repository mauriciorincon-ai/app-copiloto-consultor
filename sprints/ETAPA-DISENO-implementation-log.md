---
etapa: Etapa de Diseño (F2a)
app: copiloto-consultor
branch: diseno/fundacion
opened: 2026-09-20
status: open
orden: ~/Code/hr01-develop-ai-apps/portafolio/copiloto-consultor/ordenes/DISENO-orden.md
plan aprobado: 2026-09-20 (plan mode → «construye»)
---
# Etapa de Diseño — bitácora de implementación (Angel Ghost)

> Cero código de producto hasta G-Diseño. Entregables: `design-system.md` + `docs/diseno/`
> (maqueta navegable del H1, 8 pantallas) + `docs/diseno/README.md` con registro de miradas.

## Plan de miradas (declarado en el plan, aprobado 2026-09-20)

| Mirada | Artefacto(s) | Estado |
|---|---|---|
| 1 | `docs/diseno/panel.html` ⭐ (+ `ghost.css` y `design-system.md` borrador) | pendiente |
| 2 | `docs/diseno/kit.html` + `design-system.md` completo | pendiente |
| 3 | `sesion.html` · `permisos.html` · `honestidad.html` | pendiente |
| 4 | `corpus.html` · `notas.html` · `idioma.html` · `ia.html` | pendiente |
| 5 = G-Diseño | `index.html` + todo | pendiente |

Regla: «continúa» no aprueba diseño; cada mirada se registra en `docs/diseno/README.md`
ANTES del siguiente artefacto. Cambios al plan de miradas se aprueban antes de construir.

## Decisiones de diseño que la orden no escribió (D1–D13, del plan aprobado)

| # | Decisión |
|---|---|
| D1 | Panel flotante 380×220 (compacto) · 380×420 con transcript · hasta 380×360 con 2–3 fichas; nunca más ancho |
| D2 | Posición por defecto: esquina superior derecha, 16 px del borde, 8 px bajo la barra de menús; todos los Spaces; posición recordada |
| D3 | Opaco, sin blur ni «glass» — cuaderno, no overlay |
| D4 | Tipografía nativa de macOS sin embeber bytes: Charter (evidencia) · Avenir Next (interfaz) · Menlo (cifras, fuentes, contador; `tabular-nums`) |
| D5 | Panel: titular 17 px · línea 14 px · fuente/estado 12 px (Menlo 11.5 px) |
| D6 | Ventana principal 960×640 (mín. 800×560), rail izquierdo fijo |
| D7 | Personalidad: discreto · propio · verificable / nunca ruidoso, nunca «stealth» (vocabulario:cita), nunca corporativo |
| D8 | Oscuro primario grafito cálido + tinta marfil; claro = papel; un solo acento frío «halo»; estados ok=verde-azulado · atención=ámbar · error=coral · inactivo=gris; jamás rojo vs. verde como única distinción |
| D9 | Iconografía SVG de trazo 16 px monocromo, inline; cero emojis |
| D10 | Motion casi nulo: fade 150 ms sin desplazamiento; «buscando…» estático; reduced-motion ⇒ cero transiciones; la forma del árbol nunca depende del motion |
| D11 | Bilingüe en la maqueta: `<span lang>` pareados, conmutador es/en, `lang` del documento cambia |
| D12 | Datos sintéticos: «Comercializadora Páramo Azul S.A.S.» · «Tablero de rentabilidad por canal en Power BI» · «Adopción de datos en 4 etapas» · «Cooperativa Sur del Valle» · perfil genérico; jurisdicciones Colombia · Florida · UE |
| D13 | `docs/diseno/{index,panel,sesion,permisos,corpus,notas,honestidad,idioma,ia,kit}.html` + `assets/ghost.css` + `assets/maqueta.css` + `assets/maqueta.js` |

## Fase 0 — Setup (2026-09-20)

- Branch `diseno/fundacion` desde `main` (13cc7e5).
- Carpetas `docs/diseno/assets/` y `sprints/`.
- **Arnés de capturas** en el scratchpad de la sesión (`capturas.mjs`, NO versionado): abre cada
  `docs/diseno/*.html` por `file://`, recorre estado × tema × idioma según los botones de la
  barra de maqueta, captura a tamaño real y mide contraste WCAG por nodo de texto. Declara al
  arrancar el árbol que lee y aborta si se le pasa una ruta fuera de `docs/diseno/` (regla 17-bis b).
- **Gates nuevos, demo en rojo en el MISMO commit (regla 15):**

| Gate | Puede fallar (3ª pregunta) | Demo en rojo | A quién nombró | Verde al revertir |
|---|---|---|---|---|
| `tests/unit/vocabulario-vetado.test.ts` | Sí: eslint solo mira `src/`; nada más lee `docs/` | plantado «indetectable» (vocabulario:cita) en `docs/diseno/panel.html` | `docs/diseno/panel.html:3  «indetectable»` | ✓ 7/7 |
| `tests/unit/maqueta-autocontenida.test.ts` | Sí: ninguna regla previa inspecciona recursos externos en HTML | plantado `<script src="https://cdn…">` (autocontenida:cita) en `panel.html` | `docs/diseno/panel.html:2  URL absoluta` + `script externo` | ✓ 7/7 |

  Ambos corren en el job `quality` vía `pnpm test` (vitest incluye `tests/unit/**`).
- **Arnés de capturas — su propia demo en rojo:** un HTML temporal con texto `#333` sobre `#111`
  lo puso en rojo (`p «texto apagado» 1.49 < 4.5`) en los 4 combos tema × idioma; el HTML se
  borró. El abort fuera del árbol también se vio: `ABORTO: …/MANUAL-DE-USO.md no está dentro
  de …/docs/diseno`. Requirió `pnpm exec playwright install chromium` (1.63 pide el
  headless-shell 1243; la máquina tenía 1228/1234).
- Verificación local: `pnpm typecheck` ✓ · `pnpm lint` ✓ · `pnpm test` 3 archivos / 7 tests ✓.
- `docs/diseno/panel.html` existe como placeholder vacío para que los gates tengan qué leer
  (un gate que no lee nada no es un gate — el test lo exige).

## Fase 1 — El panel ⭐ (2026-09-20)

**Construido:** `docs/diseno/assets/ghost.css` (tokens ambos temas + componentes del panel) ·
`assets/iconos.js` (sprite SVG propio, 36 glifos, cero emojis) · `assets/maqueta.css` +
`assets/maqueta.js` (barra de la sala: estado · tema · idioma; escritorio de referencia a píxel
real con Meet detrás) · `docs/diseno/panel.html` (9 estados: esperando · buscando · ficha nueva ·
sugerencia local · sugerencia API · sin resultado · cliente sin verificar · alerta del radar ·
transcript visible) · `design-system.md` v0 (borrador) · `docs/diseno/README.md` (registro de
miradas abierto).

**Pasada de capturas (arnés, Chromium):** 9 estados × 2 temas × 2 idiomas = 36 capturas leídas
como imagen. Hallazgos corregidos ANTES de presentar:
1. `[hidden]` no vencía a las clases con `display:` → todos los estados se pintaban a la vez
   (la primera pasada lo mostró). Fix: `[hidden]{display:none!important}` en `ghost.css`.
2. Pie desbordado («corta todo» cortado) → copy más corto: `Meet · protegido` / `⌥⎋ corta`.
3. Ficha con línea de 3 renglones empujaba la fila «+2» fuera del panel → línea ≤ 2 renglones
   (copy recortado + `line-clamp: 2` en titular y línea).
4. Sugerencia API con cabecera de 2 renglones y texto cortado → cabecera `Claude Haiku · API`,
   «texto anonimizado» al tooltip de la nube (con letra vive en Ajustes de IA y Honestidad).
5. Contador de red en API: `1,2 KB` (el nº de peticiones vive en Honestidad).
6. Franja «sin verificar»: lista vertical `→` con los tres caminos alternos; párrafo a 1 renglón.
7. Transcript: columna de pista 104 px (antes solapaba «cliente 14:01» con el texto).
8. Tema claro: `--ok` `#157a62` daba 3.96 sobre su tinte → `#0f6350` (5.25).

**Contraste medido:** 36 combinaciones, **0 textos bajo AA**; peor caso 5.25 (claro), 6.14
(oscuro). Simulación deutan: el filtro del arnés no se aplicó en la captura de elemento —
se corrige en la Fase 2 (no es gate; los pares ámbar/verde-azulado/coral difieren además en
luminosidad y todo estado lleva símbolo + texto).

**Gates:** `pnpm test` 7/7 ✓ (vocabulario + autocontenida + dependabot) · `pnpm typecheck` ✓ ·
barrido de CERO ENLACES: un hit **preexistente en `main`**, `CHANGELOG.md:110` del kit escribe
`pages.dev` en claro — se corrige en la Fase 5 (escribirlo con clase de carácter).

**Decisiones tomadas en la construcción (a juzgar en la mirada 1):**
- D14 · El transcript visible es un **estado** del panel (380 × 420), no una ventana aparte.
- D15 · El estado «cliente sin verificar» ocupa el cuerpo entero (no convive con una ficha):
  antes de compartir pantalla no hay ficha que mostrar.
- D16 · El contador de red con API se pinta en el acento (`--halo`), no en ámbar ni coral:
  fue decisión del usuario; el ámbar se reserva para lo que él no decidió (radar, sin verificar).
- D17 · «Confianza» de la sugerencia: símbolo ◐ (media) / ● (alta) / ○ (baja) + palabra.

**Mirada 1 (2026-09-20): APROBADA** — «El panel se ve muy bien aprobado». Nota de proceso: la primera respuesta fue «Continua» (palabra de fase); se detuvo la construcción y se repreguntó «¿qué viste al abrirlo?» — la segunda respuesta sí delató la mirada. Registrado en `docs/diseno/README.md`.

## Fase 2 — El sistema completo + kit (2026-09-20)

**Construido:** `docs/diseno/kit.html` (10 secciones: tokens · estados · ficha de evidencia ·
contador de red · estado de permiso · bandera de jurisdicción · estado de sesión · alerta del
radar · primitivas · prohibidos; ambos temas, ambos idiomas) · `ghost.css` ampliado con la
ventana principal (rail, contenido, tarjeta, campo, conmutador, tabla, permiso, bandera,
contador grande, buffer, kill, progreso, diccionario, unidad) · `design-system.md` **v1.0.0**
(tabla de contraste por token calculada sobre los hex, vetados, contrato con el código).

**Pasada de capturas:** kit × 2 temas × 2 idiomas (página completa) + 10 secciones de cerca en
oscuro. Corregido antes de presentar:
1. `.bandera`: los hijos caían en la columna del icono (20 px) — una palabra por renglón. Fix:
   `.bandera > :not(.ic) { grid-column: 2 }`.
2. Sección de fichas: marcos de 380 px desbordaban una grilla de 3 columnas → grilla
   `minmax(380px)` y marcos `max-width: 380px`.
3. Tema claro: `--warn` `#8a5b00` (4.49) y `--err` `#b8322a` (4.45) sobre su tinte, bajo AA
   por centésimas → `#7a4f00` (5.45) y `#a32a22` (5.40).
**Contraste medido:** kit 313 textos × 4 combinaciones → 0 bajo AA (peor 4.95 claro /
5.08 oscuro). Arnés: simulación deutan corregida (filtro sobre el elemento capturado) y
capturas de página completa; nuevo `seccion.mjs` para leer secciones de cerca.

**Gates:** `pnpm test` 7/7 ✓.

**Mirada 2 (2026-09-20): APROBADA** — «Si apruebo el kit me gusto mucho muy oportuno el diseño
y elementos». Registrada en `docs/diseno/README.md`. Modelo para las Fases 3–4 fijado por el
usuario: Opus 5 (1M context).

## Fase 3 — Pantallas de sesión (2026-09-20)

**Construido:** `docs/diseno/sesion.html` (4 estados: reunión detectada · sin reunión · NDA
prohíbe transcribir · vigilancia en tu Mac) · `permisos.html` (4: sin conceder · concedido ·
revocado a mitad · consentimiento de pantalla) · `honestidad.html` (3: sesión activa · tras
kill-switch · verificación en verde). Las tres sobre el shell `.ventana` + `.rail` del kit ya
aprobado; cero componentes nuevos.

**Pasada de capturas:** 11 estados × 2 temas × 2 idiomas = 44 capturas leídas como imagen.
Corregido antes de presentar:
1. **El contenido no cabía en 960 × 640.** Compactado el sistema (no la pantalla): `.contenido`
   20/24 px, `.tarjeta` 12/16, `.grid-*` gap 12 + `align-items: start`, `.franja`, `.permiso` y
   `.bandera` más ajustados. Nuevo chequeo en el arnés: avisa si `.contenido` desborda su alto.
2. **Defecto del ARNÉS, no del diseño:** la barra sticky de la sala tapaba el borde superior del
   elemento capturado (el h1 salía cortado en todas las pantallas). Se neutraliza `position:
   sticky` solo durante la captura. Lección: un defecto de la herramienta de verificación se lee
   igual que un defecto del producto — se distingue mirando el HTML, no la captura.
3. Copy recortado en la tarjeta de Meet y en el chequeo de NDA para evitar reflujos feos.

**Decisiones de la construcción (a juzgar en la mirada 3):**
- D18 · El rail es el mismo en las 7 pantallas y se genera desde una plantilla única (no se
  copia a mano) — en producto será un componente.
- D19 · «Honestidad» no es una pantalla de ajustes: es un **estado de cuentas**. Buffers con su
  tamaño real a la izquierda, contador grande a la derecha, y lo único que persiste abajo.
- D20 · El mismo layout se reusa tras el kill-switch (todo en 0 B, tachado): la promesa se
  verifica comparando **la misma pantalla antes y después**.
- D21 · La verificación en verde muestra la fila «una fuga plantada hace fallar la prueba · se
  vio fallar · 2026-09-19»: la regla 15 del método, hecha interfaz para el usuario.

**Contraste medido:** sesión 16 capturas · permisos 16 · honestidad 12 → **0 textos bajo AA**.
Panel y kit re-medidos tras compactar el sistema: 0 bajo AA.

**Gates:** `pnpm test` 7/7 ✓.

**Mirada 3 (2026-09-20): APROBADA CON DOS CAMBIOS** — «en honestidad está bien que lo del
cliente se elimine no le veo problema pero lo que sí quiero es que me quede lo que es mío o lo
que dije o escribí, adicional quisiera tener un modo solo audio que me hable de forma paralela
por si quiero ver completamente la pantalla y no me interrumpa, todo el resto lo veo muy bien».
Ambos cambios son de PRODUCTO, no de estilo → ver «Desviación del plan» abajo.

## Desviación del plan (2026-09-20) — DOS CAMBIOS DE PRODUCTO pedidos en la mirada 3

> La planeadora es read-only: esto queda aquí y se avisa al usuario. **La VISION v1.2.0 y el
> `brief.md` necesitan actualización**, y la regla dura 1 del `CLAUDE.md` de esta app también.

### A · «Que me quede lo que es mío o lo que dije o escribí»

**Qué pidió:** que el transcript del cliente muera (lo aprueba), pero que **lo suyo** —lo que él
dijo y lo que escribió— se conserve.

**Qué cambia:** la regla dura 1 del `CLAUDE.md` dice hoy «Lo único que persiste: notas de texto
escritas por el consultor». Pasa a ser: **notas + los turnos transcritos del propio consultor
(pista de micrófono) + las fichas que se le mostraron**, todo cifrado y con retención.

**Por qué es compatible con el estándar 4-T** (`estandares.md`, apartado 4-T): su regla es
«audio, transcript y capturas **de terceros** viven en memoria y mueren al cerrar; nada **de
terceros** se persiste sin autorización previa». Lo que el consultor dijo con su propia voz no
es dato de un tercero: es suyo, igual que sus notas. La app era **más estricta que el estándar**
por decisión de diseño, no por obligación.

**Qué NO cambia (las líneas que se mantienen duras):**
- El **audio** de ambas pistas sigue muriendo siempre. Se conserva **texto**, jamás sonido.
- El transcript del **cliente** muere: no se guarda, no se exporta, no se cita textualmente.
- Cero huellas de voz, cero biometría, cero emociones.
- Lo conservado nace **cifrado**, con retención y borrado, en la carpeta del usuario.
- Es **opt-in**: se enciende en Honestidad; por defecto sigue siendo solo las notas.

**Riesgo residual declarado:** en un turno propio el consultor puede repetir datos del cliente
(«entonces su margen de canal es del 12 %»). Queda bajo su responsabilidad, igual que sus notas
— es exactamente el estatuto legal de A10 del informe legal-ético (el consultor es Responsable
de esa base de datos). La pantalla lo dice con letra, no en letra pequeña.

### B · «Un modo solo audio que me hable de forma paralela»

**Qué pidió:** poder ver la pantalla completa sin que el panel le quite espacio ni lo
interrumpa, y que la app le **hable**.

**Qué es y qué NO es:** lee **la misma ficha** (titular ≤ 8 palabras · línea · fuente) por el
oído en vez de por el ojo. **No es X5** (guion completo para leer en voz alta), que sigue
descartado: no dicta libretos, no redacta lo que él debe decir. El contenido no cambia de
tamaño ni de tono: cambia de canal.

**Funcionalidad nueva → la VISION pasa de 25 a 26.** Propuesta de ficha: **C15 · Modo solo
audio (voz al oído)**. `[MVP · personal]`. Código primero: `AVSpeechSynthesizer` de macOS, voz
on-device, cero red, cero costo, sin LLM — el texto que lee es el mismo que ya produce C6.

**Tres riesgos duros que el diseño resuelve, no advierte:**

| # | Riesgo | Salvaguarda de diseño |
|---|---|---|
| 1 | **Sin auriculares, el cliente oye la sugerencia** por los parlantes: rompe la promesa central de la app | El modo **exige auriculares**: la app comprueba la ruta de salida y **no habla** si no los detecta. No es un aviso, es una condición de arranque; si se desconectan a mitad, **calla** y lo muestra |
| 2 | **Bucle parlante → micrófono** (precedente citado en el `CLAUDE.md`: habla S3): su voz entra al mic y se transcribe como si él hablara, disparando fichas falsas | Mientras la app habla, el disparador se silencia y esa porción de la pista se descarta. Visible como estado |
| 3 | **La voz es intrínsecamente activa** — se puede no mirar un panel, no se puede no oír una voz ([S12][S14]: lo activo rompe el flujo) | Por defecto **solo habla cuando él lo pide** (⌘⇧A). El modo automático es opt-in y solo dispara en fin de turno del cliente; **jamás habla mientras alguien está hablando** |

**Dónde vive en la maqueta:** modo de arranque en `sesion.html` · estado nuevo del panel
(píldora mínima de 260 × 56, sin texto de ficha) en `panel.html` · componente «píldora de voz»
y «estado de auriculares» en `kit.html` · ajuste de voz en `ia.html` (Fase 4) · qué se conserva
en `honestidad.html`.

**Plan de miradas — cambio propuesto:** se añade una **Fase 3-bis** con su propia mirada (los
dos cambios, sobre `honestidad.html` y `panel.html`), ANTES de la Fase 4. Las miradas 4 y 5 no
se mueven. Total: 6 miradas en vez de 5.

## Fase 3-bis — los dos cambios de la mirada 3 (2026-09-20)

**A · Lo que es mío queda.** `honestidad.html` gana el estado **«al cerrar: qué queda»**: dos
columnas enfrentadas — *lo tuyo queda* (notas · tus turnos en texto · las fichas que fijaste,
cifrado, con el conmutador «Conservar lo que dije» que enciendes tú) frente a *lo del cliente
muere* (su voz · sus turnos · lo leído de la pantalla · **el audio de las dos pistas, el tuyo
incluido**). Franja con letra: lo que él repita de un dato del cliente queda en su archivo y es
su responsabilidad. El resumen del estado «sesión activa» se reescribió en la misma clave.

**B · Modo solo audio (C15).** Componente nuevo **píldora de voz** (`.pildora`, 260 × 56) que
**reemplaza** al panel. Tres estados: en silencio · hablando (halo) · sin auriculares (ámbar,
**no habla**). Vive en `panel.html` (2 estados nuevos), `kit.html` (sección 8-bis con las tres
variantes y las dos franjas de salvaguarda), `sesion.html` (tercer modo de arranque + la tarjeta
de pistas ahora comprueba auriculares) y `design-system.md` v1.1.0 (7.º componente canon, §9-bis
«qué persiste», dos prohibidos nuevos: guion para leer en voz alta · hablar por los parlantes).

**Bug de proceso encontrado y corregido:** un `str.replace` de Python sobre el CSS **falló en
silencio** (el ancla tenía otro salto de línea) y la píldora salió sin estilos; la pasada de
capturas lo vio, no el test. Desde entonces todo reemplazo va con `assert a in s` antes de
escribir — un reemplazo que no encuentra su ancla debe romper, no seguir. Es la misma clase de
fallo que la regla 15 persigue: algo que «pasó» sin haber hecho nada.

**Contraste medido:** panel 44 capturas (11 estados × 2 × 2) · kit 4 · sesión 16 · honestidad
16 · permisos 16 → **0 textos bajo AA**.

**Gates:** `pnpm test` 7/7 ✓.

**Mirada 3-bis (2026-09-20): DOS AJUSTES MÁS** — «quiero que en modo audio solo quede un icono
muy pequeño en la parte izquierda baja o tal vez una barra de lado a lado en la parte inferior,
de hecho qué tal si manejamos el panel en la parte inferior todo cuando haya audio y cuando no,
revísalo. No veo es una alerta de otro tipo de detectores que no es de grabación o de transcribir
sino esos como proctoring, anti-cheat, esos también deben ser detectados porque son invasivos».

## Fase 3-ter — posición del panel y radar invasivo (2026-09-20)

**C · ¿Dónde vive el panel? — se midió, no se opinó.** Nueva página `docs/diseno/posicion.html`:
el escritorio de referencia con **seis variantes** conmutables y los choques dibujados sobre la
pantalla real. Resultado:

| Opción | Choca con | Veredicto |
|---|---|---|
| A · tarjeta arriba der. (actual) | nada en Meet/Zoom/Teams | viable |
| B · tarjeta abajo der. | Dock · chat de Zoom | viable con reservas |
| C · banda inferior separada | **los controles de la llamada** | **descartada**: taparía el botón de colgar |
| D · banda pegada al borde | Dock | viable; obliga a recortar la frase con elipsis |
| E · **gota 44 × 44 abajo izq.** (solo audio) | nada | **lo que el usuario pidió, y la esquina más libre** |
| F · banda en modo solo audio | Dock | viable; más presencia que la gota |

**Decisión propuesta (D2 revisada):** la posición pasa a ser **elegible por el usuario** (cuatro
esquinas o banda inferior), recordada entre sesiones. Por defecto: tarjeta arriba a la derecha
con ficha · **gota abajo a la izquierda** en modo solo audio. Componentes nuevos: `.banda`
(ancho completo × 44) y `.gota` (44 × 44).

**D · El radar no cubría lo invasivo.** Tenía razón: solo había grabación de reunión y bots de
notas, ambos en ámbar. Ahora el radar tiene **dos niveles**:
- **ámbar «sábelo»** — legítimo y visible: la reunión se graba, hay un bot de notas, hay un MDM
  de empresa.
- **coral «invasivo»** — un programa del **propio** equipo que mira pantalla, cámara, teclas o
  procesos: **supervisión de exámenes (proctoring)** · **anti-trampa con acceso al núcleo** ·
  **monitoreo de empleados** · **acceso remoto activo**.

Cada fila declara **qué alcanza a ver**, no solo su nombre — es la columna que convierte el
aviso en una decisión. Vive en `sesion.html` (tabla de 5 categorías), `panel.html` (estado
nuevo `radar · software invasivo`, en coral) y `kit.html` (sección 7 con los dos niveles).
Nombres de la maqueta 100 % sintéticos; el catálogo real viaja versionado con su fuente y
**sin consultar la red** (regla dura 9: el radar jamás sondea hacia afuera).

**Contraste medido:** panel 48 capturas · sesión 16 · kit 4 · posición 24 → **0 bajo AA**.

**Gates:** `pnpm test` 7/7 ✓.

**Mirada 3-ter:** pendiente — mensaje de gate emitido.

## Fase 4 — Pantallas del cuaderno
(pendiente)
