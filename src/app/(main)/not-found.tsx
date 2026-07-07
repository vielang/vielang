'use client';

import Link from 'next/link';
import { SearchX, Users, Home } from 'lucide-react';
import { Button } from '@/components/ui/button';

export default function NotFound() {
  return (
    <div className="flex min-h-[60vh] flex-col items-center justify-center px-4 py-16 text-center">
      <div className="bg-brand/5 text-brand mb-6 flex size-16 items-center justify-center rounded-2xl">
        <SearchX className="size-8" />
      </div>
      <p className="text-accent-warm mb-2 text-[10px] font-bold tracking-widest uppercase">404</p>
      <h1 className="text-brand dark:text-accent-warm mb-3 font-serif text-2xl font-bold md:text-3xl">
        Trang không tồn tại
      </h1>
      <p className="mb-8 max-w-md text-sm text-slate-500 dark:text-slate-400">
        The page you&apos;re looking for has been moved or deleted.
      </p>
      <div className="flex flex-col gap-3 sm:flex-row">
        <Link href="/">
          <Button className="h-10 gap-2 rounded-lg shadow-sm">
            <Home className="size-4" /> Trang chủ
          </Button>
        </Link>
        <Link href="/tutors">
          <Button variant="outline" className="h-10 gap-2 rounded-lg">
            <Users className="size-4" /> Xem giáo viên
          </Button>
        </Link>
      </div>
    </div>
  );
}
