'use client';

import { useState, type ReactNode } from 'react';
import Link from 'next/link';
import { BrandMark } from './BrandMark';
import { useRouter, usePathname } from 'next/navigation';
import {
  ChevronDown,
  User,
  LogOut,
  Menu,
  GraduationCap,
  Newspaper,
  UserCircle,
  LayoutDashboard,
  MessagesSquare,
} from 'lucide-react';
import { useLang, useAuth } from '@/contexts';
import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DropdownMenuSeparator,
} from '@/components/ui/dropdown-menu';
import { Sheet, SheetContent, SheetTrigger, SheetTitle, SheetHeader } from '@/components/ui/sheet';
import { ThemeToggle } from './ThemeToggle';
import { OnlinePresence } from './OnlinePresence';

const languages = [
  { code: 'VN' as const, label: 'Tiếng Việt', flag: '🇻🇳' },
  { code: 'EN' as const, label: 'English', flag: '🇺🇸' },
];

type NavTab = { href: string; label: string; icon: ReactNode; match: (p: string) => boolean };

export function Header() {
  const router = useRouter();
  const pathname = usePathname() || '/';
  const { lang, setLang, t } = useLang();
  const { user, logout: handleLogout } = useAuth();
  const [isSheetOpen, setIsSheetOpen] = useState(false);

  // /courses route is currently a placeholder — hidden from nav until the
  // list view exists. Tutors + Sessions + News are the real customer entry
  // points; My-sessions lives under the user menu.
  const tabs: NavTab[] = [
    {
      href: '/tutors',
      label: t('tutors') || 'Tutors',
      icon: <GraduationCap className="size-4" />,
      match: (p) => p.startsWith('/tutors'),
    },
    {
      href: '/sessions',
      label: lang === 'VN' ? 'Luyện nói' : 'Sessions',
      icon: <MessagesSquare className="size-4" />,
      match: (p) => p.startsWith('/sessions'),
    },
    {
      href: '/news',
      label: t('news') || 'News',
      icon: <Newspaper className="size-4" />,
      match: (p) => p.startsWith('/news'),
    },
  ];

  const goMyPage = () => router.push('/my-page');
  const openMyPageOrLogin = () => {
    if (!user) {
      const skipReturn =
        pathname === '/login' || pathname === '/register' || pathname.startsWith('/auth');
      const target = skipReturn ? '/login' : `/login?redirect=${encodeURIComponent(pathname)}`;
      router.push(target);
      return;
    }
    goMyPage();
  };

  // Role-based dashboard shortcut. Tutor and Admin both get a link; Tutor
  // lands on /tutor, Admin on /admin.
  const dashboardHref =
    user?.role === 'admin' ? '/admin' : user?.role === 'tutor' ? '/tutor' : null;
  const dashboardLabel =
    user?.role === 'admin'
      ? lang === 'VN'
        ? 'Trang quản trị'
        : 'Admin Dashboard'
      : user?.role === 'tutor'
        ? lang === 'VN'
          ? 'Trang giáo viên'
          : 'Tutor Dashboard'
        : '';
  const showDashboardItem = !!dashboardHref && !pathname.startsWith(dashboardHref);
  const goDashboard = () => {
    if (dashboardHref) router.push(dashboardHref);
  };

  return (
    <header className="sticky top-0 z-50 border-b border-slate-100 bg-white/80 backdrop-blur-md dark:border-slate-800 dark:bg-slate-900/80">
      <a
        href="#main-content"
        className="focus:bg-primary focus:text-primary-foreground sr-only focus:not-sr-only focus:absolute focus:top-2 focus:left-2 focus:z-[60] focus:rounded-md focus:px-3 focus:py-2 focus:text-xs focus:font-semibold focus:shadow-lg"
      >
        {lang === 'VN' ? 'Đến nội dung chính' : 'Skip to main content'}
      </a>
      <div className="mx-auto flex h-14 max-w-7xl items-center justify-between gap-4 px-4 md:h-16">
        <Link
          href="/"
          onClick={() => setIsSheetOpen(false)}
          className="focus-visible:ring-primary/40 flex shrink-0 items-center gap-2 rounded-md focus-visible:ring-2 focus-visible:outline-none"
        >
          <BrandMark className="text-brand dark:text-accent-warm size-8 md:size-10" />
          <div className="text-left">
            <span className="text-base leading-none font-bold tracking-tight text-slate-900 md:text-lg dark:text-slate-100">
              Vie<span className="text-brand dark:text-accent-warm">Lang</span>
            </span>
            <p className="mt-0.5 hidden text-[10px] font-medium text-slate-500 sm:block md:text-[11px] dark:text-slate-400">
              {lang === 'VN' ? 'Học tiếng Anh 1-1' : 'Live English tutoring'}
            </p>
          </div>
        </Link>

        <nav className="hidden min-w-0 flex-1 items-center justify-center gap-0.5 md:flex lg:gap-1">
          {tabs.map((tab) => {
            const active = tab.match(pathname);
            return (
              <Link
                key={tab.href}
                href={tab.href}
                onClick={() => setIsSheetOpen(false)}
                className={`focus-visible:ring-primary/40 inline-flex h-9 items-center rounded-md px-2 text-xs font-semibold whitespace-nowrap transition-colors focus-visible:ring-2 focus-visible:outline-none lg:px-3 ${
                  active
                    ? 'bg-primary/10 text-primary'
                    : 'hover:text-brand dark:hover:text-accent-warm text-slate-500 hover:bg-slate-50 dark:text-slate-300 dark:hover:bg-slate-800'
                }`}
              >
                {tab.label}
              </Link>
            );
          })}
        </nav>

        {/* Presence pill — single instance, visible on every viewport.
            On desktop it sits between the nav and the controls cluster.
            On mobile the nav is hidden so justify-between centres it between
            the logo and the action buttons. */}
        <OnlinePresence />

        <div className="hidden shrink-0 items-center gap-2 md:flex">
          <DropdownMenu>
            <DropdownMenuTrigger
              render={
                <Button
                  variant="ghost"
                  className="text-brand dark:text-accent-warm h-9 gap-1.5 rounded-md bg-slate-50 px-3 text-xs font-medium hover:bg-slate-100 dark:bg-slate-800 dark:hover:bg-slate-700"
                />
              }
            >
              <span className="text-base">{languages.find((l) => l.code === lang)?.flag}</span>
              <span>{languages.find((l) => l.code === lang)?.label}</span>
              <ChevronDown className="size-3" />
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-40">
              {languages.map((l) => (
                <DropdownMenuItem
                  key={l.code}
                  onClick={() => setLang(l.code)}
                  className={
                    lang === l.code
                      ? 'bg-primary text-primary-foreground focus:bg-primary focus:text-primary-foreground focus:**:text-primary-foreground'
                      : ''
                  }
                >
                  <span className="mr-2 text-base">{l.flag}</span>
                  <span>{l.label}</span>
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>

          <div className="flex items-center gap-1.5">
            <ThemeToggle />
            {user ? (
              <DropdownMenu>
                <DropdownMenuTrigger
                  render={
                    <Button
                      className="h-9 gap-1.5 rounded-md px-3 text-xs font-semibold shadow-sm"
                      aria-label={user.name}
                    />
                  }
                >
                  <UserCircle className="size-4" />
                  <span className="max-w-[8rem] truncate">{user.name}</span>
                  <ChevronDown className="size-3" />
                </DropdownMenuTrigger>
                <DropdownMenuContent align="end" className="w-44">
                  {showDashboardItem && (
                    <DropdownMenuItem onClick={goDashboard}>
                      <LayoutDashboard className="size-4" />
                      <span>{dashboardLabel}</span>
                    </DropdownMenuItem>
                  )}
                  <DropdownMenuItem onClick={goMyPage}>
                    <User className="size-4" />
                    <span>{t('myPage') || 'My page'}</span>
                  </DropdownMenuItem>
                  <DropdownMenuSeparator />
                  <DropdownMenuItem
                    onClick={handleLogout}
                    className="text-red-600 focus:bg-red-50 focus:text-red-600 dark:text-red-400 dark:focus:bg-red-950/40 dark:focus:text-red-400"
                  >
                    <LogOut className="size-4" />
                    <span>{t('logout') || 'Logout'}</span>
                  </DropdownMenuItem>
                </DropdownMenuContent>
              </DropdownMenu>
            ) : (
              <Button
                onClick={openMyPageOrLogin}
                className="h-9 gap-1.5 rounded-md px-4 text-xs font-semibold shadow-sm"
              >
                <User className="size-3.5" />
                <span>{t('login')}</span>
              </Button>
            )}
          </div>
        </div>

        {/* Mobile */}
        <div className="flex items-center gap-1.5 md:hidden">
          {user ? (
            <DropdownMenu>
              <DropdownMenuTrigger
                render={
                  <Button
                    size="icon"
                    className="size-10 rounded-md shadow-sm"
                    aria-label={user.name}
                  />
                }
              >
                <User className="size-4" />
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-44">
                {showDashboardItem && (
                  <DropdownMenuItem onClick={goDashboard}>
                    <LayoutDashboard className="size-4" />
                    <span>{dashboardLabel}</span>
                  </DropdownMenuItem>
                )}
                <DropdownMenuItem onClick={goMyPage}>
                  <User className="size-4" />
                  <span>{t('myPage') || 'My page'}</span>
                </DropdownMenuItem>
                <DropdownMenuSeparator />
                <DropdownMenuItem
                  onClick={handleLogout}
                  className="text-red-600 focus:bg-red-50 focus:text-red-600 dark:text-red-400 dark:focus:bg-red-950/40 dark:focus:text-red-400"
                >
                  <LogOut className="size-4" />
                  <span>{t('logout') || 'Logout'}</span>
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
          ) : (
            <Button
              size="icon"
              onClick={openMyPageOrLogin}
              className="size-10 rounded-md shadow-sm"
              aria-label={t('login')}
            >
              <User className="size-4" />
            </Button>
          )}
          <Sheet open={isSheetOpen} onOpenChange={setIsSheetOpen}>
            <SheetTrigger
              render={
                <Button
                  variant="ghost"
                  size="icon"
                  className="text-brand dark:text-accent-warm size-10 rounded-md bg-slate-50 hover:bg-slate-100 dark:bg-slate-800 dark:hover:bg-slate-700"
                  aria-label="Open menu"
                />
              }
            >
              <Menu className="size-5" />
            </SheetTrigger>
            <SheetContent side="right" className="flex w-[88vw] max-w-xs flex-col p-0 sm:max-w-sm">
              <SheetHeader className="border-b border-slate-100 px-5 py-4 dark:border-slate-800">
                <SheetTitle className="text-brand dark:text-accent-warm text-base font-semibold">
                  {t('menu')}
                </SheetTitle>
              </SheetHeader>

              <div className="flex-1 space-y-5 overflow-y-auto px-5 py-4">
                <div className="space-y-1.5">
                  <p className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                    {t('language')}
                  </p>
                  <div className="grid grid-cols-3 gap-1.5">
                    {languages.map((l) => (
                      <Button
                        key={l.code}
                        variant={lang === l.code ? 'default' : 'secondary'}
                        size="sm"
                        onClick={() => setLang(l.code)}
                        className="h-9 gap-1.5 rounded-md px-0 text-xs"
                      >
                        <span className="text-sm leading-none">{l.flag}</span>
                        <span>{l.code}</span>
                      </Button>
                    ))}
                  </div>
                </div>

                <div className="space-y-1.5">
                  <p className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                    {lang === 'VN' ? 'Giao diện' : 'Theme'}
                  </p>
                  <div className="flex h-9 items-center justify-between rounded-md border border-slate-200 bg-slate-50 px-3 dark:border-slate-700 dark:bg-slate-800">
                    <span className="text-xs font-medium text-slate-600 dark:text-slate-300">
                      {lang === 'VN' ? 'Chế độ hiển thị' : 'Appearance'}
                    </span>
                    <ThemeToggle />
                  </div>
                </div>

                <div className="space-y-1.5 pt-1">
                  <p className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                    {t('menu')}
                  </p>
                  <div className="flex flex-col gap-1">
                    {tabs.map((tab) => {
                      const active = tab.match(pathname);
                      return (
                        <Link
                          key={tab.href}
                          href={tab.href}
                          onClick={() => setIsSheetOpen(false)}
                          className={`focus-visible:ring-primary/40 flex h-10 w-full items-center gap-3 rounded-md px-3 text-sm font-medium transition-colors focus-visible:ring-2 focus-visible:outline-none ${
                            active
                              ? 'bg-primary text-primary-foreground'
                              : 'text-slate-600 hover:bg-slate-100 dark:text-slate-300 dark:hover:bg-slate-800'
                          }`}
                        >
                          <span
                            className={
                              active
                                ? 'text-primary-foreground'
                                : 'text-slate-500 dark:text-slate-400'
                            }
                          >
                            {tab.icon}
                          </span>
                          <span className="flex-1 text-left">{tab.label}</span>
                          {active && (
                            <span className="bg-primary-foreground/80 size-1.5 rounded-full" />
                          )}
                        </Link>
                      );
                    })}
                  </div>
                </div>
              </div>
            </SheetContent>
          </Sheet>
        </div>
      </div>
    </header>
  );
}
