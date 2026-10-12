import { cleanup, render } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Cascara, Enrutador, ESTADOS } from "@/App";
import { diccionarioDeWindows } from "@/i18n";
import { CAMBIOS_DE_WINDOWS } from "@/i18n/windows-cambios";
import { ESTADOS as ESTADOS_DE_SESION } from "@/jurisdiccion";
import { ESTADOS as ESTADOS_DEL_ENSAYO } from "@/ensayo";
import { LaEscucha, LaPantalla, LaPista, LosAuriculares } from "@/pantallas/Sesion";
import type { EstadoDePista, PorQueNoAbrio, PorQueNoSeSabe, VistaDeLaPantalla } from "@/cuaderno";
import { conTeclasDeEsteSistema, PLATAFORMA_FUERA } from "@/plataforma";
import { PLATAFORMA_WINDOWS } from "@/contrato.generado";

/**
 * **EN WINDOWS NINGUNA PANTALLA HABLA DEL MAC NI PINTA SUS TECLAS** (sprint 007, ADR 022 enmienda 1).
 *
 * `es.ts` y `en.ts` están atados a la maqueta y dicen `⌃⌥A`, «tu Mac», «Verificada solo en Meet · macOS 26.6.2». La
 * invisibilidad solo se promete donde la corrida del usuario la verificó (regla dura 6): en un PC con Windows esa frase
 * sería una promesa que nadie hizo. Este test pinta las pantallas que la videollamada alcanza, en Windows, y busca lo que
 * no puede estar: las teclas de macOS, el nombre del Mac o de macOS. Con su control: las mismas pantallas en macOS SÍ las
 * pintan, así que el test no pasa de vacío.
 *
 * ¿Puede fallar? Sí: con un texto del diccionario que diga «tu Mac» sin su cambio en `windows-cambios.ts`, o un
 * `<kbd>⌃⌥A</kbd>` en vez de `<kbd>{k("⌃⌥A")}</kbd>`, es rojo (bitácora del sprint 007).
 */
function alto(px: number) {
  Object.defineProperty(globalThis, "innerHeight", { value: px, configurable: true, writable: true });
}

/** Lo que dice del Mac, con las palabras de al lado para saber de qué frase sale. */
function delMac(texto: string): string[] {
  return [...texto.matchAll(/.{0,40}(⌃|⌥|⎋|\bmacOS\b|\bMac\b|Llavero|Keychain|keychain|llavero|Finder|Touch ID|Time Machine|Spotlight|Accesibilidad|Accessibility|\btap\b|\bApple\b).{0,40}/g)].map((m) => m[0].trim());
}

/** Todo lo que una persona lee en la pantalla: el texto y lo que leen los lectores de pantalla. */
function loQueSeLee(raiz: HTMLElement): string {
  const atributos = [...raiz.querySelectorAll("[aria-label],[title],[alt],[placeholder]")].flatMap((e) =>
    ["aria-label", "title", "alt", "placeholder"].map((a) => e.getAttribute(a) ?? ""),
  );
  // `textContent` pega el texto de bloques vecinos («macOSTú los concedes») y `\b` deja de cortar la palabra: se lee
  // nodo por nodo, con un espacio entre cada uno, que es como lo lee una persona.
  const texto: string[] = [];
  const nodos = document.createTreeWalker(raiz, NodeFilter.SHOW_TEXT);
  for (let n = nodos.nextNode(); n; n = nodos.nextNode()) {
    const t = (n.textContent ?? "").trim();
    if (t) texto.push(t);
  }
  return [texto.join(" "), ...atributos].join(" · ");
}

function pinta(busqueda: string, windows: boolean, idioma: "es" | "en") {
  cleanup();
  window.history.replaceState({}, "", windows ? "/?plataforma=windows" : "/");
  document.documentElement.lang = idioma;
  return render(
    <Cascara>
      <Enrutador busqueda={busqueda} />
    </Cascara>,
  );
}

afterEach(() => {
  window.history.replaceState({}, "", "/");
  document.documentElement.lang = "";
});


/**
 * Lo que sigue hablando de Apple en Windows, declarado con su motivo (auditoría del S7, A2): nada se calla en silencio.
 * Desde el sprint 008 (Windows completo) la lista está vacía: la pantalla IA dice lo que es la sugerencia en Windows y
 * la banda pinta el radar también allí, con sus dos estados.
 */
const DECLARADO: { pantalla: string; frase: RegExp; hasta: string }[] = [];

/** Todos los estados de la banda, el radar incluido: desde el sprint 008 mira también en Windows. */
const SIN_RADAR = ESTADOS;

describe("en Windows ninguna pantalla habla del Mac ni pinta sus teclas", () => {
  for (const idioma of ["es", "en"] as const) {
    it(`la banda, en sus ${SIN_RADAR.length} estados, compacta y ampliada · ${idioma}`, () => {
      const culpables: string[] = [];
      for (const ampliada of [false, true]) {
        alto(ampliada ? 200 : 88);
        for (const estado of SIN_RADAR) {
          const { container } = pinta(`?ventana=banda&estado=${estado}&plataforma=windows`, true, idioma);
          for (const frase of new Set(delMac(loQueSeLee(container)))) {
            culpables.push(`${estado}${ampliada ? " (ampliada)" : ""}: ${frase}`);
          }
        }
      }
      expect(culpables, "la banda dice del Mac en Windows").toEqual([]);
    });

    it(`las pantallas de la app · ${idioma}`, () => {
      const culpables: string[] = [];
      // Sesión en todos sus estados (`sesion.html`): antes de empezar, en marcha, presencial, solo notas, la NDA…
      const pedidas = [
        ...ESTADOS_DE_SESION.map((e) => `pantalla=sesion&estado=${e}`),
        ...ESTADOS_DEL_ENSAYO.map((e) => `pantalla=ensayo&estado=${e}`),
        ...["notas", "ia", "idioma", "permisos", "honestidad", "corpus"].map((p) => `pantalla=${p}`),
      ];
      for (const pedida of pedidas) {
        const { container } = pinta(`?${pedida}&plataforma=windows`, true, idioma);
        for (const frase of new Set(delMac(loQueSeLee(container)))) {
          if (!DECLARADO.some((d) => pedida.startsWith(d.pantalla) && d.frase.test(frase))) {
            culpables.push(`${pedida}: ${frase}`);
          }
        }
      }
      expect(culpables, "una pantalla dice del Mac en Windows").toEqual([]);
    });
  }

  // Auditoría del S7, A1: con el audio del sistema y la pantalla en Windows, las rutas de error SE ALCANZAN allí (un PC
  // sin salida de audio, un dispositivo en modo exclusivo, una pantalla que no se entrega) y no pueden mandar a
  // «reiniciar el Mac» ni a conceder un permiso de macOS.
  for (const idioma of ["es", "en"] as const) {
    it(`las rutas de error de pista, pantalla y salida · ${idioma}`, () => {
      const MOTIVOS: PorQueNoAbrio[] = [
        "sin-permiso-del-microfono",
        "sin-permiso-del-audio",
        "dispositivo-ocupado",
        "formato-ilegible",
        "no-dejo",
      ];
      const VISTAS: VistaDeLaPantalla[] = ["apagada", "sin-permiso", "esperando-la-reunion", "leyendo", "no-pudo"];
      const SIN_SABER: PorQueNoSeSabe[] = ["sin-salida", "sin-conexion", "sin-fuente"];
      const caida = (motivo: PorQueNoAbrio): EstadoDePista => ({ abierta: false, motivo, bytes: 0 });
      const casos: [string, React.ReactElement][] = [
        ...MOTIVOS.flatMap((m): [string, React.ReactElement][] => [
          [`pista caída · ${m}`, <LaPista pista={caida(m)} caida icono="i-sistema" texto="Audio del sistema" />],
        ]),
        ...[
          [true, true],
          [true, false],
          [false, true],
          [false, false],
        ].map(([mic, sis]): [string, React.ReactElement] => [
          `escucha · mic ${mic} · sistema ${sis}`,
          <LaEscucha mic={mic} sistema={sis} />,
        ]),
        ...VISTAS.map((vista): [string, React.ReactElement] => [
          `pantalla · ${vista}`,
          <LaPantalla pantalla={{ vista, bytesEnMemoria: 0 }} />,
        ]),
        ...SIN_SABER.map((motivo): [string, React.ReactElement] => [
          `salida · no se sabe · ${motivo}`,
          <LosAuriculares salida={{ salida: "no-se-sabe", motivo, nombre: null }} />,
        ]),
      ];
      const culpables: string[] = [];
      for (const [que, ui] of casos) {
        cleanup();
        window.history.replaceState({}, "", "/?plataforma=windows");
        document.documentElement.lang = idioma;
        const { container } = render(<Cascara>{ui}</Cascara>);
        for (const frase of new Set(delMac(loQueSeLee(container)))) culpables.push(`${que}: ${frase}`);
      }
      expect(culpables, "una ruta de error dice del Mac en Windows").toEqual([]);
    });
  }

  it("CONTROL: en macOS las mismas pantallas SÍ pintan las teclas del Mac (el test no pasa de vacío)", () => {
    alto(200);
    const banda = pinta("?ventana=banda&estado=ficha", false, "es");
    expect(loQueSeLee(banda.container)).toMatch(/⌃⌥/);
    const sesion = pinta("?pantalla=sesion", false, "es");
    expect(loQueSeLee(sesion.container)).toMatch(/\bMac\b/);
  });

  it("en Windows la banda pinta las teclas de Windows, en el idioma de quien lee", () => {
    alto(200);
    const es = pinta("?ventana=banda&estado=ficha&plataforma=windows", true, "es");
    expect(loQueSeLee(es.container)).toMatch(/Alt\+Mayús\+[A-Z]/);
    const en = pinta("?ventana=banda&estado=ficha&plataforma=windows", true, "en");
    expect(loQueSeLee(en.container)).toMatch(/Alt\+Shift\+[A-Z]/);
  });

  it("los cambios de Windows tienen las mismas claves en español y en inglés, sin mirar el diccionario fusionado (M8)", () => {
    const claves = (o: unknown, ruta = ""): string[] =>
      typeof o === "object" && o !== null
        ? Object.entries(o).flatMap(([k, v]) => claves(v, `${ruta}.${k}`))
        : [ruta];
    expect(claves(CAMBIOS_DE_WINDOWS.en).sort()).toEqual(claves(CAMBIOS_DE_WINDOWS.es).sort());
  });

  it("el diccionario de Windows tiene los dos idiomas con las mismas claves (paridad, regla 20)", () => {
    const igual = (t: string) => conTeclasDeEsteSistema(t, PLATAFORMA_WINDOWS, "es");
    const claves = (o: unknown, ruta = ""): string[] =>
      typeof o === "object" && o !== null
        ? Object.entries(o).flatMap(([k, v]) => claves(v, `${ruta}.${k}`))
        : [ruta];
    expect(claves(diccionarioDeWindows("en", igual))).toEqual(claves(diccionarioDeWindows("es", igual)));
    expect(PLATAFORMA_FUERA.sistema).toBe("macos");
  });
});
