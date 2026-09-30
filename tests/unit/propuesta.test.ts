import { describe, expect, it } from "vitest";
import { es } from "@/i18n/es";
import { en } from "@/i18n/en";
import { etiquetaDe, iconoDe, origenDe, textoDe } from "@/propuesta";
import type { De, Propuesta, Regla } from "@/notas";

/**
 * **CÓMO SE LEE UNA PROPUESTA** (`src/propuesta.ts`, ADR 016 §2) — la mitad de TypeScript de la regla
 * dura 1 en las propuestas: del cliente, **un hecho en una línea con su plantilla, jamás su turno**.
 * Rust ya decidió qué fragmento se guarda; aquí se comprueba que la pantalla solo lo **viste**: cada
 * regla × tuyo/cliente × es/en, y que del cliente el texto es la plantilla más el fragmento y nada más.
 *
 * Auditoría del S3, M13: este archivo tenía un 5,6 % de líneas cubiertas. ¿Puede fallar? Sí: con la
 * plantilla de `nombre` pegando la frase entera del turno, «solo añade la plantilla» es rojo (bitácora).
 */
const REGLAS: Regla[] = ["cifra", "compromiso", "choque", "nombre", "pregunta"];
const LADOS: De[] = ["tuyo", "cliente"];
const IDIOMAS = [
  ["es", es.notas],
  ["en", en.notas],
] as const;

function propuesta(regla: Regla, de: De): Propuesta {
  return {
    regla,
    de,
    texto: regla === "pregunta" ? "limpieza · alcance" : "cuatro fuentes",
    ficha: regla === "choque" ? "tres" : null,
    seccion: regla === "choque" ? "§3.2 Alcance" : null,
    hora: "14:16",
  };
}

describe("propuesta: la pantalla viste lo que Rust decidió guardar", () => {
  it.each(IDIOMAS)("%s: tu frase se lee tal cual, en las cinco reglas", (_i, t) => {
    for (const regla of REGLAS) expect(textoDe(propuesta(regla, "tuyo"), t)).toBe(propuesta(regla, "tuyo").texto);
  });

  it.each(IDIOMAS)("%s: del cliente, la plantilla y el fragmento, y nada más", (_i, t) => {
    const p = (r: Regla) => propuesta(r, "cliente");
    expect(textoDe(p("cifra"), t)).toBe(`${t.dijeron}cuatro fuentes${t.cierraComilla}`);
    expect(textoDe(p("compromiso"), t)).toBe(`${t.dijeron}cuatro fuentes${t.cierraComilla}`);
    expect(textoDe(p("choque"), t)).toBe(`${t.dijeron}cuatro fuentes${t.tuFichaDice}tres${t.cierraConPunto}`);
    expect(textoDe({ ...p("choque"), ficha: null }, t)).toBe(`${t.dijeron}cuatro fuentes${t.cierraComilla}`);
    expect(textoDe(p("nombre"), t)).toBe(`${t.mencionaron}cuatro fuentes${t.noEstaEnTuCorpus}`);
    expect(textoDe(p("pregunta"), t)).toBe(`${t.tePreguntaron} limpieza · alcance`);
    // Lo único que viene del cliente es su fragmento: quitado, lo que queda son las plantillas.
    for (const regla of REGLAS) {
      const leido = textoDe(p(regla), t).replace(p(regla).texto, "").replace("tres", "");
      const plantillas = [t.dijeron, t.cierraComilla, t.tuFichaDice, t.cierraConPunto, t.mencionaron, t.noEstaEnTuCorpus, t.tePreguntaron, " "];
      let resto = leido;
      // De la más larga a la más corta: «»; tu ficha fijada dice «» contiene a «»».
      for (const pl of [...plantillas].sort((a, b) => b.length - a.length)) resto = resto.split(pl).join("");
      expect(resto, `${regla}: la pantalla añadió algo que no es plantilla`).toBe("");
    }
  });

  it.each(IDIOMAS)("%s: el origen dice de quién y a qué hora; el choque, con qué choca", (_i, t) => {
    for (const regla of REGLAS.filter((r) => r !== "choque")) {
      expect(origenDe(propuesta(regla, "tuyo"), t)).toBe(`${t.loDijisteTu} 14:16`);
      expect(origenDe(propuesta(regla, "cliente"), t)).toBe(`${t.loDijoElCliente} 14:16`);
    }
    for (const de of LADOS) expect(origenDe(propuesta("choque", de), t)).toBe(t.chocaConUnaFicha);
  });

  it.each(IDIOMAS)("%s: la etiqueta dice qué regla saltó; el choque, su sección", (_i, t) => {
    const esperadas: Record<Regla, string | null> = {
      cifra: t.reglaCifra,
      compromiso: t.reglaCompromiso,
      nombre: t.reglaNombre,
      pregunta: t.reglaPregunta,
      choque: "§3.2 Alcance",
    };
    for (const regla of REGLAS) for (const de of LADOS) expect(etiquetaDe(propuesta(regla, de), t)).toBe(esperadas[regla]);
  });

  it("cada regla tiene su icono, y ninguno se repite", () => {
    const iconos = REGLAS.map((r) => iconoDe(propuesta(r, "cliente")));
    expect(iconos.every((i) => i.startsWith("i-"))).toBe(true);
    expect(new Set(iconos).size).toBe(REGLAS.length);
  });
});
