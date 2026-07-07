import 'server-only';
import type { NextRequest } from 'next/server';
import { supabase } from './supabase';
import { childLogger } from './logger';

/**
 * Audit log helper. Records sensitive mutations (role changes, tutor approve/
 * reject, session cancellations) so we can trace who did what when.
 *
 * Fire-and-forget: writes are async but never awaited by the caller in the
 * request path — if the audit insert fails, we log a warning and move on
 * rather than fail the underlying operation. The alternative (fail closed)
 * makes the API brittle in ways users can't recover from.
 */

const log = childLogger('audit');

export type AuditAction =
  | 'user.role_change'
  | 'user.enable'
  | 'user.disable'
  | 'tutor.approve'
  | 'tutor.reject'
  | 'session.create'
  | 'session.cancel'
  | 'session.reschedule'
  | 'session.confirm'
  | 'session.admit'
  | 'session.deny'
  | 'session.mute'
  | 'session.unmute'
  | 'session.remove'
  | 'session.mute_all'
  | 'session.end_room'
  | 'course.publish'
  | 'course.unpublish'
  | 'course.delete'
  | 'review.moderate';

export interface AuditActor {
  id: string;
  email?: string | null;
  role: string;
}

export interface AuditEntry {
  actor: AuditActor;
  action: AuditAction;
  entity: string;
  entityId?: string | null;
  before?: unknown;
  after?: unknown;
  metadata?: Record<string, unknown>;
  request?: NextRequest;
}

/**
 * Write an audit entry. Never throws — errors are logged and swallowed.
 * The caller should NOT await this in the hot path; use `.catch()` if you
 * want to observe the write failure separately.
 */
export async function recordAudit(entry: AuditEntry): Promise<void> {
  try {
    const ip =
      entry.request?.headers.get('x-forwarded-for')?.split(',')[0].trim() ||
      entry.request?.headers.get('x-real-ip') ||
      null;
    const ua = entry.request?.headers.get('user-agent') || null;

    const { error } = await supabase.from('audit_log').insert({
      actor_id: entry.actor.id,
      actor_email: entry.actor.email ?? null,
      actor_role: entry.actor.role,
      action: entry.action,
      entity: entry.entity,
      entity_id: entry.entityId ?? null,
      before: entry.before ?? null,
      after: entry.after ?? null,
      ip,
      user_agent: ua,
      metadata: entry.metadata ?? null,
    });
    if (error) {
      log.warn({ err: error, action: entry.action, entity: entry.entity }, 'audit insert failed');
    }
  } catch (err) {
    log.warn({ err, action: entry.action, entity: entry.entity }, 'audit exception');
  }
}
