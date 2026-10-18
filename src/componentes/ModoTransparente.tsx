import { useState } from "react";
import { useIdioma } from "../i18n";
import { useTextosDeWindows } from "../i18n/windows";
import { useVoz } from "../cuaderno";
import {
  decirElAviso,
  elClienteConsintio,
  fijarModoTransparente,
  type VistaDelCliente,
} from "../jurisdiccion";
import { Ic } from "./Iconos";

/**
 * **EL MODO TRANSPARENTE** en «Este cliente» (sprint 012, ADR 024) — solo Windows, y sin maqueta (diccionario aparte,
 * ADR 022): maquetado, no visto, al gate del hito.
 *
 * Un interruptor por cliente (se recuerda, como la NDA) que enseña el aviso FIJO en los dos idiomas: lo que le dices al
 * cliente antes de empezar. «El cliente consintió» deja la constancia en tus notas cifradas —y solo ahí—; con el modo
 * solo audio encendido, la voz te lo lee. La app no manda nada a nadie y no redacta el aviso con un modelo.
 */

/** El orden de los dos avisos: el idioma de la interfaz primero. */
const ES_Y_EN = ["es", "en"] as const;
const EN_Y_ES = ["en", "es"] as const;

type Hecho = "anotada" | "no-anotada" | "dicho" | "no-dicho";

export function ElModoTransparente({
  vista,
  setVista,
  abierta,
}: {
  vista: VistaDelCliente;
  setVista: (v: VistaDelCliente) => void;
  /** Hay una reunión en marcha donde dejar la constancia (la reunión o presencial; en solo notas nada se captura y Sesión no lo ofrece). */
  abierta: boolean;
}) {
  const tt = useTextosDeWindows().transparente;
  const idioma = useIdioma();
  const voz = useVoz();
  const [hecho, setHecho] = useState<Hecho | null>(null);
  const encendido = vista.transparente;
  // El idioma de la interfaz primero: lo lees tú; el otro va debajo, con su `lang`, para quien lo escuche.
  const orden = idioma === "es" ? ES_Y_EN : EN_Y_ES;

  const cambiar = () => {
    setHecho(null);
    void fijarModoTransparente(!encendido).then((v) => v && setVista(v));
  };
  const consintio = () => {
    if (!abierta) return;
    setHecho(null);
    void elClienteConsintio().then(
      (r) => setHecho(r ? "anotada" : "no-anotada"),
      () => setHecho("no-anotada"),
    );
  };
  const decirlo = () => {
    setHecho(null);
    void decirElAviso().then(
      (r) => setHecho(r ? "dicho" : "no-dicho"),
      () => setHecho("no-dicho"),
    );
  };

  return (
    <div
      className="transparente"
      style={{ display: "flex", flexDirection: "column", gap: "6px" }}
    >
      <button
        type="button"
        role="switch"
        aria-checked={encendido}
        className={encendido ? "conm on" : "conm off"}
        style={{
          background: "none",
          border: 0,
          padding: 0,
          fontFamily: "inherit",
          alignSelf: "flex-start",
        }}
        onClick={cambiar}
      >
        <span className="track" />
        <span className="etq">
          {encendido && <Ic id="i-check-circle" s relleno />}
          <span>
            {tt.titulo} · {encendido ? tt.encendido : tt.apagado}
          </span>
        </span>
      </button>
      <p className="aviso-legal">{tt.ayuda}</p>

      {encendido && (
        <>
          <div
            role="group"
            aria-label={tt.loQueLeDices}
            style={{
              border: "var(--borde) solid var(--line)",
              borderRadius: "var(--r-ficha)",
              background: "var(--surface-2)",
              padding: "8px 10px",
              display: "flex",
              flexDirection: "column",
              gap: "4px",
            }}
          >
            <h3 className="seccion" style={{ margin: "0 0 4px" }}>
              {tt.loQueLeDices}
            </h3>
            {orden.map((l) => (
              <p key={l} className="clausula" lang={l}>
                {vista.aviso[l]}
              </p>
            ))}
          </div>
          <div className="fila" style={{ flexWrap: "wrap" }}>
            <button
              type="button"
              className="btn mini"
              aria-disabled={abierta ? undefined : true}
              aria-describedby="transparente-ayuda"
              style={
                abierta ? undefined : { opacity: 0.55, cursor: "not-allowed" }
              }
              onClick={consintio}
            >
              <Ic id="i-check-circle" s />
              <span>{tt.consintio}</span>
            </button>
            {voz.encendida && (
              <button type="button" className="btn mini" onClick={decirlo}>
                <Ic id="i-voz" s />
                <span>{tt.decirlo}</span>
              </button>
            )}
          </div>
          <p id="transparente-ayuda" className="aviso-legal">
            {abierta ? tt.consintioAyuda : tt.sinReunion}
          </p>
          {hecho && (
            <div className="fila" role="status">
              {hecho === "anotada" || hecho === "dicho" ? (
                <Ic id="i-check-circle" s relleno color="var(--ok)" />
              ) : (
                <Ic id="i-alert" s relleno color="var(--warn)" />
              )}
              <span className="crece" style={{ fontSize: "var(--t-meta)" }}>
                {hecho === "anotada"
                  ? tt.anotada
                  : hecho === "no-anotada"
                    ? tt.noAnotada
                    : hecho === "dicho"
                      ? tt.dicho
                      : tt.noCabe}
              </span>
            </div>
          )}
        </>
      )}
    </div>
  );
}
