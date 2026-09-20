# ADR-NNN — [feature] con IA generativa: por qué el código no alcanza

> **Plantilla del kit (v1.27.0).** La regla dura del pipeline (estándares §7, G-Metodo 2026-07-12)
> exige este ADR ANTES de activar cualquier feature que llame a un LLM. Copiar a
> `decisions/NNN-<tema>.md`, citar por TEMA (jamás por número) en órdenes y summaries, y enlazar
> desde el summary del sprint. Sin este ADR con las cinco secciones llenas, la feature no entra.

**Estado:** propuesto | aceptado | superado por ADR-NNN · **Fecha:** YYYY-MM-DD · **Sprint:** SNN

## 1. La funcionalidad, en una frase de usuario
<Qué hace por el usuario. Sin nombrar modelos.>

## 2. Lo que se intentó con CÓDIGO primero (obligatorio, con evidencia)
| Intento determinista | Qué resolvió | Dónde se quedó corto | Evidencia (test, métrica, kit de prueba) |
|---|---|---|---|
| <reglas / librería / algoritmo> | | | |

> Si esta tabla está vacía, el ADR no es válido: la regla es «código primero», no «código
> considerado».

## 3. Dónde entra el LLM y dónde NO
- **Entra en:** <la parte estrecha que el código no cubre — p. ej. redactar la síntesis de 3 fichas ya recuperadas>.
- **NO entra en:** <todo lo que sigue siendo determinista: disparo, recuperación, validación, persistencia>.
- **Entrada al modelo:** <qué texto/datos viajan, minimizados cómo; jamás audio/imagen cruda si hay terceros>.
- **Salida del modelo:** <esquema Zod; nunca texto libre directo a la BD o a la UI sin validar>.

## 4. Fallback determinista (obligatorio)
<Qué ve el usuario cuando el modelo no está (sin red, sin clave, timeout, salida inválida) — el
fallback ANUNCIA su motivo. La app debe ser útil sin el LLM.>

## 5. Proveedor, costo y privacidad
- **Orden de proveedores:** <on-device → self-host → API; el adapter del kit + `mock` como proveedor de primera clase>.
- **Costo medido:** <US$ por operación/reunión/día con el logger del kit; techo declarado (US$10/mes en etapa inicial)>.
- **Privacidad:** <qué se loguea (solo metadatos), retención del proveedor (no-retención exigida si hay terceros), anonimización previa>.
- **HITL:** <si el dominio es sensible, quién revisa qué antes de que llegue al usuario final>.

## Consecuencias
<Qué gana la app, qué deuda o riesgo acepta, qué gate ⭐ verifica la experiencia que la CI no ve.>
