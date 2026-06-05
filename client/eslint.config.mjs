import js from '@eslint/js';
import tsParser from '@typescript-eslint/parser';
import globals from 'globals';
import solid from 'eslint-plugin-solid';

const solidTypescriptConfig = solid.configs['flat/typescript'];

export default [
  js.configs.recommended,
  {
    ...solidTypescriptConfig,
    files: ['src/**/*.{ts,tsx}'],
    languageOptions: {
      ...solidTypescriptConfig.languageOptions,
      parser: tsParser,
      parserOptions: {
        ...solidTypescriptConfig.languageOptions?.parserOptions,
        project: './tsconfig.json',
        tsconfigRootDir: import.meta.dirname,
      },
      globals: {
        ...globals.browser,
        ...globals.es2024,
      },
    },
  },
];
