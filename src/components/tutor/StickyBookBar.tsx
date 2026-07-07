'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { CalendarPlus } from 'lucide-react';
import { useAuth, useLang } from '@/contexts';
import { BookingDialog } from '@/components/session/BookingDialog';
import { cn } from '@/lib/utils';
import { formatVnd } from '@/lib/format';
import type { Course } from '@/lib/types';

interface Props {
  tutorId: string;
  tutorName: string;
  hourlyRateVnd: number;
  courses: Course[];
}

/**
 * Sticky bottom bar for tutor detail — mobile only. Always visible while the
 * user reads the profile, so `Book a session` is one thumb-tap away. Sits at
 * the very bottom of the viewport (BottomNav is hidden on this route so
 * there's no stacking conflict).
 *
 * The desktop version of this CTA lives in the sticky `<aside>` next to the
 * hero. This bar is `md:hidden` so it doesn't double-render there.
 */
export function StickyBookBar({ tutorId, tutorName, hourlyRateVnd, courses }: Props) {
  const router = useRouter();
  const { user } = useAuth();
  const { lang } = useLang();
  const [open, setOpen] = useState(false);

  const onClick = () => {
    if (!user) {
      router.push(`/login?redirect=/tutors/${tutorId}`);
      return;
    }
    setOpen(true);
  };

  return (
    <>
      <div
        className={cn(
          'pb-safe fixed inset-x-0 bottom-0 z-40 border-t border-slate-200 bg-white/95 px-4 pt-3 backdrop-blur-md md:hidden',
          'dark:border-slate-800 dark:bg-slate-950/95',
        )}
      >
        <div className="mx-auto flex max-w-lg items-center justify-between gap-3">
          <div className="min-w-0 flex-1">
            <p className="type-eyebrow">{lang === 'VN' ? 'Từ' : 'From'}</p>
            <p className="truncate text-base font-bold text-slate-900 tabular-nums dark:text-slate-100">
              {formatVnd(hourlyRateVnd)}
              <span className="ml-1 text-xs font-medium text-slate-500 dark:text-slate-400">
                ₫ / {lang === 'VN' ? 'giờ' : 'hr'}
              </span>
            </p>
          </div>
          <button
            type="button"
            onClick={onClick}
            className="bg-brand hover:bg-brand-hover focus-visible:ring-primary/40 inline-flex h-11 items-center gap-2 rounded-xl px-5 text-sm font-semibold text-white shadow transition-colors focus-visible:ring-2 focus-visible:outline-none"
          >
            <CalendarPlus className="size-4" />
            {lang === 'VN' ? 'Đặt lịch' : 'Book'}
          </button>
        </div>
      </div>
      <BookingDialog
        open={open}
        onClose={() => setOpen(false)}
        tutorId={tutorId}
        tutorName={tutorName}
        courses={courses}
      />
    </>
  );
}
