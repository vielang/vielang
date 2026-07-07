import 'server-only';
import { Resend } from 'resend';
import { render } from '@react-email/render';
import type { ReactElement } from 'react';
import { supabase } from './supabase';
import { childLogger } from './logger';

/**
 * Email pipeline with graceful degradation.
 *
 *  - When RESEND_API_KEY is set → real emails go out.
 *  - When it's missing (dev without an account, preview builds, CI) → the
 *    call is a no-op and the caller is told which kind of email would have
 *    fired via the returned `{ delivered: false, reason: 'no_provider' }`.
 *
 * Every delivery attempt (success or drop) writes to `notification_log` so
 * we can build reports later ("how many session reminders fired last month",
 * "which templates bounce"). Failures don't throw — the calling route should
 * never fail a booking because email is down.
 *
 * Templates live in `src/lib/emails/` as React Email components. Passing the
 * component to `send()` rather than a rendered string keeps the type-safe
 * props API and dev iteration fast.
 */

const log = childLogger('email');

const FROM = process.env.EMAIL_FROM || 'VieLang <no-reply@vielang.com>';
const REPLY_TO = process.env.EMAIL_REPLY_TO || undefined;

let _resend: Resend | null = null;
function getResend(): Resend | null {
  if (_resend) return _resend;
  const apiKey = process.env.RESEND_API_KEY;
  if (!apiKey) return null;
  _resend = new Resend(apiKey);
  return _resend;
}

export type EmailKind =
  | 'session.confirmation'
  | 'session.reminder_24h'
  | 'session.reminder_1h'
  | 'session.review_request'
  | 'tutor.approved'
  | 'tutor.rejected';

export interface SendEmailArgs {
  /** Machine tag for the notification_log entry — powers rollups + throttling. */
  kind: EmailKind;
  to: string;
  subject: string;
  /** React Email element — rendered to HTML server-side via @react-email/render. */
  template: ReactElement;
  /** Fall-back plain-text body for clients that block HTML. */
  text?: string;
  /** Optional foreign key to session — indexed for "did the reminder fire?" queries. */
  sessionId?: string | null;
  /** Optional user id — indexed for "notifications for user X" queries. */
  userId?: string | null;
  /** Arbitrary extra JSON tags for reporting. */
  metadata?: Record<string, unknown>;
}

export interface SendEmailResult {
  delivered: boolean;
  providerId?: string;
  reason?: 'no_provider' | 'no_recipient' | 'provider_error';
  error?: string;
}

export async function sendEmail(args: SendEmailArgs): Promise<SendEmailResult> {
  const resend = getResend();
  if (!args.to || !args.to.trim()) {
    await logDelivery(args, { delivered: false, reason: 'no_recipient' });
    return { delivered: false, reason: 'no_recipient' };
  }
  if (!resend) {
    log.debug({ kind: args.kind, to: args.to }, 'email skipped — RESEND_API_KEY missing');
    await logDelivery(args, { delivered: false, reason: 'no_provider' });
    return { delivered: false, reason: 'no_provider' };
  }

  try {
    const html = await render(args.template);
    const text = args.text ?? (await render(args.template, { plainText: true }));
    const { data, error } = await resend.emails.send({
      from: FROM,
      to: args.to,
      subject: args.subject,
      html,
      text,
      ...(REPLY_TO ? { replyTo: REPLY_TO } : {}),
    });
    if (error) {
      log.warn({ err: error, kind: args.kind }, 'resend rejected the send');
      await logDelivery(args, {
        delivered: false,
        reason: 'provider_error',
        error: error.message,
      });
      return { delivered: false, reason: 'provider_error', error: error.message };
    }
    await logDelivery(args, { delivered: true, providerId: data?.id });
    return { delivered: true, providerId: data?.id };
  } catch (err: unknown) {
    const msg = err instanceof Error ? err.message : String(err);
    log.error({ err, kind: args.kind }, 'email send threw');
    await logDelivery(args, { delivered: false, reason: 'provider_error', error: msg });
    return { delivered: false, reason: 'provider_error', error: msg };
  }
}

async function logDelivery(args: SendEmailArgs, result: SendEmailResult): Promise<void> {
  try {
    await supabase.from('notification_log').insert({
      kind: args.kind,
      recipient: args.to,
      session_id: args.sessionId ?? null,
      user_id: args.userId ?? null,
      delivered: result.delivered,
      provider_id: result.providerId ?? null,
      reason: result.reason ?? null,
      error: result.error ?? null,
      metadata: args.metadata ?? null,
    });
  } catch (err) {
    // Never let a logging failure break the send path — the send itself
    // already resolved either way. Just note the drop.
    log.warn({ err, kind: args.kind }, 'notification_log insert failed');
  }
}
