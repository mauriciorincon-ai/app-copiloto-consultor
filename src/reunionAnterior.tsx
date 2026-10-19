import { useEffect, useRef, useState, type ReactNode } from "react";
import { Ic } from "./componentes/Iconos";
import { useTextosDeWindows } from "./i18n/windows";
import { cerrarSinGuardar, guardarLaReunion, reunionPorDecidir } from "./notas";
import { esWindows, usePlataforma } from "./plataforma";
import { hayTauri } from "./puente";

/**
 * **LA REUNIÓN ANTERIOR, ANTES DE EMPEZAR OTRA** (sprint 013, decisión 2a; lección del S12).
 *
 * Si la reunión anterior sigue abierta con algo tuyo dentro, Rust la guarda sola al empezar otra (`reunion::al_empezar`,
 * ADR 015 §7). En el Mac eso es lo que se espera. En Windows guardar por primera vez toca dos protecciones del sistema —la
 * llave de tus notas en el Administrador de credenciales y la tarea de borrado del Programador de tareas— y una
 * protección no se toca sin que lo hayas dicho. Así que **en Windows, cada camino que empieza una sesión** («Start
 * session», «Notes only», «Start in person» y los dos «iniciar de todos modos») pasa por aquí: si Rust dice que hay algo
 * por decidir, se enseña «Guardar · Descartar · Cancelar» y no se empieza nada hasta que contestes.
 *
 * Lo que NO cubre, a propósito: cerrar la ventana con la reunión abierta la guarda sola, como hoy (`reunion::al_salir`).
 *
 * En el Mac no hay cuadro, y fuera de Tauri (la maqueta, los e2e) tampoco: no hay a quién preguntar.
 */

type Accion = () => unknown;

export function useAntesDeEmpezar(): {
  /** Empieza lo que pidas, o pregunta primero. */
  antes: (accion: Accion) => void;
  /** El cuadro; `null` si no hay nada que preguntar. Va arriba de la pantalla. */
  cuadro: ReactNode;
} {
  const [plataforma] = usePlataforma();
  const t = useTextosDeWindows().reunionAnterior;
  const [pendiente, setPendiente] = useState<{ accion: Accion } | null>(null);
  const [noGuardo, setNoGuardo] = useState(false);
  const cancelar = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (pendiente) cancelar.current?.focus();
  }, [pendiente]);

  const antes = (accion: Accion) => {
    if (!esWindows(plataforma) || !hayTauri()) {
      void accion();
      return;
    }
    // Si la pregunta misma falla no se adivina: se pregunta, que es lo seguro.
    void reunionPorDecidir()
      .catch(() => true)
      .then((pide) => {
        if (pide === false) {
          void accion();
          return;
        }
        setNoGuardo(false);
        setPendiente({ accion });
      });
  };

  const seguir = () => {
    const p = pendiente;
    setPendiente(null);
    setNoGuardo(false);
    if (p) void p.accion();
  };

  const guardar = () =>
    void guardarLaReunion().then(seguir, () => setNoGuardo(true));
  const descartar = () => void cerrarSinGuardar().then(seguir);

  const cuadro = pendiente ? (
    <div
      className="franja warn"
      role="alertdialog"
      aria-label={t.titulo}
      style={{ marginBottom: "12px" }}
    >
      <Ic id="i-alert" relleno />
      <div>
        <strong>{t.titulo}</strong>
        <p>{t.detalle}</p>
        {noGuardo && (
          <p role="alert">
            <strong>{t.noGuardo}</strong>
          </p>
        )}
        <div className="fila" style={{ gap: "8px", marginTop: "8px" }}>
          <button className="btn mini" type="button" onClick={guardar}>
            <Ic id="i-candado" s />
            {t.guardar}
          </button>
          <button className="btn mini" type="button" onClick={descartar}>
            <Ic id="i-basura" s />
            {t.descartar}
          </button>
          <button
            ref={cancelar}
            className="btn mini"
            type="button"
            onClick={() => {
              setPendiente(null);
              setNoGuardo(false);
            }}
          >
            {t.cancelar}
          </button>
        </div>
      </div>
    </div>
  ) : null;

  return { antes, cuadro };
}
