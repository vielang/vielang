'use client';

import { useEffect, useMemo, useState } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Search, MessagesSquare } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { GroupSessionCard } from '@/components/session/GroupSessionCard';
import { EmptyState } from '@/components/shared/EmptyState';
import { FilterPill } from '@/components/shared/FilterPill';
import { useDebouncedValue } from '@/hooks/use-debounced-value';
import type { SessionListItem } from '@/lib/supabase';
import type { GroupLevel } from '@/lib/types';

type LevelFilter = GroupLevel | 'all';

const LEVELS: { key: LevelFilter; label: string }[] = [
  { key: 'all', label: 'All levels' },
  { key: 'a1_a2', label: 'A1–A2' },
  { key: 'b1_b2', label: 'B1–B2' },
  { key: 'c1_c2', label: 'C1–C2' },
];

const isLevel = (v: string | null): v is LevelFilter =>
  v === 'all' || v === 'a1_a2' || v === 'b1_b2' || v === 'c1_c2';

export function SessionsListClient({ sessions }: { sessions: SessionListItem[] }) {
  const router = useRouter();
  const searchParams = useSearchParams();

  // Prefill from ?q=&level= (set by the homepage hero) so a shared URL loads
  // straight into the intended filter. The full source list is fetched
  // server-side; filtering happens client-side against that in-memory copy.
  const initialQuery = searchParams.get('q') ?? '';
  const initialLevelParam = searchParams.get('level');
  const initialLevel: LevelFilter = isLevel(initialLevelParam) ? initialLevelParam : 'all';

  const [query, setQuery] = useState(initialQuery);
  const [level, setLevel] = useState<LevelFilter>(initialLevel);

  // Debounce the URL write, not the filtering. Filtering off `query` stays
  // instant (feels responsive as you type); we only push a new URL every 250 ms
  // so the address bar / history don't churn per keystroke.
  const debouncedQuery = useDebouncedValue(query.trim(), 250);
  useEffect(() => {
    const next = new URLSearchParams();
    if (debouncedQuery) next.set('q', debouncedQuery);
    if (level !== 'all') next.set('level', level);
    const nextQs = next.toString();
    const currentQs = searchParams.toString();
    if (nextQs === currentQs) return;
    router.replace(nextQs ? `/sessions?${nextQs}` : '/sessions', { scroll: false });
  }, [debouncedQuery, level, router, searchParams]);

  // Count sessions per level from the source list. Powers the badge on each
  // filter chip so users see what's in each bucket before clicking.
  const levelCounts = useMemo(() => {
    const counts: Record<LevelFilter, number> = {
      all: sessions.length,
      a1_a2: 0,
      b1_b2: 0,
      c1_c2: 0,
    };
    for (const s of sessions) {
      const key = (s.level ?? 'all') as LevelFilter;
      if (key in counts) counts[key]++;
    }
    return counts;
  }, [sessions]);

  const filtered = useMemo(() => {
    const needle = query.trim().toLowerCase();
    return sessions.filter((s) => {
      if (level !== 'all' && s.level !== level) return false;
      if (!needle) return true;
      return (
        (s.topic_en || '').toLowerCase().includes(needle) ||
        (s.topic_vn || '').toLowerCase().includes(needle) ||
        (s.description || '').toLowerCase().includes(needle) ||
        (s.tutor_name || '').toLowerCase().includes(needle)
      );
    });
  }, [sessions, query, level]);

  const hasFilters = query.trim() !== '' || level !== 'all';
  const showingAll = !hasFilters;

  const clearFilters = () => {
    setQuery('');
    setLevel('all');
  };

  return (
    <div className="space-y-6">
      <header className="space-y-1">
        <h1 className="type-page text-slate-900 dark:text-slate-100">Free-talk sessions</h1>
        <p className="max-w-xl text-sm text-slate-500 dark:text-slate-400">
          Community group rooms hosted by our team. Grab a seat, hop in and practice speaking with
          other learners.
        </p>
      </header>

      <div className="flex flex-wrap items-center gap-3">
        <div className="relative max-w-md min-w-[220px] flex-1">
          <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2" />
          <Input
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            placeholder="Search topics, hosts, keywords…"
            aria-label="Search sessions"
            className="h-9 pl-9"
          />
        </div>
        <div className="flex flex-wrap gap-1.5">
          {LEVELS.map((l) => (
            <FilterPill
              key={l.key}
              active={level === l.key}
              onClick={() => setLevel(l.key)}
              size="md"
              label={
                <>
                  {l.label}
                  <span
                    className={`ml-1.5 inline-flex h-4 min-w-[18px] items-center justify-center rounded-full px-1 text-[10px] font-semibold tabular-nums ${
                      level === l.key
                        ? 'bg-white/20 text-white'
                        : 'bg-slate-100 text-slate-500 dark:bg-slate-800 dark:text-slate-400'
                    }`}
                  >
                    {levelCounts[l.key]}
                  </span>
                </>
              }
            />
          ))}
        </div>
      </div>

      <p className="text-xs text-slate-500 tabular-nums dark:text-slate-400">
        {sessions.length === 0
          ? 'No sessions on the schedule yet.'
          : showingAll
            ? `${sessions.length} upcoming ${sessions.length === 1 ? 'session' : 'sessions'}`
            : `${filtered.length} of ${sessions.length} sessions match`}
      </p>

      {filtered.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-200 dark:border-slate-800">
          <EmptyState
            icon={<MessagesSquare className="size-6" />}
            title={
              sessions.length === 0
                ? 'No sessions on the schedule'
                : 'No sessions match your filters'
            }
            description={
              sessions.length === 0
                ? 'New free-talk rooms drop here as soon as our team schedules them. Check back in a bit.'
                : 'Try loosening your filters or clearing the search — a room for another level might be a good fit.'
            }
            action={
              hasFilters ? (
                <Button size="sm" onClick={clearFilters}>
                  Clear filters
                </Button>
              ) : undefined
            }
          />
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-3">
          {filtered.map((s, i) => (
            <GroupSessionCard key={s.id} session={s} index={i} />
          ))}
        </div>
      )}
    </div>
  );
}
