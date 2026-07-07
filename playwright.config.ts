import { defineConfig, devices } from '@playwright/test';

/**
 * Playwright config for VieLang E2E.
 *
 *  - Runs against a local `next dev` on port 3000 (started via `webServer`).
 *    Set `PLAYWRIGHT_BASE_URL` to point at a preview deployment instead.
 *  - Three viewport projects: desktop chromium, mobile chrome (Pixel 7),
 *    mobile safari (iPhone 14) — matches the Phần C responsive contract.
 *  - The Supabase env below is a build-only stub; real E2E runs need a
 *    seeded local Supabase (`supabase start` + `POST /api/seed`). Missing
 *    env keeps the app from booting but doesn't affect config load.
 *  - `RATE_LIMIT_DISABLED=true` in the webServer env — E2E floods the API
 *    faster than a real user and would hit the 5/min booking cap otherwise.
 */

const isCI = !!process.env.CI;
const baseURL = process.env.PLAYWRIGHT_BASE_URL || 'http://localhost:3000';

export default defineConfig({
  testDir: './e2e',
  timeout: 30_000,
  expect: { timeout: 5_000 },
  fullyParallel: true,
  forbidOnly: isCI,
  retries: isCI ? 2 : 0,
  workers: isCI ? 2 : undefined,
  reporter: isCI ? [['html', { open: 'never' }], ['list']] : 'list',
  use: {
    baseURL,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure',
    testIdAttribute: 'data-testid',
  },
  projects: [
    { name: 'chromium', use: { ...devices['Desktop Chrome'] } },
    { name: 'mobile-chrome', use: { ...devices['Pixel 7'] } },
    { name: 'mobile-safari', use: { ...devices['iPhone 14'] } },
  ],
  webServer: process.env.PLAYWRIGHT_BASE_URL
    ? undefined
    : {
        command: 'npm run dev',
        url: baseURL,
        reuseExistingServer: !isCI,
        timeout: 120_000,
        env: {
          RATE_LIMIT_DISABLED: 'true',
          NODE_ENV: 'test',
        },
      },
});
