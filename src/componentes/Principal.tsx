import { useState } from "react";
import { Ventana, type Seccion } from "./Ventana";
import { Sesion } from "../pantallas/Sesion";
import { Permisos } from "../pantallas/Permisos";
import { Honestidad } from "../pantallas/Honestidad";
import { useBytesALaRed, usePermisos, useReunion } from "../cuaderno";

/**
 * LA VENTANA PRINCIPAL (960 × 640) — el cuaderno.
 *
 * Tres pantallas en el sprint 001: sesión, permisos y honestidad. Corpus, notas, idioma e IA
 * siguen en el rail, apagadas: quitarlas escondería que la app va a tenerlas, y ponerlas
 * navegables prometería una pantalla que no está.
 *
 * La sección inicial se lee de la URL para que el arnés de capturas pueda recorrer las tres sin
 * hacer clic — el mismo mecanismo que usa la banda, y muere igual cuando haya navegación de
 * verdad que recordar.
 */
const SECCIONES: Seccion[] = ["sesion", "permisos", "honestidad"];

function seccionDeLaUrl(busqueda: string): Seccion {
  const pedida = new URLSearchParams(busqueda).get("pantalla");
  return SECCIONES.find((s) => s === pedida) ?? "sesion";
}

export function Principal({ busqueda = globalThis.location?.search ?? "" }: { busqueda?: string }) {
  const [seccion, setSeccion] = useState<Seccion>(() => seccionDeLaUrl(busqueda));
  const reunion = useReunion();
  const permisos = usePermisos();
  const bytes = useBytesALaRed();

  return (
    <Ventana seccion={seccion} ir={setSeccion} enSesion={reunion.que === "detectada"}>
      {seccion === "sesion" && <Sesion reunion={reunion} />}
      {seccion === "permisos" && <Permisos permisos={permisos} />}
      {seccion === "honestidad" && <Honestidad bytes={bytes} />}
    </Ventana>
  );
}
