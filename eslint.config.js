import js from "@eslint/js";
import tseslint from "typescript-eslint";
import jsxA11y from "eslint-plugin-jsx-a11y";
export default tseslint.config(
  { ignores: ["dist/", "src-tauri/", "coverage/", "playwright-report/", "test-results/"] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  { files: ["src/**/*.{ts,tsx}"], plugins: { "jsx-a11y": jsxA11y }, rules: { ...jsxA11y.configs.recommended.rules } },
);
