'use client';

import Link from 'next/link';
import Image from 'next/image';
import { motion } from 'motion/react';
import { CalendarDays, Clock, Users, ArrowUpRight } from 'lucide-react';
import { formatWhen } from '@/lib/time';
import type { SessionListItem } from '@/lib/supabase';
import { Badge } from '@/components/ui/badge';
import { cn } from '@/lib/utils';

interface Props {
  session: SessionListItem;
  index?: number;
}

// Free-talk group session card — used on the homepage preview + the /sessions
// listing. Whole card is a Link so the tap target agrees with the a11y tree.
// A separate "Book" flow lives inside the detail page, so no nested button
// here; the card just signals "there's a room, and here's who's in it".
export function GroupSessionCard({ session, index = 0 }: Props) {
  const when = formatWhen(session.scheduled_at, 'card-no-year');
  const isLive = session.status === 'live';
  const isFull = session.participant_count >= session.capacity;
  const capacityPct = Math.min(
    100,
    Math.round((session.participant_count / session.capacity) * 100),
  );

  const levelLabel = (session.level || 'all').toUpperCase().replace('_', '–');
  const topic = session.topic_en || session.topic_vn || 'Free talk session';

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: Math.min(index * 0.04, 0.32), duration: 0.35 }}
    >
      <Link
        href={`/sessions/${session.id}`}
        aria-label={`Group session: ${topic}`}
        className="focus-ring group relative flex h-full flex-col gap-4 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition-all hover:-translate-y-0.5 hover:shadow-md dark:border-slate-800 dark:bg-slate-900"
      >
        <div className="flex items-start gap-3">
          <div className="bg-brand/10 text-brand dark:bg-brand/15 flex size-11 shrink-0 items-center justify-center rounded-xl text-xl">
            {session.cover_emoji || '💬'}
          </div>
          <div className="min-w-0 flex-1">
            <h3 className="line-clamp-2 text-sm leading-snug font-semibold text-slate-900 dark:text-slate-100">
              {topic}
            </h3>
            <div className="mt-1 flex items-center gap-2 text-[11px] text-slate-500 dark:text-slate-400">
              <Badge className="rounded-full border-0 bg-slate-100 px-1.5 text-[10px] font-semibold tracking-wider text-slate-600 uppercase dark:bg-slate-800 dark:text-slate-300">
                {levelLabel}
              </Badge>
              {isLive && (
                <Badge className="inline-flex h-5 items-center gap-1 rounded-full border-0 bg-emerald-100 px-1.5 text-[10px] font-semibold tracking-wider text-emerald-700 uppercase dark:bg-emerald-950/40 dark:text-emerald-300">
                  <span
                    className="size-1.5 animate-pulse rounded-full bg-emerald-500"
                    aria-hidden
                  />
                  Live
                </Badge>
              )}
            </div>
          </div>
        </div>

        {session.description && (
          <p className="line-clamp-2 text-xs leading-relaxed text-slate-600 dark:text-slate-300">
            {session.description}
          </p>
        )}

        <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-slate-500 dark:text-slate-400">
          <span className="flex items-center gap-1">
            <CalendarDays className="size-3" />
            {when.date}
          </span>
          <span className="flex items-center gap-1">
            <Clock className="size-3" />
            {when.time} · {session.duration_min}m
          </span>
        </div>

        <div className="mt-auto space-y-2 border-t border-slate-100 pt-3 dark:border-slate-800">
          <div className="flex items-center justify-between text-[11px] font-semibold text-slate-500 dark:text-slate-400">
            <span className="flex items-center gap-1">
              {session.tutor_avatar ? (
                <Image
                  src={session.tutor_avatar}
                  alt=""
                  width={20}
                  height={20}
                  className="size-5 rounded-full object-cover"
                />
              ) : (
                <Users className="size-3" />
              )}
              {session.tutor_name || 'Community host'}
            </span>
            <span className="text-slate-700 tabular-nums dark:text-slate-200">
              {session.participant_count}/{session.capacity}
            </span>
          </div>
          <div className="relative h-1.5 overflow-hidden rounded-full bg-slate-100 dark:bg-slate-800">
            <div
              className={`absolute inset-y-0 left-0 rounded-full transition-all ${
                isFull ? 'bg-amber-500' : 'bg-brand'
              }`}
              style={{ width: `${capacityPct}%` }}
              aria-hidden
            />
          </div>
          <div className="flex items-center justify-between pt-1">
            <span
              className={`text-[11px] font-semibold ${
                isFull ? 'text-amber-600 dark:text-amber-400' : 'text-brand dark:text-accent-warm'
              }`}
            >
              {isFull ? 'Room full — waitlist' : isLive ? 'Join now' : 'Reserve a seat'}
            </span>
            <span
              aria-hidden
              className="group-hover:bg-brand flex size-7 items-center justify-center rounded-full bg-slate-100 text-slate-500 transition-colors group-hover:text-white dark:bg-slate-800 dark:text-slate-400"
            >
              <ArrowUpRight className="size-3.5" />
            </span>
          </div>
        </div>
      </Link>
    </motion.div>
  );
}
