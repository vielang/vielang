import type { Metadata } from 'next';
import { redirect } from 'next/navigation';
import { getCurrentUser } from '@/lib/auth/get-current-user';

export const metadata: Metadata = {
  title: 'My page · VieLang',
  robots: { index: false, follow: false },
};

// The dedicated MyPage lands in a later phase (profile + reviews left). For
// now, /my-page just funnels students to their sessions list, which is what
// the header dropdown expects to open.
export default async function MyPagePage() {
  const user = await getCurrentUser();
  if (!user) redirect('/login?redirect=/my-page');
  redirect('/my-sessions');
}
