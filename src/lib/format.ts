// Small display formatters shared across the UI. Kept dependency-free so
// they can be pulled from server components, client components, and email
// templates without dragging any React state along.

const VND_FORMATTER = new Intl.NumberFormat('vi-VN', { maximumFractionDigits: 0 });

/**
 * VND display without the currency symbol — the surrounding UI usually
 * pairs it with a raw "₫" glyph so we don't hardcode the position (some
 * cards put it after, some before).
 */
export function formatVnd(amount: number): string {
  return VND_FORMATTER.format(amount);
}

/**
 * Up to two-letter initials from a name. Falls back to "?" when the input
 * is empty or missing so the avatar bubble never renders empty state.
 */
export function initials(name?: string | null): string {
  if (!name) return '?';
  const parts = name
    .split(/\s+/)
    .slice(0, 2)
    .map((s) => s[0]?.toUpperCase() || '')
    .join('');
  return parts || '?';
}
