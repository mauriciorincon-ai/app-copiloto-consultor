import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it } from "vitest";
import { Cascara } from "@/App";
import { Banda } from "@/componentes/Banda";
import { HoyEnWindows } from "@/componentes/HoyEnWindows";
import { PLATAFORMA_MACOS, PLATAFORMA_WINDOWS } from "@/contrato.generado";
import { plataformaInicial, type Plataforma } from "@/plataforma";

/**
 * **LAS RAMAS DE WINDOWS QUE NINGUNA MUESTRA ALCANZABA** (sprint 008, auditoría del S7, M10, M11 y M12): una tecla que
 * el sistema no dejó registrar, los turnos que no dio tiempo a transcribir, la descarga en curso y el primer cuadro
 * dentro de Tauri. Las muestras del contrato son las del caso feliz; aquí se parten a propósito.
 *
 * ¿Puede fallar? Sí: quitar `atajosSinRegistrar` de la banda o de Honestidad, o el `> 0` de los descartados, es rojo.
 */
const sinRegistrar = (accion: Plataforma["atajos"][number]["accion"]): Plataforma => ({
  ...PLATAFORMA_WINDOWS,
  atajos: PLATAFORMA_WINDOWS.atajos.map((a) => (a.accion === accion ? { ...a, registrado: false } : a)),
});

function pinta(nodo: React.ReactNode, idioma: "es" | "en") {
  cleanup();
  document.documentElement.lang = idioma;
  return render(<Cascara>{nodo}</Cascara>);
}

afterEach(() => {
  cleanup();
  document.documentElement.lang = "";
});

describe("una tecla que el sistema no dejó registrar se dice", () => {
  it("Honestidad nombra la tecla, en los dos idiomas, con su símbolo y su texto", () => {
    pinta(<HoyEnWindows plataforma={sinRegistrar("corte")} />, "es");
    expect(screen.getByText(/Alt\+Mayús\+Q no responde/)).toBeTruthy();
    pinta(<HoyEnWindows plataforma={sinRegistrar("corte")} />, "en");
    expect(screen.getByText(/Alt\+Shift\+Q doesn't respond/)).toBeTruthy();
  });

  it("con todas registradas, Honestidad no dice nada de teclas", () => {
    pinta(<HoyEnWindows plataforma={PLATAFORMA_WINDOWS} />, "es");
    expect(screen.queryByText(/no responde:/)).toBeNull();
  });

  it("la banda lo dice también, contado, y en macOS o con todas registradas no aparece", () => {
    pinta(<Banda estado="esperando" plataforma={sinRegistrar("corte")} />, "es");
    expect(screen.getByText("1 tecla no responde")).toBeTruthy();
    pinta(<Banda estado="esperando" plataforma={PLATAFORMA_WINDOWS} />, "es");
    expect(screen.queryByText(/no responde/)).toBeNull();
    pinta(<Banda estado="esperando" plataforma={PLATAFORMA_MACOS} />, "es");
    expect(screen.queryByText(/no responde/)).toBeNull();
    const dos: Plataforma = {
      ...sinRegistrar("corte"),
      atajos: sinRegistrar("corte").atajos.map((a) => (a.accion === "callar" ? { ...a, registrado: false } : a)),
    };
    pinta(<Banda estado="esperando" plataforma={dos} />, "en");
    expect(screen.getByText("2 shortcuts aren't responding")).toBeTruthy();
  });
});

describe("los turnos sin transcribir y la descarga en curso se dicen", () => {
  const con = (t: Partial<NonNullable<Plataforma["transcripcion"]>>): Plataforma => ({
    ...PLATAFORMA_WINDOWS,
    transcripcion: { ...PLATAFORMA_WINDOWS.transcripcion!, ...t },
  });

  it("descartados > 0 aparece, sin atribuirlos, y en 0 no", () => {
    pinta(<HoyEnWindows plataforma={con({ descartados: 3 })} />, "es");
    const frase = screen.getByText(/3 turnos se quedaron sin transcribir/);
    expect(frase.textContent).not.toMatch(/tuy/i);
    pinta(<HoyEnWindows plataforma={con({ descartados: 0 })} />, "es");
    expect(screen.queryByText(/sin transcribir/)).toBeNull();
  });

  it("la descarga en curso se enseña y esconde los botones de medir", () => {
    pinta(<HoyEnWindows plataforma={con({ bajando: true })} />, "es");
    expect(screen.getByText(/Bajando el modelo de voz/)).toBeTruthy();
    expect(screen.queryByText("Medir otra vez")).toBeNull();
  });
});

describe("el primer cuadro dentro de Tauri ya sabe en qué sistema está", () => {
  const WEBVIEW2 = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0 Safari/537.36 Edg/130.0";
  const WKWEBVIEW = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko)";

  it("en Windows parte de Windows, con sus teclas y sin cifras de muestra", () => {
    const p = plataformaInicial(PLATAFORMA_MACOS, WEBVIEW2, true);
    expect(p.sistema).toBe("windows");
    expect(p.atajos.find((a) => a.accion === "corte")?.teclas).toBe("Alt+Shift+Q");
    expect(p.transcripcion).toBeNull();
    expect(p.funciones).toEqual([]);
  });

  it("en macOS, o fuera de Tauri, no cambia nada", () => {
    expect(plataformaInicial(PLATAFORMA_MACOS, WKWEBVIEW, true)).toBe(PLATAFORMA_MACOS);
    expect(plataformaInicial(PLATAFORMA_MACOS, WEBVIEW2, false)).toBe(PLATAFORMA_MACOS);
  });
});

describe("en Windows la protección se dice pedida, no verificada (auditoría del S7, B9; regla dura 6)", () => {
  it("las tres frases que decían «protegido» dicen lo que se pidió, en los dos idiomas", async () => {
    const { diccionarioDeWindows } = await import("@/i18n");
    for (const idioma of ["es", "en"] as const) {
      const d = diccionarioDeWindows(idioma, (t) => t);
      for (const frase of [d.cuaderno.panelProtegido, d.notas.clienteNoLaVe, d.cliente.panelProtegido]) {
        expect(frase, frase).toMatch(/pedida|requested/);
      }
    }
  });
});
