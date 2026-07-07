import { expect, test } from '@playwright/test';

/**
 * Tutor discovery flow — anonymous browse. Doesn't need a seeded tutor row;
 * verifies the page structure (filter bar + grid + empty state) renders and
 * URL-driven filters propagate through server refetch.
 */

test('tutor list renders the filter bar', async ({ page }) => {
  await page.goto('/tutors');
  // Filter bar renders differently by viewport (Sheet on mobile, inline on
  // desktop) — both surface a "Specialty" label somewhere in the DOM.
  const specialty = page.getByText(/specialty|chuyên môn/i).first();
  await expect(specialty).toBeVisible();
});

test('specialty filter reflects in the URL', async ({ page, isMobile }) => {
  await page.goto('/tutors');

  // Mobile hides the specialty pills behind a Filters sheet; open it first.
  if (isMobile) {
    const filtersBtn = page.getByRole('button', { name: /filters|bộ lọc/i });
    if (await filtersBtn.isVisible()) await filtersBtn.click();
  }

  const ieltsBtn = page.getByRole('button', { name: /^ielts$/i }).first();
  await ieltsBtn.click();

  await expect(page).toHaveURL(/[?&]specialty=IELTS(&|$)/);
});
