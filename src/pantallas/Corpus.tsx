import { useIdioma, useT, type Idioma } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { TodaviaNo } from "../componentes/Ventana";
import { indexarCorpus, useCorpus, type UnidadDelCorpus } from "../cuaderno";

/**
 * CORPUS — «Tus documentos, indexados donde están».
 *
 * Referencia: `docs/diseno/corpus.html`, estado **«así se ve hoy · sprint 1»** (mirada 14).
 *
 * Lo que está vivo: señalar una carpeta, leer PDF, Word y Markdown **donde están**, repartirlos en
 * las cinco unidades y decir qué quedó sin leer. Lo que no existe lleva «todavía no»: arrastrar y
 * soltar, releer solo lo que cambie, y leer lo escaneado.
 *
 * **Y tres cosas que solo se supieron construyendo, y que esta pantalla dice en vez de esconder:**
 * un PDF no trae títulos —los suyos los conjetura la app por la forma del texto, y por eso se
 * cuentan—, el índice vive en la carpeta de datos de la app con permisos cerrados, y hay un techo
 * de 2 000 documentos por carpeta que es un aviso y no un límite del que se pueda salir mal.
 */

/** Las cinco unidades en el orden del modelo de consultoría, con su descripción de la maqueta. */
const UNIDADES: { id: UnidadDelCorpus; clave: "uProp" | "uMarco" | "uCaso" | "uCliente" | "uPerfil" }[] = [
  { id: "propuesta", clave: "uProp" },
  { id: "marco", clave: "uMarco" },
  { id: "caso", clave: "uCaso" },
  { id: "cliente", clave: "uCliente" },
  { id: "perfil", clave: "uPerfil" },
];

/**
 * El tamaño del índice, **con el separador decimal del idioma**.
 *
 * La maqueta ya lo distingue —«4,2 MB» en español, «4.2 MB» en inglés— y escribir siempre la
 * coma deja media interfaz hablando español dentro de una pantalla inglesa. Lo cazó la lectura
 * de la captura, no un test: los dos encuadres estaban por debajo del umbral porque mi propio
 * bloque de la maqueta traía el mismo defecto.
 */
function legible(bytes: number, idioma: Idioma): string {
  if (bytes < 1024) return `${bytes} B`;
  const mb = bytes / (1024 * 1024);
  if (mb < 1) return `${Math.round(bytes / 1024)} kB`;
  return `${new Intl.NumberFormat(idioma, { minimumFractionDigits: 1, maximumFractionDigits: 1 }).format(mb)} MB`;
}

export function Corpus() {
  const t = useT().cuaderno;
  const b = useT().banda;
  const corpus = useCorpus();
  const idioma = useIdioma();
  const cuantos = (u: UnidadDelCorpus) =>
    corpus.porUnidad.find((p) => p.unidad === u)?.documentos ?? 0;

  return (
    <>
      <div className="titulo">
        <h1>{t.corpusTitulo}</h1>
        <p className="sub">{t.corpusSub}</p>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
        <div className="fila">
          <button className="btn" type="button" onClick={indexarCorpus}>
            <Ic id="i-doc" s />
            {t.senalarCarpeta}
          </button>
          <span className="crece" style={{ fontSize: "12.5px", color: "var(--ink-2)" }}>
            {t.noSeCopianAntes} <b>{t.noSeCopianFuerte}</b>
            {t.noSeCopianDespues}
          </span>
          <span className="red cero mono">
            <Ic id="i-subir" s />
            0 B
          </span>
        </div>

        <div className="grid-3" style={{ gap: "10px" }}>
          {UNIDADES.map((u) => (
            <div className="tarjeta" style={{ padding: "10px 12px" }} key={u.id}>
              <div className="fila">
                <span className="unidad-chip">{b.unidades[u.id]}</span>
                <span className="crece" />
                <span className="mono" style={{ fontSize: "17px" }}>
                  {cuantos(u.id)}
                </span>
              </div>
              <p style={{ marginTop: "2px" }}>{t[u.clave]}</p>
            </div>
          ))}
          {/* La sexta respuesta. Lo que no encaja en ninguna unidad **no se fuerza**: caer en la
              equivocada haría que una ficha citara una fuente que no es. */}
          <div className="tarjeta" style={{ padding: "10px 12px", borderStyle: "dashed" }}>
            <div className="fila">
              <span
                className="unidad-chip"
                style={{ background: "transparent", border: "var(--borde) dashed var(--line-2)" }}
              >
                {t.sinUnidad}
              </span>
              <span className="crece" />
              <span className="mono" style={{ fontSize: "17px" }}>
                {corpus.sinUnidad}
              </span>
            </div>
            <p style={{ marginTop: "2px" }}>{t.sinUnidadNota}</p>
          </div>
        </div>

        <div className="grid-2" style={{ gap: "12px", alignItems: "start" }}>
          <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
            <div className="tarjeta">
              <h2 className="seccion" style={{ margin: "0 0 6px" }}>
                {t.dondeVive}
              </h2>
              <div className="buffer">
                <Ic id="i-candado" s />
                <span className="que">{t.soloTu}</span>
                <span className="donde mono">700</span>
                <span className="cuanto">{legible(corpus.bytesDelIndice, idioma)}</span>
              </div>
              <p
                className="mono"
                style={{ fontSize: "10.5px", color: "var(--ink-2)", marginTop: "5px" }}
              >
                {corpus.dondeVive ?? "—"}
              </p>
            </div>

            <div className="franja warn" role="status" style={{ margin: 0 }}>
              <Ic id="i-alert" relleno />
              <div>
                <strong>
                  {corpus.ilegibles} {t.sinLeer} · {corpus.conjeturados} {t.conSeccionesConjeturadas}
                </strong>
                <p>{t.conjeturadas}</p>
              </div>
            </div>
          </div>

          <div className="tarjeta pendiente" style={{ margin: 0 }}>
            <h2 className="seccion">{t.corpusPendiente}</h2>
            {[t.arrastrar, t.releer, t.leerEscaneado].map((que) => (
              <div className="fila" key={que}>
                <span className="crece">{que}</span>
                <TodaviaNo />
              </div>
            ))}
            <p>{t.corpusNota}</p>
          </div>
        </div>
      </div>
    </>
  );
}
