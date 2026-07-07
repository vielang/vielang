import type { SessionStatus } from './types';

/**
 * Shared status → color-class map used by session cards + admin tables so
 * one surface's "confirmed = emerald" never drifts from another's.
 *
 * These are semantic (green = success, amber = pending, sky = live, red =
 * cancelled) rather than brand — status pills should not fight the brand
 * primary for attention.
 */
export const SESSION_STATUS_TONE: Record<SessionStatus, string> = {
  pending: 'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300',
  confirmed: 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300',
  live: 'bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300',
  completed: 'bg-slate-100 dark:bg-slate-800 text-slate-600 dark:text-slate-300',
  cancelled: 'bg-red-50 dark:bg-red-950/40 text-red-600 dark:text-red-400',
  no_show: 'bg-red-50 dark:bg-red-950/40 text-red-600 dark:text-red-400',
};
