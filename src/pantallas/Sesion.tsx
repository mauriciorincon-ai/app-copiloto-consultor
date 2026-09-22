import { useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { Fila, Funciona, TodaviaNo, PILA } from "../componentes/Ventana";
import {
  empezarAEscuchar,
  dejarDeEscuchar,
  DEL_CLIENTE,
  DEL_CONSULTOR,
  type EstadoDeEscucha,
  type Reunion,
  type Salida,
} from "../cuaderno";

/**
 * SESIÓN — «Antes de empezar».
 *
 * Referencia: `docs/diseno/sesion.html`, estado **«así se ve hoy · sprint 1»** (miradas 12 y 13).
 *
 * Lo que está vivo de verdad tras la fase 3: la reunión detectada y su protección, **las dos
 * pistas** y la transcripción local. Lo que no existe lleva «todavía no» — nunca verde, nunca
 * escondido.
 *
 * **La fila de los auriculares dejó de ser decorativa.** Era «todavía no» y ahora mide: si el
 * sonido sale por los altavoces internos, el micrófono va a oír también al cliente y sus turnos
 * aparecerían como del consultor. La app los marca como eco, pero la solución de verdad la tiene
 * el usuario y se le dice aquí, antes de la reunión y no después.
 */
export function Sesion({
  reunion,
  escucha,
  salida,
}: {
  reunion: Reunion;
  escucha: EstadoDeEscucha;
  salida: Salida;
}) {
  const t = useT().cuaderno;
  const hayEco = salida.salida === "altavoces";

  return (
    <>
      <div className="titulo">
        <h1>{t.sesionTitulo}</h1>
        <p className="sub">{t.sesionSub}</p>
      </div>

      <div style={PILA}>
        {reunion.que === "detectada" ? (
          <div className="tarjeta">
            <div className="fila">
              <Ic id="i-video" color="var(--halo)" />
              <h3 className="crece">
                {reunion.titulo ? `${reunion.cliente} · ${reunion.titulo}` : reunion.cliente}
              </h3>
              {reunion.proteccion === "Verificada" ? (
                <span className="estado ok">
                  <Ic id="i-check-circle" s relleno />
                  <span>{t.proteccionVerificada}</span>
                </span>
              ) : (
                <span className="estado warn">
                  <Ic id="i-alert" s relleno />
                  <span>{t.proteccionSinVerificar}</span>
                </span>
              )}
            </div>
            <p>{t.proteccionDetalle}</p>
          </div>
        ) : (
          <div className="tarjeta" style={{ alignItems: "flex-start", gap: "14px" }}>
            <span className="estado mute">
              <Ic id="i-ring" s />
              <span>{t.sinReunion}</span>
            </span>
            <p
              className="voz"
              style={{
                fontFamily: "var(--font-evidencia)",
                fontStyle: "italic",
                fontSize: "17px",
                color: "var(--ink-2)",
                maxWidth: "56ch",
              }}
            >
              {t.sinReunionVoz}
            </p>
          </div>
        )}

        <div className="grid-2">
          <div className="tarjeta">
            <h2 className="seccion">{t.dosPistas}</h2>
            {/* «Funciona» habla del grifo, no de las muestras. Sin nadie hablando no llega ni
                una, y eso no es una avería: es una reunión en silencio. */}
            <Fila icono="i-mic" color="var(--ok)" texto={t.pistaMic}>
              <Funciona />
            </Fila>
            <Fila icono="i-sistema" color="var(--ok)" texto={t.pistaSistema}>
              <Funciona />
            </Fila>
            <Fila icono="i-pantalla" texto={t.pistaPantalla} pendiente>
              <TodaviaNo />
            </Fila>
            <Fila
              icono="i-auriculares"
              color={hayEco ? "var(--warn)" : "var(--ok)"}
              texto={t.pistaAuriculares}
            >
              {hayEco ? (
                <span className="estado warn">
                  <Ic id="i-alert" s />
                  <span>{t.altavocesInternos}</span>
                </span>
              ) : (
                <Funciona />
              )}
            </Fila>
            {hayEco && (
              <p style={{ fontSize: "11.5px", color: "var(--ink-2)", marginTop: "6px" }}>
                {t.avisoDelEco}
              </p>
            )}
          </div>

          <div className="tarjeta pendiente">
            <h2 className="seccion">{t.esteCliente}</h2>
            <div className="fila">
              <span className="crece">{t.fichaNdaRadar}</span>
              <TodaviaNo />
            </div>
            <p>{t.noSeInventa}</p>
          </div>
        </div>

        <div className="tarjeta">
          <h2 className="seccion">{t.queFuncionaHoy}</h2>
          <Fila icono="i-candado" color="var(--ok)" texto={t.funcionaBanda}>
            <Funciona />
          </Fila>
          <Fila icono="i-video" color="var(--ok)" texto={t.funcionaAcople}>
            <Funciona />
          </Fila>
          <Fila icono="i-voz" color="var(--ok)" texto={t.funcionaEscucha}>
            <Funciona />
          </Fila>
          {/* El kill-switch salió de esta lista y bajó a la fila de la acción, al lado de la
              promesa que cumple: «corta todo · el sonido nunca se guarda · nada sale de tu
              equipo». La lista se quedó con lo que sí lleva la palabra «Funciona», y la pantalla
              volvió a caber en los 640 px de la ventana — que es lo que el gate de fidelidad
              midió y el ojo no. */}
          <div className="fila">
            <button
              className="btn primario"
              onClick={() =>
                escucha.escuchando ? dejarDeEscuchar() : empezarAEscuchar(DEL_CONSULTOR, DEL_CLIENTE)
              }
            >
              <Ic id="i-voz" s relleno />
              <span>{t.iniciarSesion}</span>
            </button>
            <span className="crece"></span>
            <span className="tecla">
              <kbd>⌥⎋</kbd>
            </span>
            <span className="mono" style={{ color: "var(--ink-2)" }}>
              {t.nadaSale}
            </span>
          </div>
        </div>
      </div>
    </>
  );
}
