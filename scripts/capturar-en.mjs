/**
 * CAPTURAS EN INGLÉS de lo que los sprints 5 a 8 pintan (sprint 008, cierre del H-1; regla 20: un idioma por sprint, el
 * otro al cierre del hito). Sirve el build, abre cada estado con el navegador en `en-US`, en los dos temas, y deja un PNG
 * por estado en `docs/fidelidad/s8-en/`. **Se leen como imagen, una a una**: lo que un test no ve —un texto que se parte en
 * dos líneas, un chip que se come el de al lado, una tecla del Mac en una pantalla de Windows— se ve mirando.
 *
 * No compara contra ninguna maqueta (la de Windows no existe: «maquetado, no visto»); el gate de fidelidad compara la
 * banda y el cuaderno del S5 y sus capturas en inglés ya están en `docs/fidelidad/s5-*`. Aquí van las que no tenían.
 *
 * Uso: pnpm build && node scripts/capturar-en.mjs     (puerto 4185, jamás el 3000)
 */
import { chromium } from "@playwright/test";
import { spawn } from "node:child_process";
import { mkdirSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";

const SALIDA = resolve("docs/fidelidad/s8-en");
const PUERTO = 4185;

const ESTADOS = [
  // Presencial (sprint 005, y lo que el 008 le sumó)
  { id: "sesion-presencial", url: "ventana=principal&pantalla=sesion&estado=presencial" },
  { id: "sesion-presencial-en-marcha", url: "ventana=principal&pantalla=sesion&estado=presencial-marcha" },
  { id: "notas-al-cerrar-presencial", url: "ventana=principal&pantalla=notas&estado=al-cerrar-presencial" },
  { id: "banda-presencial", url: "ventana=banda&estado=ficha-presencial", ancho: 1180, alto: 200 },
  // Windows (sprints 006 a 008)
  { id: "honestidad-windows", url: "ventana=principal&pantalla=honestidad&plataforma=windows" },
  { id: "permisos-windows", url: "ventana=principal&pantalla=permisos&plataforma=windows" },
  { id: "sesion-windows", url: "ventana=principal&pantalla=sesion&plataforma=windows" },
  { id: "ia-windows", url: "ventana=principal&pantalla=ia&plataforma=windows" },
  { id: "idioma-windows", url: "ventana=principal&pantalla=idioma&plataforma=windows" },
  { id: "banda-windows-sin-verificar", url: "ventana=banda&estado=sin-verificar&plataforma=windows", ancho: 1180, alto: 200 },
  { id: "banda-windows-radar", url: "ventana=banda&estado=radar-invasivo&plataforma=windows", ancho: 1180, alto: 200 },
];

const servidor = spawn("pnpm", ["preview", "--port", String(PUERTO), "--strictPort"], { stdio: "ignore" });
const esperar = async () => {
  for (let i = 0; i < 60; i++) {
    try {
      if ((await fetch(`http://localhost:${PUERTO}/`)).ok) return;
    } catch {}
    await new Promise((r) => setTimeout(r, 500));
  }
  throw new Error("el servidor del build no arrancó");
};

try {
  await esperar();
  rmSync(SALIDA, { recursive: true, force: true });
  mkdirSync(SALIDA, { recursive: true });
  const navegador = await chromium.launch();
  for (const tema of ["dark", "light"]) {
    const contexto = await navegador.newContext({ locale: "en-US", colorScheme: tema, reducedMotion: "reduce" });
    for (const e of ESTADOS) {
      const pagina = await contexto.newPage();
      await pagina.setViewportSize({ width: e.ancho ?? 960, height: e.alto ?? 640 });
      await pagina.goto(`http://localhost:${PUERTO}/?${e.url}`);
      await pagina.evaluate(() => document.fonts.ready);
      const desborde = await pagina.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth);
      await pagina.screenshot({ path: join(SALIDA, `${e.id}--${tema}--en.png`) });
      if (desborde > 0) console.log(`DESBORDE de ${desborde} px en ${e.id} (${tema})`);
      await pagina.close();
    }
    await contexto.close();
  }
  await navegador.close();
  console.log(`${ESTADOS.length * 2} capturas en ${SALIDA}`);
} finally {
  servidor.kill();
}
