'use client';

import { useAuth } from '@/contexts';

// Quick-login row for the three seeded roles. Hidden in production.
// Bypasses Supabase entirely — reads/writes the vielang_demo_user cookie
// that middleware + server-side getCurrentUser both honor.
const DEMO_LOGIN_ENABLED =
  process.env.NODE_ENV !== 'production' || process.env.NEXT_PUBLIC_ALLOW_DEMO_USER === 'true';

interface Props {
  disabled: boolean;
  lang: 'VN' | 'EN';
}

export function DemoLoginRow({ disabled, lang }: Props) {
  const { loginAsDemo } = useAuth();
  if (!DEMO_LOGIN_ENABLED) return null;

  const items: { role: 'user' | 'tutor' | 'admin'; label: string; tone: string }[] = [
    {
      role: 'user',
      label: lang === 'VN' ? 'Học viên' : 'Student',
      tone: 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100',
    },
    {
      role: 'tutor',
      label: lang === 'VN' ? 'Giáo viên' : 'Tutor',
      tone: 'bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300 hover:bg-sky-100',
    },
    {
      role: 'admin',
      label: 'Admin',
      tone: 'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300 hover:bg-amber-100',
    },
  ];

  return (
    <div className="space-y-2">
      <p className="text-center text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
        {lang === 'VN' ? 'Đăng nhập demo (dev)' : 'Demo login (dev)'}
      </p>
      <div className="grid grid-cols-3 gap-2">
        {items.map((it) => (
          <button
            key={it.role}
            type="button"
            disabled={disabled}
            onClick={() => loginAsDemo(it.role)}
            className={`h-9 rounded-md border border-slate-200 text-xs font-semibold transition-colors disabled:opacity-50 dark:border-slate-700 ${it.tone}`}
          >
            {it.label}
          </button>
        ))}
      </div>
    </div>
  );
}
