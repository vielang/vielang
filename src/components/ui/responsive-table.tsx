'use client';

import * as React from 'react';
import { cn } from '@/lib/utils';

/**
 * ResponsiveTable — one dataset, two renderings.
 *
 * `<TableView>` and `<CardView>` are visibility-gated wrappers that let a
 * section render as a data table on md+ (density, aligned columns) and as a
 * card list on mobile (better tap targets, no horizontal scroll).
 *
 * Usage:
 * ```tsx
 * <ResponsiveTable>
 *   <TableView>
 *     <table className="w-full text-sm">
 *       <thead>...</thead>
 *       <tbody>
 *         {rows.map(r => <tr key={r.id}>...</tr>)}
 *       </tbody>
 *     </table>
 *   </TableView>
 *   <CardView>
 *     <ul className="space-y-2">
 *       {rows.map(r => <li key={r.id}>...</li>)}
 *     </ul>
 *   </CardView>
 * </ResponsiveTable>
 * ```
 *
 * The dual rendering is intentional — auto-collapsing a table via CSS
 * (`display: block` etc.) breaks column alignment, kills semantic table
 * navigation for screen readers, and produces an awkward middle-ground UI.
 * Two views, two purposes, one data source.
 */

export function ResponsiveTable({ className, children, ...props }: React.ComponentProps<'div'>) {
  return (
    <div data-slot="responsive-table" className={cn('w-full', className)} {...props}>
      {children}
    </div>
  );
}

/** Desktop table view. Wraps its <table> child with horizontal scroll safety. */
export function TableView({ className, children, ...props }: React.ComponentProps<'div'>) {
  return (
    <div
      data-slot="responsive-table-table"
      className={cn('hidden overflow-x-auto md:block', className)}
      {...props}
    >
      {children}
    </div>
  );
}

/** Mobile card list view. Full-width, no horizontal scroll. */
export function CardView({ className, children, ...props }: React.ComponentProps<'div'>) {
  return (
    <div data-slot="responsive-table-cards" className={cn('md:hidden', className)} {...props}>
      {children}
    </div>
  );
}
