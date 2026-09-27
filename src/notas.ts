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
  bytesNota: number;
  bytesAcuerdos: number;
  bytesFijadas: number;
  bytesTurnos: number;
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

export type TurnoPropio = { hora: string; texto: string };

/** Lo que hay dentro de una reunión guardada, ya descifrado (`abrir_reunion`, tras el desbloqueo). */
export type ContenidoDeReunion = {
  version: number;
  empezo: string;
  minutos: number;
  cliente: string | null;
  nota: string;
  acuerdos: string[];
  fijadas: FichaFijada[];
  misTurnos: TurnoPropio[];
};
