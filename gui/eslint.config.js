import js from "@eslint/js";
import globals from "globals";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import tseslint from "typescript-eslint";
import { globalIgnores } from "eslint/config";

export default tseslint.config(
  globalIgnores([
    "dist",
    "node_modules",
    "src-tauri",
    "src/components/ui/**/*", // Ignore all files in components/ui/
  ]),
  {
    files: ["**/*.{ts,tsx}"],
    extends: [
      js.configs.recommended,
      tseslint.configs.recommended,
      reactHooks.configs["recommended-latest"],
      reactRefresh.configs.vite,
    ],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    rules: {
      // TODO: back to "error" once plugin manifests/payloads have typed schemas
      "@typescript-eslint/no-explicit-any": "warn",
      // Providers export their context hook alongside the component
      "react-refresh/only-export-components": [
        "error",
        { allowExportNames: ["usePlugins", "useSidebar", "useTheme"] },
      ],
    },
  }
);