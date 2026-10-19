import { useMemo, useState } from "react";
import { useIdioma, useT, useTeclas } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { Fila, Funciona, PILA } from "../componentes/Ventana";
import { hayTauri } from "../puente";
import type { Bilingue } from "../radar";
import {
  elegirCliente,
  empezarPresencial,
  empezarSoloNotas,
  estadoDeSesion,
  muestraDelCliente,
  responderNda,
  revisarNda,
  useEsteCliente,
  type LaBandera,
  type VistaDelCliente,
} from "../jurisdiccion";
import {
  empezarAEscuchar,
  dejarDeEscuchar,
  fijarIdiomaDePista,
  fijarLaVoz,
  lecturaAutomatica,
  useEntradaDeAudio,
  useIdiomasDePista,
  useQueSabeTranscribir,
  usePantalla,
  useBytesALaRed,
  useVoz,
  type EstadoDeEscucha,
  type EstadoDePista,
  type EstadoDeLaPantalla,
  type Reunion,
  type Salida,
} from "../cuaderno";
import { invasivos, useRadarDeTuMac, type EnTuMac } from "../radar";
import { estadoDelModelo, motivo } from "./Idioma";
import { useTextosDePresencial } from "../i18n/presencial-nuevo";
import { esWindows, usePlataforma } from "../plataforma";
import { useAntesDeEmpezar } from "../reunionAnterior";
import {
  asentarDesplazamiento,
  bandaCerradaDesdeLaUrl,
  desplazamientoDesdeLaUrl,
  entendidoElAvisoDeArriba,
  fijarPosicionDeLaBanda,
  mostrarLaBanda,
  PASO_DE_LA_ALTURA,
  useFranja,
  type Borde,
  type LaFranja,
} from "../franja";
import { useTextosDeWindows } from "../i18n/windows";
import { ElModoTransparente } from "../componentes/ModoTransparente";

/**
 * SESIÓN — «Antes de empezar».
 *
 * Referencia: `docs/diseno/sesion.html`, estados **«así se ve hoy · sprint 2 · pista caída»**
 * (mirada 17) y **«… · la pantalla»** (mirada 17-quater), y los porqués de `kit.html` §8-ter.
 *
 * **Las filas MIDEN.** Hasta el sprint 002 las dos pistas decían «Funciona» pase lo que pase: el
 * estado estaba escrito a mano, y si macOS negaba el audio del sistema el consultor se enteraba al
 * terminar la reunión, al ver que faltaba medio transcript (hallazgo M2 del sprint 001). Ahora,
 * con la sesión en marcha, una pista que no abrió lo dice **con su porqué y su salida**; y la fila
 * «Escucha las dos pistas» deja de afirmarlo cuando es falso.
 *
 * Fuera de una sesión las pistas no están abiertas y eso no es una avería: se enseña lo que la app
 * sabe hacer, como en el sprint 001.
 */

export function Sesion({
  reunion,
  escucha,
  salida,
  radarDeMuestra = false,
  busqueda = "",
  ensayoSinGuardar = false,
}: {
  reunion: Reunion;
  escucha: EstadoDeEscucha;
  salida: Salida;
  radarDeMuestra?: boolean;
  /**
   * Hay un ensayo terminado que no guardaste (auditoría del S4, M10): «Iniciar sesión» lo descarta, así que
   * se dice antes, en una línea bajo el botón (en el sitio de la promesa del corte).
   */
  ensayoSinGuardar?: boolean;
  /** Fuera de Tauri, el estado de `sesion.html` que pide el arnés de fidelidad por la URL. */
  busqueda?: string;
}) {
  const t = useT().cuaderno;
  const tc = useT().cliente;
  const pantalla = usePantalla();
  const bytes = useBytesALaRed();
  const fuera = !hayTauri();
  const pedido = estadoDeSesion(busqueda);
  const muestra = useMemo(() => muestraDelCliente(pedido), [pedido]);
  const [vista, setVista] = useEsteCliente(muestra);
  // **Antes de empezar otra sesión** (sprint 013, decisión 2a): en Windows, si la reunión anterior sigue abierta con algo
  // tuyo, se pregunta «Guardar · Descartar · Cancelar» antes de tocar la llave de notas ni la tarea de borrado. En el Mac
  // y fuera de Tauri `antes` empieza directo y el cuadro no existe.
  const { antes, cuadro } = useAntesDeEmpezar();
  // El modo transparente (sprint 012, ADR 024): solo Windows, y solo con un cliente elegido. En el Mac, ni una fila.
  const [plataforma] = usePlataforma();
  const transparente =
    esWindows(plataforma) && vista.elegido ? (
      <ElModoTransparente
        vista={vista}
        setVista={setVista}
        abierta={escucha.escuchando}
      />
    ) : null;
  const [preguntando, setPreguntando] = useState(fuera && pedido === "pregunta");
  const [viendoClausula, setViendoClausula] = useState(fuera && pedido === "clausula");
  /**
   * **El modo que eliges antes de empezar** (sprint 005, ADR 020): la reunión en este Mac o presencial. No se
   * recuerda (ADR 002, enmienda 9): cada sesión empieza en la reunión. Con la sesión en marcha manda Rust.
   */
  const [modo, setModo] = useState<"reunion" | "presencial">(
    fuera && (pedido === "presencial" || pedido === "presencial-marcha") ? "presencial" : "reunion",
  );
  const hayEco = salida.salida === "altavoces" || salida.salida === "altavoz-externo";
  const caida = (p: EstadoDePista) => escucha.escuchando && !p.abierta;
  /**
   * **La banda, arriba o abajo** (sprint 004). Fuera de Tauri, la de `sesion.html` · sprint 4: arriba,
   * y el aviso de la primera vez solo en «la primera vez» (`?estado=aviso`).
   */
  const franja = useFranja({
    borde: "arriba",
    barra: 0,
    avisoVisto: pedido !== "aviso",
    desplazamiento: desplazamientoDesdeLaUrl(busqueda),
    bandaAbierta: !bandaCerradaDesdeLaUrl(busqueda),
  });
  const [avisoCerrado, setAvisoCerrado] = useState(false);
  const verAviso = franja.borde === "arriba" && !franja.avisoVisto && !avisoCerrado;

  /**
   * **EL RADAR CORAL EN SESIÓN** (C14, fase 4 del sprint 002) — «software invasivo en tu Mac».
   *
   * Antes de empezar, si algo vigila este Mac, **la pantalla entera es ese aviso**: es el único
   * momento en que el consultor todavía puede decidir no empezar, y la maqueta lo dibuja así. Con
   * la sesión en marcha, un programa que aparece después va ARRIBA, sin los botones de empezar, y
   * lo demás sigue debajo —la maqueta no dibuja ese caso: queda declarado para el gate del MVP—.
   *
   * «No iniciar» y «Iniciar de todos modos» dan el aviso por visto **para esos programas**: si
   * aparece otro, vuelve.
   */
  const radar = useRadarDeTuMac(radarDeMuestra);
  const [visto, setVisto] = useState("");
  const clave = invasivos(radar)
    .map((p) => p.nombre)
    .join("|");
  const vigilado = clave !== "" && clave !== visto;
  // Un MDM solo es «sábelo»: no toma la pantalla. Sin esta fila, un Mac inscrito y sin ningún
  // invasivo no lo veía en ningún sitio, y el manual decía que Sesión lo lista (auditoría del S2, B15).
  const mdm = radar.programas.find((p) => p.categoria === "mdm");
  /** La fila del MDM: la misma en la reunión y en presencial (auditoría del S5, M4). */
  const filaMdm = mdm ? (
    <Fila icono="i-radar" color="var(--ink-2)" texto={`${t.radarClases.mdm.titulo} ${t.radarClases.mdm.sufijo}`}>
      <span className="estado mute">
        <Ic id="i-ring" s />
        <span>{t.sabelo}</span>
      </span>
    </Fila>
  ) : null;

  const titulo = (
    <>
      <div className="titulo">
        <h1>{t.sesionTitulo}</h1>
        <p className="sub">{t.sesionSub}</p>
      </div>
      {cuadro}
    </>
  );

  // «Revisar» abre la pregunta; si ya había respuesta, se borra primero (ADR 017 §4).
  const revisar = () => {
    setPreguntando(true);
    if (vista.nda !== "sin-revisar") void revisarNda().then((v) => v && setVista(v));
  };
  const responder = (prohibe: boolean) =>
    void responderNda(prohibe).then((v) => {
      if (v) setVista(v);
      setPreguntando(false);
    });

  const antesDeEmpezar = !escucha.escuchando || radarDeMuestra;
  if (vigilado && antesDeEmpezar && !escucha.soloNotas) {
    return (
      <>
        {titulo}
        <LaVigilancia
          radar={radar}
          iniciar={() => {
            setVisto(clave);
            // **Con «presencial» elegido, «Iniciar de todos modos» no empieza una reunión** (auditoría del S5, M4):
            // abriría el audio del sistema y la pantalla que ese modo promete no abrir. Vuelve a la vista de la sala,
            // donde se pulsa «Empezar en presencial».
            if (modo !== "presencial") antes(empezarAEscuchar);
          }}
          noIniciar={() => setVisto(clave)}
        />
      </>
    );
  }

  // **Solo notas, en marcha** (ADR 017 §5): nada escucha ni transcribe; se termina como cualquier otra.
  if (escucha.soloNotas) {
    return (
      <>
        {titulo}
        <EnSoloNotas terminar={dejarDeEscuchar} bytes={bytes} />
      </>
    );
  }

  if (viendoClausula) {
    return (
      <>
        {titulo}
        <LaClausula clausula={vista.clausula} volver={() => setViendoClausula(false)} />
      </>
    );
  }

  // **La NDA lo prohíbe**: el estado aprobado en la Etapa de Diseño. Nunca bloquea: se vuelve a revisar
  // con un clic, y «Iniciar en modo solo notas» es la salida.
  if (!escucha.escuchando && vista.elegido && vista.nda === "lo-prohibe") {
    return (
      <>
        {titulo}
        <LaNdaLoProhibe iniciar={() => antes(empezarSoloNotas)} revisar={revisar} />
      </>
    );
  }

  // **Presencial** (ADR 020): elegido antes de empezar, o en marcha. Una reunión en marcha se ve como reunión.
  if (escucha.presencial || (!escucha.escuchando && modo === "presencial")) {
    return (
      <>
        {titulo}
        <EnPresencial
          escucha={escucha}
          salida={salida}
          bytes={bytes}
          volver={() => setModo("reunion")}
          antes={antes}
          ensayoSinGuardar={ensayoSinGuardar}
          vigilancia={vigilado ? <LaVigilancia radar={radar} /> : null}
          filaMdm={filaMdm}
          accion={preguntando ? <LaPreguntaDeLaNda responder={responder} /> : null}
          cliente={
            <EsteCliente
              vista={vista}
              elegir={(nombre) => void elegirCliente(nombre).then((v) => v && setVista(v))}
              revisar={revisar}
              preguntando={preguntando}
              verLaClausula={() => setViendoClausula(true)}
              transparente={transparente}
            />
          }
        />
      </>
    );
  }

  return (
    <>
      {titulo}

      <div style={PILA}>
        {vigilado && <LaVigilancia radar={radar} />}
        {/* La primera vez, el aviso ocupa el sitio de la tarjeta de la reunión (`sesion.html` · la
            primera vez): las dos no caben en la ventana, y la reunión sigue en el rail («Meet
            detectado»). «Entendido» la devuelve. */}
        {verAviso ? (
          <ElAvisoDeArriba
            entendido={() => {
              setAvisoCerrado(true);
              void entendidoElAvisoDeArriba();
            }}
          />
        ) : (
          <LaReunion reunion={reunion} arriba={franja.borde === "arriba"} />
        )}

        <div className="grid-2">
          <div className="tarjeta">
            <h2 className="seccion">{t.dosPistas}</h2>
            <LaPista
              pista={escucha.microfono}
              caida={caida(escucha.microfono)}
              icono="i-mic"
              texto={t.pistaMic}
            />
            <LaPista
              pista={escucha.sistema}
              caida={caida(escucha.sistema)}
              icono="i-sistema"
              texto={t.pistaSistema}
            />
            <LaPantalla pantalla={pantalla} />
            <LosAuriculares salida={salida} />
            {hayEco && (
              <p
                style={{
                  fontSize: "11.5px",
                  color: "var(--ink-2)",
                  marginTop: "6px",
                }}
              >
                {t.avisoDelEco}
              </p>
            )}
            <LaBanda borde={franja.borde} />
            <LaAlturaDeLaBanda franja={franja} />
            {/* «Escucha las dos pistas» solo aparece cuando NO es verdad (mirada 17): con una pista
                caída, dice cuál queda. Vivía en «Qué funciona hoy», que el sprint 3 retiró al volver
                al diseño aprobado de la Etapa de Diseño. */}
            {(caida(escucha.microfono) || caida(escucha.sistema)) && (
              <LaEscucha mic={!caida(escucha.microfono)} sistema={!caida(escucha.sistema)} />
            )}
          </div>

          <EsteCliente
            vista={vista}
            elegir={(nombre) => void elegirCliente(nombre).then((v) => v && setVista(v))}
            revisar={revisar}
            preguntando={preguntando}
            verLaClausula={() => setViendoClausula(true)}
            transparente={transparente}
          />
        </div>

        {filaMdm}

        {escucha.escuchando ? (
          // El kill-switch va al lado de la promesa que cumple, en la fila de la acción.
          <div className="fila">
            <button className="btn primario" onClick={() => dejarDeEscuchar()}>
              {/* Dice lo que hace: con la sesión en marcha, la termina (auditoría del S2, B16). */}
              <Ic id="i-x-circle" s />
              <span>{t.terminarSesion}</span>
            </button>
            <span className="crece"></span>
            <Promesa bytes={bytes} />
          </div>
        ) : preguntando ? (
          <LaPreguntaDeLaNda responder={responder} />
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
            <div className="fila">
              <button className="btn primario" onClick={() => antes(empezarAEscuchar)}>
                <Ic id="i-voz" s />
                <span>{t.iniciarSesion}</span>
              </button>
              <button className="btn" onClick={() => antes(empezarSoloNotas)}>
                <Ic id="i-nota" s />
                <span>{tc.soloNotas}</span>
              </button>
              {/* El tercer modo (sprint 005, ADR 020): no empieza nada, enseña cómo será la sesión en la sala. */}
              <button className="btn" onClick={() => setModo("presencial")}>
                <Ic id="i-mic" s />
                <span>{t.presencial.boton}</span>
              </button>
            </div>
            {/* Un ensayo terminado sin guardar (auditoría del S4, M10): «Iniciar sesión» lo descarta, así que se
                dice en una línea, en el sitio de la promesa, que vuelve al guardarlo o cerrarlo. En dos líneas,
                encima del botón, la pantalla se salía de su ventana (`maqueta-cabe`, +37 px). */}
            {ensayoSinGuardar ? (
              <div className="franja warn" role="status">
                <Ic id="i-ensayo" s />
                <p>
                  <b>{t.ensayoSinGuardar}.</b> {t.ensayoSinGuardarQue}
                </p>
              </div>
            ) : (
              <div className="fila">
                <Promesa bytes={bytes} />
              </div>
            )}
          </div>
        )}
      </div>
    </>
  );
}

/**
 * **La altura de la banda y su «×»**, en Sesión (sprint 011, ADR 004 enmienda 3) — solo Windows, y sin maqueta: sus textos
 * viven en `windows.ts` (diccionario aparte, ADR 022). Maquetada, no vista: va al gate del hito.
 *
 * - Con la banda arriba, dice a qué altura quedó y «Volver arriba» la devuelve al borde (es el asa soltada en 0).
 * - Con la banda cerrada —por su «×» o por el corte—, lo dice y «Mostrar la banda» la abre en su borde y a su altura.
 */
export function LaAlturaDeLaBanda({ franja }: { franja: LaFranja }) {
  const [plataforma] = usePlataforma();
  const tw = useTextosDeWindows().banda;
  if (!esWindows(plataforma)) return null;
  return (
    <>
      {franja.borde === "arriba" && (
        <Fila icono="i-flecha" color="var(--ink-2)" texto={tw.alturaTitulo}>
          <span className="estado mute">
            {franja.desplazamiento === 0 ? tw.pegadaArriba : tw.alturaValor(franja.desplazamiento)}
          </span>
          {/* Sin arrastrar (WCAG 2.5.7): la altura también se elige por pasos, con un clic o con el teclado. */}
          <button
            type="button"
            className="btn mini"
            aria-label={tw.bajarNombre}
            onClick={() => void asentarDesplazamiento(franja.desplazamiento + PASO_DE_LA_ALTURA)}
          >
            {tw.bajar}
          </button>
          {franja.desplazamiento > 0 && (
            <>
              <button
                type="button"
                className="btn mini"
                aria-label={tw.subirNombre}
                onClick={() => void asentarDesplazamiento(Math.max(0, franja.desplazamiento - PASO_DE_LA_ALTURA))}
              >
                {tw.subir}
              </button>
              <button type="button" className="btn mini" onClick={() => void asentarDesplazamiento(0)}>
                {tw.volverArriba}
              </button>
            </>
          )}
        </Fila>
      )}
      {!franja.bandaAbierta && (
        <Fila icono="i-ring" color="var(--mute)" texto={tw.cerradaTitulo}>
          <button type="button" className="btn mini" onClick={() => void mostrarLaBanda()}>
            {tw.mostrar}
          </button>
        </Fila>
      )}
    </>
  );
}

/**
 * **«La banda: arriba · abajo»** (sprint 004, `sesion.html` · sprint 4). Elegir suelta lo que estuviera
 * acoplado y mueve la banda y su relleno al otro borde; abajo acopla la ventana de delante y arriba la de la
 * reunión, solo con la sesión iniciada (`acoplar_arriba`). `⌃⌥B` hace lo mismo desde cualquier sitio.
 */
export function LaBanda({ borde }: { borde: Borde }) {
  const t = useT().cuaderno;
  const k = useTeclas();
  const opciones: [Borde, string][] = [
    ["arriba", t.bandaArriba],
    ["abajo", t.bandaAbajo],
  ];
  return (
    <Fila icono="i-flecha" color="var(--ink-2)" texto={t.laBanda}>
      <div className="ventanas" role="radiogroup" aria-label={t.laBanda}>
        {opciones.map(([b, nombre]) => {
          const on = b === borde;
          return (
            <button
              key={b}
              type="button"
              role="radio"
              aria-checked={on}
              className={on ? "op on" : "op"}
              // El borde que ya está elegido no se vuelve a pedir: soltaría y volvería a acoplar la reunión
              // para nada (auditoría del S4, B25).
              onClick={() => {
                if (!on) void fijarPosicionDeLaBanda(b);
              }}
            >
              {on && <Ic id="i-check-circle" s relleno />}
              {nombre}
            </button>
          );
        })}
      </div>
      <span className="tecla chica">
        <kbd>{k("⌃⌥B")}</kbd>
      </span>
    </Fila>
  );
}

/**
 * **El aviso de la primera vez con la banda arriba** (sprint 004, `sesion.html` · la primera vez): dice
 * el porqué —la cámara— y cómo volver abajo. «Entendido» no vuelve a enseñarlo (`avisoDeArribaVisto`).
 */
export function ElAvisoDeArriba({ entendido }: { entendido: () => void }) {
  const t = useT().cuaderno;
  return (
    <div className="franja mute" role="status">
      <Ic id="i-flecha" s />
      <div className="fila" style={{ flexWrap: "nowrap", gap: "10px" }}>
        <div className="crece">
          <strong>{t.avisoArribaTitulo}</strong>
          <p>{t.avisoArribaTexto}</p>
        </div>
        <button className="btn mini" onClick={entendido}>
          {t.entendido}
        </button>
      </div>
    </div>
  );
}

/**
 * **PRESENCIAL** (C19, sprint 005, ADR 020; `sesion.html` «sprint 5 · presencial» y «… · en marcha», mirada de
 * DECISIÓN aprobada el 2026-10-05).
 *
 * Una pista, la sala, por el micrófono del Mac: el audio del sistema y la pantalla **no se abren**, y se dice.
 * El idioma es uno para toda la sala; la ficha llega **al oído**, salvo con altavoces que la app reconoce, y la
 * banda queda a una tecla. «Lo tuyo queda» se apaga porque la app no sabe qué turnos son tuyos (§5).
 */
function EnPresencial({
  escucha,
  salida,
  bytes,
  volver,
  cliente,
  accion,
  antes,
  ensayoSinGuardar,
  vigilancia,
  filaMdm,
}: {
  escucha: EstadoDeEscucha;
  salida: Salida;
  bytes: string;
  /** El aviso coral del radar, arriba, como en la reunión (auditoría del S5, M4). */
  vigilancia: React.ReactNode;
  /** La fila del MDM: «sábelo». */
  filaMdm: React.ReactNode;
  /** «reunión en este Mac»: vuelve a la sesión de siempre, sin empezar nada. */
  volver: () => void;
  /** La tarjeta «Este cliente», la misma de la reunión. */
  cliente: React.ReactNode;
  /** Lo que ocupa la fila de la acción mientras se contesta la NDA; `null`, los botones. */
  accion: React.ReactNode;
  /** Empieza lo que pidas, o en Windows pregunta primero por la reunión anterior (sprint 013, decisión 2a). */
  antes: (accion: () => unknown) => void;
  ensayoSinGuardar: boolean;
}) {
  const t = useT().cuaderno;
  const k = useTeclas();
  const tc = useT().cliente;
  const tp = t.presencial;
  const idiomas = useIdiomasDePista();
  const voz = useVoz();
  const enMarcha = escucha.escuchando;
  const sala = idiomas.sala ?? idiomas.cliente;
  // **El selector ofrece lo que este equipo sabe transcribir** (auditoría del S5, M5): los dos de la casa, los del motor y
  // el del cliente. Un idioma elegido no desaparece de la lista al elegir otro, y el estado de su modelo se dice al lado.
  const transcribe = useQueSabeTranscribir();
  const codigos = [...new Set(["es-ES", "en-US", ...transcribe.idiomas.map((i) => i.codigo), idiomas.cliente, sala])];
  const modeloDeLaSala = transcribe.idiomas.find((i) => i.codigo === sala)?.disponibilidad;
  // **El micrófono de la sala, nombrado antes de empezar** (auditoría del S5, M7).
  const entrada = useEntradaDeAudio();
  const te = useTextosDePresencial();
  const [plataforma] = usePlataforma();
  // Con altavoces que la app reconoce, la banda de partida (§7): la voz no hablaría con el cliente oyéndola.
  const altavoz = salida.salida === "altavoces" || salida.salida === "altavoz-externo";
  const [elegida, setElegida] = useState<boolean | null>(null);
  /** Pediste la ficha al oído al empezar (y no la cambiaste después): para decir si no hay voz (auditoría del S5, B13). */
  const [pedidaAlOido, setPedidaAlOido] = useState(false);
  // **M5 (auditoría del S6):** «Al oído» parte elegido solo si la voz PUEDE hablar —hay voz para el idioma y la salida no
  // son altavoces que la app reconoce—, que es lo que Rust dice en `voz.puede`. En Windows, sin voz del sistema para el
  // idioma, partía elegido con la voz muda. Fuera de Tauri (la maqueta) no hay a quién preguntar y manda la salida.
  const alOido = enMarcha && hayTauri() ? voz.encendida : (elegida ?? (hayTauri() ? voz.puede : !altavoz));
  const elegir = (si: boolean) => {
    setPedidaAlOido(si);
    if (enMarcha && hayTauri()) void fijarLaVoz(si);
    else setElegida(si);
  };
  const nota = { fontSize: "11.5px", marginTop: "-2px", color: "var(--ink-2)" } as const;
  // De qué aparato se habla: el que la app nombra, o lo que sabe de él.
  const aparato =
    salida.salida === "otra" || salida.salida === "altavoz-externo"
      ? `${salida.nombre}. `
      : salida.salida === "no-se-sabe"
        ? salida.nombre
          ? `${salida.nombre}. `
          : ""
        : salida.salida === "altavoces"
          ? `${t.altavocesInternos}. `
          : `${t.pistaAuriculares}. `;
  // Con USB o Bluetooth la app no distingue un auricular de un altavoz, y lo dice (§7).
  const duda = salida.salida === "otra" || salida.salida === "no-se-sabe";
  const opciones: [boolean, string][] = [
    [true, tp.alOido],
    [false, tp.laBanda],
  ];

  return (
    <div style={PILA}>
      {vigilancia}
      <div className="tarjeta">
        <div className="fila">
          <Ic id="i-mic" color="var(--halo)" />
          <h3 className="crece">{tp.titulo}</h3>
          {enMarcha ? (
            <span className="estado halo">
              <Ic id="i-dot" s relleno />
              <span>{tp.enMarcha}</span>
            </span>
          ) : (
            <div className="ventanas" role="radiogroup" aria-label={tp.modo}>
              <button type="button" role="radio" aria-checked={false} className="op" onClick={volver}>
                {tp.reunionEnEsteMac}
              </button>
              <button type="button" role="radio" aria-checked className="op on">
                <Ic id="i-check-circle" s relleno />
                {tp.presencial}
              </button>
            </div>
          )}
        </div>
      </div>

      <div className="grid-2">
        <div className="tarjeta">
          <h2 className="seccion">{tp.unaPista}</h2>
          {enMarcha ? (
            <LaPista pista={escucha.microfono} caida={!escucha.microfono.abierta} icono="i-mic" texto={tp.microfono} />
          ) : (
            <>
              <Fila icono="i-mic" color="var(--ok)" texto={tp.microfono}>
                <span className="estado ok">
                  <Ic id="i-check-circle" s relleno />
                  <span>{tp.listo}</span>
                </span>
              </Fila>
              <p style={nota}>
                <b>{tp.pcPregunta}</b> {tp.pcQueHacer}
              </p>
            </>
          )}
          {entrada && !enMarcha && (
            <p style={nota} role="status">
              {entrada.externa ? (
                <>
                  <Ic id="i-alert" s color="var(--warn)" />{" "}
                  {te.externa(entrada.nombre, esWindows(plataforma) ? te.enWindows : te.enElMac)}
                </>
              ) : (
                te.microfono(entrada.nombre)
              )}
            </p>
          )}
          <div className="fila">
            <Ic id="i-globo" s color="var(--ink-2)" />
            <span className="crece">{tp.idiomaDeLaSala}</span>
            {enMarcha ? (
              // Fijo mientras dura: la sala se transcribe en el que tenía al empezar.
              <span className="mono" style={{ color: "var(--ink-2)" }}>
                {sala}
              </span>
            ) : (
              <select
                className="selector"
                aria-label={tp.idiomaDeLaSala}
                value={sala}
                onChange={(e) => fijarIdiomaDePista("sala", e.target.value)}
              >
                {codigos.map((c) => (
                  <option key={c} value={c}>
                    {c}
                  </option>
                ))}
              </select>
            )}
          </div>
          {/* Pediste la voz y no hay voz para tu idioma: la sesión sigue con la banda y lo dice (B13 del S5). */}
          {enMarcha && hayTauri() && pedidaAlOido && !voz.encendida && !voz.puede && (
            <p style={nota} role="status">
              <Ic id="i-alert" s color="var(--warn)" /> {te.sinVoz(idiomas.consultor)}
            </p>
          )}
          {modeloDeLaSala && (
            <p style={nota} role="status">
              {estadoDelModelo(modeloDeLaSala, t.modeloInstalado) ?? motivo(t, modeloDeLaSala)}
            </p>
          )}
          <p style={nota}>{tp.todaLaSala}</p>
          <div className="fila">
            <Ic id="i-sistema-off" s color="var(--mute)" />
            <span className="crece" style={{ color: "var(--mute)" }}>
              {tp.audioDelSistema}
            </span>
            <span className="estado mute">
              <Ic id="i-ring" s />
              <span>{tp.noSeAbre}</span>
            </span>
          </div>
          <div className="fila">
            <Ic id="i-pantalla-off" s color="var(--mute)" />
            <span className="crece" style={{ color: "var(--mute)" }}>
              {tp.pantalla}
            </span>
            <span className="estado mute">
              <Ic id="i-ring" s />
              <span>{tp.noSeAbre}</span>
            </span>
          </div>
          <div className="fila" style={{ flexWrap: "nowrap" }}>
            <Ic id="i-auriculares" s color="var(--ink-2)" />
            <span className="crece">{tp.laFicha}</span>
            <div className="ventanas" role="radiogroup" aria-label={tp.laFicha} style={{ flexWrap: "nowrap" }}>
              {opciones.map(([si, nombre]) => {
                const on = si === alOido;
                return (
                  <button
                    key={nombre}
                    type="button"
                    role="radio"
                    aria-checked={on}
                    className={on ? "op on" : "op"}
                    onClick={() => {
                      if (!on) elegir(si);
                    }}
                  >
                    {on && <Ic id="i-check-circle" s relleno />}
                    {nombre}
                  </button>
                );
              })}
            </div>
          </div>
          <p style={nota}>
            {`${aparato}${duda ? `${tp.siEsUnAltavoz} ` : ""}`}
            <kbd>{k("⌃⌥V")}</kbd>
            {` ${tp.cambia}`}
          </p>
          <div className="fila">
            <Ic id="i-nota" s color="var(--mute)" />
            <span className="crece" style={{ fontSize: "var(--t-meta)" }}>
              {tp.loTuyoQueda}
            </span>
            <span className="estado mute">
              <Ic id="i-ring" s />
              <span>{tp.apagado}</span>
            </span>
          </div>
          <p style={nota}>{tp.noSabeCuales}</p>
        </div>

        {cliente}
      </div>

      {filaMdm}

      {enMarcha ? (
        <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
          <div className="fila">
            <button className="btn primario" onClick={() => dejarDeEscuchar()}>
              <Ic id="i-x-circle" s />
              <span>{t.terminarSesion}</span>
            </button>
          </div>
          <div className="fila">
            <Promesa bytes={bytes} />
          </div>
        </div>
      ) : accion ? (
        accion
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
          <div className="fila">
            <button
              className="btn primario"
              onClick={() => {
                setPedidaAlOido(alOido);
                antes(() => empezarPresencial(alOido));
              }}
            >
              <Ic id="i-mic" s />
              <span>{tp.empezar}</span>
            </button>
            <button className="btn" onClick={() => antes(empezarSoloNotas)}>
              <Ic id="i-nota" s />
              <span>{tc.soloNotas}</span>
            </button>
          </div>
          {ensayoSinGuardar ? (
            <div className="franja warn" role="status">
              <Ic id="i-ensayo" s />
              <p>
                <b>{t.ensayoSinGuardar}.</b> {t.ensayoSinGuardarQue}
              </p>
            </div>
          ) : (
            <div className="fila">
              <Promesa bytes={bytes} />
            </div>
          )}
        </div>
      )}
    </div>
  );
}

/** `⌥⎋` y la promesa que cumple: «corta todo · el sonido nunca se guarda · …». */
function Promesa({ bytes }: { bytes: string }) {
  const t = useT().cuaderno;
  const k = useTeclas();
  return (
    <>
      <span className="tecla">
        <kbd>{k("⌥⎋")}</kbd>
      </span>
      <span className="mono" style={{ color: "var(--ink-2)" }}>
        {bytes === "0 B" ? t.nadaSale : t.nadaSaleConApi}
      </span>
    </>
  );
}

/**
 * **«ESTE CLIENTE»** (C11, ADR 017; `sesion.html` «sprint 3 · este cliente», maquetado, no visto): el
 * selector con los clientes de tu corpus, la bandera de su jurisdicción, su NDA y la cláusula. «No es
 * asesoría legal» va siempre.
 */
function EsteCliente({
  vista,
  elegir,
  revisar,
  preguntando,
  verLaClausula,
  transparente = null,
}: {
  vista: VistaDelCliente;
  elegir: (nombre: string | null) => void;
  revisar: () => void;
  preguntando: boolean;
  verLaClausula: () => void;
  /** El modo transparente de este cliente (sprint 012, ADR 024): solo en Windows; en el Mac llega `null` y no pinta nada. */
  transparente?: React.ReactNode;
}) {
  const t = useT().cuaderno;
  const tc = useT().cliente;
  return (
    <div className="tarjeta">
      <div className="fila">
        <h2 className="seccion crece" style={{ margin: 0 }}>
          {t.esteCliente}
        </h2>
        <select
          className="selector"
          aria-label={t.esteCliente}
          value={vista.elegido ?? ""}
          onChange={(e) => elegir(e.target.value || null)}
        >
          <option value="">{tc.sinElegir}</option>
          {vista.clientes.map((c) => (
            <option key={c} value={c}>
              {c}
            </option>
          ))}
        </select>
      </div>
      {vista.bandera && <BanderaDelCliente bandera={vista.bandera} />}
      {vista.elegido && vista.nda !== "lo-prohibe" && (
        <div className="fila">
          {vista.nda === "no-lo-prohibe" ? (
            <Ic id="i-check-circle" s color="var(--ok)" />
          ) : (
            <Ic id="i-pendiente" s color="var(--mute)" />
          )}
          <span className="crece" style={{ fontSize: "var(--t-meta)" }}>
            {vista.nda === "no-lo-prohibe" ? tc.ndaNoLoProhibe : tc.ndaSinRevisar}
          </span>
          <button className="btn mini" type="button" aria-expanded={preguntando} onClick={revisar}>
            {tc.revisar}
          </button>
        </div>
      )}
      {transparente}
      <div className="fila">
        <span className="crece aviso-legal">{tc.noEsAsesoria}</span>
        <button className="btn mini" type="button" onClick={verLaClausula}>
          <Ic id="i-doc" s />
          {tc.clausulaDeEncargo}
        </button>
      </div>
    </div>
  );
}

/**
 * La bandera de `kit.html` §5, con lo que Rust decidió: su riesgo con símbolo, texto y color (regla
 * 8), la regla, lo que implica, las normas con la fecha del informe y, si algo no se verificó, dicho.
 */
export function BanderaDelCliente({ bandera }: { bandera: LaBandera }) {
  const tc = useT().cliente;
  const idioma = useIdioma();
  if (bandera.que !== "conocida") {
    const fuera = bandera.que === "fuera-del-catalogo";
    const [abre, cierra] = idioma === "es" ? ["«", "»"] : ["\u201c", "\u201d"];
    return (
      <div className="bandera desconocida">
        <Ic id="i-globo" />
        <div className="donde">
          {fuera ? `${abre}${bandera.escrita}${cierra} ${tc.noEstaEnElCatalogo} v${bandera.version}` : tc.noIndicada}{" "}
          <span className="estado mute">
            <Ic id="i-ring" s />
            <span>{tc.sinBandera}</span>
          </span>
        </div>
        <div className="regla">{fuera ? tc.escribela : tc.indicaDonde}</div>
        <div className="implica">{tc.mientrasTanto}</div>
        <div className="fuente">—</div>
      </div>
    );
  }
  const b = bandera.bandera;
  const bajo = b.riesgo === "bajo" || b.riesgo === "bajo-medio";
  const medio = b.riesgo === "medio" || b.riesgo === "medio-alto";
  const clase = bajo ? "riesgo-bajo" : b.riesgo === "medio" ? "riesgo-medio" : b.riesgo === "medio-alto" ? "riesgo-alto" : "desconocida";
  return (
    <div className={`bandera ${clase}`}>
      <Ic id="i-globo" />
      <div className="donde">
        {b.nombre[idioma]}{" "}
        <span className={bajo ? "estado ok" : medio ? "estado warn" : "estado mute"}>
          {bajo ? <Ic id="i-check-circle" s relleno /> : medio ? <Ic id="i-alert" s relleno /> : <Ic id="i-ring" s />}
          <span>{tc.riesgo[b.riesgo]}</span>
        </span>
      </div>
      <div className="regla">{b.regla[idioma]}</div>
      <div className="implica">{b.implica[idioma]}</div>
      <div className="fuente">{[...b.normas, b.consultado].join(" · ")}</div>
      {b.pendiente && (
        <div className="pendiente">
          {tc.sinVerificar} {b.pendiente[idioma]}
        </div>
      )}
    </div>
  );
}

/** La pregunta del chequeo de NDA, en la fila de la acción: se contesta antes de empezar. */
function LaPreguntaDeLaNda({ responder }: { responder: (prohibe: boolean) => void }) {
  const tc = useT().cliente;
  return (
    <div className="pregunta-nda" role="group" aria-label="NDA">
      <Ic id="i-pendiente" s />
      <span className="crece">{tc.pregunta}</span>
      <button className="btn mini" type="button" onClick={() => responder(true)}>
        {tc.siLoProhibe}
      </button>
      <button className="btn mini" type="button" onClick={() => responder(false)}>
        {tc.noLoProhibe}
      </button>
    </div>
  );
}

/**
 * La cláusula modelo, **las dos versiones lado a lado** (ADR 017 §6): se copia la del idioma de la
 * carta de encargo, no la de la interfaz. Cada una lleva su `lang` para que un lector de pantalla la
 * pronuncie bien.
 */
function LaClausula({ clausula, volver }: { clausula: Bilingue; volver: () => void }) {
  const tc = useT().cliente;
  const [copiada, setCopiada] = useState<"es" | "en" | null>(null);
  const copiar = (idioma: "es" | "en") =>
    void navigator.clipboard?.writeText(clausula[idioma]).then(
      () => setCopiada(idioma),
      () => setCopiada(null),
    );
  return (
    <div className="tarjeta">
      <div className="fila">
        <h2 className="seccion crece" style={{ margin: 0 }}>
          {tc.clausulaTitulo}
        </h2>
        <button className="btn mini" type="button" onClick={volver}>
          {tc.volver}
        </button>
      </div>
      <p className="aviso-legal">{tc.plantilla}</p>
      <div className="grid-2" style={{ gap: "14px", alignItems: "start" }}>
        {(["es", "en"] as const).map((l) => (
          <div key={l} style={{ display: "flex", flexDirection: "column", gap: "6px" }}>
            <div className="fila">
              <strong className="crece">{l === "es" ? "Español" : "English"}</strong>
              <button className="btn mini" type="button" onClick={() => copiar(l)}>
                {copiada === l && <Ic id="i-check-circle" s relleno />}
                {copiada === l ? tc.copiada : tc.copiar}
              </button>
            </div>
            <p className="clausula" lang={l}>
              {clausula[l]}
            </p>
          </div>
        ))}
      </div>
    </div>
  );
}

/** «Qué queda apagado» y «Qué sigue funcionando»: lo mismo antes de empezar en solo notas y durante. */
function LoQueQuedaYLoQueSigue() {
  const tc = useT().cliente;
  const k = useTeclas();
  const apagada = (icono: string, texto: string) => (
    <div className="fila" key={icono}>
      <Ic id={icono} s color="var(--mute)" />
      <span className="crece" style={{ color: "var(--mute)" }}>
        {texto}
      </span>
      <span className="estado mute">
        <Ic id="i-ring" s />
        <span>{tc.apagado}</span>
      </span>
    </div>
  );
  return (
    <div className="grid-2">
      <div className="tarjeta">
        <h2 className="seccion">{tc.queQuedaApagado}</h2>
        {apagada("i-mic-off", tc.microfono)}
        {apagada("i-sistema-off", tc.audioDelSistema)}
        {apagada("i-pantalla-off", tc.lecturaDePantalla)}
      </div>
      <div className="tarjeta">
        <h2 className="seccion">{tc.queSigue}</h2>
        <Fila icono="i-nota" color="var(--ok)" texto={tc.notasYAcuerdos}>
          <span className="tecla">
            <kbd>{k("⌃⌥N")}</kbd>
          </span>
        </Fila>
        <Fila icono="i-buscar" color="var(--ok)" texto={tc.buscarAMano}>
          <span className="tecla">
            <kbd>{k("⌃⌥A")}</kbd>
          </span>
        </Fila>
        <Fila icono="i-candado" color="var(--ok)" texto={tc.panelProtegido} />
      </div>
    </div>
  );
}

/** `sesion.html` «NDA prohíbe transcribir» (Etapa de Diseño): la app no bloquea, propone solo notas. */
function LaNdaLoProhibe({ iniciar, revisar }: { iniciar: () => void; revisar: () => void }) {
  const tc = useT().cliente;
  return (
    <div style={PILA}>
      <div className="franja warn" role="status" style={{ padding: "14px 16px" }}>
        <Ic id="i-alert" relleno />
        <div>
          <strong>{tc.ndaProhibe}</strong>
          <p style={{ marginTop: "4px" }}>
            {tc.ndaProhibeAntes} <b>{tc.ndaProhibeModo}</b>
            {tc.ndaProhibeDespues}
          </p>
        </div>
      </div>
      <LoQueQuedaYLoQueSigue />
      <div className="fila">
        <button className="btn primario" style={{ padding: "9px 16px", fontSize: "var(--t-ui)" }} onClick={iniciar}>
          <Ic id="i-nota" s />
          <span>{tc.iniciarSoloNotas}</span>
        </button>
        <button className="btn" onClick={revisar}>
          <span>{tc.volverARevisar}</span>
        </button>
      </div>
    </div>
  );
}

/** La reunión en solo notas, en marcha (`sesion.html` «sprint 3 · en solo notas»). */
function EnSoloNotas({ terminar, bytes }: { terminar: () => void; bytes: string }) {
  const t = useT().cuaderno;
  const k = useTeclas();
  const tc = useT().cliente;
  return (
    <div style={PILA}>
      <div className="franja warn" role="status" style={{ padding: "14px 16px" }}>
        <Ic id="i-nota" />
        <div>
          <strong>{tc.modoSoloNotas}</strong>
          <p style={{ marginTop: "4px" }}>
            {tc.enMarchaAntes} <b>{k("⌃⌥A")}</b> {tc.enMarchaDespues}
          </p>
        </div>
      </div>
      <LoQueQuedaYLoQueSigue />
      <div className="fila">
        <button className="btn primario" onClick={terminar}>
          <Ic id="i-x-circle" s />
          <span>{t.terminarSesion}</span>
        </button>
        <span className="crece"></span>
        <Promesa bytes={bytes} />
      </div>
    </div>
  );
}

/** Una línea de «por qué», debajo de su fila. Cabe lo que no cabe en el chip. */
function Porque({
  texto,
  color = "var(--ink-2)",
}: {
  texto: string;
  color?: string;
}) {
  return <p style={{ fontSize: "11.5px", marginTop: "2px", color }}>{texto}</p>;
}

/**
 * La tarjeta de la reunión. **«Protección verificada» es un hecho de la banda ABAJO** (decisión M20 del
 * usuario): con la banda arriba nadie ha mirado la pantalla compartida, así que arriba dice «sin
 * verificar» también en Meet, igual que el chip de la banda (auditoría del S4, A5).
 */
export function LaReunion({ reunion, arriba = false }: { reunion: Reunion; arriba?: boolean }) {
  const t = useT().cuaderno;
  const tb = useT().banda;
  if (reunion.que === "detectada") {
    return (
      <div className="tarjeta">
        <div className="fila">
          <Ic id="i-video" color="var(--halo)" />
          <h3 className="crece">
            {reunion.titulo
              ? `${reunion.cliente} · ${reunion.titulo}`
              : reunion.cliente}
          </h3>
          {reunion.proteccion === "Verificada" && !arriba ? (
            <span className="estado ok">
              <Ic id="i-check-circle" s relleno />
              <span>{t.proteccionVerificada}</span>
            </span>
          ) : (
            <span className="estado warn">
              <Ic id="i-alert" s relleno />
              {/* El cliente que HAY, no «Zoom» para todos: Teams, Safari o Edge también salían como
                  «Zoom · sin verificar» (casilla 6 del S3). Mismo nombre corto que el rail. */}
              <span>
                {reunion.cliente.replace(/^(Google|Microsoft)\s+/, "")} · {tb.sinVerificarSufijo}
              </span>
            </span>
          )}
        </div>
        <p>{t.proteccionDetalle}</p>
      </div>
    );
  }
  // **«No se puede saber» no es «no hay reunión».** Sin Accesibilidad la app no lee los títulos
  // de las ventanas del navegador, y decir «sin reunión» con Meet abierto sería afirmar lo que no
  // sabe. Se dice lo que pasa y cómo se arregla (kit §8-ter).
  const sinSaber = reunion.que === "no-se-puede-saber";
  return (
    <div className="tarjeta" style={{ alignItems: "flex-start", gap: "14px" }}>
      <span className="estado mute">
        <Ic id="i-ring" s />
        <span>{sinSaber ? t.noSePuedeSaber : t.sinReunion}</span>
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
        {sinSaber ? t.porQueNoSeVe[reunion.motivo] : t.sinReunionVoz}
      </p>
    </div>
  );
}

/**
 * Una pista. **«Funciona» habla del grifo, no de las muestras**: sin nadie hablando no llega ni
 * una, y eso no es una avería. Lo que sí lo es —el grifo que no abrió— cambia tres cosas a la vez
 * (regla 8, daltonismo): el símbolo (tachado), la palabra y el color. Y trae su porqué en su línea,
 * terminado en una salida.
 */
export function LaPista({
  pista,
  caida,
  icono,
  texto,
}: {
  pista: EstadoDePista;
  caida: boolean;
  icono: "i-mic" | "i-sistema";
  texto: string;
}) {
  const t = useT().cuaderno;
  if (!caida) {
    return (
      <Fila icono={icono} color="var(--ok)" texto={texto}>
        <Funciona />
      </Fila>
    );
  }
  return (
    <>
      <Fila icono={`${icono}-off`} color="var(--err)" texto={texto}>
        <span className="estado err">
          <Ic id="i-x-circle" s relleno />
          <span>{t.noAbrio}</span>
        </span>
      </Fila>
      <Porque
        texto={t.porQueNoAbrio[pista.motivo ?? "no-dejo"]}
        color="var(--err)"
      />
    </>
  );
}

/**
 * La pantalla (C8): cinco vistas, cada una con su chip, y las que no leen dicen por qué. Debajo,
 * siempre, el interruptor y la tecla: **apagada, la tecla sigue leyendo** cuando la pides — es la
 * salida para una NDA estricta que el usuario eligió en vez de la selección de región.
 */
export function LaPantalla({ pantalla }: { pantalla: EstadoDeLaPantalla }) {
  const t = useT().cuaderno;
  const k = useTeclas();
  const encendida = pantalla.vista !== "apagada";
  let chip;
  let porque: { texto: string; color?: string } | null = null;
  switch (pantalla.vista) {
    case "leyendo":
      chip = <Funciona />;
      break;
    case "esperando-la-reunion":
      chip = (
        <span className="estado mute">
          <Ic id="i-ring" s />
          <span>{t.pantallaEspera}</span>
        </span>
      );
      break;
    case "apagada":
      chip = (
        <span className="estado mute">
          <Ic id="i-ring" s />
          <span>{t.pantallaApagada}</span>
        </span>
      );
      porque = { texto: t.pantallaApagadaPor };
      break;
    case "sin-permiso":
      chip = (
        <span className="estado err">
          <Ic id="i-x-circle" s relleno />
          <span>{t.pantallaSinPermiso}</span>
        </span>
      );
      porque = { texto: t.pantallaSinPermisoPor, color: "var(--err)" };
      break;
    case "no-pudo":
      chip = (
        <span className="estado warn">
          <Ic id="i-alert" s />
          <span>{t.pantallaNoPudo}</span>
        </span>
      );
      porque = { texto: t.pantallaNoPudoPor };
      break;
  }
  return (
    <>
      <Fila
        icono={
          pantalla.vista === "sin-permiso" ? "i-pantalla-off" : "i-pantalla"
        }
        color={
          pantalla.vista === "leyendo" ? "var(--ok)" : pantalla.vista === "sin-permiso" ? "var(--err)" : undefined
        }
        texto={t.pistaPantalla}
      >
        {chip}
      </Fila>
      {porque && <Porque {...porque} />}
      <div className="fila" style={{ gap: "10px", margin: "-2px 0 4px 26px" }}>
        {/* Un interruptor de verdad: `role="switch"` y su estado para quien use un lector de
            pantalla. La maqueta lo dibuja como `label`; aquí un `label` sin control al que
            asociarse no conmutaría nada. El botón se viste con la clase canon y se le quita lo que
            el navegador pone por defecto. */}
        <button
          type="button"
          role="switch"
          aria-checked={encendida}
          className={encendida ? "conm on" : "conm off"}
          style={{
            background: "none",
            border: 0,
            padding: 0,
            fontFamily: "inherit",
          }}
          onClick={() => void lecturaAutomatica(!encendida)}
        >
          <span className="track"></span>
          <span className="etq">{t.leerlaSola}</span>
        </button>
        <span className="tecla chica">
          <kbd>{k("⌃⌥L")}</kbd> {t.leelaAhora}
        </span>
      </div>
    </>
  );
}

/**
 * Por dónde sale el sonido. Con los altavoces internos —o por HDMI, DisplayPort o AirPlay, casi
 * siempre un altavoz (M4)— el micrófono oye al cliente (eco); con un dispositivo por USB o
 * Bluetooth **la app no puede saber** si es un casco o un altavoz, así que dice su nombre y lo
 * único que importa: si es un altavoz, el modo solo audio no se usa.
 */
export function LosAuriculares({ salida }: { salida: Salida }) {
  const t = useT().cuaderno;
  const comillas = useT().banda;
  switch (salida.salida) {
    case "altavoces":
      return (
        <Fila
          icono="i-auriculares"
          color="var(--warn)"
          texto={t.pistaAuriculares}
        >
          <span className="estado warn">
            <Ic id="i-alert" s />
            <span>{t.altavocesInternos}</span>
          </span>
        </Fila>
      );
    case "auriculares":
      return (
        <Fila
          icono="i-auriculares"
          color="var(--ok)"
          texto={t.pistaAuriculares}
        >
          <Funciona />
        </Fila>
      );
    case "otra":
      return (
        <>
          <Fila icono="i-auriculares" color="var(--ink-2)" texto={t.pistaAuriculares}>
            <span className="estado mute">
              <Ic id="i-auriculares" s />
              {salida.nombre}
            </span>
          </Fila>
          <Porque texto={t.siEsUnAltavoz} />
        </>
      );
    case "altavoz-externo":
      return (
        <>
          <Fila icono="i-auriculares" color="var(--warn)" texto={t.pistaAuriculares}>
            <span className="estado warn">
              <Ic id="i-alert" s />
              <span>{salida.nombre}</span>
            </span>
          </Fila>
          <Porque texto={t.altavozExterno} />
        </>
      );
    case "no-se-sabe":
      return (
        <>
          <Fila icono="i-auriculares" color="var(--ink-2)" texto={t.pistaAuriculares}>
            <span className="estado mute">
              <Ic id="i-ring" s />
              <span>{t.noSeSabe}</span>
            </span>
          </Fila>
          <Porque
            texto={
              salida.nombre === null
                ? t.porQueNoSeSabe[salida.motivo]
                : `${comillas.comillaAbre}${salida.nombre}${comillas.comillaCierra} ${t.porQueNoSeSabe[salida.motivo]}`
            }
          />
        </>
      );
  }
}

/**
 * «Escucha las dos pistas» **solo cuando es verdad.** Con una caída, la fila dice cuál queda —en
 * negrita, como la maqueta— y pasa a «A medias»; con las dos caídas, lo dice sin rodeos.
 */
export function LaEscucha({ mic, sistema }: { mic: boolean; sistema: boolean }) {
  const t = useT().cuaderno;
  if (mic && sistema) {
    return (
      <Fila icono="i-voz" color="var(--ok)" texto={t.funcionaEscucha}>
        <Funciona />
      </Fila>
    );
  }
  if (!mic && !sistema) {
    return (
      <Fila icono="i-voz" texto={t.funcionaEscucha}>
        <span className="estado err">
          <Ic id="i-x-circle" s relleno />
          <span>{t.noAbrio}</span>
        </span>
      </Fila>
    );
  }
  return (
    <div className="fila">
      <Ic id="i-voz" s color="var(--warn)" />
      <span className="crece">
        {t.escuchaAMedias} <b>{mic ? t.soloTuPista : t.soloLaDelCliente}</b>
      </span>
      <span className="estado warn">
        <Ic id="i-alert" s />
        <span>{t.aMedias}</span>
      </span>
    </div>
  );
}

/**
 * «Software invasivo corriendo en tu Mac» — `sesion.html`, estado «software invasivo en tu Mac».
 * Los botones solo existen antes de empezar: con la sesión en marcha ya no hay nada que decidir.
 */
export function LaVigilancia({
  radar,
  iniciar,
  noIniciar,
}: {
  radar: EnTuMac;
  iniciar?: () => void;
  noIniciar?: () => void;
}) {
  const t = useT().cuaderno;
  const idioma = useIdioma();
  const bytes = useBytesALaRed();
  return (
    <div style={PILA}>
      <div className="franja err" role="alert" style={{ padding: "14px 16px" }}>
        <Ic id="i-x-circle" relleno />
        <div>
          <strong>{t.vigilanciaTitulo}</strong>
          <p style={{ marginTop: "4px" }}>
            {t.vigilanciaNoEs} <b>{t.vigilanciaTuEquipo}</b> {t.vigilanciaQueMiran}{" "}
            <b>{t.vigilanciaSoloTuMac}</b>
            {t.vigilanciaJamas}
          </p>
        </div>
      </div>
      <div className="tarjeta" style={{ padding: 0 }}>
        <table className="tabla">
          <thead>
            <tr>
              <th>{t.queEncontro}</th>
              <th>{t.queAlcanzaAVer}</th>
              <th>{t.nivel}</th>
              <th>{t.catalogo}</th>
            </tr>
          </thead>
          <tbody>
            {radar.programas.map((p) => {
              const clase = t.radarClases[p.categoria];
              const sabelo = p.nivel === "sabelo";
              return (
                <tr key={p.nombre} className={sabelo ? "apagada" : undefined}>
                  <td>
                    <b>{clase.titulo}</b>
                    {clase.sufijo && ` ${clase.sufijo}`}
                    <br />
                    <span className="mono" style={sabelo ? undefined : { color: "var(--ink-2)" }}>
                      {p.nombre}
                    </span>
                  </td>
                  <td>{p.alcance[idioma]}</td>
                  <td>
                    {sabelo ? (
                      <span className="estado warn">
                        <Ic id="i-alert" s relleno />
                        {t.sabelo}
                      </span>
                    ) : (
                      <span className="estado err">
                        <Ic id="i-x-circle" s relleno />
                        {t.invasivo}
                      </span>
                    )}
                  </td>
                  <td className="mono">
                    v{radar.catalogo.version} · {radar.catalogo.fecha}
                  </td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
      <div className="grid-2">
        <div className="franja ok" role="status">
          <Ic id="i-check-circle" relleno />
          <div>
            <strong>{t.tuProteccionSigue}</strong>
            <p>
              {t.panelProtegido} · {bytes} {t.aLaRed} · {t.nadaDeLaReunionEnDisco}
            </p>
          </div>
        </div>
        {iniciar && noIniciar && (
          <div className="fila" style={{ alignItems: "flex-start" }}>
            <button className="btn primario" onClick={iniciar}>
              <Ic id="i-video" s />
              <span>{t.iniciarDeTodosModos}</span>
            </button>
            <button className="btn" onClick={noIniciar}>
              {t.noIniciar}
            </button>
          </div>
        )}
      </div>
    </div>
  );
}
