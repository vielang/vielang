import { z } from 'zod';

// Wire shape for POST /api/reviews. Server derives student_id from the
// session; tutor_id from the session row too. Rating is 1-5 (matches the
// DB CHECK).
export const createReviewSchema = z.object({
  session_id: z.string().uuid(),
  rating: z.number().int().min(1).max(5),
  comment: z.string().trim().max(2000).nullable().optional(),
});
export type CreateReviewInput = z.infer<typeof createReviewSchema>;
