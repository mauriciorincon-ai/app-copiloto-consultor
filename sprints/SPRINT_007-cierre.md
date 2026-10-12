---
sprint: 007 · app: copiloto-consultor · hito: H-1 · opened: 2026-10-11 · closed: 2026-10-12 · branch: sprint-007/windows-videollamada · pr: #23
---

# Sprint 007 · Windows en la videollamada

| Campo                  | Valor                                                                                   |
| ---------------------- | --------------------------------------------------------------------------------------- |
| Objetivo               | La videollamada en Windows: audio del cliente, reunión, ficha al oído, pantalla, atajos |
| Resultado              | Construido; auditoría pagada en sus 4 altos. Falta la matriz de la regla 24 en el PC    |
| Estado                 | Listo para mergear                                                                      |
| Tiempo estimado / real | 4 h / 3,0 h                                                                             |
| Diferencia             | −1 h                                                                                    |

Horas (`scripts/horas-sprint.mjs`):

```
| Commits 9 | Sesiones 1 | Horas activas 3,0 h | 2026-10-11 22:15 → 2026-10-12 00:46 |
| F1 0,5 h | F2 1,7 h | F3 0,8 h |
```

**Entregables.** F1 (`ab85798`): kit v2.2.0. F2 (`29ffeb3`…`0a5a550`): loopback WASAPI con reloj de silencio, salida real, reunión por ejecutable (7 filas sin verificar), voz del sistema, pantalla por `PrintWindow` + OCR, cola con tope 4, atajos `Alt+Mayús`. F3 (`e36ca31`…`2458cb6`): textos y teclas de Windows en un punto, Permisos, M5, manual, guía v8 (8 ⭐ del PC), ADR 022 enmienda 1.

**Desviaciones y no hecho.** La pantalla lee la ventana de la reunión, no el monitor. El test de la cadena del kit pasó de F3 a F2. Voz y OCR contra fixtures: no hecho (lote del hito). Acople, sugerencia, radar, borrado y desbloqueo: S8.

**Auditoría** (`sprints/SPRINT_007-auditoria.md`): 0 críticos; altos A1–A4 pagados (`00fd78f`, `2458cb6`); 8 medios y 14 bajos al lote del hito con `archivo:línea`. Dependencias limpias; frases caducadas sin coincidencias en la segunda pasada.

**Medición whisper (manual).** Cliente 7,65 s · tuyo 8,15 s; el runner supera el techo de 4 s. Manda la cifra del PC.

**Relevos:** ninguno. Modelos: F1 Sonnet 5.5 · F2 Opus 5.5 · F3 Sonnet 5.5 · cierre Haiku 5.5 (bastó).

**Gate humano del hito (regla 24)**

| Acción                                             | Matriz                          | Aviso · cómo se deshace                                                                |
| -------------------------------------------------- | ------------------------------- | -------------------------------------------------------------------------------------- |
| Instalar `.msi`/`.exe` sin firma                   | Probar el binario               | SmartScreen: Más información → Ejecutar de todas formas · desinstalar en Aplicaciones  |
| Micrófono, modelo de voz 1,6 GB, idioma de voz/OCR | Tu pista · transcribir          | Privacidad → Micrófono · borrar `%LOCALAPPDATA%\com.aiapps.copiloto-consultor\modelos` |
| Videollamada real con auriculares (w6–w13)         | Loopback, voz, pantalla, atajos | Ninguno · nada que deshacer                                                            |

**Para mergear:** squash y borrar la rama; después `/cierre-sprint copiloto-consultor`.
