'use client';

import { useMemo, useState } from 'react';
import { CalendarClock, Search } from 'lucide-react';
import type { ColumnDef } from '@tanstack/react-table';
import { SESSION_STATUS_TONE } from '@/lib/status-tone';
import { formatWhenLine } from '@/lib/time';
import { FilterPill } from '@/components/shared/FilterPill';
import { DataTable } from '@/components/ui/data-table';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
import type { SessionListItem } from '@/lib/supabase';
import type { SessionStatus } from '@/lib/types';

const STATUSES: (SessionStatus | 'all')[] = [
  'all',
  'pending',
  'confirmed',
  'live',
  'completed',
  'cancelled',
  'no_show',
];

const PAGE_SIZE = 15;

const formatWhen = (iso: string) => formatWhenLine(iso, 'row', ' · ');
const formatPrice = (n: number) => `${new Intl.NumberFormat('vi-VN').format(n)}₫`;

/** Status badge — reused in the table row and the mobile card. */
function StatusBadge({ status }: { status: SessionStatus }) {
  return (
    <Badge
      className={cn(
        'rounded-full border-0 px-2 text-[10px] font-bold tracking-wide uppercase',
        SESSION_STATUS_TONE[status],
      )}
    >
      {status.replace('_', ' ')}
    </Badge>
  );
}

export function SessionsSection({ sessions }: { sessions: SessionListItem[] }) {
  const [status, setStatus] = useState<SessionStatus | 'all'>('all');
  const [search, setSearch] = useState('');

  // Facet + text filter happens here — DataTable only sees the final list.
  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return sessions.filter((s) => {
      if (status !== 'all' && s.status !== status) return false;
      if (!needle) return true;
      return (
        s.tutor_name.toLowerCase().includes(needle) ||
        s.student_name.toLowerCase().includes(needle) ||
        (s.course_title_en || '').toLowerCase().includes(needle) ||
        (s.course_title_vn || '').toLowerCase().includes(needle)
      );
    });
  }, [sessions, status, search]);

  const columns = useMemo<ColumnDef<SessionListItem>[]>(
    () => [
      {
        id: 'when',
        accessorKey: 'scheduled_at',
        header: 'When',
        // Recent-first is the useful default for a session log.
        sortDescFirst: true,
        cell: ({ row }) => (
          <div className="flex items-center gap-2 text-slate-700 dark:text-slate-200">
            <CalendarClock className="size-3.5 shrink-0 text-slate-400" />
            <span className="tabular-nums">{formatWhen(row.original.scheduled_at)}</span>
          </div>
        ),
      },
      {
        id: 'tutor',
        accessorKey: 'tutor_name',
        header: 'Tutor',
        cell: ({ row }) => (
          <span className="font-medium text-slate-700 dark:text-slate-200">
            {row.original.tutor_name || '—'}
          </span>
        ),
      },
      {
        id: 'student',
        accessorKey: 'student_name',
        header: 'Student',
        cell: ({ row }) => (
          <span className="text-slate-700 dark:text-slate-200">
            {row.original.student_name || '—'}
          </span>
        ),
      },
      {
        id: 'course',
        accessorKey: 'course_title_en',
        header: 'Course',
        enableSorting: false,
        meta: { cellClassName: 'max-w-[240px] truncate text-slate-600 dark:text-slate-300' },
        cell: ({ row }) => <>{row.original.course_title_en || '—'}</>,
      },
      {
        id: 'price',
        accessorKey: 'price_vnd',
        header: 'Price',
        // Expensive-first — admin usually scans revenue outliers.
        sortDescFirst: true,
        meta: {
          cellClassName:
            'font-semibold whitespace-nowrap text-slate-700 tabular-nums dark:text-slate-200',
        },
        cell: ({ row }) => <>{formatPrice(row.original.price_vnd)}</>,
      },
      {
        id: 'status',
        accessorKey: 'status',
        header: 'Status',
        cell: ({ row }) => <StatusBadge status={row.original.status} />,
      },
    ],
    [],
  );

  return (
    <div className="space-y-4">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="type-page text-slate-900 dark:text-slate-100">Sessions</h1>
          <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
            All bookings across the platform. Showing{' '}
            <span className="font-semibold text-slate-700 tabular-nums dark:text-slate-200">
              {filtered.length}
            </span>{' '}
            of{' '}
            <span className="font-semibold text-slate-700 tabular-nums dark:text-slate-200">
              {sessions.length}
            </span>
            .
          </p>
        </div>
      </header>

      <div className="flex flex-wrap gap-2">
        <div className="relative min-w-[180px] flex-1">
          <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search by tutor, student, course…"
            className="h-9 pl-8"
          />
        </div>
        <div className="flex flex-wrap gap-1.5">
          {STATUSES.map((s) => (
            <FilterPill
              key={s}
              active={status === s}
              onClick={() => setStatus(s)}
              label={s}
              size="md"
            />
          ))}
        </div>
      </div>

      <DataTable
        data={filtered}
        columns={columns}
        pageSize={PAGE_SIZE}
        initialSort={[{ id: 'when', desc: true }]}
        getRowId={(s) => s.id}
        emptyState={
          <div className="rounded-2xl border border-dashed border-slate-200 py-16 text-center dark:border-slate-800">
            <p className="text-sm text-slate-500 dark:text-slate-400">
              No sessions match those filters yet.
            </p>
          </div>
        }
        mobileCard={(row) => {
          const s = row.original;
          return (
            <div className="space-y-2 p-4">
              <div className="flex items-start justify-between gap-2">
                <span className="flex items-center gap-1.5 text-sm font-semibold text-slate-800 tabular-nums dark:text-slate-100">
                  <CalendarClock className="size-3.5 shrink-0 text-slate-400" />
                  {formatWhen(s.scheduled_at)}
                </span>
                <StatusBadge status={s.status} />
              </div>
              <p className="line-clamp-1 text-xs text-slate-600 dark:text-slate-300">
                {s.course_title_en || '—'}
              </p>
              <div className="flex items-center justify-between gap-2 pt-1 text-xs">
                <p className="min-w-0 truncate text-slate-500 dark:text-slate-400">
                  <span className="font-medium text-slate-700 dark:text-slate-200">
                    {s.tutor_name || '—'}
                  </span>
                  <span className="mx-1 text-slate-300 dark:text-slate-600" aria-hidden>
                    ↔
                  </span>
                  {s.student_name || '—'}
                </p>
                <span className="shrink-0 font-semibold text-slate-800 tabular-nums dark:text-slate-100">
                  {formatPrice(s.price_vnd)}
                </span>
              </div>
            </div>
          );
        }}
      />
    </div>
  );
}
