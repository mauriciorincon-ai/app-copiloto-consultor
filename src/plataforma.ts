import { useEffect, useState } from "react";
import { hayTauri, preguntar } from "./puente";
import { PLATAFORMA_MACOS, PLATAFORMA_WINDOWS } from "./contrato.generado";

/**
 * QUÉ HACE LA APP EN ESTE EQUIPO (sprint 006, ADR 022). Lo emite `src-tauri/src/plataforma.rs` y la muestra del
 * contrato (`PLATAFORMA_WINDOWS` en `contrato.generado.ts`) se compila contra estos tipos.
 *
 * En Windows cruza la lista «Hoy en Windows» del ADR 022 —qué hace hoy y qué no está planeado; desde el sprint 008 no
 * queda nada a medio llegar— y lo que la transcripción sabe de este PC: el modelo de whisper que hay y lo que tardó en
 * la calibración. En macOS la lista viene vacía: ahí la app hace lo que dicen el manual y Honestidad.
 */
export type Sistema = "macos" | "windows" | "otro";

/** Si una función está en Windows: hoy, o no por ahora. */
export type Cuando = "si" | "no-por-ahora";

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
  /** Turnos tuyos que se quedaron sin transcribir en la última escucha (sprint 007: el cliente va primero). */
  descartados: number;
  /** Una línea por modelo, del mayor al menor (sprint 009): lo que tardó cada uno EN ESTE PC. */
  modelos: ModeloEnEsteEquipo[];
};

/** Un modelo de whisper tal como está en este PC (sprint 009). */
export type ModeloEnEsteEquipo = {
  /** «large-v3-turbo» o «small». */
  nombre: string;
  /** ¿Está entero en este PC? */
  bajado: boolean;
  /** Lo que tardó tu pista, el turno de 10 s de calibración; `null`: sin medir (o sin bajar). Decide el modelo. */
  latenciaMs: number | null;
  /** Lo que tardó la pista del cliente, medida detrás de la tuya (sprint 010); `null`: sin medir. La regla no la usa. */
  latenciaClienteMs: number | null;
  /** ¿Es con el que se transcribe ahora? */
  enUso: boolean;
  /** ¿Es el que baja «Instalar» en este equipo? En un PC ARM64 sale de la regla (ADR 022, enmienda 3). */
  porDefecto: boolean;
};

/**
 * Lo que Honestidad dice de un modelo: lo que hay, sin adornarlo. Desde el sprint 010, pista por pista: «cabe» o
 * «pasa» lo decide tu pista, como la regla; si la del cliente pasa del techo aunque la tuya quepa, se dice aparte.
 */
export type EstadoDeUnModelo =
  | { que: "sin-bajar" }
  | { que: "sin-medir" }
  | { que: "a-tiempo"; ms: number; clienteMs: number | null }
  | { que: "cliente-lento"; ms: number; clienteMs: number }
  | { que: "lento"; ms: number; clienteMs: number | null };

export function estadoDeUnModelo(
  m: ModeloEnEsteEquipo,
  techoMs: number,
): EstadoDeUnModelo {
  if (!m.bajado) return { que: "sin-bajar" };
  if (m.latenciaMs === null) return { que: "sin-medir" };
  const clienteMs = m.latenciaClienteMs;
  if (m.latenciaMs > techoMs)
    return { que: "lento", ms: m.latenciaMs, clienteMs };
  if (clienteMs !== null && clienteMs > techoMs)
    return { que: "cliente-lento", ms: m.latenciaMs, clienteMs };
  return { que: "a-tiempo", ms: m.latenciaMs, clienteMs };
}

/** Lo que hace cada atajo (sprint 007, `src-tauri/src/atajos.rs`). */
export type AccionDeAtajo =
  | "corte"
  | "transcript"
  | "ayuda"
  | "modo-de-voz"
  | "leer-la-pantalla"
  | "radar"
  | "anotar"
  | "fijar"
  | "guardar-la-propuesta"
  | "banda"
  | "callar";

/** Un atajo de este sistema: sus teclas y si el sistema lo dejó registrar. */
export type Atajo = {
  accion: AccionDeAtajo;
  /** «⌃⌥T» en macOS, «Alt+Shift+C» en Windows. */
  teclas: string;
  /** `false`: otra app ya tiene esa combinación y la tecla no responde. */
  registrado: boolean;
};

export type Plataforma = {
  sistema: Sistema;
  funciones: FuncionEnWindows[];
  transcripcion: Transcripcion | null;
  /** Las teclas de este sistema (sprint 007). */
  atajos: Atajo[];
};

/** Fuera de Tauri (tests, arnés de capturas) no hay a quién preguntar: se dice macOS, sin lista. */
export const PLATAFORMA_FUERA: Plataforma = {
  sistema: "macos",
  funciones: [],
  transcripcion: null,
  atajos: [],
};

/**
 * Lo que enseña una pantalla fuera de Tauri, donde no hay parte nativa a la que preguntar. `?plataforma=windows`
 * pinta la muestra de Windows (la del contrato); sin el parámetro, macOS. Es el mismo mecanismo que `?estado=` en
 * las demás pantallas: lo usan el e2e y el arnés de capturas, nunca la app instalada.
 */
export function plataformaFuera(
  busqueda: string = globalThis.location?.search ?? "",
): Plataforma {
  return new URLSearchParams(busqueda).get("plataforma") === "windows"
    ? PLATAFORMA_WINDOWS
    : PLATAFORMA_FUERA;
}

/**
 * Las teclas de macOS tal como las escriben los textos de la app y la maqueta, y la acción de cada una. Los textos
 * (`es.ts`, `en.ts`, la banda) están atados a la maqueta y no cambian; en Windows se pintan con las teclas de
 * Windows en su lugar.
 *
 * **Salen del contrato** (sprint 008, auditoría del S7, M7): `PLATAFORMA_MACOS.atajos` es lo que `atajos.rs::en_macos`
 * emite, así que no hay una segunda copia a mano que pueda separarse de Rust. Van de la más larga a la más corta
 * porque «⎋» sola va dentro de «⌥⎋» y la sustitución tiene que haber pasado ya por la larga.
 */
export const TECLAS_DE_MACOS: [string, AccionDeAtajo][] =
  PLATAFORMA_MACOS.atajos
    .map((a): [string, AccionDeAtajo] => [a.teclas, a.accion])
    .sort((a, b) => b[0].length - a[0].length);

/** Las teclas de un atajo de Windows en el idioma de quien lee: «Mayús» en español, «Shift» en inglés. */
export function teclasEnIdioma(teclas: string, idioma: "es" | "en"): string {
  return idioma === "es" ? teclas.replace("Shift", "Mayús") : teclas;
}

/**
 * **Un texto con las teclas de este sistema** (sprint 007). En macOS, el texto tal cual. En Windows, cada tecla de
 * macOS cambia por la de Windows de la misma acción (desde el sprint 008 las tienen todas, el radar incluido).
 */
export function conTeclasDeEsteSistema(
  texto: string,
  p: Plataforma,
  idioma: "es" | "en",
): string {
  if (!esWindows(p)) return texto;
  let salida = texto;
  for (const [mac, accion] of TECLAS_DE_MACOS) {
    const suyo = p.atajos.find((a) => a.accion === accion);
    if (suyo)
      salida = salida.split(mac).join(teclasEnIdioma(suyo.teclas, idioma));
  }
  return salida;
}

/** Los atajos que el sistema no dejó registrar: otra app tiene esa combinación. */
export function atajosSinRegistrar(p: Plataforma): Atajo[] {
  return p.atajos.filter((a) => !a.registrado);
}

/** ¿Es Windows? Honestidad enseña su bloque solo entonces. */
export function esWindows(p: Plataforma): boolean {
  return p.sistema === "windows";
}

/** Las funciones agrupadas por cuándo llegan, cada grupo en el orden del ADR. */
export function porCuando(p: Plataforma): Record<Cuando, ClaveDeWindows[]> {
  const grupos: Record<Cuando, ClaveDeWindows[]> = {
    si: [],
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

/**
 * **Lo que se pinta en el primer cuadro dentro de Tauri**, hasta que Rust contesta (sprint 008, auditoría del S7,
 * M12). Antes arrancaba siempre con macOS: en Windows el primer cuadro escribía las teclas `⌃⌥` y «Verificada solo en
 * Meet · macOS 26.6.2» hasta que llegaba la respuesta. El navegador embebido ya dice en qué sistema corre (WebView2
 * dice «Windows NT»), así que el primer cuadro parte de Windows sin esperar a nadie; las funciones y la transcripción
 * vacías, para no pintar cifras de muestra. Fuera de Tauri manda `fuera` (la muestra que pida la URL).
 */
export function plataformaInicial(
  fuera: Plataforma = plataformaFuera(),
  agente: string = globalThis.navigator?.userAgent ?? "",
  dentroDeTauri: boolean = hayTauri(),
): Plataforma {
  if (!dentroDeTauri) return fuera;
  return /Windows/i.test(agente)
    ? { ...PLATAFORMA_WINDOWS, funciones: [], transcripcion: null }
    : fuera;
}

/**
 * Pregunta a Rust al montar y, si se da, otra vez cuando cambie `refrescarCon` (la banda lo usa para enterarse de
 * una tecla que no se pudo registrar al encender el modo de voz).
 */
export function usePlataforma(
  fuera: Plataforma = plataformaInicial(),
  refrescarCon?: unknown,
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
  }, [refrescarCon]);
  return [p, setP];
}

/**
 * **Vuelve a preguntar cada `cadaMs` mientras `activo`** (sprint 008, B6 del S6): «Bajando el modelo…» se enciende
 * desde Idioma, y esta pantalla, que preguntó una sola vez al montar, no se enteraba de cuándo acaba. Solo mientras
 * dura la descarga: sin `activo` no hay temporizador.
 */
export function useRefrescoMientras(
  activo: boolean,
  refrescar: () => void,
  cadaMs = 2000,
): void {
  useEffect(() => {
    if (!activo) return;
    const id = setInterval(refrescar, cadaMs);
    return () => clearInterval(id);
  }, [activo, refrescar, cadaMs]);
}

/** «Medir otra vez»: whisper transcribe los 10 s de calibración y devuelve lo que sabe ahora. */
export function calibrarTranscripcion(): Promise<Plataforma | null> {
  return preguntar<Plataforma>("calibrar_transcripcion");
}

/** «Bajar el modelo pequeño» (0,5 GB), cuando el grande pasa de 4 s en este PC. */
export function bajarModeloPequeno(): Promise<Plataforma | null> {
  return preguntar<Plataforma>("bajar_modelo_pequeno");
}
