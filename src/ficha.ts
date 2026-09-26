import { useEffect, useState } from "react";
import { escuchar, hayTauri, preguntar } from "./puente";
import { useT } from "./i18n";
import type { Pista, Turno } from "./cuaderno";

/**
 * LA FICHA EN LA BANDA — lo que la app encontró en el corpus del consultor.
 *
 * Vive al lado de `turnos.ts` y por la misma razón: lo usa la **banda**, no el cuaderno.
 *
 * **Fuera de Tauri son los datos de la maqueta**, y eso es lo que hace posible el gate de
 * FIDELIDAD: el arnés de capturas abre la banda en el navegador y compara contra
 * `docs/diseno/banda.html`. Dentro del producto son los de verdad, y si no hay ficha no se
 * enseña ninguna — pintar la muestra «Páramo Azul» dentro de la app sería inventarle al usuario
 * un corpus que no tiene.
 */

export type Unidad = "propuesta" | "marco" | "caso" | "cliente" | "perfil";

export type MotivoDelDisparo =
  | "pregunta"
  | "cifra"
  | "terminoDelCorpus"
  | "silencioLargo"
  | "atajo"
  /** La pantalla que comparte el cliente cambió y trae una cifra o uno de tus términos (C8). */
  | "pantalla";

export type Fuente = {
  documento: string;
  seccion: string | null;
  unidad: Unidad | null;
  /** La sección la conjeturó el lector por la forma del texto (PDF), nadie la escribió. */
  conjeturada: boolean;
};

export type Acumulada = { unidad: Unidad | null; texto: string };

export type Ficha = {
  clase: "ficha";
  titular: string;
  linea: string;
  lineaLarga: string;
  fuente: Fuente;
  acumuladas: Acumulada[];
};

/**
 * Cuál maniobra del catálogo. Es un identificador y no el texto: el copy es bilingüe y vive en
 * el diccionario, donde el gate que exige que toda cadena esté en la maqueta puede vigilarlo.
 */
export type IdDeManiobra =
  "credencial" | "cifra" | "plazo" | "referencia" | "contrato" | "generica";

export type SinResultado = {
  clase: "sinResultado";
  /** Los términos con los que se buscó de verdad. Si la app entendió mal, se ve en el acto. */
  buscado: string;
  cercanas: Acumulada[];
  maniobra: IdDeManiobra;
};

export type Respuesta = Ficha | SinResultado;

export type Aparicion = Respuesta & {
  motivo: MotivoDelDisparo;
  /** Del fin de turno a la ficha. El presupuesto del sprint son 4 000 ms. */
  ms: number;
  hora: string;
};

/**
 * Lo que el evento «escucha» trae, en sus cinco formas.
 *
 * **Va etiquetada por dentro**: Rust la serializa con `#[serde(tag = "que")]`, así que lo que
 * llega es `{"que":"aparece", …los campos de la aparición}` y no `{"Aparece":{…}}`. Estuvo escrito
 * al revés todo el sprint y **la ficha automática no llegó nunca a la banda**: `n.Aparece` era
 * `undefined` en cada evento, la banda se quedaba en «esperando» toda la reunión y solo funcionaba
 * `⌃⌥A`, que va por otro camino. Ningún test podía verlo —todos corren fuera de Tauri, donde esta
 * suscripción no se monta— y la cobertura lo delataba desde dos fases antes con este bloque sin
 * cubrir. Lo encontró la auditoría del sprint (hallazgo C1).
 *
 * Desde entonces las dos copias del contrato las compara un gate: `src/contrato.generado.ts` lo
 * escribe Rust con el serde de producción, y `pnpm typecheck` falla si un campo deja de encajar.
 */
export type Novedad =
  | { que: "empieza"; pista: Pista }
  | ({ que: "turno" } & Turno)
  | {
      que: "sin-texto";
      pista: Pista;
      desdeMs: number;
      hastaMs: number;
      motivo: string;
    }
  | { que: "ruido"; pista: Pista; duracionMs: number }
  | ({ que: "aparece" } & Aparicion)
  /** `⌃⌥L` leyó la pantalla y no había texto: la banda contesta igual, porque alguien preguntó. */
  | { que: "nada-en-pantalla"; hora: string };

export type LoQueLaBandaEnseña = {
  aparicion: Aparicion | null;
  /** El cliente terminó de hablar y todavía no hay respuesta. Es el estado «buscando». */
  buscando: boolean;
  /**
   * `⌃⌥L` leyó la pantalla y no había texto: la hora a la que se pidió. La banda contesta igual
   * —«Leí la pantalla: no hay texto que buscar.»—, porque alguien preguntó (mirada 17-quater).
   */
  nadaEnPantalla: string | null;
};

/**
 * La última aparición, y si hay una búsqueda en marcha.
 *
 * Escucha tres caminos porque son tres: el turno del cliente abre el «buscando», la aparición
 * automática llega dentro del evento `escucha`, y la de `⌃⌥A` llega por su propio evento y hay
 * que ir a buscarla — el atajo se salta la espera entre fichas, y hacerle esperar al evento
 * común le quitaría justo eso.
 *
 * `paraLaMuestra` solo pinta **fuera de Tauri**: es el estado que el arnés de capturas pide por
 * URL, y lo que sostiene el gate de FIDELIDAD. Dentro del producto no se mira.
 */
export function useFicha(
  paraLaMuestra: "ficha" | "sin-resultado" | string,
): LoQueLaBandaEnseña {
  const m = useT().banda.muestra;
  const [ficha, setFicha] = useState<Aparicion | null>(() =>
    hayTauri() ? null : deMuestra(m, paraLaMuestra),
  );
  const [buscando, setBuscando] = useState(false);
  const [nadaEnPantalla, setNada] = useState<string | null>(() =>
    !hayTauri() && paraLaMuestra === "pantalla-nada" ? m.hora3 : null,
  );

  useEffect(() => {
    if (!hayTauri()) return;
    const bajas = [
      escuchar<Novedad>("escucha", (n) => {
        // Un turno del cliente puede acabar en ficha o en nada, y hasta saberlo la banda dice
        // que está buscando. El eco no cuenta: es el consultor oyéndose a sí mismo.
        if (n?.que === "turno" && n.pista === "sistema" && !n.eco) {
          setBuscando(true);
          setNada(null);
        }
        if (n?.que === "aparece") {
          setFicha(n);
          setBuscando(false);
          setNada(null);
        }
        // La lectura pedida no encontró texto. Se contesta encima de lo que hubiera: fue lo
        // último que el usuario pidió, y es lo que espera ver.
        if (n?.que === "nada-en-pantalla") setNada(n.hora);
      }),
      escuchar("ficha", () => {
        setBuscando(true);
        setNada(null);
        // `null` es «todavía no he oído nada del cliente»: la banda vuelve a lo que enseñaba. Y
        // pase lo que pase —también si el puente falla—, el «buscando» se cierra: una banda que se
        // queda buscando para siempre es la avería que la corrida en vivo encontró.
        void preguntar<Aparicion>("pedir_ficha")
          .then((a) => {
            if (a !== null) setFicha(a);
          })
          .catch(() => undefined)
          .finally(() => setBuscando(false));
      }),
      // Tras el kill-switch no queda ficha en pantalla: la promesa es que no queda nada.
      escuchar("corte", () => {
        setFicha(null);
        setBuscando(false);
        setNada(null);
      }),
    ];
    return () => bajas.forEach((b) => b());
  }, []);

  return { aparicion: ficha, buscando, nadaEnPantalla };
}

/** Los datos «Páramo Azul» de la maqueta, con los textos del diccionario. */
function deMuestra(
  m: ReturnType<typeof useT>["banda"]["muestra"],
  estado: string,
): Aparicion {
  // «pregunta · 1,2 s», como `banda.html` (mirada 17-bis).
  const comun = { motivo: "pregunta" as const, ms: 1_200, hora: m.hora1 };
  if (estado === "sin-resultado") {
    return {
      clase: "sinResultado",
      buscado: m.buscado,
      cercanas: [
        { unidad: "marco", texto: m.cercana1 },
        { unidad: "propuesta", texto: m.cercana2 },
        { unidad: "caso", texto: m.cercana3 },
      ],
      maniobra: "credencial",
      ...comun,
    };
  }
  // La ficha que sale de un PDF, con su sección conjeturada: «cifra · 0,9 s».
  if (estado === "ficha-pdf") {
    return {
      clase: "ficha",
      titular: m.titularPdf,
      linea: m.lineaPdf,
      lineaLarga: m.lineaPdf,
      fuente: { documento: m.fuentePdf, seccion: null, unidad: "caso", conjeturada: true },
      acumuladas: [],
      motivo: "cifra",
      ms: 900,
      hora: m.hora1,
    };
  }
  // La ficha que trajo la pantalla, sin que nadie preguntara: «en pantalla · 0,8 s».
  if (estado === "ficha-pantalla") {
    return {
      clase: "ficha",
      titular: m.titularPantalla,
      linea: m.lineaPantalla,
      lineaLarga: m.lineaPantalla,
      fuente: { documento: m.fuentePantalla, seccion: null, unidad: "propuesta", conjeturada: false },
      acumuladas: [],
      motivo: "pantalla",
      ms: 800,
      hora: m.hora1,
    };
  }
  return {
    clase: "ficha",
    titular: m.titular,
    linea: m.linea,
    lineaLarga: m.lineaLarga,
    // Las unidades van por su CLAVE, jamás por su etiqueta traducida. Castear la etiqueta a
    // clave funcionaba en español por casualidad —«propuesta» es las dos cosas— y dejaba la
    // unidad vacía en inglés. Lo cazó el gate de fidelidad: ocho encuadres en inglés y ninguno
    // en español.
    fuente: {
      documento: m.fuente,
      seccion: null,
      unidad: "propuesta",
      conjeturada: false,
    },
    acumuladas: [
      { unidad: "marco", texto: m.acumulada1 },
      { unidad: "caso", texto: m.acumulada2 },
    ],
    ...comun,
  };
}
