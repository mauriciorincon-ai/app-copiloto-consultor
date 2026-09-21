import { useT } from "../i18n";
import { Ic } from "../componentes/Iconos";
import { Fila, TodaviaNo, PILA } from "../componentes/Ventana";
import { abrirAjustesDe, type EstadoPermiso, type Permisos as EstadoDeLosPermisos } from "../cuaderno";

/**
 * PERMISOS — «Permisos de macOS».
 *
 * Referencia: `docs/diseno/permisos.html`, estado **«así se ve hoy · sprint 1»** (mirada 12).
 *
 * **Se leen, no se piden.** El botón lleva al panel de Ajustes del Sistema: lo concede el usuario
 * allí. Es decisión de la maqueta —*«Tú los concedes en el sistema, no aquí»*— y además es lo
 * correcto: un permiso pedido en el primer arranque, antes de que la app haya demostrado nada, se
 * deniega, y un «no» de macOS es mucho más caro de deshacer que un «todavía no».
 *
 * Dos cosas que la maqueta no había escrito y el sistema obliga (§9-sexies): **«Audio del
 * sistema» y «Pantalla» son un solo permiso** —se dibujan como dos filas porque son dos usos
 * distintos, con una línea que lo dice—, y **la Accesibilidad sube a la lista principal**, porque
 * el acople se entrega en este sprint y es el único permiso que hoy cambia algo.
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
  cual: "microfono" | "pantalla" | "accesibilidad";
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
            estado={permisos.pantalla}
            cual="pantalla"
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
            {t.unSoloPermiso}
          </p>
        </div>

        <div className="grid-2">
          <div className="tarjeta pendiente">
            <h2 className="seccion">{t.sinConcederNada}</h2>
            <Fila icono="i-doc" texto={t.indexar} pendiente>
              <TodaviaNo />
            </Fila>
            <Fila icono="i-nota" texto={t.escribirNotas} pendiente>
              <TodaviaNo />
            </Fila>
            <Fila icono="i-buscar" texto={t.buscarAMano} pendiente>
              <TodaviaNo />
            </Fila>
          </div>
          <div className="tarjeta">
            <h2 className="seccion">{t.queTextoVeras}</h2>
            <p style={{ fontFamily: "var(--font-evidencia)", fontSize: "14px", color: "var(--ink)" }}>
              {t.textoMicrofono}
            </p>
            <p className="mono" style={{ color: "var(--ink-2)" }}>
              {t.claveMicrofono}
            </p>
          </div>
        </div>
      </div>
    </>
  );
}
