import { useEffect, useState } from "react";
import { escuchar, hayTauri, llamar, preguntar } from "./puente";
import { useT } from "./i18n";

/**
 * EL PUENTE DEL CUADERNO — lo que las pantallas de la ventana principal preguntan a lo nativo.
 *
 * Todo lo de aquí sigue la misma regla que `useAcoplada`: **se pregunta, no se supone**. Las tres
 * pantallas de la fase 2 muestran estado del SISTEMA —qué reunión hay abierta, qué permisos
 * concedió el usuario, cuántos bytes salieron—, y ese estado cambia fuera de la app: el usuario
 * abre Zoom, macOS revalida el permiso de pantalla a mitad de una reunión.
 *
 * **Fuera de Tauri hay valores de muestra**, los mismos que la maqueta dibuja. No es una puerta
 * trasera: es lo que hace posible el gate de FIDELIDAD, que compara estas pantallas contra
 * `docs/diseno/*.html` en un navegador. Mueren cuando el producto tenga de dónde leerlos de
 * verdad — igual que la muestra de la banda.
 */

export type Proteccion = "Verificada" | "SinVerificar";

export type Reunion =
  | { que: "ninguna" }
  | { que: "detectada"; cliente: string; titulo: string | null; proteccion: Proteccion }
  | { que: "no-se-puede-saber"; motivo: string };

export type EstadoPermiso = "sin-conceder" | "concedido" | "denegado" | "no-se-sabe";

export type Permisos = {
  microfono: EstadoPermiso;
  pantalla: EstadoPermiso;
  accesibilidad: EstadoPermiso;
};

/**
 * Lo que la maqueta dibuja, para el navegador y los tests. El título sale del **diccionario**,
 * no de una constante: escrito a mano en español, el cruce en inglés del gate de fidelidad
 * mostraba texto español dentro de una pantalla inglesa — 0,34 % de divergencia y, sobre todo,
 * una muestra que no es bilingüe en una app que promete serlo en todo.
 */
function reunionDeMuestra(titulo: string): Reunion {
  return { que: "detectada", cliente: "Google Meet", titulo, proteccion: "Verificada" };
}

const PERMISOS_DE_MUESTRA: Permisos = {
  microfono: "concedido",
  pantalla: "sin-conceder",
  accesibilidad: "concedido",
};

/**
 * Pregunta una vez al montar y cuando el usuario vuelve a la ventana.
 *
 * El `focus` es el disparador honesto: el usuario se va a Ajustes del Sistema a conceder un
 * permiso, o abre Zoom, y vuelve. Sondear cada pocos segundos gastaría llamadas a la API de
 * accesibilidad —que van a otro proceso— para preguntar por algo que solo cambia cuando el
 * usuario hace algo fuera.
 */
function usePreguntaAlVolver<T>(comando: string, deMuestra: T): T {
  const [valor, setValor] = useState<T>(deMuestra);
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const preguntarAhora = () => {
      void preguntar<T>(comando).then((v) => {
        if (vivo && v !== null) setValor(v);
      });
    };
    preguntarAhora();
    globalThis.addEventListener("focus", preguntarAhora);
    return () => {
      vivo = false;
      globalThis.removeEventListener("focus", preguntarAhora);
    };
  }, [comando]);
  return valor;
}

export function useReunion(): Reunion {
  const t = useT().cuaderno;
  return usePreguntaAlVolver<Reunion>("reunion_abierta", reunionDeMuestra(t.tituloDeMuestra));
}

export function usePermisos(): Permisos {
  return usePreguntaAlVolver<Permisos>("permisos_de_macos", PERMISOS_DE_MUESTRA);
}

/**
 * Los bytes que salieron del equipo, **leídos**. Una constante escrita en el webview no es un
 * contador: sería la interfaz afirmando el cero en vez de medirlo.
 */
export function useBytesALaRed(): string {
  const [bytes, setBytes] = useState("0 B");
  useEffect(() => {
    if (!hayTauri()) return;
    let vivo = true;
    const leer = () => {
      void preguntar<string>("bytes_a_la_red").then((b) => {
        if (vivo && b !== null) setBytes(b);
      });
    };
    leer();
    const baja = escuchar("corte", leer);
    return () => {
      vivo = false;
      baja();
    };
  }, []);
  return bytes;
}

export function abrirAjustesDe(permiso: "microfono" | "pantalla" | "accesibilidad") {
  void llamar("abrir_ajustes_de", { permiso });
}

export function cortarTodo() {
  void llamar("cortar_todo");
}
