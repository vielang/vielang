// Client-side form schemas for the session dialogs.
//
// The wire schemas in review.ts / session.ts / material.ts include fields the
// server derives (session_id, tutor_id) or that come from the parent props
// (tutorId). What the user actually edits inside the dialog is a subset — these
// schemas describe that subset so react-hook-form validates only what the form
// controls.
//
// Each schema also owns the human-facing error messages that render inline via
// <FormMessage />; those don't belong in the wire schema (server messages
// have a different audience).

import { z } from 'zod';

// ---- Review dialog ----------------------------------------------------------
export const reviewFormSchema = z.object({
  rating: z.number().int().min(1, 'Please pick a rating').max(5),
  comment: z
    .string()
    .trim()
    .max(2000, 'Keep it under 2,000 characters')
    .optional()
    .or(z.literal('')),
});
export type ReviewFormValues = z.infer<typeof reviewFormSchema>;

// ---- Booking wizard ---------------------------------------------------------
// Wizard controls navigate via clicks; the final confirm step validates
// everything at once. `slotStartAt` is the picked slot's ISO timestamp.
export const bookingFormSchema = z.object({
  courseId: z.string().uuid('Pick a course to continue'),
  slotStartAt: z.string().datetime({ offset: true, message: 'Pick a time slot' }),
  notes: z
    .string()
    .trim()
    .max(1000, 'Notes are limited to 1,000 characters')
    .optional()
    .or(z.literal('')),
});
export type BookingFormValues = z.infer<typeof bookingFormSchema>;

// ---- Reschedule wizard ------------------------------------------------------
export const rescheduleFormSchema = z.object({
  slotStartAt: z.string().datetime({ offset: true, message: 'Pick a new time slot' }),
});
export type RescheduleFormValues = z.infer<typeof rescheduleFormSchema>;

// ---- Add material (tutor dashboard) -----------------------------------------
export const materialFormSchema = z.object({
  title: z.string().trim().min(1, 'Add a title').max(200, 'Keep the title under 200 characters'),
  type: z.enum(['pdf', 'video', 'link', 'image'], {
    message: 'Pick a material type',
  }),
  url: z.string().trim().min(1, 'Add a URL').url('Must be a valid URL (https://…)'),
});
export type MaterialFormValues = z.infer<typeof materialFormSchema>;
