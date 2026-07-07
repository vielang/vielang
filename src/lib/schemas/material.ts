import { z } from 'zod';

export const createMaterialSchema = z.object({
  title: z.string().trim().min(1).max(200),
  type: z.enum(['pdf', 'video', 'link', 'image']),
  url: z.string().url(),
  order_index: z.number().int().min(0).max(9_999).optional(),
});
export type CreateMaterialInput = z.infer<typeof createMaterialSchema>;
