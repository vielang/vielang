'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import {
  Loader2,
  VideoOff,
  ArrowLeft,
  LogOut,
  Star,
  RotateCcw,
  ShieldOff,
  UserX,
  DoorClosed,
  WifiOff,
} from 'lucide-react';
import {
  LiveKitRoom,
  VideoConference,
  RoomAudioRenderer,
  useLocalParticipantPermissions,
} from '@livekit/components-react';
import type { LocalUserChoices } from '@livekit/components-core';
import { DisconnectReason } from 'livekit-client';
import '@livekit/components-styles';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { ReviewDialog } from '@/components/session/ReviewDialog';
import { Button } from '@/components/ui/button';
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
import { TerminalScreen } from '@/components/session/room/TerminalScreen';
import { PreJoinPanel, type PreJoinContext } from './PreJoinPanel';
import { HostControlsPanel } from './HostControlsPanel';
import { WaitingRoomOverlay } from './WaitingRoomOverlay';
import { SessionChatPanel } from './SessionChatPanel';
import { ConnectionStatusBar } from './ConnectionStatusBar';
import { ReactionsOverlay } from './ReactionsOverlay';
import { ScreenShareButton } from './ScreenShareButton';
import { RoomShortcuts } from './RoomShortcuts';
import { RaiseHandButton } from './RaiseHandButton';
import { useLang } from '@/contexts';
import type { Lang } from '@/contexts/LangContext';

const COPY = {
  VN: {
    setup: 'Đang thiết lập lớp học…',
    errFallbackTitle: 'Không thể vào lớp',
    errFallbackMessage: (raw: string) => raw || 'Lỗi mạng',
    networkError: 'Lỗi mạng',
    backToLobby: 'Về sảnh chờ',
    mySessions: 'Buổi học của tôi',
    backToMySessions: 'Quay lại danh sách buổi học',
    reconnect: 'Kết nối lại',
    liveEyebrow: 'Đang phát trực tiếp',
    defaultTitle: 'Buổi học tiếng Anh',
    withSuffix: (n: string) => ` · với ${n}`,
    endSession: 'Kết thúc',
    confirmLeaveTitle: 'Rời buổi học?',
    confirmLeaveDescBefore: 'Bạn sẽ bị ngắt kết nối video. Có thể vào lại từ ',
    confirmLeaveDescAfter: ' trong khi phiên vẫn còn mở.',
    stay: 'Ở lại',
    leave: 'Rời',
    rotateHint: 'Xoay ngang để có khung video lớn hơn.',
    gotIt: 'Đã hiểu',
    dismissHint: 'Bỏ qua gợi ý xoay',
    denied: {
      title: 'Bạn chưa được vào lớp',
      desc: (n: string) => (
        <>{n} đã từ chối yêu cầu tham gia. Nếu bạn nghĩ có nhầm lẫn, hãy liên hệ trực tiếp.</>
      ),
    },
    kicked: {
      title: 'Bạn đã bị đưa ra khỏi buổi học',
      desc: (n: string) => (
        <>{n} đã kết thúc buổi học với bạn. Nếu có sự cố, nhắn tin để họ mời bạn quay lại.</>
      ),
    },
    sessionEnded: {
      title: 'Buổi học đã kết thúc',
      desc: 'Cảm ơn bạn đã tham gia. Bạn có thể xem lại bản ghi và điểm danh từ Buổi học của tôi khi bản tóm tắt sẵn sàng.',
    },
    connectionLost: {
      title: 'Mất kết nối',
      desc: 'Kết nối tới lớp học bị ngắt. Kiểm tra mạng và thử lại — buổi học có thể vẫn đang diễn ra.',
    },
    review: {
      title: 'Buổi học vừa rồi thế nào?',
      desc: (n: string) => (
        <>
          Dành một chút thời gian đánh giá {n}. Điều đó giúp học viên khác chọn đúng giáo viên và
          giúp {n} tiếp tục hoàn thiện.
        </>
      ),
      leaveReview: 'Viết đánh giá',
      notNow: 'Để sau',
    },
    errors: {
      outside_join_window: {
        title: 'Còn hơi sớm',
        message: 'Lớp học mở 15 phút trước khi buổi học bắt đầu. Quay lại gần giờ hơn nhé!',
      },
      not_a_participant: {
        title: 'Buổi học này thuộc về người khác',
        message:
          'Tài khoản của bạn không phải người tham gia của buổi học này. Kiểm tra lại liên kết.',
      },
      session_not_found: {
        title: 'Không tìm thấy buổi học',
        message: 'Liên kết có thể đã hỏng, hoặc buổi học đã bị huỷ sau khi chia sẻ.',
      },
      session_closed: {
        title: 'Buổi học đã đóng',
        message: 'Buổi học này đã kết thúc, đã huỷ, hoặc bị đánh dấu vắng.',
      },
      admission_denied: {
        title: 'Người tổ chức đã từ chối',
        message:
          'Người tổ chức đã từ chối yêu cầu tham gia của bạn. Nếu bạn nghĩ có nhầm lẫn, liên hệ trực tiếp.',
      },
      session_full: {
        title: 'Buổi học đã đầy',
        message:
          'Không còn chỗ trong phòng. Liên hệ người tổ chức nếu bạn nghĩ mình phải có mặt — họ có thể tăng sức chứa hoặc tạo phòng mới.',
      },
      livekit_not_configured: {
        title: 'Dịch vụ video đang tắt',
        message: 'Máy chủ video hiện không chạy. Thử lại sau ít phút.',
      },
    },
  },
  EN: {
    setup: 'Setting up your classroom…',
    errFallbackTitle: 'Could not join',
    errFallbackMessage: (raw: string) => raw || 'Network error',
    networkError: 'Network error',
    backToLobby: 'Back to lobby',
    mySessions: 'My sessions',
    backToMySessions: 'Back to my sessions',
    reconnect: 'Reconnect',
    liveEyebrow: 'Live session',
    defaultTitle: 'English lesson',
    withSuffix: (n: string) => ` · with ${n}`,
    endSession: 'End session',
    confirmLeaveTitle: 'Leave this session?',
    confirmLeaveDescBefore: "You'll disconnect from the video call. You can rejoin from ",
    confirmLeaveDescAfter: ' while the session window is still open.',
    stay: 'Stay',
    leave: 'Leave',
    rotateHint: 'Rotate for a bigger video tile.',
    gotIt: 'Got it',
    dismissHint: 'Dismiss rotate hint',
    denied: {
      title: "You weren't admitted",
      desc: (n: string) => (
        <>
          {n} declined your request to join this session. If you think that&apos;s a mistake, get in
          touch with them directly.
        </>
      ),
    },
    kicked: {
      title: 'You were removed from the session',
      desc: (n: string) => (
        <>
          {n} ended the session for you. If something went wrong, message them and they can invite
          you back.
        </>
      ),
    },
    sessionEnded: {
      title: 'The session has ended',
      desc: 'Thanks for joining. You can review the transcript and attendance from My sessions once the recap is ready.',
    },
    connectionLost: {
      title: 'Connection lost',
      desc: 'Your connection to the classroom dropped. Check your network and try again — the session may still be running.',
    },
    review: {
      title: 'How was your session?',
      desc: (n: string) => (
        <>
          Take a moment to leave feedback for {n}. It helps other students pick the right tutor and
          helps {n} keep improving.
        </>
      ),
      leaveReview: 'Leave a review',
      notNow: 'Not now',
    },
    errors: {
      outside_join_window: {
        title: "You're a bit early",
        message:
          'The classroom opens 15 minutes before your session starts. Come back closer to the time!',
      },
      not_a_participant: {
        title: 'This session belongs to someone else',
        message:
          "Your account isn't a participant on this booking. Double-check the link you used.",
      },
      session_not_found: {
        title: 'Session not found',
        message: 'The link may be broken, or the session was cancelled after it was shared.',
      },
      session_closed: {
        title: 'Session already closed',
        message: 'This session has already ended, was cancelled, or was marked as a no-show.',
      },
      admission_denied: {
        title: 'Host declined your request',
        message:
          'The host declined your request to join this session. If you believe this is a mistake, reach out to the host directly.',
      },
      session_full: {
        title: 'This session is full',
        message:
          'There are no seats left in this room. Reach out to the host if you think you should be inside — they can bump the capacity or start a fresh group.',
      },
      livekit_not_configured: {
        title: 'Video service is offline',
        message: 'Our video backend is not running right now. Try again shortly.',
      },
    },
  },
} as const;

type ErrorCode = keyof (typeof COPY)['EN']['errors'];

interface RoomContext {
  sessionId: string;
  courseTitle: string | null;
  tutorName: string;
  studentName: string;
  isStudent: boolean;
  isTutor: boolean;
  scheduledAt: string | null;
  durationMin: number | null;
}

type AdmissionState = 'admitted' | 'waiting';

/**
 * Phases:
 *   prejoin        — device preview + countdown, no LiveKit connection yet
 *   connecting     — token fetch inflight
 *   ready          — connected; `initialState` picks between video conference
 *                    and the waiting-room overlay on first paint. Once
 *                    permissions flip on the wire we react to those instead.
 *   error          — token endpoint refused; user is bounced back to lobby
 *   ended          — student voluntarily left an admitted session; review
 *                    prompt shown
 *   denied         — host declined the student's admission (they were in
 *                    the waiting room)
 *   kicked         — host removed the student mid-session (they had been
 *                    admitted first); distinguish from `denied` so the copy
 *                    doesn't lie about which action the host took
 *   session_ended  — the LiveKit room was deleted or the connection dropped
 *                    with a server-shutdown reason; treated as "the session
 *                    is over for everyone"
 *   connection_lost — LiveKit disconnect with a reason that reads as an
 *                    infrastructure fault (state mismatch, generic unknown);
 *                    the student is offered a retry
 */
interface QueueSnapshot {
  rank: number;
  total: number;
}

type Phase =
  | { kind: 'prejoin' }
  | { kind: 'connecting'; choices: LocalUserChoices }
  | {
      kind: 'ready';
      choices: LocalUserChoices;
      token: string;
      url: string;
      initialState: AdmissionState;
      queue: QueueSnapshot | null;
    }
  | { kind: 'error'; code: string; message: string; retryable: boolean }
  | { kind: 'ended' }
  | { kind: 'denied' }
  | { kind: 'kicked' }
  | { kind: 'session_ended' }
  | { kind: 'connection_lost' };

// Which error codes are retryable — user can go back to the lobby and try
// again. Language-independent policy, so kept next to the copy map.
const ERROR_RETRYABLE: Record<ErrorCode, boolean> = {
  outside_join_window: true,
  not_a_participant: false,
  session_not_found: false,
  session_closed: false,
  admission_denied: false,
  session_full: false,
  livekit_not_configured: true,
};

export function RoomClient({ context }: { context: RoomContext }) {
  const router = useRouter();
  const { lang } = useLang();
  const lc = COPY[lang];
  const [phase, setPhase] = useState<Phase>({ kind: 'prejoin' });
  const [reviewOpen, setReviewOpen] = useState(false);
  // Latch: once the student clears the waiting room in a given `ready`
  // phase, remember it so a subsequent disconnect routes to the review
  // prompt instead of the "you were denied" screen.
  const [everAdmitted, setEverAdmitted] = useState(false);

  const otherName = context.isStudent ? context.tutorName : context.studentName;
  const goToSessions = () => router.push('/my-sessions');

  // Token fetch happens once we enter the connecting phase. Splitting it out
  // of the click handler lets a re-entry (retry from error) trigger the same
  // effect without duplicating the fetch body.
  useEffect(() => {
    if (phase.kind !== 'connecting') return;
    let cancelled = false;
    const choicesAtStart = phase.choices;

    (async () => {
      try {
        const res = await fetch('/api/livekit/token', {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
          body: JSON.stringify({ session_id: context.sessionId }),
        });
        const data = await res.json().catch(() => ({}));
        if (cancelled) return;
        if (!res.ok) {
          const code = (data?.error as string) || 'unknown';
          const known = (lc.errors as Record<string, { message: string }>)[code];
          const message = known ? known.message : data?.error || `HTTP ${res.status}`;
          const retryable = code in ERROR_RETRYABLE ? ERROR_RETRYABLE[code as ErrorCode] : true;
          setPhase({ kind: 'error', code, message, retryable });
          return;
        }
        const initialState: AdmissionState = data.state === 'waiting' ? 'waiting' : 'admitted';
        const queue: QueueSnapshot | null =
          data.queue && typeof data.queue.rank === 'number' && typeof data.queue.total === 'number'
            ? { rank: data.queue.rank, total: data.queue.total }
            : null;
        // Reset the latch on every fresh ready phase so a rejoin after a
        // reload doesn't inherit the previous session's admission state.
        setEverAdmitted(initialState === 'admitted');
        setPhase({
          kind: 'ready',
          choices: choicesAtStart,
          token: data.token,
          url: data.url,
          initialState,
          queue,
        });
      } catch (err) {
        if (!cancelled) {
          setPhase({
            kind: 'error',
            code: 'network',
            message: (err as Error)?.message || lc.networkError,
            retryable: true,
          });
        }
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [phase, context.sessionId, lc.networkError, lc.errors]);

  const onDisconnected = (reason?: DisconnectReason) => {
    // Non-student hosts go back to the sessions list regardless — their
    // review-flow doesn't exist and terminal screens are student-facing.
    if (!context.isStudent) {
      goToSessions();
      return;
    }

    // Route on LiveKit's DisconnectReason so the copy matches what the host
    // actually did. Falls back to `denied`/`ended` (the pre-existing latch
    // behavior) when the reason is missing — some older LiveKit paths emit
    // `Disconnected` without one.
    switch (reason) {
      case DisconnectReason.PARTICIPANT_REMOVED:
        // Host called RoomService.RemoveParticipant. Meaning depends on
        // whether we ever cleared the waiting overlay: pre-admission removal
        // is a deny; post-admission removal is a kick.
        setPhase(everAdmitted ? { kind: 'kicked' } : { kind: 'denied' });
        return;
      case DisconnectReason.ROOM_DELETED:
      case DisconnectReason.SERVER_SHUTDOWN:
        setPhase({ kind: 'session_ended' });
        return;
      case DisconnectReason.DUPLICATE_IDENTITY:
      case DisconnectReason.STATE_MISMATCH:
        setPhase({ kind: 'connection_lost' });
        return;
      case DisconnectReason.CLIENT_INITIATED:
      default:
        // User pressed Leave / End, or the reason was undefined. Preserve
        // the original latch: admitted → review; never admitted → denied.
        setPhase(everAdmitted ? { kind: 'ended' } : { kind: 'denied' });
        return;
    }
  };

  if (phase.kind === 'prejoin') {
    const prejoinContext: PreJoinContext = {
      sessionId: context.sessionId,
      courseTitle: context.courseTitle,
      otherName,
      userName: context.isStudent ? context.studentName : context.tutorName,
      scheduledAt: context.scheduledAt,
      durationMin: context.durationMin,
    };
    return (
      <PreJoinPanel
        context={prejoinContext}
        onReady={(choices) => setPhase({ kind: 'connecting', choices })}
      />
    );
  }

  if (phase.kind === 'connecting') {
    return (
      <RoomShell lang={lang} context={context} otherName={otherName}>
        <div className="flex flex-1 flex-col items-center justify-center gap-3 text-slate-400">
          <Loader2 className="text-brand size-8 animate-spin" />
          <p className="text-sm">{lc.setup}</p>
        </div>
      </RoomShell>
    );
  }

  if (phase.kind === 'error') {
    const knownTitle = (lc.errors as Record<string, { title: string }>)[phase.code]?.title;
    return (
      <RoomShell lang={lang} context={context} otherName={otherName}>
        <TerminalScreen
          tone="error"
          icon={<VideoOff className="size-6" />}
          title={knownTitle || lc.errFallbackTitle}
          description={phase.message}
          actions={
            <>
              <Button variant="outline" size="sm" render={<Link href="/my-sessions" />}>
                <ArrowLeft />
                {lc.mySessions}
              </Button>
              {phase.retryable && (
                <Button size="sm" onClick={() => setPhase({ kind: 'prejoin' })}>
                  {lc.backToLobby}
                </Button>
              )}
            </>
          }
        />
      </RoomShell>
    );
  }

  if (phase.kind === 'denied') {
    return (
      <RoomShell lang={lang} context={context} otherName={otherName}>
        <TerminalScreen
          tone="error"
          icon={<ShieldOff className="size-6" />}
          title={lc.denied.title}
          description={lc.denied.desc(otherName)}
          actions={
            <Button variant="outline" size="sm" render={<Link href="/my-sessions" />}>
              <ArrowLeft />
              {lc.backToMySessions}
            </Button>
          }
        />
      </RoomShell>
    );
  }

  if (phase.kind === 'kicked') {
    return (
      <RoomShell lang={lang} context={context} otherName={otherName}>
        <TerminalScreen
          tone="error"
          icon={<UserX className="size-6" />}
          title={lc.kicked.title}
          description={lc.kicked.desc(otherName)}
          actions={
            <Button variant="outline" size="sm" render={<Link href="/my-sessions" />}>
              <ArrowLeft />
              {lc.mySessions}
            </Button>
          }
        />
      </RoomShell>
    );
  }

  if (phase.kind === 'session_ended') {
    // Distinct from `ended` (the student left an admitted session on their
    // own): here the host or server closed the room, so we drop the review
    // prompt and just acknowledge the end.
    return (
      <RoomShell lang={lang} context={context} otherName={otherName}>
        <TerminalScreen
          tone="neutral"
          icon={<DoorClosed className="size-6" />}
          title={lc.sessionEnded.title}
          description={lc.sessionEnded.desc}
          actions={
            <Button size="sm" render={<Link href="/my-sessions" />}>
              {lc.backToMySessions}
            </Button>
          }
        />
      </RoomShell>
    );
  }

  if (phase.kind === 'connection_lost') {
    return (
      <RoomShell lang={lang} context={context} otherName={otherName}>
        <TerminalScreen
          tone="warning"
          icon={<WifiOff className="size-6" />}
          title={lc.connectionLost.title}
          description={lc.connectionLost.desc}
          actions={
            <>
              <Button size="sm" onClick={() => setPhase({ kind: 'prejoin' })}>
                <RotateCcw />
                {lc.reconnect}
              </Button>
              <Button variant="outline" size="sm" render={<Link href="/my-sessions" />}>
                {lc.mySessions}
              </Button>
            </>
          }
        />
      </RoomShell>
    );
  }

  if (phase.kind === 'ended') {
    return (
      <RoomShell lang={lang} context={context} otherName={otherName}>
        <TerminalScreen
          tone="success"
          icon={<Star className="size-6" />}
          title={lc.review.title}
          description={lc.review.desc(context.tutorName)}
          actions={
            <>
              <Button size="sm" onClick={() => setReviewOpen(true)}>
                <Star />
                {lc.review.leaveReview}
              </Button>
              <Button variant="outline" size="sm" onClick={goToSessions}>
                {lc.review.notNow}
              </Button>
            </>
          }
        />
        <ReviewDialog
          open={reviewOpen}
          onClose={() => setReviewOpen(false)}
          sessionId={context.sessionId}
          tutorName={context.tutorName}
          onSubmitted={() => setTimeout(goToSessions, 400)}
        />
      </RoomShell>
    );
  }

  // `ready` — pass the prejoin device selection through to LiveKitRoom so the
  // user connects with exactly what they saw in the preview. Empty deviceIds
  // fall back to browser default rather than sending `{ deviceId: '' }`.
  const { choices } = phase;
  const videoOpt = choices.videoEnabled
    ? choices.videoDeviceId
      ? { deviceId: choices.videoDeviceId }
      : true
    : false;
  const audioOpt = choices.audioEnabled
    ? choices.audioDeviceId
      ? { deviceId: choices.audioDeviceId }
      : true
    : false;

  return (
    // Fullscreen on mobile — `100dvh` tracks the visible viewport in real
    // time so a rotating device or address-bar collapse doesn't leave a
    // grey stripe under the video canvas.
    <div className="flex h-dvh min-h-[600px] flex-col overflow-hidden">
      <RoomTopBar
        lang={lang}
        context={context}
        otherName={otherName}
        onEnd={onDisconnected}
        confirmEnd
      />
      <PortraitHint lang={lang} />
      <div className="pb-safe flex-1 bg-slate-950">
        <LiveKitRoom
          token={phase.token}
          serverUrl={phase.url}
          connect
          video={videoOpt}
          audio={audioOpt}
          // adaptiveStream lets LiveKit downshift a subscriber's incoming
          // video resolution when their bandwidth or CPU can't keep up.
          // dynacast stops publishing tracks nobody's currently viewing
          // (e.g. a 5-person group where only the active speaker is
          // pinned). Together they roughly halve upstream bandwidth on
          // busy rooms with no visible quality hit.
          options={{ adaptiveStream: true, dynacast: true }}
          onDisconnected={onDisconnected}
          data-lk-theme="default"
          className="h-full"
        >
          <RoomBody
            sessionId={context.sessionId}
            initialState={phase.initialState}
            queue={phase.queue}
            otherName={otherName}
            courseTitle={context.courseTitle}
            onLeave={onDisconnected}
            onAdmitted={() => setEverAdmitted(true)}
          />
        </LiveKitRoom>
      </div>
    </div>
  );
}

/**
 * Body that lives inside `<LiveKitRoom>` and picks between the waiting-room
 * overlay and the full video conference. Split out into its own component so
 * we can use LiveKit hooks (which require the room provider) without moving
 * them into the top-level RoomClient.
 *
 * We treat both the server-provided `initialState` and the live permission
 * flags as sources of truth: `initialState` avoids a flash-of-wrong-content
 * on first paint (before LiveKit has surfaced permissions), and the live
 * permission read takes over once it's available so an admit event flips
 * the UI without a reconnect.
 */
function RoomBody({
  sessionId,
  initialState,
  queue,
  otherName,
  courseTitle,
  onLeave,
  onAdmitted,
}: {
  sessionId: string;
  initialState: AdmissionState;
  queue: QueueSnapshot | null;
  otherName: string;
  courseTitle: string | null;
  onLeave: () => void;
  onAdmitted: () => void;
}) {
  // useLocalParticipantPermissions specifically re-renders on the
  // ParticipantPermissionsChanged event — using useLocalParticipant here
  // fails to update when livekit-client mutates the same LocalParticipant
  // object in place. That was the bug that kept the WaitingRoomOverlay
  // stuck after an admit.
  const permissions = useLocalParticipantPermissions();
  const canPublish = permissions ? permissions.canPublish : initialState === 'admitted';

  useEffect(() => {
    if (canPublish) onAdmitted();
  }, [canPublish, onAdmitted]);

  if (!canPublish) {
    return (
      <WaitingRoomOverlay
        otherName={otherName}
        courseTitle={courseTitle}
        queue={queue}
        onLeave={onLeave}
      />
    );
  }

  return (
    <>
      <VideoConference />
      <RoomAudioRenderer />
      {/* Reconnecting banner + quality bars sit above the video canvas.
          Both are pointer-events-none so they never block LiveKit's own
          controls. */}
      <ConnectionStatusBar />
      {/* Emoji reactions — trigger pill + floating overlay. Uses the data
          channel; independent of chat + moderation. */}
      <ReactionsOverlay />
      {/* HostControlsPanel gates itself on the local participant's role
          metadata — students see nothing, tutors/admins see a moderation
          drawer. Mounting here keeps the LiveKit hooks in-context. */}
      <HostControlsPanel sessionId={sessionId} />
      {/* Screen share is a host-only affordance — the token endpoint
          restricts canPublishSources so a student attempting it via devtools
          is server-rejected. The component self-hides for students. */}
      <ScreenShareButton />
      {/* Global keydown listener for M/C mic + camera toggles. Renders no
          UI; belongs inside LiveKitRoom so the hook resolves. */}
      <RoomShortcuts />
      {/* Raise-hand — student-only, floating pill on the left above chat.
          Publishes on the vielang.hand data topic; the host reads it via
          HostControlsPanel's useRaisedHands hook. */}
      <RaiseHandButton />
      {/* Persistent chat drawer, own state + persistence layer. We hide the
          prefab's built-in chat toggle (see the global style below) so
          users don't see two chats fighting for attention. */}
      <SessionChatPanel sessionId={sessionId} />
      {/* Scoped once at the room level — LiveKit's ControlBar hard-codes
          `chat: true` in VideoConference so there's no prop path to
          disable it. `.lk-chat` is safety-net in case the panel ever
          renders through another route. The prefab also injects its own
          screen-share button whenever `canPublishSources` includes it; we
          hide it in favour of our own <ScreenShareButton /> so hosts see
          one prominent affordance instead of two competing controls. */}
      <style>
        {`.lk-chat-toggle,.lk-chat,.lk-button[data-lk-source="screen_share"]{display:none!important}`}
      </style>
    </>
  );
}

/**
 * Portrait-orientation hint for phones. Video calls work in portrait but the
 * LiveKit control chrome eats most of the vertical space — rotating gives the
 * user a 2× larger tile. Only fires on narrow-and-portrait viewports; hides
 * itself once the user rotates.
 */
function PortraitHint({ lang }: { lang: Lang }) {
  const lc = COPY[lang];
  const [dismissed, setDismissed] = useState(false);
  if (dismissed) return null;
  return (
    <div
      className="pointer-events-auto flex items-center justify-between gap-3 bg-amber-50 px-4 py-2 text-xs text-amber-900 sm:hidden portrait:flex landscape:hidden dark:bg-amber-950/40 dark:text-amber-200"
      role="status"
    >
      <span className="flex items-center gap-2">
        <RotateCcw className="size-3.5" />
        {lc.rotateHint}
      </span>
      <button
        type="button"
        onClick={() => setDismissed(true)}
        aria-label={lc.dismissHint}
        className="focus-ring rounded-md px-2 py-1 font-semibold text-amber-800 hover:bg-amber-100 dark:text-amber-100 dark:hover:bg-amber-900/60"
      >
        {lc.gotIt}
      </button>
    </div>
  );
}

function RoomTopBar({
  lang,
  context,
  otherName,
  onEnd,
  confirmEnd,
}: {
  lang: Lang;
  context: RoomContext;
  otherName: string;
  onEnd: () => void;
  /** If true, the End button pops an AlertDialog first; otherwise it fires
   *  immediately. Non-video shells (loading, error) skip the confirm. */
  confirmEnd?: boolean;
}) {
  const lc = COPY[lang];
  const [confirmOpen, setConfirmOpen] = useState(false);
  const trigger = (
    <Button
      variant="destructive"
      size="sm"
      onClick={confirmEnd ? () => setConfirmOpen(true) : onEnd}
      className="tap-min"
    >
      <LogOut />
      <span className="hidden sm:inline">{lc.endSession}</span>
      <span className="sr-only sm:hidden">{lc.endSession}</span>
    </Button>
  );

  return (
    <header className="pt-safe flex shrink-0 items-center gap-3 border-b border-slate-200 bg-white px-4 py-3 sm:px-6 dark:border-slate-800 dark:bg-slate-900">
      <div className="min-w-0 flex-1">
        <p className="type-eyebrow">{lc.liveEyebrow}</p>
        <p className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
          {context.courseTitle || lc.defaultTitle}
          <span className="font-normal text-slate-500 dark:text-slate-400">
            {lc.withSuffix(otherName)}
          </span>
        </p>
      </div>
      {trigger}
      {confirmEnd ? (
        <AlertDialog open={confirmOpen} onOpenChange={setConfirmOpen}>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>{lc.confirmLeaveTitle}</AlertDialogTitle>
              <AlertDialogDescription>
                {lc.confirmLeaveDescBefore}
                <strong>{lc.mySessions}</strong>
                {lc.confirmLeaveDescAfter}
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel>{lc.stay}</AlertDialogCancel>
              <AlertDialogAction onClick={onEnd}>{lc.leave}</AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      ) : null}
    </header>
  );
}

function RoomShell({
  lang,
  context,
  otherName,
  children,
}: {
  lang: Lang;
  context: RoomContext;
  otherName: string;
  children: React.ReactNode;
}) {
  return (
    // Fills the visible viewport so pre-connect UI (loading, error, review)
    // doesn't leave a strip of background behind the video area.
    <div className="flex min-h-dvh flex-col bg-slate-50 dark:bg-slate-950">
      <RoomTopBar
        lang={lang}
        context={context}
        otherName={otherName}
        onEnd={() => history.back()}
      />
      <div className="pb-safe flex flex-1 flex-col">{children}</div>
    </div>
  );
}
