import { useState } from "react";
import { useIdioma, useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { TodaviaNo, PILA } from "../componentes/Ventana";
import { useBytesALaRed } from "../cuaderno";
import { hayTauri } from "../puente";
import {
  EXTERNOS,
  apiExterna,
  borrarClave,
  dolares,
  guardarClave,
  redactarSugerencias,
  useIa,
  useLoQueSalio,
  type EstadoDeLaIa,
  type LoQueSalio,
  type Externo,
  type PorQueNoRedacta,
} from "../ia";

/**
 * IA — quién redacta la sugerencia, qué sale y cuánto cuesta (C7, sprint 002, fase 5).
 *
 * Referencia: `docs/diseno/ia.html`, estados **«sprint 3 · quién redacta»** y **«sprint 3 · lo que
 * salió»** (mirada 19, aprobada el 2026-09-27): el de «así se ve hoy · sprint 2» (mirada 18) con un
 * botón «Ver lo que salió · N» en la tarjeta del proveedor externo, que abre el estado «API encendido»
 * de la Etapa de Diseño (B37). ADR 010 y 011.
 *
 * Todo nace apagado: redactar sugerencias y el proveedor externo. Encender el externo exige su clave
 * en el Llavero, y la pantalla dice por qué no se puede cuando no se puede.
 */
export function Ia({ busqueda = "" }: { busqueda?: string }) {
  const t = useT().cuaderno;
  const idioma = useIdioma();
  const [ia, setIa] = useIa();
  const salio = useLoQueSalio();
  // «Lo que salió» se abre desde su botón; fuera de Tauri, desde la URL (el arnés de fidelidad).
  const [viendo, setViendo] = useState(new URLSearchParams(busqueda).get("vista") === "salio");
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

  // La elección se le pide SIEMPRE a Rust, también con el API apagado: si solo cambiaba aquí, el
  // estado que Rust devolvía al guardar la clave seguía diciendo «claude», la pantalla volvía sola
  // a Claude y encender pedía la clave equivocada (auditoría del S2, M3). `api_externa` con el API
  // apagado guarda la elección sin exigir clave.
  const elegir = (externo: Externo) => aplicar(apiExterna(ia.api.encendida, externo));

  if (viendo && salio.length > 0) {
    return (
      <>
        <div className="titulo">
          <h1>{t.navIa}</h1>
          <p className="sub">{t.iaSub}</p>
        </div>
        {/* Fuera de Tauri, la cifra de la maqueta: la de las tres peticiones de muestra. */}
        <LoQueSalioAlApi salio={salio} bytes={hayTauri() ? bytes : "12,4 KB"} volver={() => setViendo(false)} />
      </>
    );
  }

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
                {/* Solo cuando algo salió en esta reunión (B37, mirada 19). */}
                {salio.length > 0 && (
                  <>
                    <span className="crece" />
                    <button type="button" className="btn mini" onClick={() => setViendo(true)}>
                      <Ic id="i-nube" s />
                      <span>
                        {t.verLoQueSalio} · {salio.length}
                      </span>
                    </button>
                  </>
                )}
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

/**
 * **LO QUE SALIÓ AL API** (B37, sprint 003): el texto exacto de la última petición —con lo que se tapó
 * en tu Mac tachado y su marcador al lado—, cuántos caracteres fueron, y la tabla de las peticiones de
 * esta reunión. Todo sale del registro en memoria de Rust (`sintesis::api::Registro`): muere con ⌥⎋ y
 * al cerrar la reunión, y aquí solo se enseña.
 */
function LoQueSalioAlApi({ salio, bytes, volver }: { salio: LoQueSalio[]; bytes: string; volver: () => void }) {
  const t = useT().cuaderno;
  const idioma = useIdioma();
  const ultima = salio[0];
  const [cifra, unidad = "B"] = bytes.split(" ");
  const nombre = EXTERNOS.find((e) => e.id === ultima.externo)?.nombre ?? "";
  const usd = (v: number | null) =>
    v === null ? "—" : new Intl.NumberFormat(idioma, { minimumFractionDigits: 3, maximumFractionDigits: 3 }).format(v);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
      <div className="fila">
        <button type="button" className="btn mini" onClick={volver}>
          <span style={{ display: "inline-flex", transform: "rotate(180deg)" }}>
            <Ic id="i-flecha" s />
          </span>
          <span>{t.volverQuienRedacta}</span>
        </button>
        <span className="crece" />
        <span className="mono" style={{ color: "var(--ink-2)" }}>
          {t.seBorraAlCerrar}
        </span>
      </div>
      <div className="grid-2" style={{ gridTemplateColumns: "1.2fr 1fr", gap: "12px", alignItems: "start" }}>
        <div className="tarjeta">
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              {t.loUltimoQueSalio}
            </h2>
            <span className="mono" style={{ color: "var(--ink-2)" }}>
              {ultima.hora}
            </span>
          </div>
          <div
            className="diccionario"
            style={{ marginTop: "8px", fontFamily: "var(--font-evidencia)", fontSize: "13.5px", lineHeight: 1.6 }}
          >
            <p>
              {ultima.trozos.map((tr, i) =>
                tr.que === "texto" ? (
                  <span key={i}>{tr.texto}</span>
                ) : (
                  <span key={i}>
                    <del>{tr.original}</del>
                    <mark>{tr.marcador}</mark>
                  </span>
                ),
              )}
            </p>
          </div>
          <p style={{ fontSize: "11.5px", color: "var(--ink-2)", marginTop: "10px" }}>
            {t.estoEs} <b>{t.todoLoQueSalio}</b> {t.loQueSalioN} {ultima.caracteres} {t.caracteresTachado}
          </p>
        </div>
        <div className="contador api">
          <span className="cifra">
            {cifra} <small>{unidad}</small>
          </span>
          <span className="etq">
            {t.salieron} · {salio.length} {t.peticiones} · {nombre} API
          </span>
        </div>
      </div>
      <div className="tarjeta" style={{ paddingBottom: "6px" }}>
        <h2 className="seccion" style={{ margin: "0 0 4px" }}>
          {t.las} {salio.length} {t.peticionesDeEstaReunion}
        </h2>
        <table className="tabla">
          <thead>
            <tr>
              <th>{t.colHora}</th>
              <th>{t.colPorQue}</th>
              <th className="num">{t.colCaracteres}</th>
              <th className="num">{t.colAnonimizados}</th>
              <th className="num">USD</th>
            </tr>
          </thead>
          <tbody>
            {salio.map((p, i) => (
              <tr key={`${p.hora}-${i}`}>
                <td className="mono">{p.hora}</td>
                <td>
                  {t.redactarSugerencia} · {p.sobre}
                </td>
                <td className="num">{p.caracteres}</td>
                <td className="num">{p.tapadas}</td>
                <td className="num">{usd(p.usd)}</td>
              </tr>
            ))}
          </tbody>
        </table>
        <p style={{ fontSize: "11.5px", color: "var(--ink-2)", marginTop: "6px" }}>{t.registroEnMemoria}</p>
      </div>
    </div>
  );
}
