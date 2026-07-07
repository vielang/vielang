import next from 'eslint-config-next';
import nextCoreWebVitals from 'eslint-config-next/core-web-vitals';
import nextTypescript from 'eslint-config-next/typescript';
import prettier from 'eslint-config-prettier';

const config = [
  {
    ignores: [
      '.next/**',
      'node_modules/**',
      'build/**',
      'dist/**',
      'coverage/**',
      'public/sw.js',
      'src/types/database.ts',
      'next-env.d.ts',
      'supabase/migrations/**',
    ],
  },
  ...next,
  ...nextCoreWebVitals,
  ...nextTypescript,
  {
    rules: {
      '@typescript-eslint/no-explicit-any': 'warn',
      '@typescript-eslint/no-unused-vars': [
        'warn',
        {
          argsIgnorePattern: '^_',
          varsIgnorePattern: '^_',
          caughtErrorsIgnorePattern: '^_',
        },
      ],
      '@typescript-eslint/consistent-type-imports': [
        'warn',
        { prefer: 'type-imports', fixStyle: 'inline-type-imports' },
      ],
      'react/no-unescaped-entities': 'off',
      'react-hooks/exhaustive-deps': 'warn',

      // TODO(M1.2): raise back to 'error' after refactor.
      // New React 19 rules — legitimate patterns to fix incrementally when we
      // migrate to strict TS + refactor legacy hydration effects.
      'react-hooks/set-state-in-effect': 'warn',
      'react-hooks/globals': 'warn',
      '@typescript-eslint/no-empty-object-type': 'warn',
    },
  },
  {
    // shadcn/ui primitives access refs during render as part of the base-ui
    // provider pattern (labels bridge from SelectItem to SelectValue).
    // Downgrade the React 19 lint rule to a warn here — the primitives are
    // vendored and follow upstream shadcn conventions.
    files: ['src/components/ui/**/*.{ts,tsx}'],
    rules: {
      'react-hooks/refs': 'warn',
    },
  },
  prettier,
];

export default config;
