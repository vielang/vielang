import type { MetadataRoute } from 'next';

// PWA manifest — Next.js App Router serves this at /manifest.webmanifest
// automatically. The `themeColor` mirrors the brand primary (indigo-600) so
// iOS status bar and Android splash blend with the header. The manifest
// points at a single SVG icon (`/brand-v.svg`) — modern Android + iOS PWA
// installers scale it to whatever launcher size they need, and the maskable
// variant works because our rounded-square background covers the safe area.
export default function manifest(): MetadataRoute.Manifest {
  return {
    name: 'VieLang · Learn English Online',
    short_name: 'VieLang',
    description:
      'Learn English 1-on-1 with certified tutors via live video call. Book flexible sessions, track your progress.',
    start_url: '/',
    scope: '/',
    display: 'standalone',
    orientation: 'portrait-primary',
    lang: 'vi-VN',
    dir: 'ltr',
    theme_color: '#4F46E5',
    background_color: '#FFFFFF',
    categories: ['education', 'productivity'],
    icons: [
      {
        src: '/brand-v.svg',
        sizes: 'any',
        type: 'image/svg+xml',
        purpose: 'any',
      },
      {
        src: '/brand-v.svg',
        sizes: 'any',
        type: 'image/svg+xml',
        purpose: 'maskable',
      },
    ],
  };
}
