import { useEffect, useRef, useState, type CSSProperties } from "react";
import { useIdioma, useT, type Idioma } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { hayTauri } from "../puente";
import { formatearBytes } from "./Honestidad";
import {
  anotarAcuerdo,
  borrarReunion,
  cerrarSinGuardar,
  conservarMisTurnos,
  DIAS_DE_MUESTRA,
  elegirCarpetaDeNotas,
  escribirNota,
  exportarReunion,
  fijarRetencion,
  guardarLaReunion,
  useCuaderno,
  useMuestraDelCuaderno,
  useReuniones,
  type FichaFijada,
  type ReunionGuardada,
  type Retencion,
  type VistaDeNotas,
  type VistaDelCuaderno,
} from "../notas";

/**
 * NOTAS — «Lo que escribes tú. Es lo único de la reunión que llega al día siguiente.» (C9, ADR 015)
 *
 * Referencia: `docs/diseno/notas.html`, estados **«sprint 3 · durante»**, **«sprint 3 · al cerrar»**,
 * **«sprint 3 · el archivo»** y **«sprint 3 · exportar»** (mirada 19, aprobada el 2026-09-27), más
 * los tres que llegaron con el código: la lista vacía, los errores y la pregunta antes de borrar.
 *
 * **Qué vista se enseña lo decide el cuaderno, no la pantalla:** con la reunión abierta y escuchando,
 * «durante»; abierta y parada, «al cerrar»; sin reunión, «el archivo». Fuera de Tauri lo pide la URL
 * (`?estado=`), que es como el arnés de fidelidad recorre las cuatro.
 *
 * Aquí no se decide nada de lo que se guarda: la nota, los acuerdos y las fijadas viven en Rust
 * (`notas/`, módulo protegido) y esta pantalla solo los enseña y los manda.
 */
const VISTAS: VistaDeNotas[] = ["durante", "al-cerrar", "archivo", "exportar"];
const RETENCIONES: Retencion[] = ["7d", "30d", "90d", "1a", "siempre"];
const COLUMNA: CSSProperties = { display: "flex", flexDirection: "column", gap: "12px" };
const AYUDA: CSSProperties = { fontSize: "11.5px", color: "var(--ink-2)" };
const DIA = 86_400;
/** Una reunión que vence en dos semanas o menos se pinta apagada: está a punto de irse. */
const A_PUNTO_DE_IRSE = 14;

/** Los meses de la columna «fecha», como los escribe la maqueta: «20 sep». */
const MESES: Record<Idioma, string[]> = {
  es: ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"],
  en: ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"],
};

function vistaDeLaUrl(busqueda: string): VistaDeNotas {
  const pedida = new URLSearchParams(busqueda).get("estado");
  return VISTAS.find((v) => v === pedida) ?? "archivo";
}

export function Notas({ busqueda = "" }: { busqueda?: string }) {
  const pedida = vistaDeLaUrl(busqueda);
  const muestra = useMuestraDelCuaderno(pedida);
  const [cuaderno, volver] = useCuaderno(muestra);
  const t = useT().notas;

  // Dentro del producto la vista la decide el cuaderno; fuera, la URL. **Mientras Rust no ha
  // contestado, ninguna**: pintar «el archivo» un instante antes de «durante» era un parpadeo que
  // además se suscribía y se daba de baja por nada.
  const vista: VistaDeNotas | null = !hayTauri()
    ? pedida
    : cuaderno === null
      ? null
      : cuaderno.abierta
        ? cuaderno.escuchando
          ? "durante"
          : "al-cerrar"
        : "archivo";

  return (
    <>
      <div className="titulo">
        <h1>{t.titulo}</h1>
        <p className="sub">{t.sub}</p>
      </div>
      {vista === "durante" && cuaderno && (
        <Durante cuaderno={cuaderno} volver={volver} foco={new URLSearchParams(busqueda).get("foco") === "nota"} />
      )}
      {vista === "al-cerrar" && cuaderno && <AlCerrar cuaderno={cuaderno} volver={volver} />}
      {(vista === "archivo" || vista === "exportar") && (
        <Archivo retencion={cuaderno?.retencion ?? "90d"} preguntaInicial={vista === "exportar" ? "exportar" : null} />
      )}
    </>
  );
}

// ---- durante ---------------------------------------------------------------------------------

function Durante({ cuaderno, volver, foco }: { cuaderno: VistaDelCuaderno; volver: () => void; foco: boolean }) {
  const t = useT().notas;
  const [nota, setNota] = useState(cuaderno.nota);
  const [acuerdo, setAcuerdo] = useState("");
  const campo = useRef<HTMLTextAreaElement>(null);

  // ⌃⌥N: el cursor, al final de tu nota.
  useEffect(() => {
    const c = campo.current;
    if (!foco || !c) return;
    c.focus();
    c.setSelectionRange(c.value.length, c.value.length);
  }, [foco]);

  return (
    <div style={COLUMNA}>
      <div className="grid-2" style={{ gridTemplateColumns: "1.3fr 1fr", gap: "12px", alignItems: "start" }}>
        <div className="tarjeta" style={{ display: "flex", flexDirection: "column", gap: "8px", minHeight: "300px" }}>
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              <label htmlFor="tu-nota">{t.tuNota}</label>
            </h2>
            <span className="estado ok">
              <Ic id="i-ojo-off" s />
              <span>{t.clienteNoLaVe}</span>
            </span>
            <span className="tecla">
              <kbd>⌃</kbd>
              <kbd>⌥</kbd>
              <kbd>N</kbd>
            </span>
          </div>
          <textarea
            id="tu-nota"
            ref={campo}
            className="nota-campo crece"
            value={nota}
            onChange={(e) => {
              setNota(e.target.value);
              escribirNota(e.target.value);
            }}
          />
          <p style={AYUDA}>{t.notaAyuda}</p>
        </div>
        <div style={COLUMNA}>
          <div className="tarjeta" style={{ paddingBottom: "8px" }}>
            <div className="fila">
              <h2 className="seccion crece" style={{ margin: 0 }}>
                {t.fichasQueFijaste}
              </h2>
              <span className="tecla">
                <kbd>⌃</kbd>
                <kbd>⌥</kbd>
                <kbd>P</kbd>
              </span>
            </div>
            {/* Sin ninguna, la tarjeta se queda con su título y su tecla: ⌃⌥P ya dice cómo se llena. */}
            {cuaderno.fijadas.map((f, i) => (
              <Fijada key={`${f.titular}-${i}`} ficha={f} />
            ))}
          </div>
          <div className="tarjeta" style={{ paddingBottom: "8px" }}>
            <h2 className="seccion" style={{ margin: "0 0 4px" }}>
              <label htmlFor="acuerdo">{t.acuerdos}</label>
            </h2>
            {cuaderno.acuerdos.map((a, i) => (
              <div className="mas" key={`${a}-${i}`}>
                <Ic id="i-check-circle" s />
                <span className="titulos crece">{a}</span>
              </div>
            ))}
            <input
              id="acuerdo"
              className="acuerdo-campo"
              placeholder={t.escribeElAcuerdo}
              value={acuerdo}
              onChange={(e) => setAcuerdo(e.target.value)}
              onKeyDown={(e) => {
                if (e.key !== "Enter" || !acuerdo.trim()) return;
                e.preventDefault();
                void anotarAcuerdo(acuerdo).then(() => {
                  setAcuerdo("");
                  volver();
                });
              }}
            />
            <p style={{ ...AYUDA, marginTop: "4px" }}>{t.acuerdosAyuda}</p>
          </div>
        </div>
      </div>
      <div className="franja ok" role="status">
        <Ic id="i-check-circle" relleno />
        <div>
          <strong>{t.sobrevive}</strong>
          <p>{t.sobreviveDetalle}</p>
        </div>
      </div>
    </div>
  );
}

/** Una fijada: su titular y, a la derecha, su sección o —si no tiene— su unidad («caso»). */
function Fijada({ ficha }: { ficha: FichaFijada }) {
  const unidades = useT().banda.unidades;
  return (
    <div className="mas">
      <Ic id="i-pin-lleno" s />
      <span className="titulos crece">{ficha.titular}</span>
      <span className="n">{ficha.seccion ?? (ficha.unidad ? unidades[ficha.unidad] : "")}</span>
    </div>
  );
}

// ---- al cerrar -------------------------------------------------------------------------------

function AlCerrar({ cuaderno, volver }: { cuaderno: VistaDelCuaderno; volver: () => void }) {
  const t = useT().notas;
  const idioma = useIdioma();
  const [fallo, setFallo] = useState(false);
  const r = cuaderno.resumen;
  const kb = (b: number) => formatearBytes(b, idioma);
  const fila = (icono: string, que: string, donde: string, cuanto: string, muerto = false) => (
    <div className={muerto ? "buffer muerto" : "buffer"} key={que}>
      <Ic id={icono} s />
      <span className="que">{que}</span>
      <span className="donde">{donde}</span>
      <span className="cuanto">{cuanto}</span>
    </div>
  );
  const p = cuaderno.previsto;
  const linea = p
    ? `${p.fecha}${p.cliente ? ` · ${p.cliente}` : ""} · ${p.minutos} ${t.min} → ${p.archivo}`
    : null;

  return (
    <div style={COLUMNA}>
      {fallo && (
        <div className="franja err" role="alert">
          <Ic id="i-x-circle" relleno />
          <div>
            <strong>{t.noSeGuardo}</strong>
            <p>{t.noSeGuardoDetalle}</p>
            <div className="fila" style={{ gap: "8px", marginTop: "8px" }}>
              <button
                className="btn mini"
                type="button"
                onClick={() => void elegirCarpetaDeNotas().then((c) => c && setFallo(false))}
              >
                {t.elegirOtraCarpeta}
              </button>
            </div>
          </div>
        </div>
      )}
      <div className="grid-2" style={{ gap: "12px", alignItems: "start" }}>
        <div className="tarjeta" style={{ paddingBottom: "8px" }}>
          <h2 className="seccion" style={{ margin: "0 0 6px" }}>
            {t.seVaAGuardar}
          </h2>
          {fila("i-nota", t.tusNotas, `${r.parrafos} ${t.parrafos}`, kb(r.bytesNota))}
          {fila("i-pin-lleno", t.fichasFijadas, `${r.fijadas} · ${t.tituloYFuente}`, kb(r.bytesFijadas))}
          {fila("i-check-circle", t.acuerdos, `${r.acuerdos} · ${t.marcadosPorTi}`, kb(r.bytesAcuerdos))}
          {cuaderno.conservarMisTurnos &&
            fila("i-mic", t.tusTurnosEnTexto, `${r.turnos} · ${t.loQueDijisteTu}`, kb(r.bytesTurnos))}
          {/* Un interruptor de verdad (`role="switch"`), como los de Sesión e IA. */}
          <button
            type="button"
            role="switch"
            aria-checked={cuaderno.conservarMisTurnos}
            className={cuaderno.conservarMisTurnos ? "conm on" : "conm off"}
            style={{ background: "none", border: 0, padding: 0, fontFamily: "inherit", margin: "8px 2px 2px" }}
            onClick={() => void conservarMisTurnos(!cuaderno.conservarMisTurnos).then(volver)}
          >
            <span className="track" />
            <span className="etq">
              {cuaderno.conservarMisTurnos && <Ic id="i-check-circle" s relleno />}
              <span>{t.conservarMisTurnos}</span>
            </span>
          </button>
          <p style={{ ...AYUDA, marginTop: "4px" }}>
            {cuaderno.conservarMisTurnos ? t.conservarEncendido : t.conservarApagado} <b>{t.audioNunca}</b>
            {t.audioNuncaCola}
          </p>
        </div>
        <div className="tarjeta" style={{ paddingBottom: "8px" }}>
          <h2 className="seccion" style={{ margin: "0 0 6px" }}>
            {t.muereAlCerrar}
          </h2>
          {fila("i-sistema", t.vozDelCliente, t.sinInterruptor, "0 B", true)}
          {fila("i-ojo", t.susTurnos, `${cuaderno.turnosDelCliente} ${t.turnos}`, "0 B", true)}
          {fila("i-pantalla", t.loLeido, `${cuaderno.lecturas} ${t.lecturas}`, "0 B", true)}
          {fila("i-mic", t.tuPropioAudio, t.tambienMuere, "0 B", true)}
          <p style={{ ...AYUDA, marginTop: "10px" }}>{t.sinCasilla}</p>
        </div>
      </div>
      <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
        {linea && (
          <span className="mono" style={{ color: "var(--ink-2)", fontSize: "11.5px" }}>
            {linea}
          </span>
        )}
        <div className="fila" style={{ gap: "8px" }}>
          <span className="crece" />
          <button className="btn" type="button" onClick={() => void cerrarSinGuardar().then(volver)}>
            {t.cerrarSinGuardar}
          </button>
          <button
            className="btn primario"
            type="button"
            onClick={() =>
              guardarLaReunion().then(
                () => {
                  setFallo(false);
                  volver();
                },
                () => setFallo(true),
              )
            }
          >
            <Ic id="i-candado" s />
            {t.guardarCifrado}
          </button>
        </div>
      </div>
    </div>
  );
}

// ---- el archivo ------------------------------------------------------------------------------

type Pregunta = "exportar" | "borrar" | "no-exporto" | null;

function Archivo({ retencion, preguntaInicial }: { retencion: Retencion; preguntaInicial: Pregunta }) {
  const t = useT().notas;
  const idioma = useIdioma();
  const [lista, volverALista] = useReuniones();
  const [elegida, setElegida] = useState<string | null>(null);
  const [pregunta, setPregunta] = useState<Pregunta>(preguntaInicial);
  const [mia, setMia] = useState<Retencion>(retencion);
  useEffect(() => setMia(retencion), [retencion]);

  const ahora = Date.now() / 1000;
  const dias = (r: ReunionGuardada, i: number): number | null => {
    if (!hayTauri()) return DIAS_DE_MUESTRA[i] ?? null;
    return r.vence === 0 ? null : Math.max(0, Math.ceil((r.vence - ahora) / DIA));
  };
  const reunion = lista.reuniones.find((r) => r.archivo === elegida) ?? lista.reuniones[0];
  const indice = reunion ? lista.reuniones.indexOf(reunion) : -1;
  const carpeta = lista.carpeta ?? t.carpetaDeFabrica;
  const fecha = (s: number) => {
    const d = new Date(s * 1000);
    return `${String(d.getDate()).padStart(2, "0")} ${MESES[idioma][d.getMonth()]}`;
  };
  const seBorraEn = (r: ReunionGuardada, i: number) => {
    const d = dias(r, i);
    return d === null ? t.siempre : `${d} d`;
  };

  const retenciones = (
    <div className="tarjeta" style={{ padding: "10px 14px" }}>
      <h2 className="seccion" style={{ margin: "0 0 6px" }}>
        {t.cuantoViven}
      </h2>
      <div className="ventanas" role="radiogroup" aria-label={t.cuantoViven}>
        {RETENCIONES.map((r) => {
          const on = r === mia;
          const nombre = r === "1a" ? t.unAnio : r === "siempre" ? t.siempre : r.replace("d", " d");
          return (
            <button
              key={r}
              type="button"
              role="radio"
              aria-checked={on}
              className={on ? "op on" : "op"}
              onClick={() => {
                setMia(r);
                void fijarRetencion(r);
              }}
            >
              {on && <Ic id="i-check-circle" s relleno />}
              {nombre}
            </button>
          );
        })}
      </div>
      <p style={{ ...AYUDA, marginTop: "8px" }}>{t.cadaReunion}</p>
    </div>
  );

  if (!reunion) {
    return (
      <div style={COLUMNA}>
        <div className="grid-2" style={{ gridTemplateColumns: "1.25fr 1fr", gap: "12px", alignItems: "start" }}>
          <div className="tarjeta">
            <div className="fila">
              <Ic id="i-candado" s />
              <strong className="crece">{t.sinReuniones}</strong>
            </div>
            <p style={{ ...AYUDA, marginTop: "4px" }}>{t.sinReunionesDetalle}</p>
            <table className="tabla" style={{ marginTop: "8px" }}>
              <tbody>
                <tr>
                  <td>{t.donde}</td>
                  <td className="mono">{carpeta}</td>
                </tr>
                <tr>
                  <td>{t.llave}</td>
                  <td>{t.enTuLlavero}</td>
                </tr>
              </tbody>
            </table>
          </div>
          {retenciones}
        </div>
      </div>
    );
  }

  const d = dias(reunion, indice);
  const franja =
    pregunta === "exportar" ? (
      <div className="franja warn" role="alertdialog" aria-label={t.exportarATexto}>
        <Ic id="i-alert" relleno />
        <div>
          <strong>{t.exportarPregunta}</strong>
          <p>{t.exportarDetalle}</p>
          <div className="fila" style={{ gap: "8px", marginTop: "8px" }}>
            <button
              className="btn mini"
              type="button"
              onClick={() =>
                exportarReunion(reunion.archivo, idioma).then(
                  () => {
                    setPregunta(null);
                    volverALista();
                  },
                  () => setPregunta("no-exporto"),
                )
              }
            >
              <Ic id="i-doc" s />
              {t.exportarSinCifrado}
            </button>
            <button className="btn mini" type="button" onClick={() => setPregunta(null)}>
              {t.cancelar}
            </button>
          </div>
        </div>
      </div>
    ) : pregunta === "borrar" ? (
      <div className="franja err" role="alertdialog" aria-label={t.borrarAhora}>
        <Ic id="i-x-circle" relleno />
        <div>
          <strong>{t.borrarPregunta}</strong>
          <p>{t.borrarDetalle}</p>
          <div className="fila" style={{ gap: "8px", marginTop: "8px" }}>
            <button
              className="btn mini"
              type="button"
              onClick={() =>
                void borrarReunion(reunion.archivo).then(() => {
                  setPregunta(null);
                  setElegida(null);
                  volverALista();
                })
              }
            >
              <Ic id="i-basura" s />
              {t.borrar}
            </button>
            <button className="btn mini" type="button" onClick={() => setPregunta(null)}>
              {t.cancelar}
            </button>
          </div>
        </div>
      </div>
    ) : pregunta === "no-exporto" ? (
      <div className="franja err" role="alert">
        <Ic id="i-x-circle" relleno />
        <div>
          <strong>{t.noSeExporto}</strong>
          <p>{t.noSeExportoDetalle}</p>
        </div>
      </div>
    ) : (
      <div className="franja warn" role="status">
        <Ic id="i-alert" relleno />
        <div>
          <strong>{t.exportarQuita}</strong>
          <p>
            {t.exportarDetalle} {t.seAvisaAntes}
          </p>
        </div>
      </div>
    );

  return (
    <div style={COLUMNA}>
      <div className="grid-2" style={{ gridTemplateColumns: "1.25fr 1fr", gap: "12px", alignItems: "start" }}>
        <div className="tarjeta" style={{ paddingBottom: "6px" }}>
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              {reunion.archivo}
            </h2>
            <span className="estado ok">
              <Ic id="i-candado" s />
              <span>{t.cifrado}</span>
            </span>
          </div>
          <table className="tabla">
            <tbody>
              <tr>
                <td>{t.donde}</td>
                <td className="mono">{carpeta}</td>
              </tr>
              <tr>
                <td>{t.tamano}</td>
                <td className="mono">{formatearBytes(reunion.bytes, idioma)}</td>
              </tr>
              <tr>
                <td>{t.llave}</td>
                <td>{t.enTuLlavero}</td>
              </tr>
              <tr>
                <td>{t.retencion}</td>
                <td>
                  <span className="estado warn">
                    <Ic id="i-reloj" s />
                    <span>{d === null ? t.siempre : `${t.seBorraSoloEn} ${d} ${t.dias}`}</span>
                  </span>
                </td>
              </tr>
              <tr>
                <td>{t.delCliente}</td>
                <td>{t.nadaDelCliente}</td>
              </tr>
            </tbody>
          </table>
          <div className="fila" style={{ gap: "8px", marginTop: "10px" }}>
            <button className="btn" type="button" onClick={() => setPregunta("exportar")}>
              <Ic id="i-doc" s />
              {t.exportarATexto}
            </button>
            <span className="crece" />
            <button className="btn" type="button" onClick={() => setPregunta("borrar")}>
              <Ic id="i-basura" s />
              {t.borrarAhora}
            </button>
          </div>
        </div>
        <div style={COLUMNA}>
          {franja}
          {retenciones}
        </div>
      </div>
      <div className="tarjeta" style={{ paddingBottom: "6px" }}>
        <h2 className="seccion" style={{ margin: "0 0 4px" }}>
          {t.reunionesGuardadas}
        </h2>
        <table className="tabla">
          <thead>
            <tr>
              <th>{t.colArchivo}</th>
              <th>{t.colFecha}</th>
              <th className="num">{t.colTamano}</th>
              <th className="num">{t.colSeBorraEn}</th>
            </tr>
          </thead>
          <tbody>
            {lista.reuniones.map((r, i) => {
              const quedan = dias(r, i);
              return (
                <tr key={r.archivo} className={quedan !== null && quedan <= A_PUNTO_DE_IRSE ? "apagada" : undefined}>
                  <td className="mono">
                    <button
                      className="elegir"
                      type="button"
                      aria-current={r.archivo === reunion.archivo ? "true" : undefined}
                      onClick={() => {
                        setElegida(r.archivo);
                        setPregunta(null);
                      }}
                    >
                      {r.archivo}
                    </button>
                  </td>
                  <td className="mono">{fecha(r.guardada)}</td>
                  <td className="num">{formatearBytes(r.bytes, idioma)}</td>
                  <td className="num">{seBorraEn(r, i)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
