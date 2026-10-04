import { useEffect, useState } from "react";
import { escuchar, hayTauri, llamar, preguntar } from "./puente";
import type { Bilingue } from "./radar";
import type { EstadoDeEscucha } from "./cuaderno";
import {
  BANDERA_SIN_INDICAR,
  ESCUCHA_SOLO_NOTAS,
  NDA_LO_PROHIBE,
  NDA_NO_LO_PROHIBE,
  VISTA_DEL_CLIENTE,
} from "./contrato.generado";

/**
 * **EL MARCO EN LA MANO** (C11, ADR 017) — «Este cliente» en Sesión: a quién eliges, la bandera de
 * su jurisdicción, lo que respondiste de su NDA y la cláusula modelo.
 *
 * La bandera la decide Rust (`jurisdiccion/`, puro) con el catálogo que va dentro del binario; aquí
 * solo se pinta. Lo no verificado llega en `pendiente` y se enseña; nada de esto es asesoría legal, y
 * la pantalla lo dice siempre que hay bandera.
 */

/** Del menor al mayor. «Sin verificar» es el mayor: lo que no se sabe se trata con más cuidado. */
export type Riesgo =
  "bajo" | "bajo-medio" | "medio" | "medio-alto" | "sin-verificar";

export type Bandera = {
  nombre: Bilingue;
  riesgo: Riesgo;
  regla: Bilingue;
  implica: Bilingue;
  /** «CSJ AP1465-2018 · Ley 1581 art. 3»: las URL se quedan en el catálogo. */
  normas: string[];
  consultado: string;
  pendiente: Bilingue | null;
};

/** Las tres formas de `kit.html` §5: la conocida, la que el catálogo no trae y la que no se dijo. */
export type LaBandera =
  | { que: "conocida"; bandera: Bandera }
  | { que: "fuera-del-catalogo"; escrita: string; version: number }
  | { que: "sin-indicar" };

export type Nda = "sin-revisar" | "no-lo-prohibe" | "lo-prohibe";

export type VistaDelCliente = {
  clientes: string[];
  /** El de esta reunión. Vive en memoria: no se guarda en disco (ADR 017 §3). */
  elegido: string | null;
  bandera: LaBandera | null;
  nda: Nda;
  clausula: Bilingue;
};

/**
 * Lo que Sesión enseña fuera de Tauri, para que el arnés de fidelidad fotografíe cada estado de
 * `sesion.html` por la URL. Las muestras salen del contrato, que escribe Rust con el catálogo de
 * verdad: si una fila cambia, la muestra cambia con ella.
 */
export type EstadoDeSesion =
  | "normal"
  | "en-marcha"
  | "pregunta"
  | "sin-bandera"
  | "clausula"
  | "nda"
  | "solo-notas"
  /** La primera vez con la banda arriba (sprint 004, `sesion.html` · la primera vez). */
  | "aviso"
  /** Un ensayo terminado sin guardar (auditoría del S4, M10; `sesion.html` · ensayo sin guardar). */
  | "ensayo";

const ESTADOS: EstadoDeSesion[] = [
  "normal",
  "en-marcha",
  "pregunta",
  "sin-bandera",
  "clausula",
  "nda",
  "solo-notas",
  "aviso",
  "ensayo",
];

export function estadoDeSesion(busqueda: string): EstadoDeSesion {
  const pedido = new URLSearchParams(busqueda).get("estado");
  return ESTADOS.find((e) => e === pedido) ?? "normal";
}

export function muestraDelCliente(estado: EstadoDeSesion): VistaDelCliente {
  switch (estado) {
    case "sin-bandera":
      return {
        ...VISTA_DEL_CLIENTE,
        bandera: BANDERA_SIN_INDICAR,
        nda: NDA_NO_LO_PROHIBE,
      };
    case "nda":
      return { ...VISTA_DEL_CLIENTE, nda: NDA_LO_PROHIBE };
    default:
      return VISTA_DEL_CLIENTE;
  }
}

/**
 * La escucha que Sesión enseña fuera de Tauri. Antes de empezar, que es lo que dibuja `sesion.html`
 * «sprint 3 · este cliente»; «en marcha», escuchando; y en solo notas, la de Rust.
 */
export function escuchaDeMuestra(estado: EstadoDeSesion, base: EstadoDeEscucha): EstadoDeEscucha {
  if (estado === "solo-notas") return ESCUCHA_SOLO_NOTAS;
  return { ...base, escuchando: estado === "en-marcha" };
}

/**
 * «Este cliente». Dentro de Tauri se pregunta a Rust, y otra vez al volver a la ventana: el corpus pudo
 * reindexarse con otra ficha. Fuera, la muestra.
 */
export function useEsteCliente(
  muestra: VistaDelCliente,
): [VistaDelCliente, (v: VistaDelCliente) => void] {
  const [vista, setVista] = useState<VistaDelCliente>(muestra);
  useEffect(() => {
    if (!hayTauri()) {
      setVista(muestra);
      return;
    }
    let vivo = true;
    const leer = () => {
      void preguntar<VistaDelCliente>("este_cliente").then((v) => {
        if (vivo && v) setVista(v);
      });
    };
    leer();
    globalThis.addEventListener("focus", leer);
    // Los clientes salen del corpus: si el arranque vuelve a leer la carpeta recordada (auditoría del
    // S3, B29), la lista llega después, y la pantalla se entera.
    const baja = escuchar("corpus", leer);
    return () => {
      vivo = false;
      globalThis.removeEventListener("focus", leer);
      baja();
    };
  }, [muestra]);
  return [vista, setVista];
}

/** Elige el cliente de esta reunión, o ninguno (`null`). Devuelve la vista nueva. */
export function elegirCliente(
  nombre: string | null,
): Promise<VistaDelCliente | null> {
  return preguntar<VistaDelCliente>("elegir_cliente", { nombre });
}

/** La respuesta al chequeo de NDA: «Sí, lo prohíbe» (`true`) o «No lo prohíbe». */
export function responderNda(
  prohibe: boolean,
): Promise<VistaDelCliente | null> {
  return preguntar<VistaDelCliente>("responder_nda", { prohibe });
}

/** «Revisar» y «Volver a revisar la NDA»: la respuesta se borra y la pregunta vuelve. */
export function revisarNda(): Promise<VistaDelCliente | null> {
  return preguntar<VistaDelCliente>("revisar_nda");
}

/** «Solo notas» e «Iniciar en modo solo notas»: la reunión se abre y nada se captura. */
export function empezarSoloNotas(): Promise<boolean> {
  return llamar("empezar_solo_notas");
}
