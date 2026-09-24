import { defineConfig, devices } from "@playwright/test";

// Config que el ci.yml del kit ya asume (job e2e: "pnpm test:e2e").
// Patrón validado en app-nutri-kids S1. Móvil primero: las apps del pipeline son mobile-first.
export default defineConfig({
  testDir: "tests/e2e",
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  // En CI, los dos reporters: "github" anota los fallos en el PR (y SÍ imprime `N flaky`,
  // verificado); "list" añade lo que github NO da — el avance prueba por prueba, con su
  // duración. Sin él, el log salta de "Running N tests" al resumen y no se ve cuál se quedó
  // colgada: es lo que vuelve legible un timeout (kit v1.15.1, demo en rojo de Velo S3, que
  // de paso desmintió la razón original del cambio — ver CHANGELOG).
  reporter: process.env.CI ? [["github"], ["list"]] : "list",
  use: {
    baseURL: "http://localhost:3000",
    trace: "on-first-retry",
  },
  // K1 (sprint 001): el kit trae un proyecto MÓVIL porque las apps del pipeline son
  // mobile-first. Angel Ghost NO lo es: es una app de ESCRITORIO para macOS y la orden de
  // diseño lo dice ("sin viewport móvil"). Dejar el proyecto Pixel 7 no era neutro — duplicaba
  // cada prueba contra un viewport que el producto no tiene, y un fallo ahí habría costado
  // tiempo de depuración sobre algo que no existe. Se sustituye por los DOS tamaños reales de
  // ventana del design system §3.6.
  projects: [
    {
      // La ventana principal: 960 × 640 (--principal-w / --principal-h).
      name: "ventana-principal",
      use: { ...devices["Desktop Chrome"], viewport: { width: 960, height: 640 } },
    },
    {
      // La banda acoplada a lo ancho de la pantalla, 88 px de alto (§3.6).
      // El ancho de la banda es el de la pantalla; 1180 es el escritorio de referencia de
      // `docs/diseno/posicion.html`.
      name: "banda",
      use: { ...devices["Desktop Chrome"], viewport: { width: 1180, height: 200 } },
    },
  ],
  webServer: {
    // SIEMPRE contra el BUILD, nunca contra el dev server (kit v1.12.0 — lección Velo S1).
    // El dev server mete en la página cosas que NO existen en producción: websocket de HMR y
    // `eval()` de React dev. En una app con CSP estricta o gate de red eso produce rojos sobre
    // un árbol limpio — 5 en Velo S1 — y un suite que grita cuando no pasa nada acaba ignorado.
    // Cuesta el tiempo del build; compra que el e2e local afirme lo mismo que el de CI.
    command: "pnpm build && pnpm preview --port 3000 --strictPort",
    url: "http://localhost:3000",
    // Sin reuso: un `pnpm dev` olvidado en :3000 secuestraría el suite entero en silencio.
    reuseExistingServer: false,
    timeout: 180_000,
  },
});
