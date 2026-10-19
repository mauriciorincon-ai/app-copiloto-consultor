---
sprint: 014
app: copiloto-consultor
hito: H-2
opened: 2026-10-19
closed:
branch: sprint-014/la-sugerencia-sin-salir
pr:
---

# Sprint 014 · La sugerencia sin salir — cierre (BORRADOR: lo redacta Haiku al final; esto es el estado de trabajo)

## Estado al terminar F2 (2026-10-19)

- **F1 ✅** (`f1ae51b`, `fd9c3c7`): llama-cpp-2 =0.1.136 enlaza con whisper, un solo ggml 0.9.5; cuelgue antes de `main`
  por `msvcrtd` resuelto (`[profile.dev.package.llama-cpp-sys-2] debug-assertions = false`); la puerta retoma con
  `Range`; GGUF bajado (1 117 320 736 B, huella comprobada).
- **F2 ✅** (commit `[F2]`): `modelo_local.rs`, `sintesis/local.rs` (`Quien::Local`), `quien_redacta` con `local_listo`
  (Windows nunca `Api`), `promete_redactar(mock, local)` + `promete_enriquecer`, `prefs::para_esta_plataforma` con el
  modelo, comando `instalar_el_modelo_local`, el corte suelta el modelo en la pieza `Sugerencia`, «Terminar» y apagar
  «Redactar» lo sueltan, precarga al empezar, IA en Windows (fila «Modelo local», «Instalar el modelo (1,1 GB)»),
  «Hoy en Windows» 15 «sí», ADR 022 fila, ADR 023 enmienda 2 + `anexos/023-medida-llama-cpp.md`, `verify-ephemeral`
  prohíbe guardar el estado de llama. Verde: `cargo test` (737 lib + integración), clippy `-D warnings`, Vitest 846,
  e2e 476, `verify:ephemeral`, `cero-enlaces`, contador-de-red. 4 demos en rojo (anexo).

## La medida (va al cierre final)

| Proveedor de la app (Qwen2.5-1.5B Q4_K_M, llama.cpp en el proceso, 5 hilos) | Resultado                                       | Criterio |
| --------------------------------------------------------------------------- | ----------------------------------------------- | -------- |
| Tamaño · huella                                                             | 1 117 320 736 B · `6a1a2eb6…407e`               | —        |
| Compila (ARM64, `-j 4`)                                                     | llama.cpp ≈ 2 min                               | —        |
| Carga · RAM                                                                 | 1 499 ms · 7 → 1 112 MB (pico 1 227; suelto 18) | —        |
| Latencia 17 turnos                                                          | mediana 2 105 · p95 2 321 · máx 2 356 ms        | < 6 s ✅ |
| `fundar`                                                                    | 12 de 17                                        | ≥ 9 ✅   |
| TCP · UDP del proceso, 5 fases                                              | 0 · 0                                           | 0 ✅     |
| Canaria                                                                     | 0 archivos (42 mirados)                         | 0 ✅     |

## Falta (F3, `/model sonnet`; cierre con `/model claude-haiku-5-5`)

1. Docs: manual ES/EN (§ IA en Windows, frases «no por ahora» de MANUAL-WINDOWS(-EN).md línea 9 y 105, MANUAL-DE-USO 837,
   MANUAL-EN 833 y 923), guía `ag-w14-` (w25 instalar, w26 la sugerencia; GUIA-DE-PRUEBA-WINDOWS.html 321 y
   GUIA-DE-PRUEBA.html 933, solo la fila del modelo del PC), BLUEPRINT (356, 363, 547, 714, 834), README. Brochure al hito.
2. Corrida en vivo por CDP con su matriz (fila A: Start session = micrófono y audio del sistema): «Draft suggestions» →
   sugerencia en la banda < 6 s citando ficha · 0 B · 0 conexiones del proceso. Luego la fila 20 del manual de Windows a
   «hecho».
3. **Hallazgo para pagar** (no de este sprint): el hook de secretos de `.claude/settings.json` **falla abierto** si el bash
   que lo corre no tiene `cat` en el PATH (`inp=$(cat)` vacío → deja pasar la carnada; visto lanzando Vitest desde
   PowerShell). Arreglo: leer stdin sin `cat` (builtin) o exigir `cat` como a `gitleaks`/`jq`; mirar también
   `githooks/pre-commit`.
4. Limpieza de la Surface (matriz L1–L5, «sí» por fila; búsqueda de rastros en 0): L1 `modelos\foundry\` 1 842 MB ·
   L2 `target\debug\build\foundry-local-sdk-*` · L3 registro de cargo `foundry-local-sdk-2.1.0` · L4 compilaciones viejas
   de `llama-cpp-sys-2` (hash `2e856baa…`, `fb22b85c…`, con `msvcrtd`) y los `.crate` del scratchpad · L5 (decisión)
   `ggml-large-v3-turbo.bin` 1 549 MB. `.onnxruntime` ya no existe.
5. `/audita-sprint` (media hora propia) · `/release-check` · cierre ≤ 40 líneas con `horas-sprint.mjs`, la tabla y
   «Relevos» · `gh pr list --state open` · PR normal · checks al final.
6. Pendiente del S13 con «sí»: «sí deshacer» (llave, tarea, nota de prueba) y «sí borrar ramas».

## Relevos

- **F1 · descarga del modelo (regla 29, 2026-10-19 11:25).** Qué se intentó: bajar Qwen2.5-1.5B-Instruct Q4_K_M
  (1 117 320 736 B, SHA-256 `6a1a2eb6…`, revisión `91cad51…`) por la puerta de la app (`modelo_whisper::bajar`), dos
  veces, con el «sí» a M1. Qué falló: las dos se cortaron del lado del servidor («Peer disconnected»), a los 1 154 s y a
  los 874 s, a ≈ 1,2–1,5 MB/s; la primera, cerca del final. Lo que se sabe: la puerta no reanuda, borra el `.parcial` y
  empieza de cero. La carpeta quedó como estaba. Sin tercer intento por el mismo camino: decide el usuario. **Decisión
  del usuario: reanudar en la puerta (Opus, camino distinto)** → `fd9c3c7`; la tercera bajó con un corte retomado.

## Desviaciones (declaradas en el ADR 023 enmienda 2)

- Una sola puerta de red (la de whisper) en vez de una segunda · el corte suelta el modelo dentro de la pieza `Sugerencia`
  en vez de una pieza nueva · un solo comando (`instalar_el_modelo_local`; el estado viaja en `EstadoDeLaIa.local`) ·
  `scripts/demo-rojo.sh` no corre sin Python: equivalente en node con los mismos candados · fila 20 «hecho sin
  verificar» hasta la corrida en vivo.
