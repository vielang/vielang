'use client';

import { useEffect, useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';
import { CalendarDays, Clock, GraduationCap, ArrowRight, ArrowLeft } from 'lucide-react';
import { Spinner } from '@/components/ui/spinner';
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { ToggleGroup, ToggleGroupItem } from '@/components/ui/toggle-group';
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { useDialogForm } from '@/hooks/use-dialog-form';
import { bookingFormSchema } from '@/lib/schemas/forms';
import { formatVnd } from '@/lib/format';
import type { Course } from '@/lib/types';

type Step = 'course' | 'date' | 'time' | 'confirm';

// Small vocabulary + labels for the top-of-dialog step indicator. Kept in one
// place so the label + description below always match.
const STEPS: { key: Step; label: string; description: string }[] = [
  { key: 'course', label: 'Course', description: 'Pick a course to start with.' },
  { key: 'date', label: 'Date', description: 'Choose a day that works for you.' },
  { key: 'time', label: 'Time', description: 'Pick a start time.' },
  { key: 'confirm', label: 'Confirm', description: 'Review the details and add optional notes.' },
];
const STEP_INDEX: Record<Step, number> = { course: 0, date: 1, time: 2, confirm: 3 };
const NOTES_LIMIT = 1000;

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
  tutorId: string;
  tutorName: string;
  courses: Course[];
}

const formatDateLabel = (iso: string) => {
  const d = new Date(`${iso}T00:00:00+07:00`);
  return d.toLocaleDateString('en-US', { weekday: 'short', day: '2-digit', month: 'short' });
};

export function BookingDialog({ open, onClose, tutorId, tutorName, courses }: Props) {
  const router = useRouter();
  const [step, setStep] = useState<Step>('course');
  const [slots, setSlots] = useState<Slot[] | null>(null);
  const [slotsLoading, setSlotsLoading] = useState(false);
  const [selectedDate, setSelectedDate] = useState<string | null>(null);
  const [selectedSlot, setSelectedSlot] = useState<Slot | null>(null);

  const form = useDialogForm({
    schema: bookingFormSchema,
    defaultValues: { courseId: '', slotStartAt: '', notes: '' },
    successTitle: 'Session booked',
    successDescription: 'You will find it under My sessions.',
    errorTitle: 'Could not create booking',
    onSubmit: async (values) => {
      const res = await fetch('/api/sessions', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({
          tutor_id: tutorId,
          course_id: values.courseId,
          scheduled_at: values.slotStartAt,
          student_notes: values.notes?.trim() ? values.notes.trim() : null,
        }),
      });
      const data = await res.json();
      if (!res.ok) {
        if (data?.error === 'slot_taken') {
          toast.error('That slot was just taken — please pick another.');
          setStep('time');
          setSlots(null);
          const refresh = await fetch(
            `/api/tutors/${tutorId}/availability?courseId=${values.courseId}`,
            { headers: await getAuthHeaders() },
          );
          const refreshData = await refresh.json();
          setSlots(refreshData.slots || []);
          return 'handled' as const;
        }
        throw new Error(data?.error || 'booking_failed');
      }
      onClose();
      router.push(`/my-sessions?highlight=${data.session.id}`);
    },
  });

  const courseId = form.watch('courseId');
  const notes = form.watch('notes') ?? '';
  const course = courses.find((c) => c.id === courseId);
  const currentStepIx = STEP_INDEX[step];
  const totalSteps = STEPS.length;

  // Reset wizard + form state when the dialog closes so the next open starts
  // fresh. `form` intentionally omitted from deps: useDialogForm returns a
  // fresh object literal each render (spread + submit + isSubmitting), so
  // including it re-fires this effect every render, `form.reset` re-triggers
  // RHF state, and the whole thing loops out. The reset methods themselves
  // are stable via closure, so this is safe.
  useEffect(() => {
    if (!open) {
      setStep('course');
      setSlots(null);
      setSelectedDate(null);
      setSelectedSlot(null);
      form.reset({ courseId: '', slotStartAt: '', notes: '' });
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  // Fetch availability once a course is picked. Refetches when tutor/course
  // changes; result cached until the user closes the dialog.
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

  // Group slots by date so the date step and time step share the computation.
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

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v) onClose();
      }}
    >
      <DialogContent className="sm:max-w-lg">
        <DialogHeader>
          <DialogTitle>Book a session with {tutorName}</DialogTitle>
          <DialogDescription aria-live="polite">
            Step {currentStepIx + 1} of {totalSteps} · {STEPS[currentStepIx].description}
          </DialogDescription>
        </DialogHeader>

        {/* Step indicator — a horizontal progress rail. `aria-current="step"`
            on the active bar makes it discoverable via screen reader. */}
        <ol className="flex items-center gap-2 pb-2" aria-label="Booking progress">
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

        <Form {...form}>
          <form onSubmit={form.submit}>
            {/* Step content area — the min-height keeps the sheet stable while
                we swap steps. */}
            {step === 'course' && (
              <div className="min-h-[280px] space-y-2 overflow-y-auto sm:max-h-80">
                {courses.length === 0 ? (
                  <p className="text-sm text-slate-500">This tutor has no published courses yet.</p>
                ) : (
                  courses.map((c) => (
                    <button
                      key={c.id}
                      type="button"
                      onClick={() => {
                        form.setValue('courseId', c.id, { shouldValidate: true });
                        setStep('date');
                      }}
                      className={`focus-ring w-full rounded-xl border p-4 text-left transition-colors ${
                        courseId === c.id
                          ? 'border-brand bg-indigo-50 dark:bg-indigo-950/30'
                          : 'hover:border-brand/40 border-slate-200 dark:border-slate-700'
                      }`}
                    >
                      <div className="flex items-start justify-between gap-3">
                        <div className="min-w-0">
                          <h4 className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
                            {c.title_en}
                          </h4>
                          <p className="mt-0.5 line-clamp-1 text-xs text-slate-500 dark:text-slate-400">
                            {c.description_en}
                          </p>
                          <div className="mt-1 flex items-center gap-3 text-[11px] text-slate-500 dark:text-slate-400">
                            <span className="flex items-center gap-1">
                              <GraduationCap className="size-3" />
                              {c.level} · {c.category}
                            </span>
                            <span className="flex items-center gap-1">
                              <Clock className="size-3" />
                              {c.duration_min}m
                            </span>
                          </div>
                        </div>
                        <span className="shrink-0 text-sm font-bold text-slate-900 tabular-nums dark:text-slate-100">
                          {formatVnd(c.price_vnd)}₫
                        </span>
                      </div>
                    </button>
                  ))
                )}
              </div>
            )}

            {step === 'date' && (
              <div className="min-h-[280px] space-y-3 overflow-y-auto sm:max-h-80">
                {slotsLoading ? (
                  <div className="flex items-center justify-center py-10 text-slate-400">
                    <Spinner className="size-5" />
                  </div>
                ) : availableDates.length === 0 ? (
                  <p className="text-sm text-slate-500">
                    No availability in the next 14 days. Pick a different course or check back
                    later.
                  </p>
                ) : (
                  // Radiogroup semantics via ToggleGroup — arrow keys roll
                  // through dates, Enter/Space activates. Value=selected date
                  // so the picked chip stays visible for the split second
                  // before we advance to the time step.
                  <ToggleGroup
                    value={selectedDate ? [selectedDate] : []}
                    onValueChange={(arr) => {
                      const v = arr[0];
                      if (!v) return;
                      setSelectedDate(v);
                      setStep('time');
                    }}
                    aria-label="Select a date"
                    className="xs:grid-cols-3 grid w-full grid-cols-2 gap-2"
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
              <div className="min-h-[280px] space-y-3 overflow-y-auto sm:max-h-80">
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
                  aria-label="Select a start time"
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

            {step === 'confirm' && selectedSlot && course && (
              <div className="min-h-[280px] space-y-4 overflow-y-auto sm:min-h-0">
                <dl className="divide-y divide-slate-100 rounded-xl border border-slate-200 dark:divide-slate-800 dark:border-slate-800">
                  <Row
                    icon={<GraduationCap className="size-3.5" />}
                    label="Course"
                    value={course.title_en}
                  />
                  <Row
                    icon={<CalendarDays className="size-3.5" />}
                    label="Date"
                    value={formatDateLabel(selectedSlot.date)}
                  />
                  <Row
                    icon={<Clock className="size-3.5" />}
                    label="Time"
                    value={`${selectedSlot.startLabel} · ${selectedSlot.durationMin} min`}
                  />
                  <Row label="Price" value={`${formatVnd(course.price_vnd)}₫`} />
                </dl>
                <FormField
                  control={form.control}
                  name="notes"
                  render={({ field }) => (
                    <FormItem className="space-y-1.5">
                      <div className="flex items-baseline justify-between gap-2">
                        <label
                          htmlFor="booking-notes"
                          className="text-xs font-semibold text-slate-500 dark:text-slate-400"
                        >
                          Notes for the tutor (optional)
                        </label>
                        <span className="text-[11px] text-slate-500 tabular-nums dark:text-slate-400">
                          {notes.length}/{NOTES_LIMIT}
                        </span>
                      </div>
                      <FormControl>
                        <Textarea
                          id="booking-notes"
                          value={field.value ?? ''}
                          onChange={field.onChange}
                          onBlur={field.onBlur}
                          placeholder="What would you like to focus on this session?"
                          maxLength={NOTES_LIMIT}
                          rows={3}
                          className="text-sm"
                          disabled={form.isSubmitting}
                        />
                      </FormControl>
                      <FormMessage />
                    </FormItem>
                  )}
                />
              </div>
            )}

            <DialogFooter className="!justify-between gap-2 pt-2">
              {step !== 'course' ? (
                <Button
                  type="button"
                  variant="ghost"
                  size="sm"
                  onClick={() => {
                    const prev: Record<Step, Step> = {
                      course: 'course',
                      date: 'course',
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
                <Button type="submit" disabled={form.isSubmitting}>
                  {form.isSubmitting ? (
                    <Spinner />
                  ) : (
                    <>
                      Confirm booking <ArrowRight className="size-3.5" />
                    </>
                  )}
                </Button>
              ) : (
                <span />
              )}
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}

function Row({ icon, label, value }: { icon?: React.ReactNode; label: string; value: string }) {
  return (
    <div className="flex items-center justify-between px-4 py-3 text-sm">
      <span className="type-eyebrow flex items-center gap-2">
        {icon}
        {label}
      </span>
      <span className="text-right font-medium text-slate-800 dark:text-slate-100">{value}</span>
    </div>
  );
}
