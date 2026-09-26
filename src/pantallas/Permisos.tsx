import { useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { Fila, Funciona, TodaviaNo, PILA } from "../componentes/Ventana";
import { abrirAjustesDe, type EstadoPermiso, type Permisos as EstadoDeLosPermisos } from "../cuaderno";

/**
 * PERMISOS — «Permisos de macOS».
 *
 * Referencia: `docs/diseno/permisos.html`, estado **«así se ve hoy · sprint 2 · la pantalla»**
 * (miradas 17 y 17-quater).
 *
 * **Se leen, no se piden.** El botón lleva al panel de Ajustes del Sistema: lo concede el usuario
 * allí. Es decisión de la maqueta —*«Tú los concedes en el sistema, no aquí»*— y además es lo
 * correcto: un permiso pedido en el primer arranque, antes de que la app haya demostrado nada, se
 * deniega, y un «no» de macOS es mucho más caro de deshacer que un «todavía no».
 *
 * **«Audio del sistema» y «Pantalla» son DOS permisos** (`kTCCServiceAudioCapture` y
 * `kTCCServiceScreenCapture`), aunque Ajustes los enseñe en el mismo panel. Hasta el sprint 002 esta
 * pantalla decía lo contrario —«un solo permiso: se conceden y se caen juntos»— y leía el de
 * pantalla para las dos filas; la mirada 17-quater lo encontró falso comprobándolo en `tccd`. Cada
 * fila lee ya el suyo. Y la tarjeta de la derecha deja de citar el texto del micrófono para decir lo
 * que de verdad pasa con la pantalla: macOS pregunta con su propia frase y no admite otra.
 */

function Chip({ estado }: { estado: EstadoPermiso }) {
  const t = useT().cuaderno;
  if (estado === "concedido") {
    return (
      <span className="estado ok">
        <Ic id="i-check-circle" s relleno />
        <span>{t.concedido}</span>
      </span>
    );
  }
  // «Denegado» y «no se sabe» comparten la cara de «sin conceder» a propósito: la única acción
  // posible es la misma —ir a Ajustes— y macOS no siempre distingue entre «dijiste que no» y
  // «nunca se preguntó». Inventar la diferencia mandaría al usuario a revocar algo que jamás
  // concedió.
  return (
    <span className="estado mute">
      <Ic id="i-ring" s />
      <span>{t.sinConceder}</span>
    </span>
  );
}

function Permiso({
  icono,
  nombre,
  para,
  estado,
  cual,
}: {
  icono: string;
  nombre: string;
  para: string;
  estado: EstadoPermiso;
  cual: "microfono" | "audio" | "pantalla" | "accesibilidad";
}) {
  const t = useT().cuaderno;
  const concedido = estado === "concedido";
  return (
    <div className={concedido ? "permiso concedido" : "permiso "}>
      <Ic id={icono} />
      <span className="nombre">{nombre}</span>
      <span className="para">{para}</span>
      <div className="accion">
        <Chip estado={estado} />
        {!concedido && (
          <button className="btn mini primario" onClick={() => abrirAjustesDe(cual)}>
            {t.concederEnMacos}
          </button>
        )}
      </div>
    </div>
  );
}

export function Permisos({ permisos }: { permisos: EstadoDeLosPermisos }) {
  const t = useT().cuaderno;

  return (
    <>
      <div className="titulo">
        <h1>{t.permisosTitulo}</h1>
        <p className="sub">{t.permisosSub}</p>
      </div>

      <div style={PILA}>
        <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
          <Permiso
            icono="i-mic"
            nombre={t.permMic}
            para={t.permMicPara}
            estado={permisos.microfono}
            cual="microfono"
          />
          <Permiso
            icono="i-sistema"
            nombre={t.permSistema}
            para={t.permSistemaPara}
            estado={permisos.audio}
            cual="audio"
          />
          <Permiso
            icono="i-pantalla"
            nombre={t.permPantalla}
            para={t.permPantallaPara}
            estado={permisos.pantalla}
            cual="pantalla"
          />
          <Permiso
            icono="i-flecha"
            nombre={t.permAcople}
            para={t.permAcoplePara}
            estado={permisos.accesibilidad}
            cual="accesibilidad"
          />
          <p className="mono" style={{ color: "var(--ink-2)" }}>
            {t.dosPermisos}
          </p>
        </div>

        <div className="grid-2">
          {/* «Qué puedes hacer ya, sin conceder nada» — y desde la fase 4 se pueden hacer DOS de
              las tres. Marcarlas «todavía no» era el error simétrico del resto de la auditoría: la
              app escondiendo lo que sí hace, en la pantalla que existe para decir qué se puede
              hacer sin conceder permisos. Lo encontró el barrido de promesas aplazadas (M11).
              La tarjeta deja de ser `pendiente`: dos de tres funcionan. */}
          <div className="tarjeta">
            <h2 className="seccion">{t.sinConcederNada}</h2>
            <Fila icono="i-doc" texto={t.indexar}>
              <Funciona />
            </Fila>
            <Fila icono="i-nota" texto={t.escribirNotas} pendiente>
              <TodaviaNo />
            </Fila>
            {/* Buscar a mano es `⌃⌥A`: no necesita micrófono ni pantalla, solo el corpus. */}
            <Fila icono="i-buscar" texto={t.buscarAMano}>
              <Funciona />
            </Fila>
          </div>
          <div className="tarjeta">
            <h2 className="seccion">{t.antesDeQueMacos}</h2>
            <p style={{ fontFamily: "var(--font-evidencia)", fontSize: "14px", color: "var(--ink)" }}>
              {t.textoPantalla}
            </p>
            <p style={{ fontSize: "11.5px", color: "var(--ink-2)" }}>{t.fraseDeMacos}</p>
          </div>
        </div>
      </div>
    </>
  );
}
