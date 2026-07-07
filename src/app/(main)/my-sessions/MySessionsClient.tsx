'use client';

import { useMemo, useState, useTransition } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import Link from 'next/link';
import { CalendarPlus, CalendarCheck2, CalendarX2 } from 'lucide-react';
import { SessionCard } from '@/components/session/SessionCard';
import { EmptyState } from '@/components/shared/EmptyState';
import { useLang } from '@/contexts';
import type { SessionListItem } from '@/lib/supabase';

type Tab = 'upcoming' | 'completed' | 'cancelled';

// Splits a session list into the three tabs the UI shows. Upcoming groups
// pending/confirmed/live so students see their next lesson without hunting
// across sub-tabs; no_show falls under cancelled since neither party wants
// to see it in the "coming up" section.
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

interface Props {
  initial: SessionListItem[];
  viewerRole: 'user' | 'tutor';
  reviewedSessionIds?: string[];
}

const TAB_LABELS: Record<Tab, { vn: string; en: string }> = {
  upcoming: { vn: 'Sắp tới', en: 'Upcoming' },
  completed: { vn: 'Đã hoàn thành', en: 'Completed' },
  cancelled: { vn: 'Đã huỷ', en: 'Cancelled' },
};

const COPY = {
  VN: {
    title: 'Buổi học của tôi',
    subtitleStudent: 'Các buổi học tiếng Anh đã đặt và đã hoàn thành.',
    subtitleTutor: 'Các buổi học bạn phụ trách.',
    book: 'Đặt buổi học',
    browseTutors: 'Xem giáo viên',
    empty: {
      upcoming: {
        studentTitle: 'Chưa có buổi học nào',
        studentDesc: 'Sẵn sàng học chưa? Xem danh sách giáo viên và đặt lịch phù hợp.',
        tutorTitle: 'Chưa có lịch dạy',
        tutorDesc: 'Các buổi học của học viên sẽ hiển thị ở đây ngay khi được đặt.',
      },
      completed: {
        title: 'Chưa hoàn thành buổi học nào',
        studentDesc: 'Khi hoàn thành buổi học, nó sẽ xuất hiện ở đây kèm theo lời nhắc đánh giá.',
        tutorDesc: 'Mỗi buổi bạn dạy xong sẽ được lưu tại đây.',
      },
      cancelled: {
        title: 'Không có buổi học đã huỷ',
        studentDesc: 'Các buổi học bị huỷ bởi bạn hoặc giáo viên sẽ hiển thị ở đây.',
        tutorDesc: 'Các buổi học bị huỷ hoặc vắng mặt sẽ hiển thị ở đây.',
      },
    },
  },
  EN: {
    title: 'My sessions',
    subtitleStudent: 'Your booked and past English lessons.',
    subtitleTutor: 'Sessions you are teaching.',
    book: 'Book a session',
    browseTutors: 'Browse tutors',
    empty: {
      upcoming: {
        studentTitle: 'No lessons booked yet',
        studentDesc: 'Ready to meet a tutor? Browse the roster and grab a slot that works for you.',
        tutorTitle: 'No sessions on the calendar',
        tutorDesc: "You'll see student bookings here as soon as they land.",
      },
      completed: {
        title: 'No completed sessions yet',
        studentDesc:
          'Once you finish a lesson it will show up here — along with a prompt to leave a review.',
        tutorDesc: 'Every session you finish teaching will appear here.',
      },
      cancelled: {
        title: 'No cancelled sessions',
        studentDesc: 'Anything you or your tutor cancel will land here for reference.',
        tutorDesc: 'Sessions you cancel — or that were marked no-show — live here.',
      },
    },
  },
} as const;

export function MySessionsClient({ initial, viewerRole, reviewedSessionIds }: Props) {
  const reviewedSet = new Set(reviewedSessionIds || []);
  const searchParams = useSearchParams();
  const router = useRouter();
  const { lang } = useLang();
  const lc = COPY[lang];
  const [, startTransition] = useTransition();
  const highlight = searchParams.get('highlight') || undefined;
  const [tab, setTab] = useState<Tab>('upcoming');

  const buckets = useMemo(() => bucketize(initial), [initial]);
  const current = buckets[tab];

  const refresh = () => startTransition(() => router.refresh());

  const tabs: { key: Tab; label: string; count: number }[] = [
    {
      key: 'upcoming',
      label: TAB_LABELS.upcoming[lang === 'VN' ? 'vn' : 'en'],
      count: buckets.upcoming.length,
    },
    {
      key: 'completed',
      label: TAB_LABELS.completed[lang === 'VN' ? 'vn' : 'en'],
      count: buckets.completed.length,
    },
    {
      key: 'cancelled',
      label: TAB_LABELS.cancelled[lang === 'VN' ? 'vn' : 'en'],
      count: buckets.cancelled.length,
    },
  ];

  return (
    <div className="space-y-6">
      <header className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 className="text-brand dark:text-accent-warm font-serif text-2xl font-bold md:text-3xl">
            {lc.title}
          </h1>
          <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
            {viewerRole === 'user' ? lc.subtitleStudent : lc.subtitleTutor}
          </p>
        </div>
        {viewerRole === 'user' && (
          <Link
            href="/tutors"
            className="bg-brand hover:bg-brand-hover inline-flex h-10 items-center gap-1.5 rounded-lg px-4 text-xs font-semibold text-white transition-colors"
          >
            <CalendarPlus className="size-4" />
            {lc.book}
          </Link>
        )}
      </header>

      <nav
        role="tablist"
        aria-label={lc.title}
        className="flex gap-1 border-b border-slate-200 dark:border-slate-800"
      >
        {tabs.map((t) => (
          <button
            key={t.key}
            type="button"
            role="tab"
            aria-selected={tab === t.key}
            aria-controls={`sessions-tab-panel-${t.key}`}
            id={`sessions-tab-${t.key}`}
            onClick={() => setTab(t.key)}
            className={`focus-ring relative h-10 px-4 text-sm font-semibold transition-colors ${
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

      <div role="tabpanel" id={`sessions-tab-panel-${tab}`} aria-labelledby={`sessions-tab-${tab}`}>
        {current.length === 0 ? (
          <div className="rounded-2xl border border-dashed border-slate-200 dark:border-slate-800">
            <EmptyState
              icon={
                tab === 'upcoming' ? (
                  <CalendarPlus className="size-6" />
                ) : tab === 'completed' ? (
                  <CalendarCheck2 className="size-6" />
                ) : (
                  <CalendarX2 className="size-6" />
                )
              }
              title={
                tab === 'upcoming'
                  ? viewerRole === 'user'
                    ? lc.empty.upcoming.studentTitle
                    : lc.empty.upcoming.tutorTitle
                  : tab === 'completed'
                    ? lc.empty.completed.title
                    : lc.empty.cancelled.title
              }
              description={
                tab === 'upcoming'
                  ? viewerRole === 'user'
                    ? lc.empty.upcoming.studentDesc
                    : lc.empty.upcoming.tutorDesc
                  : tab === 'completed'
                    ? viewerRole === 'user'
                      ? lc.empty.completed.studentDesc
                      : lc.empty.completed.tutorDesc
                    : viewerRole === 'user'
                      ? lc.empty.cancelled.studentDesc
                      : lc.empty.cancelled.tutorDesc
              }
              action={
                tab === 'upcoming' && viewerRole === 'user' ? (
                  <Link
                    href="/tutors"
                    className="focus-ring bg-brand hover:bg-brand-hover inline-flex h-9 items-center gap-1.5 rounded-lg px-4 text-xs font-semibold text-white transition-colors"
                  >
                    {lc.browseTutors}
                  </Link>
                ) : undefined
              }
            />
          </div>
        ) : (
          <div className="space-y-3">
            {current.map((s) => (
              <SessionCard
                key={s.id}
                session={s}
                highlight={s.id === highlight}
                viewerRole={viewerRole}
                reviewed={reviewedSet.has(s.id)}
                onChanged={refresh}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
