'use client';

import { useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import Image from 'next/image';
import { toast } from 'sonner';
import { BookOpen, ExternalLink, Eye, EyeOff, Loader2, Search, Trash2 } from 'lucide-react';
import type { ColumnDef } from '@tanstack/react-table';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { FilterPill } from '@/components/shared/FilterPill';
import { DataTable } from '@/components/ui/data-table';
import { Button, buttonVariants } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
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
import { formatVnd } from '@/lib/format';
import type { CourseForAdmin } from '@/lib/supabase';
import type { Level } from '@/lib/types';
import { errorMessage } from '@/lib/errors';

type PublishFilter = 'all' | 'published' | 'draft';

const LEVELS: (Level | 'all')[] = ['all', 'A1', 'A2', 'B1', 'B2', 'C1', 'C2'];

/**
 * CoursesSection — admin moderation queue for every course on the platform,
 * including drafts. Two actions:
 *   • publish/unpublish toggle (instant, no confirmation — reversible)
 *   • delete (AlertDialog — hard, not reversible)
 *
 * Editing course content lives in the tutor's own dashboard. Admin's job is
 * moderation + release control, not authoring.
 */
export function CoursesSection({ courses }: { courses: CourseForAdmin[] }) {
  const router = useRouter();
  const [publish, setPublish] = useState<PublishFilter>('all');
  const [level, setLevel] = useState<Level | 'all'>('all');
  const [search, setSearch] = useState('');
  const [pending, setPending] = useState<string | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<CourseForAdmin | null>(null);

  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return courses.filter((c) => {
      if (publish === 'published' && !c.is_published) return false;
      if (publish === 'draft' && c.is_published) return false;
      if (level !== 'all' && c.level !== level) return false;
      if (!needle) return true;
      return (
        c.title_en.toLowerCase().includes(needle) ||
        (c.title_vn || '').toLowerCase().includes(needle) ||
        c.tutor_name.toLowerCase().includes(needle) ||
        (c.category || '').toLowerCase().includes(needle)
      );
    });
  }, [courses, publish, level, search]);

  const togglePublish = async (c: CourseForAdmin) => {
    setPending(c.id);
    try {
      const res = await fetch(`/api/admin/courses/${c.id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({ is_published: !c.is_published }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error || 'update_failed');
      toast.success(c.is_published ? 'Course unpublished' : 'Course published');
      router.refresh();
    } catch (err) {
      toast.error('Could not update course', { description: errorMessage(err) });
    } finally {
      setPending(null);
    }
  };

  const remove = async (c: CourseForAdmin) => {
    setPending(c.id);
    try {
      const res = await fetch(`/api/admin/courses/${c.id}`, {
        method: 'DELETE',
        headers: await getAuthHeaders(),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) throw new Error(data?.error || 'delete_failed');
      toast.success('Course removed');
      router.refresh();
    } catch (err) {
      toast.error('Could not delete course', { description: errorMessage(err) });
    } finally {
      setPending(null);
      setDeleteTarget(null);
    }
  };

  const publishedCount = courses.filter((c) => c.is_published).length;

  const columns = useMemo<ColumnDef<CourseForAdmin>[]>(
    () => [
      {
        id: 'course',
        accessorKey: 'title_en',
        header: 'Course',
        meta: { cellClassName: 'max-w-[280px]' },
        cell: ({ row }) => {
          const c = row.original;
          return (
            <div className="flex items-center gap-2.5">
              {c.image ? (
                <Image
                  src={c.image}
                  alt=""
                  width={40}
                  height={40}
                  className="size-10 shrink-0 rounded-md object-cover"
                />
              ) : (
                <div className="flex size-10 shrink-0 items-center justify-center rounded-md bg-slate-100 dark:bg-slate-800">
                  <BookOpen className="size-4 text-slate-400" />
                </div>
              )}
              <div className="min-w-0">
                <p className="truncate font-medium text-slate-800 dark:text-slate-100">
                  {c.title_en}
                </p>
                {c.title_vn && (
                  <p className="truncate text-[11px] text-slate-500 dark:text-slate-400">
                    {c.title_vn}
                  </p>
                )}
              </div>
            </div>
          );
        },
      },
      {
        id: 'tutor',
        accessorKey: 'tutor_name',
        header: 'Tutor',
        meta: { cellClassName: 'text-slate-700 dark:text-slate-200', hideBelow: 'lg' },
        cell: ({ row }) => <>{row.original.tutor_name || '—'}</>,
      },
      {
        id: 'level',
        accessorKey: 'level',
        header: 'Level · Category',
        enableSorting: false,
        meta: {
          cellClassName: 'text-xs whitespace-nowrap text-slate-500 dark:text-slate-400',
          hideBelow: 'lg',
        },
        cell: ({ row }) => (
          <>
            <span className="font-semibold text-slate-700 dark:text-slate-200">
              {row.original.level || '—'}
            </span>{' '}
            · {row.original.category || '—'}
          </>
        ),
      },
      {
        id: 'price',
        accessorKey: 'price_vnd',
        header: 'Price',
        sortDescFirst: true,
        meta: {
          cellClassName:
            'font-semibold whitespace-nowrap text-slate-800 tabular-nums dark:text-slate-100',
        },
        cell: ({ row }) => <>{formatVnd(row.original.price_vnd)}₫</>,
      },
      {
        id: 'status',
        accessorKey: 'is_published',
        header: 'Status',
        cell: ({ row }) => <StatusPill isPublished={row.original.is_published} />,
      },
      {
        id: 'actions',
        header: 'Actions',
        enableSorting: false,
        meta: { headClassName: 'w-56' },
        cell: ({ row }) => (
          <RowActions
            course={row.original}
            pending={pending === row.original.id}
            onToggle={() => togglePublish(row.original)}
            onDelete={() => setDeleteTarget(row.original)}
          />
        ),
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [pending],
  );

  return (
    <div className="space-y-4">
      <header>
        <h1 className="type-page text-slate-900 dark:text-slate-100">Courses</h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          <span className="tabular-nums">{publishedCount}</span> published ·{' '}
          <span className="tabular-nums">{courses.length - publishedCount}</span> draft ·{' '}
          <span className="tabular-nums">{courses.length}</span> total.
        </p>
      </header>

      <div className="flex flex-wrap gap-2">
        <div className="relative min-w-[200px] flex-1">
          <Search className="pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-slate-400" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search course, tutor, category…"
            className="h-9 pl-8"
          />
        </div>
        <div className="flex flex-wrap gap-1.5">
          {(['all', 'published', 'draft'] as PublishFilter[]).map((p) => (
            <FilterPill
              key={p}
              active={publish === p}
              onClick={() => setPublish(p)}
              label={p}
              size="md"
            />
          ))}
        </div>
      </div>

      <div className="flex flex-wrap gap-1.5">
        {LEVELS.map((l) => (
          <FilterPill key={l} active={level === l} onClick={() => setLevel(l)} label={l} />
        ))}
      </div>

      <DataTable
        data={filtered}
        columns={columns}
        getRowId={(c) => c.id}
        emptyState={
          <div className="rounded-2xl border border-dashed border-slate-200 py-16 text-center dark:border-slate-800">
            <BookOpen className="mx-auto mb-2 size-6 text-slate-300 dark:text-slate-600" />
            <p className="text-sm text-slate-500 dark:text-slate-400">
              No courses match this filter.
            </p>
          </div>
        }
        mobileCard={(row) => {
          const c = row.original;
          return (
            <div className="space-y-3 p-4">
              <div className="flex items-start gap-3">
                {c.image ? (
                  <Image
                    src={c.image}
                    alt=""
                    width={48}
                    height={48}
                    className="size-12 shrink-0 rounded-md object-cover"
                  />
                ) : (
                  <div className="flex size-12 shrink-0 items-center justify-center rounded-md bg-slate-100 dark:bg-slate-800">
                    <BookOpen className="size-5 text-slate-400" />
                  </div>
                )}
                <div className="min-w-0 flex-1">
                  <p className="truncate font-medium text-slate-800 dark:text-slate-100">
                    {c.title_en}
                  </p>
                  <p className="truncate text-xs text-slate-500 dark:text-slate-400">
                    by {c.tutor_name || '—'}
                  </p>
                  <p className="mt-0.5 text-[11px] text-slate-500 dark:text-slate-400">
                    <span className="font-semibold text-slate-700 dark:text-slate-200">
                      {c.level || '—'}
                    </span>{' '}
                    · {c.category || '—'} · {formatVnd(c.price_vnd)}₫
                  </p>
                </div>
                <StatusPill isPublished={c.is_published} />
              </div>
              <RowActions
                course={c}
                pending={pending === c.id}
                onToggle={() => togglePublish(c)}
                onDelete={() => setDeleteTarget(c)}
              />
            </div>
          );
        }}
      />

      <AlertDialog
        open={!!deleteTarget}
        onOpenChange={(v) => {
          if (!v) setDeleteTarget(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete &ldquo;{deleteTarget?.title_en}&rdquo;?</AlertDialogTitle>
            <AlertDialogDescription>
              This is permanent. Past sessions that used this course keep their history — the
              foreign key drops to NULL. Enrolled students lose access to any materials that were on
              it. Unpublishing is usually safer than deleting.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => deleteTarget && remove(deleteTarget)}
              className="bg-destructive hover:bg-destructive/90 text-white"
            >
              Delete course
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}

function StatusPill({ isPublished }: { isPublished: boolean }) {
  return (
    <Badge
      className={cn(
        'rounded-full border-0 px-2 text-[10px] font-bold tracking-wide uppercase',
        isPublished
          ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
          : 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-300',
      )}
    >
      {isPublished ? 'published' : 'draft'}
    </Badge>
  );
}

function RowActions({
  course,
  pending,
  onToggle,
  onDelete,
}: {
  course: CourseForAdmin;
  pending: boolean;
  onToggle: () => void;
  onDelete: () => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <a
        href={`/courses/${course.id}`}
        target="_blank"
        rel="noopener noreferrer"
        aria-label={`Open ${course.title_en} on the public site`}
        className={buttonVariants({ variant: 'outline', size: 'sm' })}
      >
        <ExternalLink className="size-3.5" />
        View
      </a>
      <Button
        variant={course.is_published ? 'outline' : 'default'}
        size="sm"
        onClick={onToggle}
        disabled={pending}
        aria-label={course.is_published ? 'Unpublish course' : 'Publish course'}
      >
        {pending ? (
          <Loader2 className="size-3.5 animate-spin" />
        ) : course.is_published ? (
          <>
            <EyeOff className="size-3.5" />
            Unpublish
          </>
        ) : (
          <>
            <Eye className="size-3.5" />
            Publish
          </>
        )}
      </Button>
      <Button
        variant="destructive"
        size="sm"
        onClick={onDelete}
        disabled={pending}
        aria-label={`Delete ${course.title_en}`}
      >
        <Trash2 className="size-3.5" />
      </Button>
    </div>
  );
}
