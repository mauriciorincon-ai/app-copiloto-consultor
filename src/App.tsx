import { useEffect, useState, type ReactNode } from "react";
import { IdiomaContext, type Idioma } from "./i18n";

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

export default function App() {
  // La fase 1 monta aquí la banda; la fase 2, la ventana principal.
  return <Cascara />;
}
