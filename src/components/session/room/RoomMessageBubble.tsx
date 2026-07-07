import { cn } from '@/lib/utils';

/**
 * Chat message bubble used by both the in-room `SessionChatPanel` (live)
 * and the post-session `SessionDetailClient` transcript. Kept small on
 * purpose — the two callers control layout + auto-scroll around it.
 *
 * Design contract: own messages hug the right edge with the brand tint;
 * everyone else hugs the left in muted slate. `senderName` is only
 * rendered for other-side bubbles because seeing "You:" on your own
 * message adds noise without information.
 */
export function RoomMessageBubble({
  body,
  time,
  senderName,
  mine,
}: {
  body: string;
  time: string;
  senderName?: string | null;
  mine: boolean;
}) {
  return (
    <div className={cn('flex', mine ? 'justify-end' : 'justify-start')}>
      <div
        className={cn(
          // `[overflow-wrap:anywhere]` handles long URLs/tokens that
          // `break-words` alone can't (URLs without spaces overflow the
          // 80% max-width on ≤320px phones). max-w widens on very narrow
          // viewports so a bubble doesn't shrink to a hard-to-read column.
          'max-w-[90%] rounded-2xl px-3 py-1.5 text-sm [overflow-wrap:anywhere] whitespace-pre-wrap sm:max-w-[80%]',
          mine
            ? 'bg-brand/90 rounded-tr-md text-white'
            : 'rounded-tl-md bg-slate-100 text-slate-900 dark:bg-slate-800 dark:text-slate-100',
        )}
      >
        {!mine && senderName ? (
          <p className="type-eyebrow mb-0.5" aria-hidden>
            {senderName}
          </p>
        ) : null}
        <p>{body}</p>
        <p
          className={cn(
            'mt-0.5 text-right text-[10px]',
            mine ? 'text-white/70' : 'text-slate-500 dark:text-slate-400',
          )}
        >
          {!mine && senderName ? <span className="sr-only">from {senderName}, </span> : null}
          <span className="sr-only">sent at </span>
          {time}
        </p>
      </div>
    </div>
  );
}
