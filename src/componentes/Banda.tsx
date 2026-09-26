import { useIdioma, useT } from "../i18n";
import { Ic } from "./Iconos";
import { useAsa } from "../asa";
import { useTurnos } from "../turnos";
import { useFicha, type Acumulada, type Aparicion } from "../ficha";
import { hayTauri } from "../puente";
import { cortarTodo, useCorpus, useEscucha, useReunion, useVoz, type LaVoz, type Turno } from "../cuaderno";
import { abrirLoQueVe, type Programa } from "../radar";

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
  | "sin-verificar"
  /** Los tres del modo solo audio (C15). Fuera de Tauri los pide el arnés de capturas por la URL. */
  | "voz"
  | "voz-espera"
  | "voz-sin"
  /**
   * Los tres de la lectura de pantalla y la ficha que se explica (sprint 002, miradas 17-bis y
   * 17-quater). Dentro de Tauri los decide lo que llega; fuera, la URL, para el arnés de capturas.
   */
  | "ficha-pdf"
  | "ficha-pantalla"
  | "pantalla-nada"
  /**
   * El radar (C14, fase 4 del sprint 002, miradas 17 y 17-ter): **ámbar** «te graban» —la pantalla
   * de la reunión muestra el aviso de grabación o un bot de notas— y **coral** «te vigilan» —un
   * programa de tu Mac—. Dentro de Tauri los decide el radar; fuera, la URL.
   */
  | "radar"
  | "radar-invasivo";

export type PropsBanda = {
  estado: EstadoBanda;
  /** El asa arriba: 200 px en vez de 88. */
  ampliada?: boolean;
  /**
   * El transcript en vivo (⌃⌥T), oculto por defecto. Vive en la columna derecha y por eso
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
  const idioma = useIdioma();
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
  const { aparicion, buscando, nadaEnPantalla, radar } = useFicha(estado);

  // **Dentro del producto el estado de contenido lo decide la ficha, no la URL.** Tener las dos
  // cosas mandando a la vez fue un defecto real: la banda pedía «sin resultado» y la ficha traía
  // una ficha, así que no se pintaba nada. `sin-verificar` es la excepción y no es capricho: lo
  // decide la protección de la ventana, que no tiene nada que ver con el corpus.
  const pedido: EstadoBanda = !hayTauri()
    ? estado
    : estado === "sin-verificar"
      ? estado
      : // El radar es lo último que pasó cuando está: cualquier cosa nueva lo quita (`useFicha`).
        radar?.que === "ambar"
        ? "radar"
        : radar?.que === "coral"
          ? "radar-invasivo"
          : nadaEnPantalla !== null
        ? "pantalla-nada"
        : buscando
          ? "buscando"
          : aparicion?.clase === "ficha"
            ? "ficha"
            : aparicion?.clase === "sinResultado"
              ? "sin-resultado"
              : "esperando";
  // La ficha del PDF y la de la pantalla SON fichas: lo que cambia es su línea de «por qué», y eso
  // lo dice la aparición. Se dibujan con la misma rama.
  const estadoReal: EstadoBanda =
    pedido === "ficha-pdf" || pedido === "ficha-pantalla" ? "ficha" : pedido;
  // Los turnos se piden SIEMPRE, no solo con el transcript abierto: el hueco entre abrirlo y
  // recibir la primera respuesta se vería como un transcript vacío, y un transcript vacío en una
  // reunión con gente hablando parece una avería.
  const turnos = useTurnos();
  const asa = useAsa();
  const voz = useVoz();

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

  /** ««MinutaBot»» con las comillas de cada idioma. */
  const entreComillas = (x: string) => `${t.comillaAbre}${x}${t.comillaCierra}`;

  /**
   * La línea del ámbar, compuesta de las piezas de la maqueta: «Meet muestra el aviso de grabación
   * y «MinutaBot» está en la lista de participantes. Ese bot no es Angel Ghost, que nunca entra a
   * la llamada. Aviso, no bloqueo.» Si hay dos bots se nombra el primero —la frase es singular— y
   * si no se sabe el cliente, se dice lo demás sin inventarlo.
   */
  const fraseDelAmbar = (grabando: boolean, bots: string[]) => {
    const partes: string[] = [];
    const [bot] = bots;
    if (grabando && cliente)
      partes.push(`${cliente} ${t.radarMuestraElAviso}${bot ? ` ${t.radarY}` : "."}`);
    if (bot) partes.push(`${entreComillas(bot)} ${t.radarEnLaLista} ${t.radarNoEsAngel}`);
    partes.push(t.radarAvisoNoBloqueo);
    return partes.join(" ");
  };

  /** «ProctorLince» (supervisión de exámenes): ve tu pantalla completa y tu cámara. */
  const fraseDelCoral = (p: Programa) =>
    `${entreComillas(p.nombre)} (${t.radarClases[p.categoria]}): ${p.ve[idioma]}.`;

  const atajos = (
    <span className="atajos-b">
      <span className="tecla">
        <kbd>⌃⌥A</kbd> {t.ayudame}
      </span>
      <span className="tecla">
        <kbd>⌥⎋</kbd> {t.corta}
      </span>
    </span>
  );

  const atajosDeFicha = (
    <span className="atajos-b">
      <span className="tecla">
        <kbd>⌃⌥P</kbd> {t.fijar}
      </span>
      <span className="tecla">
        <kbd>⌃⌥T</kbd> {t.transcript}
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
    </span>
  );

  /**
   * **Por qué llegó y cuánto tardó** (mirada 17-bis). Los dos se miden desde el sprint 001 y
   * cruzaban la costura sin que nadie los pintara. El reloj, porque el número es tiempo; «en
   * pantalla» lleva la pantalla, porque es la única ficha que llega sin que nadie diga nada
   * (17-quater). **La sección conjeturada se marca aquí y no en la fuente**: la fuente se corta con
   * puntos suspensivos cuando es larga, y un aviso de confianza no puede ser lo primero que se
   * corta.
   */
  const porQue = (f: Aparicion & { clase: "ficha" }) => (
    <span className="meta-b por-que">
      <Ic id={f.motivo === "pantalla" ? "i-pantalla" : "i-reloj"} s />
      {t.motivos[f.motivo]} · {segundos(f.ms, idioma)}
      {f.fuente.conjeturada && (
        <span className="conjetura">
          <Ic id="i-half" s />
          {t.seccionConjeturada}
        </span>
      )}
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

  /**
   * **EL MODO SOLO AUDIO (C15) — y por qué sale por aquí y no por otro estado más.**
   *
   * La banda de 44 px no es «la banda con menos cosas»: es **otra anatomía**. No tiene cabecera
   * —a ese alto no cabe— y todo su contenido es una línea. Meterla en el `cuerpo-b` de abajo
   * habría obligado a envolver la cabecera entera en una condición y a que cada estado supiera
   * de los dos altos. Se sale antes, con su propia sección, igual que la maqueta la dibuja aparte.
   *
   * Dentro de Tauri manda la parte nativa (`⌃⌥V` cambia el alto de la VENTANA, y el webview solo
   * obedece); fuera manda la URL, que es como el arnés de capturas fotografía los tres encuadres.
   */
  const enVoz = deLaMaqueta
    ? estado === "voz" || estado === "voz-espera" || estado === "voz-sin"
    : voz.encendida;
  if (enVoz) {
    const comoEsta: LaVoz = deLaMaqueta
      ? {
          encendida: true,
          puede: estado !== "voz-sin",
          diciendo: estado === "voz",
        }
      : voz;
    return (
      <BandaDeVoz
        voz={comoEsta}
        aparicion={aparicion?.clase === "ficha" ? aparicion : null}
        asa={asa}
      />
    );
  }

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

        {estadoReal === "radar" && radar?.que === "ambar" && (
          <>
            {/* Ámbar, «sábelo» (mirada 17): no es un fallo ni una alarma. `i-rec` es un anillo con
                un punto y NO lleva `relleno`: rellenarlo se come el anillo. */}
            <span className="ficha-b">
              <span className="aviso-b" role="status">
                <Ic id="i-rec" />
                <span>
                  <strong>
                    {radar.grabando && radar.bots.length > 0
                      ? t.radarGrabadaYBot
                      : radar.grabando
                        ? t.radarGrabada
                        : t.radarBot}
                  </strong>
                  <span className="salida">{fraseDelAmbar(radar.grabando, radar.bots)}</span>
                </span>
              </span>
            </span>
            <span className="lado-b">
              <span className="meta-b">
                <Ic id="i-pantalla" s />
                {t.radarLeidoDeTuPantalla} · {radar.hora}
              </span>
              {atajos}
            </span>
          </>
        )}

        {estadoReal === "radar-invasivo" && radar?.que === "coral" && !grande && (
          <>
            {/* Coral, con OTRO símbolo además del color: la equis rellena frente al punto de
                grabación del ámbar (regla 8). A 88 px no caben botones: dice qué alcanza a ver y
                deja las acciones para la ampliada, y `⌃⌥R` para ir a verlo. */}
            <span className="ficha-b">
              <span className="aviso-b err" role="alert">
                <Ic id="i-x-circle" relleno />
                <span>
                  <strong>{t.radarTeMira}</strong>
                  <span className="salida">{fraseDelCoral(radar.programa)}</span>
                </span>
              </span>
            </span>
            <span className="lado-b">
              <span className="meta-b">
                <Ic id="i-radar" s />
                {t.radarCatalogo} v{radar.catalogo.version} · {t.radarEnTuEquipo}
              </span>
              <span className="atajos-b">
                <span className="tecla">
                  <kbd>⌃⌥R</kbd> {t.radarQueVe}
                </span>
                <span className="tecla">
                  <kbd>⌥⎋</kbd> {t.corta}
                </span>
              </span>
            </span>
          </>
        )}

        {estadoReal === "radar-invasivo" && radar?.que === "coral" && grande && (
          <>
            {/* Con el asa subida, que es donde la banda tiene botones. Ninguno apaga nada del otro
                equipo: uno enseña qué alcanza a ver y el otro es el kill-switch de siempre. */}
            <span className="ficha-b">
              <div className="franja err" role="alert">
                <Ic id="i-x-circle" relleno />
                <div>
                  <strong>{t.radarTeMira}</strong>
                  <p>{fraseDelCoral(radar.programa)}</p>
                </div>
              </div>
              <span className="meta-b" style={{ marginTop: "8px" }}>
                <Ic id="i-check-circle" s relleno />
                {t.radarSigueProtegida} · {RED} {t.radarEnRed} · {t.radarNadaPersiste}
              </span>
            </span>
            <span className="lado-b">
              <span className="acciones-b">
                <button className="btn mini" type="button" onClick={abrirLoQueVe}>
                  <Ic id="i-radar" s />
                  {t.radarVerQueVe}
                </button>
                <button className="btn mini" type="button" onClick={() => void cortarTodo()}>
                  <Ic id="i-rayo" s />
                  {t.radarCortaTodo}
                </button>
              </span>
              <span className="atajos-b">
                <span className="tecla">
                  <kbd>⌥⎋</kbd> {t.corta}
                </span>
              </span>
            </span>
          </>
        )}

        {estadoReal === "pantalla-nada" && (
          <>
            {/* `⌃⌥L` leyó y no había texto —una cámara, un vídeo, una pantalla en negro—. Se
                contesta igual, porque alguien preguntó: sin esta línea la tecla parecería rota. */}
            <span className="ficha-b">
              <span className="voz-b">{t.leiLaPantalla}</span>
              <span className="meta-b">
                <Ic id="i-pantalla" s />
                {t.motivos.atajo} · {deLaMaqueta ? m.hora3 : nadaEnPantalla}
              </span>
            </span>
            <span className="lado-b">{atajos}</span>
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
              {!transcript && porQue(aparicion)}
              {transcript && <Transcript turnos={turnos} />}
              {transcript ? (
                <span className="atajos-b">
                  <span className="tecla">
                    <kbd>⌃⌥P</kbd> {t.fijar}
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
                      {t.buscarOtras} <kbd className="tecla">⌃⌥A</kbd>
                    </button>
                    <button className="btn mini" type="button">
                      <Ic id="i-nota" s />
                      {t.anotarDespues} <kbd className="tecla">⌃⌥N</kbd>
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
                      <kbd>⌃⌥A</kbd> {t.otrasPalabras}
                    </span>
                    <span className="tecla">
                      <kbd>⌃⌥N</kbd> {t.anotar}
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
 * **LA BANDA A 44 PX — el modo solo audio.**
 *
 * Nació de una frase del usuario en la mirada 3 de la etapa de diseño: *«quisiera tener un modo
 * solo audio que me hable de forma paralela por si quiero ver completamente la pantalla y no me
 * interrumpa»*. Las dos mitades están aquí: la voz la dice `habla/`, y **la pantalla la devuelve
 * este alto** — que es lo que el usuario nombró al aprobar la mirada 16: *«amplio margen para la
 * pantalla de reunión»*.
 *
 * Referencia visual: `docs/diseno/banda.html`, estados «voz» y «voz-sin» (mirada 16, aprobada el
 * 2026-09-26) y «voz-espera» (mirada 16-bis, **pendiente**).
 *
 * **Los tres estados, y de dónde sale cada uno.** Rust manda tres booleanos y ninguna frase: el
 * copy vive en `src/i18n/`, donde el gate del diccionario lo compara con la maqueta.
 *
 * | Lo que llega | Lo que se pinta |
 * |---|---|
 * | `puede: false` | «Conecta auriculares · el cliente te oiría», en ámbar y con el glifo tachado |
 * | `diciendo: true` | «Diciéndote la ficha…» y de dónde sale |
 * | el resto | **la línea de la ficha que acaba de leerse**, con su fuente |
 *
 * **El contador de red se queda, aunque la cabecera no.** A 44 px desaparece el `cab-b` entero, y
 * con él se habría ido el «0 B». Es una promesa dura de la app (regla 2), no un adorno, así que
 * baja a la línea. Es una de las tres decisiones que el usuario aprobó en la mirada 16.
 */
function BandaDeVoz({
  voz,
  aparicion,
  asa,
}: {
  voz: LaVoz;
  aparicion: (Aparicion & { clase: "ficha" }) | null;
  asa: React.RefObject<HTMLSpanElement | null>;
}) {
  const t = useT().banda;
  const estado = !voz.puede ? "voz-sin" : voz.diciendo ? "voz" : "voz-espera";

  /** El contador, que a este alto vive en la línea y no en la cabecera. */
  const red = (
    <span className="red cero mono">
      <Ic id="i-subir" s />
      {RED}
    </span>
  );

  /**
   * De dónde sale la ficha. **Sin ficha no se escribe una fuente inventada**: se calla, que es lo
   * mismo que hace la banda de 88 px desde el hallazgo A1 de la auditoría del sprint 001.
   */
  const deDonde = aparicion && (
    <>
      <span className="sep">·</span>
      <span className="fuente-b">
        {aparicion.fuente.unidad && (
          <span className="unidad">{t.unidades[aparicion.fuente.unidad]}</span>
        )}{" "}
        {aparicion.fuente.seccion
          ? `${aparicion.fuente.documento} · ${aparicion.fuente.seccion}`
          : aparicion.fuente.documento}
      </span>
    </>
  );

  return (
    // **El `aria-label` es el nombre del producto y nada más, y eso es una diferencia declarada
    // con la maqueta.** `banda.html` escribe `aria-label="Angel Ghost · modo solo audio"`, que en
    // la sala de diseño está bien porque el andamiaje es solo español; aquí sería **texto de
    // accesibilidad sin traducir en una interfaz inglesa** — el hallazgo A6 de la auditoría del
    // sprint 001, que fue exactamente eso. Quien use un lector de pantalla se entera del modo por
    // la línea, que sí es bilingüe; y el estado, para los tests y el gate de fidelidad, viaja en
    // `data-estado` como en la banda de 88 px.
    <section className="banda voz" aria-label="Angel Ghost" data-estado={estado}>
      {/* El asa sigue siendo la misma y sigue haciendo lo mismo: arrastrarla saca del modo,
          porque el alto ES el modo. No hace falta una tecla distinta para lo que ya se hace
          tirando de la banda. */}
      <span className="asa" ref={asa} title="arrastra para volver a la banda de 88 px">
        <i />
      </span>

      <div className="cuerpo-b">
        {estado === "voz-sin" ? (
          <>
            <span className="ficha-b">
              {/* Símbolo + texto + color, los tres (regla 8): el glifo de los auriculares
                  tachados, la frase, y el ámbar. Ninguno solo. */}
              <span className="linea-b warn">
                <Ic id="i-auriculares-off" s />
                <span>{t.conectaAuriculares}</span>
                <span className="sep">·</span>
                <span>{t.elClienteTeOiria}</span>
              </span>
            </span>
            <span className="lado-b">
              <span className="atajos-b">
                <span className="tecla">
                  <kbd>⌃⌥V</kbd> {t.volver}
                </span>
              </span>
              {red}
            </span>
          </>
        ) : (
          <>
            <span className="ficha-b">
              <span className="linea-b">
                <Ic id="i-voz" s />
                {/* **El mismo hueco, dos verbos.** Diciendo: «Diciéndote la ficha…». Callado: el
                    estado, dicho — «Callado · esperando el siguiente turno»—, que es lo que el
                    usuario pidió al mirar la 16-bis: *«creo que sí debería hacer evidente el
                    estado»*. La primera propuesta enseñaba la línea de la ficha y se callaba el
                    estado; una banda que no dice en qué está deja al usuario mirándola para
                    averiguarlo, que es justo lo contrario de un modo que existe para no mirar. */}
                {voz.diciendo ? (
                  <span>{t.diciendoLaFicha}</span>
                ) : (
                  <>
                    <span>{t.callado}</span>
                    <span className="sep">·</span>
                    <span>{t.esperandoElSiguienteTurno}</span>
                  </>
                )}
                {/* Y detrás, de dónde salió lo ÚLTIMO que se leyó: es lo que convierte «callado» en
                    una frase útil en vez de un cartel. Sin ficha todavía no hay nada que citar y no
                    se cita nada. */}
                {deDonde}
              </span>
            </span>
            <span className="lado-b">
              <span className="atajos-b">
                {/* `⎋` solo aparece mientras suena, porque solo está registrada mientras suena:
                    fuera de ese rato la tecla es de la reunión, no nuestra. */}
                {voz.diciendo && (
                  <span className="tecla">
                    <kbd>⎋</kbd> {t.callar}
                  </span>
                )}
                <span className="tecla">
                  <kbd>⌃⌥V</kbd> {t.volver}
                </span>
              </span>
              {red}
            </span>
          </>
        )}
      </div>
    </section>
  );
}

/** Milisegundos a «1,2 s» / «1.2 s»: el separador decimal es interfaz, y la app es bilingüe. */
function segundos(ms: number, idioma: string): string {
  const s = new Intl.NumberFormat(idioma, { minimumFractionDigits: 1, maximumFractionDigits: 1 });
  return `${s.format(ms / 1000)} s`;
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
          <kbd>⌃⌥T</kbd> {t.ocultar}
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
              {/* La hora y **cuánto duró** el turno (mirada 17-bis): con el tramo se ve de un
                  vistazo quién habló mucho y quién poco, sin leer. */}
              {/* Una sola cadena y no cuatro nodos de texto: partida, el avance fraccional de la
                  línea cambiaba y todo el texto citado se desplazaba una fracción de píxel —0,17 %
                  de divergencia contra la maqueta, que la escribe de una pieza—. */}
              <span className="hora">
                {`${turno.hora} · ${Math.max(1, Math.round((turno.hastaMs - turno.desdeMs) / 1000))} s`}
              </span>
            </span>
            <q>{turno.texto}</q>
          </span>
        );
      })}
    </span>
  );
}
