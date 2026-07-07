'use client';

import Link from 'next/link';
import { SearchX, Home } from 'lucide-react';
import { Button } from '@/components/ui/button';

export default function RootNotFound() {
  return (
    <div className="flex min-h-screen flex-col items-center justify-center bg-slate-50 px-4 text-center font-sans dark:bg-slate-950">
      <div className="mb-6 flex size-16 items-center justify-center rounded-2xl bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300">
        <SearchX className="size-8" />
      </div>
      <p className="mb-2 text-[10px] font-bold tracking-widest text-amber-500 uppercase dark:text-amber-400">
        404
      </p>
      <h1 className="dark:text-accent-warm mb-3 text-2xl font-bold text-emerald-900 md:text-3xl">
        Trang không tồn tại
      </h1>
      <p className="mb-8 max-w-md text-sm text-slate-500 dark:text-slate-400">
        The page you&apos;re looking for has been moved or deleted.
      </p>
      <Link href="/">
        <Button className="h-10 gap-2 rounded-lg">
          <Home className="size-4" /> Trang chủ
        </Button>
      </Link>
    </div>
  );
}
