'use client';

import { useMemo, useState, useTransition } from 'react';
import { useRouter } from 'next/navigation';
import Link from 'next/link';
import { toast } from 'sonner';
import { motion } from 'motion/react';
import {
  ArrowLeft,
  CalendarDays,
  Clock,
  Users,
  Video,
  Star,
  Info,
  UserCheck,
  LogIn,
  X,
} from 'lucide-react';
import { Spinner } from '@/components/ui/spinner';
import { AvatarBubble } from '@/components/shared/AvatarBubble';
import { SESSION_STATUS_TONE } from '@/lib/status-tone';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { useLang } from '@/contexts';
import { formatWhen, formatCountdown, JOIN_WINDOW_MS, COUNTDOWN_WINDOW_MS } from '@/lib/time';
import { useTicker } from '@/hooks/use-ticker';
import type { GroupSessionDetail } from '@/lib/supabase';
import { errorMessage } from '@/lib/errors';

interface Viewer {
  id: string | null;
  role: 'user' | 'tutor' | 'admin' | null;
  isHostOrAdmin: boolean;
  reserved: boolean;
}

interface Props {
  session: GroupSessionDetail;
  viewer: Viewer;
}

const COPY = {
  VN: {
    back: 'Quay lại danh sách',
    live: 'Đang diễn ra',
    startsIn: 'Bắt đầu sau',
    endsIn: 'Kết thúc sau',
    about: 'Về buổi học này',
    host: 'Điều phối',
    noHost: 'Đội ngũ VieLang',
    seats: 'Chỗ ngồi',
    seatsFull: 'Đã đầy',
    seatOf: (n: number, cap: number) => `${n}/${cap} người tham gia`,
    join: 'Vào phòng ngay',
    reserve: 'Giữ chỗ',
    reserving: 'Đang giữ chỗ…',
    reserved: 'Bạn đã có chỗ',
    reservedHint: 'Bạn sẽ nhận thông báo trước giờ học 15 phút.',
    cancelReservation: 'Huỷ chỗ',
    cancelling: 'Đang huỷ…',
    signInToReserve: 'Đăng nhập để giữ chỗ',
    hostBadge: 'Bạn là điều phối phòng này',
    hostJoin: 'Vào phòng (host)',
    full: 'Phòng đã đầy',
    fullHint: 'Bạn có thể xem các phòng khác cùng cấp độ hoặc chờ đợt sau.',
    ended: 'Buổi học đã kết thúc',
    endedHint: 'Xem các buổi sắp tới trong danh sách sessions.',
    cancelled: 'Buổi học đã bị huỷ',
    joinWindow: 'Vào phòng sẵn sàng 15 phút trước giờ bắt đầu.',
    viewTutor: 'Xem hồ sơ giáo viên',
    duration: (m: number) => `${m} phút`,
    reservationError: 'Không giữ chỗ được',
    reservationOk: 'Đã giữ chỗ thành công',
    cancelOk: 'Đã huỷ chỗ',
    cancelError: 'Không huỷ được',
    atCapacity: 'Phòng vừa được lấp đầy — vui lòng chọn phòng khác.',
    sessionClosed: 'Phòng này không còn nhận đăng ký.',
  },
  EN: {
    back: 'Back to sessions',
    live: 'Live now',
    startsIn: 'Starts in',
    endsIn: 'Ends in',
    about: 'About this session',
    host: 'Host',
    noHost: 'VieLang team',
    seats: 'Seats',
    seatsFull: 'Room full',
    seatOf: (n: number, cap: number) => `${n}/${cap} joined`,
    join: 'Join now',
    reserve: 'Reserve a seat',
    reserving: 'Reserving…',
    reserved: "You're in",
    reservedHint: "We'll ping you 15 minutes before the room opens.",
    cancelReservation: 'Cancel reservation',
    cancelling: 'Cancelling…',
    signInToReserve: 'Sign in to reserve',
    hostBadge: "You're the host of this room",
    hostJoin: 'Enter as host',
    full: 'Room is full',
    fullHint: 'Try another room at the same level or wait for the next batch.',
    ended: 'This session has ended',
    endedHint: 'Browse upcoming sessions from the sessions list.',
    cancelled: 'This session was cancelled',
    joinWindow: 'The room opens 15 minutes before start.',
    viewTutor: 'View tutor profile',
    duration: (m: number) => `${m} min`,
    reservationError: "Couldn't reserve seat",
    reservationOk: 'Seat reserved',
    cancelOk: 'Reservation cancelled',
    cancelError: "Couldn't cancel",
    atCapacity: 'Room was just filled — please pick another.',
    sessionClosed: 'This room is closed to new sign-ups.',
  },
} as const;

export function SessionDetailClient({ session, viewer }: Props) {
  const router = useRouter();
  const { lang } = useLang();
  const lc = COPY[lang];
  const [, startTransition] = useTransition();
  const now = useTicker();
  const [reserved, setReserved] = useState(viewer.reserved);
  const [participantCount, setParticipantCount] = useState(session.participant_count);
  const [pending, setPending] = useState<null | 'reserve' | 'cancel'>(null);

  const start = new Date(session.scheduled_at).getTime();
  const end = start + session.duration_min * 60_000;
  const msUntilStart = start - now;
  const msUntilEnd = end - now;
  const isActiveStatus =
    session.status === 'pending' || session.status === 'confirmed' || session.status === 'live';
  const canJoin = isActiveStatus && now >= start - JOIN_WINDOW_MS && now <= end + JOIN_WINDOW_MS;
  const isLive = isActiveStatus && now >= start && now <= end;
  const hasEnded = now > end + JOIN_WINDOW_MS || session.status === 'completed';
  const isCancelled = session.status === 'cancelled' || session.status === 'no_show';
  const isFull = participantCount >= session.capacity;
  const capacityPct = Math.min(100, Math.round((participantCount / session.capacity) * 100));

  const when = formatWhen(session.scheduled_at, 'detail');
  const topic =
    (lang === 'VN' ? session.topic_vn || session.topic_en : session.topic_en || session.topic_vn) ||
    'Free-talk session';
  const levelLabel = (session.level || 'all').toUpperCase().replace('_', '–');

  const reserve = async () => {
    if (!viewer.id) {
      router.push(`/login?redirect=/sessions/${session.id}`);
      return;
    }
    setPending('reserve');
    try {
      const res = await fetch(`/api/sessions/${session.id}/reserve`, {
        method: 'POST',
        headers: { ...(await getAuthHeaders()) },
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        if (data?.error === 'at_capacity') {
          toast.error(lc.atCapacity);
          startTransition(() => router.refresh());
          return;
        }
        if (data?.error === 'session_closed' || data?.error === 'session_ended') {
          toast.error(lc.sessionClosed);
          startTransition(() => router.refresh());
          return;
        }
        throw new Error(data?.error || 'reserve_failed');
      }
      setReserved(true);
      if (!data.alreadyReserved) {
        setParticipantCount((n) => Math.min(session.capacity, n + 1));
      }
      toast.success(lc.reservationOk);
    } catch (err) {
      toast.error(lc.reservationError, { description: errorMessage(err) });
    } finally {
      setPending(null);
    }
  };

  const cancel = async () => {
    setPending('cancel');
    try {
      const res = await fetch(`/api/sessions/${session.id}/reserve`, {
        method: 'DELETE',
        headers: { ...(await getAuthHeaders()) },
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        throw new Error(data?.error || 'cancel_failed');
      }
      setReserved(false);
      setParticipantCount((n) => Math.max(0, n - 1));
      toast.success(lc.cancelOk);
    } catch (err) {
      toast.error(lc.cancelError, { description: errorMessage(err) });
    } finally {
      setPending(null);
    }
  };

  const enterRoom = () => router.push(`/session/${session.id}/room`);

  // The avatar strip shows up to 5 real participants, then a "+N" pip. Hosts
  // never carry a session_participants row so this genuinely reflects
  // student turnout rather than double-counting the tutor.
  const visibleParticipants = useMemo(
    () => session.participants.slice(0, 5),
    [session.participants],
  );
  const overflow = Math.max(0, session.participant_count - visibleParticipants.length);

  const showCountdown = isActiveStatus && msUntilStart > 0 && msUntilStart <= COUNTDOWN_WINDOW_MS;

  return (
    <div className="space-y-6">
      <Link
        href="/sessions"
        className="focus-ring inline-flex items-center gap-1.5 rounded-md text-xs font-semibold text-slate-500 transition-colors hover:text-slate-900 dark:text-slate-400 dark:hover:text-slate-100"
      >
        <ArrowLeft className="size-3.5" />
        {lc.back}
      </Link>

      <div className="grid grid-cols-1 gap-6 lg:grid-cols-[minmax(0,1fr)_360px]">
        {/* Main column */}
        <motion.article
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.35 }}
          className="space-y-6 rounded-2xl border border-slate-200 bg-white p-6 shadow-sm md:p-8 dark:border-slate-800 dark:bg-slate-900"
        >
          <header className="space-y-4">
            <div className="flex items-start gap-4">
              <div className="bg-brand/10 text-brand dark:bg-brand/15 flex size-16 shrink-0 items-center justify-center rounded-2xl text-3xl">
                {session.cover_emoji || '💬'}
              </div>
              <div className="min-w-0 flex-1 space-y-2">
                <div className="flex flex-wrap items-center gap-2">
                  <span className="inline-flex h-5 items-center rounded-full bg-slate-100 px-2 text-[10px] font-semibold tracking-wider uppercase dark:bg-slate-800">
                    {levelLabel}
                  </span>
                  {isLive && (
                    <span className="inline-flex h-5 items-center gap-1 rounded-full bg-emerald-100 px-2 text-[10px] font-semibold tracking-wider text-emerald-700 uppercase dark:bg-emerald-950/40 dark:text-emerald-300">
                      <span
                        className="size-1.5 animate-pulse rounded-full bg-emerald-500"
                        aria-hidden
                      />
                      {lc.live}
                    </span>
                  )}
                  {!isLive && showCountdown && (
                    <span className="inline-flex h-5 items-center gap-1 rounded-full bg-amber-100 px-2 text-[10px] font-semibold tracking-wider text-amber-700 uppercase dark:bg-amber-950/40 dark:text-amber-300">
                      <span
                        className="size-1.5 animate-pulse rounded-full bg-amber-500"
                        aria-hidden
                      />
                      {lc.startsIn} {formatCountdown(msUntilStart)}
                    </span>
                  )}
                  <span
                    className={`inline-flex h-5 items-center rounded-full px-2 text-[10px] font-semibold tracking-wider uppercase ${SESSION_STATUS_TONE[session.status]}`}
                  >
                    {session.status.replace('_', ' ')}
                  </span>
                </div>
                <h1 className="text-brand dark:text-accent-warm font-serif text-2xl leading-tight font-bold md:text-3xl">
                  {topic}
                </h1>
              </div>
            </div>

            <div className="flex flex-wrap items-center gap-x-6 gap-y-2 border-y border-slate-100 py-3 text-sm text-slate-600 dark:border-slate-800 dark:text-slate-300">
              <span className="flex items-center gap-2">
                <CalendarDays className="text-brand size-4" />
                {when.date}
              </span>
              <span className="flex items-center gap-2">
                <Clock className="text-brand size-4" />
                {when.time} · {lc.duration(session.duration_min)}
              </span>
              <span className="flex items-center gap-2">
                <Users className="text-brand size-4" />
                {lc.seatOf(participantCount, session.capacity)}
              </span>
            </div>
          </header>

          {session.description && (
            <section className="space-y-2">
              <h2 className="text-xs font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                {lc.about}
              </h2>
              <p className="text-sm leading-relaxed whitespace-pre-wrap text-slate-700 dark:text-slate-200">
                {session.description}
              </p>
            </section>
          )}

          <section className="space-y-3">
            <h2 className="text-xs font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
              {lc.host}
            </h2>
            {session.tutor_id ? (
              <Link
                href={`/tutors/${session.tutor_id}`}
                className="focus-ring hover:border-brand/40 flex items-center gap-4 rounded-xl border border-slate-200 p-4 transition-colors dark:border-slate-700"
              >
                <AvatarBubble src={session.tutor_avatar} name={session.tutor_name} size={48} />
                <div className="min-w-0 flex-1">
                  <p className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
                    {session.tutor_name || lc.noHost}
                  </p>
                  {session.tutor_rating != null && session.tutor_rating > 0 && (
                    <p className="mt-0.5 flex items-center gap-1 text-xs text-slate-500 dark:text-slate-400">
                      <Star className="size-3 fill-amber-400 text-amber-400" />
                      {session.tutor_rating.toFixed(1)}
                    </p>
                  )}
                  {session.tutor_bio && (
                    <p className="mt-1 line-clamp-2 text-xs text-slate-500 dark:text-slate-400">
                      {session.tutor_bio}
                    </p>
                  )}
                </div>
                <span className="hidden text-xs font-semibold text-slate-500 sm:inline dark:text-slate-400">
                  {lc.viewTutor} →
                </span>
              </Link>
            ) : (
              <div className="flex items-center gap-3 rounded-xl border border-dashed border-slate-200 p-4 text-sm text-slate-500 dark:border-slate-700 dark:text-slate-400">
                <Info className="size-4" />
                {lc.noHost}
              </div>
            )}
          </section>

          <section className="space-y-3">
            <div className="flex items-center justify-between">
              <h2 className="text-xs font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                {lc.seats}
              </h2>
              <span className="text-xs font-semibold text-slate-700 tabular-nums dark:text-slate-200">
                {isFull ? lc.seatsFull : lc.seatOf(participantCount, session.capacity)}
              </span>
            </div>
            <div
              className="relative h-2 overflow-hidden rounded-full bg-slate-100 dark:bg-slate-800"
              aria-hidden
            >
              <div
                className={`absolute inset-y-0 left-0 rounded-full transition-all ${
                  isFull ? 'bg-amber-500' : 'bg-brand'
                }`}
                style={{ width: `${capacityPct}%` }}
              />
            </div>
            {visibleParticipants.length > 0 && (
              <div className="flex items-center gap-1.5 pt-1">
                {visibleParticipants.map((p) => (
                  <AvatarBubble
                    key={p.user_id}
                    src={p.avatar}
                    name={p.name}
                    size={28}
                    ringClassName="ring-2 ring-white dark:ring-slate-900"
                  />
                ))}
                {overflow > 0 && (
                  <span className="inline-flex size-7 items-center justify-center rounded-full bg-slate-100 text-[10px] font-semibold text-slate-600 ring-2 ring-white dark:bg-slate-800 dark:text-slate-300 dark:ring-slate-900">
                    +{overflow}
                  </span>
                )}
              </div>
            )}
          </section>
        </motion.article>

        {/* Sticky CTA rail */}
        <aside className="lg:sticky lg:top-24 lg:self-start">
          <div className="space-y-3 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900">
            {isCancelled ? (
              <div className="space-y-2 text-center">
                <p className="text-sm font-semibold text-red-600 dark:text-red-400">
                  {lc.cancelled}
                </p>
              </div>
            ) : hasEnded ? (
              <div className="space-y-2 text-center">
                <p className="text-sm font-semibold text-slate-900 dark:text-slate-100">
                  {lc.ended}
                </p>
                <p className="text-xs text-slate-500 dark:text-slate-400">{lc.endedHint}</p>
              </div>
            ) : viewer.isHostOrAdmin ? (
              <>
                <div className="rounded-lg bg-indigo-50 p-3 text-xs font-semibold text-indigo-800 dark:bg-indigo-950/40 dark:text-indigo-200">
                  <UserCheck className="mr-1.5 inline size-3.5 align-[-2px]" />
                  {lc.hostBadge}
                </div>
                <button
                  type="button"
                  onClick={enterRoom}
                  disabled={!canJoin}
                  title={!canJoin ? lc.joinWindow : undefined}
                  className="focus-ring bg-brand hover:bg-brand-hover inline-flex h-11 w-full items-center justify-center gap-1.5 rounded-lg text-sm font-semibold text-white transition-colors disabled:cursor-not-allowed disabled:opacity-40"
                >
                  <Video className="size-4" />
                  {lc.hostJoin}
                </button>
                {!canJoin && (
                  <p className="text-center text-[11px] text-slate-500 dark:text-slate-400">
                    {lc.joinWindow}
                  </p>
                )}
              </>
            ) : reserved ? (
              <>
                <div className="rounded-lg bg-emerald-50 p-3 text-xs font-semibold text-emerald-800 dark:bg-emerald-950/40 dark:text-emerald-200">
                  <UserCheck className="mr-1.5 inline size-3.5 align-[-2px]" />
                  {lc.reserved}
                </div>
                {canJoin ? (
                  <button
                    type="button"
                    onClick={enterRoom}
                    className="focus-ring bg-brand hover:bg-brand-hover inline-flex h-11 w-full items-center justify-center gap-1.5 rounded-lg text-sm font-semibold text-white transition-colors"
                  >
                    <Video className="size-4" />
                    {lc.join}
                    {isLive && (
                      <span className="text-[11px] font-normal tabular-nums opacity-80">
                        · {formatCountdown(msUntilEnd)}
                      </span>
                    )}
                  </button>
                ) : (
                  <p className="text-center text-xs text-slate-600 dark:text-slate-300">
                    {lc.reservedHint}
                  </p>
                )}
                <button
                  type="button"
                  onClick={cancel}
                  disabled={pending !== null || now >= start}
                  className="focus-ring inline-flex h-9 w-full items-center justify-center gap-1.5 rounded-lg border border-slate-200 text-xs font-semibold text-slate-600 transition-colors hover:border-red-300 hover:text-red-600 disabled:cursor-not-allowed disabled:opacity-40 dark:border-slate-700 dark:text-slate-300"
                >
                  {pending === 'cancel' ? (
                    <Spinner className="size-3.5" />
                  ) : (
                    <X className="size-3.5" />
                  )}
                  {pending === 'cancel' ? lc.cancelling : lc.cancelReservation}
                </button>
              </>
            ) : isFull ? (
              <div className="space-y-2 text-center">
                <p className="text-sm font-semibold text-amber-700 dark:text-amber-300">
                  {lc.full}
                </p>
                <p className="text-xs text-slate-500 dark:text-slate-400">{lc.fullHint}</p>
                <Link
                  href="/sessions"
                  className="focus-ring hover:border-brand/40 inline-flex h-9 w-full items-center justify-center gap-1.5 rounded-lg border border-slate-200 text-xs font-semibold text-slate-600 transition-colors dark:border-slate-700 dark:text-slate-300"
                >
                  {lc.back}
                </Link>
              </div>
            ) : !viewer.id ? (
              <Link
                href={`/login?redirect=/sessions/${session.id}`}
                className="focus-ring bg-brand hover:bg-brand-hover inline-flex h-11 w-full items-center justify-center gap-1.5 rounded-lg text-sm font-semibold text-white transition-colors"
              >
                <LogIn className="size-4" />
                {lc.signInToReserve}
              </Link>
            ) : (
              <>
                <button
                  type="button"
                  onClick={reserve}
                  disabled={pending !== null}
                  className="focus-ring bg-brand hover:bg-brand-hover inline-flex h-11 w-full items-center justify-center gap-1.5 rounded-lg text-sm font-semibold text-white transition-colors disabled:cursor-not-allowed disabled:opacity-60"
                >
                  {pending === 'reserve' ? (
                    <Spinner />
                  ) : (
                    <>
                      <Video className="size-4" />
                      {isLive ? lc.join : lc.reserve}
                    </>
                  )}
                </button>
                <p className="text-center text-[11px] text-slate-500 dark:text-slate-400">
                  {lc.joinWindow}
                </p>
              </>
            )}
          </div>
        </aside>
      </div>
    </div>
  );
}
