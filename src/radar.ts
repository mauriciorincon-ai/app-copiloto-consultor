import { useEffect, useState } from "react";
import { escuchar, hayTauri, llamar, preguntar } from "./puente";
import { useT } from "./i18n";

/**
 * EL RADAR (C14) — lo que el webview recibe de `src-tauri/src/radar/`.
 *
 * Dos mitades que llegan por dos caminos:
 *
 * - **El coral** («te vigilan»): el evento `radar` con [`EnTuMac`], que la parte nativa emite cada
 *   vez que cambia lo que corre en este Mac, y el comando `radar_de_tu_mac` para quien se monta
 *   tarde. Lo leen Sesión («software invasivo en tu Mac») y la banda.
 * - **El ámbar** («te graban»): una `Novedad` más del evento `escucha` (ver `ficha.ts`), porque
 *   nace de la lectura de pantalla y la banda la pinta en el mismo sitio que las fichas.
 *
 * Las dos formas cruzan la costura por `src/contrato.generado.ts`, que escribe Rust con su serde.
 */

/** Las clases del catálogo. Cerradas: la pantalla las nombra en dos idiomas. */
export type Categoria = "supervision" | "anti-trampa" | "monitoreo" | "acceso-remoto" | "mdm";

/** **Invasivo** mira tu equipo; **sábelo** es información (un MDM de empresa). */
export type Nivel = "invasivo" | "sabelo";

export type Bilingue = { es: string; en: string };

/** Un programa del catálogo que está corriendo en este Mac. */
export type Programa = {
  nombre: string;
  categoria: Categoria;
  nivel: Nivel;
  /** La frase corta de la banda: «ve tu pantalla completa y tu cámara». */
  ve: Bilingue;
  /** La columna «qué alcanza a ver» de Sesión. */
  alcance: Bilingue;
};

export type CatalogoDelRadar = { version: number; fecha: string };

export type EnTuMac = {
  /** Invasivos primero. */
  programas: Programa[];
  catalogo: CatalogoDelRadar;
};

const NADA: EnTuMac = { programas: [], catalogo: { version: 1, fecha: "" } };

/** Lo que el radar vio en este Mac. Fuera de Tauri, las cinco filas de la maqueta. */
export function useRadarDeTuMac(paraLaMuestra = false): EnTuMac {
  const muestra = useMuestraDeSesion();
  const [visto, setVisto] = useState<EnTuMac>(NADA);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    void preguntar<EnTuMac>("radar_de_tu_mac").then((e) => {
      if (vivo && e) setVisto(e);
    });
    const baja = escuchar<EnTuMac>("radar", (e) => {
      if (vivo && e) setVisto(e);
    });
    return () => {
      vivo = false;
      baja();
    };
  }, []);
  return !hayTauri() && paraLaMuestra ? muestra : visto;
}

export function invasivos(e: EnTuMac): Programa[] {
  return e.programas.filter((p) => p.nivel === "invasivo");
}

/** «Ver qué alcanza a ver» y `⌃⌥R`: el cuaderno se abre en Sesión, con la tabla. */
export function abrirLoQueVe() {
  void llamar("abrir_lo_que_ve");
}

/** Las cinco filas de `sesion.html` («software invasivo en tu Mac»), con sus nombres de ejemplo. */
function useMuestraDeSesion(): EnTuMac {
  const m = useT().cuaderno.muestraRadar;
  const orden: Categoria[] = ["supervision", "anti-trampa", "monitoreo", "acceso-remoto", "mdm"];
  return {
    programas: orden.map((c) => ({
      nombre: m[c].nombre,
      categoria: c,
      nivel: c === "mdm" ? "sabelo" : "invasivo",
      ve: { es: "", en: "" },
      alcance: { es: m[c].alcance, en: m[c].alcance },
    })),
    catalogo: { version: 1, fecha: "2026-09-15" },
  };
}
