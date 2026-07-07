'use client';

import { motion, AnimatePresence } from 'motion/react';
import { AlertCircle, X } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';

// Shared status blocks + separator for the auth flow. Kept together because
// each is small, they're all decorative, and the panels use them in
// combination.

export function ErrorAlert({ error }: { error: string }) {
  return (
    <AnimatePresence>
      {error && (
        <motion.div
          initial={{ opacity: 0, y: -4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -4 }}
        >
          <Alert variant="destructive" className="rounded-xl">
            <AlertCircle className="size-4" />
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

// Persistent banner shown when the user just got bounced for being disabled.
// Unlike ErrorAlert (which clears on every input change), this stays put
// until the user dismisses it — the reason text is the whole reason they're
// on this page.
export function DisabledBanner({
  message,
  onDismiss,
  lang,
}: {
  message: string | null;
  onDismiss: () => void;
  lang: 'VN' | 'EN';
}) {
  return (
    <AnimatePresence>
      {message && (
        <motion.div
          initial={{ opacity: 0, y: -4 }}
          animate={{ opacity: 1, y: 0 }}
          exit={{ opacity: 0, y: -4 }}
        >
          <Alert variant="destructive" className="relative rounded-xl pr-9">
            <AlertCircle className="size-4" />
            <AlertDescription className="whitespace-pre-line">
              <span className="mb-0.5 block font-semibold">
                {lang === 'VN' ? 'Tài khoản đã bị vô hiệu hóa' : 'Account disabled'}
              </span>
              {message}
            </AlertDescription>
            <button
              type="button"
              onClick={onDismiss}
              aria-label="Dismiss"
              className="text-destructive/70 hover:text-destructive hover:bg-destructive/10 absolute top-2 right-2 inline-flex size-6 items-center justify-center rounded-md"
            >
              <X className="size-3.5" />
            </button>
          </Alert>
        </motion.div>
      )}
    </AnimatePresence>
  );
}

export function OrDivider({ lang }: { lang: 'VN' | 'EN' }) {
  return (
    <div className="relative">
      <Separator />
      <span className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 bg-white px-4 text-sm font-medium text-slate-500 dark:bg-slate-900 dark:text-slate-400">
        {lang === 'VN' ? 'Hoặc' : 'Or'}
      </span>
    </div>
  );
}
