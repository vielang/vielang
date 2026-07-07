'use client';

import { useCallback, useEffect, useState } from 'react';
import { Hand } from 'lucide-react';
import { useLocalParticipant, useRoomContext } from '@livekit/components-react';
import { RoomEvent, type RemoteParticipant } from 'livekit-client';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { useLang } from '@/contexts';
import { HAND_TOPIC } from './useRaisedHands';

const COPY = {
  VN: {
    raise: 'Giơ tay',
    lower: 'Hạ tay',
    ariaRaise: 'Giơ tay xin phát biểu',
    ariaLower: 'Hạ tay',
    hostLowered: 'Giáo viên đã hạ tay bạn',
  },
  EN: {
    raise: 'Raise hand',
    lower: 'Lower hand',
    ariaRaise: 'Raise your hand',
    ariaLower: 'Lower your hand',
    hostLowered: 'Your tutor lowered your hand',
  },
} as const;

type RoomRole = 'tutor' | 'admin' | 'student';

function parseRole(metadata: string | undefined): RoomRole {
  if (!metadata) return 'student';
  try {
    const parsed = JSON.parse(metadata) as { role?: unknown };
    return parsed.role === 'tutor' || parsed.role === 'admin' ? parsed.role : 'student';
  } catch {
    return 'student';
  }
}

/**
 * Student-only floating pill that publishes a hand-raise on `vielang.hand`.
 *
 * Local state mirrors the last thing we published — no round-trip through
 * the data channel — so the pill feels instant. If the host force-lowers
 * (broadcast `{ lower: myIdentity }`), we listen for it and drop back to
 * the un-raised state with a toast so the student knows the tutor
 * responded to their signal.
 *
 * Hidden for tutors and admins; they see the queue in HostControlsPanel.
 */
export function RaiseHandButton() {
  const { lang } = useLang();
  const lc = COPY[lang];
  const room = useRoomContext();
  const { localParticipant } = useLocalParticipant();
  const [raised, setRaised] = useState(false);

  const role = parseRole(localParticipant?.metadata);
  const isStudent = role === 'student';

  // Listen for host-issued lower targeted at us. Also drops the pill when
  // we disconnect + reconnect (fresh room = fresh state) — the useEffect
  // cleanup handles that implicitly.
  useEffect(() => {
    if (!isStudent) return;
    const handler = (
      payload: Uint8Array,
      _participant?: RemoteParticipant,
      _kind?: unknown,
      topic?: string,
    ) => {
      if (topic !== HAND_TOPIC) return;
      try {
        const decoded = JSON.parse(new TextDecoder().decode(payload)) as { lower?: string };
        if (decoded.lower && decoded.lower === localParticipant?.identity) {
          setRaised(false);
          toast(lc.hostLowered, { duration: 2400 });
        }
      } catch {
        /* malformed — ignore */
      }
    };
    room.on(RoomEvent.DataReceived, handler);
    return () => {
      room.off(RoomEvent.DataReceived, handler);
    };
  }, [room, isStudent, localParticipant, lc.hostLowered]);

  const toggle = useCallback(async () => {
    const next = !raised;
    setRaised(next);
    try {
      // Include our identity in the payload so the host can attribute the
      // raise even if their local remoteParticipants map hasn't populated
      // us yet (participant on DataReceived is sometimes undefined during
      // that window).
      const payload = new TextEncoder().encode(
        JSON.stringify({ raised: next, from: localParticipant?.identity }),
      );
      // Reliable delivery — a dropped raise is a UX bug (student thinks
      // they signalled but the tutor never sees them).
      await room.localParticipant.publishData(payload, { reliable: true, topic: HAND_TOPIC });
    } catch {
      // Revert on failure so the button state matches what the host actually
      // sees. Silent — a raise-hand isn't worth a toast for a transient blip.
      setRaised(!next);
    }
  }, [raised, room, localParticipant]);

  if (!isStudent) return null;

  return (
    <Button
      variant={raised ? 'default' : 'secondary'}
      onClick={toggle}
      className={`fixed bottom-40 left-4 z-40 rounded-full shadow-lg sm:left-6 ${
        raised ? 'bg-amber-500 text-white hover:bg-amber-600' : ''
      }`}
      aria-label={raised ? lc.ariaLower : lc.ariaRaise}
      aria-pressed={raised}
    >
      <Hand className={raised ? 'animate-pulse' : ''} />
      <span className="hidden sm:inline">{raised ? lc.lower : lc.raise}</span>
    </Button>
  );
}
