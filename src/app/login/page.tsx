'use client';

import dynamic from 'next/dynamic';
import { useRouter } from 'next/navigation';
import { useLang, useAuth } from '@/contexts';

const LoginPage = dynamic(
  () => import('@/components/auth/LoginPage').then((m) => ({ default: m.LoginPage })),
  { ssr: false },
);

export default function Page() {
  const router = useRouter();
  const { lang, setLang } = useLang();
  const { login: handleLogin } = useAuth();

  return (
    <LoginPage
      onLogin={handleLogin}
      onBack={() => router.push('/')}
      lang={lang}
      onLangChange={setLang}
    />
  );
}
