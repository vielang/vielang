'use client';

import { AnimatePresence, motion } from 'motion/react';
import { usePathname } from 'next/navigation';
import { useAuth } from '@/contexts';
import { useLang } from '@/contexts';
import { usePresence, type PresenceUser } from '@/hooks/use-presence';

const MAX_AVATARS = 4;

const COPY = {
  VN: {
    watching: (n: number) => (n === 1 ? '1 người đang xem' : `${n} người đang xem`),
    you: 'Bạn',
    guest: 'Khách',
  },
  EN: {
    watching: (n: number) => (n === 1 ? '1 watching' : `${n} watching`),
    you: 'You',
    guest: 'Guest',
  },
} as const;

// Spring used for all avatar enter/exit transitions. Intentionally snappy —
// presence updates happen in the background and shouldn't steal attention.
const SPRING = { type: 'spring', stiffness: 500, damping: 28, mass: 0.6 } as const;

function Avatar({ user, size = 26, label }: { user: PresenceUser; size?: number; label: string }) {
  const hues = [221, 142, 262, 25, 199, 330, 47, 168];
  const hue = hues[user.id.charCodeAt(user.id.length - 1) % hues.length];
  const initial = user.isGuest ? '?' : (user.name.trim()[0] ?? '?').toUpperCase();

  return (
    <span
      className="relative inline-block shrink-0"
      style={{ width: size, height: size }}
      title={label}
      aria-label={label}
    >
      {user.avatar ? (
        // eslint-disable-next-line @next/next/no-img-element
        <img
          src={user.avatar}
          alt={user.name}
          className="size-full rounded-full object-cover ring-2 ring-white dark:ring-slate-900"
          style={{ width: size, height: size }}
          referrerPolicy="no-referrer"
        />
      ) : (
        <span
          className="flex size-full items-center justify-center rounded-full text-white ring-2 ring-white dark:ring-slate-900"
          style={{
            width: size,
            height: size,
            background: `hsl(${hue} 65% 50%)`,
            fontSize: size * 0.4,
            fontWeight: 600,
            lineHeight: 1,
          }}
        >
          {initial}
        </span>
      )}
      {/* Online dot — pings once when the avatar first mounts */}
      <span aria-hidden className="absolute right-0 bottom-0">
        {/* Ping ring */}
        <span className="absolute inline-flex size-2 rounded-full bg-emerald-400 opacity-75 motion-safe:animate-ping" />
        {/* Solid dot */}
        <span className="relative inline-flex size-2 rounded-full border-[1.5px] border-white bg-emerald-500 dark:border-slate-900" />
      </span>
    </span>
  );
}

/**
 * Compact presence pill — avatar stack + count.
 * Only renders on the homepage (pathname === '/').
 * Guests count too; no login required.
 *
 * Animation contract:
 *  - New avatar:  scale 0→1 + fade 0→1, spring, slight overshoot.
 *  - Leaving:     scale 1→0 + fade 1→0, quicker easing so the gap closes fast.
 *  - Count label: crossfades via key change so the number flips smoothly.
 *  - Overflow "+N": same enter/exit as avatars.
 */
export function OnlinePresence() {
  const pathname = usePathname();
  const { user } = useAuth();
  const { lang } = useLang();
  const lc = COPY[lang];

  const presenceUser = user ? { id: user.id, name: user.name, avatar: null } : null;
  const { users, count } = usePresence(presenceUser);

  if (pathname !== '/') return null;
  if (count === 0) return null;

  const visible = users.slice(0, MAX_AVATARS);
  const overflow = count - MAX_AVATARS;

  return (
    <div
      className="flex items-center gap-1.5"
      role="status"
      aria-label={lc.watching(count)}
      aria-live="polite"
    >
      {/* Avatar stack */}
      <div className="flex items-center">
        <AnimatePresence initial={false} mode="popLayout">
          {visible.map((u, i) => {
            const isMe = user && u.id === user.id;
            const label = isMe ? lc.you : u.isGuest ? lc.guest : u.name;
            return (
              <motion.span
                key={u.id}
                // Enter: pop in from scale 0, slight overshoot via spring.
                initial={{ scale: 0, opacity: 0 }}
                animate={{ scale: 1, opacity: 1 }}
                // Exit: shrink away quickly so the gap closes without a jump.
                exit={{ scale: 0, opacity: 0, transition: { duration: 0.15 } }}
                transition={SPRING}
                className="hover:z-10"
                style={{
                  marginLeft: i === 0 ? 0 : -8,
                  zIndex: visible.length - i,
                  // Transform origin at centre so the pop looks natural in the stack.
                  transformOrigin: 'center center',
                  display: 'inline-block',
                }}
                whileHover={{ scale: 1.15, zIndex: 20 }}
              >
                <Avatar user={u} size={26} label={label} />
              </motion.span>
            );
          })}

          {/* Overflow badge — animated the same way as avatars */}
          {overflow > 0 && (
            <motion.span
              key="overflow"
              initial={{ scale: 0, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0, opacity: 0, transition: { duration: 0.15 } }}
              transition={SPRING}
              className="relative inline-flex items-center justify-center rounded-full bg-slate-200 ring-2 ring-white dark:bg-slate-700 dark:ring-slate-900"
              style={{
                width: 26,
                height: 26,
                marginLeft: -8,
                zIndex: 0,
                fontSize: 10,
                fontWeight: 600,
              }}
              aria-hidden
            >
              +{overflow}
            </motion.span>
          )}
        </AnimatePresence>
      </div>

      {/* Count label — key-driven crossfade so the number doesn't just cut */}
      <AnimatePresence mode="popLayout" initial={false}>
        <motion.span
          key={count}
          initial={{ opacity: 0, y: -6 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: 6 }}
          transition={{ duration: 0.2 }}
          className="hidden text-[11px] font-medium text-slate-500 sm:block dark:text-slate-400"
        >
          {lc.watching(count)}
        </motion.span>
      </AnimatePresence>
    </div>
  );
}
