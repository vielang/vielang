import { type NextRequest, NextResponse } from 'next/server';
import { supabase } from '@/lib/supabase';
import { sendEmail, type EmailKind } from '@/lib/email';
import { SessionReminderEmail } from '@/lib/emails/SessionReminder';
import { childLogger } from '@/lib/logger';

export const dynamic = 'force-dynamic';
export const runtime = 'nodejs';

const log = childLogger('cron-reminders');

/**
 * Vercel Cron endpoint — fires reminder emails for upcoming sessions.
 *
 * Runs on the cron schedule declared in `vercel.json` (recommended: every
 * 10 minutes). Each invocation:
 *   1. Finds sessions starting in the 24h window ± 10min → sends T-24h.
 *   2. Finds sessions starting in the 1h window ± 10min  → sends T-1h.
 *   3. Skips any session that already has a notification_log row for that
 *      (session_id, kind) pair — dedupe against overlapping runs / retries.
 *
 * Auth: Vercel Cron adds `Authorization: Bearer <CRON_SECRET>` (when the env
 * var is set). We require it; missing/mismatched → 401.
 */
export async function GET(req: NextRequest) {
  const secret = process.env.CRON_SECRET;
  const authz = req.headers.get('authorization');
  if (!secret || authz !== `Bearer ${secret}`) {
    return NextResponse.json({ error: 'unauthorized' }, { status: 401 });
  }

  const now = Date.now();
  const window24h = { minMs: 24 * 3600_000 - 10 * 60_000, maxMs: 24 * 3600_000 + 10 * 60_000 };
  const window1h = { minMs: 60 * 60_000 - 10 * 60_000, maxMs: 60 * 60_000 + 10 * 60_000 };

  const [count24h, count1h] = await Promise.all([
    fireReminders({
      window: '24h',
      kind: 'session.reminder_24h',
      fromMs: now + window24h.minMs,
      toMs: now + window24h.maxMs,
    }),
    fireReminders({
      window: '1h',
      kind: 'session.reminder_1h',
      fromMs: now + window1h.minMs,
      toMs: now + window1h.maxMs,
    }),
  ]);

  log.info({ count24h, count1h }, 'reminder cron tick complete');
  return NextResponse.json({ ok: true, sent: { t24h: count24h, t1h: count1h } });
}

async function fireReminders(opts: {
  window: '24h' | '1h';
  kind: EmailKind;
  fromMs: number;
  toMs: number;
}): Promise<number> {
  const from = new Date(opts.fromMs).toISOString();
  const to = new Date(opts.toMs).toISOString();

  // Pull sessions in the target window that still count as upcoming. Left-
  // joins carry the student's email/name + tutor's name in one round-trip.
  const { data: sessions, error } = await supabase
    .from('sessions')
    .select(
      `
      id, scheduled_at, duration_min, student_id,
      student:users!sessions_student_id_fkey(name, email, timezone),
      tutor:users!sessions_tutor_id_fkey(name)
    `,
    )
    .in('status', ['pending', 'confirmed'])
    .gte('scheduled_at', from)
    .lte('scheduled_at', to);
  if (error) {
    log.error({ err: error, window: opts.window }, 'reminder query failed');
    return 0;
  }
  if (!sessions || sessions.length === 0) return 0;

  // Dedupe: skip sessions we already reminded for this kind. One query for
  // the batch beats N per-session lookups.
  const ids = sessions.map((s) => s.id);
  const { data: already } = await supabase
    .from('notification_log')
    .select('session_id')
    .in('session_id', ids)
    .eq('kind', opts.kind);
  const alreadySet = new Set((already || []).map((r: { session_id: string }) => r.session_id));

  const siteUrl = process.env.NEXT_PUBLIC_SITE_URL || 'https://vielang.com';

  type SessionRow = {
    id: string;
    scheduled_at: string;
    student_id: string;
    student: { name: string; email: string; timezone: string | null } | null;
    tutor: { name: string } | null;
  };
  const rows = sessions as unknown as SessionRow[];
  const pending = rows.filter((s) => !alreadySet.has(s.id) && (s.student?.email ?? ''));

  // Concurrency cap: fire up to 5 emails in parallel per chunk. Trade-off:
  // higher = faster batch runtime, but Resend rate-limits and a runaway
  // burst risks bounce-back. Five hits a sweet spot for 60s cron budgets.
  const CONCURRENCY = 5;
  // Hard per-email deadline. Resend usually replies in <500 ms; a hang
  // beyond 3 s means the provider is misbehaving and we'd rather log a
  // provider_error than let one slow send block the remaining 40 recipients.
  const PER_EMAIL_TIMEOUT_MS = 3_000;

  let sent = 0;
  for (let i = 0; i < pending.length; i += CONCURRENCY) {
    const chunk = pending.slice(i, i + CONCURRENCY);
    const results = await Promise.allSettled(
      chunk.map((s) => withTimeout(sendOne(s, opts, siteUrl), PER_EMAIL_TIMEOUT_MS)),
    );
    for (const r of results) {
      if (r.status === 'fulfilled' && (r.value.delivered || r.value.reason === 'no_provider')) {
        sent += 1;
      } else if (r.status === 'rejected') {
        log.warn({ err: r.reason, window: opts.window }, 'reminder send rejected / timed out');
      }
    }
  }
  return sent;
}

async function sendOne(
  s: {
    id: string;
    scheduled_at: string;
    student_id: string;
    student: { name: string; email: string; timezone: string | null } | null;
    tutor: { name: string } | null;
  },
  opts: { window: '24h' | '1h'; kind: EmailKind },
  siteUrl: string,
) {
  const recipient = s.student?.email || '';
  // Format the scheduled time in the student's own timezone so the reminder
  // reads "17:30 ICT" for a Ho Chi Minh user instead of the "10:30 UTC" the
  // Vercel cron would otherwise render. Fall back to Asia/Ho_Chi_Minh (our
  // default per users table) then UTC.
  const studentTz = s.student?.timezone || 'Asia/Ho_Chi_Minh';
  const scheduledAt = new Date(s.scheduled_at).toLocaleString('en-GB', {
    weekday: 'short',
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
    timeZoneName: 'short',
    timeZone: studentTz,
  });

  return sendEmail({
    kind: opts.kind,
    to: recipient,
    subject:
      opts.window === '1h'
        ? `Your VieLang session starts soon`
        : `Reminder: your VieLang session tomorrow`,
    template: SessionReminderEmail({
      studentName: s.student?.name || recipient.split('@')[0] || 'there',
      tutorName: s.tutor?.name || 'your tutor',
      scheduledAt,
      joinUrl: `${siteUrl}/session/${s.id}/room`,
      window: opts.window,
    }),
    sessionId: s.id,
    userId: s.student_id,
  });
}

/**
 * Race a promise against a timeout. If the timeout fires first, the returned
 * promise rejects with `Error('timeout')` — the caller can log + move on
 * without waiting for the underlying request to actually finish.
 */
function withTimeout<T>(p: Promise<T>, ms: number): Promise<T> {
  return new Promise<T>((resolve, reject) => {
    const timer = setTimeout(() => reject(new Error(`timeout after ${ms}ms`)), ms);
    p.then(
      (v) => {
        clearTimeout(timer);
        resolve(v);
      },
      (e) => {
        clearTimeout(timer);
        reject(e);
      },
    );
  });
}
