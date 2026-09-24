import { useT } from "../i18n";
import { Ic } from "./Iconos";
import { useAsa } from "../asa";
import { useTurnos } from "../turnos";
import { useFicha, type Acumulada, type Aparicion } from "../ficha";
import { hayTauri } from "../puente";
import { useCorpus, useEscucha, useReunion, type Turno } from "../cuaderno";

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
  const tc = useT().cuaderno;
  const m = t.muestra;
  const grande = ampliada || transcript;
  /**
   * **Fuera de Tauri manda la maqueta; dentro, lo que de verdad hay.**
   *
   * La banda llevaba todo el sprint pintando dentro del producto los datos de muestra: «Escuchando
   * · 2 pistas» aunque nadie escuchara, «143 documentos» sin corpus señalado, «Páramo Azul · 12
   * min» sin reunión, «Meet · protegido» en una llamada de Zoom, y —lo peor— **una frase inventada
   * puesta en boca del cliente, con su hora**. Hallazgo A1 de la auditoría.
   *
   * Fuera de Tauri las cadenas de la maqueta siguen tal cual, y no es una puerta trasera: es lo que
   * fotografía el arnés del gate de FIDELIDAD, que compara esta ventana contra `banda.html`. Dentro
   * del producto cada una viene de su comando: la escucha, la reunión, el corpus y los turnos.
   */
  const deLaMaqueta = !hayTauri();
  const reunion = useReunion();
  const escucha = useEscucha();
  const corpus = useCorpus();
  // La ficha viene del corpus del usuario. Fuera de Tauri es la de la maqueta, que es lo que
  // hace posible el gate de FIDELIDAD; dentro del producto es la de verdad, y si no hay ninguna
  // no se pinta ninguna.
  const { aparicion, buscando } = useFicha(estado);

  // **Dentro del producto el estado de contenido lo decide la ficha, no la URL.** Tener las dos
  // cosas mandando a la vez fue un defecto real: la banda pedía «sin resultado» y la ficha traía
  // una ficha, así que no se pintaba nada. `sin-verificar` es la excepción y no es capricho: lo
  // decide la protección de la ventana, que no tiene nada que ver con el corpus.
  const estadoReal: EstadoBanda = !hayTauri()
    ? estado
    : estado === "sin-verificar"
      ? estado
      : buscando
        ? "buscando"
        : aparicion?.clase === "ficha"
          ? "ficha"
          : aparicion?.clase === "sinResultado"
            ? "sin-resultado"
            : "esperando";
  // Los turnos se piden SIEMPRE, no solo con el transcript abierto: el hueco entre abrirlo y
  // recibir la primera respuesta se vería como un transcript vacío, y un transcript vacío en una
  // reunión con gente hablando parece una avería.
  const turnos = useTurnos();
  const asa = useAsa();

  /** «1 documento» y no «1 documentos»: el contador es visible y la app es bilingüe por regla. */
  const cuenta = (n: number, uno: string, varios: string) => `${n} ${n === 1 ? uno : varios}`;

  const pistasAbiertas = [escucha.microfono, escucha.sistema].filter((p) => p.abierta).length;
  const cliente = reunion.que === "detectada" ? reunion.cliente : null;
  /**
   * **La protección es un hecho comprobado por cliente de videollamada, no una etiqueta.** Estaba
   * puesta a `true` por defecto, así que la banda decía «protegido» en Zoom — contra la promesa
   * graduada que la app hace por escrito. El parámetro de URL sigue mandando fuera de Tauri porque
   * el arnés de capturas tiene que poder recorrer los dos encuadres.
   */
  const protegido = deLaMaqueta
    ? verificado
    : reunion.que === "detectada" && reunion.proteccion === "Verificada";

  /**
   * Lo último que dijo el cliente, que es de quien son las palabras. Se saca de los turnos que ya
   * están en la banda —pista del sistema, sin eco— y si no hay ninguno **no se dibuja nada**: la
   * alternativa era la que había, inventar una frase y ponerle una hora.
   */
  const delCliente = turnos.filter((x) => x.pista === "sistema" && !x.eco);
  const ultimoDelCliente = delCliente[delCliente.length - 1];
  const oido = deLaMaqueta
    ? { quien: m.oidoQuien, texto: m.oido }
    : ultimoDelCliente && {
        quien: `${t.cliente} ${ultimoDelCliente.hora}`,
        texto: ultimoDelCliente.texto,
      };

  /** El chip del cliente: su nombre y si la protección está comprobada **en ese** cliente. */
  const chipDelCliente = () => {
    if (deLaMaqueta) {
      return protegido ? (
        <span className="cliente-b">{t.protegido}</span>
      ) : (
        <span className="cliente-b warn">
          <Ic id="i-alert" s relleno />
          {t.sinVerificar}
        </span>
      );
    }
    // Sin reunión no hay nada que proteger, y decirlo no es una advertencia: es el estado normal
    // de la banda mientras el usuario prepara la llamada.
    if (cliente === null) return <span className="cliente-b">{tc.sinReunion}</span>;
    return protegido ? (
      <span className="cliente-b">
        {cliente} · {t.protegidoSufijo}
      </span>
    ) : (
      <span className="cliente-b warn">
        <Ic id="i-alert" s relleno />
        {cliente} · {t.sinVerificarSufijo}
      </span>
    );
  };

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

  const fuente = (f: Aparicion & { clase: "ficha" }) => (
    <span className="fuente-b">
      {f.fuente.unidad && <span className="unidad">{t.unidades[f.fuente.unidad]}</span>}{" "}
      {f.fuente.seccion ? `${f.fuente.documento} · ${f.fuente.seccion}` : f.fuente.documento}
      {/* Un PDF no trae títulos y los suyos son conjetura del lector. Aquí NO se marca: en 88 px
          no cabe copy nuevo, y el sitio donde el usuario puede juzgar cómo se leyeron sus
          documentos es la pantalla de Corpus, que lo cuenta. La sección se cita igual porque es
          una línea que está de verdad en el documento — lo conjeturado es que fuera un título. */}
    </span>
  );

  /** Las acumuladas y las cercanas se dibujan igual: unidad + texto. */
  const lista = (items: Acumulada[], estilo?: React.CSSProperties) => (
    <span className="mas-b" style={estilo}>
      {items.map((a, i) => (
        <span className="item" key={`${a.texto}-${i}`}>
          {a.unidad && <span className="unidad">{t.unidades[a.unidad]}</span>}
          <span className="t">{a.texto}</span>
        </span>
      ))}
    </span>
  );

  return (
    <section
      className={grande ? "banda ampliada" : "banda"}
      aria-label="Angel Ghost"
      data-estado={estadoReal}
    >
      {/* El asa ajusta la banda Y su relleno a la vez; el arrastre lo resuelve Rust. */}
      <span className="asa" ref={asa} title="arrastra para ajustar las dos a la vez">
        <i />
      </span>

      <div className="cab-b">
        {/* «Escuchando» solo cuando se está escuchando de verdad, y con las pistas que de verdad
            se abrieron. Antes lo decía siempre — también durante la media hora en que la banda
            está abierta y el usuario todavía no ha pulsado «Iniciar sesión». */}
        {(deLaMaqueta || escucha.escuchando) && (
          <span className="marca-min">
            <Ic id="i-check-circle" s relleno />
            {deLaMaqueta
              ? t.escuchando
              : `${t.escuchandoPrefijo} · ${cuenta(pistasAbiertas, t.pista, t.pistas)}`}
          </span>
        )}

        {!acoplada && (
          <span className="cliente-b warn">
            <Ic id="i-alert" s relleno />
            {t.sinAcople}
          </span>
        )}

        {chipDelCliente()}

        <span className="red cero mono">
          <Ic id="i-subir" s />
          {RED}
        </span>
      </div>

      <div className="cuerpo-b">
        {estadoReal === "esperando" && (
          <>
            <span className="ficha-b">
              <span className="voz-b">{t.esperando}</span>
              {/* El corpus se compone con los números que el índice tiene, no con los de la
                  maqueta. Con los datos de muestra sale la misma línea que `banda.html` dibuja —
                  143 documentos y cinco unidades—, así que el encuadre del gate no se mueve. */}
              <span className="meta-b">
                <Ic id="i-doc" s />
                {cuenta(corpus.documentos, t.documento, t.documentos)} ·{" "}
                {cuenta(
                  corpus.porUnidad.filter((u) => u.documentos > 0).length,
                  t.unidadPalabra,
                  t.unidadesPalabra,
                )}
              </span>
            </span>
            <span className="lado-b">
              <span className="meta-b">
                <Ic id="i-reloj" s />
                {/* El nombre de la reunión sale del detector. **Los «12 min» de la maqueta no se
                    pintan dentro del producto**: nadie mide todavía cuánto lleva la llamada, y
                    escribir un número que no se mide es lo que esta pantalla existe para no hacer.
                    Queda declarado en la bitácora para la mirada. */}
                {deLaMaqueta
                  ? t.reunion
                  : reunion.que === "detectada"
                    ? (reunion.titulo ?? reunion.cliente)
                    : tc.sinReunion}
              </span>
              {atajos}
            </span>
          </>
        )}

        {estadoReal === "buscando" && (
          <>
            <span className="ficha-b">
              {oido && (
                <span className="oido">
                  <span className="quien">
                    <Ic id="i-sistema" s /> {oido.quien}
                  </span>
                  <q>{oido.texto}</q>
                </span>
              )}
              <span className="meta-b">
                <Ic id="i-buscar" s />
                {t.buscando}
              </span>
            </span>
            <span className="lado-b">{atajos}</span>
          </>
        )}

        {estadoReal === "ficha" && aparicion?.clase === "ficha" && (
          <>
            <span className="ficha-b">
              <span className="titular-b">{aparicion.titular}</span>
              <span className="linea-b">{grande ? aparicion.lineaLarga : aparicion.linea}</span>
              {grande && lista(aparicion.acumuladas)}
            </span>
            <span className="lado-b">
              {fuente(aparicion)}
              {transcript && <Transcript turnos={turnos} />}
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

        {estadoReal === "sin-resultado" && aparicion?.clase === "sinResultado" && (
          <>
            <span className="ficha-b">
              {/* Con los términos que DE VERDAD se buscaron: si la app entendió mal, se ve en el
                  acto en vez de después, cuando la ficha ya se creyó. */}
              <span className="titular-b">
                {t.nadaSobre} {t.comillaAbre}
                {aparicion.buscado}
                {t.comillaCierra}
              </span>
              {grande && (deLaMaqueta || oido) && (
                <span className="oido">
                  <span className="quien">
                    <Ic id="i-sistema" s />{" "}
                    {deLaMaqueta ? `${t.cliente} ${aparicion.hora}` : oido?.quien}
                  </span>
                  <q>{deLaMaqueta ? m.oidoIso : oido?.texto}</q>
                </span>
              )}
              <span className="maniobra-b">
                <Ic id="i-flecha" s />
                <span className="t">{t.maniobras[aparicion.maniobra]}</span>
              </span>
              {grande && aparicion.cercanas.length > 0 && (
                <>
                  <span className="cercano-b" style={{ marginTop: "7px" }}>
                    <span className="et">{t.cercanoLargo}</span>
                  </span>
                  {lista(aparicion.cercanas, { marginTop: "3px" })}
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
                  {aparicion.cercanas[0] && (
                    <span className="cercano-b">
                      <span className="et">{t.cercano}</span>
                      <span className="d">
                        {aparicion.cercanas[0].unidad
                          ? `${t.unidades[aparicion.cercanas[0].unidad]} · ${aparicion.cercanas[0].texto}`
                          : aparicion.cercanas[0].texto}
                      </span>
                    </span>
                  )}
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

        {estadoReal === "sin-verificar" && (
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
function Transcript({ turnos }: { turnos: Turno[] }) {
  const t = useT().banda;
  return (
    <span className="transcript-b" aria-label="Transcript">
      <span className="cab">
        <Ic id="i-ojo" s />
        {t.transcriptCab}
        <span className="tecla">
          <kbd>⌘⇧T</kbd> {t.ocultar}
        </span>
      </span>
      {turnos.map((turno, i) => {
        // **El eco no se atribuye a nadie.** Si el micrófono captó por los altavoces lo que decía
        // el cliente, ese turno no es del consultor — pintarlo como «tú» sería ponerle en la boca
        // palabras que no dijo. Se enseña como del cliente, que es de quien son.
        const mio = turno.pista === "microfono" && !turno.eco;
        return (
          <span className={mio ? "turno tu" : "turno"} key={`${turno.desdeMs}-${i}`}>
            <span className="quien">
              <Ic id={mio ? "i-mic" : "i-sistema"} s />
              {mio ? t.tu : t.cliente}
              <span className="hora">{turno.hora}</span>
            </span>
            <q>{turno.texto}</q>
          </span>
        );
      })}
    </span>
  );
}
