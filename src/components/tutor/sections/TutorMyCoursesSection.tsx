'use client';

import { useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import Image from 'next/image';
import { toast } from 'sonner';
import { BookOpen, Eye, EyeOff, Loader2, Search, ExternalLink } from 'lucide-react';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { FilterPill } from '@/components/shared/FilterPill';
import { ResponsiveTable, TableView, CardView } from '@/components/ui/responsive-table';
import { Button } from '@/components/ui/button';
import { formatVnd } from '@/lib/format';
import type { Course } from '@/lib/types';
import { errorMessage } from '@/lib/errors';

type PublishFilter = 'all' | 'published' | 'draft';

/**
 * MyCoursesSection — read + publish-toggle only. Course creation and
 * editing intentionally aren't here; the tutor's course editor is a larger
 * form UI that gets its own page (`/tutor/courses/[id]/edit`) in a follow-
 * up. This section covers the frequent daily action — "flip a draft to
 * published" — without pushing the tutor through a modal.
 */
export function TutorMyCoursesSection({ courses }: { courses: Course[] }) {
  const router = useRouter();
  const [publish, setPublish] = useState<PublishFilter>('all');
  const [search, setSearch] = useState('');
  const [pending, setPending] = useState<string | null>(null);

  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return courses.filter((c) => {
      if (publish === 'published' && !c.is_published) return false;
      if (publish === 'draft' && c.is_published) return false;
      if (!needle) return true;
      return (
        c.title_en.toLowerCase().includes(needle) ||
        (c.title_vn || '').toLowerCase().includes(needle) ||
        (c.category || '').toLowerCase().includes(needle)
      );
    });
  }, [courses, publish, search]);

  const togglePublish = async (c: Course) => {
    setPending(c.id);
    try {
      // Tutors use the admin PATCH endpoint too — it's admin-gated today, so
      // this call fails 403 for non-admins. Once we add /api/tutor/courses/
      // the caller flips over; for now the section is read-only for tutors
      // and the button flags a friendly toast if the server refuses.
      const res = await fetch(`/api/admin/courses/${c.id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({ is_published: !c.is_published }),
      });
      const data = await res.json();
      if (!res.ok) {
        if (res.status === 403) {
          toast.info('Contact an admin to publish this course', {
            description: 'Tutor-side publishing is coming — for now, admin toggles it.',
          });
          return;
        }
        throw new Error(data?.error || 'update_failed');
      }
      toast.success(c.is_published ? 'Course unpublished' : 'Course published');
      router.refresh();
    } catch (err) {
      toast.error('Could not update course', { description: errorMessage(err) });
    } finally {
      setPending(null);
    }
  };

  const publishedCount = courses.filter((c) => c.is_published).length;

  return (
    <div className="space-y-4">
      <header>
        <h1 className="type-page text-slate-900 dark:text-slate-100">My courses</h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          <span className="tabular-nums">{publishedCount}</span> published ·{' '}
          <span className="tabular-nums">{courses.length - publishedCount}</span> draft. Editing
          course content is on its way — for now, ping admin if a title needs a change.
        </p>
      </header>

      <div className="flex flex-wrap gap-2">
        <div className="relative min-w-[200px] flex-1">
          <Search className="absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2 text-slate-400" />
          <input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search my courses…"
            className="focus:border-brand/50 h-11 w-full rounded-lg border border-slate-200 bg-white pr-3 pl-8 text-base focus:outline-none md:text-sm dark:border-slate-800 dark:bg-slate-900"
          />
        </div>
        <div className="flex gap-1.5">
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

      {filtered.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-200 py-16 text-center dark:border-slate-800">
          <BookOpen className="mx-auto mb-2 size-6 text-slate-300 dark:text-slate-600" />
          <p className="text-sm text-slate-500 dark:text-slate-400">
            {courses.length === 0
              ? 'No courses yet. Admin will seed one for you shortly.'
              : 'No courses match this filter.'}
          </p>
        </div>
      ) : (
        <ResponsiveTable className="rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900">
          <TableView>
            <table className="min-w-full text-sm">
              <thead className="bg-slate-50 dark:bg-slate-800/40">
                <tr className="text-left text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                  <th className="px-4 py-3">Course</th>
                  <th className="px-4 py-3">Level · Category</th>
                  <th className="px-4 py-3">Price</th>
                  <th className="px-4 py-3">Status</th>
                  <th className="w-48 px-4 py-3">Actions</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100 dark:divide-slate-800">
                {filtered.map((c) => (
                  <tr key={c.id} className="hover:bg-slate-50 dark:hover:bg-slate-800/40">
                    <td className="max-w-[280px] px-4 py-3">
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
                    </td>
                    <td className="px-4 py-3 text-xs whitespace-nowrap text-slate-500 dark:text-slate-400">
                      <span className="font-semibold text-slate-700 dark:text-slate-200">
                        {c.level || '—'}
                      </span>{' '}
                      · {c.category || '—'}
                    </td>
                    <td className="px-4 py-3 font-semibold whitespace-nowrap text-slate-800 tabular-nums dark:text-slate-100">
                      {formatVnd(c.price_vnd)}₫
                    </td>
                    <td className="px-4 py-3">
                      <StatusPill isPublished={c.is_published} />
                    </td>
                    <td className="px-4 py-3">
                      <RowActions
                        course={c}
                        pending={pending === c.id}
                        onToggle={() => togglePublish(c)}
                      />
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </TableView>

          <CardView>
            <ul className="divide-y divide-slate-100 dark:divide-slate-800">
              {filtered.map((c) => (
                <li key={c.id} className="space-y-3 p-4">
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
                  />
                </li>
              ))}
            </ul>
          </CardView>
        </ResponsiveTable>
      )}
    </div>
  );
}

function StatusPill({ isPublished }: { isPublished: boolean }) {
  const label = isPublished ? 'published' : 'draft';
  const tone = isPublished
    ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
    : 'bg-slate-100 text-slate-600 dark:bg-slate-800 dark:text-slate-300';
  return (
    <span
      className={`inline-flex h-5 items-center rounded-full px-2 text-[10px] font-bold tracking-wide uppercase ${tone}`}
    >
      {label}
    </span>
  );
}

function RowActions({
  course,
  pending,
  onToggle,
}: {
  course: Course;
  pending: boolean;
  onToggle: () => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <a
        href={`/courses/${course.id}`}
        target="_blank"
        rel="noopener noreferrer"
        aria-label={`Preview ${course.title_en}`}
        className="focus-ring hover:border-brand/40 hover:text-brand inline-flex h-11 items-center gap-1.5 rounded-lg border border-slate-200 px-3 text-xs font-semibold text-slate-600 transition-colors dark:border-slate-700 dark:text-slate-300"
      >
        <ExternalLink className="size-3.5" />
        Preview
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
    </div>
  );
}
