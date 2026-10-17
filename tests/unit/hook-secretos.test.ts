// Plantilla del kit (v1.37.0): corre el comando REAL del hook PreToolUse de .claude/settings.json.
// @vitest-environment node
/**
 * El hook PreToolUse de Claude Code (`.claude/settings.json`) que busca secretos en lo que se va a escribir: se corre
 * su comando real. Falla CERRADO si faltan gitleaks o jq, como el pre-commit (kit v1.37.0; origen planlang AU-S2-B12); `KIT_SIN_GITLEAKS=1` lo
 * salta a sabiendas; deja pasar un contenido limpio y bloquea la carnada canónica (carnada canónica del kit, armada
 * partida para que este archivo no la contenga).
 */
import { execFileSync, spawnSync } from "node:child_process";
import { mkdtempSync, readFileSync, symlinkSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { describe, expect, it } from "vitest";

const ajustes = JSON.parse(readFileSync(".claude/settings.json", "utf8")) as {
  hooks: { PreToolUse: { matcher: string; hooks: { command: string }[] }[] };
};
const comando = ajustes.hooks.PreToolUse.find(
  (h) => h.matcher === "Write|Edit",
)!.hooks[0]!.command;

const EN_WINDOWS = process.platform === "win32";

/**
 * El bash que corre los hooks (sprint 009): en Windows, el de Git for Windows —`<git>/usr/bin/bash.exe`, sacado de
 * `git --exec-path`—, nunca el `bash.exe` de WSL que Windows trae en System32; en el resto, `/bin/bash`.
 */
const BASH = EN_WINDOWS
  ? join(
      dirname(
        dirname(
          dirname(
            execFileSync("git", ["--exec-path"], { encoding: "utf8" }).trim(),
          ),
        ),
      ),
      "usr",
      "bin",
      "bash.exe",
    )
  : "/bin/bash";

function correr(contenido: string, env: Record<string, string | undefined>) {
  return spawnSync(BASH, ["-c", comando], {
    input: JSON.stringify({ tool_input: { content: contenido } }),
    encoding: "utf8",
    env: env as NodeJS.ProcessEnv,
  });
}

/**
 * Un PATH con lo básico del sistema y sin gitleaks ni jq. En Windows, vacío: crear enlaces simbólicos pide un permiso
 * del sistema (modo de desarrollador o administrador), y el camino que falla cerrado solo usa `echo`, que es de bash.
 */
function pathSinHerramientas(): string {
  const dir = mkdtempSync(join(tmpdir(), "hook-sin-"));
  if (EN_WINDOWS) return dir;
  for (const b of ["cat", "printf", "echo"]) {
    const r = spawnSync(BASH, ["-c", `command -v ${b}`], {
      encoding: "utf8",
    });
    if (r.stdout.trim().startsWith("/"))
      symlinkSync(r.stdout.trim(), join(dir, b));
  }
  return dir;
}

/**
 * La mitad positiva (la carnada) necesita gitleaks y jq. En local los trae el hook; en la CI, `quality` instala gitleaks
 * con versión y huella fijas (auditoría del S5, B8), así que allí NO se salta: si falta gitleaks en la CI, es rojo.
 */
const hayHerramientas =
  spawnSync(BASH, ["-c", "command -v gitleaks && command -v jq"]).status === 0;

describe("hook PreToolUse de secretos (kit v1.37.0; origen planlang AU-S2-B12)", () => {
  it("en la CI las herramientas están: la mitad positiva no se salta en silencio", () => {
    if (process.env.CI)
      expect(
        hayHerramientas,
        "gitleaks o jq faltan en la CI: la carnada no se probó",
      ).toBe(true);
  });

  it("sin gitleaks ni jq bloquea, y lo dice", () => {
    const r = correr("hola", { PATH: pathSinHerramientas() });
    expect(r.status).toBe(2);
    // Con exit 2 Claude Code le enseña al agente el stderr (auditoría del S5, B8): el porqué va ahí, no en stdout.
    expect(r.stderr).toContain("falta gitleaks o jq");
    expect(r.stdout).toBe("");
  });

  // Sprint 009: el aviso nombra el instalador de ESTE sistema (en Windows decía `brew`, que allí no existe).
  it("el aviso nombra el instalador del sistema donde corre", () => {
    const r = correr("hola", { PATH: pathSinHerramientas() });
    expect(r.stderr).toContain(EN_WINDOWS ? "winget install" : "brew install");
    expect(r.stderr).not.toContain(EN_WINDOWS ? "brew" : "winget");
  });

  it("el hook y el pre-commit traen los dos sistemas: winget y el Administrador de credenciales, brew y el Llavero", () => {
    const preCommit = readFileSync("githooks/pre-commit", "utf8");
    for (const [nombre, texto] of [
      ["hook", comando],
      ["pre-commit", preCommit],
    ] as const) {
      for (const dice of [
        "winget install --id Gitleaks.Gitleaks",
        "al Administrador de credenciales de Windows",
        "brew install gitleaks",
        "al Llavero de macOS",
      ])
        expect(texto, `${nombre}: falta «${dice}»`).toContain(dice);
    }
  });

  it("KIT_SIN_GITLEAKS=1 lo salta a sabiendas", () => {
    const r = correr("hola", {
      PATH: pathSinHerramientas(),
      KIT_SIN_GITLEAKS: "1",
    });
    expect(r.status).toBe(0);
  });

  it.runIf(hayHerramientas)(
    "con las herramientas: deja pasar lo limpio y bloquea la carnada",
    () => {
      expect(correr("const x = 1;", process.env).status).toBe(0);
      const carnada = ["AWS_ACCESS_KEY_ID=", "AKIAQ7RTZ4PX", "KM2WNB3S"].join(
        "",
      );
      const r = correr(carnada, process.env);
      expect(r.status).toBe(2);
      expect(r.stderr).toContain("SECRET DETECTADO");
      expect(r.stdout).toBe("");
    },
    // Lanza dos veces Git Bash con gitleaks y jq, de forma síncrona: con la suite entera en paralelo en una Surface pasa
    // de los 5 s por defecto de vitest (sprint 010: tardó 11 s y falló solo; suelto, pasa). Es margen, no un hook lento.
    30_000,
  );
});
