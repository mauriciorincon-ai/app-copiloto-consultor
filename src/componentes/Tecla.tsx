import { useTeclas } from "../i18n";

/**
 * **Una tecla dibujada, tecla por tecla** (`<span class="tecla"><kbd>⌃</kbd><kbd>⌥</kbd><kbd>T</kbd></span>`, como la
 * maqueta). En macOS, un `<kbd>` por símbolo; en Windows, uno por tecla de la combinación (`Alt`, `Mayús`, `H`), que
 * es como Windows las escribe. El texto de la combinación viene de `useTeclas()`, la misma tabla que usa el
 * diccionario: ninguna pantalla repite qué tecla es de qué acción.
 */
export function Tecla({ combo }: { combo: string }) {
  const escrita = useTeclas()(combo);
  const teclas = escrita === combo ? [...combo] : escrita.split("+");
  return (
    <span className="tecla">
      {teclas.map((t, i) => (
        <kbd key={`${t}-${i}`}>{t}</kbd>
      ))}
    </span>
  );
}
