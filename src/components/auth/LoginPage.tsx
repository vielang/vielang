'use client';

import { useCallback, useEffect, useState } from 'react';
import { AnimatePresence } from 'motion/react';
import { ChevronDown, Globe, X } from 'lucide-react';

import { useAuth, useLang } from '@/contexts';
import type { AppUser } from '@/contexts/AuthContext';

import { Button } from '@/components/ui/button';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';

import { BrandingSide } from './BrandingSide';
import { LoginPanel } from './LoginPanel';
import { SignupPanel } from './SignupPanel';
import { ForgotPasswordPanel } from './ForgotPasswordPanel';

// LoginPage — shell around three interchangeable auth panels.
//
// Owns:
//   • `mode` (which panel is visible)
//   • shared `error` + `loading` state (each mode has one error alert)
//   • the `?disabled=1` capture that surfaces a persistent banner for
//     users bounced by AuthContext for ACCOUNT_DISABLED
//   • language switcher + optional close button (parent /login page passes
//     onBack)
//
// The panels themselves own their own zod schema + form + submit handler and
// call setError/setLoading on the shell.

interface LoginPageProps {
  onLogin?: (user: AppUser) => void;
  onBack?: () => void;
  lang?: 'VN' | 'EN';
  onLangChange?: (lang: 'VN' | 'EN') => void;
}

const LANGS: { code: 'VN' | 'EN'; flag: string; label: string }[] = [
  { code: 'VN', flag: '🇻🇳', label: 'Tiếng Việt' },
  { code: 'EN', flag: '🇺🇸', label: 'English' },
];

type Mode = 'login' | 'signup' | 'forgot';

export function LoginPage({
  onLogin: onLoginProp,
  onBack,
  lang: langProp,
  onLangChange: onLangChangeProp,
}: LoginPageProps) {
  const { login } = useAuth();
  const { lang: ctxLang, setLang: ctxSetLang } = useLang();
  const lang = langProp ?? ctxLang;
  const onLangChange = onLangChangeProp ?? ctxSetLang;
  // Preserve prop for callers still passing it (parent /login page). The
  // panels navigate via Supabase's own redirect + AuthContext hydration.
  void (onLoginProp ?? login);

  const [mode, setMode] = useState<Mode>('login');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  // Picked up when AuthContext bounced a disabled user to /login?disabled=1.
  // Persists until dismissed, even after switching between panels.
  const [disabledBanner, setDisabledBanner] = useState<string | null>(null);

  // Clear the transient error alert when the user switches panels — the error
  // context belongs to the panel they left, not the new one.
  useEffect(() => {
    setError('');
  }, [mode]);

  // Detect the `?disabled=1` flag set by AuthContext when /api/users/me
  // returned 403 ACCOUNT_DISABLED. Strip it from the URL after capture so a
  // page reload doesn't re-show the banner unrelated to the current attempt.
  useEffect(() => {
    if (typeof window === 'undefined') return;
    const params = new URLSearchParams(window.location.search);
    if (params.get('disabled') !== '1') return;
    const reason =
      params.get('reason') ||
      (lang === 'VN'
        ? 'Tài khoản của bạn đã bị vô hiệu hóa. Vui lòng liên hệ hỗ trợ.'
        : 'Your account has been disabled. Please contact support.');
    setDisabledBanner(reason);
    params.delete('disabled');
    params.delete('reason');
    const next = params.toString();
    window.history.replaceState({}, '', window.location.pathname + (next ? `?${next}` : ''));
  }, [lang]);

  const goLogin = useCallback(() => setMode('login'), []);
  const goSignup = useCallback(() => setMode('signup'), []);
  const goForgot = useCallback(() => setMode('forgot'), []);
  const dismissDisabled = useCallback(() => setDisabledBanner(null), []);

  return (
    <div className="flex min-h-screen items-center justify-center bg-slate-50 p-2 font-sans sm:p-4 dark:bg-slate-950">
      <div className="grid w-full max-w-6xl grid-cols-1 overflow-hidden rounded-xl border border-slate-200 bg-white shadow-lg lg:grid-cols-2 dark:border-slate-800 dark:bg-slate-900">
        <BrandingSide lang={lang} />

        <div className="space-y-5 p-5 sm:space-y-6 sm:p-8 md:p-10">
          {/* Top bar: language switcher + optional close */}
          <div className="flex items-center justify-between">
            <DropdownMenu>
              <DropdownMenuTrigger
                render={
                  <Button
                    variant="outline"
                    size="sm"
                    className="hover:border-primary/40 hover:text-primary h-9 gap-2 rounded-md bg-slate-50 dark:bg-slate-800"
                  />
                }
              >
                <Globe className="size-3.5" />
                <span className="text-base">{LANGS.find((l) => l.code === lang)?.flag}</span>
                <span className="hidden sm:inline">
                  {LANGS.find((l) => l.code === lang)?.label}
                </span>
                <ChevronDown className="size-3" />
              </DropdownMenuTrigger>
              <DropdownMenuContent align="end" className="w-40">
                {LANGS.map(({ code, flag, label }) => (
                  <DropdownMenuItem
                    key={code}
                    onClick={() => onLangChange(code)}
                    className={
                      lang === code
                        ? 'bg-primary text-primary-foreground focus:bg-primary focus:text-primary-foreground focus:**:text-primary-foreground'
                        : ''
                    }
                  >
                    <span className="mr-2 text-base">{flag}</span>
                    {label}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>

            {onBack && (
              <Button
                variant="ghost"
                size="icon"
                onClick={onBack}
                aria-label="Close"
                className="rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200"
              >
                <X className="size-5" />
              </Button>
            )}
          </div>

          <AnimatePresence mode="wait">
            {mode === 'login' && (
              <LoginPanel
                key="login"
                loading={loading}
                setLoading={setLoading}
                error={error}
                setError={setError}
                disabledBanner={disabledBanner}
                onDismissDisabled={dismissDisabled}
                onGoSignup={goSignup}
                onGoForgot={goForgot}
              />
            )}
            {mode === 'signup' && (
              <SignupPanel
                key="signup"
                loading={loading}
                setLoading={setLoading}
                error={error}
                setError={setError}
                onGoLogin={goLogin}
              />
            )}
            {mode === 'forgot' && (
              <ForgotPasswordPanel
                key="forgot"
                loading={loading}
                setLoading={setLoading}
                error={error}
                setError={setError}
                onGoLogin={goLogin}
              />
            )}
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
