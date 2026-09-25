import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Cascara } from "@/App";
import { Principal } from "@/componentes/Principal";
import { SpriteIconos } from "@/componentes/Iconos";
import { es } from "@/i18n/es";

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

describe("el cuaderno: lo que no existe se dice", () => {
  /**
   * La fase 3 movió dos pistas de «todavía no» a «funciona», y ese movimiento es el que hay que
   * vigilar: **la pantalla ni se adelanta ni se queda corta**. La de pantalla sigue pendiente
   * (llega en el sprint 2) y la de auriculares dejó de ser una promesa para pasar a medir.
   */
  it("sesión: las pistas que ya escuchan dicen «funciona» y la que no, «todavía no»", () => {
    pinta("?pantalla=sesion");
    const pistas = screen.getByText(t.dosPistas).closest(".tarjeta") as HTMLElement;
    expect(within(pistas).getAllByText(t.funciona)).toHaveLength(2);
    expect(within(pistas).getAllByText(t.todaviaNo)).toHaveLength(1);
    expect(within(pistas).getByText(t.pistaPantalla).closest(".fila")?.className).toContain(
      "pendiente",
    );
  });

  /**
   * Con altavoces internos, el micrófono oye también al cliente. La pantalla **avisa antes de la
   * reunión**, que es cuando el usuario todavía puede ponerse los auriculares.
   */
  it("sesión: con altavoces avisa del eco en vez de dar las dos pistas por limpias", () => {
    pinta("?pantalla=sesion");
    const pistas = screen.getByText(t.dosPistas).closest(".tarjeta") as HTMLElement;
    expect(within(pistas).getByText(t.altavocesInternos)).toBeInTheDocument();
    expect(within(pistas).getByText(t.avisoDelEco)).toBeInTheDocument();
  });

  /**
   * La tarjeta que impide que la pantalla mienta por omisión: el sprint 001 SÍ entrega la banda
   * protegida, el acople y el kill-switch, y una pantalla llena de «todavía no» se leería como
   * que no hay nada.
   */
  it("sesión enseña lo que sí funciona hoy, y ya no queda nada pendiente en esa tarjeta", () => {
    pinta("?pantalla=sesion");
    const hoy = screen.getByText(t.queFuncionaHoy).closest(".tarjeta") as HTMLElement;
    expect(within(hoy).getAllByText(t.funciona)).toHaveLength(3);
    expect(within(hoy).getByText("⌥⎋")).toBeInTheDocument();
    expect(within(hoy).queryByText(t.todaviaNo)).toBeNull();
    // Y «Iniciar sesión» dejó de ser una promesa: es un botón que se puede pulsar.
    expect(screen.getByRole("button", { name: new RegExp(t.iniciarSesion) })).toBeInTheDocument();
  });

  it("sesión no inventa la ficha del cliente: la declara ausente con su motivo", () => {
    pinta("?pantalla=sesion");
    const cliente = screen.getByText(t.esteCliente).closest(".tarjeta") as HTMLElement;
    expect(cliente.className).toContain("pendiente");
    expect(within(cliente).getByText(t.noSeInventa)).toBeInTheDocument();
  });

  /**
   * «Audio del sistema» y «Pantalla» son UN permiso en macOS. Se dibujan como dos filas porque
   * son dos usos distintos, pero **su estado tiene que ser el mismo siempre**: dos chips
   * distintos mandarían al usuario a conceder algo que ya concedió.
   */
  it("permisos: audio del sistema y pantalla muestran el mismo estado, y se dice por qué", () => {
    const { container } = pinta("?pantalla=permisos");
    const filas = [...container.querySelectorAll(".permiso")];
    const chip = (i: number) => filas[i].querySelector(".accion .estado")?.textContent;
    expect(chip(1)).toBe(chip(2));
    expect(screen.getByText(t.unSoloPermiso)).toBeInTheDocument();
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
    const tarjeta = screen.getByText(t.sinConcederNada).closest(".tarjeta") as HTMLElement;
    // Indexar el corpus (fase 4) y buscar a mano con ⌘⇧A: ninguna necesita permisos.
    expect(within(tarjeta).getAllByText(t.funciona)).toHaveLength(2);
    // Escribir notas y acuerdos es lo único que todavía no existe.
    expect(within(tarjeta).getAllByText(t.todaviaNo)).toHaveLength(1);
    expect(within(tarjeta).getByText(t.escribirNotas).closest(".fila")?.className).toContain(
      "pendiente",
    );
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
   * Tres búferes existen y uno no. El que no existe **sigue en la lista**: quitarlo escondería
   * que la app va a leer la pantalla, y ponerlo en cero sin marca lo haría parecer vacío en vez
   * de inexistente.
   */
  it("honestidad: los búferes que existen traen cifras y el que no, su marca", () => {
    const { container } = pinta("?pantalla=honestidad");
    const bufs = [...container.querySelectorAll(".buffer")];
    expect(bufs).toHaveLength(4);
    const vivos = bufs.filter((b) => !b.className.includes("pendiente"));
    expect(vivos).toHaveLength(3);
    for (const b of vivos) {
      expect(b.querySelector(".cuanto")?.textContent).not.toBe("0 B");
    }
    const pendiente = bufs.find((b) => b.className.includes("pendiente")) as HTMLElement;
    expect(within(pendiente).getByText(t.todaviaNo)).toBeInTheDocument();
    expect(pendiente.querySelector(".cuanto")?.textContent).toBe("0 B");
  });

  /** «7 de 7» con una pieza sin construir seguiría siendo la mentira cómoda. */
  it("honestidad: el kill-switch dice cuántas piezas corta de cuántas hay", () => {
    pinta("?pantalla=honestidad");
    expect(screen.getByText(/6 de 7/)).toBeInTheDocument();
    expect(screen.queryByText(/7 de 7/)).toBeNull();
  });

  /**
   * La pantalla de Idioma nace con la fase 3. Lo que se comprueba aquí no es su forma —de eso se
   * ocupa el gate de fidelidad— sino que **no promete lo que no hay** ni niega lo que sí hay.
   *
   * **Eran tres pendientes y son dos.** El «Diccionario técnico» salió de la lista en el sprint 002,
   * fase 1, porque se construyó: una pantalla que dice «todavía no» de algo que existe miente igual
   * que una que promete lo que falta, y esta app no se permite ninguna de las dos. Se comprueban los
   * dos que quedan **por su nombre** y no solo la cuenta: un conteo que cambia en silencio no dice
   * cuál se fue.
   */
  it("idioma: enseña lo que transcribe hoy y marca lo que todavía no", () => {
    pinta("?pantalla=idioma");
    expect(screen.getByText(t.idiomaTitulo)).toBeInTheDocument();
    expect(screen.getByText(t.variosIdiomasPorPista)).toBeInTheDocument();
    expect(screen.getByText(t.conservarTusTurnos)).toBeInTheDocument();
    expect(screen.queryByText(/[Dd]iccionario técnico|[Tt]echnical dictionary/)).toBeNull();
    expect(screen.getAllByText(t.todaviaNo)).toHaveLength(2);
    expect(screen.getByText(t.cincoIdiomas)).toBeInTheDocument();
    // Las dos pistas con su idioma y el estado real de su modelo. **La del cliente no lo tiene**,
    // que es el estado más probable en un Mac de verdad y el que la muestra dibuja desde la
    // auditoría: uno en español no trae el modelo de inglés. Y por eso hay un botón.
    expect(screen.getByText(t.modeloInstalado)).toBeInTheDocument();
    expect(screen.getByText(t.sinModelo)).toBeInTheDocument();
    expect(screen.getByRole("button", { name: new RegExp(t.instalarModelo) })).toBeInTheDocument();
  });

  /**
   * La fase 4 abrió Corpus: pasa de fila apagada a enlace. Notas e IA siguen sin existir y el
   * rail lo dice — un rail lleno de enlaces que no llevan a ninguna parte es peor que uno corto.
   */
  it("el rail deja las secciones que aún no existen sin enlace", () => {
    const { container } = pinta("?pantalla=sesion");
    const rail = container.querySelector("nav.rail") as HTMLElement;
    const enlaces = [...rail.querySelectorAll("a")].map((a) => a.textContent);
    expect(enlaces).toEqual([t.navSesion, t.navPermisos, t.navCorpus, t.navHonestidad, t.navIdioma]);
    for (const nombre of [t.navNotas, t.navIa]) {
      const fila = within(rail).getByText(nombre).closest(".item") as HTMLElement;
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
    expect(container.querySelector('.titulo h1')?.textContent).toBe(t.corpusTitulo);
    expect(screen.getAllByText(t.todaviaNo)).toHaveLength(3);
    // Las cinco unidades más la sexta respuesta: lo que no encaja en ninguna.
    expect(container.querySelectorAll(".unidad-chip")).toHaveLength(6);
    expect(screen.getByText(t.sinUnidad)).toBeInTheDocument();
    // El tamaño del índice no puede estar escrito en la interfaz: se mide.
    expect(container.querySelector(".buffer .cuanto")?.textContent).not.toBe("0 B");
    expect(screen.getByText(t.soloTu)).toBeInTheDocument();
  });

  it("la pantalla la elige la URL, y una desconocida cae en sesión", () => {
    pinta("?pantalla=inventada");
    expect(screen.getByText(t.sesionTitulo)).toBeInTheDocument();
  });
});
