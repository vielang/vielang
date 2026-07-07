'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import Image from 'next/image';
import { toast } from 'sonner';
import { Check, X, Loader2, Award, Languages, Clock, Video, UserCheck } from 'lucide-react';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { Button } from '@/components/ui/button';
import { EmptyState } from '@/components/shared/EmptyState';
import { initials, formatVnd } from '@/lib/format';
import type { PendingTutor } from '@/lib/supabase';
import { errorMessage } from '@/lib/errors';

export function PendingTutorsSection({ pending }: { pending: PendingTutor[] }) {
  const router = useRouter();
  const [busy, setBusy] = useState<string | null>(null);

  const act = async (id: string, action: 'approve' | 'reject') => {
    setBusy(id);
    try {
      const res = await fetch(`/api/admin/tutors/${id}/${action}`, {
        method: 'PUT',
        headers: await getAuthHeaders(),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error || 'action_failed');
      toast.success(action === 'approve' ? 'Tutor approved' : 'Tutor rejected');
      router.refresh();
    } catch (err) {
      toast.error('Could not update tutor', { description: errorMessage(err) });
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="space-y-4">
      <header>
        <h1 className="type-page text-slate-900 dark:text-slate-100">Pending tutor approvals</h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          Signed-up tutors waiting for a background check. Approved tutors show up under /tutors
          immediately.
        </p>
      </header>

      {pending.length === 0 ? (
        <div className="rounded-2xl border border-dashed border-slate-200 dark:border-slate-800">
          <EmptyState
            icon={<UserCheck className="size-6" />}
            title="All caught up"
            description="No tutors waiting for review right now. New applications will show up here as they come in."
          />
        </div>
      ) : (
        <div className="grid grid-cols-1 gap-4 lg:grid-cols-2">
          {pending.map((t) => (
            <article
              key={t.id}
              className="space-y-4 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900"
            >
              <div className="flex items-start gap-3">
                {t.avatar ? (
                  <Image
                    src={t.avatar}
                    alt={t.name}
                    width={56}
                    height={56}
                    className="size-14 rounded-full object-cover"
                  />
                ) : (
                  <div className="bg-brand flex size-14 items-center justify-center rounded-full text-sm font-semibold text-white">
                    {initials(t.name)}
                  </div>
                )}
                <div className="min-w-0 flex-1">
                  <h3 className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
                    {t.name}
                  </h3>
                  <p className="truncate text-xs text-slate-500 dark:text-slate-400">{t.email}</p>
                  <p className="mt-1 flex items-center gap-1 text-[11px] text-slate-500 dark:text-slate-400">
                    <Clock className="size-3" />
                    Applied {new Date(t.created_at).toLocaleDateString()}
                  </p>
                </div>
              </div>

              {t.bio && (
                <p className="line-clamp-3 text-xs leading-relaxed text-slate-600 dark:text-slate-300">
                  {t.bio}
                </p>
              )}

              <dl className="grid grid-cols-2 gap-2 text-[11px]">
                <div>
                  <dt className="font-semibold tracking-wide text-slate-500 uppercase dark:text-slate-400">
                    Rate
                  </dt>
                  <dd className="font-semibold text-slate-800 dark:text-slate-100">
                    {formatVnd(t.profile.hourly_rate_vnd)}₫/h
                  </dd>
                </div>
                <div>
                  <dt className="font-semibold tracking-wide text-slate-500 uppercase dark:text-slate-400">
                    Experience
                  </dt>
                  <dd className="font-semibold text-slate-800 dark:text-slate-100">
                    {t.profile.years_experience} yrs
                  </dd>
                </div>
                {t.profile.certifications.length > 0 && (
                  <div className="col-span-2">
                    <dt className="flex items-center gap-1 font-semibold tracking-wide text-slate-500 uppercase dark:text-slate-400">
                      <Award className="size-3" /> Certifications
                    </dt>
                    <dd className="font-medium text-slate-700 dark:text-slate-200">
                      {t.profile.certifications.join(' · ')}
                    </dd>
                  </div>
                )}
                {t.profile.languages_spoken.length > 0 && (
                  <div className="col-span-2">
                    <dt className="flex items-center gap-1 font-semibold tracking-wide text-slate-500 uppercase dark:text-slate-400">
                      <Languages className="size-3" /> Speaks
                    </dt>
                    <dd className="font-medium text-slate-700 uppercase dark:text-slate-200">
                      {t.profile.languages_spoken.join(' · ')}
                    </dd>
                  </div>
                )}
              </dl>

              {t.profile.intro_video_url && (
                <a
                  href={t.profile.intro_video_url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-brand dark:text-accent-warm inline-flex items-center gap-1 text-xs font-semibold hover:underline"
                >
                  <Video className="size-3" /> Intro video
                </a>
              )}

              {/* Approve / Reject bumped to h-11 so they meet iOS HIG on
                  mobile — admins often triage on the go. */}
              <div className="flex items-center gap-2 border-t border-slate-100 pt-3 dark:border-slate-800">
                <Button
                  className="flex-1"
                  onClick={() => act(t.id, 'approve')}
                  disabled={busy === t.id}
                >
                  {busy === t.id ? (
                    <Loader2 className="size-3.5 animate-spin" />
                  ) : (
                    <Check className="size-3.5" />
                  )}
                  Approve
                </Button>
                <Button
                  variant="outline"
                  className="flex-1 hover:border-red-300 hover:text-red-600 dark:hover:border-red-700 dark:hover:text-red-400"
                  onClick={() => act(t.id, 'reject')}
                  disabled={busy === t.id}
                >
                  <X className="size-3.5" />
                  Reject
                </Button>
              </div>
            </article>
          ))}
        </div>
      )}
    </div>
  );
}
