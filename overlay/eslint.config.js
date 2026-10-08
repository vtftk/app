// @ts-check

import globals from "globals";
import eslint from "@eslint/js";
import tseslint from "typescript-eslint";
import perfectionist from "eslint-plugin-perfectionist";
import unusedImports from "eslint-plugin-unused-imports";

export default tseslint.config(
  eslint.configs.recommended,
  tseslint.configs.recommended,
  {
    plugins: {
      perfectionist,
      "unused-imports": unusedImports,
    },
    languageOptions: {
      globals: {
        ...globals.browser,
      },
    },
    rules: {
      "no-unused-vars": "off",
      "no-undef": "off",
      "unused-imports/no-unused-imports": "warn",
      "unused-imports/no-unused-vars": [
        "off",
        {
          vars: "all",
          varsIgnorePattern: "^_",
          args: "after-used",
          argsIgnorePattern: "^_",
        },
      ],
      "@typescript-eslint/no-unused-vars": [
        "warn",
        {
          args: "none",
          argsIgnorePattern: "^_",
          varsIgnorePattern: "^_",
          caughtErrorsIgnorePattern: "^_",
        },
      ],

      "perfectionist/sort-named-imports": [
        "warn",
        {
          order: "asc",
          type: "line-length",
        },
      ],
      "perfectionist/sort-named-exports": [
        "warn",
        {
          order: "asc",
          type: "line-length",
        },
      ],
      "perfectionist/sort-exports": [
        "warn",
        {
          order: "asc",
          type: "line-length",
        },
      ],
      "perfectionist/sort-imports": [
        "warn",
        {
          order: "asc",
          type: "line-length",
          newlinesBetween: 1,
          internalPattern: ["^~/.*"],
        },
      ],
    },
  },
);
