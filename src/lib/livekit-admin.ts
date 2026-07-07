import {
  RoomServiceClient,
  type ParticipantInfo,
  type ParticipantPermission,
} from 'livekit-server-sdk';

/**
 * Server-side RoomServiceClient wrapper used by our moderation endpoints
 * (mute / remove / update permissions). Distinct from the client `Room`
 * class — this talks to LiveKit's HTTP admin API using the same API key +
 * secret as the token endpoint, so any operator with the env vars set
 * already has moderation working out of the box.
 */

// Prefer a dedicated LIVEKIT_HTTP_URL for operators who put LiveKit behind an
// internal DNS. Otherwise derive from the public WS URL by swapping the scheme.
function resolveHttpUrl(): string | null {
  const explicit = process.env.LIVEKIT_HTTP_URL;
  if (explicit) return explicit;
  const ws = process.env.NEXT_PUBLIC_LIVEKIT_WS_URL;
  if (!ws) return null;
  if (ws.startsWith('wss://')) return 'https://' + ws.slice('wss://'.length);
  if (ws.startsWith('ws://')) return 'http://' + ws.slice('ws://'.length);
  return ws;
}

let cached: RoomServiceClient | null = null;

export function getRoomService(): RoomServiceClient | null {
  if (cached) return cached;
  const host = resolveHttpUrl();
  const apiKey = process.env.LIVEKIT_API_KEY;
  const apiSecret = process.env.LIVEKIT_API_SECRET;
  if (!host || !apiKey || !apiSecret) return null;
  cached = new RoomServiceClient(host, apiKey, apiSecret);
  return cached;
}

/**
 * Mute or unmute a published track. Requires the track SID which the UI
 * gets from LiveKit's participant.trackPublications map.
 */
export async function mutePublishedTrack(
  room: string,
  identity: string,
  trackSid: string,
  muted: boolean,
): Promise<void> {
  const svc = getRoomService();
  if (!svc) throw new Error('livekit_not_configured');
  await svc.mutePublishedTrack(room, identity, trackSid, muted);
}

/**
 * Force-disconnect a participant. LiveKit emits a Disconnected event with
 * a "removed by admin" reason — the target client sees a red banner.
 * The target can still re-join the room if they refresh; combine with a
 * DB flag or session_participants delete when we need a hard ban.
 */
export async function removeParticipant(room: string, identity: string): Promise<void> {
  const svc = getRoomService();
  if (!svc) throw new Error('livekit_not_configured');
  await svc.removeParticipant(room, identity);
}

/**
 * Atomically update a participant's metadata + permissions. Used to admit a
 * waiting-room student: they enter the room with `canPublish=false,
 * canSubscribe=false, state=waiting`, and this call lifts them into
 * `state=admitted` with normal publish/subscribe grants. LiveKit fires
 * `ParticipantPermissionsChanged` on the target client so their UI can
 * swap the waiting overlay for the video conference view without a full
 * reconnect.
 */
export async function updateParticipantAccess(
  room: string,
  identity: string,
  options: { metadata?: string; permission?: Partial<ParticipantPermission> },
): Promise<void> {
  const svc = getRoomService();
  if (!svc) throw new Error('livekit_not_configured');
  await svc.updateParticipant(room, identity, options);
}

/**
 * Best-effort probe: does this identity currently have a connection in the
 * LiveKit room? Used by moderation endpoints to distinguish "student left"
 * (benign no-op) from "LiveKit is broken" (bubble error).
 *
 * The SDK throws when the participant doesn't exist; we swallow that as the
 * negative answer and only rethrow if the call itself failed (network,
 * auth, etc.).
 */
export async function participantExists(room: string, identity: string): Promise<boolean> {
  const svc = getRoomService();
  if (!svc) throw new Error('livekit_not_configured');
  try {
    await svc.getParticipant(room, identity);
    return true;
  } catch (err) {
    if (isParticipantNotFoundError(err)) return false;
    throw err;
  }
}

/**
 * List every participant currently connected to a LiveKit room. Powers the
 * "mute all" bulk action — the host UI doesn't have every participant's
 * track SIDs in scope, so the endpoint pulls the authoritative list from
 * LiveKit and iterates.
 */
export async function listRoomParticipants(room: string): Promise<ParticipantInfo[]> {
  const svc = getRoomService();
  if (!svc) throw new Error('livekit_not_configured');
  return svc.listParticipants(room);
}

/**
 * Delete a LiveKit room. Every connected client disconnects with reason
 * `ROOM_DELETED` and LiveKit fires a `room_finished` webhook. Our webhook
 * handler bumps `sessions.status` to `completed` and closes any open
 * attendance rows, so this is the atomic "end for all" primitive.
 */
export async function deleteRoom(room: string): Promise<void> {
  const svc = getRoomService();
  if (!svc) throw new Error('livekit_not_configured');
  await svc.deleteRoom(room);
}

/**
 * LiveKit's Twirp error format for a missing participant is
 * `"twirp error unknown: participant does not exist"`. The grpc-node
 * variant is `"5 NOT_FOUND: ..."`. Match both without pinning the exact
 * text so a server upgrade doesn't break our probe.
 */
export function isParticipantNotFoundError(err: unknown): boolean {
  const msg = (err as { message?: string })?.message ?? '';
  return /does not exist|not[_\s-]?found/i.test(msg);
}
