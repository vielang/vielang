import type { Metadata, Viewport } from 'next';
import { Inter, Playfair_Display } from 'next/font/google';
import { Providers } from './providers';
import { getCurrentUser } from '@/lib/auth/get-current-user';
import './globals.css';

// next/font/google self-hosts the fonts at build time — no runtime request to
// fonts.googleapis.com, no layout shift (the CSS `size-adjust` fallback matches
// the metric of the real font). Vietnamese subset is required for VN copy.
const inter = Inter({
  subsets: ['latin', 'latin-ext', 'vietnamese'],
  variable: '--font-inter',
  display: 'swap',
  weight: ['400', '500', '600', '700'],
  preload: true,
});

const playfair = Playfair_Display({
  subsets: ['latin', 'latin-ext'],
  variable: '--font-playfair',
  display: 'swap',
  weight: ['400', '700'],
  preload: true,
});

export const metadata: Metadata = {
  title: 'VieLang — Learn English Online',
  manifest: '/manifest.webmanifest',
  appleWebApp: {
    capable: true,
    title: 'VieLang',
    statusBarStyle: 'black-translucent',
  },
  // Favicons + apple-touch-icon are auto-served by Next.js from
  // `src/app/icon.svg`. One scalable source drives every size, so we don't
  // need a `scripts/generate-icons.mjs` step or a PNG icon set anymore.
};

// Tinting the OS chrome (Android Chrome address bar, iOS status bar
// in standalone PWA) to match the brand primary — indigo-600.
export const viewport: Viewport = {
  width: 'device-width',
  initialScale: 1,
  themeColor: [
    { media: '(prefers-color-scheme: light)', color: '#4F46E5' },
    { media: '(prefers-color-scheme: dark)', color: '#818CF8' },
  ],
};

export default async function RootLayout({ children }: { children: React.ReactNode }) {
  // Resolve the signed-in user on the server so the Header and dashboards can
  // render their authenticated state on the first paint — no Skeleton, no
  // client-side getSession() round-trip wait.
  const initialUser = await getCurrentUser();

  // Static "vi" — LangContext hydrates the preferred VN/EN pair on the client
  // from localStorage; matching that against SSR here would produce a
  // hydration mismatch. Vietnamese is our default audience so the crawler
  // hint stays on VN.
  return (
    <html lang="vi" suppressHydrationWarning className={`${inter.variable} ${playfair.variable}`}>
      <body>
        <Providers initialUser={initialUser}>{children}</Providers>
      </body>
    </html>
  );
}
