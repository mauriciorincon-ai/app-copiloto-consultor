import { useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { TodaviaNo, PILA } from "../componentes/Ventana";
import { DEL_CLIENTE, DEL_CONSULTOR, type Disponibilidad, type QueSabeTranscribir } from "../cuaderno";

/**
 * IDIOMA — «Idioma y transcripción».
 *
 * Referencia: `docs/diseno/idioma.html`, estado **«así se ve hoy · sprint 1»** (mirada 13).
 *
 * Lo que está vivo: la transcripción corre en el Mac y nace **oculta**, y cada pista tiene su
 * idioma con el estado real de su modelo leído del sistema. Lo que no existe lleva «todavía no»:
 * varios idiomas por pista, el diccionario técnico y conservar tus turnos.
 *
 * **Y dos cosas que solo se supieron construyendo, y que esta pantalla dice en vez de esconder:**
 * macOS solo deja tener cinco idiomas listos a la vez, y el modelo de cada idioma lo descarga
 * macOS cuando el usuario se lo pide — la única vez que un módulo protegido de esta app toca la
 * red, y en la dirección contraria: entra el modelo, no sale nada.
 */

function estadoDelModelo(d: Disponibilidad | undefined, instalado: string): string | null {
  if (!d) return null;
  return d.estado === "listo" ? instalado : null;
}

export function Idioma({ transcribe }: { transcribe: QueSabeTranscribir }) {
  const t = useT().cuaderno;
  const de = (codigo: string) => transcribe.idiomas.find((i) => i.codigo === codigo)?.disponibilidad;

  /** Una pista con su idioma y el estado de su modelo. Si el modelo no está, se dice por qué. */
  const pista = (icono: string, quien: string, codigo: string) => {
    const d = de(codigo);
    const listo = estadoDelModelo(d, t.modeloInstalado);
    return (
      <div className="buffer" key={codigo}>
        <Ic id={icono} s />
        <span className="que">{quien}</span>
        <span className="donde">
          {listo ?? (
            <span className="estado warn">
              <Ic id="i-alert" s />
              <span>{motivo(d, transcribe.motivo)}</span>
            </span>
          )}
        </span>
        <span className="cuanto">{codigo}</span>
      </div>
    );
  };

  return (
    <>
      <div className="titulo">
        <h1>{t.idiomaTitulo}</h1>
        <p className="sub">{t.idiomaSub}</p>
      </div>

      <div style={PILA}>
        <div className="grid-2" style={{ gridTemplateColumns: "1.15fr 1fr" }}>
          <div className="tarjeta">
            <div className="fila">
              <h2 className="seccion crece" style={{ margin: 0 }}>
                {t.transcripcionEnVivo}
              </h2>
              {/* La maqueta lo dibuja como un interruptor, y aquí es un `span`, no un `label`.
                  En este sprint la transcripción se abre **en la banda** con ⌘⇧T, no desde esta
                  pantalla: un `label` sin control al que asociarse sería un interruptor que no
                  conmuta nada — lo dice el lint de accesibilidad y tiene razón. Se pinta igual
                  (las reglas de `ghost.css` van por clase) y se lee como lo que es: un estado. */}
              <span className="conm off" role="status">
                <span className="track"></span>
                <span className="etq">
                  <Ic id="i-ojo-off" s />
                  <span>{t.oculta}</span>
                </span>
              </span>
            </div>
            <p style={{ marginTop: "8px" }}>{t.naceOculta}</p>
            <div className="fila" style={{ marginTop: "10px", gap: "10px" }}>
              <span className="tecla">
                <kbd>⌘</kbd>
                <kbd>⇧</kbd>
                <kbd>T</kbd>
              </span>
              <span style={{ fontSize: "12.5px", color: "var(--ink-2)" }}>{t.laMuestraCuando}</span>
            </div>
          </div>

          <div className="tarjeta">
            <h2 className="seccion" style={{ margin: "0 0 4px" }}>
              {t.idiomaPorPista}
            </h2>
            {pista("i-mic", t.tuMicrofono, DEL_CONSULTOR)}
            {pista("i-sistema", t.clienteSistema, DEL_CLIENTE)}
            <p style={{ fontSize: "11.5px", color: "var(--ink-2)", marginTop: "6px" }}>
              {t.cincoIdiomas}
            </p>
          </div>
        </div>

        <div className="franja ok" role="status">
          <Ic id="i-mac" relleno />
          <div>
            <strong>{t.transcribeTuMac}</strong>
            <p>{t.transcribeTuMacDetalle}</p>
          </div>
        </div>

        {/* Las tres cosas que faltan, juntas y en una sola tarjeta. Separadas ocupaban media
            pantalla y empujaban la tercera fuera de la ventana — y además se leían como tres
            ausencias distintas cuando son la misma: lo que llega después de este sprint. */}
        <div className="tarjeta pendiente">
          <h2 className="seccion">{t.loQueTodaviaNo}</h2>
          <div className="fila">
            <span className="crece">{t.variosIdiomasPorPista}</span>
            <TodaviaNo />
          </div>
          <div className="fila">
            <span className="crece">{t.diccionarioTecnico}</span>
            <TodaviaNo />
          </div>
          <div className="fila">
            <span className="crece">{t.conservarTusTurnos}</span>
            <TodaviaNo />
          </div>
          <p>{t.loQueFaltaDetalle}</p>
        </div>
      </div>
    </>
  );
}

/**
 * Por qué no se puede transcribir un idioma, en español llano.
 *
 * Los tres motivos son distintos y se enseñan distintos: **el modelo no está** (se puede
 * instalar), **el Mac no sabe ese idioma** (no hay nada que instalar) y **no hay motor** (no es
 * cosa del idioma). Colapsarlos en un «no disponible» dejaría al usuario sin saber si esperar,
 * instalar o rendirse.
 */
function motivo(d: Disponibilidad | undefined, general: string | null): string {
  if (!d) return general ?? "—";
  switch (d.estado) {
    case "sin-modelo":
      return "sin modelo";
    case "idioma-desconocido":
      return "no lo reconoce";
    case "sin-motor":
      return d.motivo;
    default:
      return "";
  }
}
