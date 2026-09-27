import { useCallback, useEffect, useState } from "react";
import { escuchar, hayTauri, llamar, preguntar } from "./puente";
import { useT } from "./i18n";
import type { Unidad } from "./ficha";

/**
 * **TUS NOTAS** (C9, sprint 003, ADR 015) — lo único de la reunión que sobrevive, visto desde la
 * interfaz.
 *
 * Lo que es tuyo lo decide Rust (`src-tauri/src/notas/`, módulo protegido) y lo escribe cifrado
 * `src-tauri/src/carpeta.rs`. Aquí viven las formas que cruzan, atadas al serde de producción por
 * `src/contrato.generado.ts` (regla 19). La pantalla que las lee llega tras la mirada 19.
 */

/** Cuánto viven tus notas guardadas: una elección para todas las reuniones (ADR 015 §6). */
export type Retencion = "7d" | "30d" | "90d" | "1a" | "siempre";

/** Una ficha que fijaste con ⌃⌥P: cómo se llamaba y de dónde salía, no el texto del documento. */
export type FichaFijada = {
  titular: string;
  documento: string;
  seccion: string | null;
  unidad: Unidad | null;
};

/** Cuánto hay de cada cosa: lo que dice «Se va a guardar», y lo único que va al log. */
export type ResumenDelCuaderno = {
  parrafos: number;
  acuerdos: number;
  fijadas: number;
  turnos: number;
  /** Las propuestas que guardaste y las que esperan tu decisión (ADR 016). */
  propuestas: number;
  sinDecidir: number;
  bytesNota: number;
  bytesAcuerdos: number;
  bytesFijadas: number;
  bytesTurnos: number;
  bytesPropuestas: number;
};

/** La línea de «al cerrar»: «2026-09-20 · 47 min → reunion-2026-09-20-1402.ghost». */
export type Previsto = {
  fecha: string;
  minutos: number;
  cliente: string | null;
  archivo: string;
};

/** El cuaderno de la reunión de ahora (`cuaderno_de_la_reunion`). */
export type VistaDelCuaderno = {
  nota: string;
  acuerdos: string[];
  fijadas: FichaFijada[];
  resumen: ResumenDelCuaderno;
  conservarMisTurnos: boolean;
  /** Hay reunión abierta: sesión en marcha, o parada y sin guardar ni descartar. */
  abierta: boolean;
  /** Sigue escuchando: Notas enseña «durante»; parada, «al cerrar». */
  escuchando: boolean;
  previsto: Previsto | null;
  /** «Muere al cerrar»: los turnos del cliente y las lecturas de pantalla, solo contados. */
  turnosDelCliente: number;
  lecturas: number;
  retencion: Retencion;
};

/** Una reunión guardada, como la lista la enseña: sin abrirla (`reuniones_guardadas`). */
export type ReunionGuardada = {
  archivo: string;
  bytes: number;
  /** Segundos Unix. */
  guardada: number;
  /** Segundos Unix; 0 es «siempre». */
  vence: number;
};

/** Lo que devuelve «Guardar cifrado y cerrar»; `null` si no había nada tuyo. */
export type Guardada = { archivo: string; bytes: number; vence: number };

/** Las reuniones guardadas y dónde viven (`reuniones_guardadas`). */
export type ListaDeReuniones = {
  /** `null`: la de fábrica, `~/Documentos/Angel Ghost/`, que la interfaz nombra en su idioma. */
  carpeta: string | null;
  reuniones: ReunionGuardada[];
};

// ---------------------------------------------------------------------------------------------
// Los hooks. Tras la mirada 19 (aprobada el 2026-09-27).
// ---------------------------------------------------------------------------------------------


/** Las cuatro vistas de Notas. Dentro del producto la decide el cuaderno; fuera, la URL. */
export type VistaDeNotas = "durante" | "al-cerrar" | "archivo" | "exportar";

/** El cuaderno de ahora, **fuera de Tauri**: lo que dibuja la maqueta. */
export function useMuestraDelCuaderno(vista: VistaDeNotas): VistaDelCuaderno {
  const m = useT().notas.muestra;
  const durante = vista === "durante";
  return {
    nota: `${m.nota1}\n${m.nota2}`,
    acuerdos: durante ? [m.acuerdo1] : [],
    fijadas: [
      { titular: m.fijada1, documento: "Propuesta Páramo Azul", seccion: "§3.2", unidad: "propuesta" },
      { titular: m.fijada2, documento: "Sur del Valle", seccion: null, unidad: "caso" },
    ],
    // «Al cerrar» de la maqueta del sprint 3: la casilla apagada, como nace, y sin cliente elegido.
    resumen: {
      parrafos: 3,
      acuerdos: 2,
      fijadas: 2,
      turnos: 0,
      propuestas: 0,
      sinDecidir: 0,
      bytesNota: 2_048,
      bytesAcuerdos: 1_024,
      bytesFijadas: 1_024,
      bytesTurnos: 0,
      bytesPropuestas: 0,
    },
    conservarMisTurnos: false,
    abierta: vista === "durante" || vista === "al-cerrar",
    escuchando: durante,
    previsto: { fecha: "2026-09-20", minutos: 47, cliente: null, archivo: "reunion-2026-09-20-1402.ghost" },
    turnosDelCliente: 63,
    lecturas: 9,
    retencion: "90d",
  };
}

/** Las tres reuniones de la maqueta (`notas.html`, «sprint 3 · el archivo»), a su fecha. */
export const REUNIONES_DE_MUESTRA: ListaDeReuniones = {
  carpeta: null,
  reuniones: [
    { archivo: "paramo-azul-2026-09-20.ghost", bytes: 22_528, guardada: Date.UTC(2026, 8, 20, 12) / 1000, vence: 0 },
    { archivo: "paramo-azul-2026-09-06.ghost", bytes: 17_408, guardada: Date.UTC(2026, 8, 6, 12) / 1000, vence: 0 },
    { archivo: "sur-del-valle-2026-07-02.ghost", bytes: 31_744, guardada: Date.UTC(2026, 6, 2, 12) / 1000, vence: 0 },
  ],
};
/** Los días que le quedan a cada reunión de muestra, como los pinta la maqueta: 90 d · 76 d · 10 d. */
export const DIAS_DE_MUESTRA = [90, 76, 10];

/**
 * El cuaderno de la reunión de ahora. Se pregunta al montar, al volver a la ventana, cuando Rust avisa
 * de que cambió desde fuera (⌃⌥P, el principio o el fin de una reunión, el corte) y tras cada acción
 * de esta pantalla (`volver`).
 */
export function useCuaderno(muestra: VistaDelCuaderno): [VistaDelCuaderno | null, () => void] {
  const [vista, setVista] = useState<VistaDelCuaderno | null>(() => (hayTauri() ? null : muestra));
  const [vuelta, setVuelta] = useState(0);
  const volver = useCallback(() => setVuelta((v) => v + 1), []);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const ahora = () => {
      void preguntar<VistaDelCuaderno | null>("cuaderno_de_la_reunion").then((v) => {
        if (vivo) setVista(v);
      });
    };
    ahora();
    globalThis.addEventListener("focus", ahora);
    const bajaCuaderno = escuchar("cuaderno", ahora);
    // Una pista que abre o se cierra cambia «durante» por «al cerrar».
    const bajaEscucha = escuchar<{ que?: string }>("escucha", (n) => {
      if (n?.que === "empieza") ahora();
    });
    return () => {
      vivo = false;
      globalThis.removeEventListener("focus", ahora);
      bajaCuaderno();
      bajaEscucha();
    };
  }, [vuelta]);
  return [vista, volver];
}

/** Las reuniones guardadas. Se pregunta al montar, al volver y tras guardar, borrar o exportar. */
export function useReuniones(): [ListaDeReuniones, () => void] {
  const [lista, setLista] = useState<ListaDeReuniones>(() =>
    hayTauri() ? { carpeta: null, reuniones: [] } : REUNIONES_DE_MUESTRA,
  );
  const [vuelta, setVuelta] = useState(0);
  const volver = useCallback(() => setVuelta((v) => v + 1), []);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const ahora = () => {
      void preguntar<ListaDeReuniones>("reuniones_guardadas").then((l) => {
        if (vivo && l) setLista(l);
      });
    };
    ahora();
    globalThis.addEventListener("focus", ahora);
    const baja = escuchar("cuaderno", ahora);
    return () => {
      vivo = false;
      globalThis.removeEventListener("focus", ahora);
      baja();
    };
  }, [vuelta]);
  return [lista, volver];
}

export function escribirNota(texto: string) {
  void llamar("escribir_nota", { texto });
}

export function anotarAcuerdo(texto: string): Promise<boolean | null> {
  return preguntar<boolean>("anotar_acuerdo", { texto });
}

export function conservarMisTurnos(si: boolean): Promise<boolean> {
  return llamar("conservar_mis_turnos", { si });
}

/** «Guardar cifrado y cerrar». Rechaza con el motivo si no se pudo: la nota se queda. */
export function guardarLaReunion(): Promise<Guardada | null> {
  return preguntar<Guardada | null>("guardar_la_reunion");
}

export function cerrarSinGuardar(): Promise<boolean> {
  return llamar("cerrar_sin_guardar");
}

/** Pide Touch ID y dónde; `false` si cancelaste el diálogo. Rechaza si no se desbloqueó. */
export function exportarReunion(archivo: string, idioma: string): Promise<boolean | null> {
  return preguntar<boolean>("exportar_reunion", { archivo, idioma });
}

export function borrarReunion(archivo: string): Promise<boolean> {
  return llamar("borrar_reunion", { archivo });
}

export function fijarRetencion(retencion: Retencion): Promise<boolean> {
  return llamar("fijar_retencion", { retencion });
}

export function elegirCarpetaDeNotas(): Promise<string | null> {
  return preguntar<string | null>("elegir_carpeta_de_notas");
}

/** «Anotar para después» en la banda: lo mismo que ⌃⌥N. */
export function irANotas() {
  void llamar("ir_a_notas");
}
