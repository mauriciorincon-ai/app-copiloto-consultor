import { useEffect, useRef, useState } from "react";
import { llamar } from "./puente";
import type { Borde } from "./franja";

/**
 * EL ASA de la banda.
 *
 * Arrastrarla cambia el alto de la banda **y el de su relleno** —y, al soltarla, el acople rehace
 * la ventana de la reunión—. Por eso no lo hace el webview con `setSize`: si el
 * relleno pudiera quedarse atrás aunque fuera un instante, la franja descubierta mostraría lo que
 * hay detrás. La geometría vive en un solo sitio, en Rust (`ajustar_banda`).
 *
 * Se mide con `screenY`, no con `clientY`: la ventana se está moviendo mientras arrastras (el
 * borde inferior queda clavado y el superior sube), así que la coordenada relativa a la ventana
 * cambia sola y el arrastre se volvería inestable.
 *
 * **Arriba (sprint 004) el asa va en el borde de abajo** y se arrastra hacia abajo para ampliar: el
 * borde que queda clavado es el superior, bajo la barra de menús. La cuenta es la misma con el signo
 * cambiado, y vive en [`altoAlArrastrar`] para que un test la vea en los dos bordes.
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

/**
 * **`⌃⌥T` AGRANDA LA BANDA** para que quepa el transcript. El transcript vive en la columna derecha
 * de la banda ampliada (`banda.html` · transcript, 200 px), y abrirlo cambia el dibujo a «ampliada»;
 * hasta el sprint 003 solo cambiaba el dibujo, y en una ventana de 88 px la banda salía recortada
 * por abajo (casilla 6 del S3). Ahora la ventana crece como la haría crecer el asa —`ajustar_banda`
 * y luego `asentar_banda`, así que el modo solo audio se apaga igual y la reunión se vuelve a hacer
 * sitio— y al cerrarlo vuelve a la compacta, **solo si fue el transcript quien la agrandó**: si la
 * habías ampliado tú con el asa, se queda como la dejaste.
 */
export function useAltoDelTranscript(transcript: boolean, alto: number, activo: boolean): void {
  const agrandada = useRef(false);
  useEffect(() => {
    if (!activo) return;
    if (transcript && alto < DESDE_AMPLIADA && !agrandada.current) {
      agrandada.current = true;
      pedirAlto(ALTO_AMPLIADA);
    } else if (!transcript && agrandada.current) {
      agrandada.current = false;
      if (alto >= DESDE_AMPLIADA) pedirAlto(ALTO_COMPACTA);
    }
  }, [transcript, alto, activo]);
}

/** El mismo par que el asa al soltarla: primero lo nuestro, luego la reunión. */
function pedirAlto(alto: number): void {
  void llamar("ajustar_banda", { alto }).then(() => llamar("asentar_banda", { alto }));
}

/**
 * El alto que pide el asa a mitad de arrastre: desde dónde se agarró, dónde va el puntero y en qué
 * borde vive la banda. Abajo, subir el puntero la amplía; arriba, bajarlo. Siempre entre las dos
 * alturas del diseño.
 */
export function altoAlArrastrar(borde: Borde, altoInicial: number, desde: number, ahora: number): number {
  const delta = borde === "arriba" ? ahora - desde : desde - ahora;
  return Math.round(Math.min(ALTO_AMPLIADA, Math.max(ALTO_COMPACTA, altoInicial + delta)));
}

export function useAsa(borde: Borde = "abajo") {
  const asa = useRef<HTMLSpanElement | null>(null);

  useEffect(() => {
    const el = asa.current;
    if (!el) return;

    let desde = 0;
    let altoInicial = 0;
    let pedido = 0;

    const mover = (e: PointerEvent) => {
      const alto = altoAlArrastrar(borde, altoInicial, desde, e.screenY);
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
      // **Y solo si el alto cambió** (sprint 004, medido en vivo): un clic en el asa para traer la banda
      // al frente —el asa arriba queda justo donde la banda toca la reunión— soltaba y volvía a acoplar
      // la ventana de la reunión sin que nada hubiera cambiado (338 ms de ventana moviéndose de más).
      if (pedido !== altoInicial) void llamar("asentar_banda", { alto: pedido });
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
  }, [borde]);

  return asa;
}
