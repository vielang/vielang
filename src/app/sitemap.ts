import type { MetadataRoute } from 'next';
import { supabase } from '@/lib/supabase';

const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || 'https://vielang.com';

// Dynamic sitemap: static entries + every public tutor profile + every
// published course + every upcoming group session. Runs on request (Next.js
// generates it lazily via /sitemap.xml). Bounded to a few hundred rows per
// entity so it stays small enough for crawlers to fetch in one request.
export default async function sitemap(): Promise<MetadataRoute.Sitemap> {
  const now = new Date();

  const staticEntries: MetadataRoute.Sitemap = [
    { url: `${siteUrl}/`, lastModified: now, changeFrequency: 'daily', priority: 1 },
    { url: `${siteUrl}/tutors`, lastModified: now, changeFrequency: 'daily', priority: 0.9 },
    { url: `${siteUrl}/sessions`, lastModified: now, changeFrequency: 'hourly', priority: 0.8 },
    { url: `${siteUrl}/news`, lastModified: now, changeFrequency: 'weekly', priority: 0.7 },
    { url: `${siteUrl}/faq`, lastModified: now, changeFrequency: 'monthly', priority: 0.5 },
    { url: `${siteUrl}/contact`, lastModified: now, changeFrequency: 'monthly', priority: 0.4 },
    { url: `${siteUrl}/terms`, lastModified: now, changeFrequency: 'yearly', priority: 0.3 },
    { url: `${siteUrl}/privacy`, lastModified: now, changeFrequency: 'yearly', priority: 0.3 },
    { url: `${siteUrl}/refund`, lastModified: now, changeFrequency: 'yearly', priority: 0.3 },
  ];

  // Approved tutors — public profile pages. Cap at 500 to keep the file
  // small; a healthy platform never has more than a few dozen approved
  // tutors before we'd want to paginate the sitemap into an index.
  const [tutorRes, courseRes, sessionRes] = await Promise.all([
    supabase
      .from('users')
      .select('id, created_at, profile:tutor_profiles!inner(is_approved)')
      .eq('role', 'tutor')
      .eq('tutor_profiles.is_approved', true)
      .limit(500),
    supabase
      .from('courses')
      .select('id, created_at')
      .eq('is_published', true)
      .limit(500)
      .order('created_at', { ascending: false }),
    supabase
      .from('sessions')
      .select('id, scheduled_at')
      .eq('type', 'group')
      .in('status', ['pending', 'confirmed', 'live'])
      .gte('scheduled_at', new Date(now.getTime() - 30 * 60_000).toISOString())
      .limit(200)
      .order('scheduled_at', { ascending: true }),
  ]);

  const tutorEntries: MetadataRoute.Sitemap = (
    (tutorRes.data ?? []) as Array<{
      id: string;
      created_at: string;
    }>
  ).map((t) => ({
    url: `${siteUrl}/tutors/${t.id}`,
    lastModified: new Date(t.created_at),
    changeFrequency: 'weekly',
    priority: 0.8,
  }));

  const courseEntries: MetadataRoute.Sitemap = (
    (courseRes.data ?? []) as Array<{
      id: string;
      created_at: string;
    }>
  ).map((c) => ({
    url: `${siteUrl}/courses/${c.id}`,
    lastModified: new Date(c.created_at),
    changeFrequency: 'weekly',
    priority: 0.7,
  }));

  const sessionEntries: MetadataRoute.Sitemap = (
    (sessionRes.data ?? []) as Array<{
      id: string;
      scheduled_at: string;
    }>
  ).map((s) => ({
    url: `${siteUrl}/sessions/${s.id}`,
    lastModified: new Date(s.scheduled_at),
    changeFrequency: 'hourly',
    priority: 0.6,
  }));

  return [...staticEntries, ...tutorEntries, ...courseEntries, ...sessionEntries];
}
