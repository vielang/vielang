import { BrandMark } from './BrandMark';

interface BrandedLoaderProps {
  variant?: 'full' | 'inline';
  label?: string;
}

export function BrandedLoader({ variant = 'full', label }: BrandedLoaderProps) {
  const containerClass =
    variant === 'full'
      ? 'min-h-[60vh] flex flex-col items-center justify-center gap-5'
      : 'py-10 flex flex-col items-center justify-center gap-4';

  return (
    <div className={containerClass} role="status" aria-live="polite">
      <div className="relative">
        <div className="absolute inset-0 animate-ping rounded-full bg-emerald-200/60 dark:bg-emerald-700/30" />
        <div className="relative flex size-16 items-center justify-center rounded-full bg-white shadow-md ring-1 ring-slate-100 dark:bg-slate-900 dark:ring-slate-700">
          <BrandMark className="text-brand dark:text-accent-warm size-11" />
        </div>
      </div>
      <div className="flex items-center gap-2">
        <span className="size-1.5 animate-bounce rounded-full bg-emerald-500 [animation-delay:-0.3s] dark:bg-emerald-400" />
        <span className="size-1.5 animate-bounce rounded-full bg-emerald-500 [animation-delay:-0.15s] dark:bg-emerald-400" />
        <span className="size-1.5 animate-bounce rounded-full bg-emerald-500 dark:bg-emerald-400" />
      </div>
      {label && (
        <p className="text-xs font-medium tracking-wide text-slate-500 uppercase dark:text-slate-400">
          {label}
        </p>
      )}
      <span className="sr-only">Loading</span>
    </div>
  );
}
