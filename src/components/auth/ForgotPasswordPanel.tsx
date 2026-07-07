'use client';

import { useEffect, useMemo, useState } from 'react';
import { motion } from 'motion/react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { ArrowLeft, Mail } from 'lucide-react';

import { resetPassword } from '@/lib/auth-client';
import { useLang } from '@/contexts';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Spinner } from '@/components/ui/spinner';
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form';

import { ErrorAlert } from './auth-alerts';

interface Props {
  loading: boolean;
  setLoading: (v: boolean) => void;
  error: string;
  setError: (v: string) => void;
  onGoLogin: () => void;
  /** Fires when the panel is (re-)mounted so the shell can clear the sent flag on mode change. */
  onMounted?: () => void;
}

export function ForgotPasswordPanel({
  loading,
  setLoading,
  error,
  setError,
  onGoLogin,
  onMounted,
}: Props) {
  const { lang } = useLang();
  const [resetSent, setResetSent] = useState(false);

  useEffect(() => {
    onMounted?.();
  }, [onMounted]);

  const schema = useMemo(() => {
    const required = lang === 'VN' ? 'Bắt buộc' : 'Required';
    const invalidEmail = lang === 'VN' ? 'Email không hợp lệ' : 'Invalid email';
    return z.object({
      email: z.string().min(1, required).email(invalidEmail),
    });
  }, [lang]);

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: { email: '' },
  });

  const onSubmit = async (values: z.infer<typeof schema>) => {
    setError('');
    setLoading(true);
    try {
      await resetPassword(values.email);
      setResetSent(true);
    } catch (err: unknown) {
      const raw = err instanceof Error ? err.message : String(err);
      const code = (err as { code?: string })?.code || '';
      if (code === 'auth/user-not-found' || code === 'auth/invalid-email') {
        setError(lang === 'VN' ? 'Email chưa được đăng ký.' : 'Email not found.');
      } else {
        setError(raw || 'An error occurred.');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <motion.div
      key="forgot"
      initial={{ opacity: 0, x: 20 }}
      animate={{ opacity: 1, x: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className="space-y-6"
    >
      <Button
        type="button"
        variant="ghost"
        size="sm"
        onClick={onGoLogin}
        className="hover:text-primary -ml-2 text-slate-400"
      >
        <ArrowLeft className="size-3.5" />
        {lang === 'VN' ? 'Đăng nhập' : 'Log in'}
      </Button>

      <h2 className="text-center text-2xl font-bold text-slate-800 sm:text-3xl">
        {lang === 'VN' ? 'Đặt lại mật khẩu' : 'Reset Password'}
      </h2>

      <p className="text-center text-xs text-slate-500 sm:text-sm">
        {lang === 'VN'
          ? 'Nhập email đã đăng ký, chúng tôi sẽ gửi link đặt lại mật khẩu cho bạn.'
          : 'Enter your registered email and we will send you a password reset link.'}
      </p>

      <ErrorAlert error={error} />

      {resetSent ? (
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          className="space-y-5"
        >
          <div className="flex flex-col items-center gap-3 py-6">
            <div className="flex h-14 w-14 items-center justify-center rounded-full bg-emerald-50 dark:bg-emerald-950/40">
              <Mail className="size-6 text-emerald-600 dark:text-emerald-400" />
            </div>
            <p className="text-center text-sm font-bold text-slate-700 dark:text-slate-200">
              {lang === 'VN' ? 'Email đã được gửi!' : 'Email sent!'}
            </p>
            <p className="max-w-xs text-center text-xs text-slate-500">
              {lang === 'VN'
                ? `Link đặt lại mật khẩu đã được gửi đến ${form.getValues('email')}. Vui lòng kiểm tra hộp thư.`
                : `A password reset link has been sent to ${form.getValues('email')}. Please check your inbox.`}
            </p>
          </div>
          <Button onClick={onGoLogin} className="h-10 w-full rounded-md text-sm font-semibold">
            {lang === 'VN' ? 'Quay lại đăng nhập' : 'Back to Login'}
          </Button>
        </motion.div>
      ) : (
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormControl>
                    <Input
                      type="email"
                      placeholder={lang === 'VN' ? 'Địa chỉ email' : 'Email address'}
                      className="h-10 rounded-md border-slate-200 bg-slate-50 dark:border-slate-700 dark:bg-slate-800"
                      {...field}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <Button
              type="submit"
              disabled={loading}
              className="h-10 w-full rounded-md text-sm font-semibold"
            >
              {loading ? (
                <Spinner className="size-5" />
              ) : lang === 'VN' ? (
                'Gửi link đặt lại'
              ) : (
                'Send Reset Link'
              )}
            </Button>
          </form>
        </Form>
      )}
    </motion.div>
  );
}
