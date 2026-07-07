'use client';

import { Wifi, WifiOff, Loader2 } from 'lucide-react';
import {
  useConnectionState,
  useConnectionQualityIndicator,
  useLocalParticipant,
} from '@livekit/components-react';
import { ConnectionState, ConnectionQuality } from 'livekit-client';
import { useLang } from '@/contexts';

const COPY = {
  VN: {
    reconnecting: 'Đang kết nối lại lớp học…',
    good: 'Kết nối ổn định',
    poor: 'Kết nối yếu — âm thanh và hình ảnh có thể giật',
    measuring: 'Đang đo kết nối…',
  },
  EN: {
    reconnecting: 'Reconnecting to the classroom…',
    good: 'Connection is good',
    poor: 'Connection is poor — audio and video may lag',
    measuring: 'Measuring connection…',
  },
} as const;

/**
 * Two lightweight, non-blocking UI signals that live on top of the video
 * conference so the user can tell why audio/video is stuttering before they
 * have to guess:
 *
 *   • A full-width amber banner at the top when LiveKit is in the middle of
 *     a reconnect (signal or media). LiveKit retries automatically for ~15s
 *     and the participant almost always comes back, but a silent retry looks
 *     identical to a dead network from the user's chair.
 *   • A 4-bar quality glyph fixed to the top-right showing the local
 *     participant's connection quality. Excellent/Good stay quiet (small
 *     green icon); Poor/Unknown promotes to red with a tooltip so the user
 *     knows it isn't the tutor's mic — it's their pipe.
 *
 * Both surfaces live inside a LiveKitRoom so the hooks resolve; mount inside
 * RoomBody's admitted branch alongside VideoConference.
 */
export function ConnectionStatusBar() {
  const { lang } = useLang();
  const lc = COPY[lang];
  const state = useConnectionState();
  const isReconnecting =
    state === ConnectionState.Reconnecting || state === ConnectionState.SignalReconnecting;

  return (
    <>
      {isReconnecting && (
        <div
          role="status"
          aria-live="polite"
          className="pointer-events-none absolute inset-x-0 top-0 z-30 flex items-center justify-center gap-2 bg-amber-500/90 px-4 py-2 text-xs font-semibold text-amber-950 shadow-md"
        >
          <Loader2 className="size-3.5 animate-spin" />
          {lc.reconnecting}
        </div>
      )}
      <QualityIndicator />
    </>
  );
}

function QualityIndicator() {
  const { lang } = useLang();
  const lc = COPY[lang];
  // Pass the local participant explicitly — the hook throws when no
  // participant is in scope and there's no ParticipantContext at this
  // level of the tree (VideoConference sets one per tile but not
  // component-wide). Fallback to null until the participant object
  // resolves, otherwise the first render throws under React 19.
  const { localParticipant } = useLocalParticipant();
  const { quality } = useConnectionQualityIndicator({ participant: localParticipant });
  if (!localParticipant) return null;

  const good = quality === ConnectionQuality.Excellent || quality === ConnectionQuality.Good;
  const poor = quality === ConnectionQuality.Poor || quality === ConnectionQuality.Lost;

  // Excellent → 4 lit bars; Good → 3; Poor → 2; anything else → 1.
  const lit =
    quality === ConnectionQuality.Excellent
      ? 4
      : quality === ConnectionQuality.Good
        ? 3
        : quality === ConnectionQuality.Poor
          ? 2
          : 1;

  const tone = good ? 'text-emerald-400' : poor ? 'text-red-400' : 'text-slate-400';

  const label = good ? lc.good : poor ? lc.poor : lc.measuring;

  const Icon = quality === ConnectionQuality.Lost ? WifiOff : Wifi;

  return (
    <div
      className="pointer-events-none absolute top-3 right-3 z-30 flex items-center gap-1.5 rounded-full bg-slate-900/70 px-2.5 py-1 text-[10px] font-semibold text-white backdrop-blur-sm"
      title={label}
      aria-label={label}
    >
      <Icon className={`size-3 ${tone}`} />
      <div className="flex items-end gap-0.5" aria-hidden>
        {[1, 2, 3, 4].map((n) => (
          <span
            key={n}
            className={`w-0.5 rounded-sm ${n <= lit ? tone.replace('text-', 'bg-') : 'bg-white/20'}`}
            style={{ height: `${3 + n * 2}px` }}
          />
        ))}
      </div>
    </div>
  );
}
