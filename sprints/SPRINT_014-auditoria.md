# Sprint 014 · La sugerencia sin salir — auditoría de una pasada

**Auditor:** subagente independiente, solo lectura (`git diff origin/main...HEAD`, Read, Grep), Sonnet 5.5 (high), 2026-10-19.
**Veredicto del auditor:** listo para cierre; 0 críticos, 0 altos; 8 medios y 12 bajos, todos con sitio y ajuste.
**Estado a la fecha del cierre:** 0 C/A abiertos. Medios: 5 pagados, 1 parcial (M5), 2 con deuda (M6, M8). Bajos: 11 pagados, 1 parcial (B5).

## Casillas

1. **Alcance:** completo, con reservas declaradas: la fila 20 sigue «hecho sin verificar» (1 de 3 corridas en vivo dentro del techo); la puerta de red es una (desviación, ADR 023 enm. 2); macOS sin cambio de comportamiento (lo cubre `sintesis/mod.rs` con prueba).
2. **Calidad:** completa tras pagar M1 a M4 y B1 a B4.
3. **Seguridad y privacidad:** sin secretos, sin datos reales, sin red nueva (una sola línea de `ureq`, la puerta de whisper), sin disco para el estado de llama. Protecciones tocadas: solo la descarga del modelo, con su fila y su «sí».
4. **Frases caducadas:** corregidas en manuales, guía, BLUEPRINT, README y ADR 022/023, salvo M6 (CLAUDE.md) y M8 (brochure).

## CRÍTICOS (0)

Ninguno.

## ALTOS (0)

Ninguno.

## MEDIOS (8)

| ID     | Sev | Sitio                                                                          | Qué                                                                                                    | Estado                                                                                                                                                               |
| ------ | --- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **M1** | M   | `src-tauri/src/modelo_local.rs:119-145`; `src-tauri/src/lib.rs` (`sintetizar`) | `soltar()` esperaba a una carga en curso; un hilo lanzado antes del corte podía recargar el modelo     | pagado · 634d8d4 (guarda con `soltar_al_terminar`; época comprobada al arrancar `sintetizar`)                                                                        |
| **M2** | M   | `decisions/anexos/023-medida-llama-cpp.md:81-104`                              | 1 de 3 corridas en vivo dentro del techo                                                               | pagado en el anexo (tabla de las 3 corridas; la generación se para en el techo, c3eb2ea). La causa de las corridas lentas sigue sin atribuir, y así lo dice el anexo |
| **M3** | M   | `scripts/verify-ephemeral.mjs:144-147`                                         | La compuerta de estado de llama no tenía prueba permanente                                             | pagado · `tests/unit/verify-ephemeral-win32.test.ts` (patrones en rojo) · 634d8d4                                                                                    |
| **M4** | M   | `tests/unit/logs-de-la-sintesis.test.ts:46`                                    | El gate de logs no miraba `[modelo-local]`                                                             | pagado · 634d8d4 (filtro y `CONTENIDO` ampliados)                                                                                                                    |
| **M5** | M   | `decisions/anexos/023-medida-llama-cpp.md:111-114`                             | Árbol de WebView2 de la app: 2–3 conexiones a Microsoft en reposo (dos arranques en frío)              | pagado en lo documentado (anexo, BLUEPRINT y guía w26 · 7684fed, a7e9713). deuda: declararlo en Honestidad y aplicar los flags de WebView2, decisión del usuario     |
| **M6** | M   | `CLAUDE.md:15`, `:17`, `:130`                                                  | Tres frases falsas: sugerencia en Windows, «cuatro» pruebas `*-en-windows.rs`, una sola puerta de Rust | deuda · tu «sí» para editar `CLAUDE.md`; no se edita sin él                                                                                                          |
| **M7** | M   | `docs/GUIA-DE-PRUEBA.html:1112` (u24)                                          | La fila describía dos grupos en Honestidad                                                             | pagado · a7e9713 (un solo grupo, quince funciones)                                                                                                                   |
| **M8** | M   | `docs/BROCHURE.html:381`, `:516`, `:599`; `docs/brochure-export.json:90`       | El brochure dice que en Windows la sugerencia queda «no por ahora»                                     | deuda · lote del hito (regla 13); el usuario decide si se paga ya                                                                                                    |

## BAJOS (12)

| ID      | Sev | Sitio                                                                                                           | Qué                                                                 | Estado                                                                                                                                                                                       |
| ------- | --- | --------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **B1**  | B   | `src-tauri/src/sintesis/local.rs`                                                                               | La limpieza con ceros no cubría el camino de error                  | pagado · 634d8d4 (guarda `PisarSiSeVa`; el lote y la salida se pisan salga como salga)                                                                                                       |
| **B2**  | B   | `src-tauri/src/modelo_whisper.rs` (rama de error)                                                               | Un error justo tras el último byte pedía `Range` más allá del final | pagado · 634d8d4 (`un_error_tras_el_ultimo_byte_no_pide_otro_tramo`)                                                                                                                         |
| **B3**  | B   | `src-tauri/src/modelo_local.rs`                                                                                 | IA decía «la descarga falló» mientras bajaba el modelo de voz       | pagado · 634d8d4 (`bajar_mira_si_la_puerta_esta_ocupada_antes_de_marcar_nada`)                                                                                                               |
| **B4**  | B   | `src/pantallas/Ia.tsx` (estado «bajando»)                                                                       | Estado sin símbolo (regla 8)                                        | pagado · F3 (`i-reloj`)                                                                                                                                                                      |
| **B5**  | B   | `src-tauri/src/sintesis/local.rs:398`, `:425`; `src-tauri/src/modelo_local.rs:295`, `:329`                      | Pruebas que leen el fuente en vez de comportarse                    | pagado en el tramo (función pura `inicio_del_tramo`, `src-tauri/src/modelo_whisper.rs:228`, con pruebas de comportamiento). deuda: las cuatro pruebas de fuente siguen, decisión del usuario |
| **B6**  | B   | `tests/unit/windows-sin-api-y-reunion-anterior.test.tsx:292`                                                    | Tamaño del modelo escrito a mano en `DE_MUESTRA`                    | pagado · F3 (prueba contra `ESTADO_DE_LA_IA_EN_WINDOWS.local.bytes`)                                                                                                                         |
| **B7**  | B   | `tests/e2e/windows.spec.ts`, `tests/e2e/en-us.spec.ts`                                                          | Ruido de formato ajeno al cambio                                    | pagado · F3                                                                                                                                                                                  |
| **B8**  | B   | `.claude/settings.json`                                                                                         | El hook de secretos dejaba pasar si no había `cat`                  | pagado · d515266 (falla cerrado; `tests/unit/hook-secretos.test.ts`, 7 de 7). Declarado como desviación en el cierre                                                                         |
| **B9**  | B   | `decisions/022-windows-detras-de-la-misma-frontera.md:99`, `:249`, `:254`                                       | Frases del ciclo anterior                                           | pagado · a7e9713                                                                                                                                                                             |
| **B10** | B   | `decisions/023-la-sugerencia-en-windows.md` (enm. 1)                                                            | «El modelo de Foundry se queda a propósito»                         | pagado · a7e9713 (la frase ya no está; el retiro queda en la enmienda 2)                                                                                                                     |
| **B11** | B   | `docs/BLUEPRINT.html:496`, `:644`, `:767`                                                                       | Tres detalles de as-built                                           | pagado · a7e9713 y c06121c                                                                                                                                                                   |
| **B12** | B   | `docs/MANUAL-WINDOWS.md:105`, `:146`; `docs/MANUAL-WINDOWS-EN.md`; `docs/MANUAL-DE-USO.md`; `docs/MANUAL-EN.md` | «Durante una compilación» (no atribuido) y cómo quitar el modelo    | pagado · a7e9713 (el manual dice cómo quitar el modelo y recuperar 1,1 GB)                                                                                                                   |

## Verificación de los pagos

Cada pago se comprobó en el repo con una búsqueda (frase ausente o prueba presente) antes de marcarlo como pagado. Lo parcial y lo pendiente se declaran tal cual en el cierre.
