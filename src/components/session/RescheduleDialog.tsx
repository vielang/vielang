'use client';

import { useEffect, useMemo, useState } from 'react';
import { toast } from 'sonner';
import { CalendarDays, ArrowLeft } from 'lucide-react';
import { Spinner } from '@/components/ui/spinner';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { useDialogForm } from '@/hooks/use-dialog-form';
import { rescheduleFormSchema } from '@/lib/schemas/forms';

interface Slot {
  startAt: string;
  endAt: string;
  date: string;
  startLabel: string;
  durationMin: number;
}

interface Props {
  open: boolean;
  onClose: () => void;
  sessionId: string;
  tutorId: string;
  courseId: string | null;
  currentStartAt: string;
  onRescheduled: () => void;
}

type Step = 'date' | 'time' | 'confirm';

const STEPS: { key: Step; label: string; description: string }[] = [
  { key: 'date', label: 'New date', description: 'Pick a new day that works for you.' },
  { key: 'time', label: 'New time', description: 'Pick a new start time.' },
  { key: 'confirm', label: 'Confirm', description: 'Confirm the new slot.' },
];
const STEP_INDEX: Record<Step, number> = { date: 0, time: 1, confirm: 2 };

const formatDateLabel = (iso: string) => {
  const d = new Date(`${iso}T00:00:00+07:00`);
  return d.toLocaleDateString('en-US', { weekday: 'short', day: '2-digit', month: 'short' });
};

const formatCurrent = (iso: string) => {
  const d = new Date(iso);
  return d.toLocaleString(undefined, {
    weekday: 'short',
    day: '2-digit',
    month: 'short',
    hour: '2-digit',
    minute: '2-digit',
  });
};

/**
 * Reschedule dialog — mirrors the date/time steps of BookingDialog but skips
 * course pick (the session already has one) and calls the PATCH endpoint
 * instead of POST. The API blocks reschedule within 24 h of the current
 * start, so the caller decides whether to open us at all.
 */
export function RescheduleDialog({
  open,
  onClose,
  sessionId,
  tutorId,
  courseId,
  currentStartAt,
  onRescheduled,
}: Props) {
  const [step, setStep] = useState<Step>('date');
  const [slots, setSlots] = useState<Slot[] | null>(null);
  const [slotsLoading, setSlotsLoading] = useState(false);
  const [selectedDate, setSelectedDate] = useState<string | null>(null);
  const [selectedSlot, setSelectedSlot] = useState<Slot | null>(null);

  const form = useDialogForm({
    schema: rescheduleFormSchema,
    defaultValues: { slotStartAt: '' },
    successTitle: 'Session rescheduled',
    successDescription: 'The tutor was notified — status returns to pending until they re-confirm.',
    errorTitle: 'Could not reschedule',
    onSubmit: async (values) => {
      const res = await fetch(`/api/sessions/${sessionId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({
          action: 'reschedule',
          scheduled_at: values.slotStartAt,
        }),
      });
      const data = await res.json();
      if (!res.ok) {
        if (data?.error === 'slot_taken') {
          // Special-case: keep the dialog open, refresh slots, drop back to
          // the time step. Return 'handled' so the hook stays silent — we
          // surface a bespoke toast instead of the generic error copy.
          toast.error('That slot was just taken — please pick another.');
          setStep('time');
          setSlots(null);
          if (courseId) {
            const refresh = await fetch(
              `/api/tutors/${tutorId}/availability?courseId=${courseId}`,
              { headers: await getAuthHeaders() },
            );
            const refreshData = await refresh.json();
            setSlots(refreshData.slots || []);
          }
          return 'handled' as const;
        }
        if (data?.error === 'reschedule_window_closed') {
          onClose();
          throw new Error('You can only reschedule up to 24 hours before the session.');
        }
        throw new Error(data?.error || 'reschedule_failed');
      }
      onRescheduled();
      onClose();
    },
  });

  // Reset wizard state whenever the dialog opens/closes so a stale slot pick
  // from the previous open doesn't leak in. `form` omitted from deps — see
  // BookingDialog for the reset-loop story.
  useEffect(() => {
    if (!open) {
      setStep('date');
      setSlots(null);
      setSelectedDate(null);
      setSelectedSlot(null);
      form.reset({ slotStartAt: '' });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  // Fetch availability once dialog is open and we know the course.
  useEffect(() => {
    if (!open || !courseId) return;
    let cancelled = false;
    setSlotsLoading(true);
    (async () => {
      try {
        const res = await fetch(`/api/tutors/${tutorId}/availability?courseId=${courseId}`, {
          headers: await getAuthHeaders(),
        });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data = await res.json();
        if (!cancelled) setSlots(data.slots as Slot[]);
      } catch (err: unknown) {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : String(err);
          toast.error('Could not load availability', { description: message });
          setSlots([]);
        }
      } finally {
        if (!cancelled) setSlotsLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [open, tutorId, courseId]);

  const grouped = useMemo(() => {
    const map = new Map<string, Slot[]>();
    for (const s of slots || []) {
      const list = map.get(s.date) || [];
      list.push(s);
      map.set(s.date, list);
    }
    return map;
  }, [slots]);

  const availableDates = Array.from(grouped.keys());
  const slotsForDate = selectedDate ? grouped.get(selectedDate) || [] : [];
  const currentStepIx = STEP_INDEX[step];

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v && !form.isSubmitting) onClose();
      }}
    >
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Reschedule session</DialogTitle>
          <DialogDescription aria-live="polite">
            Step {currentStepIx + 1} of {STEPS.length} · {STEPS[currentStepIx].description}
          </DialogDescription>
        </DialogHeader>

        <div className="flex items-center gap-2 rounded-lg border border-slate-100 bg-slate-50 px-3 py-2 text-xs text-slate-600 dark:border-slate-800 dark:bg-slate-800/40 dark:text-slate-300">
          <CalendarDays className="size-3.5 shrink-0 text-slate-400" />
          Currently booked for{' '}
          <strong className="font-semibold">{formatCurrent(currentStartAt)}</strong>
        </div>

        <ol className="flex items-center gap-2 pb-2" aria-label="Reschedule progress">
          {STEPS.map((s, ix) => {
            const done = ix < currentStepIx;
            const current = ix === currentStepIx;
            return (
              <li key={s.key} className="flex-1">
                <div
                  aria-current={current ? 'step' : undefined}
                  className={`h-1 rounded-full transition-colors ${
                    done || current ? 'bg-brand' : 'bg-slate-200 dark:bg-slate-700'
                  }`}
                />
                <p
                  className={`mt-1.5 text-center text-[10px] font-semibold tracking-wider uppercase ${
                    current
                      ? 'text-brand'
                      : done
                        ? 'text-slate-500 dark:text-slate-400'
                        : 'text-slate-400 dark:text-slate-500'
                  }`}
                >
                  {s.label}
                </p>
              </li>
            );
          })}
        </ol>

        {step === 'date' && (
          <div className="max-h-80 space-y-3 overflow-y-auto">
            {slotsLoading ? (
              <div className="flex items-center justify-center py-10 text-slate-400">
                <Spinner className="size-5" />
              </div>
            ) : availableDates.length === 0 ? (
              <p className="text-sm text-slate-500">
                No availability in the next 14 days. Try a different tutor or wait for new slots.
              </p>
            ) : (
              // Radiogroup semantics — arrow keys navigate, Enter activates.
              // See BookingDialog for the same pattern.
              <ToggleGroup
                value={selectedDate ? [selectedDate] : []}
                onValueChange={(arr) => {
                  const v = arr[0];
                  if (!v) return;
                  setSelectedDate(v);
                  setStep('time');
                }}
                aria-label="Pick a new date"
                className="grid w-full grid-cols-3 gap-2"
              >
                {availableDates.map((d) => {
                  const count = grouped.get(d)?.length || 0;
                  return (
                    <ToggleGroupItem
                      key={d}
                      value={d}
                      aria-label={`${formatDateLabel(d)}, ${count} slot${count === 1 ? '' : 's'}`}
                      className="focus-ring hover:border-brand/40 data-[state=on]:border-brand h-auto min-w-0 rounded-lg border border-slate-200 bg-transparent p-3 text-center transition-colors data-[state=on]:bg-indigo-50 dark:border-slate-700 dark:data-[state=on]:bg-indigo-950/30"
                    >
                      <span className="block w-full">
                        <span className="block text-xs font-semibold text-slate-800 dark:text-slate-100">
                          {formatDateLabel(d)}
                        </span>
                        <span className="mt-1 block text-[10px] text-slate-500">
                          {count} slot{count === 1 ? '' : 's'}
                        </span>
                      </span>
                    </ToggleGroupItem>
                  );
                })}
              </ToggleGroup>
            )}
          </div>
        )}

        {step === 'time' && (
          <div className="max-h-80 space-y-3 overflow-y-auto">
            <p className="text-xs text-slate-500 dark:text-slate-400">
              {formatDateLabel(selectedDate!)}
            </p>
            <ToggleGroup
              value={selectedSlot ? [selectedSlot.startAt] : []}
              onValueChange={(arr) => {
                const v = arr[0];
                if (!v) return;
                const s = slotsForDate.find((slot) => slot.startAt === v);
                if (!s) return;
                setSelectedSlot(s);
                form.setValue('slotStartAt', s.startAt, { shouldValidate: true });
                setStep('confirm');
              }}
              aria-label="Pick a new start time"
              className="grid w-full grid-cols-3 gap-2 sm:grid-cols-4"
            >
              {slotsForDate.map((s) => (
                <ToggleGroupItem
                  key={s.startAt}
                  value={s.startAt}
                  aria-label={s.startLabel}
                  className="focus-ring hover:border-brand/40 data-[state=on]:border-brand data-[state=on]:text-brand h-11 min-w-0 rounded-lg border border-slate-200 bg-transparent text-xs font-semibold tabular-nums transition-colors data-[state=on]:bg-indigo-50 dark:border-slate-700 dark:data-[state=on]:bg-indigo-950/30"
                >
                  {s.startLabel}
                </ToggleGroupItem>
              ))}
            </ToggleGroup>
            <p className="pt-1 text-[11px] text-slate-500 dark:text-slate-400">
              Times shown in Vietnam local (GMT+7).
            </p>
          </div>
        )}

        {step === 'confirm' && selectedSlot && (
          <div className="space-y-3">
            <p className="text-sm text-slate-600 dark:text-slate-300">
              We'll move your session from <strong>{formatCurrent(currentStartAt)}</strong> to{' '}
              <strong>
                {formatDateLabel(selectedSlot.date)} at {selectedSlot.startLabel}
              </strong>
              . The status returns to pending until the tutor confirms the new time.
            </p>
          </div>
        )}

        <DialogFooter className="!justify-between gap-2 pt-2">
          {step !== 'date' ? (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              onClick={() => {
                const prev: Record<Step, Step> = {
                  date: 'date',
                  time: 'date',
                  confirm: 'time',
                };
                setStep(prev[step]);
              }}
              disabled={form.isSubmitting}
            >
              <ArrowLeft className="size-3.5" /> Back
            </Button>
          ) : (
            <span />
          )}

          {step === 'confirm' ? (
            <Button type="button" onClick={() => form.submit()} disabled={form.isSubmitting}>
              {form.isSubmitting ? <Spinner /> : 'Confirm reschedule'}
            </Button>
          ) : (
            <span />
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
