import type { MetadataRoute } from 'next';

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || 'https://vielang.com';

export default function robots(): MetadataRoute.Robots {
  return {
    rules: [
      {
        userAgent: '*',
        allow: '/',
        // Private surfaces: dashboards, session rooms, personal pages, and
        // any auth-adjacent path a bot might otherwise crawl into.
        disallow: [
          '/api/',
          '/admin',
          '/tutor',
          '/login',
          '/auth/',
          '/my-sessions',
          '/my-page',
          '/session/',
        ],
      },
    ],
    sitemap: `${siteUrl}/sitemap.xml`,
  };
}
