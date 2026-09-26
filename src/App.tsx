import { useEffect, useState, type ReactNode } from "react";
import { IdiomaContext, type Idioma } from "./i18n";
import { SpriteIconos } from "./componentes/Iconos";
import { Banda, type EstadoBanda } from "./componentes/Banda";
import { Relleno } from "./componentes/Relleno";
import { Principal } from "./componentes/Principal";
import { ventanaActual } from "./ventanas";
import { useAltoDeVentana, DESDE_AMPLIADA } from "./asa";
import { useAcoplada } from "./acople";
import { useTranscriptVisible } from "./turnos";

/**
 * Cáscara de la app.
 *
 * Tema e idioma viven en el elemento `<html>`, exactamente como en la maqueta: `data-theme`
 * y `lang`. Un solo lugar de verdad del que cuelgan el CSS (los tokens se redefinen por
 * `html[data-theme]`) y el diccionario. En la maqueta el conmutador era un botón de la sala
 * de diseño; en producto lo moverán las preferencias del usuario y el sistema.
 */

/** Lee el tema del sistema la primera vez; el design system manda oscuro como primario. */
function temaInicial(): "dark" | "light" {
  const declarado = document.documentElement.dataset.theme;
  if (declarado === "light" || declarado === "dark") return declarado;
  return window.matchMedia?.("(prefers-color-scheme: light)").matches ? "light" : "dark";
}

function idiomaInicial(): Idioma {
  const declarado = document.documentElement.lang;
  if (declarado.startsWith("en")) return "en";
  if (declarado.startsWith("es")) return "es";
  return navigator.language?.startsWith("en") ? "en" : "es";
}

export function Cascara({ children }: { children?: ReactNode }) {
  const [tema] = useState(temaInicial);
  const [idioma] = useState(idiomaInicial);

  useEffect(() => {
    document.documentElement.dataset.theme = tema;
  }, [tema]);

  useEffect(() => {
    document.documentElement.lang = idioma;
  }, [idioma]);

  return <IdiomaContext.Provider value={idioma}>{children}</IdiomaContext.Provider>;
}

const ESTADOS: EstadoBanda[] = [
  "esperando",
  "buscando",
  "ficha",
  "sin-resultado",
  "sin-verificar",
  // Los tres del modo solo audio (C15, sprint 002). Dentro de Tauri **no mandan**: el modo lo
  // decide `⌃⌥V`, que cambia el alto de la VENTANA, y la banda obedece a `estado_de_la_voz`. Estos
  // tres existen para que el gate de FIDELIDAD pueda fotografiar los encuadres de 44 px fuera de
  // Tauri, que es donde el arnés corre — igual que los cinco de arriba desde la fase 1.
  "voz",
  "voz-espera",
  "voz-sin",
];

/**
 * Qué estado muestra la banda mientras no hay ni audio ni corpus.
 *
 * Hasta la **fase 3** (dos pistas + fin de turno) y la **fase 4** (corpus y disparo) no existe
 * nada que decida el estado de verdad, así que lo elige la URL. No es una puerta trasera: es lo
 * que hace posible el **gate de FIDELIDAD** —recorrer los nueve encuadres de `banda.html` en la
 * ventana real, en los dos temas y los dos idiomas— y muere en cuanto el disparo sea real.
 */
function bandaDesdeLaUrl(busqueda: string, alto: number) {
  const p = new URLSearchParams(busqueda);
  const pedido = p.get("estado");
  const estado = ESTADOS.find((e) => e === pedido) ?? "esperando";
  return {
    estado,
    // `ampliada` NO se pide: se deduce del alto de la VENTANA, que es quien manda. Así la banda
    // no puede dibujarse ampliada dentro de un marco de 88 px (ni al revés) y el asa funciona
    // sin avisar a nadie: cambia la ventana, y la banda se entera midiendo.
    ampliada: alto >= DESDE_AMPLIADA,
    verificado: p.get("verificado") !== "0",
  };
}

/**
 * `acoplada` tampoco se pide: se PREGUNTA a la parte nativa, porque depende de si el usuario
 * concedió Accesibilidad y de si la ventana de la reunión se dejó recortar. El parámetro de URL
 * sigue existiendo —sin él, el gate de fidelidad no podría recorrer el encuadre «sin acople» en
 * un navegador— pero solo manda cuando está escrito.
 */
function acopleDesdeLaUrl(busqueda: string): boolean | undefined {
  const pedido = new URLSearchParams(busqueda).get("acoplada");
  return pedido === null ? undefined : pedido !== "0";
}

export function Enrutador({ busqueda = globalThis.location?.search ?? "" }: { busqueda?: string }) {
  const ventana = ventanaActual(busqueda);
  const alto = useAltoDeVentana();
  const acoplada = useAcoplada(acopleDesdeLaUrl(busqueda));
  // `⌃⌥T` conmuta el transcript desde la parte nativa. El parámetro de URL sigue existiendo para
  // que el arnés de capturas pueda fotografiar el encuadre abierto sin pulsar una tecla global.
  const transcript = useTranscriptVisible(
    new URLSearchParams(busqueda).get("transcript") === "1",
  );

  // La identidad de la ventana vive en `<html>`, al lado del tema y del idioma: un solo lugar de
  // verdad del que cuelga el CSS de ventana — y, de paso, lo que un e2e puede leer sin adivinar.
  useEffect(() => {
    document.documentElement.dataset.ventana = ventana;
  }, [ventana]);

  switch (ventana) {
    case "relleno":
      // Sin sprite y sin nada: el relleno no dibuja contenido, por definición.
      return <Relleno />;
    case "banda":
      return (
        <>
          <SpriteIconos />
          <Banda {...bandaDesdeLaUrl(busqueda, alto)} transcript={transcript} acoplada={acoplada} />
        </>
      );
    default:
      return (
        <>
          <SpriteIconos />
          <Principal busqueda={busqueda} />
        </>
      );
  }
}

export default function App() {
  return (
    <Cascara>
      <Enrutador />
    </Cascara>
  );
}
