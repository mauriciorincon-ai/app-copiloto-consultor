import { useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { Fila, Funciona, TodaviaNo, PILA } from "../componentes/Ventana";
import type { Reunion } from "../cuaderno";

/**
 * SESIÓN — «Antes de empezar».
 *
 * Referencia: `docs/diseno/sesion.html`, estado **«así se ve hoy · sprint 1»** (mirada 12).
 *
 * Lo que está vivo de verdad en este sprint: **la reunión detectada y su protección**, leídas del
 * Mac por el catálogo versionado. Lo que no existe lleva «todavía no» — nunca verde, nunca
 * escondido. Y la tarjeta «Qué funciona hoy» está ahí porque sin ella la pantalla mentía por
 * omisión: la banda protegida, el acople y `⌥⎋` sí funcionan.
 */
export function Sesion({ reunion }: { reunion: Reunion }) {
  const t = useT().cuaderno;

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
            <Fila icono="i-mic" texto={t.pistaMic} pendiente>
              <TodaviaNo />
            </Fila>
            <Fila icono="i-sistema" texto={t.pistaSistema} pendiente>
              <TodaviaNo />
            </Fila>
            <Fila icono="i-pantalla" texto={t.pistaPantalla} pendiente>
              <TodaviaNo />
            </Fila>
            <Fila icono="i-auriculares" texto={t.pistaAuriculares} pendiente>
              <TodaviaNo />
            </Fila>
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
          <Fila icono="i-rayo" color="var(--ok)" texto={t.funcionaCorte}>
            <span className="tecla">
              <kbd>⌥⎋</kbd>
            </span>
          </Fila>
          <Fila icono="i-voz" texto={t.iniciarSesion} pendiente>
            <TodaviaNo />
          </Fila>
        </div>

        <div className="fila">
          <span className="crece"></span>
          <span className="mono" style={{ color: "var(--ink-2)" }}>
            {t.nadaSale}
          </span>
        </div>
      </div>
    </>
  );
}
