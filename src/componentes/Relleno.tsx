import { useEffect, useState } from "react";
import { preguntar } from "../puente";
import { useAltoDeVentana } from "../asa";
import { useFranja, type Borde, type LaFranja } from "../franja";

/**
 * EL RELLENO DE LA FRANJA.
 *
 * La banda lleva el flag que la borra de cualquier captura, así que al compartir la pantalla
 * completa el cliente vería, en esa franja, **lo que haya detrás**: no solo el escritorio — también
 * las ventanas de otras aplicaciones. Por eso la banda no viaja sola: viaja con una ventana de
 * relleno de su mismo tamaño, sin protección, que es lo único que la captura encuentra ahí.
 *
 * **Nace sin contenido y no puede tener contenido.** No es un contenedor: es un color o una
 * imagen. Su capability no incluye nada que pueda cargar o mostrar algo, y este componente no
 * recibe props a propósito — no hay forma de meterle nada desde fuera.
 *
 * El usuario eligió **su fondo de escritorio** (mirada 3-quinquies), con negro a una tecla. La
 * imagen la da Rust (`fondo_del_relleno`), ya en `data:`, y el encuadre lo reproduce este
 * componente: la pantalla entera, subida para que por la franja asome justo el trozo que estaría
 * debajo. En cualquier fallo —sin imagen, formato desconocido, demasiado pesada— queda el negro
 * del CSS, que no es un modo degradado: es la otra opción aprobada, y la única que no filtra nada.
 *
 * **Con la banda arriba (sprint 004)** el trozo que asoma es el de justo debajo de la barra de menús:
 * la imagen sube lo que mide la barra, sea cual sea el alto. Abajo, como siempre, sube la pantalla
 * entera menos el alto. La cuenta vive en [`desplazamientoDelFondo`], con su test en los dos bordes.
 */
function useFondoDeEscritorio(): string | null {
  const [fondo, setFondo] = useState<string | null>(null);
  useEffect(() => {
    let vivo = true;
    void preguntar<string | null>("fondo_del_relleno").then((f) => {
      if (vivo) setFondo(f);
    });
    return () => {
      vivo = false;
    };
  }, []);
  return fondo;
}

/**
 * Cuánto se sube el fondo de escritorio (la `top` de la imagen, en px). Abajo, la franja es el final
 * de la pantalla: se sube todo menos el alto. Arriba, la franja empieza bajo la barra de menús: se sube
 * lo que mide la barra, y el asa no cambia nada.
 */
export function desplazamientoDelFondo(borde: Borde, barra: number, alto: number, pantalla: number): number {
  return borde === "arriba" ? -barra : alto - pantalla;
}

/** Fuera de Tauri, la franja de abajo: la del arnés de capturas desde el S1. */
const FUERA: LaFranja = { borde: "abajo", barra: 0, avisoVisto: true };

export function Relleno() {
  const fondo = useFondoDeEscritorio();
  // El alto de la ventana, no el de la pantalla: el asa lo cambia, y con él cambia cuánto hay
  // que subir la imagen para que la costura siga cayendo en el mismo sitio.
  const alto = useAltoDeVentana();
  const { borde, barra } = useFranja(FUERA);

  if (!fondo) return <div className="relleno-franja" aria-hidden="true" />;

  const pantalla = globalThis.screen;
  return (
    <div className="relleno-franja" aria-hidden="true">
      <div
        className="relleno-fondo"
        style={{
          backgroundImage: `url("${fondo}")`,
          width: `${pantalla.width}px`,
          height: `${pantalla.height}px`,
          top: `${desplazamientoDelFondo(borde, barra, alto, pantalla.height)}px`,
        }}
      />
    </div>
  );
}
