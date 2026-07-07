'use client';

import { useEffect } from 'react';

/**
 * Registers /sw.js once on first mount in production builds.
 *
 * We deliberately skip dev — Next's HMR + the SW's stale-while-revalidate
 * fights each other in unhelpful ways (stale chunks, white screens,
 * developer confusion). Production only is the standard PWA pattern.
 *
 * The registration is fire-and-forget. If it fails (privacy mode, an
 * unsupported browser, an explicit user opt-out), the site keeps
 * working without offline caching — no error surface needed.
 */
export function ServiceWorkerRegister() {
  useEffect(() => {
    if (process.env.NODE_ENV !== 'production') return;
    if (typeof navigator === 'undefined' || !('serviceWorker' in navigator)) return;

    // Register lazily after the page has settled so we don't fight the
    // initial-paint critical path on slow devices.
    const onLoad = () => {
      navigator.serviceWorker.register('/sw.js', { scope: '/' }).catch(() => {
        // Swallow — the page works fine without a SW.
      });
    };
    if (document.readyState === 'complete') onLoad();
    else window.addEventListener('load', onLoad, { once: true });

    return () => window.removeEventListener('load', onLoad);
  }, []);

  return null;
}
