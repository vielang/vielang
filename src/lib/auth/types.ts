// Auth-related types shared between server (get-current-user.ts) and client
// (admin-loader). Lives in a separate file so 'use client' components can
// `import type { CurrentUser }` without pulling in the `server-only`
// module that owns the resolver.
export interface CurrentUser {
  /** Canonical app id — usually a stable id like "admin-1" or a Supabase UUID. */
  id: string;
  email: string;
  name: string;
  role: 'admin' | 'tutor' | 'user';
}
