import js from "@eslint/js";
import tseslint from "typescript-eslint";
import jsxA11y from "eslint-plugin-jsx-a11y";
export default tseslint.config(
  { ignores: ["dist/", "src-tauri/", "coverage/", "playwright-report/", "test-results/"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    plugins: { "jsx-a11y": jsxA11y },
    rules: {
      ...jsxA11y.configs.recommended.rules,
      // Una región que SE DESPLAZA tiene que poder recibir el foco, o lo que queda por debajo
      // del borde no se alcanza con el teclado — es la regla `scrollable-region-focusable` de
      // axe, y la cazó en vivo el primer e2e de accesibilidad de esta app. La regla de eslint,
      // con sus valores por defecto, solo admite `tabIndex` en un `<section>` o un `tabpanel`;
      // aquí se le añade `region`, que es exactamente el caso que WCAG contempla. No se afloja
      // nada más: sigue prohibido poner `tabIndex` en cualquier otra cosa no interactiva.
      "jsx-a11y/no-noninteractive-tabindex": ["error", { tags: [], roles: ["region", "tabpanel"] }],
    },
  },
);
