'use client';

import { useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import Image from 'next/image';
import { toast } from 'sonner';
import { Loader2, Search, Star, Trash2 } from 'lucide-react';
import type { ColumnDef } from '@tanstack/react-table';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { FilterPill } from '@/components/shared/FilterPill';
import { DataTable } from '@/components/ui/data-table';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { initials } from '@/lib/format';
import type { ReviewForAdmin } from '@/lib/supabase';
import { errorMessage } from '@/lib/errors';

type RatingFilter = 'all' | '5' | '4' | '3' | '2' | '1';

const formatDate = (iso: string) =>
  new Date(iso).toLocaleDateString(undefined, {
    day: '2-digit',
    month: 'short',
    year: 'numeric',
  });

/**
 * ReviewsSection — admin moderation queue for every review on the platform.
 * The default sort is newest first, matching the incoming-content mental
 * model. Delete is a hard delete; the DB trigger recomputes tutor rating
 * averages so the public profile updates the moment we refresh.
 */
export function ReviewsSection({ reviews }: { reviews: ReviewForAdmin[] }) {
  const router = useRouter();
  const [search, setSearch] = useState('');
  const [rating, setRating] = useState<RatingFilter>('all');
  const [pending, setPending] = useState<string | null>(null);
  const [confirmTarget, setConfirmTarget] = useState<ReviewForAdmin | null>(null);

  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return reviews.filter((r) => {
      if (rating !== 'all' && r.rating !== Number(rating)) return false;
      if (!needle) return true;
      return (
        r.tutor_name.toLowerCase().includes(needle) ||
        r.student_name.toLowerCase().includes(needle) ||
        (r.comment || '').toLowerCase().includes(needle)
      );
    });
  }, [reviews, rating, search]);

  const remove = async (id: string) => {
    setPending(id);
    try {
      const res = await fetch(`/api/admin/reviews/${id}`, {
        method: 'DELETE',
        headers: await getAuthHeaders(),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(data?.error || 'delete_failed');
      toast.success('Review removed');
      router.refresh();
    } catch (err) {
      toast.error('Could not remove review', { description: errorMessage(err) });
    } finally {
      setPending(null);
      setConfirmTarget(null);
    }
  };

  const ratingCounts = useMemo(() => {
    const map = new Map<number, number>();
    for (const r of reviews) map.set(r.rating, (map.get(r.rating) || 0) + 1);
    return map;
  }, [reviews]);

  const columns = useMemo<ColumnDef<ReviewForAdmin>[]>(
    () => [
      {
        id: 'rating',
        accessorKey: 'rating',
        header: 'Rating',
        sortDescFirst: true,
        cell: ({ row }) => <StarRating value={row.original.rating} />,
      },
      {
        id: 'people',
        accessorKey: 'student_name',
        header: 'Student → Tutor',
        cell: ({ row }) => {
          const r = row.original;
          return (
            <div className="flex items-center gap-2">
              <MiniAvatar name={r.student_name} avatar={r.student_avatar} />
              <div className="text-xs">
                <p className="font-medium text-slate-800 dark:text-slate-100">
                  {r.student_name || '—'}
                </p>
                <p className="text-slate-500 dark:text-slate-400">→ {r.tutor_name || '—'}</p>
              </div>
            </div>
          );
        },
      },
      {
        id: 'comment',
        accessorKey: 'comment',
        header: 'Comment',
        enableSorting: false,
        meta: {
          cellClassName: 'max-w-[360px] text-xs text-slate-600 dark:text-slate-300',
          hideBelow: 'lg',
        },
        cell: ({ row }) =>
          row.original.comment ? (
            <p className="line-clamp-2 leading-relaxed">{row.original.comment}</p>
          ) : (
            <span className="text-slate-400 italic">no comment</span>
          ),
      },
      {
        id: 'when',
        accessorKey: 'created_at',
        header: 'When',
        sortDescFirst: true,
        meta: {
          cellClassName:
            'text-xs whitespace-nowrap text-slate-500 tabular-nums dark:text-slate-400',
        },
        cell: ({ row }) => <>{formatDate(row.original.created_at)}</>,
      },
      {
        id: 'actions',
        header: '',
        enableSorting: false,
        meta: { headClassName: 'w-24', cellClassName: 'text-right' },
        cell: ({ row }) => (
          <Button
            variant="destructive"
            size="sm"
            onClick={() => setConfirmTarget(row.original)}
            disabled={pending === row.original.id}
            aria-label={`Delete review by ${row.original.student_name}`}
          >
            {pending === row.original.id ? (
              <Loader2 className="size-3.5 animate-spin" />
            ) : (
              <Trash2 className="size-3.5" />
            )}
          </Button>
        ),
      },
    ],
    [pending],
  );

  return (
    <div className="space-y-4">
      <header>
        <h1 className="type-page text-slate-900 dark:text-slate-100">Reviews</h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          <span className="tabular-nums">{filtered.length}</span> of{' '}
          <span className="tabular-nums">{reviews.length}</span> reviews. Deleting a review is
          immediate — tutor rating recomputes automatically.
        </p>
      </header>

      <div className="flex flex-wrap gap-2">
        <div className="relative min-w-[200px] flex-1">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-slate-400" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search tutor, student, or comment…"
            className="h-9 pl-8"
          />
        </div>
        <div className="flex gap-1.5">
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
                  <span className="text-[10px] opacity-70">
                    ({ratingCounts.get(Number(r)) || 0})
                  </span>
                </span>
              }
              size="md"
            />
          ))}
        </div>
      </div>

      <DataTable
        data={filtered}
        columns={columns}
        getRowId={(r) => r.id}
        initialSort={[{ id: 'when', desc: true }]}
        emptyState={
          <div className="rounded-2xl border border-dashed border-slate-200 py-16 text-center dark:border-slate-800">
            <p className="text-sm text-slate-500 dark:text-slate-400">
              No reviews match those filters.
            </p>
          </div>
        }
        mobileCard={(row) => {
          const r = row.original;
          return (
            <div className="space-y-2 p-4">
              <div className="flex items-start justify-between gap-2">
                <StarRating value={r.rating} />
                <span className="text-[10px] whitespace-nowrap text-slate-500 tabular-nums dark:text-slate-400">
                  {formatDate(r.created_at)}
                </span>
              </div>
              <div className="flex items-center gap-2">
                <MiniAvatar name={r.student_name} avatar={r.student_avatar} />
                <div className="min-w-0 text-xs">
                  <p className="truncate font-medium text-slate-800 dark:text-slate-100">
                    {r.student_name || '—'}
                  </p>
                  <p className="truncate text-slate-500 dark:text-slate-400">
                    → {r.tutor_name || '—'}
                  </p>
                </div>
              </div>
              {r.comment ? (
                <p className="text-sm leading-relaxed text-slate-700 dark:text-slate-200">
                  {r.comment}
                </p>
              ) : (
                <p className="text-xs text-slate-400 italic">no comment</p>
              )}
              <div className="flex justify-end pt-1">
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => setConfirmTarget(r)}
                  disabled={pending === r.id}
                >
                  {pending === r.id ? (
                    <Loader2 className="size-3.5 animate-spin" />
                  ) : (
                    <>
                      <Trash2 className="size-3.5" /> Delete
                    </>
                  )}
                </Button>
              </div>
            </div>
          );
        }}
      />

      <AlertDialog
        open={!!confirmTarget}
        onOpenChange={(v) => {
          if (!v) setConfirmTarget(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete this review?</AlertDialogTitle>
            <AlertDialogDescription>
              This is permanent. The tutor&apos;s rating average will recompute automatically.
              Consider contacting the student first if the review is a mistake or a policy
              edge-case.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => confirmTarget && remove(confirmTarget.id)}
              className="bg-destructive hover:bg-destructive/90 text-white"
            >
              Delete review
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function StarRating({ value }: { value: number }) {
  return (
    <div className="flex items-center gap-0.5 text-amber-500" aria-label={`${value} stars`}>
      {Array.from({ length: 5 }).map((_, i) => (
        <Star
          key={i}
          className={`size-3.5 ${i < value ? 'fill-amber-500' : 'text-slate-300 dark:text-slate-600'}`}
        />
      ))}
    </div>
  );
}

function MiniAvatar({ name, avatar }: { name: string; avatar: string | null }) {
  if (avatar) {
    return (
      <Image
        src={avatar}
        alt=""
        width={28}
        height={28}
        className="size-7 shrink-0 rounded-full object-cover"
      />
    );
  }
  return (
    <div className="bg-brand flex size-7 shrink-0 items-center justify-center rounded-full text-[10px] font-semibold text-white">
      {initials(name)}
    </div>
  );
}
