import { useEffect, useRef, useState } from "react";
import { llamar } from "./puente";

/**
 * EL ASA de la banda.
 *
 * Arrastrarla cambia el alto de la banda **y el de su relleno** —y, cuando exista el acople,
 * también el de la ventana de la reunión—. Por eso no lo hace el webview con `setSize`: si el
 * relleno pudiera quedarse atrás aunque fuera un instante, la franja descubierta mostraría lo que
 * hay detrás. La geometría vive en un solo sitio, en Rust (`ajustar_banda`).
 *
 * Se mide con `screenY`, no con `clientY`: la ventana se está moviendo mientras arrastras (el
 * borde inferior queda clavado y el superior sube), así que la coordenada relativa a la ventana
 * cambia sola y el arrastre se volvería inestable.
 */
export const ALTO_COMPACTA = 88;
export const ALTO_AMPLIADA = 200;
/** A partir de aquí la banda se dibuja ampliada. Es el punto medio entre las dos alturas. */
export const DESDE_AMPLIADA = (ALTO_COMPACTA + ALTO_AMPLIADA) / 2;

/** El alto de la VENTANA es la verdad: la banda no decide cuánto mide, la obedece. */
export function useAltoDeVentana(): number {
  const [alto, setAlto] = useState(() => globalThis.innerHeight || ALTO_COMPACTA);
  useEffect(() => {
    const al = () => setAlto(globalThis.innerHeight);
    globalThis.addEventListener("resize", al);
    return () => globalThis.removeEventListener("resize", al);
  }, []);
  return alto;
}

export function useAsa() {
  const asa = useRef<HTMLSpanElement | null>(null);

  useEffect(() => {
    const el = asa.current;
    if (!el) return;

    let desde = 0;
    let altoInicial = 0;
    let pedido = 0;

    const mover = (e: PointerEvent) => {
      const alto = Math.round(
        Math.min(ALTO_AMPLIADA, Math.max(ALTO_COMPACTA, altoInicial + (desde - e.screenY))),
      );
      if (alto === pedido) return;
      pedido = alto;
      void llamar("ajustar_banda", { alto });
    };

    const soltar = (e: PointerEvent) => {
      el.releasePointerCapture?.(e.pointerId);
      globalThis.removeEventListener("pointermove", mover);
      globalThis.removeEventListener("pointerup", soltar);
      // El acople se rehace AQUÍ, no en cada cuadro del arrastre: recortar la ventana de la
      // reunión son varias idas y vueltas a otro proceso por la Accessibility API, y hacerlo
      // sesenta veces por segundo convierte el arrastre en un tirón y deja la reunión
      // parpadeando. Lo nuestro se mueve mientras arrastras; lo ajeno, al soltar.
      void llamar("asentar_banda", { alto: pedido });
    };

    const agarrar = (e: PointerEvent) => {
      desde = e.screenY;
      altoInicial = globalThis.innerHeight;
      pedido = altoInicial;
      el.setPointerCapture?.(e.pointerId);
      globalThis.addEventListener("pointermove", mover);
      globalThis.addEventListener("pointerup", soltar);
      e.preventDefault();
    };

    el.addEventListener("pointerdown", agarrar);
    return () => {
      el.removeEventListener("pointerdown", agarrar);
      globalThis.removeEventListener("pointermove", mover);
      globalThis.removeEventListener("pointerup", soltar);
    };
  }, []);

  return asa;
}
