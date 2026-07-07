import { z } from 'zod';

export const loginSchema = z.object({
  email: z.string().min(1, 'required'),
  password: z.string().min(1, 'required'),
});
export type LoginInput = z.infer<typeof loginSchema>;

export const signupSchema = z
  .object({
    fullName: z.string().min(1, 'required'),
    email: z.string().email('invalidEmail'),
    password: z.string().min(6, 'passwordMin6'),
    confirmPassword: z.string().min(1, 'required'),
  })
  .refine((data) => data.password === data.confirmPassword, {
    message: 'passwordMismatch',
    path: ['confirmPassword'],
  });
export type SignupInput = z.infer<typeof signupSchema>;

export const forgotSchema = z.object({
  email: z.string().email('invalidEmail'),
});
export type ForgotInput = z.infer<typeof forgotSchema>;
