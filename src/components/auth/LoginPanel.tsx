'use client';

import { useMemo } from 'react';
import { motion } from 'motion/react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { z } from 'zod';

import { loginWithEmail } from '@/lib/auth-client';
import { useLang } from '@/contexts';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Spinner } from '@/components/ui/spinner';
import { Form, FormControl, FormField, FormItem, FormMessage } from '@/components/ui/form';

import { ErrorAlert, DisabledBanner, OrDivider } from './auth-alerts';
import { SocialButtons } from './SocialButtons';
import { DemoLoginRow } from './DemoLoginRow';

interface Props {
  loading: boolean;
  setLoading: (v: boolean) => void;
  error: string;
  setError: (v: string) => void;
  disabledBanner: string | null;
  onDismissDisabled: () => void;
  onGoSignup: () => void;
  onGoForgot: () => void;
}

export function LoginPanel({
  loading,
  setLoading,
  error,
  setError,
  disabledBanner,
  onDismissDisabled,
  onGoSignup,
  onGoForgot,
}: Props) {
  const { lang, t } = useLang();

  const schema = useMemo(() => {
    const required = lang === 'VN' ? 'Bắt buộc' : 'Required';
    return z.object({
      email: z.string().min(1, required),
      password: z.string().min(1, required),
    });
  }, [lang]);

  const form = useForm<z.infer<typeof schema>>({
    resolver: zodResolver(schema),
    defaultValues: { email: '', password: '' },
  });

  const onSubmit = async (values: z.infer<typeof schema>) => {
    setError('');
    setLoading(true);
    try {
      await loginWithEmail(values.email, values.password);
    } catch (err: unknown) {
      const raw = err instanceof Error ? err.message : String(err);
      const msg = raw.toLowerCase();
      if (msg.includes('invalid login') || msg.includes('credentials')) {
        setError(lang === 'VN' ? 'Email hoặc mật khẩu không đúng.' : 'Invalid email or password.');
      } else {
        setError(raw || 'An error occurred.');
      }
    } finally {
      setLoading(false);
    }
  };

  return (
    <motion.div
      key="login"
      initial={{ opacity: 0, y: 6 }}
      animate={{ opacity: 1, y: 0 }}
      exit={{ opacity: 0, x: -20 }}
      className="space-y-6"
    >
      <h2 className="text-center text-2xl font-bold text-slate-800 sm:text-3xl">{t('login')}</h2>
      <DisabledBanner message={disabledBanner} onDismiss={onDismissDisabled} lang={lang} />
      <ErrorAlert error={error} />

      <Form {...form}>
        <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
          <FormField
            control={form.control}
            name="email"
            render={({ field }) => (
              <FormItem>
                <FormControl>
                  <Input
                    type="text"
                    autoFocus
                    autoComplete="username"
                    placeholder={lang === 'VN' ? 'Email / Số điện thoại' : 'Email / Phone number'}
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
                    placeholder="••••••••"
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
            {loading ? <Spinner className="size-5" /> : t('login')}
          </Button>
        </form>
      </Form>

      <OrDivider lang={lang} />
      <SocialButtons disabled={loading} onError={setError} onLoadingChange={setLoading} />
      <DemoLoginRow disabled={loading} lang={lang} />

      <div className="flex items-center justify-between pt-2 text-xs font-bold">
        <div className="text-slate-500">
          {t('noAccount')}{' '}
          <button type="button" onClick={onGoSignup} className="text-primary hover:underline">
            {t('registerNow')}
          </button>
        </div>
        <button
          type="button"
          onClick={onGoForgot}
          className="hover:text-primary text-slate-400 transition-colors hover:underline"
        >
          {t('forgotPassword')}
        </button>
      </div>
    </motion.div>
  );
}
