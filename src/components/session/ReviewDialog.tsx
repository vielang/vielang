'use client';

import { useEffect, useState } from 'react';
import { Star } from 'lucide-react';
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
import { Textarea } from '@/components/ui/textarea';
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { useDialogForm } from '@/hooks/use-dialog-form';
import { reviewFormSchema } from '@/lib/schemas/forms';

interface Props {
  open: boolean;
  onClose: () => void;
  sessionId: string;
  tutorName: string;
  onSubmitted: () => void;
}

const COMMENT_LIMIT = 2000;

// One-word feedback tied to the picked rating. Feels warmer than a plain
// "5/5 stars" readout and gives the student a tiny reward for high scores.
const RATING_HINT: Record<1 | 2 | 3 | 4 | 5, string> = {
  1: 'Very disappointing',
  2: 'Not what I hoped',
  3: 'It was okay',
  4: 'I liked it',
  5: 'Loved it!',
};

export function ReviewDialog({ open, onClose, sessionId, tutorName, onSubmitted }: Props) {
  const [hover, setHover] = useState<1 | 2 | 3 | 4 | 5 | null>(null);

  const form = useDialogForm({
    schema: reviewFormSchema,
    defaultValues: { rating: 5, comment: '' },
    successTitle: 'Review posted',
    successDescription: 'Thanks for the feedback!',
    errorTitle: 'Could not post review',
    onSubmit: async (values) => {
      const res = await fetch('/api/reviews', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({
          session_id: sessionId,
          rating: values.rating,
          comment: values.comment?.trim() ? values.comment.trim() : null,
        }),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok) {
        if (data?.error === 'already_reviewed') {
          throw new Error("You've already reviewed this session.");
        }
        throw new Error(data?.error || 'submit_failed');
      }
      onSubmitted();
      onClose();
    },
  });

  // Reset form state whenever the dialog reopens so the next session's review
  // doesn't inherit the previous one's rating. `form` omitted from deps —
  // see BookingDialog for the reset-loop story.
  useEffect(() => {
    if (open) form.reset({ rating: 5, comment: '' });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [open]);

  const rating = form.watch('rating');
  const comment = form.watch('comment') ?? '';
  const shownRating = (hover ?? rating) as 1 | 2 | 3 | 4 | 5;

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v && !form.isSubmitting) onClose();
      }}
    >
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Rate your session with {tutorName}</DialogTitle>
          <DialogDescription>
            Your feedback helps other students find great tutors and helps {tutorName} improve.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.submit} className="space-y-5">
            <FormField
              control={form.control}
              name="rating"
              render={({ field }) => (
                <FormItem className="space-y-1.5">
                  <div
                    role="radiogroup"
                    aria-label="Rating"
                    className="flex items-center justify-center gap-1.5"
                    onMouseLeave={() => setHover(null)}
                  >
                    {[1, 2, 3, 4, 5].map((n) => {
                      const value = n as 1 | 2 | 3 | 4 | 5;
                      const active = (hover ?? rating) >= value;
                      return (
                        <button
                          key={value}
                          type="button"
                          role="radio"
                          aria-checked={rating === value}
                          aria-label={`${value} out of 5 stars`}
                          onMouseEnter={() => setHover(value)}
                          onFocus={() => setHover(value)}
                          onClick={() => field.onChange(value)}
                          disabled={form.isSubmitting}
                          className="focus-ring rounded-md p-1 transition-colors disabled:opacity-40"
                        >
                          <Star
                            className={`size-8 transition-colors ${
                              active
                                ? 'fill-amber-400 text-amber-400'
                                : 'text-slate-300 dark:text-slate-600'
                            }`}
                          />
                        </button>
                      );
                    })}
                  </div>
                  <p
                    className="text-center text-xs font-semibold text-slate-600 dark:text-slate-300"
                    aria-live="polite"
                  >
                    {RATING_HINT[shownRating]}
                  </p>
                  <FormMessage className="text-center" />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="comment"
              render={({ field }) => (
                <FormItem className="space-y-1.5">
                  <div className="flex items-baseline justify-between gap-2">
                    <label
                      htmlFor="review-comment"
                      className="text-xs font-semibold text-slate-500 dark:text-slate-400"
                    >
                      Add a note (optional)
                    </label>
                    <span className="text-[11px] text-slate-500 tabular-nums dark:text-slate-400">
                      {comment.length}/{COMMENT_LIMIT}
                    </span>
                  </div>
                  <FormControl>
                    <Textarea
                      id="review-comment"
                      value={field.value ?? ''}
                      onChange={field.onChange}
                      onBlur={field.onBlur}
                      placeholder="What went well? Anything the tutor could improve?"
                      rows={4}
                      maxLength={COMMENT_LIMIT}
                      disabled={form.isSubmitting}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button type="button" variant="ghost" onClick={onClose} disabled={form.isSubmitting}>
                Cancel
              </Button>
              <Button type="submit" disabled={form.isSubmitting}>
                {form.isSubmitting ? <Spinner /> : 'Post review'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
