'use client';

import { useState } from 'react';
import { Loader2, Hourglass, LogOut, RefreshCw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { useTicker } from '@/hooks/use-ticker';
import { useLang } from '@/contexts';

const COPY = {
  VN: {
    title: 'Bạn đang ở phòng chờ',
    letYouIn: (name: string) => `${name} sẽ cho bạn vào ngay.`,
    onceAdmitted: (title: string) => `Khi được phép vào, buổi học ${title} sẽ bắt đầu.`,
    waitingPrefix: 'Đã chờ',
    queue: (rank: number, total: number) => `Bạn thứ #${rank} trong ${total}`,
    tip1: 'Máy ảnh và micro sẽ tắt cho đến khi giáo viên cho bạn vào.',
    tip2: 'Giữ tab này mở — bạn sẽ tự động vào khi được phép.',
    refresh: 'Làm mới & thử lại',
    leave: 'Rời phòng chờ',
  },
  EN: {
    title: "You're in the waiting room",
    letYouIn: (name: string) => `${name} will let you in shortly.`,
    onceAdmitted: (title: string) => `Once you're admitted, the ${title} session will start.`,
    waitingPrefix: 'Waiting',
    queue: (rank: number, total: number) => `You're #${rank} of ${total}`,
    tip1: 'Your camera and microphone stay off until the host lets you in.',
    tip2: "Keep this tab open — you'll join automatically as soon as you're admitted.",
    refresh: 'Refresh & retry',
    leave: 'Leave the waiting room',
  },
} as const;

// After this long, we show a "Still waiting? Refresh" affordance. Two failure
// modes it recovers from: (1) LiveKit's PermissionsChanged event never
// propagated so the client stays waiting even though the DB says admitted;
// (2) the tutor genuinely forgot about the student and they want to try
// rejoining fresh. The threshold is deliberately generous — a busy tutor
// admitting 6 students takes a while.
const STALE_THRESHOLD_MS = 60_000;

function formatElapsed(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const m = Math.floor(total / 60);
  const s = total % 60;
  if (m === 0) return `${s}s`;
  return `${m}m ${s.toString().padStart(2, '0')}s`;
}

/**
 * Full-viewport lobby shown to a student whose LiveKit permissions still say
 * `canPublish: false` — they connected but the host hasn't admitted them yet.
 *
 * The component intentionally has no LiveKit hook dependencies: it renders
 * static content and relies on its parent (`RoomBody`) to unmount it when
 * `useLocalParticipantPermissions()` reports canPublish=true.
 *
 * A live elapsed timer + a stale-timeout Refresh action solve two problems:
 * knowing how long you've been waiting (was I forgotten?), and recovering
 * from a LiveKit signal event that never landed (client stuck at
 * canPublish=false even though the server marked us admitted).
 */
export function WaitingRoomOverlay({
  otherName,
  courseTitle,
  queue,
  onLeave,
}: {
  otherName: string;
  courseTitle: string | null;
  /** Position among currently-waiting requesters (1-based). Null when the
   *  session doesn't require admission or the queue snapshot was skipped. */
  queue: { rank: number; total: number } | null;
  onLeave: () => void;
}) {
  const { lang } = useLang();
  const lc = COPY[lang];
  const [startedAt] = useState(() => Date.now());
  const now = useTicker(1_000);

  const elapsedMs = now - startedAt;
  const isStale = elapsedMs > STALE_THRESHOLD_MS;

  return (
    <div
      className="flex h-full flex-col items-center justify-center gap-6 bg-slate-950 px-6 py-10 text-center text-slate-100"
      role="status"
      aria-live="polite"
    >
      <div className="relative flex size-20 items-center justify-center">
        <Loader2 className="text-brand/50 absolute size-20 animate-spin" aria-hidden />
        <Hourglass className="text-brand size-8" aria-hidden />
      </div>

      <div className="max-w-md space-y-2">
        <h1 className="font-heading text-2xl font-bold">{lc.title}</h1>
        <p className="text-sm leading-relaxed text-slate-400">
          {lc.letYouIn(otherName)}
          {courseTitle ? <> {lc.onceAdmitted(courseTitle)}</> : null}
        </p>
        <p className="text-xs text-slate-500 tabular-nums" aria-live="off">
          {lc.waitingPrefix} {formatElapsed(elapsedMs)}
          {queue && queue.total > 1 ? (
            <>
              {' · '}
              <span className="text-slate-300">{lc.queue(queue.rank, queue.total)}</span>
            </>
          ) : null}
        </p>
      </div>

      <ul className="w-full max-w-xs space-y-2 text-left text-xs text-slate-400">
        <li className="rounded-lg border border-slate-800 bg-slate-900/70 px-3 py-2">{lc.tip1}</li>
        <li className="rounded-lg border border-slate-800 bg-slate-900/70 px-3 py-2">{lc.tip2}</li>
      </ul>

      <div className="flex flex-wrap items-center justify-center gap-3">
        {isStale && (
          <Button
            variant="outline"
            size="sm"
            onClick={() => window.location.reload()}
            className="border-amber-500/50 bg-transparent text-amber-300 hover:border-amber-500 hover:bg-amber-500/10 hover:text-amber-200"
          >
            <RefreshCw />
            {lc.refresh}
          </Button>
        )}
        <Button
          variant="outline"
          size="sm"
          onClick={onLeave}
          className="border-slate-700 bg-transparent text-slate-300 hover:border-red-500 hover:bg-red-500/10 hover:text-red-300"
        >
          <LogOut />
          {lc.leave}
        </Button>
      </div>
    </div>
  );
}
