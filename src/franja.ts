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
  /**
   * A qué altura vive la banda arriba (sprint 011): los puntos que la bajaste desde el borde superior del área útil,
   * ya acotados a la pantalla. 0, pegada arriba; abajo no se mira.
   */
  desplazamiento: number;
  /** Que la banda está en pantalla. Tras su «×» o el corte, no: Sesión ofrece «Mostrar la banda». */
  bandaAbierta: boolean;
};

/** El evento con el que Rust avisa de que la banda cambió de borde, de altura o se cerró. */
export const EVENTO_FRANJA = "franja";

/** `?borde=arriba|abajo` en la URL, para el arnés de capturas. */
export function bordeDesdeLaUrl(busqueda: string): Borde | null {
  const pedido = new URLSearchParams(busqueda).get("borde");
  return pedido === "arriba" || pedido === "abajo" ? pedido : null;
}

/**
 * `?desplazamiento=132` y `?banda=cerrada` en la URL, para el arnés y el e2e de Windows (sprint 011): fuera de Tauri no hay
 * parte nativa que diga a qué altura está la banda ni si sigue abierta.
 */
export function desplazamientoDesdeLaUrl(busqueda: string): number {
  const px = Number(new URLSearchParams(busqueda).get("desplazamiento"));
  return Number.isFinite(px) && px > 0 ? Math.round(px) : 0;
}

export function bandaCerradaDesdeLaUrl(busqueda: string): boolean {
  return new URLSearchParams(busqueda).get("banda") === "cerrada";
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

/**
 * «La banda: arriba · abajo». Rust suelta, recoloca y vuelve a acoplar; **el evento `franja` trae el borde
 * nuevo**, así que el comando no devuelve nada (auditoría del S4, B37: su retorno no lo leía nadie).
 */
export function fijarPosicionDeLaBanda(borde: Borde): Promise<boolean> {
  return llamar("fijar_posicion_de_la_banda", { borde });
}

/** «Entendido» en el aviso de la primera vez: no vuelve a salir. */
export function entendidoElAvisoDeArriba(): Promise<boolean> {
  return llamar("entendido_el_aviso_de_arriba");
}

/**
 * **La banda a tu altura** (sprint 011, ADR 004 enmienda 3). Las cuatro puertas de Rust, una por gesto:
 *
 * - `desplazarBanda`: el asa de posición **mientras arrastras** —mueve la banda, no guarda nada—;
 * - `asentarDesplazamiento`: **al soltarla** —guarda la altura acotada a la pantalla y reacopla la reunión—, y
 *   «Volver arriba» en Sesión, con 0;
 * - `cerrarLaBanda`: el «×» —con una sesión en marcha la termina, devuelve la reunión y cierra la banda; la app sigue—;
 * - `mostrarLaBanda`: «Mostrar la banda» en Sesión, tras el «×» o el corte.
 *
 * Ninguna devuelve nada que haga falta leer: la altura y si la banda está en pantalla llegan por el evento `franja`.
 */
export function desplazarBanda(desplazamiento: number): Promise<boolean> {
  return llamar("desplazar_banda", { desplazamiento });
}

/**
 * Cuánto baja o sube la banda cada paso de «Bajar» y «Subir» en Sesión: la altura también se elige sin arrastrar
 * (WCAG 2.5.7, auditoría del S11, B4). Lo que sobra o falta lo acota Rust.
 */
export const PASO_DE_LA_ALTURA = 40;

export function asentarDesplazamiento(
  desplazamiento: number,
): Promise<boolean> {
  return llamar("asentar_desplazamiento", { desplazamiento });
}

export function cerrarLaBanda(): Promise<boolean> {
  return llamar("cerrar_la_banda");
}

export function mostrarLaBanda(): Promise<boolean> {
  return llamar("mostrar_la_banda");
}
