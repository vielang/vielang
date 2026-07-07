'use client';

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';
import { ChevronDown, Loader2, RotateCcw, Save } from 'lucide-react';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { Button } from '@/components/ui/button';
import type { Availability } from '@/lib/types';
import { errorMessage } from '@/lib/errors';

// Weekday columns start on Monday for VN school-day mental model. Sunday
// lives at the end but keeps weekday=0 in the DB.
const WEEKDAY_COLUMNS: { weekday: number; label: string; short: string }[] = [
  { weekday: 1, label: 'Monday', short: 'Mon' },
  { weekday: 2, label: 'Tuesday', short: 'Tue' },
  { weekday: 3, label: 'Wednesday', short: 'Wed' },
  { weekday: 4, label: 'Thursday', short: 'Thu' },
  { weekday: 5, label: 'Friday', short: 'Fri' },
  { weekday: 6, label: 'Saturday', short: 'Sat' },
  { weekday: 0, label: 'Sunday', short: 'Sun' },
];

// 30-min steps between 06:00 and 22:00 gives 32 rows (last row 21:30–22:00).
const SLOT_MIN = 6 * 60;
const SLOT_MAX = 22 * 60;
const STEP_MIN = 30;
const SLOTS_PER_DAY = (SLOT_MAX - SLOT_MIN) / STEP_MIN;

const pad2 = (n: number) => n.toString().padStart(2, '0');

/** Return "HH:MM" for a given minutes-of-day value. */
const minutesToLabel = (m: number) => `${pad2(Math.floor(m / 60))}:${pad2(m % 60)}`;

/** Parse "HH:MM(:SS)" into minutes-of-day. */
function parseTime(t: string) {
  const [h = '0', m = '0'] = t.split(':');
  return Number(h) * 60 + Number(m);
}

/** Build the initial selection Set from persisted (weekday, start, end) rows. */
function rowsToSelection(rows: Availability[]): Set<string> {
  const set = new Set<string>();
  for (const r of rows) {
    const start = parseTime(r.start_time);
    const end = parseTime(r.end_time);
    for (let m = start; m < end; m += STEP_MIN) {
      set.add(`${r.weekday}-${m}`);
    }
  }
  return set;
}

/** Compress a selection Set back to (weekday, start, end) ranges — merging
 *  contiguous cells inside the same day so the DB gets minimal rows. */
function selectionToSlots(sel: Set<string>) {
  const byDay = new Map<number, number[]>();
  for (const cell of sel) {
    const [wd, m] = cell.split('-').map(Number);
    const list = byDay.get(wd) || [];
    list.push(m);
    byDay.set(wd, list);
  }
  const slots: { weekday: number; start_time: string; end_time: string }[] = [];
  for (const [wd, mins] of byDay) {
    mins.sort((a, b) => a - b);
    let runStart = mins[0];
    let prev = mins[0];
    for (let i = 1; i < mins.length; i += 1) {
      if (mins[i] === prev + STEP_MIN) {
        prev = mins[i];
        continue;
      }
      slots.push({
        weekday: wd,
        start_time: minutesToLabel(runStart),
        end_time: minutesToLabel(prev + STEP_MIN),
      });
      runStart = mins[i];
      prev = mins[i];
    }
    slots.push({
      weekday: wd,
      start_time: minutesToLabel(runStart),
      end_time: minutesToLabel(prev + STEP_MIN),
    });
  }
  return slots;
}

/** Stable string key for a set diff comparison. */
function selectionKey(sel: Set<string>) {
  return Array.from(sel).sort().join('|');
}

/** Count active slots for a given weekday. Uses linear scan of the Set — fine
 *  for <=224 total cells (7*32). */
function countDaySlots(sel: Set<string>, weekday: number) {
  const prefix = `${weekday}-`;
  let n = 0;
  for (const k of sel) if (k.startsWith(prefix)) n += 1;
  return n;
}

export function TutorAvailabilitySection({ initial }: { initial: Availability[] }) {
  const router = useRouter();
  const initialSelection = useMemo(() => rowsToSelection(initial), [initial]);
  const [selection, setSelection] = useState<Set<string>>(() => new Set(initialSelection));
  const [saving, setSaving] = useState(false);

  // Drag state (desktop grid only): whether we're currently sweeping cells,
  // and what "mode" the sweep is in — select (turn cells on) or deselect
  // (turn cells off). The mode is decided by the state of the first cell
  // touched. Mobile view uses simple tap-toggle so no drag state needed.
  const dragMode = useRef<'select' | 'deselect' | null>(null);

  // Global mouseup / blur stops the sweep even if the pointer leaves the grid.
  useEffect(() => {
    const stop = () => {
      dragMode.current = null;
    };
    window.addEventListener('mouseup', stop);
    window.addEventListener('blur', stop);
    return () => {
      window.removeEventListener('mouseup', stop);
      window.removeEventListener('blur', stop);
    };
  }, []);

  const dirty = selectionKey(selection) !== selectionKey(initialSelection);
  const totalSlots = selection.size;
  const totalHours = (totalSlots * STEP_MIN) / 60;

  const toggle = useCallback((key: string, force?: boolean) => {
    setSelection((prev) => {
      const next = new Set(prev);
      const currentlyOn = next.has(key);
      const shouldBeOn = force ?? !currentlyOn;
      if (shouldBeOn && !currentlyOn) next.add(key);
      else if (!shouldBeOn && currentlyOn) next.delete(key);
      else return prev;
      return next;
    });
  }, []);

  const handleMouseDown = (key: string, currentlyOn: boolean) => {
    dragMode.current = currentlyOn ? 'deselect' : 'select';
    toggle(key, !currentlyOn);
  };

  const handleMouseEnter = (key: string) => {
    if (!dragMode.current) return;
    toggle(key, dragMode.current === 'select');
  };

  const reset = () => setSelection(new Set(initialSelection));

  const save = async () => {
    setSaving(true);
    try {
      const slots = selectionToSlots(selection);
      const res = await fetch('/api/availability', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({ slots }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error || 'save_failed');
      toast.success('Schedule saved');
      router.refresh();
    } catch (err) {
      toast.error('Could not save schedule', { description: errorMessage(err) });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="type-page text-slate-900 dark:text-slate-100">Weekly availability</h1>
          <p className="mt-1 max-w-2xl text-sm text-slate-500 dark:text-slate-400">
            <span className="hidden md:inline">
              Click a cell to toggle it. Drag across cells to sweep several at once.
            </span>
            <span className="md:hidden">Tap a slot to toggle it on or off.</span> Each slot is a
            30-minute block. Save when you&apos;re happy — it replaces your whole schedule.
          </p>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          <Button variant="outline" size="sm" onClick={reset} disabled={!dirty || saving}>
            <RotateCcw className="size-3.5" />
            Reset
          </Button>
          <Button size="sm" onClick={save} disabled={!dirty || saving}>
            {saving ? <Loader2 className="size-3.5 animate-spin" /> : <Save className="size-3.5" />}
            Save schedule
          </Button>
        </div>
      </header>

      <div className="flex flex-wrap items-center gap-x-4 gap-y-1 text-xs text-slate-500 dark:text-slate-400">
        <span>
          <span className="font-semibold text-slate-700 tabular-nums dark:text-slate-200">
            {totalSlots}
          </span>{' '}
          slots
        </span>
        <span>
          <span className="font-semibold text-slate-700 tabular-nums dark:text-slate-200">
            {totalHours.toFixed(1)}
          </span>{' '}
          hours per week
        </span>
        <span>Times shown in Vietnam local (GMT+7)</span>
      </div>

      {/* Mobile view (<md): 7 collapsible days, each with a 4-col grid of
          h-11 (44px) toggles. Rendered in the DOM but visually hidden at md+;
          shares the same `selection` state as the desktop grid so both views
          stay in sync if the viewport is resized. */}
      <MobileAvailability selection={selection} toggle={toggle} />

      {/* Desktop view (md+): the original 7×32 drag-select grid. */}
      <div className="hidden overflow-x-auto rounded-2xl border border-slate-200 bg-white shadow-sm md:block dark:border-slate-800 dark:bg-slate-900">
        <div className="select-none">
          {/* Header row: day columns */}
          <div className="grid grid-cols-[48px_repeat(7,minmax(0,1fr))] border-b border-slate-100 bg-slate-50/60 dark:border-slate-800 dark:bg-slate-800/40">
            <div />
            {WEEKDAY_COLUMNS.map((d) => (
              <div key={d.weekday} className="type-eyebrow py-2 text-center">
                {d.short}
              </div>
            ))}
          </div>

          {/* One row per 30-min slot. Hour boundaries get an extra top border to
              act as visual guides. */}
          {Array.from({ length: SLOTS_PER_DAY }).map((_, i) => {
            const minute = SLOT_MIN + i * STEP_MIN;
            const isHourStart = minute % 60 === 0;
            return (
              <div
                key={minute}
                className={`grid grid-cols-[48px_repeat(7,minmax(0,1fr))] ${
                  isHourStart ? 'border-t border-slate-200 dark:border-slate-700' : ''
                }`}
              >
                <div className="flex items-center justify-end pr-2 text-[10px] font-semibold text-slate-400 tabular-nums dark:text-slate-500">
                  {isHourStart ? minutesToLabel(minute) : ''}
                </div>
                {WEEKDAY_COLUMNS.map((d) => {
                  const key = `${d.weekday}-${minute}`;
                  const on = selection.has(key);
                  return (
                    <button
                      key={key}
                      type="button"
                      onMouseDown={() => handleMouseDown(key, on)}
                      onMouseEnter={() => handleMouseEnter(key)}
                      onClick={(e) => {
                        // Keyboard activation — real mouse clicks were already
                        // handled by mouseDown. detail=0 signals keyboard.
                        if (e.detail !== 0) return;
                        toggle(key);
                      }}
                      aria-pressed={on}
                      aria-label={`${d.label} ${minutesToLabel(minute)}`}
                      className={`focus-ring h-7 border-l border-slate-100 transition-colors dark:border-slate-800 ${
                        on
                          ? 'bg-brand hover:bg-brand-hover'
                          : 'bg-white hover:bg-slate-100 dark:bg-slate-900 dark:hover:bg-slate-800'
                      }`}
                    />
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>

      <p className="max-w-2xl text-[11px] text-slate-500 dark:text-slate-400">
        The grid replaces your recurring weekly schedule. Existing sessions already booked against
        the old times aren&apos;t affected — students keep their reservations.
      </p>
    </div>
  );
}

/** Mobile disclosure list: one collapsible per weekday, with a 4-column grid
 *  of h-11 (44px) slot toggles inside. Multiple days can be open at once —
 *  tutors often edit similar hours across consecutive days. */
function MobileAvailability({
  selection,
  toggle,
}: {
  selection: Set<string>;
  toggle: (key: string, force?: boolean) => void;
}) {
  const [openDays, setOpenDays] = useState<Set<number>>(() => new Set());

  const toggleDay = (weekday: number) => {
    setOpenDays((prev) => {
      const next = new Set(prev);
      if (next.has(weekday)) next.delete(weekday);
      else next.add(weekday);
      return next;
    });
  };

  const clearDay = (weekday: number) => {
    for (let m = SLOT_MIN; m < SLOT_MAX; m += STEP_MIN) {
      toggle(`${weekday}-${m}`, false);
    }
  };

  return (
    <div className="space-y-2 md:hidden">
      {WEEKDAY_COLUMNS.map((d) => {
        const isOpen = openDays.has(d.weekday);
        const slots = countDaySlots(selection, d.weekday);
        const hours = (slots * STEP_MIN) / 60;
        return (
          <div
            key={d.weekday}
            className="overflow-hidden rounded-xl border border-slate-200 bg-white dark:border-slate-700 dark:bg-slate-900"
          >
            <button
              type="button"
              onClick={() => toggleDay(d.weekday)}
              aria-expanded={isOpen}
              aria-controls={`avail-day-${d.weekday}`}
              className="focus-ring flex w-full items-center gap-3 p-4 text-left"
            >
              <div className="min-w-0 flex-1">
                <p className="text-brand dark:text-accent-warm font-serif text-sm font-bold">
                  {d.label}
                </p>
                <p className="mt-0.5 text-[11px] text-slate-500 dark:text-slate-400">
                  {slots > 0 ? (
                    <>
                      <span className="font-semibold text-slate-700 tabular-nums dark:text-slate-200">
                        {slots}
                      </span>{' '}
                      slots · {hours.toFixed(1)}h
                    </>
                  ) : (
                    'No slots yet'
                  )}
                </p>
              </div>
              <ChevronDown
                className={`size-4 shrink-0 text-slate-400 transition-transform duration-200 ${
                  isOpen ? 'rotate-180' : ''
                }`}
                aria-hidden="true"
              />
            </button>
            {isOpen && (
              <div
                id={`avail-day-${d.weekday}`}
                className="border-t border-slate-100 p-4 dark:border-slate-800"
              >
                <div className="mb-3 flex items-center justify-between text-[11px]">
                  <span className="text-slate-500 dark:text-slate-400">
                    Tap a slot to toggle · 30-minute blocks
                  </span>
                  <button
                    type="button"
                    onClick={() => clearDay(d.weekday)}
                    disabled={slots === 0}
                    className="focus-ring rounded-md px-2 py-0.5 font-semibold text-slate-500 hover:text-slate-700 disabled:opacity-40 dark:text-slate-400 dark:hover:text-slate-200"
                  >
                    Clear day
                  </button>
                </div>
                <div className="grid grid-cols-4 gap-2">
                  {Array.from({ length: SLOTS_PER_DAY }).map((_, i) => {
                    const minute = SLOT_MIN + i * STEP_MIN;
                    const key = `${d.weekday}-${minute}`;
                    const on = selection.has(key);
                    return (
                      <button
                        key={key}
                        type="button"
                        onClick={() => toggle(key)}
                        aria-pressed={on}
                        aria-label={`${d.label} ${minutesToLabel(minute)}`}
                        className={`focus-ring h-11 rounded-lg border text-xs font-semibold tabular-nums transition-colors ${
                          on
                            ? 'bg-brand hover:bg-brand-hover border-brand text-white'
                            : 'border-slate-200 bg-white text-slate-700 hover:bg-slate-50 dark:border-slate-700 dark:bg-slate-800/60 dark:text-slate-200 dark:hover:bg-slate-800'
                        }`}
                      >
                        {minutesToLabel(minute)}
                      </button>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}
