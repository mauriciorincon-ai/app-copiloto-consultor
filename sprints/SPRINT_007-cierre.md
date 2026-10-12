---
sprint: 007 · app: copiloto-consultor · hito: H-1 Vitrina funcional · opened: 2026-10-11 · closed: 2026-10-12 · branch: sprint-007/windows-videollamada · pr: #23
---

# Sprint 007 · Windows en la videollamada

| Campo             | Valor                                                                                                            |
| ----------------- | ---------------------------------------------------------------------------------------------------------------- |
| Objetivo planeado | La videollamada en Windows: audio del cliente por el sistema, reunión detectada, ficha al oído, pantalla, atajos |
| Resultado         | Construido y pagada la auditoría. Pendiente en el PC: las pruebas de la matriz de la regla 24 (gate del hito)    |
| Estado            | Listo para mergear (squash, lo hace el usuario)                                                                  |
| Tiempo estimado   | 4 h (F1 0,5 · F2 3 h · F3 1,5 h)                                                                                 |
| Tiempo real       | 3,0 h activas (`scripts/horas-sprint.mjs`, 8 commits, 1 sesión)                                                  |
| Diferencia        | −1 h. F2 llegó a 1,7 h por heurística de git; el tiempo real de F2 fue mayor que lo que marca el script          |

```
### Horas del sprint (scripts/horas-sprint.mjs, rango origin/main..HEAD)
| Commits | Sesiones | Horas activas | Span |
| 8 | 1 | 3,0 h | 2026-10-11 22:15 → 2026-10-12 00:46 (2,5 h) |
| F1 | 1 | 0,5 h |
| F2 | 4 | 1,7 h |
| F3 | 3 | 0,8 h |
```

## Entregables

- **F1 (ab85798):** kit v2.2.0 en `CLAUDE.md`, § 29 en el anexo, `plan-sprint.md`. `dependabot.yml` ya era idéntico.
- **F2 (29ffeb3 → fd67822, 0a5a550):** audio del sistema por loopback WASAPI con reloj de silencio; salida real por forma de dispositivo; reunión por ejecutable con `data/clientes-windows.json` (7 filas, sin verificar); voz del sistema; pantalla por `PrintWindow` + OCR; cola `ClientePrimero` con tope 4; atajos `Alt+Mayús`; medición de whisper en el runner.
- **F3 (e36ca31, 00fd78f, 2458cb6):** textos y teclas de Windows en un solo punto (`windows-cambios.ts`, `EsteSistemaContext`); Permisos «No pide permiso»; M5 del S6; manual, guía v8 (8 ⭐ del PC), README, ADR 022 enmienda 1; auditoría A1-A4 pagada.

## No completado y desviaciones

- **Desviación:** la pantalla lee la ventana de la reunión, no el monitor (`PrintWindow`). El motor de OCR sale del perfil del usuario, no fijo en es-ES/en-US.
- **Desviación:** el test de la cadena del kit se movió de F3 a F2, para la demo en rojo.
- **No hecho:** voz y OCR contra fixtures en `build-windows` (M6 de la auditoría, lote del hito).
- **Diferido al S8:** acople, sugerencia, radar, borrado con la app cerrada, desbloqueo.

## Auditoría (una pasada, `sprints/SPRINT_007-auditoria.md`)

| Casilla                  | Resultado                                                                                                    |
| ------------------------ | ------------------------------------------------------------------------------------------------------------ |
| 1 · Cobertura de alcance | Completo con las dos desviaciones de arriba                                                                  |
| 2 · Calidad de código    | 4 altos pagados: A1 textos de error, A2 gate que no veía el texto pegado, A3 voz, A4 reapertura del loopback |
| 3 · Dependencias         | Limpia (`windows` 0.62 es la de `cpal`)                                                                      |
| 4 · Frases caducadas     | Barrido por promesa aplazada; segunda pasada del constructor sin coincidencias                               |

Medios y bajos: 8 medios y 14 bajos al lote del hito, cada uno con `archivo:línea`. Críticos: ninguno.

## Causas, decisiones y aprendizajes

- **A3 necesitó dos intentos:** el primer arreglo (un hilo corto de COM) tumbó el proceso de pruebas de Windows; el segundo (hilo permanente) pasó. Son mecanismos distintos, así que no es un bucle (regla 29).
- Leer las capturas como imagen encontró dos fugas que el test de texto no veía. Aprendizaje: el gate lee nodo por nodo.
- La CI de Windows es el juez de todo lo `cfg(windows)`; cada vuelta cuesta unos 10 min.

## Medición de whisper en el runner (manual)

Cliente (5,9 s) 7,65 s · tuyo detrás (10 s) 8,15 s; techo 4 s. **El runner de la CI supera el techo.** La cifra del PC manda, en el gate del hito.

## Relevos

Ninguno. Modelos: F1 Sonnet 5.5 · F2 Opus 5.5 · F3 Sonnet 5.5 · cierre Haiku 5.5 (bastó: sí, redacción y commits sin desvío).

## Para el gate humano del hito (lo que solo el PC ve)

| Acción                                     | Matriz regla 24                 | Aviso que verás                                         | Cómo se deshace                                               |
| ------------------------------------------ | ------------------------------- | ------------------------------------------------------- | ------------------------------------------------------------- |
| Instalar el `.msi`/`.exe` sin firma        | Probar el binario               | SmartScreen: Más información → Ejecutar de todas formas | Ajustes → Aplicaciones → desinstalar                          |
| Micrófono de la app                        | Tu pista                        | Privacidad y seguridad → Micrófono                      | Apagar el interruptor                                         |
| Modelo de voz (1,6 GB)                     | Transcribir                     | Ninguno; huella SHA-256 antes de usarlo                 | Borrar `%LOCALAPPDATA%\com.aiapps.copiloto-consultor\modelos` |
| Videollamada real con auriculares (w6–w13) | Loopback, voz, pantalla, atajos | Ninguno del sistema                                     | Nada que deshacer                                             |
| Idioma de voz/OCR, si falta                | Voz al oído y OCR               | Hora e idioma → añadir paquete                          | Quitar el paquete ahí                                         |
| Credential Manager (notas)                 | Llave de tus notas              | Ninguno                                                 | Administrador de credenciales → quitar «Angel Ghost · …»      |

**Para mergear:** squash y borrar la rama; después `/cierre-sprint copiloto-consultor`.
