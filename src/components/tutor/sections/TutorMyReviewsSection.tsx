'use client';

import { useMemo, useState } from 'react';
import Image from 'next/image';
import { Star } from 'lucide-react';
import { FilterPill } from '@/components/shared/FilterPill';
import { initials } from '@/lib/format';
import type { ReviewWithStudent } from '@/lib/supabase';

type RatingFilter = 'all' | '5' | '4' | '3' | '2' | '1';

const formatDate = (iso: string) =>
  new Date(iso).toLocaleDateString(undefined, {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  });

/**
 * MyReviewsSection — read-only. Tutors see what students said about them.
 * Deleting a review is admin-only (moderation queue in AdminShell).
 *
 * Sorted newest-first, with a rating pill filter so a tutor can find that
 * one 3-star review among a wall of 5-star ones.
 */
export function TutorMyReviewsSection({ reviews }: { reviews: ReviewWithStudent[] }) {
  const [rating, setRating] = useState<RatingFilter>('all');

  const filtered = useMemo(() => {
    if (rating === 'all') return reviews;
    return reviews.filter((r) => r.rating === Number(rating));
  }, [reviews, rating]);

  const counts = useMemo(() => {
    const map = new Map<number, number>();
    for (const r of reviews) map.set(r.rating, (map.get(r.rating) || 0) + 1);
    return map;
  }, [reviews]);

  const avg = useMemo(() => {
    if (reviews.length === 0) return 0;
    const sum = reviews.reduce((acc, r) => acc + r.rating, 0);
    return sum / reviews.length;
  }, [reviews]);

  return (
    <div className="space-y-4">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="type-page text-slate-900 dark:text-slate-100">My reviews</h1>
          <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
            {reviews.length} total. What students said after their sessions.
          </p>
        </div>
        {reviews.length > 0 && (
          <div className="flex items-center gap-2 rounded-full bg-amber-50 px-3 py-1.5 text-sm font-semibold text-amber-700 dark:bg-amber-950/40 dark:text-amber-300">
            <Star className="size-4 fill-amber-500 text-amber-500" />
            <span className="tabular-nums">{avg.toFixed(1)}</span>
            <span className="text-xs font-medium opacity-70">avg over {reviews.length}</span>
          </div>
        )}
      </header>

      {reviews.length > 0 && (
        <div className="flex flex-wrap gap-1.5">
          <FilterPill
            active={rating === 'all'}
            onClick={() => setRating('all')}
            label="all"
            size="md"
          />
          {(['5', '4', '3', '2', '1'] as const).map((r) => (
            <FilterPill
              key={r}
              active={rating === r}
              onClick={() => setRating(r)}
              label={
                <span className="flex items-center gap-1">
                  <Star className="size-3 fill-current" />
                  {r}
                  <span className="text-[10px] opacity-70">({counts.get(Number(r)) || 0})</span>
                </span>
              }
              size="md"
            />
          ))}
        </div>
      )}

      {filtered.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-200 py-16 text-center dark:border-slate-800">
          <Star className="mx-auto mb-2 size-6 text-slate-300 dark:text-slate-600" />
          <p className="text-sm text-slate-500 dark:text-slate-400">
            {reviews.length === 0
              ? 'No reviews yet — after your first completed session, students can leave one.'
              : 'No reviews match this filter.'}
          </p>
        </div>
      ) : (
        <ul className="grid grid-cols-1 gap-3 md:grid-cols-2">
          {filtered.map((r) => (
            <li
              key={r.id}
              className="rounded-2xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-900"
            >
              <div className="mb-2 flex items-start justify-between gap-2">
                <div className="flex items-center gap-2.5">
                  {r.student_avatar ? (
                    <Image
                      src={r.student_avatar}
                      alt=""
                      width={32}
                      height={32}
                      className="size-8 rounded-full object-cover"
                    />
                  ) : (
                    <div className="bg-brand flex size-8 items-center justify-center rounded-full text-xs font-semibold text-white">
                      {initials(r.student_name || '?')}
                    </div>
                  )}
                  <p className="text-sm font-medium text-slate-800 dark:text-slate-100">
                    {r.student_name || 'Student'}
                  </p>
                </div>
                <span className="text-[11px] whitespace-nowrap text-slate-500 tabular-nums dark:text-slate-400">
                  {formatDate(r.created_at)}
                </span>
              </div>
              <div className="mb-2 flex items-center gap-0.5">
                {Array.from({ length: 5 }).map((_, i) => (
                  <Star
                    key={i}
                    className={`size-3.5 ${
                      i < r.rating
                        ? 'fill-amber-500 text-amber-500'
                        : 'text-slate-200 dark:text-slate-600'
                    }`}
                  />
                ))}
              </div>
              {r.comment ? (
                <p className="text-sm leading-relaxed text-slate-700 dark:text-slate-200">
                  {r.comment}
                </p>
              ) : (
                <p className="text-xs text-slate-400 italic">no comment</p>
              )}
            </li>
          ))}
        </ul>
      )}
    </div>
  );
}
