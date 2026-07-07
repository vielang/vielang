import type { NextConfig } from 'next';
import bundleAnalyzer from '@next/bundle-analyzer';

/**
 * Bundle analyzer wrapper. Runs only when `ANALYZE=true`, so the plugin is
 * a no-op on Vercel / CI / dev. Invoke locally with:
 *
 *   ANALYZE=true npm run build
 *
 * Two HTML reports drop into `.next/analyze/` — one for the client bundle,
 * one for the server bundle. Open in a browser to spot bloat.
 */
const withBundleAnalyzer = bundleAnalyzer({
  enabled: process.env.ANALYZE === 'true',
});

/**
 * Security-header policy. Notes:
 *  - CSP: `unsafe-inline` on script-src is a Next.js constraint — the RSC
 *    runtime injects inline hydration payloads. We compensate with `frame-
 *    ancestors 'none'` (clickjacking) + strict `connect-src` allowlist.
 *  - `connect-src` includes wss:// for LiveKit. The exact LiveKit origin is
 *    derived from `NEXT_PUBLIC_LIVEKIT_WS_URL` so dev (ws://localhost:7880)
 *    and prod (wss://livekit.vielang.com) both work without a hand-edit.
 *  - HSTS + preload: only meaningful over HTTPS. Vercel serves the app on
 *    HTTPS, so we can safely opt in.
 *  - Permissions-Policy locks down microphone/camera to same-origin only —
 *    LiveKit signaling runs on wss so the `self` grant is enough.
 */
function livekitConnectOrigins(): string[] {
  // Pin whatever the app is actually configured to reach, plus the LiveKit
  // Cloud wildcard as a fallback for hosted deployments. Both http+ws and
  // https+wss variants are added when a URL is present so the browser
  // accepts the WS upgrade regardless of scheme.
  const origins = new Set<string>(['wss://*.livekit.cloud', 'wss://livekit.vielang.com']);
  const wsUrl = process.env.NEXT_PUBLIC_LIVEKIT_WS_URL;
  if (wsUrl) {
    try {
      const parsed = new URL(wsUrl);
      const host = parsed.host; // includes port when present
      if (parsed.protocol === 'ws:' || parsed.protocol === 'http:') {
        origins.add(`ws://${host}`);
        origins.add(`http://${host}`);
      } else {
        origins.add(`wss://${host}`);
        origins.add(`https://${host}`);
      }
    } catch {
      /* malformed URL — leave the default allowlist */
    }
  }
  return Array.from(origins);
}

const csp = [
  "default-src 'self'",
  "script-src 'self' 'unsafe-inline' 'unsafe-eval' https://*.supabase.co",
  "style-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob: https:",
  "font-src 'self' data:",
  [
    "connect-src 'self'",
    'https://*.supabase.co',
    'https://*.supabase.in',
    'wss://*.supabase.co',
    ...livekitConnectOrigins(),
  ].join(' '),
  "media-src 'self' blob:",
  "worker-src 'self' blob:",
  "frame-ancestors 'none'",
  "base-uri 'self'",
  "form-action 'self'",
  "object-src 'none'",
].join('; ');

const securityHeaders = [
  { key: 'Content-Security-Policy', value: csp },
  { key: 'Strict-Transport-Security', value: 'max-age=63072000; includeSubDomains; preload' },
  { key: 'X-Frame-Options', value: 'DENY' },
  { key: 'X-Content-Type-Options', value: 'nosniff' },
  { key: 'Referrer-Policy', value: 'strict-origin-when-cross-origin' },
  {
    key: 'Permissions-Policy',
    value:
      'camera=(self), microphone=(self), geolocation=(), interest-cohort=(), payment=(), usb=()',
  },
];

const nextConfig: NextConfig = {
  reactStrictMode: true,
  compress: true,
  images: {
    // Explicit allowlist. Wildcard ('**') would let any HTTPS host be
    // proxied through next/image, which is an SSRF amplifier (attacker
    // forces our server to fetch arbitrary internal/external URLs). Add
    // a domain here only after confirming it's a trusted CDN we control
    // or a known partner.
    remotePatterns: [
      { protocol: 'https', hostname: '*.supabase.co' },
      { protocol: 'https', hostname: '*.supabase.in' },
      { protocol: 'https', hostname: 'images.unsplash.com' },
      { protocol: 'https', hostname: 'cdn.vielang.com' },
      // Google OAuth avatars land here for users signed in via Google.
      { protocol: 'https', hostname: 'lh3.googleusercontent.com' },
    ],
  },
  experimental: {
    optimizePackageImports: ['lucide-react', 'motion'],
  },
  async headers() {
    return [{ source: '/(.*)', headers: securityHeaders }];
  },
};

export default withBundleAnalyzer(nextConfig);
