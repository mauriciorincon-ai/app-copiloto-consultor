import path from "node:path";
import { defineConfig } from "vitest/config";

// Config que el ci.yml del kit ya asume (job quality: "pnpm test").
// Patrón validado en app-nutri-kids S1 y app-ds S1. Cada app ajusta `coverage.include` a sus
// motores puros y puede subir los umbrales.
// ⚠ K7 (ds S1): los umbrales SOLO se aplican si el script `test` pasa `--coverage`. El estampado
// lo omite a propósito (CI verde día 0 sin tests); al escribir los PRIMEROS tests del S1, añade
// `--coverage` al script `test` de package.json — es parte del setup del sprint, no una sorpresa.
export default defineConfig({
  resolve: {
    alias: { "@": path.resolve(__dirname, "src") },
  },
  test: {
    environment: "jsdom",
    setupFiles: ["./tests/setup.ts"],
    include: ["tests/unit/**/*.test.{ts,tsx}"],
    coverage: {
      provider: "v8",
      // K2 (sprint 001): el kit apunta a `src/lib/**` y `src/engine/**` porque asume que los
      // motores puros viven en TypeScript. En Angel Ghost NO: VAD, fin de turno, BM25 y el
      // disparador son Rust, y los cubre `cargo test` en el job build-escritorio. Lo que vive
      // en TS es la capa visual y el diccionario. Dejar los globs del kit habría dado un
      // umbral que se cumple solo porque no mide nada — el peor tipo de verde.
      include: ["src/**/*.{ts,tsx}"],
      exclude: ["src/main.tsx", "src/vite-env.d.ts"],
      thresholds: {
        // Regla 2 del CLAUDE.md para la capa de UI.
        lines: 50,
        functions: 50,
        branches: 50,
        statements: 50,
        // Si algún día aparece lógica pura en TS, se le exige lo de los motores.
        "src/lib/**/*.ts": {
          lines: 80,
          functions: 80,
          branches: 80,
          statements: 80,
        },
      },
    },
  },
});
