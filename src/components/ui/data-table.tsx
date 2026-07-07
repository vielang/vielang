'use client';

import * as React from 'react';
import {
  type ColumnDef,
  type Row,
  type SortingState,
  flexRender,
  getCoreRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from '@tanstack/react-table';
import { ArrowDown, ArrowUp, ArrowUpDown, ChevronLeft, ChevronRight } from 'lucide-react';

import { cn } from '@/lib/utils';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';

/**
 * DataTable — shared admin-list surface built on TanStack Table v8.
 *
 * Renders semantic <table> markup on md+ (with click-to-sort headers and
 * pagination) and delegates to a `mobileCard(row)` render prop below md so
 * each row becomes a tap-friendly card. The dual rendering matches the
 * design decision documented in `responsive-table.tsx`: auto-collapsing a
 * table via CSS destroys column alignment and breaks screen-reader table
 * navigation. Keep two views, one dataset.
 *
 * Column-level responsive controls:
 *   - `meta.headClassName` / `meta.cellClassName` — extra classes per column
 *   - `meta.hideBelow: 'lg'` — hide the column on <lg (still tapable on card)
 *
 * Filtering / facet chips stay OUTSIDE this component — sections manage
 * their own status/role/level filters and pass the already-filtered rows in.
 * That keeps DataTable a pure "list + sort + paginate" surface.
 */
export interface DataTableColumnMeta {
  /** Extra classes appended to <th>. */
  headClassName?: string;
  /** Extra classes appended to <td>. */
  cellClassName?: string;
  /** Hide the column below this Tailwind breakpoint (desktop table only). */
  hideBelow?: 'sm' | 'md' | 'lg' | 'xl';
  /** Force asc-first / desc-first when this column becomes the sort target.
   *  Default (undefined) = TanStack default (asc first). */
  sortDescFirst?: boolean;
}

// TanStack v8 ships an empty ColumnMeta by default; teach it about our fields
// so `meta` on ColumnDef gets our extra properties without `any`.
declare module '@tanstack/react-table' {
  /* eslint-disable @typescript-eslint/no-empty-object-type, @typescript-eslint/no-unused-vars */
  interface ColumnMeta<TData, TValue> extends DataTableColumnMeta {}
  /* eslint-enable @typescript-eslint/no-empty-object-type, @typescript-eslint/no-unused-vars */
}

export interface DataTableProps<TData> {
  data: TData[];
  columns: ColumnDef<TData, unknown>[];
  /** Row => card ReactNode for the <md view. If omitted the desktop table
   *  still renders on mobile (horizontal scroll). */
  mobileCard?: (row: Row<TData>) => React.ReactNode;
  /** Rows per page for both md+ table and mobile card list. */
  pageSize?: number;
  /** Initial sort — `[{ id, desc }]`. */
  initialSort?: SortingState;
  /** Message rendered when `data` is empty after external filtering. */
  emptyState?: React.ReactNode;
  /** Extra classes for the wrapper card. */
  className?: string;
  /** Stable key for each row (defaults to (row) => row.id if present). */
  getRowId?: (row: TData, index: number) => string;
}

const HIDE_BELOW: Record<NonNullable<DataTableColumnMeta['hideBelow']>, string> = {
  sm: 'hidden sm:table-cell',
  md: 'hidden md:table-cell',
  lg: 'hidden lg:table-cell',
  xl: 'hidden xl:table-cell',
};

export function DataTable<TData>({
  data,
  columns,
  mobileCard,
  pageSize = 15,
  initialSort = [],
  emptyState,
  className,
  getRowId,
}: DataTableProps<TData>) {
  const [sorting, setSorting] = React.useState<SortingState>(initialSort);
  const [pageIndex, setPageIndex] = React.useState(0);

  // React Compiler flags useReactTable as incompatible (its returned functions
  // can't be safely memoized). That's fine for our usage — we don't hand `table`
  // to any downstream memoized component.
  // eslint-disable-next-line react-hooks/incompatible-library
  const table = useReactTable<TData>({
    data,
    columns,
    state: { sorting, pagination: { pageIndex, pageSize } },
    onSortingChange: setSorting,
    onPaginationChange: (updater) => {
      const next = typeof updater === 'function' ? updater({ pageIndex, pageSize }) : updater;
      setPageIndex(next.pageIndex);
    },
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getRowId,
    manualPagination: false,
  });

  // Whenever external filtering shrinks the dataset past the current page,
  // walk back to a valid one. TanStack doesn't do this automatically.
  React.useEffect(() => {
    const total = table.getPageCount();
    if (total > 0 && pageIndex >= total) setPageIndex(total - 1);
  }, [data.length, pageIndex, table]);

  const rows = table.getRowModel().rows;
  const totalPages = Math.max(1, table.getPageCount());

  if (data.length === 0) {
    return (
      <>
        {emptyState ?? (
          <div className="rounded-2xl border border-dashed border-slate-200 py-16 text-center dark:border-slate-800">
            <p className="text-sm text-slate-500 dark:text-slate-400">No results.</p>
          </div>
        )}
      </>
    );
  }

  return (
    <div className="space-y-3">
      {/* Desktop: semantic table with sort headers */}
      <div
        className={cn(
          'hidden overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm md:block dark:border-slate-800 dark:bg-slate-900',
          className,
        )}
      >
        <Table className="min-w-full">
          <TableHeader className="bg-slate-50 dark:bg-slate-800/40">
            {table.getHeaderGroups().map((group) => (
              <TableRow key={group.id} className="hover:bg-transparent">
                {group.headers.map((header) => {
                  const meta = header.column.columnDef.meta;
                  const canSort = header.column.getCanSort();
                  const sortDir = header.column.getIsSorted();
                  return (
                    <TableHead
                      key={header.id}
                      aria-sort={
                        sortDir === 'asc' ? 'ascending' : sortDir === 'desc' ? 'descending' : 'none'
                      }
                      className={cn(
                        'text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400',
                        meta?.hideBelow && HIDE_BELOW[meta.hideBelow],
                        meta?.headClassName,
                      )}
                    >
                      {header.isPlaceholder ? null : canSort ? (
                        <button
                          type="button"
                          onClick={header.column.getToggleSortingHandler()}
                          className={cn(
                            'focus-ring inline-flex items-center gap-1.5 tracking-wider uppercase transition-colors',
                            sortDir
                              ? 'text-brand'
                              : 'text-slate-500 hover:text-slate-700 dark:text-slate-400 dark:hover:text-slate-200',
                          )}
                        >
                          {flexRender(header.column.columnDef.header, header.getContext())}
                          {sortDir === 'asc' ? (
                            <ArrowUp className="size-3" />
                          ) : sortDir === 'desc' ? (
                            <ArrowDown className="size-3" />
                          ) : (
                            <ArrowUpDown className="size-3 opacity-40" />
                          )}
                        </button>
                      ) : (
                        flexRender(header.column.columnDef.header, header.getContext())
                      )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {rows.map((row) => (
              <TableRow
                key={row.id}
                className="border-t border-slate-100 hover:bg-slate-50 dark:border-slate-800 dark:hover:bg-slate-800/40"
              >
                {row.getVisibleCells().map((cell) => {
                  const meta = cell.column.columnDef.meta;
                  return (
                    <TableCell
                      key={cell.id}
                      className={cn(
                        'px-4 py-3',
                        meta?.hideBelow && HIDE_BELOW[meta.hideBelow],
                        meta?.cellClassName,
                      )}
                    >
                      {flexRender(cell.column.columnDef.cell, cell.getContext())}
                    </TableCell>
                  );
                })}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>

      {/* Mobile: card list. Falls back to the desktop table (with horizontal
          scroll) if the caller didn't provide a mobileCard renderer. */}
      {mobileCard ? (
        <div className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm md:hidden dark:border-slate-800 dark:bg-slate-900">
          <ul className="divide-y divide-slate-100 dark:divide-slate-800">
            {rows.map((row) => (
              <li key={row.id}>{mobileCard(row)}</li>
            ))}
          </ul>
        </div>
      ) : null}

      {totalPages > 1 && (
        <div className="flex items-center justify-between gap-3 text-xs">
          <p className="text-slate-500 tabular-nums dark:text-slate-400">
            Page {pageIndex + 1} of {totalPages}
          </p>
          <div className="flex items-center gap-2">
            <button
              type="button"
              onClick={() => setPageIndex((p) => Math.max(0, p - 1))}
              disabled={pageIndex === 0}
              className="focus-ring tap-min hover:border-brand/40 hover:text-brand inline-flex items-center gap-1 rounded-lg border border-slate-200 px-3 font-semibold text-slate-600 transition-colors disabled:opacity-40 dark:border-slate-700 dark:text-slate-300"
            >
              <ChevronLeft className="size-3.5" /> Prev
            </button>
            <button
              type="button"
              onClick={() => setPageIndex((p) => Math.min(totalPages - 1, p + 1))}
              disabled={pageIndex >= totalPages - 1}
              className="focus-ring tap-min hover:border-brand/40 hover:text-brand inline-flex items-center gap-1 rounded-lg border border-slate-200 px-3 font-semibold text-slate-600 transition-colors disabled:opacity-40 dark:border-slate-700 dark:text-slate-300"
            >
              Next <ChevronRight className="size-3.5" />
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
