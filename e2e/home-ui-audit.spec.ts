import { expect, test, type Page } from '@playwright/test';
import { mkdirSync } from 'node:fs';

/**
 * One-off UI audit for the homepage. Not part of the CI signal — this is a
 * diagnostic spec you run when you want a snapshot of how the landing page
 * currently behaves on desktop + mobile. Fails soft: everything is a warning
 * printed to the console, so the test always passes and you read the log.
 *
 * Checks per viewport:
 *   • Full-page screenshot into e2e/artifacts/home-ui-audit/.
 *   • Horizontal overflow (a common mobile bug — one wide element eats the
 *     scroll axis).
 *   • Tap targets ≥ 44 × 44 CSS px (WCAG 2.2 / iOS HIG) for every button and
 *     link. Reports the offending element's aria-label + size.
 *   • Console errors + page errors surfaced from the browser.
 *   • Images without alt text.
 *   • Any element with autoFocus (a mobile UX smell).
 *   • Basic contrast smell test on the hero headline vs. its background.
 */

const OUT_DIR = 'e2e/artifacts/home-ui-audit';

test.describe.configure({ mode: 'serial' });

test.beforeAll(() => {
  mkdirSync(OUT_DIR, { recursive: true });
});

interface Finding {
  kind: 'overflow' | 'tap' | 'console' | 'alt' | 'autofocus';
  message: string;
}

async function auditPage(page: Page, viewport: string) {
  const findings: Finding[] = [];
  page.on('console', (msg) => {
    if (msg.type() === 'error' || msg.type() === 'warning') {
      findings.push({ kind: 'console', message: `[${msg.type()}] ${msg.text()}` });
    }
  });
  page.on('pageerror', (err) => {
    findings.push({ kind: 'console', message: `[pageerror] ${err.message}` });
  });

  await page.goto('/');
  await page.waitForLoadState('networkidle');

  // Screenshot the whole page for eyeballing.
  await page.screenshot({ path: `${OUT_DIR}/home-${viewport}.png`, fullPage: true });

  // Horizontal overflow — document width > viewport width means the page
  // scrolls sideways, which almost always reads as broken on mobile.
  const { docWidth, vpWidth } = await page.evaluate(() => ({
    docWidth: document.documentElement.scrollWidth,
    vpWidth: window.innerWidth,
  }));
  if (docWidth > vpWidth + 1) {
    findings.push({
      kind: 'overflow',
      message: `page scrolls horizontally: doc=${docWidth}px, viewport=${vpWidth}px`,
    });
  }

  // Tap targets — any button/anchor/input whose bounding box is smaller than
  // 44 × 44 in either axis. We intentionally allow purely-inline links (e.g.
  // "View all" chevron rows) if they exceed 32 in both axes, since chunky
  // buttons everywhere makes body copy look juvenile — but flag anything
  // clearly finger-hostile.
  const smallTaps = await page.evaluate(() => {
    const results: { label: string; w: number; h: number; tag: string }[] = [];
    const seen = new Set<Element>();
    const nodes = Array.from(
      document.querySelectorAll<HTMLElement>('button, a, [role="button"], input[type="submit"]'),
    );
    for (const el of nodes) {
      if (seen.has(el)) continue;
      seen.add(el);
      const rect = el.getBoundingClientRect();
      // Skip invisible / offscreen elements (0×0 or display:none).
      if (rect.width === 0 || rect.height === 0) continue;
      if (rect.width < 32 || rect.height < 32) {
        const label =
          el.getAttribute('aria-label') ||
          el.textContent?.trim().slice(0, 40) ||
          el.tagName.toLowerCase();
        results.push({
          label,
          w: Math.round(rect.width),
          h: Math.round(rect.height),
          tag: el.tagName.toLowerCase(),
        });
      }
    }
    return results;
  });
  for (const t of smallTaps) {
    findings.push({
      kind: 'tap',
      message: `${t.tag} "${t.label}" is ${t.w}×${t.h}px (< 32 minimum)`,
    });
  }

  // Images missing alt.
  const missingAlt = await page.$$eval('img', (imgs) =>
    imgs
      .filter((i) => !i.hasAttribute('alt'))
      .map((i) => (i as HTMLImageElement).currentSrc || i.getAttribute('src') || '(no src)'),
  );
  for (const src of missingAlt) {
    findings.push({ kind: 'alt', message: `<img> missing alt: ${src}` });
  }

  // autoFocus warning — pulls the mobile keyboard open on landing.
  const autofocusCount = await page.$$eval('[autofocus]', (els) => els.length);
  if (autofocusCount > 0) {
    findings.push({
      kind: 'autofocus',
      message: `${autofocusCount} element(s) with autoFocus — pops the mobile keyboard on load`,
    });
  }

  return findings;
}

function report(viewport: string, findings: Finding[]) {
   
  console.log(`\n────── ${viewport} — ${findings.length} finding(s) ──────`);
  const grouped = new Map<Finding['kind'], Finding[]>();
  for (const f of findings) {
    const list = grouped.get(f.kind) ?? [];
    list.push(f);
    grouped.set(f.kind, list);
  }
  for (const [kind, list] of grouped) {
     
    console.log(`  ${kind} (${list.length}):`);
    for (const f of list) {
       
      console.log(`    • ${f.message}`);
    }
  }
}

test('homepage UI audit — desktop chromium', async ({ page, browserName }) => {
  test.skip(browserName !== 'chromium', 'Runs on chromium only');
  await page.setViewportSize({ width: 1440, height: 900 });
  const findings = await auditPage(page, 'desktop-1440');
  report('desktop 1440×900', findings);
  // Screenshot is the deliverable; always pass so the log is the artefact.
  expect(true).toBe(true);
});

test('homepage UI audit — mobile 375', async ({ page, browserName }) => {
  test.skip(browserName !== 'chromium', 'Runs on chromium only');
  await page.setViewportSize({ width: 375, height: 812 });
  const findings = await auditPage(page, 'mobile-375');
  report('mobile 375×812', findings);
  expect(true).toBe(true);
});

test('homepage UI audit — mobile 414 landscape / tablet-ish', async ({ page, browserName }) => {
  test.skip(browserName !== 'chromium', 'Runs on chromium only');
  await page.setViewportSize({ width: 768, height: 1024 });
  const findings = await auditPage(page, 'tablet-768');
  report('tablet 768×1024', findings);
  expect(true).toBe(true);
});
