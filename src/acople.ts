import { useEffect, useState } from "react";
import { escuchar, hayTauri, preguntar } from "./puente";

/**
 * EL ACOPLE, visto desde la banda.
 *
 * La banda dibuja «acoplada» o «sin acople» en su cabecera. Es un estado del SISTEMA —depende de
 * si el usuario concedió Accesibilidad y de si la ventana de la reunión se dejó recortar—, así
 * que no puede ser una etiqueta fija: sería exactamente la clase de promesa que esta app existe
 * para no hacer.
 *
 * Se pregunta una vez al montar y se escucha a partir de ahí. Preguntar sola dejaría a la banda
 * sondeando cada dos segundos por algo que cambia tres veces en una reunión; escuchar sin
 * preguntar se perdería el estado inicial, porque el acople del arranque ocurre antes de que este
 * webview exista.
 */
export type EstadoDelAcople = { permiso: boolean; acoplada: boolean };

/** El evento con el que Rust avisa de que el acople cambió. */
export const EVENTO_ACOPLE = "acople";

/**
 * @param forzado fuerza el valor y no pregunta a nadie. Es lo que usa el arnés del gate de
 * fidelidad para recorrer los dos encuadres (`?acoplada=0` y `?acoplada=1`) en un navegador,
 * donde no hay parte nativa a la que preguntar.
 */
export function useAcoplada(forzado?: boolean): boolean {
  // Fuera de Tauri —los tests, `pnpm dev`, el arnés de capturas— la respuesta es «sí»: es el
  // estado normal del producto y el que la maqueta dibuja por defecto.
  const [acoplada, setAcoplada] = useState(forzado ?? true);

  useEffect(() => {
    if (forzado !== undefined || !hayTauri()) return;
    let vivo = true;
    void preguntar<EstadoDelAcople>("estado_del_acople").then((e) => {
      if (vivo && e) setAcoplada(e.acoplada);
    });
    const baja = escuchar<EstadoDelAcople>(EVENTO_ACOPLE, (e) => setAcoplada(e.acoplada));
    return () => {
      vivo = false;
      baja();
    };
  }, [forzado]);

  return acoplada;
}
