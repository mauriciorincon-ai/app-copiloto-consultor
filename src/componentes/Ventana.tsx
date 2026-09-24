import type { ReactNode } from "react";
import { useT } from "../i18n";
import { Ic } from "./Iconos";

/**
 * LA VENTANA DEL CUADERNO (960 × 640) — el marco de las pantallas de la ventana principal.
 *
 * Reproduce `main.ventana > nav.rail + div.contenido` de la maqueta, con las clases canon de
 * `ghost.css`. El rail es el mismo en las siete pantallas; lo que cambia es el contenido.
 *
 * Las secciones que aún no existen **siguen en el rail**, apagadas. Quitarlas escondería que la
 * app va a tenerlas; ponerlas navegables prometería una pantalla que no está. Es la misma
 * decisión que «todavía no» (design-system §9-sexies), aplicada a la navegación.
 */
export type Seccion = "sesion" | "permisos" | "corpus" | "honestidad" | "idioma";

const RAIL: { id: Seccion | null; icono: string; clave: keyof ReturnType<typeof useT>["cuaderno"] }[] = [
  { id: "sesion", icono: "i-video", clave: "navSesion" },
  { id: "permisos", icono: "i-candado", clave: "navPermisos" },
  { id: "corpus", icono: "i-doc", clave: "navCorpus" },
  { id: null, icono: "i-nota", clave: "navNotas" },
  { id: "honestidad", icono: "i-ram", clave: "navHonestidad" },
  { id: "idioma", icono: "i-globo", clave: "navIdioma" },
  { id: null, icono: "i-chispa", clave: "navIa" },
];

/** Cómo se llama una sección, con la misma palabra que usa el rail. */
function nombreDe(t: ReturnType<typeof useT>["cuaderno"], seccion: Seccion): string {
  const fila = RAIL.find((r) => r.id === seccion);
  return fila ? String(t[fila.clave]) : "";
}

export function Ventana({
  seccion,
  ir,
  enSesion = false,
  children,
}: {
  seccion: Seccion;
  ir: (s: Seccion) => void;
  /** Hay una reunión detectada: el chip de abajo lo dice. */
  enSesion?: boolean;
  children: ReactNode;
}) {
  const t = useT().cuaderno;

  return (
    <main className="ventana">
      <nav className="rail">
        <div className="marca">
          <Ic id="i-ojo-off" />
          <span>
            {t.marca}
            <small>{t.marcaSub}</small>
          </span>
        </div>

        {RAIL.map((fila) => {
          const texto = t[fila.clave];
          if (fila.id === null) {
            return (
              // `item` es la clase canon del rail: sin ella la fila pierde el `display:flex`,
              // el hueco y el relleno, y el icono se pega al texto. Se vio en la comparación —a
              // ojo la lista «parecía bien» y valía 10 % de divergencia.
              <span key={fila.clave} className="item pendiente" aria-disabled="true">
                <Ic id={fila.icono} s />
                <span>{texto}</span>
              </span>
            );
          }
          const actual = fila.id === seccion;
          return (
            <a
              key={fila.clave}
              href={`?pantalla=${fila.id}`}
              aria-current={actual ? "page" : undefined}
              onClick={(e) => {
                e.preventDefault();
                ir(fila.id as Seccion);
              }}
            >
              <Ic id={fila.icono} s />
              <span>{texto}</span>
            </a>
          );
        })}

        <div className="abajo">
          {enSesion ? (
            <span className="estado halo">
              <Ic id="i-video" s />
              <span>{t.meetDetectado}</span>
            </span>
          ) : (
            <span className="estado mute">
              <Ic id="i-ring" s />
              <span>{t.sinSesion}</span>
            </span>
          )}
        </div>
      </nav>

      {/* Esto no es adorno. La ventana principal **es redimensionable**, y cuando el usuario la
          hace más baja este contenedor se vuelve desplazable: sin foco, lo que queda por debajo
          del borde no se alcanza con el teclado. Axe lo nombró `scrollable-region-focusable` en
          la pantalla de Idioma — el primer hallazgo del primer e2e de accesibilidad de esta app,
          que llevaba cuatro fases sin correr.
          El nombre sale del rail y no de una cadena nueva: es la misma palabra que el usuario
          acaba de pulsar para llegar aquí. */}
      <div className="contenido" role="region" aria-label={nombreDe(t, seccion)} tabIndex={0}>
        {children}
      </div>
    </main>
  );
}

/** El chip «todavía no» (§9-sexies). Trazo discontinuo, aro punteado y la palabra: tres señales,
 *  y la principal no es el color — el color es el mismo gris que «apagado». */
export function TodaviaNo() {
  const t = useT().cuaderno;
  return (
    <span className="estado pendiente">
      <Ic id="i-pendiente" s />
      <span>{t.todaviaNo}</span>
    </span>
  );
}

export function Funciona() {
  const t = useT().cuaderno;
  return (
    <span className="estado ok">
      <Ic id="i-check-circle" s relleno />
      <span>{t.funciona}</span>
    </span>
  );
}

/** Una fila de tarjeta: icono + texto que crece + lo que diga a la derecha. */
export function Fila({
  icono,
  color,
  texto,
  pendiente,
  children,
}: {
  icono: string;
  color?: string;
  texto: string;
  pendiente?: boolean;
  children?: ReactNode;
}) {
  return (
    <div className={pendiente ? "fila pendiente" : "fila"}>
      <Ic id={icono} s color={color} />
      <span className="crece">{texto}</span>
      {children}
    </div>
  );
}

/**
 * La PILA de bloques de una pantalla del cuaderno: 14 px entre ellos.
 *
 * No son los 16 px que `.contenido` pone entre sus hijos: en la maqueta cada pantalla envuelve
 * sus bloques en un contenedor propio con `gap: 14px`, y esos 2 px de diferencia se ACUMULAN —
 * tres bloques más abajo la pantalla entera va desplazada 6 px. A ojo «se ve igual»; la resta de
 * mapas de bits lo llamó 9 % de divergencia.
 */
export const PILA = { display: "flex", flexDirection: "column", gap: "14px" } as const;
