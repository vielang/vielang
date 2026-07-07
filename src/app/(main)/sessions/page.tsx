import type { Metadata } from 'next';
import { getUpcomingGroupSessions } from '@/lib/supabase';
import { SessionsListClient } from './SessionsListClient';

export const dynamic = 'force-dynamic';

export const metadata: Metadata = {
  title: 'Free-talk sessions · VieLang',
  description:
    'Live and upcoming group English sessions — join community free-talk rooms hosted by our team.',
};

export default async function SessionsPage() {
  const sessions = await getUpcomingGroupSessions().catch(() => []);
  return <SessionsListClient sessions={sessions} />;
}
