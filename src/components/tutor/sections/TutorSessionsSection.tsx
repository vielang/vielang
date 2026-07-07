'use client';

import { useState, useMemo, useTransition } from 'react';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';
import { Check, Loader2, CalendarClock, CalendarCheck2, CalendarX2 } from 'lucide-react';
import { SessionCard } from '@/components/session/SessionCard';
import { EmptyState } from '@/components/shared/EmptyState';
import { getAuthHeaders } from '@/contexts/AuthContext';
import type { SessionListItem } from '@/lib/supabase';
import { errorMessage } from '@/lib/errors';

type Tab = 'upcoming' | 'completed' | 'cancelled';

function bucketize(all: SessionListItem[]) {
  const upcoming: SessionListItem[] = [];
  const completed: SessionListItem[] = [];
  const cancelled: SessionListItem[] = [];
  for (const s of all) {
    if (s.status === 'completed') completed.push(s);
    else if (s.status === 'cancelled' || s.status === 'no_show') cancelled.push(s);
    else upcoming.push(s);
  }
  return { upcoming, completed, cancelled };
}

export function TutorSessionsSection({ sessions }: { sessions: SessionListItem[] }) {
  const router = useRouter();
  const [, startTransition] = useTransition();
  const [tab, setTab] = useState<Tab>('upcoming');
  const [confirming, setConfirming] = useState<string | null>(null);

  const buckets = useMemo(() => bucketize(sessions), [sessions]);
  const current = buckets[tab];
  const pendingCount = sessions.filter((s) => s.status === 'pending').length;

  const refresh = () => startTransition(() => router.refresh());

  const confirmSession = async (id: string) => {
    setConfirming(id);
    try {
      const res = await fetch(`/api/sessions/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({ action: 'confirm' }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error || 'confirm_failed');
      toast.success('Session confirmed');
      refresh();
    } catch (err) {
      toast.error('Could not confirm', { description: errorMessage(err) });
    } finally {
      setConfirming(null);
    }
  };

  const tabs: { key: Tab; label: string; count: number }[] = [
    { key: 'upcoming', label: 'Upcoming', count: buckets.upcoming.length },
    { key: 'completed', label: 'Completed', count: buckets.completed.length },
    { key: 'cancelled', label: 'Cancelled', count: buckets.cancelled.length },
  ];

  return (
    <div className="space-y-6">
      <header>
        <h1 className="text-brand dark:text-accent-warm font-serif text-2xl font-bold">
          My sessions
        </h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          Every class you're teaching.{' '}
          {pendingCount > 0 && (
            <span className="font-semibold text-amber-600 dark:text-amber-400">
              {pendingCount} awaiting your confirmation.
            </span>
          )}
        </p>
      </header>

      <nav className="flex gap-1 border-b border-slate-200 dark:border-slate-800">
        {tabs.map((t) => (
          <button
            key={t.key}
            type="button"
            onClick={() => setTab(t.key)}
            className={`relative h-10 px-4 text-sm font-semibold transition-colors ${
              tab === t.key
                ? 'text-brand dark:text-accent-warm'
                : 'text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200'
            }`}
          >
            {t.label}
            <span
              className={`ml-1.5 inline-flex h-5 min-w-5 items-center justify-center rounded-full px-1.5 text-[10px] font-bold ${
                tab === t.key
                  ? 'bg-brand text-white'
                  : 'bg-slate-100 text-slate-500 dark:bg-slate-800 dark:text-slate-400'
              }`}
            >
              {t.count}
            </span>
            {tab === t.key && (
              <span className="bg-brand absolute right-2 -bottom-px left-2 h-0.5 rounded-full" />
            )}
          </button>
        ))}
      </nav>

      {current.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-200 dark:border-slate-800">
          <EmptyState
            icon={
              tab === 'upcoming' ? (
                <CalendarClock className="size-6" />
              ) : tab === 'completed' ? (
                <CalendarCheck2 className="size-6" />
              ) : (
                <CalendarX2 className="size-6" />
              )
            }
            title={
              tab === 'upcoming'
                ? 'No sessions on the calendar'
                : tab === 'completed'
                  ? 'No completed sessions yet'
                  : 'No cancelled sessions'
            }
            description={
              tab === 'upcoming'
                ? "You'll see student bookings here as soon as they land — remember to confirm pending ones so students know you're in."
                : tab === 'completed'
                  ? 'Every session you finish teaching will appear here.'
                  : 'Sessions you cancel — or that were marked no-show — live here.'
            }
          />
        </div>
      ) : (
        <div className="space-y-3">
          {current.map((s) => (
            <div key={s.id} className="relative">
              <SessionCard session={s} viewerRole="tutor" onChanged={refresh} />
              {s.status === 'pending' && (
                <button
                  type="button"
                  onClick={() => confirmSession(s.id)}
                  disabled={confirming === s.id}
                  className="absolute top-4 right-4 inline-flex h-8 items-center gap-1.5 rounded-lg bg-emerald-100 px-3 text-xs font-semibold text-emerald-800 hover:bg-emerald-200 disabled:opacity-40"
                >
                  {confirming === s.id ? (
                    <Loader2 className="size-3.5 animate-spin" />
                  ) : (
                    <Check className="size-3.5" />
                  )}
                  Confirm
                </button>
              )}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
