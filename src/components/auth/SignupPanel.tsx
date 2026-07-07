'use client';

import { useMemo } from 'react';
import { motion } from 'motion/react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';
import { ArrowLeft } from 'lucide-react';

import { registerWithEmail } from '@/lib/auth-client';
import { useLang } from '@/contexts';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Spinner } from '@/components/ui/spinner';
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form';

import { ErrorAlert, OrDivider } from './auth-alerts';
import { SocialButtons } from './SocialButtons';

interface Props {
  loading: boolean;
  setLoading: (v: boolean) => void;
  error: string;
  setError: (v: string) => void;
  onGoLogin: () => void;
}

export function SignupPanel({ loading, setLoading, error, setError, onGoLogin }: Props) {
  const { lang, t } = useLang();

  const schema = useMemo(() => {
    const required = lang === 'VN' ? 'Bắt buộc' : 'Required';
    const invalidEmail = lang === 'VN' ? 'Email không hợp lệ' : 'Invalid email';
    const min6 =
      lang === 'VN' ? 'Mật khẩu phải có ít nhất 6 ký tự' : 'Password must be at least 6 characters';
    return z
      .object({
        fullName: z.string().min(1, required),
        email: z.string().min(1, required).email(invalidEmail),
        password: z.string().min(6, min6),
        confirmPassword: z.string().min(1, required),
      })
      .refine((d) => d.password === d.confirmPassword, {
        message: t('passwordMismatch'),
        path: ['confirmPassword'],
      });
  }, [lang, t]);

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: { fullName: '', email: '', password: '', confirmPassword: '' },
  });

  const onSubmit = async (values: z.infer<typeof schema>) => {
    setError('');
    setLoading(true);
    try {
      await registerWithEmail(values.email, values.password, values.fullName);
    } catch (err: unknown) {
      const raw = err instanceof Error ? err.message : String(err);
      const msg = raw.toLowerCase();
      if (msg.includes('already registered') || msg.includes('already exists')) {
        setError(lang === 'VN' ? 'Email đã được sử dụng.' : 'Email is already in use.');
      } else if (msg.includes('password') && msg.includes('6')) {
        setError(
          lang === 'VN'
            ? 'Mật khẩu phải có ít nhất 6 ký tự.'
            : 'Password must be at least 6 characters.',
        );
      } else {
        setError(raw || 'An error occurred.');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <motion.div
      key="signup"
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
        {t('login')}
      </Button>

      <h2 className="text-center text-2xl font-bold text-slate-800 sm:text-3xl">{t('signUp')}</h2>
      <ErrorAlert error={error} />

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          <FormField
            control={form.control}
            name="fullName"
            render={({ field }) => (
              <FormItem>
                <FormControl>
                  <Input
                    placeholder={lang === 'VN' ? 'Họ và tên' : 'Full name'}
                    className="h-10 rounded-md border-slate-200 bg-slate-50 dark:border-slate-700 dark:bg-slate-800"
                    {...field}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="email"
            render={({ field }) => (
              <FormItem>
                <FormControl>
                  <Input
                    type="email"
                    placeholder="Email"
                    className="h-10 rounded-md border-slate-200 bg-slate-50 dark:border-slate-700 dark:bg-slate-800"
                    {...field}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="password"
            render={({ field }) => (
              <FormItem>
                <FormControl>
                  <Input
                    type="password"
                    placeholder={lang === 'VN' ? 'Mật khẩu' : 'Password'}
                    className="h-10 rounded-md border-slate-200 bg-slate-50 dark:border-slate-700 dark:bg-slate-800"
                    {...field}
                  />
                </FormControl>
                <FormMessage />
              </FormItem>
            )}
          />
          <FormField
            control={form.control}
            name="confirmPassword"
            render={({ field }) => (
              <FormItem>
                <FormControl>
                  <Input
                    type="password"
                    placeholder={lang === 'VN' ? 'Xác nhận mật khẩu' : 'Confirm password'}
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
            {loading ? <Spinner className="size-5" /> : t('signUpButton')}
          </Button>
        </form>
      </Form>

      <OrDivider lang={lang} />
      <SocialButtons disabled={loading} onError={setError} onLoadingChange={setLoading} />

      <div className="pt-1 text-center text-xs font-bold text-slate-500">
        {t('alreadyHaveAccount')}{' '}
        <button type="button" onClick={onGoLogin} className="text-primary hover:underline">
          {t('loginNow')}
        </button>
      </div>
    </motion.div>
  );
}
