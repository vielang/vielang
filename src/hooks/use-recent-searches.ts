'use client';

import { useCallback, useEffect, useState } from 'react';

// LocalStorage-backed recent search list. Small (max 5), string-only, and
// safe against SSR / disabled storage. Consumers own when to `push` (usually
// after a submitted search) and when to `clear`.

const MAX_ENTRIES = 5;

const read = (key: string): string[] => {
  if (typeof window === 'undefined') return [];
  try {
    const raw = window.localStorage.getItem(key);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.filter((v) => typeof v === 'string').slice(0, MAX_ENTRIES);
  } catch {
    return [];
  }
};

const write = (key: string, value: string[]) => {
  if (typeof window === 'undefined') return;
  try {
    window.localStorage.setItem(key, JSON.stringify(value));
  } catch {
    /* quota or disabled — silently drop */
  }
};

export function useRecentSearches(storageKey: string) {
  // Start empty on SSR + first client render, hydrate on mount. Avoids
  // hydration mismatches if a saved value exists.
  const [items, setItems] = useState<string[]>([]);
  const [hydrated, setHydrated] = useState(false);

  useEffect(() => {
    setItems(read(storageKey));
    setHydrated(true);
  }, [storageKey]);

  const push = useCallback(
    (raw: string) => {
      const value = raw.trim();
      if (!value) return;
      setItems((prev) => {
        const next = [value, ...prev.filter((v) => v.toLowerCase() !== value.toLowerCase())].slice(
          0,
          MAX_ENTRIES,
        );
        write(storageKey, next);
        return next;
      });
    },
    [storageKey],
  );

  const clear = useCallback(() => {
    setItems([]);
    write(storageKey, []);
  }, [storageKey]);

  return { items, push, clear, hydrated };
}
