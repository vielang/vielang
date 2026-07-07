import { defineConfig } from 'vitest/config';
import react from '@vitejs/plugin-react-swc';

export default defineConfig({
  plugins: [react()],
  resolve: {
    // Vite now supports tsconfig paths natively — @/* → ./src/*.
    tsconfigPaths: true,
    alias: {
      // `server-only` is a Next.js runtime marker (throws if imported into a
      // client bundle). Vitest runs in Node, so the enforcement is meaningless
      // here — alias to an empty stub so `import 'server-only'` becomes a
      // no-op instead of a module-not-found.
      'server-only': new URL('./src/test/server-only-shim.ts', import.meta.url).pathname,
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./src/test/setup.ts'],
    css: false,
    include: ['src/**/*.{test,spec}.{ts,tsx}'],
    // e2e/** is Playwright territory — must not run under Vitest or the
    // @playwright/test globals will collide with Vitest's.
    exclude: ['node_modules', '.next', 'e2e/**', 'playwright-report/**'],
    coverage: {
      provider: 'v8',
      reporter: ['text', 'html', 'lcov'],
      // Narrow the coverage surface to modules we already have tests for.
      // Widen this list as new test files land (M2.3+, then M4 features).
      include: [
        'src/lib/schemas/session.ts',
        'src/lib/api-errors.ts',
        'src/lib/promo.ts',
        'src/lib/booking.ts',
        'src/lib/auth-server.ts',
      ],
      exclude: ['src/**/*.{test,spec}.{ts,tsx}', 'src/types/**'],
      thresholds: {
        // Per-file thresholds — tested modules must stay well-covered.
        // The overall repo-wide gate is added in a follow-up milestone.
        lines: 70,
        functions: 70,
        branches: 65,
        statements: 70,
      },
    },
    testTimeout: 10_000,
  },
});
