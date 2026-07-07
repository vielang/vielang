'use client';

import { useEffect, useRef, useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';
import { Video, X, Star, BookOpen, MoreHorizontal, CalendarClock, FileText } from 'lucide-react';
import { Spinner } from '@/components/ui/spinner';
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
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from '@/components/ui/dropdown-menu';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { useLang } from '@/contexts';
import { errorMessage } from '@/lib/errors';
import { ReviewDialog } from './ReviewDialog';
import { MaterialsDialog } from './MaterialsDialog';
import { RescheduleDialog } from './RescheduleDialog';
import type { SessionListItem } from '@/lib/supabase';

const COPY = {
  VN: {
    join: 'Vào lớp',
    leaveReview: 'Đánh giá',
    details: 'Chi tiết',
    materials: 'Tài liệu',
    reschedule: 'Đổi lịch',
    cancel: 'Huỷ',
    cancelSession: 'Huỷ buổi học',
    moreActions: 'Thêm',
    cancelTitle: 'Huỷ buổi học này?',
    cancelDesc: (name: string) =>
      `Điều này sẽ giải phóng khung giờ và thông báo cho ${name}. Không thể hoàn tác.`,
    keepBooking: 'Giữ lịch',
    confirmCancel: 'Xác nhận huỷ',
  },
  EN: {
    join: 'Join class',
    leaveReview: 'Leave review',
    details: 'Details',
    materials: 'Materials',
    reschedule: 'Reschedule',
    cancel: 'Cancel',
    cancelSession: 'Cancel session',
    moreActions: 'More actions',
    cancelTitle: 'Cancel this session?',
    cancelDesc: (name: string) =>
      `This will free the slot and notify ${name}. This can't be undone.`,
    keepBooking: 'Keep booking',
    confirmCancel: 'Yes, cancel',
  },
} as const;

interface Props {
  session: SessionListItem;
  viewerRole: 'user' | 'tutor';
  now: number;
  reviewed: boolean;
  otherName: string;
  onChanged: () => void;
}

/**
 * Action row + dialog mounts for `<SessionCard>`. Kept separate from the
 * presentational shell so the shell stays close to a static render — cheaper
 * to reason about and cheaper to re-render when the tick or a dialog opens.
 *
 * Layout: primary CTA (Join / Leave review) inline on the left; secondary
 * actions inline on desktop, collapsed into a "More" dropdown on mobile.
 * Cancel is a confirm-then-fire mutation because it frees the tutor slot.
 */
export function SessionCardActions({
  session,
  viewerRole,
  now,
  reviewed,
  otherName,
  onChanged,
}: Props) {
  const router = useRouter();
  const { lang } = useLang();
  const lc = COPY[lang];
  const [confirmingCancel, setConfirmingCancel] = useState(false);
  const [pending, setPending] = useState<null | 'cancel'>(null);
  const [reviewOpen, setReviewOpen] = useState(false);
  const [materialsOpen, setMaterialsOpen] = useState(false);
  const [rescheduleOpen, setRescheduleOpen] = useState(false);

  const start = new Date(session.scheduled_at).getTime();
  const end = start + session.duration_min * 60_000;
  const msUntilStart = start - now;
  const isActiveStatus =
    session.status === 'confirmed' || session.status === 'live' || session.status === 'pending';
  // Disable once the scheduled end time has passed — the class is over.
  // 'live' rooms are exempt: the webhook confirmed the room is still open.
  const isPastEnd = session.status !== 'live' && now > end;
  const canJoin = isActiveStatus && !isPastEnd;

  // Pulse the Join button once on mount for active sessions so the CTA is
  // immediately visible after the booking redirect.
  const [pulseJoin, setPulseJoin] = useState(false);
  const pulseFired = useRef(false);
  useEffect(() => {
    if (!canJoin || pulseFired.current) return;
    pulseFired.current = true;
    setPulseJoin(true);
    const id = setTimeout(() => setPulseJoin(false), 3000);
    return () => clearTimeout(id);
  }, [canJoin]);

  const canCancel = session.status === 'pending' || session.status === 'confirmed';
  const hoursUntilStart = msUntilStart / 3_600_000;
  const canReschedule = canCancel && hoursUntilStart >= 24;
  // Completed sessions get a "Details" affordance leading to the post-session
  // transcript + attendance viewer.
  const showDetailsLink = session.status === 'completed';

  const cancel = async () => {
    setPending('cancel');
    try {
      const res = await fetch(`/api/sessions/${session.id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({ action: 'cancel' }),
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        throw new Error(data?.error || 'cancel_failed');
      }
      toast.success('Session cancelled');
      onChanged();
    } catch (err) {
      toast.error('Could not cancel', { description: errorMessage(err) });
    } finally {
      setPending(null);
      setConfirmingCancel(false);
    }
  };

  const join = () => {
    // The room page fetches a fresh token + guards the join window itself,
    // so we can navigate optimistically. Keeps click → open snappy.
    router.push(`/session/${session.id}/room`);
  };

  const hasSecondaryActions = showDetailsLink || !!session.course_id || canReschedule || canCancel;

  return (
    <>
      <div className="flex items-center gap-2 pt-3">
        {session.status !== 'completed' && (
          <button
            type="button"
            onClick={join}
            disabled={!canJoin || pending !== null}
            aria-label={lc.join}
            className={`focus-ring bg-brand hover:bg-brand-hover inline-flex h-9 items-center gap-1.5 rounded-lg px-4 text-xs font-semibold text-white transition-colors disabled:cursor-not-allowed disabled:opacity-40 ${
              pulseJoin
                ? 'ring-brand/40 ring-4 ring-offset-2 ring-offset-white motion-safe:animate-pulse dark:ring-offset-slate-900'
                : ''
            }`}
          >
            <Video className="size-3.5" />
            {lc.join}
          </button>
        )}
        {viewerRole === 'user' && session.status === 'completed' && !reviewed && (
          <button
            type="button"
            onClick={() => setReviewOpen(true)}
            className="focus-ring bg-brand hover:bg-brand-hover inline-flex h-9 items-center gap-1.5 rounded-lg px-4 text-xs font-semibold text-white transition-colors"
          >
            <Star className="size-3.5" />
            {lc.leaveReview}
          </button>
        )}

        {/* Secondary actions collapse into a "More" menu once the user is on
            mobile so the primary Join / Review never wraps. The menu items
            mirror the same set — hidden on desktop where they render inline. */}
        <div className="ml-auto flex items-center gap-2">
          {showDetailsLink && (
            <Link
              href={`/my-sessions/${session.id}`}
              className="focus-ring hover:border-brand/40 hover:text-brand hidden h-9 items-center gap-1.5 rounded-lg border border-slate-200 px-3 text-xs font-semibold text-slate-600 transition-colors sm:inline-flex dark:border-slate-700 dark:text-slate-300"
            >
              <FileText className="size-3.5" />
              {lc.details}
            </Link>
          )}
          {session.course_id && (
            <button
              type="button"
              onClick={() => setMaterialsOpen(true)}
              className="focus-ring hover:border-brand/40 hover:text-brand hidden h-9 items-center gap-1.5 rounded-lg border border-slate-200 px-3 text-xs font-semibold text-slate-600 transition-colors sm:inline-flex dark:border-slate-700 dark:text-slate-300"
            >
              <BookOpen className="size-3.5" />
              {lc.materials}
            </button>
          )}
          {canReschedule && (
            <button
              type="button"
              onClick={() => setRescheduleOpen(true)}
              className="focus-ring hover:border-brand/40 hover:text-brand hidden h-9 items-center gap-1.5 rounded-lg border border-slate-200 px-3 text-xs font-semibold text-slate-600 transition-colors sm:inline-flex dark:border-slate-700 dark:text-slate-300"
            >
              <CalendarClock className="size-3.5" />
              {lc.reschedule}
            </button>
          )}
          {canCancel && (
            <button
              type="button"
              onClick={() => setConfirmingCancel(true)}
              disabled={pending !== null}
              className="focus-ring hidden h-9 items-center gap-1.5 rounded-lg border border-slate-200 px-3 text-xs font-semibold text-slate-600 transition-colors hover:border-red-300 hover:text-red-600 sm:inline-flex dark:border-slate-700 dark:text-slate-300"
            >
              {pending === 'cancel' ? <Spinner className="size-3.5" /> : <X className="size-3.5" />}
              {lc.cancel}
            </button>
          )}

          {hasSecondaryActions && (
            <DropdownMenu>
              <DropdownMenuTrigger
                render={
                  <button
                    type="button"
                    aria-label={lc.moreActions}
                    className="focus-ring hover:border-brand/40 hover:text-brand inline-flex size-11 items-center justify-center rounded-lg border border-slate-200 text-slate-500 transition-colors sm:hidden dark:border-slate-700 dark:text-slate-300"
                  >
                    <MoreHorizontal className="size-4" />
                  </button>
                }
              />
              <DropdownMenuContent align="end">
                {showDetailsLink && (
                  <DropdownMenuItem
                    render={
                      <Link href={`/my-sessions/${session.id}`}>
                        <FileText className="size-3.5" />
                        {lc.details}
                      </Link>
                    }
                  />
                )}
                {session.course_id && (
                  <DropdownMenuItem onClick={() => setMaterialsOpen(true)}>
                    <BookOpen className="size-3.5" />
                    {lc.materials}
                  </DropdownMenuItem>
                )}
                {canReschedule && (
                  <DropdownMenuItem onClick={() => setRescheduleOpen(true)}>
                    <CalendarClock className="size-3.5" />
                    {lc.reschedule}
                  </DropdownMenuItem>
                )}
                {canCancel && (
                  <>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem
                      onClick={() => setConfirmingCancel(true)}
                      className="text-red-600 focus:text-red-600 dark:text-red-400 dark:focus:text-red-400"
                    >
                      <X className="size-3.5" />
                      {lc.cancelSession}
                    </DropdownMenuItem>
                  </>
                )}
              </DropdownMenuContent>
            </DropdownMenu>
          )}
        </div>
      </div>

      {session.course_id && (
        <MaterialsDialog
          open={materialsOpen}
          onClose={() => setMaterialsOpen(false)}
          courseId={session.course_id}
          courseTitle={session.course_title_en || 'Materials'}
        />
      )}

      {viewerRole === 'user' && session.course_id && (
        <ReviewDialog
          open={reviewOpen}
          onClose={() => setReviewOpen(false)}
          sessionId={session.id}
          tutorName={session.tutor_name || 'your tutor'}
          onSubmitted={onChanged}
        />
      )}

      {canReschedule && session.tutor_id && (
        <RescheduleDialog
          open={rescheduleOpen}
          onClose={() => setRescheduleOpen(false)}
          sessionId={session.id}
          tutorId={session.tutor_id}
          courseId={session.course_id}
          currentStartAt={session.scheduled_at}
          onRescheduled={onChanged}
        />
      )}

      <AlertDialog open={confirmingCancel} onOpenChange={setConfirmingCancel}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>{lc.cancelTitle}</AlertDialogTitle>
            <AlertDialogDescription>{lc.cancelDesc(otherName)}</AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>{lc.keepBooking}</AlertDialogCancel>
            <AlertDialogAction onClick={cancel}>{lc.confirmCancel}</AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </>
  );
}
