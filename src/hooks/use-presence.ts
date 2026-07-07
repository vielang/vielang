'use client';

import { useEffect, useRef, useState } from 'react';
import { supabaseBrowser } from '@/lib/supabase/client';

export interface PresenceUser {
  id: string;
  name: string;
  avatar: string | null;
  isGuest: boolean;
}

interface TrackedPayload extends PresenceUser {
  presence_ref?: string;
}

const CHANNEL = 'presence:homepage';

/**
 * Returns a stable anonymous ID for the current browser tab. Stored in
 * sessionStorage so it persists across soft navigations but resets when the
 * tab is closed — guests don't accumulate across sessions.
 */
function getGuestId(): string {
  const key = 'vl_gid';
  let id = sessionStorage.getItem(key);
  if (!id) {
    id = `g-${Math.random().toString(36).slice(2, 10)}`;
    sessionStorage.setItem(key, id);
  }
  return id;
}

/**
 * Supabase Realtime Presence — tracks who is currently on the homepage.
 *
 * Authenticated users broadcast their real name + avatar. Guests broadcast
 * an anonymous payload so they still appear in the count without leaking
 * identity. The hook subscribes on mount and tears down cleanly on unmount.
 */
export function usePresence(
  currentUser: { id: string; name: string; avatar?: string | null } | null,
) {
  const [users, setUsers] = useState<PresenceUser[]>([]);
  // Avoid stale closures in the effect by keeping a ref to currentUser.
  const userRef = useRef(currentUser);
  useEffect(() => {
    userRef.current = currentUser;
  });

  useEffect(() => {
    const channel = supabaseBrowser.channel(CHANNEL, {
      config: { presence: { key: userRef.current?.id ?? getGuestId() } },
    });

    const syncPresence = () => {
      const raw = channel.presenceState<TrackedPayload>();
      const list: PresenceUser[] = Object.values(raw)
        .flat()
        .map(({ id, name, avatar, isGuest }) => ({ id, name, avatar: avatar ?? null, isGuest }));
      // Deduplicate by id — the same user might have multiple tabs open.
      const seen = new Set<string>();
      setUsers(list.filter((u) => (seen.has(u.id) ? false : seen.add(u.id) && true)));
    };

    channel.on('presence', { event: 'sync' }, syncPresence).subscribe(async (status) => {
      if (status !== 'SUBSCRIBED') return;
      const u = userRef.current;
      const payload: PresenceUser = u
        ? { id: u.id, name: u.name, avatar: u.avatar ?? null, isGuest: false }
        : { id: getGuestId(), name: 'Guest', avatar: null, isGuest: true };
      await channel.track(payload);
    });

    return () => {
      void supabaseBrowser.removeChannel(channel);
    };
  }, []);

  return { users, count: users.length };
}
