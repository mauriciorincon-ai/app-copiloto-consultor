import { useCallback, useEffect, useState } from "react";
import { escuchar, hayTauri, llamar, preguntar } from "./puente";
import type { Fuente } from "./ficha";
import type { PorQueNoAbrio } from "./cuaderno";
import { useT } from "./i18n";

/**
 * EL ENSAYO (C18, sprint 004, ADR 019) — lo que la pantalla Ensayo recibe de `src-tauri/src/ensayo/`.
 * Las formas cruzan la costura por `src/contrato.generado.ts` (regla 19). **Nada de aquí decide**: el
 * banco, la sesión, el oído y la evaluación viven en Rust; la pantalla pinta y conmuta.
 */

export type IdiomaDelEnsayo = "es" | "en";
/** De dónde sale una pregunta (`banco::De`). */
export type De = "propuesta" | "ficha" | "objeciones" | "modelo";
export type Fase = "preguntando" | "respondiendo" | "evaluada" | "cerrado";

export type Elegible = { ruta: string; nombre: string };
export type Cuentas = { propuesta: number; ficha: number; objeciones: number };

/** «Preparar»: con quién, con qué propuesta, cuántas preguntas y de dónde salen. */
export type Preparacion = {
  clientes: string[];
  cliente: string | null;
  propuestas: Elegible[];
  propuesta: string | null;
  topes: number[];
  tope: number;
  cuentas: Cuentas;
  idioma: IdiomaDelEnsayo;
  enriquecer: boolean;
  /** Este Mac sabe transcribir el idioma del ensayo. */
  transcribe: boolean;
  sinCorpus: boolean;
  /** Cuántos ensayos guardados hay con ese cliente, contados por el nombre del archivo (ADR 015, enmienda 4). */
  guardados: number;
};

export type PreguntaEnPantalla = {
  texto: string;
  de: De;
  seccion: string | null;
  fuente: string | null;
};
export type Evidencia = {
  titular: string;
  fuente: Fuente;
  citada: boolean;
  dichaPorTi: boolean;
};
export type Muletilla = { frase: string; veces: number };
export type Evaluacion = {
  evidencia: Evidencia[];
  tiempoMs: number;
  ppm: number | null;
  muletillas: Muletilla[];
};
export type Fila = {
  numero: number;
  texto: string;
  citadas: number;
  evidencia: number;
  tiempoMs: number | null;
  ppm: number | null;
  saltada: boolean;
};
export type Informe = {
  respondidas: number;
  saltadas: number;
  citadas: number;
  evidencia: number;
  ppmMedio: number | null;
  muletillas: number;
  laQueMas: Muletilla | null;
  tiempoMedioMs: number | null;
  filas: Fila[];
};
export type PorQueNoSeEnriquecio =
  | "apagado"
  | "sin-proveedor"
  | "fuera-del-esquema"
  | "nada-fundado"
  | "tarde"
  | "fallo";
export type EstadoDelBanco =
  | { que: "apagado" }
  | { que: "en-camino" }
  | { que: "sumadas"; cuantas: number }
  | { que: "no-se-enriquecio"; porque: PorQueNoSeEnriquecio };

/** Lo que la pantalla Ensayo enseña mientras ensayas y al terminar. */
export type VistaDelEnsayo = {
  fase: Fase;
  cliente: string;
  indice: number;
  total: number;
  pregunta: PreguntaEnPantalla | null;
  leyendo: boolean;
  cerrando: boolean;
  respuesta: string;
  transcurridoMs: number;
  evaluacion: Evaluacion | null;
  banco: EstadoDelBanco;
  informe: Informe | null;
};

/** De cuánto a cuánto cambió una cifra; la flecha la pone la pantalla. */
export type Cambio = { desde: number; hasta: number };
/** «Desde el primero»: cada cifra, del primer ensayo que la tiene al último. `null` sin dos ensayos con ella. */
export type DesdeElPrimero = {
  evidencia: Cambio | null;
  ritmo: Cambio | null;
  muletillas: Cambio | null;
  tiempo: Cambio | null;
};
/** Un ensayo en la tabla del progreso: las cuatro cifras de su informe, y cuándo. */
export type FilaDelProgreso = {
  /** «2026-10-04 09:12». */
  empezo: string;
  citadas: number;
  evidencia: number;
  ppmMedio: number | null;
  muletillas: number | null;
  tiempoMedioMs: number | null;
};
/** **Tu progreso con un cliente**: solo cifras; tus respuestas no cruzan (ADR 015, enmienda 4). */
export type Progreso = {
  cliente: string;
  filas: FilaDelProgreso[];
  /** Los ensayos más viejos que no caben en la tabla. */
  antes: number;
  desdeElPrimero: DesdeElPrimero;
};

/** Por qué no empezó. */
export type NoEmpezo =
  | { que: "en-reunion" }
  | { que: "sin-corpus" }
  | { que: "microfono"; porque: PorQueNoAbrio };

/** Los estados de `ensayo.html` que la pantalla dibuja fuera de Tauri (el gate de fidelidad los pide por la URL). */
export type EstadoDeMuestra =
  | "preparar"
  | "no-empezo"
  | "guardado"
  | "preguntando"
  | "del-modelo"
  | "respondiendo"
  | "evaluada"
  | "cerrado"
  | "progreso"
  | "progreso-borrar"
  | "sin-corpus";

const ESTADOS: EstadoDeMuestra[] = [
  "preparar",
  "no-empezo",
  "guardado",
  "preguntando",
  "del-modelo",
  "respondiendo",
  "evaluada",
  "cerrado",
  "progreso",
  "progreso-borrar",
  "sin-corpus",
];

export function estadoDeLaUrl(busqueda: string): EstadoDeMuestra {
  const pedido = new URLSearchParams(busqueda).get("estado");
  return ESTADOS.find((e) => e === pedido) ?? "preparar";
}

// ─── las muestras de la maqueta ──────────────────────────────────────────────────────────────────

/** «Preparar» de la maqueta: Páramo Azul, su propuesta, ocho preguntas (5 · 1 · 2). */
export function usePreparacionDeMuestra(estado: EstadoDeMuestra): Preparacion {
  const m = useT().ensayo.muestra;
  const sinCorpus = estado === "sin-corpus";
  return {
    clientes: [m.paramo, m.surDelValle],
    cliente: sinCorpus ? m.surDelValle : m.paramo,
    propuestas: sinCorpus ? [] : [{ ruta: "muestra", nombre: m.propuesta }],
    propuesta: sinCorpus ? null : "muestra",
    topes: [5, 8, 12],
    tope: 8,
    cuentas: sinCorpus
      ? { propuesta: 0, ficha: 0, objeciones: 0 }
      : { propuesta: 5, ficha: 1, objeciones: 2 },
    idioma: "es",
    enriquecer: false,
    transcribe: true,
    sinCorpus,
    // «1c · guardado»: vuelves a «preparar» con cuatro ensayos guardados con Páramo Azul.
    guardados: estado === "guardado" ? 4 : 0,
  };
}

/** «6 · tu progreso» de la maqueta: cuatro ensayos con Páramo Azul, del 21 sep al 04 oct. */
export function useProgresoDeMuestra(estado: EstadoDeMuestra): Progreso | null {
  const m = useT().ensayo.muestra;
  if (estado !== "progreso" && estado !== "progreso-borrar") return null;
  const fila = (
    empezo: string,
    citadas: number,
    ppm: number,
    muletillas: number,
    s: number,
  ): FilaDelProgreso => ({
    empezo,
    citadas,
    evidencia: 21,
    ppmMedio: ppm,
    muletillas,
    tiempoMedioMs: s * 1000,
  });
  return {
    cliente: m.paramo,
    filas: [
      fila("2026-09-21 10:05", 9, 161, 17, 81),
      fila("2026-09-27 18:30", 11, 150, 12, 69),
      fila("2026-10-02 08:45", 12, 143, 11, 62),
      fila("2026-10-04 09:12", 14, 138, 9, 58),
    ],
    antes: 0,
    desdeElPrimero: {
      evidencia: { desde: 9, hasta: 14 },
      ritmo: { desde: 161, hasta: 138 },
      muletillas: { desde: 17, hasta: 9 },
      tiempo: { desde: 81_000, hasta: 58_000 },
    },
  };
}

/** La pregunta 3 de 8 de la maqueta en cada estado (o la 9 de 10, del modelo). */
export function useVistaDeMuestra(
  estado: EstadoDeMuestra,
): VistaDelEnsayo | null {
  const m = useT().ensayo.muestra;
  const fase: Fase | null =
    estado === "preguntando" || estado === "del-modelo"
      ? "preguntando"
      : estado === "respondiendo"
        ? "respondiendo"
        : estado === "evaluada"
          ? "evaluada"
          : estado === "cerrado"
            ? "cerrado"
            : null;
  if (fase === null) return null;
  const fuente = (
    documento: string,
    seccion: string,
    unidad: Fuente["unidad"],
  ): Fuente => ({
    documento,
    seccion,
    unidad,
    conjeturada: false,
  });
  const delModelo = estado === "del-modelo";
  return {
    fase,
    cliente: m.paramo,
    indice: delModelo ? 8 : 2,
    total: delModelo ? 10 : 8,
    pregunta:
      fase === "cerrado"
        ? null
        : delModelo
          ? {
              texto: m.preguntaDelModelo,
              de: "modelo",
              seccion: m.quienDecide,
              fuente: null,
            }
          : {
              texto: m.pregunta,
              de: "propuesta",
              seccion: m.supuestos,
              fuente: null,
            },
    leyendo: fase === "preguntando",
    cerrando: false,
    respuesta: estado === "respondiendo" ? m.respuesta : "",
    transcurridoMs:
      estado === "respondiendo" ? 42_000 : estado === "evaluada" ? 72_000 : 0,
    evaluacion:
      fase === "evaluada"
        ? {
            evidencia: [
              {
                titular: m.evidencia1,
                fuente: fuente(m.paramo, m.supuestos, "propuesta"),
                citada: true,
                dichaPorTi: false,
              },
              {
                titular: m.evidencia2,
                fuente: fuente(m.surDelValle, m.cierre, "caso"),
                citada: true,
                dichaPorTi: false,
              },
              {
                titular: m.evidencia3,
                fuente: fuente(m.paramo, m.etapa2, "marco"),
                citada: false,
                dichaPorTi: false,
              },
            ],
            tiempoMs: 72_000,
            ppm: 142,
            muletillas: [
              { frase: m.oSea, veces: 3 },
              { frase: m.basicamente, veces: 1 },
            ],
          }
        : null,
    banco: delModelo ? { que: "sumadas", cuantas: 2 } : { que: "apagado" },
    informe:
      fase === "cerrado"
        ? {
            respondidas: 7,
            saltadas: 1,
            citadas: 14,
            evidencia: 21,
            ppmMedio: 138,
            muletillas: 9,
            laQueMas: { frase: m.oSea, veces: 5 },
            tiempoMedioMs: 58_000,
            filas: [
              {
                numero: 1,
                texto: m.fila1,
                citadas: 2,
                evidencia: 3,
                tiempoMs: 51_000,
                ppm: 131,
                saltada: false,
              },
              {
                numero: 2,
                texto: m.fila2,
                citadas: 3,
                evidencia: 3,
                tiempoMs: 44_000,
                ppm: 140,
                saltada: false,
              },
              {
                numero: 3,
                texto: m.pregunta,
                citadas: 2,
                evidencia: 3,
                tiempoMs: 72_000,
                ppm: 142,
                saltada: false,
              },
              {
                numero: 4,
                texto: m.fila4,
                citadas: 0,
                evidencia: 0,
                tiempoMs: null,
                ppm: null,
                saltada: true,
              },
              {
                numero: 5,
                texto: m.fila1,
                citadas: 2,
                evidencia: 3,
                tiempoMs: 50_000,
                ppm: 135,
                saltada: false,
              },
              {
                numero: 6,
                texto: m.fila2,
                citadas: 2,
                evidencia: 3,
                tiempoMs: 48_000,
                ppm: 137,
                saltada: false,
              },
              {
                numero: 7,
                texto: m.pregunta,
                citadas: 1,
                evidencia: 3,
                tiempoMs: 60_000,
                ppm: 139,
                saltada: false,
              },
              {
                numero: 8,
                texto: m.fila2,
                citadas: 2,
                evidencia: 3,
                tiempoMs: 55_000,
                ppm: 142,
                saltada: false,
              },
            ],
          }
        : null,
  };
}

// ─── los puentes ─────────────────────────────────────────────────────────────────────────────────

/**
 * El ensayo que hay en Rust, si hay: se pide al montarse y con cada señal `ensayo` (sin dato: lo que
 * dijiste no viaja en eventos). Fuera de Tauri, `null`: la pantalla usa la muestra de la URL.
 */
export function useEnsayo(): [VistaDelEnsayo | null, () => void] {
  const [vista, setVista] = useState<VistaDelEnsayo | null>(null);
  const pedir = useCallback(() => {
    void preguntar<VistaDelEnsayo | null>("estado_del_ensayo").then((v) =>
      setVista(v ?? null),
    );
  }, []);
  useEffect(() => {
    if (!hayTauri()) return;
    pedir();
    const baja = escuchar<unknown>("ensayo", pedir);
    const bajaCorte = escuchar<unknown>("corte", pedir);
    return () => {
      baja();
      bajaCorte();
    };
  }, [pedir]);
  return [vista, pedir];
}

/** Lo que «preparar» enseña para lo elegido. Se vuelve a pedir al cambiar cualquier cosa. */
export function usePreparacion(
  cliente: string | null,
  propuesta: string | null,
  tope: number,
): Preparacion | null {
  const [prep, setPrep] = useState<Preparacion | null>(null);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const pedir = () => {
      void preguntar<Preparacion>("preparar_el_ensayo", {
        cliente,
        propuesta,
        tope,
      }).then((p) => {
        if (vivo && p) setPrep(p);
      });
    };
    pedir();
    // El corpus se indexa con la pantalla abierta, y «Enriquecer el banco» se enciende en IA.
    const bajaCorpus = escuchar<unknown>("corpus", pedir);
    const bajaIa = escuchar<unknown>("ia", pedir);
    return () => {
      vivo = false;
      bajaCorpus();
      bajaIa();
    };
  }, [cliente, propuesta, tope]);
  return prep;
}

/** Empieza el ensayo. Si no empieza, la promesa se rechaza con el porqué (`NoEmpezo`). */
export function empezarElEnsayo(
  cliente: string,
  propuesta: string | null,
  tope: number,
  voz: boolean,
) {
  return preguntar<VistaDelEnsayo>("empezar_el_ensayo", {
    cliente,
    propuesta,
    tope,
    voz,
  });
}

export const ensayoListo = () => llamar("ensayo_listo");
export const ensayoRepetir = () => llamar("ensayo_repetir");
export const ensayoSaltar = () => llamar("ensayo_saltar");
export const ensayoTerminar = () => llamar("ensayo_terminar");
export const ensayoSiLoDije = (indice: number) =>
  llamar("ensayo_si_lo_dije", { indice });
export const cerrarElEnsayo = () => llamar("cerrar_el_ensayo");

/**
 * «Guardar con tus notas» (ADR 015, enmienda 4): cifra sin pedir nada. Si falla, la promesa se rechaza y el
 * informe sigue en Rust, entero.
 */
export const guardarElEnsayo = () => preguntar<null>("guardar_el_ensayo");
/** «Exportar como texto»: pide el desbloqueo de tus notas y dónde. `false` si cancelaste el diálogo. */
export const exportarElEnsayo = (idioma: string) =>
  preguntar<boolean>("exportar_el_ensayo", { idioma });
/** «Ver tu progreso»: abre tus ensayos de ese cliente con el desbloqueo de tus notas. Solo cifras. */
export const progresoDelEnsayo = (cliente: string, idioma: string) =>
  preguntar<Progreso>("progreso_del_ensayo", { cliente, idioma });
/** «Borrar los ensayos de este cliente»: al momento y sin abrirlos. Devuelve cuántos. */
export const borrarLosEnsayos = (cliente: string) =>
  preguntar<number>("borrar_los_ensayos", { cliente });

/** ¿Es un `NoEmpezo` lo que rechazó la promesa? */
export function esNoEmpezo(e: unknown): e is NoEmpezo {
  return typeof e === "object" && e !== null && "que" in e;
}

// ─── formatos ────────────────────────────────────────────────────────────────────────────────────

/** «1:12» — minutos y segundos, sin horas (una respuesta no dura una hora). */
export function reloj(ms: number): string {
  const s = Math.max(0, Math.floor(ms / 1000));
  return `${Math.floor(s / 60)}:${String(s % 60).padStart(2, "0")}`;
}

/**
 * «21 sep» en español, «Sep 21» en inglés: el día del ensayo en la tabla del progreso. Los meses salen del
 * diccionario (`ensayo.meses`), que los tiene escritos en la maqueta, y no de `Intl`: el de macOS escribe
 * «sept», y la tabla aprobada dice «sep».
 */
export function diaCorto(empezo: string, meses: readonly string[], idioma: string): string {
  const [, mes, dia] = /^\d{4}-(\d{2})-(\d{2})/.exec(empezo) ?? [];
  const nombre = meses[Number(mes) - 1];
  if (!nombre || !dia) return empezo;
  return idioma === "en" ? `${nombre} ${dia}` : `${dia} ${nombre}`;
}

/** La flecha de un cambio: hacia dónde fue la cifra, sin decir si es bueno o malo. */
export function flecha(c: Cambio): string {
  return c.hasta > c.desde ? "↑" : c.hasta < c.desde ? "↓" : "=";
}

/** «o sea» en español, “I mean” en inglés: las comillas son de la interfaz, no de la frase. */
export function entreComillas(frase: string, idioma: string): string {
  return idioma === "en" ? `“${frase}”` : `«${frase}»`;
}
