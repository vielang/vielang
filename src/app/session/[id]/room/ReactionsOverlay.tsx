'use client';

import { useCallback, useEffect, useRef, useState } from 'react';
import { useRoomContext } from '@livekit/components-react';
import { RoomEvent, type RemoteParticipant } from 'livekit-client';
import { useLang } from '@/contexts';

// The five reactions we support. Kept short so the trigger bar stays
// compact — a longer palette would need to swipe/scroll on mobile.
// Each entry carries a screen-reader label (per language) since
// "Send 👍 reaction" is read as "Send emoji reaction" by most screen
// readers — useless.
type ReactionKey = 'thumbsUp' | 'applause' | 'laughing' | 'love' | 'thinking';

const REACTIONS: ReadonlyArray<{ emoji: string; key: ReactionKey }> = [
  { emoji: '👍', key: 'thumbsUp' },
  { emoji: '👏', key: 'applause' },
  { emoji: '😂', key: 'laughing' },
  { emoji: '❤️', key: 'love' },
  { emoji: '🤔', key: 'thinking' },
];

const REACTION_LABELS: Record<
  'VN' | 'EN',
  { names: Record<ReactionKey, string>; send: (label: string) => string }
> = {
  VN: {
    names: {
      thumbsUp: 'thích',
      applause: 'vỗ tay',
      laughing: 'cười',
      love: 'yêu thích',
      thinking: 'suy nghĩ',
    },
    send: (label) => `Gửi cảm xúc ${label}`,
  },
  EN: {
    names: {
      thumbsUp: 'thumbs up',
      applause: 'applause',
      laughing: 'laughing',
      love: 'love',
      thinking: 'thinking',
    },
    send: (label) => `Send ${label} reaction`,
  },
};

// Data-channel topic. Everyone in the room subscribes; the payload is a
// tiny JSON `{ emoji: '…' }`. Reactions are ephemeral — we use lossy
// (unreliable) delivery so a laggy peer doesn't build up a backlog.
const REACTION_TOPIC = 'vielang.reaction';
const REACTION_LIFETIME_MS = 3000;
const MAX_CONCURRENT_FLOATS = 15;

interface Float {
  id: string;
  emoji: string;
  x: number; // percentage from the container's left edge
  createdAt: number;
}

/**
 * Floating-emoji reactions overlay. Two moving parts:
 *
 *   • A pill of tap targets at the bottom of the room that publishes a
 *     `{ emoji }` payload to REACTION_TOPIC and shows a local echo.
 *   • A pointer-events-none layer above the video grid that animates each
 *     spawned emoji up + fades it after REACTION_LIFETIME_MS.
 *
 * The local echo is important — even a fast reliable send takes a beat to
 * bounce back through LiveKit, and clicking a button that appears to do
 * nothing feels broken. Since we spawn locally on click, everyone-else's
 * receive fires without spawning our own copy again (checked via
 * participant identity in the handler).
 */
export function ReactionsOverlay() {
  const { lang } = useLang();
  const labels = REACTION_LABELS[lang];
  const room = useRoomContext();
  const [floats, setFloats] = useState<Float[]>([]);
  const idCounter = useRef(0);

  const spawn = useCallback((emoji: string) => {
    setFloats((prev) => {
      const now = Date.now();
      const alive = prev.filter((f) => now - f.createdAt < REACTION_LIFETIME_MS);
      // Cap so a joker mashing reactions can't blow up the DOM.
      const capped = alive.length >= MAX_CONCURRENT_FLOATS ? alive.slice(1) : alive;
      return [
        ...capped,
        {
          id: `r-${idCounter.current++}`,
          emoji,
          // Spread across the middle 40% so multiple emojis of the same
          // kind don't stack visually.
          x: 30 + Math.random() * 40,
          createdAt: now,
        },
      ];
    });
  }, []);

  // Garbage-collect aged-out floats. React alone won't drop them because
  // `floats` state changes only on new spawns; without this a lone
  // reaction would linger in state forever.
  useEffect(() => {
    if (floats.length === 0) return;
    const t = setTimeout(() => {
      setFloats((prev) => prev.filter((f) => Date.now() - f.createdAt < REACTION_LIFETIME_MS));
    }, REACTION_LIFETIME_MS + 100);
    return () => clearTimeout(t);
  }, [floats.length]);

  // Subscribe to incoming reactions from other participants. We ignore
  // messages on other topics so this hook coexists cleanly with any other
  // data-channel consumers (e.g. LiveKit's own chat).
  useEffect(() => {
    const handler = (
      payload: Uint8Array,
      participant?: RemoteParticipant,
      _kind?: unknown,
      topic?: string,
    ) => {
      if (topic !== REACTION_TOPIC) return;
      // We already local-echoed our own reactions on click — participant is
      // undefined when the message is our own outbound, so guard against it.
      if (!participant) return;
      try {
        const decoded = JSON.parse(new TextDecoder().decode(payload)) as { emoji?: string };
        if (typeof decoded?.emoji === 'string') spawn(decoded.emoji);
      } catch {
        /* malformed payload — ignore */
      }
    };
    room.on(RoomEvent.DataReceived, handler);
    return () => {
      room.off(RoomEvent.DataReceived, handler);
    };
  }, [room, spawn]);

  const emit = useCallback(
    async (emoji: string) => {
      // Local echo first so tapping the button feels instantaneous. The
      // remote broadcast can lag behind by whatever the SFU adds.
      spawn(emoji);
      try {
        const payload = new TextEncoder().encode(JSON.stringify({ emoji }));
        await room.localParticipant.publishData(payload, {
          reliable: false,
          topic: REACTION_TOPIC,
        });
      } catch {
        /* reactions are ephemeral by design; a lost one is acceptable */
      }
    },
    [room, spawn],
  );

  return (
    <>
      {/*
        bottom-36 on mobile lifts the pill above the chat + host pills
        (both at bottom-24 = 96px). Without it, the three clusters overlap
        horizontally on ≤375px viewports because each pill takes ~52px of
        their sides plus reactions' 240px middle.
      */}
      <div className="pointer-events-auto absolute bottom-36 left-1/2 z-30 flex -translate-x-1/2 items-center gap-1 rounded-full bg-slate-900/70 px-1.5 py-1 backdrop-blur-sm sm:bottom-28">
        {REACTIONS.map((r) => (
          <button
            key={r.emoji}
            type="button"
            onClick={() => void emit(r.emoji)}
            aria-label={labels.send(labels.names[r.key])}
            className="focus-ring inline-flex size-11 items-center justify-center rounded-full text-2xl transition-transform hover:scale-110 active:scale-95"
          >
            <span aria-hidden>{r.emoji}</span>
          </button>
        ))}
      </div>

      <div className="pointer-events-none absolute inset-0 z-20 overflow-hidden">
        {floats.map((f) => (
          <span
            key={f.id}
            className="vielang-reaction-float absolute bottom-32 text-3xl"
            style={{ left: `${f.x}%` }}
          >
            {f.emoji}
          </span>
        ))}
      </div>

      {/*
        Keyframes live inline so we don't touch tailwind.config or add a
        global stylesheet. `prefers-reduced-motion` skips the animation and
        just shows the emoji briefly (opacity drops on its own via the
        GC timeout).
      */}
      <style>{`
        .vielang-reaction-float {
          animation: vielang-reaction-float 3s ease-out forwards;
        }
        @media (prefers-reduced-motion: reduce) {
          .vielang-reaction-float { animation: vielang-reaction-fade 3s linear forwards; }
        }
        @keyframes vielang-reaction-float {
          0%   { transform: translateY(0) scale(0.6); opacity: 0; }
          15%  { opacity: 1; transform: translateY(-20px) scale(1); }
          80%  { opacity: 1; }
          100% { transform: translateY(-180px) scale(1); opacity: 0; }
        }
        @keyframes vielang-reaction-fade {
          0%, 20%  { opacity: 1; }
          100%     { opacity: 0; }
        }
      `}</style>
    </>
  );
}
