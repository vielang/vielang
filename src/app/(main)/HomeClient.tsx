'use client';

import { HomeContent, type HomeStats } from '@/components/home/HomeContent';
import type { Tutor } from '@/lib/types';
import type { SessionListItem } from '@/lib/supabase';

export function HomeClient({
  featuredTutors,
  upcomingSessions,
  stats,
}: {
  featuredTutors: Tutor[];
  upcomingSessions: SessionListItem[];
  stats: HomeStats;
}) {
  return (
    <HomeContent
      featuredTutors={featuredTutors}
      upcomingSessions={upcomingSessions}
      stats={stats}
    />
  );
}
