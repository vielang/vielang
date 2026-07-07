'use client';

import Link from 'next/link';
import { usePathname } from 'next/navigation';
import { Home, GraduationCap, CalendarCheck, User } from 'lucide-react';
import { useAuth, useLang } from '@/contexts';
import { cn } from '@/lib/utils';

/**
 * Mobile bottom navigation bar for authenticated users.
 *
 * Hidden on md+ (the desktop Header handles all nav there) and hidden entirely
 * when no user is signed in (anonymous visitors get the header hamburger only).
 *
 * 4 tabs is the sweet spot — 5 crowds the thumb zone on iPhone SE, 3 wastes
 * space. Home / Tutors / My-Sessions / My-Page covers the full student flow;
 * tutors + admins have their own dashboards linked from the profile menu.
 */
// Routes where the BottomNav gets in the way of a task-focused surface
// (tutor detail has its own sticky Book CTA; the video room is fullscreen).
const HIDE_ON_ROUTES = [/^\/tutors\/[^/]+\/?$/, /^\/session\/[^/]+\/room\/?$/];

export function BottomNav() {
  const pathname = usePathname() || '/';
  const { user } = useAuth();
  const { lang } = useLang();

  if (!user) return null;
  if (HIDE_ON_ROUTES.some((re) => re.test(pathname))) return null;

  const tabs = [
    {
      href: '/',
      label: lang === 'VN' ? 'Trang chủ' : 'Home',
      icon: Home,
      match: (p: string) => p === '/',
    },
    {
      href: '/tutors',
      label: lang === 'VN' ? 'Giáo viên' : 'Tutors',
      icon: GraduationCap,
      match: (p: string) => p.startsWith('/tutors'),
    },
    {
      href: '/my-sessions',
      label: lang === 'VN' ? 'Buổi học' : 'Sessions',
      icon: CalendarCheck,
      match: (p: string) => p.startsWith('/my-sessions'),
    },
    {
      href: '/my-page',
      label: lang === 'VN' ? 'Tôi' : 'Me',
      icon: User,
      match: (p: string) => p.startsWith('/my-page'),
    },
  ];

  return (
    <nav
      aria-label="Primary"
      className="pb-safe fixed inset-x-0 bottom-0 z-40 border-t border-slate-200 bg-white/95 backdrop-blur-md md:hidden dark:border-slate-800 dark:bg-slate-950/95"
    >
      <ul className="mx-auto grid max-w-lg grid-cols-4">
        {tabs.map((tab) => {
          const active = tab.match(pathname);
          const Icon = tab.icon;
          return (
            <li key={tab.href}>
              <Link
                href={tab.href}
                aria-current={active ? 'page' : undefined}
                // tap-min guarantees 44px hit area even if visual chrome shrinks.
                // Flex-col centers the icon over the label; gap tuned so the
                // label sits close but doesn't touch the icon on any DPR.
                className={cn(
                  'tap-min focus-visible:ring-primary/40 flex flex-col items-center justify-center gap-0.5 px-2 py-1.5 text-[11px] font-medium transition-colors focus-visible:ring-2 focus-visible:outline-none',
                  active
                    ? 'text-primary'
                    : 'text-slate-500 hover:text-slate-800 dark:text-slate-400 dark:hover:text-slate-100',
                )}
              >
                <Icon className={cn('size-5', active && 'fill-primary/10')} aria-hidden="true" />
                <span className="truncate leading-none">{tab.label}</span>
              </Link>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}
