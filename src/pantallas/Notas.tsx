import { useEffect, useRef, useState, type CSSProperties, type ReactNode } from "react";
import { useIdioma, useT, type Idioma } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { hayTauri } from "../puente";
import { formatearBytes } from "./Honestidad";
import {
  abrirLaBandeja,
  anotarAcuerdo,
  borrarReunion,
  cambiarLaVentana,
  cerrarSinGuardar,
  conservarMisTurnos,
  decidirEnLaBandeja,
  decidirTodaLaBandeja,
  descartarPropuesta,
  DIAS_DE_MUESTRA,
  escribirNota,
  exportarReunion,
  fijarRetencion,
  fijarVentana,
  guardarLaReunion,
  guardarPropuesta,
  mostrarEnFinder,
  duracion,
  HORA_DE_MUESTRA,
  reloj,
  useBandeja,
  useCuaderno,
  useMuestraDeLaBandeja,
  useMuestraDelCuaderno,
  useQuedan,
  useReuniones,
  type FichaFijada,
  type Propuesta,
  type ReunionGuardada,
  type Retencion,
  type Ventana,
  type VistaDeLaBandeja,
  type VistaDeNotas,
  type VistaDelCuaderno,
} from "../notas";
import { etiquetaDe, iconoDe, origenDe, textoDe } from "../propuesta";

/**
 * NOTAS — «Lo que escribes tú. Es lo único de la reunión que llega al día siguiente.» (C9, ADR 015)
 *
 * Referencia: `docs/diseno/notas.html`, estados **«sprint 3 · durante»**, **«sprint 3 · al cerrar»**,
 * **«sprint 3 · el archivo»** y **«sprint 3 · exportar»** (mirada 19, aprobada el 2026-09-27), más
 * los tres que llegaron con el código: la lista vacía, los errores y la pregunta antes de borrar.
 *
 * **Qué vista se enseña lo decide el cuaderno, no la pantalla:** con la reunión abierta y escuchando,
 * «durante»; abierta y parada, «al cerrar»; sin reunión, «el archivo». Fuera de Tauri lo pide la URL
 * (`?estado=`), que es como el arnés de fidelidad recorre las cuatro.
 *
 * Aquí no se decide nada de lo que se guarda: la nota, los acuerdos y las fijadas viven en Rust
 * (`notas/`, módulo protegido) y esta pantalla solo los enseña y los manda.
 */
const VISTAS: VistaDeNotas[] = [
  "durante",
  "propuestas",
  "al-cerrar",
  "al-cerrar-bandeja",
  "al-cerrar-cero",
  "archivo",
  "exportar",
  "bandeja",
  "bandeja-llave",
  "vencida",
];
const RETENCIONES: Retencion[] = ["7d", "30d", "90d", "1a", "siempre"];
const VENTANAS: Ventana[] = ["0", "1h", "3h", "fin", "24h"];
/** En la bandeja caben tres filas; si guardaste alguna desde aquí, la última se queda a la vista. */
const FILAS_DE_LA_BANDEJA = 3;
const COLUMNA: CSSProperties = { display: "flex", flexDirection: "column", gap: "12px" };
const AYUDA: CSSProperties = { fontSize: "11.5px", color: "var(--ink-2)" };
const DIA = 86_400;
/** Una reunión que vence en dos semanas o menos se pinta apagada: está a punto de irse. */
const A_PUNTO_DE_IRSE = 14;

/** Los meses de la columna «fecha», como los escribe la maqueta: «20 sep». */
const MESES: Record<Idioma, string[]> = {
  es: ["ene", "feb", "mar", "abr", "may", "jun", "jul", "ago", "sep", "oct", "nov", "dic"],
  en: ["jan", "feb", "mar", "apr", "may", "jun", "jul", "aug", "sep", "oct", "nov", "dec"],
};

function vistaDeLaUrl(busqueda: string): VistaDeNotas {
  const pedida = new URLSearchParams(busqueda).get("estado");
  return VISTAS.find((v) => v === pedida) ?? "archivo";
}

export function Notas({ busqueda = "", alCambiarDeVista }: {
  busqueda?: string;
  /** El rail lo necesita para su chip: «Bandeja · vencida» mientras esta pantalla la enseña. */
  alCambiarDeVista?: (v: VistaDeNotas | null) => void;
}) {
  const pedida = vistaDeLaUrl(busqueda);
  const muestra = useMuestraDelCuaderno(pedida);
  const [cuaderno, volver] = useCuaderno(muestra);
  const [bandeja, bandejaLista, volverALaBandeja] = useBandeja(useMuestraDeLaBandeja(pedida));
  // La bandeja que venció mientras la mirabas: se queda en pantalla, apagada, hasta que te vas.
  const [vencida, setVencida] = useState<VistaDeLaBandeja | null>(null);
  // «Ir a tus reuniones»: el archivo, aunque haya bandeja esperando.
  const [alArchivo, setAlArchivo] = useState(false);
  const t = useT().notas;

  // Dentro del producto la vista la deciden el cuaderno y la bandeja; fuera, la URL. **Mientras
  // Rust no ha contestado, ninguna**: pintar «el archivo» un instante antes de «durante» era un
  // parpadeo que además se suscribía y se daba de baja por nada.
  const vista: VistaDeNotas | null = !hayTauri()
    ? pedida
    : cuaderno === null || !bandejaLista
      ? null
      : cuaderno.abierta
        ? cuaderno.escuchando
          ? "durante"
          : "al-cerrar"
        : vencida
          ? "vencida"
          : bandeja && !alArchivo
            ? bandeja.propuestas === null
              ? "bandeja-llave"
              : "bandeja"
            : "archivo";
  useEffect(() => alCambiarDeVista?.(vista), [vista, alCambiarDeVista]);
  const irAlArchivo = () => {
    setVencida(null);
    setAlArchivo(true);
  };

  return (
    <>
      <div className="titulo">
        <h1>{t.titulo}</h1>
        <p className="sub">{t.sub}</p>
      </div>
      {(vista === "durante" || vista === "propuestas") && cuaderno && (
        <Durante cuaderno={cuaderno} volver={volver} foco={new URLSearchParams(busqueda).get("foco") === "nota"} />
      )}
      {(vista === "al-cerrar" || vista === "al-cerrar-bandeja" || vista === "al-cerrar-cero") && cuaderno && (
        <AlCerrar cuaderno={cuaderno} volver={volver} />
      )}
      {vista === "bandeja" && bandeja && (
        <LaBandeja
          bandeja={bandeja}
          volver={volverALaBandeja}
          alVencer={(b) => setVencida(b)}
          irAlArchivo={irAlArchivo}
        />
      )}
      {vista === "bandeja-llave" && bandeja && (
        <BandejaConLlave bandeja={bandeja} volver={volverALaBandeja} irAlArchivo={irAlArchivo} />
      )}
      {vista === "vencida" && (vencida ?? bandeja) && (
        <BandejaVencida bandeja={(vencida ?? bandeja)!} irAlArchivo={irAlArchivo} />
      )}
      {(vista === "archivo" || vista === "exportar") && (
        <Archivo
          retencion={cuaderno?.retencion ?? "90d"}
          preguntaInicial={vista === "exportar" ? "exportar" : null}
          bandeja={hayTauri() ? bandeja : null}
          volverALaBandeja={() => setAlArchivo(false)}
        />
      )}
    </>
  );
}

// ---- una propuesta -------------------------------------------------------------------------------

/** Una fila de propuesta, como la maqueta: icono de su regla, la línea, de dónde salió y sus botones. */
function FilaDePropuesta({ p, aceptada, acciones }: { p: Propuesta; aceptada?: boolean; acciones: ReactNode }) {
  const t = useT().notas;
  const etiqueta = aceptada ? null : etiquetaDe(p, t);
  return (
    <div className={aceptada ? "propuesta aceptada" : "propuesta"}>
      {aceptada ? <Ic id="i-check-circle" relleno /> : <Ic id={iconoDe(p)} />}
      <span className="que">{textoDe(p, t)}</span>
      <span className="de">
        <span className="origen">{aceptada ? t.guardadaYaEnTuArchivo : origenDe(p, t)}</span>
        {etiqueta && <span>{`· ${etiqueta}`}</span>}
      </span>
      <span className="acciones">{acciones}</span>
    </div>
  );
}

function GuardarONo({ guardar, no }: { guardar: () => void; no: () => void }) {
  const t = useT().notas;
  return (
    <>
      <button className="btn mini primario" type="button" onClick={guardar}>
        {t.guardar}
      </button>
      <button className="btn mini" type="button" onClick={no}>
        {t.no}
      </button>
    </>
  );
}

/** Los nombres de la ventana de la bandeja: «al cerrar · 1 h · 3 h · fin del día · 24 h». */
function useNombreDeVentana(): (v: Ventana) => string {
  const t = useT().notas;
  return (v) =>
    v === "0" ? t.ventanaAlCerrar : v === "fin" ? t.ventanaFinDelDia : v === "1h" ? "1 h" : v === "3h" ? "3 h" : "24 h";
}

/** Las cinco ventanas, como botones de radio. `desactivadas` en la bandeja con llave. */
function Ventanas({ elegida, elegir, desactivadas = false, estilo }: {
  elegida: Ventana;
  elegir: (v: Ventana) => void;
  desactivadas?: boolean;
  estilo?: CSSProperties;
}) {
  const t = useT().notas;
  const nombre = useNombreDeVentana();
  return (
    <div className="ventanas" role="radiogroup" aria-label={t.cuantoQuieresEsperar} style={estilo}>
      {VENTANAS.map((v) => {
        const on = v === elegida;
        return (
          <button
            key={v}
            type="button"
            role="radio"
            aria-checked={on}
            disabled={desactivadas}
            className={on ? "op on" : "op"}
            onClick={() => elegir(v)}
          >
            {on && <Ic id="i-check-circle" s relleno />}
            {nombre(v)}
          </button>
        );
      })}
    </div>
  );
}

// ---- durante ---------------------------------------------------------------------------------

function Durante({ cuaderno, volver, foco }: { cuaderno: VistaDelCuaderno; volver: () => void; foco: boolean }) {
  const t = useT().notas;
  const [nota, setNota] = useState(cuaderno.nota);
  const [acuerdo, setAcuerdo] = useState("");
  const campo = useRef<HTMLTextAreaElement>(null);

  // ⌃⌥N: el cursor, al final de tu nota.
  useEffect(() => {
    const c = campo.current;
    if (!foco || !c) return;
    c.focus();
    c.setSelectionRange(c.value.length, c.value.length);
  }, [foco]);

  // Con propuestas, la nota cede alto: las propuestas van a lo ancho, debajo (mirada 20,
  // «sprint 3 · durante, con propuestas»). Sin ninguna, la pantalla es la de la mirada 19.
  const guardadas = cuaderno.resumen.propuestas;
  const conPropuestas = cuaderno.propuestas.length > 0 || guardadas > 0;

  return (
    <div style={{ ...COLUMNA, gap: conPropuestas ? "9px" : "12px" }}>
      {/* macOS no dejó proteger el cuaderno al empezar: se dice aquí, no solo en la consola
          (auditoría del S3, B4 — maquetado, no visto). */}
      {cuaderno.sinProteger && (
        <div className="franja warn" role="alert">
          <Ic id="i-alert" relleno />
          <div>
            <strong>{t.sinProteger}</strong>
          </div>
        </div>
      )}
      <div
        className="grid-2"
        style={{ gridTemplateColumns: "1.3fr 1fr", gap: "12px", alignItems: conPropuestas ? "stretch" : "start" }}
      >
        <div
          className="tarjeta"
          style={{ display: "flex", flexDirection: "column", gap: "8px", minHeight: conPropuestas ? undefined : "300px" }}
        >
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              <label htmlFor="tu-nota">{t.tuNota}</label>
            </h2>
            <span className="estado ok">
              <Ic id="i-ojo-off" s />
              <span>{t.clienteNoLaVe}</span>
            </span>
            <span className="tecla">
              <kbd>⌃</kbd>
              <kbd>⌥</kbd>
              <kbd>N</kbd>
            </span>
          </div>
          <textarea
            id="tu-nota"
            ref={campo}
            className="nota-campo crece"
            style={conPropuestas ? { minHeight: 0, flex: "1 1 auto" } : undefined}
            value={nota}
            onChange={(e) => {
              setNota(e.target.value);
              escribirNota(e.target.value);
            }}
          />
          {!conPropuestas && <p style={AYUDA}>{t.notaAyuda}</p>}
        </div>
        <div style={COLUMNA}>
          <div className="tarjeta" style={{ paddingBottom: "8px" }}>
            <div className="fila">
              <h2 className="seccion crece" style={{ margin: 0 }}>
                {t.fichasQueFijaste}
              </h2>
              <span className="tecla">
                <kbd>⌃</kbd>
                <kbd>⌥</kbd>
                <kbd>P</kbd>
              </span>
            </div>
            {/* Sin ninguna, la tarjeta se queda con su título y su tecla: ⌃⌥P ya dice cómo se llena. */}
            {cuaderno.fijadas.map((f, i) => (
              <Fijada key={`${f.titular}-${i}`} ficha={f} />
            ))}
          </div>
          <div className="tarjeta" style={{ paddingBottom: "8px" }}>
            <h2 className="seccion" style={{ margin: "0 0 4px" }}>
              <label htmlFor="acuerdo">{t.acuerdos}</label>
            </h2>
            {cuaderno.acuerdos.map((a, i) => (
              <div className="mas" key={`${a}-${i}`}>
                <Ic id="i-check-circle" s />
                <span className="titulos crece">{a}</span>
              </div>
            ))}
            <input
              id="acuerdo"
              className="acuerdo-campo"
              placeholder={t.escribeElAcuerdo}
              value={acuerdo}
              onChange={(e) => setAcuerdo(e.target.value)}
              onKeyDown={(e) => {
                if (e.key !== "Enter" || !acuerdo.trim()) return;
                e.preventDefault();
                void anotarAcuerdo(acuerdo).then(() => {
                  setAcuerdo("");
                  volver();
                });
              }}
            />
            <p style={{ ...AYUDA, marginTop: "4px" }}>{t.acuerdosAyuda}</p>
          </div>
        </div>
      </div>
      {conPropuestas && (
        <div className="tarjeta" style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              {t.tePropongo}
            </h2>
            {guardadas > 0 && (
              <span className="estado ok">
                <Ic id="i-check-circle" s relleno />
                <span>{`${guardadas} ${guardadas === 1 ? t.guardada : t.guardadas}`}</span>
              </span>
            )}
            <span className="tecla">
              <kbd>⌃</kbd>
              <kbd>⌥</kbd>
              <kbd>↵</kbd>
            </span>
          </div>
          {cuaderno.propuestas.map((p) => (
            <FilaDePropuesta
              key={p.id}
              p={p}
              acciones={
                <GuardarONo
                  guardar={() => void guardarPropuesta(p.id).then(volver)}
                  no={() => void descartarPropuesta(p.id).then(volver)}
                />
              }
            />
          ))}
          <p style={{ fontSize: "11.5px", color: "var(--ink-2)" }}>{cuaderno.lleno ? t.lleno : t.sonReglas}</p>
        </div>
      )}
      <div className="franja ok" role="status">
        <Ic id="i-check-circle" relleno />
        <div>
          {conPropuestas ? (
            <strong>{t.soloSobreviveLoTuyo}</strong>
          ) : (
            <>
              <strong>{t.sobrevive}</strong>
              <p>{t.sobreviveDetalle}</p>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

/** Una fijada: su titular y, a la derecha, su sección o —si no tiene— su unidad («caso»). */
function Fijada({ ficha }: { ficha: FichaFijada }) {
  const unidades = useT().banda.unidades;
  return (
    <div className="mas">
      <Ic id="i-pin-lleno" s />
      <span className="titulos crece">{ficha.titular}</span>
      <span className="n">{ficha.seccion ?? (ficha.unidad ? unidades[ficha.unidad] : "")}</span>
    </div>
  );
}

/**
 * «Dónde»: la carpeta privada de la app, nombrada y no escrita como ruta, y «Mostrar en Finder»
 * para verla (ADR 016, decisión A). Con `archivo`, Finder la abre con esa reunión seleccionada.
 */
function DondeViven({ archivo }: { archivo?: string }) {
  const t = useT().notas;
  return (
    <div className="fila" style={{ gap: "8px" }}>
      <span className="crece">{t.carpetaDeLaApp}</span>
      <button className="btn mini" type="button" onClick={() => void mostrarEnFinder(archivo).catch(() => undefined)}>
        {t.mostrarEnFinder}
      </button>
    </div>
  );
}

// ---- al cerrar -------------------------------------------------------------------------------

function AlCerrar({ cuaderno, volver }: { cuaderno: VistaDelCuaderno; volver: () => void }) {
  const t = useT().notas;
  const idioma = useIdioma();
  const [fallo, setFallo] = useState(false);
  const guardar = () =>
    guardarLaReunion().then(
      () => {
        setFallo(false);
        volver();
      },
      () => setFallo(true),
    );
  const [ventana, setVentana] = useState<Ventana>(cuaderno.ventana);
  useEffect(() => setVentana(cuaderno.ventana), [cuaderno.ventana]);
  const r = cuaderno.resumen;
  const kb = (b: number) => formatearBytes(b, idioma);
  const fila = (icono: string, que: string, donde: string, cuanto: string, muerto = false) => (
    <div className={muerto ? "buffer muerto" : "buffer"} key={que}>
      <Ic id={icono} s />
      <span className="que">{que}</span>
      <span className="donde">{donde}</span>
      <span className="cuanto">{cuanto}</span>
    </div>
  );
  const p = cuaderno.previsto;
  const linea = p
    ? `${p.fecha}${p.cliente ? ` · ${p.cliente}` : ""} · ${p.minutos} ${t.min} → ${p.archivo}`
    : null;

  return (
    <div style={COLUMNA}>
      {fallo && (
        <div className="franja err" role="alert">
          <Ic id="i-x-circle" relleno />
          <div>
            <strong>{t.noSeGuardo}</strong>
            <p>{t.noSeGuardoDetalle}</p>
            <div className="fila" style={{ gap: "8px", marginTop: "8px" }}>
              <button className="btn mini" type="button" onClick={() => void guardar()}>
                {t.intentarOtraVez}
              </button>
            </div>
          </div>
        </div>
      )}
      <div className="grid-2" style={{ gap: "12px", alignItems: "start" }}>
        <div className="tarjeta" style={{ paddingBottom: "8px" }}>
          <h2 className="seccion" style={{ margin: "0 0 6px" }}>
            {t.seVaAGuardar}
          </h2>
          {fila("i-nota", t.tusNotas, `${r.parrafos} ${t.parrafos}`, kb(r.bytesNota))}
          {fila("i-pin-lleno", t.fichasFijadas, `${r.fijadas} · ${t.tituloYFuente}`, kb(r.bytesFijadas))}
          {fila("i-check-circle", t.acuerdos, `${r.acuerdos} · ${t.marcadosPorTi}`, kb(r.bytesAcuerdos))}
          {cuaderno.conservarMisTurnos &&
            fila("i-mic", t.tusTurnosEnTexto, `${r.turnos} · ${t.loQueDijisteTu}`, kb(r.bytesTurnos))}
          {/* Un interruptor de verdad (`role="switch"`), como los de Sesión e IA. */}
          <button
            type="button"
            role="switch"
            aria-checked={cuaderno.conservarMisTurnos}
            className={cuaderno.conservarMisTurnos ? "conm on" : "conm off"}
            style={{ background: "none", border: 0, padding: 0, fontFamily: "inherit", margin: "8px 2px 2px" }}
            onClick={() => void conservarMisTurnos(!cuaderno.conservarMisTurnos).then(volver)}
          >
            <span className="track" />
            <span className="etq">
              {cuaderno.conservarMisTurnos && <Ic id="i-check-circle" s relleno />}
              <span>{t.conservarMisTurnos}</span>
            </span>
          </button>
          <p style={{ ...AYUDA, marginTop: "4px" }}>
            {cuaderno.conservarMisTurnos ? t.conservarEncendido : t.conservarApagado} <b>{t.audioNunca}</b>
            {t.audioNuncaCola}
          </p>
        </div>
        <div className="tarjeta" style={{ paddingBottom: "8px" }}>
          <h2 className="seccion" style={{ margin: "0 0 6px" }}>
            {t.muereAlCerrar}
          </h2>
          {fila("i-sistema", t.vozDelCliente, t.sinInterruptor, "0 B", true)}
          {fila("i-ojo", t.susTurnos, `${cuaderno.turnosDelCliente} ${t.turnos}`, "0 B", true)}
          {fila("i-pantalla", t.loLeido, `${cuaderno.lecturas} ${t.lecturas}`, "0 B", true)}
          {fila("i-mic", t.tuPropioAudio, t.tambienMuere, "0 B", true)}
          {/* Con la franja de la bandeja debajo, la línea se va: la pantalla no cabría (mirada 20). */}
          {r.sinDecidir === 0 && <p style={{ ...AYUDA, marginTop: "10px" }}>{t.sinCasilla}</p>}
        </div>
      </div>
      {r.sinDecidir > 0 && (
        <FranjaDeLaBandeja
          quedan={r.sinDecidir}
          guardadas={r.propuestas}
          ventana={ventana}
          elegir={(v) => {
            setVentana(v);
            void fijarVentana(v).then(volver);
          }}
        />
      )}
      <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
        {linea && (
          <span className="mono" style={{ color: "var(--ink-2)", fontSize: "11.5px" }}>
            {linea}
          </span>
        )}
        <div className="fila" style={{ gap: "8px" }}>
          <span className="crece" />
          <button className="btn" type="button" onClick={() => void cerrarSinGuardar().then(volver)}>
            {t.cerrarSinGuardar}
          </button>
          <button className="btn primario" type="button" onClick={() => void guardar()}>
            <Ic id="i-candado" s />
            {t.guardarCifrado}
          </button>
        </div>
      </div>
    </div>
  );
}

/**
 * Al cerrar con propuestas sin decidir (mirada 20): cuántas quedan, la ventana —que se elige aquí
 * mismo— y qué pasará al pulsar «Guardar cifrado y cerrar». En «al cerrar», gris: no hay bandeja.
 */
function FranjaDeLaBandeja({ quedan, guardadas, ventana, elegir }: {
  quedan: number;
  guardadas: number;
  ventana: Ventana;
  elegir: (v: Ventana) => void;
}) {
  const t = useT().notas;
  const cero = ventana === "0";
  const lasTuyas = guardadas > 0 ? `${t.las} ${guardadas} ${t.queGuardasteVan}` : "";
  return (
    <div className={cero ? "franja mute" : "franja warn"} role="status">
      <Ic id="i-reloj" />
      <div>
        <strong>{`${quedan} ${cero ? t.muerenAlCerrar : t.esperaranEnLaBandeja}`}</strong>
        <Ventanas elegida={ventana} elegir={elegir} estilo={{ margin: "6px 0 4px" }} />
        <p>{cero ? `${t.ceroDetalle}${lasTuyas && ` ${lasTuyas}`}` : `${lasTuyas && `${lasTuyas} `}${t.enLaBandejaDetalle}`}</p>
      </div>
    </div>
  );
}

// ---- la bandeja (ADR 016 §4) ----------------------------------------------------------------------

/** «Ya murió, al cerrar»: lo que no espera ninguna ventana, en 0 B. */
function YaMurio({ titulo, bandeja }: { titulo: string; bandeja?: ReactNode }) {
  const t = useT().notas;
  const fila = (icono: string, que: string) => (
    <div className="buffer muerto" key={que}>
      <Ic id={icono} s />
      <span className="que">{que}</span>
      <span className="cuanto">0 B</span>
    </div>
  );
  return (
    <div className="tarjeta" style={{ paddingBottom: "6px" }}>
      <h2 className="seccion" style={{ margin: "0 0 4px" }}>
        {titulo}
      </h2>
      {fila("i-mic", t.audioDosPistas)}
      {fila("i-ojo", t.transcriptDelCliente)}
      {fila("i-pantalla", t.lecturasDePantalla)}
      {bandeja}
    </div>
  );
}

function EsUnaVentana() {
  const t = useT().notas;
  return (
    <div className="franja warn" role="status">
      <Ic id="i-alert" relleno />
      <div>
        <strong>{t.esUnaVentana}</strong>
        <p>
          {t.alLlegarACero} <b>{t.aunqueNuncaVuelvas}</b>
          {t.elPrecio}
        </p>
      </div>
    </div>
  );
}

function IrAlArchivo({ irAlArchivo, mini }: { irAlArchivo: () => void; mini?: boolean }) {
  const t = useT().notas;
  return (
    <button className={mini ? "btn mini" : "btn"} type="button" onClick={irAlArchivo}>
      <Ic id="i-nota" s />
      {t.irATusReuniones}
    </button>
  );
}

/** La bandeja, abierta: la cuenta atrás, lo que espera tu decisión y la ventana (maqueta «sprint 3 · la bandeja»). */
function LaBandeja({ bandeja, volver, alVencer, irAlArchivo }: {
  bandeja: VistaDeLaBandeja;
  volver: () => void;
  alVencer: (b: VistaDeLaBandeja) => void;
  irAlArchivo: () => void;
}) {
  const t = useT().notas;
  const idioma = useIdioma();
  const quedan = useQuedan(bandeja.vence) ?? 0;
  const [ventana, setVentana] = useState<Ventana>(bandeja.ventana);
  useEffect(() => setVentana(bandeja.ventana), [bandeja.ventana]);
  // Venció mientras la mirabas: la pantalla lo dice en vez de desaparecer.
  useEffect(() => {
    if (hayTauri() && quedan === 0) alVencer(bandeja);
  }, [quedan, bandeja, alVencer]);

  const pendientes = bandeja.propuestas ?? [];
  const total = pendientes.length + bandeja.guardadas.length;
  const ultima = bandeja.guardadas[bandeja.guardadas.length - 1];
  const visibles = ultima ? pendientes.slice(0, FILAS_DE_LA_BANDEJA - 1) : pendientes.slice(0, FILAS_DE_LA_BANDEJA);
  const [larga] = duracion(quedan, t);
  const decidir = (i: number, guardar: boolean) => void decidirEnLaBandeja(bandeja.archivo, i, guardar).then(volver);

  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
      <div className="cuenta" role="status">
        <Ic id="i-reloj" />
        <span className="reloj">{reloj(quedan)}</span>
        <span className="txt">
          <b>{t.laReunionCerro}</b> {`${t.teQuedan} ${larga} ${t.paraDecidirDeLas} ${total} ${t.propuestasAlLlegar}`}
        </span>
      </div>
      <div className="grid-2" style={{ gridTemplateColumns: "1.5fr 1fr", gap: "12px", alignItems: "start" }}>
        <div className="tarjeta" style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              {t.esperanTuDecision}
            </h2>
            <span className="mono" style={{ color: "var(--ink-2)", fontSize: "11px" }}>
              {`${total} · ${formatearBytes(bandeja.bytes, idioma)}`}
            </span>
          </div>
          {visibles.map((p, i) => (
            <FilaDePropuesta
              key={`${p.hora}-${i}`}
              p={p}
              acciones={<GuardarONo guardar={() => decidir(i, true)} no={() => decidir(i, false)} />}
            />
          ))}
          {ultima && (
            <FilaDePropuesta
              p={ultima}
              aceptada
              acciones={
                <span className="estado ok">
                  <Ic id="i-check-circle" s relleno />
                  <span>{t.guardada}</span>
                </span>
              }
            />
          )}
          <div className="fila">
            <span className="mono crece" style={{ color: "var(--ink-2)", fontSize: "11px" }}>
              {`${visibles.length + (ultima ? 1 : 0)} ${t.de} ${total}`}
            </span>
            <button
              className="btn mini"
              type="button"
              disabled={pendientes.length === 0}
              onClick={() => void decidirTodaLaBandeja(bandeja.archivo, true).then(volver)}
            >
              {t.guardarTodas}
            </button>
            <button
              className="btn mini"
              type="button"
              disabled={pendientes.length === 0}
              onClick={() => void decidirTodaLaBandeja(bandeja.archivo, false).then(volver)}
            >
              {t.descartarTodas}
            </button>
          </div>
          <div className="fila">
            <span className="crece" />
            <IrAlArchivo irAlArchivo={irAlArchivo} mini />
          </div>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
          <YaMurio
            titulo={t.yaMurioAlCerrar}
            bandeja={
              <p style={{ fontSize: "11px", color: "var(--ink-2)", marginTop: "6px" }}>
                {t.bandejaGuarda} <b>{t.frases}</b>
                {t.noLaReunion}
              </p>
            }
          />
          <div className="tarjeta" style={{ padding: "10px 14px" }}>
            <h2 className="seccion" style={{ margin: "0 0 6px" }}>
              {t.cuantoQuieresEsperar}
            </h2>
            <Ventanas
              elegida={ventana}
              elegir={(v) => {
                setVentana(v);
                void cambiarLaVentana(bandeja.archivo, v).then(volver);
              }}
            />
          </div>
          <EsUnaVentana />
        </div>
      </div>
    </div>
  );
}

/** Una bandeja de otra sesión de la app: la cuenta se ve; lo que dice pide Touch ID. */
function BandejaConLlave({ bandeja, volver, irAlArchivo }: {
  bandeja: VistaDeLaBandeja;
  volver: () => void;
  irAlArchivo: () => void;
}) {
  const t = useT().notas;
  const idioma = useIdioma();
  const quedan = useQuedan(bandeja.vence) ?? 0;
  const [larga] = duracion(quedan, t);
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
      <div className="cuenta" role="status">
        <Ic id="i-reloj" />
        <span className="reloj">{reloj(quedan)}</span>
        <span className="txt">
          <b>{t.laReunionCerro}</b> {`${t.teQuedan} ${larga} ${t.paraDecidirCerrada}`}
        </span>
      </div>
      <div className="grid-2" style={{ gridTemplateColumns: "1.5fr 1fr", gap: "12px", alignItems: "start" }}>
        <div className="tarjeta" style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              {t.esperanTuDecision}
            </h2>
            <span className="mono" style={{ color: "var(--ink-2)", fontSize: "11px" }}>
              {formatearBytes(bandeja.bytes, idioma)}
            </span>
          </div>
          <div className="fila">
            <Ic id="i-candado" s />
            <strong className="crece">{t.cerradaConLlave}</strong>
          </div>
          <p style={{ fontSize: "12.5px", color: "var(--ink-2)" }}>{t.paraLeerla}</p>
          <div className="fila">
            <span className="mono crece" style={{ color: "var(--ink-2)", fontSize: "11px" }}>
              {bandeja.mas > 0 ? `${t.bandejasDetras} ${bandeja.mas}` : ""}
            </span>
            <IrAlArchivo irAlArchivo={irAlArchivo} />
            <button
              className="btn primario"
              type="button"
              onClick={() => void abrirLaBandeja(bandeja.archivo, idioma).then(volver, volver)}
            >
              <Ic id="i-llave" s />
              {t.abrirConTouchId}
            </button>
          </div>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
          <div className="tarjeta" style={{ padding: "10px 14px" }}>
            <h2 className="seccion" style={{ margin: "0 0 6px" }}>
              {t.cuantoQuieresEsperar}
            </h2>
            <Ventanas elegida={bandeja.ventana} elegir={() => undefined} desactivadas />
          </div>
          <EsUnaVentana />
        </div>
      </div>
    </div>
  );
}

/** La bandeja al llegar a cero, si la estabas mirando (maqueta «sprint 3 · bandeja vencida»). */
function BandejaVencida({ bandeja, irAlArchivo }: { bandeja: VistaDeLaBandeja; irAlArchivo: () => void }) {
  const t = useT().notas;
  const idioma = useIdioma();
  const murieron = bandeja.propuestas?.length ?? 0;
  const guardadas = bandeja.guardadas.length;
  const d = new Date(bandeja.vence * 1000);
  const hora = hayTauri()
    ? `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`
    : HORA_DE_MUESTRA;
  // «la reunión del 20 sep»: la fecha va en el nombre del archivo de la reunión.
  const f = /(\d{4})-(\d{2})-(\d{2})/.exec(bandeja.archivo);
  const mes = f ? MESES[idioma][Number(f[2]) - 1] : "";
  const delDia = f ? `${Number(f[3])} ${idioma === "en" ? mes.charAt(0).toUpperCase() + mes.slice(1) : mes}` : "";
  const primera = `${t.las} ${murieron} ${t.queNoDecidisteMurieron}`;
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
      <div className="cuenta vencida" role="status">
        <Ic id="i-reloj" />
        <span className="reloj">{reloj(0)}</span>
        <span className="txt">
          <b>{`${t.laBandejaVencio} ${hora} ${t.ySeBorroSola}`}</b>{" "}
          {guardadas > 0
            ? `${primera} ${t.lasMinuscula} ${guardadas} ${t.queGuardasteEstan}`
            : primera.replace(/;$/, ".")}
        </span>
      </div>
      <div className="grid-2" style={{ gridTemplateColumns: "1.5fr 1fr", gap: "12px", alignItems: "start" }}>
        <div className="tarjeta" style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              {t.esperanTuDecision}
            </h2>
            <span className="mono" style={{ color: "var(--ink-2)", fontSize: "11px" }}>
              0 · 0 B
            </span>
          </div>
          {guardadas > 0 && (
            <p style={{ fontSize: "12.5px", color: "var(--ink-2)" }}>
              {idioma === "en"
                ? `${t.nadaLoQueGuardaste} ${delDia} ${t.cifradoConSuRetencion}`
                : `${t.nadaLoQueGuardaste} ${delDia}${t.cifradoConSuRetencion}`}
            </p>
          )}
          <div className="fila">
            <span className="crece" />
            <IrAlArchivo irAlArchivo={irAlArchivo} />
          </div>
        </div>
        <YaMurio
          titulo={t.yaMurio}
          bandeja={
            <>
              <div className="buffer muerto">
                <Ic id="i-reloj" s />
                <span className="que">{`${t.laBandejaPunto} ${murieron} ${t.propuestas}`}</span>
                <span className="cuanto">0 B</span>
              </div>
              <p style={{ fontSize: "11px", color: "var(--ink-2)", marginTop: "6px" }}>{t.lasTresPrimeras}</p>
            </>
          }
        />
      </div>
    </div>
  );
}

// ---- el archivo ------------------------------------------------------------------------------

type Pregunta = "exportar" | "borrar" | "no-exporto" | null;

function Archivo({ retencion, preguntaInicial, bandeja, volverALaBandeja }: {
  retencion: Retencion;
  preguntaInicial: Pregunta;
  /** La bandeja que sigue esperando mientras miras tu archivo: un botón te devuelve a ella. */
  bandeja: VistaDeLaBandeja | null;
  volverALaBandeja: () => void;
}) {
  const t = useT().notas;
  const quedanBandeja = useQuedan(bandeja?.vence ?? null);
  const idioma = useIdioma();
  const [lista, volverALista] = useReuniones();
  const [elegida, setElegida] = useState<string | null>(null);
  const [pregunta, setPregunta] = useState<Pregunta>(preguntaInicial);
  const [mia, setMia] = useState<Retencion>(retencion);
  useEffect(() => setMia(retencion), [retencion]);

  const ahora = Date.now() / 1000;
  const dias = (r: ReunionGuardada, i: number): number | null => {
    if (!hayTauri()) return DIAS_DE_MUESTRA[i] ?? null;
    return r.vence === 0 ? null : Math.max(0, Math.ceil((r.vence - ahora) / DIA));
  };
  const reunion = lista.reuniones.find((r) => r.archivo === elegida) ?? lista.reuniones[0];
  const indice = reunion ? lista.reuniones.indexOf(reunion) : -1;
  const fecha = (s: number) => {
    const d = new Date(s * 1000);
    return `${String(d.getDate()).padStart(2, "0")} ${MESES[idioma][d.getMonth()]}`;
  };
  const seBorraEn = (r: ReunionGuardada, i: number) => {
    const d = dias(r, i);
    return d === null ? t.siempre : `${d} d`;
  };

  const retenciones = (
    <div className="tarjeta" style={{ padding: "10px 14px" }}>
      <h2 className="seccion" style={{ margin: "0 0 6px" }}>
        {t.cuantoViven}
      </h2>
      <div className="ventanas" role="radiogroup" aria-label={t.cuantoViven}>
        {RETENCIONES.map((r) => {
          const on = r === mia;
          const nombre = r === "1a" ? t.unAnio : r === "siempre" ? t.siempre : r.replace("d", " d");
          return (
            <button
              key={r}
              type="button"
              role="radio"
              aria-checked={on}
              className={on ? "op on" : "op"}
              onClick={() => {
                setMia(r);
                void fijarRetencion(r);
              }}
            >
              {on && <Ic id="i-check-circle" s relleno />}
              {nombre}
            </button>
          );
        })}
      </div>
      <p style={{ ...AYUDA, marginTop: "8px" }}>{t.cadaReunion}</p>
    </div>
  );

  if (!reunion) {
    return (
      <div style={COLUMNA}>
        <div className="grid-2" style={{ gridTemplateColumns: "1.25fr 1fr", gap: "12px", alignItems: "start" }}>
          <div className="tarjeta">
            <div className="fila">
              <Ic id="i-candado" s />
              <strong className="crece">{t.sinReuniones}</strong>
            </div>
            <p style={{ ...AYUDA, marginTop: "4px" }}>{t.sinReunionesDetalle}</p>
            <table className="tabla" style={{ marginTop: "8px" }}>
              <tbody>
                <tr>
                  <td>{t.donde}</td>
                  <td>
                    <DondeViven />
                  </td>
                </tr>
                <tr>
                  <td>{t.llave}</td>
                  <td>{t.enTuLlavero}</td>
                </tr>
              </tbody>
            </table>
          </div>
          {retenciones}
        </div>
      </div>
    );
  }

  const d = dias(reunion, indice);
  const franja =
    pregunta === "exportar" ? (
      <div className="franja warn" role="alertdialog" aria-label={t.exportarATexto}>
        <Ic id="i-alert" relleno />
        <div>
          <strong>{t.exportarPregunta}</strong>
          <p>{t.exportarDetalle}</p>
          <div className="fila" style={{ gap: "8px", marginTop: "8px" }}>
            <button
              className="btn mini"
              type="button"
              onClick={() =>
                exportarReunion(reunion.archivo, idioma).then(
                  () => {
                    setPregunta(null);
                    volverALista();
                  },
                  () => setPregunta("no-exporto"),
                )
              }
            >
              <Ic id="i-doc" s />
              {t.exportarSinCifrado}
            </button>
            <button className="btn mini" type="button" onClick={() => setPregunta(null)}>
              {t.cancelar}
            </button>
          </div>
        </div>
      </div>
    ) : pregunta === "borrar" ? (
      <div className="franja err" role="alertdialog" aria-label={t.borrarAhora}>
        <Ic id="i-x-circle" relleno />
        <div>
          <strong>{t.borrarPregunta}</strong>
          <p>{t.borrarDetalle}</p>
          <div className="fila" style={{ gap: "8px", marginTop: "8px" }}>
            <button
              className="btn mini"
              type="button"
              onClick={() =>
                void borrarReunion(reunion.archivo).then(() => {
                  setPregunta(null);
                  setElegida(null);
                  volverALista();
                })
              }
            >
              <Ic id="i-basura" s />
              {t.borrar}
            </button>
            <button className="btn mini" type="button" onClick={() => setPregunta(null)}>
              {t.cancelar}
            </button>
          </div>
        </div>
      </div>
    ) : pregunta === "no-exporto" ? (
      <div className="franja err" role="alert">
        <Ic id="i-x-circle" relleno />
        <div>
          <strong>{t.noSeExporto}</strong>
          <p>{t.noSeExportoDetalle}</p>
        </div>
      </div>
    ) : (
      <div className="franja warn" role="status">
        <Ic id="i-alert" relleno />
        <div>
          <strong>{t.exportarQuita}</strong>
          <p>
            {t.exportarDetalle} {t.seAvisaAntes}
          </p>
        </div>
      </div>
    );

  return (
    <div style={COLUMNA}>
      <div className="grid-2" style={{ gridTemplateColumns: "1.25fr 1fr", gap: "12px", alignItems: "start" }}>
        <div className="tarjeta" style={{ paddingBottom: "6px" }}>
          <div className="fila">
            <h2 className="seccion crece" style={{ margin: 0 }}>
              {reunion.archivo}
            </h2>
            <span className="estado ok">
              <Ic id="i-candado" s />
              <span>{t.cifrado}</span>
            </span>
          </div>
          <table className="tabla">
            <tbody>
              <tr>
                <td>{t.donde}</td>
                <td>
                  <DondeViven archivo={reunion.archivo} />
                </td>
              </tr>
              <tr>
                <td>{t.tamano}</td>
                <td className="mono">{formatearBytes(reunion.bytes, idioma)}</td>
              </tr>
              <tr>
                <td>{t.llave}</td>
                <td>{t.enTuLlavero}</td>
              </tr>
              <tr>
                <td>{t.retencion}</td>
                <td>
                  <span className="estado warn">
                    <Ic id="i-reloj" s />
                    <span>{d === null ? t.siempre : `${t.seBorraSoloEn} ${d} ${t.dias}`}</span>
                  </span>
                </td>
              </tr>
              <tr>
                <td>{t.delCliente}</td>
                <td>{t.nadaDelCliente}</td>
              </tr>
            </tbody>
          </table>
          <div className="fila" style={{ gap: "8px", marginTop: "10px" }}>
            <button className="btn" type="button" onClick={() => setPregunta("exportar")}>
              <Ic id="i-doc" s />
              {t.exportarATexto}
            </button>
            <span className="crece" />
            <button className="btn" type="button" onClick={() => setPregunta("borrar")}>
              <Ic id="i-basura" s />
              {t.borrarAhora}
            </button>
          </div>
        </div>
        <div style={COLUMNA}>
          {franja}
          {retenciones}
        </div>
      </div>
      <div className="tarjeta" style={{ paddingBottom: "6px" }}>
        {bandeja && quedanBandeja !== null && quedanBandeja > 0 ? (
          <div className="fila" style={{ marginBottom: "4px" }}>
            <h2 className="seccion crece" style={{ margin: 0 }}>
              {t.reunionesGuardadas}
            </h2>
            <button className="btn mini" type="button" onClick={volverALaBandeja}>
              <Ic id="i-reloj" s />
              {`${t.bandejaChip} ${duracion(quedanBandeja, t)[1]}`}
            </button>
          </div>
        ) : (
          <h2 className="seccion" style={{ margin: "0 0 4px" }}>
            {t.reunionesGuardadas}
          </h2>
        )}
        <table className="tabla">
          <thead>
            <tr>
              <th>{t.colArchivo}</th>
              <th>{t.colFecha}</th>
              <th className="num">{t.colTamano}</th>
              <th className="num">{t.colSeBorraEn}</th>
            </tr>
          </thead>
          <tbody>
            {lista.reuniones.map((r, i) => {
              const quedan = dias(r, i);
              return (
                <tr key={r.archivo} className={quedan !== null && quedan <= A_PUNTO_DE_IRSE ? "apagada" : undefined}>
                  <td className="mono">
                    <button
                      className="elegir"
                      type="button"
                      aria-current={r.archivo === reunion.archivo ? "true" : undefined}
                      onClick={() => {
                        setElegida(r.archivo);
                        setPregunta(null);
                      }}
                    >
                      {r.archivo}
                    </button>
                  </td>
                  <td className="mono">{fecha(r.guardada)}</td>
                  <td className="num">{formatearBytes(r.bytes, idioma)}</td>
                  <td className="num">{seBorraEn(r, i)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
}
