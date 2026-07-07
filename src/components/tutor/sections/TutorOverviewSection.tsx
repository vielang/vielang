'use client';

import { CalendarClock, Star, TrendingUp, MessageSquareText, CheckCheck } from 'lucide-react';
import { formatWhenLine } from '@/lib/time';
import { formatVnd } from '@/lib/format';
import type { SessionListItem, TutorKpis } from '@/lib/supabase';
import type { TutorProfile } from '@/lib/types';

const formatWhen = (iso: string) => formatWhenLine(iso, 'card-no-year', ' · ');

interface Props {
  kpis: TutorKpis;
  profile: TutorProfile;
  sessions: SessionListItem[];
}

export function TutorOverviewSection({ kpis, profile, sessions }: Props) {
  const upcoming = sessions
    .filter((s) => ['pending', 'confirmed', 'live'].includes(s.status))
    .slice(0, 5);

  return (
    <div className="space-y-6">
      <header>
        <h1 className="text-brand dark:text-accent-warm font-serif text-2xl font-bold md:text-3xl">
          Overview
        </h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          Your teaching activity at a glance.
        </p>
      </header>

      {!profile.is_approved && (
        <div className="rounded-2xl border border-amber-200/60 bg-amber-50 p-4 text-sm text-amber-800 dark:border-amber-900/40 dark:bg-amber-950/30 dark:text-amber-200">
          Your tutor profile is pending admin approval. Students can't book sessions with you until
          an admin reviews your profile.
        </div>
      )}

      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <KpiCard
          icon={<CalendarClock className="size-4" />}
          label="Upcoming"
          value={kpis.upcomingCount}
        />
        <KpiCard
          icon={<CalendarClock className="size-4" />}
          label="Sessions today"
          value={kpis.todayCount}
        />
        <KpiCard
          icon={<CalendarClock className="size-4" />}
          label="This week"
          value={kpis.weekCount}
        />
        <KpiCard
          icon={<CalendarClock className="size-4" />}
          label="This month"
          value={kpis.monthCount}
        />
      </div>

      <div className="grid grid-cols-2 gap-3 lg:grid-cols-4">
        <KpiCard
          icon={<CheckCheck className="size-4" />}
          label="Completed lifetime"
          value={kpis.completedTotal}
        />
        <KpiCard
          icon={<Star className="size-4" />}
          label="Avg rating"
          value={kpis.ratingAvg > 0 ? kpis.ratingAvg.toFixed(1) : '—'}
          hint={`${kpis.reviewsCount} review${kpis.reviewsCount === 1 ? '' : 's'}`}
        />
        <KpiCard
          icon={<TrendingUp className="size-4" />}
          label="Revenue this week"
          value={`${formatVnd(kpis.revenueWeekVnd)}₫`}
          hint="Completed only"
        />
        <KpiCard
          icon={<TrendingUp className="size-4" />}
          label="Revenue this month"
          value={`${formatVnd(kpis.revenueMonthVnd)}₫`}
          hint="Completed only"
        />
      </div>

      <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
        <h2 className="mb-3 text-sm font-semibold text-slate-900 dark:text-slate-100">
          Next 5 upcoming sessions
        </h2>
        {upcoming.length === 0 ? (
          <p className="text-sm text-slate-500 dark:text-slate-400">
            Nothing on the calendar yet. Add availability so students can book you.
          </p>
        ) : (
          <ul className="divide-y divide-slate-100 dark:divide-slate-800">
            {upcoming.map((s) => (
              <li key={s.id} className="flex items-center justify-between gap-4 py-3">
                <div className="min-w-0">
                  <p className="truncate text-sm font-semibold text-slate-800 dark:text-slate-100">
                    {s.student_name || 'Student'}
                  </p>
                  {s.course_title_en && (
                    <p className="truncate text-xs text-slate-500 dark:text-slate-400">
                      {s.course_title_en}
                    </p>
                  )}
                </div>
                <div className="shrink-0 text-right">
                  <p className="text-sm font-medium text-slate-700 dark:text-slate-200">
                    {formatWhen(s.scheduled_at)}
                  </p>
                  <span
                    className={`inline-flex items-center gap-1 text-[10px] font-bold tracking-wide uppercase ${
                      s.status === 'pending'
                        ? 'text-amber-600 dark:text-amber-400'
                        : 'text-emerald-600 dark:text-emerald-400'
                    }`}
                  >
                    {s.status === 'pending' && <MessageSquareText className="size-3" />}
                    {s.status}
                  </span>
                </div>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

function KpiCard({
  icon,
  label,
  value,
  hint,
}: {
  icon: React.ReactNode;
  label: string;
  value: string | number;
  hint?: string;
}) {
  return (
    <article className="rounded-2xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-900">
      <div className="flex items-center gap-2 text-[11px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
        {icon}
        {label}
      </div>
      <p className="mt-1.5 font-serif text-2xl font-bold text-slate-900 dark:text-slate-100">
        {value}
      </p>
      {hint && <p className="mt-0.5 text-[11px] text-slate-500 dark:text-slate-400">{hint}</p>}
    </article>
  );
}
