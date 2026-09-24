// Setup global de Vitest (referenciado por vitest.config.ts).
// Matchers de Testing Library (toBeInTheDocument, toHaveAccessibleName, ...).
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// K3 (sprint 001): Testing Library solo registra su limpieza automática cuando `globals` está
// activo en vitest, y aquí no lo está. Sin esto los renders se ACUMULAN entre tests: el segundo
// `getByTestId` encuentra dos nodos y falla con un mensaje que parece del componente ("found
// multiple elements") cuando el defecto es del arnés. Lo descubrió el primer test de UI del
// sprint; se arregla aquí una vez para todos los que vienen.
afterEach(cleanup);
