import { useEffect, useState } from "react";
import { hayTauri, preguntar } from "./puente";
import { PLATAFORMA_WINDOWS } from "./contrato.generado";

/**
 * QUÉ HACE LA APP EN ESTE EQUIPO (sprint 006, ADR 022). Lo emite `src-tauri/src/plataforma.rs` y la muestra del
 * contrato (`PLATAFORMA_WINDOWS` en `contrato.generado.ts`) se compila contra estos tipos.
 *
 * En Windows cruza la lista «Hoy en Windows» del ADR 022 —qué hace hoy, qué llega en el sprint 007 y qué no está
 * planeado— y lo que la transcripción sabe de este PC: el modelo de whisper que hay y lo que tardó en la calibración.
 * En macOS la lista viene vacía: ahí la app hace lo que dicen el manual y Honestidad.
 */
export type Sistema = "macos" | "windows" | "otro";

/** Cuándo está una función en Windows: hoy, en el sprint 007, o no por ahora. */
export type Cuando = "si" | "todavia-no" | "no-por-ahora";

/** Las claves del ADR 022, en su orden. */
export const CLAVES_DE_WINDOWS = [
  "banda-protegida",
  "presencial",
  "corpus-y-ficha",
  "notas-cifradas",
  "videollamada",
  "pantalla",
  "voz-al-oido",
  "acople",
  "ensayo",
  "sugerencia",
  "radar",
  "atajos",
  "borrado-con-la-app-cerrada",
  "desbloqueo",
  "puerta-local",
] as const;

export type ClaveDeWindows = (typeof CLAVES_DE_WINDOWS)[number];

export type FuncionEnWindows = {
  clave: ClaveDeWindows;
  cuando: Cuando;
};

export type Transcripcion = {
  /** «large-v3-turbo» o «small»; `null` si no hay ningún modelo entero. */
  modelo: string | null;
  /** Lo que tardó ese modelo en los 10 s de calibración; `null` si no se midió en esta sesión. */
  latenciaMs: number | null;
  /** El techo del fin de turno a la ficha: 4 s. */
  techoMs: number;
  /** Se está bajando un modelo ahora mismo. */
  bajando: boolean;
};

export type Plataforma = {
  sistema: Sistema;
  funciones: FuncionEnWindows[];
  transcripcion: Transcripcion | null;
};

/** Fuera de Tauri (tests, arnés de capturas) no hay a quién preguntar: se dice macOS, sin lista. */
export const PLATAFORMA_FUERA: Plataforma = {
  sistema: "macos",
  funciones: [],
  transcripcion: null,
};

/**
 * Lo que enseña una pantalla fuera de Tauri, donde no hay parte nativa a la que preguntar. `?plataforma=windows`
 * pinta la muestra de Windows (la del contrato); sin el parámetro, macOS. Es el mismo mecanismo que `?estado=` en
 * las demás pantallas: lo usan el e2e y el arnés de capturas, nunca la app instalada.
 */
export function plataformaFuera(busqueda: string = globalThis.location?.search ?? ""): Plataforma {
  return new URLSearchParams(busqueda).get("plataforma") === "windows" ? PLATAFORMA_WINDOWS : PLATAFORMA_FUERA;
}

/** ¿Es Windows? Honestidad enseña su bloque solo entonces. */
export function esWindows(p: Plataforma): boolean {
  return p.sistema === "windows";
}

/** Las funciones agrupadas por cuándo llegan, cada grupo en el orden del ADR. */
export function porCuando(p: Plataforma): Record<Cuando, ClaveDeWindows[]> {
  const grupos: Record<Cuando, ClaveDeWindows[]> = {
    si: [],
    "todavia-no": [],
    "no-por-ahora": [],
  };
  for (const f of p.funciones) grupos[f.cuando].push(f.clave);
  return grupos;
}

/** Lo que Honestidad dice de whisper en este PC: lo que hay, sin adornarlo. */
export type EstadoDeWhisper =
  | { que: "bajando" }
  | { que: "sin-modelo" }
  | { que: "sin-medir"; modelo: string }
  | { que: "a-tiempo"; modelo: string; ms: number }
  | { que: "lento"; modelo: string; ms: number; techoMs: number };

export function estadoDeWhisper(
  t: Transcripcion | null,
): EstadoDeWhisper | null {
  if (!t) return null;
  if (t.bajando) return { que: "bajando" };
  if (t.modelo === null) return { que: "sin-modelo" };
  if (t.latenciaMs === null) return { que: "sin-medir", modelo: t.modelo };
  if (t.latenciaMs > t.techoMs)
    return {
      que: "lento",
      modelo: t.modelo,
      ms: t.latenciaMs,
      techoMs: t.techoMs,
    };
  return { que: "a-tiempo", modelo: t.modelo, ms: t.latenciaMs };
}

/** Lo que Honestidad dice de whisper en este equipo; `null` fuera de Windows. */
export function whisperEnEsteEquipo(p: Plataforma): EstadoDeWhisper | null {
  return estadoDeWhisper(p.transcripcion);
}

/** Pregunta a Rust una vez al montar. */
export function usePlataforma(
  fuera: Plataforma = plataformaFuera(),
): [Plataforma, (p: Plataforma) => void] {
  const [p, setP] = useState<Plataforma>(fuera);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    void preguntar<Plataforma>("plataforma_de_este_equipo").then((r) => {
      if (vivo && r) setP(r);
    });
    return () => {
      vivo = false;
    };
  }, []);
  return [p, setP];
}

/** «Medir otra vez»: whisper transcribe los 10 s de calibración y devuelve lo que sabe ahora. */
export function calibrarTranscripcion(): Promise<Plataforma | null> {
  return preguntar<Plataforma>("calibrar_transcripcion");
}

/** «Bajar el modelo pequeño» (0,5 GB), cuando el grande pasa de 4 s en este PC. */
export function bajarModeloPequeno(): Promise<Plataforma | null> {
  return preguntar<Plataforma>("bajar_modelo_pequeno");
}
