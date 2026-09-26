import { render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { Cascara } from "@/App";
import { Principal } from "@/componentes/Principal";
import { SpriteIconos } from "@/componentes/Iconos";
import { es } from "@/i18n/es";
import {
  LaEscucha,
  LaPantalla,
  LaPista,
  LosAuriculares,
  Sesion,
} from "@/pantallas/Sesion";
import { Permisos } from "@/pantallas/Permisos";
import { Idioma, TuDiccionario } from "@/pantallas/Idioma";
import { LaReunion } from "@/pantallas/Sesion";
import {
  ESTADO_DE_LA_ESCUCHA,
  ESTADO_DEL_DICCIONARIO,
  PANTALLA_APAGADA,
  PANTALLA_ESPERANDO_LA_REUNION,
  PANTALLA_LEYENDO,
  PANTALLA_NO_PUDO,
  PANTALLA_SIN_PERMISO,
  PERMISOS,
  REUNION_NO_SE_PUEDE_SABER,
  SALIDA_DE_AUDIO_NO_SE_SABE,
  SALIDA_DE_AUDIO_OTRA,
} from "@/contrato.generado";
import type { EstadoDeEscucha, EstadoDePista, Reunion, Salida } from "@/cuaderno";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

/**
 * LAS PANTALLAS DEL CUADERNO (fases 2 y 3).
 *
 * El gate de FIDELIDAD ya compara estas pantallas contra la maqueta píxel a píxel, así que aquí
 * no se repite la forma. Lo que se prueba es lo que una imagen no puede afirmar: que **lo que no
 * existe se marca como que no existe**, y que los números que la pantalla enseña vienen de fuera
 * en vez de estar escritos en la interfaz.
 */
const t = es.cuaderno;

function pinta(busqueda: string) {
  // El idioma sale de `html[lang]`, igual que en la maqueta. En jsdom viene vacío y la cáscara
  // cae en el del navegador (en-US), así que se fija aquí: si no, estas aserciones comprobarían
  // el diccionario inglés creyendo comprobar el español.
  document.documentElement.lang = "es";
  return render(
    <Cascara>
      <SpriteIconos />
      <Principal busqueda={busqueda} />
    </Cascara>,
  );
}

const abierta: EstadoDePista = { abierta: true, motivo: null, bytes: 1_920_000 };
const cerrada: EstadoDePista = { abierta: false, motivo: null, bytes: 0 };
const escuchando: EstadoDeEscucha = {
  escuchando: true,
  microfono: abierta,
  sistema: abierta,
  bytesDelTranscript: 2_048,
};

/** Sesión con lo que el test quiera cambiar: la reunión, la escucha o la salida de audio. */
function pintaSesion(cambios: { reunion?: Reunion; escucha?: EstadoDeEscucha; salida?: Salida }) {
  document.documentElement.lang = "es";
  return render(
    <Cascara>
      <SpriteIconos />
      <Sesion
        reunion={cambios.reunion ?? { que: "ninguna" }}
        escucha={cambios.escucha ?? escuchando}
        salida={cambios.salida ?? { salida: "auriculares" }}
      />
    </Cascara>,
  );
}

describe("el cuaderno: lo que no existe se dice", () => {
  // Un test pinta en inglés; sin esto, los que vienen detrás comprobarían el diccionario inglés
  // creyendo comprobar el español.
  beforeEach(() => {
    document.documentElement.lang = "es";
  });

  /**
   * **Sprint 002: ya no queda ninguna fila pendiente en «Las dos pistas».** La pantalla se lee
   * (C8), y la fila de los auriculares dice el NOMBRE del dispositivo externo y lo único que la
   * app no puede saber de él. Debajo de la pantalla, su interruptor y su tecla (mirada 17-quater).
   */
  it("sesión: las cuatro filas miden, y la pantalla trae su interruptor y su tecla", () => {
    pinta("?pantalla=sesion");
    const pistas = screen.getByText(t.dosPistas).closest(".tarjeta") as HTMLElement;
    expect(within(pistas).getAllByText(t.funciona)).toHaveLength(3);
    expect(within(pistas).queryByText(t.todaviaNo)).toBeNull();
    expect(within(pistas).getByText("AirPods Pro")).toBeInTheDocument();
    expect(within(pistas).getByText(t.siEsUnAltavoz)).toBeInTheDocument();
    const interruptor = within(pistas).getByRole("switch", { name: t.leerlaSola });
    expect(interruptor.getAttribute("aria-checked")).toBe("true");
    expect(within(pistas).getByText("⌃⌥L")).toBeInTheDocument();
  });

  /**
   * Con altavoces internos, el micrófono oye también al cliente. La pantalla **avisa antes de la
   * reunión**, que es cuando el usuario todavía puede ponerse los auriculares.
   */
  it("sesión: con altavoces avisa del eco en vez de dar las dos pistas por limpias", () => {
    pintaSesion({ salida: { salida: "altavoces" } });
    const pistas = screen.getByText(t.dosPistas).closest(".tarjeta") as HTMLElement;
    expect(within(pistas).getByText(t.altavocesInternos)).toBeInTheDocument();
    expect(within(pistas).getByText(t.avisoDelEco)).toBeInTheDocument();
  });

  /**
   * **M2 del sprint 001, pagado: la pista que no abrió lo DICE.** Hasta el sprint 002 las dos
   * decían «Funciona» pase lo que pase. Tres cosas cambian a la vez —símbolo, palabra y color—, el
   * porqué va en su línea con su salida, y la fila «Escucha las dos pistas» deja de afirmar lo que
   * ya no es verdad.
   */
  it("sesión: una pista que no abrió dice por qué, y la escucha pasa a «a medias»", () => {
    pintaSesion({
      escucha: {
        ...escuchando,
        sistema: { abierta: false, motivo: "dispositivo-ocupado", bytes: 0 },
      },
    });
    const pistas = screen.getByText(t.dosPistas).closest(".tarjeta") as HTMLElement;
    expect(within(pistas).getByText(t.noAbrio)).toBeInTheDocument();
    expect(
      within(pistas).getByText(t.porQueNoAbrio["dispositivo-ocupado"]),
    ).toBeInTheDocument();
    const hoy = screen.getByText(t.queFuncionaHoy).closest(".tarjeta") as HTMLElement;
    expect(within(hoy).queryByText(t.funcionaEscucha)).toBeNull();
    expect(within(hoy).getByText(t.soloTuPista)).toBeInTheDocument();
    expect(within(hoy).getByText(t.aMedias)).toBeInTheDocument();
  });

  it("sesión: cada uno de los cinco porqués de una pista caída tiene su frase", () => {
    for (const motivo of [
      "sin-permiso-del-microfono",
      "sin-permiso-del-audio",
      "dispositivo-ocupado",
      "formato-ilegible",
      "no-dejo",
    ] as const) {
      const { unmount } = render(
        <Cascara>
          <LaPista
            pista={{ abierta: false, motivo, bytes: 0 }}
            caida
            icono="i-mic"
            texto={t.pistaMic}
          />
        </Cascara>,
      );
      expect(screen.getByText(t.porQueNoAbrio[motivo])).toBeInTheDocument();
      unmount();
    }
  });

  /** Fuera de una sesión nada está abierto, y eso no es una avería: no se pinta «no abrió». */
  it("sesión: sin sesión, una pista cerrada no se da por caída", () => {
    pintaSesion({ escucha: { ...escuchando, escuchando: false, microfono: cerrada, sistema: cerrada } });
    expect(screen.queryByText(t.noAbrio)).toBeNull();
  });

  it("sesión: con el micrófono caído, lo que queda es la pista del cliente", () => {
    render(
      <Cascara>
        <LaEscucha mic={false} sistema />
      </Cascara>,
    );
    expect(screen.getByText(t.soloLaDelCliente)).toBeInTheDocument();
  });

  /**
   * La pantalla (C8), sus cinco vistas: cada una con su palabra, y las tres que no leen con su
   * porqué. **Apagada, el interruptor dice apagado pero la tecla sigue**: es la salida para una
   * NDA estricta.
   */
  it("sesión: la pantalla dice cada una de sus cinco vistas", () => {
    const casos = [
      { vista: "leyendo", chip: t.funciona, porque: null },
      { vista: "esperando-la-reunion", chip: t.pantallaEspera, porque: null },
      { vista: "apagada", chip: t.pantallaApagada, porque: t.pantallaApagadaPor },
      { vista: "sin-permiso", chip: t.pantallaSinPermiso, porque: t.pantallaSinPermisoPor },
      { vista: "no-pudo", chip: t.pantallaNoPudo, porque: t.pantallaNoPudoPor },
    ] as const;
    for (const { vista, chip, porque } of casos) {
      const { unmount } = render(
        <Cascara>
          <LaPantalla pantalla={{ vista, bytesEnMemoria: 0 }} />
        </Cascara>,
      );
      expect(screen.getByText(chip)).toBeInTheDocument();
      if (porque) expect(screen.getByText(porque)).toBeInTheDocument();
      expect(screen.getByRole("switch").getAttribute("aria-checked")).toBe(
        vista === "apagada" ? "false" : "true",
      );
      expect(screen.getByText("⌃⌥L")).toBeInTheDocument();
      unmount();
    }
  });

  /** «No se sabe» cita el dispositivo cuando lo hay: el nombre no se traduce, la frase sí. */
  it("sesión: la salida que no se sabe dice por qué, con el nombre del dispositivo", () => {
    render(
      <Cascara>
        <LosAuriculares salida={{ salida: "no-se-sabe", motivo: "sin-conexion", nombre: "Altavoz USB" }} />
      </Cascara>,
    );
    expect(screen.getByText(t.noSeSabe)).toBeInTheDocument();
    expect(
      screen.getByText(`«Altavoz USB» ${t.porQueNoSeSabe["sin-conexion"]}`),
    ).toBeInTheDocument();
  });

  /** Sin Accesibilidad no se ve si hay Meet: decir «sin reunión» afirmaría lo que no se sabe. */
  it("sesión: sin poder mirar, no dice «sin reunión»", () => {
    pintaSesion({ reunion: { que: "no-se-puede-saber", motivo: "sin-accesibilidad" } });
    expect(screen.getByText(t.noSePuedeSaber)).toBeInTheDocument();
    expect(screen.getByText(t.porQueNoSeVe["sin-accesibilidad"])).toBeInTheDocument();
    expect(screen.queryByText(t.sinReunion)).toBeNull();
  });

  /**
   * La tarjeta que impide que la pantalla mienta por omisión: el sprint 001 SÍ entrega la banda
   * protegida, el acople y el kill-switch, y una pantalla llena de «todavía no» se leería como
   * que no hay nada.
   */
  it("sesión enseña lo que sí funciona hoy, y ya no queda nada pendiente en esa tarjeta", () => {
    pinta("?pantalla=sesion");
    const hoy = screen
      .getByText(t.queFuncionaHoy)
      .closest(".tarjeta") as HTMLElement;
    expect(within(hoy).getAllByText(t.funciona)).toHaveLength(3);
    expect(within(hoy).getByText("⌥⎋")).toBeInTheDocument();
    expect(within(hoy).queryByText(t.todaviaNo)).toBeNull();
    // Y «Iniciar sesión» dejó de ser una promesa: es un botón que se puede pulsar.
    expect(
      screen.getByRole("button", { name: new RegExp(t.iniciarSesion) }),
    ).toBeInTheDocument();
  });

  it("sesión no inventa la ficha del cliente: la declara ausente con su motivo", () => {
    pinta("?pantalla=sesion");
    const cliente = screen
      .getByText(t.esteCliente)
      .closest(".tarjeta") as HTMLElement;
    expect(cliente.className).toContain("pendiente");
    expect(within(cliente).getByText(t.noSeInventa)).toBeInTheDocument();
  });

  /**
   * **«Audio del sistema» y «Pantalla» son DOS permisos** (`kTCCServiceAudioCapture` y
   * `kTCCServiceScreenCapture`). Hasta el sprint 002 esta pantalla decía que eran uno y leía el de
   * pantalla para las dos filas; este test afirmaba lo mismo («muestran el mismo estado») y pasaba
   * en verde. Ahora cada fila lee el suyo, y con uno concedido y otro no, se ven distintos.
   */
  it("permisos: audio del sistema y pantalla son dos permisos, y cada fila lee el suyo", () => {
    const { container } = render(
      <Cascara>
        <Permisos
          permisos={{
            microfono: "concedido",
            audio: "concedido",
            pantalla: "sin-conceder",
            accesibilidad: "concedido",
          }}
        />
      </Cascara>,
    );
    const filas = [...container.querySelectorAll(".permiso")];
    const chip = (i: number) => filas[i].querySelector(".accion .estado")?.textContent;
    expect(chip(1)).toBe(t.concedido);
    expect(chip(2)).toBe(t.sinConceder);
    expect(screen.getByText(t.dosPermisos)).toBeInTheDocument();
    // Y lo que de verdad pasa con la pantalla: macOS pregunta con su frase y no admite otra.
    expect(screen.getByText(t.antesDeQueMacos)).toBeInTheDocument();
    expect(screen.getByText(t.textoPantalla)).toBeInTheDocument();
    expect(screen.getByText(t.fraseDeMacos)).toBeInTheDocument();
  });

  it("permisos: la accesibilidad está en la lista, porque el acople se entrega en este sprint", () => {
    pinta("?pantalla=permisos");
    expect(screen.getByText(t.permAcople)).toBeInTheDocument();
  });

  /**
   * **Y este test decía «tres todavía no» mientras dos de las tres cosas ya funcionaban.**
   *
   * Es la cara simétrica del resto de la auditoría: la app escondiendo lo que SÍ hace, justo en la
   * pantalla que existe para decir qué se puede hacer sin conceder un solo permiso. El test pasaba
   * en verde porque repetía lo que la pantalla decía, no lo que la app hacía — un test escrito
   * contra la interfaz y no contra el producto no puede cazar esto. Hallazgo M11.
   */
  it("permisos: lo que ya se puede hacer sin permisos dice «funciona», y solo lo que falta «todavía no»", () => {
    pinta("?pantalla=permisos");
    const tarjeta = screen
      .getByText(t.sinConcederNada)
      .closest(".tarjeta") as HTMLElement;
    // Indexar el corpus (fase 4) y buscar a mano con ⌃⌥A: ninguna necesita permisos.
    expect(within(tarjeta).getAllByText(t.funciona)).toHaveLength(2);
    // Escribir notas y acuerdos es lo único que todavía no existe.
    expect(within(tarjeta).getAllByText(t.todaviaNo)).toHaveLength(1);
    expect(
      within(tarjeta).getByText(t.escribirNotas).closest(".fila")?.className,
    ).toContain("pendiente");
  });

  /**
   * El cero de la red no puede estar escrito en la interfaz: sería la interfaz AFIRMANDO el cero
   * en vez de medirlo, que es justo lo que esta pantalla existe para no hacer.
   */
  it("honestidad: el contador no es una constante del webview", async () => {
    const { container } = pinta("?pantalla=honestidad");
    const cifra = container.querySelector(".contador .cifra") as HTMLElement;
    expect(cifra.textContent?.replace(/\s+/g, " ").trim()).toBe("0 B");
    // La unidad va en su propio elemento, como en la maqueta: el CSS las dimensiona distinto.
    expect(cifra.querySelector("small")?.textContent).toBe("B");
  });

  /**
   * **Los cuatro búferes existen** desde la fase 3 del sprint 002: el último cuadro leído de la
   * pantalla dejó de ser «todavía no» y se cuenta, en su fila y en el total de RAM.
   */
  it("honestidad: los cuatro búferes traen cifras, el último cuadro incluido", () => {
    const { container } = pinta("?pantalla=honestidad");
    const bufs = [...container.querySelectorAll(".buffer")];
    expect(bufs).toHaveLength(4);
    expect(bufs.filter((b) => b.className.includes("pendiente"))).toHaveLength(0);
    for (const b of bufs) {
      expect(b.querySelector(".cuanto")?.textContent).not.toBe("0 B");
    }
    const cuadro = within(container).getByText(t.bufFrame).closest(".buffer") as HTMLElement;
    expect(within(cuadro).getByText(t.soloEnMemoriaElUltimo)).toBeInTheDocument();
    expect(within(cuadro).getByText("1,4 MB")).toBeInTheDocument();
    // Y el total lo cuenta: sin el cuadro serían 3,7 MB.
    expect(screen.getByText(/RAM · 5,1 MB/)).toBeInTheDocument();
  });

  /**
   * «8 de 8» con una pieza sin construir sería la mentira cómoda — y desde la fase 3 del sprint 002
   * ya no falta ninguna: la lectura de pantalla (C8) era la última, y se corta de verdad.
   *
   * La cuenta la da Rust (`corte::TODAS` con su `match` sin comodín), no esta pantalla: el test lee
   * la muestra del contrato, que es lo que Rust emite.
   */
  it("honestidad: el kill-switch dice cuántas piezas corta de cuántas hay", () => {
    pinta("?pantalla=honestidad");
    // «El botón corta»: sin sujeto, «8 de 8 piezas» se leyó como «leyó todo bien» (mirada 17-quater).
    expect(
      screen.getByText(`${t.botonCorta} 8 ${t.de} 8 ${t.piezasNingunaFuera}`),
    ).toBeInTheDocument();
  });

  /** «de» vive en el diccionario: escrita en el componente, la interfaz inglesa decía «8 de 8 pieces». */
  it("honestidad: la cuenta del kill-switch es bilingüe", () => {
    document.documentElement.lang = "en";
    render(
      <Cascara>
        <SpriteIconos />
        <Principal busqueda="?pantalla=honestidad" />
      </Cascara>,
    );
    expect(screen.getByText(/The button cuts 8 of 8 pieces/)).toBeInTheDocument();
    expect(screen.queryByText(/8 de 8/)).toBeNull();
  });

  /**
   * Idioma, en su estado «sprint 2» (mirada 17-bis): el motor **con nombre y techo**, la tarjeta
   * del diccionario con sus dos filas y la ruta del archivo, y dos pendientes —por su nombre: un
   * conteo que cambia en silencio no dice cuál se fue—.
   */
  it("idioma: enseña el motor, el diccionario y lo que todavía no", () => {
    pinta("?pantalla=idioma");
    expect(screen.getByText(t.idiomaTitulo)).toBeInTheDocument();
    expect(screen.getByText(t.variosIdiomasPorPista)).toBeInTheDocument();
    expect(screen.getByText(t.conservarTusTurnos)).toBeInTheDocument();
    expect(screen.getAllByText(t.todaviaNo)).toHaveLength(2);
    expect(
      screen.getByText(`SpeechAnalyzer · macOS 26 · 5 ${t.idiomasListos}`),
    ).toBeInTheDocument();
    // El diccionario técnico existe desde la fase 1, y se enseña con números de verdad.
    const dicc = screen.getByText(t.tuDiccionario).closest(".tarjeta") as HTMLElement;
    expect(within(dicc).getByText("17")).toBeInTheDocument();
    expect(within(dicc).getByText(t.deTuCorpus)).toBeInTheDocument();
    expect(within(dicc).getByText(/diccionario\.yaml$/)).toBeInTheDocument();
    // Las dos pistas con su idioma y el estado real de su modelo. **La del cliente no lo tiene**,
    // que es el estado más probable en un Mac de verdad: uno en español no trae el modelo de inglés.
    expect(screen.getByText(t.modeloInstalado)).toBeInTheDocument();
    expect(screen.getByText(t.sinModelo)).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: new RegExp(t.instalarModelo) }),
    ).toBeInTheDocument();
  });

  /** Sin motor, la franja lo dice con su porqué —cerrado, y en los dos idiomas— y lo que sigue funcionando. */
  it("idioma: sin motor de voz, dice por qué y qué sigue funcionando", () => {
    for (const motivo of ["sin-transcriptor", "sin-puente", "no-contesta"] as const) {
      const { unmount } = render(
        <Cascara>
          <Idioma transcribe={{ motor: "mudo", techo: 0, idiomas: [], motivo }} />
        </Cascara>,
      );
      expect(screen.getByText(t.sinMotorTitulo)).toBeInTheDocument();
      expect(
        screen.getByText(`${t.porQueNoHayMotor[motivo]} ${t.laBandaSigue}`),
      ).toBeInTheDocument();
      expect(screen.queryByText(t.transcribeTuMac)).toBeNull();
      unmount();
    }
  });

  /**
   * La fase 4 abrió Corpus: pasa de fila apagada a enlace. Notas e IA siguen sin existir y el
   * rail lo dice — un rail lleno de enlaces que no llevan a ninguna parte es peor que uno corto.
   */
  it("el rail deja las secciones que aún no existen sin enlace", () => {
    const { container } = pinta("?pantalla=sesion");
    const rail = container.querySelector("nav.rail") as HTMLElement;
    const enlaces = [...rail.querySelectorAll("a")].map((a) => a.textContent);
    expect(enlaces).toEqual([
      t.navSesion,
      t.navPermisos,
      t.navCorpus,
      t.navHonestidad,
      t.navIdioma,
    ]);
    for (const nombre of [t.navNotas, t.navIa]) {
      const fila = within(rail)
        .getByText(nombre)
        .closest(".item") as HTMLElement;
      expect(fila.className).toContain("pendiente");
      expect(fila.tagName).not.toBe("A");
    }
  });

  /**
   * Corpus nace con la fase 4. Como en Idioma, aquí no se comprueba la forma —de eso se ocupa el
   * gate de fidelidad— sino que **las cifras vienen de fuera** y que lo que falta se marca.
   */
  it("corpus: enseña lo que indexó hoy y marca lo que todavía no", () => {
    const { container } = pinta("?pantalla=corpus");
    // «Corpus» aparece dos veces —en el rail y en el título—, así que se busca el del título.
    expect(container.querySelector(".titulo h1")?.textContent).toBe(
      t.corpusTitulo,
    );
    expect(screen.getAllByText(t.todaviaNo)).toHaveLength(3);
    // Las cinco unidades más la sexta respuesta: lo que no encaja en ninguna.
    expect(container.querySelectorAll(".unidad-chip")).toHaveLength(6);
    expect(screen.getByText(t.sinUnidad)).toBeInTheDocument();
    // El tamaño del índice no puede estar escrito en la interfaz: se mide.
    expect(container.querySelector(".buffer .cuanto")?.textContent).not.toBe(
      "0 B",
    );
    expect(screen.getByText(t.soloTu)).toBeInTheDocument();
    // Qué carpeta señaló el usuario y cuántas secciones salieron de ella (mirada 17-bis).
    expect(screen.getByText(t.tuCarpeta)).toBeInTheDocument();
    expect(screen.getByText(`412 ${t.secciones}`)).toBeInTheDocument();
  });

  it("la pantalla la elige la URL, y una desconocida cae en sesión", () => {
    pinta("?pantalla=inventada");
    expect(screen.getByText(t.sesionTitulo)).toBeInTheDocument();
  });
});

/**
 * **LO QUE RUST EMITE, PINTADO** (regla 19). Cada forma que la fase 3 del sprint 002 hizo cruzar —o
 * cambió— se pinta aquí desde su muestra de `src/contrato.generado.ts`, que escribe Rust con el
 * serde de producción: los porqués cerrados, la salida que cita su dispositivo, el permiso de audio
 * aparte, las cinco vistas de la pantalla y el diccionario. Escribir los literales a mano repetiría
 * el defecto que el gate nació para cazar: dos copias del contrato que coinciden hasta que no.
 */
describe("lo que Rust emite, pintado", () => {
  beforeEach(() => {
    document.documentElement.lang = "es";
  });
  const conCascara = (hijo: React.ReactNode) => render(<Cascara>{hijo}</Cascara>);

  it("la pista caída de la muestra trae su porqué", () => {
    if (ESTADO_DE_LA_ESCUCHA.sistema.motivo === null) throw new Error("la muestra dejó de traer motivo");
    pintaSesion({ escucha: ESTADO_DE_LA_ESCUCHA });
    expect(screen.getByText(t.porQueNoAbrio[ESTADO_DE_LA_ESCUCHA.sistema.motivo])).toBeInTheDocument();
  });

  it("la salida que no se sabe y la externa, con su nombre", () => {
    const { unmount } = conCascara(<LosAuriculares salida={SALIDA_DE_AUDIO_NO_SE_SABE} />);
    if (SALIDA_DE_AUDIO_NO_SE_SABE.salida !== "no-se-sabe") throw new Error("cambió la muestra");
    expect(
      screen.getByText(
        `«${SALIDA_DE_AUDIO_NO_SE_SABE.nombre}» ${t.porQueNoSeSabe[SALIDA_DE_AUDIO_NO_SE_SABE.motivo]}`,
      ),
    ).toBeInTheDocument();
    unmount();
    conCascara(<LosAuriculares salida={SALIDA_DE_AUDIO_OTRA} />);
    if (SALIDA_DE_AUDIO_OTRA.salida !== "otra") throw new Error("cambió la muestra");
    expect(screen.getByText(SALIDA_DE_AUDIO_OTRA.nombre)).toBeInTheDocument();
  });

  it("la reunión que no se puede ver", () => {
    conCascara(<LaReunion reunion={REUNION_NO_SE_PUEDE_SABER} />);
    if (REUNION_NO_SE_PUEDE_SABER.que !== "no-se-puede-saber") throw new Error("cambió la muestra");
    expect(screen.getByText(t.porQueNoSeVe[REUNION_NO_SE_PUEDE_SABER.motivo])).toBeInTheDocument();
  });

  it("los permisos de la muestra, cada uno en su fila", () => {
    const { container } = conCascara(<Permisos permisos={PERMISOS} />);
    const chips = [...container.querySelectorAll(".permiso .accion .estado")].map((c) => c.textContent);
    const palabra = (e: string) => (e === "concedido" ? t.concedido : t.sinConceder);
    expect(chips).toEqual([
      palabra(PERMISOS.microfono),
      palabra(PERMISOS.audio),
      palabra(PERMISOS.pantalla),
      palabra(PERMISOS.accesibilidad),
    ]);
  });

  it("las cinco vistas de la pantalla, desde sus muestras", () => {
    const palabra = {
      leyendo: t.funciona,
      "esperando-la-reunion": t.pantallaEspera,
      apagada: t.pantallaApagada,
      "sin-permiso": t.pantallaSinPermiso,
      "no-pudo": t.pantallaNoPudo,
    } as const;
    for (const muestra of [
      PANTALLA_LEYENDO,
      PANTALLA_ESPERANDO_LA_REUNION,
      PANTALLA_APAGADA,
      PANTALLA_SIN_PERMISO,
      PANTALLA_NO_PUDO,
    ]) {
      const { unmount } = conCascara(<LaPantalla pantalla={muestra} />);
      expect(screen.getByText(palabra[muestra.vista])).toBeInTheDocument();
      unmount();
    }
  });

  it("el diccionario de la muestra: el total, sus dos filas y la ruta", () => {
    const { container } = conCascara(<TuDiccionario diccionario={ESTADO_DEL_DICCIONARIO} />);
    const numeros = [...container.querySelectorAll(".num")].map((n) => Number(n.textContent));
    expect(numeros).toEqual([ESTADO_DEL_DICCIONARIO.delCorpus, ESTADO_DEL_DICCIONARIO.enTuArchivo]);
    expect(screen.getByText(String(ESTADO_DEL_DICCIONARIO.terminos))).toBeInTheDocument();
    expect(screen.getByText(ESTADO_DEL_DICCIONARIO.ruta)).toBeInTheDocument();
  });
});
