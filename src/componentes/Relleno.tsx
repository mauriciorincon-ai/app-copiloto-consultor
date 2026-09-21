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
 * recibe props a propósito — no hay forma de meterle nada.
 *
 * El usuario eligió **su fondo de escritorio** (mirada 3-quinquies), con negro a una tecla. El
 * fondo de escritorio llega en el siguiente paso de esta fase; hasta entonces, negro, que es la
 * opción segura: no filtra absolutamente nada.
 */
export function Relleno() {
  return <div className="relleno-franja" aria-hidden="true" />;
}
