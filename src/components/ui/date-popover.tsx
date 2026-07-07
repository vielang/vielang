'use client';

import { useState, type ReactNode } from 'react';
import { Calendar as CalendarIcon } from 'lucide-react';
import { Calendar } from '@/components/ui/calendar';
import { Popover, PopoverContent, PopoverTrigger } from '@/components/ui/popover';
import { cn } from '@/lib/utils';

const isoFromDate = (d: Date) =>
  `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`;

const parseIso = (s?: string): Date | undefined => {
  if (!s) return undefined;
  const [y, m, d] = s.split('-').map(Number);
  if (!y || !m || !d) return undefined;
  return new Date(y, m - 1, d);
};

export type DatePopoverProps = {
  value: string;
  onChange: (iso: string) => void;
  locale?: 'KR' | 'VN' | 'EN';
  placeholder?: string;
  disablePast?: boolean;
  fromDate?: Date;
  toDate?: Date;
  className?: string;
  triggerClassName?: string;
  size?: 'sm' | 'md';
  prefix?: ReactNode;
  align?: 'start' | 'center' | 'end';
};

export function DatePopover({
  value,
  onChange,
  locale = 'EN',
  placeholder = 'Pick a date',
  disablePast,
  fromDate,
  toDate,
  triggerClassName,
  size = 'md',
  prefix,
  align = 'start',
}: DatePopoverProps) {
  const [open, setOpen] = useState(false);

  const localeTag = locale === 'KR' ? 'ko-KR' : locale === 'VN' ? 'vi-VN' : 'en-US';
  const selected = parseIso(value);
  const label = selected
    ? selected.toLocaleDateString(localeTag, {
        day: '2-digit',
        month: 'short',
        year: 'numeric',
        weekday: 'short',
      })
    : placeholder;

  const sizeCls = size === 'sm' ? 'h-9 text-xs' : 'h-10 text-sm';
  const padding = prefix ? 'pl-9 pr-3' : 'px-3';

  const disabled = [
    ...(disablePast ? [{ before: new Date(new Date().setHours(0, 0, 0, 0)) }] : []),
    ...(fromDate ? [{ before: fromDate }] : []),
    ...(toDate ? [{ after: toDate }] : []),
  ] as { before: Date }[] | { after: Date }[] | Array<{ before: Date } | { after: Date }>;

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        render={
          <button
            type="button"
            className={cn(
              'group text-brand dark:text-accent-warm focus-visible:ring-brand/30 dark:focus-visible:ring-accent-warm/40 focus-visible:border-brand/30 relative w-full rounded-lg border border-slate-200 bg-slate-50 text-left font-medium transition-colors outline-none hover:bg-slate-100 focus-visible:ring-2 dark:border-slate-700 dark:bg-slate-800 dark:hover:bg-slate-700',
              sizeCls,
              padding,
              !selected && 'font-normal text-slate-500 dark:text-slate-400',
              triggerClassName,
            )}
          />
        }
      >
        {prefix && (
          <span className="text-brand/70 pointer-events-none absolute top-1/2 left-3 -translate-y-1/2">
            {prefix}
          </span>
        )}
        <span className="block w-full truncate">{label}</span>
      </PopoverTrigger>
      <PopoverContent align={align} className="w-auto p-0">
        <Calendar
          mode="single"
          selected={selected}
          onSelect={(d) => {
            if (d) onChange(isoFromDate(d));
            setOpen(false);
          }}
          disabled={disabled.length > 0 ? disabled : undefined}
          initialFocus
        />
      </PopoverContent>
    </Popover>
  );
}

// Convenience: default-prefixed with a calendar icon
export function DatePopoverWithIcon(props: Omit<DatePopoverProps, 'prefix'>) {
  return <DatePopover {...props} prefix={<CalendarIcon className="size-4" />} />;
}
