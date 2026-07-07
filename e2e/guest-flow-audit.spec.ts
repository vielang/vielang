import { expect, test } from '@playwright/test';
import { createClient } from '@supabase/supabase-js';
import { config as loadDotenv } from 'dotenv';
import { mkdirSync } from 'node:fs';

loadDotenv({ path: '.env.local', quiet: true });

/**
 * Guest-user flow audit. Walks an unauthenticated visitor through every
 * public route and records what they see: HTTP status, notFound render,
 * console errors, screenshots. Fail-soft — the log is the deliverable.
 *
 * Static routes are hit as-is. Dynamic routes ([id]) pull a real id from
 * the DB before hitting the URL — if the seed doesn't have a row, that
 * route is skipped with a note.
 *
 * Guarded routes (/my-sessions, /my-page, /admin, /tutor) are visited too,
 * but the expectation is a redirect to /login — we log whichever landing
 * page actually renders.
 */

const OUT_DIR = 'e2e/artifacts/guest-flow';

test.beforeAll(() => {
  mkdirSync(OUT_DIR, { recursive: true });
});

function admin() {
  const url = process.env.SUPABASE_URL || process.env.NEXT_PUBLIC_SUPABASE_URL;
  const key = process.env.SUPABASE_SERVICE_ROLE_KEY;
  if (!url || !key) throw new Error('Missing Supabase env');
  return createClient(url, key);
}

async function pickIds() {
  const sb = admin();
  const [tutor, groupSession, course, newsPost] = await Promise.all([
    sb
      .from('users')
      .select('id')
      .eq('role', 'tutor')
      .limit(1)
      .maybeSingle()
      .then((r) => r.data?.id ?? null),
    sb
      .from('sessions')
      .select('id')
      .eq('type', 'group')
      .in('status', ['pending', 'confirmed', 'live'])
      .limit(1)
      .maybeSingle()
      .then((r) => r.data?.id ?? null),
    sb
      .from('courses')
      .select('id')
      .eq('is_published', true)
      .limit(1)
      .maybeSingle()
      .then((r) => r.data?.id ?? null),
    // `news` table may not exist in every seed — swallow with then(null, ...)
    // because supabase's builder returns a PromiseLike without a .catch chain.
    sb
      .from('news')
      .select('id')
      .eq('is_published', true)
      .limit(1)
      .maybeSingle()
      .then(
        (r) => r.data?.id ?? null,
        () => null,
      ),
  ]);
  return { tutor, groupSession, course, newsPost };
}

interface RouteFinding {
  route: string;
  status: number | null;
  notFound: boolean;
  finalUrl: string;
  h1: string | null;
  errors: string[];
  skipped?: string;
}

const findings: RouteFinding[] = [];

async function visit(page: import('@playwright/test').Page, route: string): Promise<RouteFinding> {
  const errors: string[] = [];
  const errorHandler = (msg: import('@playwright/test').ConsoleMessage) => {
    if (msg.type() === 'error') {
      const t = msg.text();
      // Devtools + hydration noise is fine to ignore — we care about
      // application-level errors.
      if (t.includes('Failed to load resource') && t.includes('devtools')) return;
      errors.push('[console.error] ' + t);
    }
  };
  const pageErrorHandler = (err: Error) => {
    errors.push('[pageerror] ' + err.message);
  };
  page.on('console', errorHandler);
  page.on('pageerror', pageErrorHandler);
  try {
    const res = await page.goto(route, { waitUntil: 'networkidle', timeout: 20_000 });
    const notFound = await page
      .getByRole('heading', { name: /trang không tồn tại|page not found/i })
      .count();
    const h1 = await page
      .locator('h1')
      .first()
      .textContent()
      .catch(() => null);
    const finalUrl = new URL(page.url()).pathname + new URL(page.url()).search;
    const slug = route.replace(/[^a-z0-9]+/gi, '-').replace(/^-+|-+$/g, '') || 'root';
    await page.screenshot({ path: `${OUT_DIR}/${slug}.png`, fullPage: true }).catch(() => {});
    return {
      route,
      status: res?.status() ?? null,
      notFound: notFound > 0,
      finalUrl,
      h1: h1?.trim().slice(0, 80) ?? null,
      errors,
    };
  } catch (err) {
    return {
      route,
      status: null,
      notFound: false,
      finalUrl: route,
      h1: null,
      errors: [...errors, `[navigate error] ${(err as Error).message}`],
    };
  } finally {
    page.off('console', errorHandler);
    page.off('pageerror', pageErrorHandler);
  }
}

test('guest-flow audit', async ({ page, browserName }) => {
  test.skip(browserName !== 'chromium', 'chromium only');
  test.setTimeout(180_000);

  const ids = await pickIds();

  console.log('[seed ids]', ids);

  const staticRoutes = [
    '/',
    '/tutors',
    '/sessions',
    '/news',
    '/faq',
    '/contact',
    '/privacy',
    '/terms',
    '/refund',
    '/login',
  ];
  const guardedRoutes = ['/my-sessions', '/my-page', '/admin', '/tutor'];
  const dynamicRoutes: string[] = [];
  if (ids.tutor) dynamicRoutes.push(`/tutors/${ids.tutor}`);
  if (ids.groupSession) dynamicRoutes.push(`/sessions/${ids.groupSession}`);
  if (ids.course) dynamicRoutes.push(`/courses/${ids.course}`);
  if (ids.newsPost) dynamicRoutes.push(`/news/${ids.newsPost}`);

  for (const r of [...staticRoutes, ...dynamicRoutes, ...guardedRoutes]) {
    findings.push(await visit(page, r));
  }

  // Report.
  const bad = findings.filter(
    (f) => f.notFound || (f.status !== null && f.status >= 400) || f.errors.length > 0,
  );
  const ok = findings.filter((f) => !bad.includes(f));

  console.log('\n════════ GUEST FLOW AUDIT ════════');

  console.log(`OK: ${ok.length}    ISSUES: ${bad.length}`);

  console.log('\n── OK routes ──');
  for (const f of ok) {
    console.log(`  ✓ ${f.route.padEnd(50)} → ${f.status}  h1="${f.h1 ?? ''}"`);
  }
  if (bad.length > 0) {
    console.log('\n── ISSUES ──');
    for (const f of bad) {
      console.log(
        `  ✗ ${f.route}  status=${f.status}  notFound=${f.notFound}  final=${f.finalUrl}`,
      );

      if (f.h1) console.log(`      h1: "${f.h1}"`);
      for (const e of f.errors) {
        console.log(`      ${e}`);
      }
    }
  }

  expect(true).toBe(true);
});
