'use client';

import { useEffect, useState } from 'react';
import Link from 'next/link';
import { ArrowLeft, CalendarDays, Clock, MessageCircle, Users, Timer } from 'lucide-react';
import { getAuthHeaders } from '@/contexts/AuthContext';
import type { SessionStatus } from '@/lib/types';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Badge, type badgeVariants } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Skeleton } from '@/components/ui/skeleton';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import type { VariantProps } from 'class-variance-authority';
import { RoomMessageBubble } from '@/components/session/room/RoomMessageBubble';

export interface SessionDetailContext {
  sessionId: string;
  status: string;
  scheduledAt: string;
  durationMin: number;
  courseTitle: string | null;
  tutorName: string | null;
  studentName: string | null;
  viewerRole: 'tutor' | 'student' | 'admin';
  myId: string;
}

interface StoredMessage {
  id: string;
  sender_id: string;
  sender_name: string | null;
  body: string;
  sent_at: string;
  client_id: string | null;
}

interface AttendanceRow {
  user_id: string;
  user_name: string | null;
  avatar: string | null;
  first_joined_at: string;
  last_left_at: string | null;
  total_duration_sec: number;
  event_count: number;
  still_present: boolean;
}

type StatusVariant = NonNullable<VariantProps<typeof badgeVariants>['variant']>;

// Status → Badge variant. Semantic mapping: terminal-negative uses destructive,
// active/live uses the primary tint, historic uses secondary (muted), and
// still-pending gets outline so it doesn't compete with tabs for attention.
const STATUS_VARIANT: Record<SessionStatus, StatusVariant> = {
  pending: 'outline',
  confirmed: 'secondary',
  live: 'default',
  completed: 'secondary',
  cancelled: 'destructive',
  no_show: 'destructive',
};

/**
 * Post-session review view. Two tabs — Transcript (chat history) + Attendance
 * (aggregated join/leave timeline). Data fetches are lazy per tab so a user
 * that only cares about the chat doesn't pay for an unused attendance query.
 */
export function SessionDetailClient({ context }: { context: SessionDetailContext }) {
  const scheduled = new Date(context.scheduledAt).toLocaleString(undefined, {
    weekday: 'short',
    day: '2-digit',
    month: 'short',
    year: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
  const otherName = context.viewerRole === 'student' ? context.tutorName : context.studentName;

  const knownStatuses: SessionStatus[] = [
    'pending',
    'confirmed',
    'live',
    'completed',
    'cancelled',
    'no_show',
  ];
  const statusKey = knownStatuses.includes(context.status as SessionStatus)
    ? (context.status as SessionStatus)
    : 'completed';

  return (
    <main className="mx-auto flex w-full max-w-4xl flex-col gap-6 px-4 py-6 sm:px-6 lg:px-8">
      <Button
        variant="ghost"
        size="sm"
        render={<Link href="/my-sessions" />}
        className="text-muted-foreground self-start"
      >
        <ArrowLeft />
        Back to my sessions
      </Button>

      <Card>
        <CardContent className="p-6">
          <div className="flex items-start justify-between gap-3">
            <div className="min-w-0 flex-1 space-y-1.5">
              <p className="type-eyebrow">Session</p>
              <h1 className="font-heading text-xl font-bold text-slate-900 dark:text-slate-100">
                {context.courseTitle || 'English lesson'}
              </h1>
              {otherName && (
                <p className="text-muted-foreground text-sm">
                  {context.viewerRole === 'student' ? 'With' : 'Student'} · {otherName}
                </p>
              )}
            </div>
            <Badge variant={STATUS_VARIANT[statusKey]} className="uppercase">
              {context.status.replace('_', ' ')}
            </Badge>
          </div>

          <div className="text-muted-foreground mt-4 flex flex-wrap items-center gap-x-5 gap-y-1 text-xs">
            <span className="flex items-center gap-1.5">
              <CalendarDays className="size-3.5" />
              {scheduled}
            </span>
            <span className="flex items-center gap-1.5">
              <Clock className="size-3.5" />
              {context.durationMin} minutes
            </span>
          </div>
        </CardContent>
      </Card>

      <Tabs defaultValue="transcript" className="gap-4">
        <TabsList className="w-full max-w-sm">
          <TabsTrigger value="transcript">
            <MessageCircle />
            Transcript
          </TabsTrigger>
          <TabsTrigger value="attendance">
            <Users />
            Attendance
          </TabsTrigger>
        </TabsList>
        <TabsContent value="transcript">
          <TranscriptPanel context={context} />
        </TabsContent>
        <TabsContent value="attendance">
          <AttendancePanel context={context} />
        </TabsContent>
      </Tabs>
    </main>
  );
}

function TranscriptPanel({ context }: { context: SessionDetailContext }) {
  const [state, setState] = useState<
    | { kind: 'loading' }
    | { kind: 'error'; message: string }
    | { kind: 'ready'; messages: StoredMessage[] }
  >({ kind: 'loading' });

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await fetch(`/api/sessions/${context.sessionId}/messages`, {
          headers: await getAuthHeaders(),
        });
        const data = await res.json().catch(() => ({}));
        if (cancelled) return;
        if (!res.ok) {
          setState({ kind: 'error', message: data?.error || `HTTP ${res.status}` });
          return;
        }
        setState({ kind: 'ready', messages: (data.messages || []) as StoredMessage[] });
      } catch (err) {
        if (!cancelled) {
          setState({ kind: 'error', message: (err as Error)?.message || 'Network error' });
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [context.sessionId]);

  if (state.kind === 'loading') return <TranscriptSkeleton />;
  if (state.kind === 'error') return <PanelError message={state.message} />;
  if (state.messages.length === 0) {
    return (
      <PanelEmpty
        icon={<MessageCircle className="size-6" />}
        title="No chat during this session"
        subtitle="Chat messages sent during a live session appear here after it ends."
      />
    );
  }
  return (
    <Card>
      <CardContent className="space-y-2 p-4">
        {state.messages.map((m) => (
          <RoomMessageBubble
            key={m.id}
            body={m.body}
            senderName={m.sender_name || 'Guest'}
            mine={m.sender_id === context.myId}
            time={new Date(m.sent_at).toLocaleTimeString(undefined, {
              hour: '2-digit',
              minute: '2-digit',
            })}
          />
        ))}
      </CardContent>
    </Card>
  );
}

function AttendancePanel({ context }: { context: SessionDetailContext }) {
  const [state, setState] = useState<
    | { kind: 'loading' }
    | { kind: 'error'; message: string }
    | { kind: 'ready'; rows: AttendanceRow[] }
  >({ kind: 'loading' });

  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await fetch(`/api/sessions/${context.sessionId}/attendance`, {
          headers: await getAuthHeaders(),
        });
        const data = await res.json().catch(() => ({}));
        if (cancelled) return;
        if (!res.ok) {
          setState({ kind: 'error', message: data?.error || `HTTP ${res.status}` });
          return;
        }
        setState({ kind: 'ready', rows: (data.attendance || []) as AttendanceRow[] });
      } catch (err) {
        if (!cancelled) {
          setState({ kind: 'error', message: (err as Error)?.message || 'Network error' });
        }
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [context.sessionId]);

  if (state.kind === 'loading') return <AttendanceSkeleton />;
  if (state.kind === 'error') return <PanelError message={state.message} />;
  if (state.rows.length === 0) {
    return (
      <PanelEmpty
        icon={<Users className="size-6" />}
        title="No attendance records"
        subtitle="Attendance is captured from LiveKit as participants join and leave. This session may pre-date the tracking rollout, or nobody made it in."
      />
    );
  }
  return (
    <Card>
      <CardContent className="p-0">
        <ul className="divide-border divide-y">
          {state.rows.map((r) => (
            <AttendanceItem key={r.user_id} row={r} />
          ))}
        </ul>
      </CardContent>
    </Card>
  );
}

function AttendanceItem({ row }: { row: AttendanceRow }) {
  const displayName = row.user_name || 'Guest';
  const joined = new Date(row.first_joined_at).toLocaleTimeString(undefined, {
    hour: '2-digit',
    minute: '2-digit',
  });
  const left = row.last_left_at
    ? new Date(row.last_left_at).toLocaleTimeString(undefined, {
        hour: '2-digit',
        minute: '2-digit',
      })
    : null;
  return (
    <li className="flex items-center gap-4 px-5 py-3">
      <div className="bg-brand/10 text-brand flex size-10 shrink-0 items-center justify-center rounded-full text-sm font-semibold uppercase">
        {displayName.slice(0, 1)}
      </div>
      <div className="min-w-0 flex-1">
        <p className="truncate text-sm font-medium text-slate-900 dark:text-slate-100">
          {displayName}
        </p>
        <p className="type-eyebrow mt-0.5">
          Joined {joined}
          {left ? ` · left ${left}` : row.still_present ? ' · still in room' : ''}
          {row.event_count > 1 ? ` · ${row.event_count} joins` : ''}
        </p>
      </div>
      <div className="bg-muted text-foreground flex shrink-0 items-center gap-1.5 rounded-lg px-3 py-1.5 text-xs font-semibold tabular-nums">
        <Timer className="size-3.5" />
        {formatDuration(row.total_duration_sec)}
      </div>
    </li>
  );
}

function formatDuration(sec: number) {
  if (sec <= 0) return '0m';
  const totalMin = Math.round(sec / 60);
  if (totalMin < 60) return `${totalMin}m`;
  const h = Math.floor(totalMin / 60);
  const m = totalMin % 60;
  return `${h}h ${m.toString().padStart(2, '0')}m`;
}

function TranscriptSkeleton() {
  return (
    <Card>
      <CardContent className="space-y-3 p-4" aria-busy="true" aria-live="polite">
        {[0, 1, 2].map((i) => (
          <div key={i} className={i % 2 === 0 ? 'flex justify-start' : 'flex justify-end'}>
            <Skeleton
              className={i % 2 === 0 ? 'h-10 w-2/3 rounded-2xl' : 'h-10 w-1/2 rounded-2xl'}
            />
          </div>
        ))}
      </CardContent>
    </Card>
  );
}

function AttendanceSkeleton() {
  return (
    <Card>
      <CardContent className="p-0" aria-busy="true" aria-live="polite">
        <ul className="divide-border divide-y">
          {[0, 1, 2].map((i) => (
            <li key={i} className="flex items-center gap-4 px-5 py-3">
              <Skeleton className="size-10 shrink-0 rounded-full" />
              <div className="min-w-0 flex-1 space-y-1.5">
                <Skeleton className="h-4 w-32" />
                <Skeleton className="h-3 w-48" />
              </div>
              <Skeleton className="h-7 w-16 rounded-lg" />
            </li>
          ))}
        </ul>
      </CardContent>
    </Card>
  );
}

function PanelError({ message }: { message: string }) {
  return (
    <Alert variant="destructive">
      <AlertTitle>Something went wrong</AlertTitle>
      <AlertDescription>{message}</AlertDescription>
    </Alert>
  );
}

function PanelEmpty({
  icon,
  title,
  subtitle,
}: {
  icon: React.ReactNode;
  title: string;
  subtitle: string;
}) {
  return (
    <Card>
      <CardContent className="flex flex-col items-center justify-center gap-3 px-6 py-12 text-center">
        <div className="bg-muted text-muted-foreground flex size-12 items-center justify-center rounded-full">
          {icon}
        </div>
        <p className="text-sm font-semibold text-slate-900 dark:text-slate-100">{title}</p>
        <p className="text-muted-foreground max-w-sm text-xs">{subtitle}</p>
      </CardContent>
    </Card>
  );
}
