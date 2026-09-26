import { useState } from "react";
import { useIdioma, useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { TodaviaNo, PILA } from "../componentes/Ventana";
import { useBytesALaRed } from "../cuaderno";
import {
  EXTERNOS,
  apiExterna,
  borrarClave,
  dolares,
  guardarClave,
  redactarSugerencias,
  useIa,
  type EstadoDeLaIa,
  type Externo,
  type PorQueNoRedacta,
} from "../ia";

/**
 * IA — quién redacta la sugerencia, qué sale y cuánto cuesta (C7, sprint 002, fase 5).
 *
 * Referencia: `docs/diseno/ia.html`, estado **«así se ve hoy · sprint 2»** (mirada 18; su veredicto
 * viaja al gate del MVP), armado con piezas que la Etapa de Diseño ya aprobó. ADR 010 y 011.
 *
 * Todo nace apagado: redactar sugerencias y el proveedor externo. Encender el externo exige su clave
 * en el Llavero, y la pantalla dice por qué no se puede cuando no se puede.
 */
export function Ia() {
  const t = useT().cuaderno;
  const idioma = useIdioma();
  const [ia, setIa] = useIa();
  const bytes = useBytesALaRed();
  const [cifra, unidad = "B"] = bytes.split(" ");
  const [clave, setClave] = useState("");
  const [sinClave, setSinClave] = useState(false);

  const aplicar = (p: Promise<EstadoDeLaIa | null>) =>
    void p
      .then((e) => {
        if (e) setIa(e);
        setSinClave(false);
      })
      .catch(() => setSinClave(true));

  const porQue: PorQueNoRedacta | null =
    sinClave || (ia.api.encendida && !ia.api.hayClave)
      ? "sin-clave"
      : ia.mesUsd >= ia.topeUsd
        ? "tope-del-mes"
        : ia.sistema;

  const segundos = (ms: number) => {
    const s = (ms / 1000).toFixed(1);
    return `${idioma === "en" ? s : s.replace(".", ",")} s`;
  };

  const elegir = (externo: Externo) =>
    ia.api.encendida
      ? aplicar(apiExterna(true, externo))
      : setIa({ ...ia, api: { ...ia.api, externo } });

  return (
    <>
      <div className="titulo">
        <h1>{t.navIa}</h1>
        <p className="sub">{t.iaSub}</p>
      </div>

      <div style={{ ...PILA, gap: "12px" }}>
        <div className="grid-2" style={{ gridTemplateColumns: "1.25fr 1fr", gap: "12px", alignItems: "start" }}>
          <div className="tarjeta" style={{ paddingBottom: "8px" }}>
            <div className="fila">
              <h2 className="seccion crece" style={{ margin: 0 }}>
                {t.quienRedacta}
              </h2>
              <QuienRedacta ia={ia} />
            </div>
            <table className="tabla">
              <thead>
                <tr>
                  <th>{t.proveedor}</th>
                  <th>{t.estadoColumna}</th>
                  <th className="num">{t.latencia}</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>{t.modeloDelSistema}</td>
                  <td>
                    {ia.quien === "sistema" ? (
                      <span className="estado ok">
                        <Ic id="i-mac" s relleno />
                        <span>{t.enUso}</span>
                      </span>
                    ) : ia.sistema === null ? (
                      t.respaldo
                    ) : (
                      <span className="estado warn">
                        <Ic id="i-alert" s relleno />
                        <span>{t.noDisponible}</span>
                      </span>
                    )}
                  </td>
                  <td className="num">
                    {ia.quien === "sistema" && ia.latenciaMs !== null ? segundos(ia.latenciaMs) : "—"}
                  </td>
                </tr>
                <tr className="apagada">
                  <td>MLX · Qwen 3 4B</td>
                  <td>
                    <TodaviaNo />
                  </td>
                  <td className="num">—</td>
                </tr>
                <tr className={ia.api.encendida ? undefined : "apagada"}>
                  <td>{t.apiExterno}</td>
                  <td>
                    {ia.quien === "api" ? (
                      <span className="estado ok">
                        <Ic id="i-nube" s />
                        <span>{t.enUso}</span>
                      </span>
                    ) : ia.api.encendida ? (
                      <span className="estado warn">
                        <Ic id="i-alert" s relleno />
                        <span>{t.noDisponible}</span>
                      </span>
                    ) : (
                      t.apagado
                    )}
                  </td>
                  <td className="num">
                    {ia.quien === "api" && ia.latenciaMs !== null ? segundos(ia.latenciaMs) : "—"}
                  </td>
                </tr>
              </tbody>
            </table>
            {porQue && (
              <p style={{ fontSize: "11.5px", color: "var(--ink-2)", marginTop: "6px" }}>
                {t.porQueNoRedacta[porQue]}
              </p>
            )}
          </div>
          <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
            <div className={bytes === "0 B" ? "contador cero" : "contador api"}>
              <span className="cifra">
                {cifra} <small>{unidad}</small>
              </span>
              <span className="etq">{t.salieronDeTuEquipo}</span>
            </div>
            <div className="franja ok" role="status">
              <Ic id="i-check-circle" relleno />
              <div>
                <strong>{t.siFallaQuedan}</strong>
                <p>{t.siFallaDetalle}</p>
              </div>
            </div>
          </div>
        </div>

        <Interruptor
          encendido={ia.redactar}
          etiqueta={t.redactarSugerencias}
          alCambiar={() => aplicar(redactarSugerencias(!ia.redactar))}
          relleno="2px"
        />

        <div className="grid-2" style={{ gridTemplateColumns: "1.25fr 1fr", gap: "12px", alignItems: "start" }}>
          <div className="tarjeta">
            <div className="fila">
              <h2 className="seccion crece" style={{ margin: 0 }}>
                {t.proveedorExterno}
              </h2>
              <Interruptor
                encendido={ia.api.encendida}
                etiqueta={ia.api.encendida ? t.encendido : t.apagado}
                alCambiar={() => aplicar(apiExterna(!ia.api.encendida, ia.api.externo))}
              />
            </div>
            <div className="fila" style={{ gap: "6px", marginTop: "8px" }}>
              {EXTERNOS.map((e) => (
                <button
                  key={e.id}
                  type="button"
                  className="btn mini"
                  aria-pressed={ia.api.externo === e.id}
                  onClick={() => elegir(e.id)}
                >
                  {ia.api.externo === e.id && <Ic id="i-check-circle" s relleno color="var(--halo)" />}
                  {e.nombre}
                </button>
              ))}
            </div>
            <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "8px" }}>
              <label className="campo">
                <span>{t.tuClave}</span>
                <input
                  className="mono"
                  type="password"
                  placeholder="sk-…"
                  autoComplete="off"
                  spellCheck={false}
                  value={clave}
                  onChange={(e) => setClave(e.target.value)}
                />
                <span className="ayuda">{t.enTuLlavero}</span>
              </label>
              <div className="fila" style={{ gap: "8px" }}>
                <button
                  type="button"
                  className="btn mini"
                  disabled={clave.trim() === ""}
                  onClick={() => {
                    const k = clave;
                    setClave("");
                    aplicar(guardarClave(ia.api.externo, k));
                  }}
                >
                  <Ic id="i-candado" s />
                  <span>{t.guardarEnLlavero}</span>
                </button>
                <button type="button" className="btn mini" onClick={() => aplicar(borrarClave(ia.api.externo))}>
                  <span>{t.borrarLaClave}</span>
                </button>
              </div>
            </div>
          </div>
          <div className="tarjeta" style={{ paddingBottom: "6px" }}>
            <h2 className="seccion" style={{ margin: "0 0 4px" }}>
              {t.costo}
            </h2>
            <table className="tabla">
              <tbody>
                <tr>
                  <td>{t.estaReunion}</td>
                  <td className="num">{dolares(ia.reunionUsd, idioma)}</td>
                </tr>
                <tr>
                  <td>{t.esteMes}</td>
                  <td className="num">{dolares(ia.mesUsd, idioma)}</td>
                </tr>
                <tr>
                  <td>{t.topeDelMes}</td>
                  <td className="num">{dolares(ia.topeUsd, idioma)}</td>
                </tr>
              </tbody>
            </table>
            <p style={{ fontSize: "11.5px", color: "var(--ink-2)", marginTop: "6px" }}>{t.alLlegarAlTope}</p>
          </div>
        </div>
      </div>
    </>
  );
}

/** El chip de la cabecera: quién redacta ahora, con símbolo y palabra. */
function QuienRedacta({ ia }: { ia: EstadoDeLaIa }) {
  const t = useT().cuaderno;
  if (ia.quien === "sistema")
    return (
      <span className="estado ok">
        <Ic id="i-mac" s relleno />
        <span>{t.enTuMac}</span>
      </span>
    );
  if (ia.quien === "api")
    return (
      <span className="estado halo">
        <Ic id="i-nube" s />
        <span>{EXTERNOS.find((e) => e.id === ia.api.externo)?.nombre} · API</span>
      </span>
    );
  if (ia.quien === "mock")
    return (
      <span className="estado mute">
        <span>mock</span>
      </span>
    );
  return (
    <span className="estado warn">
      <Ic id="i-alert" s relleno />
      <span>{t.nadie}</span>
    </span>
  );
}

/** Un interruptor de verdad (`role="switch"`), vestido con la clase canon, como el de Sesión. */
function Interruptor({
  encendido,
  etiqueta,
  alCambiar,
  relleno = "0",
}: {
  encendido: boolean;
  etiqueta: string;
  alCambiar: () => void;
  /** La maqueta da 2 px al de «Redactar sugerencias», que va suelto, y ninguno al de la tarjeta. */
  relleno?: string;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={encendido}
      className={encendido ? "conm on" : "conm off"}
      style={{ background: "none", border: 0, padding: relleno, fontFamily: "inherit", alignSelf: "flex-start" }}
      onClick={alCambiar}
    >
      <span className="track"></span>
      <span className="etq">{etiqueta}</span>
    </button>
  );
}
