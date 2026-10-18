import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

/**
 * **VITEST CORRE TAMBIÉN EN LOS DOS JOBS DE WINDOWS** (sprint 010).
 *
 * Hasta el sprint 009 vitest solo corría en Linux (`quality`) y en la Surface de quien construía: el sprint 009 encontró
 * ocho fallos que solo existían en Windows (rutas con `\`, el hook de secretos con Git Bash, el diccionario por `USERPROFILE`)
 * y nadie los habría visto en la CI. Un `path.sep` que vuelva a colarse tiene que ser rojo en la PR, no un hallazgo de la
 * siguiente vez que alguien abra la Surface. Este gate exige el paso `pnpm test` en `build-windows` (x64) y en
 * `build-windows-arm`, después de instalar y antes de construir.
 *
 * ¿Puede fallar? Sí: quitar el paso de uno de los dos jobs, o ponerlo detrás de `pnpm build`, es rojo (demo del sprint 010).
 */
const CI = readFileSync(".github/workflows/ci.yml", "utf8");

/** El texto de un job: desde su `  nombre:` hasta el siguiente job de dos espacios. */
function job(nombre: string): string {
  const i = CI.indexOf(`\n  ${nombre}:\n`);
  expect(i, `el job ${nombre} no existe en ci.yml`).toBeGreaterThan(-1);
  const resto = CI.slice(i + 1);
  const fin = resto.slice(1).search(/\n {2}[a-z][a-z-]*:\n/);
  return fin < 0 ? resto : resto.slice(0, fin + 1);
}

describe("vitest corre en Windows, x64 y ARM64", () => {
  it.each(["build-windows", "build-windows-arm"])(
    "%s: `pnpm test` va después de instalar y antes de construir",
    (nombre) => {
      const texto = job(nombre);
      const instalar = texto.indexOf("pnpm install --frozen-lockfile");
      // Anclada al inicio de la línea: un `# run: pnpm test` comentado no cuenta como el paso (auditoría del S10, B10).
      const probar = texto.search(/^\s+run: pnpm test\s*$/m);
      const construir = texto.indexOf("run: pnpm build");
      expect(instalar, `${nombre}: falta pnpm install`).toBeGreaterThan(-1);
      expect(
        probar,
        `${nombre}: falta el paso de vitest (\`run: pnpm test\`)`,
      ).toBeGreaterThan(-1);
      expect(construir, `${nombre}: falta pnpm build`).toBeGreaterThan(-1);
      expect(
        instalar < probar && probar < construir,
        `${nombre}: vitest va entre instalar y construir`,
      ).toBe(true);
    },
  );

  it("CONTROL: `quality`, en Linux, ya lo corría (el lector de jobs no pasa de vacío)", () => {
    expect(job("quality")).toMatch(/run: pnpm test\b/);
  });

  it("el paso dice por qué existe", () => {
    expect(CI).toMatch(/vitest TAMBIÉN en Windows \(sprint 010\)/);
  });
});
