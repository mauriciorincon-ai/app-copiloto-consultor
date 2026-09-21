import { render, screen, within } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { Cascara } from "@/App";
import { Principal } from "@/componentes/Principal";
import { SpriteIconos } from "@/componentes/Iconos";
import { es } from "@/i18n/es";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

/**
 * LAS TRES PANTALLAS DEL CUADERNO (fase 2).
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
  it("sesión marca las cuatro pistas como «todavía no», ninguna en verde", () => {
    pinta("?pantalla=sesion");
    const pistas = screen.getByText(t.dosPistas).closest(".tarjeta") as HTMLElement;
    expect(within(pistas).getAllByText(t.todaviaNo)).toHaveLength(4);
    expect(within(pistas).queryByText(t.funciona)).toBeNull();
  });

  /**
   * La tarjeta que impide que la pantalla mienta por omisión: el sprint 001 SÍ entrega la banda
   * protegida, el acople y el kill-switch, y una pantalla llena de «todavía no» se leería como
   * que no hay nada.
   */
  it("sesión enseña lo que sí funciona hoy, y son tres cosas", () => {
    pinta("?pantalla=sesion");
    const hoy = screen.getByText(t.queFuncionaHoy).closest(".tarjeta") as HTMLElement;
    expect(within(hoy).getAllByText(t.funciona)).toHaveLength(2);
    expect(within(hoy).getByText("⌥⎋")).toBeInTheDocument();
    expect(within(hoy).getAllByText(t.todaviaNo)).toHaveLength(1);
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

  it("permisos: lo que la app aún no puede hacer sin permisos lleva «todavía no»", () => {
    pinta("?pantalla=permisos");
    const tarjeta = screen.getByText(t.sinConcederNada).closest(".tarjeta") as HTMLElement;
    expect(within(tarjeta).getAllByText(t.todaviaNo)).toHaveLength(3);
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

  it("honestidad: los cuatro búferes están en cero y marcados como inexistentes", () => {
    const { container } = pinta("?pantalla=honestidad");
    const bufs = [...container.querySelectorAll(".buffer")];
    expect(bufs).toHaveLength(4);
    for (const b of bufs) {
      expect(b.className).toContain("pendiente");
      expect(b.querySelector(".cuanto")?.textContent).toBe("0 B");
    }
  });

  /** «7 de 7» con cuatro piezas sin construir sería la mentira cómoda. */
  it("honestidad: el kill-switch dice cuántas piezas corta de cuántas hay", () => {
    pinta("?pantalla=honestidad");
    expect(screen.getByText(/3 de 7/)).toBeInTheDocument();
  });

  it("el rail deja las secciones que aún no existen sin enlace", () => {
    const { container } = pinta("?pantalla=sesion");
    const rail = container.querySelector("nav.rail") as HTMLElement;
    const enlaces = [...rail.querySelectorAll("a")].map((a) => a.textContent);
    expect(enlaces).toEqual([t.navSesion, t.navPermisos, t.navHonestidad]);
    for (const nombre of [t.navCorpus, t.navNotas, t.navIdioma, t.navIa]) {
      const fila = within(rail).getByText(nombre).closest(".item") as HTMLElement;
      expect(fila.className).toContain("pendiente");
      expect(fila.tagName).not.toBe("A");
    }
  });

  it("la pantalla la elige la URL, y una desconocida cae en sesión", () => {
    pinta("?pantalla=inventada");
    expect(screen.getByText(t.sesionTitulo)).toBeInTheDocument();
  });
});
