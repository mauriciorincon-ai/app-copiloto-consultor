import {
  useCallback,
  useEffect,
  useId,
  useRef,
  useState,
  type KeyboardEvent,
  type ReactNode,
} from "react";
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
 * Lo que cuida (auditoría del S13): una sola contestación a la vez (los tres botones esperan a la que está en vuelo); si lo
 * que ibas a empezar deja de ser lo que ves —cambiaste de modo, el radar vio otros programas— el cuadro se cierra y no
 * arranca nada; Esc cancela, Tab no sale del cuadro y el foco vuelve a donde estaba.
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
  /** Cierra el cuadro sin empezar nada, y tira lo que estuviera por preguntarse. Estable entre pintadas. */
  cancelar: () => void;
} {
  const [plataforma] = usePlataforma();
  const t = useTextosDeWindows().reunionAnterior;
  const [pendiente, setPendiente] = useState<{ accion: Accion } | null>(null);
  const [fallo, setFallo] = useState<"guardar" | "descartar" | null>(null);
  const [ocupado, setOcupado] = useState(false);
  const alCancelar = useRef<HTMLButtonElement>(null);
  const quienAbrio = useRef<HTMLElement | null>(null);
  /** Cada pregunta y cada cierre la cambian: lo que vuelve de Rust después de un cierre ya no manda. */
  const vez = useRef(0);
  /** Dos clics antes de repintar no mandan dos veces: esto se marca en el acto, sin esperar al estado. */
  const enVuelo = useRef(false);
  const idDetalle = useId();

  useEffect(() => {
    if (pendiente) alCancelar.current?.focus();
  }, [pendiente]);

  const cancelar = useCallback(() => {
    vez.current += 1;
    enVuelo.current = false;
    setPendiente(null);
    setFallo(null);
    setOcupado(false);
    const el = quienAbrio.current;
    quienAbrio.current = null;
    if (el?.isConnected) el.focus();
  }, []);

  const antes = (accion: Accion) => {
    if (!esWindows(plataforma) || !hayTauri()) {
      void accion();
      return;
    }
    const mia = (vez.current += 1);
    if (document.activeElement instanceof HTMLElement)
      quienAbrio.current = document.activeElement;
    // Si la pregunta misma falla no se adivina: se pregunta, que es lo seguro.
    void reunionPorDecidir()
      .catch(() => true)
      .then((pide) => {
        if (mia !== vez.current) return; // cambiaste de idea mientras Rust contestaba
        if (pide === false) {
          quienAbrio.current = null;
          void accion();
          return;
        }
        setFallo(null);
        setOcupado(false);
        enVuelo.current = false;
        setPendiente({ accion });
      });
  };

  /** Una contestación a la vez: Guardar o Descartar, y los tres botones esperan a que vuelva. */
  const contestar = (cual: "guardar" | "descartar") => {
    if (enVuelo.current || !pendiente) return;
    enVuelo.current = true;
    const { accion } = pendiente;
    const mia = vez.current;
    setOcupado(true);
    setFallo(null);
    void (cual === "guardar" ? guardarLaReunion() : cerrarSinGuardar()).then(
      () => {
        if (mia !== vez.current) return; // se cerró el cuadro mientras tanto: no se empieza lo que ya no ves
        quienAbrio.current = null;
        vez.current += 1;
        setPendiente(null);
        setOcupado(false);
        void accion();
      },
      () => {
        if (mia !== vez.current) return;
        enVuelo.current = false;
        setOcupado(false);
        setFallo(cual);
      },
    );
  };

  const alTeclear = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Escape") {
      e.stopPropagation();
      if (!ocupado) cancelar();
      return;
    }
    if (e.key !== "Tab") return;
    // Es un cuadro modal: Tab da la vuelta dentro de él y no sale a los botones de Start que quedan detrás.
    const botones = Array.from(
      e.currentTarget.querySelectorAll<HTMLButtonElement>("button"),
    );
    const primero = botones[0];
    const ultimo = botones[botones.length - 1];
    if (!primero || !ultimo) return;
    if (e.shiftKey && document.activeElement === primero) {
      e.preventDefault();
      ultimo.focus();
    } else if (!e.shiftKey && document.activeElement === ultimo) {
      e.preventDefault();
      primero.focus();
    }
  };

  const espera = {
    "aria-disabled": ocupado || undefined,
    style: ocupado ? { opacity: 0.55 } : undefined,
  } as const;

  const cuadro = pendiente ? (
    // Un cuadro modal recoge Esc y Tab de sus botones: es el patrón de WAI-ARIA para `alertdialog`, y la regla no lo conoce.
    // eslint-disable-next-line jsx-a11y/no-noninteractive-element-interactions
    <div
      className="franja warn"
      role="alertdialog"
      aria-modal="true"
      aria-label={t.titulo}
      aria-describedby={idDetalle}
      onKeyDown={alTeclear}
      style={{ marginBottom: "12px" }}
    >
      <Ic id="i-alert" relleno />
      <div>
        <strong>{t.titulo}</strong>
        <p id={idDetalle}>{t.detalle}</p>
        {fallo && (
          <p role="alert">
            <strong>{fallo === "guardar" ? t.noGuardo : t.noDescarto}</strong>
          </p>
        )}
        <div className="fila" style={{ gap: "8px", marginTop: "8px" }}>
          <button
            className="btn mini"
            type="button"
            onClick={() => contestar("guardar")}
            {...espera}
          >
            <Ic id="i-candado" s />
            {t.guardar}
          </button>
          <button
            className="btn mini"
            type="button"
            onClick={() => contestar("descartar")}
            {...espera}
          >
            <Ic id="i-basura" s />
            {t.descartar}
          </button>
          <button
            ref={alCancelar}
            className="btn mini"
            type="button"
            onClick={() => {
              if (!ocupado) cancelar();
            }}
            {...espera}
          >
            {t.cancelar}
          </button>
        </div>
      </div>
    </div>
  ) : null;

  return { antes, cuadro, cancelar };
}
