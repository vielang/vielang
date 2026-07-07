'use client';

import { forwardRef } from 'react';
import type { ButtonHTMLAttributes, ReactNode } from 'react';

interface FilterPillProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'children'> {
  active: boolean;
  label: ReactNode;
  /** Optional Lucide icon-in-a-span component; rendered on the left. */
  icon?: ReactNode;
  /**
   * Visual size — 'sm' is the compact pill used inside filter bars,
   * 'md' is roomier for standalone filter chips. Default sm.
   */
  size?: 'sm' | 'md';
}

/**
 * Toggleable pill used by every filter/tab strip on the site — tutor
 * discovery specialty picker, admin status filter, admin role filter,
 * tutor sessions tabs. Extracted so subtle divergence between call sites
 * (different heights, missing focus rings, inconsistent hover) can't drift.
 *
 * Uses `aria-pressed` for a11y — the pill behaves like a toggle button.
 */
export const FilterPill = forwardRef<HTMLButtonElement, FilterPillProps>(function FilterPill(
  { active, label, icon, size = 'sm', className = '', ...rest },
  ref,
) {
  const height = size === 'md' ? 'h-9 px-3' : 'h-8 px-3';
  const font = 'text-xs font-semibold';
  return (
    <button
      ref={ref}
      type="button"
      aria-pressed={active}
      className={`focus-ring inline-flex items-center gap-1.5 ${height} ${font} rounded-full border transition-colors ${
        active
          ? 'bg-brand border-brand text-white'
          : 'hover:border-brand/40 hover:text-brand border-slate-200 bg-white text-slate-600 dark:border-slate-700 dark:bg-slate-900 dark:text-slate-300'
      } ${className}`}
      {...rest}
    >
      {icon}
      {label}
    </button>
  );
});
