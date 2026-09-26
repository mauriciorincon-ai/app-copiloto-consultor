import { useEffect, useState } from "react";
import { escuchar, hayTauri, preguntar } from "./puente";
import { useT } from "./i18n";
import type { Turno } from "./cuaderno";

/**
 * EL TRANSCRIPT DE LA BANDA — los últimos turnos, y la tecla que los enseña.
 *
 * Vive aparte de `cuaderno.ts` porque lo usa la **otra ventana**: la banda no necesita saber nada
 * de permisos, reuniones ni contadores, y mezclarlo todo en un módulo haría que abrir la banda
 * cargara el puente entero del cuaderno.
 *
 * **Cuántos turnos.** Tres, que son los que caben en la banda ampliada según `banda.html`. La
 * ventana de memoria guarda doce (`stt::ventana::TURNOS`); pedir los tres últimos es pedir lo que
 * se va a dibujar, no traerse la reunión entera a la interfaz para tirar nueve.
 */
const EN_LA_BANDA = 3;

/**
 * Los últimos turnos. **Fuera de Tauri son los de la maqueta**; dentro, los de verdad — y si no
 * hay ninguno, ninguno: enseñar la muestra dentro del producto sería inventar una conversación.
 */
export function useTurnos(): Turno[] {
  const m = useT().banda;
  const [turnos, setTurnos] = useState<Turno[]>(() => deMuestra(m));

  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const leer = () => {
      void preguntar<Turno[]>("turnos_recientes", { cuantos: EN_LA_BANDA }).then((t) => {
        if (vivo && t !== null) setTurnos(t);
      });
    };
    leer();
    const bajas = [escuchar("escucha", leer), escuchar("corte", leer)];
    return () => {
      vivo = false;
      bajas.forEach((b) => b());
    };
  }, []);

  return turnos;
}

/**
 * ¿Se enseña el transcript? Lo decide `⌃⌥T`, que registra la parte nativa.
 *
 * `inicial` existe para el arnés de capturas (`?transcript=1`), que tiene que poder fotografiar el
 * encuadre con el transcript abierto sin pulsar una tecla global.
 */
export function useTranscriptVisible(inicial = false): boolean {
  const [visible, setVisible] = useState(inicial);
  useEffect(() => {
    if (!hayTauri()) return;
    return escuchar("transcript", () => setVisible((v) => !v));
  }, []);
  return visible;
}

/** Los tres turnos que dibuja `banda.html`, con los textos del diccionario. */
function deMuestra(m: ReturnType<typeof useT>["banda"]): Turno[] {
  return [
    { pista: "sistema", desdeMs: 0, hastaMs: 2_000, texto: m.muestra.turno1, hora: m.muestra.hora1, eco: false },
    { pista: "microfono", desdeMs: 2_400, hastaMs: 5_000, texto: m.muestra.turno2, hora: m.muestra.hora2, eco: false },
    { pista: "sistema", desdeMs: 5_400, hastaMs: 8_000, texto: m.muestra.turno3, hora: m.muestra.hora2, eco: false },
  ];
}
