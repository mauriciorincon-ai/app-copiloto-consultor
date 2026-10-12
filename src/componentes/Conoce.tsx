import { useState } from "react";
import { Ic } from "./Iconos";
import { useTextosDeConoce } from "../i18n/conoce";
import { abrirConoce } from "../cuaderno";
import { hayTauri } from "../puente";

/**
 * «CONOCE ANGEL GHOST» — el pie de Honestidad que abre el recorrido visual (`docs/BROCHURE.html`) en el navegador.
 *
 * Solo dentro de la app: fuera de Tauri no hay recurso que abrir, y la pantalla de Honestidad aprobada en la maqueta no
 * cambia (el gate de fidelidad la fotografía fuera de Tauri). El archivo es local y el botón no toca la red: lo comprueba
 * `tests/unit/conoce-angel-ghost.test.ts`.
 *
 * **Estado: maquetado, no visto** (sprint 008): va al gate humano del hito.
 */
export function Conoce() {
  const t = useTextosDeConoce();
  const [fallo, setFallo] = useState(false);
  if (!hayTauri()) return null;
  return (
    <section className="tarjeta" aria-labelledby="conoce-angel-ghost">
      <h2 id="conoce-angel-ghost" className="seccion">
        {t.titulo}
      </h2>
      <p>{t.detalle}</p>
      <button
        className="btn"
        type="button"
        onClick={() => {
          setFallo(false);
          void abrirConoce().then((ok) => setFallo(!ok));
        }}
      >
        <Ic id="i-doc" s />
        <span>{t.boton}</span>
      </button>
      {fallo && (
        <p className="estado warn" role="status">
          <Ic id="i-alert" s />
          <span>{t.noSePudo}</span>
        </p>
      )}
    </section>
  );
}
