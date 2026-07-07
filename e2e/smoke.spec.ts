import { expect, test } from '@playwright/test';

/**
 * Smoke tests — proves the shell renders on every viewport project.
 * Deliberately shallow: no data assertions, no network mocks, no auth.
 * If these fail, the whole suite is likely broken and CI should stop
 * before running deeper flows.
 */

test('homepage responds and renders the brand mark', async ({ page }) => {
  const response = await page.goto('/');
  expect(response?.ok(), 'homepage should return 2xx').toBeTruthy();
  await expect(page).toHaveTitle(/VieLang/);
  // Brand wordmark appears in Header on every viewport (mobile hamburger
  // still shows the logo).
  await expect(page.getByRole('link', { name: /vielang/i }).first()).toBeVisible();
});

test('login page renders the form + Google button', async ({ page }) => {
  await page.goto('/login');
  await expect(page.getByRole('button', { name: /google/i })).toBeVisible();
  // Email field is type="text" not type="email" because it also accepts a
  // phone number (see LoginPanel placeholder "Email / Phone number").
  // autocomplete="username" is the stable semantic anchor.
  await expect(page.locator('input[autocomplete="username"]')).toBeVisible();
  await expect(page.locator('input[type="password"]')).toBeVisible();
});

test('/api/health returns ok', async ({ request }) => {
  const res = await request.get('/api/health');
  expect(res.ok()).toBeTruthy();
  const body = await res.json();
  expect(body.status).toBe('ok');
  expect(body.service).toBe('vielang-web');
});
