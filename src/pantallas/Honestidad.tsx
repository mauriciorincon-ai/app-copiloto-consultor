import { useIdioma, useT, type Idioma } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { TodaviaNo, PILA } from "../componentes/Ventana";
import { cortarTodo, type EstadoDeEscucha } from "../cuaderno";

/**
 * HONESTIDAD — «Qué vive en la memoria ahora mismo y qué salió de tu equipo».
 *
 * Referencia: `docs/diseno/honestidad.html`, estado **«así se ve hoy · sprint 1»** (miradas 12 y 13).
 *
 * Tras la fase 3 **sí vive algo en memoria**, y esta pantalla lo dice contado: dos anillos de
 * treinta segundos y una ventana de doce turnos, leídos de lo nativo y formateados allí. Escribir
 * las cifras en la interfaz sería la interfaz afirmando en vez de medir, que es justo lo que esta
 * pantalla existe para no hacer.
 *
 * El cero de la red sigue sin mantenerse por disciplina: no existe código capaz de abrir una
 * conexión. Y la cuenta del kill-switch subió de tres piezas a seis — no porque cambiara la
 * interfaz, sino porque un `match` sin comodín en `corte.rs` no dejó compilar hasta resolverlas.
 */

/**
 * Las piezas del kill-switch, en el mismo orden que `src-tauri/src/corte.rs`.
 *
 * Están escritas aquí y **no leídas de lo nativo**, y conviene decir por qué se acepta: el número
 * que importa no es este sino el que la app corta de verdad, y a ese lo vigila el compilador en
 * Rust. Si algún día se separan, lo que hay que arreglar es que este lado lo pregunte — queda
 * anotado, no disimulado.
 */
const PIEZAS_CORTADAS = 6;
const PIEZAS_TOTALES = 7;

export function Honestidad({ bytes, escucha }: { bytes: string; escucha: EstadoDeEscucha }) {
  const t = useT().cuaderno;
  const idioma = useIdioma();
  const [cifra, unidad = "B"] = bytes.split(" ");
  // Las cifras se formatean **aquí**, con el separador decimal del idioma. Lo nativo también las
  // manda escritas (`legible`, `ramLegible`) y siempre con coma: sirven para el log, que es
  // español, pero puestas en una pantalla inglesa dejaban «1,8 MB» dentro de «What lives in
  // memory now». La app promete ser bilingüe en TODO, y un separador decimal es interfaz.
  const ram = escucha.microfono.bytes + escucha.sistema.bytes + escucha.bytesDelTranscript;

  /** Un búfer que ya existe: se dice dónde vive y cuánto ocupa. */
  const buffer = (icono: string, que: string, donde: string, cuanto: string) => (
    <div className="buffer" key={que}>
      <Ic id={icono} s />
      <span className="que">{que}</span>
      <span className="donde">{donde}</span>
      <span className="cuanto">{cuanto}</span>
    </div>
  );

  /** Un búfer que todavía no existe. Ni verde ni escondido. */
  const pendiente = (icono: string, que: string) => (
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
              <span
                className="mono"
                style={{ color: ram === 0 ? "var(--ok)" : "var(--ink-2)" }}
              >
                RAM · {formatear(ram, idioma)}
              </span>
            </div>
            <div style={{ margin: "0 -4px" }}>
              {buffer("i-mic", t.bufMic, t.ringBuffer30, formatear(escucha.microfono.bytes, idioma))}
              {buffer("i-sistema", t.bufSistema, t.ringBuffer30, formatear(escucha.sistema.bytes, idioma))}
              {buffer("i-ojo", t.bufTranscript, t.ventana12, formatear(escucha.bytesDelTranscript, idioma))}
              {pendiente("i-pantalla", t.bufFrame)}
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

/**
 * Los bytes, escritos **con el separador decimal del idioma**.
 *
 * Desde la fase 5 se formatean todos aquí y ninguno se toma ya escrito de lo nativo. `red::
 * formatear` sigue existiendo y sigue poniendo coma: su sitio es el log, que es español. Lo que
 * no podía seguir era que esa coma llegara a una pantalla inglesa — «1,8 MB» dentro de «What
 * lives in memory now». Es la misma corrección que la fase 4 hizo en Corpus, traída a una
 * pantalla ya aprobada; su delta visual va a la mirada de esta fase.
 *
 * Base 1024, como `red::formatear`: el test de esta pantalla compara las dos contra los mismos
 * números.
 */
function formatear(bytes: number, idioma: Idioma): string {
  if (bytes === 0) return "0 B";
  const KB = 1024;
  const unidades: [number, string][] = [
    [KB * KB * KB, "GB"],
    [KB * KB, "MB"],
    [KB, "KB"],
  ];
  for (const [tamano, nombre] of unidades) {
    if (bytes >= tamano) {
      const v = Math.round((bytes / tamano) * 10) / 10;
      return `${new Intl.NumberFormat(idioma, { maximumFractionDigits: 1 }).format(v)} ${nombre}`;
    }
  }
  return `${bytes} B`;
}

export { formatear as formatearBytes };
