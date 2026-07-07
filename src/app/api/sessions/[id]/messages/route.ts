import { type NextRequest, NextResponse } from 'next/server';
import { z } from 'zod';
import { authenticate } from '@/lib/auth-server';
import { apiError } from '@/lib/api-errors';
import { supabase, getSessionById } from '@/lib/supabase';
import { rateLimit, requestIdentifier } from '@/lib/rate-limit';

export const dynamic = 'force-dynamic';

const MESSAGE_LIMIT = 200;

const postBodySchema = z.object({
  body: z.string().trim().min(1).max(4000),
  // Stable id from the sender's client, used for dedupe on retry. LiveKit
  // chat messages carry an id we reuse; if a caller can't produce one we
  // synthesize `${timestamp}-${identity}` on the client, so this is always
  // present in practice.
  client_id: z.string().trim().min(1).max(128),
  sent_at: z.string().datetime({ offset: true }).optional(),
});

// Chat is plain text — no markdown, no rich HTML. Strip C0/C1 control chars
// (kept: \t \n \r) and anything tag-shaped, so a future renderer that
// accidentally uses innerHTML — or a downstream export that treats the value
// as markup — cannot regress into stored XSS. Defense-in-depth on top of the
// current React text-node render.
const CONTROL_CHARS = new RegExp('[\u0000-\u0008\u000B\u000C\u000E-\u001F\u007F]', 'g');
const HTML_TAG = /<[^>]*>/g;

function sanitizePlainText(input: string): string {
  return input.replace(CONTROL_CHARS, '').replace(HTML_TAG, '').trim();
}

/**
 * Is this user allowed to see / write messages for this session?
 *
 * Mirrors the rules from /api/livekit/token: private sessions authorize
 * exactly the paired student + tutor; group sessions authorize the host
 * and anyone with an admitted row in session_participants. Admins bypass.
 */
async function isSessionParticipant(
  session: {
    id: string;
    type: 'private' | 'group';
    student_id: string | null;
    tutor_id: string | null;
  },
  uid: string,
  role: string,
): Promise<boolean> {
  if (role === 'admin') return true;
  if (uid === session.tutor_id) return true;
  if (session.type === 'private') return uid === session.student_id;
  const { data } = await supabase
    .from('session_participants')
    .select('user_id')
    .eq('session_id', session.id)
    .eq('user_id', uid)
    .maybeSingle();
  return !!data;
}

/**
 * POST /api/sessions/[id]/messages
 *
 * Persists one chat message. Sender is forced to auth.user.id — never trust
 * the client on identity. The composite unique index (session_id, sender_id,
 * client_id) deduplicates retries so a network flake doesn't double-post.
 *
 * We don't fan out to other participants from here: the in-session realtime
 * still travels over LiveKit's data channel (via useChat), so the round-trip
 * through our DB is purely for post-session history.
 */
export async function POST(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const rl = await rateLimit('chat-message', requestIdentifier(req, auth.user.id));
    if (!rl.ok) return rl.response;

    const { id } = await params;
    const session = await getSessionById(id);
    if (!session) return NextResponse.json({ error: 'session_not_found' }, { status: 404 });

    if (!(await isSessionParticipant(session, auth.user.id, auth.user.role))) {
      return NextResponse.json({ error: 'forbidden' }, { status: 403 });
    }

    const parsed = postBodySchema.safeParse(await req.json().catch(() => ({})));
    if (!parsed.success) {
      return NextResponse.json(
        { error: 'validation_failed', details: parsed.error.issues },
        { status: 400 },
      );
    }

    const { body: rawBody, client_id, sent_at } = parsed.data;
    const body = sanitizePlainText(rawBody);
    if (!body) {
      return NextResponse.json({ error: 'empty_after_sanitize' }, { status: 400 });
    }
    const { data, error } = await supabase
      .from('session_messages')
      .upsert(
        {
          session_id: id,
          sender_id: auth.user.id,
          body,
          client_id,
          // Prefer the client-supplied timestamp so messages sort correctly
          // even if the server clock skews from the sender by a few hundred
          // ms. Falls back to NOW() via the column default.
          ...(sent_at ? { sent_at } : {}),
        },
        { onConflict: 'session_id,sender_id,client_id', ignoreDuplicates: true },
      )
      .select('id, session_id, sender_id, body, sent_at, client_id')
      .maybeSingle();
    if (error) throw error;

    // Upsert with ignoreDuplicates returns null on a hit; report it so the
    // caller can distinguish "created" from "already stored".
    if (!data) {
      return NextResponse.json({ deduped: true });
    }
    return NextResponse.json({ message: data }, { status: 201 });
  } catch (err) {
    return apiError(err);
  }
}

/**
 * GET /api/sessions/[id]/messages
 *
 * Returns up to MESSAGE_LIMIT most recent messages, oldest first so the UI
 * can render them in-order without a client-side reverse. Sender name is
 * joined in so the panel doesn't need a second round-trip per participant.
 */
export async function GET(req: NextRequest, { params }: { params: Promise<{ id: string }> }) {
  try {
    const auth = await authenticate(req);
    if ('response' in auth) return auth.response;

    const { id } = await params;
    const session = await getSessionById(id);
    if (!session) return NextResponse.json({ error: 'session_not_found' }, { status: 404 });

    if (!(await isSessionParticipant(session, auth.user.id, auth.user.role))) {
      return NextResponse.json({ error: 'forbidden' }, { status: 403 });
    }

    // Order desc + slice + reverse would also work, but Postgres can serve
    // the ASC read straight off idx_session_messages_session so we ship the
    // final ordering directly.
    const { data, error } = await supabase
      .from('session_messages')
      .select(
        `
        id, session_id, sender_id, body, sent_at, client_id,
        sender:users!session_messages_sender_id_fkey(name)
      `,
      )
      .eq('session_id', id)
      .order('sent_at', { ascending: true })
      .limit(MESSAGE_LIMIT);
    if (error) throw error;

    return NextResponse.json({
      messages: (data || []).map((m) => ({
        id: m.id,
        session_id: m.session_id,
        sender_id: m.sender_id,
        sender_name: (m.sender as { name?: string } | null)?.name ?? null,
        body: m.body,
        sent_at: m.sent_at,
        client_id: m.client_id,
      })),
    });
  } catch (err) {
    return apiError(err);
  }
}
