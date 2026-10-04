import { useT } from "./i18n";
import { useEffect, useState } from "react";
import { escuchar, hayTauri, preguntar } from "./puente";
import type { Fuente } from "./ficha";

/**
 * LA SÍNTESIS (C7) — lo que el webview recibe de `src-tauri/src/sintesis/` y de la capa que decide
 * quién redacta (ADR 010 y 011). Las formas cruzan la costura por `src/contrato.generado.ts`.
 */

export type Quien = "sistema" | "api" | "mock";
export type Confianza = "alta" | "media" | "baja";
export type Externo = "claude" | "groq";

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
  /** «Enriquecer el banco» del ensayo (sprint 004, ADR 019 §3). */
  enriquecer: boolean;
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
  enriquecer: false,
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

/**
 * Las tres peticiones de la maqueta (`ia.html`, «sprint 3 · lo que salió»), **fuera de Tauri**: lo que
 * hace posible comparar la vista con la maqueta. Dentro del producto, lo que Rust anotó de verdad.
 */
export function useSalioDeMuestra(): LoQueSalio[] {
  const m = useT().cuaderno.muestraSalio;
  return [
    {
      hora: "14:22",
      externo: "claude",
      sobre: m.sobre1,
      trozos: [
        { que: "texto", texto: `Client: ${m.t1} ` },
        { que: "tapado", marcador: "[CLIENTE_1]", original: "Páramo Azul" },
        { que: "texto", texto: ` ${m.t2} ` },
        { que: "tapado", marcador: "[PERSONA_1]", original: "Andrea Villalba" },
        { que: "texto", texto: ` ${m.t3}` },
      ],
      caracteres: 412,
      tapadas: 2,
      usd: 0.004,
    },
    { hora: "14:16", externo: "claude", sobre: m.sobre2, trozos: [], caracteres: 377, tapadas: 3, usd: 0.003 },
    { hora: "14:09", externo: "claude", sobre: m.sobre3, trozos: [], caracteres: 501, tapadas: 1, usd: null },
  ];
}

/** Las peticiones de la reunión, de la más nueva a la más vieja. Se vuelve a pedir con cada `ia`. */
export function useLoQueSalio(): LoQueSalio[] {
  const muestra = useSalioDeMuestra();
  const [salio, setSalio] = useState<LoQueSalio[]>(() => (hayTauri() ? [] : muestra));
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

/**
 * Los nombres de los proveedores externos, como la app los enseña. Gemini salió por decisión del usuario
 * (2026-09-29, ADR 011): su API no ofrece retención cero en ningún nivel.
 */
export const EXTERNOS: { id: Externo; nombre: string }[] = [
  { id: "claude", nombre: "Claude" },
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

export function enriquecerElBanco(si: boolean) {
  return preguntar<EstadoDeLaIa>("enriquecer_el_banco", { si });
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
