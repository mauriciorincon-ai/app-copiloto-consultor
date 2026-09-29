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

/** Qué regla saltó (ADR 016 §1): el catálogo publicado, sin modelo. */
export type Regla = "cifra" | "compromiso" | "choque" | "nombre" | "pregunta";

/** De quién salió: por la pista y el eco, nunca por la voz (regla dura 4). */
export type De = "tuyo" | "cliente";

/**
 * Una propuesta: lo que Notas y la banda enseñan y, si dices que sí, lo que se guarda. Tuya, la
 * frase; del cliente, un fragmento de ≤ 8 palabras, jamás su turno (ADR 016 §2).
 */
export type Propuesta = {
  regla: Regla;
  de: De;
  texto: string;
  /** Solo `choque`: lo que dice tu ficha fijada («tres»). */
  ficha: string | null;
  /** Solo `choque`: la sección de esa ficha. */
  seccion: string | null;
  /** «14:16», la hora del turno. */
  hora: string;
};

/** Una propuesta que espera tu decisión, con el número con que se guarda o se descarta. */
export type EnEspera = Propuesta & { id: number };

/** Lo que la banda recibe: la última propuesta sin decidir, o nada. */
export type LineaDePropuesta = Propuesta | null;

/** Cuánto esperan en la bandeja: al cerrar (0) · 1 h · 3 h · fin del día · 24 h (ADR 016 §4). */
export type Ventana = "0" | "1h" | "3h" | "fin" | "24h";

/** La bandeja que vence antes (`la_bandeja`). */
export type VistaDeLaBandeja = {
  archivo: string;
  /** Segundos Unix. */
  vence: number;
  bytes: number;
  /** `null`: cerrada con llave; abrirla pide Touch ID. */
  propuestas: Propuesta[] | null;
  /** Las que guardaste desde aquí mientras la app sigue abierta. */
  guardadas: Propuesta[];
  /** Cuántas bandejas más esperan. */
  mas: number;
  ventana: Ventana;
};

/** Lo que Honestidad dice de la bandeja, sin abrirla (`estado_de_la_bandeja`). */
export type EstadoDeLaBandeja = {
  /** El vencimiento más próximo, en segundos Unix; `null` sin bandeja. */
  vence: number | null;
  /** La tarea de borrado no corrió mientras la app estaba cerrada (ADR 016 §5). */
  noCorrio: boolean;
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
  /** Las que esperan tu decisión, en el orden en que llegaron. */
  propuestas: EnEspera[];
  /** Se llegó al tope y alguna no entró. */
  lleno: boolean;
  /** Cuánto esperarán en la bandeja las que no decidas. */
  ventana: Ventana;
  /** macOS no dejó proteger el cuaderno al empezar: Notas lo dice (auditoría del S3, B4). */
  sinProteger: boolean;
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


/**
 * Las reuniones guardadas (`reuniones_guardadas`). Dónde viven no viaja: siempre en la carpeta
 * privada de la app (ADR 016, decisión A), que la interfaz nombra y enseña con «Mostrar en Finder».
 */
export type ListaDeReuniones = {
  reuniones: ReunionGuardada[];
};

// ---------------------------------------------------------------------------------------------
// Los hooks. Tras la mirada 19 (aprobada el 2026-09-27).
// ---------------------------------------------------------------------------------------------


/**
 * Las vistas de Notas. Dentro del producto las decide el cuaderno (y la bandeja); fuera, la URL, que
 * es como el arnés de fidelidad recorre cada estado de `notas.html`.
 */
export type VistaDeNotas =
  | "durante"
  | "propuestas"
  | "al-cerrar"
  | "al-cerrar-bandeja"
  | "al-cerrar-cero"
  | "archivo"
  | "exportar"
  | "bandeja"
  | "bandeja-llave"
  | "vencida";

/** Las dos propuestas de la maqueta («sprint 3 · durante, con propuestas»): tuya y un choque. */
function propuestasDeMuestra(m: { propuestaTuya: string; choqueDicho: string; choqueFicha: string }): EnEspera[] {
  return [
    { id: 1, regla: "cifra", de: "tuyo", texto: m.propuestaTuya, ficha: null, seccion: null, hora: "14:16" },
    { id: 2, regla: "choque", de: "cliente", texto: m.choqueDicho, ficha: m.choqueFicha, seccion: "§3.2", hora: "14:18" },
  ];
}

/** El cuaderno de ahora, **fuera de Tauri**: lo que dibuja la maqueta. */
export function useMuestraDelCuaderno(vista: VistaDeNotas): VistaDelCuaderno {
  const m = useT().notas.muestra;
  const conPropuestas = vista === "propuestas";
  const durante = vista === "durante" || conPropuestas;
  const cerrando = vista === "al-cerrar" || vista === "al-cerrar-bandeja" || vista === "al-cerrar-cero";
  const quedan = vista === "al-cerrar-bandeja" || vista === "al-cerrar-cero" ? 4 : 0;
  return {
    nota: conPropuestas ? m.nota1 : `${m.nota1}\n${m.nota2}`,
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
      propuestas: conPropuestas ? 1 : quedan ? 3 : 0,
      sinDecidir: conPropuestas ? 2 : quedan,
      bytesNota: 2_048,
      bytesAcuerdos: 1_024,
      bytesFijadas: 1_024,
      bytesTurnos: 0,
    },
    conservarMisTurnos: false,
    abierta: durante || cerrando,
    escuchando: durante,
    previsto: { fecha: "2026-09-20", minutos: 47, cliente: null, archivo: "reunion-2026-09-20-1402.ghost" },
    turnosDelCliente: 63,
    lecturas: 9,
    retencion: "90d",
    propuestas: conPropuestas ? propuestasDeMuestra(m) : [],
    lleno: false,
    ventana: vista === "al-cerrar-cero" ? "0" : "3h",
    sinProteger: false,
  };
}

/**
 * «Ahora» fuera de Tauri: la cuenta atrás de la maqueta se queda quieta en 2:41:08, y la bandeja
 * vencida dice «a las 17:32» en cualquier zona horaria (lo pinta la maqueta, no el reloj).
 */
export const AHORA_DE_MUESTRA = 1_790_517_600;
export const HORA_DE_MUESTRA = "17:32";

/** La bandeja de la maqueta («sprint 3 · la bandeja» / «con llave» / «vencida»), fuera de Tauri. */
export function useMuestraDeLaBandeja(vista: VistaDeNotas): VistaDeLaBandeja | null {
  const m = useT().notas.muestra;
  if (vista !== "bandeja" && vista !== "bandeja-llave" && vista !== "vencida") return null;
  const [tuya, choque] = propuestasDeMuestra(m);
  // Las que no caben en las tres filas no se ven: basta con que cuenten.
  const otra = (hora: string): Propuesta => ({ ...tuya, hora });
  const guardada: Propuesta = { ...tuya, texto: m.aceptada, hora: "14:25" };
  const base = {
    archivo: "reunion-2026-09-20-1402.ghost",
    vence: AHORA_DE_MUESTRA + 2 * 3_600 + 41 * 60 + 8,
    bytes: 4_096,
    mas: 0,
    ventana: "3h" as Ventana,
  };
  if (vista === "bandeja-llave") return { ...base, propuestas: null, guardadas: [], mas: 1 };
  if (vista === "vencida") {
    // Al vencer: 4 sin decidir que mueren y 3 guardadas que siguen en su reunión.
    const quedaban = [tuya, choque, otra("14:21"), otra("14:40")];
    return { ...base, vence: AHORA_DE_MUESTRA, propuestas: quedaban, guardadas: [guardada, guardada, guardada] };
  }
  const resto = [21, 22, 23, 24].map((m) => otra(`14:${m}`));
  return { ...base, propuestas: [tuya, choque, ...resto], guardadas: [guardada] };
}

/** Las tres reuniones de la maqueta (`notas.html`, «sprint 3 · el archivo»), a su fecha. */
export const REUNIONES_DE_MUESTRA: ListaDeReuniones = {
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
    hayTauri() ? { reuniones: [] } : REUNIONES_DE_MUESTRA,
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

/** «Guardar cifrado y cerrar». Rechaza con el motivo si no se pudo: la nota se queda. No devuelve lo
 *  guardado: nadie lo leía (auditoría del S3, B13). */
export function guardarLaReunion(): Promise<null> {
  return preguntar<null>("guardar_la_reunion");
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

/** «Mostrar en Finder»: la carpeta de tus notas o, con `archivo`, esa reunión seleccionada. */
export function mostrarEnFinder(archivo?: string): Promise<boolean> {
  return llamar("mostrar_las_notas_en_finder", { archivo: archivo ?? null });
}

/** «Anotar para después» en la banda: lo mismo que ⌃⌥N. */
export function irANotas() {
  void llamar("ir_a_notas");
}

// ---------------------------------------------------------------------------------------------
// Las propuestas y la bandeja (ADR 016).
// ---------------------------------------------------------------------------------------------

export function guardarPropuesta(id: number): Promise<boolean> {
  return llamar("guardar_propuesta", { id });
}

export function descartarPropuesta(id: number): Promise<boolean> {
  return llamar("descartar_propuesta", { id });
}

/** La ventana de la bandeja, elegida en «al cerrar». Se recuerda. */
export function fijarVentana(ventana: Ventana): Promise<boolean> {
  return llamar("fijar_ventana", { ventana });
}

/** «Abrir con Touch ID». Rechaza si no se desbloqueó: la bandeja sigue cerrada con llave. */
export function abrirLaBandeja(archivo: string, idioma: string): Promise<boolean> {
  return llamar("abrir_la_bandeja", { archivo, idioma });
}

export function decidirEnLaBandeja(archivo: string, indice: number, guardar: boolean): Promise<boolean> {
  return llamar("decidir_en_la_bandeja", { archivo, indice, guardar });
}

export function decidirTodaLaBandeja(archivo: string, guardar: boolean): Promise<boolean> {
  return llamar("decidir_toda_la_bandeja", { archivo, guardar });
}

/** La ventana, cambiada desde la bandeja: la vuelve a sellar. «Al cerrar» se la lleva ahora. */
export function cambiarLaVentana(archivo: string, ventana: Ventana): Promise<boolean> {
  return llamar("cambiar_la_ventana", { archivo, ventana });
}

/**
 * La bandeja que vence antes. Se pregunta al montar, al volver a la ventana, cuando Rust avisa de que
 * el cuaderno cambió (al cerrar una reunión, al barrer lo vencido) y tras cada acción (`volver`).
 */
export function useBandeja(muestra: VistaDeLaBandeja | null): [VistaDeLaBandeja | null, boolean, () => void] {
  const [bandeja, setBandeja] = useState<VistaDeLaBandeja | null>(() => (hayTauri() ? null : muestra));
  const [lista, setLista] = useState(!hayTauri());
  const [vuelta, setVuelta] = useState(0);
  const volver = useCallback(() => setVuelta((v) => v + 1), []);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const ahora = () => {
      void preguntar<VistaDeLaBandeja | null>("la_bandeja").then((b) => {
        if (!vivo) return;
        setBandeja(b);
        setLista(true);
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
  return [bandeja, lista, volver];
}

/** Lo que Honestidad dice de la bandeja, sin abrirla. Fuera de Tauri, lo de la maqueta pedida. */
export function useEstadoDeLaBandeja(muestra: EstadoDeLaBandeja): EstadoDeLaBandeja {
  const [estado, setEstado] = useState<EstadoDeLaBandeja>(() =>
    hayTauri() ? { vence: null, noCorrio: false } : muestra,
  );
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const ahora = () => {
      void preguntar<EstadoDeLaBandeja>("estado_de_la_bandeja").then((e) => {
        if (vivo && e) setEstado(e);
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
  }, []);
  return estado;
}

/** Los segundos que le quedan a `vence`, contados cada segundo. Fuera de Tauri, quietos. */
export function useQuedan(vence: number | null): number | null {
  const [ahora, setAhora] = useState(() => (hayTauri() ? Date.now() / 1000 : AHORA_DE_MUESTRA));
  useEffect(() => {
    if (!hayTauri() || vence === null) return;
    const reloj = globalThis.setInterval(() => setAhora(Date.now() / 1000), 1_000);
    return () => globalThis.clearInterval(reloj);
  }, [vence]);
  return vence === null ? null : Math.max(0, Math.floor(vence - ahora));
}

/**
 * **La línea de la banda** (mirada 20): la última propuesta sin decidir, o nada. La manda Rust por el
 * evento `propuesta` a la banda y solo a ella. Fuera de Tauri, la de la maqueta si se pide.
 */
export function useLineaDePropuesta(muestra: Propuesta | null): Propuesta | null {
  const [linea, setLinea] = useState<Propuesta | null>(() => (hayTauri() ? null : muestra));
  useEffect(() => escuchar<LineaDePropuesta>("propuesta", (p) => setLinea(p ?? null)), []);
  return linea;
}

/**
 * **La señal «fijada»** (mirada 20, fila 9): ⌃⌥P fijó la ficha que la banda enseña. Dura mientras
 * esa ficha siga en la banda: una ficha nueva (`clave`) la apaga.
 */
export function useFijada(clave: unknown, deLaMaqueta: boolean): boolean {
  const [fijada, setFijada] = useState(deLaMaqueta);
  useEffect(() => escuchar<boolean>("fijada", (si) => setFijada(si)), []);
  useEffect(() => {
    if (hayTauri()) setFijada(false);
  }, [clave]);
  return fijada;
}

/** «2 h 41 min» (o «41 min») de la cuenta atrás, y «2 h 41» del chip. */
export function duracion(segundos: number, t: { h: string; min: string }): [string, string] {
  const h = Math.floor(segundos / 3_600);
  const m = Math.floor((segundos % 3_600) / 60);
  return [
    h > 0 ? `${h} ${t.h} ${m} ${t.min}` : `${m} ${t.min}`,
    h > 0 ? `${h} ${t.h} ${String(m).padStart(2, "0")}` : `${m} ${t.min}`,
  ];
}

/** «2:41:08», la cuenta atrás de la maqueta. */
export function reloj(segundos: number): string {
  const h = Math.floor(segundos / 3_600);
  const m = Math.floor((segundos % 3_600) / 60);
  const s = segundos % 60;
  return `${h}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
}
