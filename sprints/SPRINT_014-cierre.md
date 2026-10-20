---
sprint: 014 · app: copiloto-consultor · hito: H-2 · opened: 2026-10-19 · closed: 2026-10-19 · branch: sprint-014/la-sugerencia-sin-salir · pr: #31
---

# Sprint 014 · La sugerencia sin salir — cierre

**Resultado:** cumplido. En Windows la sugerencia la redacta un modelo local dentro del proceso (llama.cpp, Qwen2.5-1.5B Q4_K_M). Estado: listo para PR; tres decisiones tuyas pendientes (abajo).

| Medida                                 | Perfil dev (ADR 023 enm. 2)    | Perfil release (hoy)           | Criterio |
| -------------------------------------- | ------------------------------ | ------------------------------ | -------- |
| Carga · RAM inicio→cargado→pico→suelto | 1 499 ms · 7→1 112→1 227→18 MB | 1 402 ms · 8→1 112→1 224→14 MB | —        |
| Latencia mediana · p95                 | 2 105 · 2 321 ms               | 1 834 · 2 066 ms               | < 6 s ✅ |
| `fundar`                               | 12 de 17                       | 11 de 17                       | ≥ 9 ✅   |
| TCP · UDP del proceso, 5 fases         | 0 · 0                          | 0 · 0                          | 0 ✅     |
| Canaria                                | 0 archivos (42 mirados)        | 0 archivos (45 mirados)        | 0 ✅     |

Corrida en vivo en la app (3 corridas): 6 023 ms `Tarde` · 6 025 ms `Tarde` · 2 311 ms con sugerencia en la banda. Dentro del techo en 1 de 3: la fila 20 sigue «hecho sin verificar». Binario release: 21 694 464 B (20,7 MB). **Tiempo** (`node scripts/horas-sprint.mjs`): 11 commits · 2 sesiones · **4,2 h activas** (F1 1,3 h · F2 0,5 h · F3 2,4 h). Archivo: 4 h · plan: 5 h · diferencia +5 %.

**Entregables:** `modelo_local.rs` (estado, descarga por la puerta de whisper, carga y suelta) · `sintesis/local.rs` (cuarto proveedor) · Windows nunca elige el API · el corte suelta el modelo · IA y Honestidad en Windows · arnés `modelo-local-en-windows.rs` · ADR 023 enm. 2 y anexo · manuales ES/EN, guía `ag-w14-`, BLUEPRINT, README.

**Pendientes (decisiones tuyas; ninguna bloquea el PR):**

1. M-6 · `CLAUDE.md` líneas 15, 17 y 130 (tres frases falsas). Lo edito con tu «sí».
2. M-8 · brochure (`BROCHURE.html:381, :516, :599`; `brochure-export.json:90`): ¿lo pago ya o va al lote del hito (regla 13)?
3. B-5 · cuatro pruebas siguen leyendo el fuente (`sintesis/local.rs:398, :425`; `modelo_local.rs:295, :329`): ¿las cambio ya o quedan en deuda?
4. M-5 · WebView2 mantiene 2–3 conexiones a Microsoft en reposo (anterior al sprint): ¿Honestidad lo declara y aplicamos los flags?
5. S13: «sí deshacer» (credencial, tarea y nota de prueba) y «sí borrar ramas» (sprint-011, 012, 013).

**Auditoría** (una pasada, `sprints/SPRINT_014-auditoria.md`): 0 críticos, 0 altos. Medios: 5 pagados, 1 parcial (M-5), 2 pendientes (M-6, M-8). Bajos: 11 pagados, 1 parcial (B-5).

**Desviaciones:** una sola puerta de red (la de whisper), no dos · el corte suelta el modelo dentro de la pieza `Sugerencia` · un solo comando `instalar_el_modelo_local` · `demo-rojo.sh` no corre sin Python: pruebas Vitest permanentes para M-3 · hook de secretos arreglado fuera de alcance (`d515266`, B-8) · corrida en vivo: 1 de 3 dentro del techo.

**Relevos:** 1. F1, descarga cortada dos veces (a 874 s y 1 154 s, sin reanudar). Decidió el usuario: reanudar por `Range` en la puerta, camino distinto (`fd9c3c7`). La tercera descarga retomó y terminó. **Modelo por fase:** F1 y F2 Opus 5.5 (high) · F3 y auditoría Sonnet 5.5 · cierre Haiku 5.5. Haiku bastó para redactar el cierre; si se queda lo decides tú tras tres sprints.

**/release-check:** 1 arranque sin permisos ✅ · 2 capturas leídas ✅ · 3 binario ✅ · 4 TCC n/a (Windows) · 5 ventana protegida ✅ · 6 red y canaria ✅ · 7 `cargo audit` lo corre la CI · 8 manual y cierre ✅ · 9 checks al final · 10 protecciones: solo la descarga (M1) y la corrida en vivo, ambas con «sí»; credencial y tarea sin cambios.

**Aprendizaje y recomendación:** WebView2 abre conexiones propias a servidores de Microsoft; hay que medir el árbol entero del proceso, no solo el de Rust. El cierre del hito H-2 trae brochure, lote de medios y bajos, `design-sync` y el ensayo general.

**Para mergear:** squash y borrar la rama; después `/cierre-sprint copiloto-consultor`.
