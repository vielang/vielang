'use client';

import { useEffect, useState } from 'react';
import {
  ExternalLink,
  FileText,
  Video,
  Image as ImageIcon,
  Link2,
  BookX,
  RefreshCw,
} from 'lucide-react';
import { Spinner } from '@/components/ui/spinner';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { getAuthHeaders } from '@/contexts/AuthContext';
import type { Material, MaterialType } from '@/lib/types';

const TYPE_ICON: Record<MaterialType, React.ComponentType<any>> = {
  pdf: FileText,
  video: Video,
  image: ImageIcon,
  link: Link2,
};

// Server-code → student-friendly copy. Anything we don't recognize falls back
// to a generic "something went wrong" with a retry button.
const ERROR_COPY: Record<string, { title: string; body: string; retryable: boolean }> = {
  no_access: {
    title: 'Materials are for booked students',
    body: "Book a session for this course first — your tutor's materials unlock as soon as the booking is confirmed.",
    retryable: false,
  },
  course_not_found: {
    title: 'Course not found',
    body: 'This course may have been unpublished. Try refreshing the tutor page.',
    retryable: false,
  },
  network: {
    title: 'Connection problem',
    body: "We couldn't reach the server. Check your connection and try again.",
    retryable: true,
  },
};

interface Props {
  open: boolean;
  onClose: () => void;
  courseId: string;
  courseTitle: string;
}

export function MaterialsDialog({ open, onClose, courseId, courseTitle }: Props) {
  const [items, setItems] = useState<Material[] | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    if (!open) return;
    let cancelled = false;
    setItems(null);
    setError(null);
    (async () => {
      try {
        const res = await fetch(`/api/courses/${courseId}/materials`, {
          headers: await getAuthHeaders(),
        });
        const data = await res.json().catch(() => ({}));
        if (cancelled) return;
        if (!res.ok) {
          setError(data?.error || `http_${res.status}`);
          return;
        }
        setItems(data.materials as Material[]);
      } catch {
        if (!cancelled) setError('network');
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [open, courseId, attempt]);

  const errorCopy = error ? ERROR_COPY[error] : null;

  return (
    <Dialog
      open={open}
      onOpenChange={(v) => {
        if (!v) onClose();
      }}
    >
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>Course materials</DialogTitle>
          <DialogDescription className="line-clamp-1">{courseTitle}</DialogDescription>
        </DialogHeader>

        {items === null && !error && (
          <div className="flex flex-col items-center justify-center gap-2 py-8 text-slate-500">
            <Spinner className="text-brand size-5" />
            <p className="text-xs">Loading materials…</p>
          </div>
        )}

        {error && (
          <div className="flex flex-col items-center gap-3 py-4 text-center">
            <div className="flex size-12 items-center justify-center rounded-full bg-slate-100 text-slate-500 dark:bg-slate-800">
              <BookX className="size-5" />
            </div>
            <div className="max-w-xs space-y-1">
              <p className="text-sm font-semibold text-slate-800 dark:text-slate-100">
                {errorCopy?.title || 'Something went wrong'}
              </p>
              <p className="text-xs leading-relaxed text-slate-500 dark:text-slate-400">
                {errorCopy?.body || `We hit an unexpected error (${error}). Please try again.`}
              </p>
            </div>
            {(errorCopy?.retryable ?? true) && (
              <button
                type="button"
                onClick={() => setAttempt((n) => n + 1)}
                className="focus-ring bg-brand hover:bg-brand-hover inline-flex h-9 items-center gap-1.5 rounded-lg px-4 text-xs font-semibold text-white transition-colors"
              >
                <RefreshCw className="size-3.5" />
                Try again
              </button>
            )}
          </div>
        )}

        {items && items.length === 0 && (
          <div className="flex flex-col items-center gap-2 py-6 text-center">
            <div className="flex size-12 items-center justify-center rounded-full bg-slate-100 text-slate-500 dark:bg-slate-800">
              <FileText className="size-5" />
            </div>
            <p className="text-sm font-semibold text-slate-800 dark:text-slate-100">
              No materials yet
            </p>
            <p className="max-w-xs text-xs text-slate-500 dark:text-slate-400">
              Your tutor hasn't attached any resources to this course yet. Feel free to nudge them
              in the session.
            </p>
          </div>
        )}

        {items && items.length > 0 && (
          <ul className="max-h-80 space-y-2 overflow-y-auto">
            {items.map((m) => {
              const Icon = TYPE_ICON[m.type];
              return (
                <li key={m.id}>
                  <a
                    href={m.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="focus-ring group hover:border-brand/40 flex items-center gap-3 rounded-xl border border-slate-200 p-3 transition-colors hover:bg-slate-50 dark:border-slate-700 dark:hover:bg-slate-800/40"
                  >
                    <div className="bg-brand/10 text-brand flex size-10 shrink-0 items-center justify-center rounded-lg">
                      <Icon className="size-4" />
                    </div>
                    <div className="min-w-0 flex-1">
                      <p className="truncate text-sm font-semibold text-slate-800 dark:text-slate-100">
                        {m.title}
                      </p>
                      <p className="text-[11px] tracking-wide text-slate-500 uppercase dark:text-slate-400">
                        {m.type}
                      </p>
                    </div>
                    <ExternalLink className="group-hover:text-brand size-3.5 text-slate-400" />
                  </a>
                </li>
              );
            })}
          </ul>
        )}
      </DialogContent>
    </Dialog>
  );
}
