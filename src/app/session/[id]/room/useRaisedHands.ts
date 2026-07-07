'use client';

import { useCallback, useEffect, useState } from 'react';
import { useRoomContext } from '@livekit/components-react';
import { RoomEvent, type RemoteParticipant } from 'livekit-client';

/**
 * Raise-hand protocol.
 *
 * Topic: `vielang.hand`. Payload is one of:
 *
 *   { raised: true }             — from student, they want the floor
 *   { raised: false }            — from student, lowering their own hand
 *   { lower: <identity> }        — from host, force-lower target's hand
 *
 * Broadcast + lossy delivery. We track FIFO order via a Date.now()
 * timestamp on receipt so the host queue always shows the earliest
 * unresolved hand first. A duplicate raise from the same participant
 * refreshes the timestamp — treating repeats as re-raises matches user
 * intent ("did you see me?") without complicating the model.
 */
export const HAND_TOPIC = 'vielang.hand';

interface HandPayload {
  raised?: boolean;
  lower?: string;
  /**
   * Sender identity. LiveKit's DataReceived event resolves the sending
   * `participant` from the packet's identity string, but in practice we
   * sometimes see it fire with `participant: undefined` while the receiver
   * is still populating its remoteParticipants map — the packet arrives on
   * the wire before the ParticipantConnected event lands. Including the
   * identity in the payload makes attribution independent of that race.
   */
  from?: string;
}

/**
 * Host-only view: which remote participants currently have their hand up,
 * sorted by raise timestamp so the queue is FIFO. Non-hosts get an empty
 * map — they don't need to render the queue.
 *
 * Returns a stable object with `raised` (Map) and `lower(identity)` (fn to
 * publish a host-issued lower). Consumers can iterate `[...raised.keys()]`
 * to render the ordered queue.
 */
export function useRaisedHands(): {
  raised: Map<string, number>;
  lower: (identity: string) => void;
} {
  const room = useRoomContext();
  const [raised, setRaised] = useState<Map<string, number>>(() => new Map());

  useEffect(() => {
    const handler = (
      payload: Uint8Array,
      participant?: RemoteParticipant,
      _kind?: unknown,
      topic?: string,
    ) => {
      if (topic !== HAND_TOPIC) return;
      let decoded: HandPayload;
      try {
        decoded = JSON.parse(new TextDecoder().decode(payload)) as HandPayload;
      } catch {
        return;
      }
      // Host-issued lower — everyone drops the target from their view.
      if (typeof decoded.lower === 'string') {
        setRaised((prev) => {
          if (!prev.has(decoded.lower!)) return prev;
          const next = new Map(prev);
          next.delete(decoded.lower!);
          return next;
        });
        return;
      }
      // Prefer LiveKit's resolved participant, fall back to the sender-
      // supplied `from` field. See HandPayload.from for why.
      const id = participant?.identity ?? decoded.from;
      if (!id) return;
      if (decoded.raised === true) {
        setRaised((prev) => {
          const next = new Map(prev);
          next.set(id, Date.now());
          return next;
        });
      } else if (decoded.raised === false) {
        setRaised((prev) => {
          if (!prev.has(id)) return prev;
          const next = new Map(prev);
          next.delete(id);
          return next;
        });
      }
    };
    room.on(RoomEvent.DataReceived, handler);
    // Also clean up hand state when a participant disconnects — a raised
    // student who drops out shouldn't linger in the host queue forever.
    const onLeave = (p: RemoteParticipant) => {
      setRaised((prev) => {
        if (!prev.has(p.identity)) return prev;
        const next = new Map(prev);
        next.delete(p.identity);
        return next;
      });
    };
    room.on(RoomEvent.ParticipantDisconnected, onLeave);
    return () => {
      room.off(RoomEvent.DataReceived, handler);
      room.off(RoomEvent.ParticipantDisconnected, onLeave);
    };
  }, [room]);

  const lower = useCallback(
    (identity: string) => {
      const payload = new TextEncoder().encode(JSON.stringify({ lower: identity } as HandPayload));
      // Reliable — a lost lower would strand the queue.
      void room.localParticipant.publishData(payload, { reliable: true, topic: HAND_TOPIC });
      // Also optimistically drop from the local view.
      setRaised((prev) => {
        if (!prev.has(identity)) return prev;
        const next = new Map(prev);
        next.delete(identity);
        return next;
      });
    },
    [room],
  );

  return { raised, lower };
}
