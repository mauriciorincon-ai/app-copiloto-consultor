# Informe del sprint 006 · relevo de modelo y pruebas al gate del hito

Written for: la planeadora (para su núcleo y su cierre de sprint) y los constructores de cada app del pipeline.

## La orden

1. **Si un modelo siente que no puede resolver una tarea, para y pide cambio o relevo a un modelo más capaz.** No se queda en un bucle de reintentos.
2. **Tope: dos intentos fallidos en la misma clase de problema = parada.** Un tercer intento no se hace. Gastar tokens en repetir lo que ya falló no es eficiencia.
3. **La parada es una línea:** `actual · recomendado · por qué · comando` (`/model X`, `/effort Y`). Después se espera la respuesta del usuario. Nada de seguir probando «por si».
4. **Lo que requiere la máquina del usuario no bloquea el sprint.** Las matrices de PC y las corridas en vivo van al gate humano del hito, con su línea en el cierre.

Esta orden aplica a todas las apps y a todos los modelos, sin excepción por plataforma o por fase.

## Lo que pasó en el sprint 006 (Angel Ghost)

- **Fase 3 con Haiku 5.5** (cierre, textos, e2e, auditoría). Falló dos veces en la misma clase de problema:
  - texto de macOS en pantallas de Windows (auditoría A4 y A5);
  - gates que pasaban con la mutación (el de atajos no exigía el `return`; lo encontró la demo en rojo y se corrigió en `09ec07b`).
- **Lo que debió hacer:** parar tras el segundo fallo y pedir relevo a Sonnet 5.5 con la línea de la regla. **No lo hizo.** Siguió, y la auditoría de una pasada fue la que lo cazó.
- **Un tercer caso, el más claro:** la compilación de Windows fallaba por un tipo privado usado desde otro módulo (E0446). Haiku lo corrigió en la primera pasada; la corrida de CI confirmó el arreglo. Ese caso cuenta como un intento, no dos.
- **Por qué pasó:** la constitución ya decía «te detienes y recomiendas en UNA línea» (paso 3 del workflow, sección de modelo y esfuerzo), pero no fijaba un tope, no decía que el bucle está prohibido y no obligaba a registrar el relevo. Una regla sin tope se incumple sin que nadie lo vea.

## Qué cambia desde ya

- Tope de dos intentos por clase de problema. Al segundo fallo: parada y línea de relevo.
- El cierre registra **relevos pedidos** y **relevos ignorados**, con su motivo. Un relevo ignorado es deuda del proceso, no un detalle.
- Las pruebas que necesitan la máquina del usuario van al gate del hito. No frenan el sprint.

## Pruebas de PC (regla 24): al gate del hito

Las cuatro filas de la matriz para el PC con Windows no se ejecutan en este sprint:

| #   | Qué                                                         | Estado        |
| --- | ----------------------------------------------------------- | ------------- |
| 1   | Instalar el `.msi`/`.exe` sin firma                         | Gate del hito |
| 2   | Conceder el micrófono                                       | Gate del hito |
| 3   | Descarga del modelo de voz (~1,6 GB) y medición de latencia | Gate del hito |
| 4   | Notas cifradas en Credential Manager                        | Gate del hito |

El sprint se cierra con la CI verde (`quality`, `e2e`, `build-escritorio`, `build-windows`, `un-pr-vivo`), y las cuatro pruebas quedan declaradas en el cierre.

## Para la planeadora (cambio al núcleo, a su criterio)

Propuesta de redacción para el paso 3 del workflow y para la tabla de modelo:

> **Relevo sin bucles.** Si el modelo no puede resolver una tarea, para tras **dos intentos fallidos en la misma clase de problema** y pide relevo en una línea: `actual · recomendado · por qué · comando`. Un tercer intento no se hace. El cierre registra los relevos pedidos y los ignorados.

Y en la sección de pruebas: «lo que requiere la máquina del usuario va al gate del hito y no bloquea el cierre».

No he tocado `CLAUDE.md`: es el núcleo que regenera la planeadora. Lo que sí queda registrado en esta app es la memoria de feedback y el cierre del sprint.
