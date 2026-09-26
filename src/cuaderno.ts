import { useEffect, useState } from "react";
import { escuchar, hayTauri, llamar, preguntar } from "./puente";
import { useT } from "./i18n";
import { INFORME_DEL_CORTE } from "./contrato.generado";

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
  | {
      que: "detectada";
      cliente: string;
      titulo: string | null;
      proteccion: Proteccion;
    }
  | { que: "no-se-puede-saber"; motivo: string };

export type EstadoPermiso =
  "sin-conceder" | "concedido" | "denegado" | "no-se-sabe";

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
  return {
    que: "detectada",
    cliente: "Google Meet",
    titulo,
    proteccion: "Verificada",
  };
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
/**
 * @param deMuestra lo que se enseña **fuera de Tauri**: los datos que dibuja la maqueta, y lo que
 * hace posible el gate de FIDELIDAD.
 * @param vacio lo que se enseña **dentro del producto mientras lo nativo no ha contestado**. Es un
 * parámetro aparte y no el mismo valor por una razón que costó un hallazgo de auditoría: con la
 * muestra como estado inicial, el producto pintaba «143 documentos · 5 unidades» y «Meet ·
 * protegido» durante el primer instante de cada arranque —y para siempre si el comando fallaba—.
 * Datos de una consultora inventada dentro de la app de alguien. La familia del hallazgo A1.
 */
function usePreguntaAlVolver<T>(comando: string, deMuestra: T, vacio: T): T {
  const [valor, setValor] = useState<T>(() => (hayTauri() ? vacio : deMuestra));
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
  return usePreguntaAlVolver<Reunion>(
    "reunion_abierta",
    reunionDeMuestra(t.tituloDeMuestra),
    {
      que: "ninguna",
    },
  );
}

/**
 * Las piezas del kill-switch, **leídas** de lo nativo.
 *
 * Antes de que conteste no se enseña una cuenta inventada: se enseña la lista vacía, que la
 * pantalla sabe leer como «todavía no lo sé». La muestra para la maqueta es la del contrato, que
 * es la forma que Rust emite de verdad.
 */
export function usePiezasDelCorte(): InformeDelCorte {
  return usePreguntaAlVolver<InformeDelCorte>(
    "piezas_del_corte",
    INFORME_DEL_CORTE,
    {
      piezas: [],
      bytesEnRed: 0,
    },
  );
}

/** Lo que se sabe de los permisos antes de preguntar: nada. Y «no lo sé» **no es «no»**. */
const PERMISOS_SIN_PREGUNTAR: Permisos = {
  microfono: "no-se-sabe",
  pantalla: "no-se-sabe",
  accesibilidad: "no-se-sabe",
};

export function usePermisos(): Permisos {
  return usePreguntaAlVolver<Permisos>(
    "permisos_de_macos",
    PERMISOS_DE_MUESTRA,
    PERMISOS_SIN_PREGUNTAR,
  );
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
  motor: string;
};

/**
 * Quién habló, **por el origen de la muestra**: el micrófono es el consultor y el audio del
 * sistema es la contraparte. Regla dura de la casa — la atribución se resuelve por pista y jamás
 * por biometría.
 */
/**
 * El kill-switch, visto desde la interfaz.
 *
 * Las piezas y su suerte las decide `src-tauri/src/corte.rs` con un `match` sin comodín: quien
 * añada una pieza y no la resuelva no compila. Aquí solo se leen — hasta el sprint 002 esta cuenta
 * estaba escrita a mano en la pantalla de Honestidad, con un comentario que lo confesaba.
 */
export type PiezaDelCorte =
  /** La voz que sale (C15, sprint 002). Es la primera que se corta: la única que se OYE. */
  | "voz"
  | "audio-del-microfono"
  | "audio-del-sistema"
  | "ultimo-frame"
  | "transcript"
  | "contador-de-red"
  | "banda"
  | "acople";

/** `cortada` = existe y se corta · `aun-no-existe` = todavía no está construida, y se dice. */
export type SuerteDelCorte = "cortada" | "aun-no-existe";

export type InformeDelCorte = {
  piezas: [PiezaDelCorte, SuerteDelCorte][];
  bytesEnRed: number;
};

export type Pista = "microfono" | "sistema";

export type Turno = {
  pista: Pista;
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
  microfono: {
    abierta: true,
    motivo: null,
    bytes: 1_920_000,
    segundos: 30,
    muestrasRecibidas: 480_000,
    hablando: false,
  },
  sistema: {
    abierta: true,
    motivo: null,
    bytes: 1_920_000,
    segundos: 30,
    muestrasRecibidas: 480_000,
    hablando: false,
  },
  turnosEnMemoria: 12,
  bytesDelTranscript: 2_048,
  motor: "apple-speechanalyzer",
};

const SALIDA_DE_MUESTRA: Salida = { salida: "altavoces" };

/**
 * La muestra de la maqueta: el modelo del consultor instalado y **el del cliente sin instalar**.
 *
 * No es un capricho ni pesimismo: es el estado más probable en un Mac real —uno en español no trae
 * el modelo de inglés— y es el que la maqueta dibuja desde la fase 2 de la auditoría. Mientras la
 * muestra tenía los dos modelos listos, el gate de FIDELIDAD fotografiaba sesenta encuadres sin
 * pasar nunca por el estado que la app iba a enseñarle a casi todo el mundo: el que no tenía cómo
 * instalar nada (hallazgos A6 y A7).
 */
const TRANSCRIPCION_DE_MUESTRA: QueSabeTranscribir = {
  motor: "apple-speechanalyzer",
  techo: 5,
  idiomas: [
    { codigo: "es-ES", disponibilidad: { estado: "listo" } },
    { codigo: "en-US", disponibilidad: { estado: "sin-modelo" } },
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
      void preguntar<EstadoDeEscucha | null>("estado_de_la_escucha").then(
        (e) => {
          if (vivo) setEstado(e ?? APAGADA);
        },
      );
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
  microfono: {
    abierta: false,
    motivo: null,
    bytes: 0,
    segundos: 0,
    muestrasRecibidas: 0,
    hablando: false,
  },
  sistema: {
    abierta: false,
    motivo: null,
    bytes: 0,
    segundos: 0,
    muestrasRecibidas: 0,
    hablando: false,
  },
  turnosEnMemoria: 0,
  bytesDelTranscript: 0,
  motor: "—",
};

/**
 * **EL MODO SOLO AUDIO (C15), visto desde la banda.**
 *
 * Tres booleanos y ninguna frase, y eso es deliberado en el lado de Rust: las frases son copy y el
 * copy vive en `src/i18n/`, donde el gate del diccionario las compara una a una con
 * `docs/diseno/banda.html`. Si la parte nativa mandara «Conecta auriculares», ese texto se podría
 * cambiar sin que ninguna mirada lo viera.
 *
 * | Campo | Qué decide |
 * |---|---|
 * | `encendida` | la banda vive a 44 px en vez de 88 |
 * | `puede` | cuál de los dos estados se pinta: «Diciéndote la ficha…» o «Conecta auriculares» |
 * | `diciendo` | si está sonando ahora mismo |
 */
export type LaVoz = {
  encendida: boolean;
  puede: boolean;
  diciendo: boolean;
};

/** La voz apagada, que es como nace la app — y como la pinta el arnés de capturas. */
export const VOZ_APAGADA: LaVoz = {
  encendida: false,
  puede: false,
  diciendo: false,
};

/**
 * Cómo está la voz. Se pregunta al montarse y se escucha a partir de ahí: el evento llega cuando
 * el usuario pulsa `⌘⇧V` o `⎋`, cuando empieza o acaba una ficha, y cuando el kill-switch la calla.
 *
 * **No se sondea.** El estado cambia unas cuantas veces por reunión y preguntarlo cada dos segundos
 * sería despertar Core Audio —`salida_de_audio` lee el dispositivo por defecto— para casi nunca
 * enterarse de nada.
 */
export function useVoz(): LaVoz {
  const [voz, setVoz] = useState<LaVoz>(VOZ_APAGADA);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const leer = () => {
      void preguntar<LaVoz>("estado_de_la_voz").then((v) => {
        if (vivo) setVoz(v ?? VOZ_APAGADA);
      });
    };
    leer();
    const baja = escuchar<LaVoz>("voz", (v) => {
      if (vivo) setVoz(v ?? VOZ_APAGADA);
    });
    return () => {
      vivo = false;
      baja();
    };
  }, []);
  return voz;
}

/**
 * La LECTURA DE PANTALLA (C8, sprint 002), tal y como la cuenta Rust (`pantalla::EstadoDeLaPantalla`).
 *
 * | Campo | Quién lo lee |
 * |---|---|
 * | `vista` | la fila «Pantalla — solo cuando cambia» de Sesión |
 * | `bytesEnMemoria` | la fila de la pantalla en «Qué vive en la memoria ahora» de Honestidad |
 */
export type VistaDeLaPantalla =
  "apagada" | "sin-permiso" | "esperando-la-reunion" | "leyendo" | "no-pudo";

export type EstadoDeLaPantalla = {
  vista: VistaDeLaPantalla;
  bytesEnMemoria: number;
};

/** Sin sesión, la lectura espera a la reunión: es como nace la app y como la pinta el arnés. */
export const PANTALLA_EN_ESPERA: EstadoDeLaPantalla = {
  vista: "esperando-la-reunion",
  bytesEnMemoria: 0,
};

/**
 * Qué hace la lectura de pantalla. Se pregunta al montarse y se escucha el evento `pantalla`, que
 * llega cuando cambia la vista (se enciende, se apaga, aparece o desaparece la reunión). Tras el
 * kill-switch se vuelve a preguntar: el corte la para sin emitir nada, y la fila no puede quedarse
 * diciendo «leyendo» de una lectura que ya no existe.
 */
export function usePantalla(): EstadoDeLaPantalla {
  const [estado, setEstado] = useState<EstadoDeLaPantalla>(PANTALLA_EN_ESPERA);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const leer = () => {
      void preguntar<EstadoDeLaPantalla>("estado_de_la_pantalla").then((e) => {
        if (vivo) setEstado(e ?? PANTALLA_EN_ESPERA);
      });
    };
    leer();
    const baja = escuchar<EstadoDeLaPantalla>("pantalla", (e) => {
      if (vivo) setEstado(e ?? PANTALLA_EN_ESPERA);
    });
    const bajaCorte = escuchar<unknown>("corte", leer);
    return () => {
      vivo = false;
      baja();
      bajaCorte();
    };
  }, []);
  return estado;
}

/** El interruptor de Sesión: enciende o apaga la lectura automática. */
export function lecturaAutomatica(
  encendida: boolean,
): Promise<EstadoDeLaPantalla | null> {
  return preguntar<EstadoDeLaPantalla>("lectura_automatica", { encendida });
}

/** Lee la ventana de la reunión UNA vez, ahora — la lectura bajo demanda, sin región. */
export function leerLaPantallaAhora(): Promise<boolean | null> {
  return preguntar<boolean>("leer_la_pantalla_ahora");
}

export function useSalidaDeAudio(): Salida {
  // El motivo va **vacío** a propósito: nadie lo pinta hoy, y escribir aquí una frase en español
  // sería meter copy de una sola lengua en el camino de una pantalla — el hallazgo A6 otra vez. Si
  // algún día se enseña, saldrá del diccionario. Lo vigila el barrido de copy cableado.
  return usePreguntaAlVolver<Salida>("salida_de_audio", SALIDA_DE_MUESTRA, {
    salida: "no-se-sabe",
    motivo: "",
  });
}

export function useQueSabeTranscribir(): QueSabeTranscribir {
  return usePreguntaAlVolver<QueSabeTranscribir>(
    "que_sabe_transcribir",
    TRANSCRIPCION_DE_MUESTRA,
    { motor: "—", techo: 0, idiomas: [], motivo: null },
  );
}

/**
 * Qué idioma escucha cada pista en este sprint.
 *
 * Fijos, y por eso viven aquí y no en la pantalla de Idioma: los usa quien **enciende** la escucha
 * (la pantalla de Sesión) y quien los **enseña** (la de Idioma), y una constante que dos pantallas
 * copian por su cuenta es una constante que un día dirá dos cosas distintas. Elegirlos es de la
 * fase siguiente; hasta entonces la pantalla de Idioma los marca como lo que son.
 */
export const DEL_CONSULTOR = "es-ES";
export const DEL_CLIENTE = "en-US";

/* ----------------------------------------------------------------- el corpus (fase 4) ------ */

export type UnidadDelCorpus =
  "propuesta" | "marco" | "caso" | "cliente" | "perfil";

export type PorUnidad = { unidad: UnidadDelCorpus; documentos: number };

export type EstadoDelCorpus = {
  /** `null` mientras el usuario no haya señalado carpeta: el estado «vacío» de la maqueta. */
  carpeta: string | null;
  documentos: number;
  secciones: number;
  porUnidad: PorUnidad[];
  sinUnidad: number;
  ilegibles: number;
  /** Documentos cuyas secciones se conjeturaron por la forma del texto (PDF). */
  conjeturados: number;
  /** Dónde acabó el derivado. Quien confía su carpeta tiene derecho a saberlo. */
  dondeVive: string | null;
  bytesDelIndice: number;
};

/**
 * El corpus de muestra que se enseña **fuera de Tauri** — los mismos números que dibuja
 * `corpus.html`, que es lo que hace comparable el gate de FIDELIDAD. Datos 100 % sintéticos.
 */
export const CORPUS_DE_MUESTRA: EstadoDelCorpus = {
  carpeta: "~/Documentos/Consultoría",
  documentos: 143,
  secciones: 1_284,
  porUnidad: [
    { unidad: "propuesta", documentos: 31 },
    { unidad: "marco", documentos: 12 },
    { unidad: "caso", documentos: 28 },
    { unidad: "cliente", documentos: 63 },
    { unidad: "perfil", documentos: 9 },
  ],
  sinUnidad: 0,
  ilegibles: 4,
  conjeturados: 9,
  dondeVive: "~/Library/…/Angel Ghost/corpus",
  bytesDelIndice: 19_293_798,
};

/** Sin carpeta señalada —el primer arranque de la app— no hay nada, y eso es lo que se enseña. */
export const CORPUS_VACIO: EstadoDelCorpus = {
  carpeta: null,
  documentos: 0,
  secciones: 0,
  porUnidad: [],
  sinUnidad: 0,
  ilegibles: 0,
  conjeturados: 0,
  dondeVive: null,
  bytesDelIndice: 0,
};

export function useCorpus(): EstadoDelCorpus {
  return usePreguntaAlVolver<EstadoDelCorpus>(
    "estado_del_corpus",
    CORPUS_DE_MUESTRA,
    CORPUS_VACIO,
  );
}

/**
 * Señalar una carpeta e indexarla. Dos llamadas y no una: elegir es instantáneo, indexar ciento
 * cuarenta documentos no, y juntarlas dejaría la ventana congelada desde el clic hasta el final.
 */
export async function indexarCorpus(): Promise<void> {
  const carpeta = await preguntar<string | null>("elegir_carpeta");
  if (!carpeta) return; // el usuario canceló: no es un fallo y no se dice nada
  await preguntar("indexar_corpus", { carpeta });
}

export function empezarAEscuchar(
  idiomaDelConsultor: string,
  idiomaDelCliente: string,
) {
  void llamar("empezar_a_escuchar", { idiomaDelConsultor, idiomaDelCliente });
}

export function dejarDeEscuchar() {
  void llamar("dejar_de_escuchar");
}

export async function instalarIdioma(
  codigo: string,
): Promise<Disponibilidad | null> {
  return preguntar<Disponibilidad>("instalar_idioma", { codigo });
}

export function abrirAjustesDe(
  permiso: "microfono" | "pantalla" | "accesibilidad",
) {
  void llamar("abrir_ajustes_de", { permiso });
}

export function cortarTodo() {
  void llamar("cortar_todo");
}
