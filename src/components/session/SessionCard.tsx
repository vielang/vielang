'use client';

import { CalendarDays, Clock, MessageSquareText, GraduationCap } from 'lucide-react';
import { AvatarBubble } from '@/components/shared/AvatarBubble';
import { SESSION_STATUS_TONE } from '@/lib/status-tone';
import { formatWhen, formatCountdown } from '@/lib/time';
import { useTicker } from '@/hooks/use-ticker';
import { useLang } from '@/contexts';
import { SessionCardActions } from './SessionCardActions';
import type { SessionListItem } from '@/lib/supabase';
import type { SessionStatus } from '@/lib/types';

interface Props {
  session: SessionListItem;
  highlight?: boolean;
  viewerRole: 'user' | 'tutor';
  reviewed?: boolean;
  onChanged: () => void;
}

const STATUS_LABEL: Record<SessionStatus, { vn: string; en: string }> = {
  pending: { vn: 'Chờ xác nhận', en: 'Pending' },
  confirmed: { vn: 'Đã xác nhận', en: 'Confirmed' },
  live: { vn: 'Đang diễn ra', en: 'Live' },
  completed: { vn: 'Hoàn thành', en: 'Completed' },
  cancelled: { vn: 'Đã huỷ', en: 'Cancelled' },
  no_show: { vn: 'Vắng mặt', en: 'No show' },
};

const COPY = {
  VN: {
    with: (n: string) => `Với ${n}`,
    unknownTutor: 'Giáo viên chưa rõ',
    unknownStudent: 'Học viên chưa rõ',
    liveEnds: (t: string) => `Live · còn ${t}`,
    startsIn: (t: string) => `Bắt đầu sau ${t}`,
  },
  EN: {
    with: (n: string) => `With ${n}`,
    unknownTutor: 'Unknown tutor',
    unknownStudent: 'Unknown student',
    liveEnds: (t: string) => `Live · ends in ${t}`,
    startsIn: (t: string) => `Starts in ${t}`,
  },
} as const;

/**
 * Presentational shell for one session in the /my-sessions or tutor list.
 * Renders avatar, headline, when, notes, live/countdown pills; delegates all
 * mutations (join / review / cancel / reschedule / materials) to
 * `<SessionCardActions>`.
 *
 * The shared 15 s tick is created here so the pill and the action row read
 * from a single clock — cheaper and avoids visual drift between them.
 */
export function SessionCard({
  session,
  highlight,
  viewerRole,
  reviewed = false,
  onChanged,
}: Props) {
  const now = useTicker();
  const { lang } = useLang();
  const lc = COPY[lang];

  const fallback = viewerRole === 'user' ? lc.unknownTutor : lc.unknownStudent;
  const otherName = (viewerRole === 'user' ? session.tutor_name : session.student_name) || fallback;
  const otherAvatar = viewerRole === 'user' ? session.tutor_avatar : null;
  const courseTitle =
    (lang === 'VN' ? session.course_title_vn : session.course_title_en) ??
    session.course_title_en ??
    session.course_title_vn;
  const when = formatWhen(session.scheduled_at);
  const statusLabel = STATUS_LABEL[session.status][lang === 'VN' ? 'vn' : 'en'];

  const start = new Date(session.scheduled_at).getTime();
  const end = start + session.duration_min * 60_000;
  const isActiveStatus =
    session.status === 'confirmed' || session.status === 'live' || session.status === 'pending';
  const msUntilStart = start - now;
  const msUntilEnd = end - now;
  const showCountdown = isActiveStatus && msUntilStart > 0;
  const isLive = isActiveStatus && now >= start && now <= end;

  return (
    <article
      className={`rounded-2xl border bg-white p-5 transition-all dark:bg-slate-900 ${
        highlight
          ? 'border-brand shadow-lg ring-2 ring-indigo-100 dark:ring-indigo-900/40'
          : 'border-slate-200 shadow-sm dark:border-slate-800'
      }`}
    >
      <div className="flex items-start gap-4">
        <div className="shrink-0">
          <AvatarBubble
            src={otherAvatar}
            name={otherName}
            size={56}
            ringClassName="ring-2 ring-indigo-100 dark:ring-indigo-900/40"
          />
        </div>

        <div className="min-w-0 flex-1 space-y-1.5">
          <div className="flex items-start justify-between gap-2">
            <div className="min-w-0">
              <h3 className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
                {viewerRole === 'user' ? lc.with(otherName) : otherName}
              </h3>
              {courseTitle && (
                <p className="mt-0.5 flex items-center gap-1 text-xs text-slate-500 dark:text-slate-400">
                  <GraduationCap className="size-3" />
                  {courseTitle}
                </p>
              )}
            </div>
            <span
              className={`inline-flex h-6 shrink-0 items-center rounded-full px-2.5 text-[10px] font-bold tracking-wide uppercase ${SESSION_STATUS_TONE[session.status]}`}
            >
              {statusLabel}
            </span>
          </div>

          <div className="flex flex-wrap items-center gap-x-4 gap-y-1 pt-1 text-xs text-slate-500 dark:text-slate-400">
            <span className="flex items-center gap-1">
              <CalendarDays className="size-3" />
              {when.date}
            </span>
            <span className="flex items-center gap-1">
              <Clock className="size-3" />
              {when.time} · {session.duration_min}m
            </span>
            {isLive ? (
              <span
                className="inline-flex h-5 items-center gap-1 rounded-full bg-emerald-100 px-1.5 font-semibold text-emerald-700 tabular-nums dark:bg-emerald-950/40 dark:text-emerald-300"
                aria-label={lc.liveEnds(formatCountdown(msUntilEnd))}
              >
                <span
                  className="size-1.5 rounded-full bg-emerald-500 motion-safe:animate-pulse"
                  aria-hidden
                />
                {lc.liveEnds(formatCountdown(msUntilEnd))}
              </span>
            ) : (
              showCountdown && (
                <span className="inline-flex h-5 items-center gap-1 rounded-full bg-amber-100 px-1.5 font-semibold text-amber-700 tabular-nums dark:bg-amber-950/40 dark:text-amber-300">
                  <span
                    className="size-1.5 rounded-full bg-amber-500 motion-safe:animate-pulse"
                    aria-hidden
                  />
                  {lc.startsIn(formatCountdown(msUntilStart))}
                </span>
              )
            )}
          </div>

          {session.student_notes && (
            <div className="flex items-start gap-2 pt-2 text-xs text-slate-500 dark:text-slate-400">
              <MessageSquareText className="mt-0.5 size-3 shrink-0" />
              <p className="line-clamp-2">{session.student_notes}</p>
            </div>
          )}

          <SessionCardActions
            session={session}
            viewerRole={viewerRole}
            now={now}
            reviewed={reviewed}
            otherName={otherName}
            onChanged={onChanged}
          />
        </div>
      </div>
    </article>
  );
}
