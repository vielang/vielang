import { z } from 'zod';

// Wire-shape validators for POST /api/sessions and PATCH /api/sessions/[id].
// Keep these narrow — the server enriches server-controlled fields (price,
// duration, livekit_room_name) from the resolved course.

// Private 1-on-1 booking — a student picks a tutor + course + slot.
export const createPrivateSessionSchema = z.object({
  type: z.literal('private').optional(), // implicit default
  tutor_id: z.string().uuid(),
  course_id: z.string().uuid(),
  scheduled_at: z.string().datetime({ offset: true }),
  student_notes: z.string().trim().max(1000).optional().nullable(),
});
export type CreatePrivateSessionInput = z.infer<typeof createPrivateSessionSchema>;

// Admin creates a group free-talk session. host_tutor_id is optional — an
// admin can self-host. capacity 2–100 keeps rooms sane; LiveKit itself scales
// further but the UI isn't designed for it.
// `require_admission` turns on the waiting-room flow: students who join the
// LiveKit room start in a restricted state and can only publish/subscribe
// after a host clicks Admit in the room's host controls.
export const createGroupSessionSchema = z.object({
  type: z.literal('group'),
  host_tutor_id: z.string().uuid().optional().nullable(),
  topic_en: z.string().trim().min(3).max(120),
  topic_vn: z.string().trim().max(120).optional().nullable(),
  description: z.string().trim().max(1000).optional().nullable(),
  level: z.enum(['all', 'a1_a2', 'b1_b2', 'c1_c2']).default('all'),
  scheduled_at: z.string().datetime({ offset: true }),
  duration_min: z.number().int().min(15).max(180).default(45),
  capacity: z.number().int().min(2).max(100).default(20),
  cover_emoji: z.string().trim().max(4).optional().nullable(),
  require_admission: z.boolean().default(false),
});
export type CreateGroupSessionInput = z.infer<typeof createGroupSessionSchema>;

// Discriminated union for POST /api/sessions body. type='group' → admin only.
export const createSessionSchema = z.union([createPrivateSessionSchema, createGroupSessionSchema]);
export type CreateSessionInput = z.infer<typeof createSessionSchema>;

export const patchSessionSchema = z
  .object({
    action: z.enum(['cancel', 'confirm', 'reschedule']),
    scheduled_at: z.string().datetime({ offset: true }).optional(),
    cancelled_reason: z.string().trim().max(500).optional().nullable(),
    tutor_notes: z.string().trim().max(1000).optional().nullable(),
  })
  .refine((v) => v.action !== 'reschedule' || !!v.scheduled_at, {
    path: ['scheduled_at'],
    message: 'scheduled_at required for reschedule',
  });
export type PatchSessionInput = z.infer<typeof patchSessionSchema>;
