import { useCallback, useState } from "react";
import { Ic } from "./Iconos";
import { useIdioma } from "../i18n";
import { preguntar } from "../puente";
import { cifraEnIdioma, useTextosDeWindows } from "../i18n/windows";
import {
  atajosSinRegistrar,
  bajarModeloPequeno,
  calibrarTranscripcion,
  esWindows,
  estadoDeWhisper,
  porCuando,
  teclasEnIdioma,
  useRefrescoMientras,
  usePlataforma,
  type ClaveDeWindows,
  type Cuando,
  type Plataforma,
} from "../plataforma";

/**
 * «HOY EN WINDOWS» — el bloque de Honestidad que existe solo en Windows (sprint 006, ADR 022).
 *
 * Lo que dice sale de la lista del ADR, que vive en Rust y llega por `plataforma_de_este_equipo`; los
 * textos, de `src/i18n/windows.ts`. En macOS no pinta nada: la pantalla de Honestidad aprobada en la
 * maqueta no cambia, y por eso este bloque no pasa por el gate de fidelidad.
 *
 * **Símbolo, texto y color** en cada fila (regla dura 8): el color solo no dice nada a quien tiene
 * daltonismo leve.
 */

const ICONO: Record<Cuando, { id: string; color: string }> = {
  si: { id: "i-check-circle", color: "var(--ok)" },
  "no-por-ahora": { id: "i-x-circle", color: "var(--mute)" },
};

export function HoyEnWindows({
  plataforma: prestada,
}: { plataforma?: Plataforma } = {}) {
  const [propia, setPropia] = usePlataforma();
  const p = prestada ?? propia;
  const t = useTextosDeWindows();
  const idioma = useIdioma();
  const [ocupado, setOcupado] = useState(false);
  // «Bajando…» se refresca solo mientras dura la descarga, aunque la haya empezado Idioma (B6 del S6).
  const refrescar = useCallback(() => {
    void preguntar<Plataforma>("plataforma_de_este_equipo").then((r) => {
      if (r) setPropia(r);
    });
  }, [setPropia]);
  useRefrescoMientras(!prestada && p.transcripcion?.bajando === true, refrescar);

  if (!esWindows(p)) return null;

  const grupos = porCuando(p);
  const porClave = new Map(p.funciones.map((f) => [f.clave, f]));
  const whisper = estadoDeWhisper(p.transcripcion);
  const { si, "no-por-ahora": noPorAhora } = grupos;

  const fila = (clave: ClaveDeWindows) => {
    const f = porClave.get(clave);
    const texto = t.funciones[clave];
    const icono = ICONO[f?.cuando ?? "si"];
    return (
      <li
        key={clave}
        className="fila"
        style={{ flexWrap: "nowrap", alignItems: "flex-start" }}
      >
        <Ic id={icono.id} s relleno color={icono.color} />
        <span className="crece">
          <b>{texto.nombre}</b>: {texto.detalle}
        </span>
      </li>
    );
  };

  const bloque = (titulo: string, claves: ClaveDeWindows[]) =>
    claves.length > 0 && (
      <>
        <h3 className="seccion" style={{ margin: "12px 0 4px" }}>
          {titulo}
        </h3>
        <ul
          style={{
            listStyle: "none",
            margin: 0,
            padding: 0,
            display: "flex",
            flexDirection: "column",
            gap: "6px",
          }}
        >
          {claves.map(fila)}
        </ul>
      </>
    );

  const cifra = (ms: number) => cifraEnIdioma(ms, idioma);

  const repetir = async (orden: () => Promise<Plataforma | null>) => {
    setOcupado(true);
    try {
      const nueva = await orden();
      if (nueva) setPropia(nueva);
    } finally {
      setOcupado(false);
    }
  };

  return (
    <section className="tarjeta" aria-labelledby="hoy-en-windows">
      <h2 id="hoy-en-windows" className="seccion">
        {t.titulo}
      </h2>
      <p>{t.intro}</p>
      {bloque(t.ya, si)}
      {bloque(t.noPorAhora, noPorAhora)}
      {atajosSinRegistrar(p).map((a) => (
        <p key={a.accion} className="estado warn">
          <Ic id="i-alert" s />
          <span>{t.atajoOcupado(teclasEnIdioma(a.teclas, idioma))}</span>
        </p>
      ))}

      <h3 className="seccion" style={{ margin: "12px 0 4px" }}>
        {t.whisper.titulo}
      </h3>
      {whisper?.que === "bajando" && (
        <p className="estado halo">
          <Ic id="i-subir" s />
          <span>{t.whisper.bajando}</span>
        </p>
      )}
      {whisper?.que === "sin-modelo" && (
        <p className="estado warn">
          <Ic id="i-alert" s />
          <span>{t.whisper.sinModelo}</span>
        </p>
      )}
      {whisper?.que === "sin-medir" && (
        <p className="estado mute">
          <Ic id="i-ring" s />
          <span>{t.whisper.sinMedir(whisper.modelo)}</span>
        </p>
      )}
      {whisper?.que === "a-tiempo" && (
        <p className="estado ok">
          <Ic id="i-check-circle" s relleno />
          <span>{t.whisper.aTiempo(whisper.modelo, cifra(whisper.ms))}</span>
        </p>
      )}
      {whisper?.que === "lento" && (
        <p className="estado warn">
          <Ic id="i-alert" s />
          <span>{t.whisper.lento(whisper.modelo, cifra(whisper.ms))}</span>
        </p>
      )}
      {(p.transcripcion?.descartados ?? 0) > 0 && (
        <p className="estado warn">
          <Ic id="i-alert" s />
          <span>{t.whisper.descartados(p.transcripcion?.descartados ?? 0)}</span>
        </p>
      )}
      <p className="mono" style={{ color: "var(--ink-2)" }}>
        {t.whisper.sinReunion}
      </p>
      {whisper && whisper.que !== "bajando" && whisper.que !== "sin-modelo" && (
        <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
          <button
            className="btn mini"
            disabled={ocupado}
            onClick={() => repetir(calibrarTranscripcion)}
          >
            {t.whisper.medirOtra}
          </button>
          {/* Con el modelo pequeño ya puesto, ofrecer bajarlo no tiene sentido (B7 del S6). */}
          {whisper.que === "lento" && whisper.modelo !== "small" && (
            <button
              className="btn mini primario"
              disabled={ocupado}
              onClick={() => repetir(bajarModeloPequeno)}
            >
              {t.whisper.bajarPequeno}
            </button>
          )}
        </div>
      )}
    </section>
  );
}
