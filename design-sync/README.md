# design-sync — el bundle publicable de Angel Ghost

Espejo en el repo de lo que se publica en Claude Design. **No se escribe a mano: se genera.**

```
design-system.md   →   design-sync/   →   el proyecto en Claude Design
(fuente de verdad)     (este bundle)      (vitrina; jamás se edita allá)
```

- **Regenerar:** `node scripts/design-sync-bundle.mjs`
- **Gate:** `tests/unit/design-sync-espejo.test.ts` — el bundle del repo tiene que ser el que el
  generador emite hoy. Si tocas `ghost.css`, la maqueta o el generador y no regeneras, el gate se
  pone en rojo con el archivo y la línea.
- **Publicar** no es trabajo de este bundle ni de este script: lo hace `/design-sync`, que **solo
  invoca el usuario**, en el cierre de ciclo y **después del gate ⭐⭐** — no se publica un sistema
  que el usuario no haya juzgado. El destino vive en `project.json` y hoy está en `null` porque
  nunca se ha publicado.

Cada tarjeta es **autocontenida** (CSS y sprite en línea, cero CDNs), lleva su primera línea
`<!-- @dsCard … -->` —por donde Claude Design la indexa— y enseña el componente en **los dos
temas**, con las dos lenguas dentro (se muestra el español).

| Grupo | Tarjeta | Archivo |
|---|---|---|
| Fundamentos | Tokens | `components/fundamentos/tokens.html` |
| Fundamentos | Estados | `components/fundamentos/estados.html` |
| Fundamentos | Anti-patrones | `components/fundamentos/anti-patrones.html` |
| Componentes | Ficha de evidencia | `components/componentes/ficha-de-evidencia.html` |
| Componentes | Contador de red | `components/componentes/contador-de-red.html` |
| Componentes | Estado de permiso | `components/componentes/estado-de-permiso.html` |
| Componentes | Bandera de jurisdicción | `components/componentes/bandera-de-jurisdiccion.html` |
| Componentes | Estado de sesión | `components/componentes/estado-de-sesion.html` |
| Componentes | Alerta del radar | `components/componentes/alerta-del-radar.html` |
| Componentes | Primitivas | `components/componentes/primitivas.html` |
| Componentes | Píldora de voz | `components/componentes/pildora-de-voz.html` |
| Componentes · S1 | La banda — seis estados | `components/s1/la-banda.html` |
| Componentes · S1 | «Todavía no» | `components/s1/todavia-no.html` |

Derivado de `design-system.md` **v1.11.0** y de la maqueta aprobada en G-Diseño
(`docs/diseno/`). Nace en el sprint 001, el primero con UI, por la regla 16 de `CLAUDE.md`:
el bundle se actualiza en el MISMO PR que toca la UI, para que el cierre de ciclo sea un delta
pequeño y nunca una reconstrucción.
