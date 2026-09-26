import { useState } from "react";
import { useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { TodaviaNo, PILA } from "../componentes/Ventana";
import {
  DEL_CLIENTE,
  DEL_CONSULTOR,
  instalarIdioma,
  useDiccionario,
  type Disponibilidad,
  type EstadoDelDiccionario,
  type QueSabeTranscribir,
} from "../cuaderno";

/**
 * IDIOMA — «Idioma y transcripción».
 *
 * Referencia: `docs/diseno/idioma.html`, estados **«así se ve hoy · sprint 2»** y **«… · sin motor de
 * voz»** (mirada 17-bis).
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

/**
 * El nombre que la pantalla da a cada motor. Es el de `idioma.html` —el motor y el sistema que lo
 * trae—, no el identificador interno. Un motor que no esté aquí se enseña por su identificador:
 * mejor un nombre feo que uno inventado.
 */
const NOMBRE_DEL_MOTOR: Record<string, string> = {
  "apple-speechanalyzer": "SpeechAnalyzer · macOS 26",
};

/**
 * **Tu diccionario técnico** (mirada 17-bis, opción a): de dónde salen sus términos y **la ruta
 * entera del archivo**, que es la única puerta para editarlo — un formulario haría escribir al
 * módulo protegido. De la franja de la mirada 4 queda la mitad que es exactamente verdad.
 */
export function TuDiccionario({ diccionario }: { diccionario: EstadoDelDiccionario }) {
  const t = useT().cuaderno;
  return (
    <div className="tarjeta diccionario">
      <div className="fila">
        <h2 className="seccion crece" style={{ margin: 0 }}>
          {t.tuDiccionario}
        </h2>
        <span className="mono" style={{ color: "var(--ink-2)" }}>
          {diccionario.terminos}
        </span>
      </div>
      <table className="tabla">
        <tbody>
          <tr>
            <td>{t.deTuCorpus}</td>
            <td className="num">{diccionario.delCorpus}</td>
          </tr>
          <tr>
            <td>{t.enTuArchivo}</td>
            <td className="num">{diccionario.enTuArchivo}</td>
          </tr>
        </tbody>
      </table>
      <p
        className="mono"
        style={{
          fontSize: "11px",
          color: "var(--ink-2)",
          marginTop: "6px",
          overflowWrap: "anywhere",
          display: "flex",
          gap: "6px",
        }}
      >
        <span style={{ flex: "0 0 auto", marginTop: "1px", display: "inline-flex" }}>
          <Ic id="i-doc" s />
        </span>
        <span>{diccionario.ruta}</span>
      </p>
      <p style={{ fontSize: "11.5px", color: "var(--ink-2)", marginTop: "6px" }}>{t.jamasCompleta}</p>
    </div>
  );
}

function estadoDelModelo(d: Disponibilidad | undefined, instalado: string): string | null {
  if (!d) return null;
  return d.estado === "listo" ? instalado : null;
}

export function Idioma({ transcribe }: { transcribe: QueSabeTranscribir }) {
  const t = useT().cuaderno;
  const diccionario = useDiccionario();
  /**
   * **Lo que pasó al instalar, que la pregunta de `useQueSabeTranscribir` no sabe todavía.**
   *
   * Instalar tarda —lo descarga macOS— y el comando devuelve el estado nuevo. Esperar a que la
   * ventana recupere el foco para volver a preguntar dejaría el botón como si no hubiera pasado
   * nada durante toda la descarga.
   */
  const [instalado, setInstalado] = useState<Record<string, Disponibilidad | "instalando">>({});
  const de = (codigo: string): Disponibilidad | "instalando" | undefined =>
    instalado[codigo] ?? transcribe.idiomas.find((i) => i.codigo === codigo)?.disponibilidad;

  /** El botón que faltaba: pide a macOS el modelo de ese idioma. Hallazgo A7 de la auditoría. */
  const instalar = async (codigo: string) => {
    setInstalado((antes) => ({ ...antes, [codigo]: "instalando" }));
    const d = await instalarIdioma(codigo);
    setInstalado((antes) => {
      const nuevo = { ...antes };
      // `null` es «no hay nadie al otro lado» (fuera de Tauri): se deja como estaba en vez de
      // inventar un desenlace.
      if (d === null) delete nuevo[codigo];
      else nuevo[codigo] = d;
      return nuevo;
    });
  };

  const botonDeInstalar = (codigo: string) => {
    const d = de(codigo);
    if (d === "instalando") {
      return (
        <button className="btn mini" type="button" disabled key={codigo}>
          <Ic id="i-nube" s />
          {t.instalando}
        </button>
      );
    }
    if (!d || d.estado !== "sin-modelo") return null;
    return (
      <button className="btn mini" type="button" onClick={() => void instalar(codigo)} key={codigo}>
        <Ic id="i-nube" s />
        {t.instalarModelo}
      </button>
    );
  };

  /** Una pista con su idioma y el estado de su modelo. Si el modelo no está, se dice por qué. */
  const pista = (icono: string, quien: string, codigo: string) => {
    const d = de(codigo);
    const listo = d !== "instalando" && estadoDelModelo(d, t.modeloInstalado);
    return (
      <div className="buffer" key={codigo}>
        <Ic id={icono} s />
        <span className="que">{quien}</span>
        {/* **El botón va al lado del motivo que arregla**, y su etiqueta es una palabra.
            Costó tres pasadas del gate de fidelidad y una captura leída como imagen: en fila
            propia empujaba la pantalla 58 px fuera de la ventana de 640; en la columna de la
            derecha, 29 px **y partía en dos líneas** —«Instalar el» / «modelo»— con el código del
            idioma flotando encima. Eso último no lo dice ningún número: se vio mirando el PNG. */}
        <span className="donde">
          {listo || (
            <>
              <span className="estado warn">
                <Ic id="i-alert" s />
                <span>{motivo(t, d)}</span>
              </span>{" "}
              {botonDeInstalar(codigo)}
            </>
          )}
        </span>
        <span className="cuanto">{codigo}</span>
      </div>
    );
  };

  /**
   * El botón solo aparece cuando hay algo que instalar. Con el idioma que este Mac no conoce, o sin
   * motor de voz, **no hay nada que descargar** y ofrecerlo sería mandar al usuario a un botón que
   * no puede funcionar.
   */


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
                  En este sprint la transcripción se abre **en la banda** con ⌃⌥T, no desde esta
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
                <kbd>⌃</kbd>
                <kbd>⌥</kbd>
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
          </div>
        </div>

        {/* **El motor tiene NOMBRE, y cuando falta tiene MOTIVO** (mirada 17-bis). Los dos
            campos cruzaban la costura desde el sprint 001 y la pantalla los contaba en prosa —«el
            motor de voz de macOS»— sin decir cuál ni cuántos idiomas admite. El porqué es cerrado
            (17-quater): la frase sale del diccionario, en los dos idiomas. */}
        {transcribe.motivo === null ? (
          <div className="franja ok" role="status">
            <Ic id="i-mac" relleno />
            <div>
              <strong>{t.transcribeTuMac}</strong>
              <p className="mono" style={{ fontSize: "11px", color: "var(--ink-2)", margin: "2px 0 4px" }}>
                {NOMBRE_DEL_MOTOR[transcribe.motor] ?? transcribe.motor} · {transcribe.techo}{" "}
                {t.idiomasListos}
              </p>
              <p>{t.transcribeTuMacDetalle}</p>
            </div>
          </div>
        ) : (
          <div className="franja warn" role="status">
            <Ic id="i-alert" relleno />
            <div>
              <strong>{t.sinMotorTitulo}</strong>
              <p>
                {t.porQueNoHayMotor[transcribe.motivo]} {t.laBandaSigue}
              </p>
            </div>
          </div>
        )}

        <div className="grid-2">
          {diccionario && <TuDiccionario diccionario={diccionario} />}
          {/* Las dos cosas que faltan, juntas. El diccionario técnico salió de esta lista en la
              fase 1 del sprint 002: una pantalla que dice «todavía no» de algo que existe miente
              igual que una que promete lo que falta. */}
          <div className="tarjeta pendiente">
            <h2 className="seccion">{t.loQueTodaviaNo}</h2>
            <div className="fila">
              <span className="crece">{t.variosIdiomasPorPista}</span>
              <TodaviaNo />
            </div>
            <div className="fila">
              <span className="crece">{t.conservarTusTurnos}</span>
              <TodaviaNo />
            </div>
            <p>{t.loQueFaltaDetalle}</p>
          </div>
        </div>
      </div>
    </>
  );
}

/**
 * Por qué no se puede transcribir un idioma. **Del diccionario, en los dos idiomas.**
 *
 * Los tres motivos son distintos y se enseñan distintos: **el modelo no está** (se puede
 * instalar), **el Mac no sabe ese idioma** (no hay nada que instalar) y **no hay motor** (no es
 * cosa del idioma). Colapsarlos en un «no disponible» dejaría al usuario sin saber si esperar,
 * instalar o rendirse.
 *
 * **Estaban escritos aquí en español**, y así llegaban a la interfaz inglesa (hallazgo A6). Y el
 * `motivo` que manda la parte nativa tampoco se pinta: es prosa en español escrita para el log —
 * una frase del sistema en una pantalla inglesa es el mismo defecto por otra puerta—. Lo que se
 * pierde de detalle se gana en que la app diga la verdad en el idioma del usuario.
 */
function motivo(
  t: ReturnType<typeof useT>["cuaderno"],
  d: Disponibilidad | "instalando" | undefined,
): string {
  if (d === "instalando") return t.instalando;
  if (!d) return "—";
  switch (d.estado) {
    case "sin-modelo":
      return t.sinModelo;
    case "idioma-desconocido":
      return t.noLoReconoce;
    case "sin-motor":
      return t.sinMotorDeVoz;
    default:
      return "";
  }
}
