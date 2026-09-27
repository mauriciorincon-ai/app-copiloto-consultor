import { useEffect, useState } from "react";
import { escuchar, hayTauri, preguntar } from "./puente";
import type { Fuente } from "./ficha";

/**
 * LA SÍNTESIS (C7) — lo que el webview recibe de `src-tauri/src/sintesis/` y de la capa que decide
 * quién redacta (ADR 010 y 011). Las formas cruzan la costura por `src/contrato.generado.ts`.
 */

export type Quien = "sistema" | "api" | "mock";
export type Confianza = "alta" | "media" | "baja";
export type Externo = "claude" | "gemini" | "groq";

export type PorQueNoRedacta =
  | "apple-intelligence-apagado"
  | "mac-no-compatible"
  | "no-disponible"
  | "modelo-descargandose"
  | "sin-puente"
  | "sin-clave"
  | "tope-del-mes";

/** La ficha que respalda una sugerencia: su titular y su fuente, siempre a la vista. */
export type FichaCitada = { titular: string; fuente: Fuente };

/** Una sugerencia. Solo existe fundada en una de las fichas que se le dieron al modelo. */
export type Sugerencia = {
  titular: string;
  linea: string;
  confianza: Confianza;
  ficha: FichaCitada;
  quien: Quien;
  /** «Modelo del sistema», «Claude Haiku»… */
  nombre: string;
};

export type EstadoDelApi = { encendida: boolean; externo: Externo; hayClave: boolean };

export type EstadoDeLaIa = {
  redactar: boolean;
  /** Quién redactaría ahora. `null`: nadie puede. */
  quien: Quien | null;
  /** Por qué el modelo del sistema no puede. `null`: puede. */
  sistema: PorQueNoRedacta | null;
  api: EstadoDelApi;
  latenciaMs: number | null;
  reunionUsd: number;
  mesUsd: number;
  topeUsd: number;
};

/** Lo que la maqueta dibuja en «así se ve hoy · sprint 2»: nadie redacta, todo apagado. */
const DE_MUESTRA: EstadoDeLaIa = {
  redactar: false,
  quien: null,
  sistema: "apple-intelligence-apagado",
  api: { encendida: false, externo: "claude", hayClave: false },
  latenciaMs: null,
  reunionUsd: 0,
  mesUsd: 0,
  topeUsd: 10,
};

/**
 * **Lo que salió al API en esta reunión** (auditoría del S2, B37): el texto exacto, trozo a trozo,
 * con lo que la bóveda reemplazó **en tu Mac** al lado de cada marcador. Solo en memoria: lo vacían el
 * corte y el final de la sesión, y se pide por comando desde esta ventana —el texto no viaja en
 * ningún evento—.
 */
export type Trozo =
  | { que: "texto"; texto: string }
  | { que: "tapado"; marcador: string; original: string };

export type LoQueSalio = {
  hora: string;
  externo: Externo;
  /** El titular de la ficha que provocó la petición. */
  sobre: string;
  trozos: Trozo[];
  caracteres: number;
  tapadas: number;
  /** `null` hasta que el proveedor contesta (o si no contestó). */
  usd: number | null;
};

/** Las peticiones de la reunión, de la más nueva a la más vieja. Se vuelve a pedir con cada `ia`. */
export function useLoQueSalio(): LoQueSalio[] {
  const [salio, setSalio] = useState<LoQueSalio[]>([]);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const pedir = () => {
      void preguntar<LoQueSalio[]>("lo_que_salio_al_api").then((s) => {
        if (vivo) setSalio(s ?? []);
      });
    };
    pedir();
    const baja = escuchar<unknown>("ia", pedir);
    const bajaCorte = escuchar<unknown>("corte", pedir);
    return () => {
      vivo = false;
      baja();
      bajaCorte();
    };
  }, []);
  return salio;
}

/** Los nombres de los tres proveedores externos, como la app los enseña. */
export const EXTERNOS: { id: Externo; nombre: string }[] = [
  { id: "claude", nombre: "Claude" },
  { id: "gemini", nombre: "Gemini" },
  { id: "groq", nombre: "Groq" },
];

/**
 * El estado de la pantalla IA: se pregunta al montarse, **al volver a la ventana** y con cada evento
 * `ia`. Lo del foco es de la casilla 6 del S3: si el usuario apaga Apple Intelligence en Ajustes y
 * vuelve, nadie emite `ia` —Rust no se entera de Ajustes—, y la pantalla seguía diciendo «en tu Mac».
 * Es lo mismo que Permisos hace desde el S1.
 */
export function useIa(): [EstadoDeLaIa, (e: EstadoDeLaIa) => void] {
  const [estado, setEstado] = useState<EstadoDeLaIa>(DE_MUESTRA);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const pedir = () => {
      void preguntar<EstadoDeLaIa>("estado_de_la_ia").then((e) => {
        if (vivo && e) setEstado(e);
      });
    };
    pedir();
    globalThis.addEventListener("focus", pedir);
    const baja = escuchar<EstadoDeLaIa>("ia", (e) => {
      if (vivo && e) setEstado(e);
    });
    return () => {
      vivo = false;
      globalThis.removeEventListener("focus", pedir);
      baja();
    };
  }, []);
  return [estado, setEstado];
}

export function redactarSugerencias(si: boolean) {
  return preguntar<EstadoDeLaIa>("redactar_sugerencias", { si });
}

/** Encender el API exige su clave: sin ella, Rust contesta `sin-clave` y la promesa se rechaza. */
export function apiExterna(encendida: boolean, externo: Externo) {
  return preguntar<EstadoDeLaIa>("api_externa", { encendida, externo });
}

export function guardarClave(externo: Externo, clave: string) {
  return preguntar<EstadoDeLaIa>("guardar_clave_del_api", { externo, clave });
}

export function borrarClave(externo: Externo) {
  return preguntar<EstadoDeLaIa>("borrar_clave_del_api", { externo });
}

/**
 * «USD 0,031» en español y «USD 0.031» en inglés. Dos decimales, y tres solo cuando el tercero dice
 * algo en una cifra de menos de un dólar: un envío cuesta centésimas, y «USD 0,03» escondería la
 * mitad de lo que se gastó.
 */
export function dolares(usd: number, idioma: string): string {
  const decimales = usd > 0 && usd < 1 && Math.round(usd * 1000) % 10 !== 0 ? 3 : 2;
  const t = usd.toFixed(decimales);
  return `USD ${idioma === "en" ? t : t.replace(".", ",")}`;
}
