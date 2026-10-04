import { useEffect, useState } from "react";
import { useIdioma, useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import type { Seccion } from "../componentes/Ventana";
import { hayTauri } from "../puente";
import {
  cerrarElEnsayo,
  empezarElEnsayo,
  ensayoListo,
  ensayoRepetir,
  ensayoSaltar,
  ensayoSiLoDije,
  ensayoTerminar,
  entreComillas,
  esNoEmpezo,
  estadoDeLaUrl,
  reloj,
  useEnsayo,
  usePreparacion,
  usePreparacionDeMuestra,
  useVistaDeMuestra,
  type EstadoDelBanco,
  type Evidencia,
  type Muletilla,
  type NoEmpezo,
  type Preparacion,
  type PreguntaEnPantalla,
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
export function Ensayo({
  busqueda = "",
  ir,
}: {
  busqueda?: string;
  ir: (s: Seccion) => void;
}) {
  const t = useT().ensayo;
  const enTauri = hayTauri();
  const muestra = estadoDeLaUrl(busqueda);
  const [real] = useEnsayo();
  const deMuestra = useVistaDeMuestra(muestra);
  const vista = enTauri ? real : deMuestra;

  return (
    <>
      <div className="titulo">
        <h1>{t.titulo}</h1>
        <p className="sub">{t.sub}</p>
      </div>
      {vista ? (
        <EnCurso vista={vista} />
      ) : (
        <Preparar busqueda={busqueda} ir={ir} />
      )}
    </>
  );
}

// ─── 1 · preparar · 1b · no empezó · 7 · sin corpus ───────────────────────────────────────────────

function Preparar({
  busqueda,
  ir,
}: {
  busqueda: string;
  ir: (s: Seccion) => void;
}) {
  const t = useT().ensayo;
  const enTauri = hayTauri();
  const muestra = estadoDeLaUrl(busqueda);
  const [cliente, setCliente] = useState<string | null>(null);
  const [propuesta, setPropuesta] = useState<string | null>(null);
  const [tope, setTope] = useState(8);
  const [voz, setVoz] = useState(true);
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
    setCliente(c);
    setPropuesta(null);
    setNoEmpezo(null);
  };

  if (prep.sinCorpus)
    return <SinCorpus prep={prep} elegir={elegirCliente} ir={ir} />;

  const empezar = () => {
    if (!prep.cliente || empezando) return;
    setEmpezando(true);
    setNoEmpezo(null);
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
      {noEmpezo && noEmpezo.que !== "sin-corpus" && (
        <PorQueNoEmpezo no={noEmpezo} />
      )}
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
          <div className="fila">
            <span className="crece">{t.propuesta}</span>
            <select
              className="selector"
              aria-label={t.propuesta}
              value={prep.propuesta ?? ""}
              onChange={(e) => setPropuesta(e.target.value || null)}
            >
              {prep.propuestas.map((p) => (
                <option key={p.ruta} value={p.ruta}>
                  {p.nombre}
                </option>
              ))}
            </select>
          </div>
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
                    onClick={() => setTope(n)}
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
            onClick={() => setVoz(!voz)}
          >
            <span className="track"></span>
            <span className="etq">{t.leerEnVozAlta}</span>
          </button>
          <p style={{ fontSize: 11.5, color: "var(--ink-2)" }}>
            {t.vozSinReunion}
          </p>
          <p style={{ fontSize: 11.5, color: "var(--ink-2)" }}>
            {t.idiomaDelEnsayo[prep.idioma]}
          </p>
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
        <p className="crece" style={{ fontSize: 11.5, color: "var(--ink-2)" }}>
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
  if (no.que === "en-reunion") {
    return (
      <div className="franja warn">
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
    <div className="franja err">
      <Ic id="i-mic-off" s />
      <div>
        <strong>{t.microfonoNoAbrio}</strong>
        <p>{sinPermiso ? t.microfonoSinPermiso : t.microfonoOtro}</p>
      </div>
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
      if (e.metaKey || e.ctrlKey || e.altKey) return;
      const objetivo = e.target as HTMLElement | null;
      if (
        objetivo &&
        ["INPUT", "SELECT", "TEXTAREA"].includes(objetivo.tagName)
      )
        return;
      // Enter sobre un botón ya lo pulsa el botón: no se pulsa dos veces.
      if (e.key === "Enter" && objetivo?.tagName === "BUTTON") return;
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

function EnCurso({ vista }: { vista: VistaDelEnsayo }) {
  useTeclas(vista);
  switch (vista.fase) {
    case "preguntando":
      return <Preguntando vista={vista} />;
    case "respondiendo":
      return <Respondiendo vista={vista} />;
    case "evaluada":
      return <Evaluada vista={vista} />;
    case "cerrado":
      return <ElInforme vista={vista} />;
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
    <p style={{ fontSize: 11.5, color: "var(--ink-2)", marginTop: -6 }}>
      {texto}
    </p>
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
        <span style={{ fontSize: 11.5, color: "var(--ink-2)" }}>
          {t.seCierra}
        </span>
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

function Evaluada({ vista }: { vista: VistaDelEnsayo }) {
  const t = useT().ensayo;
  const unidades = useT().banda.unidades;
  const idioma = useIdioma();
  const e = vista.evaluacion;
  if (!e) return null;
  const conIndice = e.evidencia.map((ev, i) => ({ ev, i }));
  const usadas = conIndice.filter(({ ev }) => ev.citada || ev.dichaPorTi);
  const sinUsar = conIndice.filter(({ ev }) => !ev.citada && !ev.dichaPorTi);
  const citadas = usadas.length;
  const muletillas = e.muletillas.reduce((n, m) => n + m.veces, 0);
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
          n={String(muletillas)}
          d={muletillas > 0 ? detalleDe(e.muletillas, idioma) : undefined}
        />
        <Cifra
          q={t.evidencia}
          n={
            e.evidencia.length === 0
              ? "—"
              : `${citadas} ${t.de} ${e.evidencia.length}`
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
        <p className="crece" style={{ fontSize: 11.5, color: "var(--ink-2)" }}>
          {t.sinPuntaje}
        </p>
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

function ElInforme({ vista }: { vista: VistaDelEnsayo }) {
  const t = useT().ensayo;
  const idioma = useIdioma();
  const i = vista.informe;
  if (!i) return null;
  const resto = i.filas.length - FILAS_A_LA_VISTA;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 10 }}>
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
          n={String(i.muletillas)}
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
              <th className="num">ppm</th>
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
          <p
            style={{ fontSize: 11.5, color: "var(--ink-2)", margin: "6px 0 0" }}
          >
            {t.yMas} {resto} {t.mas}
          </p>
        )}
      </div>
      <div className="fila" style={{ gap: 8 }}>
        {/* Guardar y exportar llegan con la fase 4 (enmienda 4 del ADR 015): hasta entonces, apagados. */}
        <button type="button" className="btn primario" disabled>
          <Ic id="i-candado" s />
          {t.guardar}
        </button>
        <button type="button" className="btn" disabled>
          {t.exportar}
        </button>
        <button
          type="button"
          className="btn"
          onClick={() => void cerrarElEnsayo()}
        >
          {t.cerrarSinGuardar}
        </button>
      </div>
    </div>
  );
}
