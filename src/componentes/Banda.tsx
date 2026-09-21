import { useT } from "../i18n";
import { Ic } from "./Iconos";

/**
 * LA BANDA — la forma principal de Angel Ghost durante una reunión.
 *
 * Reproduce la anatomía y las clases canon de `docs/diseno/assets/ghost.css` tal cual; el CSS es
 * literalmente el mismo archivo (lo importa `src/main.tsx`), así que aquí no hay un solo valor
 * de estilo. La referencia visual es `docs/diseno/banda.html` (mirada 11) y el gate de FIDELIDAD
 * compara esta ventana contra ella, en los dos temas y los dos idiomas.
 *
 * Gramática de la banda (design-system §9-quinquies): **lo que la banda dice va a la izquierda**
 * (`ficha-b`); **de dónde sale y qué teclas hay, a la derecha** (`lado-b`). Y las acciones son
 * TECLAS: 88 px es una superficie de teclado, no de clic. Los botones vuelven al ampliar.
 */

export type EstadoBanda =
  | "esperando"
  | "buscando"
  | "ficha"
  | "sin-resultado"
  | "sin-verificar";

export type PropsBanda = {
  estado: EstadoBanda;
  /** El asa arriba: 200 px en vez de 88. */
  ampliada?: boolean;
  /**
   * El transcript en vivo (⌘⇧T), oculto por defecto. Vive en la columna derecha y por eso
   * **solo existe en la banda ampliada**: en 88 px no caben tres turnos. Encenderlo desde la
   * banda compacta la amplía, que es lo mismo que haría el asa.
   */
  transcript?: boolean;
  /**
   * Con el permiso de Accesibilidad la banda recorta la reunión; sin él flota encima. Es el
   * único permiso opcional de la app y el fallback está declarado en la maqueta.
   */
  acoplada?: boolean;
  /** El cliente detectado tiene la protección verificada por el spike. */
  verificado?: boolean;
};

/** En el sprint 001 no hay red: ni un byte sale del equipo, y el contador lo dice. */
const RED = "0 B";

export function Banda({
  estado,
  ampliada = false,
  transcript = false,
  acoplada = true,
  verificado = true,
}: PropsBanda) {
  const t = useT().banda;
  const m = t.muestra;
  const grande = ampliada || transcript;

  const atajos = (
    <span className="atajos-b">
      <span className="tecla">
        <kbd>⌘⇧A</kbd> {t.ayudame}
      </span>
      <span className="tecla">
        <kbd>⌥⎋</kbd> {t.corta}
      </span>
    </span>
  );

  const atajosDeFicha = (
    <span className="atajos-b">
      <span className="tecla">
        <kbd>⌘⇧P</kbd> {t.fijar}
      </span>
      <span className="tecla">
        <kbd>⌘⇧T</kbd> {t.transcript}
      </span>
      <span className="tecla">
        <kbd>⌥⎋</kbd>
      </span>
    </span>
  );

  const fuente = (
    <span className="fuente-b">
      <span className="unidad">{m.unidad}</span> {m.fuente}
    </span>
  );

  return (
    <section
      className={grande ? "banda ampliada" : "banda"}
      aria-label="Angel Ghost"
      data-estado={estado}
    >
      <span className="asa">
        <i />
      </span>

      <div className="cab-b">
        <span className="marca-min">
          <Ic id="i-check-circle" s relleno />
          {t.escuchando}
        </span>

        {!acoplada && (
          <span className="cliente-b warn">
            <Ic id="i-alert" s relleno />
            {t.sinAcople}
          </span>
        )}

        {verificado ? (
          <span className="cliente-b">{t.protegido}</span>
        ) : (
          <span className="cliente-b warn">
            <Ic id="i-alert" s relleno />
            {t.sinVerificar}
          </span>
        )}

        <span className="red cero mono">
          <Ic id="i-subir" s />
          {RED}
        </span>
      </div>

      <div className="cuerpo-b">
        {estado === "esperando" && (
          <>
            <span className="ficha-b">
              <span className="voz-b">{t.esperando}</span>
              <span className="meta-b">
                <Ic id="i-doc" s />
                {t.corpus}
              </span>
            </span>
            <span className="lado-b">
              <span className="meta-b">
                <Ic id="i-reloj" s />
                {t.reunion}
              </span>
              {atajos}
            </span>
          </>
        )}

        {estado === "buscando" && (
          <>
            <span className="ficha-b">
              <span className="oido">
                <span className="quien">
                  <Ic id="i-sistema" s /> {m.oidoQuien}
                </span>
                <q>{m.oido}</q>
              </span>
              <span className="meta-b">
                <Ic id="i-buscar" s />
                {t.buscando}
              </span>
            </span>
            <span className="lado-b">{atajos}</span>
          </>
        )}

        {estado === "ficha" && (
          <>
            <span className="ficha-b">
              <span className="titular-b">{m.titular}</span>
              <span className="linea-b">{grande ? m.lineaLarga : m.linea}</span>
              {grande && (
                <span className="mas-b">
                  <span className="item">
                    <span className="unidad">{m.acumulada1Unidad}</span>
                    <span className="t">{m.acumulada1}</span>
                  </span>
                  <span className="item">
                    <span className="unidad">{m.acumulada2Unidad}</span>
                    <span className="t">{m.acumulada2}</span>
                  </span>
                </span>
              )}
            </span>
            <span className="lado-b">
              {fuente}
              {transcript && <Transcript />}
              {transcript ? (
                <span className="atajos-b">
                  <span className="tecla">
                    <kbd>⌘⇧P</kbd> {t.fijar}
                  </span>
                  <span className="tecla">
                    <kbd>⌥⎋</kbd>
                  </span>
                </span>
              ) : (
                atajosDeFicha
              )}
            </span>
          </>
        )}

        {estado === "sin-resultado" && (
          <>
            <span className="ficha-b">
              <span className="titular-b">{t.nada}</span>
              {grande && (
                <span className="oido">
                  <span className="quien">
                    <Ic id="i-sistema" s /> {m.oidoQuien}
                  </span>
                  <q>{m.oidoIso}</q>
                </span>
              )}
              <span className="maniobra-b">
                <Ic id="i-flecha" s />
                <span className="t">{t.maniobra}</span>
              </span>
              {grande && (
                <>
                  <span className="cercano-b" style={{ marginTop: "7px" }}>
                    <span className="et">{t.cercanoLargo}</span>
                  </span>
                  <span className="mas-b" style={{ marginTop: "3px" }}>
                    <span className="item">
                      <span className="unidad">{m.cercana1Unidad}</span>
                      <span className="t">{m.cercana1}</span>
                    </span>
                    <span className="item">
                      <span className="unidad">{m.cercana2Unidad}</span>
                      <span className="t">{m.cercana2}</span>
                    </span>
                    <span className="item">
                      <span className="unidad">{m.cercana3Unidad}</span>
                      <span className="t">{m.cercana3}</span>
                    </span>
                  </span>
                </>
              )}
            </span>
            <span className="lado-b">
              {grande ? (
                <>
                  <span className="acciones-b">
                    <button className="btn mini" type="button">
                      <Ic id="i-buscar" s />
                      {t.buscarOtras} <kbd className="tecla">⌘⇧A</kbd>
                    </button>
                    <button className="btn mini" type="button">
                      <Ic id="i-nota" s />
                      {t.anotarDespues} <kbd className="tecla">⌘⇧N</kbd>
                    </button>
                  </span>
                  <span className="atajos-b">
                    <span className="tecla">
                      <kbd>⌥⎋</kbd> {t.corta}
                    </span>
                  </span>
                </>
              ) : (
                <>
                  <span className="cercano-b">
                    <span className="et">{t.cercano}</span>
                    <span className="d">{m.cercana}</span>
                  </span>
                  <span className="atajos-b">
                    <span className="tecla">
                      <kbd>⌘⇧A</kbd> {t.otrasPalabras}
                    </span>
                    <span className="tecla">
                      <kbd>⌘⇧N</kbd> {t.anotar}
                    </span>
                    <span className="tecla">
                      <kbd>⌥⎋</kbd>
                    </span>
                  </span>
                </>
              )}
            </span>
          </>
        )}

        {estado === "sin-verificar" && (
          <>
            <span className="ficha-b">
              {grande ? (
                <div className="franja warn" role="status">
                  <Ic id="i-alert" relleno />
                  <div>
                    <strong>{t.sinVerificarTitulo}</strong>
                    <p>{t.sinVerificarCuando}</p>
                    <ul className="vertical">
                      <li>{t.sinVerificarVentana}</li>
                      <li>{t.sinVerificarMonitor}</li>
                      <li>{t.sinVerificarNotas}</li>
                    </ul>
                  </div>
                </div>
              ) : (
                <span className="aviso-b" role="status">
                  <Ic id="i-alert" relleno />
                  <span>
                    <strong>{t.sinVerificarTitulo}</strong>
                    <span className="salida">{t.sinVerificarSalida}</span>
                  </span>
                </span>
              )}
            </span>
            <span className="lado-b">
              {grande ? (
                <>
                  <span className="acciones-b">
                    <button className="btn mini" type="button">
                      <Ic id="i-check-circle" s relleno />
                      {t.yaVerifique}
                    </button>
                    <button className="btn mini" type="button">
                      <Ic id="i-nota" s />
                      {t.soloNotas}
                    </button>
                  </span>
                  <span className="atajos-b">
                    <span className="tecla">
                      <kbd>⌥⎋</kbd> {t.corta}
                    </span>
                  </span>
                </>
              ) : (
                <>
                  <span className="meta-b">{t.sinVerificarCuando}</span>
                  {atajos}
                </>
              )}
            </span>
          </>
        )}
      </div>
    </section>
  );
}

/**
 * El transcript en vivo. **Solo en memoria**: cada turno lleva su pista (sistema = cliente,
 * mic = tú), jamás un nombre — la atribución se resuelve por pista y nunca por biometría.
 */
function Transcript() {
  const t = useT().banda;
  const m = t.muestra;
  return (
    <span className="transcript-b" aria-label="Transcript">
      <span className="cab">
        <Ic id="i-ojo" s />
        {t.transcriptCab}
        <span className="tecla">
          <kbd>⌘⇧T</kbd> {t.ocultar}
        </span>
      </span>
      <span className="turno">
        <span className="quien">
          <Ic id="i-sistema" s />
          {t.cliente} {m.hora1}
        </span>
        <q>{m.turno1}</q>
      </span>
      <span className="turno tu">
        <span className="quien">
          <Ic id="i-mic" s />
          {t.tu} {m.hora2}
        </span>
        <q>{m.turno2}</q>
      </span>
      <span className="turno">
        <span className="quien">
          <Ic id="i-sistema" s />
          {t.cliente} {m.hora2}
        </span>
        <q>{m.turno3}</q>
      </span>
    </span>
  );
}
