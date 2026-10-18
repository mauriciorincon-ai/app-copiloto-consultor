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
  estadoDeUnModelo,
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

/**
 * Las líneas de estado de este bloque se parten (sprint 010, visto en vivo en la Surface): `.estado` es un chip de una
 * línea en `ghost.css` (`white-space: nowrap`), y aquí las frases son largas —con las dos pistas, una línea de modelo
 * pasa de 140 caracteres— y sacaban una barra de desplazamiento horizontal. Solo aquí: el chip de la maqueta no cambia.
 */
const PARTIR = { whiteSpace: "normal" } as const;

export function HoyEnWindows({
  plataforma: prestada,
}: { plataforma?: Plataforma } = {}) {
  const [propia, setPropia] = usePlataforma();
  const p = prestada ?? propia;
  const t = useTextosDeWindows();
  const idioma = useIdioma();
  // Qué está haciendo ahora el procesador por orden tuya (sprint 010, visto en vivo: «Medir otra vez» solo se apagaba y
  // parecía que no hacía nada durante el medio minuto del modelo grande). `null`: nada.
  const [ocupado, setOcupado] = useState<"midiendo" | "bajando" | null>(null);
  // «Bajando…» se refresca solo mientras dura la descarga, aunque la haya empezado Idioma (B6 del S6).
  const refrescar = useCallback(() => {
    void preguntar<Plataforma>("plataforma_de_este_equipo").then((r) => {
      if (r) setPropia(r);
    });
  }, [setPropia]);
  useRefrescoMientras(
    !prestada && p.transcripcion?.bajando === true,
    refrescar,
  );

  if (!esWindows(p)) return null;

  const grupos = porCuando(p);
  const porClave = new Map(p.funciones.map((f) => [f.clave, f]));
  const whisper = estadoDeWhisper(p.transcripcion);
  const hayModelos = (p.transcripcion?.modelos.length ?? 0) > 0;
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

  const repetir = async (
    que: "midiendo" | "bajando",
    orden: () => Promise<Plataforma | null>,
  ) => {
    setOcupado(que);
    try {
      const nueva = await orden();
      if (nueva) setPropia(nueva);
    } finally {
      setOcupado(null);
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
        <p key={a.accion} style={PARTIR} className="estado warn">
          <Ic id="i-alert" s />
          <span>{t.atajoOcupado(teclasEnIdioma(a.teclas, idioma))}</span>
        </p>
      ))}

      <h3 className="seccion" style={{ margin: "12px 0 4px" }}>
        {t.whisper.titulo}
      </h3>
      {whisper?.que === "bajando" && (
        <p style={PARTIR} className="estado halo">
          <Ic id="i-subir" s />
          <span>{t.whisper.bajando}</span>
        </p>
      )}
      {whisper?.que === "sin-modelo" && (
        <p style={PARTIR} className="estado warn">
          <Ic id="i-alert" s />
          <span>{t.whisper.sinModelo}</span>
        </p>
      )}
      {/* «Sin medir» y «a tiempo» las dicen las líneas por modelo de abajo (sprint 009, auditoría: A1/B5); este resumen
          solo se pinta si no hay esas líneas. «Lento» sí se queda: trae el consejo y el botón del modelo pequeño. */}
      {whisper?.que === "sin-medir" && !hayModelos && (
        <p style={PARTIR} className="estado mute">
          <Ic id="i-ring" s />
          <span>{t.whisper.sinMedir(whisper.modelo)}</span>
        </p>
      )}
      {whisper?.que === "a-tiempo" && !hayModelos && (
        <p style={PARTIR} className="estado ok">
          <Ic id="i-check-circle" s relleno />
          <span>{t.whisper.aTiempo(whisper.modelo, cifra(whisper.ms))}</span>
        </p>
      )}
      {whisper?.que === "lento" && (
        <p style={PARTIR} className="estado warn">
          <Ic id="i-alert" s />
          <span>{t.whisper.lento(whisper.modelo, cifra(whisper.ms))}</span>
        </p>
      )}
      {(p.transcripcion?.descartados ?? 0) > 0 && (
        <p style={PARTIR} className="estado warn">
          <Ic id="i-alert" s />
          <span>
            {t.whisper.descartados(p.transcripcion?.descartados ?? 0)}
          </span>
        </p>
      )}
      {hayModelos && (
        <>
          <h3 className="seccion" style={{ margin: "12px 0 4px" }}>
            {t.whisper.modelos.titulo}
          </h3>
          <div
            style={{ display: "flex", flexDirection: "column", gap: "6px" }}
            data-testid="modelos-de-voz"
          >
            {p.transcripcion?.modelos.map((m) => {
              const e = estadoDeUnModelo(m, p.transcripcion?.techoMs ?? 4000);
              const delCliente = (ms: number | null) =>
                ms === null ? null : cifra(ms);
              const frase =
                e.que === "sin-bajar"
                  ? t.whisper.modelos.sinBajar(m.nombre)
                  : e.que === "sin-medir"
                    ? t.whisper.modelos.sinMedir(m.nombre)
                    : e.que === "a-tiempo"
                      ? t.whisper.modelos.aTiempo(
                          m.nombre,
                          cifra(e.ms),
                          delCliente(e.clienteMs),
                        )
                      : e.que === "cliente-lento"
                        ? t.whisper.modelos.clienteLento(
                            m.nombre,
                            cifra(e.ms),
                            cifra(e.clienteMs),
                          )
                        : t.whisper.modelos.lento(
                            m.nombre,
                            cifra(e.ms),
                            delCliente(e.clienteMs),
                          );
              const etiquetas = [
                m.enUso ? t.whisper.modelos.enUso : null,
                m.porDefecto ? t.whisper.modelos.porDefecto : null,
              ].filter((x): x is string => x !== null);
              // Símbolo, texto y color (regla dura 8): cabe, pasa del techo (tu pista o la del cliente), o todavía no
              // hay cifra.
              const [clase, icono, relleno] =
                e.que === "a-tiempo"
                  ? (["ok", "i-check-circle", true] as const)
                  : e.que === "lento" || e.que === "cliente-lento"
                    ? (["warn", "i-alert", false] as const)
                    : (["mute", "i-ring", false] as const);
              return (
                <p key={m.nombre} style={PARTIR} className={`estado ${clase}`}>
                  <Ic id={icono} s relleno={relleno} />
                  <span>
                    {frase}
                    {etiquetas.length > 0 && ` (${etiquetas.join(" · ")})`}
                  </span>
                </p>
              );
            })}
          </div>
          <p className="mono" style={{ color: "var(--ink-2)" }}>
            {t.whisper.modelos.regla}
          </p>
        </>
      )}
      <p className="mono" style={{ color: "var(--ink-2)" }}>
        {t.whisper.sinReunion}
      </p>
      {/* Lo que el procesador está haciendo por tu orden, dicho con símbolo, texto y color (regla dura 8) y anunciado
          a los lectores de pantalla: un botón apagado no basta para saber que algo está en marcha. */}
      <div role="status" aria-live="polite">
        {ocupado === "midiendo" && (
          <p style={PARTIR} className="estado halo">
            <Ic id="i-ring" s />
            <span>{t.whisper.midiendo}</span>
          </p>
        )}
        {ocupado === "bajando" && whisper?.que !== "bajando" && (
          <p style={PARTIR} className="estado halo">
            <Ic id="i-subir" s />
            <span>{t.whisper.bajandoYMidiendo}</span>
          </p>
        )}
      </div>
      {whisper && whisper.que !== "bajando" && whisper.que !== "sin-modelo" && (
        <div style={{ display: "flex", gap: "8px", flexWrap: "wrap" }}>
          <button
            className="btn mini"
            disabled={ocupado !== null}
            aria-busy={ocupado === "midiendo"}
            onClick={() => repetir("midiendo", calibrarTranscripcion)}
          >
            {ocupado === "midiendo"
              ? t.whisper.midiendoBoton
              : t.whisper.medirOtra}
          </button>
          {/* Con el modelo pequeño ya puesto, ofrecer bajarlo no tiene sentido (B7 del S6). */}
          {whisper.que === "lento" && whisper.modelo !== "small" && (
            <button
              className="btn mini primario"
              disabled={ocupado !== null}
              aria-busy={ocupado === "bajando"}
              onClick={() => repetir("bajando", bajarModeloPequeno)}
            >
              {t.whisper.bajarPequeno}
            </button>
          )}
        </div>
      )}
    </section>
  );
}
