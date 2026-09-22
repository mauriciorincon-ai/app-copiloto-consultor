import { useEffect, useState } from "react";
import { escuchar, hayTauri, llamar, preguntar } from "./puente";
import { useT } from "./i18n";

/**
 * EL PUENTE DEL CUADERNO — lo que las pantallas de la ventana principal preguntan a lo nativo.
 *
 * Todo lo de aquí sigue la misma regla que `useAcoplada`: **se pregunta, no se supone**. Las tres
 * pantallas de la fase 2 muestran estado del SISTEMA —qué reunión hay abierta, qué permisos
 * concedió el usuario, cuántos bytes salieron—, y ese estado cambia fuera de la app: el usuario
 * abre Zoom, macOS revalida el permiso de pantalla a mitad de una reunión.
 *
 * **Fuera de Tauri hay valores de muestra**, los mismos que la maqueta dibuja. No es una puerta
 * trasera: es lo que hace posible el gate de FIDELIDAD, que compara estas pantallas contra
 * `docs/diseno/*.html` en un navegador. Mueren cuando el producto tenga de dónde leerlos de
 * verdad — igual que la muestra de la banda.
 */

export type Proteccion = "Verificada" | "SinVerificar";

export type Reunion =
  | { que: "ninguna" }
  | { que: "detectada"; cliente: string; titulo: string | null; proteccion: Proteccion }
  | { que: "no-se-puede-saber"; motivo: string };

export type EstadoPermiso = "sin-conceder" | "concedido" | "denegado" | "no-se-sabe";

export type Permisos = {
  microfono: EstadoPermiso;
  pantalla: EstadoPermiso;
  accesibilidad: EstadoPermiso;
};

/**
 * Lo que la maqueta dibuja, para el navegador y los tests. El título sale del **diccionario**,
 * no de una constante: escrito a mano en español, el cruce en inglés del gate de fidelidad
 * mostraba texto español dentro de una pantalla inglesa — 0,34 % de divergencia y, sobre todo,
 * una muestra que no es bilingüe en una app que promete serlo en todo.
 */
function reunionDeMuestra(titulo: string): Reunion {
  return { que: "detectada", cliente: "Google Meet", titulo, proteccion: "Verificada" };
}

const PERMISOS_DE_MUESTRA: Permisos = {
  microfono: "concedido",
  pantalla: "sin-conceder",
  accesibilidad: "concedido",
};

/**
 * Pregunta una vez al montar y cuando el usuario vuelve a la ventana.
 *
 * El `focus` es el disparador honesto: el usuario se va a Ajustes del Sistema a conceder un
 * permiso, o abre Zoom, y vuelve. Sondear cada pocos segundos gastaría llamadas a la API de
 * accesibilidad —que van a otro proceso— para preguntar por algo que solo cambia cuando el
 * usuario hace algo fuera.
 */
function usePreguntaAlVolver<T>(comando: string, deMuestra: T): T {
  const [valor, setValor] = useState<T>(deMuestra);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const preguntarAhora = () => {
      void preguntar<T>(comando).then((v) => {
        if (vivo && v !== null) setValor(v);
      });
    };
    preguntarAhora();
    globalThis.addEventListener("focus", preguntarAhora);
    return () => {
      vivo = false;
      globalThis.removeEventListener("focus", preguntarAhora);
    };
  }, [comando]);
  return valor;
}

export function useReunion(): Reunion {
  const t = useT().cuaderno;
  return usePreguntaAlVolver<Reunion>("reunion_abierta", reunionDeMuestra(t.tituloDeMuestra));
}

export function usePermisos(): Permisos {
  return usePreguntaAlVolver<Permisos>("permisos_de_macos", PERMISOS_DE_MUESTRA);
}

/**
 * Los bytes que salieron del equipo, **leídos**. Una constante escrita en el webview no es un
 * contador: sería la interfaz afirmando el cero en vez de medirlo.
 */
export function useBytesALaRed(): string {
  const [bytes, setBytes] = useState("0 B");
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const leer = () => {
      void preguntar<string>("bytes_a_la_red").then((b) => {
        if (vivo && b !== null) setBytes(b);
      });
    };
    leer();
    const baja = escuchar("corte", leer);
    return () => {
      vivo = false;
      baja();
    };
  }, []);
  return bytes;
}

// ---------------------------------------------------------------------------------------------
// Fase 3 — las dos pistas, la transcripción y el idioma
// ---------------------------------------------------------------------------------------------

/** Por dónde sale el sonido del Mac, y por tanto si el micrófono va a oír al cliente. */
export type Salida =
  | { salida: "altavoces" }
  | { salida: "auriculares" }
  | { salida: "otra"; nombre: string }
  | { salida: "no-se-sabe"; motivo: string };

/** En qué estado está el motor de transcripción para un idioma. */
export type Disponibilidad =
  | { estado: "listo" }
  | { estado: "sin-modelo" }
  | { estado: "idioma-desconocido" }
  | { estado: "sin-motor"; motivo: string };

export type EstadoDePista = {
  /** **Esto, y no el número de muestras, distingue una avería de un silencio.** Cuando nadie
   *  habla, macOS no entrega ni una muestra: cero no es un fallo. */
  abierta: boolean;
  motivo: string | null;
  bytes: number;
  legible: string;
  segundos: number;
  muestrasRecibidas: number;
  hablando: boolean;
};

export type EstadoDeEscucha = {
  escuchando: boolean;
  microfono: EstadoDePista;
  sistema: EstadoDePista;
  turnosEnMemoria: number;
  bytesDelTranscript: number;
  ramLegible: string;
  motor: string;
};

export type Turno = {
  pista: "microfono" | "sistema";
  desdeMs: number;
  hastaMs: number;
  texto: string;
  /** «14:02» — la hora del reloj, calculada en la parte nativa y viva solo en memoria. */
  hora: string;
  /** El micrófono captó por los altavoces lo que decía el cliente. No se borra: se marca. */
  eco: boolean;
};

export type QueSabeTranscribir = {
  motor: string;
  techo: number;
  idiomas: { codigo: string; disponibilidad: Disponibilidad }[];
  motivo: string | null;
};

/**
 * Lo que la maqueta dibuja en el estado «así se ve hoy · sprint 1»: las dos pistas abiertas con
 * sus treinta segundos dentro. Vive aquí por la misma razón que la muestra de la banda — es lo
 * que hace posible comparar el producto contra la maqueta en un navegador — y muere igual.
 */
const ESCUCHA_DE_MUESTRA: EstadoDeEscucha = {
  escuchando: true,
  microfono: { abierta: true, motivo: null, bytes: 1_920_000, legible: "1,8 MB", segundos: 30, muestrasRecibidas: 480_000, hablando: false },
  sistema: { abierta: true, motivo: null, bytes: 1_920_000, legible: "1,8 MB", segundos: 30, muestrasRecibidas: 480_000, hablando: false },
  turnosEnMemoria: 12,
  bytesDelTranscript: 2_048,
  ramLegible: "3,7 MB",
  motor: "apple-speechanalyzer",
};

const SALIDA_DE_MUESTRA: Salida = { salida: "altavoces" };

const TRANSCRIPCION_DE_MUESTRA: QueSabeTranscribir = {
  motor: "apple-speechanalyzer",
  techo: 5,
  idiomas: [
    { codigo: "es-ES", disponibilidad: { estado: "listo" } },
    { codigo: "en-US", disponibilidad: { estado: "listo" } },
  ],
  motivo: null,
};

/**
 * Lo que vive en memoria ahora mismo. Se refresca con cada novedad de la escucha y con el corte,
 * **no con un temporizador**: los bytes solo cambian cuando entra audio o cuando se corta, y un
 * sondeo cada segundo gastaría un candado compartido con el hilo que mira los marcos.
 */
export function useEscucha(): EstadoDeEscucha {
  const [estado, setEstado] = useState<EstadoDeEscucha>(ESCUCHA_DE_MUESTRA);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const leer = () => {
      void preguntar<EstadoDeEscucha | null>("estado_de_la_escucha").then((e) => {
        if (vivo) setEstado(e ?? APAGADA);
      });
    };
    leer();
    const bajas = [escuchar("escucha", leer), escuchar("corte", leer)];
    globalThis.addEventListener("focus", leer);
    return () => {
      vivo = false;
      bajas.forEach((b) => b());
      globalThis.removeEventListener("focus", leer);
    };
  }, []);
  return estado;
}

/** Nadie está escuchando: ni pistas abiertas ni bytes. No es un error, es el estado de reposo. */
const APAGADA: EstadoDeEscucha = {
  escuchando: false,
  microfono: { abierta: false, motivo: null, bytes: 0, legible: "0 B", segundos: 0, muestrasRecibidas: 0, hablando: false },
  sistema: { abierta: false, motivo: null, bytes: 0, legible: "0 B", segundos: 0, muestrasRecibidas: 0, hablando: false },
  turnosEnMemoria: 0,
  bytesDelTranscript: 0,
  ramLegible: "0 B",
  motor: "—",
};

export function useSalidaDeAudio(): Salida {
  return usePreguntaAlVolver<Salida>("salida_de_audio", SALIDA_DE_MUESTRA);
}

export function useQueSabeTranscribir(): QueSabeTranscribir {
  return usePreguntaAlVolver<QueSabeTranscribir>("que_sabe_transcribir", TRANSCRIPCION_DE_MUESTRA);
}

export function empezarAEscuchar(idiomaDelConsultor: string, idiomaDelCliente: string) {
  void llamar("empezar_a_escuchar", { idiomaDelConsultor, idiomaDelCliente });
}

export function dejarDeEscuchar() {
  void llamar("dejar_de_escuchar");
}

export async function instalarIdioma(codigo: string): Promise<Disponibilidad | null> {
  return preguntar<Disponibilidad>("instalar_idioma", { codigo });
}

export function abrirAjustesDe(permiso: "microfono" | "pantalla" | "accesibilidad") {
  void llamar("abrir_ajustes_de", { permiso });
}

export function cortarTodo() {
  void llamar("cortar_todo");
}
