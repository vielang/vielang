import { BrandedLoader } from '@/components/shared/BrandedLoader';

export default function Loading() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-slate-50 dark:bg-slate-950">
      <BrandedLoader variant="full" />
    </div>
  );
}
