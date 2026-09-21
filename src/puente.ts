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

export async function llamar(comando: string, args?: Record<string, unknown>): Promise<boolean> {
  if (!hayTauri()) return false;
  const { invoke } = await import("@tauri-apps/api/core");
  await invoke(comando, args);
  return true;
}
