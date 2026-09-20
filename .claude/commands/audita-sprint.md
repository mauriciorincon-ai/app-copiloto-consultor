---
description: Auditoría final de dos fases del sprint — OBLIGATORIA al concluir la construcción, antes de la guía de prueba/gate ⭐ y del cierre (método v1.10.0).
---

# /audita-sprint

Realiza la **auditoría final del sprint antes del cierre**. Es un paso OBLIGATORIO de todo
sprint (método v1.10.0): se ejecuta justo al concluir la construcción — ANTES de la
validación con `docs/GUIA-DE-PRUEBA.html`/gate ⭐ del usuario y ANTES del summary definitivo.

Trabaja en **dos fases** y **no modifiques nada hasta que el usuario apruebe la Fase 1**.

## Regla de modelos (léela primero)

**El modelo que audita PLANEA; cualquier modelo ejecuta.** La Fase 1 la corre un modelo
poderoso y su entregable más importante es dejar cada ajuste **tan bien definido — archivo,
línea, cambio exacto, criterio de verificación — que CUALQUIER modelo de menor capacidad
pueda ejecutarlo de la mejor manera sin ambigüedad**. La Fase 2 puede (y suele) correrse con
un modelo menor siguiendo ese plan al pie. El usuario decide el modelo de cada fase con
`/model` — este comando se lo recuerda al entregar el reporte.

## Quién audita la Fase 1 (kit v1.26.0)

**La Fase 1 la corre un auditor INDEPENDIENTE del constructor, con el diff del sprint
delante** (`git diff main...HEAD --stat` y luego archivo por archivo): una sesión nueva o un
subagente que NO construyó el sprint, sin la bitácora como única fuente — la bitácora dice
qué se creyó hacer; el diff dice qué se hizo. El constructor responde, no se audita a sí
mismo. *(Origen: hoja-de-vida S7 — los dos hallazgos más caros —un agujero del gate de
enlaces con host sin esquema y una ficha en el frente equivocado pasando el build— salieron
de revisar lo que el constructor daba por bueno.)*

## FASE 1 — Auditoría (SOLO LECTURA)

1. **Cobertura de alcance:** contrasta CADA ítem planeado (el plan aprobado del sprint +
   la orden de la planeadora) contra lo implementado. Clasifica cada uno como:
   **Completo / Parcial / No implementado / Implementado con desviación** — justificando
   cada clasificación con citas de archivos y líneas.
2. **Calidad de código** de lo desarrollado en el sprint:
   - **Correctitud:** bugs, edge cases no manejados, manejo de errores.
   - **Diseño:** acoplamiento, duplicación, responsabilidades mal ubicadas.
   - **Seguridad:** inputs sin validar, secretos expuestos, inyecciones.
   - **Tests:** cobertura de lo nuevo, casos faltantes, tests frágiles.
   - **Consistencia:** adherencia a las convenciones existentes del repo.
3. **Herramientas/dependencias:** señala SOLO librerías o patrones con una alternativa
   claramente superior (mantenimiento, seguridad, idiomática del stack) con justificación
   fuerte — jamás preferencias de estilo.
4. **¿QUÉ FRASES CADUCARON? (kit v1.15.0 — obligatorio).** Los pasos 1–3 auditan lo que el
   sprint **construyó**; este audita lo que el sprint dejó **falso**. Una feature nueva no solo
   añade pantallas: **vuelve mentira afirmaciones de las viejas.** Recorre los textos de alcance
   —portada/landing, `docs/MANUAL-DE-USO.md`, `README.md`, `docs/GUIA-DE-PRUEBA.html`, copy de
   estados vacíos, y el brochure si existe— y responde por escrito: ***¿qué afirmaba la app
   antes que ya no es cierto?***

   **CÓMO se busca (v1.15.2 — esto no es un detalle, es lo que decide si la casilla sirve):
   por PROMESA APLAZADA, jamás por la palabra de la feature.** Barre el vocabulario de la
   promesa, que es corto y estable: **`todavía no` · `aún no` · `por ahora` · `de momento` ·
   `mientras tanto` · `próximamente` · `llega después` · `en esta versión` · `más adelante` ·
   `no (se) puede` · `sin embargo` (+ el futuro: `podrás`, `permitirá`)**. Y revisa **cada
   coincidencia contra lo que la app hace HOY**, no contra lo que el sprint construyó.
   *(Origen: Velo S3 corrió esta casilla, dejó su inventario… y se le escaparon las dos frases
   vivas, porque buscó la palabra de la feature —`irreversible`— y las frases decían «no se
   puede **revertir**» y «el **camino de vuelta**». Al buscar por promesa aplazada salieron las
   dos, más una tercera de propina que llevaba **un sprint entero** caducada. El nombre de la
   feature cambia cada sprint; el vocabulario de la promesa, no.)*
5. **CAMPOS DEL CONTRATO SIN CONSUMIDOR (kit v1.15.0 — comprobación mecánica).** Por cada tipo
   de salida que el sprint creó o amplió (contratos de worker, tipos de resultado, payloads),
   **cuenta cuántos de sus campos tienen al menos un lector fuera de su propia construcción y
   sus tests** (`grep` del nombre del campo). Los huérfanos se reportan como hallazgo, no como
   nota: *un motor probado no es un producto probado — el defecto vive en el cable, no en la
   pieza.* *(Origen: Velo S2 — el reporte del tratamiento existía entero y probado **sin un solo
   llamador**: 6 de 9 campos huérfanos, invisible para 542 unitarias verdes. Esta comprobación
   lo habría encontrado en segundos.)*

**Entrega de la Fase 1:** un reporte con hallazgos clasificados por severidad
(**Crítico / Alto / Medio / Bajo**) y una recomendación explícita: **"listo para cierre"** o
**"requiere ajustes"**. Valida que todo quede documentado en los archivos correspondientes
(bitácora, ADRs, deuda declarada). Cada hallazgo Crítico/Alto debe traer su **ajuste
ejecutable**: archivo(s) y línea(s), cambio exacto propuesto, y el criterio observable de
"ajuste verificado" — el formato que un modelo de menor capacidad puede seguir sin pensar
de más. Cierra recordando al usuario: *"aprueba la Fase 1 y fija el modelo de la Fase 2 con
`/model` (un modelo menor basta si sigue este plan)"*.

## FASE 2 — Correcciones (SOLO tras aprobación del usuario)

1. Propón el **plan de ajustes para los hallazgos Crítico/Alto** (los Medio/Bajo se declaran
   como deuda con pago asignado, salvo que el usuario pida incluirlos).
2. **Espera la validación del usuario** del plan.
3. Solo entonces implementa — siguiendo el plan de la Fase 1 al pie; cualquier desviación se
   declara antes de ejecutarla.
4. Al terminar: registra en la bitácora y en el `SPRINT_NNN-summary.md` los hallazgos, los
   pagos y la deuda aceptada. **Sin auditoría registrada en el summary, el cierre del sprint
   queda condicionado** (lo verifica el `/cierre-sprint` de la planeadora).

## Posición en el flujo del sprint

construcción concluida → **`/audita-sprint`** (Fase 1 → aprobación → Fase 2) →
`/deploy-check` → summary (registra la auditoría) → PR → gate ⭐ del usuario → merge.
