'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { ArrowLeft, Clock, User } from 'lucide-react';
import { PreJoin } from '@livekit/components-react';
import type { LocalUserChoices } from '@livekit/components-core';
import '@livekit/components-styles';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { useLang } from '@/contexts';
import type { Lang } from '@/contexts/LangContext';
import { PreJoinAudioMeter } from './PreJoinAudioMeter';

const COPY = {
  VN: {
    back: 'Quay lại',
    eyebrow: 'Chuẩn bị',
    englishLesson: 'Buổi học tiếng Anh',
    withPrefix: (n: string) => ` · với ${n}`,
    opensIn: (t: string) => `Mở sau ${t}`,
    sessionEnded: 'Buổi học đã kết thúc',
    joinClassroom: 'Vào lớp',
    permBlocked:
      'Trình duyệt của bạn đã chặn camera hoặc micro. Nhấn biểu tượng ổ khoá trên thanh địa chỉ, cho phép cả hai rồi tải lại trang.',
    permNotFound:
      'Không tìm thấy camera hoặc micro. Cắm thiết bị vào rồi tải lại — bạn vẫn có thể vào chỉ với âm thanh.',
    permGeneric: 'Không thể truy cập thiết bị đa phương tiện.',
    mediaBlockedTitle: 'Bị chặn truy cập thiết bị',
    notYetTitle: 'Chưa đến giờ',
    notYetOpens: 'Lớp học mở 15 phút trước khi buổi học bắt đầu.',
    notYetEnded: 'Buổi học này đã kết thúc.',
    scheduleSession: 'Buổi học',
    scheduleUnavailable: 'Không có lịch',
    minutes: (n: number) => `${n} phút`,
    withCol: 'Với',
    classroomOpen: 'Lớp học đã mở. Vào khi bạn sẵn sàng.',
    opensInLabel: 'Mở sau',
    scheduleEnded: 'Buổi học này đã kết thúc.',
    scheduleFallback: 'Không có lịch — bạn vẫn có thể thử vào.',
    beforeYouJoin: 'Trước khi vào',
    tip1: 'Dùng tai nghe để tránh vọng âm.',
    tip2: 'Đảm bảo mặt bạn được chiếu sáng tốt.',
    tip3: 'Đóng các tab tốn băng thông (video, tải xuống).',
  },
  EN: {
    back: 'Back',
    eyebrow: 'Get ready',
    englishLesson: 'English lesson',
    withPrefix: (n: string) => ` · with ${n}`,
    opensIn: (t: string) => `Opens in ${t}`,
    sessionEnded: 'Session ended',
    joinClassroom: 'Join classroom',
    permBlocked:
      'Your browser blocked camera or microphone access. Click the padlock in the address bar and allow both, then reload.',
    permNotFound:
      'No camera or microphone was detected. Plug one in and reload — you can still join audio-only from the mic toggle.',
    permGeneric: 'Could not access media devices.',
    mediaBlockedTitle: 'Media access blocked',
    notYetTitle: 'Not yet',
    notYetOpens: 'The classroom opens 15 minutes before your session starts.',
    notYetEnded: 'This session has already ended.',
    scheduleSession: 'Session',
    scheduleUnavailable: 'Schedule unavailable',
    minutes: (n: number) => `${n} minutes`,
    withCol: 'With',
    classroomOpen: "The classroom is open. Join when you're ready.",
    opensInLabel: 'Opens in',
    scheduleEnded: 'This session has ended.',
    scheduleFallback: 'Schedule unavailable — you may still try to join.',
    beforeYouJoin: 'Before you join',
    tip1: 'Use headphones to avoid echo.',
    tip2: 'Make sure your face is well-lit.',
    tip3: 'Close bandwidth-heavy tabs (video, downloads).',
  },
} as const;

export interface PreJoinContext {
  sessionId: string;
  courseTitle: string | null;
  otherName: string;
  userName: string;
  scheduledAt: string | null;
  durationMin: number | null;
}

// Mirror of the server-side JOIN_GRACE_MS in /api/livekit/token. Kept in sync
// by convention — if the server changes its window, update this too.
const JOIN_GRACE_MS = 15 * 60_000;

type Gate =
  | { kind: 'too_early'; opensInMs: number }
  | { kind: 'open' }
  | { kind: 'expired' }
  | { kind: 'unknown_schedule' };

function computeGate(scheduledAt: string | null, durationMin: number | null, now: number): Gate {
  if (!scheduledAt || durationMin == null) return { kind: 'unknown_schedule' };
  const start = new Date(scheduledAt).getTime();
  const end = start + durationMin * 60_000;
  if (now < start - JOIN_GRACE_MS) {
    return { kind: 'too_early', opensInMs: start - JOIN_GRACE_MS - now };
  }
  if (now > end + JOIN_GRACE_MS) return { kind: 'expired' };
  return { kind: 'open' };
}

function formatCountdown(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  if (h > 0) return `${h}h ${m}m`;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

/**
 * Two-phase room UX: this pane runs before we mint a LiveKit token. It gives
 * the user a chance to check their camera + mic (via LiveKit's PreJoin prefab)
 * and shows a live countdown so they can't punch through the 15-minute join
 * window the token endpoint enforces.
 */
export function PreJoinPanel({
  context,
  onReady,
}: {
  context: PreJoinContext;
  onReady: (choices: LocalUserChoices) => void;
}) {
  const { lang } = useLang();
  const lc = COPY[lang];
  const [now, setNow] = useState(() => Date.now());
  const [permError, setPermError] = useState<string | null>(null);
  const [earlyAttempts, setEarlyAttempts] = useState(0);

  useEffect(() => {
    // Align the first tick to the next whole-second boundary so the visible
    // countdown steps at the same moment regardless of when the component
    // mounted. Without this the "Opens in 4:23" flicker looks jittery
    // because setInterval fires 400ms into one second, then again 400ms
    // into the next, and the rounded label changes at inconsistent offsets.
    const msUntilNextSecond = 1000 - (Date.now() % 1000);
    let intervalId: ReturnType<typeof setInterval> | null = null;
    const alignId = setTimeout(() => {
      setNow(Date.now());
      intervalId = setInterval(() => setNow(Date.now()), 1_000);
    }, msUntilNextSecond);
    return () => {
      clearTimeout(alignId);
      if (intervalId) clearInterval(intervalId);
    };
  }, []);

  const gate = computeGate(context.scheduledAt, context.durationMin, now);
  const canJoin = gate.kind === 'open' || gate.kind === 'unknown_schedule';

  const joinLabel =
    gate.kind === 'too_early'
      ? lc.opensIn(formatCountdown(gate.opensInMs))
      : gate.kind === 'expired'
        ? lc.sessionEnded
        : lc.joinClassroom;

  return (
    <div className="flex min-h-dvh flex-col bg-slate-50 dark:bg-slate-950">
      <header className="pt-safe flex shrink-0 items-center gap-3 border-b border-slate-200 bg-white px-4 py-3 sm:px-6 dark:border-slate-800 dark:bg-slate-900">
        <Button variant="outline" size="sm" render={<Link href="/my-sessions" />}>
          <ArrowLeft />
          {lc.back}
        </Button>
        <div className="min-w-0 flex-1">
          <p className="type-eyebrow">{lc.eyebrow}</p>
          <p className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
            {context.courseTitle || lc.englishLesson}
            <span className="text-muted-foreground font-normal">
              {lc.withPrefix(context.otherName)}
            </span>
          </p>
        </div>
      </header>

      <div className="pb-safe mx-auto flex w-full max-w-6xl flex-1 flex-col gap-6 px-4 py-6 sm:px-6 lg:flex-row lg:items-start">
        <div className="w-full lg:flex-1">
          <div className="vielang-prejoin rounded-2xl border border-slate-200 bg-white p-2 shadow-sm dark:border-slate-800 dark:bg-slate-900">
            <PreJoin
              onSubmit={onReady}
              onValidate={() => {
                if (!canJoin) {
                  setEarlyAttempts((n) => n + 1);
                  return false;
                }
                return true;
              }}
              onError={(err) => {
                const name = (err as { name?: string })?.name || '';
                if (name === 'NotAllowedError' || /denied|permission/i.test(err.message || '')) {
                  setPermError(lc.permBlocked);
                } else if (name === 'NotFoundError') {
                  setPermError(lc.permNotFound);
                } else {
                  setPermError(err.message || lc.permGeneric);
                }
              }}
              defaults={{ username: context.userName }}
              joinLabel={joinLabel}
              persistUserChoices
              data-lk-theme="default"
            />
          </div>
        </div>

        <aside className="w-full space-y-4 lg:w-80">
          <PreJoinAudioMeter />
          <ScheduleCard lang={lang} context={context} gate={gate} />
          {permError && (
            <Alert variant="destructive">
              <AlertTitle>{lc.mediaBlockedTitle}</AlertTitle>
              <AlertDescription>{permError}</AlertDescription>
            </Alert>
          )}
          {earlyAttempts > 0 && !canJoin && (
            <Alert>
              <AlertTitle>{lc.notYetTitle}</AlertTitle>
              <AlertDescription>
                {gate.kind === 'too_early' ? lc.notYetOpens : lc.notYetEnded}
              </AlertDescription>
            </Alert>
          )}
          <TipsCard lang={lang} />
        </aside>
      </div>

      {/*
        The LiveKit prefab hard-codes a username input into its join form. Our
        users are already authenticated so we prefill + hide it — the join
        button (`.lk-join-button`) stays visible.
      */}
      <style>{`.vielang-prejoin .lk-username-container input.lk-form-control{display:none}`}</style>
    </div>
  );
}

function ScheduleCard({
  lang,
  context,
  gate,
}: {
  lang: Lang;
  context: PreJoinContext;
  gate: Gate;
}) {
  const lc = COPY[lang];
  const scheduled = context.scheduledAt
    ? new Date(context.scheduledAt).toLocaleString(lang === 'VN' ? 'vi-VN' : undefined, {
        weekday: 'short',
        day: '2-digit',
        month: 'short',
        hour: '2-digit',
        minute: '2-digit',
      })
    : null;

  return (
    <Card size="sm">
      <CardContent className="space-y-3">
        <div className="flex items-start gap-3">
          <div className="bg-brand/10 mt-0.5 flex size-9 items-center justify-center rounded-lg">
            <Clock className="text-brand size-4" />
          </div>
          <div className="min-w-0 flex-1 space-y-0.5">
            <p className="type-eyebrow">{lc.scheduleSession}</p>
            {scheduled ? (
              <p className="text-sm font-medium text-slate-900 dark:text-slate-100">{scheduled}</p>
            ) : (
              <p className="text-muted-foreground text-sm">{lc.scheduleUnavailable}</p>
            )}
            <p className="text-muted-foreground text-xs">
              {context.durationMin ? lc.minutes(context.durationMin) : ''}
            </p>
          </div>
        </div>
        <div className="flex items-start gap-3">
          <div className="bg-brand/10 mt-0.5 flex size-9 items-center justify-center rounded-lg">
            <User className="text-brand size-4" />
          </div>
          <div className="min-w-0 flex-1">
            <p className="type-eyebrow">{lc.withCol}</p>
            <p className="truncate text-sm font-medium text-slate-900 dark:text-slate-100">
              {context.otherName}
            </p>
          </div>
        </div>
        <div className="border-border border-t pt-3">
          {gate.kind === 'open' && (
            <p className="text-xs font-medium text-emerald-700 dark:text-emerald-400">
              {lc.classroomOpen}
            </p>
          )}
          {gate.kind === 'too_early' && (
            <p className="text-muted-foreground text-xs">
              {lc.opensInLabel}{' '}
              <span
                className="font-semibold text-slate-900 tabular-nums dark:text-slate-100"
                aria-live="polite"
              >
                {formatCountdown(gate.opensInMs)}
              </span>
            </p>
          )}
          {gate.kind === 'expired' && (
            <p className="text-destructive text-xs font-medium">{lc.scheduleEnded}</p>
          )}
          {gate.kind === 'unknown_schedule' && (
            <p className="text-muted-foreground text-xs">{lc.scheduleFallback}</p>
          )}
        </div>
      </CardContent>
    </Card>
  );
}

function TipsCard({ lang }: { lang: Lang }) {
  const lc = COPY[lang];
  return (
    <Card size="sm">
      <CardContent>
        <p className="type-eyebrow mb-2">{lc.beforeYouJoin}</p>
        <ul className="text-muted-foreground space-y-1.5 text-xs leading-relaxed">
          <li>• {lc.tip1}</li>
          <li>• {lc.tip2}</li>
          <li>• {lc.tip3}</li>
        </ul>
      </CardContent>
    </Card>
  );
}
