'use client';

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { MessageCircle, Send, Loader2 } from 'lucide-react';
import { toast } from 'sonner';
import { useChat, useLocalParticipant, useRemoteParticipants } from '@livekit/components-react';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { useLang } from '@/contexts';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  Sheet,
  SheetContent,
  SheetDescription,
  SheetHeader,
  SheetTitle,
} from '@/components/ui/sheet';
import { Textarea } from '@/components/ui/textarea';
import { RoomMessageBubble } from '@/components/session/room/RoomMessageBubble';

const COPY = {
  VN: {
    you: 'Bạn',
    guest: 'Khách',
    title: 'Chat buổi học',
    messagesCount: (n: number) => (n === 1 ? '1 tin nhắn' : `${n} tin nhắn`),
    empty: 'Chào một câu để bắt đầu trò chuyện.',
    placeholder: 'Nhập tin nhắn…',
    chatBtn: 'Chat',
    ariaOpenChat: (unread: number) => (unread > 0 ? `Mở chat, ${unread} tin chưa đọc` : 'Mở chat'),
    ariaMessage: 'Tin nhắn chat',
    ariaSend: 'Gửi tin nhắn',
    notSavedTitle: 'Tin nhắn không được lưu',
    notSavedDesc: 'Mọi người trong buổi học đã thấy, nhưng sẽ không xuất hiện trong lịch sử.',
    notSentTitle: 'Không gửi được tin',
    sendFallback: 'Gửi thất bại',
  },
  EN: {
    you: 'You',
    guest: 'Guest',
    title: 'Session chat',
    messagesCount: (n: number) => (n === 1 ? '1 message' : `${n} messages`),
    empty: 'Say hi to get the conversation started.',
    placeholder: 'Type a message…',
    chatBtn: 'Chat',
    ariaOpenChat: (unread: number) => (unread > 0 ? `Open chat, ${unread} unread` : 'Open chat'),
    ariaMessage: 'Chat message',
    ariaSend: 'Send message',
    notSavedTitle: 'Message not saved',
    notSavedDesc: 'Everyone in the call saw it, but it will be missing from the transcript later.',
    notSentTitle: 'Message not sent',
    sendFallback: 'Failed to send',
  },
} as const;

interface StoredMessage {
  id: string;
  sender_id: string;
  sender_name: string | null;
  body: string;
  sent_at: string;
  client_id: string | null;
}

interface MergedMessage {
  key: string;
  senderIdentity: string;
  senderName: string;
  body: string;
  sentMs: number;
  mine: boolean;
}

/**
 * Persistent chat sitting inside `<LiveKitRoom>`. Realtime delivery still
 * travels over LiveKit's data channel (via `useChat`); this component adds
 * two things on top:
 *
 *   • History load — students joining mid-session see what was said before
 *     they arrived, fetched from /api/sessions/[id]/messages on mount.
 *   • Post-session persistence — every message the local participant sends
 *     is POSTed to the same endpoint. Recipients don't POST — the sender
 *     side alone owns persistence, which keeps the DB free of duplicates
 *     without needing server-side dedupe beyond the unique index.
 *
 * The panel is a slide-in drawer from the left. We deliberately put it on
 * the opposite side from HostControlsPanel so hosts can have both open at
 * once without them stacking.
 */
export function SessionChatPanel({ sessionId }: { sessionId: string }) {
  const { lang } = useLang();
  const lc = COPY[lang];
  const { localParticipant } = useLocalParticipant();
  const remotes = useRemoteParticipants();
  const chat = useChat();

  const [open, setOpen] = useState(false);
  const [history, setHistory] = useState<StoredMessage[]>([]);
  const [input, setInput] = useState('');
  const [unread, setUnread] = useState(0);
  const [sendError, setSendError] = useState<string | null>(null);
  // Root ref lets the auto-scroll effect walk into base-ui ScrollArea's
  // internal viewport (data-slot="scroll-area-viewport") — the primitive
  // doesn't expose that node directly.
  const scrollRootRef = useRef<HTMLDivElement>(null);
  // Which LiveKit message ids we've already POSTed. Prevents a duplicate
  // POST if send() succeeds but we happen to also see the echo through
  // chat.chatMessages (which we do — send returns the same object).
  const persistedRef = useRef<Set<string>>(new Set());

  // History load once on mount. Best-effort — if this fails we still render
  // the realtime channel and log the error inline for the user.
  useEffect(() => {
    let cancelled = false;
    (async () => {
      try {
        const res = await fetch(`/api/sessions/${sessionId}/messages`, {
          headers: await getAuthHeaders(),
        });
        if (!res.ok) return;
        const data = (await res.json()) as { messages: StoredMessage[] };
        if (cancelled) return;
        setHistory(data.messages || []);
        // Seed persistedRef so we don't re-POST our own history-loaded
        // messages if useChat replays them.
        for (const m of data.messages || []) {
          if (m.client_id) persistedRef.current.add(m.client_id);
        }
      } catch {
        /* history is nice-to-have; realtime still works */
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [sessionId]);

  const nameByIdentity = useMemo(() => {
    const map = new Map<string, string>();
    if (localParticipant) {
      map.set(localParticipant.identity, localParticipant.name || lc.you);
    }
    for (const r of remotes) map.set(r.identity, r.name || r.identity);
    return map;
  }, [localParticipant, remotes, lc.you]);

  // Merge history + live realtime. Dedupe key priority: LiveKit ChatMessage
  // id (also used as client_id when we POST) → falls back to server-side
  // UUID if a message came from history but was never sent through LiveKit
  // in this session.
  const messages = useMemo<MergedMessage[]>(() => {
    const merged = new Map<string, MergedMessage>();
    const myIdentity = localParticipant?.identity;

    for (const m of history) {
      const key = m.client_id || m.id;
      // For history rows, sender_id is our users.id UUID which happens to be
      // the LiveKit identity we set in the token. So `mine` matches cleanly.
      const senderName = m.sender_name || nameByIdentity.get(m.sender_id) || lc.guest;
      merged.set(key, {
        key,
        senderIdentity: m.sender_id,
        senderName,
        body: m.body,
        sentMs: new Date(m.sent_at).getTime(),
        mine: m.sender_id === myIdentity,
      });
    }
    for (const lm of chat.chatMessages) {
      const senderIdentity = lm.from?.identity ?? '';
      const senderName = nameByIdentity.get(senderIdentity) || lm.from?.name || senderIdentity;
      merged.set(lm.id, {
        key: lm.id,
        senderIdentity,
        senderName,
        body: lm.message,
        sentMs: lm.timestamp,
        mine: senderIdentity === myIdentity,
      });
    }
    return Array.from(merged.values()).sort((a, b) => a.sentMs - b.sentMs);
  }, [history, chat.chatMessages, nameByIdentity, localParticipant, lc.guest]);

  // Auto-scroll: if the drawer is open, keep the newest message visible.
  // We deliberately don't do smooth-scroll — over a long history it looks
  // slow when someone opens the drawer for the first time. The actual
  // scrollable node lives inside base-ui's ScrollArea viewport.
  useEffect(() => {
    if (!open) return;
    const viewport = scrollRootRef.current?.querySelector<HTMLDivElement>(
      '[data-slot="scroll-area-viewport"]',
    );
    if (viewport) viewport.scrollTop = viewport.scrollHeight;
  }, [messages.length, open]);

  // Unread indicator: bump the counter when the drawer is closed and the
  // message list grows. We compare against a ref rather than storing the
  // count in state so a re-render never double-counts.
  const prevMsgCountRef = useRef(messages.length);
  useEffect(() => {
    const prev = prevMsgCountRef.current;
    const now = messages.length;
    prevMsgCountRef.current = now;
    // Only track growth while the drawer is closed. Reset on open happens in
    // the open button's click handler so we don't need a second effect just
    // to zero the counter.
    if (!open && now > prev) setUnread((n) => n + (now - prev));
  }, [messages.length, open]);

  const send = useCallback(
    async (text: string) => {
      const body = text.trim();
      if (!body || chat.isSending) return;
      setSendError(null);
      setInput('');
      try {
        const sent = await chat.send(body);
        if (!persistedRef.current.has(sent.id)) {
          persistedRef.current.add(sent.id);
          // Fire the persist POST after LiveKit succeeds so the in-room copy
          // was delivered even if the DB write is lost. The catch below
          // surfaces the loss with a toast — the recipient still saw the
          // message, but it won't show up in the post-session transcript.
          try {
            const authHeaders = await getAuthHeaders();
            const res = await fetch(`/api/sessions/${sessionId}/messages`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json', ...authHeaders },
              body: JSON.stringify({
                body,
                client_id: sent.id,
                sent_at: new Date(sent.timestamp).toISOString(),
              }),
            });
            if (!res.ok && res.status !== 200) {
              // 200 with {deduped:true} is a benign hit on the unique index —
              // treat only 4xx/5xx as an actual persistence failure.
              const detail = await res
                .text()
                .then((t) => t.slice(0, 200))
                .catch(() => `HTTP ${res.status}`);
              throw new Error(detail);
            }
          } catch (persistErr) {
            // Message reached LiveKit → other participants saw it. It won't
            // land in the transcript though, so warn the sender.
            toast.warning(lc.notSavedTitle, { description: lc.notSavedDesc });
            // Non-fatal — we intentionally do not rethrow.
            console.warn('chat persist failed', persistErr);
          }
        }
      } catch (err) {
        // LiveKit send() itself failed → nobody saw the message.
        const msg = (err as Error)?.message || lc.sendFallback;
        setSendError(msg);
        toast.error(lc.notSentTitle, { description: msg });
      }
    },
    [chat, sessionId, lc.notSavedTitle, lc.notSavedDesc, lc.notSentTitle, lc.sendFallback],
  );

  return (
    <>
      {/* Controlled trigger — see comment in HostControlsPanel for why we
          skip SheetTrigger in favor of a plain onClick. */}
      <Button
        variant="secondary"
        onClick={() => {
          setUnread(0);
          setOpen(true);
        }}
        className="fixed bottom-24 left-4 z-40 rounded-full bg-slate-900 text-white shadow-lg hover:bg-slate-800 sm:left-6 dark:bg-slate-100 dark:text-slate-900 dark:hover:bg-slate-200"
        aria-label={lc.ariaOpenChat(unread)}
      >
        <MessageCircle />
        <span className="hidden sm:inline">{lc.chatBtn}</span>
        {unread > 0 && (
          <span className="bg-destructive ml-1 rounded-full px-1.5 py-0.5 text-[10px] text-white tabular-nums">
            {unread > 9 ? '9+' : unread}
          </span>
        )}
      </Button>

      <Sheet open={open} onOpenChange={setOpen}>
        <SheetContent side="left" className="flex flex-col gap-0 p-0">
          <SheetHeader className="border-border shrink-0 flex-row items-center gap-3 border-b px-5 py-4">
            <MessageCircle className="text-brand size-4" aria-hidden />
            <div className="min-w-0 flex-1">
              <SheetTitle>{lc.title}</SheetTitle>
              <SheetDescription>{lc.messagesCount(messages.length)}</SheetDescription>
            </div>
          </SheetHeader>

          <ScrollArea ref={scrollRootRef} className="flex-1" aria-live="polite">
            <div className="space-y-2 px-4 py-4">
              {messages.length === 0 && (
                <p className="text-muted-foreground pt-16 text-center text-xs">{lc.empty}</p>
              )}
              {messages.map((m) => (
                <RoomMessageBubble
                  key={m.key}
                  body={m.body}
                  senderName={m.senderName}
                  mine={m.mine}
                  time={new Date(m.sentMs).toLocaleTimeString(undefined, {
                    hour: '2-digit',
                    minute: '2-digit',
                  })}
                />
              ))}
            </div>
          </ScrollArea>

          {sendError && (
            <Alert variant="destructive" className="mx-4 mb-2">
              <AlertDescription>{sendError}</AlertDescription>
            </Alert>
          )}

          <form
            className="border-border flex shrink-0 items-end gap-2 border-t px-4 py-3"
            onSubmit={(e) => {
              e.preventDefault();
              void send(input);
            }}
          >
            <Textarea
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                // Enter to send; Shift+Enter for a newline. Mirrors Zoom
                // and Slack — the convention users already have.
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault();
                  void send(input);
                }
              }}
              rows={1}
              maxLength={4000}
              placeholder={lc.placeholder}
              aria-label={lc.ariaMessage}
              className="min-h-11 flex-1 resize-none"
            />
            <Button
              type="submit"
              size="icon"
              disabled={!input.trim() || chat.isSending}
              aria-label={lc.ariaSend}
            >
              {chat.isSending ? <Loader2 className="animate-spin" /> : <Send />}
            </Button>
          </form>
        </SheetContent>
      </Sheet>
    </>
  );
}
