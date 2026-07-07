'use client';

import { useEffect, useMemo, useRef, useState } from 'react';
import {
  Shield,
  MicOff,
  Mic,
  UserMinus,
  Loader2,
  UserCheck,
  UserX,
  MicOff as MicOffIcon,
  DoorClosed,
  Hand,
} from 'lucide-react';
import { toast } from 'sonner';
import { useLocalParticipant, useRemoteParticipants } from '@livekit/components-react';
import { Track, type RemoteParticipant } from 'livekit-client';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { useLang } from '@/contexts';
import type { Lang } from '@/contexts/LangContext';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet';
import { useRaisedHands } from './useRaisedHands';

const COPY = {
  VN: {
    title: 'Điều khiển của giáo viên',
    participantsCount: (n: number) => `${n} người tham gia`,
    waitingSection: (n: number) => `Đang chờ vào · ${n}`,
    inSessionSection: (n: number) => `Trong buổi học · ${n}`,
    roomActionsSection: 'Hành động lớp',
    handRaised: 'Đang giơ tay',
    lowerHand: 'Hạ tay',
    muteAll: 'Tắt mic tất cả',
    endForAll: 'Kết thúc cho mọi người',
    noneYet: 'Chưa có ai khác vào.',
    requestingToJoin: 'Đang xin vào',
    role: { tutor: 'giáo viên', admin: 'quản trị', student: 'học viên' },
    micOn: 'micro đang bật',
    micOff: 'micro đang tắt',
    openHost: 'Mở điều khiển giáo viên',
    admit: 'Cho vào',
    cancel: 'Huỷ',
    confirm: 'Xác nhận',
    endConfirmTitle: 'Kết thúc buổi học cho mọi người?',
    endConfirmDesc:
      'Tất cả người tham gia sẽ bị ngắt kết nối ngay lập tức. Buổi học sẽ được đánh dấu là đã hoàn thành.',
    endConfirm: 'Kết thúc',
    muteAllConfirmTitle: 'Tắt mic tất cả?',
    muteAllConfirmDesc:
      'Micro của mọi người trong lớp sẽ bị tắt. Họ có thể bật lại nếu muốn — đây không phải tắt vĩnh viễn.',
    tooltip: {
      noMic: 'Chưa có micro nào phát',
      unmute: 'Bật lại mic',
      mute: 'Tắt mic',
      deny: 'Từ chối yêu cầu',
      remove: 'Xoá khỏi buổi học',
      lower: 'Hạ tay',
    },
    aria: {
      deny: (n: string) => `Từ chối ${n}`,
      unmuteOf: (n: string) => `Bật lại mic của ${n}`,
      muteOf: (n: string) => `Tắt mic của ${n}`,
      removeOf: (n: string) => `Xoá ${n} khỏi buổi học`,
      waitingLi: (n: string) => `${n} đang xin vào`,
      lowerHandOf: (n: string) => `Hạ tay của ${n}`,
    },
    toast: {
      muted: (n: string) => `Đã tắt mic của ${n}`,
      unmuted: (n: string) => `Đã bật mic của ${n}`,
      removed: (n: string) => `Đã xoá ${n} khỏi buổi học`,
      admitted: (n: string) => `Đã cho ${n} vào`,
      declined: (n: string) => `Đã từ chối ${n}`,
      muteAllOk: (n: number) => `Đã tắt mic của ${n} người`,
      endOk: 'Buổi học đã kết thúc',
      failed: 'Hành động thất bại',
    },
  },
  EN: {
    title: 'Host controls',
    participantsCount: (n: number) => `${n} ${n === 1 ? 'participant' : 'participants'}`,
    waitingSection: (n: number) => `Waiting to join · ${n}`,
    inSessionSection: (n: number) => `In session · ${n}`,
    roomActionsSection: 'Room actions',
    handRaised: 'Hand raised',
    lowerHand: 'Lower hand',
    muteAll: 'Mute all',
    endForAll: 'End for everyone',
    noneYet: 'No one else has joined yet.',
    requestingToJoin: 'Requesting to join',
    role: { tutor: 'tutor', admin: 'admin', student: 'student' },
    micOn: 'microphone on',
    micOff: 'microphone off',
    openHost: 'Open host controls',
    admit: 'Admit',
    cancel: 'Cancel',
    confirm: 'Confirm',
    endConfirmTitle: 'End the session for everyone?',
    endConfirmDesc:
      'Every participant will be disconnected immediately. The session will be marked as completed.',
    endConfirm: 'End session',
    muteAllConfirmTitle: 'Mute everyone?',
    muteAllConfirmDesc:
      "Everyone in the room will have their mic muted. They can unmute themselves — this isn't a permanent silence.",
    tooltip: {
      noMic: 'No microphone published yet',
      unmute: 'Unmute mic',
      mute: 'Mute mic',
      deny: 'Deny request',
      remove: 'Remove from session',
      lower: 'Lower hand',
    },
    aria: {
      deny: (n: string) => `Deny ${n}`,
      unmuteOf: (n: string) => `Unmute microphone of ${n}`,
      muteOf: (n: string) => `Mute microphone of ${n}`,
      removeOf: (n: string) => `Remove ${n} from session`,
      waitingLi: (n: string) => `${n} is waiting to join`,
      lowerHandOf: (n: string) => `Lower ${n}'s hand`,
    },
    toast: {
      muted: (n: string) => `Muted ${n}`,
      unmuted: (n: string) => `Unmuted ${n}`,
      removed: (n: string) => `Removed ${n} from the session`,
      admitted: (n: string) => `Admitted ${n}`,
      declined: (n: string) => `Declined ${n}`,
      muteAllOk: (n: number) => `Muted ${n} ${n === 1 ? 'participant' : 'participants'}`,
      endOk: 'Session ended',
      failed: 'Action failed',
    },
  },
} as const;

type RoomRole = 'tutor' | 'admin' | 'student';
type AdmissionState = 'admitted' | 'waiting';

interface ParticipantMeta {
  role: RoomRole;
  state: AdmissionState;
}

function parseMeta(metadata: string | undefined): ParticipantMeta {
  if (!metadata) return { role: 'student', state: 'admitted' };
  try {
    const parsed = JSON.parse(metadata) as Partial<ParticipantMeta>;
    return {
      role: parsed.role === 'tutor' || parsed.role === 'admin' ? parsed.role : 'student',
      state: parsed.state === 'waiting' ? 'waiting' : 'admitted',
    };
  } catch {
    return { role: 'student', state: 'admitted' };
  }
}

// Track which target + action is currently in flight — we lock the pair so a
// double-click doesn't fire two POSTs, but leave other participants clickable.
interface PendingState {
  identity: string;
  action: 'mute' | 'unmute' | 'remove' | 'admit' | 'deny';
}

/**
 * Moderation drawer for tutors and admins.
 *
 * Renders nothing at all for students — the trigger button never appears in
 * their DOM. Server-side, /api/livekit/moderate and /api/livekit/admit
 * enforce the same policy, so bypassing this component (e.g. via devtools)
 * doesn't grant a student any moderation power.
 *
 * The drawer splits remote participants into two sections:
 *   • Waiting to join — students whose LiveKit metadata says state='waiting'.
 *     Only the Admit / Deny actions apply; they haven't published anything
 *     to mute yet and removing them would deny them without an audit trail.
 *   • In session — everyone else. Mic mute/unmute + remove.
 *
 * Every action fires a small POST to a moderation endpoint. On success we
 * rely on LiveKit's server events (ParticipantPermissionsChanged, etc.) to
 * propagate the state change back to every client — no optimistic UI here,
 * so a failed request never leaves the panel out of sync with the room.
 */
export function HostControlsPanel({ sessionId }: { sessionId: string }) {
  const { lang } = useLang();
  const lc = COPY[lang];
  const { localParticipant } = useLocalParticipant();
  const remotes = useRemoteParticipants();
  const { raised: raisedHands, lower: lowerHand } = useRaisedHands();
  const [open, setOpen] = useState(false);
  const [pending, setPending] = useState<PendingState | null>(null);
  const [confirmRemove, setConfirmRemove] = useState<string | null>(null);
  const [confirmMuteAll, setConfirmMuteAll] = useState(false);
  const [confirmEnd, setConfirmEnd] = useState(false);
  const [roomActionPending, setRoomActionPending] = useState<'mute_all' | 'end_room' | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Don't memoize on localParticipant identity — livekit-client mutates the
  // same object across re-renders, so the identity never changes even when
  // metadata later flips from undefined to '{"role":"tutor",...}'. Compute
  // on every render; parseMeta is a JSON.parse of a short string, cheap.
  const localRole = parseMeta(localParticipant?.metadata).role;
  const isHost = localRole === 'tutor' || localRole === 'admin';

  const { waiting, admitted } = useMemo(() => {
    const w: RemoteParticipant[] = [];
    const a: RemoteParticipant[] = [];
    for (const p of remotes) {
      (parseMeta(p.metadata).state === 'waiting' ? w : a).push(p);
    }
    // Sort admitted so raised hands come first, FIFO by raise timestamp.
    // Others keep their natural order (LiveKit's join order).
    a.sort((x, y) => {
      const rx = raisedHands.get(x.identity);
      const ry = raisedHands.get(y.identity);
      if (rx && ry) return rx - ry;
      if (rx) return -1;
      if (ry) return 1;
      return 0;
    });
    return { waiting: w, admitted: a };
  }, [remotes, raisedHands]);

  if (!isHost) return null;

  const post = async (
    url: string,
    body: Record<string, unknown>,
    action: PendingState['action'],
    identity: string,
    successToast: string,
  ) => {
    setError(null);
    setPending({ identity, action });
    try {
      const res = await fetch(url, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify(body),
      });
      if (!res.ok) {
        const data = (await res.json().catch(() => ({}))) as { error?: string; detail?: string };
        const msg = data.detail || data.error || `HTTP ${res.status}`;
        setError(msg);
        toast.error(lc.toast.failed, { description: msg });
      } else {
        toast.success(successToast);
      }
    } catch (err) {
      const msg = (err as Error)?.message || 'Network error';
      setError(msg);
      toast.error(lc.toast.failed, { description: msg });
    } finally {
      setPending(null);
      setConfirmRemove(null);
    }
  };

  const moderate = (
    target: RemoteParticipant,
    action: 'mute' | 'unmute' | 'remove',
    trackSid?: string,
  ) => {
    const name = target.name || target.identity;
    const successToast =
      action === 'mute'
        ? lc.toast.muted(name)
        : action === 'unmute'
          ? lc.toast.unmuted(name)
          : lc.toast.removed(name);
    return post(
      '/api/livekit/moderate',
      { session_id: sessionId, target_identity: target.identity, action, track_sid: trackSid },
      action,
      target.identity,
      successToast,
    );
  };

  const decide = (target: RemoteParticipant, decision: 'admit' | 'deny') => {
    const name = target.name || target.identity;
    const successToast = decision === 'admit' ? lc.toast.admitted(name) : lc.toast.declined(name);
    return post(
      '/api/livekit/admit',
      { session_id: sessionId, target_identity: target.identity, decision },
      decision,
      target.identity,
      successToast,
    );
  };

  /**
   * Fire a whole-room action. Confirmation dialogs are handled at the button
   * level so this helper always runs the destructive path — no double-checks.
   */
  const roomAction = async (action: 'mute_all' | 'end_room') => {
    setError(null);
    setRoomActionPending(action);
    try {
      const res = await fetch('/api/livekit/room-action', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({ session_id: sessionId, action }),
      });
      const data = (await res.json().catch(() => ({}))) as {
        error?: string;
        detail?: string;
        muted?: number;
      };
      if (!res.ok) {
        const msg = data.detail || data.error || `HTTP ${res.status}`;
        setError(msg);
        toast.error(lc.toast.failed, { description: msg });
        return;
      }
      if (action === 'mute_all') {
        toast.success(lc.toast.muteAllOk(data.muted ?? 0));
      } else {
        toast.success(lc.toast.endOk);
      }
    } catch (err) {
      const msg = (err as Error)?.message || 'Network error';
      setError(msg);
      toast.error(lc.toast.failed, { description: msg });
    } finally {
      setRoomActionPending(null);
      setConfirmMuteAll(false);
      setConfirmEnd(false);
    }
  };

  return (
    <>
      {/* Controlled trigger (not SheetTrigger) so aria-label + click handler
          are simple props on our own Button — SheetTrigger's render-prop merge
          plus base-ui's own onClick made Playwright's role-based query flaky. */}
      <Button
        variant="default"
        onClick={() => setOpen(true)}
        className="fixed right-4 bottom-24 z-40 rounded-full shadow-lg sm:right-6"
        aria-label={lc.openHost}
      >
        <Shield />
        <span className="hidden sm:inline">{lang === 'VN' ? 'Giáo viên' : 'Host'}</span>
        {remotes.length > 0 && (
          <span
            className={`ml-1 rounded-full px-1.5 py-0.5 text-[10px] tabular-nums ${
              waiting.length > 0 ? 'bg-amber-400 text-amber-950' : 'bg-white/20'
            }`}
          >
            {waiting.length > 0 ? `${waiting.length}!` : remotes.length}
          </span>
        )}
      </Button>

      <Sheet open={open} onOpenChange={setOpen}>
        <SheetContent side="right" className="flex flex-col gap-0 p-0">
          <SheetHeader className="border-border shrink-0 flex-row items-center gap-3 border-b px-5 py-4">
            <Shield className="text-brand size-4" aria-hidden />
            <div className="min-w-0 flex-1">
              <SheetTitle>{lc.title}</SheetTitle>
              <SheetDescription>{lc.participantsCount(remotes.length)}</SheetDescription>
            </div>
          </SheetHeader>

          {error && (
            <Alert variant="destructive" className="mx-5 mt-4">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <ScrollArea className="flex-1">
            <section aria-labelledby="room-actions-heading" className="border-border border-b">
              <h3
                id="room-actions-heading"
                className="border-border bg-muted text-muted-foreground border-b px-5 py-2 text-[11px] font-semibold tracking-wide uppercase"
              >
                {lc.roomActionsSection}
              </h3>
              <div className="flex flex-wrap gap-2 px-5 py-3">
                <Button
                  variant="outline"
                  size="sm"
                  onClick={() => setConfirmMuteAll(true)}
                  disabled={roomActionPending !== null || remotes.length === 0}
                >
                  {roomActionPending === 'mute_all' ? (
                    <Loader2 className="animate-spin" />
                  ) : (
                    <MicOffIcon />
                  )}
                  {lc.muteAll}
                </Button>
                <Button
                  variant="destructive"
                  size="sm"
                  onClick={() => setConfirmEnd(true)}
                  disabled={roomActionPending !== null}
                >
                  {roomActionPending === 'end_room' ? (
                    <Loader2 className="animate-spin" />
                  ) : (
                    <DoorClosed />
                  )}
                  {lc.endForAll}
                </Button>
              </div>
            </section>

            {waiting.length > 0 && (
              <section aria-labelledby="waiting-heading">
                <h3
                  id="waiting-heading"
                  className="sticky top-0 z-10 border-b border-amber-200 bg-amber-50 px-5 py-2 text-[11px] font-semibold tracking-wide text-amber-900 uppercase dark:border-amber-900/60 dark:bg-amber-950/40 dark:text-amber-200"
                >
                  {lc.waitingSection(waiting.length)}
                </h3>
                <ul className="divide-border divide-y">
                  {waiting.map((p) => (
                    <WaitingRow
                      key={p.identity}
                      lang={lang}
                      participant={p}
                      pending={pending?.identity === p.identity ? pending.action : null}
                      onAdmit={() => decide(p, 'admit')}
                      onDeny={() => decide(p, 'deny')}
                    />
                  ))}
                </ul>
              </section>
            )}

            <section aria-labelledby="admitted-heading">
              {waiting.length > 0 && (
                <h3
                  id="admitted-heading"
                  className="border-border bg-muted text-muted-foreground sticky top-0 z-10 border-b px-5 py-2 text-[11px] font-semibold tracking-wide uppercase"
                >
                  {lc.inSessionSection(admitted.length)}
                </h3>
              )}
              <ul className="divide-border divide-y">
                {remotes.length === 0 && (
                  <li className="text-muted-foreground px-5 py-8 text-center text-xs">
                    {lc.noneYet}
                  </li>
                )}
                {admitted.map((p) => (
                  <ParticipantRow
                    key={p.identity}
                    lang={lang}
                    participant={p}
                    handRaised={raisedHands.has(p.identity)}
                    pending={pending?.identity === p.identity ? pending.action : null}
                    confirming={confirmRemove === p.identity}
                    onMuteToggle={(trackSid, muted) =>
                      moderate(p, muted ? 'unmute' : 'mute', trackSid)
                    }
                    onLowerHand={() => lowerHand(p.identity)}
                    onRequestRemove={() => setConfirmRemove(p.identity)}
                    onCancelRemove={() => setConfirmRemove(null)}
                    onConfirmRemove={() => moderate(p, 'remove')}
                  />
                ))}
              </ul>
            </section>
          </ScrollArea>
        </SheetContent>
      </Sheet>

      <AlertDialog open={confirmMuteAll} onOpenChange={setConfirmMuteAll}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{lc.muteAllConfirmTitle}</AlertDialogTitle>
            <AlertDialogDescription>{lc.muteAllConfirmDesc}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{lc.cancel}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void roomAction('mute_all')}>
              {lc.muteAll}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog open={confirmEnd} onOpenChange={setConfirmEnd}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{lc.endConfirmTitle}</AlertDialogTitle>
            <AlertDialogDescription>{lc.endConfirmDesc}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{lc.cancel}</AlertDialogCancel>
            <AlertDialogAction onClick={() => void roomAction('end_room')}>
              {lc.endConfirm}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}

/**
 * Lower-hand icon button. Wrapped in its own component with a mount-time
 * guard because base-ui's Button primitive (via shadcn) fires an onClick
 * dispatched by React on the very first render — presumably an
 * accessibility-related synthetic event during hydration. Without the
 * guard, a fresh mount (when handRaised flips true) auto-invokes
 * `onLower`, immediately dropping the hand the student just raised.
 */
function LowerHandButton({
  onLower,
  label,
  tooltip,
}: {
  onLower: () => void;
  label: string;
  tooltip: string;
}) {
  // Bind the click handler imperatively after mount rather than via the
  // onClick prop. Some combination of base-ui / React 19 dev-mode strict
  // mounting fires a synthetic click on the first render when this button
  // is inside a Dialog Popup — with the JSX onClick binding, that click
  // reaches our handler and immediately lowers the hand the student just
  // raised. Attaching listeners in useEffect side-steps the synthetic
  // dispatch because it only observes real DOM click events fired after
  // the effect runs.
  const btnRef = useRef<HTMLButtonElement | null>(null);
  useEffect(() => {
    const el = btnRef.current;
    if (!el) return;
    const handler = () => onLower();
    el.addEventListener('click', handler);
    return () => el.removeEventListener('click', handler);
  }, [onLower]);
  return (
    <button
      ref={btnRef}
      type="button"
      aria-label={label}
      title={tooltip}
      className="focus-ring inline-flex size-9 items-center justify-center rounded-lg border border-amber-500/40 text-amber-700 hover:bg-amber-500/10 hover:text-amber-800 dark:text-amber-300 dark:hover:text-amber-200"
    >
      <Hand className="size-4" />
    </button>
  );
}

function WaitingRow({
  lang,
  participant,
  pending,
  onAdmit,
  onDeny,
}: {
  lang: Lang;
  participant: RemoteParticipant;
  pending: PendingState['action'] | null;
  onAdmit: () => void;
  onDeny: () => void;
}) {
  const lc = COPY[lang];
  const displayName = participant.name || participant.identity;
  const busy = pending === 'admit' || pending === 'deny';
  return (
    <li className="flex items-center gap-3 px-5 py-3" aria-label={lc.aria.waitingLi(displayName)}>
      <div
        className="flex size-9 shrink-0 items-center justify-center rounded-full bg-amber-100 text-xs font-semibold text-amber-800 uppercase dark:bg-amber-950/40 dark:text-amber-200"
        aria-hidden
      >
        {displayName.slice(0, 1)}
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-slate-900 dark:text-slate-100">
          {displayName}
        </p>
        <p className="type-eyebrow">{lc.requestingToJoin}</p>
      </div>
      <div className="flex items-center gap-1.5">
        <Button
          variant="outline"
          size="icon-sm"
          onClick={onDeny}
          disabled={busy}
          aria-label={lc.aria.deny(displayName)}
          title={lc.tooltip.deny}
          className="border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
        >
          {pending === 'deny' ? <Loader2 className="animate-spin" /> : <UserX />}
        </Button>
        <Button size="sm" onClick={onAdmit} disabled={busy}>
          {pending === 'admit' ? <Loader2 className="animate-spin" /> : <UserCheck />}
          {lc.admit}
        </Button>
      </div>
    </li>
  );
}

function ParticipantRow({
  lang,
  participant,
  handRaised,
  pending,
  confirming,
  onMuteToggle,
  onLowerHand,
  onRequestRemove,
  onCancelRemove,
  onConfirmRemove,
}: {
  lang: Lang;
  participant: RemoteParticipant;
  handRaised: boolean;
  pending: PendingState['action'] | null;
  confirming: boolean;
  onMuteToggle: (trackSid: string, currentlyMuted: boolean) => void;
  onLowerHand: () => void;
  onRequestRemove: () => void;
  onCancelRemove: () => void;
  onConfirmRemove: () => void;
}) {
  const lc = COPY[lang];
  const meta = parseMeta(participant.metadata);
  const mic = participant.getTrackPublication(Track.Source.Microphone);
  const micSid = mic?.trackSid;
  const micMuted = mic?.isMuted ?? true;
  const displayName = participant.name || participant.identity;
  const roleLabel = lc.role[meta.role];
  const micLabel = micMuted ? lc.micOff : lc.micOn;
  const rowLabel = handRaised
    ? `${displayName}, ${roleLabel}, ${micLabel}, ${lc.handRaised}`
    : `${displayName}, ${roleLabel}, ${micLabel}`;

  return (
    <li
      className={`flex items-center gap-3 px-5 py-3 ${handRaised ? 'bg-amber-50 dark:bg-amber-950/20' : ''}`}
      aria-label={rowLabel}
    >
      <div
        className="bg-brand/10 text-brand flex size-9 shrink-0 items-center justify-center rounded-full text-xs font-semibold uppercase"
        aria-hidden
      >
        {displayName.slice(0, 1)}
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-slate-900 dark:text-slate-100">
          {displayName}
        </p>
        <p className="type-eyebrow flex items-center gap-1.5">
          {meta.role !== 'student' && (
            <Badge variant="secondary" className="h-4 px-1.5 text-[10px] uppercase">
              {roleLabel}
            </Badge>
          )}
          {handRaised && (
            <Badge className="h-4 gap-1 bg-amber-500 px-1.5 text-[10px] text-white uppercase">
              <Hand className="size-2.5" aria-hidden />
              {lc.handRaised}
            </Badge>
          )}
          {micMuted ? (
            <MicOff className="size-3" aria-label={lc.micOff} />
          ) : (
            <Mic className="size-3" aria-label={lc.micOn} />
          )}
        </p>
      </div>

      {confirming ? (
        <div className="flex items-center gap-1.5">
          <Button variant="ghost" size="sm" onClick={onCancelRemove}>
            {lc.cancel}
          </Button>
          <Button
            variant="destructive"
            size="sm"
            onClick={onConfirmRemove}
            disabled={pending === 'remove'}
          >
            {pending === 'remove' && <Loader2 className="animate-spin" />}
            {lc.confirm}
          </Button>
        </div>
      ) : (
        <div className="flex items-center gap-1.5">
          {handRaised && (
            <LowerHandButton
              onLower={onLowerHand}
              label={lc.aria.lowerHandOf(displayName)}
              tooltip={lc.tooltip.lower}
            />
          )}
          <Button
            variant="outline"
            size="icon-sm"
            onClick={() => micSid && onMuteToggle(micSid, micMuted)}
            disabled={!micSid || pending === 'mute' || pending === 'unmute'}
            title={!micSid ? lc.tooltip.noMic : micMuted ? lc.tooltip.unmute : lc.tooltip.mute}
            aria-label={micMuted ? lc.aria.unmuteOf(displayName) : lc.aria.muteOf(displayName)}
          >
            {pending === 'mute' || pending === 'unmute' ? (
              <Loader2 className="animate-spin" />
            ) : micMuted ? (
              <Mic />
            ) : (
              <MicOff />
            )}
          </Button>
          <Button
            variant="outline"
            size="icon-sm"
            onClick={onRequestRemove}
            aria-label={lc.aria.removeOf(displayName)}
            title={lc.tooltip.remove}
            className="border-destructive/40 text-destructive hover:bg-destructive/10 hover:text-destructive"
          >
            <UserMinus />
          </Button>
        </div>
      )}
    </li>
  );
}
