import { useCallback, useState } from "react";
import { Ventana, type Seccion } from "./Ventana";
import { Sesion } from "../pantallas/Sesion";
import { Permisos } from "../pantallas/Permisos";
import { Honestidad } from "../pantallas/Honestidad";
import { Corpus } from "../pantallas/Corpus";
import { Idioma } from "../pantallas/Idioma";
import { Ia } from "../pantallas/Ia";
import { Notas } from "../pantallas/Notas";
import { Ensayo } from "../pantallas/Ensayo";
import { estadoDeLaUrl, useEnsayo } from "../ensayo";
import {
  duracion,
  useBandeja,
  useCuaderno,
  useMuestraDeLaBandeja,
  useMuestraDelCuaderno,
  useQuedan,
  type VistaDeNotas,
} from "../notas";
import { useT } from "../i18n";
import { escuchaDeMuestra, estadoDeSesion } from "../jurisdiccion";
import { hayTauri } from "../puente";
import {
  useBytesALaRed,
  useEscucha,
  usePermisos,
  useQueSabeTranscribir,
  useReunion,
  useSalidaDeAudio,
} from "../cuaderno";

/**
 * LA VENTANA PRINCIPAL (960 × 640) — el cuaderno.
 *
 * Las ocho pantallas del rail desde el sprint 004: sesión, ensayo, permisos, corpus, notas,
 * honestidad, idioma e IA. Ensayo fue la última en encenderse (fase 3, C18).
 *
 * La sección inicial se lee de la URL para que el arnés de capturas pueda recorrer las tres sin
 * hacer clic — el mismo mecanismo que usa la banda, y muere igual cuando haya navegación de
 * verdad que recordar.
 */
const SECCIONES: Seccion[] = ["sesion", "ensayo", "permisos", "corpus", "notas", "honestidad", "idioma", "ia"];

/** Los estados de `ensayo.html` con el micrófono abierto: el chip del rail dice «Ensayando». */
const ESTADOS_ENSAYANDO = ["preguntando", "del-modelo", "respondiendo", "evaluada"];

function seccionDeLaUrl(busqueda: string): Seccion {
  const pedida = new URLSearchParams(busqueda).get("pantalla");
  return SECCIONES.find((s) => s === pedida) ?? "sesion";
}

export function Principal({ busqueda = globalThis.location?.search ?? "" }: { busqueda?: string }) {
  const [seccion, setSeccion] = useState<Seccion>(() => seccionDeLaUrl(busqueda));
  const reunion = useReunion();
  const permisos = usePermisos();
  const bytes = useBytesALaRed();
  const escucha = useEscucha();
  const salida = useSalidaDeAudio();
  const transcribe = useQueSabeTranscribir();
  // El chip del rail dice «Cerrando…» mientras una reunión parada espera que la guardes o la
  // descartes. Fuera de Tauri, cuando la URL pide Notas «al cerrar».
  const estadoPedido = new URLSearchParams(busqueda).get("estado");
  const [cuaderno] = useCuaderno(
    useMuestraDelCuaderno(estadoPedido?.startsWith("al-cerrar") ? "al-cerrar" : "archivo"),
  );
  const cerrando = Boolean(cuaderno?.abierta && !cuaderno.escuchando);
  // El chip de la bandeja (ADR 016): la que espera, con su cuenta; o «vencida» mientras Notas enseña
  // la que acaba de vencer. Fuera de Tauri, la de la URL.
  const tn = useT().notas;
  const [bandeja] = useBandeja(
    useMuestraDeLaBandeja(estadoPedido === "bandeja" || estadoPedido === "bandeja-llave" || estadoPedido === "vencida" ? estadoPedido : "archivo"),
  );
  const quedan = useQuedan(bandeja?.vence ?? null);
  const [vistaDeNotas, setVistaDeNotas] = useState<VistaDeNotas | null>(null);
  const alCambiarDeVista = useCallback((v: VistaDeNotas | null) => setVistaDeNotas(v), []);
  const vencidaEnPantalla = seccion === "notas" && vistaDeNotas === "vencida";
  // Fuera de Tauri la escucha de muestra «escucha» siempre (la maqueta del sprint 1): no cuenta aquí.
  const chipDeLaBandeja = hayTauri() && escucha.escuchando
    ? null
    : vencidaEnPantalla
      ? { texto: tn.vencidaChip, vencida: true }
      : quedan !== null && quedan > 0
        ? { texto: `${tn.bandejaChip} ${duracion(quedan, tn)[1]}`, vencida: false }
        : null;

  // La puerta local abierta y la puerta cerrada a mano no conviven con una reunión (ADR 018 §5): fuera de
  // Tauri, esos dos estados de `ia.html` enseñan el rail sin sesión; «se cerró sola», con la reunión.
  const q = new URLSearchParams(busqueda);
  // Y el ensayo no convive con una reunión (ADR 019 §6.5): fuera de Tauri, sus estados van sin sesión.
  const sinReunionDeMuestra =
    !hayTauri() &&
    ((q.get("vista") === "puerta" && q.get("puerta") !== "en-reunion") || q.get("pantalla") === "ensayo");
  const enSesion = reunion.que === "detectada" && !sinReunionDeMuestra;
  // El ensayo (sprint 004): el chip dice «Ensayando» mientras su micrófono está abierto. Fuera de Tauri,
  // en los estados de la maqueta que ensayan.
  const [ensayo] = useEnsayo();
  const ensayando = hayTauri()
    ? ensayo !== null && ensayo.fase !== "cerrado"
    : q.get("pantalla") === "ensayo" && ESTADOS_ENSAYANDO.includes(estadoDeLaUrl(busqueda));

  return (
    <Ventana
      seccion={seccion}
      ir={setSeccion}
      enSesion={enSesion}
      cerrando={cerrando}
      bandeja={chipDeLaBandeja}
      ensayando={ensayando}
      cliente={reunion.que === "detectada" ? reunion.cliente : undefined}
    >
      {seccion === "sesion" && (
        <Sesion
          reunion={reunion}
          // Fuera de Tauri, la escucha del estado de `sesion.html` que pide la URL: antes de empezar,
          // en marcha o en solo notas (ADR 017). Dentro, la de verdad.
          escucha={hayTauri() ? escucha : escuchaDeMuestra(estadoDeSesion(busqueda), escucha)}
          salida={salida}
          // Fuera de Tauri, el estado «software invasivo en tu Mac» de la maqueta: lo pide el arnés
          // del gate de fidelidad por la URL. Dentro del producto lo decide el radar.
          radarDeMuestra={new URLSearchParams(busqueda).get("radar") === "vigilancia"}
          busqueda={busqueda}
        />
      )}
      {seccion === "ensayo" && <Ensayo busqueda={busqueda} ir={setSeccion} />}
      {seccion === "permisos" && <Permisos permisos={permisos} />}
      {seccion === "honestidad" && <Honestidad bytes={bytes} escucha={escucha} busqueda={busqueda} />}
      {seccion === "corpus" && <Corpus />}
      {seccion === "idioma" && <Idioma transcribe={transcribe} />}
      {seccion === "ia" && <Ia busqueda={busqueda} />}
      {seccion === "notas" && <Notas busqueda={busqueda} alCambiarDeVista={alCambiarDeVista} />}
    </Ventana>
  );
}
