import { useEffect, useRef, useState } from "react";
import { useIdioma, useT } from "../i18n";
import { nombreDeLaRetencion, useCuaderno, useMuestraDelCuaderno } from "../notas";
import { Ic } from "../componentes/Iconos";
import type { Seccion } from "../componentes/Ventana";
import { hayTauri } from "../puente";
import {
  borrarLosEnsayos,
  cerrarElEnsayo,
  diaCorto,
  empezarElEnsayo,
  ensayoListo,
  ensayoRepetir,
  ensayoSaltar,
  ensayoSiLoDije,
  ensayoTerminar,
  entreComillas,
  esNoEmpezo,
  estadoDeLaUrl,
  exportarElEnsayo,
  flecha,
  guardarElEnsayo,
  progresoDelEnsayo,
  reloj,
  useEnsayo,
  usePreparacion,
  usePreparacionDeMuestra,
  useProgresoDeMuestra,
  useVistaDeMuestra,
  type Cambio,
  type EstadoDelBanco,
  type Evidencia,
  type Muletilla,
  type NoEmpezo,
  type Preparacion,
  type PreguntaEnPantalla,
  type Progreso,
  type VistaDelEnsayo,
} from "../ensayo";

/**
 * EL ENSAYO (C18, sprint 004, ADR 019) — `docs/diseno/ensayo.html`, aprobada en la mirada de DECISIÓN 1
 * («La abrí y la apruebo la pantalla ensayo»).
 *
 * Pinta y conmuta, nada más: el banco, la sesión, el oído y las cifras viven en Rust. Las teclas son
 * **de esta ventana**, no globales —Enter listo, R repetir, S saltar, Esc terminar—, y ⌥⎋ lo corta todo
 * como siempre.
 *
 * Fuera de Tauri dibuja el estado de la maqueta que pide la URL (`?pantalla=ensayo&estado=…`): es lo que
 * el gate de fidelidad fotografía.
 */
/**
 * Lo elegido en «preparar» (auditoría del S4, M13): vive aquí y no en `Preparar`, que se desmonta al ver tu
 * progreso y durante el ensayo. Volver te deja con el mismo cliente, la misma propuesta, el mismo tope y la
 * voz como la dejaste. `tope` en `null` es el de fábrica, que dice el catálogo (M18).
 */
export type Eleccion = {
  cliente: string | null;
  propuesta: string | null;
  tope: number | null;
  voz: boolean;
};

export function Ensayo({
  busqueda = "",
  ir,
}: {
  busqueda?: string;
  ir: (s: Seccion) => void;
}) {
  const t = useT().ensayo;
  const enTauri = hayTauri();
  const [eleccion, setEleccion] = useState<Eleccion>({
    cliente: null,
    propuesta: null,
    tope: null,
    voz: true,
  });
  const muestra = estadoDeLaUrl(busqueda);
  const [real] = useEnsayo();
  const deMuestra = useVistaDeMuestra(muestra);
  const vista = enTauri ? real : deMuestra;
  // Guardado, el informe se va y vuelves a «preparar» con la franja que lo dice (ADR 015, enmienda 4).
  const [guardado, setGuardado] = useState(
    () => !enTauri && muestra === "guardado",
  );
  // Tu progreso, abierto con el desbloqueo de tus notas: se enseña en lugar de «preparar».
  const deMuestraDelProgreso = useProgresoDeMuestra(muestra);
  const [abierto, setAbierto] = useState<{
    progreso: Progreso;
    clientes: string[];
  } | null>(null);
  const m = t.muestra;
  const progreso = enTauri
    ? abierto
    : deMuestraDelProgreso && {
        progreso: deMuestraDelProgreso,
        clientes: [m.paramo, m.surDelValle],
      };

  return (
    <>
      <div className="titulo">
        <h1>{t.titulo}</h1>
        <p className="sub">{t.sub}</p>
      </div>
      {vista ? (
        <EnCurso vista={vista} alGuardar={() => setGuardado(true)} />
      ) : progreso ? (
        <TuProgreso
          inicial={progreso.progreso}
          clientes={progreso.clientes}
          preguntarAntes={!enTauri && muestra === "progreso-borrar"}
          volver={(cliente) => {
            setAbierto(null);
            // El cliente que mirabas en tu progreso es el que queda elegido; su propuesta, si es el mismo.
            setEleccion((e) =>
              cliente === e.cliente ? e : { ...e, cliente, propuesta: null },
            );
          }}
        />
      ) : (
        <Preparar
          busqueda={busqueda}
          ir={ir}
          eleccion={eleccion}
          elegir={(cambio) => setEleccion((e) => ({ ...e, ...cambio }))}
          guardado={guardado}
          alEmpezar={() => setGuardado(false)}
          abrirProgreso={(p, clientes) => {
            setGuardado(false);
            setAbierto({ progreso: p, clientes });
          }}
        />
      )}
    </>
  );
}

// ─── 1 · preparar · 1b · no empezó · 7 · sin corpus ───────────────────────────────────────────────

function Preparar({
  busqueda,
  ir,
  eleccion,
  elegir,
  guardado,
  alEmpezar,
  abrirProgreso,
}: {
  busqueda: string;
  ir: (s: Seccion) => void;
  eleccion: Eleccion;
  elegir: (cambio: Partial<Eleccion>) => void;
  /** Acabas de guardar un ensayo: la franja lo dice. */
  guardado: boolean;
  alEmpezar: () => void;
  abrirProgreso: (p: Progreso, clientes: string[]) => void;
}) {
  const t = useT().ensayo;
  const idioma = useIdioma();
  const enTauri = hayTauri();
  const [noSeAbrieron, setNoSeAbrieron] = useState(false);
  const muestra = estadoDeLaUrl(busqueda);
  const { cliente, propuesta, tope, voz } = eleccion;
  const [noEmpezo, setNoEmpezo] = useState<NoEmpezo | null>(() =>
    !enTauri && muestra === "no-empezo"
      ? { que: "microfono", porque: "sin-permiso-del-microfono" }
      : null,
  );
  const [empezando, setEmpezando] = useState(false);
  const real = usePreparacion(cliente, propuesta, tope);
  const deMuestra = usePreparacionDeMuestra(muestra);
  const prep = enTauri ? real : deMuestra;
  if (!prep) return null;

  const elegirCliente = (c: string) => {
    elegir({ cliente: c, propuesta: null });
    setNoEmpezo(null);
  };

  // Sin preguntas para este cliente, aunque lo diga `empezar` y no «preparar» (auditoría del S4, B26).
  if (prep.sinCorpus || noEmpezo?.que === "sin-corpus")
    return <SinCorpus prep={prep} elegir={elegirCliente} ir={ir} />;

  const verProgreso = () => {
    if (!prep.cliente) return;
    setNoSeAbrieron(false);
    // Pide el desbloqueo de tus notas, una vez por sesión de la app; cancelarlo deja todo en su sitio.
    progresoDelEnsayo(prep.cliente, idioma)
      .then((p) => {
        if (p) abrirProgreso(p, prep.clientes);
      })
      .catch(() => setNoSeAbrieron(true));
  };

  const empezar = () => {
    if (!prep.cliente || empezando) return;
    setEmpezando(true);
    setNoEmpezo(null);
    alEmpezar();
    empezarElEnsayo(prep.cliente, prep.propuesta, prep.tope, voz)
      .catch((e: unknown) =>
        setNoEmpezo(
          esNoEmpezo(e) ? e : { que: "microfono", porque: "no-dejo" },
        ),
      )
      .finally(() => setEmpezando(false));
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      {guardado && (
        <div className="franja ok" role="status">
          <Ic id="i-check-circle" s relleno />
          <div>
            <strong>{t.guardadoConTusNotas}</strong>
          </div>
        </div>
      )}
      {noSeAbrieron && <NoSeAbrieron />}
      {noEmpezo && <PorQueNoEmpezo no={noEmpezo} />}
      {!prep.transcribe && (
        <div className="franja warn">
          <Ic id="i-globo" s />
          <div>
            <strong>{t.sinTranscripcion}</strong>
            <p>{t.sinTranscripcionQue}</p>
          </div>
        </div>
      )}
      <div
        className="grid-2"
        style={{
          gridTemplateColumns: "1.1fr 1fr",
          gap: 12,
          alignItems: "start",
        }}
      >
        <div
          className="tarjeta"
          style={{ display: "flex", flexDirection: "column", gap: 9 }}
        >
          <h2 className="seccion" style={{ margin: 0 }}>
            {t.conQuien}
          </h2>
          <div className="fila">
            <span className="crece">{t.cliente}</span>
            <SelectorDeCliente prep={prep} elegir={elegirCliente} />
          </div>
          {/* Con uno o más guardados —contados por el nombre del archivo, sin abrir ninguno—, el camino a tu
              progreso con ese cliente (FORMA de la fase 4, maquetada, no vista). */}
          {prep.guardados > 0 && (
            <div className="fila">
              <span className="crece">{t.guardados}</span>
              <span className="mono">{prep.guardados}</span>
              <button type="button" className="btn mini" onClick={verProgreso}>
                <Ic id="i-ensayo" s />
                {t.verTuProgreso}
              </button>
            </div>
          )}
          {/* Un cliente con solo su ficha no tiene propuesta que elegir (auditoría del S4, B39). */}
          {prep.propuestas.length > 0 && (
            <div className="fila">
              <span className="crece">{t.propuesta}</span>
              <select
                className="selector"
                aria-label={t.propuesta}
                value={prep.propuesta ?? ""}
                onChange={(e) => elegir({ propuesta: e.target.value || null })}
              >
                {prep.propuestas.map((p) => (
                  <option key={p.ruta} value={p.ruta}>
                    {p.nombre}
                  </option>
                ))}
              </select>
            </div>
          )}
          <div className="fila">
            <span className="crece">{t.preguntas}</span>
            <div
              className="ventanas"
              role="radiogroup"
              aria-label={t.preguntas}
            >
              {prep.topes.map((n) => {
                const on = n === prep.tope;
                return (
                  <button
                    key={n}
                    type="button"
                    role="radio"
                    aria-checked={on}
                    className={on ? "op on" : "op"}
                    onClick={() => elegir({ tope: n })}
                  >
                    {on && <Ic id="i-check-circle" s relleno />}
                    {n}
                  </button>
                );
              })}
            </div>
          </div>
          <button
            type="button"
            role="switch"
            aria-checked={voz}
            className={voz ? "conm on" : "conm off"}
            style={{
              background: "none",
              border: 0,
              padding: 0,
              fontFamily: "inherit",
              alignSelf: "flex-start",
            }}
            onClick={() => elegir({ voz: !voz })}
          >
            <span className="track"></span>
            <span className="etq">{t.leerEnVozAlta}</span>
          </button>
          <p className="ayuda-e">{t.vozSinReunion}</p>
          <p className="ayuda-e">{t.idiomaDelEnsayo[prep.idioma]}</p>
        </div>
        <div className="tarjeta" style={{ paddingBottom: 8 }}>
          <h2 className="seccion" style={{ margin: "0 0 4px" }}>
            {t.deDonde}
          </h2>
          <div style={{ margin: "0 -4px" }}>
            <div className="buffer">
              <Ic id="i-doc" s />
              <span className="que">{t.tuPropuesta}</span>
              <span className="donde">{t.tuPropuestaQue}</span>
              <span className="cuanto">{prep.cuentas.propuesta}</span>
            </div>
            <div className="buffer">
              <Ic id="i-nota" s />
              <span className="que">{t.suFicha}</span>
              <span className="donde">{t.suFichaQue}</span>
              <span className="cuanto">{prep.cuentas.ficha}</span>
            </div>
            <div className="buffer">
              <Ic id="i-alert" s />
              <span className="que">{t.objeciones}</span>
              <span className="donde">{t.objecionesQue}</span>
              <span className="cuanto">{prep.cuentas.objeciones}</span>
            </div>
          </div>
          <div className="franja mute" style={{ marginTop: 8 }}>
            <Ic id="i-chispa" s />
            <div>
              <strong>
                {prep.enriquecer ? t.enriquecerEncendido : t.enriquecerApagado}
              </strong>
              <p>
                {prep.enriquecer
                  ? t.enriquecerEncendidoQue
                  : t.enriquecerApagadoQue}
              </p>
            </div>
          </div>
        </div>
      </div>
      <div className="fila" style={{ gap: 12 }}>
        <button
          type="button"
          className="btn primario"
          onClick={empezar}
          disabled={empezando}
        >
          <Ic id="i-mic" s />
          {t.empezar}
        </button>
        <p className="crece ayuda-e">
          <b>{t.soloTuMicrofono}</b> {t.soloTuMicrofonoQue}
        </p>
      </div>
    </div>
  );
}

function SelectorDeCliente({
  prep,
  elegir,
}: {
  prep: Preparacion;
  elegir: (c: string) => void;
}) {
  const t = useT().ensayo;
  return (
    <select
      className="selector"
      aria-label={t.cliente}
      value={prep.cliente ?? ""}
      onChange={(e) => elegir(e.target.value)}
    >
      {prep.clientes.map((c) => (
        <option key={c} value={c}>
          {c}
        </option>
      ))}
    </select>
  );
}

function PorQueNoEmpezo({ no }: { no: NoEmpezo }) {
  const t = useT().ensayo;
  // Con una videollamada abierta y el sonido por altavoces, no se ensaya (auditoría del S4, A1).
  if (no.que === "videollamada" || no.que === "no-se-sabe-si-hay-llamada") {
    const sabe = no.que === "videollamada";
    return (
      <div className="franja warn" role="alert">
        <Ic id="i-auriculares" s />
        <div>
          <strong>{sabe ? t.hayVideollamada : t.noSeSabeSiHayLlamada}</strong>
          <p>{sabe ? t.hayVideollamadaQue : t.noSeSabeSiHayLlamadaQue}</p>
        </div>
      </div>
    );
  }
  if (no.que === "en-reunion") {
    return (
      <div className="franja warn" role="alert">
        <Ic id="i-video" s />
        <div>
          <strong>{t.hayReunion}</strong>
          <p>{t.hayReunionQue}</p>
        </div>
      </div>
    );
  }
  const sinPermiso =
    no.que === "microfono" && no.porque === "sin-permiso-del-microfono";
  return (
    <div className="franja err" role="alert">
      <Ic id="i-mic-off" s />
      <div>
        <strong>{t.microfonoNoAbrio}</strong>
        <p>{sinPermiso ? t.microfonoSinPermiso : t.microfonoOtro}</p>
      </div>
    </div>
  );
}

/** El desbloqueo se canceló, o tus ensayos no se dejaron abrir: siguen cifrados y en su sitio. */
function NoSeAbrieron() {
  const t = useT().ensayo;
  return (
    <div className="franja warn" role="status">
      <Ic id="i-candado" s />
      <div>
        <strong>{t.noSeAbrieron}</strong>
        <p>{t.noSeAbrieronQue}</p>
      </div>
    </div>
  );
}

// ─── 6 · tu progreso · 6b · borrar ───────────────────────────────────────────────────────────────

/** «↑ evidencia 9 → 14», «↓ ritmo 161 → 138 ppm»: hacia dónde fue, sin decir si es bueno o malo. La
 *  unidad va una vez, al final, como en la maqueta. */
function Tendencia({
  que,
  c,
  como = String,
  unidad,
}: {
  que: string;
  c: Cambio;
  como?: (n: number) => string;
  unidad?: string;
}) {
  return (
    <span className="estado mute">
      <span>
        {`${flecha(c)} ${que} ${como(c.desde)} → ${como(c.hasta)}${unidad ? ` ${unidad}` : ""}`}
      </span>
    </span>
  );
}

function TuProgreso({
  inicial,
  clientes,
  preguntarAntes,
  volver,
}: {
  inicial: Progreso;
  clientes: string[];
  /** La muestra de «6b · borrar»: la pregunta ya abierta. */
  preguntarAntes: boolean;
  /** Vuelve a «preparar» con el cliente que mirabas (auditoría del S4, M13). */
  volver: (cliente: string) => void;
}) {
  const t = useT().ensayo;
  const idioma = useIdioma();
  const [p, setP] = useState(inicial);
  const [borrando, setBorrando] = useState(preguntarAntes);
  const [noSeAbrieron, setNoSeAbrieron] = useState(false);
  const [noSeBorraron, setNoSeBorraron] = useState(false);
  // **El foco va y vuelve con la pregunta** (auditoría del S4, M15): al abrirla, a «Cancelar», que es lo
  // seguro; al cerrarla, al botón que la abrió. La muestra de la maqueta la abre al montarse y no roba el foco.
  const abrirBorrar = useRef<HTMLButtonElement>(null);
  const cancelarBorrar = useRef<HTMLButtonElement>(null);
  const montado = useRef(false);
  useEffect(() => {
    if (!montado.current) {
      montado.current = true;
      return;
    }
    (borrando ? cancelarBorrar : abrirBorrar).current?.focus();
  }, [borrando]);
  const d = p.desdeElPrimero;
  const hayCambio = Boolean(d.evidencia || d.ritmo || d.muletillas || d.tiempo);

  // Otro cliente: el desbloqueo ya se hizo en esta sesión de la app, así que no vuelve a preguntar.
  const elegir = (cliente: string) => {
    setNoSeAbrieron(false);
    setNoSeBorraron(false);
    setBorrando(false);
    progresoDelEnsayo(cliente, idioma)
      .then((nuevo) => {
        if (nuevo) setP(nuevo);
      })
      .catch(() => setNoSeAbrieron(true));
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      {noSeAbrieron && <NoSeAbrieron />}
      {noSeBorraron && (
        <div className="franja err" role="alert">
          <Ic id="i-x-circle" s relleno />
          <div>
            <strong>{t.noSeBorraron}</strong>
            <p>{t.noSeBorraronQue}</p>
          </div>
        </div>
      )}
      <div className="tarjeta" style={{ padding: "10px 12px 8px" }}>
        <div className="fila" style={{ marginBottom: 4 }}>
          <h2 className="seccion crece" style={{ margin: 0 }}>
            {t.tuProgreso}
          </h2>
          <select
            className="selector"
            aria-label={t.cliente}
            value={p.cliente}
            onChange={(e) => elegir(e.target.value)}
          >
            {clientes.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
        </div>
        {p.filas.length === 0 ? (
          <p className="ayuda-e">{t.ningunoGuardado}</p>
        ) : (
          <table className="tabla">
            <thead>
              <tr>
                <th>{t.colEnsayo}</th>
                <th className="num">{t.evidenciaUsada}</th>
                <th className="num">{t.ritmo}</th>
                <th className="num">{t.muletillas}</th>
                <th className="num">{t.tiempoMedio}</th>
              </tr>
            </thead>
            <tbody>
              {/* Dos ensayos en el mismo minuto tienen el mismo `empezo` (auditoría del S4, B31). */}
              {p.filas.map((f, i) => (
                <tr key={`${i}·${f.empezo}`}>
                  <td className="mono">{diaCorto(f.empezo, t.meses, idioma)}</td>
                  <td className="num">
                    {f.evidencia === 0 ? "—" : `${f.citadas} ${t.de} ${f.evidencia}`}
                  </td>
                  <td className="num">{f.ppmMedio ?? "—"}</td>
                  <td className="num">{f.muletillas ?? "—"}</td>
                  <td className="num">
                    {f.tiempoMedioMs === null ? "—" : reloj(f.tiempoMedioMs)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
        {p.antes > 0 && (
          <p className="ayuda-e tras">
            {t.yMas} {p.antes} {t.mas}
          </p>
        )}
      </div>
      {hayCambio && (
        <div className="fila" style={{ gap: 8, flexWrap: "wrap" }}>
          <span className="ayuda-e rotulo">{t.desdeElPrimero}</span>
          {d.evidencia && <Tendencia que={t.chipEvidencia} c={d.evidencia} />}
          {d.ritmo && <Tendencia que={t.chipRitmo} c={d.ritmo} unidad={t.ppm} />}
          {d.muletillas && <Tendencia que={t.chipMuletillas} c={d.muletillas} />}
          {d.tiempo && <Tendencia que={t.chipTiempo} c={d.tiempo} como={reloj} />}
        </div>
      )}
      <p className="ayuda-e">{t.soloCifras}</p>
      {borrando ? (
        <div className="franja err" role="alertdialog" aria-label={t.borrarLosDeEsteCliente}>
          <Ic id="i-x-circle" s relleno />
          <div>
            <strong>{t.borrarPregunta}</strong>
            <p>{t.borrarDetalle}</p>
            <div className="fila" style={{ gap: 8, marginTop: 8 }}>
              <button
                type="button"
                className="btn mini"
                onClick={() =>
                  // Si falla, la pantalla lo dice: siguen cifrados y en su sitio (auditoría del S4, M14).
                  void borrarLosEnsayos(p.cliente).then(
                    () => volver(p.cliente),
                    () => {
                      setBorrando(false);
                      setNoSeBorraron(true);
                    },
                  )
                }
              >
                <Ic id="i-basura" s />
                {t.borrar}
              </button>
              <button
                ref={cancelarBorrar}
                type="button"
                className="btn mini"
                onClick={() => setBorrando(false)}
              >
                {t.cancelar}
              </button>
            </div>
          </div>
        </div>
      ) : (
        <div className="fila" style={{ gap: 8 }}>
          <button type="button" className="btn" onClick={() => volver(p.cliente)}>
            {t.volver}
          </button>
          {p.filas.length > 0 && (
            <button
              ref={abrirBorrar}
              type="button"
              className="btn"
              onClick={() => {
                setNoSeBorraron(false);
                setBorrando(true);
              }}
            >
              <Ic id="i-basura" s />
              {t.borrarLosDeEsteCliente}
            </button>
          )}
        </div>
      )}
    </div>
  );
}

function SinCorpus({
  prep,
  elegir,
  ir,
}: {
  prep: Preparacion;
  elegir: (c: string) => void;
  ir: (s: Seccion) => void;
}) {
  const t = useT().ensayo;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <div className="tarjeta">
        <div className="fila">
          <h2 className="seccion crece" style={{ margin: 0 }}>
            {t.conQuien}
          </h2>
          {prep.clientes.length > 0 && (
            <SelectorDeCliente prep={prep} elegir={elegir} />
          )}
        </div>
      </div>
      <div className="franja mute">
        <Ic id="i-doc" s />
        <div>
          <strong>{t.nadaDeEsteCliente}</strong>
          <p>{t.nadaDeEsteClienteQue}</p>
        </div>
      </div>
      <div className="fila">
        <button type="button" className="btn" onClick={() => ir("corpus")}>
          <Ic id="i-doc" s />
          {t.irACorpus}
        </button>
      </div>
    </div>
  );
}

// ─── 2 · preguntando · 3 · respondiendo · 4 · evaluada · 5 · el informe ───────────────────────────

/** **Las teclas de esta ventana** (ADR 019 §6.4). No globales: con el foco en otra app no hacen nada. */
function useTeclas(vista: VistaDelEnsayo) {
  useEffect(() => {
    if (vista.fase === "cerrado") return;
    const alPulsar = (e: KeyboardEvent) => {
      // Una tecla sostenida repite: una orden por pulsación, no una por repetición (auditoría del S4, M11).
      if (e.repeat || e.isComposing || e.metaKey || e.ctrlKey || e.altKey) return;
      // El objetivo puede no ser un elemento (la ventana, el documento): entonces no es ni un campo ni un botón.
      const objetivo = e.target instanceof HTMLElement ? e.target : null;
      if (
        objetivo &&
        ["INPUT", "SELECT", "TEXTAREA"].includes(objetivo.tagName)
      )
        return;
      // Enter sobre un botón o un enlace ya lo pulsa él: ni se pulsa dos veces, ni se le roba la navegación
      // a un enlace del rail (auditoría del S4, M16).
      if (e.key === "Enter" && objetivo?.closest("a, button")) return;
      const tecla = e.key.toLowerCase();
      if (e.key === "Enter") void ensayoListo();
      else if (e.key === "Escape") void ensayoTerminar();
      else if (tecla === "r" && vista.fase !== "evaluada") void ensayoRepetir();
      else if (tecla === "s" && vista.fase !== "evaluada") void ensayoSaltar();
      else return;
      e.preventDefault();
    };
    globalThis.addEventListener("keydown", alPulsar);
    return () => globalThis.removeEventListener("keydown", alPulsar);
  }, [vista.fase]);
}

function EnCurso({
  vista,
  alGuardar,
}: {
  vista: VistaDelEnsayo;
  alGuardar: () => void;
}) {
  useTeclas(vista);
  switch (vista.fase) {
    case "preguntando":
      return <Preguntando vista={vista} />;
    case "respondiendo":
      return <Respondiendo vista={vista} />;
    case "evaluada":
      return <Evaluada vista={vista} />;
    case "cerrado":
      return <ElInforme vista={vista} alGuardar={alGuardar} />;
  }
}

function Progreso({ vista }: { vista: VistaDelEnsayo }) {
  const t = useT().ensayo;
  const ancho = vista.total > 0 ? ((vista.indice + 1) / vista.total) * 100 : 0;
  return (
    <>
      <div className="progreso">
        <span>
          {t.pregunta} {vista.indice + 1} {t.de} {vista.total}
        </span>
        <span className="pista">
          <i style={{ width: `${ancho}%` }}></i>
        </span>
        <span>{vista.cliente}</span>
      </div>
      <LineaDelModelo banco={vista.banco} />
    </>
  );
}

/** «Enriquecer el banco» en una línea (ADR 019 §4). Con el interruptor apagado, nada. */
function LineaDelModelo({ banco }: { banco: EstadoDelBanco }) {
  const t = useT().ensayo;
  let texto: string | null = null;
  if (banco.que === "en-camino") texto = t.bancoEnCamino;
  else if (banco.que === "sumadas")
    texto =
      banco.cuantas === 1
        ? t.sumoUna
        : `${t.sumoAntes} ${banco.cuantas} ${t.sumoDespues}`;
  else if (banco.que === "no-se-enriquecio" && banco.porque !== "apagado")
    texto = `${t.noSumo} ${t.porQueNoSumo[banco.porque]}`;
  if (!texto) return null;
  return (
    <p className="ayuda-e pegada">{texto}</p>
  );
}

function LaPregunta({
  pregunta,
  chica,
}: {
  pregunta: PreguntaEnPantalla | null;
  chica?: boolean;
}) {
  const t = useT().ensayo;
  if (!pregunta) return null;
  const donde = pregunta.seccion ?? pregunta.fuente;
  return (
    <div className={chica ? "pregunta-e chica" : "pregunta-e"}>
      <div className="de">
        <span className="unidad-chip">{t.chips[pregunta.de]}</span>
        {donde && <span>{donde}</span>}
      </div>
      <p className="texto">{pregunta.texto}</p>
    </div>
  );
}

function Tecla({ k, texto }: { k: string; texto: string }) {
  return (
    <span className="tecla">
      <kbd>{k}</kbd> {texto}
    </span>
  );
}

function Preguntando({ vista }: { vista: VistaDelEnsayo }) {
  const t = useT().ensayo;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <Progreso vista={vista} />
      <LaPregunta pregunta={vista.pregunta} />
      <div className="fila" style={{ gap: 8 }}>
        {vista.leyendo && (
          <span className="estado halo">
            <Ic id="i-voz" s />
            <span>{t.teLaEstaLeyendo}</span>
          </span>
        )}
        <span className="estado mute">
          <Ic id="i-mic-off" s />
          <span>{t.microfonoEspera}</span>
        </span>
      </div>
      <div className="fila" style={{ gap: 14 }}>
        <Tecla k="R" texto={t.repetir} />
        <Tecla k="S" texto={t.saltar} />
        <Tecla k="Esc" texto={t.terminar} />
      </div>
    </div>
  );
}

/** El reloj de tu respuesta: lo que dijo Rust al pintar, y desde ahí corre aquí. */
function useReloj(desdeMs: number, corre: boolean): number {
  const [ahora, setAhora] = useState(() => ({
    base: desdeMs,
    en: Date.now(),
    ms: desdeMs,
  }));
  useEffect(() => {
    setAhora({ base: desdeMs, en: Date.now(), ms: desdeMs });
    if (!corre || !hayTauri()) return;
    const cada = setInterval(
      () => setAhora((a) => ({ ...a, ms: a.base + (Date.now() - a.en) })),
      250,
    );
    return () => clearInterval(cada);
  }, [desdeMs, corre]);
  return ahora.ms;
}

function Respondiendo({ vista }: { vista: VistaDelEnsayo }) {
  const t = useT().ensayo;
  const ms = useReloj(vista.transcurridoMs, !vista.cerrando);
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
      <Progreso vista={vista} />
      <LaPregunta pregunta={vista.pregunta} chica />
      <div
        className="tarjeta"
        style={{ display: "flex", flexDirection: "column", gap: 8 }}
      >
        <div className="fila">
          <span className="estado ok">
            <Ic id="i-mic" s />
            <span>{t.teEscucho}</span>
          </span>
          <span className="crece"></span>
          <span className="reloj-e" role="timer" aria-live="off">
            <Ic id="i-reloj" s />
            {reloj(ms)}
          </span>
        </div>
        <h2 className="seccion" style={{ margin: 0 }}>
          {t.tuRespuesta}
        </h2>
        <p className="respuesta-e">{vista.respuesta}</p>
      </div>
      <div className="fila" style={{ gap: 14 }}>
        <Tecla k="↵" texto={t.listo} />
        <Tecla k="R" texto={t.repetir} />
        <Tecla k="S" texto={t.saltar} />
        <Tecla k="Esc" texto={t.terminar} />
        <span className="crece"></span>
        <span className="ayuda-e">{t.seCierra}</span>
      </div>
    </div>
  );
}

function fuenteDe(e: Evidencia, unidades: Record<string, string>): string {
  const u = e.fuente.unidad ? unidades[e.fuente.unidad] : e.fuente.documento;
  return e.fuente.seccion ? `${u} · ${e.fuente.seccion}` : u;
}

function detalleDe(muletillas: Muletilla[], idioma: string): string {
  return muletillas
    .map((m) => `${entreComillas(m.frase, idioma)} ×${m.veces}`)
    .join(" · ");
}

/** La sección que el lector de PDF conjeturó, marcada en la evidencia como en la banda (auditoría del S4, B33). */
function Conjetura({ ev }: { ev: Evidencia }) {
  const tb = useT().banda;
  if (!ev.fuente.conjeturada) return null;
  return (
    <span className="conjetura">
      <Ic id="i-half" s />
      {tb.seccionConjeturada}
    </span>
  );
}

function Evaluada({ vista }: { vista: VistaDelEnsayo }) {
  const t = useT().ensayo;
  const unidades = useT().banda.unidades;
  const idioma = useIdioma();
  const e = vista.evaluacion;
  if (!e) return null;
  const conIndice = e.evidencia.map((ev, i) => ({ ev, i }));
  // En qué lista va cada ficha es cosa de la pantalla; las cuentas, de Rust (auditoría del S4, M12).
  const usadas = conIndice.filter(({ ev }) => ev.citada || ev.dichaPorTi);
  const sinUsar = conIndice.filter(({ ev }) => !ev.citada && !ev.dichaPorTi);
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      <Progreso vista={vista} />
      <LaPregunta pregunta={vista.pregunta} chica />
      <div className="grid-2" style={{ gap: 10, alignItems: "start" }}>
        <div className="tarjeta" style={{ padding: "10px 12px" }}>
          <h2 className="seccion" style={{ margin: "0 0 7px" }}>
            {t.usaste}
          </h2>
          <ul className="evidencia-e">
            {usadas.map(({ ev, i }) => (
              <li key={i} className="usada">
                <Ic id="i-check-circle" s relleno />
                <span className="titular">{ev.titular}</span>
                {ev.dichaPorTi ? (
                  <button
                    type="button"
                    className="btn mini"
                    aria-pressed="true"
                    onClick={() => void ensayoSiLoDije(i)}
                  >
                    {t.siLoDije}
                  </button>
                ) : (
                  <span></span>
                )}
                <span className="fuente">{fuenteDe(ev, unidades)}</span>
                <Conjetura ev={ev} />
              </li>
            ))}
          </ul>
        </div>
        <div className="tarjeta" style={{ padding: "10px 12px" }}>
          <h2 className="seccion" style={{ margin: "0 0 7px" }}>
            {t.teniasYNo}
          </h2>
          <ul className="evidencia-e">
            {sinUsar.map(({ ev, i }) => (
              <li key={i} className="sin-usar">
                <Ic id="i-pendiente" s />
                <span className="titular">{ev.titular}</span>
                <button
                  type="button"
                  className="btn mini"
                  aria-pressed="false"
                  onClick={() => void ensayoSiLoDije(i)}
                >
                  {t.siLoDije}
                </button>
                <span className="fuente">{fuenteDe(ev, unidades)}</span>
                <Conjetura ev={ev} />
              </li>
            ))}
          </ul>
        </div>
      </div>
      <div className="cifras-e">
        <Cifra q={t.tiempo} n={reloj(e.tiempoMs)} />
        <Cifra q={t.ritmo} n={e.ppm === null ? "—" : `${e.ppm} ${t.ppm}`} />
        <Cifra
          q={t.muletillas}
          n={vista.muletillas === null ? "—" : String(vista.muletillas)}
          d={vista.muletillas ? detalleDe(e.muletillas, idioma) : undefined}
        />
        <Cifra
          q={t.evidencia}
          n={
            e.evidencia.length === 0
              ? "—"
              : `${vista.usadas} ${t.de} ${e.evidencia.length}`
          }
        />
      </div>
      <div className="fila" style={{ gap: 12 }}>
        <button
          type="button"
          className="btn primario"
          onClick={() => void ensayoListo()}
        >
          {t.siguiente}
        </button>
        <span className="tecla">
          <kbd>↵</kbd>
        </span>
        <p className="crece ayuda-e">{t.sinPuntaje}</p>
      </div>
    </div>
  );
}

function Cifra({ q, n, d }: { q: string; n: string; d?: string }) {
  return (
    <div className="cifra-e">
      <div className="q">{q}</div>
      <div className="n">{n}</div>
      {d && <div className="d">{d}</div>}
    </div>
  );
}

/** Cuántas filas enseña el informe antes de «y N más»: las de la maqueta aprobada. */
const FILAS_A_LA_VISTA = 4;

function ElInforme({
  vista,
  alGuardar,
}: {
  vista: VistaDelEnsayo;
  alGuardar: () => void;
}) {
  const t = useT().ensayo;
  const tn = useT().notas;
  const idioma = useIdioma();
  // La retención es la de tus notas (ADR 015, enmienda 4): la misma que Notas enseña.
  const [cuaderno] = useCuaderno(useMuestraDelCuaderno("archivo"));
  // Sin la respuesta del cuaderno no se sabe tu retención, y no se inventa (auditoría del S4, B30).
  const retencion = cuaderno?.retencion ?? null;
  const [ocupado, setOcupado] = useState(false);
  const [fallo, setFallo] = useState<"guardar" | "exportar" | null>(null);
  const i = vista.informe;
  if (!i) return null;
  const resto = i.filas.length - FILAS_A_LA_VISTA;

  // Guardar no pide nada; si falla, el informe sigue aquí, entero, y se puede volver a intentar.
  const guardar = () => {
    setOcupado(true);
    setFallo(null);
    guardarElEnsayo()
      .then(alGuardar)
      .catch(() => setFallo("guardar"))
      .finally(() => setOcupado(false));
  };
  // Exportar pide el desbloqueo de tus notas y dónde; cancelar el diálogo no es un fallo.
  const exportar = () => {
    setOcupado(true);
    setFallo(null);
    exportarElEnsayo(idioma)
      .catch(() => setFallo("exportar"))
      .finally(() => setOcupado(false));
  };

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
      {fallo && (
        <div className="franja err" role="alert">
          <Ic id="i-x-circle" s relleno />
          <div>
            <strong>{fallo === "guardar" ? t.noSeGuardo : t.noSeExporto}</strong>
            <p>{fallo === "guardar" ? t.noSeGuardoQue : t.noSeExportoQue}</p>
          </div>
        </div>
      )}
      <div className="franja ok">
        <Ic id="i-check-circle" s relleno />
        <div>
          <strong>
            {t.terminado} {i.respondidas}{" "}
            {i.respondidas === 1 ? t.respondida : t.respondidas}, {i.saltadas}{" "}
            {i.saltadas === 1 ? t.saltada : t.saltadas}
          </strong>
        </div>
      </div>
      <div className="cifras-e">
        <Cifra
          q={t.evidenciaUsada}
          n={i.evidencia === 0 ? "—" : `${i.citadas} ${t.de} ${i.evidencia}`}
        />
        <Cifra
          q={t.ritmoMedio}
          n={i.ppmMedio === null ? "—" : `${i.ppmMedio} ${t.ppm}`}
        />
        <Cifra
          q={t.muletillas}
          n={i.muletillas === null ? "—" : String(i.muletillas)}
          d={
            i.laQueMas
              ? `${t.laQueMas} ${entreComillas(i.laQueMas.frase, idioma)} ×${i.laQueMas.veces}`
              : undefined
          }
        />
        <Cifra
          q={t.tiempoMedio}
          n={i.tiempoMedioMs === null ? "—" : reloj(i.tiempoMedioMs)}
        />
      </div>
      <div className="tarjeta" style={{ padding: "4px 10px 6px" }}>
        <table className="tabla">
          <thead>
            <tr>
              <th>#</th>
              <th>{t.pregunta}</th>
              <th className="num">{t.evidencia}</th>
              <th className="num">{t.tiempo}</th>
              <th className="num">{t.ppm}</th>
            </tr>
          </thead>
          <tbody>
            {i.filas.slice(0, FILAS_A_LA_VISTA).map((f) => (
              <tr key={f.numero} className={f.saltada ? "apagada" : undefined}>
                <td className="mono">{f.numero}</td>
                <td>{f.texto}</td>
                <td className="num">
                  {f.saltada || f.evidencia === 0
                    ? "—"
                    : `${f.citadas}/${f.evidencia}`}
                </td>
                <td className="num">
                  {f.tiempoMs === null ? "—" : reloj(f.tiempoMs)}
                </td>
                <td className="num">
                  {f.saltada ? t.saltada : f.ppm === null ? "—" : f.ppm}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {resto > 0 && (
          <p className="ayuda-e tras">
            {t.yMas} {resto} {t.mas}
          </p>
        )}
      </div>
      <div className="fila" style={{ gap: 8 }}>
        <button
          type="button"
          className="btn primario"
          onClick={guardar}
          disabled={ocupado}
        >
          <Ic id="i-candado" s />
          {t.guardar}
        </button>
        <button type="button" className="btn" onClick={exportar} disabled={ocupado}>
          {t.exportar}
        </button>
        <button
          type="button"
          className="btn"
          onClick={() => void cerrarElEnsayo()}
          disabled={ocupado}
        >
          {t.cerrarSinGuardar}
        </button>
      </div>
      {retencion && (
        <p className="ayuda-e">
          {retencion === "siempre"
            ? t.seQueda
            : `${t.seCifraAntes} ${nombreDeLaRetencion(retencion, tn)}${t.seCifraDespues}`}
        </p>
      )}
    </div>
  );
}
