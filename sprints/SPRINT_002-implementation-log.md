# Bitácora — Sprint 002 «Cuándo, qué y quién mira»

Branch `sprint-002/cuando-que-y-quien-mira`, desde `main` en `7625683`. Orden
`SPRINT_002-orden.md` (aprobada 2026-09-23) · plan autoritativo `SPRINT_002.md` · kit v1.28.0 ·
método v1.30.0 · ciclo H1, sprint **2 de 3**.

Plan de fases aprobado el 2026-09-24, con su bloque de arranque y el «construye» del usuario.

---

## Desviación del plan — lo primero que pidió el usuario al arrancar

El plan que se aprobó traía una desviación numerada 4: **cerrar como «no reproducibles» los quince
hallazgos medios y bajos del sprint 001** cuyo detalle no existe en ningún archivo del repo. El
usuario la rechazó en el mismo mensaje del «construye», y con una razón que cambia el sprint:

> _«esto no puede pasar nunca más, los hallazgos se deben resolver al finalizar el sprint; siempre
> al finalizar el sprint se buscan hallazgos y siempre me caracterizo por resolver todos, hasta los
> bajos»_.

**Qué cambia, entonces:**

1. Los quince **no se cierran**. Sus superficies entran en el alcance de la `/audita-sprint` del S2,
   que por el kit v1.28.0 ya exige `archivo:línea` en todas las severidades: lo que siga siendo
   cierto volverá a salir con su ubicación, y se paga.
2. La fase 6 planea tiempo para **pagar también los medios y los bajos**, no para declararlos.
3. Y lo que impide que se repita **no es una buena intención, es un gate** — abajo.

---

## Fase 0 · El gate del artefacto de auditoría

### Por qué existe

El auditor del S1 entregó 21 hallazgos medios y bajos. Al escribirlos en el artefacto del repo se
resumieron en una sola frase —«**M3, M5, M6, M7, M8, M12, M13, M14** y **B1–B7**: ver el detalle en
la bitácora de ajustes»— con un puntero que además estaba **roto**: la bitácora tampoco los tenía.
Quince hallazgos desaparecieron con lo único que los hace pagables, que es dónde están.

`tests/unit/auditoria-con-sitio.test.ts` le exige dos cosas a todo `SPRINT_NNN-auditoria.md`:

1. **cada severidad declara su cuenta y la cuenta cuadra** — `## MEDIOS (14)` obliga a catorce
   filas, de `M1` a `M14`;
2. **cada hallazgo dice dónde** — un `archivo:línea` en su fila o dentro de su sección, y la única
   salida es marcarlo `irrecuperable` con su razón, que es una confesión escrita y no un silencio.

### Su rojo no hubo que fabricarlo: el artefacto del S1 estaba roto hoy

```
× sprints/SPRINT_001-auditoria.md — cada hallazgo dice DÓNDE, o se declara irrecuperable
  M11 · M2 · M4 · M9
× sprints/SPRINT_001-auditoria.md — la numeración no tiene huecos
  M3 (medio) · M5 (medio) · M6 (medio) · M7 (medio) · M8 (medio)
```

### Y el agujero que el propio gate tenía, encontrado al leer ESA salida

La primera versión buscaba los identificadores uno a uno (`**M12**`) y comprobaba que no hubiera
huecos entre los que encontrara. Mirando el rojo se ve lo que le faltó: denunció `M3` y `M5–M8`
—huecos por debajo del máximo que veía— **y no dijo ni una palabra de `M12`, `M13`, `M14` ni de los
siete `B`**. Estaban escritos dentro de una sola negrita agrupada, así que para ese gate no
existían; y como no existían, la numeración que él veía era continua y perfecta.

**Un gate que mide lo que el documento le enseña es un gate que el documento puede engañar.** La
versión que quedó cuenta contra **la cuenta que el propio documento declara de sí mismo**: si el
encabezado dice catorce, hay que enseñar catorce filas. Y por eso un encabezado sin cuenta —`##
ALTOS`, tal como estaba— también es rojo: un documento que no dice cuántos tiene no se puede cuadrar
con nada. El segundo rojo, ya con la regla nueva, sacó los diecisiete que faltaban y los dos
encabezados mudos.

Una tercera vuelta lo hizo menos rígido en lo que debía: `C1` vive como **sección** (`### C1 · …`)
porque un crítico merece prosa, y obligarlo a caber en una celda sería empujarlo a resumirse, que es
justo el defecto que este gate persigue. Ahora un hallazgo puede ser fila, viñeta o sección, y el
sitio se busca dentro de su trozo.

### Qué se arregló en el artefacto del S1

`## CRÍTICO` y `## ALTOS` ganan su cuenta —**(1)** y **(10)**—, y la sección de medios y bajos pasa
de una frase a **veintiuna filas**: las seis que tenían detalle con su `archivo:línea` y su estado
(M11 **pagado**; M1, M2, M4, M9 y M10 deuda con su fase de pago), y las quince restantes **marcadas
una a una como irrecuperables**, con una nota de cabecera que dice qué pasó y remite a la auditoría
del S2. Verde: 3 de 3, y la suite entera en 153.

**Lo que este episodio deja dicho:** el S1 escribió el gate de contrato porque un defecto se coló
entre dos lenguajes. Este es el mismo movimiento aplicado al método: el artefacto de auditoría
también es un puente —entre el auditor y el sprint que viene—, y también necesitaba su gate.
