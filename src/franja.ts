import { useEffect, useState } from "react";
import { escuchar, hayTauri, llamar, preguntar } from "./puente";

/**
 * LA FRANJA, vista desde las tres ventanas — dónde vive la banda (sprint 004, ADR 004 enmienda 1).
 *
 * La banda va **arriba de fábrica**, bajo la barra de menús y junto a la cámara, o abajo como en el
 * H1. Lo decide Rust —es una preferencia tuya (ADR 002, enmienda 8) y es Rust quien mueve las
 * ventanas— y las tres ventanas lo leen de aquí:
 *
 * - la **banda**, para dibujar el asa y la sombra en el borde que da a la reunión (`data-borde`) y
 *   para que arrastrar el asa crezca hacia donde hay sitio;
 * - el **relleno**, para subir el fondo de escritorio lo que mide la barra de menús y que por la
 *   franja asome justo el trozo que habría debajo;
 * - **Sesión**, para el interruptor «arriba · abajo» y el aviso de la primera vez.
 *
 * Se pregunta una vez al montar y se escucha a partir de ahí: cambia cuando pulsas `⌃⌥B` o el
 * interruptor, que puede ser desde otra ventana.
 */
export type Borde = "arriba" | "abajo";

export type LaFranja = {
  borde: Borde;
  /** Lo que mide la barra de menús (con el notch), en puntos. */
  barra: number;
  /** Que ya viste el aviso de la primera vez con la banda arriba. */
  avisoVisto: boolean;
};

/** El evento con el que Rust avisa de que la banda cambió de borde. */
export const EVENTO_FRANJA = "franja";

/** `?borde=arriba|abajo` en la URL, para el arnés de capturas. */
export function bordeDesdeLaUrl(busqueda: string): Borde | null {
  const pedido = new URLSearchParams(busqueda).get("borde");
  return pedido === "arriba" || pedido === "abajo" ? pedido : null;
}

/**
 * @param fuera lo que se enseña fuera de Tauri —los tests, el arnés de fidelidad—, donde no hay parte
 * nativa a la que preguntar. Cada ventana pasa el de su maqueta.
 */
export function useFranja(fuera: LaFranja): LaFranja {
  const [franja, setFranja] = useState<LaFranja>(fuera);

  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    void preguntar<LaFranja>("la_franja").then((f) => {
      if (vivo && f) setFranja(f);
    });
    const baja = escuchar<LaFranja>(EVENTO_FRANJA, setFranja);
    return () => {
      vivo = false;
      baja();
    };
  }, []);

  return hayTauri() ? franja : fuera;
}

/** «La banda: arriba · abajo». Rust suelta, recoloca y vuelve a acoplar; el evento trae el borde nuevo. */
export function fijarPosicionDeLaBanda(borde: Borde): Promise<LaFranja | null> {
  return preguntar<LaFranja>("fijar_posicion_de_la_banda", { borde });
}

/** «Entendido» en el aviso de la primera vez: no vuelve a salir. */
export function entendidoElAvisoDeArriba(): Promise<boolean> {
  return llamar("entendido_el_aviso_de_arriba");
}
