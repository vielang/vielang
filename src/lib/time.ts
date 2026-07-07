// Time formatting helpers used across session cards, tables, and the
// video-room UI. Every timestamp we render comes back from the API as an
// ISO UTC string; the viewer's local tz is applied at the display layer so
// a HCM student sees "11:00" while an admin in Seoul sees "13:00" for the
// same row.

const FALLBACK_TZ = 'UTC';

export function getUserTz(): string {
  return typeof Intl !== 'undefined'
    ? Intl.DateTimeFormat().resolvedOptions().timeZone
    : FALLBACK_TZ;
}

export interface WhenParts {
  date: string;
  time: string;
}

// Variants map to the surfaces that render a session's when:
//   - 'card'    short weekday + day + short month + year (list cards)
//   - 'card-no-year'  same but without year (compact / homepage previews)
//   - 'detail'  long weekday + day + long month + year (detail page hero)
//   - 'row'     day + short month + year, no weekday (tabular views)
export type WhenVariant = 'card' | 'card-no-year' | 'detail' | 'row';

const DATE_OPTS: Record<WhenVariant, Intl.DateTimeFormatOptions> = {
  card: { weekday: 'short', day: '2-digit', month: 'short', year: 'numeric' },
  'card-no-year': { weekday: 'short', day: '2-digit', month: 'short' },
  detail: { weekday: 'long', day: '2-digit', month: 'long', year: 'numeric' },
  row: { day: '2-digit', month: 'short', year: 'numeric' },
};

const TIME_OPTS: Intl.DateTimeFormatOptions = { hour: '2-digit', minute: '2-digit' };

/**
 * Split "when" label — the shape session cards expect. Returns `date` and
 * `time` separately so the caller controls layout (side-by-side vs stacked).
 */
export function formatWhen(iso: string, variant: WhenVariant = 'card'): WhenParts {
  const d = new Date(iso);
  const tz = getUserTz();
  return {
    date: d.toLocaleDateString(undefined, { ...DATE_OPTS[variant], timeZone: tz }),
    time: d.toLocaleTimeString(undefined, { ...TIME_OPTS, timeZone: tz }),
  };
}

/**
 * Compact single-string variant for tables. Concatenates the parts of
 * `formatWhen` with the caller's separator so a comparator doesn't need
 * to touch string internals.
 */
export function formatWhenLine(iso: string, variant: WhenVariant = 'row', sep = ' · '): string {
  const { date, time } = formatWhen(iso, variant);
  return `${date}${sep}${time}`;
}

/**
 * Positive-duration label. Handles minutes, hours, and days so the countdown
 * pill stays readable across the full range of session lead times.
 *   < 60 min  → "45m"
 *   < 24 h    → "3h 05m"
 *   ≥ 24 h    → "2d 4h"  (minutes dropped — precision isn't useful at that scale)
 */
export function formatCountdown(ms: number): string {
  const totalMin = Math.max(0, Math.ceil(ms / 60_000));
  if (totalMin < 60) return `${totalMin}m`;
  const totalH = Math.floor(totalMin / 60);
  if (totalH < 24) return `${totalH}h ${(totalMin % 60).toString().padStart(2, '0')}m`;
  const d = Math.floor(totalH / 24);
  const h = totalH % 24;
  return h > 0 ? `${d}d ${h}h` : `${d}d`;
}

// Reserved window before start when the LiveKit token endpoint opens the
// room. Kept here so every entry point (cards, detail, pre-join panel) agrees.
// Set to a large value so participants can enter the room immediately after
// booking — useful for verifying LiveKit is connected before the live session.
export const JOIN_WINDOW_MS = 7 * 24 * 60 * 60_000; // 7 days
// Show the "Starts in X" countdown pill for any upcoming active session.
export const COUNTDOWN_WINDOW_MS = 7 * 24 * 60 * 60_000; // 7 days
