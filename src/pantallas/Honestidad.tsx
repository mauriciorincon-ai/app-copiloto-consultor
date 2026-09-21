import { useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { TodaviaNo, PILA } from "../componentes/Ventana";
import { cortarTodo } from "../cuaderno";

/**
 * HONESTIDAD — «Qué vive en la memoria ahora mismo y qué salió de tu equipo».
 *
 * Referencia: `docs/diseno/honestidad.html`, estado **«así se ve hoy · sprint 1»** (mirada 12).
 *
 * Es la pantalla más honesta de las tres en este sprint, y no por mérito nuestro: hoy **de verdad
 * no vive nada en memoria** —no hay audio, ni transcript, ni lectura de pantalla— y el cero de la
 * red no se mantiene por disciplina sino porque no existe código capaz de abrir una conexión.
 *
 * El número de bytes llega **leído** de lo nativo. Escribirlo en la interfaz sería la interfaz
 * afirmando el cero en vez de medirlo, que es justo lo que esta pantalla existe para no hacer.
 */

/** Las piezas del kill-switch, en el mismo orden que `src-tauri/src/corte.rs`. */
const PIEZAS_CORTADAS = 3;
const PIEZAS_TOTALES = 7;

export function Honestidad({ bytes }: { bytes: string }) {
  const t = useT().cuaderno;
  const [cifra, unidad = "B"] = bytes.split(" ");

  const buffer = (icono: string, que: string) => (
    <div className="buffer pendiente" key={que}>
      <Ic id={icono} s />
      <span className="que">{que}</span>
      <span className="donde">
        <TodaviaNo />
      </span>
      <span className="cuanto">0 B</span>
    </div>
  );

  return (
    <>
      <div className="titulo">
        <h1>{t.honestidadTitulo}</h1>
        <p className="sub">{t.honestidadSub}</p>
      </div>

      <div style={PILA}>
        <div className="grid-2" style={{ gridTemplateColumns: "1.25fr 1fr" }}>
          <div className="tarjeta" style={{ paddingBottom: "4px" }}>
            <div className="fila">
              <h2 className="seccion crece" style={{ margin: 0 }}>
                {t.queViveEnMemoria}
              </h2>
              <span className="mono" style={{ color: "var(--ok)" }}>
                RAM · 0 B
              </span>
            </div>
            <div style={{ margin: "0 -4px" }}>
              {buffer("i-mic", t.bufMic)}
              {buffer("i-sistema", t.bufSistema)}
              {buffer("i-ojo", t.bufTranscript)}
              {buffer("i-pantalla", t.bufFrame)}
            </div>
          </div>

          <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
            <div className="contador cero">
              {/* La maqueta separa la cifra de la unidad («0» grande, «B» pequeña) y el CSS las
                  dimensiona distinto. Si llegaran como un solo texto, la unidad saldría del tamaño
                  de la cifra — 24 px de diferencia que el gate de fidelidad vería y el ojo no. */}
              <div className="cifra">
                {cifra} <small>{unidad}</small>
              </div>
              <div className="etq">
                <Ic id="i-check-circle" s relleno color="var(--ok)" />{" "}
                <span>{t.salieronDeTuEquipo}</span>
              </div>
            </div>
            <div className="tarjeta">
              <h2 className="seccion">{t.modo}</h2>
              <span className="estado ok">
                <Ic id="i-mac" s relleno />
                <span>{t.modoLocal}</span>
              </span>
              <p>{t.modoDetalle}</p>
            </div>
            <button className="kill" style={{ justifyContent: "center" }} onClick={cortarTodo}>
              <Ic id="i-rayo" s relleno />
              <span>{t.funcionaCorte}</span> <kbd>⌥⎋</kbd>
            </button>
            <p className="mono" style={{ color: "var(--ink-2)" }}>
              {PIEZAS_CORTADAS} de {PIEZAS_TOTALES} {t.piezasCola}
            </p>
          </div>
        </div>

        <div className="tarjeta pendiente">
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              {t.loQueQuedara}
            </h2>
            <TodaviaNo />
          </div>
          <p>{t.loQueQuedaraDetalle}</p>
        </div>
      </div>
    </>
  );
}
