/**
 * El puente con la parte nativa.
 *
 * Fuera de Tauri —`pnpm dev`, `pnpm preview`, los e2e, los tests— no hay nadie al otro lado. En
 * vez de que cada sitio recuerde comprobarlo, se comprueba aquí una vez: `llamar` no hace nada y
 * lo dice, en lugar de lanzar un error que rompería la pantalla entera por no poder mover una
 * ventana. La UI de la banda tiene que seguir dibujándose en un navegador: es lo que hace posible
 * el gate de fidelidad.
 */
export function hayTauri(): boolean {
  return "__TAURI_INTERNALS__" in globalThis;
}

/**
 * **Los dos módulos de Tauri se importan UNA vez y se comparten** (sprint 004). Con dos preguntas a la
 * vez —el relleno pide su fondo y la franja al montarse— había dos `import()` simultáneos del mismo
 * módulo, y en los tests el segundo resolvía al módulo REAL en vez del fingido: «`__TAURI_INTERNALS__
 * .invoke` is not a function», suelto, después de pasar la prueba (medido: `fondo_del_relleno` iba al
 * fingido y `la_franja` al real). En la app no cambia nada: el mismo módulo, cargado una vez.
 */
let nucleo: Promise<typeof import("@tauri-apps/api/core")> | null = null;
let eventos: Promise<typeof import("@tauri-apps/api/event")> | null = null;
const elNucleo = () => (nucleo ??= import("@tauri-apps/api/core"));
const losEventos = () => (eventos ??= import("@tauri-apps/api/event"));

export async function llamar(comando: string, args?: Record<string, unknown>): Promise<boolean> {
  if (!hayTauri()) return false;
  const { invoke } = await elNucleo();
  await invoke(comando, args);
  return true;
}

/** Como `llamar`, pero con respuesta. Fuera de Tauri devuelve `null`: no hay a quién preguntar. */
export async function preguntar<T>(
  comando: string,
  args?: Record<string, unknown>,
): Promise<T | null> {
  if (!hayTauri()) return null;
  const { invoke } = await elNucleo();
  return (await invoke(comando, args)) as T;
}

/**
 * Se suscribe a un evento de la parte nativa y devuelve cómo darse de baja.
 *
 * Devuelve una función SIEMPRE —vacía fuera de Tauri— para que quien limpia no tenga que
 * comprobar nada: un `useEffect` que a veces devuelve `undefined` es una fuga esperando a que
 * alguien reordene las condiciones.
 */
export function escuchar<T>(evento: string, alOir: (dato: T) => void): () => void {
  if (!hayTauri()) return () => {};
  let apagar: (() => void) | null = null;
  let vivo = true;
  void losEventos().then(async ({ listen }) => {
    const baja = await listen<T>(evento, (e) => alOir(e.payload));
    if (vivo) apagar = () => soltar(baja);
    else soltar(baja);
  });
  return () => {
    vivo = false;
    apagar?.();
  };
}

/**
 * **Darse de baja sin dejar una promesa rechazada suelta.**
 *
 * La baja de Tauri es asíncrona y **puede fallar**: si el componente se desmonta mientras `listen`
 * todavía no contestó —React lo hace a propósito en desarrollo, montando dos veces—, Tauri ya no
 * tiene al oyente en su tabla y su baja revienta con «`listeners[eventId].handlerId`». Nadie la
 * esperaba, así que salía como «Unhandled rejection» en cada arranque: tres por arranque en los
 * logs en vivo de las fases 1, 3 y 4 del sprint 002, y nadie lo había anotado hasta la corrida en
 * vivo del radar. Un oyente que ya no está no hay que quitarlo: el fallo se atrapa y se sigue.
 */
function soltar(baja: () => unknown) {
  void Promise.resolve()
    .then(baja)
    .catch(() => undefined);
}
