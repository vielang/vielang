'use client';

import Link from 'next/link';
import { motion } from 'motion/react';
import { Star, GraduationCap, Clock, ArrowUpRight } from 'lucide-react';
import { AvatarBubble } from '@/components/shared/AvatarBubble';
import { formatVnd } from '@/lib/format';
import type { Tutor } from '@/lib/types';

interface Props {
  tutor: Tutor;
  index?: number;
}

/**
 * Tutor card. Whole card is one Link so a11y-tree + mouse target agree.
 * "View profile" affordance is a subtle arrow — no nested button.
 * Rows for zero rating / zero sessions are hidden so brand-new tutors don't
 * look like they've been rejected by the platform.
 */
export function TutorCard({ tutor, index = 0 }: Props) {
  const { profile } = tutor;
  const hasRating = profile.rating_avg > 0;
  const hasSessions = profile.session_count > 0;

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: Math.min(index * 0.04, 0.32), duration: 0.35 }}
    >
      <Link
        href={`/tutors/${tutor.id}`}
        aria-label={`View ${tutor.name}'s profile`}
        className="focus-ring group relative flex h-full flex-col gap-4 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm transition-all hover:-translate-y-0.5 hover:shadow-md dark:border-slate-800 dark:bg-slate-900"
      >
        <div className="flex items-start gap-4">
          <div className="shrink-0">
            <AvatarBubble
              src={tutor.avatar}
              name={tutor.name}
              size={64}
              ringClassName="ring-2 ring-indigo-100 dark:ring-indigo-900/40"
            />
          </div>
          <div className="min-w-0 flex-1">
            <div className="flex items-start justify-between gap-2">
              <h3 className="truncate text-base font-semibold text-slate-900 dark:text-slate-100">
                {tutor.name}
              </h3>
              {hasRating && (
                <div className="flex shrink-0 items-center gap-1 text-amber-500">
                  <Star className="size-3.5 fill-amber-500" />
                  <span className="text-xs font-semibold text-slate-700 tabular-nums dark:text-slate-200">
                    {profile.rating_avg.toFixed(1)}
                  </span>
                </div>
              )}
            </div>
            <div className="type-muted mt-1 flex items-center gap-3">
              <span className="flex items-center gap-1">
                <GraduationCap className="size-3" />
                {profile.years_experience} yrs teaching
              </span>
              {hasSessions && (
                <span className="flex items-center gap-1">
                  <Clock className="size-3" />
                  {profile.session_count} sessions
                </span>
              )}
            </div>
          </div>
        </div>

        {tutor.bio && (
          <p className="line-clamp-2 text-xs leading-relaxed text-slate-600 dark:text-slate-300">
            {tutor.bio}
          </p>
        )}

        {profile.specialties.length > 0 && (
          <div className="flex flex-wrap gap-1.5">
            {profile.specialties.slice(0, 3).map((s) => (
              <span
                key={s}
                className="bg-brand/10 text-brand dark:bg-brand/15 rounded-full px-2 py-0.5 text-[10px] font-semibold tracking-wide uppercase dark:text-indigo-300"
              >
                {s}
              </span>
            ))}
            {profile.specialties.length > 3 && (
              <span className="type-muted self-center">+{profile.specialties.length - 3}</span>
            )}
          </div>
        )}

        {/* Footer — price + subtle "go" affordance. Price uses a plain slate
            colour so the number reads as data, not a decorative brand accent. */}
        <div className="mt-auto flex items-end justify-between border-t border-slate-100 pt-3 dark:border-slate-800">
          <div>
            <p className="type-eyebrow">From</p>
            <p className="text-base font-bold text-slate-900 tabular-nums dark:text-slate-100">
              {formatVnd(profile.hourly_rate_vnd)}
              <span className="ml-1 text-xs font-medium text-slate-500 dark:text-slate-400">
                ₫ / hour
              </span>
            </p>
          </div>
          <span
            aria-hidden
            className="group-hover:bg-brand flex size-8 items-center justify-center rounded-full bg-slate-100 text-slate-500 transition-colors group-hover:text-white dark:bg-slate-800 dark:text-slate-400"
          >
            <ArrowUpRight className="size-4" />
          </span>
        </div>
      </Link>
    </motion.div>
  );
}
