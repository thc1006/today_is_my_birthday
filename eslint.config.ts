import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import type { ConfigArray } from 'typescript-eslint';

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ['ts/**/*.ts'],
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
    rules: {
      '@typescript-eslint/explicit-function-return-type': 'error',
      '@typescript-eslint/no-unused-vars': 'error',
      'no-console': 'warn',
    },
  },
  {
    ignores: ['node_modules/', 'static/', '*.js', '*.cjs', '*.mjs'],
  }
);
