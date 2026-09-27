import { useState } from "react";
import { Ventana, type Seccion } from "./Ventana";
import { Sesion } from "../pantallas/Sesion";
import { Permisos } from "../pantallas/Permisos";
import { Honestidad } from "../pantallas/Honestidad";
import { Corpus } from "../pantallas/Corpus";
import { Idioma } from "../pantallas/Idioma";
import { Ia } from "../pantallas/Ia";
import { Notas } from "../pantallas/Notas";
import { useCuaderno, useMuestraDelCuaderno } from "../notas";
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
 * Las siete pantallas del rail desde el sprint 003: sesión, permisos, corpus, notas, honestidad,
 * idioma e IA. Notas fue la última en encenderse (fase 1, C9).
 *
 * La sección inicial se lee de la URL para que el arnés de capturas pueda recorrer las tres sin
 * hacer clic — el mismo mecanismo que usa la banda, y muere igual cuando haya navegación de
 * verdad que recordar.
 */
const SECCIONES: Seccion[] = ["sesion", "permisos", "corpus", "notas", "honestidad", "idioma", "ia"];

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
  const [cuaderno] = useCuaderno(useMuestraDelCuaderno(estadoPedido === "al-cerrar" ? "al-cerrar" : "archivo"));
  const cerrando = Boolean(cuaderno?.abierta && !cuaderno.escuchando);

  return (
    <Ventana
      seccion={seccion}
      ir={setSeccion}
      enSesion={reunion.que === "detectada"}
      cerrando={cerrando}
      cliente={reunion.que === "detectada" ? reunion.cliente : undefined}
    >
      {seccion === "sesion" && (
        <Sesion
          reunion={reunion}
          escucha={escucha}
          salida={salida}
          // Fuera de Tauri, el estado «software invasivo en tu Mac» de la maqueta: lo pide el arnés
          // del gate de fidelidad por la URL. Dentro del producto lo decide el radar.
          radarDeMuestra={new URLSearchParams(busqueda).get("radar") === "vigilancia"}
        />
      )}
      {seccion === "permisos" && <Permisos permisos={permisos} />}
      {seccion === "honestidad" && <Honestidad bytes={bytes} escucha={escucha} />}
      {seccion === "corpus" && <Corpus />}
      {seccion === "idioma" && <Idioma transcribe={transcribe} />}
      {seccion === "ia" && <Ia busqueda={busqueda} />}
      {seccion === "notas" && <Notas busqueda={busqueda} />}
    </Ventana>
  );
}
