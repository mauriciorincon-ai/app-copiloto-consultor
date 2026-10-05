import { useEffect, useState } from "react";
import { escuchar, hayTauri, preguntar } from "./puente";
import {
  VISTA_DE_LA_PUERTA_ABIERTA,
  VISTA_DE_LA_PUERTA_CERRADA,
  VISTA_DE_LA_PUERTA_EN_REUNION,
} from "./contrato.generado";

/**
 * **LA PUERTA LOCAL PARA TU AGENTE** (C16, ADR 018) — lo que IA enseña de ella y cómo se abre.
 *
 * La política vive en Rust (`puerta/`, módulo protegido): qué órdenes hay, qué se deniega, qué se
 * registra. Aquí solo se pinta y se conmuta. La puerta nace cerrada en cada arranque y se abre a
 * mano, con el conmutador de esta pantalla; en reunión o durante un ensayo se cierra sola.
 */

/** Por qué se denegó una orden. */
export type Motivo =
  | "llave-errada"
  | "en-reunion"
  | "el-api-es-tuyo"
  | "no-delegable"
  | "orden-desconocida";

/** Qué pasó con una orden. Un fallo no lleva su error: podría llevar contenido. */
export type Resultado =
  | { que: "hecho"; cuenta: number | null }
  | { que: "denegado"; motivo: Motivo }
  | { que: "fallo" };

/** Una línea de «Qué hizo tu agente»: la orden **sin su contenido**. */
export type Entrada = { hora: string; orden: string; resultado: Resultado };

/** Por qué está cerrada, si se cerró en esta sesión de la app. */
export type Cierre = "a-tu-mano" | "en-reunion";

/** Por qué no se abrió la última vez que lo intentaste. */
export type NoAbre = "en-reunion" | "ruta-larga" | "llavero" | "socket";

export type VistaDeLaPuerta = {
  abierta: boolean;
  cerro: Cierre | null;
  noAbre: NoAbre | null;
  /** La ruta de `ghost`, si está compilado junto a la app. En H1 no se instala en el PATH. */
  ghost: string | null;
  /** De la más reciente a la más antigua. */
  registro: Entrada[];
};

/**
 * La puerta que IA enseña fuera de Tauri, para que el arnés de fidelidad fotografíe cada estado de
 * `ia.html` por la URL (`?vista=puerta&puerta=abierta`). Las muestras salen del contrato, que escribe
 * Rust con su serde.
 */
export function muestraDeLaPuerta(busqueda: string): VistaDeLaPuerta {
  switch (new URLSearchParams(busqueda).get("puerta")) {
    case "abierta":
      return VISTA_DE_LA_PUERTA_ABIERTA;
    case "en-reunion":
      return VISTA_DE_LA_PUERTA_EN_REUNION;
    default:
      return VISTA_DE_LA_PUERTA_CERRADA;
  }
}

/**
 * La puerta. Dentro de Tauri se le pregunta a Rust, y Rust avisa cuando cambia —se abre, se cierra
 * sola en una reunión, atiende una orden— con el evento `puerta`, que solo recibe la ventana
 * principal. Fuera, la muestra.
 */
export function usePuerta(
  muestra: VistaDeLaPuerta,
): [VistaDeLaPuerta, (v: VistaDeLaPuerta) => void] {
  const [vista, setVista] = useState<VistaDeLaPuerta>(muestra);
  useEffect(() => {
    if (!hayTauri()) {
      setVista(muestra);
      return;
    }
    let vivo = true;
    void preguntar<VistaDeLaPuerta>("la_puerta").then((v) => {
      if (vivo && v) setVista(v);
    });
    const baja = escuchar<VistaDeLaPuerta>("puerta", (v) => {
      if (vivo) setVista(v);
    });
    return () => {
      vivo = false;
      baja();
    };
  }, [muestra]);
  return [vista, setVista];
}

export function abrirLaPuerta(): Promise<VistaDeLaPuerta | null> {
  return preguntar<VistaDeLaPuerta>("abrir_la_puerta");
}

export function cerrarLaPuerta(): Promise<VistaDeLaPuerta | null> {
  return preguntar<VistaDeLaPuerta>("cerrar_la_puerta");
}

/**
 * El comando que se le da a Claude Code: la ruta de `ghost` entre comillas simples —una carpeta con
 * espacios no lo parte— y `--help`, que es lo primero que un agente tiene que leer.
 */
export function comandoDeGhost(ruta: string): string {
  return `'${ruta.split("'").join("'\\''")}' --help`;
}
