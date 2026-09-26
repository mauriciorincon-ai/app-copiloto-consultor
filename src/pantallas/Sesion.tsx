import { useState } from "react";
import { useIdioma, useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { Fila, Funciona, TodaviaNo, PILA } from "../componentes/Ventana";
import {
  empezarAEscuchar,
  dejarDeEscuchar,
  lecturaAutomatica,
  usePantalla,
  DEL_CLIENTE,
  DEL_CONSULTOR,
  type EstadoDeEscucha,
  type EstadoDePista,
  type EstadoDeLaPantalla,
  type Reunion,
  type Salida,
} from "../cuaderno";
import { invasivos, useRadarDeTuMac, type EnTuMac } from "../radar";

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
}: {
  reunion: Reunion;
  escucha: EstadoDeEscucha;
  salida: Salida;
  radarDeMuestra?: boolean;
}) {
  const t = useT().cuaderno;
  const pantalla = usePantalla();
  const hayEco = salida.salida === "altavoces";
  const caida = (p: EstadoDePista) => escucha.escuchando && !p.abierta;

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

  const titulo = (
    <div className="titulo">
      <h1>{t.sesionTitulo}</h1>
      <p className="sub">{t.sesionSub}</p>
    </div>
  );

  // La muestra de escucha de la maqueta está «escuchando» (es la de las pistas que funcionan); el
  // estado «software invasivo en tu Mac» se dibuja antes de empezar, y así se fotografía.
  const antesDeEmpezar = !escucha.escuchando || radarDeMuestra;
  if (vigilado && antesDeEmpezar) {
    return (
      <>
        {titulo}
        <LaVigilancia
          radar={radar}
          iniciar={() => {
            setVisto(clave);
            empezarAEscuchar(DEL_CONSULTOR, DEL_CLIENTE);
          }}
          noIniciar={() => setVisto(clave)}
        />
      </>
    );
  }

  return (
    <>
      {titulo}

      <div style={PILA}>
        {vigilado && <LaVigilancia radar={radar} />}
        <LaReunion reunion={reunion} />

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
          </div>

          <div className="tarjeta pendiente">
            <h2 className="seccion">{t.esteCliente}</h2>
            <div className="fila">
              <span className="crece">{t.fichaYNda}</span>
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
          <LaEscucha
            mic={!caida(escucha.microfono)}
            sistema={!caida(escucha.sistema)}
          />
          {/* El kill-switch salió de esta lista y bajó a la fila de la acción, al lado de la
              promesa que cumple: «corta todo · el sonido nunca se guarda · nada sale de tu
              equipo». La lista se quedó con lo que sí lleva la palabra «Funciona», y la pantalla
              volvió a caber en los 640 px de la ventana — que es lo que el gate de fidelidad
              midió y el ojo no. */}
          <div className="fila">
            <button
              className="btn primario"
              onClick={() =>
                escucha.escuchando
                  ? dejarDeEscuchar()
                  : empezarAEscuchar(DEL_CONSULTOR, DEL_CLIENTE)
              }
            >
              <Ic id="i-voz" s />
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

export function LaReunion({ reunion }: { reunion: Reunion }) {
  const t = useT().cuaderno;
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
        <span className="tecla" style={{ fontSize: "11px" }}>
          <kbd>⌃⌥L</kbd> {t.leelaAhora}
        </span>
      </div>
    </>
  );
}

/**
 * Por dónde sale el sonido. Con los altavoces internos el micrófono oye al cliente (eco); con un
 * dispositivo externo **la app no puede saber** si es un casco o un altavoz, así que dice su nombre
 * y lo único que importa: si es un altavoz, el modo solo audio no se usa.
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
            <p>{t.panelProtegido}</p>
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
