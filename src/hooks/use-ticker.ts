import { useEffect, useState } from 'react';

/**
 * Return the current wall-clock timestamp and refresh it every `intervalMs`.
 * Powers countdowns / join-window flips on session cards without forcing a
 * full data refetch.
 *
 * The default 15 000 ms matches the join-window state precision — anything
 * finer would spend a lot of render cycles for a countdown that only visibly
 * changes once a minute anyway.
 */
export function useTicker(intervalMs = 15_000): number {
  const [now, setNow] = useState<number>(() => Date.now());
  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), intervalMs);
    return () => clearInterval(id);
  }, [intervalMs]);
  return now;
}
