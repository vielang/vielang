'use client';

import { useEffect, useMemo, useState } from 'react';
import Link from 'next/link';
import { useRouter } from 'next/navigation';
import { motion } from 'motion/react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { CheckCircle2, KeyRound, Lock, AlertTriangle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Spinner } from '@/components/ui/spinner';
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form';
import { useLang } from '@/contexts';
import { supabaseBrowser } from '@/lib/supabase/client';
import { errorMessage } from '@/lib/errors';

type Phase = 'awaiting-event' | 'ready' | 'no-session' | 'submitting' | 'done';

/**
 * Landing page for the password-reset email link.
 *
 * How Supabase's flow works:
 *   1. User submits their email on /login (forgot-password mode).
 *   2. Supabase emails them a link like /auth/reset-password?code=…
 *   3. When the page loads, @supabase/ssr detects the recovery hash and
 *      fires an `onAuthStateChange` event with `event === 'PASSWORD_RECOVERY'`.
 *      That event includes a short-lived session — we don't sign the user in
 *      persistently, we just get permission to call `updateUser({ password })`.
 *   4. User submits a new password. Success → redirect to /login.
 *
 * If the page loads WITHOUT the recovery event within ~2 s, we assume the
 * link is stale or the user hit the URL directly and show a helpful error.
 */
export default function ResetPasswordPage() {
  const router = useRouter();
  const { lang } = useLang();
  const [phase, setPhase] = useState<Phase>('awaiting-event');
  const [error, setError] = useState('');

  useEffect(() => {
    // Subscribe first so we never miss the event; then check the current
    // session as a fallback for browsers that fire the recovery before we
    // subscribe (rare, but possible in fast-cache reloads).
    const { data: sub } = supabaseBrowser.auth.onAuthStateChange((event) => {
      if (event === 'PASSWORD_RECOVERY') setPhase('ready');
    });

    (async () => {
      const { data } = await supabaseBrowser.auth.getSession();
      if (data.session) setPhase('ready');
    })();

    // If nothing came through within 2s the link is probably stale.
    const timeout = setTimeout(() => {
      setPhase((cur) => (cur === 'awaiting-event' ? 'no-session' : cur));
    }, 2_000);

    return () => {
      sub.subscription.unsubscribe();
      clearTimeout(timeout);
    };
  }, []);

  const copy = {
    VN: {
      title: 'Đặt mật khẩu mới',
      intro: 'Nhập mật khẩu mới cho tài khoản của bạn.',
      newPassword: 'Mật khẩu mới',
      confirmPassword: 'Xác nhận mật khẩu',
      submit: 'Cập nhật mật khẩu',
      submitting: 'Đang cập nhật…',
      done: 'Đã đổi mật khẩu',
      doneBody: 'Bạn có thể đăng nhập bằng mật khẩu mới.',
      backToLogin: 'Về trang đăng nhập',
      invalid: 'Link không hợp lệ hoặc đã hết hạn',
      invalidBody:
        'Vui lòng yêu cầu lại link đặt lại mật khẩu từ trang đăng nhập. Link chỉ có hiệu lực trong khoảng 1 giờ.',
      requestAgain: 'Gửi lại link',
      passwordMin: 'Mật khẩu tối thiểu 8 ký tự',
      passwordMismatch: 'Mật khẩu không khớp',
    },
    EN: {
      title: 'Set a new password',
      intro: 'Enter a new password for your account.',
      newPassword: 'New password',
      confirmPassword: 'Confirm password',
      submit: 'Update password',
      submitting: 'Updating…',
      done: 'Password updated',
      doneBody: 'You can now log in with your new password.',
      backToLogin: 'Back to login',
      invalid: 'Invalid or expired link',
      invalidBody:
        'Please request a new reset link from the login page. Links are valid for about one hour.',
      requestAgain: 'Request a new link',
      passwordMin: 'Password must be at least 8 characters',
      passwordMismatch: 'Passwords do not match',
    },
  }[lang];

  const schema = useMemo(
    () =>
      z
        .object({
          password: z.string().min(8, copy.passwordMin),
          confirm: z.string(),
        })
        .refine((v) => v.password === v.confirm, {
          path: ['confirm'],
          message: copy.passwordMismatch,
        }),
    [copy.passwordMin, copy.passwordMismatch],
  );

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: { password: '', confirm: '' },
  });

  const onSubmit = async (values: z.infer<typeof schema>) => {
    setError('');
    setPhase('submitting');
    try {
      const { error: updateErr } = await supabaseBrowser.auth.updateUser({
        password: values.password,
      });
      if (updateErr) throw updateErr;
      // Sign out so the recovery session doesn't accidentally auto-log the
      // user in without them re-entering credentials. Better UX + matches
      // most users' mental model of "reset means you need to log in again".
      await supabaseBrowser.auth.signOut();
      setPhase('done');
      setTimeout(() => router.push('/login'), 2000);
    } catch (err) {
      setError(errorMessage(err));
      setPhase('ready');
    }
  };

  return (
    <main className="flex min-h-dvh items-center justify-center bg-slate-50 px-4 py-10 dark:bg-slate-950">
      <motion.div
        initial={{ opacity: 0, y: 8 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ duration: 0.35 }}
        className="w-full max-w-md space-y-6 rounded-2xl border border-slate-200 bg-white p-8 shadow-sm dark:border-slate-800 dark:bg-slate-900"
      >
        <header className="space-y-2 text-center">
          <div className="bg-brand/10 text-brand mx-auto flex size-12 items-center justify-center rounded-2xl">
            <KeyRound className="size-6" />
          </div>
          <h1 className="text-brand dark:text-accent-warm font-serif text-2xl font-bold">
            {copy.title}
          </h1>
          <p className="text-sm text-slate-500 dark:text-slate-400">{copy.intro}</p>
        </header>

        {phase === 'awaiting-event' && (
          <div className="flex flex-col items-center gap-3 py-8 text-slate-500 dark:text-slate-400">
            <Spinner className="size-6" />
          </div>
        )}

        {phase === 'no-session' && (
          <div className="space-y-4">
            <div className="flex flex-col items-center gap-3 rounded-xl bg-red-50 p-5 text-center dark:bg-red-950/40">
              <AlertTriangle className="size-6 text-red-600 dark:text-red-400" />
              <div>
                <p className="text-sm font-semibold text-red-700 dark:text-red-300">
                  {copy.invalid}
                </p>
                <p className="mt-1 text-xs text-red-600/80 dark:text-red-400/80">
                  {copy.invalidBody}
                </p>
              </div>
            </div>
            <Link href="/login">
              <Button className="h-10 w-full rounded-md text-sm font-semibold">
                {copy.requestAgain}
              </Button>
            </Link>
          </div>
        )}

        {(phase === 'ready' || phase === 'submitting') && (
          <Form {...form}>
            <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
              {error && (
                <div className="rounded-lg bg-red-50 px-3 py-2 text-xs text-red-700 dark:bg-red-950/40 dark:text-red-300">
                  {error}
                </div>
              )}
              <FormField
                control={form.control}
                name="password"
                render={({ field }) => (
                  <FormItem>
                    <FormControl>
                      <div className="relative">
                        <Lock className="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-slate-400" />
                        <Input
                          type="password"
                          autoComplete="new-password"
                          placeholder={copy.newPassword}
                          className="h-10 rounded-md border-slate-200 bg-slate-50 pl-9 dark:border-slate-700 dark:bg-slate-800"
                          {...field}
                        />
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <FormField
                control={form.control}
                name="confirm"
                render={({ field }) => (
                  <FormItem>
                    <FormControl>
                      <div className="relative">
                        <Lock className="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-slate-400" />
                        <Input
                          type="password"
                          autoComplete="new-password"
                          placeholder={copy.confirmPassword}
                          className="h-10 rounded-md border-slate-200 bg-slate-50 pl-9 dark:border-slate-700 dark:bg-slate-800"
                          {...field}
                        />
                      </div>
                    </FormControl>
                    <FormMessage />
                  </FormItem>
                )}
              />
              <Button
                type="submit"
                disabled={phase === 'submitting'}
                className="h-10 w-full rounded-md text-sm font-semibold"
              >
                {phase === 'submitting' ? (
                  <>
                    <Spinner className="size-4" /> {copy.submitting}
                  </>
                ) : (
                  copy.submit
                )}
              </Button>
            </form>
          </Form>
        )}

        {phase === 'done' && (
          <div className="flex flex-col items-center gap-3 py-6">
            <div className="flex size-14 items-center justify-center rounded-full bg-emerald-50 dark:bg-emerald-950/40">
              <CheckCircle2 className="size-6 text-emerald-600 dark:text-emerald-400" />
            </div>
            <p className="text-sm font-semibold text-slate-800 dark:text-slate-100">{copy.done}</p>
            <p className="text-center text-xs text-slate-500 dark:text-slate-400">
              {copy.doneBody}
            </p>
            <Link href="/login">
              <Button variant="outline" size="sm" className="mt-2">
                {copy.backToLogin}
              </Button>
            </Link>
          </div>
        )}
      </motion.div>
    </main>
  );
}
