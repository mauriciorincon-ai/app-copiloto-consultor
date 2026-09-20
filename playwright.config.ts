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
  projects: [
    {
      name: "mobile-chromium",
      use: { ...devices["Pixel 7"] },
    },
    {
      name: "desktop-chromium",
      use: { ...devices["Desktop Chrome"] },
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
