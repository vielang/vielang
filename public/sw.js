/*
 * VieLang service worker — hand-written, Workbox-free.
 *
 * Strategies by request type:
 *  - Next static chunks under /_next/static/*   : cache-first (file path
 *    is already content-hashed by Next, so a stale cache hit is impossible
 *    for *current* deploy assets and harmless for old ones)
 *  - Same-origin images                          : cache-first
 *  - GET /api/tutors, /api/courses, /api/banners, /api/news
 *                                                : stale-while-revalidate
 *    (instant render from cache while we refresh in the background)
 *  - Other /api/*                                : network-only (auth,
 *    sessions, reviews, materials must NOT be cached)
 *  - Navigation (HTML)                           : network-first → /offline
 *    fallback if both network and cache miss
 *
 * Cache versioning: bumping CACHE_VERSION on a deploy purges every old
 * cache the moment the new SW activates. No partial-stale state.
 */

const CACHE_VERSION = 'v2';
const STATIC_CACHE = `static-${CACHE_VERSION}`;
const RUNTIME_CACHE = `runtime-${CACHE_VERSION}`;
const IMAGE_CACHE = `images-${CACHE_VERSION}`;
const API_CACHE = `api-${CACHE_VERSION}`;

const OFFLINE_URL = '/offline';
// Pre-cached at install so the offline page is guaranteed available
// the first time the user loses network — without this the first
// offline navigation would still hard-fail.
const PRECACHE = [OFFLINE_URL, '/manifest.webmanifest'];

// API GETs we treat as cacheable. Anything else (auth, sessions, reviews,
// materials, mutations) is intentionally bypass-cache so we never serve
// stale or cross-user data.
const CACHEABLE_API = [
  '/api/tutors',
  '/api/courses',
  '/api/banners',
  '/api/news',
];

self.addEventListener('install', (event) => {
  event.waitUntil(
    (async () => {
      const cache = await caches.open(STATIC_CACHE);
      await cache.addAll(PRECACHE).catch(() => {
        // Network-failed precache shouldn't block install — we'll
        // retry on first activation runtime fetch.
      });
      self.skipWaiting();
    })(),
  );
});

self.addEventListener('activate', (event) => {
  event.waitUntil(
    (async () => {
      const keep = new Set([STATIC_CACHE, RUNTIME_CACHE, IMAGE_CACHE, API_CACHE]);
      const names = await caches.keys();
      await Promise.all(names.filter((n) => !keep.has(n)).map((n) => caches.delete(n)));
      await self.clients.claim();
    })(),
  );
});

const isStaticAsset = (url) =>
  url.pathname.startsWith('/_next/static/') || url.pathname.startsWith('/icons/');

const isImage = (req, url) =>
  req.destination === 'image' ||
  /\.(?:png|jpg|jpeg|webp|gif|svg|ico)$/i.test(url.pathname);

const isCacheableApi = (url) =>
  CACHEABLE_API.some((p) => url.pathname === p || url.pathname.startsWith(`${p}/`));

const isOtherApi = (url) => url.pathname.startsWith('/api/');

async function cacheFirst(req, cacheName) {
  const cache = await caches.open(cacheName);
  const cached = await cache.match(req);
  if (cached) return cached;
  try {
    const fresh = await fetch(req);
    if (fresh.ok) cache.put(req, fresh.clone()).catch(() => {});
    return fresh;
  } catch {
    if (cached) return cached;
    throw new Error('Offline and no cache');
  }
}

async function staleWhileRevalidate(req, cacheName) {
  const cache = await caches.open(cacheName);
  const cached = await cache.match(req);
  const network = fetch(req)
    .then((res) => {
      if (res.ok) cache.put(req, res.clone()).catch(() => {});
      return res;
    })
    .catch(() => undefined);
  return cached || (await network) || new Response('Offline', { status: 503 });
}

async function networkFirstNav(req) {
  try {
    const fresh = await fetch(req);
    return fresh;
  } catch {
    const cache = await caches.open(STATIC_CACHE);
    const offline = await cache.match(OFFLINE_URL);
    if (offline) return offline;
    return new Response('Offline', {
      status: 503,
      headers: { 'Content-Type': 'text/plain' },
    });
  }
}

self.addEventListener('fetch', (event) => {
  const req = event.request;

  // SW only governs GET — POSTs (bookings, payments, auth) always go
  // straight to network, no interception.
  if (req.method !== 'GET') return;

  const url = new URL(req.url);

  // Only handle same-origin. Cross-origin (LiveKit signaling, Supabase
  // storage, Unsplash images) can hit the network directly and the
  // browser's own caching is fine.
  if (url.origin !== self.location.origin) return;

  // 1. Static assets — cache-first
  if (isStaticAsset(url)) {
    event.respondWith(cacheFirst(req, STATIC_CACHE));
    return;
  }

  // 2. Cacheable API endpoints — stale-while-revalidate
  if (isCacheableApi(url)) {
    event.respondWith(staleWhileRevalidate(req, API_CACHE));
    return;
  }

  // 3. Other API endpoints — network-only (skip SW completely)
  if (isOtherApi(url)) return;

  // 4. Same-origin images — cache-first
  if (isImage(req, url)) {
    event.respondWith(cacheFirst(req, IMAGE_CACHE));
    return;
  }

  // 5. Navigation (HTML) — network-first → /offline fallback
  if (req.mode === 'navigate') {
    event.respondWith(networkFirstNav(req));
    return;
  }

  // 6. Everything else (fonts, CSS, JS chunks not under _next/static)
  //    — stale-while-revalidate so first paint is instant but updates
  //    catch up on next visit.
  event.respondWith(staleWhileRevalidate(req, RUNTIME_CACHE));
});
