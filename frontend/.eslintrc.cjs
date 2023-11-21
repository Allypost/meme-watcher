// eslint-disable-next-line @typescript-eslint/no-var-requires
const path = require("path");

/** @type {import("eslint").Linter.Config} */
const config = {
  overrides: [
    {
      extends: [
        "plugin:@typescript-eslint/recommended-requiring-type-checking",
      ],
      files: ["*.ts", "*.tsx"],
      parserOptions: {
        project: path.join(__dirname, "tsconfig.json"),
      },
    },
  ],
  parser: "@typescript-eslint/parser",
  parserOptions: {
    project: path.join(__dirname, "tsconfig.json"),
  },
  plugins: ["prettier", "simple-import-sort", "import", "@typescript-eslint"],
  extends: [
    "next/core-web-vitals",
    "plugin:react/recommended",
    "plugin:react/jsx-runtime",
    "plugin:@typescript-eslint/recommended-type-checked",
    "plugin:@typescript-eslint/stylistic-type-checked",
    "plugin:@typescript-eslint/strict",
  ],
  rules: {
    "@next/next/no-img-element": "off",

    "@typescript-eslint/consistent-type-imports": [
      "warn",
      {
        prefer: "type-imports",
        fixStyle: "inline-type-imports",
      },
    ],
    "require-await": "off",
    "@typescript-eslint/require-await": "warn",
    "no-unused-vars": "off",
    "@typescript-eslint/no-unused-vars": [
      "error",
      {
        args: "after-used",
        argsIgnorePattern: "^_",
        varsIgnorePattern: "^_",
        caughtErrors: "none",
      },
    ],
    "@typescript-eslint/member-delimiter-style": ["error"],
    "@typescript-eslint/no-non-null-assertion": "off",
    "@typescript-eslint/consistent-type-definitions": ["error", "type"],
    "no-redeclare": "off",
    "@typescript-eslint/no-redeclare": ["error"],
    "@typescript-eslint/no-var-requires": "off",

    "react/prop-types": "off",
    "react/void-dom-elements-no-children": ["error"],
    "react/jsx-curly-brace-presence": ["error"],
    "react/jsx-first-prop-new-line": ["error", "multiline"],
    "react/style-prop-object": ["error"],
    "react/sort-comp": ["warn"],
    "react/self-closing-comp": "error",
    "react/hook-use-state": "error",
    "react/function-component-definition": [
      "error",
      {
        namedComponents: ["arrow-function", "function-declaration"],
        unnamedComponents: ["arrow-function"],
      },
    ],
    "react/jsx-sort-props": [
      "error",
      {
        callbacksLast: true,
        shorthandFirst: true,
        multiline: "last",
        reservedFirst: true,
      },
    ],
    "react/button-has-type": ["error"],
    "react/jsx-boolean-value": ["error", "never"],
    "react/jsx-filename-extension": [
      "error",
      {
        allow: "as-needed",
        extensions: [".tsx"],
      },
    ],
    "react/jsx-fragments": ["error", "syntax"],
    "react/jsx-handler-names": ["error"],
    "react/jsx-key": [
      "error",
      {
        checkFragmentShorthand: true,
        checkKeyMustBeforeSpread: true,
        warnOnDuplicates: true,
      },
    ],
    "react/jsx-no-comment-textnodes": ["error"],
    "react/jsx-no-constructed-context-values": ["error"],
    "react/jsx-no-duplicate-props": ["error"],
    "react/jsx-no-leaked-render": ["error"],
    "react/jsx-no-target-blank": ["error"],
    "react/jsx-no-useless-fragment": [
      "error",
      {
        allowExpressions: true,
      },
    ],
    "react/jsx-pascal-case": [
      "error",
      {
        allowNamespace: true,
      },
    ],
    "react/no-array-index-key": ["error"],
    "react/no-object-type-as-default-prop": ["error"],
    "react/no-this-in-sfc": ["error"],
    "react/no-unstable-nested-components": ["error"],
    "react/prefer-stateless-function": ["error"],

    "no-lone-blocks": "off",
    camelcase: [
      "error",
      {
        ignoreDestructuring: true,
      },
    ],
    curly: ["error", "all"],
    "dot-notation": "error",
    eqeqeq: ["error", "always"],
    "guard-for-in": "error",
    "linebreak-style": ["error", "unix"],
    "no-array-constructor": "error",
    "no-bitwise": "error",
    "no-mixed-operators": "error",
    "no-multi-assign": "error",
    "no-console": [
      "warn",
      {
        allow: ["warn", "error", "info"],
      },
    ],
    "no-nested-ternary": "error",
    "no-new-func": "error",
    "no-tabs": "warn",
    "no-new-wrappers": "error",
    "no-return-assign": ["error", "always"],
    "no-script-url": "error",
    "no-self-compare": "error",
    "no-sequences": "error",
    "no-useless-constructor": "error",
    "object-shorthand": ["error", "always"],
    "prefer-arrow-callback": "warn",
    "prefer-const": "warn",
    "prefer-destructuring": [
      "warn",
      {
        array: true,
        object: true,
      },
      {
        enforceForRenamedProperties: false,
      },
    ],
    "prefer-numeric-literals": "error",
    "prefer-rest-params": "error",
    "prefer-spread": "warn",
    "prefer-template": "warn",
    "wrap-iife": ["error", "inside"],

    "prettier/prettier": "error",

    "sort-imports": "off",
    "simple-import-sort/imports": "error",
    "simple-import-sort/exports": "error",
    "import/first": "error",
    "import/newline-after-import": "error",
    "import/no-duplicates": "error",
  },
};

module.exports = config;
