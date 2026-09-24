/**
 * Qué ventana soy.
 *
 * Las tres ventanas de Angel Ghost cargan el mismo webview y se distinguen por su **etiqueta**,
 * que es la que declara `src-tauri/tauri.conf.json`. Fuera de Tauri —`pnpm dev`, `pnpm preview`,
 * los e2e— no hay etiqueta: entonces manda el parámetro `?ventana=`, y si tampoco está, la
 * principal. Sin eso, abrir el proyecto en un navegador mostraría una banda suelta y nadie
 * entendería qué está viendo.
 */
export const VENTANAS = ["principal", "banda", "relleno"] as const;
export type Ventana = (typeof VENTANAS)[number];

function esVentana(v: string | undefined | null): v is Ventana {
  return VENTANAS.includes(v as Ventana);
}

/** La etiqueta de Tauri, o `undefined` si no estamos dentro de Tauri. */
function etiquetaDeTauri(): string | undefined {
  try {
    const internos = (
      globalThis as unknown as {
        __TAURI_INTERNALS__?: { metadata?: { currentWindow?: { label?: string } } };
      }
    ).__TAURI_INTERNALS__;
    return internos?.metadata?.currentWindow?.label;
  } catch {
    return undefined;
  }
}

export function ventanaActual(busqueda = globalThis.location?.search ?? ""): Ventana {
  const etiqueta = etiquetaDeTauri();
  if (esVentana(etiqueta)) return etiqueta;

  const pedida = new URLSearchParams(busqueda).get("ventana");
  if (esVentana(pedida)) return pedida;

  return "principal";
}
