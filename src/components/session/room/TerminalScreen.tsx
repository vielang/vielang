import type { ReactNode } from 'react';
import { cn } from '@/lib/utils';

/**
 * Full-viewport terminal-state message used by every RoomClient phase that
 * pushes the user out of the video conference: error (token refused),
 * denied (host declined admission), ended (session finished), and other
 * "you're done, here's what to do next" screens.
 *
 * The pattern was hand-rolled three times in RoomClient with subtle drift;
 * consolidating here keeps the visual weight, spacing, and iconography in
 * lockstep no matter which reason bounced the user out.
 */
export function TerminalScreen({
  tone = 'neutral',
  icon,
  title,
  description,
  actions,
}: {
  /** Colours the icon halo. `error` reads as blocking, `warning` as advisory. */
  tone?: 'error' | 'warning' | 'success' | 'neutral';
  icon: ReactNode;
  title: string;
  description: ReactNode;
  actions?: ReactNode;
}) {
  const haloClass = {
    error: 'bg-destructive/10 text-destructive',
    warning: 'bg-amber-500/10 text-amber-600 dark:text-amber-400',
    success: 'bg-brand/10 text-brand',
    neutral: 'bg-muted text-muted-foreground',
  }[tone];

  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-4 px-6 py-16 text-center">
      <div
        className={cn('flex size-14 items-center justify-center rounded-full', haloClass)}
        aria-hidden
      >
        {icon}
      </div>
      <div className="max-w-sm space-y-1">
        <h1 className="font-heading text-lg font-bold text-slate-900 dark:text-slate-100">
          {title}
        </h1>
        <p className="text-muted-foreground text-sm">{description}</p>
      </div>
      {actions ? (
        <div className="flex flex-wrap items-center justify-center gap-3 pt-2">{actions}</div>
      ) : null}
    </div>
  );
}
