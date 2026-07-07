'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { CalendarPlus } from 'lucide-react';
import { useAuth } from '@/contexts';
import { BookingDialog } from '@/components/session/BookingDialog';
import type { Course } from '@/lib/types';

interface Props {
  tutorId: string;
  tutorName: string;
  courses: Course[];
}

export function BookCta({ tutorId, tutorName, courses }: Props) {
  const router = useRouter();
  const { user } = useAuth();
  const [open, setOpen] = useState(false);

  const onClick = () => {
    if (!user) {
      router.push(`/login?redirect=/tutors/${tutorId}`);
      return;
    }
    setOpen(true);
  };

  return (
    <>
      <button
        type="button"
        onClick={onClick}
        className="bg-brand hover:bg-brand-hover inline-flex h-12 w-full items-center justify-center gap-2 rounded-xl text-sm font-semibold text-white shadow transition-colors"
      >
        <CalendarPlus className="size-4" />
        Book a session
      </button>
      <BookingDialog
        open={open}
        onClose={() => setOpen(false)}
        tutorId={tutorId}
        tutorName={tutorName}
        courses={courses}
      />
    </>
  );
}
