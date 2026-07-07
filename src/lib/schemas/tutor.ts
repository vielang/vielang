import { z } from 'zod';

// Availability slot input (POST). Times are 'HH:MM' or 'HH:MM:SS' local to
// the tutor's timezone (which is UTC+7 for the MVP — see lib/booking.ts).
const timeRe = /^([01]\d|2[0-3]):[0-5]\d(:[0-5]\d)?$/;

export const availabilitySchema = z
  .object({
    weekday: z.number().int().min(0).max(6),
    start_time: z.string().regex(timeRe, 'HH:MM(:SS) required'),
    end_time: z.string().regex(timeRe, 'HH:MM(:SS) required'),
  })
  .refine((v) => v.start_time < v.end_time, {
    path: ['end_time'],
    message: 'end_time must be greater than start_time',
  });
export type AvailabilityInput = z.infer<typeof availabilitySchema>;

// Bulk replace input for PUT /api/availability — every schedule the tutor
// wants to keep, sent as one atomic set. 200 slot cap is well above what
// any reasonable weekly schedule needs; keeps a rogue client from DOS'ing
// the endpoint with 10 000 rows.
export const availabilityReplaceSchema = z.object({
  slots: z.array(availabilitySchema).max(200),
});
export type AvailabilityReplaceInput = z.infer<typeof availabilityReplaceSchema>;

// Tutor self-service profile patch. Every field optional so the client can
// PATCH just what changed.
export const tutorProfilePatchSchema = z.object({
  name: z.string().trim().min(1).max(120).optional(),
  bio: z.string().trim().max(2000).nullable().optional(),
  avatar: z.string().url().nullable().optional(),
  hourly_rate_vnd: z.number().int().min(0).max(50_000_000).optional(),
  intro_video_url: z.string().url().nullable().optional(),
  specialties: z.array(z.string().trim().min(1).max(40)).max(12).optional(),
  years_experience: z.number().int().min(0).max(80).optional(),
  certifications: z.array(z.string().trim().min(1).max(80)).max(12).optional(),
  languages_spoken: z.array(z.string().trim().min(2).max(6)).max(12).optional(),
});
export type TutorProfilePatchInput = z.infer<typeof tutorProfilePatchSchema>;
