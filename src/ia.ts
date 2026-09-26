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

/** Los nombres de los tres proveedores externos, como la app los enseña. */
export const EXTERNOS: { id: Externo; nombre: string }[] = [
  { id: "claude", nombre: "Claude" },
  { id: "gemini", nombre: "Gemini" },
  { id: "groq", nombre: "Groq" },
];

/** El estado de la pantalla IA: se pregunta al montarse y se escucha el evento `ia`. */
export function useIa(): [EstadoDeLaIa, (e: EstadoDeLaIa) => void] {
  const [estado, setEstado] = useState<EstadoDeLaIa>(DE_MUESTRA);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    void preguntar<EstadoDeLaIa>("estado_de_la_ia").then((e) => {
      if (vivo && e) setEstado(e);
    });
    const baja = escuchar<EstadoDeLaIa>("ia", (e) => {
      if (vivo && e) setEstado(e);
    });
    return () => {
      vivo = false;
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
