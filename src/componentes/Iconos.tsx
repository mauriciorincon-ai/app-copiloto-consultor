// El sprite de iconos del design system, tomado de la maqueta SIN copiarlo.
//
// `docs/diseno/assets/iconos.js` es un script clásico (la maqueta se abre con doble clic y
// `file://` no admite módulos), así que no se puede importar como tal. Pero sí como TEXTO: se
// extrae el sprite en tiempo de compilación y se inyecta igual que en la maqueta. Una sola
// fuente para los cuarenta símbolos; cero posibilidad de deriva entre maqueta y producto, que
// es justo lo que un gate de fidelidad no puede comprobar (un icono equivocado se ve, uno que
// falta en silencio no).
import fuente from "../../docs/diseno/assets/iconos.js?raw";

/** El sprite: lo que va entre las comillas invertidas del script de la maqueta. */
function extraer(js: string): string {
  const m = js.match(/`(<svg[\s\S]*?<\/svg>)`/);
  if (!m) {
    // Falla ruidosamente: sin sprite, la banda se dibuja sin un solo icono y el estado deja de
    // ser «símbolo + texto + color». Es exactamente el fallo que nadie nota hasta la demo.
    throw new Error(
      "no se pudo extraer el sprite de docs/diseno/assets/iconos.js (¿cambió su forma?)",
    );
  }
  return m[1];
}

export const SPRITE = extraer(fuente);

/** Ids que el sprite declara, para que un test pueda comprobar los que la banda usa. */
export function idsDelSprite(sprite = SPRITE): string[] {
  return [...sprite.matchAll(/id="(i-[a-z0-9-]+)"/g)].map((m) => m[1]);
}

/** Se monta una vez por ventana; todos los `<use href="#i-…">` cuelgan de aquí. */
export function SpriteIconos() {
  // El contenido es una constante de compilación sacada de un archivo de este mismo repo:
  // no hay entrada de usuario en juego.
  return <div hidden aria-hidden="true" dangerouslySetInnerHTML={{ __html: SPRITE }} />;
}

/** Un icono del sistema. `s` = 13 px; `relleno` = sólido en vez de trazo. */
export function Ic({
  id,
  s,
  relleno,
}: {
  id: string;
  s?: boolean;
  relleno?: boolean;
}) {
  const clases = ["ic", s ? "s" : "", relleno ? "relleno" : ""].filter(Boolean).join(" ");
  return (
    <svg className={clases} aria-hidden="true">
      <use href={`#${id}`} />
    </svg>
  );
}
